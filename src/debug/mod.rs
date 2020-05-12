// SPDX-License-Identifier: Unlicense

//! A module

pub mod logger;

/// The dbg macro.
#[macro_export]
macro_rules! dbg {
    () => {
        $crate::debug::logger::_print(format_args_nl!(
            "DEBUG[{:>50} {:3}]  dbg!()",
            module_path!(),
            line!(),
        ));
    };
    ($val:expr) => {
        // Use of `match` here is intentional because it affects the lifetimes
        // of temporaries - https://stackoverflow.com/a/48732525/1063961
        match $val {
            tmp => {
                $crate::debug::logger::_print(format_args_nl!(
                    "DEBUG[{:>50} {:3}]  {} = {:#?}",
                    module_path!(),
                    line!(),
                    stringify!($val),
                    &tmp
                ));
                tmp
            }
        }
    };
}

/// Log, with a newline.
#[macro_export]
macro_rules! log {
    ($lvl:expr, $string:expr) => ({
        $crate::debug::logger::_print(format_args_nl!(
            concat!("{:5}[{:>50} {:3}]  ", $string),
            $lvl,
            module_path!(),
            line!(),
        ));
    });
    ($lvl:expr, $format_string:expr, $($arg:tt)*) => ({
        $crate::debug::logger::_print(format_args_nl!(
            concat!("{:5}[{:>50} {:3}]  ", $format_string),
            $lvl,
            module_path!(),
            line!(),
            $($arg)*
        ));
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
