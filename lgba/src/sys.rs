//! Various functions and helper types for basic GBA system functions.

use core::arch::asm;

mod init {
    /// Not public API.
    #[doc(hidden)]
    #[macro_export]
    macro_rules! __lgba_macro_export__marker {
        ($name:ident, $str:expr) => {
            #[no_mangle]
            pub static $name: &'static str = concat!($str, "\0");
        };
    }

    #[no_mangle]
    pub static __lgba_exh_lib_cname: &str = env!("CARGO_PKG_NAME");
    #[no_mangle]
    pub static __lgba_exh_lib_cver: &str = env!("CARGO_PKG_VERSION");

    #[no_mangle]
    pub unsafe extern "C" fn __lgba_init_rust() {}

    #[no_mangle]
    pub unsafe extern "C" fn __lgba_main_func_returned() -> ! {
        panic!("Internal error: Main function returned?")
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
