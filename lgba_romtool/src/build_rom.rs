use anyhow::{bail, Result};
use goblin::elf::section_header::SHF_ALLOC;
use log::{debug, info, warn};
use std::{collections::HashMap, fmt, fmt::Formatter, ops::Range};

#[derive(Clone, Debug)]
struct ExhInfo {
    version: u16,
    range: Range<usize>,
}

#[derive(Clone)]
pub struct RomData {
    data: Vec<u8>,
    exh: HashMap<[u8; 4], Vec<ExhInfo>>,
    base_addr: Option<usize>,
}
impl RomData {
    /// Produces a ROM from ELF data.
    pub fn from_elf(elf_data: &[u8]) -> Result<Self> {
        // parse binary and translate into a GBA file
        info!("Parsing binary...");
        let elf = goblin::elf::Elf::parse(elf_data)?;

        assert!(!elf.is_lib, "Error: Given ELF file is a dynamic library.");
        assert!(!elf.is_64, "Error: Given ELF file is 64-bit.");

        let mut rom_program = Vec::<u8>::new();
        const SECTION_ORDER: &[&str] = &[
            ".header",
            ".text",
            ".rodata",
            ".ewram",
            ".ewram_text",
            ".iwram",
            ".iwram_text",
            ".bss",
        ];

        let mut section_no = 0;
        for segment in &elf.section_headers {
            let name = elf.shdr_strtab.get_at(segment.sh_name).unwrap();
            debug!("Found segment: {name} = {segment:?}");

            if segment.sh_flags as u32 & SHF_ALLOC == 0 {
                continue;
            }

            let seg_start = segment.sh_offset as usize;
            let seg_end = (segment.sh_offset + segment.sh_size) as usize;
            let segment_data = &elf_data[seg_start..seg_end];
            while rom_program.len() % segment.sh_addralign as usize != 0 {
                rom_program.push(0);
            }
            assert_eq!(name, SECTION_ORDER[section_no], "Wrong section found!");
            rom_program.extend(segment_data);
            section_no += 1;
        }
        assert_eq!(
            section_no,
            SECTION_ORDER.len(),
            "Expected section not found: {}",
            SECTION_ORDER[section_no]
        );

        // build the final GBA file
        info!("Building rom...");
        assert!(rom_program.len() <= 1024 * 1024 * 32, "GBA ROMs have a maximum size of 32 MiB");
        Self::from_bin(&rom_program)
    }

    /// Produces a ROM from binary data.
    pub fn from_bin(bin_data: &[u8]) -> Result<Self> {
        // check that this isn't an ELF binary
        if bin_data.len() >= 4 && &bin_data[0..4] == b"\x7fELF" {
            bail!("ELF binaries cannot be loaded with from_bin")
        }

        // read exheaders
        let mut exh = HashMap::new();

        'find_exh: {
            // finds the offset of the first exheader
            let mut offset = 0;
            'find_first_exh: {
                while offset < 1024 && offset + 4 <= bin_data.len() {
                    if &bin_data[offset..offset + 4] == b"lGex" {
                        break 'find_first_exh;
                    }
                    offset += 4;
                }

                // bail out early
                warn!("Could not find lGex header.");
                break 'find_exh;
            }

