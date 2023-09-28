use crate::error;
use lgba_common::hashes::hashed;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as SynTokenStream;
use quote::quote;
use syn::{parse2, spanned::Spanned, Lit, Result};

fn hashed_impl_0(crate_name: SynTokenStream, input: TokenStream) -> Result<TokenStream> {
    let input: SynTokenStream = input.into();
    let parsed: Lit = parse2(input)?;
    let value = match parsed {
        Lit::Str(str) => str.value(),
        _ => error(input.span(), "unknown literal type")?,
    };
    let hashed = hashed(value, 10000);
    quote! {
        #crate_name::__macro_export::new_hash(#hashed)
    }
}
pub fn hashed_impl(crate_name: SynTokenStream, input: TokenStream) -> TokenStream {
    match hashed_impl_0(crate_name, input) {
        Ok(x) => x,
        Err(e) => e.into_compile_error().into(),
    }
}