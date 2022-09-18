#![feature(alloc_error_handler)]
#![no_std]

mod entry;
mod panicking;
mod sys;
#[cfg(feature = "terminal")]
mod terminal;

// public reexports
pub use lgba_macros::{entry, ewram, iwram};
pub use sys::*;
#[cfg(feature = "terminal")]
pub use terminal::Terminal;
