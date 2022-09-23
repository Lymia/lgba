use crate::dma::DmaChannel;
use core::ffi::c_void;

#[inline(never)]
fn invalid_glyph_id(min: usize, max: usize) {
    panic!("Glyph ID out of range: {min}..{max}");
}

#[inline(never)]
fn char_count_incorrect() {
    panic!("Character data ends with an incomplete character.");
}

#[inline(never)]
fn char_id_is_odd() {
    panic!("8bpp character IDs must be even.");
}

mod private {
    use crate::dma::DmaChannel;

    pub trait CharDataSealed {
        fn char_count_4bpp(&self) -> usize;
        fn char_count_8bpp(&self) -> usize;
        unsafe fn write_vram(&self, ptr: *mut u32);
        unsafe fn write_vram_dma(&self, ch: DmaChannel, ptr: *mut u32);
    }
}

/// A type that can be used for character data.
///
/// Notably, due to the combined limitations of writing to VRAM on the GBA and the ARM
/// architecture, it is not efficient to store character data aligned to any alignment lower than
/// 2 bytes.
pub trait CharData: private::CharDataSealed {}
macro_rules! simple_char_data {
    ($ty:ty) => {
        impl private::CharDataSealed for [$ty] {
            fn char_count_4bpp(&self) -> usize {
                const COUNT: usize = 32 / core::mem::size_of::<$ty>();
                if self.len() % COUNT != 0 {
                    char_count_incorrect();
                }
                self.len() / COUNT
            }
            fn char_count_8bpp(&self) -> usize {
                const COUNT: usize = 64 / core::mem::size_of::<$ty>();
                if self.len() % COUNT != 0 {
                    char_count_incorrect();
                }
                self.len() / COUNT
            }
            unsafe fn write_vram(&self, ptr: *mut u32) {
                core::ptr::copy(self.as_ptr(), ptr as *mut $ty, self.len())
            }
            unsafe fn write_vram_dma(&self, mut ch: DmaChannel, ptr: *mut u32) {
                ch.unsafe_transfer(
                    self.as_ptr() as *const c_void,
                    ptr as *mut c_void,
                    self.len() * core::mem::size_of::<$ty>(),
                );
            }
        }
        impl CharData for [$ty] {}
    };
}
simple_char_data!(u32);
simple_char_data!(u16);

/// A helper type used to write character data into VRAM.
#[derive(Debug)]
pub struct CharAccess {
    base: usize,
    lower_bound: usize,
    upper_bound: usize,
}
impl CharAccess {
    pub(crate) fn new(base: usize, lower_bound: usize, upper_bound: usize) -> Self {
        CharAccess { base, lower_bound, upper_bound }
    }

    fn check_bounds(&self, id: usize, count: usize) {
        let end_id = id + count;
        if count >= 2048
            || id < self.lower_bound
            || id >= self.upper_bound
            || end_id < self.lower_bound
            || end_id >= self.upper_bound
        {
            invalid_glyph_id(self.lower_bound, self.upper_bound)
        }
    }
    fn base_index(&self, id: usize) -> *mut u32 {
        (self.base + 32 * id) as *mut u32
    }

    /// Writes 4bpp character data to the given character ID.
    pub fn write_char_4bpp(&self, id: usize, data: &impl CharData) {
        unsafe {
            self.check_bounds(id, data.char_count_4bpp());
            let offset = self.base_index(id);
            data.write_vram(offset as *mut u32);
        }
    }

    /// Writes 4bpp character data to the given character ID.
    pub fn write_char_4bpp_dma(&self, channel: DmaChannel, id: usize, data: &impl CharData) {
        unsafe {
            self.check_bounds(id, data.char_count_4bpp());
            let offset = self.base_index(id);
            data.write_vram_dma(channel, offset as *mut u32);
        }
    }

    /// Writes 8bpp character data to the given character ID.
    pub fn write_char_8bpp(&self, id: usize, data: &impl CharData) {
        unsafe {
            if id % 2 != 0 {
                char_id_is_odd();
            }
            self.check_bounds(id, data.char_count_8bpp());
            let offset = self.base_index(id);
            data.write_vram(offset as *mut u32);
        }
    }

    /// Writes 8bpp character data to the given character ID.
    pub fn write_char_8bpp_dma(&self, channel: DmaChannel, id: usize, data: &impl CharData) {
        unsafe {
            if id % 2 != 0 {
                char_id_is_odd();
            }
            self.check_bounds(id, data.char_count_8bpp());
            let offset = self.base_index(id);
            data.write_vram_dma(channel, offset as *mut u32);
        }
    }
}
