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
/// # Warning
///
/// V-blank interrupts must be enabled in both the graphics controller settings and the interrupt
/// settings, or else this function will freeze indefinitely.
///
/// These functions are enabled by default by lgba, and this limitation is only a concern if your
/// code manually changes the configuration for either.
pub fn wait_for_vblank() {
    unsafe {
        asm!("swi #0x05");
    }
}
