#![feature(alloc_error_handler, isa_attribute)]
#![no_std]

pub mod debug;

mod gba_header;
mod mmio;
mod panic_handler;

pub mod display;
pub mod dma;
pub mod irq;
pub mod sync;
pub mod sys;

// public reexports
pub use mmio::*;

pub use lgba_macros::{entry, ewram, iwram};

/// **NOT** public API!! Only for this crate's macros.
#[doc(hidden)]
pub mod __macro_export {
    pub use crate::gba_header::*;
    pub use core;
}
