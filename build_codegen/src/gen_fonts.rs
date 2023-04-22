use crate::download_fonts::{data_is_half_width, CharacterInfo, CharacterSets};
use anyhow::*;
use std::{
    collections::{HashMap, HashSet},
    fs::File,
    hash::Hash,
    io::Write,
};
use unic_ucd::{block::Block, common::is_control, normal::is_combining_mark, BidiClass};

#[derive(Copy, Clone)]
pub struct FontConfiguration {
    pub font_name: &'static str,
    pub font_type: &'static str,
    pub description: &'static str,
    pub low_plane_limit: usize,
    pub low_plane_dupe_limit: usize,
    pub disable_unscii: bool,
    pub unscii_blocks: &'static [&'static str],
    pub disable_misaki: bool,
    pub misaki_blocks: &'static [&'static str],
    pub allow_all_blocks: bool,
    pub whitelisted_blocks: &'static [&'static str],
    pub whitelisted_chars: &'static [char],
    pub allow_halfwidth_blocks: &'static [&'static str],
    pub fallback_char: char,
    pub kanji_max_level: kanji::Level,
    pub character_count: usize,
    pub delta: f32,
}

const DEBUG_FONT_UNSCII: FontConfiguration = FontConfiguration {
    font_name: "*",
    font_type: "unscii",
    description: "",
    low_plane_limit: 0,
    low_plane_dupe_limit: 0,
    disable_unscii: false,
    unscii_blocks: &[],
    disable_misaki: true,
    misaki_blocks: &[],
    allow_all_blocks: true,
    whitelisted_blocks: &[],
    whitelisted_chars: &[],
    allow_halfwidth_blocks: &[],
    fallback_char: '?',
    kanji_max_level: kanji::Level::One,
    character_count: 0,
    delta: 0.0,
};
const DEBUG_FONT_MISAKI: FontConfiguration = FontConfiguration {
    font_name: "*",
    font_type: "misaki",
    description: "",
    low_plane_limit: 0,
    low_plane_dupe_limit: 0,
    disable_unscii: true,
    unscii_blocks: &[],
    disable_misaki: false,
    misaki_blocks: &[],
    allow_all_blocks: true,
    whitelisted_blocks: &[],
    whitelisted_chars: &[],
    allow_halfwidth_blocks: &[],
    fallback_char: '?',
    kanji_max_level: kanji::Level::One,
    character_count: 0,
    delta: 0.0,
};

fn block_name(ch: char) -> &'static str {
    match Block::of(ch) {
        None => "Unknown Block",
        Some(block) => block.name,
    }
}
fn list_to_set<V: Copy + Hash + Eq>(list: &[V]) -> HashSet<V> {
    let mut set = HashSet::new();
    for s in list {
        set.insert(*s);
    }
    set
}
fn process_char(config: &FontConfiguration, mut char: CharacterInfo) -> CharacterInfo {
    if !config.allow_halfwidth_blocks.contains(&block_name(char.ch)) {
        char.is_half_width = false;
    }
    char
}

fn parse_fonts(config: &FontConfiguration, characters: &CharacterSets) -> Vec<CharacterInfo> {
    let mut char_map = HashMap::new();

    // parse unscii-8
    let unscii_blocks = list_to_set(config.unscii_blocks);
    for char in &characters.unscii {
        let is_override = unscii_blocks.contains(block_name(char.ch));
        if char.ch != '\0' && (!config.disable_unscii || is_override) {
            char_map.insert(char.ch, process_char(config, *char));
        }
    }

    // add characters from misaki font
    let misaki_blocks = list_to_set(config.misaki_blocks);
    for char in &characters.misaki {
        let is_override = misaki_blocks.contains(block_name(char.ch));
        if is_override || (!config.disable_misaki && !char_map.contains_key(&char.ch)) {
            char_map.insert(char.ch, process_char(config, *char));
        }
    }

    // return the downloaded characters
    let mut characters: Vec<_> = char_map.values().cloned().collect();
    characters.sort_by_key(|x| x.ch as u32);
    characters
}

