use anyhow::*;
use byteorder::{ReadBytesExt, LE};
use derive_setters::Setters;
use goblin::elf::section_header::SHF_ALLOC;
use std::{io::Cursor, path::PathBuf};
use tracing::{debug, error, info};

#[derive(Setters)]
#[setters(strip_option)]
pub struct BuildRomConfig {
    #[setters(skip)]
    binary: PathBuf,
    #[setters(skip)]
    output: PathBuf,
}
impl BuildRomConfig {
    pub fn new(binary: PathBuf, output: PathBuf) -> Self {
        BuildRomConfig { binary, output }
    }
}

pub fn build_rom(args: &BuildRomConfig) -> Result<()> {
    info!("Translating '{}' -> '{}'", args.binary.display(), args.output.display());

    // parse binary and translate into a GBA file
    info!("Parsing binary...");
    let data = std::fs::read(&args.binary)?;
    let elf = goblin::elf::Elf::parse(&data)?;

    assert!(!elf.is_lib, "Error: Given ELF file is a dynamic library.");
    assert!(!elf.is_64, "Error: Given ELF file is 64-bit.");

    let mut rom_program = Vec::<u8>::new();
    enum State {
        WaitHeader,
        WaitText,
        WaitRoData,
        WaitEwram,
        WaitEwramText,
        WaitIwram,
        WaitIwramText,
        WaitBss,
        End,
    }

    let mut state = State::WaitHeader;
    for segment in &elf.section_headers {
        let name = elf.shdr_strtab.get_at(segment.sh_name).unwrap();
        debug!("Found segment: {name} = {segment:?}");

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
                state = State::WaitEwramText;
            }
            State::WaitEwramText => {
                assert_eq!(name, ".ewram_text", "Wrong section found!");
                rom_program.extend(segment_data);
                state = State::WaitIwram;
            }
            State::WaitIwram => {
                assert_eq!(name, ".iwram", "Wrong section found!");
                rom_program.extend(segment_data);
                state = State::WaitIwramText;
            }
            State::WaitIwramText => {
                assert_eq!(name, ".iwram_text", "Wrong section found!");
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

    // build the final GBA file
    info!("Building rom...");
    assert!(rom_program.len() <= 1024 * 1024 * 32, "GBA ROMs have a maximum size of 32 MiB");

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

    // print statistics about the ROM generated
    let mut rom_cname = "unknown_crate";
    let mut rom_cver = "<unknown>";
    let mut rom_repository = "<unknown>";
    let mut lgba_version = "<unknown>";
    if &rom_program[0xE4..0xEC] == b"lgba_exh" {
        debug!("Found lgba extra header.");

        let mut read = Cursor::new(&rom_program[0xEC..]);
        let major = read.read_u16::<LE>()?;
        let minor = read.read_u16::<LE>()?;
        debug!("exh version       : {major}.{minor}");

        if major == 1 && minor == 0 {
            let rom_cname_off = read.read_u32::<LE>()?;
            let rom_cver_off = read.read_u32::<LE>()?;
            let rom_repository_off = read.read_u32::<LE>()?;
            let lgba_version_off = read.read_u32::<LE>()?;

            debug!("rom_cname_off     : 0x{rom_cname_off:x}");
            debug!("rom_cver_off      : 0x{rom_cver_off:x}");
            debug!("rom_repository_off: 0x{rom_repository_off:x}");
            debug!("lgba_version_off  : 0x{lgba_version_off:x}");

            macro_rules! read_str {
                ($ident:ident, $off:expr) => {{
                    let mut read = Cursor::new(&rom_program[$off as usize - 0x8000000..]);
                    // TODO: Don't depend on #[repr(Rust)], that is against the contract -w-
                    let str_off = read.read_u32::<LE>()? as usize - 0x8000000;
                    let str_len = read.read_u32::<LE>()? as usize;
                    let data = &rom_program[str_off..str_off + str_len];
                    $ident = std::str::from_utf8(data)?;
                }};
            }
            read_str!(rom_cname, rom_cname_off);
            read_str!(rom_cver, rom_cver_off);
            read_str!(rom_repository, rom_repository_off);
            read_str!(lgba_version, lgba_version_off);
        } else {
            error!("lgba extra header version {major}.{minor} not recognized.");
        }
    }

    info!("");
    info!("==================================================================");
    info!("Statistics");
    info!("==================================================================");
    info!("ROM File       : {}", args.binary.display());
    info!("ROM Version    : {rom_cname} {rom_cver}");
    info!("LGBA Version   : lgba {lgba_version}");
    info!("Bug Report URL : {rom_repository}");
    info!("Code Usage     : {:.1} KiB", rom_program.len() as f32 / 1024.0);
    info!("Rom Size       : {:} KiB", size / 1024);
    info!("==================================================================");
    info!("");

    // write the final ROM file to disk
    info!("Writing rom file...");
    std::fs::write(&args.output, vec)?;
    info!("Done!");

    Ok(())
}
