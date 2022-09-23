//! A module allowing use of the GBA's DMA hardware.

use crate::{
    mmio::{reg::*, sys::DmaCnt},
    sync::{RawMutex, RawMutexGuard},
};
use core::{
    arch::asm,
    ffi::c_void,
    sync::atomic::{compiler_fence, Ordering},
};

static DMA0_LOCK: RawMutex = RawMutex::new();
static DMA1_LOCK: RawMutex = RawMutex::new();
static DMA2_LOCK: RawMutex = RawMutex::new();
static DMA3_LOCK: RawMutex = RawMutex::new();

const DMA_SAD: [Register<*const c_void, UnsafeReg>; 4] = [DMA0SAD, DMA1SAD, DMA2SAD, DMA3SAD];
const DMA_DAD: [Register<*mut c_void, UnsafeReg>; 4] = [DMA0DAD, DMA1DAD, DMA2DAD, DMA3DAD];
const DMA_CNT_L: [Register<u16, UnsafeReg>; 4] = [DMA0CNT_L, DMA1CNT_L, DMA2CNT_L, DMA3CNT_L];
const DMA_CNT_H: [Register<DmaCnt, UnsafeReg>; 4] = [DMA0CNT_H, DMA1CNT_H, DMA2CNT_H, DMA3CNT_H];

#[derive(Copy, Clone, Debug)]
#[repr(u8)]
enum DmaChannelId {
    Dma0,
    Dma1,
    Dma2,
    Dma3,
}

#[inline]
fn check_align(
    src_internal: bool,
    dst_internal: bool,
    src: *const c_void,
    dst: *mut c_void,
    byte_count: usize,
) -> (bool, u16) {
    if src_internal && (src as usize) >= 0x8000000 {
        dma_is_external();
    }
    if dst_internal && (dst as usize) >= 0x8000000 {
        dma_is_external();
    }
    if (src as usize) % 2 != 0 || (dst as usize) % 2 != 0 || byte_count % 2 != 0 {
        dma_not_aligned();
    }

    let is_u32 = (src as usize) % 4 == 0 || (dst as usize) % 4 == 0 || byte_count % 4 == 0;
    let word_shift = is_u32 as usize + 1;
    let word_count = byte_count >> word_shift;
    if word_count >= 0x10000 {
        dma_too_large()
    }
    (is_u32, word_count as u16)
}

#[inline]
unsafe fn raw_tx(
    ch: DmaChannelId,
    src: *const c_void,
    dst: *mut c_void,
    word_count: u16,
    cnt: DmaCnt,
) {
    crate::sync::memory_read_hint(src);
    DMA_SAD[ch as usize].write(src);
    DMA_DAD[ch as usize].write(dst);
    DMA_CNT_L[ch as usize].write(word_count);
    DMA_CNT_H[ch as usize].write(cnt);
    asm!("nop", "nop"); // wait for the DMA to begin.
    crate::sync::memory_write_hint(dst);
}

#[derive(Debug)]
pub struct DmaChannel {
    channel: DmaChannelId,
    src_internal: bool,
    dst_internal: bool,
    irq_notify: bool,
    mutex: RawMutexGuard<'static>,
}
impl DmaChannel {
    /// Triggers an IRQ whenever this DMA transfer completes successfully.
    pub fn with_irq_notify(mut self) -> Self {
        self.irq_notify = true;
        self
    }

    /// Transfers data from one slice into another via DMA.
    ///
    /// Both the start of `src` and `dst` must be aligned to a multiple of `2` bytes. If it is not,
    /// this function will panic.
    #[inline]
    pub fn transfer<T: Copy>(&mut self, src: &[T], dst: &mut [T]) -> &mut Self {
        if src.len() != dst.len() {
            dma_size_not_equal();
        }
        unsafe {
            self.unsafe_transfer(
                src.as_ptr() as *const _,
                dst.as_mut_ptr() as *mut _,
                src.len() * core::mem::size_of::<T>(),
            );
        }
        self
    }

    /// Transfers data from one pointer to another via DMA.
    ///
    /// Both the start of `src` and `dst` must be aligned to a multiple of `2` bytes. If it is not,
    /// this function will panic.
    #[inline]
    pub unsafe fn unsafe_transfer(
        &mut self,
        src: *const c_void,
        dst: *mut c_void,
        byte_count: usize,
    ) -> &mut Self {
        let (is_u32, word_count) =
            check_align(self.src_internal, self.dst_internal, src, dst, byte_count);
        let cnt = DmaCnt::default()
            .with_send_irq(self.irq_notify)
            .with_transfer_u32(is_u32)
            .with_enabled(true);
        raw_tx(self.channel, src, dst, word_count, cnt);
        self
    }
}

#[inline(never)]
fn dma_size_not_equal() -> ! {
    crate::panic_handler::static_panic("DMA transfer between two lists of unequal size!")
}

#[inline(never)]
fn dma_not_aligned() -> ! {
    crate::panic_handler::static_panic("DMA transfer between unaligned lists!")
}

#[inline(never)]
fn dma_too_large() -> ! {
    crate::panic_handler::static_panic("DMA transfer is too large!")
}

#[inline(never)]
fn dma_channel_in_use() -> ! {
    crate::panic_handler::static_panic("DMA channel already in use!")
}

#[inline(never)]
fn dma_is_external() -> ! {
    crate::panic_handler::static_panic("DMA channel does not support cartridge addresses!")
}

pub fn dma0() -> DmaChannel {
    DmaChannel {
        channel: DmaChannelId::Dma0,
        src_internal: true,
        dst_internal: true,
        irq_notify: false,
        mutex: DMA0_LOCK.try_lock().unwrap_or_else(|| dma_channel_in_use()),
    }
}
pub fn dma1() -> DmaChannel {
    DmaChannel {
        channel: DmaChannelId::Dma1,
        src_internal: true,
        dst_internal: true,
        irq_notify: false,
        mutex: DMA1_LOCK.try_lock().unwrap_or_else(|| dma_channel_in_use()),
    }
}
pub fn dma2() -> DmaChannel {
    DmaChannel {
        channel: DmaChannelId::Dma2,
        src_internal: true,
        dst_internal: true,
        irq_notify: false,
        mutex: DMA2_LOCK.try_lock().unwrap_or_else(|| dma_channel_in_use()),
    }
}
pub fn dma3() -> DmaChannel {
    DmaChannel {
        channel: DmaChannelId::Dma3,
        src_internal: true,
        dst_internal: true,
        irq_notify: false,
        mutex: DMA3_LOCK.try_lock().unwrap_or_else(|| dma_channel_in_use()),
    }
}

/// Pauses running DMAs and restores them afterwards.
pub fn pause_dma<R>(func: impl FnOnce() -> R) -> R {
    unsafe {
        let dma0_cnt = DMA0CNT_H.read();
        let dma1_cnt = DMA1CNT_H.read();
        let dma2_cnt = DMA2CNT_H.read();
        let dma3_cnt = DMA3CNT_H.read();

        compiler_fence(Ordering::Acquire);
        let result = func();
        compiler_fence(Ordering::Release);

        DMA0CNT_H.write(dma0_cnt);
        DMA1CNT_H.write(dma1_cnt);
        DMA2CNT_H.write(dma2_cnt);
        DMA3CNT_H.write(dma3_cnt);

        result
    }
}
