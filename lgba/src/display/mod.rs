//! Module containing interfaces to the GBA's graphics chip.

#[cfg(feature = "terminal")]
mod terminal;

#[cfg(feature = "terminal")]
pub use terminal::Terminal;
