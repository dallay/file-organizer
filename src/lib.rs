use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::env;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};
use walkdir::{DirEntry, WalkDir};

mod categories;

#[cfg(test)]
pub(crate) use categories::category_for;
pub use categories::CategoryRule;
pub(crate) use categories::{
    apply_categories, classify, is_generated_category, is_unsafe_category_name,
    validate_categories, DEFAULT_CATEGORY,
};

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
    /// User-supplied category declarations. Order = TOML declaration order.
    pub categories: Vec<CategoryRule>,
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
            categories: Vec::new(),
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
    if let Some(path) = env::var_os("ORGANIZA_CONFIG") {
        return PathBuf::from(path);
    }

    if cfg!(windows) {
        env::var_os("APPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|| home_directory().join("AppData").join("Roaming"))
            .join("organiza")
            .join("config.toml")
    } else {
        env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| home_directory().join(".config"))
            .join("organiza")
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

/// Resolve the runtime `Config` from the CLI flags and the config file.
///
/// Precedence (locked in `intent.md`):
/// 1. CLI positional directories (always win).
/// 2. Configured `source_directories` (loaded from TOML).
/// 3. `ORGANIZA_DOWNLOADS` env var (via `default_downloads_path`).
/// 4. Platform auto-detect (via `default_downloads_path`).
///
/// First-run behavior: when the default config path does not exist AND no
/// `--config` flag is supplied, we synthesize `Config::default()` instead of
/// erroring. Explicit `--config /missing/path.toml` still surfaces the
/// existing load error.
pub fn resolve_config(cli_config: Option<&Path>, positional_dirs: &[PathBuf]) -> Result<Config> {
    let config_path = cli_config
        .map(Path::to_path_buf)
        .unwrap_or_else(default_config_path);

    let mut config = match (cli_config.is_some(), config_path.exists()) {
        (false, false) => Config::default(),
        _ => load_config(&config_path)
            .with_context(|| format!("no se pudo leer {}", config_path.display()))?,
    };

    if !positional_dirs.is_empty() {
        config.source_directories = positional_dirs.to_vec();
    } else if config.source_directories.is_empty() {
        if let Some(detected) = default_downloads_path(None) {
            config.source_directories.push(detected);
        }
    }

    Ok(config)
}

fn validate_config(config: &Config) -> Result<()> {
    validate_categories(&config.categories)?;
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
        if is_unsafe_category_name(category) {
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

    let canonical_log = config
        .log_file
        .as_deref()
        .filter(|path| !is_null_device(path))
        .and_then(|path| fs::canonicalize(path).ok());
    let mut skip_paths: HashSet<PathBuf> = HashSet::new();
    if let Some(log) = &canonical_log {
        skip_paths.insert(log.clone());
    }

    // Compose the extension→category map once per run: `move_file` classifies
    // every source path, and recomposing the map per file is pure waste.
    let composed = apply_categories(config);

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

        let dirs = collect_top_level_directories(&root, config, &skip_paths)?;

        // Move top-level directories first so their contents land under
        // `Other/<dirname>/` instead of being classified file-by-file.
        for (source, destination_root) in dirs {
            if let Err(error) = move_dir(&source, &destination_root, config, options, &mut logger) {
                logger.line(format!("ERROR: {}", error))?;
                failures += 1;
            }
        }

        let files = collect_files(&root, config, &skip_paths)?;
        for source in files {
            if is_recent(&source, config.min_age_seconds) {
                if options.verbose {
                    logger.line(format!("Reciente, omitido: {}", source.display()))?;
                }
                continue;
            }

            if let Err(error) = move_file(&source, &root, &composed, options, &mut logger) {
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

fn collect_files(
    root: &Path,
    config: &Config,
    skip_paths: &HashSet<PathBuf>,
) -> Result<Vec<PathBuf>> {
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
        if entry.file_type().is_file()
            && should_process_file(&entry, config)
            && !skip_paths.contains(&entry.path().to_path_buf())
        {
            files.push(entry.path().to_path_buf());
        }
    }
    Ok(files)
}

/// Collect depth-1 directories that should be moved into
/// `<root>/Other/<dirname>/`. Returns `(canonical_source, destination_root)`
/// pairs. The pre-classify drops symlinks, hidden (when `ignore_hidden`),
/// generated, empty, and skip-path matches.
fn collect_top_level_directories(
    root: &Path,
    config: &Config,
    skip_paths: &HashSet<PathBuf>,
) -> Result<Vec<(PathBuf, PathBuf)>> {
    let mut dirs = Vec::new();
    let walker = WalkDir::new(root).max_depth(1).min_depth(1).into_iter();

    for entry in walker {
        let entry = entry?;
        if entry.depth() == 0 {
            continue;
        }
        let path = entry.path();
        if entry.file_type().is_symlink() {
            continue;
        }
        if !entry.file_type().is_dir() {
            continue;
        }
        if config.ignore_hidden && is_hidden(path) {
            continue;
        }
        if is_generated_category(path, root, config) {
            continue;
        }
        let count = fs::read_dir(path)?.count();
        if count == 0 {
            continue;
        }
        let canonical_src = fs::canonicalize(path)
            .with_context(|| format!("no se pudo resolver {}", path.display()))?;
        if skip_paths.contains(&canonical_src) {
            continue;
        }
        let destination_root = root.join(DEFAULT_CATEGORY);
        dirs.push((canonical_src, destination_root));
    }
    Ok(dirs)
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
    composed: &HashMap<String, String>,
    options: RunOptions,
    logger: &mut Logger,
) -> Result<()> {
    let category = classify(source, composed);
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

/// Move a directory into `<destination_root>/<dirname>/`. Mirrors the file
/// conflict policy (`Overwrite+is_dir` skips) and dry-run parity (logs and
/// returns without creating the destination or renaming).
fn move_dir(
    source: &Path,
    destination_root: &Path,
    config: &Config,
    options: RunOptions,
    logger: &mut Logger,
) -> Result<()> {
    let file_name = source
        .file_name()
        .context("directorio sin nombre")?
        .to_owned();
    let requested_destination = destination_root.join(&file_name);

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
    logger.line(format!(
        "{}: {} → {}",
        action,
        source.display(),
        DEFAULT_CATEGORY
    ))?;
    if options.dry_run {
        return Ok(());
    }

    fs::create_dir_all(destination_root)?;
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
    let _ = config;
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
            .join("organiza.lock")
    } else {
        home_directory().join(".cache").join("organiza.lock")
    }
}

/// Resolve the Downloads directory used when neither CLI positional
/// directories nor `source_directories` are present. Order:
/// `ORGANIZA_DOWNLOADS` → Linux `XDG_DOWNLOAD_DIR` → macOS
/// `<home>/Downloads` → Windows `%USERPROFILE%/Downloads` with localized
/// fallbacks.
///
/// `home_override` lets tests inject a synthetic HOME without touching the
/// process environment. Production callers pass `None`.
pub fn default_downloads_path(home_override: Option<&Path>) -> Option<PathBuf> {
    if let Some(value) = env::var_os("ORGANIZA_DOWNLOADS") {
        let raw = PathBuf::from(value);
        let expanded = expand_home(&raw);
        if expanded.is_dir() {
            return Some(expanded);
        }
    }

    if cfg!(target_os = "linux") {
        let home = home_override
            .map(Path::to_path_buf)
            .or_else(home_directory_from_env);
        if let Some(home) = home {
            if let Some(path) = read_xdg_download_dir(&home) {
                if path.is_dir() {
                    return Some(path);
                }
            }
        }
    }

    let home = home_override
        .map(Path::to_path_buf)
        .or_else(home_directory_from_env);

    if cfg!(target_os = "macos") {
        if let Some(ref home) = home {
            let candidate = home.join("Downloads");
            if candidate.is_dir() {
                return Some(candidate);
            }
        }
    }

    if cfg!(target_os = "windows") {
        if let Some(ref home) = home {
            let primary = home.join("Downloads");
            if primary.is_dir() {
                return Some(primary);
            }
            for localized in ["Descargas", "Téléchargements", "Scaricati", "下载"] {
                let candidate = home.join(localized);
                if candidate.is_dir() {
                    return Some(candidate);
                }
            }
        }
    }

    None
}

/// Read `XDG_DOWNLOAD_DIR` from `<home>/.config/user-dirs.dirs`. Strips
/// surrounding double quotes and expands a literal `$HOME` token. Returns
/// `None` if the file is missing, the key is absent, or the value does not
/// point to an existing directory.
fn read_xdg_download_dir(home: &Path) -> Option<PathBuf> {
    let path = home.join(".config").join("user-dirs.dirs");
    let contents = fs::read_to_string(&path).ok()?;
    for line in contents.lines() {
        let line = line.trim();
        let Some(rest) = line.strip_prefix("XDG_DOWNLOAD_DIR=") else {
            continue;
        };
        let unquoted = rest.trim_matches('"');
        if unquoted.is_empty() {
            return None;
        }
        let resolved = if let Some(suffix) = unquoted.strip_prefix("$HOME/") {
            home.join(suffix)
        } else if unquoted == "$HOME" {
            home.to_path_buf()
        } else {
            PathBuf::from(unquoted)
        };
        return Some(resolved);
    }
    None
}

fn home_directory_from_env() -> Option<PathBuf> {
    if cfg!(windows) {
        env::var_os("USERPROFILE").map(PathBuf::from)
    } else {
        env::var_os("HOME").map(PathBuf::from)
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

    /// Tests that call `run` with a non-dry-run acquire the real directory
    /// lock at `~/.cache/organiza.lock`. Running them in parallel
    /// against the same lock would race. This mutex serializes them so the
    /// lock contention is deterministic instead of flaky.
    static RUN_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    /// Tests that mutate `ORGANIZA_DOWNLOADS` / `USERPROFILE` env vars
    /// race with each other under parallel execution. This mutex serializes
    /// them so the assertions see the env state they set.
    static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    fn run_serialized(config: &Config, options: RunOptions) -> Result<()> {
        let _guard = RUN_LOCK.lock().unwrap_or_else(|err| err.into_inner());
        run(config, options)
    }

    #[test]
    fn classifies_extensions_case_insensitively() {
        let config = Config::default();
        assert_eq!(category_for(Path::new("PHOTO.JPG"), &config), "Image");
        assert_eq!(category_for(Path::new("README.TXT"), &config), "Text");
        assert_eq!(category_for(Path::new("Resume.PDF"), &config), "Text");
        assert_eq!(category_for(Path::new("Song.MP3"), &config), "Audio");
        assert_eq!(category_for(Path::new("README"), &config), "Other");
        assert_eq!(category_for(Path::new("data.custom"), &config), "Other");
    }

    /// The crate must be published as `organiza` (RDP-1 rebrand); the binary
    /// name and every distribution artifact derive from this package name.
    ///
    /// We intentionally do not assert `CARGO_PKG_VERSION` here: release-please
    /// bumps the version on every release and a hardcoded literal would fail
    /// the next release PR. Version synchronization is validated separately
    /// by the release tooling (`scripts/update-versions.js` + manifest).
    #[test]
    fn package_is_named_organiza() {
        assert_eq!(env!("CARGO_PKG_NAME"), "organiza");
    }

    /// The default config path must live under the rebranded `organiza`
    /// directory, never the legacy `file-organizer` one (RDP-1).
    ///
    /// Compare path *components* instead of rendering to a string with a
    /// hardcoded `/` separator: on Windows the path uses `\`, so a string
    /// suffix check like `ends_with("organiza/config.toml")` fails even
    /// though the path is correct. `file_name()`/`parent()` are
    /// separator-agnostic and work on every platform.
    ///
    /// We also hold `ENV_LOCK` and clear `ORGANIZA_CONFIG` for the duration
    /// of the call: `default_config_path()` checks `ORGANIZA_CONFIG` first
    /// and returns it verbatim if present. The test
    /// `missing_default_config_uses_config_default_and_autodetect` sets that
    /// variable under the same lock; without our own guard a parallel test
    /// runner could observe that temporary value and the assertions below
    /// would race-fail.
    #[test]
    fn default_config_path_uses_organiza_directory() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|err| err.into_inner());
        // SAFETY: test-only env mutation guarded by ENV_LOCK; value restored
        // before the guard is released.
        let previous = env::var_os("ORGANIZA_CONFIG");
        unsafe {
            env::remove_var("ORGANIZA_CONFIG");
        }
        let path = default_config_path();
        match previous {
            Some(value) => unsafe {
                env::set_var("ORGANIZA_CONFIG", value);
            },
            None => unsafe {
                env::remove_var("ORGANIZA_CONFIG");
            },
        }
        assert_eq!(
            path.file_name().and_then(|name| name.to_str()),
            Some("config.toml"),
            "default config path must be config.toml, got: {}",
            path.display()
        );
        assert_eq!(
            path.parent()
                .and_then(|parent| parent.file_name())
                .and_then(|name| name.to_str()),
            Some("organiza"),
            "default config path must live under organiza/, got: {}",
            path.display()
        );
        assert!(
            !path.to_string_lossy().contains("file-organizer"),
            "default config path must not reference file-organizer, got: {}",
            path.display()
        );
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
        fs::create_dir(temporary.path().join("Image")).unwrap();
        fs::write(temporary.path().join("Image/already.JPG"), "image").unwrap();
        let config = test_config(temporary.path());

        run_serialized(
            &config,
            RunOptions {
                dry_run: false,
                verbose: false,
                conflict_policy: ConflictPolicy::Rename,
            },
        )
        .unwrap();

        assert!(temporary.path().join("Image/photo.JPG").exists());
        assert!(temporary.path().join("Image/already.JPG").exists());
    }

    #[test]
    fn category_for_unknown_extension_returns_other() {
        let config = Config::default();
        assert_eq!(category_for(Path::new("data.unknownext"), &config), "Other");
    }

    #[test]
    fn every_builtin_extension_maps_to_nonempty_category() {
        let map = crate::categories::default_categories();
        assert!(!map.is_empty(), "default_categories is empty");
        for (extension, category) in &map {
            assert!(
                !category.is_empty(),
                "extension '{}' maps to empty category",
                extension
            );
            assert_ne!(
                *category, "Other",
                "extension '{}' maps to fallback Other",
                extension
            );
        }
        // Regression risk per risks.md #1: xls and ppt must remain under Text.
        assert_eq!(map.get("xls"), Some(&"Text"));
        assert_eq!(map.get("ppt"), Some(&"Text"));
    }

    #[test]
    fn supplemental_category_rule_adds_extensions() {
        let mut config = Config::default();
        config.categories.push(CategoryRule {
            name: "Text".to_string(),
            extensions: vec!["foo".to_string(), "bar".to_string()],
            replace: false,
        });
        assert_eq!(category_for(Path::new("a.foo"), &config), "Text");
        assert_eq!(category_for(Path::new("a.bar"), &config), "Text");
        // Built-ins remain.
        assert_eq!(category_for(Path::new("a.txt"), &config), "Text");
        assert_eq!(category_for(Path::new("a.md"), &config), "Text");
    }

    #[test]
    fn replace_true_substitutes_builtin_list() {
        let mut config = Config::default();
        config.categories.push(CategoryRule {
            name: "Text".to_string(),
            extensions: vec!["onlytxt".to_string()],
            replace: true,
        });
        assert_eq!(category_for(Path::new("a.onlytxt"), &config), "Text");
        // Built-ins removed.
        assert_eq!(category_for(Path::new("a.txt"), &config), "Other");
        assert_eq!(category_for(Path::new("a.md"), &config), "Other");
        assert_eq!(category_for(Path::new("a.pdf"), &config), "Other");
    }

    #[test]
    fn non_colliding_category_adds_untouched() {
        let mut config = Config::default();
        config.categories.push(CategoryRule {
            name: "Design".to_string(),
            extensions: vec!["psd".to_string(), "ai".to_string()],
            replace: false,
        });
        assert_eq!(category_for(Path::new("a.psd"), &config), "Design");
        assert_eq!(category_for(Path::new("a.ai"), &config), "Design");
        // Built-ins untouched.
        assert_eq!(category_for(Path::new("a.jpg"), &config), "Image");
        assert_eq!(category_for(Path::new("a.txt"), &config), "Text");
        assert_eq!(category_for(Path::new("a.md"), &config), "Text");
    }

    #[test]
    fn extension_override_wins_after_rules() {
        let mut config = Config::default();
        config.categories.push(CategoryRule {
            name: "Text".to_string(),
            extensions: vec!["md".to_string()],
            replace: false,
        });
        config
            .extensions
            .insert("md".to_string(), "Docs".to_string());
        assert_eq!(category_for(Path::new("a.md"), &config), "Docs");
    }

    #[test]
    fn category_name_absolute_or_parent_traversal_rejected() {
        #[cfg(not(windows))]
        let windows_cases: &[&str] = &[];
        #[cfg(windows)]
        let windows_cases: &[&str] = &["C:\\Windows", "\\Windows", "..\\..\\etc"];
        let cases: Vec<&str> = ["/etc/passwd", "../escape", "Sub/../Other"]
            .iter()
            .chain(windows_cases.iter())
            .copied()
            .collect();
        for bad in cases {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("bad.toml");
            let payload = format!("[[categories]]\nname = \"{}\"\nextensions = [\"x\"]\n", bad);
            fs::write(&path, payload).unwrap();
            assert!(
                load_config(&path).is_err(),
                "expected load_config to reject category name '{}'",
                bad
            );
        }
    }

    #[test]
    fn category_with_empty_extensions_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("empty.toml");
        fs::write(
            &path,
            "[[categories]]\nname = \"EmptyCat\"\nextensions = []\n",
        )
        .unwrap();
        assert!(load_config(&path).is_err());
    }

    #[test]
    fn duplicate_category_name_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("dupe.toml");
        fs::write(
            &path,
            "[[categories]]\nname = \"Text\"\nextensions = [\"a\"]\n\
             [[categories]]\nname = \"Text\"\nextensions = [\"b\"]\n",
        )
        .unwrap();
        assert!(load_config(&path).is_err());
    }

    #[test]
    fn run_moves_top_level_directory_to_other() {
        let temporary = tempfile::tempdir().unwrap();
        fs::create_dir(temporary.path().join("Projects")).unwrap();
        fs::write(temporary.path().join("Projects/notes.txt"), "inside").unwrap();
        let mut config = test_config(temporary.path());
        config.recursive = true;

        run_serialized(
            &config,
            RunOptions {
                dry_run: false,
                verbose: false,
                conflict_policy: ConflictPolicy::Rename,
            },
        )
        .unwrap();

        assert!(temporary.path().join("Other/Projects/notes.txt").exists());
        assert!(!temporary.path().join("Projects").exists());
    }

    #[test]
    fn generated_top_level_directory_not_moved() {
        let temporary = tempfile::tempdir().unwrap();
        fs::create_dir(temporary.path().join("Audio")).unwrap();
        fs::write(temporary.path().join("Audio/track.mp3"), "audio").unwrap();
        let mut config = test_config(temporary.path());
        config.recursive = true;

        run_serialized(
            &config,
            RunOptions {
                dry_run: true,
                verbose: false,
                conflict_policy: ConflictPolicy::Rename,
            },
        )
        .unwrap();

        assert!(temporary.path().join("Audio/track.mp3").exists());
        assert!(!temporary.path().join("Other/Audio").exists());
    }

    #[test]
    fn generated_other_not_reentered() {
        let temporary = tempfile::tempdir().unwrap();
        fs::create_dir(temporary.path().join("Other")).unwrap();
        fs::write(temporary.path().join("Other/already.txt"), "stays").unwrap();
        fs::create_dir(temporary.path().join("Loose")).unwrap();
        fs::write(temporary.path().join("Loose/item.txt"), "loose").unwrap();
        let mut config = test_config(temporary.path());
        config.recursive = true;

        run_serialized(
            &config,
            RunOptions {
                dry_run: false,
                verbose: false,
                conflict_policy: ConflictPolicy::Rename,
            },
        )
        .unwrap();

        assert!(temporary.path().join("Other/already.txt").exists());
        assert!(temporary.path().join("Other/Loose/item.txt").exists());
        assert!(!temporary.path().join("Loose").exists());
    }

    #[test]
    fn empty_top_level_directory_skipped() {
        let temporary = tempfile::tempdir().unwrap();
        fs::create_dir(temporary.path().join("Empty")).unwrap();
        let config = test_config(temporary.path());

        run_serialized(
            &config,
            RunOptions {
                dry_run: true,
                verbose: false,
                conflict_policy: ConflictPolicy::Rename,
            },
        )
        .unwrap();

        assert!(temporary.path().join("Empty").exists());
        assert!(!temporary.path().join("Other/Empty").exists());
    }

    #[cfg(unix)]
    #[test]
    fn symlinked_top_level_directory_skipped() {
        use std::os::unix::fs::symlink;

        let temporary = tempfile::tempdir().unwrap();
        let target = tempfile::tempdir().unwrap();
        fs::write(target.path().join("payload.txt"), "x").unwrap();
        symlink(target.path(), temporary.path().join("Linked")).unwrap();
        let config = test_config(temporary.path());

        run_serialized(
            &config,
            RunOptions {
                dry_run: true,
                verbose: false,
                conflict_policy: ConflictPolicy::Rename,
            },
        )
        .unwrap();

        assert!(temporary.path().join("Linked").exists());
        assert!(!temporary.path().join("Other/Linked").exists());
    }

    #[test]
    fn hidden_top_level_directory_skipped_with_ignore_hidden() {
        let temporary = tempfile::tempdir().unwrap();
        fs::create_dir(temporary.path().join(".cache")).unwrap();
        fs::write(temporary.path().join(".cache/data.bin"), "x").unwrap();
        let config = test_config(temporary.path()); // ignore_hidden defaults to true

        run_serialized(
            &config,
            RunOptions {
                dry_run: true,
                verbose: false,
                conflict_policy: ConflictPolicy::Rename,
            },
        )
        .unwrap();

        assert!(temporary.path().join(".cache").exists());
        assert!(!temporary.path().join("Other/.cache").exists());
    }

    #[test]
    fn default_downloads_path_honors_env_var() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|err| err.into_inner());
        let temporary = tempfile::tempdir().unwrap();
        fs::create_dir(temporary.path().join("custom-downloads")).unwrap();
        let custom = temporary.path().join("custom-downloads");
        // SAFETY: test-only env mutation guarded by ENV_LOCK; values restored
        // after the assertion.
        unsafe {
            env::set_var("ORGANIZA_DOWNLOADS", &custom);
        }
        let result = default_downloads_path(Some(temporary.path()));
        unsafe {
            env::remove_var("ORGANIZA_DOWNLOADS");
        }
        assert_eq!(result.as_deref(), Some(custom.as_path()));
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn default_downloads_path_reads_xdg_user_dirs() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|err| err.into_inner());
        let temporary = tempfile::tempdir().unwrap();
        let xdg_dir = temporary.path().join(".config");
        fs::create_dir(&xdg_dir).unwrap();
        let quoted = "XDG_DOWNLOAD_DIR=\"$HOME/Downloads\"\nXDG_DESKTOP_DIR=\"$HOME/Desktop\"\n";
        fs::write(xdg_dir.join("user-dirs.dirs"), quoted).unwrap();
        let downloads = temporary.path().join("Downloads");
        fs::create_dir(&downloads).unwrap();
        unsafe {
            env::remove_var("ORGANIZA_DOWNLOADS");
        }
        let result = default_downloads_path(Some(temporary.path()));
        assert_eq!(result.as_deref(), Some(downloads.as_path()));
        let rendered = result.unwrap().to_string_lossy().into_owned();
        assert!(
            !rendered.contains("\"$HOME\""),
            "literal $HOME survives: {}",
            rendered
        );
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn default_downloads_path_returns_home_downloads() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|err| err.into_inner());
        let temporary = tempfile::tempdir().unwrap();
        let downloads = temporary.path().join("Downloads");
        fs::create_dir(&downloads).unwrap();
        unsafe {
            env::remove_var("ORGANIZA_DOWNLOADS");
        }
        let result = default_downloads_path(Some(temporary.path()));
        assert_eq!(result.as_deref(), Some(downloads.as_path()));
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn default_downloads_path_selects_userprofile_downloads_first() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|err| err.into_inner());
        let temporary = tempfile::tempdir().unwrap();
        let downloads = temporary.path().join("Downloads");
        fs::create_dir(&downloads).unwrap();
        let localized = temporary.path().join("Descargas");
        fs::create_dir(&localized).unwrap();
        unsafe {
            env::set_var("USERPROFILE", temporary.path());
            env::remove_var("ORGANIZA_DOWNLOADS");
        }
        let result = default_downloads_path(None);
        unsafe {
            env::remove_var("USERPROFILE");
        }
        assert_eq!(result.as_deref(), Some(downloads.as_path()));
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn default_downloads_path_uses_localized_fallback() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|err| err.into_inner());
        let temporary = tempfile::tempdir().unwrap();
        let localized = temporary.path().join("Descargas");
        fs::create_dir(&localized).unwrap();
        unsafe {
            env::set_var("USERPROFILE", temporary.path());
            env::remove_var("ORGANIZA_DOWNLOADS");
        }
        let result = default_downloads_path(None);
        unsafe {
            env::remove_var("USERPROFILE");
        }
        assert_eq!(result.as_deref(), Some(localized.as_path()));
    }

    #[test]
    fn missing_default_config_uses_config_default_and_autodetect() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|err| err.into_inner());
        let temp = tempfile::tempdir().unwrap();
        let downloads = temp.path().join("Downloads");
        fs::create_dir(&downloads).unwrap();
        // Point default_config_path at a nonexistent path via env.
        unsafe {
            env::set_var("ORGANIZA_CONFIG", temp.path().join("absent.toml"));
            env::set_var("ORGANIZA_DOWNLOADS", &downloads);
        }
        let config = resolve_config(None, &[]).unwrap();
        unsafe {
            env::remove_var("ORGANIZA_CONFIG");
            env::remove_var("ORGANIZA_DOWNLOADS");
        }
        assert_eq!(config.source_directories, vec![downloads]);
    }

    #[test]
    fn explicit_missing_config_path_still_errors() {
        let temp = tempfile::tempdir().unwrap();
        let missing = temp.path().join("missing.toml");
        let error = resolve_config(Some(&missing), &[]).unwrap_err();
        let rendered = format!("{:#}", error);
        assert!(
            rendered.contains("missing.toml"),
            "error message must name the missing path, got: {}",
            rendered
        );
    }

    #[test]
    fn configured_sources_win_over_env_and_detection() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|err| err.into_inner());
        let temp = tempfile::tempdir().unwrap();
        let config_path = temp.path().join("config.toml");
        fs::write(&config_path, "source_directories = [\"/srv/inbox\"]\n").unwrap();
        let env_downloads = temp.path().join("env-dl");
        fs::create_dir(&env_downloads).unwrap();
        unsafe {
            env::set_var("ORGANIZA_DOWNLOADS", &env_downloads);
        }
        let config = resolve_config(Some(&config_path), &[]).unwrap();
        unsafe {
            env::remove_var("ORGANIZA_DOWNLOADS");
        }
        assert_eq!(config.source_directories, vec![PathBuf::from("/srv/inbox")]);
    }

    #[test]
    fn positional_directories_override_config_and_detection() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|err| err.into_inner());
        let temp = tempfile::tempdir().unwrap();
        let config_path = temp.path().join("config.toml");
        fs::write(&config_path, "source_directories = [\"/srv/inbox\"]\n").unwrap();
        let env_downloads = temp.path().join("env-dl");
        fs::create_dir(&env_downloads).unwrap();
        unsafe {
            env::set_var("ORGANIZA_DOWNLOADS", &env_downloads);
        }
        let positional = vec![PathBuf::from("/cli/arg")];
        let config = resolve_config(Some(&config_path), &positional).unwrap();
        unsafe {
            env::remove_var("ORGANIZA_DOWNLOADS");
        }
        assert_eq!(config.source_directories, vec![PathBuf::from("/cli/arg")]);
    }

    #[test]
    fn log_file_in_downloads_is_excluded_from_classification() {
        let temp = tempfile::tempdir().unwrap();
        let log_file = temp.path().join("log.txt");
        fs::write(&log_file, "pre-existing log\n").unwrap();
        let config = Config {
            source_directories: vec![temp.path().to_path_buf()],
            log_file: Some(log_file.clone()),
            min_age_seconds: 0,
            ..Config::default()
        };

        run_serialized(
            &config,
            RunOptions {
                dry_run: false,
                verbose: false,
                conflict_policy: ConflictPolicy::Rename,
            },
        )
        .unwrap();

        // The log file must not be moved into Text/.
        assert!(log_file.exists(), "log file was moved or removed");
        assert!(
            !temp.path().join("Text/log.txt").exists(),
            "log file was classified into Text/"
        );
    }
}
