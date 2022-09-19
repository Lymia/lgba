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
    DispCnt, DispCntAccess, u16,

    // Not directly documented here, as this API will only be used internally.
    (mode, set_mode, with_mode, DispMode, 0..=2),
    (active_frame, set_active_frame, with_active_frame, usize, 4..=4),
    (hblank_oam_access, set_hblank_oam_access, with_hblank_oam_access, bool, 5),
    (use_2d_obj_vram, set_use_2d_obj_vram, with_use_2d_obj_vram, bool, 6),
    (forced_blank, set_forced_blank, with_forced_blank, bool, 7),
    (display_bg0, set_display_bg0, with_display_bg0, bool, 8),
    (display_bg1, set_display_bg1, with_display_bg1, bool, 9),
    (display_bg2, set_display_bg2, with_display_bg2, bool, 10),
    (display_bg3, set_display_bg3, with_display_bg3, bool, 11),
    (display_obj, set_display_obj, with_display_obj, bool, 12),
    (use_window_0, set_use_window_0, with_use_window_0, bool, 13),
    (use_window_1, set_use_window_1, with_use_window_1, bool, 14),
    (use_obj_window, set_use_obj_window, with_use_obj_window, bool, 15),
);
pub const DISPCNT: Register<DispCnt> = unsafe { Register::new(0x4000000) };

/// Used to retrieve the status of graphics rendering, and control rendering interrupts.
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug, Hash, Default)]
#[repr(transparent)]
pub struct DispStat(u16);
#[rustfmt::skip]
packed_struct_fields!(
    DispStat, DispStatAccess, u16,

    /// Whether the graphics chip is currently in vblank.
    (is_vblank, -, with_is_vblank, bool, 0),
    /// Whether the graphics chip is currently in hblank.
    (is_hblank, -, with_is_hblank, bool, 1),
    /// Whether the graphics chip is processing the scanline matching the `vcount_scanline`
    /// setting.
    (is_vcount, -, with_is_vcount, bool, 2),
    /// Whether to send an IRQ when vblank is reached.
    (vblank_irq_enabled, set_vblank_irq_enabled, with_vblank_irq_enabled, bool, 3),
    /// Whether to send an IRQ when hblank is reached.
    (hblank_irq_enabled, set_hblank_irq_enabled, with_hblank_irq_enabled, bool, 4),
    /// Whether to send an IRQ when the scanline matching the `vcount_scanline` setting is reached.
    (vcount_irq_enabled, set_vcount_irq_enabled, with_vcount_irq_enabled, bool, 5),
    /// Determines the vcount scanline in use for `is_vcount` and `vcount_irq_enabled`.
    (vcount_scanline, set_vcount_scanline, with_vcount_scanline, u32, 8..=15),
);
pub const DISPSTAT: Register<DispStat> = unsafe { Register::new(0x4000004) };

pub const VCOUNT: Register<u16> = unsafe { Register::new(0x4000006) };

/// Used to control the behavior of a background layer.
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug, Hash, Default)]
#[repr(transparent)]
pub struct BgCnt(u16);
#[rustfmt::skip]
packed_struct_fields!(
    BgCnt, BgCntAccess, u16,

    // Not directly documented here, as this API will only be used internally.
    (bg_priority, set_bg_priority, with_bg_priority, u32, 0..=1),
    (tiles_base, set_tiles_base, with_tiles_base, usize, 2..=3),
    (enable_mosaic, set_enable_mosaic, with_mosaic, bool, 6),
    (enable_256_color, set_enable_256_color, with_enable_256_color, bool, 7),
    (tile_map_base, set_tile_map_base, with_tile_map_base, usize, 8..=12),
    (screen_size, set_screen_size, with_screen_size, u32, 14..=15),
);
pub const BG0CNT: Register<BgCnt> = unsafe { Register::new(0x4000008) };
pub const BG1CNT: Register<BgCnt> = unsafe { Register::new(0x400000A) };
pub const BG2CNT: Register<BgCnt> = unsafe { Register::new(0x400000C) };
pub const BG3CNT: Register<BgCnt> = unsafe { Register::new(0x400000E) };

