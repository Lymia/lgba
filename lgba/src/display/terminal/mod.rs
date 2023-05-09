/// Represents a font that can be rendered in a terminal.
pub trait TerminalFont {
    /// Returns the glyph that represents a character.
    ///
    /// The first value (referred to as the plane) returned represents which bit of the character
    /// will be rendered, the second represents which character will be rendered, and the third
    /// represents whether the glyph is half-width.
    ///
    /// When the plane is set to `n`, the `(n+1)`th most significant bit of each pixel will
    /// determine if the pixel is set.
    fn get_font_glyph(id: char) -> (u8, u16, bool);
    /// Returns the raw character data used by the font.
    fn get_font_data() -> &'static [u32];
    /// Returns whether the font has any half-width characters.
    fn has_half_width() -> bool;
}
pub use lgba_macros::TerminalFont; // allow users to make custom terminal fonts

/// A terminal font supporting only 7-bit ASCII characters.
#[derive(TerminalFont)]
#[font(block = "Basic Latin")]
pub enum TerminalFontAscii {}

/// A terminal font supporting many scripts and a reasonable selection of graphics characters for
/// rendering menus.
#[derive(TerminalFont)]
#[font(low_plane_limit = 0x400)]
#[font(block = "Basic Latin")]
#[font(block = "Block Elements")]
#[font(block = "CJK Symbols and Punctuation")]
#[font(block = "Currency Symbols")]
#[font(block = "Cyrillic")]
#[font(block = "Greek and Coptic")]
#[font(block = "Hiragana")]
#[font(block = "IPA Extensions")]
#[font(block = "Katakana")]
#[font(block = "Latin Extended Additional")]
#[font(block = "Latin Extended-A")]
#[font(block = "Latin Extended-B")]
#[font(block = "Latin-1 Supplement")]
#[font(block = "Supplemental Punctuation")]
#[font(chars = "①②③④⑤⑥⑦⑧⑨■□●★♪⌛⏩⏪↓↔↕‐‑‒–—―†‡•․…⁇▲▶▼◀▩⌘♀♂←↑→")]
#[font(chars = "─│┌┐└┘├┤┬┴┼╭╮╯╰")]
#[font(fallback_char = "⁇")]
pub enum TerminalFontBasic {}

/// A terminal font supporting most characters from the source fonts.
///
/// Only kanji on the jouyou list are included. This font is not suited for rendering Chinese
/// or Korean text.
#[derive(TerminalFont)]
#[font(low_plane_limit = 0x400)]
#[font(block = "Arrows")]
#[font(block = "Basic Latin")]
#[font(block = "Block Elements")]
#[font(block = "Box Drawing")]
#[font(block = "CJK Compatibility")]
#[font(block = "CJK Symbols and Punctuation")]
#[font(block = "CJK Unified Ideographs")]
#[font(block = "Currency Symbols")]
#[font(block = "Cyrillic")]
#[font(block = "Dingbats")]
#[font(block = "Enclosed Alphanumerics")]
#[font(block = "Enclosed CJK Letters and Months")]
#[font(block = "General Punctuation")]
#[font(block = "Geometric Shapes")]
#[font(block = "Greek Extended")]
#[font(block = "Greek and Coptic")]
#[font(block = "Halfwidth and Fullwidth Forms")]
#[font(block = "Hiragana")]
#[font(block = "IPA Extensions")]
#[font(block = "Katakana")]
#[font(block = "Latin Extended Additional")]
#[font(block = "Latin Extended-A")]
#[font(block = "Latin Extended-B")]
#[font(block = "Latin-1 Supplement")]
#[font(block = "Letterlike Symbols")]
#[font(block = "Mathematical Operators")]
#[font(block = "Miscellaneous Mathematical Symbols-B")]
#[font(block = "Miscellaneous Symbols")]
#[font(block = "Miscellaneous Symbols and Arrows")]
#[font(block = "Miscellaneous Technical")]
#[font(block = "Number Forms")]
#[font(block = "Runic")]
#[font(block = "Spacing Modifier Letters")]
#[font(block = "Superscripts and Subscripts")]
#[font(block = "Supplemental Punctuation")]
#[font(block = "Unified Canadian Aboriginal Syllabics")]
#[font(allow_halfwidth_blocks = "Halfwidth and Fullwidth Forms")]
#[font(kanji_max_level = "2")]
#[font(fallback_char = "⁇")]
pub enum TerminalFontFull {}

/// A terminal display mode that makes it easy to display text..
pub struct Terminal {}
