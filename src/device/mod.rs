// SPDX-License-Identifier: Unlicense

//! A module for the kernel devices which must be used from the kernel.
//!
//! This is a very short list - interrupt controller, VM swap disk, and
//! debug console. Other devices are managed through user-level threads
//! and by receiving the relevant capabilities which allow them to access
//! the necessary system resources - chiefly physical memory and interrupts.

pub mod intc;
pub mod serial;
pub mod virtio;

use crate::archs::arch::Arch;
use crate::archs::DeviceTrait;
use crate::pager::{
    get_range, Addr, AddrRange, HandlerReturnAction, PhysAddr, PhysAddrRange, RangeContent,
};
use crate::util::locked::Locked;
use crate::{Error, Result};

use alloc::{boxed::Box, collections::BTreeMap, string::String};
use dtb::StructItems;

/// Pointer to Device Tree Blob in physical memory, if available.
///
/// Set during reset, before memory is overwritten, so that pager can reserve and map.
pub static mut PDTB: Option<PhysAddrRange> = None;

#[derive(Copy, Clone, Debug)]
enum DeviceTypes {
    Unknown,
    Block,
}

/// Initialise the device subsystem.
///
/// Discover and register available devices by iterating through device drivers.
pub fn init() -> Result<()> {
    major!("init");

    let dtb_root = get_dtb_root()?;

    Arch::device_init(dtb_root.clone())?;

    virtio::init(dtb_root)
}

fn get_dtb_root() -> Result<StructItems<'static>> {
    let virt_addr = get_range(RangeContent::DTB)?.base();
    let reader = unsafe {
        dtb::Reader::read_from_address(virt_addr.get()).or(Err(Error::DeviceIncompatible))?
    };
    Ok(reader.struct_items())
}

pub trait InterruptController: Send {
    fn add_handler(&mut self, interrupt: u8, handler: fn() -> HandlerReturnAction) -> Result<()>;
}

/// Functions for a block storage device
pub trait Block {
    fn name(&self) -> String;
    fn read(&self, phys_addr: PhysAddr, sector: u64, length: usize) -> Result<()>;
    fn write(&self, phys_addr: PhysAddr, sector: u64, length: usize) -> Result<()>;
    fn discard(&self, phys_addr: PhysAddr, sector: u64, length: usize) -> Result<()>;
    fn zero(&self, phys_addr: PhysAddr, sector: u64, length: usize) -> Result<()>;
    fn flush(&self) -> Result<()>;
}

static BLOCK_DEVICES: Locked<BTreeMap<String, Locked<Box<dyn Block + Send>>>> =
    Locked::new(BTreeMap::new());

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn can_call_on_test_arch() {
        // init().expect("init");
    }
}
