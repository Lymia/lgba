//! Various functions and helper types for basic GBA system functions.

use crate::mmio::{
    reg::{KEYCNT, KEYINPUT},
    sys::{ButtonCondition, KeyCnt},
};
use core::ops::Range;
use enumset::EnumSet;

/// Crashes the console on purpose, preventing it from running any code until it is reset.
///
/// This sets the GBA into a state where no functions (such as DMA or interrupts) are running, and
/// no further code will be run. This will also disable sound to prevent this state from hurting
/// the player's ears.
///
/// This function is available to assembly code under the name `__lgba_abort`.
#[inline(always)]
pub fn abort() -> ! {
    crate::asm::abort()
}

#[doc(inline)]
pub use crate::mmio::sys::Button;

/// Returns the currently pressed keys.
///
/// This should be called once a frame, instead of every time button state is checked.
pub fn pressed_keys() -> EnumSet<Button> {
    !KEYINPUT.read()
}

/// Sets the keys that trigger a [`Keypad`] interrupt.
///
/// The interrupt must still be enabled separately with [`irq::enable`].
///
/// [`irq::enable`]: crate::irq::enable
/// [`Keypad`]: crate::irq::Interrupt::Keypad
pub fn set_keypad_irq_keys(keys: impl Into<EnumSet<Button>>) {
    KEYCNT.write(
        KeyCnt::default()
            .with_enable_irq(true)
            .with_condition(ButtonCondition::LogicalOr)
            .with_keys(keys.into()),
    );
}

/// Sets the key combination that triggers a [`Keypad`] interrupt.
///
/// The interrupt must still be enabled separately with [`irq::enable`].
///
/// [`irq::enable`]: crate::irq::enable
/// [`Keypad`]: crate::irq::Interrupt::Keypad
pub fn set_keypad_irq_combo(combo: impl Into<EnumSet<Button>>) {
    KEYCNT.write(
        KeyCnt::default()
            .with_enable_irq(true)
            .with_condition(ButtonCondition::LogicalAnd)
            .with_keys(combo.into()),
    );
}

/// Disables hardware from sending a [`Keypad`] interrupt.
///
/// [`Keypad`]: crate::irq::Interrupt::Keypad
pub fn disable_keypad_irq() {
    KEYCNT.write(KeyCnt::default());
}

/// The range of iwram that isn't allocated by either the stack or static variables.
pub fn iwram_free_range() -> Range<*const u8> {
    let raw_range = crate::asm::iwram_free_range();
    raw_range.start as *const u8..raw_range.end as *const u8
}

/// The range of ewram that isn't allocated by static variables.
pub fn ewram_free_range() -> Range<*const u8> {
    let raw_range = crate::asm::ewram_free_range();
    raw_range.start as *const u8..raw_range.end as *const u8
}

/// Manually checks that the canary for the user stack has not been changed.
///
/// If it has been, this function panics.
#[cfg(feature = "low_level")]
#[doc(cfg(feature = "low_level"))]
pub fn check_user_canary() {
    crate::asm::check_user_canary();
}

/// Manually checks that the canary for the interrupt stack has not been changed.
///
/// If it has been, this function panics.
#[cfg(feature = "low_level")]
#[doc(cfg(feature = "low_level"))]
pub fn check_interrupt_canary() {
    crate::asm::check_interrupt_canary();
}

mod bios;
#[macro_use]
mod macros;
#[cfg(feature = "allocator")]
pub(crate) mod allocator;

#[cfg(feature = "allocator")]
pub use allocator::*;
pub use bios::*;
