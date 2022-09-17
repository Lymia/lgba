// Copyright (c) 2022 Lymia Aluysia
// Copyright (c) 2017 10X Genomics, Inc. All rights reserved.
// Copyright (c) 2015 Guillaume Rizk
// Some portions of this code are derived from https://github.com/rizkg/BBHash (MIT license)

//! ### boomphf - Fast and scalable minimal perfect hashing for massive key sets
//! A Rust implementation of the BBHash method for constructing minimal perfect hash functions,
//! as described in "Fast and scalable minimal perfect hashing for massive key sets"
//! [https://arxiv.org/abs/1702.03154](https://arxiv.org/abs/1702.03154). The library generates
//! a minimal perfect hash function (MPHF) for a collection of hashable objects. Note: minimal
//! perfect hash functions can only be used with the set of objects used when hash function
//! was created. Hashing a new object will return an arbitrary hash value. If your use case
//! may result in hashing new values, you will need an auxiliary scheme to detect this condition.
//!
//! ```
//! use lgba_phf::*;
//! // Generate MPHF
//! let possible_objects = vec![1, 10, 1000, 23, 457, 856, 845, 124, 912];
//! let n = possible_objects.len();
//! let phf = Mphf::new(1.7, &possible_objects);
//! // Get hash value of all objects
//! let mut hashes = Vec::new();
//! for v in possible_objects {
//!     hashes.push(phf.hash(&v));
//! }
//! hashes.sort();
//!
//! // Expected hash output is set of all integers from 0..n
//! let expected_hashes: Vec<u64> = (0 .. n as u64).collect();
//! assert_eq!(hashes, expected_hashes)
//! ```

mod bitvector;
use bitvector::BitVector;

use std::{
    borrow::Borrow,
    fmt::Debug,
    hash::{Hash, Hasher},
    marker::PhantomData,
};

#[inline]
fn fold(v: u64) -> u32 {
    ((v & 0xFFFFFFFF) as u32) ^ ((v >> 32) as u32)
}

#[inline]
fn hash_with_seed<T: Hash + ?Sized>(iter: u64, v: &T) -> u64 {
    let mut state = fnv::FnvHasher::with_key(1 << (iter + iter));
    v.hash(&mut state);
    state.finish()
}

#[inline]
fn hash_with_seed32<T: Hash + ?Sized>(iter: u64, v: &T) -> u32 {
    fold(hash_with_seed(iter, v))
}

#[inline]
fn fastmod(hash: u32, n: u32) -> u64 {
    ((hash as u64) * (n as u64)) >> 32
}

#[inline]
fn hashmod<T: Hash + ?Sized>(iter: u64, v: &T, n: usize) -> u64 {
    // when n < 2^32, use the fast alternative to modulo described here:
    // https://lemire.me/blog/2016/06/27/a-fast-alternative-to-the-modulo-reduction/
    if n < 1 << 32 {
        let h = hash_with_seed32(iter, v);
        fastmod(h, n as u32) as u64
    } else {
        let h = hash_with_seed(iter, v);
        h % (n as u64)
    }
}

/// A minimal perfect hash function over a set of objects of type `T`.
#[derive(Clone, Debug)]
pub struct Mphf<T> {
    bitvecs: Box<[BitVector]>,
    ranks: Box<[Box<[u64]>]>,
    phantom: PhantomData<T>,
}

const MAX_ITERS: u64 = 100;

