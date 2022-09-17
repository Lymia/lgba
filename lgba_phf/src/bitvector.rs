// Copyright (c) 2022 Lymia Aluysia
// Copyright (c) 2018 10x Genomics, Inc. All rights reserved.
//
// Note this code was copied from https://github.com/zhaihj/bitvector (MIT licensed),
// and modified to add rank/select operations, and to use atomic primitives to allow
// multi-threaded access. The original copyright license text is here:
//
// The MIT License (MIT)
//
// Copyright (c) 2016 Hongjie Zhai

//! ### BitVector Module
//!
//! BitVector uses one bit to represent a bool state.
//! BitVector is useful for the programs that need fast set operation (intersection, union,
//! difference), because that all these operations can be done with simple bitand, bitor, bitxor.
//!
//! ### Implementation Details
//!
//! BitVector is realized with a `Vec<u32>`. Each bit of an u32 represent if a elements exists.
//! BitVector always increases from the end to begin, it meats that if you add element `0` to an
//! empty bitvector, then the `Vec<u32>` will change from `0x00` to `0x01`.
//!
//! Of course, if the real length of set can not be divided by 32,
//! it will have a `capacity() % 32` bit memory waste.

use alloc::{boxed::Box, vec, vec::Vec};

/// Bitvector
#[derive(Clone, Debug)]
pub struct BitVector {
    bits: usize,
    vector: Box<[u32]>,
}

impl PartialEq for BitVector {
    fn eq(&self, other: &BitVector) -> bool {
        self.eq_left(other, self.bits)
    }
}

impl BitVector {
    /// Build a new empty bitvector
    pub fn new(bits: usize) -> Self {
        let n = u32_size(bits);
        let v = vec![0; n];
        BitVector { bits, vector: v.into_boxed_slice() }
    }

    /// new bitvector contains all elements
    ///
    /// If `bits % 32 > 0`, the last u32 is guaranteed not to have any extra 1 bits.
    #[allow(dead_code)]
    pub fn ones(bits: usize) -> Self {
        let (word, offset) = word_offset(bits);
        let mut bvec = Vec::with_capacity(word + 1);
        for _ in 0..word {
            bvec.push(u32::MAX);
        }

        bvec.push(u32::MAX >> (32 - offset));
        BitVector { bits, vector: bvec.into_boxed_slice() }
    }

    /// return if this set is empty
    ///
    /// if set does not contain any elements, return true;
    /// else return false.
    ///
    /// This method is averagely faster than `self.len() > 0`.
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.vector.iter().all(|x| *x == 0)
    }

    /// If `bit` belongs to set, return `true`, else return `false`.
    ///
    /// Insert, remove and contains do not do bound check.
    #[inline]
    pub fn contains(&self, bit: usize) -> bool {
        let (word, mask) = word_mask(bit);
        (self.get_word(word) & mask) != 0
    }

    /// compare if the following is true:
    ///
    /// self \cap {0, 1, ... , bit - 1} == other \cap {0, 1, ... ,bit - 1}
    pub fn eq_left(&self, other: &BitVector, bit: usize) -> bool {
        if bit == 0 {
            return true;
        }
        let (word, offset) = word_offset(bit - 1);
        // We can also use slice comparison, which only take 1 line.
        // However, it has been reported that the `Eq` implementation of slice
        // is extremly slow.
        //
        // self.vector.as_slice()[0 .. word] == other.vector.as_slice[0 .. word]
        //
        self.vector
            .iter()
            .zip(other.vector.iter())
            .take(word)
            .all(|(s1, s2)| s1 == s2)
            && (self.get_word(word) << (31 - offset)) == (other.get_word(word) << (31 - offset))
    }

    /// insert a new element synchronously.
    /// requires &mut self, but doesn't use
    /// atomic instructions so may be faster
    /// than `insert()`.
    ///
    /// If value is inserted, return true,
    /// if value already exists in set, return false.
    ///
    /// Insert, remove and contains do not do bound check.
    #[inline]
    pub fn insert(&mut self, bit: usize) -> bool {
        let (word, mask) = word_mask(bit);

        let old_data = self.vector[word];
        self.vector[word] |= mask;
        old_data & mask == 0
    }

    /// remove an element from set
    ///
    /// If value is removed, return true,
    /// if value doesn't exist in set, return false.
    ///
    /// Insert, remove and contains do not do bound check.
    pub fn remove(&mut self, bit: usize) -> bool {
        let (word, mask) = word_mask(bit);

        let prev = self.vector[word];
        self.vector[word] &= !mask;
        prev & mask != 0
    }

    /// the max number of elements can be inserted into set
    pub fn capacity(&self) -> usize {
        self.bits
    }

    #[inline]
    pub fn get_word(&self, word: usize) -> u32 {
        self.vector[word]
    }

    pub fn num_words(&self) -> usize {
        self.vector.len()
    }
}

#[inline]
fn u32_size(elements: usize) -> usize {
    (elements + 31) / 32
}

#[inline]
fn word_offset(index: usize) -> (usize, usize) {
    (index / 32, index % 32)
}

#[inline]
fn word_mask(index: usize) -> (usize, u32) {
    let word = index / 32;
    let mask = 1 << (index % 32);
    (word, mask)
}
