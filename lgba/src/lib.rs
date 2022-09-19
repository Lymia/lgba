#![feature(alloc_error_handler, isa_attribute)]
#![no_std]

mod mmio;
mod panic_handler;

//pub mod debug;
pub mod display;
pub mod irq;
pub mod sync;
pub mod sys;

// public reexports
pub use mmio::*;

pub use lgba_macros::{entry, ewram, iwram};
