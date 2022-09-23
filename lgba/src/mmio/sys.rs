use crate::mmio::prelude::*;
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
    (transfer_u32, with_transfer_u32, bool, 11),
    (game_pak_drq, with_game_pak_drq, bool, 12),
    (send_irq, with_send_irq, bool, 14),
    (enabled, with_enabled, bool, 15),
);
