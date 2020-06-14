// SPDX-License-Identifier: Unlicense

//! Debug logging to serial available from kernel_init

#[allow(unused_imports)]
use super::Level;

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
pub fn _is_enabled(level: Level, module_path: &str) -> bool {

    let setting = LOG_LEVEL_SETTINGS
        .into_iter()
        .fold(Level::Trace, |base, (pat, level)| {
            if module_path.ends_with(pat) {
                *level
            } else {
                base
            }
        });
    level >= setting
}

#[cfg(test)]
pub fn _print(args: Arguments) {
    extern crate std;
    use std::print;
    print!("{}", args);
}

#[cfg(test)]
pub fn _is_enabled(_lvl: Level, _module_path: &str) -> bool {
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

/// Log message filtering settings by module.
///
/// Referenced in `_is_enabled`.
#[cfg(not(test))]
const LOG_LEVEL_SETTINGS: &[(&str, Level)] = &[("aarch64::pager", Level::Debug)];