//! Module containing code useful for working with interrupts.

use crate::mmio::reg::IME;

// Executes a closure with interrupts disabled in its body.
pub fn disable<R>(func: impl FnOnce() -> R) -> R {
    let prev_ime = IME.read();
    let result = func();
    IME.write(prev_ime);
    result
}
