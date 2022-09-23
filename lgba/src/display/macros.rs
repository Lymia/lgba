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
            if BYTES.len() % 4 != 0 {
                $crate::__macro_export::core::panic!("File length is not multiple of 4!")
            }
            let mut u32_data = [0u32; BYTES.len() / 4];

            let mut i = 0;
            while i < u32_data.len() {
                u32_data[i] = u32::from_le_bytes([
                    BYTES[i * 4],
                    BYTES[i * 4 + 1],
                    BYTES[i * 4 + 2],
                    BYTES[i * 4 + 3],
                ]);
                i += 1;
            }

            u32_data
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
            if BYTES.len() % 2 != 0 {
                $crate::__macro_export::core::panic!("File length is not multiple of 2!")
            }
            let mut u16_data = [0u16; BYTES.len() / 2];

            let mut i = 0;
            while i < u16_data.len() {
                u16_data[i] = u16::from_le_bytes([BYTES[i * 2], BYTES[i * 2 + 1]]);
                i += 1;
            }

            u16_data
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
