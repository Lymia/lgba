#[rustfmt::skip]
mod font_ascii;
#[rustfmt::skip]
mod font_ascii_half;
#[rustfmt::skip]
mod font_basic;
#[rustfmt::skip]
mod font_full;

/// Represents a font that can be rendered in a terminal.
pub trait TerminalFont {
    /// Returns a static instance of this font.
    fn instance() -> &'static Self;
    /// Returns the glyph that represents a character.
    ///
    /// The first value (referred to as the plane) returned represents which bit of the character
    /// will be rendered, the second represents which character will be rendered.
    ///
    /// When the plane is set to `n`, the `(n+1)`th most significant bit of each pixel will
    /// determine if the pixel is set.
    fn get_font_glyph(&self, id: char) -> (u8, u16);
    /// Returns the raw character data used by the font.
    fn get_font_data(&self) -> &'static [u32];
}

pub use font_ascii::TerminalFontAscii;
pub use font_ascii_half::TerminalFontAsciiHalf;
pub use font_basic::TerminalFontBasic;
pub use font_full::TerminalFontFull;

/// A terminal display mode that makes it easy to display text..
pub struct Terminal {}
