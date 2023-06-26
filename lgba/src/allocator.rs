use core::alloc::{GlobalAlloc, Layout};

struct NoAlloc;
unsafe impl GlobalAlloc for NoAlloc {
    #[track_caller]
    unsafe fn alloc(&self, _: Layout) -> *mut u8 {
        crate::irq::suppress(|| crate::panic_handler::static_panic("No allocator available."))
    }
    #[track_caller]
    unsafe fn dealloc(&self, _: *mut u8, _: Layout) {
        crate::irq::suppress(|| crate::panic_handler::static_panic("No allocator available."))
    }
}

#[global_allocator]
static ALLOC: NoAlloc = NoAlloc;
