//! Support for emitting debug messages on emulators.
//!
//! Currently supported emulators:
//! * mgba-qt
//! * NO$GBA

use crate::{
    mmio::{
        emulator,
        emulator::{MgbaDebugFlag, MgbaDebugLevel},
    },
    sync::{InitOnce, Static},
};
use core::fmt::{Arguments, Debug, Error, Write};

// TODO: Handle format!("{i}");

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
        crate::irq::suppress(|| {
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
            DEBUG_MASTER_FLAG.write(false);
            DebugType::None
        })
    })
}

static DEBUG_MASTER_FLAG: Static<bool> = Static::new(true);

/// Returns whether debugging will actually log any messages.
#[inline(always)]
pub fn is_enabled() -> bool {
    DEBUG_MASTER_FLAG.read()
}

struct MGBADebug {
    flag: emulator::MgbaDebugFlag,
    bytes_written: usize,
}
impl Write for MGBADebug {
    #[inline(never)]
    fn write_str(&mut self, s: &str) -> Result<(), Error> {
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
    #[inline(never)]
    fn drop(&mut self) {
        if self.bytes_written != 0 {
            emulator::MGBA_DEBUG_FLAG.write(self.flag);
        }
    }
}

struct NoCashDebug;
impl Write for NoCashDebug {
    #[inline(never)]
    fn write_str(&mut self, s: &str) -> Result<(), Error> {
        for b in s.bytes() {
            emulator::NO_CASH_CHAR.write(b);
        }
        Ok(())
    }
}

/// Something that can be written to a debug stream.
pub trait DebugPrintable {
    fn print(&self, w: impl Write) -> Result<(), Error>;
}
impl<'a> DebugPrintable for Arguments<'a> {
    fn print(&self, mut w: impl Write) -> Result<(), Error> {
        w.write_fmt(*self)
    }
}
impl<'a> DebugPrintable for &'a str {
    fn print(&self, mut w: impl Write) -> Result<(), Error> {
        w.write_str(*self)
    }
}
macro_rules! debug_printable_tuple {
    ($($num:tt $id:ident)*) => {
        impl<$($id: DebugPrintable,)*> DebugPrintable for ($($id,)*) {
            fn print(&self, mut w: impl Write) -> Result<(), Error> {
                $(self.$num.print(&mut w)?;)*
                Ok(())
            }
        }
    };
}
debug_printable_tuple!(0 A 1 B);
debug_printable_tuple!(0 A 1 B 2 C);
debug_printable_tuple!(0 A 1 B 2 C 3 D);
debug_printable_tuple!(0 A 1 B 2 C 3 D 4 E);
debug_printable_tuple!(0 A 1 B 2 C 3 D 4 E 5 F);
debug_printable_tuple!(0 A 1 B 2 C 3 D 4 E 5 F 6 G);
debug_printable_tuple!(0 A 1 B 2 C 3 D 4 E 5 F 6 G 7 H);
debug_printable_tuple!(0 A 1 B 2 C 3 D 4 E 5 F 6 G 7 H 8 I);
debug_printable_tuple!(0 A 1 B 2 C 3 D 4 E 5 F 6 G 7 H 8 I 9 J);

trait DebugPrintableLowered {
    fn print_mgba(&self, w: MGBADebug) -> Result<(), Error>;
    fn print_nocash(&self, w: NoCashDebug) -> Result<(), Error>;
}
impl<T: DebugPrintable> DebugPrintableLowered for T {
    fn print_mgba(&self, w: MGBADebug) -> Result<(), Error> {
        self.print(w)
    }
    fn print_nocash(&self, w: NoCashDebug) -> Result<(), Error> {
        self.print(w)
    }
}

fn debug_print_0(level: DebugLevel, args: &dyn DebugPrintableLowered) -> Result<(), Error> {
    if DEBUG_MASTER_FLAG.read() {
        match detect_debug_type() {
            DebugType::None => Ok(()),
            DebugType::MGba => {
                let level = match level {
                    DebugLevel::Error => MgbaDebugLevel::Error,
                    DebugLevel::Warn => MgbaDebugLevel::Warn,
                    DebugLevel::Info => MgbaDebugLevel::Info,
                    DebugLevel::Debug => MgbaDebugLevel::Debug,
                };
                args.print_mgba(MGBADebug {
                    flag: MgbaDebugFlag::default().with_level(level).with_send(true),
                    bytes_written: 0,
                })
            }
            DebugType::NoCash => {
                let mut write = NoCashDebug;
                write.write_str("User: [")?;
                match level {
                    DebugLevel::Error => write.write_str("Error")?,
                    DebugLevel::Warn => write.write_str("Warn")?,
                    DebugLevel::Info => write.write_str("Info")?,
                    DebugLevel::Debug => write.write_str("Debug")?,
                }
                args.print_nocash(write)
            }
        }
    } else {
        Ok(())
    }
}

/// Logs a message to the debug log.
#[inline(never)]
pub fn debug_print(level: DebugLevel, args: impl DebugPrintable) {
    match debug_print_0(level, &args) {
        Ok(_) => (),
        Err(_) => crate::sys::abort(),
    }
}

/// DO NOT USE: This macro is not public API.
#[doc(hidden)]
#[macro_export]
macro_rules! __lgba__internal_print_impl {
    (@mod_prefix) => {
        $crate::__lgba__internal_print_impl!(@concat
            $crate::__macro_export::core::module_path!(),
            " | ",
        )
    };
    (@file_line_prefix) => {
        $crate::__lgba__internal_print_impl!(@concat
            "[",
            $crate::__macro_export::core::file!(),
            ":",
        )
    };
    (@file_line_suffix) => {
        $crate::__lgba__internal_print_impl!(@concat
            $crate::__macro_export::core::line!(),
            "] ",
        )
    };
    (@concat $($rest:tt)*) => {
        $crate::__macro_export::core::concat!($($rest)*)
    };
    (@print $level:ident, $expr:expr) => {
        if $crate::debug::is_enabled() {
            $crate::debug::debug_print($crate::debug::DebugLevel::$level, $expr);
        }
    };
    (@plain $level:ident, $text:literal) => {
        $crate::__lgba__internal_print_impl!(@print $level, $text)
    };
    (@plain $level:ident, ) => {
        $crate::__lgba__internal_print_impl!(@print $level, "")
    };
    (@plain $level:ident, $($rest:tt)*) => {
        $crate::__lgba__internal_print_impl!(@print $level,
            $crate::__macro_export::core::format_args!($($rest)*)
        )
    };
}

/// Logs a message to the debug log at a given log level.
///
/// This macro takes a single identifier (`Error`, `Warn`, `Info`, or `Debug`) followed by a comma
/// and an argument list identical to [`format_args!`]. See the documentation for that macro for
/// further details.
///
/// [`format_args!`]: https://doc.rust-lang.org/std/macro.format_args.html
#[macro_export]
macro_rules! log {
    ($level:ident, $text:literal) => {
        $crate::__lgba__internal_print_impl!(@print $level, (
            $crate::__lgba__internal_print_impl!(@mod_prefix),
            $text,
        ));
    };
    ($level:ident, $($rest:tt)*) => {
        $crate::__lgba__internal_print_impl!(@print $level, (
            $crate::__lgba__internal_print_impl!(@mod_prefix),
            $crate::__macro_export::core::format_args!($($rest)*),
        ));
    };
}

/// Prints a message to the debug log at the info level.
///
/// This macro takes an argument list identical to [`format_args!`]. See the documentation for
/// that macro for further details.
///
/// [`format_args!`]: https://doc.rust-lang.org/std/macro.format_args.html
#[macro_export]
macro_rules! println {
    ($($rest:tt)*) => {
        $crate::__lgba__internal_print_impl!(@plain Info, $($rest)*)
    };
}

/// Prints a message to the debug log at the error level.
///
/// This macro takes an argument list identical to [`format_args!`]. See the documentation for
/// that macro for further details.
///
/// [`format_args!`]: https://doc.rust-lang.org/std/macro.format_args.html
#[macro_export]
macro_rules! eprintln {
    ($($rest:tt)*) => {
        $crate::__lgba__internal_print_impl!(@plain Error, $($rest)*)
    };
}

/// Logs a message in the debug log at the error level.
///
/// This macro takes an argument list identical to [`format_args!`]. See the documentation for
/// that macro for further details.
///
/// [`format_args!`]: https://doc.rust-lang.org/std/macro.format_args.html
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
///
/// [`format_args!`]: https://doc.rust-lang.org/std/macro.format_args.html
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
///
/// [`format_args!`]: https://doc.rust-lang.org/std/macro.format_args.html
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
///
/// [`format_args!`]: https://doc.rust-lang.org/std/macro.format_args.html
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
    () => {
        $crate::__lgba__internal_print_impl!(@print Debug, (
            $crate::__lgba__internal_print_impl!(@file_line_prefix),
            $crate::__lgba__internal_print_impl!(@file_line_suffix),
        ));
    };
    ($val:expr $(,)?) => {
        // Use of `match` here is intentional because it affects the lifetimes
        // of temporaries - https://stackoverflow.com/a/48732525/1063961
        match $val {
            tmp => {
                $crate::__lgba__internal_print_impl!(@print Debug, (
                    $crate::__lgba__internal_print_impl!(@file_line_prefix),
                    $crate::__lgba__internal_print_impl!(@file_line_suffix),
                    $crate::__lgba__internal_print_impl!(@concat stringify!($val), " = "),
                    $crate::__macro_export::core::format_args!("{:#?}", &tmp),
                ));
                tmp
            }
        }
    };
    ($($val:expr),+ $(,)?) => {
        ($($crate::dbg!($val)),+,)
    };
}
