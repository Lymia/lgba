use crate::build_fonts::{
    font_data::{data_is_half_width, load_fonts, CharacterInfo, CharacterSets},
    FontConfig,
};
use kanji::Level;
use lgba_phf::generator::SpecialTy;
use proc_macro2::{Literal, TokenStream};
use quote::quote;
use std::collections::{BTreeSet, HashMap, HashSet};
use syn::Result;
use unic_ucd_bidi::BidiClass;
use unic_ucd_block::Block;
use unic_ucd_common::is_control;
use unic_ucd_normal::is_combining_mark;

#[derive(Clone, Debug)]
enum DecodedMap {
    Contents(BTreeSet<&'static str>),
    Wildcard,
}
impl DecodedMap {
    fn from_list(list: Vec<&'static str>) -> Self {
        let mut set = BTreeSet::new();
        for i in list {
            if i == "*" {
                return DecodedMap::Wildcard;
            }
            set.insert(i);
        }
        DecodedMap::Contents(set)
    }
    fn contains(&self, value: &str) -> bool {
        match self {
            DecodedMap::Contents(map) => map.contains(value),
            DecodedMap::Wildcard => true,
        }
    }
}

#[derive(Clone, Debug)]
struct DecodedFontConfig {
    low_plane_limit: usize,
    disable_unscii: DecodedMap,
    disable_misaki: DecodedMap,
    chars: HashSet<char>,
    block: DecodedMap,
    allow_halfwidth_blocks: DecodedMap,
    fallback_char: char,
    kanji_max_level: Level,
    delta: f32,
}
impl DecodedFontConfig {
    fn from_config(config: FontConfig) -> Result<DecodedFontConfig> {
        let mut whitelisted_chars = HashSet::new();
        for string in config.chars {
            for char in string.chars() {
                whitelisted_chars.insert(char);
            }
        }

        Ok(DecodedFontConfig {
            low_plane_limit: config.low_plane_limit.unwrap_or(0x100),
            disable_unscii: DecodedMap::from_list(config.disable_unscii),
            disable_misaki: DecodedMap::from_list(config.disable_misaki),
            chars: whitelisted_chars,
            block: DecodedMap::from_list(config.block),
            allow_halfwidth_blocks: DecodedMap::from_list(config.allow_halfwidth_blocks),
            fallback_char: config.fallback_char.unwrap_or('?'),
            kanji_max_level: match config.kanji_max_level.unwrap_or_else(|| "10") {
                "10" | "Ten" | "ten" => Level::Ten,
                "9" | "Nine" | "nine" => Level::Nine,
                "8" | "Eight" | "eight" => Level::Eight,
                "7" | "Seven" | "seven" => Level::Seven,
                "6" | "Six" | "six" => Level::Six,
                "5" | "Five" | "five" => Level::Five,
                "4" | "Four" | "four" => Level::Four,
                "3" | "Three" | "three" => Level::Three,
                "PreTwo" | "pretwo" => Level::PreTwo,
                "2" | "Two" | "two" => Level::Two,
                "PreOne" | "preone" => Level::PreOne,
                "1" | "One" | "one" => Level::One,
                x => panic!("'{}' is not a valid kanji max level.", x),
            },
            delta: config.delta.unwrap_or(1.0),
        })
    }
}

fn block_name(ch: char) -> &'static str {
    match Block::of(ch) {
        None => "Unknown Block",
        Some(block) => block.name,
    }
}
fn process_char(config: &DecodedFontConfig, mut char: CharacterInfo) -> CharacterInfo {
    if !config.allow_halfwidth_blocks.contains(&block_name(char.ch)) {
        char.is_half_width = false;
    }
    char
}

const TECHNICAL_CHAR_BLANK: u64 = 0;
const TECHNICAL_CHAR_HALF: u64 = 0xF0F0F0F0F0F0F0F0;
const TECHNICAL_CHAR_FULL: u64 = !0;

fn is_technical_char(data: u64) -> bool {
    data == TECHNICAL_CHAR_BLANK || data == TECHNICAL_CHAR_FULL || data == TECHNICAL_CHAR_FULL
}
fn is_lgba_private_use(ch: char) -> bool {
    let ch = ch as u32;
    (ch >= 0xF400 && ch < 0xF480) || (ch >= 0xF500 && ch < 0xF510)
}