impl<'a, T: 'a + Hash + Debug> Mphf<T> {
    /// Constructs an MPHF from a (possibly lazy) iterator over iterators.
    /// This allows construction of very large MPHFs without holding all the keys
    /// in memory simultaneously.
    /// `objects` is an `IntoInterator` yielding a stream of `IntoIterator`s that must not contain any duplicate items.
    /// `objects` must be able to be iterated over multiple times and yield the same stream of items each time.
    /// `gamma` controls the tradeoff between the construction-time and run-time speed,
    /// and the size of the datastructure representing the hash function. See the paper for details.
    /// `max_iters` - None to never stop trying to find a perfect hash (safe if no duplicates).
    /// NOTE: the inner iterator `N::IntoIter` should override `nth` if there's an efficient way to skip
    /// over items when iterating.  This is important because later iterations of the MPHF construction algorithm
    /// skip most of the items.
    pub fn from_chunked_iterator<I, N>(gamma: f64, objects: &'a I, n: usize) -> Mphf<T>
    where
        &'a I: IntoIterator<Item = N>,
        N: IntoIterator<Item = T> + Send,
        <N as IntoIterator>::IntoIter: ExactSizeIterator,
        <&'a I as IntoIterator>::IntoIter: Send,
        I: Sync,
    {
        let mut iter = 0;
        let mut bitvecs = Vec::new();
        let mut done_keys = BitVector::new(std::cmp::max(255, n));

        assert!(gamma > 1.01);

        loop {
            if iter > MAX_ITERS {
                panic!("ran out of key space. items: {:?}", done_keys.len());
            }

            let keys_remaining = if iter == 0 { n } else { n - done_keys.len() };

            let size = std::cmp::max(255, (gamma * keys_remaining as f64) as u64);

            let mut a = BitVector::new(size as usize);
            let mut collide = BitVector::new(size as usize);

            let seed = iter;
            let mut offset = 0;

            for object in objects {
                let mut object_iter = object.into_iter();

                // Note: we will use Iterator::nth() to advance the iterator if
                // we've skipped over some items.
                let mut object_pos = 0;
                let len = object_iter.len();

                for object_index in 0..len {
                    let index = offset + object_index;

                    if !done_keys.contains(index) {
                        let key = match object_iter.nth(object_index - object_pos) {
                            None => panic!("ERROR: max number of items overflowed"),
                            Some(key) => key,
                        };

                        object_pos = object_index + 1;

                        let idx = hashmod(seed, &key, size as usize);

                        if collide.contains(idx as usize) {
                            continue;
                        }
                        let a_was_set = !a.insert(idx as usize);
                        if a_was_set {
                            collide.insert(idx as usize);
                        }
                    }
                } // end-window for

                offset += len;
            } // end-objects for

            let mut offset = 0;
            for object in objects {
                let mut object_iter = object.into_iter();

                // Note: we will use Iterator::nth() to advance the iterator if
                // we've skipped over some items.
                let mut object_pos = 0;
                let len = object_iter.len();

                for object_index in 0..len {
                    let index = offset + object_index;

                    if !done_keys.contains(index) {
                        // This will fast-forward the iterator over unneeded items.
                        let key = match object_iter.nth(object_index - object_pos) {
                            None => panic!("ERROR: max number of items overflowed"),
                            Some(key) => key,
                        };

                        object_pos = object_index + 1;

                        let idx = hashmod(seed, &key, size as usize);

                        if collide.contains(idx as usize) {
                            a.remove(idx as usize);
                        } else {
                            done_keys.insert(index as usize);
                        }
                    }
                } // end-window for

                offset += len;
            } // end- objects for

            bitvecs.push(a);
            if done_keys.len() == n {
                break;
            }
            iter += 1;
        }

        let ranks = Self::compute_ranks(&bitvecs);
        Mphf { bitvecs: bitvecs.into_boxed_slice(), ranks, phantom: PhantomData }
    }
}

impl<T: Hash + Debug> Mphf<T> {
    /// Generate a minimal perfect hash function for the set of `objects`.
    /// `objects` must not contain any duplicate items.
    /// `gamma` controls the tradeoff between the construction-time and run-time speed,
    /// and the size of the datastructure representing the hash function. See the paper for details.
    /// `max_iters` - None to never stop trying to find a perfect hash (safe if no duplicates).
    pub fn new(gamma: f64, objects: &[T]) -> Mphf<T> {
        assert!(gamma > 1.01);
        let mut bitvecs = Vec::new();
        let mut iter = 0;

        let mut cx = Context::new(
            std::cmp::max(255, (gamma * objects.len() as f64) as usize).next_power_of_two(),
            iter,
        );

        objects.iter().for_each(|v| cx.find_collisions(v));
        let mut redo_keys = objects
            .iter()
            .filter_map(|v| cx.filter(v))
            .collect::<Vec<_>>();

        bitvecs.push(cx.a);
        iter += 1;

        while !redo_keys.is_empty() {
            let mut cx = Context::new(
                std::cmp::max(255, (gamma * redo_keys.len() as f64) as usize).next_power_of_two(),
                iter,
            );

            (&redo_keys)
                .iter()
                .for_each(|&v| cx.find_collisions(v));
            redo_keys = (&redo_keys).iter().filter_map(|&v| cx.filter(v)).collect();

            bitvecs.push(cx.a);
            iter += 1;
            if iter > MAX_ITERS {
                panic!("ran out of key space. items: {:?}", redo_keys);
            }
        }

        Mphf {
            ranks: Self::compute_ranks(&bitvecs),
            bitvecs: bitvecs.into_boxed_slice(),
            phantom: PhantomData,
        }
    }

