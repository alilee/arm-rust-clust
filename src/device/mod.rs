// SPDX-License-Identifier: Unlicense

//! A module for devices.

pub mod uart;

use crate::Result;

/// Initialise the device subsystem.
///
/// Discover and register available devices by iterating through device drivers.
pub fn init() -> Result<()> {
    info!("init");
    // eg. timer
    Ok(())
}