fn build_from_fonts(config: &DecodedFontConfig, characters: &CharacterSets) -> Vec<CharacterInfo> {
    let mut char_map = HashMap::new();

    // add characters from unscii
    for char in &characters.unscii {
        let is_disabled = !config.disable_unscii.contains(block_name(char.ch));
        if char.ch != '\0' && is_disabled {
            char_map.insert(char.ch, process_char(config, *char));
        }
    }

    // add characters from misaki
    for char in &characters.misaki {
        let is_disabled = !config.disable_misaki.contains(block_name(char.ch));
        if char.ch != '\0' && is_disabled && !char_map.contains_key(&char.ch) {
            char_map.insert(char.ch, process_char(config, *char));
        }
        if (char.ch as u32) < 0x80 {
            let new_ch = char::from_u32(0xF400 + char.ch as u32).unwrap();
            let mut new_char = *char;
            new_char.ch = new_ch;
            char_map.insert(new_ch, new_char);
        }
    }

    // add space characters
    char_map.insert(
        ' ',
        process_char(config, CharacterInfo {
            ch: ' ',
            data: TECHNICAL_CHAR_BLANK,
            is_half_width: true,
        }),
    );
    char_map.insert('\u{F420}', CharacterInfo {
        ch: '\u{F420}',
        data: TECHNICAL_CHAR_BLANK,
        is_half_width: true,
    });

    // return the downloaded characters
    let mut characters: Vec<_> = char_map.values().cloned().collect();
    characters.sort_by_key(|x| x.ch as u32);
    characters
}

struct CharacterData {
    characters: Vec<CharacterInfo>,
    glyph_count: usize,
}

fn filter_characters(config: &DecodedFontConfig, characters: Vec<CharacterInfo>) -> CharacterData {
    let kanji_level = kanji::level_table();

    let mut filtered_characters = Vec::new();
    let mut glyphs = HashSet::new();
    glyphs.insert(0);

    for char in characters {
        if config.chars.contains(&char.ch)
            || is_lgba_private_use(char.ch)
            || (config.block.contains(block_name(char.ch))
                && !is_control(char.ch)
                && !is_combining_mark(char.ch)
                && !BidiClass::of(char.ch).is_rtl()
                && (char.ch as u32) < 0x10000)
        {
            let is_kanji_too_advanced = if let Some(kanji) = kanji::Kanji::new(char.ch) {
                if let Some(level) = kanji_level.get(&kanji) {
                    *level > config.kanji_max_level
                } else {
                    config.kanji_max_level != Level::One
                }
            } else {
                false
            };

            if !is_kanji_too_advanced {
                filtered_characters.push(char);
                glyphs.insert(char.data);
            }
        }
    }

    filtered_characters.sort_by_key(|x| x.ch as u32);

    CharacterData { characters: filtered_characters, glyph_count: glyphs.len() }
}

struct GlyphData {
    tile_count: usize,
    data: Vec<u8>,
    low_plane: Vec<bool>,
    low_plane_half_width: Vec<bool>,
    glyph_map: HashMap<u16, (usize, usize, bool)>,
    glyph_lookup: HashMap<u16, (usize, usize, bool)>,
}

struct GlyphPlaneBuilder {
    tile_count: usize,

    low_plane_table: Vec<bool>,
    low_plane_half_width: Vec<bool>,
    low_plane_assigned: HashSet<(usize, usize)>,

    available: Vec<(usize, usize)>,
    available_half: Vec<(usize, usize)>,
    technical_reserved: Vec<usize>,

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

    fn split_plane(&self, id: char) -> (usize, usize) {
        assert!((id as usize) < self.low_plane_table.len());
        (id as usize % 4, id as usize / 4)
    }

    fn try_insert_low_plane(&mut self, i: &CharacterInfo) {
        if self.glyph_lookup.contains_key(&(i.ch as u16)) {
            return;
        }

        let (plane, char) = self.split_plane(i.ch);
        let low_plane_valid = !self.low_plane_assigned.contains(&(plane, char))
            || self.glyph_planes[plane][char] == i.data;
        if !low_plane_valid
            || (self.glyph_assigned.contains_key(&i.data)
                && (i.ch as usize) >= self.low_plane_table.len())
            || is_technical_char(i.data)
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
        self.set_plane_width(
            char,
            i.data,
            i.is_half_width || self.glyph_needs_half.contains(&i.data),
        );

        if !self.glyph_assigned.contains_key(&i.data) {
            self.glyph_assigned.insert(i.data, (plane, char));
        } else {
            self.dupe_low += 1;
        }
    }

