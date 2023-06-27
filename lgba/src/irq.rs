//! Allows running code when the GBA receives a hardware interrupt.
//!
//! # Example
//!
//! Example of basic interrupt usage:
//!
//! ```rust
//! use core::pin::pin;
//! use lgba::{irq, println};
//!
//! let int_vblank = pin!(irq::InterruptHandler::new(|| println!("VBlank!")));
//! int_vblank.register(irq::Interrupt::VBlank);
//! ```
//!
//! Example of using interrupts with timers:
//!
//! ```rust
//! use core::pin::pin;
//! use lgba::{irq, println, timer};
//!
//! let mut timer = timer::TimerId::Timer0.create();
//! timer.set_overflow_at(10000).set_interrupt_enabled(true).set_enabled(true);
//!
//! let int_timer0 = pin!(irq::InterruptHandler::new(|| println!("Timer0!")));
//! int_timer0.register(irq::Interrupt::Timer0);
//! irq::enable(irq::Interrupt::Timer0);
//! ```
//!
//! # Technical details
//!
//! The interrupt handler used by `lgba` is written entirely in Rust. Hence, it runs in the `irq`
//! CPU mode rather than `user`. Furthermore, it supports recursive interrupts by processing the
//! [`IF`] register in a loop, rather than enabling interrupts during its execution.
//!
//! An example of code equivalent to the default interrupt handler can be found in the
//! documentation for [`DEFAULT_INTERRUPT_HANDLER`].
//!
//! [`IF`]: https://mgba-emu.github.io/gbatek/#4000202h---if---interrupt-request-flags--irq-acknowledge-rw-see-below

use crate::{
    mmio::reg::{BIOS_IF, DISPSTAT, IE, IF, IME},
    sync::Static,
};
use core::{ffi::c_void, pin::Pin};
use enumset::EnumSet;

pub use crate::mmio::sys::Interrupt;
use crate::sync::memory_write_hint;

// TODO: Better document limitations and expectations for interrupt handlers in lgba.

const INIT_STATIC_NONE: Static<*mut InterruptHandlerNode> = Static::new(core::ptr::null_mut());
static INTERRUPT_TABLE: [Static<*mut InterruptHandlerNode>; 14] = [INIT_STATIC_NONE; 14];
static IS_IN_INTERRUPT: Static<bool> = Static::new(false);

/// An interrupt handler.
///
/// This object must be pinned and then registered in order to actually run during interrupts.
pub struct InterruptHandler<T: FnMut() + Send + Sync> {
    func: T,
    node: InterruptHandlerNode,
}
impl<T: FnMut() + Send + Sync> InterruptHandler<T> {
    /// Creates a new interrupt handler wrapping a given function.
    pub fn new(func: T) -> Self {
        InterruptHandler { func, node: Default::default() }
    }

    unsafe fn call_wrapper(data: *mut c_void) {
        let func = &mut *(data as *mut T);
        func();
    }

    /// Registers the interrupt handler for execution.
    #[track_caller]
    pub fn register(self: Pin<&mut Self>, int: Interrupt) {
        suppress(|| unsafe {
            if is_in_interrupt() {
                interrupt_change_in_interrupt();
            }
            if self.node.is_registered {
                interrupt_already_registered();
            }

            let handler = Pin::into_inner_unchecked(self);
            let node_ptr = &mut handler.node as *mut _;
            let old_head = INTERRUPT_TABLE[int as usize].replace(node_ptr);

            handler.node.data = &mut handler.func as *mut _ as *mut _;
            handler.node.func = Self::call_wrapper;
            handler.node.next = old_head;
            handler.node.interrupt = int;
            handler.node.is_registered = true;

            if !old_head.is_null() {
                (*old_head).prev = node_ptr;
            }
        })
    }

    /// Unregistered the interrupt handler for execution.
    #[track_caller]
    pub fn deregister(self: Pin<&mut Self>) {
        suppress(|| unsafe {
            if is_in_interrupt() {
                interrupt_change_in_interrupt();
            }
            if self.node.is_registered {
                interrupt_not_registered();
            }

            let handler = Pin::into_inner_unchecked(self);

            if !handler.node.next.is_null() {
                (*handler.node.next).prev = handler.node.prev;
            }
            if !handler.node.prev.is_null() {
                (*handler.node.prev).next = handler.node.next;
            } else {
                INTERRUPT_TABLE[handler.node.interrupt as usize].write(handler.node.next);
            }
        })
    }
}
impl<T: FnMut() + Send + Sync> Drop for InterruptHandler<T> {
    fn drop(&mut self) {
        let pin = unsafe { Pin::new_unchecked(self) };
        pin.deregister();
    }
}