fn gather_characters(
    config: &FontConfiguration,
    characters: Vec<CharacterInfo>,
) -> Vec<CharacterInfo> {
    // calculate list of (root) characters to include in the font
    let mut map_characters = HashSet::new();
    let mut blocks = HashMap::new();
    let mut glyphs = HashSet::new();

    let whitelisted_blocks = list_to_set(config.whitelisted_blocks);
    let whitelisted_chars = list_to_set(config.whitelisted_chars);
    let kanji_level = kanji::level_table();

    glyphs.insert(0);
    for char in &characters {
        if whitelisted_chars.contains(&char.ch)
            || ((config.allow_all_blocks || whitelisted_blocks.contains(block_name(char.ch)))
                && !is_control(char.ch)
                && !is_combining_mark(char.ch)
                && !BidiClass::of(char.ch).is_rtl()
                && (char.ch as u32) < 0x10000)
        {
            let is_kanji_too_advanced = if let Some(kanji) = kanji::Kanji::new(char.ch) {
                if let Some(level) = kanji_level.get(&kanji) {
                    *level > config.kanji_max_level
                } else {
                    config.kanji_max_level != kanji::Level::One
                }
            } else {
                false
            };

            if !is_kanji_too_advanced {
                map_characters.insert(*char);
                glyphs.insert(char.data);
            }
        }
    }
    for char in &map_characters {
        blocks
            .entry(block_name(char.ch))
            .or_insert_with(Vec::new)
            .push(char.ch);
    }

    // print statistics
    let mut block_names: Vec<_> = blocks.keys().collect();
    block_names.sort();
    for block in block_names {
        let mut block_contents = blocks.get(block).unwrap().clone();
        block_contents.sort();
        if block_contents.len() < 130 {
            println!("{block} ({} characters): {block_contents:?}", block_contents.len());
        } else {
            println!("{block} ({} characters): [elided]", block_contents.len());
        }
    }
    println!();

    // find list of allowed/available characters
    println!("Supported codepoints: {}", map_characters.len());
    println!("Total glyph count: {}", glyphs.len());
    let mut map_characters: Vec<_> = map_characters.drain().collect();
    map_characters.sort_by_key(|x| x.ch as u32);
    map_characters
}

struct GlyphData {
    data: Vec<u8>,
    low_plane: Vec<bool>,
    low_plane_half_width: Vec<bool>,
    glyph_map: HashMap<u16, (usize, usize, bool)>,
    glyph_lookup: HashMap<u16, (usize, usize, bool)>,
}

struct GlyphPlaneBuilder {
    config: FontConfiguration,

    low_plane_table: Vec<bool>,
    low_plane_half_width: Vec<bool>,
    low_plane_assigned: HashSet<(usize, usize)>,

    available: Vec<(usize, usize)>,
    available_half: Vec<(usize, usize)>,

    glyph_planes: Vec<Vec<u64>>,
    glyph_map: HashMap<u16, (usize, usize, bool)>,
    glyph_lookup: HashMap<u16, (usize, usize, bool)>,
    char_is_half: Vec<Option<bool>>,
    glyph_needs_half: HashSet<u64>,
    glyph_assigned: HashMap<u64, (usize, usize)>,

    dupe_low: usize,
}
impl GlyphPlaneBuilder {
    fn preprocess_glyph(&mut self, i: &CharacterInfo) {
        if i.is_half_width {
            self.glyph_needs_half.insert(i.data);
        }
    }

    fn set_plane_width(&mut self, char: usize, data: u64, is_half_width: bool) {
        if is_half_width {
            assert_ne!(self.char_is_half[char], Some(false));
            self.char_is_half[char] = Some(true);
        } else if !data_is_half_width(data) {
            assert_ne!(self.char_is_half[char], Some(true));
            self.char_is_half[char] = Some(false);
        }
    }

