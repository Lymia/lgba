use core::{
    hash::{Hash, Hasher},
    num::Wrapping,
};

pub struct Hashes {
    pub g: u32,
    pub f1: u32,
    pub f2: u32,
}

pub type DisplacementData = u16;
#[cfg(feature = "generator")]
pub const MAX_DISP: u32 = 256;

#[cfg(feature = "generator")]
pub fn pack_displacement(a: u32, b: u32) -> DisplacementData {
    debug_assert!(a <= MAX_DISP && b <= MAX_DISP);
    ((a << 8) | b) as u16
}
pub fn unpack_displacement(disp: DisplacementData) -> (u32, u32) {
    ((disp >> 8) as u32, (disp & 0xFF) as u32)
}

#[inline]
pub fn displace(f1: u32, f2: u32, d1: u32, d2: u32) -> u32 {
    let (f1, f2, d1, d2) = (Wrapping(f1), Wrapping(f2), Wrapping(d1), Wrapping(d2));
    (d2 + f1 * d1 + f2).0
}
pub fn get_index(
    hashes: &Hashes,
    disps: &[DisplacementData],
    disp_mask: u32,
    len_mask: u32,
) -> u32 {
    let (d1, d2) = unpack_displacement(disps[(hashes.g & disp_mask) as usize]);
    displace(hashes.f1, hashes.f2, d1, d2) & len_mask
}

pub type HashKey = u32;
pub fn make_hash<T: ?Sized + Hash>(key: HashKey, value: &T) -> Hashes {
    let mut raw_hasher = twox_hash::XxHash32::with_seed(key);
    value.hash(&mut raw_hasher);
    let hash_a = raw_hasher.finish() as u32;

    Hashes {
        g: hash_a,
        f1: hash_a.rotate_left(10),
        f2: hash_a.rotate_left(20).wrapping_mul(29943829),
    }
}