    fn compute_ranks(bvs: &[BitVector]) -> Box<[Box<[u64]>]> {
        let mut ranks = Vec::new();
        let mut pop = 0_u64;

        for bv in bvs {
            let mut rank: Vec<u64> = Vec::new();
            for i in 0..bv.num_words() {
                let v = bv.get_word(i);

                if i % 8 == 0 {
                    rank.push(pop)
                }

                pop += v.count_ones() as u64;
            }

            ranks.push(rank.into_boxed_slice())
        }

        ranks.into_boxed_slice()
    }

    #[inline]
    fn get_rank(&self, hash: u64, i: usize) -> u64 {
        let idx = hash as usize;
        let bv = self.bitvecs.get(i).expect("that level doesn't exist");
        let ranks = self.ranks.get(i).expect("that level doesn't exist");

        // Last pre-computed rank
        let mut rank = ranks[idx / 512];

        // Add rank of intervening words
        for j in (idx / 64) & !7..idx / 64 {
            rank += bv.get_word(j).count_ones() as u64;
        }

        // Add rank of final word up to hash
        let final_word = bv.get_word(idx / 64);
        if idx % 64 > 0 {
            rank += (final_word << (64 - (idx % 64))).count_ones() as u64;
        }
        rank
    }

    /// Compute the hash value of `item`. This method should only be used
    /// with items known to be in construction set. Use `try_hash` if you cannot
    /// guarantee that `item` was in the construction set. If `item` was not present
    /// in the construction set this function may panic.
    pub fn hash(&self, item: &T) -> u64 {
        for i in 0..self.bitvecs.len() {
            let bv = &self.bitvecs[i];
            let hash = hashmod(i as u64, item, bv.capacity());

            if bv.contains(hash as usize) {
                return self.get_rank(hash, i);
            }
        }

        unreachable!("must find a hash value");
    }

    /// Compute the hash value of `item`. If `item` was not present
    /// in the set of objects used to construct the hash function, the return
    /// value will an arbitrary value Some(x), or None.
    pub fn try_hash<Q>(&self, item: &Q) -> Option<u64>
    where
        T: Borrow<Q>,
        Q: ?Sized + Hash,
    {
        for i in 0..self.bitvecs.len() {
            let bv = &(self.bitvecs)[i];
            let hash = hashmod(i as u64, item, bv.capacity());

            if bv.contains(hash as usize) {
                return Some(self.get_rank(hash, i));
            }
        }

        None
    }
}

struct Context {
    size: usize,
    seed: u64,
    a: BitVector,
    collide: BitVector,
}

impl Context {
    fn new(size: usize, seed: u64) -> Self {
        Self { size, seed, a: BitVector::new(size), collide: BitVector::new(size) }
    }

