use anyhow::*;
use std::{cmp::max, hash::Hash, io::Cursor};
use zip::ZipArchive;

// Download URLs for the source fonts
const UNSCII_FONT_URL: &str = "http://viznut.fi/unscii/unscii-8.hex";
const MISAKI_FONT_URL: &str = "https://littlelimit.net/arc/misaki/misaki_bdf_2021-05-05.zip";

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug, Hash)]
pub struct CharacterInfo {
    pub ch: char,
    pub data: u64,
}

pub struct CharacterSets {
    pub unscii: Vec<CharacterInfo>,
    pub misaki: Vec<CharacterInfo>,
}

pub fn download_fonts() -> Result<CharacterSets> {
    let mut characters = CharacterSets { unscii: Vec::new(), misaki: Vec::new() };

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
        if ch != '\0' {
            characters.unscii.push(CharacterInfo { ch, data });
        }
    }

    // download and parse misaki font
    let misaki_font_zip = reqwest::blocking::get(MISAKI_FONT_URL)?.bytes()?.to_vec();
    let mut zip_reader = ZipArchive::new(Cursor::new(misaki_font_zip))?;
    let misaki_font = bdf::read(zip_reader.by_name("misaki_gothic_2nd.bdf")?)?;

    // add characters from misaki font
    for (char, glyph) in misaki_font.glyphs() {
        // compute the bounds of the glyph
        let x_off = glyph.bounds().x as u32;
        let y_off = if glyph.height() != 8 {
            (8 - glyph.height()) - 1 - (max(0, glyph.bounds().y) as u32)
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
        characters
            .misaki
            .push(CharacterInfo { ch: *char, data: glyph_data });
    }

    // return the downloaded characters
    characters.unscii.sort_by_key(|x| x.ch as u32);
    characters.misaki.sort_by_key(|x| x.ch as u32);
    Ok(characters)
}
