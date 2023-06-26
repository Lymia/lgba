//! Allows running code when the GBA receives a hardware interrupt.

use crate::{
    mmio::reg::{BIOS_IF, IE, IF, IME},
    sync::Static,
};
use core::{
    ffi::c_void,
    pin::Pin,
    sync::atomic::{compiler_fence, Ordering},
};

pub use crate::mmio::sys::Interrupt;

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

            if old_head.is_null() {
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

    // enable interrupts
    IS_IN_INTERRUPT.write(false);
    IME.write(true);
}

/// Returns whether the GBA is currently processing an interrupt.
pub fn is_in_interrupt() -> bool {
    IS_IN_INTERRUPT.read()
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

// Executes a closure with interrupts disabled in its body.
pub fn suppress<R>(func: impl FnOnce() -> R) -> R {
    let prev_ime = IME.read();

    compiler_fence(Ordering::Acquire);
    let result = func();
    compiler_fence(Ordering::Release);

    IME.write(prev_ime);
    result
}
