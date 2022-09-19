use crate::mmio::prelude::*;
use num_enum::{IntoPrimitive, TryFromPrimitive};

pub const MGBA_DEBUG_ENABLE: Register<u16> = unsafe { Register::new(0x4fff780) };
pub const MGBA_DEBUG_ENABLE_INPUT: u16 = 0xC0DE;
pub const MGBA_DEBUG_ENABLE_OUTPUT: u16 = 0x1DEA;

pub const MGBA_DEBUG_STR: RegArray<u8, 256> = unsafe { RegArray::new(0x4fff600) };

#[derive(IntoPrimitive, TryFromPrimitive)]
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug, Hash)]
#[repr(u16)]
pub enum MgbaDebugLevel {
    Fatal,
    Error,
    Warn,
    Info,
    Debug,
    Stub,
    GameError,
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug, Hash, Default)]
#[repr(transparent)]
pub struct MgbaDebugFlag(u16);
#[rustfmt::skip]
packed_struct_fields!(
    MgbaDebugFlag, MgbaDebugFlagAccess, u16,

    (level, set_level, with_level, MgbaDebugLevel, 0..=2),
    (send, set_send, with_send, bool, 8),
);
pub const MGBA_DEBUG_FLAG: Register<MgbaDebugFlag> = unsafe { Register::new(0x4fff700) };

pub const NO_CASH_CHAR: Register<u8> = unsafe { Register::new(0x04fffa1c) };
pub const NO_CASH_SIG: RegArray<u8, 16> = unsafe { RegArray::new(0x04fffa00) };
pub const NO_CASH_EXPECTED_SIG: [u8; 7] = *b"no$gba ";
