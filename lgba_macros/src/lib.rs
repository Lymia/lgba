#![feature(doc_cfg)]

#[cfg(feature = "lgba")]
mod lgba_attrs;

#[cfg(feature = "hashes")]
mod hashes;

#[cfg(feature = "data")]
mod lgba_data_attrs;

extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::Span;
use std::fmt::Display;
use syn::{Error, Result};

/// Helper function for emitting compile errors.
fn error<T>(span: Span, message: impl Display) -> Result<T> {
    Err(Error::new(span, message))
}

/// Stores the item this is placed on in iwram rather than its default location.
#[cfg(feature = "lgba")]
#[proc_macro_attribute]
pub fn iwram(_: TokenStream, input: TokenStream) -> TokenStream {
    lgba_attrs::iwram_impl(input)
}

/// Stores the item this is placed on in ewram rather than its default location.
#[cfg(feature = "lgba")]
#[proc_macro_attribute]
pub fn ewram(_: TokenStream, input: TokenStream) -> TokenStream {
    lgba_attrs::ewram_impl(input)
}

/// Calls this function when the game first starts, before the entry function is called.
///
/// All rustc and lgba features are available for use by the time function is called, though there
/// are no guarantees about the order constructor functions are run in.
#[cfg(feature = "lgba")]
#[proc_macro_attribute]
pub fn ctor(args: TokenStream, input: TokenStream) -> TokenStream {
    lgba_attrs::ctor_impl(args, input)
}

/// Marks the function this is placed on as an ARM function.
#[cfg(feature = "lgba")]
#[proc_macro_attribute]
pub fn arm(_: TokenStream, input: TokenStream) -> TokenStream {
    lgba_attrs::arm_impl(input)
}

/// Marks the function this is placed on as a Thumb function.
#[cfg(feature = "lgba")]
#[proc_macro_attribute]
pub fn thumb(_: TokenStream, input: TokenStream) -> TokenStream {
    lgba_attrs::thumb_impl(input)
}

// TODO: Document
#[cfg(feature = "lgba")]
#[doc(cfg(feature = "low_level"))]
#[proc_macro_attribute]
pub fn unsafe_alloc_zones(args: TokenStream, input: TokenStream) -> TokenStream {
    lgba_attrs::unsafe_alloc_zones(args, input)
}

// TODO: Document
#[cfg(feature = "lgba")]
#[proc_macro_attribute]
pub fn entry(args: TokenStream, input: TokenStream) -> TokenStream {
    lgba_attrs::entry(args, input)
}

#[cfg(feature = "hashes")]
#[proc_macro]
pub fn hash_lgba_data(item: TokenStream) -> TokenStream {
    hashes::hashed_impl(quote::quote! { lgba_data }, item)
}

#[cfg(feature = "data")]
#[proc_macro]
pub fn load_data_impl(item: TokenStream) -> TokenStream {
    lgba_data_attrs::load_data_impl(item)
}