pub const BG0HOFS: Register<u16> = unsafe { Register::new(0x4000010) };
pub const BG0VOFS: Register<u16> = unsafe { Register::new(0x4000012) };
pub const BG1HOFS: Register<u16> = unsafe { Register::new(0x4000014) };
pub const BG1VOFS: Register<u16> = unsafe { Register::new(0x4000016) };
pub const BG2HOFS: Register<u16> = unsafe { Register::new(0x4000018) };
pub const BG2VOFS: Register<u16> = unsafe { Register::new(0x400001A) };
pub const BG3HOFS: Register<u16> = unsafe { Register::new(0x400001C) };
pub const BG3VOFS: Register<u16> = unsafe { Register::new(0x400001E) };

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug, Hash, Default)]
#[repr(transparent)]
pub struct GbaFrac32(u32);
#[rustfmt::skip]
packed_struct_fields!(
    GbaFrac32, GbaFrac32Access, u32,

    // Not directly documented here, as this API will only be used internally.
    (frac, set_frac, with_frac, u32, 0..=7),
    (int, set_int, with_int, u32, 8..=26),
    (sign, set_sign, with_sign, bool, 27),
);

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug, Hash, Default)]
#[repr(transparent)]
pub struct GbaFrac16(u16);
#[rustfmt::skip]
packed_struct_fields!(
    GbaFrac16, GbaFrac16Access, u16,

    // Not directly documented here, as this API will only be used internally.
    (frac, set_frac, with_frac, u32, 0..=7),
    (int, set_int, with_int, u32, 8..=14),
    (sign, set_sign, with_sign, bool, 15),
);

pub const BG2X: Register<GbaFrac32> = unsafe { Register::new(0x4000028) };
pub const BG2Y: Register<GbaFrac32> = unsafe { Register::new(0x400002C) };
pub const BG2PA: Register<GbaFrac16> = unsafe { Register::new(0x4000020) };
pub const BG2PB: Register<GbaFrac16> = unsafe { Register::new(0x4000022) };
pub const BG2PC: Register<GbaFrac16> = unsafe { Register::new(0x4000024) };
pub const BG2PD: Register<GbaFrac16> = unsafe { Register::new(0x4000026) };

pub const BG3X: Register<GbaFrac32> = unsafe { Register::new(0x4000038) };
pub const BG3Y: Register<GbaFrac32> = unsafe { Register::new(0x400003C) };
pub const BG3PA: Register<GbaFrac16> = unsafe { Register::new(0x4000030) };
pub const BG3PB: Register<GbaFrac16> = unsafe { Register::new(0x4000032) };
pub const BG3PC: Register<GbaFrac16> = unsafe { Register::new(0x4000034) };
pub const BG3PD: Register<GbaFrac16> = unsafe { Register::new(0x4000036) };

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug, Hash, Default)]
#[repr(transparent)]
pub struct WinBound(u16);
#[rustfmt::skip]
packed_struct_fields!(
    WinBound, WinBoundAccess, u16,

    // Not directly documented here, as this API will only be used internally.
    (max, set_max, with_max, u32, 0..=7),
    (min, set_min, with_min, u32, 8..=15),
);

pub const WIN0H: Register<WinBound> = unsafe { Register::new(0x4000040) };
pub const WIN1H: Register<WinBound> = unsafe { Register::new(0x4000042) };
pub const WIN0V: Register<WinBound> = unsafe { Register::new(0x4000044) };
pub const WIN1V: Register<WinBound> = unsafe { Register::new(0x4000046) };

#[derive(EnumSetType, Debug)]
pub enum WinTarget {
    Bg0 = 0,
    Bg1 = 1,
    Bg2 = 2,
    Bg3 = 3,
    Obj = 4,
    ColorEffect = 5,
}
pub const WININ: Register<[EnumSet<WinTarget>; 2]> = unsafe { Register::new(0x4000048) };
pub const WINOUT: Register<[EnumSet<WinTarget>; 2]> = unsafe { Register::new(0x400004A) };

/// Used to control the size of the mosaic renderer.
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug, Hash, Default)]
#[repr(transparent)]
pub struct Mosaic(u16);
#[rustfmt::skip]
packed_struct_fields!(
    Mosaic, MosaicAccess, u16,

    /// Sets the horizontal size of the BG0-3 mosaic.
    (bg_mosaic_x, set_bg_mosaic_x, with_bg_mosaic_x, u32, 0..=3),
    /// Sets the vertical size of the BG0-3 mosaic.
    (bg_mosaic_y, set_bg_mosaic_y, with_bg_mosaic_y, u32, 4..=7),
    /// Sets the horizontal size of the OBJ mosaic.
    (obj_mosaic_x, set_obj_mosaic_x, with_obj_mosaic_x, u32, 8..=11),
    /// Sets the vertical size of the OBJ mosaic.
    (obj_mosaic_y, set_obj_mosaic_y, with_obj_mosaic_y, u32, 12..=15),
);
pub const MOSAIC: Register<Mosaic> = unsafe { Register::new(0x400004C) };

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
    BldCnt, BldCntAccess, u16,

    // Not directly documented here, as this API will only be used internally.
    (target_a, set_target_a, with_target_a, (@enumset BlendTarget), 0..=5),
    (mode, set_mode, with_mode, BlendingMode, 6..=7),
    (target_b, set_target_b, with_target_b, (@enumset BlendTarget), 8..=13),
);
pub const BLDCNT: Register<BldCnt> = unsafe { Register::new(0x4000050) };

pub const BLDALPHA: Register<[u8; 2]> = unsafe { Register::new(0x4000052) };
pub const BLDY: Register<u16> = unsafe { Register::new(0x4000054) };
