use anyhow::*;
use std::{
    cmp::max,
    collections::{HashMap, HashSet},
    fs::File,
    hash::Hash,
    io::{Cursor, Write},
};
use unic_ucd::{block::Block, common::is_control, normal::is_combining_mark, BidiClass, Name};
use zip::ZipArchive;

// Download URLs for the source fonts
const UNSCII_FONT_URL: &str = "http://viznut.fi/unscii/unscii-8.hex";
const MISAKI_FONT_URL: &str = "https://littlelimit.net/arc/misaki/misaki_bdf_2021-05-05.zip";

// Block configurations
const MISAKI_OVERRIDE_BLOCKS: &[&str] = &["Halfwidth and Fullwidth Forms"];
const BLACKLISTED_BLOCKS: &[&str] = &[
    // Exclude character sets we cannot render well.
    "Combining Diacritical Marks", // combining
    // Exclude characters not used in modern languages.
    "Greek Extended",
    "Runic",
    // These take up far too many characters.
    "Braille Patterns",
    "CJK Compatibility",
    "CJK Unified Ideographs",
    // For whatever reason, this block is *very* incomplete in unscii.
    "Unified Canadian Aboriginal Syllabics",
    // Don't include graphical characters
    "Dingbats",
    "Enclosed Alphanumerics",
    "Enclosed CJK Letters and Months",
    "Geometric Shapes",
    "Mathematical Operators",
    "Miscellaneous Mathematical Symbols-B",
    "Miscellaneous Symbols",
    "Miscellaneous Symbols and Arrows",
    "Miscellaneous Technical",
    "Number Forms",
    "Private Use Area",
    "Spacing Modifier Letters",
    "Superscripts and Subscripts",
    "Yijing Hexagram Symbols",
];
const BLACKLISTED_NAMES: &[(&str, &str)] = &[
    ("Box Drawing", "DOUBLE"),
    ("Box Drawing", "HEAVY"),
    ("Halfwidth and Fullwidth Forms", "HALFWIDTH KATAKANA"),
];
const WHITELISTED_CHARS: &[char] = &[
    '①', '②', '③', '④', '⑤', '⑥', '⑦', '⑧', '⑨', '■', '□', '▢', '▲', '△', '▶', '▷', '▼', '▽', '◀',
    '◁', '◆', '◇', '○', '●', '≠', '≤', '≥', '★', '☆', '♩', '♪', '♫', '♬', '⮜', '⮝', '⮞', '⮟', '⯀',
    '⯅', '⯆', '⯇', '⯈', '∞', '⌘', '⌛', '⏩', '⏪', 'Ⅰ', 'Ⅱ', 'Ⅲ', 'Ⅳ', 'Ⅴ', 'Ⅵ', 'Ⅶ', 'Ⅷ', 'Ⅸ',
    'Ⅹ',
];

// Misc configuration
const FALLBACK_CHARACTER: char = '⁇';

fn block_name(ch: char) -> &'static str {
    Block::of(ch).unwrap().name
}
fn check_name(ch: char, block: &str, name: &str) -> bool {
    block_name(ch) == block && Name::of(ch).unwrap().to_string().contains(name)
}
fn list_to_set<V: Copy + Hash + Eq>(list: &'static [V]) -> HashSet<V> {
    let mut set = HashSet::new();
    for s in list {
        set.insert(*s);
    }
    set
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug, Hash)]
struct CharacterInfo {
    ch: char,
    data: u64,
}

fn download_fonts() -> Result<Vec<CharacterInfo>> {
    let mut char_map = HashMap::new();

    // download unscii-8
    let unscii_string =
        String::from_utf8(reqwest::blocking::get(UNSCII_FONT_URL)?.bytes()?.to_vec())?;

    // parse unscii-8
    for line in unscii_string.split('\n').filter(|x| !x.is_empty()) {
        let mut split = line.split(':');

        let hex_str = split.next().unwrap();
        let hex_bmp = split.next().unwrap();

        let ch = char::from_u32(u32::from_str_radix(&hex_str, 16)?).unwrap();
        let data = u64::from_str_radix(&hex_bmp, 16)?;
        if !char_map.contains_key(&ch) {
            if ch != '\0' {
                char_map.insert(ch, CharacterInfo { ch, data });
            }
        }
    }

    // download and parse misaki font
    let misaki_font_zip = reqwest::blocking::get(MISAKI_FONT_URL)?.bytes()?.to_vec();
    let mut zip_reader = ZipArchive::new(Cursor::new(misaki_font_zip))?;
    let misaki_font = bdf::read(zip_reader.by_name("misaki_gothic_2nd.bdf")?)?;

    // add characters from misaki font
    let override_blocks = list_to_set(MISAKI_OVERRIDE_BLOCKS);
    for (char, glyph) in misaki_font.glyphs() {
        if char_map.contains_key(char) && !override_blocks.contains(block_name(*char)) {
            continue;
        }

        // compute the bounds of the glyph
        let x_off = glyph.bounds().x as u32;
        let y_off = if glyph.height() != 8 {
            (8 - glyph.height()) - 1 - max(0, glyph.bounds().y as u32)
        } else {
            0
        };

        // copy the glyph to a `u64` format
        let mut glyph_data = 0u64;
        for x in 0..glyph.width() {
            for y in 0..glyph.height() {
                let tx = x + x_off;
                let ty = y + y_off;
                glyph_data |= (glyph.get(x, y) as u64) << (63 - (tx + ty * 8));
            }
        }

        // add the glyph to the character map
        char_map.insert(*char, CharacterInfo { ch: *char, data: glyph_data });
    }

    // return the downloaded characters
    let mut characters: Vec<_> = char_map.values().cloned().collect();
    characters.sort_by_key(|x| x.ch as u32);
    Ok(characters)
}

