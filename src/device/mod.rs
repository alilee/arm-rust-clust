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

use alloc::{boxed::Box, collections::BTreeMap, string::String, sync::Arc};

use dtb::{StructItem, StructItems};

/// Pointer to Device Tree Blob in physical memory, if available.
///
/// Set during reset, before memory is overwritten, so that pager can reserve and map.
pub static mut PDTB: Option<PhysAddrRange> = None;

/// Get physical memory location from direct-mapped physical DTB address (to bootstrap paging)
///
/// Unsafety: This function must only be called while physical memory is identity-mapped.
pub unsafe fn get_ram_range_early() -> Result<PhysAddrRange> {
    let dtb_addr = PDTB.ok_or(Error::UnInitialised)?.base();
    let reader =
        dtb::Reader::read_from_address(dtb_addr.get()).or(Err(Error::DeviceIncompatible))?;
    let dtb_root = reader.struct_items();
    let (prop, _) = dtb_root.path_struct_items("/memory/reg").next().unwrap();
    let phys_addr_range = make_addr_range(prop)?;
    Ok(phys_addr_range)
}

fn make_addr_range(prop: StructItem) -> Result<PhysAddrRange> {
    let mut buf = [0u8; 32];
    let list = prop
        .value_u32_list(&mut buf)
        .or(Err(Error::DeviceIncompatible))?;
    Ok(PhysAddrRange::new(
        PhysAddr::fixed((list[0] as usize) << 32 | (list[1] as usize)),
        (list[2] as usize) << 32 | (list[3] as usize),
    ))
}

#[derive(Debug)]
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

#[derive(Copy, Clone, Debug)]
#[repr(u8)]
pub enum RequestStatus {
    Ok = 0,
    IOErr = 1,
    Unsupp = 2,
    Init = !0,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct RequestId(u16);

#[derive(Copy, Clone, Debug)]
pub struct Sector(pub u64);

/// Functions for a block storage device
pub trait Block {
    fn name(&self) -> String;
    fn status(&mut self, id: RequestId) -> Result<u32>;
    fn read(&mut self, page_addrs: &[PhysAddr], sector: Sector) -> Result<RequestId>;
    fn write(&mut self, page_addrs: &[PhysAddr], sector: Sector) -> Result<RequestId>;
    fn discard(&mut self, sector: Sector, pages: usize) -> Result<RequestId>;
    fn zero(&mut self, sector: Sector, pages: usize) -> Result<RequestId>;
    fn flush(&mut self) -> Result<RequestId>;
}

pub static BLOCK_DEVICES: Locked<BTreeMap<String, Arc<Locked<Box<dyn Block + Send>>>>> =
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
