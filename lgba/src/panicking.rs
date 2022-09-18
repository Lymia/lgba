use core::{alloc::Layout, panic::PanicInfo};

#[panic_handler]
fn handle_panic(_: &PanicInfo) -> ! {
    // TODO: Panic handler
    loop {}
}

#[alloc_error_handler]
fn handle_alloc_error(_: Layout) -> ! {
    // TODO: Allocation error handler
    loop {}
}
