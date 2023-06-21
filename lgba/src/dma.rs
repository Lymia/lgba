//! A module allowing use of the GBA's DMA hardware.

use crate::{
    mmio::{
        reg::*,
        sys::{DmaAddrCnt, DmaCnt},
    },
    sync::{RawMutex, RawMutexGuard},
};
use core::{
    arch::asm,
    ffi::c_void,
    sync::atomic::{compiler_fence, Ordering},
};

static DMA_LOCK: [RawMutex; 4] =
    [RawMutex::new(), RawMutex::new(), RawMutex::new(), RawMutex::new()];

/// Used to specify a particular DMA channel ID.
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug, Hash)]
#[repr(u8)]
pub enum DmaChannelId {
    Dma0,
    Dma1,
    Dma2,
    Dma3,
}
impl DmaChannelId {
    /// Returns whether the source address for DMA on this channel must be internal to the GBA.
    pub fn is_source_internal_only(&self) -> bool {
        *self == DmaChannelId::Dma0
    }

    /// Returns whether the target address for DMA on this channel must be internal to the GBA.
    pub fn is_target_internal_only(&self) -> bool {
        *self <= DmaChannelId::Dma2
    }

    /// Creates a new DMA channel for this ID.
    #[track_caller]
    pub fn create(&self) -> DmaChannel {
        DmaChannel {
            channel: *self,
            irq_notify: false,
            _lock: DMA_LOCK[*self as usize]
                .try_lock()
                .unwrap_or_else(|| dma_channel_in_use()),
        }
    }
}

#[inline]
#[track_caller]
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
    DMA_SAD.index(ch as usize).write(src);
    DMA_DAD.index(ch as usize).write(dst);
    DMA_CNT_L.index(ch as usize).write(word_count);
    DMA_CNT_H.index(ch as usize).write(cnt);
    asm!("nop", "nop"); // wait for the DMA to begin.
    crate::sync::memory_write_hint(dst);
}

/// A DMA channel.
#[derive(Debug)]
pub struct DmaChannel {
    channel: DmaChannelId,
    irq_notify: bool,
    _lock: RawMutexGuard<'static>,
}
impl DmaChannel {
    /// Triggers an IRQ whenever this DMA transfer completes successfully.
    pub fn with_irq_notify(mut self) -> Self {
        self.irq_notify = true;
        self
    }

    /// Sets all values of a slice to a given value using DMA.
    ///
    /// `T` must have one of the following memory layouts:
    /// * A size of two bytes, and an alignment of two bytes.
    /// * A size of four bytes and an alignment of four bytes.
    #[inline]
    #[track_caller]
    pub fn set<T: Copy>(&mut self, src: T, dst: &mut [T]) -> &mut Self {
        unsafe {
            self.unsafe_set(src, dst.as_mut_ptr(), dst.len());
        }
        self
    }

    /// Sets all values of a pointer to a given value using DMA.
    ///
    /// Both the start of `src` and `dst` must be aligned to a multiple of `2` bytes. If it is not,
    /// this function will panic.
    #[inline]
    #[track_caller]
    pub unsafe fn unsafe_set<T: Copy>(
        &mut self,
        src: T,
        dst: *mut T,
        word_count: usize,
    ) -> &mut Self {
        let is_u32 = if core::mem::size_of::<T>() == 2 && core::mem::align_of::<T>() == 2 {
            false
        } else if core::mem::size_of::<T>() == 4 && core::mem::align_of::<T>() == 4 {
            true
        } else {
            dma_invalid_size()
        };
        if word_count >= 0x10000 {
            dma_too_large()
        }
        let cnt = DmaCnt::default()
            .with_src_ctl(DmaAddrCnt::Fixed)
            .with_send_irq(self.irq_notify)
            .with_transfer_u32(is_u32)
            .with_enabled(true);
        raw_tx(
            self.channel,
            &src as *const T as *const c_void,
            dst as *mut c_void,
            word_count as u16,
            cnt,
        );
        self
    }

    /// Transfers data from one slice into another via DMA.
    ///
    /// Both the start of `src` and `dst` must be aligned to a multiple of `2` bytes. If it is not,
    /// this function will panic.
    #[inline]
    #[track_caller]
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
    #[track_caller]
    pub unsafe fn unsafe_transfer(
        &mut self,
        src: *const c_void,
        dst: *mut c_void,
        byte_count: usize,
    ) -> &mut Self {
        let (is_u32, word_count) = check_align(
            self.channel.is_source_internal_only(),
            self.channel.is_target_internal_only(),
            src,
            dst,
            byte_count,
        );
        let cnt = DmaCnt::default()
            .with_send_irq(self.irq_notify)
            .with_transfer_u32(is_u32)
            .with_enabled(true);
        raw_tx(self.channel, src, dst, word_count, cnt);
        self
    }
}

#[inline(never)]
#[track_caller]
fn dma_size_not_equal() -> ! {
    crate::panic_handler::static_panic("DMA transfer between two lists of unequal size!")
}

#[inline(never)]
#[track_caller]
fn dma_not_aligned() -> ! {
    crate::panic_handler::static_panic("DMA transfer between unaligned lists!")
}

#[inline(never)]
#[track_caller]
fn dma_too_large() -> ! {
    crate::panic_handler::static_panic("DMA transfer is too large!")
}

#[inline(never)]
#[track_caller]
fn dma_invalid_size() -> ! {
    crate::panic_handler::static_panic("Cannot use `set` on objects of this size!")
}

#[inline(never)]
#[track_caller]
fn dma_channel_in_use() -> ! {
    crate::panic_handler::static_panic("DMA channel already in use!")
}

#[inline(never)]
#[track_caller]
fn dma_is_external() -> ! {
    crate::panic_handler::static_panic("DMA channel does not support cartridge addresses!")
}

/// Pauses running DMAs and restores them afterwards.
pub fn pause_dma<R>(func: impl FnOnce() -> R) -> R {
    unsafe {
        let mut dma_cnt = [DmaCnt::default(); 4];
        for i in 0..4 {
            dma_cnt[i] = DMA_CNT_H.index(i).read();
        }
        for i in 0..4 {
            if dma_cnt[i].enabled() {
                DMA_CNT_H.index(i).write(dma_cnt[i].with_enabled(false));
            }
        }

        compiler_fence(Ordering::Acquire);
        let result = func();
        compiler_fence(Ordering::Release);

        for i in 0..4 {
            if dma_cnt[i].enabled() {
                DMA_CNT_H.index(i).write(dma_cnt[i]);
            }
        }

        result
    }
}
