

/// An enum representing the available verbosity levels of the logging framework
///
/// A `LogLevel` may be compared directly to a `LogLevelFilter`.
#[repr(usize)]
pub enum LogLevel {
    /// The "error" level.
    ///
    /// Designates very serious errors.
    Error = 1, // This way these line up with the discriminants for LogLevelFilter below
    /// The "warn" level.
    ///
    /// Designates hazardous situations.
    Warn,
    /// The "info" level.
    ///
    /// Designates useful information.
    Info,
    /// The "debug" level.
    ///
    /// Designates lower priority information.
    Debug,
    /// The "trace" level.
    ///
    /// Designates very low priority, often extremely verbose, information.
    Trace,
}

use core::fmt;

impl fmt::Display for LogLevel {
    
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "INFO")
    }
    
}


use uart::UART0;
use core::fmt::Write;

pub fn __log(lvl: LogLevel, file: &str, line: u32, module_path: &str, msg: &str) {
    write!(UART0, "[{}] {}/{}:{} {}\n", lvl, module_path, file, line, msg);
}

#[macro_export]
macro_rules! log {
    ($lvl:expr, $msg:expr) => ( log::__log($lvl, file!(), line!(), module_path!(), $msg) )
}

#[macro_export]
macro_rules! info {
    ($msg:expr) => ( log::__log(log::LogLevel::Info, file!(), line!(), module_path!(), $msg) )
}
