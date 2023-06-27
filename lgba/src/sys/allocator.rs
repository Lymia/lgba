use crate::sync::{Mutex, Static};
use core::{
    alloc::{GlobalAlloc, Layout},
    ops::Range,
    ptr,
};
use linked_list_allocator::Heap;

static ALLOC_IWRAM: Mutex<Heap> = Mutex::new(Heap::empty());
static ALLOC_EWRAM: Mutex<Heap> = Mutex::new(Heap::empty());

static IWRAM_SIZE: Static<usize> = Static::new(0);
static EWRAM_SIZE: Static<usize> = Static::new(0);

fn map_layout(l: Layout) -> Layout {
    // We align everything to 4 bytes, so we can do some questionable things
    // safely. (e.g. use `Vec<u8>` as if it was 4-byte aligned.)
    if l.align() < 4 {
        Layout::from_size_align(l.size(), 4).unwrap()
    } else {
        l
    }
}

struct TrackingAlloc {
    size: Static<usize>,
}
unsafe impl GlobalAlloc for TrackingAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let layout = map_layout(layout);
        let (mut a0, mut a1) = if layout.size() <= 64 {
            (ALLOC_IWRAM.lock(), ALLOC_EWRAM.lock())
        } else {
            (ALLOC_EWRAM.lock(), ALLOC_IWRAM.lock())
        };
        let result = a0
            .allocate_first_fit(layout)
            .or_else(|_| a1.allocate_first_fit(layout));
        match result {
            Ok(v) => {
                self.size.write(self.size.read() + layout.size());
                v.as_ptr()
            }
            Err(_) => ptr::null_mut(),
        }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let layout = map_layout(layout);
        if !ptr.is_null() {
            self.size.write(self.size.read() - layout.size());
            let heap = if (ptr as usize) < 0x2010000 {
                &ALLOC_IWRAM
            } else {
                &ALLOC_EWRAM
            };
            heap.lock()
                .deallocate(ptr::NonNull::new_unchecked(ptr), layout);
        }
    }
}

#[global_allocator]
static ALLOC: TrackingAlloc = TrackingAlloc { size: Static::new(0) };

extern "C" {
    static __heap_start: u8;
}

#[inline(always)]
pub fn heap_capacity() -> usize {
    0
}

#[inline(always)]
pub fn heap_used() -> usize {
    0
}

fn align_alloc(range: Range<usize>) -> Range<usize> {
    ((range.start + 63) & !63)..range.end
}
pub unsafe fn init_rust_alloc() {
    let iwram = align_alloc(crate::asm::iwram_alloc_range());
    let ewram = align_alloc(crate::asm::ewram_alloc_range());

    crate::println!("iwram range: {iwram:x?}");
    crate::println!("ewram range: {ewram:x?}");

    ALLOC_IWRAM.lock().init(iwram.start as *mut u8, iwram.len());
    ALLOC_EWRAM.lock().init(ewram.start as *mut u8, ewram.len());

    IWRAM_SIZE.write(iwram.len());
    EWRAM_SIZE.write(ewram.len());
}
