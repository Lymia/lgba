use crate::dma::DmaChannel;
use core::ffi::c_void;

mod private {
    use crate::dma::DmaChannel;

    pub trait CharDataSealed {
        fn char_count_4bpp(&self) -> usize;
        fn char_count_8bpp(&self) -> usize;
        unsafe fn write_vram(&self, ptr: *mut u32);
        unsafe fn write_vram_dma(&self, ch: DmaChannel, ptr: *mut u32);
    }
}

unsafe fn copy_volatile<T>(mut src: *const T, mut dst: *mut T, len: usize) {
    for _ in 0..len {
        core::ptr::write_volatile(dst, core::ptr::read_volatile(src));
        src = src.offset(1);
        dst = dst.offset(1);
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
                copy_volatile(self.as_ptr(), ptr as *mut $ty, self.len())
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
#[derive(Copy, Clone, Debug)]
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
    pub fn write_char_4bpp_dma(
        &self,
        channel: DmaChannel,
        id: usize,
        data: &(impl CharData + ?Sized),
    ) {
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
    pub fn write_char_8bpp_dma(
        &self,
        channel: DmaChannel,
        id: usize,
        data: &(impl CharData + ?Sized),
    ) {
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

/// A helper type used to write data into tile maps.
#[derive(Copy, Clone, Debug)]
pub struct MapAccess {
    base: usize,
    map_scale: usize,
    map_mask: usize,
    map_size: usize,
}
impl MapAccess {
    pub(crate) fn new(base: usize, scale_shift: usize) -> Self {
        let scale = 1 << scale_shift;
        MapAccess { base, map_scale: scale, map_mask: scale - 1, map_size: scale * scale }
    }

    fn index(&self, x: usize, y: usize) -> usize {
        if x >= self.map_scale || y >= self.map_scale {
            invalid_tile_map_coordinate(self.map_scale);
        }
        x + (y << self.map_mask)
    }
    fn check_bounds(&self, x: usize, y: usize, count: usize) {
        let start_idx = self.index(x, y);
        let end_idx = start_idx + count;

        if end_idx >= self.map_size {
            invalid_tile_map_coordinate(self.map_scale);
        }
    }
    fn base_index(&self, x: usize, y: usize) -> *mut VramTile {
        (self.base + self.index(x, y)) as *mut VramTile
    }

    /// Sets a coordinate to a given tile.
    pub fn set_tile(&self, x: usize, y: usize, tile: VramTile) {
        unsafe { core::ptr::write_volatile(self.base_index(x, y), tile) }
    }

    /// Sets the data in the tile map starting at a given coordinate.
    ///
    /// The list of tiles is laid out horizontally, and will roll over to the start of the next
    /// row if it reaches the end of a row.
    pub fn set_tiles(&self, x: usize, y: usize, tile: &[VramTile]) {
        self.check_bounds(x, y, tile.len());
        unsafe {
            copy_volatile(tile.as_ptr(), self.base_index(x, y), tile.len());
        }
    }
}

#[inline(never)]
fn invalid_tile_map_coordinate(scale_max: usize) {
    panic!("Tile map coordinate out of range: 0..{scale_max}");
}

#[inline(never)]
fn invalid_screen_block() {
    panic!("Screen block out of range: 0..=31");
}

#[doc(inline)]
pub use crate::mmio::display::VramTile;
