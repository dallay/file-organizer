use anyhow::{Context, Result};
use clap::{Args, Parser, Subcommand};
use organiza::{default_config_path, load_config, resolve_config, run, RunOptions};
use std::path::PathBuf;

// Re-export the t! macro from rust-i18n via our i18n module
rust_i18n::i18n!("locales");
use rust_i18n::t;

#[derive(Debug, Parser)]
#[command(
    name = "organiza",
    version,
    about = t!("about")
)]
struct Cli {
    #[arg(long, global = true, value_name = "FILE", help = t!("config_arg"))]
    config: Option<PathBuf>,

    #[arg(
        long,
        global = true,
        value_name = "LANG",
        help = t!("lang_arg"),
        value_parser = parse_lang
    )]
    lang: Option<String>,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    #[command(about = t!("run_files"))]
    Run(RunCommand),
    #[command(about = t!("validate_config"))]
    ValidateConfig,
}

#[derive(Debug, Args)]
struct RunCommand {
    #[arg(long, help = t!("dry_run_arg"))]
    dry_run: bool,

    #[arg(long, help = t!("verbose_arg"))]
    verbose: bool,

    #[arg(long, value_name = "FILE", help = t!("log_arg"))]
    log: Option<PathBuf>,

    #[arg(value_name = "DIRECTORY", help = t!("directories_arg"))]
    directories: Vec<PathBuf>,
}

/// Parse and validate the `--lang` CLI value against the en|es contract.
///
/// Clap calls this only after the locale is already initialized, so the
/// rejection message is localized to the active interface language.
fn parse_lang(value: &str) -> Result<String, clap::Error> {
    if organiza::i18n::is_supported_language(value) {
        Ok(value.to_string())
    } else {
        Err(clap::Error::raw(
            clap::error::ErrorKind::ValueValidation,
            rust_i18n::t!("invalid_language", lang = value).to_string(),
        ))
    }
}

/// Scan the raw args for `--lang`/`--config` so the locale can be
/// initialized *before* Clap builds the help/error output. Without this
/// pre-pass, `--help` would always render in the default locale.
///
/// Uses `args_os()` so non-UTF-8 arguments (e.g. a path with arbitrary
/// bytes) never panic. Parsing stops at a standalone `--`: everything after
/// it is positional and must not be interpreted as an option.
fn pre_scan_args() -> (Option<String>, Option<PathBuf>) {
    pre_scan_args_from(std::env::args_os().skip(1))
}

/// Pure parsing core of [`pre_scan_args`], testable with an arbitrary arg
/// stream.
fn pre_scan_args_from<I>(args: I) -> (Option<String>, Option<PathBuf>)
where
    I: IntoIterator<Item = std::ffi::OsString>,
{
    use std::ffi::OsStr;

    let mut lang = None;
    let mut config = None;
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        // `--` marks the end of options; positional args follow. Do not
        // interpret them (a directory named "--config" is still a directory).
        if arg == OsStr::new("--") {
            break;
        }
        let text = arg.to_string_lossy();
        match text.as_ref() {
            "--lang" => {
                lang = args
                    .next()
                    .map(|value| value.to_string_lossy().into_owned())
            }
            "--config" => config = args.next().map(PathBuf::from),
            _ if text.starts_with("--lang=") => {
                lang = Some(text.trim_start_matches("--lang=").to_string())
            }
            _ if text.starts_with("--config=") => {
                config = Some(PathBuf::from(
                    text.trim_start_matches("--config=").to_string(),
                ))
            }
            _ => {}
        }
    }
    (lang, config)
}

