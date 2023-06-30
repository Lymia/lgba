use anyhow::*;
use clap::{Parser, Subcommand};
use lgba_romtool::{BuildBinConfig, BuildRomConfig};
use std::{path::PathBuf, result::Result::Ok};

#[cfg(not(feature = "binary"))]
compile_error!("`binary` feature must be enabled to compile binary. -w-");

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct BuildBin {
    #[arg(long)]
    linker_script: Option<PathBuf>,
    #[arg(short = 'p', long)]
    package: String,
    #[arg(short = 'o', long)]
    output: PathBuf,
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct BuildRom {
    #[arg(short = 'b', long)]
    binary: PathBuf,
    #[arg(short = 'o', long)]
    output: PathBuf,
}

#[derive(Subcommand)]
enum Commands {
    /// Compiles a GBA binary from a cargo package
    BuildBin(BuildBin),
    /// Converts a GBA binary to a proper GBA ROM
    BuildRom(BuildRom),
}

fn execute(cli: Cli) -> Result<()> {
    match cli.command {
        Commands::BuildBin(v) => {
            let mut config = BuildBinConfig::new(v.package, v.output);
            if let Some(linker_script) = v.linker_script {
                config = config.linker_script(linker_script);
            }
            lgba_romtool::build_bin(&config)?;
        }
        Commands::BuildRom(v) => {
            let config = BuildRomConfig::new(v.binary, v.output);
            lgba_romtool::build_rom(&config)?;
        }
    }
    Ok(())
}

fn main() {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }
    env_logger::init();

    let cli = Cli::parse();
    match execute(cli) {
        Ok(_) => {}
        Err(e) => eprintln!("Error encountered: {e:?}"),
    }
}
