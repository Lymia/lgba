#![no_std]
#![no_main]

#[macro_use]
extern crate lgba;

use core::alloc::{GlobalAlloc, Layout};
use lgba::lcd::{DispCnt, DispMode, DISPCNT};

#[inline(never)]
#[lgba::iwram]
fn main_impl() -> ! {
    unsafe {
        let mut i = 0;
        let mut rng = 1u32;
        DISPCNT.write(
            DispCnt::default()
                .with_mode(DispMode::Mode3)
                .with_display_bg2(true),
        );
        loop {
            (0x06000000 as *mut u16)
                .offset(i)
                .write_volatile((rng >> 16) as u16);
            i += 1;
            if i > 0xA000 {
                //i = 0
                lgba::sys::abort();
            }
            rng = rng.wrapping_mul(2891336453).wrapping_add(1234561);
        }
    }
}

#[lgba::entry]
#[rom(title = "LGBA_TESTROM", code = "LGTR")]
fn rom_entry() -> ! {
    log!(Info, "log~");
    println!("println~");
    eprintln!("eprintln~");
    error!("error~");
    warn!("warn~");
    info!("info~");
    debug!("debug~");
    dbg!();
    dbg!(dbg!(3) + 3);
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