#[repr(C)]
struct InterruptHandlerNode {
    data: *mut c_void,
    func: unsafe fn(*mut c_void),
    next: *mut InterruptHandlerNode,
    prev: *mut InterruptHandlerNode,
    interrupt: Interrupt,
    is_registered: bool,
}
impl Default for InterruptHandlerNode {
    fn default() -> Self {
        InterruptHandlerNode {
            data: core::ptr::null_mut(),
            func: |_| {},
            interrupt: Interrupt::VBlank,
            next: core::ptr::null_mut(),
            prev: core::ptr::null_mut(),
            is_registered: false,
        }
    }
}

#[inline(always)]
unsafe fn run_chain(mut node: *mut InterruptHandlerNode) {
    while !node.is_null() {
        let cur_node = &mut *node;
        node = cur_node.next;
        (cur_node.func)(cur_node.data);
    }
}

pub(crate) unsafe fn interrupt_handler() {
    // disable interrupts & check user canaries
    IME.write(false);
    crate::asm::check_user_canary();

    // handle interrupts until none are left queued
    // this emulates something like nested interrupts without actually nesting interrupts
    mark_in_interrupt_0(|| {
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
            trigger_interrupts_0(triggered_interrupts);
        }
    });

    // enable interrupts & check interrupt canaries
    crate::asm::check_interrupt_canary();
    IME.write(true);
}

/// Returns whether the GBA is currently processing an interrupt.
pub fn is_in_interrupt() -> bool {
    IS_IN_INTERRUPT.read()
}

/// Enables a given set of interrupts.
///
/// By default, the [`VBlank`](`Interrupt::VBlank`) interrupt is enabled, and nothing else.
#[inline]
pub fn enable(interrupts: impl Into<EnumSet<Interrupt>>) {
    suppress(|| {
        if is_in_interrupt() {
            interrupt_change_in_interrupt();
        }

        let old_ie = IE.read();
        raw_set_interrupts(old_ie, old_ie | interrupts.into());
    });
}

/// Disables a given set of interrupts.
///
/// By default, the [`VBlank`](`Interrupt::VBlank`) interrupt is enabled, and nothing else.
#[inline]
pub fn disable(interrupts: impl Into<EnumSet<Interrupt>>) {
    suppress(|| {
        if is_in_interrupt() {
            interrupt_change_in_interrupt();
        }

        let old_ie = IE.read();
        raw_set_interrupts(old_ie, old_ie - interrupts.into());
    });
}

/// Returns the set of interrupts enabled.
pub fn enabled() -> EnumSet<Interrupt> {
    IE.read()
}

#[inline]
fn raw_set_interrupts(old_ie: EnumSet<Interrupt>, new_ie: EnumSet<Interrupt>) {
    let changed = new_ie ^ old_ie;

    IE.write(new_ie);
    if !changed.is_disjoint(Interrupt::VBlank | Interrupt::HBlank | Interrupt::VCounter) {
        DISPSTAT.write(
            DISPSTAT
                .read()
                .with_vblank_irq_enabled(new_ie.contains(Interrupt::VBlank))
                .with_hblank_irq_enabled(new_ie.contains(Interrupt::HBlank))
                .with_vcount_irq_enabled(new_ie.contains(Interrupt::VCounter)),
        );
    }
}

#[inline(never)]
#[track_caller]
const fn interrupt_already_registered() {
    panic!("Interrupt already registered.");
}

#[inline(never)]
#[track_caller]
const fn interrupt_not_registered() {
    panic!("Interrupt is not registered.");
}

#[inline(never)]
#[track_caller]
const fn interrupt_change_in_interrupt() {
    panic!("Cannot change registered interrupts in an interrupt.");
}

/// Executes a closure with interrupts disabled in its body.
pub fn suppress<R>(mut func: impl FnOnce() -> R) -> R {
    let prev_ime = IME.read();

    memory_write_hint(&mut func);
    let mut result = func();
    memory_write_hint(&mut result);

    IME.write(prev_ime);
    result
}

/// Reads the set of interrupts that have been triggered since the last time they were checked.
#[cfg(feature = "low_level")]
#[doc(cfg(feature = "low_level"))]
pub fn interrupts_triggered() -> EnumSet<Interrupt> {
    IF.read()
}

/// Acknowledges a set of interrupts, clearing them from the triggered interrupts flags.
#[cfg(feature = "low_level")]
#[doc(cfg(feature = "low_level"))]
pub fn acknowledge_interrupts(interrupts: impl Into<EnumSet<Interrupt>>) {
    let interrupts = interrupts.into();
    IF.write(interrupts);
    BIOS_IF.write(BIOS_IF.read() | interrupts);
}

