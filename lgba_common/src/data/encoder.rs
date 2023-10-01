use crate::{
    common::SerialSlice,
    data::{
        load,
        loader::{LoadedEntry, LoadedFilesystem, LoadedRoot},
        DataHeader, FileData, FileList, FilterManager, ParsedManifest, RomDataInfo, RomDataType,
        RomRoot,
    },
    encoder::BaseEncoder,
    hashes::hashed,
};
use anyhow::*;
use serde::Serialize;
use std::{
    collections::BTreeMap, fmt::Debug, hash::Hash, marker::PhantomData, path::Path, vec::Vec,
};

// TODO: Increase amount of caching.

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

    fn pre_encode_file_data_typed<T: Ord + Eq>(
        &mut self,
        data: &BTreeMap<T, LoadedEntry>,
    ) -> Result<()> {
        for (_, entry) in data {
            for (_, files) in &entry.partitions {
                for file in files {
                    self.write_serial_bytes(&file)?;
                }
            }
        }
        Ok(())
    }
    fn pre_encode_file_data(&mut self, loaded: &LoadedFilesystem) -> Result<()> {
        for (_, root) in &loaded.roots {
            match root {
                LoadedRoot::Empty => {}
                LoadedRoot::MapStr(map) => self.pre_encode_file_data_typed(map)?,
                LoadedRoot::MapU16(map) => self.pre_encode_file_data_typed(map)?,
                LoadedRoot::MapU16U16(map) => self.pre_encode_file_data_typed(map)?,
                LoadedRoot::MapU32(map) => self.pre_encode_file_data_typed(map)?,
            }
        }
        Ok(())
    }
    fn encode_file_list(&mut self, data: &[Vec<u8>]) -> Result<RomDataInfo> {
        if data.is_empty() {
            Ok(RomDataInfo::new(RomDataType::NoFiles, 0x8000000))
        } else if data.len() == 1 {
            let data = self.write_serial_bytes(&data[0])?;
            let offset = self.encoder.encode(&FileData { data })?;
            Ok(RomDataInfo::new(RomDataType::FileData, offset as u32))
        } else {
            let list_offset = self.encoder.cur_offset();
            for file in data {
                let data = self.write_serial_bytes(file)?;
                self.encoder.encode(&FileData { data })?;
            }

            let offset = self.encoder.encode(&FileList {
                data: SerialSlice {
                    ptr: list_offset as u32,
                    len: data.len() as u32,
                    _phantom: Default::default(),
                },
            })?;

            Ok(RomDataInfo::new(RomDataType::FileList, offset as u32))
        }
    }
    fn encode_entry(&mut self, data: &LoadedEntry) -> Result<u32> {
        let mut encoded = Vec::new();
        for (_, partition) in &data.partitions {
            encoded.push(self.encode_file_list(partition)?);
        }

        let offset = self.encoder.cur_offset();
        for encoded in encoded {
            self.encoder.encode(&encoded)?;
        }
        Ok(offset as u32)
    }
    fn encode_root_typed<T: Copy + Ord + Eq + Hash + Serialize + Debug>(
        &mut self,
        data: &BTreeMap<T, LoadedEntry>,
        ty: RomDataType,
    ) -> Result<RomDataInfo> {
        assert!(data.len() > 0);

        let mut phf_raw_data = Vec::new();
        let mut partition_count = None;

        for (key, entry) in data {
            match partition_count {
                Some(x) => assert_eq!(x, entry.partitions.len()),
                None => partition_count = Some(entry.partitions.len()),
            }

            let entry = self.encode_entry(entry)?;
            phf_raw_data.push((*key, entry));
        }

        let phf_offset = self.encoder.cur_offset() as u32;
        let data = crate::phf::build_phf(phf_offset, &phf_raw_data);
        self.encoder.encode_bytes_raw(&data);

        let offset = self.encoder.encode(&RomRoot {
            partition_count: partition_count.unwrap() as u32,
            table: phf_offset,
            _phantom: PhantomData::<T>,
        })?;
        Ok(RomDataInfo::new(ty, offset as u32))
    }

    fn write_filesystem(&mut self, loaded: &LoadedFilesystem) -> Result<SerialSlice<RomDataInfo>> {
        let hash = hashed(loaded, 3);
        if !self.encoder.cached_objects.contains_key(&hash) {
            self.pre_encode_file_data(loaded)?;

            let mut roots = Vec::new();
            for (_, root) in &loaded.roots {
                roots.push(match root {
                    LoadedRoot::Empty => RomDataInfo::new(RomDataType::NoFiles, 0x8000000),
                    LoadedRoot::MapStr(map) => {
                        self.encode_root_typed(map, RomDataType::RootStr)?
                    }
                    LoadedRoot::MapU16(map) => {
                        self.encode_root_typed(map, RomDataType::RootU16)?
                    }
                    LoadedRoot::MapU16U16(map) => {
                        self.encode_root_typed(map, RomDataType::RootU16U16)?
                    }
                    LoadedRoot::MapU32(map) => {
                        self.encode_root_typed(map, RomDataType::RootU32)?
                    }
                });
            }

            let offset = self.encoder.cur_offset();
            for root in roots {
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
