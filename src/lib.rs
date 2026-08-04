use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};
use walkdir::{DirEntry, WalkDir};

const DEFAULT_CATEGORY: &str = "Otros";
const NO_EXTENSION_CATEGORY: &str = "Sin extensión";

#[derive(Debug, Clone, Copy, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConflictPolicy {
    Skip,
    #[default]
    Rename,
    Overwrite,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct Config {
    pub source_directories: Vec<PathBuf>,
    pub recursive: bool,
    pub on_conflict: ConflictPolicy,
    pub min_age_seconds: u64,
    pub ignore_hidden: bool,
    pub log_file: Option<PathBuf>,
    /// Extension-to-category overrides. Keys are case-insensitive.
    pub extensions: HashMap<String, String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            source_directories: Vec::new(),
            recursive: false,
            on_conflict: ConflictPolicy::Rename,
            min_age_seconds: 60,
            ignore_hidden: true,
            log_file: None,
            extensions: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RunOptions {
    pub dry_run: bool,
    pub verbose: bool,
    pub conflict_policy: ConflictPolicy,
}

pub fn default_config_path() -> PathBuf {
    if let Some(path) = env::var_os("FILE_ORGANIZER_CONFIG") {
        return PathBuf::from(path);
    }

    if cfg!(windows) {
        env::var_os("APPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|| home_directory().join("AppData").join("Roaming"))
            .join("file-organizer")
            .join("config.toml")
    } else {
        env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| home_directory().join(".config"))
            .join("file-organizer")
            .join("config.toml")
    }
}

pub fn load_config(path: &Path) -> Result<Config> {
    let source = fs::read_to_string(path)
        .with_context(|| format!("no se pudo leer la configuración {}", path.display()))?;
    let mut config: Config =
        toml::from_str(&source).with_context(|| format!("TOML inválido en {}", path.display()))?;
    config.source_directories = config
        .source_directories
        .into_iter()
        .map(|path| expand_home(&path))
        .collect();
    config.log_file = config.log_file.map(|path| expand_home(&path));
    validate_config(&config)?;
    Ok(config)
}

fn validate_config(config: &Config) -> Result<()> {
    if config
        .source_directories
        .iter()
        .any(|path| path.as_os_str().is_empty())
    {
        anyhow::bail!("source_directories contiene una ruta vacía");
    }
    for (extension, category) in &config.extensions {
        if extension.trim().is_empty() || category.trim().is_empty() {
            anyhow::bail!("extensions no puede contener claves o categorías vacías");
        }
        if Path::new(category).is_absolute() || category.split('/').any(|part| part == "..") {
            anyhow::bail!("la categoría '{}' contiene una ruta no permitida", category);
        }
    }
    Ok(())
}

pub fn run(config: &Config, options: RunOptions) -> Result<()> {
    let lock = if options.dry_run {
        None
    } else {
        Some(Lock::acquire(&default_lock_path())?)
    };
    let mut logger = Logger::new(config.log_file.as_deref())?;
    let mut failures = 0;

    for configured_root in &config.source_directories {
        let root = expand_home(configured_root);
        if !root.is_dir() {
            logger.line(format!("La carpeta no existe: {}", root.display()))?;
            failures += 1;
            continue;
        }

        let root = fs::canonicalize(&root)
            .with_context(|| format!("no se pudo resolver {}", root.display()))?;
        logger.line(format!("Procesando: {}", root.display()))?;
        let files = collect_files(&root, config)?;

        for source in files {
            if is_recent(&source, config.min_age_seconds) {
                if options.verbose {
                    logger.line(format!("Reciente, omitido: {}", source.display()))?;
                }
                continue;
            }

            if let Err(error) = move_file(&source, &root, config, options, &mut logger) {
                logger.line(format!("ERROR: {}", error))?;
                failures += 1;
            }
        }
    }

    drop(lock);
    if failures > 0 {
        anyhow::bail!(
            "{} carpeta(s) o archivo(s) no se pudieron procesar",
            failures
        );
    }
    Ok(())
}

fn collect_files(root: &Path, config: &Config) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    let walker = if config.recursive {
        WalkDir::new(root).min_depth(1)
    } else {
        WalkDir::new(root).max_depth(1).min_depth(1)
    };

    for entry in walker
        .into_iter()
        .filter_entry(|entry| should_visit(entry, root, config))
    {
        let entry = entry?;
        if entry.file_type().is_file() && should_process_file(&entry, config) {
            files.push(entry.path().to_path_buf());
        }
    }
    Ok(files)
}

fn should_visit(entry: &DirEntry, root: &Path, config: &Config) -> bool {
    if entry.depth() == 0 {
        return true;
    }
    if config.ignore_hidden && is_hidden(entry.path()) {
        return false;
    }
    if entry.file_type().is_dir() && is_generated_category(entry.path(), root, config) {
        return false;
    }
    true
}

fn should_process_file(entry: &DirEntry, config: &Config) -> bool {
    !config.ignore_hidden || !is_hidden(entry.path())
}

