//! Various functions and helper types for basic GBA system functions.

use core::arch::asm;

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
#[inline(always)]
pub fn abort() -> ! {
    crate::asm::abort()
}
