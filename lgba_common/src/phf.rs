use crate::base_repr::SerialSlice;
use core::hash::Hash;
use lgba_phf::{DisplacementData, HashKey};
#[cfg(feature = "generator_phf")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "generator_phf", derive(Serialize, Deserialize))]
#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct PhfTable<K, V> {
    pub hash_key: HashKey,
    pub disps: SerialSlice<DisplacementData>,
    pub keys: SerialSlice<K>,
    pub values: SerialSlice<V>,
    pub presence_map: SerialSlice<u32>,
}
impl<K: Eq + Hash, V> PhfTable<K, V> {
    pub unsafe fn lookup(&self, k: &K) -> Option<&'static V> {
        let bucket =
            lgba_phf::hash_dynamic(self.hash_key, self.disps.as_slice(), k, self.keys.len - 1);

        if *self.presence_map.offset(bucket / 32) & (1 << (bucket % 32)) == 0 {
            None
        } else {
            if &*self.keys.offset(bucket) == k {
                Some(&*self.values.offset(bucket))
            } else {
                None
            }
        }
    }
}
impl<K, V> Default for PhfTable<K, V> {
    fn default() -> Self {
        PhfTable {
            hash_key: 0,
            disps: Default::default(),
            keys: Default::default(),
            values: Default::default(),
            presence_map: Default::default(),
        }
    }
}

#[cfg(feature = "generator_phf")]
pub fn build_phf<K: Eq + Hash + Clone + Serialize, V: Clone + Serialize>(
    base_offset: u32,
    entries: &[(K, V)],
) -> std::vec::Vec<u8> {
    use std::{mem, vec, vec::Vec};

    let mut keys = Vec::new();
    for (k, _) in entries {
        keys.push(k);
    }
    let generator = lgba_phf::generator::generate_hash(1.0, &keys);

    let mut new_data = Vec::new();
    new_data.extend(vec![0; mem::size_of::<PhfTable<K, V>>()]);

    let start_disps_table = base_offset + new_data.len() as u32;
    for disp in &generator.disps {
        new_data.extend(disp.to_le_bytes());
    }

    let start_key_table = base_offset + new_data.len() as u32;
    let mut presence_table = vec![0u32; (generator.map.len() + 31) / 32];
    for (i, idx) in generator.map.iter().enumerate() {
        if *idx == !0 {
            new_data.extend(vec![0; mem::size_of::<K>()])
        } else {
            let mut data = vec![0; mem::size_of::<K>()];
            assert_eq!(
                ssmarshal::serialize(&mut data, &entries[*idx].0.clone()).unwrap(),
                data.len(),
            );
            presence_table[i / 32] |= 1 << (i % 32);
        }
    }

    let start_value_table = base_offset + new_data.len() as u32;
    for idx in &generator.map {
        if *idx == !0 {
            new_data.extend(vec![0; mem::size_of::<V>()])
        } else {
            let mut data = vec![0; mem::size_of::<V>()];
            assert_eq!(
                ssmarshal::serialize(&mut data, &entries[*idx].1.clone()).unwrap(),
                data.len(),
            );
        }
    }

    let start_presence_map = base_offset + new_data.len() as u32;
    for entry in &presence_table {
        new_data.extend(entry.to_le_bytes());
    }

    let table = PhfTable::<K, V> {
        hash_key: generator.key,
        disps: SerialSlice {
            ptr: start_disps_table,
            len: generator.disps.len() as u32,
            _phantom: Default::default(),
        },
        keys: SerialSlice {
            ptr: start_key_table,
            len: generator.map.len() as u32,
            _phantom: Default::default(),
        },
        values: SerialSlice {
            ptr: start_value_table,
            len: generator.map.len() as u32,
            _phantom: Default::default(),
        },
        presence_map: SerialSlice {
            ptr: start_presence_map,
            len: presence_table.len() as u32,
            _phantom: Default::default(),
        },
    };
    assert_eq!(
        ssmarshal::serialize(&mut new_data, &table).unwrap(),
        mem::size_of::<PhfTable<K, V>>()
    );

    new_data
}
