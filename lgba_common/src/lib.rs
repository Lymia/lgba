//! A structures-only library for data structures shared between code that runs on the host system
//! and code that runs on the GBA.
//!
//! Not public API.

#![no_std]

#[cfg(feature = "generator_base")]
extern crate std;

pub mod base_repr;

#[cfg(feature = "data")]
pub mod data;

#[cfg(feature = "phf")]
pub mod phf;

#[cfg(feature = "generator_build")]
mod base_encoder;

#[cfg(feature = "generator_hash")]
fn hashed<T: core::hash::Hash + ?Sized>(data: &T, nonce: u32) -> [u8; 32] {
    use core::hash::{Hash, Hasher};

    struct HasherWrapper<'a>(&'a mut blake3::Hasher);
    impl<'a> Hasher for HasherWrapper<'a> {
        fn finish(&self) -> u64 {
            unreachable!()
        }
        fn write(&mut self, bytes: &[u8]) {
            self.0.update(bytes);
        }
    }

    let mut hasher = blake3::Hasher::new();
    (std::any::type_name::<T>(), data).hash(&mut HasherWrapper(&mut hasher));
    hasher.update(&nonce.to_ne_bytes());
    *hasher.finalize().as_bytes()
}
