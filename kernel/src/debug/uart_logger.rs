//! Some doco

extern crate log;

use crate::device::uart;
use crate::util::locked::Locked;

use log::{Level, LevelFilter, Metadata, Record, SetLoggerError};

use core::fmt::Write;

impl log::Log for Locked<uart::Uart> {
    #[allow(array_into_iter)]
    fn enabled(&self, metadata: &Metadata) -> bool {
        use Level::*;
        let levels = [
            ("frames", Info),
            ("gic", Info),
            ("gicv2", Trace),
            ("timer", Info),
            ("aarch64::pager", Trace),
            ("pager::table", Trace),
        ];
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
            const BUFFER: [&str; 6] = ["", "!", "*", " ", "  ", "  "];
            let mut locked = self.lock();
            writeln!(
                locked,
                "{:5} [{:>50} {:3}] {}{}",
                record.level(),
                record.target(),
                record.line().unwrap_or(0),
                BUFFER[record.level() as usize],
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

pub fn reset() -> Result<(), u64> {
    let result = uart::UART0.lock().reset();
    log::info!("uart_logger reset");
    result
}
