mod font_data;
mod gen_fonts;

const FONT_ASCII: gen_fonts::FontConfiguration = gen_fonts::FontConfiguration {
    font_name: "font_ascii",
    font_type: "TerminalFontAscii",
    description: "An 8x8 terminal font supporting only 7-bit ASCII characters.",
    low_plane_limit: 0x60,
    low_plane_dupe_limit: 0x60,
    disable_unscii: false,
    unscii_blocks: &[],
    disable_misaki: false,
    misaki_blocks: &[],
    allow_all_blocks: false,
    whitelisted_blocks: &["Basic Latin"],
    whitelisted_chars: &[],
    allow_halfwidth_blocks: &[],
    fallback_char: '?',
    kanji_max_level: kanji::Level::Ten,
    character_count: 0x60,
    delta: 1.0,
};
const FONT_ASCII_HALF: gen_fonts::FontConfiguration = gen_fonts::FontConfiguration {
    font_name: "font_ascii_half",
    font_type: "TerminalFontAsciiHalf",
    description: "An 8x4 terminal font supporting only 7-bit ASCII characters.",
    low_plane_limit: 0x60,
    low_plane_dupe_limit: 0x60,
    disable_unscii: true,
    unscii_blocks: &[],
    disable_misaki: false,
    misaki_blocks: &[],
    allow_all_blocks: false,
    whitelisted_blocks: &["Basic Latin"],
    whitelisted_chars: &[],
    allow_halfwidth_blocks: &["Basic Latin"],
    fallback_char: '?',
    kanji_max_level: kanji::Level::Ten,
    character_count: 0x60,
    delta: 1.0,
};
const FONT_BASIC: gen_fonts::FontConfiguration = gen_fonts::FontConfiguration {
    font_name: "font_basic",
    font_type: "TerminalFontBasic",
    description: "\
        An 8x8 basic terminal font supporting many scripts and a reasonable selection of \
        graphical characters useful for rendering menus.
    ",
    low_plane_limit: 0x300,
    low_plane_dupe_limit: 0x300,
    disable_unscii: false,
    unscii_blocks: &[],
    disable_misaki: false,
    misaki_blocks: &[],
    allow_all_blocks: false,
    whitelisted_blocks: &[
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
    whitelisted_chars: &[
        '①', '②', '③', '④', '⑤', '⑥', '⑦', '⑧', '⑨', '■', '□', '●', '★', '♪', '⌛', '⏩', '⏪',
        '─', '│', '┌', '┐', '└', '┘', '├', '┤', '┬', '┴', '┼', '╭', '╮', '╯', '╰', '←', '↑', '→',
        '↓', '↔', '↕', '‐', '‑', '‒', '–', '—', '―', '†', '‡', '•', '․', '…', '⁇', '▲', '▶', '▼',
        '◀', '▩', '⌘', '♀', '♂',
    ],
    allow_halfwidth_blocks: &[],
    fallback_char: '⁇',
    kanji_max_level: kanji::Level::Ten,
    character_count: 0x400,
    delta: 2.0,
};
const FONT_FULL: gen_fonts::FontConfiguration = gen_fonts::FontConfiguration {
    font_name: "font_full",
    font_type: "TerminalFontFull",
    description: "\
        An 8x8 terminal font supporting most characters from the source fonts.\n\
        \n\
        Only kanji on the jouyou list are included. This font is not suited for rendering \
        Chinese or Korean text.\
    ",
    low_plane_limit: 0x400,
    low_plane_dupe_limit: 0x400,
    disable_unscii: false,
    unscii_blocks: &[],
    disable_misaki: false,
    misaki_blocks: &["Halfwidth and Fullwidth Forms"],
    allow_all_blocks: false,
    whitelisted_blocks: &[
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
    whitelisted_chars: &[],
    allow_halfwidth_blocks: &["Halfwidth and Fullwidth Forms"],
    fallback_char: '⁇',
    kanji_max_level: kanji::Level::Two,
    character_count: 0xF80,
    delta: 2.0,
};

fn main() {
    let characters = font_data::parse_fonts().expect("Could not parse included fonts??");
    gen_fonts::print_all_blocks(&characters);
    gen_fonts::generate_fonts(&FONT_ASCII, &characters);
    gen_fonts::generate_fonts(&FONT_ASCII_HALF, &characters);
    gen_fonts::generate_fonts(&FONT_BASIC, &characters);
    gen_fonts::generate_fonts(&FONT_FULL, &characters);
}
