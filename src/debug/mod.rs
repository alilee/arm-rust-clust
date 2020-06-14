// SPDX-License-Identifier: Unlicense

//! Capture and log debugging output.

pub mod logger;

/// Buffer to match logging indent.
pub const BUFFER: &str = "                                                               ";

/// Decreasingly verbose levels of debug logging.
///
/// NOTE: Widened to five characters for display.
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub enum Level {
    /// Log everything.
    Trace,
    /// Detailed understanding of events.
    Debug,
    /// Notable milestone..
    Info,
    /// Unusual conditions.
    Warn,
    /// Significant milestones.
    Major,
    /// Something went wrong.
    Error,
    /// Can't continue.
    Fatal,
}

impl Into<&str> for Level {
    fn into(self) -> &'static str {
        match self {
            Self::Trace => "TRACE",
            Self::Debug => "DEBUG",
            Self::Info => "INFO",
            Self::Warn => "WARN",
            Self::Major => "MAJOR",
            Self::Error => "ERROR",
            Self::Fatal => "FATAL",
        }
    }
}

/// True if logging is enabled for this module at this level.
#[macro_export]
macro_rules! log_enabled {
    ($lvl:expr) => {
        $crate::debug::logger::_is_enabled($lvl, module_path!())
    };
}

/// The dbg macro.
#[macro_export]
macro_rules! dbg {
    () => {
        $crate::log!($crate::debug::Level::Debug, "");
    };
    ($val:expr) => {
        match $val {
            tmp => {
                $crate::log!(
                    $crate::debug::Level::Debug,
                    concat!(stringify!($val), " = {:?}"),
                    &tmp
                );
                tmp
            }
        }
    };
}

/// Log, with a newline.
#[macro_export]
macro_rules! log {
    ($lvl:expr, $string:expr) => ({
        if $crate::debug::logger::_is_enabled($lvl, module_path!()) {
            let lvl: &str = $lvl.into();
            $crate::debug::logger::_print(format_args_nl!(
                concat!("{:>5}[{:>50} {:3}]  ", $string),
                lvl,
                module_path!().trim_start_matches("libkernel::").trim_start_matches("archs::"),
                line!(),
            ))
        };
    });
    ($lvl:expr, $format_string:expr, $($arg:tt)*) => ({
        if $crate::debug::logger::_is_enabled($lvl, module_path!()) {
            let lvl: &str = $lvl.into();
            $crate::debug::logger::_print(format_args_nl!(
                concat!("{:>5}[{:>50} {:3}]  ", $format_string),
                lvl,
                module_path!().trim_start_matches("libkernel::").trim_start_matches("archs::"),
                line!(),
                $($arg)*
            ))
        };
    })
}

/// Log an error, with a newline
#[macro_export]
macro_rules! error {
    ($string:expr) => (
        $crate::log!($crate::debug::Level::Error, $string);
    );
    ($format_string:expr, $($arg:tt)*) => (
        $crate::log!($crate::debug::Level::Error, $format_string, $($arg)*);
    )
}

/// Log an info, with a newline
#[macro_export]
macro_rules! info {
    ($string:expr) => (
        $crate::log!($crate::debug::Level::Info, $string);
    );
    ($format_string:expr, $($arg:tt)*) => (
        $crate::log!($crate::debug::Level::Info, $format_string, $($arg)*);
    )
}

/// Log a debug, with a newline
#[macro_export]
macro_rules! debug {
    ($string:expr) => (
        $crate::log!($crate::debug::Level::Debug, $string);
    );
    ($format_string:expr, $($arg:tt)*) => (
        $crate::log!($crate::debug::Level::Debug, $format_string, $($arg)*);
    )
}

/// Log an info, with a newline
#[macro_export]
macro_rules! trace {
    ($string:expr) => (
        $crate::log!($crate::debug::Level::Trace, $string);
    );
    ($format_string:expr, $($arg:tt)*) => (
        $crate::log!($crate::debug::Level::Trace, $format_string, $($arg)*);
    )
}

#[cfg(test)]
mod tests {
    #[test]
    fn logging() {
        dbg!();
        dbg!(1);
        log!(super::Level::Major, "{}", 1);
        error!("error");
        info!("info");
        debug!("debug");
        trace!("trace");
    }
}
