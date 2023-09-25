// implementation note: Most of the actual code for this is in `lgba_common` because it needs to be
// shared between the GBA and the host system.

#![no_std]

use lgba_common::{
    common::ExHeader,
    data::{
        DataHeader, DirectoryData, DirectoryRoot, FileData, FilesystemDataInfo, FilesystemDataType,
    },
};

unsafe fn get_root(
    ty: FilesystemDataType,
    header: &'static ExHeader<DataHeader>,
    root_id: usize,
) -> u32 {
    let id = *header.data.roots.offset(root_id);
    if id.ty() != ty {
        fs_type_error(id.ty(), ty);
    }
    id.ptr()
}

/// Allows access to a singular file loaded into the ROM.
#[derive(Copy, Clone)]
pub struct FileAccess {
    header: &'static ExHeader<DataHeader>,
    root_id: usize,
}
impl FileAccess {
    /// Returns the file contents.
    pub fn as_slice(&self) -> &'static [u8] {
        unsafe {
            let data = get_root(FilesystemDataType::FileData, self.header, self.root_id);
            (*(data as *const FileData)).as_slice()
        }
    }
}

/// Allows access to a directory tree loaded into the ROM.
#[derive(Copy, Clone)]
pub struct DirAccess {
    header: &'static ExHeader<DataHeader>,
    root_id: usize,
}
impl DirAccess {
    unsafe fn dir_root(&self) -> &'static DirectoryRoot {
        let data = get_root(FilesystemDataType::FileData, self.header, self.root_id);
        &*(data as *const DirectoryRoot)
    }

    /// Returns the directory root if one is loaded.
    pub fn root(&self) -> Option<DirNodeAccess> {
        unsafe {
            let root = self.dir_root().root;
            if root == 0 {
                None
            } else {
                Some(DirNodeAccess { node: &*(root as *const _) })
            }
        }
    }
}

/// Represents a entry stored in a [`DirAccess`].
#[derive(Copy, Clone)]
pub enum NodeAccess {
    /// A directory entry.
    Directory(DirNodeAccess),
    /// A file entry.
    File(FileNodeAccess),
    /// A special file that is not supported by `lgba`.
    Invalid,
}
impl NodeAccess {
    fn for_data(data: FilesystemDataInfo) -> Self {
        unsafe {
            match data.ty() {
                FilesystemDataType::FileData => {
                    NodeAccess::File(FileNodeAccess { node: &*(data.ptr() as *const _) })
                }
                FilesystemDataType::DirectoryData => {
                    NodeAccess::Directory(DirNodeAccess { node: &*(data.ptr() as *const _) })
                }
                FilesystemDataType::Invalid => NodeAccess::Invalid,
                x => fs_node_type_error(x),
            }
        }
    }
}

/// Represents a file entry stored in a [`DirAccess`].
#[derive(Copy, Clone)]
pub struct FileNodeAccess {
    node: &'static FileData,
}
impl FileNodeAccess {
    /// Returns the file contents.
    pub fn as_slice(&self) -> &'static [u8] {
        unsafe { self.node.as_slice() }
    }
}

/// Represents a subdirectory entry stored in a [`DirAccess`].
#[derive(Copy, Clone)]
pub struct DirNodeAccess {
    node: &'static DirectoryData,
}
impl DirNodeAccess {
    /// Returns the file names of all children of this directory.
    pub fn child_names(&self) -> impl Iterator<Item = &'static str> {
        unsafe { self.node.iter_child_names() }
    }

    /// Returns the contents of all children of this directory.
    pub fn child_nodes(&self) -> impl Iterator<Item = NodeAccess> {
        unsafe { self.node.iter_child_nodes().map(NodeAccess::for_data) }
    }

    /// Returns all children of this directory.
    pub fn children(&self) -> impl Iterator<Item = (&'static str, NodeAccess)> {
        self.child_names().zip(self.child_nodes())
    }
}

#[inline(never)]
fn fs_type_error(found: FilesystemDataType, expected: FilesystemDataType) -> ! {
    panic!("Wrong object type. (found: {found:?}, expected: {expected:?})");
}

#[inline(never)]
fn fs_node_type_error(found: FilesystemDataType) -> ! {
    panic!("Wrong object type. (found: {found:?}, expected: FileData|DirectoryData|Invalid)");
}
