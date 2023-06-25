use core::arch::asm;

/// Resets the GBA.
pub fn reset() -> ! {
    unsafe {
        asm!("swi #0x00");
    }
    crate::sys::abort()
}

/// Waits until VBlank.
///
/// V-blank interrupts must be enabled in both the graphics controller settings and the interrupt
/// settings, or else this function will freeze indefinitely.
pub fn wait_for_vblank() {
    unsafe {
        asm!("swi #0x05");
    }
}