fn main() -> Result<()> {
    // Initialize the locale BEFORE Clap builds help/error output so the
    // interface language applies to `--help`, subcommand docs, and errors.
    let (cli_lang, cli_config) = pre_scan_args();
    let config_path = cli_config.clone().unwrap_or_else(default_config_path);
    let config_lang = if config_path.exists() {
        organiza::i18n::preflight_config_language(&config_path)
    } else {
        None
    };
    organiza::i18n::init_locale(cli_lang.as_deref(), config_lang.as_deref());

    let cli = Cli::parse();
    // Cloned once at startup so resolve_config can distinguish the
    // "user pointed --config at a missing path" branch from the
    // "user said nothing and the default is absent" branch.
    let config_path = cli.config.clone().unwrap_or_else(default_config_path);

    match cli.command {
        None => {
            // Sin subcomando: ejecutar organización con configuración por defecto
            let config = resolve_config(cli.config.as_deref(), &[])?;
            if config.source_directories.is_empty() {
                anyhow::bail!("{}", rust_i18n::t!("no_folders_error"));
            }
            run(
                &config,
                RunOptions {
                    dry_run: false,
                    verbose: false,
                    conflict_policy: config.on_conflict,
                },
            )?;
        }
        Some(Command::ValidateConfig) => {
            let config = match (cli.config.is_some(), config_path.exists()) {
                (false, false) => organiza::Config::default(),
                _ => load_config(&config_path).with_context(|| {
                    rust_i18n::t!("validate_failed", path = config_path.display().to_string())
                })?,
            };
            println!(
                "{}",
                rust_i18n::t!("config_valid", path = config_path.display().to_string())
            );
            println!(
                "{}",
                rust_i18n::t!(
                    "folders_configured",
                    count = config.source_directories.len()
                )
            );
        }
        Some(Command::Run(command)) => {
            let mut config = resolve_config(cli.config.as_deref(), &command.directories)?;
            if config.source_directories.is_empty() {
                anyhow::bail!("{}", rust_i18n::t!("no_folders_error"));
            }
            if let Some(log) = command.log {
                config.log_file = Some(log);
            }
            run(
                &config,
                RunOptions {
                    dry_run: command.dry_run,
                    verbose: command.verbose,
                    conflict_policy: config.on_conflict,
                },
            )?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    /// Serializes tests that mutate the global locale. The binary tests run
    /// in a separate process from the library tests, so this only needs to
    /// coordinate tests within this module.
    static TEST_LOCALE_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    /// The CLI must expose itself as `organiza` so `--help`/`--version` and
    /// completion metadata match the rebranded crate (RDP-1).
    #[test]
    fn cli_name_is_organiza() {
        assert_eq!(Cli::command().get_name(), "organiza");
    }

    /// Help texts must follow the active locale, not a hardcoded language.
    #[test]
    fn cli_help_follows_locale() {
        // One guard for the whole test: the lock is not reentrant, so we
        // switch locales directly once inside the guarded region.
        let _guard = LocaleGuard::set("es");
        let es_about = Cli::command()
            .get_about()
            .map(|text| text.to_string())
            .unwrap_or_default();
        assert!(
            es_about.contains("Organiza archivos"),
            "expected Spanish about text, got: {}",
            es_about
        );

        rust_i18n::set_locale("en");
        let en_about = Cli::command()
            .get_about()
            .map(|text| text.to_string())
            .unwrap_or_default();
        assert!(
            en_about.contains("Organize files"),
            "expected English about text, got: {}",
            en_about
        );
    }

    #[test]
    fn pre_scan_args_stops_at_double_dash() {
        // A directory literally named "--config" after `--` must not be
        // consumed as the config flag.
        let (lang, config) = pre_scan_args_from([
            std::ffi::OsString::from("--lang"),
            std::ffi::OsString::from("es"),
            std::ffi::OsString::from("--"),
            std::ffi::OsString::from("--config"),
            std::ffi::OsString::from("/tmp/should-not-be-parsed.toml"),
        ]);
        assert_eq!(lang.as_deref(), Some("es"));
        assert_eq!(config, None);
    }

    #[cfg(unix)]
    #[test]
    fn pre_scan_args_handles_non_utf8_values() {
        use std::os::unix::ffi::OsStringExt;
        // A non-UTF-8 argument before `--` must not panic; it is simply not
        // matched as a flag.
        let (lang, config) = pre_scan_args_from([
            std::ffi::OsString::from("--lang"),
            std::ffi::OsString::from_vec(vec![0xFF, 0xFE]),
            std::ffi::OsString::from("--config=./config.toml"),
        ]);
        assert_eq!(lang, Some("\u{FFFD}\u{FFFD}".to_string()));
        assert_eq!(
            config,
            Some(PathBuf::from("./config.toml")),
            "--config=... should still be parsed after a non-UTF-8 value"
        );
    }

    /// RAII guard for the binary tests: switches locale and restores it on
    /// drop, including during assertion unwinding. Holds [`TEST_LOCALE_LOCK`]
    /// for the whole lifetime so parallel tests never observe a half-set
    /// locale.
    struct LocaleGuard {
        _lock: std::sync::MutexGuard<'static, ()>,
        previous: String,
    }

    impl LocaleGuard {
        fn set(locale: &str) -> Self {
            let _lock = TEST_LOCALE_LOCK
                .lock()
                .unwrap_or_else(|err| err.into_inner());
            let previous = rust_i18n::locale().to_string();
            rust_i18n::set_locale(locale);
            Self { _lock, previous }
        }
    }

    impl Drop for LocaleGuard {
        fn drop(&mut self) {
            rust_i18n::set_locale(&self.previous);
        }
    }

    #[test]
    fn parse_lang_rejects_unsupported_languages() {
        let _guard = LocaleGuard::set("en");
        assert!(parse_lang("en").is_ok());
        assert!(parse_lang("es").is_ok());
        let error = parse_lang("fr").unwrap_err();
        assert!(
            error.to_string().contains("Unsupported language 'fr'"),
            "expected English invalid_language, got: {}",
            error
        );
    }
}
