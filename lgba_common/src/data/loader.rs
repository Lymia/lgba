use crate::data::{ParsedManifest, ParsedRoot, ParsedSpecShape, StrHash};
use anyhow::{bail, ensure, Result};
use log::{trace, warn};
use std::{
    boxed::Box,
    collections::{BTreeMap, HashMap},
    format, fs,
    hash::Hash,
    path::{Path, PathBuf},
    str::FromStr,
    string::{String, ToString},
    vec::Vec,
};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct LoadedRootData {
    pub partitions: BTreeMap<String, Vec<Vec<u8>>>,
}

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum LoadedRoot {
    Empty,
    MapStr(BTreeMap<StrHash, LoadedRootData>),
    MapU16(BTreeMap<u16, LoadedRootData>),
    MapU16U16(BTreeMap<(u16, u16), LoadedRootData>),
    MapU32(BTreeMap<u32, LoadedRootData>),
}

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Default)]
pub struct LoadedFilesystem {
    pub roots: BTreeMap<String, LoadedRoot>,
}

#[derive(Copy, Clone, Debug)]
pub enum IdKey {
    Str(StrHash),
    U16(u16),
    U16U16(u16, u16),
    U32(u32),
}
impl IdKey {
    fn correct_shape(&self) -> ParsedSpecShape {
        match self {
            IdKey::Str(_) => ParsedSpecShape::Str,
            IdKey::U16(_) => ParsedSpecShape::U16,
            IdKey::U16U16(_, _) => ParsedSpecShape::U16U16,
            IdKey::U32(_) => ParsedSpecShape::U32,
        }
    }
}

/// Allows filtering directories and files loaded from disk.
pub trait FilterVisitor {
    /// Creates a new instance of this directory visitor, that filters calls to `parent`.
    ///
    /// You may call `visit` in this function to generate data unconditionally.
    fn create(parent: Box<dyn FilterVisitor>) -> Result<Self>
    where Self: Sized;

    /// Visits a particular piece of file data.
    fn visit(&mut self, root: &str, key: IdKey, partition: &str, data: Vec<u8>) -> Result<()>;
}

/// Manager for creating directory managers from lists of filters.
#[derive(Default)]
pub struct FilterManager {
    new_filter:
        HashMap<String, Box<dyn Fn(Box<dyn FilterVisitor>) -> Result<Box<dyn FilterVisitor>>>>,
}
impl FilterManager {
    /// Registers a new filter with a given name.
    pub fn register_filter<T: FilterVisitor + 'static>(&mut self, name: &str) {
        assert!(!self.new_filter.contains_key(name), "Duplicate filter '{name}'.");
        self.new_filter
            .insert(name.to_string(), Box::new(|visitor| Ok(Box::new(T::create(visitor)?))));
    }

    fn create(
        &self,
        mut visitor: Box<dyn FilterVisitor>,
        filters: &[String],
    ) -> Result<Box<dyn FilterVisitor>> {
        for filter in filters.iter().rev() {
            assert!(self.new_filter.contains_key(filter), "No such filter '{filter}'.");
            visitor = self.new_filter[filter.as_str()](visitor)?;
        }
        Ok(visitor)
    }
}

#[derive(Default)]
struct RootFilterVisitor {
    template: ParsedManifest,
    filesystem: LoadedFilesystem,
}
impl RootFilterVisitor {
    fn visit_typed<T: Copy + Ord + Eq>(
        map: &mut BTreeMap<T, LoadedRootData>,
        key: T,
        partition: &str,
        data: Vec<u8>,
    ) {
        if !map.contains_key(&key) {
            map.insert(key, LoadedRootData { partitions: Default::default() });
        }
        let partitions = &mut map[&key].partitions;
        if !partitions.contains_key(partition) {
            partitions.insert(partition.into(), Vec::new());
        }
        partitions[partition].push(data);
    }
    fn finalize_typed<T: Copy + Eq + Hash>(
        map: &mut BTreeMap<T, LoadedRootData>,
        root: &ParsedRoot,
    ) {
        for (_, entry) in map {
            for (partition, _) in &root.partitions {
                if !entry.partitions.contains_key(partition) {
                    entry.partitions.insert(partition.into(), Vec::new());
                }
            }
        }
    }
    fn finalize(mut self) -> LoadedFilesystem {
        for (key, root) in &self.template.roots {
            match self.filesystem.roots.get_mut(key) {
                Some(x) => match x {
                    LoadedRoot::Empty => {}
                    LoadedRoot::MapStr(map) => Self::finalize_typed(map, root),
                    LoadedRoot::MapU16(map) => Self::finalize_typed(map, root),
                    LoadedRoot::MapU16U16(map) => Self::finalize_typed(map, root),
                    LoadedRoot::MapU32(map) => Self::finalize_typed(map, root),
                },
                None => {
                    self.filesystem
                        .roots
                        .insert(key.to_string(), LoadedRoot::Empty);
                }
            }
        }
        self.filesystem
    }
}
impl FilterVisitor for RootFilterVisitor {
    fn create(parent: Box<dyn FilterVisitor>) -> Result<Self>
    where Self: Sized {
        unimplemented!()
    }
    fn visit(&mut self, root: &str, key: IdKey, partition: &str, data: Vec<u8>) -> Result<()> {
        let name = &self.template.name.map_or("(unnamed)", |x| x.as_str());
        if !self.template.roots.contains_key(root) {
            warn!(
                "Could not store file data in '{name}:{root}/{key:?}/{partition}' \
                 (root '{root}' does not exist)",
            );
            Ok(())
        } else if !self.template.roots[root].partitions.contains_key(partition) {
            warn!(
                "Could not store file data in '{name}:{root}/{key:?}/{partition}' \
                 (partition '{root}/*/{partition}' does not exist)",
            );
            Ok(())
        } else if self.template.roots[root].shape != key.correct_shape() {
            warn!(
                "Could not store file data in '{name}:{root}/{key:?}/{partition}' \
                 (IdKey has incorrect shape. expected: {:?} , got: {:?})",
                self.template.roots[root].shape,
                key.correct_shape(),
            );
            Ok(())
        } else {
            trace!("Storing data: '{name}:{root}/{key:?}/{partition}'");
            macro_rules! dispatch_branch {
                ($key:expr, $var:ident) => {{
                    let key = $key;
                    if !self.filesystem.roots.contains_key(partition) {
                        let mut tree = Default::default();
                        Self::visit_typed(&mut tree, key, partition, data);
                        self.filesystem
                            .roots
                            .insert(partition.into(), LoadedRoot::$var(tree));
                    } else if let LoadedRoot::$var(tree) = &mut self.filesystem.roots[partition] {
                        Self::visit_typed(&mut tree, key, partition, data);
                    } else {
                        panic!("invalid internal state")
                    }
                    Ok(())
                }};
            }
            match key {
                IdKey::Str(name) => dispatch_branch!(name, MapStr),
                IdKey::U16(a) => dispatch_branch!(a, MapU16),
                IdKey::U16U16(a, b) => dispatch_branch!((a, b), MapU16U16),
                IdKey::U32(a) => dispatch_branch!(a, MapU32),
            }
        }
    }
}

