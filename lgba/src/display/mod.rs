//! Module containing interfaces to the GBA's graphics chip.

#[cfg(feature = "terminal")]
mod terminal;
mod vram;

#[cfg(feature = "terminal")]
pub use terminal::Terminal;

pub use vram::*;
