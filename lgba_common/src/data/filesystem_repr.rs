use crate::{
    base_repr::{ExHeader, ExHeaderType, SerialSlice},
    phf::PhfTable,
};
use core::hash::{Hash, Hasher};
use fnv::FnvHasher;
#[cfg(feature = "generator")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "generator", derive(Serialize, Deserialize))]
#[derive(Copy, Clone, Debug)]
#[repr(u8)]
pub enum FilesystemDataType {
    FileData,
    DirectoryData,
    DirectoryRoot,
    PhfU16,
    PhfU16U16,
    PhfU32,
}

#[cfg_attr(feature = "generator", derive(Serialize, Deserialize))]
#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct FileData {
    pub data: SerialSlice<u8>,
}
impl FileData {
    pub unsafe fn as_slice(&self) -> &'static [u8] {
        self.data.as_slice()
    }
}

#[cfg_attr(feature = "generator", derive(Serialize, Deserialize))]
#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct DirectoryData {
    pub child_names: SerialSlice<SerialSlice<u8>>,
    pub child_offsets: SerialSlice<u32>,
}
impl DirectoryData {
    pub unsafe fn iter_children(&self) -> impl Iterator<Item = &'static str> {
        if self.child_names.ptr == 0 {
            panic!("children names not available")
        }

        self.child_names
            .as_slice()
            .iter()
            .map(|x| core::str::from_utf8_unchecked(x.as_slice()))
    }

    pub unsafe fn iter_offsets(&self) -> impl Iterator<Item = u32> {
        self.child_offsets.as_slice().iter().map(|x| *x)
    }

    pub unsafe fn iter(&self) -> impl Iterator<Item = (&'static str, u32)> {
        self.iter_children().zip(self.iter_offsets())
    }
}

#[cfg_attr(feature = "generator", derive(Serialize, Deserialize))]
#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct DirectoryRoot {
    pub entries: PhfTable<u64, FilesystemDataInfo>,
    pub root: DirectoryData,
}

#[derive(Copy, Clone, Debug)]
pub enum FilesystemData {
    FileData(&'static FileData),
    DirectoryData(&'static DirectoryData),
    DirectoryRoot(&'static DirectoryRoot),
    PhfU16(&'static PhfTable<u16, FilesystemDataInfo>),
    PhfU16U16(&'static PhfTable<(u16, u16), FilesystemDataInfo>),
    PhfU32(&'static PhfTable<u32, FilesystemDataInfo>),
}

#[cfg_attr(feature = "generator", derive(Serialize, Deserialize))]
#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct FilesystemDataInfo {
    pub ty: FilesystemDataType,
    pub _padding: [u8; 3], // explicit padding for ssmarshal
    pub ptr: u32,
}
impl FilesystemDataInfo {
    pub const fn new(ty: FilesystemDataType, ptr: u32) -> Self {
        FilesystemDataInfo { ty, _padding: [0; 3], ptr }
    }

    pub unsafe fn decode(&self) -> FilesystemData {
        match self.ty {
            FilesystemDataType::FileData => {
                FilesystemData::FileData(&*(self.ptr as usize as *const _))
            }
            FilesystemDataType::DirectoryData => {
                FilesystemData::DirectoryData(&*(self.ptr as usize as *const _))
            }
            FilesystemDataType::DirectoryRoot => {
                FilesystemData::DirectoryRoot(&*(self.ptr as usize as *const _))
            }
            FilesystemDataType::PhfU16 => {
                FilesystemData::PhfU16(&*(self.ptr as usize as *const _))
            }
            FilesystemDataType::PhfU16U16 => {
                FilesystemData::PhfU16U16(&*(self.ptr as usize as *const _))
            }
            FilesystemDataType::PhfU32 => {
                FilesystemData::PhfU32(&*(self.ptr as usize as *const _))
            }
        }
    }
}

#[cfg_attr(feature = "generator", derive(Serialize, Deserialize))]
#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct DataHeader {
    pub hash: u64,
    pub roots: SerialSlice<FilesystemDataInfo>,
}
impl DataHeader {
    pub const fn new(hash: u64) -> DataHeader {
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

pub struct DataRoot {
    pub header: &'static ExHeader<DataHeader>,
}

pub fn fs_hash(name: &str) -> u64 {
    let mut hash = FnvHasher::with_key(123456001);
    for path in name.split('/') {
        if !path.is_empty() {
            path.hash(&mut hash);
        }
    }
    hash.finish()
}