fn is_generated_category(path: &Path, root: &Path, config: &Config) -> bool {
    let categories = default_categories()
        .into_values()
        .chain(config.extensions.values().cloned())
        .chain([
            DEFAULT_CATEGORY.to_string(),
            NO_EXTENSION_CATEGORY.to_string(),
        ]);
    categories
        .map(|category| root.join(category))
        .any(|category_path| path == category_path || path.starts_with(category_path))
}

fn is_hidden(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.starts_with('.'))
}

fn is_recent(path: &Path, min_age_seconds: u64) -> bool {
    if min_age_seconds == 0 {
        return false;
    }
    let Ok(modified) = fs::metadata(path).and_then(|metadata| metadata.modified()) else {
        return true;
    };
    SystemTime::now()
        .duration_since(modified)
        .unwrap_or(Duration::ZERO)
        < Duration::from_secs(min_age_seconds)
}

fn move_file(
    source: &Path,
    root: &Path,
    config: &Config,
    options: RunOptions,
    logger: &mut Logger,
) -> Result<()> {
    let category = category_for(source, config);
    let destination_directory = root.join(&category);
    let requested_destination =
        destination_directory.join(source.file_name().context("archivo sin nombre")?);
    let destination = match (requested_destination.exists(), options.conflict_policy) {
        (false, _) => requested_destination,
        (true, ConflictPolicy::Skip) => {
            logger.line(format!("Omitido por conflicto: {}", source.display()))?;
            return Ok(());
        }
        (true, ConflictPolicy::Rename) => unique_destination(&requested_destination),
        (true, ConflictPolicy::Overwrite) if requested_destination.is_dir() => {
            logger.line(format!(
                "Omitido: destino es una carpeta: {}",
                requested_destination.display()
            ))?;
            return Ok(());
        }
        (true, ConflictPolicy::Overwrite) => requested_destination,
    };

    let action = if options.dry_run {
        "Se movería"
    } else {
        "Movido"
    };
    logger.line(format!("{}: {} → {}", action, source.display(), category))?;
    if options.dry_run {
        return Ok(());
    }

    fs::create_dir_all(&destination_directory)?;
    if matches!(options.conflict_policy, ConflictPolicy::Overwrite) && destination.exists() {
        fs::remove_file(&destination)?;
    }
    fs::rename(source, &destination).with_context(|| {
        format!(
            "no se pudo mover {} a {}",
            source.display(),
            destination.display()
        )
    })?;
    Ok(())
}

fn unique_destination(destination: &Path) -> PathBuf {
    let directory = destination.parent().unwrap_or_else(|| Path::new("."));
    let filename = destination
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("archivo");
    let (base, extension) = match filename.rsplit_once('.') {
        Some((base, extension)) if !base.is_empty() => (base, Some(extension)),
        _ => (filename, None),
    };
    for counter in 1.. {
        let candidate_name = match extension {
            Some(extension) => format!("{} ({}).{}", base, counter, extension),
            None => format!("{} ({})", base, counter),
        };
        let candidate = directory.join(candidate_name);
        if !candidate.exists() {
            return candidate;
        }
    }
    unreachable!()
}

fn category_for(path: &Path, config: &Config) -> String {
    let Some(filename) = path.file_name().and_then(|name| name.to_str()) else {
        return NO_EXTENSION_CATEGORY.to_string();
    };
    let Some(extension) = filename.rsplit_once('.').map(|(_, extension)| extension) else {
        return NO_EXTENSION_CATEGORY.to_string();
    };
    if extension.is_empty() {
        return NO_EXTENSION_CATEGORY.to_string();
    }
    config
        .extensions
        .iter()
        .find(|(key, _)| key.eq_ignore_ascii_case(extension))
        .map(|(_, category)| category.clone())
        .or_else(|| {
            default_categories()
                .get(&extension.to_ascii_lowercase())
                .cloned()
        })
        .unwrap_or_else(|| DEFAULT_CATEGORY.to_string())
}

fn default_categories() -> HashMap<String, String> {
    let mut categories = HashMap::new();
    add(
        &mut categories,
        "Imágenes",
        &[
            "jpg", "jpeg", "png", "gif", "webp", "heic", "svg", "tiff", "tif",
        ],
    );
    add(&mut categories, "Documentos/PDF", &["pdf"]);
    add(&mut categories, "Documentos/Word", &["doc", "docx"]);
    add(&mut categories, "Documentos/Pages", &["pages"]);
    add(&mut categories, "Documentos/Texto", &["txt", "md", "rtf"]);
    add(&mut categories, "Documentos/Libros", &["epub"]);
    add(
        &mut categories,
        "Documentos/Hojas de cálculo",
        &["xls", "xlsx", "numbers", "csv"],
    );
    add(
        &mut categories,
        "Documentos/Presentaciones",
        &["ppt", "pptx", "key"],
    );
    add(
        &mut categories,
        "Vídeos",
        &["mp4", "mov", "mkv", "avi", "webm", "m4v"],
    );
    add(
        &mut categories,
        "Audio",
        &["mp3", "m4a", "wav", "flac", "ogg", "aac"],
    );
    add(
        &mut categories,
        "Comprimidos",
        &["zip", "rar", "7z", "tar", "gz", "bz2", "xz"],
    );
    add(
        &mut categories,
        "Instaladores",
        &["dmg", "pkg", "msi", "exe", "deb", "rpm"],
    );
    add(
        &mut categories,
        "Código",
        &[
            "java", "kt", "kts", "js", "ts", "jsx", "tsx", "py", "rb", "go", "rs", "sh", "zsh",
            "json", "yaml", "yml", "xml", "html", "css", "sql",
        ],
    );
    categories
}

