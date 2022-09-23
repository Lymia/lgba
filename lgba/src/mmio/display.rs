use crate::mmio::prelude::*;
use enumset::{EnumSet, EnumSetType};
use num_enum::{IntoPrimitive, TryFromPrimitive};

/// Represents one of the graphical display modes
#[derive(IntoPrimitive, TryFromPrimitive)]
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug, Hash)]
#[repr(u16)]
pub enum DispMode {
    /// A graphics mode with four tile-based background layers.
    Mode0,
    /// A graphics mode with three tile-based background layers, one of which can be rotated and
    /// scaled.
    Mode1,
    /// A graphics mode with two tiled background layers, both of which can be rotated and scaled.
    Mode2,
    /// A graphics mode with a single 16bpp full-resolution bitmap background layer that can be
    /// rotated and scaled.
    Mode3,
    /// A graphics mode with a single double-buffered paletted 8bpp full-resolution bitmap
    /// background layer that can be rotated and scaled.
    Mode4,
    /// A graphics mode with a single double-buffered 16bpp half-resolution bitmap background
    /// layer.
    Mode5,
}

/// Controls the overall behavior of the LCD display.
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug, Hash, Default)]
#[repr(transparent)]
pub struct DispCnt(u16);
#[rustfmt::skip]
packed_struct_fields!(
    DispCnt, u16,

    // Not directly documented here, as this API will only be used internally.
    (mode, with_mode, DispMode, 0..=2),
    (active_frame, with_active_frame, usize, 4..=4),
    (hblank_oam_access, with_hblank_oam_access, bool, 5),
    (use_2d_obj_vram, with_use_2d_obj_vram, bool, 6),
    (forced_blank, with_forced_blank, bool, 7),
    (display_bg0, with_display_bg0, bool, 8),
    (display_bg1, with_display_bg1, bool, 9),
    (display_bg2, with_display_bg2, bool, 10),
    (display_bg3, with_display_bg3, bool, 11),
    (display_obj, with_display_obj, bool, 12),
    (use_window_0, with_use_window_0, bool, 13),
    (use_window_1, with_use_window_1, bool, 14),
    (use_obj_window, with_use_obj_window, bool, 15),
);

/// Used to retrieve the status of graphics rendering, and control rendering interrupts.
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug, Hash, Default)]
#[repr(transparent)]
pub struct DispStat(u16);
#[rustfmt::skip]
packed_struct_fields!(
    DispStat, u16,

    /// Whether the graphics chip is currently in vblank.
    (is_vblank, with_is_vblank, bool, 0),
    /// Whether the graphics chip is currently in hblank.
    (is_hblank, with_is_hblank, bool, 1),
    /// Whether the graphics chip is processing the scanline matching the `vcount_scanline`
    /// setting.
    (is_vcount, with_is_vcount, bool, 2),
    /// Whether to send an IRQ when vblank is reached.
    (vblank_irq_enabled, with_vblank_irq_enabled, bool, 3),
    /// Whether to send an IRQ when hblank is reached.
    (hblank_irq_enabled, with_hblank_irq_enabled, bool, 4),
    /// Whether to send an IRQ when the scanline matching the `vcount_scanline` setting is reached.
    (vcount_irq_enabled, with_vcount_irq_enabled, bool, 5),
    /// Determines the vcount scanline in use for `is_vcount` and `vcount_irq_enabled`.
    (vcount_scanline, with_vcount_scanline, u32, 8..=15),
);

/// Used to control the behavior of a background layer.
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug, Hash, Default)]
#[repr(transparent)]
pub struct BgCnt(u16);
#[rustfmt::skip]
packed_struct_fields!(
    BgCnt, u16,

    // Not directly documented here, as this API will only be used internally.
    (bg_priority, with_bg_priority, u32, 0..=1),
    (char_base, with_char_base, usize, 2..=3),
    (enable_mosaic, with_mosaic, bool, 6),
    (enable_256_color, with_enable_256_color, bool, 7),
    (tile_map_base, with_tile_map_base, usize, 8..=12),
    (screen_size, with_screen_size, u32, 14..=15),
);

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug, Hash, Default)]
#[repr(transparent)]
pub struct GbaFrac32(u32);
#[rustfmt::skip]
packed_struct_fields!(
    GbaFrac32, u32,

    // Not directly documented here, as this API will only be used internally.
    (frac, with_frac, u32, 0..=7),
    (int, with_int, u32, 8..=26),
    (sign, with_sign, bool, 27),
);

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug, Hash, Default)]
#[repr(transparent)]
pub struct GbaFrac16(u16);
#[rustfmt::skip]
packed_struct_fields!(
    GbaFrac16, u16,

    // Not directly documented here, as this API will only be used internally.
    (frac, with_frac, u32, 0..=7),
    (int, with_int, u32, 8..=14),
    (sign, with_sign, bool, 15),
);

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug, Hash, Default)]
#[repr(transparent)]
pub struct WinBound(u16);
#[rustfmt::skip]
packed_struct_fields!(
    WinBound, u16,

    // Not directly documented here, as this API will only be used internally.
    (max, with_max, u32, 0..=7),
    (min, with_min, u32, 8..=15),
);