    fn find_collisions<T: Hash>(&mut self, v: &T) {
        let idx = hashmod(self.seed, v, self.size) as usize;
        if !self.collide.contains(idx) && !self.a.insert(idx) {
            self.collide.insert(idx);
        }
    }

    fn filter<'t, T: Hash>(&mut self, v: &'t T) -> Option<&'t T> {
        let idx = hashmod(self.seed, v, self.size) as usize;
        if self.collide.contains(idx) {
            self.a.remove(idx);
            Some(v)
        } else {
            None
        }
    }
}

#[cfg(test)]
#[macro_use]
extern crate quickcheck;

#[cfg(test)]
mod tests {

    use super::*;
    use std::{collections::HashSet, iter::FromIterator};

    /// Check that a Minimal perfect hash function (MPHF) is generated for the set xs
    fn check_mphf<T>(xs: HashSet<T>) -> bool
    where T: Sync + Hash + PartialEq + Eq + Debug + Send {
        let xsv: Vec<T> = xs.into_iter().collect();

        // test single-shot data input
        check_mphf_serial(&xsv)
    }

    /// Check that a Minimal perfect hash function (MPHF) is generated for the set xs
    fn check_mphf_serial<T>(xsv: &[T]) -> bool
    where T: Hash + PartialEq + Eq + Debug {
        // Generate the MPHF
        let phf = Mphf::new(1.7, xsv);

        // Hash all the elements of xs
        let mut hashes: Vec<u64> = xsv.iter().map(|v| phf.hash(v)).collect();

        hashes.sort_unstable();

        // Hashes must equal 0 .. n
        let gt: Vec<u64> = (0..xsv.len() as u64).collect();
        hashes == gt
    }

    fn check_chunked_mphf<T>(values: Vec<Vec<T>>, total: usize) -> bool
    where T: Sync + Hash + PartialEq + Eq + Debug + Send {
        let phf = Mphf::from_chunked_iterator(1.7, &values, total);

        // Hash all the elements of xs
        let mut hashes: Vec<u64> = values
            .iter()
            .flat_map(|x| x.iter().map(|v| phf.hash(&v)))
            .collect();

        hashes.sort_unstable();

        // Hashes must equal 0 .. n
        let gt: Vec<u64> = (0..total as u64).collect();
        hashes == gt
    }

    quickcheck! {
        fn check_int_slices(v: HashSet<u64>, lens: Vec<usize>) -> bool {

            let mut lens = lens;

            let items: Vec<u64> = v.iter().cloned().collect();
            if lens.is_empty() || lens.iter().all(|x| *x == 0) {
                lens.clear();
                lens.push(items.len())
            }

            let mut slices: Vec<Vec<u64>> = Vec::new();

            let mut total = 0_usize;
            for slc_len in lens {
                let end = std::cmp::min(items.len(), total.saturating_add(slc_len));
                let slc = Vec::from(&items[total..end]);
                slices.push(slc);
                total = end;

                if total == items.len() {
                    break;
                }
            }

            check_chunked_mphf(slices.clone(), total)
        }
    }

    quickcheck! {
        fn check_string(v: HashSet<Vec<String>>) -> bool {
            check_mphf(v)
        }
    }

    quickcheck! {
        fn check_u32(v: HashSet<u32>) -> bool {
            check_mphf(v)
        }
    }

    quickcheck! {
        fn check_isize(v: HashSet<isize>) -> bool {
            check_mphf(v)
        }
    }

    quickcheck! {
        fn check_u64(v: HashSet<u64>) -> bool {
            check_mphf(v)
        }
    }

    quickcheck! {
        fn check_vec_u8(v: HashSet<Vec<u8>>) -> bool {
            check_mphf(v)
        }
    }

    #[test]
    fn from_ints_serial() {
        let items = (0..1000000).map(|x| x * 2);
        assert!(check_mphf(HashSet::from_iter(items)));
    }
}
