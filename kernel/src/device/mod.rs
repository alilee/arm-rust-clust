//! A module for devices.

use log::info;

pub mod uart;

pub mod ram {
    use crate::pager::{PhysAddr, PhysAddrRange};

    pub fn range() -> PhysAddrRange {
        PhysAddrRange::new(PhysAddr::new(0x40000000), 0x10000000)
    }
}

pub fn init() -> () {
    info!("init");
    // eg. timer
}
