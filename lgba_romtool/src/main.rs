#![feature(exit_status_error)]

use anyhow::*;
use clap::{Parser, Subcommand};
use std::{path::PathBuf, result::Result::Ok};

mod build_bin;
mod build_rom;

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
    package_name: String,
    output_path: PathBuf,
}

#[derive(Subcommand)]
enum Commands {
    /// Compiles a GBA binary from a cargo package
    BuildBin(BuildBin),
    /// Converts a GBA binary to a proper GBA ROM
    BuildRom { elf_path: PathBuf, rom_path: PathBuf },
}

fn execute(cli: Cli) -> Result<()> {
    match cli.command {
        Commands::BuildBin(v) => build_bin::build_bin(&v)?,
        Commands::BuildRom { elf_path, rom_path } => build_rom::build_rom(&elf_path, &rom_path)?,
    }
    Ok(())
}
fn main() {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();
    match execute(cli) {
        Ok(_) => {}
        Err(e) => eprintln!("Error encountered: {e:?}"),
    }
}
