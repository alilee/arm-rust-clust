//! Some doco

extern crate log;

use log::{Record, Level, LevelFilter, Metadata, SetLoggerError};
use crate::device::uart;

use core::fmt::Write;

impl log::Log for uart::Uart {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
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
            ).unwrap_or(());
        }
    }

    fn flush(&self) {}
}

/// Doco
pub fn init() -> Result<(), SetLoggerError> {
    log::set_logger(&uart::UART0)
        .map(|()| log::set_max_level(LevelFilter::Info))
}
