// This file is generated by the `build_codegen` script found in the repository root.
// Do not edit it manually.

use super::*;

const REPLACEMENT_GLYPH: (u8, u16) = (0, 519);
static LO_MAP_DATA: [u16; 64] = *include_u16!("lo_map.bin");
static GLYPH_CHECK: [u16; 4096] = *include_u16!("glyph_check.bin");
static GLYPH_ID_HI: [u16; 1024] = *include_u16!("glyph_id_hi.bin");
static GLYPH_ID_LO: [u8; 4096] = *include_u8!("glyph_id_lo.bin");
static FONT_DATA: [u32; 3968 * 2] = *include_u32!("font.chr");

fn lookup_glyph(value: &u16) -> usize {
    const KEY: u32 = 0x499602d2;
    const DISPS: [u16; 512] = *include_u16!("phf_disps.bin");
    lgba_phf::hash::<512, 4096, _>(KEY, &DISPS, value)
}

/// A terminal font supporting most characters from the source fonts.
/// 
/// Only kanji on the jouyou list are included. This font is not suited for rendering Chinese or Korean text.
/// 
/// The data files for this font require 46.12 KiB of ROM space, not including
/// any font-specific code that may be generated.
/// 
/// # Licencing
/// 
/// The data of this font is based on a combined subset of the following fonts:
/// 
/// * [Unscii 2.0](http://viznut.fi/unscii/)
/// * [Misaki Font Gothic 2](https://littlelimit.net/misaki.htm)
/// 
/// Both are released under the public domain.
/// 
/// # Available Characters
/// 
/// The following Unicode blocks are available in this font:
/// 
/// * Arrows
/// * Basic Latin
/// * Block Elements
/// * Box Drawing
/// * CJK Compatibility
/// * CJK Symbols and Punctuation
/// * CJK Unified Ideographs
/// * Currency Symbols
/// * Cyrillic
/// * Dingbats
/// * Enclosed Alphanumerics
/// * Enclosed CJK Letters and Months
/// * General Punctuation
/// * Geometric Shapes
/// * Greek Extended
/// * Greek and Coptic
/// * Halfwidth and Fullwidth Forms
/// * Hiragana
/// * IPA Extensions
/// * Katakana
/// * Latin Extended Additional
/// * Latin Extended-A
/// * Latin Extended-B
/// * Latin-1 Supplement
/// * Letterlike Symbols
/// * Mathematical Operators
/// * Miscellaneous Mathematical Symbols-B
/// * Miscellaneous Symbols
/// * Miscellaneous Symbols and Arrows
/// * Miscellaneous Technical
/// * Number Forms
/// * Runic
/// * Spacing Modifier Letters
/// * Superscripts and Subscripts
/// * Supplemental Punctuation
/// * Unified Canadian Aboriginal Syllabics
pub struct TerminalFontFull(());

fn get_font_glyph(id: char) -> (u8, u16) {
    let id = id as usize;
    if id < 1024 {
        // We check the low plane bitmap to see if we have this glyph.
        let word = LO_MAP_DATA[id >> 4];
        if word & 1 << (id & 15) != 0 {
            ((id & 3) as u8, (id >> 2) as u16)
        } else {
            REPLACEMENT_GLYPH
        }
    } else if id < 0x10000 {
        // Check the PHF to see if we have this glyph.
        let slot = lookup_glyph(&(id as u16));
        if id == GLYPH_CHECK[slot] as usize {
            let hi_mask = (1 << 4) - 1;
            let char_mask = (1 << 10) - 1;
            
            let word = GLYPH_ID_HI[slot >> 3];
            let hi = (word >> (4 * (slot & 3))) & hi_mask;
            let lo = GLYPH_ID_LO[slot];
            let packed = (hi << 8) | (lo as u16);
            ((packed >> 10) as u8, packed & char_mask)
        } else {
            REPLACEMENT_GLYPH
        }
    } else {
        // We only support the BMP, don't bother.
        REPLACEMENT_GLYPH
    }
}

impl TerminalFont for TerminalFontFull {
    fn instance() -> &'static Self {
        &TerminalFontFull(())
    }
    fn get_font_glyph(&self, id: char) -> (u8, u16) {
        get_font_glyph(id)
    }
    fn get_font_data(&self) -> &'static [u32] {
        &FONT_DATA
    }
}
