// SPDX-License-Identifier: Unlicense

//! Debug logging to serial available from kernel_init

use crate::Result;
use crate::device::uart::Uart;
use crate::util::locked::Locked;
use crate::pager::Translate;

use core::fmt::Arguments;

#[cfg(not(test))]
static LOGGER: Locked<Uart> = Locked::new(Uart::debug());

/// Print debug output to the debug Uart
#[cfg(not(test))]
pub fn _print(args: Arguments) {
    use core::fmt::Write;

    let mut log = LOGGER.lock();
    log.write_fmt(args).expect("write_fmt");
}

#[cfg(test)]
pub fn _print(args: Arguments) {
    extern crate std;
    use std::print;
    print!("{}", args);
}

/// Move the debug UART address (after enabling paging)
#[cfg(not(test))]
pub fn offset(remap: impl Translate) -> Result<()> {
    {
        let mut log = LOGGER.lock();
        unsafe { log.translate(remap)?; }
    }

    use crate::info;

    info!("remapped debug UART successfully");
    Ok(())
}