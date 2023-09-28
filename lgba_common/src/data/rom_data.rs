use crate::{
    common::{ExHeader, ExHeaderType, SerialSlice, SerialStr},
    phf::PhfTable,
};
use core::hash::{Hash, Hasher};
use fnv::FnvHasher;
use num_enum::TryFromPrimitive;
#[cfg(feature = "data_build")]
use serde::{Deserialize, Serialize};

/// The data underlying a PHF root.
#[cfg_attr(feature = "data_build", derive(Serialize, Deserialize))]
#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct PhfData<T: Eq + Hash> {
    /// The size of the bare array `table` points to.
    pub partition_count: u32,

    /// The underlying PHF data.
    ///
    /// Pointer to a bare array of [`FilesystemDataInfo`] objects.
    pub table: PhfTable<T, u32>,
}
impl<T: Eq + Hash> PhfData<T> {
    pub unsafe fn lookup(&self, t: &T) -> Option<&'static [PhfDataEntry]> {
        match self.table.lookup(t) {
            None => None,
            Some(v) => {
                Some(core::slice::from_raw_parts(*v as *const _, self.partition_count as usize))
            }
        }
    }
}

#[derive(Copy, Clone, Debug)]
#[repr(transparent)]
pub struct PhfDataEntry(FilesystemDataInfo);
impl PhfDataEntry {
    pub unsafe fn as_slice(&self) -> &'static [FileData] {
        match self.0.decode() {
            FilesystemData::NoFiles => &[],
            FilesystemData::FileData(v) => core::slice::from_ref(v),
            FilesystemData::FileList(v) => v.as_slice(),
            _ => type_error(),
        }
    }
}

/// Stores the hash for a string name, for use in PhfData.
#[cfg_attr(feature = "data_build", derive(Serialize, Deserialize))]
#[derive(Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
#[repr(transparent)]
pub struct StrHash(u32);
impl StrHash {
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

/// A marker type used to store the data type of a filesystem node.
#[cfg_attr(feature = "data_build", derive(Serialize, Deserialize))]
#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, TryFromPrimitive)]
#[repr(u8)]
pub enum FilesystemDataType {
    /// There are no files present.
    NoFiles,
    /// The data for a single file.
    FileData,
    /// The data for multiple files.
    FileList,
    /// A PHF string node. Points to a [`PhfData<StrHash>`].
    PhfStr,
    /// A PHF ID node. Points to a [`PhfData<u16>`].
    PhfU16,
    /// A PHF ID node. Points to a [`PhfData<(u16, u16)>`].
    PhfU16U16,
    /// A PHF ID node. Points to a [`PhfData<u32>`].
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

/// Stores the data for a list of files.
#[cfg_attr(feature = "data_build", derive(Serialize, Deserialize))]
#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct FileList {
    /// The contents of the file.s
    pub data: SerialSlice<FileData>,
}
impl FileList {
    pub unsafe fn as_slice(&self) -> &'static [FileData] {
        self.data.as_slice()
    }
}

/// A typed [`FilesystemDataInfo`], used internally to decode them.
#[derive(Copy, Clone, Debug)]
pub enum FilesystemData {
    NoFiles,
    FileData(&'static FileData),
    FileList(&'static FileList),
    PhfStr(&'static PhfData<StrHash>),
    PhfU16(&'static PhfData<u16>),
    PhfU16U16(&'static PhfData<(u16, u16)>),
    PhfU32(&'static PhfData<u32>),
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
            FilesystemDataType::NoFiles => FilesystemData::NoFiles,
            FilesystemDataType::FileList => {
                FilesystemData::FileData(&*(self.ptr() as usize as *const _))
            }
            FilesystemDataType::PhfStr => {
                FilesystemData::FileData(&*(self.ptr() as usize as *const _))
            }
            FilesystemDataType::FileData => {
                FilesystemData::FileData(&*(self.ptr() as usize as *const _))
            }
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

#[inline(never)]
fn type_error() -> ! {
    panic!("internal type error in lgba_data");
}
