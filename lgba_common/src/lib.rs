//! A structures-only library for data structures shared between code that runs on the host system
//! and code that runs on the GBA.
//!
//! Not public API.

#![no_std]

#[cfg(feature = "generator_base")]
extern crate std;

pub mod common;

#[cfg(feature = "data")]
pub mod data;

#[cfg(feature = "phf")]
pub mod phf;

#[cfg(feature = "generator_build")]
mod encoder;

#[cfg(feature = "hashes")]
pub mod hashes;
