use crate::mmio::{
    display::*,
    sys::{DmaCnt, Interrupt, TimerCnt},
};
use core::{ffi::c_void, marker::PhantomData};
use enumset::EnumSet;

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub enum Safe {}
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub enum Unsafe {}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub struct Register<T: Copy, S = Safe>(*mut T, PhantomData<S>);
impl<T: Copy, S> Register<T, S> {
    pub const unsafe fn new(offset: usize) -> Self {
        Register(offset as *mut T, PhantomData)
    }
    pub fn as_ptr(&self) -> *mut T {
        self.0
    }
}
impl<T: Copy> Register<T, Safe> {
    pub fn write(&self, t: T) {
        unsafe { self.0.write_volatile(t) }
    }
    pub fn read(&self) -> T {
        unsafe { self.0.read_volatile() }
    }
}
impl<T: Copy> Register<T, Unsafe> {
    pub unsafe fn assert_safe(&self) -> Register<T, Safe> {
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
pub struct RegArray<T: Copy, const COUNT: usize, S = Safe>(*mut T, PhantomData<S>);
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

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub struct RegSpanned<T: Copy, const COUNT: usize, const SPAN: usize, S = Safe>(
    *mut T,
    PhantomData<S>,
);
impl<T: Copy, const COUNT: usize, const SPAN: usize, S> RegSpanned<T, COUNT, SPAN, S> {
    pub const unsafe fn new(offset: usize) -> Self {
        RegSpanned(offset as *mut T, PhantomData)
    }
    pub fn index(&self, offset: usize) -> Register<T, S> {
        if offset >= COUNT {
            index_out_of_bounds()
        } else {
            unsafe { Register(self.0.offset((SPAN * offset) as isize), PhantomData) }
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
pub const BG_CNT: RegArray<BgCnt, 4> = unsafe { RegArray::new(0x4000008) };
pub const BG_HOFS: RegSpanned<i16, 4, 2> = unsafe { RegSpanned::new(0x4000010) };
pub const BG_VOFS: RegSpanned<i16, 4, 2> = unsafe { RegSpanned::new(0x4000012) };
pub const BG_X: RegSpanned<GbaFrac32, 2, 4> = unsafe { RegSpanned::new(0x4000028) };
pub const BG_Y: RegSpanned<GbaFrac32, 2, 4> = unsafe { RegSpanned::new(0x400002C) };
pub const BG_PA: RegSpanned<GbaFrac16, 2, 8> = unsafe { RegSpanned::new(0x4000020) };
pub const BG_PB: RegSpanned<GbaFrac16, 2, 8> = unsafe { RegSpanned::new(0x4000022) };
pub const BG_PC: RegSpanned<GbaFrac16, 2, 8> = unsafe { RegSpanned::new(0x4000024) };
pub const BG_PD: RegSpanned<GbaFrac16, 2, 8> = unsafe { RegSpanned::new(0x4000026) };
pub const WIN_H: RegSpanned<WinBound, 2, 2> = unsafe { RegSpanned::new(0x4000040) };
pub const WIN_V: RegSpanned<WinBound, 2, 2> = unsafe { RegSpanned::new(0x4000042) };
pub const WININ: Register<WinCnt> = unsafe { Register::new(0x4000048) };
pub const WINOUT: Register<WinCnt> = unsafe { Register::new(0x400004A) };
pub const MOSAIC: Register<Mosaic> = unsafe { Register::new(0x400004C) };
pub const BLDCNT: Register<BldCnt> = unsafe { Register::new(0x4000050) };
pub const BLDALPHA: Register<[u8; 2]> = unsafe { Register::new(0x4000052) };
pub const BLDY: Register<u16> = unsafe { Register::new(0x4000054) };

//
// VRAM Offsets
//
pub const BG_PALETTE_RAM: RegArray<u16, 256> = unsafe { RegArray::new(0x5000000) };
pub const OBJ_PALETTE_RAM: RegArray<u16, 256> = unsafe { RegArray::new(0x5000200) };
pub const VRAM_BASE: usize = 0x6000000;
pub const VRAM_END: usize = 0x6010000;
pub const VRAM_OBJ_BASE: usize = 0x6010000;
pub const VRAM_OBJ_END: usize = 0x6018000;

//
// DMA Transfer Registers
//
pub const DMA_SAD: RegSpanned<*const c_void, 4, 3, Unsafe> = unsafe { RegSpanned::new(0x40000B0) };
pub const DMA_DAD: RegSpanned<*mut c_void, 4, 3, Unsafe> = unsafe { RegSpanned::new(0x40000B4) };
pub const DMA_CNT_L: RegSpanned<u16, 4, 6, Unsafe> = unsafe { RegSpanned::new(0x40000B8) };
pub const DMA_CNT_H: RegSpanned<DmaCnt, 4, 6, Unsafe> = unsafe { RegSpanned::new(0x40000BA) };

//
// Interrupt-related Registers
//
pub const IME: Register<bool> = unsafe { Register::new(0x4000208) };
pub const IE: Register<EnumSet<Interrupt>> = unsafe { Register::new(0x4000200) };
pub const IF: Register<EnumSet<Interrupt>> = unsafe { Register::new(0x4000202) };

//
// Timer-related Registers
//
pub const TM_CNT_L: RegSpanned<u16, 4, 2> = unsafe { RegSpanned::new(0x4000100) };
pub const TM_CNT_H: RegSpanned<TimerCnt, 4, 2> = unsafe { RegSpanned::new(0x4000102) };
