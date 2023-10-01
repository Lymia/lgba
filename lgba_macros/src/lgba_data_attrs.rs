use crate::error;
use lgba_common::data::{ParsedManifest, ParsedSpecShape};
use proc_macro::TokenStream;
use proc_macro2::{Ident, TokenStream as SynTokenStream};
use quote::quote;
use std::{
    fs,
    hash::{Hash, Hasher},
    path::PathBuf,
};
use syn::{
    parse::{Parse, ParseStream},
    spanned::Spanned,
    LitStr, Result, Token, Visibility,
};

struct LoadDataInvocation {
    vis: Visibility,
    name: Ident,
    loc: LitStr,
}
impl Parse for LoadDataInvocation {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let visibility: Visibility = input.parse()?;
        let name: Ident = input.parse()?;
        input.parse::<Token![,]>()?;
        let loc: LitStr = input.parse()?;
        Ok(LoadDataInvocation { vis: visibility, name, loc })
    }
}

fn load_data_impl_0(args: SynTokenStream) -> Result<SynTokenStream> {
    // parses the invocation
    let invocation: LoadDataInvocation = syn::parse2(args.clone())?;

    // load the manifest
    let path = std::env::var("CARGO_MANIFEST_DIR").expect("Could not find CARGO_MANIFEST_DIR?");
    let mut path = PathBuf::from(path);
    path.push(invocation.loc.value());
    let manifest = match fs::read_to_string(&path) {
        Ok(v) => v,
        Err(e) => error(
            args.span(),
            format_args!("Could not read manifest at '{}': {e}", path.display()),
        )?,
    };
    let manifest = match ParsedManifest::parse(&manifest) {
        Ok(v) => v,
        Err(e) => error(
            args.span(),
            format_args!("Could not parse manifest at '{}': {e}", path.display()),
        )?,
    };
    let path_str = path.to_string_lossy();

    // generates a unique hash
    let mut hasher = fnv::FnvHasher::with_key(0x1234123F);
    format!("{args} {:?}", args.span()).hash(&mut hasher);
    let manifest_hash = manifest.hash();
    manifest_hash.hash(&mut hasher);
    let hash = hasher.finish();

    // generate important names
    let module = Ident::new(&format!("__lgba_load_data__{hash:x}"), args.span());
    let exh_name = Ident::new(&format!("__lgba_load_data__{hash:x}__EXH"), args.span());

    // generate fields and types for each individual root
    let mut gen_types = Vec::new();
    let mut gen_impls = Vec::new();
    for (i, (root_name, root)) in manifest.roots.iter().enumerate() {
        let root_name_id = Ident::new(root_name.as_str(), args.span());
        let try_root_name_id = Ident::new(&format!("try_{root_name}"), args.span());
        let type_name = Ident::new(&format!("RootAccess_{root_name}"), args.span());

        let source = format!(
            "{}/{root_name}",
            manifest
                .name
                .as_ref()
                .map(|x| x.as_str())
                .unwrap_or("(unknown)")
        );

        // setup parameters for the code generation
        let (args, lookup_val, root_key) = match root.shape {
            ParsedSpecShape::Str => todo!(),
            ParsedSpecShape::U16 => (quote! { v: u16 }, quote! { v }, quote! { u16 }),
            ParsedSpecShape::U16U16 => {
                (quote! { a: u16, b: u16 }, quote! { (a, b) }, quote! { (u16, u16) })
            }
            ParsedSpecShape::U32 => (quote! { v: u32 }, quote! { v }, quote! { u32 }),
        };

        // generate the accessor methods on the root type
        let mut type_methods = Vec::new();
        for (j, (partition_name, _)) in root.partitions.iter().enumerate() {
            let part_id = Ident::new(partition_name.as_str(), args.span());
            type_methods.push(quote! {
                pub fn #part_id(&self) -> FileList {
                    self.0.partition_by_id(#j)
                }
            })
        }
        if root.partitions.contains_key("data") {
            type_methods.push(quote! {
                pub fn as_slice(&self) -> &'static [u8] {
                    self.data().as_slice()
                }
            });
        }

        // generate the actual types for this root
        gen_types.push(quote! {
            pub struct #type_name(EntryAccess);
            impl #type_name {
                #(#type_methods)*
            }
        });
        gen_impls.push(quote! {
            pub fn #try_root_name_id(&self, #args) -> Option<#type_name> {
                unsafe {
                    let raw = RootAccess::<#root_key>::new(&#exh_name, #i).get(#lookup_val);
                    raw.map(#type_name)
                }
            }
            pub fn #root_name_id(&self, #args) -> #type_name {
                unsafe {
                    let raw = RootAccess::<#root_key>::new(&#exh_name, #i).get(#lookup_val);
                    match raw {
                        Some(v) => #type_name(v),
                        None => not_found(#lookup_val, #source),
                    }
                }
            }
        });
    }

    // make invocation fields available to the quote block
    let vis = &invocation.vis;
    let name = &invocation.name;

    Ok(quote! {
        #vis struct #name;

        /// NOT PUBLIC API!
        const _: () = {
            mod #module {
                use ::lgba_data::__macro_export::*;

                // makes rustc recompile when data tomls are changed
                const _: &[u8] = include_bytes!(#path_str);

                mod exh_mod {
                    use super::*;
                    #[link_section = ".header"]
                    #[doc(hidden)]
                    #[no_mangle]
                    pub static #exh_name: ExHeader<DataHeader> = ExHeader::new(DataHeader {
                        hash: [#(#manifest_hash,)*],
                        roots: SerialSlice { ptr: 0, len: 0, _phantom: PhantomData },
                    });
                }

                extern {
                    pub static #exh_name: ExHeader<DataHeader>;
                }

                #(#gen_types)*

                impl super::#name {
                    #(#gen_impls)*
                }
            }
        };
    })
}
pub fn load_data_impl(args: TokenStream) -> TokenStream {
    match load_data_impl_0(args.into()) {
        Ok(v) => v.into(),
        Err(e) => e.into_compile_error().into(),
    }
}