struct RootFilterWrapper(Rc<RefCell<RootFilterVisitor>>);
impl FilterVisitor for RootFilterWrapper {
    fn create(parent: Box<dyn FilterVisitor>) -> Result<Self> where Self: Sized {
        unimplemented!()
    }
    fn visit(&mut self, root: &str, key: IdKey, partition: &str, data: Vec<u8>) -> Result<()> {
        self.0.get_mut().visit(root, key, partition, data)
    }
}

fn maybe_hex<T>(str: &str, is_hex: bool) -> Result<T>
where T: TryFrom<u32> {
    let raw = if is_hex {
        u32::from_str_radix(str, 16)
    } else {
        u32::from_str(str)
    };
    match raw {
        Ok(v) => match T::try_from(v) {
            Ok(v) => Ok(v),
            Err(_) => bail!("Number out of range for {}", core::any::type_name::<T>()),
        }
        Err(_) => bail!("Number out of range for {}", core::any::type_name::<T>()),
    }
}
fn dispatch_root(
    root_dir: &Path,
    root_visitor: Rc<RefCell<RootFilterVisitor>>,
    root: &ParsedRoot,
    filters: &FilterManager,
) -> Result<()> {
    let root_str = root_dir.display().to_string();
    let mut visitor = filters.create(Box::new(RootFilterWrapper(root_visitor)), &root.filters)?;
    for (partition, spec) in &root.partitions {
        if let Some(spec) = spec {
            let shape = spec.shape()?;
            let pattern = format!("{}/{}", root_str, spec.glob()?);
            let regex = regex_lite::Regex::new(&spec.regex()?)?;
            let is_hex = spec.is_hex()?;

            for path in glob::glob(&pattern)? {
                let path = path?;
                if path.is_file() {
                    let path_str = path.display().to_string();
                    ensure!(path_str.starts_with(&root_str));
                    let path_str = &path_str[root_str.len() + 1..];

                    let Some(captures) = regex.captures(path_str) else {
                        bail!("Regex match failure after glob match success?");
                    };

                    let id = match shape {
                        ParsedSpecShape::Str => IdKey::Str(StrHash::new(&captures[0])),
                        ParsedSpecShape::U16 => IdKey::U16(maybe_hex(&captures[0], is_hex[0])?),
                        ParsedSpecShape::U16U16 => IdKey::U16U16(
                            maybe_hex(&captures[0], is_hex[0])?,
                            maybe_hex(&captures[1], is_hex[1])?,
                        ),
                        ParsedSpecShape::U32 => IdKey::U32(maybe_hex(&captures[0], is_hex[0])?),
                    };

                    visitor.visit(&root.name, id, partition.as_str(), fs::read(path)?)?;
                }
            }
        }
    }
    Ok(())
}

pub fn load(
    root_dir: &Path,
    manifest: &ParsedManifest,
    filters: &FilterManager,
) -> Result<LoadedFilesystem> {
    let root_visitor = Rc::new(RefCell::new(RootFilterVisitor {
        template: manifest.clone(),
        filesystem: Default::default(),
    }));
    for (_, root) in &manifest.roots {
        dispatch_root(root_dir, root_visitor.clone(), root, filters)?;
    }
    Ok(root_visitor.replace(Default::default()).finalize())
}
