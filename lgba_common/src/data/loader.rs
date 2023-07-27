use crate::data::{ParsedManifest, ParsedRoot};
use anyhow::Result;
use log::warn;
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    string::{String, ToString},
    vec::Vec,
};

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct LoadedMapData {
    pub partitions: BTreeMap<String, Vec<Vec<u8>>>,
}

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum LoadedDirectoryNode {
    File(Vec<u8>),
    Directory(BTreeMap<String, LoadedDirectoryNode>),
    InvalidNode,
}

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct LoadedDirectory {
    pub root: LoadedDirectoryNode,
}

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum LoadedRoot {
    Directory(LoadedDirectoryNode),
    File(Vec<u8>),
    MapU16(BTreeMap<u16, LoadedMapData>),
    MapU16U16(BTreeMap<(u16, u16), LoadedMapData>),
    MapU32(BTreeMap<u32, LoadedMapData>),
}

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct LoadedFilesystem {
    pub roots: BTreeMap<String, LoadedRoot>,
}

///
pub trait FileFilter {}

fn load_dir_node(name: &str) -> Result<LoadedDirectoryNode> {
    fn load_dir_node(path: &Path) -> Result<LoadedDirectoryNode> {
        if path.is_symlink() {
            warn!(
                "Path {} is a symlink. It will be treated as an unknown filesystem object.",
                path.display()
            );
            Ok(LoadedDirectoryNode::InvalidNode)
        } else if path.is_dir() {
            let mut files = BTreeMap::new();
            for file in path.read_dir()? {
                let file = file?;
                files.insert(
                    file.file_name().to_string_lossy().to_string(),
                    load_dir_node(&file.path())?,
                );
            }
            Ok(LoadedDirectoryNode::Directory(files))
        } else if path.is_file() {
            Ok(LoadedDirectoryNode::File(fs::read(path)?))
        } else {
            warn!("Path {} is an unknown filesystem object.", path.display());
            Ok(LoadedDirectoryNode::InvalidNode)
        }
    }
    if name == "*" {
        Ok(LoadedDirectoryNode::Directory(BTreeMap::new()))
    } else {
        load_dir_node(&PathBuf::from(name))
    }
}
pub fn load(manifest: &ParsedManifest) -> Result<LoadedFilesystem> {
    let mut loaded_roots = BTreeMap::new();
    for (name, root) in &manifest.roots {
        loaded_roots.insert(name.clone(), match root {
            ParsedRoot::Directory(v) => LoadedRoot::Directory(load_dir_node(&v.path)?),
            ParsedRoot::File(path) => LoadedRoot::File(fs::read(&path.path)?),
            ParsedRoot::IdMap(_) => todo!(),
        });
    }
    Ok(LoadedFilesystem { roots: loaded_roots })
}
