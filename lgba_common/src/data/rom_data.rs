use crate::{
    common::{ExHeaderType, SerialSlice},
    phf::PhfTable,
};
use core::{
    hash::{Hash, Hasher},
    marker::PhantomData,
};
use fnv::FnvHasher;
use num_enum::TryFromPrimitive;
#[cfg(feature = "data_build")]
use serde::{Deserialize, Serialize};

/// The data underlying a PHF root.
#[cfg_attr(feature = "data_build", derive(Serialize, Deserialize))]
#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct RomRoot<T: Eq + Hash> {
    /// The size of the bare array `table` points to.
    pub partition_count: u32,

    /// The underlying PHF data.
    ///
    /// Key is `T`.
    /// Value is a pointer to a bare array of [`RomDataInfo`] objects.
    pub table: u32,

    /// phantom data
    pub _phantom: PhantomData<T>,
}
impl<T: Eq + Hash> RomRoot<T> {
    pub unsafe fn lookup(&self, t: &T) -> Option<&'static [RomPartitionData]> {
        match (*(self.table as *const PhfTable<T, u32>)).lookup(t) {
            None => None,
            Some(v) => {
                Some(core::slice::from_raw_parts(*v as *const _, self.partition_count as usize))
            }
        }
    }
}

#[derive(Copy, Clone, Debug)]
#[repr(transparent)]
pub struct RomPartitionData(RomDataInfo);
impl RomPartitionData {
    pub unsafe fn as_slice(&self) -> &'static [FileData] {
        match self.0.decode() {
            RomData::NoFiles => &[],
            RomData::FileData(v) => core::slice::from_ref(v),
            RomData::FileList(v) => v.as_slice(),
            _ => type_error(),
        }
    }
}

/// Stores the hash for a string name, for use in PhfData.
#[cfg_attr(feature = "data_build", derive(Serialize, Deserialize))]
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
#[repr(transparent)]
pub struct RawStrHash(pub u32);
impl RawStrHash {
    pub fn new(name: &str) -> RawStrHash {
        let mut hash = FnvHasher::with_key(123456001);
        for path in name.split('/') {
            if !path.is_empty() {
                path.hash(&mut hash);
            }
        }
        RawStrHash(hash.finish() as u32)
    }
}

/// A marker type used to store the data type of a filesystem node.
#[cfg_attr(feature = "data_build", derive(Serialize, Deserialize))]
#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, TryFromPrimitive)]
#[repr(u8)]
pub enum RomDataType {
    /// There are no files present.
    NoFiles,
    /// The data for a single file.
    FileData,
    /// The data for multiple files.
    FileList,
    /// A PHF string node. Points to a [`RomRoot<RawStrHash>`].
    RootStr,
    /// A PHF ID node. Points to a [`RomRoot<u16>`].
    RootU16,
    /// A PHF ID node. Points to a [`RomRoot<(u16, u16)>`].
    RootU16U16,
    /// A PHF ID node. Points to a [`RomRoot<u32>`].
    RootU32,
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

/// A typed [`RomDataInfo`], used internally to decode them.
#[derive(Copy, Clone, Debug)]
pub enum RomData {
    NoFiles,
    FileData(&'static FileData),
    FileList(&'static FileList),
    PhfStr(&'static RomRoot<RawStrHash>),
    PhfU16(&'static RomRoot<u16>),
    PhfU16U16(&'static RomRoot<(u16, u16)>),
    PhfU32(&'static RomRoot<u32>),
}

/// A pointer to a typed filesystem node.
///
/// The type is compacted into the top 7 bits, and the offset into the bottom 7. This works by
/// assuming the pointer is to the unmirrored ROM mapping.
#[cfg_attr(feature = "data_build", derive(Serialize, Deserialize))]
#[derive(Copy, Clone, Debug)]
#[repr(transparent)]
pub struct RomDataInfo(pub u32);
impl RomDataInfo {
    pub fn new(ty: RomDataType, ptr: u32) -> Self {
        assert_eq!(ptr & 0xFE000000, 0x08000000, "ptr must be in 0x8000000-0x9FFFFFF range.");
        RomDataInfo(ptr & 0x01FFFFFF | ((ty as u32) << 25))
    }

    pub fn ptr(&self) -> u32 {
        (self.0 & 0x01FFFFFF) | 0x08000000
    }

    pub fn ty(&self) -> RomDataType {
        RomDataType::try_from((self.0 >> 25) as u8).unwrap()
    }

    pub unsafe fn decode(&self) -> RomData {
        match self.ty() {
            RomDataType::NoFiles => RomData::NoFiles,
            RomDataType::FileList => RomData::FileData(&*(self.ptr() as usize as *const _)),
            RomDataType::RootStr => RomData::FileData(&*(self.ptr() as usize as *const _)),
            RomDataType::FileData => RomData::FileData(&*(self.ptr() as usize as *const _)),
            RomDataType::RootU16 => RomData::PhfU16(&*(self.ptr() as usize as *const _)),
            RomDataType::RootU16U16 => RomData::PhfU16U16(&*(self.ptr() as usize as *const _)),
            RomDataType::RootU32 => RomData::PhfU32(&*(self.ptr() as usize as *const _)),
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
    pub roots: SerialSlice<RomDataInfo>,
}
impl DataHeader {
    pub const fn new(hash: [u8; 12]) -> DataHeader {
        DataHeader { hash, roots: SerialSlice::default() }
    }

    pub unsafe fn get_root(&self, root_idx: usize) -> RomDataInfo {
        if self.roots.ptr == 0 {
            panic!("data root not available. Please use lgba_romtool.");
        }
        *self.roots.offset(root_idx)
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
