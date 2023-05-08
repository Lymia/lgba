mod attrs;
mod build_fonts;

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
#[proc_macro_attribute]
pub fn iwram(_: TokenStream, input: TokenStream) -> TokenStream {
    attrs::iwram_impl(input)
}

/// Stores the item this is placed on in ewram rather than its default location.
#[proc_macro_attribute]
pub fn ewram(_: TokenStream, input: TokenStream) -> TokenStream {
    attrs::ewram_impl(input)
}

#[proc_macro_attribute]
pub fn entry(args: TokenStream, input: TokenStream) -> TokenStream {
    attrs::entry(args, input)
}
