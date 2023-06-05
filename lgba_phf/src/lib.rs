#![no_std]

#[cfg(feature = "generator")]
extern crate alloc;

#[cfg(feature = "generator")]
pub mod generator;

mod params;

pub fn hash<const DISP_LEN: usize, const DATA_LEN: usize, T: core::hash::Hash>(
    key: params::HashKey,
    disps: &[params::DisplacementData; DISP_LEN],
    value: &T,
) -> usize {
    let hashes = params::make_hash(key, value);
    hash_core::<DISP_LEN, DATA_LEN>(disps, hashes)
}

pub const fn hash_u16<const DISP_LEN: usize, const DATA_LEN: usize>(
    key: params::HashKey,
    disps: &[params::DisplacementData; DISP_LEN],
    value: &u16,
) -> usize {
    let hashes = params::make_hash_const(key, &value.to_le_bytes());
    hash_core::<DISP_LEN, DATA_LEN>(disps, hashes)
}

const fn hash_core<const DISP_LEN: usize, const DATA_LEN: usize>(
    disps: &[params::DisplacementData; DISP_LEN],
    hashes: params::Hashes,
) -> usize {
    assert!(DISP_LEN.is_power_of_two(), "DISP_LEN must be a power of two.");
    assert!(DATA_LEN.is_power_of_two(), "DATA_LEN must be a power of two.");
    assert!(DISP_LEN <= u32::MAX as usize, "DISP_LEN is too large.");
    assert!(DATA_LEN <= u32::MAX as usize, "DATA_LEN is too large.");

    let disp_mask = DISP_LEN - 1;
    let data_mask = DATA_LEN - 1;

    params::get_index(&hashes, disps, disp_mask as u32, data_mask as u32) as usize
}

pub fn hash_dynamic<T: core::hash::Hash>(
    key: params::HashKey,
    disps: &[params::DisplacementData],
    value: &T,
    data_len: usize,
) -> usize {
    let disp_len = disps.len();
    assert!(disp_len.is_power_of_two(), "DISP_LEN must be a power of two.");
    assert!(data_len.is_power_of_two(), "DATA_LEN must be a power of two.");
    assert!(disp_len <= u32::MAX as usize, "DISP_LEN is too large.");
    assert!(data_len <= u32::MAX as usize, "DATA_LEN is too large.");

    let disp_mask = disp_len - 1;
    let data_mask = data_len - 1;

    let hashes = params::make_hash(key, value);
    params::get_index(&hashes, disps, disp_mask as u32, data_mask as u32) as usize
}
