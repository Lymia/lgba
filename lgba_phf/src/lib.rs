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

#![no_std]

extern crate alloc;

mod bitvector;
use bitvector::BitVector;

use alloc::{boxed::Box, vec::Vec};
use core::{
    borrow::Borrow,
    cmp,
    fmt::Debug,
    hash::{Hash, Hasher},
    marker::PhantomData,
};

#[inline]
fn hash_with_seed<T: Hash + ?Sized>(iter: u64, v: &T) -> u64 {
    let key = 1 << (iter + iter);
    let mut state = siphasher::sip::SipHasher13::new_with_keys(key, key);
    v.hash(&mut state);
    state.finish()
}

#[inline]
fn hashmod<T: Hash + ?Sized>(iter: u64, v: &T, n: usize) -> u64 {
    let h = hash_with_seed(iter, v);
    h % (n as u64)
}

/// A minimal perfect hash function over a set of objects of type `T`.
#[derive(Clone, Debug)]
pub struct Mphf<T> {
    bitvecs: Box<[BitVector]>,
    ranks: Box<[Box<[u64]>]>,
    phantom: PhantomData<T>,
}

impl<T: Hash + Debug> Mphf<T> {
    /// Generate a minimal perfect hash function for the set of `objects`.
    /// `objects` must not contain any duplicate items.
    /// `gamma` controls the tradeoff between the construction-time and run-time speed,
    /// and the size of the datastructure representing the hash function. See the paper for details.
    pub fn new(gamma: f32, objects: &[T]) -> Mphf<T> {
        assert!(gamma > 1.0);
        let mut bitvecs = Vec::new();
        let mut iter = 0;

        let mut cx = Context::new(
            cmp::max(256, (gamma * objects.len() as f32) as usize).next_power_of_two(),
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
                cmp::max(256, (gamma * redo_keys.len() as f32) as usize).next_power_of_two(),
                iter,
            );

            (&redo_keys).iter().for_each(|&v| cx.find_collisions(v));
            redo_keys = (&redo_keys).iter().filter_map(|&v| cx.filter(v)).collect();

            bitvecs.push(cx.a);
            iter += 1;
            if iter > 100 {
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
        let mut rank = ranks[idx / 256];

        // Add rank of intervening words
        for j in (idx / 32) & !7..idx / 32 {
            rank += bv.get_word(j).count_ones() as u64;
        }

        // Add rank of final word up to hash
        let final_word = bv.get_word(idx / 32);
        if idx % 32 > 0 {
            rank += (final_word << (32 - (idx % 32))).count_ones() as u64;
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
