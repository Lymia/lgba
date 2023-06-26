use anyhow::*;
use clap::{Parser, Subcommand};
use std::{path::PathBuf, result::Result::Ok};

mod build_rom;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Converts an ELF format binary to a proper GBA ROM
    BuildROM { elf_path: PathBuf, rom_path: PathBuf },
}

fn execute(cli: Cli) -> Result<()> {
    match cli.command {
        Commands::BuildROM { elf_path, rom_path } => build_rom::build_rom(&elf_path, &rom_path)?,
    }
    Ok(())
}
fn main() {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();
    match execute(cli) {
        Ok(_) => {}
        Err(e) => eprintln!("Error encountered: {:?}", e),
    }
}
