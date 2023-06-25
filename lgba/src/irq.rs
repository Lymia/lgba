//! Module containing code useful for working with interrupts.

use crate::{
    mmio::{
        reg::{BIOS_IF, IE, IF, IME},
        sys::Interrupt,
    },
    sync::Static,
};

const INIT_STATIC_NONE: Static<Option<fn()>> = Static::new(None);
static INTERRUPT_TABLE: [Static<Option<fn()>>; 14] = [INIT_STATIC_NONE; 14];
static IS_IN_INTERRUPT: Static<bool> = Static::new(false);

pub(crate) fn interrupt_handler() {
    // disable interrupts
    IME.write(false);
    IS_IN_INTERRUPT.write(true);

    // handle interrupts until none are left queued
    // this emulates something like nested interrupts without actually nesting interrupts
    loop {
        // determine the interrupts that have been triggered
        let triggered_interrupts = IE.read() & IF.read();
        if triggered_interrupts.is_empty() {
            break;
        }

        // notify the bios and hardware that we have handled interrupts
        IF.write(triggered_interrupts);
        BIOS_IF.write(BIOS_IF.read() | triggered_interrupts);

        // check interrupt functions
        macro_rules! check_interrupt {
            ($interrupt:expr) => {
                if triggered_interrupts.contains($interrupt) {
                    if let Some(handler) = INTERRUPT_TABLE[$interrupt as usize].read() {
                        handler();
                    }
                }
            };
        }
        check_interrupt!(Interrupt::VBlank);
        check_interrupt!(Interrupt::HBlank);
        check_interrupt!(Interrupt::VCounter);
        check_interrupt!(Interrupt::Timer0);
        check_interrupt!(Interrupt::Timer1);
        check_interrupt!(Interrupt::Timer2);
        check_interrupt!(Interrupt::Timer3);
        check_interrupt!(Interrupt::Serial);
        check_interrupt!(Interrupt::Dma0);
        check_interrupt!(Interrupt::Dma1);
        check_interrupt!(Interrupt::Dma2);
        check_interrupt!(Interrupt::Dma3);
        check_interrupt!(Interrupt::Keypad);
        check_interrupt!(Interrupt::GamePak);
    }

    // enable interrupts
    IS_IN_INTERRUPT.write(false);
    IME.write(true);
}

///
pub struct InterruptHandlerGuard(Interrupt);
impl Drop for InterruptHandlerGuard {
    fn drop(&mut self) {
        INTERRUPT_TABLE[self.0 as usize].write(None);
    }
}

#[track_caller]
pub fn register(interrupt: Interrupt, handler: fn()) -> InterruptHandlerGuard {
    disable(|| {
        if IS_IN_INTERRUPT.read() {
            interrupt_change_in_interrupt();
        }
        if INTERRUPT_TABLE[interrupt as usize].read().is_some() {
            interrupt_already_registered();
        }
        INTERRUPT_TABLE[interrupt as usize].write(Some(handler));
        InterruptHandlerGuard(interrupt)
    })
}

#[inline(never)]
#[track_caller]
const fn interrupt_already_registered() {
    panic!("Interrupt already registered.");
}

#[inline(never)]
#[track_caller]
const fn interrupt_change_in_interrupt() {
    panic!("Cannot change registered interrupts in an interrupt.");
}

// Executes a closure with interrupts disabled in its body.
pub fn disable<R>(func: impl FnOnce() -> R) -> R {
    let prev_ime = IME.read();
    let result = func();
    IME.write(prev_ime);
    result
}
