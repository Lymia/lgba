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

/// Represents a scaling factor for a timer.
#[derive(IntoPrimitive, TryFromPrimitive)]
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug, Hash)]
#[repr(u16)]
pub enum TimerScale {
    /// The timer will increase once for every clock cycle the GBA executes.
    ///
    /// This happens exactly 2<sup>24</sup> (about 16.7 million) times per second, and exactly
    /// 280896 times per frame.
    ///
    /// Note that these numbers can be used to derive all other durations/frequencies found in the
    /// documentation.
    ///
    /// A duration of one microsecond lasts approximately 16.8 timer cycles, and a duration of
    /// one millisecond lasts approximately 16777 timer cycles.
    NoScaling = 0,
    /// The timer will increase once for every 64 clock cycles the GBA executes.
    ///
    /// This happens approximately 1 million times per second, and exactly 4389 times per frame.
    ///
    /// A duration of one millisecond lasts approximately 262 timer cycles.
    Div64 = 1,
    /// The timer will increase once for every 256 clock cycles the GBA executes.
    ///
    /// This happens exactly 65536 times per second, and exactly 1097.25 times per frame.
    ///
    /// A duration of one millisecond lasts approximately 65.5 timer cycles.
    Div256 = 2,
    /// The timer will increase once for every 1024 clock cycles the GBA executes.
    ///
    /// This happens exactly 16384 times per second, and approximately 274.3 times per frame.
    ///
    /// A duration of one millisecond lasts approximately 16.4 timer cycles.
    Div1024 = 3,
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug, Hash, Default)]
#[repr(transparent)]
pub struct TimerCnt(u16);
#[rustfmt::skip]
packed_struct_fields!(
    TimerCnt, u16,

    (scale, with_scale, TimerScale, 0..=1),
    (count_up, with_count_up, bool, 2),
    (enable_irq, with_enable_irq, bool, 6),
    (enabled, with_enabled, bool, 7),
);
