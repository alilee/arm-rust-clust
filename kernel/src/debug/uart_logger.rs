//! Some doco

extern crate log;

use crate::device::uart;
use log::{Level, LevelFilter, Metadata, Record, SetLoggerError};

use core::fmt::Write;

impl log::Log for uart::Uart {
    #[allow(array_into_iter)]
    fn enabled(&self, metadata: &Metadata) -> bool {
        use Level::*;
        let levels = [("gic", Info), ("gicv2", Trace), ("timer", Info)];
        let level = levels.into_iter().fold(Trace, |base, (suffix, level)| {
            if metadata.target().ends_with(suffix) {
                *level
            } else {
                base
            }
        });
        metadata.level() <= level
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            writeln!(
                uart::UART0,
                "[{}] {}: [{}:{}] {}",
                record.level(),
                record.target(),
                record.file().unwrap_or("<unknown>"),
                record.line().unwrap_or(0),
                record.args()
            )
            .unwrap_or(());
        }
    }

    fn flush(&self) {}
}

/// Doco
pub fn init() -> Result<(), SetLoggerError> {
    log::set_logger(&uart::UART0).map(|()| log::set_max_level(LevelFilter::Trace))
}
