// implementation note: Most of the actual code for this is in `lgba_common` because it needs to be
// shared between the GBA and the host system.

#![no_std]

use core::{
    hash::{Hash, Hasher},
    ops::Index,
};
use fnv::FnvHasher;
use lgba_common::data::{FileData, RawStrHash};

mod raw;

/// **NOT** public API!! Only for this crate's macros.
#[doc(hidden)]
pub mod __macro_export {
    pub use crate::{
        raw::{EntryAccess, RootAccess, ValidRootKey},
        FileList, StrHash,
    };
    pub use core::marker::PhantomData;
    pub use lgba_common::{
        common::{ExHeader, SerialSlice},
        data::{DataHeader, RawStrHash},
    };
    pub use lgba_macros::{hash_impl, load_data_impl};

    pub fn new_hash(v: u32) -> StrHash {
        StrHash(v)
    }

    #[inline(never)]
    pub fn not_found<T: core::fmt::Debug>(entry: T, source: &str) -> ! {
        panic!("Entry '{entry:?}' not found in {source}")
    }
}

/// A 32-bit hash of a string, used to index game data.
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
#[repr(transparent)]
pub struct StrHash(u32);
impl StrHash {
    /// Creates a new [`StrHash`] from a string.
    pub fn new(name: &str) -> StrHash {
        let mut hash = FnvHasher::with_key(123456001);
        for path in name.split('/') {
            if !path.is_empty() {
                path.hash(&mut hash);
            }
        }
        StrHash(hash.finish() as u32)
    }
}
impl<'a> From<&'a str> for StrHash {
    fn from(value: &'a str) -> Self {
        StrHash::new(value)
    }
}
impl From<StrHash> for RawStrHash {
    fn from(value: StrHash) -> Self {
        RawStrHash(value.0)
    }
}

/// Allows access to a list of files.
#[repr(transparent)]
pub struct FileList(&'static [FileData]);
impl FileList {
    /// Returns the only value in this list, or else panics if there are multiple values or none.
    pub fn as_slice(&self) -> &'static [u8] {
        if self.0.len() == 0 {
            empty_list_error()
        } else if self.0.len() > 1 {
            multiple_items_error()
        } else {
            self.get(0)
        }
    }

    /// Returns a static reference to a given file.
    pub fn get(&self, index: usize) -> &'static [u8] {
        unsafe { self.0[index].as_slice() }
    }

    /// Returns `true` if the list is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the length of the file list
    pub fn len(&self) -> usize {
        self.0.len()
    }
}
impl Index<usize> for FileList {
    type Output = [u8];
    fn index(&self, index: usize) -> &Self::Output {
        unsafe { self.0[index].as_slice() }
    }
}

#[inline(never)]
fn empty_list_error() -> ! {
    panic!("`FileList` contains no entries!");
}

#[inline(never)]
fn multiple_items_error() -> ! {
    panic!("`FileList` contains multiple entries!");
}

#[macro_export]
macro_rules! StrHash {
    ($str:literal $(,)?) => {
        $crate::__macro_export::hash_impl!($str)
    };
}

#[macro_export]
macro_rules! load_data {
    ($vis:vis $name:ident, $source:literal $(,)?) => {
        $crate::__macro_export::load_data_impl!($vis $name, $source);
    };
}