    fn try_insert_low_plane(&mut self, i: &CharacterInfo) {
        if self.glyph_lookup.contains_key(&(i.ch as u16)) {
            return;
        }

        let (plane, char) = split_plane(&self.config, i.ch);
        let low_plane_valid = !self.low_plane_assigned.contains(&(plane, char))
            || self.glyph_planes[plane][char] == i.data;
        if !low_plane_valid
            || (self.glyph_assigned.contains_key(&i.data)
                && (i.ch as usize) >= self.config.low_plane_dupe_limit)
            || self.char_is_half[char] == Some(!self.glyph_needs_half.contains(&i.data))
        {
            return;
        }

        self.low_plane_table[i.ch as usize] = true;
        self.low_plane_half_width[i.ch as usize] = i.is_half_width;
        self.low_plane_assigned.insert((plane, char));
        self.glyph_planes[plane][char] = i.data;
        self.glyph_lookup
            .insert(i.ch as u16, (plane, char, i.is_half_width));
        self.set_plane_width(char, i.data, i.is_half_width);

        if !self.glyph_assigned.contains_key(&i.data) {
            self.glyph_assigned.insert(i.data, (plane, char));
        } else {
            self.dupe_low += 1;
        }
    }

    fn find_available(&mut self) {
        for char in 0..self.config.character_count / 4 {
            for plane in 0..4 {
                if !self.low_plane_assigned.contains(&(plane, char)) {
                    if self.char_is_half[char] != Some(true) {
                        self.available.push((plane, char));
                    } else {
                        self.available_half.push((plane, char));
                    }
                }
            }
        }
        self.available.reverse();
        self.available_half.reverse();
    }

    fn first_available_half(&mut self, can_use_normal: bool) -> (usize, usize) {
        for i in 0..self.available.len() {
            if self.char_is_half[self.available[i].1] != Some(false) {
                return self.available.remove(i);
            }
        }
        if can_use_normal {
            self.available
                .pop()
                .or_else(|| self.available_half.pop())
                .expect("Ran out of glyph slots!!")
        } else {
            panic!("Ran out of glyph slots!!")
        }
    }
    fn next_available(&mut self, is_half: bool, data: u64) -> (usize, usize) {
        if is_half {
            if let Some(slot) = self.available_half.pop() {
                slot
            } else {
                self.first_available_half(false)
            }
        } else if data_is_half_width(data) {
            self.first_available_half(true)
        } else {
            while let Some((plane, char)) = self.available.pop() {
                if self.char_is_half[char] == Some(true) {
                    self.available_half.push((plane, char))
                } else {
                    return (plane, char);
                }
            }
            panic!("Ran out of glyph slots!!")
        }
    }

    fn try_assign_character(&mut self, i: &CharacterInfo) {
        if self.glyph_lookup.contains_key(&(i.ch as u16)) {
            return;
        }
        if let Some((plane, char)) = self.glyph_assigned.get(&i.data) {
            let map_slot = (*plane, *char, i.is_half_width);
            self.glyph_map.insert(i.ch as u16, map_slot);
            self.glyph_lookup.insert(i.ch as u16, map_slot);
        } else {
            let (plane, char) = self.next_available(i.is_half_width, i.data);
            let map_slot = (plane, char, i.is_half_width);
            self.glyph_map.insert(i.ch as u16, map_slot);
            self.glyph_lookup.insert(i.ch as u16, map_slot);
            self.glyph_planes[plane][char] = i.data;
            self.glyph_assigned.insert(i.data, (plane, char));
            self.set_plane_width(char, i.data, i.is_half_width);
        }
    }
}

