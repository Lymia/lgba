#[inline(never)]
fn invalid_glyph_id(min: usize, max: usize) {
    panic!("Glyph ID out of range: {min}..{max}");
}

mod private {
    pub trait CharDataSealed {
        unsafe fn write_vram(&self, ptr: *mut u32);
        unsafe fn write_vram_dma(&self, ptr: *mut u32);
    }
}

/// A type that can be used for character data.
pub trait CharData: private::CharDataSealed {}
impl private::CharDataSealed for [u32] {
    unsafe fn write_vram(&self, ptr: *mut u32) {
        todo!()
    }
    unsafe fn write_vram_dma(&self, ptr: *mut u32) {
        todo!()
    }
}
/// This storage type for character data is optimally efficient.
impl CharData for [u32] {}

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

    fn base_index(&self, id: usize) -> *mut u32 {
        if id < self.lower_bound || id >= self.upper_bound {
            invalid_glyph_id(self.lower_bound, self.upper_bound)
        }
        (self.base + 32 * id) as *mut u32
    }

    pub fn write_char_4bpp(&self, id: usize, data: &[u32; 8]) {}
}