fn add(categories: &mut HashMap<String, String>, category: &str, extensions: &[&str]) {
    for extension in extensions {
        categories.insert((*extension).to_string(), category.to_string());
    }
}

fn expand_home(path: &Path) -> PathBuf {
    let value = path.to_string_lossy();
    if value == "~" {
        return home_directory();
    }
    if let Some(rest) = value.strip_prefix("~/") {
        return home_directory().join(rest);
    }
    path.to_path_buf()
}

fn home_directory() -> PathBuf {
    env::var_os("HOME")
        .or_else(|| env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

fn default_lock_path() -> PathBuf {
    if cfg!(windows) {
        env::var_os("LOCALAPPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|| home_directory().join("AppData").join("Local"))
            .join("file-organizer.lock")
    } else {
        home_directory().join(".cache").join("file-organizer.lock")
    }
}

struct Lock {
    path: PathBuf,
}

impl Lock {
    fn acquire(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        match fs::create_dir(path) {
            Ok(()) => Ok(Self {
                path: path.to_path_buf(),
            }),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
                anyhow::bail!("ya hay otra ejecución en curso: {}", path.display())
            }
            Err(error) => Err(error.into()),
        }
    }
}

impl Drop for Lock {
    fn drop(&mut self) {
        let _ = fs::remove_dir(&self.path);
    }
}

struct Logger {
    file: Option<File>,
}

impl Logger {
    fn new(path: Option<&Path>) -> Result<Self> {
        let Some(path) = path.filter(|path| !is_null_device(path)) else {
            return Ok(Self { file: None });
        };
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let file = OpenOptions::new().create(true).append(true).open(path)?;
        Ok(Self { file: Some(file) })
    }

    fn line(&mut self, message: String) -> Result<()> {
        let timestamp = chrono_like_timestamp();
        let line = format!("{} {}", timestamp, message);
        println!("{}", line);
        if let Some(file) = &mut self.file {
            writeln!(file, "{}", line)?;
            file.flush()?;
        }
        Ok(())
    }
}

fn is_null_device(path: &Path) -> bool {
    matches!(path.to_str(), Some("/dev/null" | "NUL" | "nul"))
}

fn chrono_like_timestamp() -> String {
    // Keep the binary dependency-light; log timestamps remain ISO-like via the OS date command only
    // in the shell wrapper. Rust logs use seconds since UNIX epoch for portability.
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|duration| duration.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn test_config(root: &Path) -> Config {
        Config {
            source_directories: vec![root.to_path_buf()],
            min_age_seconds: 0,
            log_file: None,
            ..Config::default()
        }
    }

    #[test]
    fn classifies_extensions_case_insensitively() {
        let config = Config::default();
        assert_eq!(category_for(Path::new("PHOTO.JPG"), &config), "Imágenes");
        assert_eq!(category_for(Path::new("README"), &config), "Sin extensión");
        assert_eq!(category_for(Path::new("data.custom"), &config), "Otros");
    }

    #[test]
    fn custom_extension_overrides_defaults() {
        let mut config = Config::default();
        config.extensions.insert("pdf".into(), "Revisar".into());
        assert_eq!(category_for(Path::new("document.pdf"), &config), "Revisar");
    }

    #[test]
    fn unique_destination_preserves_extension() {
        let temporary = tempfile::tempdir().unwrap();
        let destination = temporary.path().join("file.pdf");
        fs::write(&destination, "existing").unwrap();
        assert_eq!(
            unique_destination(&destination),
            temporary.path().join("file (1).pdf")
        );
    }

    #[test]
    fn run_moves_files_and_leaves_generated_categories_out_of_scan() {
        let temporary = tempfile::tempdir().unwrap();
        fs::write(temporary.path().join("photo.JPG"), "image").unwrap();
        fs::create_dir(temporary.path().join("Imágenes")).unwrap();
        fs::write(temporary.path().join("Imágenes/already.JPG"), "image").unwrap();
        let config = test_config(temporary.path());

        run(
            &config,
            RunOptions {
                dry_run: false,
                verbose: false,
                conflict_policy: ConflictPolicy::Rename,
            },
        )
        .unwrap();

        assert!(temporary.path().join("Imágenes/photo.JPG").exists());
        assert!(temporary.path().join("Imágenes/already.JPG").exists());
    }
}
