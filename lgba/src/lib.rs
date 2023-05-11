#![feature(alloc_error_handler)]
#![no_std]

// TODO: Liberal use of #[track_caller]

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
pub use lgba_macros::{entry, ewram, iwram};

// hack for the procedural macros
use crate as lgba;

/// **NOT** public API!! Only for this crate's macros.
#[doc(hidden)]
pub mod __macro_export {
    pub use core;
    pub use lgba_phf;

    pub mod gba_header {
        pub use crate::gba_header::*;
    }

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
