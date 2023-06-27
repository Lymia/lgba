use crate::{mmio::prelude::*, sys};
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
    /// Triggered at the start of vertical blank.
    VBlank = 0,
    /// Triggered at the start of horizontal blank.
    HBlank = 1,
    /// Triggered when the scanline equals a user configured value.
    ///
    /// The scanline this interrupt is triggered during is configured using the
    /// [`display::set_counter_scanline`] function.
    ///
    /// [`display::set_counter_scanline`]: crate::display::set_counter_scanline
    VCounter = 2,
    /// Triggered by the first timer.
    ///
    /// See the [`timer`] module for more information. This is the timer that corresponds to the
    /// [`TimerId::Timer0`] variant.
    ///
    /// [`timer`]: crate::timer
    /// [`TimerId::Timer0`]: crate::timer::TimerId::Timer0
    Timer0 = 3,
    /// Triggered by the second timer.
    ///
    /// See the [`timer`] module for more information. This is the timer that corresponds to the
    /// [`TimerId::Timer1`] variant.
    ///
    /// [`timer`]: crate::timer
    /// [`TimerId::Timer1`]: crate::timer::TimerId::Timer1
    Timer1 = 4,
    /// Triggered by the third timer.
    ///
    /// See the [`timer`] module for more information. This is the timer that corresponds to the
    /// [`TimerId::Timer2`] variant.
    ///
    /// [`timer`]: crate::timer
    /// [`TimerId::Timer2`]: crate::timer::TimerId::Timer2
    Timer2 = 5,
    /// Triggered by the fourth timer.
    ///
    /// See the [`timer`] module for more information. This is the timer that corresponds to the
    /// [`TimerId::Timer3`] variant.
    ///
    /// [`timer`]: crate::timer
    /// [`TimerId::Timer3`]: crate::timer::TimerId::Timer3
    Timer3 = 6,
    /// Triggered by serial communication.
    Serial = 7,
    /// Triggered by the first DMA channel.
    ///
    /// See the [`dma`] module for more information. This is the channel that corresponds to the
    /// [`DmaChannelId::Dma0`] variant.
    ///
    /// [`dma`]: crate::dma
    /// [`DmaChannelId::Dma0`]: crate::dma::DmaChannelId::Dma0
    Dma0 = 8,
    /// Triggered by the second DMA channel.
    ///
    /// See the [`dma`] module for more information. This is the channel that corresponds to the
    /// [`DmaChannelId::Dma1`] variant.
    ///
    /// [`dma`]: crate::dma
    /// [`DmaChannelId::Dma1`]: crate::dma::DmaChannelId::Dma1
    Dma1 = 9,
    /// Triggered by the third DMA channel.
    ///
    /// See the [`dma`] module for more information. This is the channel that corresponds to the
    /// [`DmaChannelId::Dma2`] variant.
    ///
    /// [`dma`]: crate::dma
    /// [`DmaChannelId::Dma2`]: crate::dma::DmaChannelId::Dma2
    Dma2 = 10,
    /// Triggered by the fourth DMA channel.
    ///
    /// See the [`dma`] module for more information. This is the channel that corresponds to the
    /// [`DmaChannelId::Dma3`] variant.
    ///
    /// [`dma`]: crate::dma
    /// [`DmaChannelId::Dma3`]: crate::dma::DmaChannelId::Dma3
    Dma3 = 11,
    /// Triggered by specific keypad input.
    ///
    /// See [`sys::set_keypad_irq_combo`] and [`sys::set_keypad_irq_keys`] for more information.
    Keypad = 12,
    /// Triggered externally by optional hardware in the Game Pak.
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
