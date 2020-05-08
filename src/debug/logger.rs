// SPDX-License-Identifier: Unlicense

//! Debug logging to serial available from kernel_init

use core::fmt::Arguments;

/// Print debug output to the debug Uart
#[cfg(not(test))]
pub fn _print(args: Arguments) {
    use crate::device::uart::Uart;
    use crate::util::locked::Locked;
    use core::fmt::Write;

    static LOGGER: Locked<Uart> = Locked::new(Uart::debug());

    let mut log = LOGGER.lock();
    log.write_fmt(args).expect("write_fmt");
}

#[cfg(test)]
pub fn _print(args: Arguments) {
    extern crate std;
    use std::print;
    print!("{}", args);
}