fn split_plane(config: &FontConfiguration, id: char) -> (usize, usize) {
    assert!((id as usize) < config.character_count);
    (id as usize % 4, id as usize / 4)
}
fn build_planes(config: &FontConfiguration, characters: Vec<CharacterInfo>) -> GlyphData {
    let mut builder = GlyphPlaneBuilder {
        config: *config,
        low_plane_table: vec![false; config.low_plane_limit],
        low_plane_half_width: vec![false; config.low_plane_limit],
        low_plane_assigned: Default::default(),
        available: vec![],
        available_half: vec![],
        glyph_planes: vec![vec![0u64; config.character_count / 4]; 4],
        glyph_map: Default::default(),
        glyph_lookup: Default::default(),
        char_is_half: vec![None; config.character_count / 4],
        glyph_needs_half: Default::default(),
        glyph_assigned: Default::default(),
        dupe_low: 0,
    };

    // preprocess glyphs
    for i in &characters {
        builder.preprocess_glyph(i);
    }

    // assign low plane
    for i in &characters {
        if i.ch == ' ' {
            builder.try_insert_low_plane(i);
        }
    }
    for i in &characters {
        if i.ch as usize >= config.low_plane_limit {
            break;
        }
        if builder.glyph_needs_half.contains(&i.data) {
            builder.try_insert_low_plane(i);
        }
    }
    for i in &characters {
        if i.ch as usize >= config.low_plane_limit {
            break;
        }
        builder.try_insert_low_plane(i);
    }
    println!(
        "Low character slots used: {}",
        builder.low_plane_table.iter().filter(|x| **x).count()
    );
    println!("Duplicated low characters: {}", builder.dupe_low);

    // build table of available glyph locations
    builder.find_available();

    // assign remaining characters to the glyph planes
    for i in &characters {
        builder.try_assign_character(i);
    }
    println!("Glyph table size: {}", builder.glyph_map.len());
    println!(
        "Remaining glyph slots: {}",
        builder.available.len() + builder.available_half.len()
    );

    // Interlace planes into something the GBA can use.
    let mut data = vec![0u8; ((config.character_count / 4) * 8 * 8 * 4) / 8];
    for plane in 0..4 {
        for char in 0..config.character_count / 4 {
            // iterate through the glyph's pixels
            let glyph = builder.glyph_planes[plane][char];
            for x in 0..8 {
                for y in 0..8 {
                    // check if the pixel is on
                    if glyph & (1 << (63 - (x + y * 8))) != 0 {
                        // set the appropriate bit
                        let byte = char * 32 + (x >> 1) + y * 4;
                        let bit = (3 - plane) + (x % 2) * 4;
                        data[byte] |= 1 << bit;
                    }
                }
            }
        }
    }

    // Ensure all characters are assigned successfully
    for i in &characters {
        let glyph = builder.glyph_lookup.get(&(i.ch as u16)).unwrap();
        assert_eq!(i.data, builder.glyph_planes[glyph.0][glyph.1]);
        assert_eq!(i.is_half_width, glyph.2);
    }

    // Returns the glyph data
    GlyphData {
        data,
        low_plane: builder.low_plane_table,
        low_plane_half_width: builder.low_plane_half_width,
        glyph_map: builder.glyph_map,
        glyph_lookup: builder.glyph_lookup,
    }
}

