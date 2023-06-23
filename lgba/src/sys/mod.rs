//! Various functions and helper types for basic GBA system functions.

use crate::mmio::{reg::KEYINPUT, sys::Button};
use core::arch::asm;
use enumset::EnumSet;

/// Resets the GBA.
pub fn reset() -> ! {
    unsafe {
        asm!("swi #0x00");
    }
    abort()
}

/// Crashes the console on purpose, preventing it from running any code until it is reset.
///
/// This sets the GBA into a state where no functions (such as DMA or interrupts) are running, and
/// no further code will be run. This will also disable sound to prevent this state from hurting
/// the player's ears.
#[inline(always)]
pub fn abort() -> ! {
    crate::asm::abort()
}

/// Returns the currently pressed keys.
///
/// This should be called once a frame, instead of every time button state is checked.
pub fn pressed_keys() -> EnumSet<Button> {
    !KEYINPUT.read()
}
