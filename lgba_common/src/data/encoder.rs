use crate::{
    base_repr::{SerialSlice, SerialStr},
    data::{
        fs_hash, hashed,
        loader::{LoadedDirectory, LoadedDirectoryNode, LoadedFilesystem, LoadedRoot},
        DirectoryData, DirectoryRoot, FileData, FilesystemDataInfo, FilesystemDataType,
    },
};
use anyhow::Result;
use serde::Serialize;
use std::{
    collections::HashMap,
    format,
    string::{String, ToString},
    vec,
    vec::Vec,
};

pub struct DataEncoder {
    base: usize, // should actually be u32, but, convenience
    data: Vec<u8>,
    usage: HashMap<String, usize>,
    usage_hint: String,
    cached_objects: HashMap<[u8; 32], usize>,
}
impl DataEncoder {
    pub fn new(base: usize) -> Self {
        DataEncoder {
            base,
            data: vec![],
            usage: Default::default(),
            usage_hint: "Game Data".to_string(),
            cached_objects: Default::default(),
        }
    }
    pub fn set_usage_hint(&mut self, str: &str) {
        self.usage_hint = str.to_string();
    }
    pub fn iter_usage(&self) -> impl Iterator<Item = (&str, usize)> + '_ {
        self.usage.iter().map(|x| (x.0.as_str(), *x.1))
    }

    fn cur_offset(&self) -> usize {
        self.base + self.data.len()
    }
    fn mark_usage(&mut self, start_offset: usize) {
        if self.usage.contains_key(&self.usage_hint) {
            *self.usage.get_mut(&self.usage_hint).unwrap() += self.cur_offset() - start_offset;
        } else {
            self.usage
                .insert(self.usage_hint.clone(), self.cur_offset() - start_offset);
        }
    }

    fn encode<T: Serialize>(&mut self, data: &T) -> Result<()> {
        let start_off = self.cur_offset();
        let start = self.data.len();
        self.data.resize(start + std::mem::size_of::<T>(), 0);
        let written = ssmarshal::serialize(&mut self.data[start..], data)?;
        assert_eq!(written, std::mem::size_of::<T>());
        self.mark_usage(start_off);
        Ok(())
    }
    fn encode_bytes(&mut self, data: &[u8]) -> Result<()> {
        let start_off = self.cur_offset();
        let start = self.data.len();
        self.data.resize(start + data.len(), 0);
        self.data[start..].copy_from_slice(data);
        self.mark_usage(start_off);
        Ok(())
    }

    fn write_serial_bytes(&mut self, data: &[u8]) -> Result<SerialSlice<u8>> {
        let hash = hashed(data, 0);
        if self.cached_objects.contains_key(&hash) {
            Ok(SerialSlice {
                ptr: self.cached_objects[&hash] as u32,
                len: data.len() as u32,
                _phantom: Default::default(),
            })
        } else {
            let ptr = self.cur_offset();
            self.encode_bytes(data)?;
            self.cached_objects.insert(hash, ptr);
            Ok(SerialSlice {
                ptr: ptr as u32,
                len: data.len() as u32,
                _phantom: Default::default(),
            })
        }
    }
    fn write_serial_str(&mut self, data: &str) -> Result<SerialStr> {
        let slice = self.write_serial_bytes(data.as_bytes())?;
        Ok(SerialStr { ptr: slice.ptr, len: slice.len })
    }

    fn write_directory_node(
        &mut self,
        node: &LoadedDirectoryNode,
        enable_file_names: bool,
    ) -> Result<FilesystemDataInfo> {
        let hash = hashed(&(node, enable_file_names), 1);
        if self.cached_objects.contains_key(&hash) {
            Ok(FilesystemDataInfo(self.cached_objects[&hash] as u32))
        } else {
            let new_data = match node {
                LoadedDirectoryNode::File(file) => {
                    let slice = self.write_serial_bytes(file.as_slice())?;
                    let offset = self.cur_offset();
                    self.encode(&FileData { data: slice })?;
                    FilesystemDataInfo::new(FilesystemDataType::FileData, offset as u32)
                }
                LoadedDirectoryNode::Directory(dir) => {
                    // encode all parent nodes
                    let mut offsets = Vec::new();
                    for offset in dir.values() {
                        offsets.push(self.write_directory_node(offset, enable_file_names)?);
                    }

                    // encode all filename strings
                    let mut strings = Vec::new();
                    if enable_file_names {
                        for name in dir.keys() {
                            strings.push(self.write_serial_str(&name)?);
                        }
                    }

                    // encode the child names list
                    let mut child_names_start = self.cur_offset();
                    for string in &strings {
                        self.encode(string)?;
                    }
                    let child_names: SerialSlice<SerialStr> = SerialSlice {
                        ptr: child_names_start as u32,
                        len: strings.len() as u32,
                        _phantom: Default::default(),
                    };

                    // encode the child node list
                    let child_offsets: SerialSlice<FilesystemDataInfo> = if enable_file_names {
                        let mut child_offsets_start = self.cur_offset();
                        for offset in &offsets {
                            self.encode(offset)?;
                        }
                        SerialSlice {
                            ptr: child_offsets_start as u32,
                            len: offsets.len() as u32,
                            _phantom: Default::default(),
                        }
                    } else {
                        SerialSlice { ptr: 0, len: 0, _phantom: Default::default() }
                    };

                    // encode the actual filesystem data offset
                    let offset = self.cur_offset();
                    self.encode(&DirectoryData { child_names, child_offsets })?;
                    FilesystemDataInfo::new(FilesystemDataType::DirectoryData, offset as u32)
                }
                LoadedDirectoryNode::InvalidNode => {
                    FilesystemDataInfo::new(FilesystemDataType::Invalid, 0)
                }
            };
            self.cached_objects.insert(hash, new_data.0 as usize);
            Ok(new_data)
        }
    }
    fn iter_nodes(
        &mut self,
        path: &str,
        node: &LoadedDirectoryNode,
        dir: &LoadedDirectory,
        root_index: &mut Vec<(u64, FilesystemDataInfo)>,
    ) -> Result<()> {
        fn compose_name(a: &str, b: &str) -> String {
            if a.is_empty() {
                b.to_string()
            } else {
                format!("{a}/{b}")
            }
        }

        if !matches!(node, LoadedDirectoryNode::Directory(_)) || dir.enable_dir_listing {
            let encoded = self.write_directory_node(node, dir.enable_file_names)?;
            root_index.push((fs_hash(path), encoded));
        }
        if let LoadedDirectoryNode::Directory(entries) = node {
            for (name, node) in entries {
                self.iter_nodes(&compose_name(path, name), node, dir, root_index)?;
            }
        }

        Ok(())
    }
    fn write_directory_root(&mut self, dir: &LoadedDirectory) -> Result<FilesystemDataInfo> {
        let hash = hashed(dir, 2);
        if self.cached_objects.contains_key(&hash) {
            Ok(FilesystemDataInfo(self.cached_objects[&hash] as u32))
        } else {
            let mut root_index = Vec::new();

            // create the full list of hashes and node offsets
            self.iter_nodes("", &dir.root, dir, &mut root_index)?;

            // create the root directory reference (if requested)
            let root = if dir.enable_dir_listing {
                self.write_directory_node(&dir.root, dir.enable_file_names)?
                    .ptr()
            } else {
                0
            };

            // create the root PHF table
            let phf_offset = self.cur_offset() as u32;
            self.encode_bytes(&crate::phf::build_phf(phf_offset, &root_index))?;

            // create the root object
            let offset = self.cur_offset();
            self.encode(&DirectoryRoot { hash_lookup: phf_offset, root })?;
            let obj = FilesystemDataInfo::new(FilesystemDataType::DirectoryRoot, offset as u32);
            self.cached_objects.insert(hash, obj.0 as usize);
            Ok(obj)
        }
    }

    pub fn write_filesystem(
        &mut self,
        loaded: &LoadedFilesystem,
    ) -> Result<SerialSlice<FilesystemDataInfo>> {
        let hash = hashed(loaded, 3);
        if self.cached_objects.contains_key(&hash) {
            Ok(SerialSlice {
                ptr: self.cached_objects[&hash] as u32,
                len: loaded.roots.len() as u32,
                _phantom: Default::default(),
            })
        } else {
            let mut roots = Vec::new();
            for root in loaded.roots.values() {
                match root {
                    LoadedRoot::Directory(dir) => {
                        roots.push(self.write_directory_root(dir)?);
                    }
                    LoadedRoot::File(_) => todo!(),
                    LoadedRoot::MapU16(_) => todo!(),
                    LoadedRoot::MapU16U16(_) => todo!(),
                    LoadedRoot::MapU32(_) => todo!(),
                }
            }

            Ok(SerialSlice { ptr: 0, len: 0, _phantom: Default::default() })
        }
    }
}
