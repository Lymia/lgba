//! Module containing interfaces to the GBA's graphics chip.

#[macro_use]
mod macros;

#[cfg(feature = "terminal")]
mod terminal;
mod vram;

#[cfg(feature = "terminal")]
pub use terminal::Terminal;

pub use vram::*;