fn gather_characters(characters: Vec<CharacterInfo>) -> Vec<CharacterInfo> {
    // calculate list of (root) characters to include in the font
    let mut map_characters = Vec::new();
    let mut blocks = HashMap::new();
    let mut glyphs = HashSet::new();

    let blacklisted_blocks = list_to_set(BLACKLISTED_BLOCKS);
    let whitelisted_chars = list_to_set(WHITELISTED_CHARS);

    glyphs.insert(0);
    'outer: for char in &characters {
        if let Some(block) = Block::of(char.ch) {
            for (block, name) in BLACKLISTED_NAMES {
                if check_name(char.ch, *block, *name) {
                    continue 'outer;
                }
            }

            if whitelisted_chars.contains(&char.ch)
                || (!blacklisted_blocks.contains(block.name)
                    && !is_control(char.ch)
                    && !is_combining_mark(char.ch)
                    && !BidiClass::of(char.ch).is_rtl()
                    && (char.ch as u32) < 0x10000)
            {
                glyphs.insert(char.data);
                blocks
                    .entry(block.name)
                    .or_insert_with(Vec::new)
                    .push(char.ch);
            }
        }
    }
    for char in &characters {
        if glyphs.contains(&char.data) {
            map_characters.push(*char);
        }
    }

    // print statistics
    let mut block_names: Vec<_> = blocks.keys().collect();
    block_names.sort();
    for block in block_names {
        let block_contents = blocks.get(block).unwrap();
        println!("{block} ({} characters): {block_contents:?}", block_contents.len());
    }

    // find list of allowed/available characters
    println!("Supported codepoints: {}", map_characters.len());
    println!("Total glyph count: {}", glyphs.len());
    println!(
        "Character map table size: {}",
        map_characters
            .iter()
            .filter(|x| (x.ch as u32) >= 256)
            .count()
    );
    map_characters.sort_by_key(|x| x.ch as u32);
    map_characters
}

struct GlyphData {
    data: [u8; 1024 * 8],
    low_plane: [bool; 256],
    glyph_map: HashMap<u16, (usize, usize)>,
}

