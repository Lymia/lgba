#![feature(exit_status_error)]

mod build_fonts;

fn main() {
    println!(
        "{}",
        build_fonts::make_terminal_font("Test", build_fonts::FontConfig {
            low_plane_limit: None,
            disable_unscii: vec![],
            disable_misaki: vec![],
            chars: vec![],
            block: vec!["Basic Latin".to_string()],
            allow_halfwidth_blocks: vec![],
            fallback_char: None,
            kanji_max_level: None,
            delta: None,
        })
    );
    println!("?");
}
