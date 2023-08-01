mod font_data;
mod gen_fonts;

use std::io::Write;
use std::process::{Command, Stdio};
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

fn rustfmt(tokens: TokenStream) -> String {
    let mut command = Command::new("rustfmt")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn().unwrap();

    command.stdin.as_mut().unwrap().write(tokens.to_string().as_bytes()).unwrap();
    command.stdin.take();

    let output = command.wait_with_output().unwrap();
    output.status.exit_ok().unwrap();
    String::from_utf8_lossy(&output.stdout).to_string()
}

pub fn make_terminal_font(name: &str, config: FontConfig) -> String {
    let ident = Ident::new(name, Span::call_site());
    let tokens = gen_fonts::generate_fonts(config, quote! { #ident }).unwrap();
    rustfmt(tokens)
}

pub use gen_fonts::FontConfig;