#[derive(EnumSetType, Debug)]
pub enum WinTarget {
    Bg0 = 0,
    Bg1 = 1,
    Bg2 = 2,
    Bg3 = 3,
    Obj = 4,
    ColorEffect = 5,
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug, Hash, Default)]
#[repr(transparent)]
pub struct WinCnt(u16);
#[rustfmt::skip]
packed_struct_fields!(
    WinCnt, u16,

    (cnt_a, with_cnt_a, (@enumset WinTarget), 0..=5),
    (cnt_b, with_cnt_b, (@enumset WinTarget), 8..=13),
);

/// Used to control the size of the mosaic renderer.
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug, Hash, Default)]
#[repr(transparent)]
pub struct Mosaic(u16);
#[rustfmt::skip]
packed_struct_fields!(
    Mosaic, u16,

    /// Sets the horizontal size of the BG0-3 mosaic.
    (bg_mosaic_x, with_bg_mosaic_x, u32, 0..=3),
    /// Sets the vertical size of the BG0-3 mosaic.
    (bg_mosaic_y, with_bg_mosaic_y, u32, 4..=7),
    /// Sets the horizontal size of the OBJ mosaic.
    (obj_mosaic_x, with_obj_mosaic_x, u32, 8..=11),
    /// Sets the vertical size of the OBJ mosaic.
    (obj_mosaic_y, with_obj_mosaic_y, u32, 12..=15),
);

/// Represents a layer that may be blended.
#[derive(EnumSetType, Debug)]
pub enum BlendTarget {
    Bg0 = 0,
    Bg1 = 1,
    Bg2 = 2,
    Bg3 = 3,
    Obj = 4,
    Backdrop = 5,
}

/// Represents one of the blending modes of the GBA.
#[derive(IntoPrimitive, TryFromPrimitive)]
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug, Hash)]
#[repr(u16)]
pub enum BlendingMode {
    /// No blending is applied.
    None,
    /// The two targets are mixed.
    Alpha,
    /// The first target is lightened by the second.
    Lighten,
    /// The first target is darkened by the second.
    Darken,
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug, Hash, Default)]
#[repr(transparent)]
pub struct BldCnt(u16);
#[rustfmt::skip]
packed_struct_fields!(
    BldCnt, u16,

    // Not directly documented here, as this API will only be used internally.
    (target_a, with_target_a, (@enumset BlendTarget), 0..=5),
    (mode, with_mode, BlendingMode, 6..=7),
    (target_b, with_target_b, (@enumset BlendTarget), 8..=13),
);

/// Represents a tile in a background layer.
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug, Hash, Default)]
#[repr(transparent)]
pub struct VramTile(u16);
#[rustfmt::skip]
packed_struct_fields!(
    VramTile, u16,

    /// The ID of the character to render.
    ///
    /// This must be a number between 0-1023.
    (char, with_char, u16, 0..=9),
    /// Whether to flip the tile horizontally.
    (h_flip, with_h_flip, bool, 10),
    /// Whether to flip the tile vertically.
    (v_flip, with_v_flip, bool, 11),
    /// The ID of the palette to use.
    ///
    /// This must be a number between 0-15.
    (palette, with_palette, u8, 12..=15),
);

/// Controls which special effects an object is rendered using.
#[derive(IntoPrimitive, TryFromPrimitive)]
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug, Hash)]
#[repr(u16)]
pub enum ObjMode {
    /// Render this object with no special effects.
    Normal,
    /// Applies the alpha blending settings to this object.
    SemiTransparent,
    /// Apply the OBJ Window to the object.
    ObjWindow,
}

#[derive(IntoPrimitive, TryFromPrimitive)]
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug, Hash)]
#[repr(u16)]
pub enum ObjShape {
    Square,
    Horizontal,
    Vertical,
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug, Hash, Default)]
#[repr(transparent)]
pub struct ObjAttr0(u16);
#[rustfmt::skip]
packed_struct_fields!(
    ObjAttr0, u16,

    (y_coordinate, with_y_coordinate, u32, 0..=7),
    (rotation_enabled, with_rotation_enabled, bool, 8),
    (double_size, with_double_size, bool, 9),
    (disabled, with_disabled, bool, 9),
    (obj_mode, with_obj_mode, ObjMode, 10..=11),
    (mosiac_enabled, with_mosiac_enabled, bool, 12),
    (use_256_color, with_use_256_color, bool, 13),

);

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug, Hash, Default)]
#[repr(transparent)]
pub struct ObjAttr1(u16);
#[rustfmt::skip]
packed_struct_fields!(
    ObjAttr1, u16,

    (x_coordinate, with_x_coordinate, u32, 0..=7),
    (rotation_id, with_rotation_id, usize, 9..=13),
    (h_flip, with_h_flip, bool, 12),
    (v_flip, with_v_flip, bool, 13),
    (obj_shape, with_obj_shape, u8, 14..=15),
);

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug, Hash, Default)]
#[repr(transparent)]
pub struct ObjAttr2(u16);
#[rustfmt::skip]
packed_struct_fields!(
    ObjAttr2, u16,

    (x_coordinate, with_x_coordinate, u32, 0..=7),
    (rotation_id, with_rotation_id, usize, 9..=13),
    (h_flip, with_h_flip, bool, 12),
    (v_flip, with_v_flip, bool, 13),
    (obj_shape, with_obj_shape, u8, 14..=15),
);
