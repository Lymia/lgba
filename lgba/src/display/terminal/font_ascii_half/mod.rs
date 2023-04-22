// This file is generated by the `build_codegen` script found in the repository root.
// Do not edit it manually.

use super::*;

const FALLBACK_GLYPH: (u8, u16, bool) = (3, 15, false);
static LO_MAP_DATA: [u16; 6] = *include_u16!("lo_map.bin");
static GLYPH_CHECK: [u16; 32] = *include_u16!("glyph_check.bin");
static GLYPH_ID_LO: [u8; 32] = *include_u8!("glyph_id_lo.bin");
static FONT_DATA: [u32; 96 * 2] = *include_u32!("font.chr");

fn lookup_glyph(value: &u16) -> usize {
    const KEY: u32 = 0x2d2a7a82;
    const DISPS: [u16; 4] = *include_u16!("phf_disps.bin");
    lgba_phf::hash::<4, 32, _>(KEY, &DISPS, value)
}

/// An 8x4 terminal font supporting only 7-bit ASCII characters.
/// 
/// The data files for this font require 0.86 KiB of ROM space, not including
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
pub struct TerminalFontAsciiHalf(());

const CHAR_MASK: u16 = (1 << 5) - 1;
fn get_font_glyph(id: char) -> (u8, u16, bool) {
    let id = id as usize;
    if id < 96 {
        // We check the low plane bitmap to see if we have this glyph.
        let word = LO_MAP_DATA[id >> 4];
        if word & (1 << (id & 15)) != 0 {
            ((id & 3) as u8, (id >> 2) as u16, false)
        } else {
            FALLBACK_GLYPH
        }
    } else if id < 0x10000 {
        // Check the PHF to see if we have this glyph.
        let slot = lookup_glyph(&(id as u16));
        if id == GLYPH_CHECK[slot] as usize {
            let packed = GLYPH_ID_LO[slot] as u16;
            ((packed >> 5) as u8, packed & CHAR_MASK, false)
        } else {
            FALLBACK_GLYPH
        }
    } else {
        // We only support the BMP, don't bother.
        FALLBACK_GLYPH
    }
}

impl TerminalFont for TerminalFontAsciiHalf {
    fn instance() -> &'static Self {
        &TerminalFontAsciiHalf(())
    }
    fn get_font_glyph(&self, id: char) -> (u8, u16, bool) {
        get_font_glyph(id)
    }
    fn get_font_data(&self) -> &'static [u32] {
        &FONT_DATA
    }
    fn has_half_width(&self) -> bool {
        true
    }
}