extern crate log;

use log::{LogRecord, LogLevel, LogLevelFilter, LogMetadata, SetLoggerError, ShutdownLoggerError};
use dev::uart;

use core::fmt::Write;

impl log::Log for uart::Uart {
    fn enabled(&self, metadata: &LogMetadata) -> bool {
        metadata.level() <= LogLevel::Info
    }

    fn log(&self, record: &LogRecord) {
        if self.enabled(record.metadata()) {
            writeln!(
                uart::UART0,
                "[{}] {}: [{}:{}] {}",
                record.level(),
                record.target(),
                record.location().file(),
                record.location().line(),
                record.args()
            ).unwrap_or(());
        }
    }
}

pub fn init() -> Result<(), SetLoggerError> {
    unsafe {
        log::set_logger_raw(|max_log_level| {
            max_log_level.set(LogLevelFilter::Info);
            &uart::UART0
        })
    }
}

pub fn shutdown() -> Result<(), ShutdownLoggerError> {
    log::shutdown_logger_raw().map(|_| { writeln!(uart::UART0, ".!").unwrap(); })
}
