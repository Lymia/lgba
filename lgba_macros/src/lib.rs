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
    title: Option<String>,
    #[darling(default)]
    code: Option<String>,
    #[darling(default)]
    developer: Option<String>,
    #[darling(default)]
    version: Option<u8>,
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
fn derive_enum_set_type_0(mut input: ItemFn, attrs: EntryAttrs) -> Result<SynTokenStream> {
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

    // Generate the entry point code
    let name = &input.sig.ident;
    let title = match &attrs.title {
        None => quote! { env!("CARGO_PKG_NAME") },
        Some(title) => {
            if title.len() > 12 {
                error(Span::call_site(), "ROM title cannot be longer than 12 characters.")?;
            }
            quote! { #title }
        }
    };
    let code = match &attrs.code {
        None => quote! { "" },
        Some(code) => {
            if code.len() != 4 {
                error(Span::call_site(), "ROM code must be exactly 4 characters.")?;
            }
            quote! { #code }
        }
    };
    let developer = match &attrs.developer {
        None => quote! { "" },
        Some(developer) => {
            if developer.len() != 2 {
                error(Span::call_site(), "ROM developer code must be exactly 2 characters.")?;
            }
            quote! { #developer }
        }
    };
    let version = match attrs.version {
        None => quote! { 0 },
        Some(version) => quote! { #version },
    };
    let title_auto = attrs.title.is_none();
    let code_auto = attrs.code.is_none();
    let developer_auto = attrs.developer.is_none();

    let new_attrs: Vec<_> = input
        .attrs
        .iter()
        .cloned()
        .filter(|x| !x.path.is_ident("rom"))
        .collect();
    input.attrs = new_attrs;
    Ok(quote! {
        #input

        /// The module used by lgba for its entry attribute codegen.
        mod __lgba_entry {
            use lgba::__macro_export::*;

            #[no_mangle]
            #[link_section = ".lgba.header.dynamic"]
            pub static __lgba_header_dynamic: GbaHeader = {
                let mut h = GBA_HEADER_TEMPLATE;
                h = set_header_field(h, #title, 0, 12, #title_auto);
                h = set_header_field(h, #code, 12, 4, #code_auto);
                h = set_header_field(h, #developer, 16, 2, #developer_auto);
                h[0x1C] = #version as u8;
                h = calculate_complement(h);
                h
            };

            #[no_mangle]
            pub static __lgba_exh_rom_cname: &str = env!("CARGO_PKG_NAME");
            #[no_mangle]
            pub static __lgba_exh_rom_cver: &str = env!("CARGO_PKG_VERSION");
            #[no_mangle]
            pub static __lgba_exh_rom_repository: &str = env!("CARGO_PKG_REPOSITORY");

            #[no_mangle]
            pub unsafe extern "C" fn __lgba_rom_entry() -> ! {
                super::#name()
            }
        }
    })
}
