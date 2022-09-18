#![feature(start, alloc_error_handler)]
#![no_std]

extern crate lgba;

use core::{
    alloc::{GlobalAlloc, Layout},
    panic::PanicInfo,
};
use lgba::lcd::{DispCnt, DispMode, DISPCNT};

#[start]
fn main(_: isize, _: *const *const u8) -> isize {
    unsafe {
        let mut i = 0;
        let mut rng = 1u32;
        loop {
            DISPCNT.write(
                DispCnt::default()
                    .with_mode(DispMode::Mode3)
                    .with_display_bg2(true),
            );
            for _ in 0..100 {
                (0x06000000 as *mut u16)
                    .offset(i)
                    .write_volatile((rng >> 16) as u16);
                i += 1;
                if i > 0xA000 {
                    i = 0
                }
                rng = rng.wrapping_mul(2891336453).wrapping_add(1234561);
            }
        }
    }
}

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    loop {}
}

#[alloc_error_handler]
fn alloc_error(_: Layout) -> ! {
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
