//! Implementation for the data structures and data formats used to pack files into the ROM through
//! the romtool.
//!
//! Unlike with `include_bytes!`, this can include entire directories at once without needing an
//! indexing system - and can allow the game data to be modified without recompilation of the core
//! binary.
//!
//! # Encoding at compile-time
//!
//! `lgba_data` uses the exheader mechanism in romtool to allow for easy ROM modifications after
//! the core image is compiled.
//!
//! The header has a type of `"data"` and consists of a 12 byte hash of the manifest file followed
//! by a pointer to the game data. The header is linked into the ROM with weak linkage and a name
//! that includes the hash - hence allowing separate loads of the same data to be shared.
//!
//! No actual data is stored in the compiled ELF binary, romtool adds it into the final image after
//! the fact. The structures added by romtool can be found in `rom_data`.
//!
//! `manifest` contains the definitions used for the .toml file used to define the structure.
//!
//! `loader` contains the code and definitions used to store filesystem data loaded from the
//! manifest and ready to encode into a ROM.

mod rom_data;
pub use rom_data::*;

#[cfg(feature = "data_manifest")]
mod manifest;

#[cfg(feature = "data_manifest")]
pub use manifest::*;

#[cfg(feature = "data_build")]
mod loader;

#[cfg(feature = "data_build")]
pub use loader::{load, DirVisitor, FilterManager};

#[cfg(feature = "data_build")]
mod encoder;

#[cfg(feature = "data_build")]
pub use encoder::FilesystemEncoder;

#[cfg(feature = "data_manifest")]
fn hashed<T: core::hash::Hash + ?Sized>(data: &T, nonce: u32) -> [u8; 32] {
    use core::hash::{Hash, Hasher};

    struct HasherWrapper<'a>(&'a mut blake3::Hasher);
    impl<'a> Hasher for HasherWrapper<'a> {
        fn finish(&self) -> u64 {
            unreachable!()
        }
        fn write(&mut self, bytes: &[u8]) {
            self.0.update(bytes);
        }
    }

    let mut hasher = blake3::Hasher::new();
    (std::any::type_name::<T>(), data).hash(&mut HasherWrapper(&mut hasher));
    hasher.update(&nonce.to_ne_bytes());
    *hasher.finalize().as_bytes()
}
