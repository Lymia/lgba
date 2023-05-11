//! Module containing interfaces to the GBA's graphics chip.

#[macro_use]
mod macros;

mod layers;
mod modes;
mod terminal;
mod vram;

pub use layers::{ActiveTileLayer, ActiveTileLayerEditGuard, TileLayer};
pub use terminal::{
    Terminal, TerminalFont, TerminalFontAscii, TerminalFontBasic, TerminalFontFull,
};
pub use vram::{CharAccess, CharData, VramTile};
