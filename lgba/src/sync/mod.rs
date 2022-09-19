//! Module containing GBA-specific synchronization primitives.

mod locks;
mod statics;

pub use locks::*;
pub use statics::*;

use core::arch::asm;

/// Marks that a pointer is read without actually reading from this.
///
/// This uses an [`asm!`] instruction that marks the parameter as being read,
/// requiring the compiler to treat this function as if anything could be
/// done to it.
#[inline(always)]
pub fn memory_read_hint<T>(val: *const T) {
    unsafe { asm!("/* {0} */", in(reg) val, options(readonly, nostack)) }
}

/// Marks that a pointer is read or written to without actually writing to it.
///
/// This uses an [`asm!`] instruction that marks the parameter as being read
/// and written, requiring the compiler to treat this function as if anything
/// could be done to it.
#[inline(always)]
pub fn memory_write_hint<T>(val: *mut T) {
    unsafe { asm!("/* {0} */", in(reg) val, options(nostack)) }
}
