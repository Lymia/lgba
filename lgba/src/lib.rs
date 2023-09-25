//! A low-level GBA library designed to allow maximum control over the hardware without sacrificing
//! memory safety or ergonomics.
//!
//! # Cargo Features
//!
//! * The `allocator` feature registers a global allocator. This feature is enabled by default, and
//!   can be disabled if you want to use a different memory allocator.
//! * The `low_level` feature enables low-level and extremely unsafe functions that are not useful
//!   for typical homebrew. The features used here are meant for software such as ROM hacks that
//!   must manually setup `lgba` and work with existing hardware state controlled by external code.
//! * The `log` feature enables compatibility with the [`log`] crate, registering a logger on
//!   startup.
//!
//! [`log`]: https://docs.rs/log/latest/log/
//!
//! # Using a custom entry point
//!
//! `lgba` is designed to be used with a custom entry point for more advanced use cases. While
//! you cannot disable the `__start` symbol, you can edit `lgba.ld` to use a different symbol as
//! the start in your project.
//!
//! Your custom initialization function should do the following:
//!
//! * Call `__lgba_init_memory` before *any* Rust code is executed. This function copies the
//!   initial contents of iwram and ewram from the ROM. In most modern systems this is done by a
//!   executable loader, but the GBA has none. This subroutine has no parameters and may clobber
//!   any register.
//! * Call `__lgba_init` from assembly code, or [`init_lgba`] from Rust code. This must be done
//!   before using any other lgba functionality.
//! * Optionally, call `__lgba_setup` from assembly, or [`setup_lgba`] from Rust code. This should
//!   be done after `__lgba_init`.
//!
//! TODO: stack canaries, offsets for iwram/ewram, etc
//!
//! # Stability
//!
//! Any item containing `__` or that is marked `#[doc(hidden)]` is not public API and should not be
//! used. Furthermore, while this crate contains many `#[no_mangle]` symbols, they are not public
//! API and there are no stability guarantees unless otherwise stated.

#![feature(allocator_api)]
#![feature(alloc_error_handler)]
#![feature(panic_info_message)]
#![feature(slice_ptr_get)]
#![feature(doc_cfg)]
#![feature(linkage)]
#![no_std]

extern crate alloc;

pub mod debug;

mod asm;
mod mmio;
mod panic_handler;

pub mod display;
pub mod dma;
pub mod irq;
pub mod save;
pub mod sync;
pub mod sys;
pub mod timer;

// public reexports
#[cfg(feature = "low_level")]
pub use asm::{init_lgba, setup_lgba};
#[cfg(feature = "low_level")]
pub use lgba_macros::unsafe_alloc_zones;
pub use lgba_macros::{arm, entry, ewram, iwram, thumb};

/// A module allowing easier usage of memory-mapped registers.
#[cfg(feature = "low_level")]
#[doc(cfg(feature = "low_level"))]
pub mod reg {
    pub use crate::mmio::reg::{RegArray, RegSpanned, Register, Safe, Unsafe};
}

/// **NOT** public API!! Only for this crate's macros.
#[doc(hidden)]
pub mod __macro_export {
    pub use crate::asm::gba_header;
    pub use core;
    pub use lgba_common::common::StaticStr;
    pub use lgba_phf;

    //noinspection RsAssertEqual
    pub const fn xfer_u8_u16<const N: usize>(data: &[u8]) -> [u16; N] {
        assert!(data.len() == N * 2, "Array length is not a multiple of 2.");

        let mut u16_data = [0u16; N];

        let mut i = 0;
        while i < u16_data.len() {
            u16_data[i] = u16::from_le_bytes([data[i * 2], data[i * 2 + 1]]);
            i += 1;
        }

        u16_data
    }

    //noinspection RsAssertEqual
    pub const fn xfer_u8_u32<const N: usize>(data: &[u8]) -> [u32; N] {
        assert!(data.len() == N * 4, "Array length is not a multiple of 4.");

        let mut u32_data = [0u32; N];

        let mut i = 0;
        while i < u32_data.len() {
            u32_data[i] = u32::from_le_bytes([
                data[i * 4],
                data[i * 4 + 1],
                data[i * 4 + 2],
                data[i * 4 + 3],
            ]);
            i += 1;
        }

        u32_data
    }
}
