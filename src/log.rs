

/// An enum representing the available verbosity levels of the logging framework
///
/// A `LogLevel` may be compared directly to a `LogLevelFilter`.
#[repr(usize)]
pub enum LogLevel {
    /// The "error" level.
    ///
    /// Designates very serious errors.
    Error = 1, 
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
        match *self {
            LogLevel::Error => write!(f, "ERROR"),
            LogLevel::Warn  => write!(f, "WARN "),
            LogLevel::Info  => write!(f, "Info "),
            LogLevel::Debug => write!(f, "Debug"),
            LogLevel::Trace => write!(f, "Trace"),
        }
    }
    
}

use uart::UART0;
use core::fmt::Write;

pub fn __log(lvl: LogLevel, file: &str, line: u32, module_path: &str, msg: &str) {
    write!(UART0, "[{}] {}/{}:{} {}\n", lvl, module_path, file, line, msg).ok();
}

#[macro_export]
macro_rules! log {
    ($lvl:expr, $msg:expr) => ( log::__log($lvl, file!(), line!(), module_path!(), $msg) )
}

#[macro_export]
macro_rules! error {
    ($msg:expr) => ( log::__log(log::LogLevel::Error, file!(), line!(), module_path!(), $msg) )
}

#[macro_export]
macro_rules! warn {
    ($msg:expr) => ( log::__log(log::LogLevel::Warn, file!(), line!(), module_path!(), $msg) )
}

#[macro_export]
macro_rules! info {
    ($msg:expr) => ( log::__log(log::LogLevel::Info, file!(), line!(), module_path!(), $msg) )
}


#[macro_export]
macro_rules! debug {
    ($msg:expr) => ( log::__log(log::LogLevel::Debug, file!(), line!(), module_path!(), $msg) )
}

#[macro_export]
macro_rules! trace {
    ($msg:expr) => ( log::__log(log::LogLevel::Trace, file!(), line!(), module_path!(), $msg) )
}
