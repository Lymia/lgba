extern crate proc_macro;

use darling::*;
use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as SynTokenStream};
use quote::*;
use std::fmt::Display;
use syn::{spanned::Spanned, Error, Result, *};

/// Helper function for emitting compile errors.
fn error<T>(span: Span, message: impl Display) -> Result<T> {
    Err(Error::new(span, message))
}

/// Decodes the custom attributes for our custom derive.
#[derive(FromAttributes, Default)]
#[darling(attributes(rom), default)]
struct EntryAttrs {
    #[darling(default)]
    rom_title: Option<String>,
    #[darling(default)]
    rom_code: Option<String>,
    #[darling(default)]
    rom_developer: Option<String>,
    #[darling(default)]
    rom_version: Option<u16>,
}

/// Stores the item this is placed on in iwram rather than its default location.
#[proc_macro_attribute]
pub fn iwram(_: TokenStream, input: TokenStream) -> TokenStream {
    let input: SynTokenStream = input.into();
    (quote! {
        #[link_section = ".iwram"]
        #input
    })
    .into()
}

/// Stores the item this is placed on in ewram rather than its default location.
#[proc_macro_attribute]
pub fn ewram(_: TokenStream, input: TokenStream) -> TokenStream {
    let input: SynTokenStream = input.into();
    (quote! {
        #[link_section = ".ewram"]
        #input
    })
    .into()
}

#[proc_macro_attribute]
pub fn entry(args: TokenStream, input: TokenStream) -> TokenStream {
    let args: SynTokenStream = args.into();
    let input: SynTokenStream = input.into();

    let input: ItemFn = match syn::parse2(input) {
        Ok(v) => v,
        Err(_) => {
            return Error::new(args.span(), "#[lgba::entry] must be placed on a function.")
                .to_compile_error()
                .into()
        }
    };

    let attrs: EntryAttrs = match EntryAttrs::from_attributes(&input.attrs) {
        Ok(attrs) => attrs,
        Err(e) => return e.write_errors().into(),
    };
    match derive_enum_set_type_0(input, attrs) {
        Ok(v) => v.into(),
        Err(e) => e.to_compile_error().into(),
    }
}
fn derive_enum_set_type_0(input: ItemFn, attrs: EntryAttrs) -> Result<SynTokenStream> {
    // Check function signature
    match &input.sig.output {
        ReturnType::Type(_, ty) if matches!(**ty, Type::Never(_)) => {} // ok
        _ => error(
            input.sig.output.span(),
            "#[lgba::entry] functions must have a signature of `[unsafe] fn() -> !`",
        )?,
    }
    if !input.sig.inputs.is_empty() {
        error(
            input.sig.output.span(),
            "#[lgba::entry] functions must have a signature of `[unsafe] fn() -> !`",
        )?;
    }
    if input.sig.asyncness.is_some() {
        error(
            input.sig.asyncness.span(),
            "#[lgba::entry] cannot be applied to async functions.",
        )?;
    }
    if !input.sig.generics.params.is_empty() {
        error(
            input.sig.generics.span(),
            "#[lgba::entry] cannot be applied to generic functions.",
        )?;
    }
    if input.sig.variadic.is_some() {
        error(
            input.sig.variadic.span(),
            "#[lgba::entry] cannot be applied to varargs functions.",
        )?;
    }

    // Generate
    let name = &input.sig.ident;
    let title = match attrs.rom_title {
        None => quote! { env!("CARGO_PKG_NAME") },
        Some(title) => {
            if title.len() > 12 {
                error(Span::call_site(), "ROM title cannot be longer than 12 characters.")?;
            }
            quote! { #title }
        }
    };
    let code = match attrs.rom_code {
        None => quote! { "LGBA" },
        Some(code) => {
            if code.len() != 4 {
                error(Span::call_site(), "ROM code must be exactly 4 characters.")?;
            }
            quote! { #code }
        }
    };
    let developer = match attrs.rom_developer {
        None => quote! { "00" },
        Some(developer) => {
            if developer.len() != 2 {
                error(Span::call_site(), "ROM developer code must be exactly 2 characters.")?;
            }
            quote! { #developer }
        }
    };
    let version = match attrs.rom_version {
        None => quote! { "0" },
        Some(version) => {
            let version = version.to_string();
            quote! { #version }
        }
    };

    Ok(quote! {
        #input

        /// The module used by lgba for its entry attribute codegen.
        mod __lgba_entry {
            #[no_mangle]
            pub static __lgba_exh_rom_title: &str = #title;
            #[no_mangle]
            pub static __lgba_exh_rom_code: &str = #code;
            #[no_mangle]
            pub static __lgba_exh_rom_developer: &str = #developer;
            #[no_mangle]
            pub static __lgba_exh_rom_ver: &str = #version;
            #[no_mangle]
            pub static __lgba_exh_rom_cname: &str = env!("CARGO_PKG_NAME");
            #[no_mangle]
            pub static __lgba_exh_rom_cver: &str = env!("CARGO_PKG_VERSION");

            #[no_mangle]
            pub unsafe extern "C" fn __lgba_rom_entry() -> ! {
                super::#name()
            }
        }
    })
}
