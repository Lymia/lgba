#![no_std]
#![no_main]

#[macro_use]
extern crate lgba;

use core::{
    alloc::{GlobalAlloc, Layout},
    hint::black_box,
};
use lgba::{
    display::{Terminal, TerminalFontFull},
    dma::DmaChannelId,
};

#[inline(never)]
fn test_func() {
    lgba::timer::temp_time(|| {
        for x in 0u64..500 {
            for y in 0u64..500 {
                black_box(black_box(x) * black_box(y));
            }
        }
    });
}

#[lgba::entry]
#[rom(title = "LGBA_TESTROM", code = "LGTR")]
fn rom_entry() -> ! {
    test_func();
    loop {}
}

struct NoAlloc;
unsafe impl GlobalAlloc for NoAlloc {
    unsafe fn alloc(&self, _: Layout) -> *mut u8 {
        todo!()
    }
    unsafe fn dealloc(&self, _: *mut u8, _: Layout) {
        todo!()
    }
}
#[global_allocator]
static ALLOC: NoAlloc = NoAlloc;
