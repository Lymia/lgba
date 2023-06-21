#![no_std]
#![no_main]

#[macro_use]
extern crate lgba;

use core::alloc::{GlobalAlloc, Layout};

#[lgba::entry]
#[rom(title = "LGBA_PNICTST", code = "LGPT")]
fn rom_entry() -> ! {
    panic!("??? this is a long-winded error message that doesn't really mean anything, that exists entirely to test the panic handler screen. yep yep hello!");
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
