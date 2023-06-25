mod attrs;
mod build_fonts;

extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::{Ident, Span, TokenStream as SynTokenStream};
use proc_macro_crate::FoundCrate;
use quote::quote;
use std::fmt::Display;
use syn::{Error, Result};

/// Helper function for emitting compile errors.
fn error<T>(span: Span, message: impl Display) -> Result<T> {
    Err(Error::new(span, message))
}

/// Contains the crate paths used by the macros here.
#[allow(unused)]
struct Paths {
    lgba: SynTokenStream,
    internal: SynTokenStream,
    lgba_phf: SynTokenStream,
}
impl Paths {
    fn new() -> Result<Paths> {
        let root_crate = match proc_macro_crate::crate_name("lgba") {
            Ok(FoundCrate::Itself) => quote! { crate::lgba },
            Ok(FoundCrate::Name(name)) => {
                let literal = Ident::new(name.as_str(), Span::call_site());
                quote! { #literal }
            }
            Err(_) => quote! { lgba },
        };

        Ok(Paths {
            lgba: root_crate.clone(),
            internal: quote! { #root_crate::__macro_export },
            lgba_phf: quote! { #root_crate::__macro_export::lgba_phf },
        })
    }
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

/// Marks the function this is placed on as an ARM function.
#[proc_macro_attribute]
pub fn arm(_: TokenStream, input: TokenStream) -> TokenStream {
    attrs::arm_impl(input)
}

/// Marks the function this is placed on as a Thumb function.
#[proc_macro_attribute]
pub fn thumb(_: TokenStream, input: TokenStream) -> TokenStream {
    attrs::thumb_impl(input)
}

#[proc_macro_attribute]
pub fn entry(args: TokenStream, input: TokenStream) -> TokenStream {
    attrs::entry(args, input)
}

#[proc_macro_derive(TerminalFont, attributes(font))]
pub fn derive_terminal_font(input: TokenStream) -> TokenStream {
    match build_fonts::derive_terminal_font(input.into()) {
        Ok(v) => v.into(),
        Err(e) => e.to_compile_error().into(),
    }
}
