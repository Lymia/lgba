use anyhow::*;
use clap::{Parser, Subcommand};
use goblin::elf::section_header::SHF_ALLOC;
use std::{path::PathBuf, result::Result::Ok};

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
        Commands::BuildROM { elf_path, rom_path } => {
            println!("ELF path: {}", elf_path.display());
            println!("ROM path: {}", rom_path.display());
            println!();

            println!("Parsing ELF...");
            let data = std::fs::read(elf_path)?;
            let elf = goblin::elf::Elf::parse(&data)?;

            assert!(!elf.is_lib, "Error: Given ELF file is a dynamic library.");
            assert!(!elf.is_64, "Error: Given ELF file is 64-bit.");

            let mut rom_program = Vec::<u8>::new();
            enum State {
                WaitHeader,
                WaitText,
                WaitRoData,
                WaitEwram,
                WaitIwram,
                WaitBss,
                End,
            }

            let mut state = State::WaitHeader;
            for segment in &elf.section_headers {
                let name = elf.shdr_strtab.get_at(segment.sh_name).unwrap();
                println!("Found segment: {} = {:?}", name, segment);

                if segment.sh_flags as u32 & SHF_ALLOC == 0 {
                    continue;
                }

                let seg_start = segment.sh_offset as usize;
                let seg_end = (segment.sh_offset + segment.sh_size) as usize;
                let segment_data = &data[seg_start..seg_end];
                while rom_program.len() % segment.sh_addralign as usize != 0 {
                    rom_program.push(0);
                }
                match state {
                    State::WaitHeader => {
                        assert_eq!(name, ".header", "Wrong section found!");
                        rom_program.extend(segment_data);
                        state = State::WaitText;
                    }
                    State::WaitText => {
                        assert_eq!(name, ".text", "Wrong section found!");
                        rom_program.extend(segment_data);
                        state = State::WaitRoData;
                    }
                    State::WaitRoData => {
                        assert_eq!(name, ".rodata", "Wrong section found!");
                        rom_program.extend(segment_data);
                        state = State::WaitEwram;
                    }
                    State::WaitEwram => {
                        assert_eq!(name, ".ewram", "Wrong section found!");
                        rom_program.extend(segment_data);
                        state = State::WaitIwram;
                    }
                    State::WaitIwram => {
                        assert_eq!(name, ".iwram", "Wrong section found!");
                        rom_program.extend(segment_data);
                        state = State::WaitBss;
                    }
                    State::WaitBss => {
                        assert_eq!(name, ".bss", "Wrong section found!");
                        state = State::End;
                    }
                    State::End => {
                        panic!("Wrong section found!");
                    }
                }
            }
            println!();

            println!("Generating ROM file...");
            assert!(
                rom_program.len() <= 1024 * 1024 * 32,
                "GBA ROMs have a maximum size of 32 MiB"
            );

            let mut size = None;
            for s in [1, 2, 4, 8, 16, 32] {
                if rom_program.len() <= 1024 * 1024 * s {
                    size = Some(1024 * 1024 * s);
                    break;
                }
            }
            let size = size.unwrap();

            let mut vec = vec![0; size];
            vec[..rom_program.len()].copy_from_slice(&rom_program);
            std::fs::write(rom_path, vec)?;
        }
    }
    Ok(())
}
fn main() {
    let cli = Cli::parse();
    match execute(cli) {
        Ok(_) => {}
        Err(e) => println!("Error encountered: {:?}", e),
    }
}
