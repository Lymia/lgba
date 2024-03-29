use anyhow::*;
use clap::{Parser, Subcommand};
use lgba_romtool::{CompileConfig, RomData};
use std::{fs, path::PathBuf, result::Result::Ok};

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
struct Compile {
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
    #[arg(short = 'd', long)]
    data_file: Vec<PathBuf>,
}

#[derive(Subcommand)]
enum Commands {
    /// Compiles a GBA binary from a cargo package
    Compile(Compile),
    /// Converts a GBA binary to a proper GBA ROM
    BuildRom(BuildRom),
}

fn execute(cli: Cli) -> Result<()> {
    match cli.command {
        Commands::Compile(v) => {
            let config = CompileConfig::new(v.package, v.output);
            lgba_romtool::compile(&config)?;
        }
        Commands::BuildRom(v) => {
            let mut rom = RomData::from_elf(&fs::read(v.binary)?)?;
            for file in v.data_file {
                rom.add_data_source(file)?;
            }
            rom.print_statistics()?;
            fs::write(v.output, rom.produce_rom()?)?;
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
