mod download_fonts;
mod gen_fonts;

const FONT_ASCII: gen_fonts::FontConfiguration = gen_fonts::FontConfiguration {
    font_name: "font_ascii",
    font_type: "TerminalFontAscii",
    description: "A minimal terminal font supporting only 7-bit ASCII characters.",
    low_plane_limit: 0x60,
    low_plane_dupe_limit: 0x60,
    misaki_override_blocks: &[],
    allow_all_blocks: false,
    whitelisted_blocks: &["Basic Latin"],
    glyph_whitelisted_blocks: &[],
    whitelisted_chars: &[],
    fallback_char: '?',
    kanji_max_level: kanji::Level::Ten,
    character_count: 0x60,
    delta: 1.0,
};
const FONT_BASIC: gen_fonts::FontConfiguration = gen_fonts::FontConfiguration {
    font_name: "font_basic",
    font_type: "TerminalFontBasic",
    description: "\
        A basic terminal font supporting many scripts and a limited number of characters useful \
        for rendering menus.\
    ",
    low_plane_limit: 0x300,
    low_plane_dupe_limit: 0x300,
    misaki_override_blocks: &[],
    allow_all_blocks: false,
    whitelisted_blocks: &[
        "Basic Latin",
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
    glyph_whitelisted_blocks: &["Greek Extended"],
    whitelisted_chars: &[
        '①', '②', '③', '④', '⑤', '⑥', '⑦', '⑧', '⑨', '■', '□', '●', '≠', '≤', '≥', '★', '♪', '⌚',
        '⌛', '⏩', '⏪', '█', '▉', '▊', '▋', '▌', '▍', '▁', '▂', '▃', '▄', '▅', '▆', '▇', '▎',
        '▏', '─', '│', '┌', '┐', '└', '┘', '├', '┤', '┬', '┴', '┼', '←', '↑', '→', '↓', '↔', '↕',
        '‐', '‑', '‒', '–', '—', '―', '†', '‡', '•', '․', '…', '⁇', '▲', '▶', '▼', '◀', '▀', '▐',
        '░', '▒', '▓', '○', '▖', '▗', '▘', '▙', '▚', '▛', '▜', '▝', '▞', '▟', '▩', '⌘', '♀', '♂',
    ],
    fallback_char: '⁇',
    kanji_max_level: kanji::Level::Ten,
    character_count: 0x340,
    delta: 2.0,
};
const FONT_FULL: gen_fonts::FontConfiguration = gen_fonts::FontConfiguration {
    font_name: "font_full",
    font_type: "TerminalFontFull",
    description: "\
        A terminal font supporting most characters from the source fonts.\n\
        \n\
        Only kanji on the jouyou list are included. This font is not suited for rendering \
        Chinese or Korean text.\
    ",
    low_plane_limit: 0x400,
    low_plane_dupe_limit: 0x400,
    misaki_override_blocks: &["Halfwidth and Fullwidth Forms"],
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
    glyph_whitelisted_blocks: &[],
    whitelisted_chars: &[],
    fallback_char: '⁇',
    kanji_max_level: kanji::Level::Two,
    character_count: 0xF80,
    delta: 2.0,
};

fn main() {
    let characters = download_fonts::download_fonts().expect("Could not download and parse fonts.");
    gen_fonts::print_all_blocks(&characters);
    gen_fonts::generate_fonts(&FONT_ASCII, &characters);
    gen_fonts::generate_fonts(&FONT_BASIC, &characters);
    gen_fonts::generate_fonts(&FONT_FULL, &characters);
}
