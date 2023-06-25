use crate::Paths;
use darling::FromAttributes;
use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as SynTokenStream};
use quote::quote;
use syn::{spanned::Spanned, Error, ItemFn, ReturnType, Type};

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
    #[darling(default)]
    report_url: Option<String>,
}

pub fn iwram_impl(input: TokenStream) -> TokenStream {
    let input: SynTokenStream = input.into();
    (quote! {
        #[link_section = ".iwram"]
        #input
    })
    .into()
}

/// Stores the item this is placed on in ewram rather than its default location.
pub fn ewram_impl(input: TokenStream) -> TokenStream {
    let input: SynTokenStream = input.into();
    (quote! {
        #[link_section = ".ewram"]
        #input
    })
    .into()
}

pub fn arm_impl(input: TokenStream) -> TokenStream {
    let input: SynTokenStream = input.into();
    (quote! {
        #[instruction_set(arm::a32)]
        #input
    })
    .into()
}

/// Stores the item this is placed on in ewram rather than its default location.
pub fn thumb_impl(input: TokenStream) -> TokenStream {
    let input: SynTokenStream = input.into();
    (quote! {
        #[instruction_set(arm::t32)]
        #input
    })
    .into()
}

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
    match entry_0(input, attrs) {
        Ok(v) => v.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

fn entry_0(mut input: ItemFn, attrs: EntryAttrs) -> syn::Result<SynTokenStream> {
    let paths = Paths::new()?;
    let internal = &paths.internal;

    // Check function signature
    match &input.sig.output {
        ReturnType::Type(_, ty) if matches!(**ty, Type::Never(_)) => {} // ok
        _ => crate::error(
            input.sig.output.span(),
            "#[lgba::entry] functions must have a signature of `[unsafe] fn() -> !`",
        )?,
    }
    if !input.sig.inputs.is_empty() {
        crate::error(
            input.sig.output.span(),
            "#[lgba::entry] functions must have a signature of `[unsafe] fn() -> !`",
        )?;
    }
    if input.sig.asyncness.is_some() {
        crate::error(
            input.sig.asyncness.span(),
            "#[lgba::entry] cannot be applied to async functions.",
        )?;
    }
    if !input.sig.generics.params.is_empty() {
        crate::error(
            input.sig.generics.span(),
            "#[lgba::entry] cannot be applied to generic functions.",
        )?;
    }
    if input.sig.variadic.is_some() {
        crate::error(
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
                crate::error(Span::call_site(), "ROM title cannot be longer than 12 characters.")?;
            }
            quote! { #title }
        }
    };
    let code = match &attrs.code {
        None => quote! { "" },
        Some(code) => {
            if code.len() != 4 {
                crate::error(Span::call_site(), "ROM code must be exactly 4 characters.")?;
            }
            quote! { #code }
        }
    };
    let developer = match &attrs.developer {
        None => quote! { "" },
        Some(developer) => {
            if developer.len() != 2 {
                crate::error(
                    Span::call_site(),
                    "ROM developer code must be exactly 2 characters.",
                )?;
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
    let report_url = match &attrs.report_url {
        None => quote! { env!("CARGO_PKG_REPOSITORY") },
        Some(report_url) => quote! { #report_url },
    };

    let new_attrs: Vec<_> = input
        .attrs
        .iter()
        .cloned()
        .filter(|x| !x.path().is_ident("rom"))
        .collect();
    input.attrs = new_attrs;
    Ok(quote! {
        #input

        /// The module used by lgba for its entry attribute codegen.
        mod __lgba_entry {
            use #internal::gba_header::*;

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
            pub static __lgba_exh_rom_repository: &str = #report_url;

            #[no_mangle]
            pub unsafe extern "C" fn __lgba_rom_entry() -> ! {
                super::#name()
            }
        }
    })
}
