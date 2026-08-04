use anyhow::{Context, Result};
use clap::{Args, Parser, Subcommand};
use file_organizer::{default_config_path, load_config, run, RunOptions};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(
    name = "file-organizer",
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
    let config_path = cli.config.unwrap_or_else(default_config_path);

    match cli.command {
        Command::ValidateConfig => {
            let config = load_config(&config_path)
                .with_context(|| format!("no se pudo validar {}", config_path.display()))?;
            println!("Configuración válida: {}", config_path.display());
            println!("Carpetas configuradas: {}", config.source_directories.len());
        }
        Command::Run(command) => {
            let mut config = load_config(&config_path)
                .with_context(|| format!("no se pudo leer {}", config_path.display()))?;
            if !command.directories.is_empty() {
                config.source_directories = command.directories;
            }
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
