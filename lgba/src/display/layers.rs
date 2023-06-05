use crate::{
    display::{vram::MapAccess, CharAccess},
    mmio::{display::BgCnt, reg::*},
};

const BG_CNT: [Register<BgCnt, SafeReg>; 4] = [BG0CNT, BG1CNT, BG2CNT, BG3CNT];
const BG_HOFS: [Register<i16, SafeReg>; 4] = [BG0HOFS, BG1HOFS, BG2HOFS, BG3HOFS];
const BG_VOFS: [Register<i16, SafeReg>; 4] = [BG0VOFS, BG1VOFS, BG2VOFS, BG3VOFS];

// TODO: Introduce a autogenerating procedural macro to this module.

#[derive(Copy, Clone, Debug)]
#[repr(u8)]
pub enum LayerId {
    Layer0,
    Layer1,
    Layer2,
    Layer3,
}

/// The tile size of a layer.
#[derive(Copy, Clone, Debug)]
#[repr(u8)]
pub enum TileLayerSize {
    Map256x256,
    Map512x256,
    Map256x512,
    Map512x512,
}
impl TileLayerSize {
    /// Returns the number of tile maps used by this layer size
    pub fn map_count(&self) -> usize {
        match self {
            TileLayerSize::Map256x256 => 1,
            TileLayerSize::Map512x256 => 2,
            TileLayerSize::Map256x512 => 2,
            TileLayerSize::Map512x512 => 4,
        }
    }
}

macro_rules! standard_ops_inactive {
    () => {
        /// Whether this layer is enabled.
        pub fn enabled(&self) -> bool {
            self.is_enabled
        }

        /// Sets whether this layer is enabled.
        pub fn set_enabled(&mut self, enabled: bool) -> &mut Self {
            self.is_enabled = enabled;
            self
        }

        /// Whether the horizontal offset of this layer.
        pub fn h_offsets(&self) -> i16 {
            self.h_offset
        }

        /// Sets the horizontal offset of this layer.
        pub fn set_h_offset(&mut self, value: i16) -> &mut Self {
            self.h_offset = value;
            self
        }

        /// Whether the vertical offset of this layer.
        pub fn v_offset(&self) -> i16 {
            self.v_offset
        }

        /// Sets the vertical offset of this layer.
        pub fn set_v_offset(&mut self, value: i16) -> &mut Self {
            self.v_offset = value;
            self
        }

        /// Returns the offset of this layer
        pub fn offset(&self) -> (i16, i16) {
            (self.h_offset, self.v_offset)
        }

        /// Sets the offset of this layer.
        pub fn set_offset(&mut self, x: i16, y: i16) -> &mut Self {
            self.set_h_offset(x);
            self.set_v_offset(y);
            self
        }
    };
}
macro_rules! standard_ops_active {
    ($guard:ident, $lt:lifetime) => {
        /// Whether this layer is enabled.
        pub fn enabled(&self) -> bool {
            self.layer.enabled()
        }

        /// Sets whether this layer is enabled.
        pub fn set_enabled<'b>(&'b mut self, enabled: bool) -> $guard<'b, $lt> {
            self.layer.set_enabled(enabled);
            $guard::new(self).mark_enabled_dirty()
        }

        /// Whether the horizontal offset of this layer.
        pub fn h_offsets(&self) -> i16 {
            self.layer.h_offsets()
        }

        /// Sets the horizontal offset of this layer.
        pub fn set_h_offset<'b>(&'b mut self, value: i16) -> $guard<'b, $lt> {
            self.layer.set_h_offset(value);
            $guard::new(self).mark_hoff_dirty()
        }

        /// Whether the vertical offset of this layer.
        pub fn v_offset(&self) -> i16 {
            self.layer.v_offset()
        }

        /// Sets the vertical offset of this layer.
        pub fn set_v_offset<'b>(&'b mut self, value: i16) -> $guard<'b, $lt> {
            self.layer.set_v_offset(value);
            $guard::new(self).mark_voff_dirty()
        }

        /// Returns the offset of this layer
        pub fn offset(&self) -> (i16, i16) {
            self.layer.offset()
        }

        /// Sets the offset of this layer.
        pub fn set_offset<'b>(&'b mut self, x: i16, y: i16) -> $guard<'b, $lt> {
            self.layer.set_offset(x, y);
            $guard::new(self).mark_hoff_dirty().mark_voff_dirty()
        }
    };
}
macro_rules! standard_ops_guard {
    () => {
        /// Sets whether this layer is enabled.
        pub fn set_enabled(self, enabled: bool) -> Self {
            self.layer.layer.set_enabled(enabled);
            self.mark_enabled_dirty()
        }

        /// Sets the horizontal offset of this layer.
        pub fn set_h_offset(self, value: i16) -> Self {
            self.layer.layer.set_h_offset(value);
            self.mark_hoff_dirty()
        }

        /// Sets the vertical offset of this layer.
        pub fn set_v_offset(self, value: i16) -> Self {
            self.layer.layer.set_v_offset(value);
            self.mark_voff_dirty()
        }

        /// Sets the offset of this layer.
        pub fn set_offset(self, x: i16, y: i16) -> Self {
            self.layer.layer.set_offset(x, y);
            self.mark_hoff_dirty().mark_voff_dirty()
        }
    };
}