fn build_planes(characters: Vec<CharacterInfo>) -> GlyphData {
    let mut low_plane_table = [false; 256];
    let mut glyph_planes = [[0u64; 256]; 4];
    let mut glyph_map = HashMap::new();
    let mut assigned = HashMap::new();

    // space is always placed in ' '
    low_plane_table[' ' as usize] = true;
    assigned.insert(0, (0, ' ' as usize));

    // Assign characters < 256 to plane 0 at a position equaling the character id
    let mut dupe_low = 0;
    for i in &characters {
        if i.ch == ' ' {
            continue;
        }
        if (i.ch as u32) >= 256 {
            break;
        }

        low_plane_table[i.ch as usize] = true;
        glyph_planes[0][i.ch as usize] = i.data;
        if !assigned.contains_key(&i.data) {
            assigned.insert(i.data, (0, i.ch as usize));
        } else {
            dupe_low += 1;
        }
    }
    println!("Low character slots used: {}", low_plane_table.iter().filter(|x| **x).count());
    println!("Duplicated low characters: {dupe_low}");

    // build table of available glyph locations
    let mut available = Vec::new();
    for plane in 0..4 {
        for char in 0..256 {
            if plane != 0 || !low_plane_table[char] {
                available.push((plane, char));
            }
        }
    }
    available.reverse();

    // assign remaining characters to the glyph planes
    for i in &characters {
        if (i.ch as u32) < 256 {
            continue;
        }

        if assigned.contains_key(&i.data) {
            glyph_map.insert(i.ch as u16, *assigned.get(&i.data).unwrap());
        } else {
            let slot = available.pop().expect("Ran out of glyph slots!!");
            glyph_map.insert(i.ch as u16, slot);
            glyph_planes[slot.0][slot.1] = i.data;
            assigned.insert(i.data, slot);
        }
    }
    println!("Remaining glyph slots: {}", available.len());

    // Interlace planes into something the GBA can use.
    let mut data = [0u8; (256 * 8 * 8 * 4) / 8];
    for plane in 0..4 {
        for char in 0..256 {
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
    GlyphData { data, low_plane: low_plane_table, glyph_map }
}

macro_rules! to_array {
    ($target:ident, $data:expr, $format:expr, $count:expr) => {
        let $target = {
            let mut $target = String::new();
            for (i, byte) in $data.iter().enumerate() {
                if i % $count == 0 {
                    $target.push_str("    ");
                }
                $target.push_str(&format!($format, byte));
                if i % $count == $count - 1 {
                    $target.push_str("\n");
                }
            }
            if $data.len() % $count == $count - 1 {
                $target.pop();
            }
            $target
        };
    };
}
fn make_glyphs_file(glyphs: GlyphData) {
    // Creates the data table
    to_array!(data_str, glyphs.data, "0x{:02x},", 22);

    // Creates the low plane table bitset
    let mut low_plane = [0u16; 16];
    for (i, is_low_glyph) in glyphs.low_plane.iter().enumerate() {
        if *is_low_glyph {
            low_plane[i >> 4] |= 1 << (i % 16);
        }
    }
    to_array!(low_plane_str, low_plane, "0x{:04x},", 22);

    // Compute the raw PHF for the high planes
    let entries: Vec<_> = glyphs.glyph_map.keys().cloned().collect();
    let phf = lgba_phf::generator::generate_hash(&entries);
    let phf_code = phf.generate_rust_code("lookup_glyph", "u16");

    // Build the PHF glyph data
    let mut glyph_check = [0u16; 1024];
    let mut glyph_id_hi = [0u16; 1024 / 8];
    let mut glyph_id_lo = [0u8; 1024];
    for (i, map) in phf.map.iter().enumerate() {
        if *map == !0 {
            continue;
        }

        let glyph = entries[*map];
        glyph_check[i] = glyph;
        let (hi, lo) = glyphs.glyph_map.get(&glyph).unwrap();
        glyph_id_hi[i / 8] |= (*hi << (2 * (i % 8))) as u16;
        glyph_id_lo[i] = *lo as u8;
    }
    to_array!(glyph_check_str, glyph_check, "0x{:04x},", 16);
    to_array!(glyph_id_hi_str, glyph_id_hi, "0x{:04x},", 16);
    to_array!(glyph_id_lo_str, glyph_id_lo, "0x{:02x},", 22);

    // Find the replacement glyph
    let (replacement_hi, replacement_lo) = glyphs.glyph_map[&(FALLBACK_CHARACTER as u16)];

    // Write the font data source file.
    let raw_source = format!(
        "\
            // This file is generated by the `build_codegen` script found in the repository root.\n\
            // Do not edit it manually.\n\
            \n\
            // Data is based on a subset of the following fonts:\n\
            // - Unscii 2.0 (http://viznut.fi/unscii/)\n\
            // - Misaki Font Gothic 2 (https://littlelimit.net/misaki.htm)\n\
            // \n\
            // Both are released under the public domain.\n\
            \n\
            pub static REPLACEMENT_GLYPH: (u8, u8) = ({replacement_hi}, {replacement_lo});
            \n\
            pub static LOW_PLANE_BITMAP: [u16; 16] = [\n\
                {low_plane_str}\n\
            ];\n\
            pub static GLYPH_CHECK: [u16; 1024] = [\n\
                {glyph_check_str}\n\
            ];\n\
            pub static GLYPH_ID_HI: [u16; 1024/8] = [\n\
                {glyph_id_hi_str}\n\
            ];\n\
            pub static GLYPH_ID_LO: [u8; 1024] = [\n\
                {glyph_id_lo_str}\n\
            ];\n\
            \n\
            #[repr(align(4))]\n\
            struct Align([u8; 1024 * 8]);\n\
            static RAW_FONT_DATA: Align = Align([\n\
                {data_str}\n\
            ]);\n\
            pub static FONT_DATA: &[u8; 1024 * 8] = &RAW_FONT_DATA.0;\n\
            \n\
            pub {phf_code}\
        "
    );
    File::create("../lgba/src/terminal/font_data.rs")
        .unwrap()
        .write_all(raw_source.as_bytes())
        .unwrap();
}

fn main() {
    let characters = download_fonts().expect("Could not download and parse fonts.");
    let character_list = gather_characters(characters);
    let glyphs = build_planes(character_list);
    make_glyphs_file(glyphs)
}
