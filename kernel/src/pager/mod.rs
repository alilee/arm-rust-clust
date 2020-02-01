pub mod frames;
mod phys_addr;
pub mod range;

use crate::arch;
use crate::device;
use core::mem;
use log::info;
pub use phys_addr::{PhysAddr, PhysAddrRange};

pub const PAGESIZE_BYTES: usize = 4096;

#[repr(align(4096))]
pub struct Page([u8; PAGESIZE_BYTES]);

/// Initialise the system by initialising the submodules and mapping initial memory contents.
pub fn init(boot3: fn() -> !) -> ! {
    info!("init");
    arch::pager::init().unwrap();
    range::init().unwrap();
    let ram = device::ram::range();
    frames::reset(ram).unwrap();
    arch::pager::enable(boot3)
}

pub fn device_map<T>(base: PhysAddr) -> Result<*mut T, u64> {
    let length = mem::size_of::<T>();
    let device_addr = arch::pager::device_map(PhysAddrRange::new(base, length))?;
    Ok(device_addr as *mut T)
}
