//! Module containing code useful for working with interrupts.

use crate::mmio::reg::{BIOS_IF, IE, IF, IME};

pub(crate) fn interrupt_handler() {
    // disable interrupts
    IME.write(false);

    // clear interrupts
    let triggered_interrupts = IE.read() & IF.read();
    IF.write(triggered_interrupts);
    BIOS_IF.write(triggered_interrupts);

    // enable interrupts
    IME.write(true);
}

// Executes a closure with interrupts disabled in its body.
pub fn disable<R>(func: impl FnOnce() -> R) -> R {
    let prev_ime = IME.read();
    let result = func();
    IME.write(prev_ime);
    result
}
