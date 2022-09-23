use crate::download_fonts::{CharacterInfo, CharacterSets};
use anyhow::*;
use std::{
    collections::{HashMap, HashSet},
    fs::File,
    hash::Hash,
    io::Write,
};
use unic_ucd::{block::Block, common::is_control, normal::is_combining_mark, BidiClass};

pub struct FontConfiguration {
    pub font_name: &'static str,
    pub font_type: &'static str,
    pub description: &'static str,
    pub low_plane_limit: usize,
    pub low_plane_dupe_limit: usize,
    pub misaki_override_blocks: &'static [&'static str],
    pub allow_all_blocks: bool,
    pub whitelisted_blocks: &'static [&'static str],
    pub glyph_whitelisted_blocks: &'static [&'static str],
    pub whitelisted_chars: &'static [char],
    pub fallback_char: char,
    pub kanji_max_level: kanji::Level,
    pub character_count: usize,
    pub delta: f32,
}

const DEBUG_FONT: FontConfiguration = FontConfiguration {
    font_name: "*",
    font_type: "",
    description: "",
    low_plane_limit: 0,
    low_plane_dupe_limit: 0,
    misaki_override_blocks: &[],
    allow_all_blocks: true,
    whitelisted_blocks: &[],
    glyph_whitelisted_blocks: &[],
    whitelisted_chars: &[],
    fallback_char: '?',
    kanji_max_level: kanji::Level::One,
    character_count: 0,
    delta: 0.0,
};

fn block_name(ch: char) -> &'static str {
    Block::of(ch).unwrap().name
}
fn list_to_set<V: Copy + Hash + Eq>(list: &[V]) -> HashSet<V> {
    let mut set = HashSet::new();
    for s in list {
        set.insert(*s);
    }
    set
}

