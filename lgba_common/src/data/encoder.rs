use crate::{
    base_encoder::BaseEncoder,
    base_repr::{SerialSlice, SerialStr},
    data::{
        fs_hash, load,
        loader::{LoadedDirectory, LoadedDirectoryNode, LoadedFilesystem, LoadedRoot},
        DataHeader, DirectoryData, DirectoryRoot, FileData, FilesystemDataInfo,
        FilesystemDataType, FilterManager, ParsedManifest,
    },
    hashed,
};
use anyhow::*;
use std::{
    collections::HashSet,
    format,
    path::Path,
    string::{String, ToString},
    vec::Vec,
};

#[derive(Debug)]
pub struct FilesystemEncoder {
    encoder: BaseEncoder,
}
impl FilesystemEncoder {
    pub fn new(base: usize) -> Self {
        FilesystemEncoder { encoder: BaseEncoder::new(base) }
    }
    pub fn set_usage_hint(&mut self, str: &str) {
        self.encoder.set_usage_hint(str);
    }
    pub fn iter_usage(&self) -> impl Iterator<Item = (&str, usize)> + '_ {
        self.encoder.iter_usage()
    }
    pub fn data(&self) -> &[u8] {
        self.encoder.data()
    }

    fn write_serial_bytes(&mut self, data: &[u8]) -> Result<SerialSlice<u8>> {
        let hash = hashed(data, 0);
        if !self.encoder.cached_objects.contains_key(&hash) {
            let ptr = self.encoder.encode_bytes(data)?;
            self.encoder.cached_objects.insert(hash, ptr);
        }
        Ok(SerialSlice {
            ptr: self.encoder.cached_objects[&hash] as u32,
            len: data.len() as u32,
            _phantom: Default::default(),
        })
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
        if !self.encoder.cached_objects.contains_key(&hash) {
            let new_data = match node {
                LoadedDirectoryNode::File(file) => {
                    let slice = self.write_serial_bytes(file.as_slice())?;
                    let offset = self.encoder.encode(&FileData { data: slice })?;
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
                    let child_names_start = self.encoder.align::<SerialStr>();
                    for string in &strings {
                        self.encoder.encode(string)?;
                    }
                    let child_names: SerialSlice<SerialStr> = SerialSlice {
                        ptr: child_names_start as u32,
                        len: strings.len() as u32,
                        _phantom: Default::default(),
                    };

                    // encode the child node list
                    let child_offsets: SerialSlice<FilesystemDataInfo> = if enable_file_names {
                        let child_offsets_start = self.encoder.align::<FilesystemDataInfo>();
                        for offset in &offsets {
                            self.encoder.encode(offset)?;
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
                    let offset = self
                        .encoder
                        .encode(&DirectoryData { child_names, child_offsets })?;
                    FilesystemDataInfo::new(FilesystemDataType::DirectoryData, offset as u32)
                }
                LoadedDirectoryNode::InvalidNode => {
                    FilesystemDataInfo::new(FilesystemDataType::Invalid, 0)
                }
            };
            self.encoder
                .cached_objects
                .insert(hash, new_data.0 as usize);
        }
        Ok(FilesystemDataInfo(self.encoder.cached_objects[&hash] as u32))
    }

    fn iter_nodes(
        &mut self,
        path: &str,
        node: &LoadedDirectoryNode,
        dir: &LoadedDirectory,
        root_index: &mut Vec<(u32, FilesystemDataInfo)>,
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
        if !self.encoder.cached_objects.contains_key(&hash) {
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

            // check PHF table for duplicates
            {
                let mut dupe_check = HashSet::new();
                dupe_check.extend(root_index.iter().map(|x| x.0));
                ensure!(dupe_check.len() == root_index.len(), "Hash collision in file table.");
                std::println!("{:?}", dupe_check);
            }

            // create the root PHF table
            std::println!("phf");
            let phf_offset = self.encoder.cur_offset() as u32;
            self.encoder
                .encode_bytes(&crate::phf::build_phf(phf_offset, &root_index))?;

            // create the root object
            let offset = self
                .encoder
                .encode(&DirectoryRoot { hash_lookup: phf_offset, root })?;
            let obj = FilesystemDataInfo::new(FilesystemDataType::DirectoryRoot, offset as u32);
            self.encoder.cached_objects.insert(hash, obj.0 as usize);
        }
        Ok(FilesystemDataInfo(self.encoder.cached_objects[&hash] as u32))
    }

    fn write_filesystem(
        &mut self,
        loaded: &LoadedFilesystem,
    ) -> Result<SerialSlice<FilesystemDataInfo>> {
        let hash = hashed(loaded, 3);
        if !self.encoder.cached_objects.contains_key(&hash) {
            let mut roots = Vec::new();
            for root in loaded.roots.values() {
                match root {
                    LoadedRoot::Directory(dir) => {
                        roots.push(self.write_directory_root(dir)?);
                    }
                    LoadedRoot::File(data) => {
                        roots.push(self.write_directory_node(
                            &LoadedDirectoryNode::File(data.clone()),
                            false,
                        )?);
                    }
                    LoadedRoot::MapU16(_) => todo!(),
                    LoadedRoot::MapU16U16(_) => todo!(),
                    LoadedRoot::MapU32(_) => todo!(),
                }
            }

            let offset = self.encoder.align::<LoadedDirectoryNode>();
            for root in roots {
                std::println!("{:x?}", root);
                self.encoder.encode(&root)?;
            }
            self.encoder.cached_objects.insert(hash, offset);
        }
        Ok(SerialSlice {
            ptr: self.encoder.cached_objects[&hash] as u32,
            len: loaded.roots.len() as u32,
            _phantom: Default::default(),
        })
    }

    pub fn load_filesystem(
        &mut self,
        root_dir: &Path,
        manifest: &ParsedManifest,
        filters: &FilterManager,
    ) -> Result<DataHeader> {
        let loaded = load(root_dir, manifest, filters)?;
        let roots = self.write_filesystem(&loaded)?;
        Ok(DataHeader { hash: manifest.hash(), roots })
    }
}