fn make_u16_file(data: &[u16]) -> Vec<u8> {
    let mut vec = Vec::new();
    for v in data {
        vec.extend_from_slice(&v.to_le_bytes());
    }
    vec
}
fn make_glyphs_file(config: &FontConfiguration, glyphs: GlyphData) -> Result<()> {
    // Creates the low plane table bitset
    let mut low_plane = vec![0u16; config.low_plane_limit / 16];
    for (i, is_low_glyph) in glyphs.low_plane.iter().enumerate() {
        if *is_low_glyph {
            low_plane[i >> 4] |= 1 << (i % 16);
        }
    }

    // Compute the raw PHF for the high planes
    let entries: Vec<_> = glyphs.glyph_map.keys().cloned().collect();
    let phf = lgba_phf::generator::generate_hash(config.delta, &entries);
    let phf_code = phf.generate_rust_code_include("lookup_glyph", "u16", "phf_disps.bin");

    // Build the PHF glyph data
    let glyph_size = (glyphs.glyph_map.len() - 1).next_power_of_two();
    let glyph_char_bits = (config.character_count / 4 - 1)
        .next_power_of_two()
        .trailing_zeros();
    let (hi_bits, divisor) = match glyph_char_bits + 2 {
        x if x <= 8 => (0, 0xFFFFFFFF),
        x if x <= 9 => (1, 16),
        x if x <= 10 => (2, 8),
        x if x <= 12 => (4, 4),
        x if x <= 16 => (8, 2),
        _ => unreachable!(),
    };
    let hi_arr_size = glyph_size / divisor;

    let mut glyph_check = vec![0u16; glyph_size];
    let mut glyph_id_hi = vec![0u16; hi_arr_size];
    let mut glyph_id_lo = vec![0u8; glyph_size];
    for (i, map) in phf.map.iter().enumerate() {
        if *map == !0 {
            continue;
        }

        let glyph = entries[*map];
        glyph_check[i] = glyph;

        let (plane, char, _) = glyphs.glyph_map.get(&glyph).unwrap();
        let packed = (*plane << glyph_char_bits) | *char;

        if hi_bits != 0 {
            let hi = (packed >> 8) as u16;
            glyph_id_hi[i / divisor] |= hi << (hi_bits * (i % divisor));
        }
        glyph_id_lo[i] = packed as u8;
    }

    // Find the replacement glyph
    let (fallback_hi, fallback_lo, _) = glyphs.glyph_lookup[&(config.fallback_char as u16)];

    // Calculate statistics
    let bytes = glyphs.data.len()
        + low_plane.len() * 2
        + glyph_check.len() * 2
        + glyph_id_hi.len() * 2
        + glyph_id_lo.len()
        + phf.disps.len() * 2;
    let kib = (bytes as f32) / 1024.0;
    println!("Total bytes: {bytes} bytes, {:.2} KiB", kib);

    // Write the font data source file.
    let font_type = config.font_type;
    let lo_map_size = config.low_plane_limit;
    let lo_map_len = config.low_plane_limit / 16;
    let char_count = config.character_count;
    let divisor_mask = divisor - 1;
    let divisor_shift = divisor.trailing_zeros();
    let description = config.description.replace("\n", "\n/// ");
    let has_half_width = !config.allow_halfwidth_blocks.is_empty();

    let mut available_blocks = String::new();
    for block in config.whitelisted_blocks {
        available_blocks.push_str(&format!("/// * {block}\n"))
    }

    let mut additional_characters = String::new();
    if config.whitelisted_chars.len() != 0 {
        additional_characters
            .push_str("///\n/// The following additional characters are available:\n/// * ");
        let mut whitelisted_chars = config.whitelisted_chars.to_vec();
        whitelisted_chars.sort();
        for char in config.whitelisted_chars {
            additional_characters.push_str(&format!("`{char}`, "))
        }
        additional_characters.pop();
        additional_characters.pop();
        additional_characters.push('\n');
    }

    let glyph_hi_array = if hi_bits != 0 {
        format!("static GLYPH_ID_HI: [u16; {hi_arr_size}] = *include_u16!(\"glyph_id_hi.bin\");\n")
    } else {
        String::new()
    };
    let hi_mask = if hi_bits != 0 {
        format!("const HI_MASK: u16 = (1 << {hi_bits}) - 1;\n")
    } else {
        String::new()
    };
    let load_hi = if hi_bits != 0 {
        format!(
            "@            let word = GLYPH_ID_HI[slot >> {divisor_shift}];\n\
             @            let hi = (word >> ({hi_bits} * (slot & {divisor_mask}))) & HI_MASK;\n\
             @            let packed = (hi << 8) | (GLYPH_ID_LO[slot] as u16);"
        )
    } else {
        format!("@            let packed = GLYPH_ID_LO[slot] as u16;\n")
    };

    let raw_source = format!(
        "\
            // This file is generated by the `build_codegen` script found in the repository root.\n\
            // Do not edit it manually.\n\
            \n\
            use super::*;\n\
            \n\
            const FALLBACK_GLYPH: (u8, u16, bool) = ({fallback_hi}, {fallback_lo}, false);\n\
            static LO_MAP_DATA: [u16; {lo_map_len}] = *include_u16!(\"lo_map.bin\");\n\
            static GLYPH_CHECK: [u16; {glyph_size}] = *include_u16!(\"glyph_check.bin\");\n\
            {glyph_hi_array}\
            static GLYPH_ID_LO: [u8; {glyph_size}] = *include_u8!(\"glyph_id_lo.bin\");\n\
            static FONT_DATA: [u32; {char_count} * 2] = *include_u32!(\"font.chr\");\n\
            \n\
            {phf_code}\
            \n\
            /// {description}\n\
            /// \n\
            /// The data files for this font require {kib:.2} KiB of ROM space, not including\n\
            /// any font-specific code that may be generated.\n\
            /// \n\
            /// # Licencing\n\
            /// \n\
            /// The data of this font is based on a combined subset of the following fonts:\n\
            /// \n\
            /// * [Unscii 2.0](http://viznut.fi/unscii/)\n\
            /// * [Misaki Font Gothic 2](https://littlelimit.net/misaki.htm)\n\
            /// \n\
            /// Both are released under the public domain.\n\
            /// \n\
            /// # Available Characters\n\
            /// \n\
            /// The following Unicode blocks are available in this font:\n\
            /// \n\
            {available_blocks}\
            {additional_characters}\
            pub struct {font_type}(());\n\
            \n\
            {hi_mask}\
            const CHAR_MASK: u16 = (1 << {glyph_char_bits}) - 1;\n\
            fn get_font_glyph(id: char) -> (u8, u16, bool) {{\n\
           @    let id = id as usize;\n\
           @    if id < {lo_map_size} {{\n\
           @        // We check the low plane bitmap to see if we have this glyph.\n\
           @        let word = LO_MAP_DATA[id >> 4];\n\
           @        if word & (1 << (id & 15)) != 0 {{\n\
           @            ((id & 3) as u8, (id >> 2) as u16, false)\n\
           @        }} else {{\n\
           @            FALLBACK_GLYPH\n\
           @        }}\n\
           @    }} else if id < 0x10000 {{\n\
           @        // Check the PHF to see if we have this glyph.\n\
           @        let slot = lookup_glyph(&(id as u16));\n\
           @        if id == GLYPH_CHECK[slot] as usize {{\n\
                        {load_hi}\
           @            ((packed >> {glyph_char_bits}) as u8, packed & CHAR_MASK, false)\n\
           @        }} else {{\n\
           @            FALLBACK_GLYPH\n\
           @        }}\n\
           @    }} else {{\n\
           @        // We only support the BMP, don't bother.\n\
           @        FALLBACK_GLYPH\n\
           @    }}\n\
            }}\n\
            \n\
            impl TerminalFont for {font_type} {{\n\
           @    fn instance() -> &'static Self {{\n\
           @        &{font_type}(())\n\
           @    }}\n\
           @    fn get_font_glyph(&self, id: char) -> (u8, u16, bool) {{\n\
           @        get_font_glyph(id)\n\
           @    }}\n\
           @    fn get_font_data(&self) -> &'static [u32] {{\n\
           @        &FONT_DATA\n\
           @    }}\n\
           @    fn has_half_width(&self) -> bool {{\n\
           @        {has_half_width}\n\
           @    }}\n\
            }}\n\
        "
    );
    let raw_source = raw_source.replace('@', "");

    std::fs::create_dir_all(format!("../lgba/src/display/terminal/{}", config.font_name)).ok();
    File::create(format!("../lgba/src/display/terminal/{}/font.chr", config.font_name))?
        .write_all(&glyphs.data)?;
    File::create(format!("../lgba/src/display/terminal/{}/lo_map.bin", config.font_name))?
        .write_all(&make_u16_file(&low_plane))?;
    File::create(format!("../lgba/src/display/terminal/{}/glyph_check.bin", config.font_name))?
        .write_all(&make_u16_file(&glyph_check))?;
    if hi_bits != 0 {
        File::create(format!("../lgba/src/display/terminal/{}/glyph_id_hi.bin", config.font_name))?
            .write_all(&make_u16_file(&glyph_id_hi))?;
    }
    File::create(format!("../lgba/src/display/terminal/{}/glyph_id_lo.bin", config.font_name))?
        .write_all(&glyph_id_lo)?;
    File::create(format!("../lgba/src/display/terminal/{}/phf_disps.bin", config.font_name))?
        .write_all(&make_u16_file(&phf.disps))?;
    File::create(format!("../lgba/src/display/terminal/{}/mod.rs", config.font_name))?
        .write_all(raw_source.as_bytes())?;

    Ok(())
}

pub fn print_all_blocks(characters: &CharacterSets) {
    generate_fonts(&DEBUG_FONT_UNSCII, characters);
    generate_fonts(&DEBUG_FONT_MISAKI, characters);
}
pub fn generate_fonts(config: &FontConfiguration, characters: &CharacterSets) {
    if config.font_name == "*" {
        println!("###### Available blocks: {} #####", config.font_type);
    } else {
        println!("###### Generating font: {} #####", config.font_name);
    }
    println!();
    let characters = parse_fonts(config, characters);
    let character_list = gather_characters(config, characters);
    if config.font_name != "*" {
        let glyphs = build_planes(config, character_list);
        make_glyphs_file(config, glyphs).expect("Failed to write font files...");
    }
    println!();
}
