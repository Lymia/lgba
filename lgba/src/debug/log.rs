use crate::debug::DebugLevel;
use log::{Level, Log, Metadata, Record};

pub struct GbaLogger;
impl Log for GbaLogger {
    fn enabled(&self, _: &Metadata) -> bool {
        crate::debug::is_enabled()
    }
    fn log(&self, record: &Record) {
        let level = match record.level() {
            Level::Error => DebugLevel::Error,
            Level::Warn => DebugLevel::Warn,
            Level::Info => DebugLevel::Info,
            Level::Debug => DebugLevel::Debug,
            Level::Trace => DebugLevel::Trace,
        };
        crate::debug::debug_print(
            level,
            format_args!("{} | {}", record.module_path().unwrap_or("<unknown>"), record.args()),
        );
    }
    fn flush(&self) {}
}
