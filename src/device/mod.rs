// SPDX-License-Identifier: Unlicense

//! A module for devices.

pub mod uart;

/// Initialise the device subsystem.
///
/// Discover and register available devices by iterating through device drivers.
pub fn init() -> () {
    info!("init");
    // eg. timer
}
