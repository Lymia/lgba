use crate::data::{ParsedManifest, ParsedRoot};
use anyhow::{bail, ensure, Result};
use log::warn;
use std::{
    boxed::Box,
    cell::RefCell,
    collections::{BTreeMap, HashMap},
    fs,
    hash::Hash,
    path::{Path, PathBuf},
    rc::Rc,
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
    pub enable_dir_listing: bool,
    pub enable_file_names: bool,
}

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum LoadedRoot {
    Directory(LoadedDirectory),
    File(Vec<u8>),
    MapU16(BTreeMap<u16, LoadedMapData>),
    MapU16U16(BTreeMap<(u16, u16), LoadedMapData>),
    MapU32(BTreeMap<u32, LoadedMapData>),
}

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct LoadedFilesystem {
    pub roots: BTreeMap<String, LoadedRoot>,
}

/// Allows filtering directories and files loaded from disk.
pub trait DirVisitor {
    /// Creates a new instance of this directory visitor, that filters calls to `parent`.
    fn create(parent: Box<dyn DirVisitor>) -> Result<Self>
    where Self: Sized;

    /// Visits a directory.
    fn visit_dir(
        &mut self,
        name: &str,
        callback: &mut dyn FnMut(&mut dyn DirVisitor) -> Result<()>,
    ) -> Result<()>;

    /// Visits a file.
    fn visit_file(&mut self, name: &str, contents: Vec<u8>) -> Result<()>;

    /// Visits an invalid node.
    ///
    /// This is usually a symbolic link, and rarely a device node or something else strange.
    fn visit_invalid(&mut self, name: &str) -> Result<()>;
}

/// Manager for creating directory managers from lists of filters.
pub struct FilterManager {
    new_filter: HashMap<String, Box<dyn Fn(Box<dyn DirVisitor>) -> Result<Box<dyn DirVisitor>>>>,
}
impl FilterManager {
    /// Registers a new filter with a given name.
    pub fn register_filter<T: DirVisitor + 'static>(&mut self, name: &str) {
        assert!(!self.new_filter.contains_key(name), "Duplicate filter '{name}'.");
        self.new_filter
            .insert(name.to_string(), Box::new(|visitor| Ok(Box::new(T::create(visitor)?))));
    }

    fn create(
        &self,
        mut visitor: Box<dyn DirVisitor>,
        filters: &[String],
    ) -> Result<Box<dyn DirVisitor>> {
        for filter in filters.iter().rev() {
            assert!(self.new_filter.contains_key(filter), "No such filter '{filter}'.");
            visitor = self.new_filter[filter.as_str()](visitor)?;
        }
        Ok(visitor)
    }
}

struct DirNodeVisitor(Rc<RefCell<LoadedDirectoryNode>>);
impl DirNodeVisitor {
    fn add_node(&mut self, name: &str, node: LoadedDirectoryNode) -> Result<()> {
        let mut lock = self.0.borrow_mut();
        let dir = match &mut *lock {
            LoadedDirectoryNode::File(_) => unimplemented!(),
            LoadedDirectoryNode::Directory(map) => map,
            LoadedDirectoryNode::InvalidNode => unimplemented!(),
        };
        ensure!(!dir.contains_key(name), "duplicate entry {name} in node");
        dir.insert(name.to_string(), node);
        Ok(())
    }
}
impl DirVisitor for DirNodeVisitor {
    fn create(_: Box<dyn DirVisitor>) -> Result<Self>
    where Self: Sized {
        unimplemented!()
    }

    fn visit_dir(
        &mut self,
        name: &str,
        callback: &mut dyn FnMut(&mut dyn DirVisitor) -> Result<()>,
    ) -> Result<()> {
        let new_node = Rc::new(RefCell::new(LoadedDirectoryNode::Directory(BTreeMap::new())));
        {
            let mut new_node_visitor = DirNodeVisitor(new_node.clone());
            callback(&mut new_node_visitor)?;
        }
        self.add_node(name, new_node.replace(LoadedDirectoryNode::InvalidNode))?;
        Ok(())
    }

    fn visit_file(&mut self, name: &str, contents: Vec<u8>) -> Result<()> {
        self.add_node(name, LoadedDirectoryNode::File(contents))?;
        Ok(())
    }

    fn visit_invalid(&mut self, name: &str) -> Result<()> {
        self.add_node(name, LoadedDirectoryNode::InvalidNode)?;
        Ok(())
    }
}

struct FileNodeVisitor(Rc<RefCell<Option<Vec<u8>>>>);
impl DirVisitor for FileNodeVisitor {
    fn create(_: Box<dyn DirVisitor>) -> Result<Self>
    where Self: Sized {
        unimplemented!()
    }

    fn visit_dir(
        &mut self,
        _: &str,
        _: &mut dyn FnMut(&mut dyn DirVisitor) -> Result<()>,
    ) -> Result<()> {
        bail!("Single-file roots can only store single files.");
    }

    fn visit_file(&mut self, name: &str, contents: Vec<u8>) -> Result<()> {
        ensure!(self.0.borrow().is_none(), "Single-file roots can only store single files.");
        self.0.replace(Some(contents));
        Ok(())
    }

    fn visit_invalid(&mut self, name: &str) -> Result<()> {
        bail!("Single-file roots can only store single files.");
    }
}

fn send_dir_to_visitor(path: &Path, visitor: &mut dyn DirVisitor) -> Result<()> {
    let name = path.file_name().unwrap().to_string_lossy();
    if path.is_symlink() {
        warn!("Path '{}' is a symbolic link and cannot be properly stored.", path.display());
        visitor.visit_invalid(&name)?;
    } else if path.is_file() {
    } else {
        warn!("Path '{}' is a special file and cannot be properly stored.", path.display());
        visitor.visit_invalid(&name)?;
    }
    Ok(())
}

pub fn load(
    root_dir: &Path,
    manifest: &ParsedManifest,
    filters: &FilterManager,
) -> Result<LoadedFilesystem> {
    let mut loaded_roots = BTreeMap::new();
    for (name, root) in &manifest.roots {
        loaded_roots.insert(name.clone(), match root {
            ParsedRoot::Directory(v) => {
                let new_node =
                    Rc::new(RefCell::new(LoadedDirectoryNode::Directory(BTreeMap::new())));
                let mut path = PathBuf::from(root_dir);
                path.push(&v.path);
                {
                    let mut visitor =
                        filters.create(Box::new(DirNodeVisitor(new_node.clone())), &v.filters)?;
                    send_dir_to_visitor(&path, &mut *visitor)?;
                }
                LoadedRoot::Directory(LoadedDirectory {
                    root: new_node.replace(LoadedDirectoryNode::InvalidNode),
                    enable_dir_listing: v.enable_dir_listing,
                    enable_file_names: v.enable_file_names,
                })
            }
            ParsedRoot::File(v) => {
                let file_ref = Rc::new(RefCell::new(None));
                let mut path = PathBuf::from(root_dir);
                path.push(&v.path);
                {
                    let mut visitor =
                        filters.create(Box::new(FileNodeVisitor(file_ref.clone())), &v.filters)?;
                    visitor.visit_file(
                        &path.file_name().unwrap().to_string_lossy(),
                        fs::read(&path)?,
                    )?;
                }
                LoadedRoot::File(file_ref.replace(None).unwrap())
            }
            ParsedRoot::IdMap(_) => todo!(),
        });
    }
    Ok(LoadedFilesystem { roots: loaded_roots })
}