    fn find_available(&mut self) {
        let mut technical_reserved = 0;
        for char in 0..self.tile_count / 4 {
            let mut any_assigned = false;
            for plane in 0..4 {
                if self.low_plane_assigned.contains(&(plane, char)) {
                    any_assigned = true;
                }
            }

            if !any_assigned && technical_reserved != 3 {
                self.technical_reserved.push(char);
                technical_reserved += 1;
            } else {
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
        }
        if technical_reserved != 3 {
            panic!("Could not reserve sufficient technical character slots!");
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

    fn assign_technical(&mut self, ch: char, data: Option<u64>, plane: usize, char: usize) {
        self.glyph_map.insert(ch as u16, (plane, char, false));
        self.glyph_lookup.insert(ch as u16, (plane, char, false));
        if let Some(data) = data {
            self.glyph_assigned.insert(data, (plane, char));
        }
    }
}

fn build_planes(config: &mut DecodedFontConfig, ch_data: CharacterData) -> GlyphData {
    let tile_count = ((ch_data.glyph_count + 4 + 15) / 16) * 16;
    if config.low_plane_limit > tile_count {
        config.low_plane_limit = tile_count;
    }

    let dupe_count = {
        let mut ch_count = 0;
        let mut low_plane_dupe_check = HashSet::new();
        for ch in &ch_data.characters {
            if ch.ch as usize > config.low_plane_limit {
                break;
            }
            low_plane_dupe_check.insert(ch.data);
            ch_count += 1;
        }
        ch_count - low_plane_dupe_check.len()
    };
    let tile_count = ((ch_data.glyph_count + dupe_count + 4 + 15) / 16) * 16 + 16;
    assert!(tile_count <= 4096);

    // create a new glyph builder
    let mut builder = GlyphPlaneBuilder {
        tile_count,
        low_plane_table: vec![false; config.low_plane_limit],
        low_plane_half_width: vec![false; config.low_plane_limit],
        low_plane_assigned: Default::default(),
        available: vec![],
        available_half: vec![],
        technical_reserved: vec![],
        glyph_planes: vec![vec![0u64; tile_count / 4]; 4],
        glyph_map: Default::default(),
        glyph_lookup: Default::default(),
        char_is_half: vec![None; tile_count / 4],
        glyph_needs_half: Default::default(),
        glyph_assigned: Default::default(),
        dupe_low: 0,
    };

    // preprocess glyphs
    for i in &ch_data.characters {
        builder.preprocess_glyph(i);
    }

    // assign low plane
    for i in &ch_data.characters {
        if i.ch as usize >= config.low_plane_limit {
            break;
        }
        if builder.glyph_needs_half.contains(&i.data) {
            builder.try_insert_low_plane(i);
        }
    }
    for i in &ch_data.characters {
        if i.ch as usize >= config.low_plane_limit {
            break;
        }
        builder.try_insert_low_plane(i);
    }

    // build table of available glyph locations
    builder.find_available();

    // assign technical characters to the glyph planes
    let technical_full = builder.technical_reserved.pop().unwrap();
    builder.glyph_planes[0][technical_full] = TECHNICAL_CHAR_FULL;
    builder.assign_technical('\u{F500}', Some(TECHNICAL_CHAR_FULL), 0, technical_full);
    builder.assign_technical('\u{F501}', None, 1, technical_full);

    let technical_half = builder.technical_reserved.pop().unwrap();
    builder.glyph_planes[0][technical_half] = TECHNICAL_CHAR_HALF;
    builder.assign_technical('\u{F504}', Some(TECHNICAL_CHAR_HALF), 0, technical_half);
    builder.assign_technical('\u{F505}', None, 1, technical_half);

    let technical_blank = builder.technical_reserved.pop().unwrap();
    builder.assign_technical('\u{F508}', Some(TECHNICAL_CHAR_BLANK), 0, technical_blank);

    // assign remaining characters to the glyph planes
    for i in &ch_data.characters {
        builder.try_assign_character(i);
    }

    // Interlace planes into something the GBA can use.
    let mut data = vec![0u8; ((tile_count / 4) * 8 * 8 * 4) / 8];
    for plane in 0..4 {
        for char in 0..tile_count / 4 {
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
    for i in &ch_data.characters {
        let glyph = builder.glyph_lookup.get(&(i.ch as u16)).unwrap();
        assert_eq!(i.data, builder.glyph_planes[glyph.0][glyph.1]);
        assert_eq!(i.is_half_width, glyph.2);
    }

    // Returns the glyph data
    GlyphData {
        tile_count,
        data,
        low_plane: builder.low_plane_table,
        low_plane_half_width: builder.low_plane_half_width,
        glyph_map: builder.glyph_map,
        glyph_lookup: builder.glyph_lookup,
    }
}

fn make_u8_literal(data: &[u8]) -> TokenStream {
    let literal_data = Literal::byte_string(data);
    quote! { *#literal_data }
}
fn make_u16_literal(data: &[u16]) -> TokenStream {
    quote! { [#(#data,)*] }
}
fn make_u32_data_literal(data: &[u8]) -> TokenStream {
    assert_eq!(data.len() % 4, 0);
    let literal_data = Literal::byte_string(data);
    let data_len = data.len() / 4;
    quote! { crate::__macro_export::xfer_u8_u32::<#data_len>(#literal_data) }
}

fn make_glyphs_file(
    config: &DecodedFontConfig,
    glyphs: GlyphData,
    target_ty: TokenStream,
) -> TokenStream {
    // Creates the low plane table bitset
    let mut low_plane = vec![0u16; config.low_plane_limit / 16];
    for (i, is_low_glyph) in glyphs.low_plane.iter().enumerate() {
        if *is_low_glyph {
            low_plane[i >> 4] |= 1 << (i % 16);
        }
    }

    // Creates the low plane half-width table bitset
    let mut low_plane_half = vec![0u16; config.low_plane_limit / 16];
    for (i, is_half) in glyphs.low_plane_half_width.iter().enumerate() {
        if *is_half {
            low_plane_half[i >> 4] |= 1 << (i % 16);
        }
    }
    if low_plane_half.iter().all(|x| *x == 0) {
        low_plane_half.clear();
    }

    // Compute the raw PHF for the high planes
    let entries: Vec<_> = glyphs.glyph_map.keys().cloned().collect();
    let phf = lgba_phf::generator::generate_hash(config.delta, &entries);
    let phf_func = phf.generate_syn_code(
        quote! { lookup_glyph },
        quote! { &u16 },
        quote! { lgba_phf },
        Some(SpecialTy::U16),
    );

    // Build the PHF glyph data
    let glyph_size = (glyphs.glyph_map.len() - 1).next_power_of_two();
    let glyph_char_bits = (glyphs.tile_count / 4 - 1)
        .next_power_of_two()
        .trailing_zeros();
    let (hi_bits, divisor) = match glyph_char_bits + 3 {
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

        let (plane, char, is_half) = glyphs.glyph_map.get(&glyph).unwrap();
        let meta = ((*is_half as usize) << 2) | *plane;
        let packed = (meta << glyph_char_bits) | *char;

        if hi_bits != 0 {
            let hi = (packed >> 8) as u16;
            glyph_id_hi[i / divisor] |= hi << (hi_bits * (i % divisor));
        }
        glyph_id_lo[i] = packed as u8;
    }

    // Find the replacement glyph
    let (fb_hi, fb_lo, fb_half) = glyphs.glyph_lookup[&(config.fallback_char as u16)];

    // Calculate statistics
    let bytes = glyphs.data.len()
        + low_plane.len() * 2
        + low_plane_half.len() * 2
        + glyph_check.len() * 2
        + glyph_id_hi.len() * 2
        + glyph_id_lo.len()
        + phf.disps.len() * 2;
    let kib = (bytes as f32) / 1024.0;

    // Build the documentation string.
    let mut documentation = String::new();
    documentation.push_str(&format!(
        "The data files for this font require {kib:.2} KiB of ROM space, not including any code \
         specific to this font that may be generated.\n\n",
    ));

    if let DecodedMap::Contents(blocks) = &config.block {
        documentation.push_str("# Available characters\n\n");
        documentation.push_str("The following Unicode blocks are available in this font:\n\n");
        for block in blocks {
            documentation.push_str(&format!("* {block}\n"));
        }
        documentation.push('\n');

        if !config.chars.is_empty() {
            documentation.push_str("The following additional characters are available:\n* ");

            let mut whitelisted_chars: Vec<_> = config.chars.iter().cloned().collect();
            whitelisted_chars.sort();
            for char in whitelisted_chars {
                documentation.push_str(&format!("`{char}`, "))
            }
            documentation.pop();
            documentation.pop();
            documentation.push('\n');
        }
    }

    // Create the new implementation for the type
    let lo_map_data = make_u16_literal(&low_plane);
    let lo_map_len = low_plane.len();
    let lo_map_half_data = make_u16_literal(&low_plane_half);
    let lo_map_half_len = low_plane_half.len();
    let lo_map_size = config.low_plane_limit;

    let glyph_check_data = make_u16_literal(&glyph_check);
    let glyph_id_lo_data = make_u8_literal(&glyph_id_lo);

    let font_data = make_u32_data_literal(&glyphs.data);
    let font_data_size = glyphs.tile_count * 2;

    let (load_hi_defines, load_hi) = if hi_bits != 0 {
        let glyph_id_hi_data = make_u16_literal(&glyph_id_hi);
        let divisor_mask = divisor - 1;
        let divisor_shift = divisor.trailing_zeros();

        (
            quote! {
                static GLYPH_ID_HI: [u16; #hi_arr_size] = #glyph_id_hi_data;
                const HI_MASK: u16 = (1 << #hi_bits) - 1;
            },
            quote! {
                let word = GLYPH_ID_HI[slot >> #divisor_shift];
                let hi = (word >> (#hi_bits * (slot & #divisor_mask))) & HI_MASK;
                let packed = (hi << 8) | (GLYPH_ID_LO[slot] as u16);
            },
        )
    } else {
        (quote! {}, quote! {
            let packed = GLYPH_ID_LO[slot] as u16;
        })
    };
    let impl_content = quote! {
        const FALLBACK_GLYPH: (u8, u16, bool) = (#fb_hi as u8, #fb_lo as u16, #fb_half);
        static LO_MAP_DATA: [u16; #lo_map_len] = #lo_map_data;
        static LO_MAP_HALF_DATA: [u16; #lo_map_half_len] = #lo_map_half_data;
        static GLYPH_CHECK: [u16; #glyph_size] = #glyph_check_data;
        static GLYPH_ID_LO: [u8; #glyph_size] = #glyph_id_lo_data;
        static FONT_DATA: [u32; #font_data_size] = #font_data;

        #load_hi_defines
        #phf_func

        const CHAR_MASK: u16 = (1 << #glyph_char_bits) - 1;

        fn get_font_glyph_phf(id: usize) -> (u8, u16, bool) {
            let slot = lookup_glyph(&(id as u16));
            if id == GLYPH_CHECK[slot] as usize {
                #load_hi

                let plane = ((packed >> #glyph_char_bits) & 3) as u8;
                let char = packed & CHAR_MASK;
                let is_half = ((packed >> (#glyph_char_bits + 2)) & 1) as u8;
                (plane, char, is_half != 0)
            } else {
                FALLBACK_GLYPH
            }
        }
        #[inline(never)]
        fn get_font_glyph(id: char) -> (u8, u16, bool) {
            let id = id as usize;
            if id < #lo_map_size {
                // We check the low plane bitmap to see if we have this glyph.
                let word = LO_MAP_DATA[id >> 4];
                if word & (1 << (id & 15)) != 0 {
                    let target_i = id >> 4;
                    let half_word = if target_i < LO_MAP_HALF_DATA.len() {
                        LO_MAP_HALF_DATA[target_i]
                    } else {
                        0
                    };
                    let is_half = half_word & (1 << (id & 15)) != 0;
                    ((id & 3) as u8, (id >> 2) as u16, is_half)
                } else {
                    // half-width shenanigans can cause this to be in the PHF instead
                    get_font_glyph_phf(id)
                }
            } else if id < 0x10000 {
                // Check the PHF to see if we have this glyph.
                get_font_glyph_phf(id)
            } else {
                // We only support the BMP, don't bother.
                FALLBACK_GLYPH
            }
        }

        #[doc = #documentation]
        impl crate::display::TerminalFont for #target_ty {
            fn get_font_glyph(id: char) -> (u8, u16, bool) {
                get_font_glyph(id)
            }
            fn get_font_data() -> &'static [u32] {
                &FONT_DATA
            }
        }
    };

    if std::env::var("LGBA_DUMP_FONT_DATA").is_ok() {
        std::fs::write(format!("font_data_{target_ty}.bin"), &glyphs.data).unwrap();
    }

    quote! {
        const _: () = {
            #impl_content
            ()
        };
    }
}

pub fn generate_fonts(config: FontConfig, target_ty: TokenStream) -> Result<TokenStream> {
    let characters = load_fonts();

    let mut config = DecodedFontConfig::from_config(config)?;
    let characters = build_from_fonts(&config, &characters);
    let character_list = filter_characters(&config, characters);
    let glyphs = build_planes(&mut config, character_list);
    let tokens = make_glyphs_file(&config, glyphs, target_ty);

    Ok(tokens)
}