/// A tile layer that is not currently active.
#[derive(Debug)]
pub struct TileLayer {
    pub(crate) id: LayerId,
    cnt: BgCnt,
    h_offset: i16,
    v_offset: i16,
    is_enabled: bool,
}
impl TileLayer {
    pub(crate) fn new(id: LayerId) -> TileLayer {
        TileLayer { id, cnt: BgCnt::default(), h_offset: 0, v_offset: 0, is_enabled: false }
    }

    fn write_cnt(&self) {
        BG_CNT[self.id as usize].write(self.cnt);
    }
    fn write_hoff(&self) {
        BG_HOFS[self.id as usize].write(-self.h_offset);
    }
    fn write_voff(&self) {
        BG_VOFS[self.id as usize].write(-self.v_offset);
    }
    pub(crate) fn write_enabled_from_guard(&self) {
        // TODO: Cleanup
        let mut disp_cnt = DISPCNT.read();
        match self.id {
            LayerId::Layer0 => disp_cnt = disp_cnt.with_display_bg0(self.is_enabled),
            LayerId::Layer1 => disp_cnt = disp_cnt.with_display_bg1(self.is_enabled),
            LayerId::Layer2 => disp_cnt = disp_cnt.with_display_bg2(self.is_enabled),
            LayerId::Layer3 => disp_cnt = disp_cnt.with_display_bg3(self.is_enabled),
        }
        DISPCNT.write(disp_cnt)
    }
    fn write_all(&self) {
        if self.is_enabled {
            self.write_cnt();
            self.write_hoff();
            self.write_voff();
        }
    }

    pub(crate) fn activate(&mut self) -> ActiveTileLayer {
        self.write_all();
        ActiveTileLayer { layer: self }
    }

    /// Returns a character access appropriate for this layer.
    pub fn char_access(&self) -> CharAccess {
        let base = VRAM_BASE + 16 * 1024 * self.cnt.char_base();
        let available_chars = core::cmp::min(1024, (VRAM_END - base) / 32);
        CharAccess::new(base, 0, available_chars)
    }

    /// Returns a tile map access appropriate for this layer.
    pub fn map_access(&self, screen: usize) -> MapAccess {
        let map_count = self.tile_map_size().map_count();
        if screen >= map_count {
            invalid_map_access_screen(map_count);
        }
        let raw_screen = (self.tile_base() + screen) % 32;
        let base = VRAM_BASE + 2048 * raw_screen;
        MapAccess::new(base, 5)
    }

    standard_ops_inactive!();

    /// The priority of this layer.
    ///
    /// 0 is the highest, and 3 is the lowest. This function panics if any other values are used.
    pub fn bg_priority(&self) -> u32 {
        self.cnt.bg_priority()
    }

    /// Sets whether the priority of this layer.
    ///
    /// 0 is the highest, and 3 is the lowest. This function panics if any other values are used.
    pub fn set_bg_priority(&mut self, value: u32) -> &mut Self {
        self.cnt = self.cnt.with_bg_priority(value);
        self
    }

    /// The character data base for this layer.
    ///
    /// The value must be between 0 and 3, inclusive.
    pub fn char_base(&self) -> usize {
        self.cnt.char_base()
    }

    /// Sets the character data base for this layer.
    ///
    /// The value must be between 0 and 3, inclusive.
    pub fn set_char_base(&mut self, value: usize) -> &mut Self {
        self.cnt = self.cnt.with_char_base(value);
        self
    }

