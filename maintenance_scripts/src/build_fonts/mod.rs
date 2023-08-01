mod font_data;
mod gen_fonts;

use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use std::{
    io::Write,
    process::{Command, Stdio},
};

#[derive(Clone, Debug)]
pub struct FontConfig {
    pub name: &'static str,
    pub description: &'static str,
    pub low_plane_limit: Option<usize>,
    pub disable_unscii: Vec<&'static str>,
    pub disable_misaki: Vec<&'static str>,
    pub chars: Vec<&'static str>,
    pub block: Vec<&'static str>,
    pub allow_halfwidth_blocks: Vec<&'static str>,
    pub fallback_char: Option<char>,
    pub kanji_max_level: Option<&'static str>,
    pub delta: Option<f32>,
}

fn rustfmt(tokens: TokenStream) -> String {
    let mut command = Command::new("rustfmt")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    command
        .stdin
        .as_mut()
        .unwrap()
        .write(tokens.to_string().as_bytes())
        .unwrap();
    command.stdin.take();

    let output = command.wait_with_output().unwrap();
    output.status.exit_ok().unwrap();
    String::from_utf8_lossy(&output.stdout).to_string()
}

pub fn make_terminal_font(target: &str, config: FontConfig) {
    let ident = Ident::new(config.name, Span::call_site());
    let doc = config.description;
    let tokens = gen_fonts::generate_fonts(config, quote! { #ident }).unwrap();
    let source = rustfmt(quote! {
        #[doc = #doc]
        pub enum #ident {}
        #tokens
    });
    std::fs::write(target, format!("// This is generated code. Do not edit.\n{source}")).unwrap();
}
