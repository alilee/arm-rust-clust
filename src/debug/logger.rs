// SPDX-License-Identifier: Unlicense

//! Debug logging to serial available from kernel_init

use crate::pager::Translate;
use crate::Result;

#[cfg(not(test))]
use crate::device::uart::Uart;
#[cfg(not(test))]
use crate::util::locked::Locked;

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

/// True iff logs at level should be displayed for logging from the module_path.
#[cfg(not(test))]
pub fn _is_enabled(level: &str, module_path: &str) -> bool {
    fn ord(level: &str) -> u8 {
        match level {
            "CRIT" => 6u8,
            "ERROR" => 5u8,
            "MAJOR" => 4u8,
            "WARN" => 3u8,
            "INFO" => 2u8,
            "DEBUG" => 1u8,
            "TRACE" => 0u8,
            _ => 255u8,
        }
    }

    let ord_level = ord(level);

    extern "Rust" {
        #[no_mangle]
        static LOG_LEVEL_SETTINGS: &'static [(&'static str, &'static str)];
    }

    let setting = unsafe {
        LOG_LEVEL_SETTINGS.into_iter().fold(0, |base, (pat, level)| {
            if module_path.ends_with(pat) {
                ord(level)
            } else {
                base
            }
        })
    };
    ord_level >= setting
}

#[cfg(test)]
pub fn _print(args: Arguments) {
    extern crate std;
    use std::print;
    print!("{}", args);
}

#[cfg(test)]
pub fn _is_enabled(_lvl: &str, _module_path: &str) -> bool {
    true
}

/// Move the debug UART address (after enabling paging)
#[cfg_attr(test, allow(unused_variables))]
pub fn offset(remap: impl Translate) -> Result<()> {
    #[cfg(not(test))]
    {
        let mut log = LOGGER.lock();
        unsafe {
            log.translate(remap)?;
        }

        use crate::info;
        info!("remapped debug UART successfully");
    }
    Ok(())
}
