//! Support for emitting debug messages on emulators.
//!
//! Currently supported emulators:
//! * mgba-qt
//! * NO$GBA

use crate::{
    emulator::{MgbaDebugFlag, MgbaDebugLevel},
    mmio::emulator,
    sync::{InitOnce, Static},
};
use core::fmt::{Arguments, Debug, Error, Write};

//pub mod mgba;
//pub mod nocash;

/// A debug level that a log message may be emitted at.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(missing_docs)]
pub enum DebugLevel {
    Error,
    Warn,
    Info,
    Debug,
}

#[derive(Copy, Clone, Eq, PartialEq)]
enum DebugType {
    None,
    MGba,
    NoCash,
}

/// Debug interface detection only happens once.
static CACHED_DEBUG_TYPE: InitOnce<DebugType> = InitOnce::new();
fn detect_debug_type() -> DebugType {
    *CACHED_DEBUG_TYPE.get(|| {
        crate::irq::disable(|| {
            // check if this is mgba
            emulator::MGBA_DEBUG_ENABLE.write(emulator::MGBA_DEBUG_ENABLE_INPUT);
            if emulator::MGBA_DEBUG_ENABLE.read() == emulator::MGBA_DEBUG_ENABLE_OUTPUT {
                return DebugType::MGba;
            }

            // check if this is no$gba
            let is_no_cash = (0..emulator::NO_CASH_EXPECTED_SIG.len()).all(|i| {
                emulator::NO_CASH_SIG.index(i).read() == emulator::NO_CASH_EXPECTED_SIG[i]
            });
            if is_no_cash {
                return DebugType::NoCash;
            }

            // we are either on real hardware or an emulator we don't recognize
            DebugType::None
        })
    })
}