            // finds each present exheader
            while offset + 12 <= bin_data.len() {
                // proper exh end tag
                if &bin_data[offset..offset + 4] == b"exh_" {
                    break 'find_exh;
                }

                // check that we have a correct exheader tag
                if &bin_data[offset..offset + 4] != b"lGex" {
                    warn!("Incorrect exheader magic number found! Ignoring exheaders.");
                    exh.clear();
                    break 'find_exh;
                }

                // parse exheader contents
                let mut tag = [0; 4];
                tag.copy_from_slice(&bin_data[offset + 4..offset + 8]);
                let mut version = [0; 2];
                version.copy_from_slice(&bin_data[offset + 8..offset + 10]);
                let mut length = [0; 2];
                length.copy_from_slice(&bin_data[offset + 10..offset + 12]);
                let version = u16::from_le_bytes(version);
                let length = u16::from_le_bytes(length);

                // check header for correctness
                let name = String::from_utf8_lossy(&tag);
                if exh.contains_key(&tag) {
                    warn!("Found duplicate exheader: '{}'", name);
                    continue;
                }
                let end_offset = offset + 12 + length as usize;
                if end_offset > bin_data.len() {
                    warn!("Exheader length exceeds ROM length. Ignoring exheaders.");
                    exh.clear();
                    break 'find_exh;
                }

                // register header
                debug!("Found exheader: name = '{}', len = {}", name, length);
                let new_exh = ExhInfo { version, range: offset + 12..end_offset };
                exh.entry(tag).or_insert(Vec::new()).push(new_exh);
                offset = end_offset;
            }
        };

        // try to calculate the base address
        let base_addr = if let Some(x) = exh.get(b"meta").and_then(|x| x.first()) {
            let range = x.range.clone();

            let mut base = [0; 4];
            base.copy_from_slice(&bin_data[range.start..range.start + 4]);
            let base = u32::from_le_bytes(base) as usize;

            Some(base - range.start)
        } else {
            warn!("meta exheader not found!");
            None
        };

        Ok(RomData { data: Vec::from(bin_data), exh, base_addr })
    }

    /// Returns the base address of this ROM.
    pub fn base_addr(&self) -> Result<usize> {
        match self.base_addr {
            Some(x) => Ok(x),
            None => bail!("exheaders could not be loaded."),
        }
    }

    /// Returns the data underlying this ROM.
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Returns a mutable reference to the data underlying this ROM.
    pub fn data_mut(&mut self) -> &mut [u8] {
        &mut self.data
    }

    /// Returns an extra header.
    pub fn get_exh(&self, header: &[u8; 4]) -> Result<ExHeader> {
        match self.exh.get(header) {
            Some(v) => {
                if v.len() == 1 {
                    Ok(ExHeader { exh: v[0].clone(), base_addr: self.base_addr()? })
                } else {
                    bail!("Multiple headers found. Use `iter_exh` instead.")
                }
            }
            None => bail!("No such header found."),
        }
    }

    /// Iterates the exheaders for a given item.
    pub fn iter_exh(&self, header: &[u8; 4]) -> Result<impl Iterator<Item = ExHeader> + '_> {
        let base_addr = self.base_addr()?;
        match self.exh.get(header) {
            Some(v) => Ok(v
                .iter()
                .map(move |x| ExHeader { exh: x.clone(), base_addr })),
            None => bail!("No such header found."),
        }
    }

    /// Returns the data for an extra header.
    pub fn get_exh_data(&self, header: &[u8; 4]) -> Result<&[u8]> {
        let exh = self.get_exh(header)?;
        Ok(&self.data[exh.file_range()])
    }

    /// Returns a mutable view of the data for an extra header.
    pub fn get_exh_data_mut(&mut self, header: &[u8; 4]) -> Result<&mut [u8]> {
        let exh = self.get_exh(header)?;
        Ok(&mut self.data[exh.file_range()])
    }

    /// Reads a `u32` from a given offset
    pub fn read_u32(&self, offset: usize) -> Result<u32> {
        let base = self.base_addr()?;
        let offset = match offset.checked_sub(base) {
            Some(x) => x,
            None => bail!("invalid offset"),
        };
        if offset + 4 > self.data.len() {
            bail!("invalid offset");
        }

        let mut data = [0; 4];
        data.copy_from_slice(&self.data[offset..offset + 4]);
        Ok(u32::from_le_bytes(data))
    }

    /// Reads a `usize` from a given offset.
    pub fn read_usize(&self, offset: usize) -> Result<usize> {
        Ok(self.read_u32(offset)? as usize)
    }

    fn read_str(&self, offset: usize) -> Result<&str> {
        // TODO: Don't depend on #[repr(Rust)], that is against the contract -w-
        let base = self.base_addr()?;
        let start = self.read_usize(offset)? - base;
        let end = start + self.read_usize(offset + 4)?;
        Ok(std::str::from_utf8(&self.data[start..end])?)
    }

    /// Prints statistics about the ROM using the `log` crate.
    pub fn print_statistics(&self) -> Result<()> {
        let mut rom_cname = "unknown_crate";
        let mut rom_cver = "<unknown>";
        let mut rom_repository = "<unknown>";
        let mut lgba_version = "<unknown>";
        if let Ok(exh) = self.get_exh(b"meta") {
            let meta_offset = exh.start_addr();
            if let Ok(str) = self.read_str(self.read_usize(meta_offset + 4)?) {
                rom_cname = str;
            }
            if let Ok(str) = self.read_str(self.read_usize(meta_offset + 8)?) {
                rom_cver = str;
            }
            if let Ok(str) = self.read_str(self.read_usize(meta_offset + 12)?) {
                rom_repository = str;
            }
            if let Ok(str) = self.read_str(self.read_usize(meta_offset + 16)?) {
                lgba_version = str;
            }
        }

        info!("");
        info!("==================================================================");
        info!("Statistics");
        info!("==================================================================");
        info!("ROM Version    : {rom_cname} {rom_cver}");
        info!("LGBA Version   : lgba {lgba_version}");
        info!("Bug Report URL : {rom_repository}");
        info!("Code Usage     : {:.1} KiB", self.data.len() as f32 / 1024.0);
        info!("==================================================================");
        info!("");

        Ok(())
    }

    /// Produces a binary file based on this data.
    ///
    /// The output format is meant to be used with [`from_bin`](`RomData::from_bin`).
    pub fn produce_bin(&self) -> Result<Vec<u8>> {
        Ok(self.data.clone())
    }

    /// Produces a ROM based on this data.
    pub fn produce_rom(&self) -> Result<Vec<u8>> {
        info!("Padding rom...");

        let mut size = None;
        for s in [1, 2, 4, 8, 16, 32] {
            if self.data.len() <= 1024 * 1024 * s {
                size = Some(1024 * 1024 * s);
                break;
            }
        }
        let size = size.unwrap();

        let mut vec = vec![0; size];
        vec[..self.data.len()].copy_from_slice(&self.data);
        Ok(vec)
    }
}
impl fmt::Debug for RomData {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        struct Length(usize);
        impl fmt::Debug for Length {
            fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
                write!(f, "[{} bytes]", self.0)
            }
        }
        f.debug_struct("RomData")
            .field("data", &Length(self.data.len()))
            .field("exh", &self.exh)
            .finish()
    }
}

#[derive(Clone, Debug)]
pub struct ExHeader {
    exh: ExhInfo,
    base_addr: usize,
}
impl ExHeader {
    pub fn version(&self) -> u16 {
        self.exh.version
    }

    pub fn start_addr(&self) -> usize {
        self.exh.range.start + self.base_addr
    }

    pub fn len(&self) -> usize {
        self.exh.range.len()
    }

    pub fn file_range(&self) -> Range<usize> {
        self.exh.range.clone()
    }

    pub fn mem_range(&self) -> Range<usize> {
        let base_addr = self.start_addr();
        base_addr..self.len()
    }
}
