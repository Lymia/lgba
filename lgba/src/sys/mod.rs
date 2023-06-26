//! Various functions and helper types for basic GBA system functions.

use crate::mmio::{
    reg::{KEYCNT, KEYINPUT},
    sys::{ButtonCondition, KeyCnt},
};
use enumset::EnumSet;

/// Crashes the console on purpose, preventing it from running any code until it is reset.
///
/// This sets the GBA into a state where no functions (such as DMA or interrupts) are running, and
/// no further code will be run. This will also disable sound to prevent this state from hurting
/// the player's ears.
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

mod bios;
pub use bios::*;
