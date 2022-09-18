use crate::sys::prelude::*;
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

/// Controls the overall state of the LCD display.
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug, Hash, Default)]
#[repr(transparent)]
pub struct DispCnt(u16);
#[rustfmt::skip]
packed_struct_fields!(
    DispCnt, DispCntAccess, u16,

    /// Which display mode is currently in use..
    (mode, set_mode, with_mode, DispMode, 0..=2),
    /// Whether frame 0 or 1 is active for BG modes 4 and 5.
    (active_frame, set_active_frame, with_active_frame, usize, 4..=4),
    /// When set to true, the OAM memory may be accessed during H-Blank.
    (allow_hblank_oam_access, set_allow_hblank_oam_access, with_allow_hblank_oam_access, bool, 5),
    /// Whether to use two-dimensional mapping for OBJ character VRAM.
    (use_2d_obj_vram, set_use_2d_obj_vram, with_use_2d_obj_vram, bool, 6),
    /// When set to `true`, the screen is blanked, and video memory may be accessed far faster.
    (forced_blank, set_forced_blank, with_forced_blank, bool, 7),
    /// Whether to display BG0.
    (display_bg0, set_display_bg0, with_display_bg0, bool, 8),
    /// Whether to display BG1.
    (display_bg1, set_display_bg1, with_display_bg1, bool, 9),
    /// Whether to display BG2.
    (display_bg2, set_display_bg2, with_display_bg2, bool, 10),
    /// Whether to display BG3.
    (display_bg3, set_display_bg3, with_display_bg3, bool, 11),
    /// Whether to display OBJs.
    (display_obj, set_display_obj, with_display_obj, bool, 12),
    /// Whether to use window 0 in rendering.
    (use_window_0, set_use_window_0, with_use_window_0, bool, 13),
    /// Whether to use window 1 in rendering.
    (use_window_1, set_use_window_1, with_use_window_1, bool, 14),
    /// Whether to use the object window in rendering.
    (use_obj_window, set_use_obj_window, with_use_obj_window, bool, 15),
);
pub const DISPCNT: Register<DispCnt, SafeReg> = unsafe { Register::new(0x4000000) };