/// The function type used for debug hooks.
pub type DebugHook = fn(DebugLevel, &Arguments<'_>) -> Result<(), Error>;

static DEBUG_MASTER_FLAG: Static<bool> = Static::new(true);
static DEBUG_DISABLED: Static<bool> = Static::new(false);
static DEBUG_HOOK: Static<Option<DebugHook>> = Static::new(None);

fn recalculate_master_flag() {
    if DEBUG_DISABLED.read() {
        DEBUG_MASTER_FLAG.write(false);
    } else if let Some(_) = DEBUG_HOOK.read() {
        DEBUG_MASTER_FLAG.write(true);
    } else {
        DEBUG_MASTER_FLAG.write(detect_debug_type() != DebugType::None)
    }
}

/// Sets whether debug messages is enabled.
///
/// Defaults to `true`.
pub fn set_enabled(enabled: bool) {
    crate::irq::disable(|| {
        DEBUG_DISABLED.write(!enabled);
        recalculate_master_flag();
    })
}

/// Sets the debug hook.
///
/// Defaults to `None`.
pub fn set_debug_hook(hook: Option<DebugHook>) {
    crate::irq::disable(|| {
        DEBUG_HOOK.write(hook);
        recalculate_master_flag();
    })
}

/// Returns whether debugging will actually log any messages.
///
/// This does *not* return the value from [`set_enabled`] directly, as it will also return false
/// if `lgba` has detected no possible way to log.
#[inline(always)]
pub fn is_enabled() -> bool {
    DEBUG_MASTER_FLAG.read()
}

struct MGBADebug {
    flag: emulator::MgbaDebugFlag,
    bytes_written: usize,
}
impl core::fmt::Write for MGBADebug {
    fn write_str(&mut self, s: &str) -> Result<(), core::fmt::Error> {
        for byte in s.bytes() {
            if byte == b'\r' {
                continue;
            }
            if self.bytes_written == 256 || byte == b'\n' {
                emulator::MGBA_DEBUG_FLAG.write(self.flag);
                self.bytes_written = 0;
            }

            emulator::MGBA_DEBUG_STR
                .index(self.bytes_written)
                .write(byte);
            self.bytes_written += 1;
        }
        Ok(())
    }
}
impl Drop for MGBADebug {
    fn drop(&mut self) {
        if self.bytes_written != 0 {
            emulator::MGBA_DEBUG_FLAG.write(self.flag);
        }
    }
}

struct NoCashDebug;
impl core::fmt::Write for NoCashDebug {
    fn write_str(&mut self, s: &str) -> Result<(), core::fmt::Error> {
        for b in s.bytes() {
            emulator::NO_CASH_CHAR.write(b);
        }
        Ok(())
    }
}

/// Prints a line to the debug interface, if there is any.
#[inline(never)]
fn debug_print_0(level: DebugLevel, args: &Arguments<'_>) -> Result<(), Error> {
    if DEBUG_MASTER_FLAG.read() {
        if let Some(hook) = DEBUG_HOOK.read() {
            hook(level, args)
        } else {
            match detect_debug_type() {
                DebugType::None => {
                    crate::irq::disable(recalculate_master_flag);
                    Ok(())
                }
                DebugType::MGba => {
                    let level = match level {
                        DebugLevel::Error => MgbaDebugLevel::Error,
                        DebugLevel::Warn => MgbaDebugLevel::Warn,
                        DebugLevel::Info => MgbaDebugLevel::Info,
                        DebugLevel::Debug => MgbaDebugLevel::Debug,
                    };
                    let mut write = MGBADebug {
                        flag: MgbaDebugFlag::default().with_level(level).with_send(true),
                        bytes_written: 0,
                    };
                    write!(write, "{}", args)
                }
                DebugType::NoCash => {
                    write!(NoCashDebug, "User: [{:?}] {}\n", level, args)
                }
            }
        }
    } else {
        Ok(())
    }
}

/// Logs a message in the debug log.
#[inline(never)]
pub fn debug_print(level: DebugLevel, args: &Arguments<'_>) {
    match debug_print_0(level, args) {
        Ok(_) => (),
        Err(_) => crate::sys::abort(),
    }
}

/// Logs a message to the debug log at a given log level.
///
/// This macro takes a single identifier (`Error`, `Warn`, `Info`, or `Debug`) followed by a comma
/// and an argument list identical to [`format_args!`]. See the documentation for that macro for
/// further details.
#[macro_export]
macro_rules! log {
    ($level:ident, $($rest:tt)*) => {
        let args = format_args!($($rest)*);
        if $crate::debug::is_enabled() {
            $crate::debug::debug_print(
                $crate::debug::DebugLevel::$level,
                &$crate::__macro_export::core::format_args!(
                    "{} | {}",
                    $crate::__macro_export::core::module_path!(),
                    args,
                ),
            );
        }
    };
}

/// Prints a message to the debug log at the info level.
///
/// This macro takes an argument list identical to [`format_args!`]. See the documentation for
/// that macro for further details.
#[macro_export]
macro_rules! println {
    ($($rest:tt)*) => {
        if $crate::debug::is_enabled() {
            $crate::debug::debug_print(
                $crate::debug::DebugLevel::Info,
                &$crate::__macro_export::core::format_args!($($rest)*),
            );
        }
    };
}

/// Prints a message to the debug log at the error level.
///
/// This macro takes an argument list identical to [`format_args!`]. See the documentation for
/// that macro for further details.
#[macro_export]
macro_rules! eprintln {
    ($($rest:tt)*) => {
        if $crate::debug::is_enabled() {
            $crate::debug::debug_print(
                $crate::debug::DebugLevel::Error,
                &$crate::__macro_export::core::format_args!($($rest)*),
            );
        }
    };
}

/// Logs a message in the debug log at the error level.
///
/// This macro takes an argument list identical to [`format_args!`]. See the documentation for
/// that macro for further details.
#[macro_export]
macro_rules! error {
    ($($rest:tt)*) => {
        $crate::log!(Error, $($rest)*)
    };
}

/// Logs a message in the debug log at the warn level.
///
/// This macro takes an argument list identical to [`format_args!`]. See the documentation for
/// that macro for further details.
#[macro_export]
macro_rules! warn {
    ($($rest:tt)*) => {
        $crate::log!(Warn, $($rest)*)
    };
}

/// Logs a message in the debug log at the info level.
///
/// This macro takes an argument list identical to [`format_args!`]. See the documentation for
/// that macro for further details.
#[macro_export]
macro_rules! info {
    ($($rest:tt)*) => {
        $crate::log!(Info, $($rest)*)
    };
}

/// Logs a message in the debug log at the debug level.
///
/// This macro takes an argument list identical to [`format_args!`]. See the documentation for
/// that macro for further details.
#[macro_export]
macro_rules! debug {
    ($($rest:tt)*) => {
        $crate::log!(Debug, $($rest)*)
    };
}


/// Prints and returns the value of a given expression for quick and dirty debugging.
///
/// This is based on the [`std::dbg!`](https://doc.rust-lang.org/std/macro.dbg.html) macro (which
/// is not otherwise available in `#![no_std]` platforms) See the documentation for that macro
/// for further details.
#[macro_export]
macro_rules! dbg {
    // NOTE: We cannot use `concat!` to make a static string as a format argument
    // of `eprintln!` because `file!` could contain a `{` or
    // `$val` expression could be a block (`{ .. }`), in which case the `eprintln!`
    // will be malformed.
    () => {
        $crate::eprintln!(
            "[{}:{}]",
            $crate::__macro_export::core::file!(),
            $crate::__macro_export::core::line!(),
        )
    };
    ($val:expr $(,)?) => {
        // Use of `match` here is intentional because it affects the lifetimes
        // of temporaries - https://stackoverflow.com/a/48732525/1063961
        match $val {
            tmp => {
                $crate::eprintln!(
                    "[{}:{}] {} = {:#?}",
                    $crate::__macro_export::core::file!(),
                    $crate::__macro_export::core::line!(),
                    $crate::__macro_export::core::stringify!($val),
                    &tmp,
                );
                tmp
            }
        }
    };
    ($($val:expr),+ $(,)?) => {
        ($($crate::dbg!($val)),+,)
    };
}
