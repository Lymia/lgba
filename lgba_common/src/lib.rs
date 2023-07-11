//! A structures-only library for data structures shared between code that runs on the host system
//! and code that runs on the GBA.
//!
//! Not public API.

#![no_std]

#[cfg(feature = "generator")]
extern crate std;

pub mod base_repr;

#[cfg(feature = "data")]
pub mod data;

#[cfg(feature = "phf")]
pub mod phf;
