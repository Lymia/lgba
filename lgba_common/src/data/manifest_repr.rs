use serde::{Deserialize, Serialize};
use std::{string::String, vec::Vec};

#[derive(Clone, Debug, Hash, Serialize, Deserialize)]
pub struct FilesystemDirectory {
    pub name: String,
    pub path: String,
    #[serde(default)]
    pub exclude: Vec<String>,
    #[serde(default)]
    pub enable_dir_listing: bool,
    #[serde(default)]
    pub enable_file_names: bool,
}

#[derive(Clone, Debug, Hash, Serialize, Deserialize)]
pub struct FilesystemFile {
    pub name: String,
    pub path: String,
}

#[derive(Clone, Debug, Hash, Serialize, Deserialize)]
pub struct FilesystemIdMap {
    pub name: String,
    pub spec: String,
}

#[derive(Clone, Debug, Hash, Serialize, Deserialize)]
pub struct FilesystemManifest {
    #[serde(default)]
    pub dir: Vec<FilesystemDirectory>,
    #[serde(default)]
    pub file: Vec<FilesystemFile>,
    #[serde(default)]
    pub id_map: Vec<FilesystemIdMap>,
}
