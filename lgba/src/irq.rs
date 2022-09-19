//! Module containing code useful for working with interrupts.

// TODO: Document
pub fn disable<R>(func: impl FnOnce() -> R) -> R {
    // TODO: Actually disable IRQs
    func()
}