    /// Whether the mosaic effect is enabled for this layer.
    pub fn mosaic_enabled(&self) -> bool {
        self.cnt.enable_mosaic()
    }

    /// Sets whether the mosaic effect is enabled for this layer.
    pub fn set_mosaic_enabled(&mut self, value: bool) -> &mut Self {
        self.cnt = self.cnt.with_mosaic(value);
        self
    }

    /// Whether to use 256-color palettes for this layer.
    pub fn enable_256_color(&self) -> bool {
        self.cnt.enable_256_color()
    }

    /// Sets whether to use 256-color palettes for this layer.
    pub fn set_enable_256_color(&mut self, value: bool) -> &mut Self {
        self.cnt = self.cnt.with_enable_256_color(value);
        self
    }

    /// The tile map data base for this layer.
    pub fn tile_base(&self) -> usize {
        self.cnt.tile_map_base()
    }

    /// Sets the tile map data base for this layer.
    pub fn set_tile_base(&mut self, value: usize) -> &mut Self {
        self.cnt = self.cnt.with_tile_map_base(value);
        self
    }

    /// The tile map size for this layer.
    pub fn tile_map_size(&self) -> TileLayerSize {
        match self.cnt.screen_size() {
            0 => TileLayerSize::Map256x256,
            1 => TileLayerSize::Map512x256,
            2 => TileLayerSize::Map256x512,
            3 => TileLayerSize::Map512x512,
            _ => unreachable!(),
        }
    }

    /// Sets the tile map size for this layer.
    pub fn set_tile_map_size(&mut self, value: TileLayerSize) -> &mut Self {
        self.cnt = self.cnt.with_screen_size(value as u32);
        self
    }
}

#[inline(never)]
fn invalid_map_access_screen(max_screen: usize) {
    panic!("Screen id out of range: 0..{max_screen}");
}

#[derive(Debug)]
pub struct ActiveTileLayer<'a> {
    layer: &'a mut TileLayer,
}
impl<'a> ActiveTileLayer<'a> {
    /// Returns a character access appropriate for this layer.
    pub fn char_access(&self) -> CharAccess {
        self.layer.char_access()
    }

    /// Returns a tile map access appropriate for this layer.
    pub fn map_access(&self, screen: usize) -> MapAccess {
        self.layer.map_access(screen)
    }

    standard_ops_active!(ActiveTileLayerEditGuard, 'a);

    /// The priority of this layer.
    ///
    /// 0 is the highest, and 3 is the lowest. This function panics if any other values are used.
    pub fn bg_priority(&self) -> u32 {
        self.layer.bg_priority()
    }

    /// Sets whether the priority of this layer.
    ///
    /// 0 is the highest, and 3 is the lowest. This function panics if any other values are used.
    pub fn set_bg_priority<'b>(&'b mut self, value: u32) -> ActiveTileLayerEditGuard<'b, 'a> {
        self.layer.set_bg_priority(value);
        ActiveTileLayerEditGuard::new(self).mark_cnt_dirty()
    }

    /// The character data base for this layer.
    ///
    /// The value must be between 0 and 3, inclusive.
    pub fn char_base(&self) -> usize {
        self.layer.char_base()
    }

    /// Sets the character data base for this layer.
    ///
    /// The value must be between 0 and 3, inclusive.
    pub fn set_char_base<'b>(&'b mut self, value: usize) -> ActiveTileLayerEditGuard<'b, 'a> {
        self.layer.set_char_base(value);
        ActiveTileLayerEditGuard::new(self).mark_cnt_dirty()
    }

    /// Whether the mosaic effect is enabled for this layer.
    pub fn mosaic_enabled(&self) -> bool {
        self.layer.mosaic_enabled()
    }

    /// Sets whether the mosaic effect is enabled for this layer.
    pub fn set_mosaic_enabled<'b>(&'b mut self, value: bool) -> ActiveTileLayerEditGuard<'b, 'a> {
        self.layer.set_mosaic_enabled(value);
        ActiveTileLayerEditGuard::new(self).mark_cnt_dirty()
    }

    /// Whether to use 256-color palettes for this layer.
    pub fn enable_256_color(&self) -> bool {
        self.layer.enable_256_color()
    }

    /// Sets whether to use 256-color palettes for this layer.
    pub fn set_enable_256_color<'b>(
        &'b mut self,
        value: bool,
    ) -> ActiveTileLayerEditGuard<'b, 'a> {
        self.layer.set_enable_256_color(value);
        ActiveTileLayerEditGuard::new(self).mark_cnt_dirty()
    }

    /// The tile map data base for this layer.
    pub fn tile_base(&self) -> usize {
        self.layer.tile_base()
    }

    /// Sets the tile map data base for this layer.
    pub fn set_tile_base<'b>(&'b mut self, value: usize) -> ActiveTileLayerEditGuard<'b, 'a> {
        self.layer.set_tile_base(value);
        ActiveTileLayerEditGuard::new(self).mark_cnt_dirty()
    }

    /// The tile map size for this layer.
    pub fn tile_map_size(&self) -> TileLayerSize {
        self.layer.tile_map_size()
    }

    /// Sets the tile map size for this layer.
    pub fn set_tile_map_size<'b>(
        &'b mut self,
        value: TileLayerSize,
    ) -> ActiveTileLayerEditGuard<'b, 'a> {
        self.layer.set_tile_map_size(value);
        ActiveTileLayerEditGuard::new(self).mark_cnt_dirty()
    }
}

