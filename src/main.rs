use anyhow::{Context, Result};
use clap::{Args, Parser, Subcommand};
use organiza::{default_config_path, load_config, resolve_config, run, RunOptions};
use std::path::PathBuf;

// Re-export the t! macro from rust-i18n via our i18n module
rust_i18n::i18n!("locales");

#[derive(Debug, Parser)]
#[command(
    name = "organiza",
    version,
    about = "Organiza archivos por tipo en varias plataformas"
)]
struct Cli {
    /// Archivo TOML de configuración.
    #[arg(long, global = true, value_name = "FILE")]
    config: Option<PathBuf>,

    /// Idioma de la interfaz (en, es). Si no se especifica, detecta del sistema.
    #[arg(long, global = true, value_name = "LANG")]
    lang: Option<String>,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Clasifica los archivos de las carpetas configuradas o indicadas.
    Run(RunCommand),
    /// Comprueba que la configuración es válida sin mover archivos.
    ValidateConfig,
}

#[derive(Debug, Args)]
struct RunCommand {
    /// Muestra los movimientos sin modificar el sistema de archivos.
    #[arg(long)]
    dry_run: bool,

    /// Incluye archivos omitidos en la salida.
    #[arg(long)]
    verbose: bool,

    /// Sobrescribe el log configurado. Usa /dev/null o NUL para desactivarlo.
    #[arg(long, value_name = "FILE")]
    log: Option<PathBuf>,

    /// Carpetas a procesar. Si se indican, sustituyen las de la configuración.
    #[arg(value_name = "DIRECTORY")]
    directories: Vec<PathBuf>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    // Cloned once at startup so resolve_config can distinguish the
    // "user pointed --config at a missing path" branch from the
    // "user said nothing and the default is absent" branch.
    let config_path = cli.config.clone().unwrap_or_else(default_config_path);

    // Load config first to get language preference
    let config_lang = if config_path.exists() {
        load_config(&config_path).ok().and_then(|c| c.language)
    } else {
        None
    };

    // Initialize locale before any user-facing messages
    organiza::i18n::init_locale(cli.lang.as_deref(), config_lang.as_deref());

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
}
