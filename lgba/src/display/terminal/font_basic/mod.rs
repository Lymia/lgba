// This file is generated by the `build_codegen` script found in the repository root.
// Do not edit it manually.

use super::*;

const REPLACEMENT_GLYPH: (u8, u16) = (1, 105);
static LO_MAP_DATA: [u16; 48] = *include_u16!("lo_map.bin");
static GLYPH_CHECK: [u16; 512] = *include_u16!("glyph_check.bin");
static GLYPH_ID_HI: [u16; 64] = *include_u16!("glyph_id_hi.bin");
static GLYPH_ID_LO: [u8; 512] = *include_u8!("glyph_id_lo.bin");
static FONT_DATA: [u32; 832 * 2] = *include_u32!("font.chr");

fn lookup_glyph(value: &u16) -> usize {
    const KEY: u32 = 0x499602d2;
    const DISPS: [u16; 64] = *include_u16!("phf_disps.bin");
    lgba_phf::hash::<64, 512, _>(KEY, &DISPS, value)
}

/// An 8x8 basic terminal font supporting many scripts and a limited number of characters useful for rendering menus.
/// 
/// The data files for this font require 8.34 KiB of ROM space, not including
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
/// * Basic Latin
/// * CJK Symbols and Punctuation
/// * Currency Symbols
/// * Cyrillic
/// * Greek and Coptic
/// * Hiragana
/// * IPA Extensions
/// * Katakana
/// * Latin Extended Additional
/// * Latin Extended-A
/// * Latin Extended-B
/// * Latin-1 Supplement
/// * Supplemental Punctuation
///
/// The following additional characters are available:
/// * `①`, `②`, `③`, `④`, `⑤`, `⑥`, `⑦`, `⑧`, `■`, `□`, `●`, `★`, `♪`, `⌚`, `⌛`, `⏩`, `⏪`, `█`, `▉`, `▊`, `▋`, `▌`, `▍`, `▁`, `▂`, `▃`, `▄`, `▅`, `▆`, `▇`, `▎`, `▏`, `─`, `│`, `┌`, `┐`, `└`, `┘`, `├`, `┤`, `┬`, `┴`, `┼`, `←`, `↑`, `→`, `↓`, `↔`, `↕`, `‐`, `‑`, `‒`, `–`, `—`, `―`, `†`, `‡`, `•`, `․`, `…`, `⁇`, `▲`, `▶`, `▼`, `◀`, `▀`, `▐`, `░`, `▒`, `▓`, `○`, `▖`, `▗`, `▘`, `▙`, `▚`, `▛`, `▜`, `▝`, `▞`, `▟`, `▩`, `⌘`, `♀`, `♂`, `╭`, `╮`, `╯`, `╰`
pub struct TerminalFontBasic(());

const HI_MASK: u16 = (1 << 2) - 1;
const CHAR_MASK: u16 = (1 << 8) - 1;
fn get_font_glyph(id: char) -> (u8, u16) {
    let id = id as usize;
    if id < 768 {
        // We check the low plane bitmap to see if we have this glyph.
        let word = LO_MAP_DATA[id >> 4];
        if word & (1 << (id & 15)) != 0 {
            ((id & 3) as u8, (id >> 2) as u16)
        } else {
            REPLACEMENT_GLYPH
        }
    } else if id < 0x10000 {
        // Check the PHF to see if we have this glyph.
        let slot = lookup_glyph(&(id as u16));
        if id == GLYPH_CHECK[slot] as usize {
            let word = GLYPH_ID_HI[slot >> 3];
            let hi = (word >> (2 * (slot & 7))) & HI_MASK;
            let packed = (hi << 8) | (GLYPH_ID_LO[slot] as u16);            ((packed >> 8) as u8, packed & CHAR_MASK)
        } else {
            REPLACEMENT_GLYPH
        }
    } else {
        // We only support the BMP, don't bother.
        REPLACEMENT_GLYPH
    }
}

impl TerminalFont for TerminalFontBasic {
    fn instance() -> &'static Self {
        &TerminalFontBasic(())
    }
    fn get_font_glyph(&self, id: char) -> (u8, u16) {
        get_font_glyph(id)
    }
    fn get_font_data(&self) -> &'static [u32] {
        &FONT_DATA
    }
}
