//! See [the `phf` crate's documentation][phf] for details.
//!
//! [phf]: https://docs.rs/phf

use core::{
    fmt,
    hash::{Hash, Hasher},
    num::Wrapping,
};
use fnv::FnvHasher;

#[non_exhaustive]
pub struct Hashes {
    pub g: u32,
    pub f1: u32,
    pub f2: u32,
}

/// A central typedef for hash keys
///
/// Makes experimentation easier by only needing to be updated here.
pub type HashKey = u64;

#[inline]
pub fn displace(f1: u32, f2: u32, d1: u32, d2: u32) -> u32 {
    (Wrapping(d2) + Wrapping(f1) * Wrapping(d1) + Wrapping(f2)).0
}

/// `key` is from `phf_generator::HashState`.
#[inline]
pub fn hash<T: ?Sized + Hash>(x: &T, key: &HashKey) -> Hashes {
    let mut hasher = FnvHasher::with_key(*key);
    x.hash(&mut hasher);
    let raw = hasher.finish();

    Hashes {
        g: raw as u32 & 0x1FFFFF,
        f1: (raw >> 21) as u32 & 0x1FFFFF,
        f2: (raw >> 42) as u32 & 0x1FFFFF,
    }
}

/// Return an index into `phf_generator::HashState::map`.
///
/// * `hash` is from `hash()` in this crate.
/// * `disps` is from `phf_generator::HashState::disps`.
/// * `len` is the length of `phf_generator::HashState::map`.
#[inline]
pub fn get_index(hashes: &Hashes, disps: &[(u32, u32)], len: usize) -> u32 {
    let (d1, d2) = disps[(hashes.g % (disps.len() as u32)) as usize];
    displace(hashes.f1, hashes.f2, d1, d2) % (len as u32)
}
