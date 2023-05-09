mod font_data;
mod gen_fonts;

use crate::build_fonts::gen_fonts::FontConfig;
use darling::FromAttributes;
use proc_macro2::TokenStream as SynTokenStream;
use quote::quote;
use syn::{parse2, DeriveInput, Result};

pub fn derive_terminal_font(input: SynTokenStream) -> Result<SynTokenStream> {
    let parsed = parse2::<DeriveInput>(input)?;
    let config = FontConfig::from_attributes(&parsed.attrs)?;
    let ident = &parsed.ident;
    gen_fonts::generate_fonts(config, quote! { #ident })
}
