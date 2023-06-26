use crate::mmio::reg::IME;
use core::arch::asm;

/// Resets the GBA.
pub fn reset() -> ! {
    IME.write(false); // prevent crashes during the BIOS reset from interrupts
    unsafe {
        asm!("swi #0x00");
    }
    crate::sys::abort()
}

/// Waits until VBlank.
///
/// # Warning
///
/// The vblank interrupt must be enabled or else this function will freeze indefinitely.
///
/// The interrupt is enabled at startup by lgba, and this is only a concern if your code manually
/// disables the interrupt at some point.
pub fn wait_for_vblank() {
    if crate::irq::is_in_interrupt() {
        wait_for_vblank_in_interrupt();
    }
    unsafe {
        asm!("swi #0x05");
    }
}

#[inline(never)]
#[track_caller]
const fn wait_for_vblank_in_interrupt() {
    panic!("wait_for_vblank cannot be called in an interrupt.");
}
