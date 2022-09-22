//! Various functions and helper types for basic GBA system functions.

use core::arch::asm;

mod asm_export {
    #[no_mangle]
    pub unsafe extern "C" fn __lgba_init_rust() {}

    #[no_mangle]
    pub unsafe extern "C" fn __lgba_main_func_returned() -> ! {
        crate::panic_handler::static_panic("Internal error: Main function returned?")
    }
}

extern "C" {
    fn __lgba_abort() -> !;
}

/// Resets the GBA.
pub fn reset() -> ! {
    unsafe {
        asm!("swi #0x00");
    }
    abort()
}

/// Aborts the GBA, and all its functions.
///
/// This sets the GBA into a state where no functions (such as DMA or interrupts) are running, and
/// no further code will be run. This will also disable sound to prevent this state from hurting
/// the player's ears.
pub fn abort() -> ! {
    unsafe { __lgba_abort() }
}
