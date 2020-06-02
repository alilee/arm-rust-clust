// SPDX-License-Identifier: Unlicense

//! Capture and log debugging output.

pub mod logger;

/// Buffer to match logging indent.
pub const BUFFER: &str = "                                                               ";

/// The dbg macro.
#[macro_export]
macro_rules! dbg {
    () => {
        $crate::log!("DEBUG", "");
    };
    ($val:expr) => {
        match $val {
            tmp => {
                $crate::log!("DEBUG", concat!(stringify!($val), " = {:?}"), &tmp);
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
            $crate::debug::logger::_print(format_args_nl!(
                concat!("{:5}[{:>50} {:3}]  ", $string),
                $lvl,
                module_path!().trim_start_matches("libkernel::").trim_start_matches("archs::"),
                line!(),
            ))
        };
    });
    ($lvl:expr, $format_string:expr, $($arg:tt)*) => ({
        if $crate::debug::logger::_is_enabled($lvl, module_path!()) {
            $crate::debug::logger::_print(format_args_nl!(
                concat!("{:5}[{:>50} {:3}]  ", $format_string),
                $lvl,
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
        $crate::log!("ERROR", $string);
    );
    ($format_string:expr, $($arg:tt)*) => (
        $crate::log!("ERROR", $format_string, $($arg)*);
    )
}

/// Log an info, with a newline
#[macro_export]
macro_rules! info {
    ($string:expr) => (
        $crate::log!("INFO", $string);
    );
    ($format_string:expr, $($arg:tt)*) => (
        $crate::log!("INFO", $format_string, $($arg)*);
    )
}

/// Log a debug, with a newline
#[macro_export]
macro_rules! debug {
    ($string:expr) => (
        $crate::log!("DEBUG", $string);
    );
    ($format_string:expr, $($arg:tt)*) => (
        $crate::log!("DEBUG", $format_string, $($arg)*);
    )
}

/// Log an info, with a newline
#[macro_export]
macro_rules! trace {
    ($string:expr) => (
        $crate::log!("TRACE", $string);
    );
    ($format_string:expr, $($arg:tt)*) => (
        $crate::log!("TRACE", $format_string, $($arg)*);
    )
}

/// Log message filtering settings by module.
///
/// These are the defaults unless overridden in main or integration test.
/// Referenced in `debug::logger::_is_enabled`.
#[cfg(not(test))]
#[no_mangle]
#[linkage = "weak"]
static LOG_LEVEL_SETTINGS: &[(&str, &str)] = &[];

#[cfg(test)]
mod tests {
    #[test]
    fn logging() {
        dbg!();
        dbg!(1);
        log!("MAJOR", "{}", 1);
        error!("error");
        info!("info");
        debug!("debug");
        trace!("trace");
    }
}