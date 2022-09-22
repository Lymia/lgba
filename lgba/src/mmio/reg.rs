use crate::mmio::{display::*, sys::DmaCnt};
use core::{ffi::c_void, marker::PhantomData};
use enumset::EnumSet;

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub enum SafeReg {}
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub enum UnsafeReg {}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub struct Register<T: Copy, S = SafeReg>(*mut T, PhantomData<S>);
impl<T: Copy, S> Register<T, S> {
    pub const unsafe fn new(offset: usize) -> Self {
        Register(offset as *mut T, PhantomData)
    }
    pub fn as_ptr(&self) -> *mut T {
        self.0
    }
}
impl<T: Copy> Register<T, SafeReg> {
    pub fn write(&self, t: T) {
        unsafe { self.0.write_volatile(t) }
    }
    pub fn read(&self) -> T {
        unsafe { self.0.read_volatile() }
    }
}
impl<T: Copy> Register<T, UnsafeReg> {
    pub unsafe fn assert_safe(&self) -> Register<T, SafeReg> {
        Register(self.0, PhantomData)
    }
    pub unsafe fn write(&self, t: T) {
        self.0.write_volatile(t)
    }
    pub unsafe fn read(&self) -> T {
        self.0.read_volatile()
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub struct RegArray<T: Copy, const COUNT: usize, S = SafeReg>(*mut T, PhantomData<S>);
impl<T: Copy, const COUNT: usize, S> RegArray<T, COUNT, S> {
    pub const unsafe fn new(offset: usize) -> Self {
        RegArray(offset as *mut T, PhantomData)
    }
    pub fn index(&self, offset: usize) -> Register<T, S> {
        if offset >= COUNT {
            index_out_of_bounds()
        } else {
            unsafe { Register(self.0.offset(offset as isize), PhantomData) }
        }
    }
    pub fn as_ptr(&self) -> *mut T {
        self.0
    }
}

#[inline(never)]
fn index_out_of_bounds() -> ! {
    crate::panic_handler::static_panic("indexed register out of bounds!")
}

//
// LCD Control Registers
//
pub const DISPCNT: Register<DispCnt> = unsafe { Register::new(0x4000000) };
pub const DISPSTAT: Register<DispStat> = unsafe { Register::new(0x4000004) };
pub const VCOUNT: Register<u16> = unsafe { Register::new(0x4000006) };
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
pub const WIN0H: Register<WinBound> = unsafe { Register::new(0x4000040) };
pub const WIN1H: Register<WinBound> = unsafe { Register::new(0x4000042) };
pub const WIN0V: Register<WinBound> = unsafe { Register::new(0x4000044) };
pub const WIN1V: Register<WinBound> = unsafe { Register::new(0x4000046) };
pub const WININ: Register<WinCnt> = unsafe { Register::new(0x4000048) };
pub const WINOUT: Register<WinCnt> = unsafe { Register::new(0x400004A) };
pub const MOSAIC: Register<Mosaic> = unsafe { Register::new(0x400004C) };
pub const BLDCNT: Register<BldCnt> = unsafe { Register::new(0x4000050) };
pub const BLDALPHA: Register<[u8; 2]> = unsafe { Register::new(0x4000052) };
pub const BLDY: Register<u16> = unsafe { Register::new(0x4000054) };

//
// VRAM Offsets
//

//
// DMA Transfer Registers
//
pub const DMA0SAD: Register<*mut c_void, UnsafeReg> = unsafe { Register::new(0x40000B0) };
pub const DMA0DAD: Register<*mut c_void, UnsafeReg> = unsafe { Register::new(0x40000B4) };
pub const DMA0CNT_L: Register<u16, UnsafeReg> = unsafe { Register::new(0x40000B8) };
pub const DMA0CNT_H: Register<DmaCnt, UnsafeReg> = unsafe { Register::new(0x40000BA) };
pub const DMA1SAD: Register<*mut c_void, UnsafeReg> = unsafe { Register::new(0x40000BC) };
pub const DMA1DAD: Register<*mut c_void, UnsafeReg> = unsafe { Register::new(0x40000C0) };
pub const DMA1CNT_L: Register<u16, UnsafeReg> = unsafe { Register::new(0x40000C4) };
pub const DMA1CNT_H: Register<DmaCnt, UnsafeReg> = unsafe { Register::new(0x40000C6) };
pub const DMA2SAD: Register<*mut c_void, UnsafeReg> = unsafe { Register::new(0x40000C8) };
pub const DMA2DAD: Register<*mut c_void, UnsafeReg> = unsafe { Register::new(0x40000CC) };
pub const DMA2CNT_L: Register<u16, UnsafeReg> = unsafe { Register::new(0x40000D0) };
pub const DMA2CNT_H: Register<DmaCnt, UnsafeReg> = unsafe { Register::new(0x40000D2) };
pub const DMA3SAD: Register<*mut c_void, UnsafeReg> = unsafe { Register::new(0x40000D4) };
pub const DMA3DAD: Register<*mut c_void, UnsafeReg> = unsafe { Register::new(0x40000D8) };
pub const DMA3CNT_L: Register<u16, UnsafeReg> = unsafe { Register::new(0x40000DC) };
pub const DMA3CNT_H: Register<DmaCnt, UnsafeReg> = unsafe { Register::new(0x40000DE) };
