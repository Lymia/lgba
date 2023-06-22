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
