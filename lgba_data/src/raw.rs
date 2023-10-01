use crate::FileList;
use core::hash::Hash;
use lgba_common::{
    common::ExHeader,
    data::{DataHeader, RawStrHash, RomDataType, RomPartitionData, RomRoot},
};

/// Allows access to a specific entry located in a root.
#[derive(Copy, Clone)]
pub struct EntryAccess {
    partitions: &'static [RomPartitionData],
}
impl EntryAccess {
    /// Retrieves a partition's data by its ID.
    pub fn partition_by_id(&self, id: usize) -> FileList {
        FileList(unsafe { self.partitions[id].as_slice() })
    }
}

/// A type supported for a root's key.
pub trait ValidRootKey {
    type UnderlyingType: 'static + Copy + Eq + Hash;
    const ROM_TYPE: RomDataType;
    fn convert(&self) -> Self::UnderlyingType;
}
impl ValidRootKey for str {
    type UnderlyingType = RawStrHash;
    const ROM_TYPE: RomDataType = RomDataType::RootStr;
    fn convert(&self) -> Self::UnderlyingType {
        RawStrHash::new(self)
    }
}
macro_rules! simple_root_ty {
    ($ty:ty, $variant:ident) => {
        impl ValidRootKey for $ty {
            type UnderlyingType = $ty;
            const ROM_TYPE: RomDataType = RomDataType::$variant;
            fn convert(&self) -> Self::UnderlyingType {
                *self
            }
        }
    };
}
simple_root_ty!(u16, RootU16);
simple_root_ty!((u16, u16), RootU16U16);
simple_root_ty!(u32, RootU32);

/// Allows access to a specific root and its contents.
#[derive(Copy, Clone)]
pub struct RootAccess<T: ValidRootKey> {
    root: Option<&'static RomRoot<T::UnderlyingType>>,
}
impl<T: ValidRootKey> RootAccess<T> {
    pub unsafe fn new(header: &'static ExHeader<DataHeader>, root_id: usize) -> Self {
        let id = header.data.get_root(root_id);
        if id.ty() == RomDataType::NoFiles {
            RootAccess { root: None }
        } else if id.ty() == T::ROM_TYPE {
            RootAccess { root: Some(&*(id.ptr() as *const RomRoot<T::UnderlyingType>)) }
        } else {
            type_error()
        }
    }

    pub fn get(&self, key: T) -> Option<EntryAccess> {
        unsafe {
            self.root
                .as_ref()
                .and_then(|x| x.lookup(&T::convert(&key)))
                .map(|x| EntryAccess { partitions: x })
        }
    }
}

#[inline(never)]
fn type_error() -> ! {
    panic!("internal type error in lgba_data");
}
