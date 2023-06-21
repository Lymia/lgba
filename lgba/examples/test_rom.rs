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

fn test_copy<const LEN: usize>() {
    let mut data = [0u8; LEN];
    let mut data2 = [0u8; LEN];
    lgba::timer::temp_time(|| {
        for i in 0..10000 {
            black_box((&mut data, &mut data2));
            black_box(&mut data2).copy_from_slice(black_box(&data));
            black_box((&mut data, &mut data2));
        }
    });
    lgba::timer::temp_time(|| {
        for i in 0..10000 {
            black_box((&mut data, &mut data2));
            unsafe {
                core::ptr::write_bytes(black_box(data.as_mut_ptr()), black_box(30), black_box(data.len()));
            }
            black_box((&mut data, &mut data2));
        }
    });
}

#[inline(never)]
fn test_func() {
    test_copy::<16>();
    test_copy::<60>();
    test_copy::<120>();
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
