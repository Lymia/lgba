use std::fs;
use lgba_common::data::{FilterManager, ParsedManifest};
use std::path::PathBuf;

pub fn main() {
    let manifest = include_str!("DataTest.toml");
    let parsed = ParsedManifest::parse(manifest).unwrap();
    println!("{:#?}", parsed);
    let loaded =
        lgba_common::data::load(&PathBuf::from("lgba_common"), &parsed, &FilterManager::default())
            .unwrap();
    println!("{:?}", loaded);
    let mut encoded = lgba_common::data::FilesystemEncoder::new(0x8000000);
    encoded
        .load_filesystem(&PathBuf::from("lgba_common"), &parsed, &FilterManager::default())
        .unwrap();
    println!("{:?}", encoded);

    fs::write("test.bin", encoded.data()).unwrap();
}
