use anyhow::{Context, Result};
use clap::{Args, Parser, Subcommand};
use organiza::{default_config_path, load_config, resolve_config, run, RunOptions};
use std::path::PathBuf;

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

    #[command(subcommand)]
    command: Command,
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

    match cli.command {
        Command::ValidateConfig => {
            let config = match (cli.config.is_some(), config_path.exists()) {
                (false, false) => organiza::Config::default(),
                _ => load_config(&config_path)
                    .with_context(|| format!("no se pudo validar {}", config_path.display()))?,
            };
            println!("Configuración válida: {}", config_path.display());
            println!("Carpetas configuradas: {}", config.source_directories.len());
        }
        Command::Run(command) => {
            let mut config = resolve_config(cli.config.as_deref(), &command.directories)?;
            if config.source_directories.is_empty() {
                anyhow::bail!("no hay carpetas configuradas ni indicadas en la línea de comandos");
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