/// A temporary guard created to allow chaining operations on a [`ActiveTileLayer`], and only
/// writing once to the memory mapped IO.
#[derive(Debug)]
pub struct ActiveTileLayerEditGuard<'a, 'b: 'a> {
    layer: &'a mut ActiveTileLayer<'b>,
    is_cnt_dirty: bool,
    is_hoff_dirty: bool,
    is_voff_dirty: bool,
    is_enabled_dirty: bool,
}
impl<'a, 'b: 'a> ActiveTileLayerEditGuard<'a, 'b> {
    pub(crate) fn new(layer: &'a mut ActiveTileLayer<'b>) -> Self {
        ActiveTileLayerEditGuard {
            layer,
            is_cnt_dirty: false,
            is_hoff_dirty: false,
            is_voff_dirty: false,
            is_enabled_dirty: false,
        }
    }

    fn mark_cnt_dirty(mut self) -> Self {
        self.is_cnt_dirty = true;
        self
    }
    fn mark_hoff_dirty(mut self) -> Self {
        self.is_hoff_dirty = true;
        self
    }
    fn mark_voff_dirty(mut self) -> Self {
        self.is_voff_dirty = true;
        self
    }
    fn mark_enabled_dirty(mut self) -> Self {
        self.is_enabled_dirty = true;
        self
    }

    standard_ops_guard!();

    /// Sets whether the priority of this layer.
    ///
    /// 0 is the highest, and 3 is the lowest. This function panics if any other values are used.
    pub fn set_bg_priority(self, value: u32) -> Self {
        self.layer.layer.set_bg_priority(value);
        self.mark_cnt_dirty()
    }

    /// Sets the character data base for this layer.
    ///
    /// The value must be between 0 and 3, inclusive.
    pub fn set_char_base(self, value: usize) -> Self {
        self.layer.layer.set_char_base(value);
        self.mark_cnt_dirty()
    }

    /// Sets whether the mosaic effect is enabled for this layer.
    pub fn set_mosaic_enabled(self, value: bool) -> Self {
        self.layer.layer.set_mosaic_enabled(value);
        self.mark_cnt_dirty()
    }

    /// Sets whether to use 256-color palettes for this layer.
    pub fn set_enable_256_color(self, value: bool) -> Self {
        self.layer.layer.set_enable_256_color(value);
        self.mark_cnt_dirty()
    }

    /// Sets the tile map data base for this layer.
    pub fn set_tile_base(self, value: usize) -> Self {
        self.layer.layer.set_tile_base(value);
        self.mark_cnt_dirty()
    }

    /// Sets the tile map size for this layer.
    pub fn set_tile_map_size(self, value: TileLayerSize) -> Self {
        self.layer.layer.set_tile_map_size(value);
        self.mark_cnt_dirty()
    }
}
impl<'a, 'b: 'a> Drop for ActiveTileLayerEditGuard<'a, 'b> {
    fn drop(&mut self) {
        if self.is_cnt_dirty {
            self.layer.layer.write_cnt();
        }
        if self.is_hoff_dirty {
            self.layer.layer.write_hoff();
        }
        if self.is_voff_dirty {
            self.layer.layer.write_voff();
        }
        if self.is_enabled_dirty {
            self.layer.layer.write_enabled_from_guard();
        }
    }
}
