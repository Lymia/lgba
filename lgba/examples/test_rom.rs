#![no_std]
#![no_main]

use core::alloc::{GlobalAlloc, Layout};
use lgba::lcd::{DispCnt, DispMode, DISPCNT};

#[inline(never)]
#[lgba::iwram]
fn main_impl() -> ! {
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

#[lgba::entry]
fn rom_entry() -> ! {
    main_impl()
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