fn parse_fonts(config: &FontConfiguration, characters: &CharacterSets) -> Vec<CharacterInfo> {
    let mut char_map = HashMap::new();

    // parse unscii-8
    for char in &characters.unscii {
        if char.ch != '\0' {
            char_map.insert(char.ch, *char);
        }
    }

    // add characters from misaki font
    let override_blocks = list_to_set(config.misaki_override_blocks);
    for char in &characters.misaki {
        if !char_map.contains_key(&char.ch) || override_blocks.contains(block_name(char.ch)) {
            char_map.insert(char.ch, *char);
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
    let glyph_whitelisted_blocks = list_to_set(config.glyph_whitelisted_blocks);
    let kanji_level = kanji::level_table();

    glyphs.insert(0);
    for char in &characters {
        if let Some(block) = Block::of(char.ch) {
            if whitelisted_chars.contains(&char.ch)
                || ((config.allow_all_blocks || whitelisted_blocks.contains(block.name))
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
    }
    for char in &characters {
        if let Some(block) = Block::of(char.ch) {
            if glyphs.contains(&char.data)
                && (whitelisted_blocks.contains(block.name)
                    || glyph_whitelisted_blocks.contains(block.name))
            {
                map_characters.insert(*char);
            }
        }
    }
    for char in &map_characters {
        if let Some(block) = Block::of(char.ch) {
            blocks
                .entry(block.name)
                .or_insert_with(Vec::new)
                .push(char.ch);
        }
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
    glyph_map: HashMap<u16, (usize, usize)>,
    glyph_lookup: HashMap<u16, (usize, usize)>,
}

fn split_plane(config: &FontConfiguration, id: char) -> (usize, usize) {
    assert!((id as usize) < config.character_count);
    (id as usize % 4, id as usize / 4)
}
fn build_planes(config: &FontConfiguration, characters: Vec<CharacterInfo>) -> GlyphData {
    let mut low_plane_table = vec![false; config.low_plane_limit];
    let mut low_plane_assigned = HashSet::new();
    let mut glyph_planes = vec![vec![0u64; config.character_count / 4]; 4];
    let mut glyph_map = HashMap::new();
    let mut glyph_lookup = HashMap::new();
    let mut assigned = HashMap::new();

    // space is always placed in ' '
    if config.low_plane_limit > ' ' as usize {
        low_plane_table[' ' as usize] = true;
        let (plane, char) = split_plane(config, ' ');
        assigned.insert(0, (plane, char));
    }

    // Assign characters < 256 to plane 0 at a position equaling the character id
    let mut dupe_low = 0;
    for i in &characters {
        if i.ch == ' ' {
            continue;
        }
        if i.ch as usize >= config.low_plane_limit {
            break;
        }

        let (plane, char) = split_plane(config, i.ch);
        if (low_plane_assigned.contains(&(plane, char))
            && assigned.get(&i.data) != Some(&(plane, char)))
            || (assigned.contains_key(&i.data) && (i.ch as usize) >= config.low_plane_dupe_limit)
        {
            continue;
        }

        low_plane_table[i.ch as usize] = true;
        low_plane_assigned.insert((plane, char));
        glyph_planes[plane][char] = i.data;
        glyph_lookup.insert(i.ch as u16, (plane, char));

        if !assigned.contains_key(&i.data) {
            assigned.insert(i.data, (plane, char));
        } else {
            dupe_low += 1;
        }
    }
    println!("Low character slots used: {}", low_plane_table.iter().filter(|x| **x).count());
    println!("Duplicated low characters: {dupe_low}");

    // build table of available glyph locations
    let mut available = Vec::new();
    for plane in 0..4 {
        for char in 0..config.character_count / 4 {
            if !low_plane_assigned.contains(&(plane, char)) {
                available.push((plane, char));
            }
        }
    }
    available.reverse();

    // assign remaining characters to the glyph planes
    for i in &characters {
        if (i.ch as usize) < config.low_plane_limit && low_plane_table[i.ch as usize] {
            continue;
        }

        if assigned.contains_key(&i.data) {
            let slot = *assigned.get(&i.data).unwrap();
            glyph_map.insert(i.ch as u16, slot);
            glyph_lookup.insert(i.ch as u16, slot);
        } else {
            let slot = available.pop().expect("Ran out of glyph slots!!");
            glyph_map.insert(i.ch as u16, slot);
            glyph_lookup.insert(i.ch as u16, slot);
            glyph_planes[slot.0][slot.1] = i.data;
            assigned.insert(i.data, slot);
        }
    }
    println!("Glyph table size: {}", glyph_map.len());
    println!("Remaining glyph slots: {}", available.len());

    // Interlace planes into something the GBA can use.
    let mut data = vec![0u8; ((config.character_count / 4) * 8 * 8 * 4) / 8];
    for plane in 0..4 {
        for char in 0..config.character_count / 4 {
            // iterate through the glyph's pixels
            let glyph = glyph_planes[plane][char];
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

    // Returns the glyph data
    GlyphData { data, low_plane: low_plane_table, glyph_map, glyph_lookup }
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

        let (plane, char) = glyphs.glyph_map.get(&glyph).unwrap();
        let packed = (*plane << glyph_char_bits) | *char;

        if hi_bits != 0 {
            let hi = (packed >> 8) as u16;
            glyph_id_hi[i / divisor] |= hi << (hi_bits * (i % divisor));
        }
        glyph_id_lo[i] = packed as u8;
    }

    // Find the replacement glyph
    let (replacement_hi, replacement_lo) = glyphs.glyph_lookup[&(config.fallback_char as u16)];

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
    let description = config.description.replace("\n", "\n/// ");
    let mut available_blocks = String::new();
    for block in config.whitelisted_blocks {
        available_blocks.push_str(&format!("/// * {block}\n"))
    }
    let mut additional_characters = String::new();
    if config.whitelisted_chars.len() != 0 {
        additional_characters.push_str(
            "///\n/// The following additional characters are available:\n/// * "
        );
        let mut whitelisted_chars = config.whitelisted_chars.to_vec();
        whitelisted_chars.sort();
        for char in config.whitelisted_chars {
            additional_characters.push_str(&format!("`{char}`, "))
        }
        additional_characters.pop();
        additional_characters.pop();
        additional_characters.push('\n');
    }
    let raw_source = format!(
        "\
            // This file is generated by the `build_codegen` script found in the repository root.\n\
            // Do not edit it manually.\n\
            \n\
            use super::*;\n\
            \n\
            const REPLACEMENT_GLYPH: (u8, u16) = ({replacement_hi}, {replacement_lo});\n\
            static LO_MAP_DATA: [u16; {lo_map_len}] = *include_u16!(\"lo_map.bin\");\n\
            static GLYPH_CHECK: [u16; {glyph_size}] = *include_u16!(\"glyph_check.bin\");\n\
            static GLYPH_ID_HI: [u16; {hi_arr_size}] = *include_u16!(\"glyph_id_hi.bin\");\n\
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
            fn get_font_glyph(id: char) -> (u8, u16) {{\n\
           @    let id = id as usize;\n\
           @    if id < {lo_map_size} {{\n\
           @        // We check the low plane bitmap to see if we have this glyph.\n\
           @        let word = LO_MAP_DATA[id >> 4];\n\
           @        if word & 1 << (id & 15) != 0 {{\n\
           @            ((id & 3) as u8, (id >> 2) as u16)\n\
           @        }} else {{\n\
           @            REPLACEMENT_GLYPH\n\
           @        }}\n\
           @    }} else if id < 0x10000 {{\n\
           @        // Check the PHF to see if we have this glyph.\n\
           @        let slot = lookup_glyph(&(id as u16));\n\
           @        if id == GLYPH_CHECK[slot] as usize {{\n\
           @            let hi_mask = (1 << {hi_bits}) - 1;\n\
           @            let char_mask = (1 << {glyph_char_bits}) - 1;\n\
           @            \n\
           @            let word = GLYPH_ID_HI[slot >> 3];\n\
           @            let hi = (word >> ({hi_bits} * (slot & {divisor_mask}))) & hi_mask;\n\
           @            let lo = GLYPH_ID_LO[slot];\n\
           @            let packed = (hi << 8) | (lo as u16);\n\
           @            ((packed >> {glyph_char_bits}) as u8, packed & char_mask)\n\
           @        }} else {{\n\
           @            REPLACEMENT_GLYPH\n\
           @        }}\n\
           @    }} else {{\n\
           @        // We only support the BMP, don't bother.\n\
           @        REPLACEMENT_GLYPH\n\
           @    }}\n\
            }}\n\
            \n\
            impl TerminalFont for {font_type} {{\n\
           @    fn instance() -> &'static Self {{\n\
           @        &{font_type}(())\n\
           @    }}\n\
           @    fn get_font_glyph(&self, id: char) -> (u8, u16) {{\n\
           @        get_font_glyph(id)\n\
           @    }}\n\
           @    fn get_font_data(&self) -> &'static [u32] {{\n\
           @        &FONT_DATA\n\
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
    File::create(format!("../lgba/src/display/terminal/{}/glyph_id_hi.bin", config.font_name))?
        .write_all(&make_u16_file(&glyph_id_hi))?;
    File::create(format!("../lgba/src/display/terminal/{}/glyph_id_lo.bin", config.font_name))?
        .write_all(&glyph_id_lo)?;
    File::create(format!("../lgba/src/display/terminal/{}/phf_disps.bin", config.font_name))?
        .write_all(&make_u16_file(&phf.disps))?;
    File::create(format!("../lgba/src/display/terminal/{}/mod.rs", config.font_name))?
        .write_all(raw_source.as_bytes())?;

    Ok(())
}

pub fn print_all_blocks(characters: &CharacterSets) {
    generate_fonts(&DEBUG_FONT, characters);
}
pub fn generate_fonts(config: &FontConfiguration, characters: &CharacterSets) {
    if config.font_name == "*" {
        println!("###### Available Blocks #####");
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
