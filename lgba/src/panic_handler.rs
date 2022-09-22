use crate::{debug::DebugLevel, eprintln};
use core::{alloc::Layout, panic::PanicInfo};

#[panic_handler]
fn handle_panic(error: &PanicInfo) -> ! {
    eprintln!("ROM panicked: {}", error);
    crate::sys::abort()
}

#[alloc_error_handler]
fn handle_alloc_error(layout: Layout) -> ! {
    eprintln!("Could not allocate memory: {:?}", layout);
    crate::sys::abort()
}

pub fn static_panic(msg: &str) -> ! {
    crate::debug::debug_print(DebugLevel::Error, msg);
    crate::sys::abort()
}
