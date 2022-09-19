use core::{alloc::Layout, panic::PanicInfo};

#[panic_handler]
fn handle_panic(_: &PanicInfo) -> ! {
    // TODO: Panic handler
    crate::sys::abort()
}

#[alloc_error_handler]
fn handle_alloc_error(_: Layout) -> ! {
    // TODO: Allocation error handler
    crate::sys::abort()
}
