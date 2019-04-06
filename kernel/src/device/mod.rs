//! A module for devices.

use log::info;

pub mod uart;
pub mod timer;


pub fn init() -> () {
    info!("initialising");
    // eg. timer
}