/// Triggers the interrupt handlers for a given set of interrupts.
///
/// # Safety
///
/// This function *must* be called under the context of [`mark_in_interrupt`], as it iterates an
/// internal intrusive linked list with no checks for iterator invalidation.
///
/// Furthermore, it may cause incorrect behavior in user or library code that expects interrupts to
/// only happen after certain processes have finished.
#[cfg(feature = "low_level")]
#[doc(cfg(feature = "low_level"))]
pub unsafe fn trigger_interrupts(interrupts: impl Into<EnumSet<Interrupt>>) {
    trigger_interrupts_0(interrupts.into());
}
unsafe fn trigger_interrupts_0(interrupts: EnumSet<Interrupt>) {
    macro_rules! check_interrupt {
        ($interrupt:expr) => {
            if interrupts.contains($interrupt) {
                unsafe {
                    run_chain(INTERRUPT_TABLE[$interrupt as usize].read());
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

/// Sets the [`IME`] flag directly.
///
/// [`IME`]: https://mgba-emu.github.io/gbatek/#4000208h---ime---interrupt-master-enable-register-rw
///
/// # Safety
///
/// `lgba` is written under the assumption that interrupts are enabled by default, and once
/// disabled cannot be enabled again by safe code. Enabling interrupts at any time in which
/// `lgba` expects it to be disabled may cause undefined behavior.
#[cfg(feature = "low_level")]
#[doc(cfg(feature = "low_level"))]
pub unsafe fn set_interrupts_enabled(enabled: bool) {
    IME.write(enabled);
}

/// Sets the underlying interrupt handler used by the GBA.
///
/// Changing this to anything other than [`DEFAULT_INTERRUPT_HANDLER`] may cause `lgba`
/// functionality to break.
///
/// # Safety
///
/// The `handler` function **MUST** be ARM code or undefined behavior will happen. Furthermore,
/// the function cannot access any `lgba` functionality that would normally cause a panic during
/// interrupts, as this bypasses that check.
#[cfg(feature = "low_level")]
#[doc(cfg(feature = "low_level"))]
pub unsafe fn set_interrupt_handler(handler: unsafe extern "C" fn()) {
    use crate::mmio::reg::BIOS_IRQ_ENTRY;
    BIOS_IRQ_ENTRY.write(handler);
}

/// Calls a function as if it were executing in an interrupt.
///
/// This means any functions in `lgba` that would normally panic when called in an interrupt will
/// no longer work inside the closure body. This should be used by any code that includes a custom
/// interrupt handler.
///
/// # Safety
///
/// This function should only be called when interrupts are disabled in order to avoid a race
/// condition.
#[cfg(feature = "low_level")]
#[doc(cfg(feature = "low_level"))]
pub unsafe fn mark_in_interrupt<R>(mut func: impl FnOnce() -> R) -> R {
    mark_in_interrupt_0(func)
}
fn mark_in_interrupt_0<R>(mut func: impl FnOnce() -> R) -> R {
    let prev_in_interrupt = IS_IN_INTERRUPT.read();

    memory_write_hint(&mut func);
    let mut result = func();
    memory_write_hint(&mut result);

    IS_IN_INTERRUPT.write(prev_in_interrupt);
    result
}

/// The interrupt handler used by lgba.
///
/// The function this points to is available to assembly code under the symbol
/// `__lgba_interrupt_handler`, even when the `low_level` feature is disabled.
///
/// # Implementation
///
/// The default handler is equivalent to the following code.
///
/// ```rust,no_run
/// use lgba::{arm, irq, sys};
///
/// #[arm]
/// unsafe extern "C" fn default_interrupt_handler() {
///     unsafe fn handler_body() {
///         irq::set_interrupts_enabled(false);
///         sys::check_user_canary();
///         irq::mark_in_interrupt(|| {
///             loop {
///                 let triggered_interrupts = irq::enabled() | irq::interrupts_triggered();
///                 if triggered_interrupts.is_empty() {
///                     break;
///                 }
///                 irq::acknowledge_interrupts(triggered_interrupts);
///                 irq::trigger_interrupts(triggered_interrupts);
///             }
///         });
///         sys::check_interrupt_canary();
///         irq::set_interrupts_enabled(true);
///     }
///
///     handler_body();
/// }
/// ```
#[cfg(feature = "low_level")]
#[doc(cfg(feature = "low_level"))]
pub static DEFAULT_INTERRUPT_HANDLER: unsafe extern "C" fn() =
    crate::asm::DEFAULT_INTERRUPT_HANDLER;
