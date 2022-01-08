// SPDX-License-Identifier: Unlicense

//! A module for kernel devices.

pub mod gic;
pub mod uart;
pub mod virtio;

use crate::pager::PhysAddr;
use crate::util::locked::Locked;
use crate::Result;
use alloc::{boxed::Box, collections::BTreeMap, string::String};

/// Initialise the device subsystem.
///
/// Discover and register available devices by iterating through device drivers.
pub fn init() -> Result<()> {
    info!("init");

    gic::init()?;
    virtio::init()
}

trait Block: Send {
    fn name(&self) -> String;
    fn read(&self, phys_addr: PhysAddr, sector: u64, length: usize) -> Result<()>;
    fn write(&self, phys_addr: PhysAddr, sector: u64, length: usize) -> Result<()>;
    fn discard(&self, phys_addr: PhysAddr, sector: u64, length: usize) -> Result<()>;
    fn zero(&self, phys_addr: PhysAddr, sector: u64, length: usize) -> Result<()>;
    fn flush(&self) -> Result<()>;
}

static BLOCK_DEVICES: Locked<BTreeMap<String, Box<dyn Block>>> = Locked::new(BTreeMap::new());

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn can_call_on_test_arch() {
        // init().expect("init");
    }
}
