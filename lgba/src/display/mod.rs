//! Module containing interfaces to the GBA's graphics chip.

mod layers;
mod modes;
mod terminal;
mod vram;

use crate::mmio::reg::{DISPSTAT, VCOUNT};

pub use layers::{ActiveTileLayer, ActiveTileLayerEditGuard, TileLayer};
pub use terminal::{
    fonts::{TerminalFont, TerminalFontAscii, TerminalFontBasic, TerminalFontFull},
    ActiveTerminal, ActiveTerminalAccess, ActiveTerminalWrite, Terminal,
};
pub use vram::{CharAccess, CharData, VramTile};

/// Packs three 5-bit color components into a GBA color.
#[inline(always)]
#[track_caller]
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

#[inline(never)]
#[track_caller]
const fn color_not_valid() {
    panic!("Color data must be in the range 0..31");
}

/// Returns whether the graphics chip is currently in a vertical blank period.
pub fn is_vblank() -> bool {
    DISPSTAT.read().is_vblank()
}

/// Returns whether the graphics chip is currently in a horizontal blank period.
pub fn is_hblank() -> bool {
    DISPSTAT.read().is_hblank()
}

/// Returns the scanline the graphics chip is currently processing.
pub fn scanline() -> usize {
    VCOUNT.read() as usize
}

/// Sets the scanline the [`VCounter`] interrupt triggers on.
///
/// The interrupt must still be enabled separately with [`irq::enable`].
///
/// [`irq::enable`]: crate::irq::enable
/// [`VCounter`]: crate::irq::Interrupt::VCounter
pub fn set_counter_scanline(count: usize) {
    if count > 227 {
        vcount_not_valid();
    }
    DISPSTAT.write(DISPSTAT.read().with_vcount_scanline(count as u32));
}

#[inline(never)]
#[track_caller]
const fn vcount_not_valid() {
    panic!("vcount scanline must be between 0 and 227 inclusive");
}
