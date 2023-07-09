use crate::base_repr::{ExHeader, ExHeaderType, SerialSlice};

#[derive(Copy, Clone, Debug)]
#[repr(u8)]
pub enum FilesystemDataType {
    FileData,
    PhfU16,
    PhfU32,
}

#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct FilesystemData {
    ty: FilesystemDataType,
    ptr: u32,
}

#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct DataHeader {
    pub hash: u64,
    pub data: SerialSlice<FilesystemData>,
}
impl DataHeader {
    pub const fn new(hash: u64) -> DataHeader {
        DataHeader { hash, data: SerialSlice::default() }
    }
}
impl ExHeaderType for DataHeader {
    const NAME: [u8; 4] = *b"meta";
    const VERSION: u16 = 0;
}

pub struct DataRoot {
    header: &'static ExHeader<DataHeader>,
}
