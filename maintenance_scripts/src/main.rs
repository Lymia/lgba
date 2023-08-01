#![feature(exit_status_error)]

mod build_fonts;

fn main() {
    build_fonts::make_terminal_font(
        "lgba/src/display/terminal/gen_font_ascii.rs",
        build_fonts::FontConfig {
            name: "TerminalFontAscii",
            description: "\
                A terminal font supporting only 7-bit ASCII characters.

                This font does not require additional storage space in the ROM, as it is used by
                the panic handler.
            "
            .trim(),
            low_plane_limit: None,
            disable_unscii: vec![],
            disable_misaki: vec![],
            chars: vec![],
            block: vec!["Basic Latin"],
            allow_halfwidth_blocks: vec![],
            fallback_char: None,
            kanji_max_level: None,
            delta: None,
        },
    );
    build_fonts::make_terminal_font(
        "lgba/src/display/terminal/gen_font_basic.rs",
        build_fonts::FontConfig {
            name: "TerminalFontBasic",
            description: "\
                A terminal font supporting many scripts and a reasonable selection of graphics
                characters for rendering menus.
            "
            .trim(),
            low_plane_limit: Some(0x400),
            disable_unscii: vec![],
            disable_misaki: vec![],
            chars: vec!["①②③④⑤⑥⑦⑧⑨■□●○★♪⌛⏩⏪←↑→↓↔↕‐‑‒–—―†‡•․…⁇▲▶▼◀▩⌘♀♂─│┌┐└┘├┤┬┴┼╭╮╯╰"],
            block: vec![
                "Basic Latin",
                "Block Elements",
                "CJK Symbols and Punctuation",
                "Currency Symbols",
                "Cyrillic",
                "Greek and Coptic",
                "Hiragana",
                "IPA Extensions",
                "Katakana",
                "Latin Extended Additional",
                "Latin Extended-A",
                "Latin Extended-B",
                "Latin-1 Supplement",
                "Supplemental Punctuation",
            ],
            allow_halfwidth_blocks: vec![],
            fallback_char: Some('⁇'),
            kanji_max_level: None,
            delta: None,
        },
    );
    build_fonts::make_terminal_font(
        "lgba/src/display/terminal/gen_font_full.rs",
        build_fonts::FontConfig {
            name: "TerminalFontFull",
            description: "\
                A terminal font supporting most characters from the source fonts.

                Only kanji on the jouyou list are included. This font is not suited for rendering
                Chinese or Korean text.
            "
            .trim(),
            low_plane_limit: Some(0x400),
            disable_unscii: vec!["Halfwidth and Fullwidth Forms"],
            disable_misaki: vec![],
            chars: vec![],
            block: vec![
                "Arrows",
                "Basic Latin",
                "Block Elements",
                "Box Drawing",
                "CJK Compatibility",
                "CJK Symbols and Punctuation",
                "CJK Unified Ideographs",
                "Currency Symbols",
                "Cyrillic",
                "Dingbats",
                "Enclosed Alphanumerics",
                "Enclosed CJK Letters and Months",
                "General Punctuation",
                "Geometric Shapes",
                "Greek Extended",
                "Greek and Coptic",
                "Halfwidth and Fullwidth Forms",
                "Hiragana",
                "IPA Extensions",
                "Katakana",
                "Latin Extended Additional",
                "Latin Extended-A",
                "Latin Extended-B",
                "Latin-1 Supplement",
                "Letterlike Symbols",
                "Mathematical Operators",
                "Miscellaneous Mathematical Symbols-B",
                "Miscellaneous Symbols",
                "Miscellaneous Symbols and Arrows",
                "Miscellaneous Technical",
                "Number Forms",
                "Runic",
                "Spacing Modifier Letters",
                "Superscripts and Subscripts",
                "Supplemental Punctuation",
                "Unified Canadian Aboriginal Syllabics",
            ],
            allow_halfwidth_blocks: vec!["Halfwidth and Fullwidth Forms"],
            fallback_char: Some('⁇'),
            kanji_max_level: Some("2"),
            delta: None,
        },
    );
}
