mod font_data;

pub fn get_font_glyph(id: char) -> (u8, u8) {
    let id = id as usize;
    if id < 256 {
        // We check the low plane bitmap to see if we have this glyph.
        let word = font_data::LOW_PLANE_BITMAP[id >> 4];
        if word & 1 << (id & 15) != 0 {
            (0, id as u8)
        } else {
            font_data::REPLACEMENT_GLYPH
        }
    } else if id < 0x10000 {
        // Check the PHF to see if we have this glyph.
        let slot = font_data::lookup_glyph(&(id as u16));
        if id == font_data::GLYPH_CHECK[slot] as usize {
            let word = font_data::GLYPH_ID_HI[slot >> 3];
            let hi = (word >> (2 * (slot & 7))) & 3;
            let lo = font_data::GLYPH_ID_LO[slot];
            (hi as u8, lo)
        } else {
            font_data::REPLACEMENT_GLYPH
        }
    } else {
        // We only support the BMP, don't bother.
        font_data::REPLACEMENT_GLYPH
    }
}
