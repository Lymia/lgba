//! Contains the code needed to generate a GBA header.

#![allow(missing_docs)]

pub type GbaHeader = [u8; 0x20];

#[rustfmt::skip]
pub const GBA_HEADER_TEMPLATE: GbaHeader = [
    // Game Title
    b'L', b'G', b'B', b'A', b'_', b'R', b'O', b'M', b' ', b' ', b' ', b' ',
    // Game code
    b'L', b'G', b'B', b'A',
    // Maker code
    b'0', b'0',
    // Fixed value
    0x96,
    // Main unit code
    0,
    // Device type
    0,
    // Reserved area
    0, 0, 0, 0, 0, 0, 0,
    // Software version
    0,
    // Complement check
    0,
    // Reserved area
    0, 0,
];

const fn byte_to_upper(i: u8, to_upper: bool) -> u8 {
    match i {
        b'a'..=b'z' if to_upper => i - b'a' + b'A',
        x => x,
    }
}

pub const fn set_header_field(
    mut header: GbaHeader,
    str: &str,
    off: usize,
    len: usize,
    to_upper: bool,
) -> GbaHeader {
    let title_bytes = str.as_bytes();
    let mut copy_len = title_bytes.len();
    if copy_len > len {
        copy_len = len;
    }

    let mut i = 0;
    while i < copy_len {
        header[off + i] = byte_to_upper(title_bytes[i], to_upper);
        i += 1;
    }

    header
}
pub const fn calculate_complement(mut header: GbaHeader) -> GbaHeader {
    let mut i = 0;
    let mut c = 0;
    while i < 0xBD - 0xA0 {
        c += header[i] as i32;
        i += 1;
    }

    header[0x1D] = (-(0x19 + c)) as u8;
    header
}
