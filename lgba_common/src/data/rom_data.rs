use crate::{
    common::{ExHeader, ExHeaderType, SerialSlice, SerialStr},
    phf::PhfTable,
};
use core::hash::{Hash, Hasher};
use fnv::FnvHasher;
use num_enum::TryFromPrimitive;
#[cfg(feature = "data_build")]
use serde::{Deserialize, Serialize};

/// A marker type used to store the data type of a filesystem node.
#[cfg_attr(feature = "data_build", derive(Serialize, Deserialize))]
#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, TryFromPrimitive)]
#[repr(u8)]
pub enum FilesystemDataType {
    /// A file data node. Points to a [`FileData`].
    FileData,
    /// A directory data node. Points to a [`DirectoryData`].
    DirectoryData,
    /// A directory root node. Points to a [`DirectoryRoot`].
    DirectoryRoot,
    /// An invalid directory entry. Does not point to anything.
    Invalid,
    /// A PHF ID node. Points to a [`PhfTable<u16, FilesystemDataInfo>`].
    PhfU16,
    /// A PHF ID node. Points to a [`PhfTable<(u16, u16), FilesystemDataInfo>`].
    PhfU16U16,
    /// A PHF ID node. Points to a [`PhfTable<u32, FilesystemDataInfo>`].
    PhfU32,
}

/// Stores the data for a single file.
#[cfg_attr(feature = "data_build", derive(Serialize, Deserialize))]
#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct FileData {
    /// The contents of the file.s
    pub data: SerialSlice<u8>,
}
impl FileData {
    pub unsafe fn as_slice(&self) -> &'static [u8] {
        self.data.as_slice()
    }
}

/// Stores the data for a single directory.
#[cfg_attr(feature = "data_build", derive(Serialize, Deserialize))]
#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct DirectoryData {
    /// A list of file names, in order.
    pub child_names: SerialSlice<SerialStr>,

    /// A list of offsets to child nodes, in order.
    ///
    /// Can be `FileData`, `DirectoryData`, or `Invalid`s.
    pub child_offsets: SerialSlice<FilesystemDataInfo>,
}
impl DirectoryData {
    pub unsafe fn iter_child_names(&self) -> impl Iterator<Item = &'static str> {
        if self.child_names.ptr == 0 {
            panic!("children names not available")
        }

        self.child_names.as_slice().iter().map(|x| x.as_str())
    }

    pub unsafe fn iter_child_nodes(&self) -> impl Iterator<Item = FilesystemDataInfo> {
        self.child_offsets.as_slice().iter().map(|x| *x)
    }

    pub unsafe fn iter(&self) -> impl Iterator<Item = (&'static str, FilesystemDataInfo)> {
        self.iter_child_names().zip(self.iter_child_nodes())
    }
}

#[cfg_attr(feature = "data_build", derive(Serialize, Deserialize))]
#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct DirectoryRoot {
    /// The offset of the hash lookup table.
    ///
    /// This points to a `PhfTable<u32, FilesystemDataInfo>`
    pub hash_lookup: u32,
    /// The root directory. Points to a [`DirectoryData`].
    pub root: u32,
}
impl DirectoryRoot {
    pub unsafe fn lookup(&self, hash: u32) -> Option<FilesystemData> {
        let hash_lookup = self.hash_lookup as *const PhfTable<u32, FilesystemDataInfo>;
        (*hash_lookup).lookup(&hash).map(|x| x.decode())
    }

    pub unsafe fn root(&self) -> &DirectoryData {
        if self.root == 0 {
            root_not_found()
        } else {
            &*(self.root() as *const DirectoryData)
        }
    }
}

#[inline(never)]
fn root_not_found() -> ! {
    panic!("DirectoryRoot has no root listing enabled!")
}

/// A typed [`FilesystemDataInfo`], used internally to decode them.
#[derive(Copy, Clone, Debug)]
pub enum FilesystemData {
    FileData(&'static FileData),
    DirectoryData(&'static DirectoryData),
    DirectoryRoot(&'static DirectoryRoot),
    Invalid,

    // phf types
    PhfU16(&'static PhfTable<u16, FilesystemDataInfo>),
    PhfU16U16(&'static PhfTable<(u16, u16), FilesystemDataInfo>),
    PhfU32(&'static PhfTable<u32, FilesystemDataInfo>),
}

/// A pointer to a typed filesystem node.
///
/// The type is compacted into the top 7 bits, and the offset into the bottom 7. This works by
/// assuming the pointer is to the unmirrored ROM mapping.
#[cfg_attr(feature = "data_build", derive(Serialize, Deserialize))]
#[derive(Copy, Clone, Debug)]
#[repr(transparent)]
pub struct FilesystemDataInfo(pub u32);
impl FilesystemDataInfo {
    pub fn new(ty: FilesystemDataType, ptr: u32) -> Self {
        assert_eq!(ptr & 0xFE000000, 0x08000000, "ptr must be in 0x8000000-0x9FFFFFF range.");
        FilesystemDataInfo(ptr & 0x01FFFFFF | ((ty as u32) << 25))
    }

    pub fn ptr(&self) -> u32 {
        (self.0 & 0x01FFFFFF) | 0x08000000
    }

    pub fn ty(&self) -> FilesystemDataType {
        FilesystemDataType::try_from((self.0 >> 25) as u8).unwrap()
    }

    pub unsafe fn decode(&self) -> FilesystemData {
        match self.ty() {
            FilesystemDataType::FileData => {
                FilesystemData::FileData(&*(self.ptr() as usize as *const _))
            }
            FilesystemDataType::DirectoryData => {
                FilesystemData::DirectoryData(&*(self.ptr() as usize as *const _))
            }
            FilesystemDataType::DirectoryRoot => {
                FilesystemData::DirectoryRoot(&*(self.ptr() as usize as *const _))
            }
            FilesystemDataType::Invalid => FilesystemData::Invalid,
            FilesystemDataType::PhfU16 => {
                FilesystemData::PhfU16(&*(self.ptr() as usize as *const _))
            }
            FilesystemDataType::PhfU16U16 => {
                FilesystemData::PhfU16U16(&*(self.ptr() as usize as *const _))
            }
            FilesystemDataType::PhfU32 => {
                FilesystemData::PhfU32(&*(self.ptr() as usize as *const _))
            }
        }
    }
}

/// The contents of the exheader for game data.
#[cfg_attr(feature = "data_build", derive(Serialize, Deserialize))]
#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct DataHeader {
    /// The hash derived from the manifest.
    pub hash: [u8; 12],
    /// An array of roots for the manifest.
    pub roots: SerialSlice<FilesystemDataInfo>,
}
impl DataHeader {
    pub const fn new(hash: [u8; 12]) -> DataHeader {
        DataHeader { hash, roots: SerialSlice::default() }
    }

    pub unsafe fn get_root(&self, root_idx: usize) -> FilesystemData {
        if self.roots.ptr == 0 {
            panic!("data root not available. Please use lgba_romtool.");
        }
        (*self.roots.offset(root_idx)).decode()
    }
}
impl ExHeaderType for DataHeader {
    const NAME: [u8; 4] = *b"data";
    const VERSION: u16 = 0;
}

/// Hashes a file name for use in the lookup table of a directory root.
pub fn fs_hash(name: &str) -> u32 {
    let mut hash = FnvHasher::with_key(123456001);
    for path in name.split('/') {
        if !path.is_empty() {
            path.hash(&mut hash);
        }
    }
    hash.finish() as u32
}
