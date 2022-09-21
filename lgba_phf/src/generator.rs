use crate::{
    params,
    params::{DisplacementData, HashKey},
};
use alloc::{string::String, vec, vec::Vec};
use core::{cmp::min, fmt::Write, hash::Hash};

#[derive(Clone, Debug)]
pub struct HashState {
    pub key: HashKey,
    pub disps: Vec<DisplacementData>,
    pub map: Vec<usize>,
}
impl HashState {
    pub fn generate_rust_code(&self, hash_fn_name: &str, in_ty: &str) -> String {
        let mut disps = String::new();
        for (i, disp) in self.disps.iter().enumerate() {
            if i % 13 == 0 {
                disps.push_str("        ");
            }
            write!(disps, "0x{disp:04x},").unwrap();
            if i % 13 == 12 {
                disps.push_str("\n");
            }
        }
        if self.disps.len() % 13 != 0 {
            disps.push_str("\n");
        }

        let mut buf = String::new();
        writeln!(buf, "fn {hash_fn_name}(value: &{in_ty}) -> usize {{").unwrap();
        writeln!(buf, "    const KEY: u32 = 0x{:08x};", self.key).unwrap();
        writeln!(buf, "    const DISPS: [u16; {}] = [", self.disps.len()).unwrap();
        buf.push_str(&disps);
        writeln!(buf, "    ];").unwrap();
        writeln!(buf, "    lgba_phf::hash::<{}, {}, _>(", self.disps.len(), self.map.len())
            .unwrap();
        writeln!(buf, "        KEY, &DISPS, value,").unwrap();
        writeln!(buf, "    )").unwrap();
        writeln!(buf, "}}").unwrap();
        buf
    }
}

pub fn generate_hash<H: Hash>(entries: &[H]) -> HashState {
    let mut seed = 1234567890;
    loop {
        if let Some(result) = try_generate_hash(6.0, seed, entries) {
            return result;
        }
        seed = seed.wrapping_mul(1588635695).wrapping_add(12345);
    }
}

fn try_generate_hash<H: Hash>(delta: f32, key: HashKey, entries: &[H]) -> Option<HashState> {
    struct Bucket {
        idx: usize,
        keys: Vec<usize>,
    }

    let hashes: Vec<_> = entries
        .iter()
        .map(|entry| crate::params::make_hash(key, entry))
        .collect();

    let buckets_len = ((hashes.len() as f32 / delta) as usize).next_power_of_two();
    let mut buckets = (0..buckets_len)
        .map(|i| Bucket { idx: i, keys: vec![] })
        .collect::<Vec<_>>();

    for (i, hash) in hashes.iter().enumerate() {
        buckets[(hash.g % (buckets_len as u32)) as usize]
            .keys
            .push(i);
    }

    // Sort descending
    buckets.sort_by(|a, b| a.keys.len().cmp(&b.keys.len()).reverse());

    let table_len = hashes.len().next_power_of_two();
    let mut map = vec![None; table_len];
    let mut disps = vec![0; buckets_len];

    // store whether an element from the bucket being placed is
    // located at a certain position, to allow for efficient overlap
    // checks. It works by storing the generation in each cell and
    // each new placement-attempt is a new generation, so you can tell
    // if this is legitimately full by checking that the generations
    // are equal. (A u64 is far too large to overflow in a reasonable
    // time for current hardware.)
    let mut try_map = vec![0u64; table_len];
    let mut generation = 0u64;

    // the actual values corresponding to the markers above, as
    // (index, key) pairs, for adding to the main map once we've
    // chosen the right disps.
    let mut values_to_add = vec![];

    let bound = min(params::MAX_DISP, table_len as u32);
    'buckets: for bucket in &buckets {
        let disps_lo = (0..bound).flat_map(|y| (0..y).map(move |x| (x, y)));
        let disps_hi = (0..bound).flat_map(|y| (y..bound).map(move |x| (x, y)));
        let disps_it = disps_lo.chain(disps_hi);

        'disps: for (d1, d2) in disps_it {
            values_to_add.clear();
            generation += 1;

            for &key in &bucket.keys {
                let idx = (params::displace(hashes[key].f1, hashes[key].f2, d1, d2)
                    % (table_len as u32)) as usize;
                if map[idx].is_some() || try_map[idx] == generation {
                    continue 'disps;
                }
                try_map[idx] = generation;
                values_to_add.push((idx, key));
            }

            // We've picked a good set of disps
            disps[bucket.idx] = params::pack_displacement(d1, d2);
            for &(idx, key) in &values_to_add {
                map[idx] = Some(key);
            }
            continue 'buckets;
        }

        // Unable to find displacements for a bucket
        return None;
    }

    Some(HashState { key, disps, map: map.into_iter().map(|i| i.unwrap_or(!0)).collect() })
}
