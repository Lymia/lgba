/// Includes a file as a reference to a `u32` array.
///
/// The file is read assuming it is in little-endian order. The file is located relative to the
/// current file (similarly to how modules are found). For further information, see the related
/// macro [`core::include_bytes!`].
///
/// This macro will yield an expression of type `&'static [u32; N]` which is the contents of the
/// file, interpreted as little-endian. If the file length is not a multiple of 4 bytes, this
/// macro will produce a compile error.
///
/// [`core::include_bytes!`]: https://doc.rust-lang.org/core/macro.include_bytes.html
#[macro_export]
macro_rules! include_u32 {
    ($file:expr $(,)?) => {
        &{
            const BYTES: &[u8] = $crate::__macro_export::core::include_bytes!($file);
            $crate::__macro_export::xfer_u8_u32::<{ BYTES.len() / 4 }>(BYTES)
        }
    };
}

/// Includes a file as a reference to a `u16` array.
///
/// The file is read assuming it is in little-endian order. The file is located relative to the
/// current file (similarly to how modules are found). For further information, see the related
/// macro [`core::include_bytes!`].
///
/// This macro will yield an expression of type `&'static [u16; N]` which is the contents of the
/// file, interpreted as little-endian. If the file length is not a multiple of 2 bytes, this
/// macro will produce a compile error.
///
/// [`core::include_bytes!`]: https://doc.rust-lang.org/core/macro.include_bytes.html
#[macro_export]
macro_rules! include_u16 {
    ($file:expr $(,)?) => {
        &{
            const BYTES: &[u8] = $crate::__macro_export::core::include_bytes!($file);
            $crate::__macro_export::xfer_u8_u16::<{ BYTES.len() / 2 }>(BYTES)
        }
    };
}

/// Includes a file as a reference to a `u8` array.
///
/// This is an alias to [`core::include_bytes!`], included for consistency with the other
/// lgba-specific include macros.
///
/// [`core::include_bytes!`]: https://doc.rust-lang.org/core/macro.include_bytes.html
#[macro_export]
macro_rules! include_u8 {
    ($file:expr $(,)?) => {
        $crate::__macro_export::core::include_bytes!($file)
    };
}
