//! A module

pub mod uart_logger;

/// The dbg macro.
#[macro_export]
macro_rules! dbg {
    () => {
        crate::device::uart::_dbg_writer(format_args!("[{}:{}]\n", file!(), line!()));
    };
    ($val:expr) => {
        // Use of `match` here is intentional because it affects the lifetimes
        // of temporaries - https://stackoverflow.com/a/48732525/1063961
        match $val {
            tmp => {
                crate::device::uart::_dbg_writer(format_args!("[{}:{}] {} = {:#?}\n",
                    file!(), line!(), stringify!($val), &tmp));
                tmp
            }
        }
    };
    // Trailing comma with single argument is ignored
    ($val:expr,) => { dbg!($val) };
    ($($val:expr),+ $(,)?) => {
        ($($crate::debug::dbg!($val)),+,)
    };
}
