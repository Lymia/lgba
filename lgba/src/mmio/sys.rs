use crate::mmio::prelude::*;
use enumset::EnumSetType;
use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(IntoPrimitive, TryFromPrimitive)]
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug, Hash)]
#[repr(u16)]
pub enum DmaAddrCnt {
    Increment,
    Decrement,
    Fixed,
}

#[derive(IntoPrimitive, TryFromPrimitive)]
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug, Hash)]
#[repr(u16)]
pub enum DmaStartTiming {
    Immediately,
    VBlank,
    HBlank,
    Special,
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug, Hash, Default)]
#[repr(transparent)]
pub struct DmaCnt(u16);
#[rustfmt::skip]
packed_struct_fields!(
    DmaCnt, u16,

    (dst_ctl, with_dst_ctl, DmaAddrCnt, 5..=6),
    (src_ctl, with_src_ctl, DmaAddrCnt, 7..=8),
    (repeat, with_repeat, bool, 9),
    (transfer_u32, with_transfer_u32, bool, 10),
    (game_pak_drq, with_game_pak_drq, bool, 11),
    (start_timing, with_start_timing, DmaStartTiming, 12..=13),
    (send_irq, with_send_irq, bool, 14),
    (enabled, with_enabled, bool, 15),
);

/// Represents the various kinds of interrupts that can be raised on the GBA.
#[derive(EnumSetType, Debug)]
#[enumset(repr = "u16")]
pub enum Interrupt {
    VBlank = 0,
    HBlank = 1,
    VCounter = 2,
    Timer0 = 3,
    Timer1 = 4,
    Timer2 = 5,
    Timer3 = 6,
    Serial = 7,
    Dma0 = 8,
    Dma1 = 9,
    Dma2 = 10,
    Dma3 = 11,
    Keypad = 12,
    GamePak = 13,
}

#[derive(IntoPrimitive, TryFromPrimitive)]
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug, Hash)]
#[repr(u16)]
pub enum TimerScale {
    NoDiv = 0,
    Div64 = 1,
    Div256 = 2,
    Div1024 = 3,
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug, Hash, Default)]
#[repr(transparent)]
pub struct TimerCnt(u16);
#[rustfmt::skip]
packed_struct_fields!(
    TimerCnt, u16,

    (scale, with_scale, TimerScale, 0..=1),
    (cascade, with_cascade, bool, 2),
    (enable_irq, with_enable_irq, bool, 6),
    (enabled, with_enabled, bool, 7),
);

/// An enum representing all buttons available on the GBA.
#[derive(EnumSetType, Debug, Ord, PartialOrd, Hash)]
#[enumset(repr = "u16")]
pub enum Button {
    /// The A button.
    A = 0,
    /// The B button.
    B = 1,
    /// The Select button.
    Select = 2,
    /// The Start button.
    Start = 3,
    /// A right press on the direction pad.
    Right = 4,
    /// A left press on the direction pad.
    Left = 5,
    /// An up press on the direction pad.
    Up = 6,
    /// A down press on the direction pad.
    Down = 7,
    /// The R trigger.
    R = 8,
    /// The L trigger.
    L = 9,
}

#[derive(IntoPrimitive, TryFromPrimitive)]
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug, Hash)]
#[repr(u16)]
pub enum ButtonCondition {
    LogicalOr = 0,
    LogicalAnd = 1,
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug, Hash, Default)]
#[repr(transparent)]
pub struct KeyCnt(u16);
#[rustfmt::skip]
packed_struct_fields!(
    KeyCnt, u16,

    (keys, with_keys, (@enumset Button), 0..=9),
    (enable_irq, with_enable_irq, bool, 14),
    (condition, with_condition, ButtonCondition, 15..=15),
);

#[derive(IntoPrimitive, TryFromPrimitive)]
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug, Hash)]
#[repr(u16)]
pub enum WaitState {
    Wait4,
    Wait3,
    Wait2,
    Wait8,
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug, Hash, Default)]
#[repr(transparent)]
pub struct WaitCnt(u16);
#[rustfmt::skip]
packed_struct_fields!(
    WaitCnt, u16,

    (sram_wait, with_sram_wait, WaitState, 0..=1),
    (rom0_wait, with_rom0_wait, WaitState, 2..=3),
    (rom0_fast_seq, with_rom0_fast_seq, bool, 4),
    (rom1_wait, with_rom1_wait, WaitState, 5..=6),
    (rom1_fast_seq, with_rom1_fast_seq, bool, 7),
    (rom2_wait, with_rom2_wait, WaitState, 8..=9),
    (rom2_fast_seq, with_rom2_fast_seq, bool, 10),
    // TODO: PHI Terminal Output
    (enable_rom_prefetch, with_enable_rom_prefetch, bool, 14),
    (is_cgb, with_is_cgb, bool, 15),
);
