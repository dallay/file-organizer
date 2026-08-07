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

    #[arg(long, global = true, value_name = "LANG", help = t!("lang_arg"))]
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

/// Scan the raw args for `--lang`/`--config` so the locale can be
/// initialized *before* Clap builds the help/error output. Without this
/// pre-pass, `--help` would always render in the default locale.
fn pre_scan_args() -> (Option<String>, Option<PathBuf>) {
    let mut lang = None;
    let mut config = None;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--lang" => lang = args.next(),
            "--config" => config = args.next().map(PathBuf::from),
            _ if arg.starts_with("--lang=") => {
                lang = Some(arg.trim_start_matches("--lang=").to_string())
            }
            _ if arg.starts_with("--config=") => {
                config = Some(PathBuf::from(arg.trim_start_matches("--config=")))
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
        load_config(&config_path).ok().and_then(|c| c.language)
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

    /// The CLI must expose itself as `organiza` so `--help`/`--version` and
    /// completion metadata match the rebranded crate (RDP-1).
    #[test]
    fn cli_name_is_organiza() {
        assert_eq!(Cli::command().get_name(), "organiza");
    }

    /// Help texts must follow the active locale, not a hardcoded language.
    #[test]
    fn cli_help_follows_locale() {
        rust_i18n::set_locale("es");
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
}
