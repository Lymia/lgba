//! Module containing interfaces to the GBA's graphics chip.

#[macro_use]
mod macros;

mod layers;
mod modes;
mod terminal;
mod vram;

pub use layers::{ActiveTileLayer, ActiveTileLayerEditGuard, TileLayer};
pub use terminal::{
    fonts::{TerminalFont, TerminalFontAscii, TerminalFontBasic, TerminalFontFull},
    Terminal,
};
pub use vram::{CharAccess, CharData, VramTile};

#[inline(never)]
const fn color_not_valid() {
    panic!("Color data must be in the range 0..31");
}
/// Packs three 5-bit color components into a GBA color.
#[inline(always)]
pub const fn rgb(r: u8, g: u8, b: u8) -> u16 {
    if r >= 32 || g >= 32 || b >= 32 {
        color_not_valid()
    }
    r as u16 | ((g as u16) << 5) | ((b as u16) << 10)
}
#[inline(always)]
const fn convert_24bpp_15bpp(ch: u8) -> u8 {
    ch.saturating_add(16) / 8
}
/// Packs three 8-bit color components into a GBA color.
#[inline(always)]
pub const fn rgb_24bpp(r: u8, g: u8, b: u8) -> u16 {
    rgb(convert_24bpp_15bpp(r), convert_24bpp_15bpp(g), convert_24bpp_15bpp(b))
}
