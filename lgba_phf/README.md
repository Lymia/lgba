# Perfect Hash Functions for GBA (and similar platforms)

A Rust impl of [**Fast and scalable minimal perfect hashing for massive key sets**](https://arxiv.org/abs/1702.03154)
based on [`rust-boomphf`](https://github.com/10XGenomics/rust-boomphf).

The library generates a minimal perfect hash functions (MPHF) for a collection of hashable objects. This algorithm
generates MPHFs that consume ~3-6 bits/item.  The memory consumption during construction is a small multiple (< 2x)
of the size of the dataset and final size of the MPHF. 

Note, minimal perfect hash functions only return a usable hash value for objects in the set used to create the MPHF.
Hashing a new object will return an arbitrary hash value. If your use case may result in hashing new values, you will
need an auxiliary scheme to detect this condition.

Unlike the original version, this version supports `#![no_std]` environments, and has several optimizations
specifically for older CPUs like the GBA -- namely using power-of-two sized bit vectors and simpler hash functions to
account for lack hardware of modulo support.