use anyhow::*;
use std::{cmp::max, fs::File, path::PathBuf};

fn convert_bdf() -> Result<()> {
    let mut directory = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    directory.push("src/build_fonts/font_data");

    // load the BDF version
    directory.push("misaki_gothic_2nd.bdf");
    let misaki_font = bdf::read(File::open(&directory)?)?;
    directory.pop();

    // convert the misaki font to a UNSCII-like u64 format
    let mut accum = String::new();
    let mut vec: Vec<_> = misaki_font.glyphs().iter().map(|x| (*x.0, x.1)).collect();
    vec.sort_by_key(|x| x.0);
    for (ch, glyph) in vec {
        // compute the bounds of the glyph
        let x_off = glyph.bounds().x as u32;
        let y_off = if glyph.height() != 8 {
            (8 - glyph.height()) - 1 - (max(0, glyph.bounds().y) as u32)
        } else {
            0
        };

        // copy the glyph to a `u64` format
        let mut data = 0u64;
        for x in 0..glyph.width() {
            for y in 0..glyph.height() {
                let tx = x + x_off;
                let ty = y + y_off;
                data |= (glyph.get(x, y) as u64) << (63 - (tx + ty * 8));
            }
        }

        // add the glyph to the character map
        accum.push_str(&format!("{:05X}:{data:016X}\n", ch as u32));
    }

    // output the .hex version
    directory.push("misaki_gothic_2nd.hex");
    std::fs::write(directory, accum)?;

    Ok(())
}

fn main() {
    convert_bdf().unwrap()
}
