pub mod frames;
mod phys_addr;

use crate::arch;
use log::info;
pub use phys_addr::{PhysAddr, PhysAddrRange};

pub const PAGESIZE_BYTES: usize = 4096;

/// Initialise the system by initialising the submodules and mapping initial memory contents.
pub fn init(boot3: fn() -> !) -> ! {
    info!("init");
    let ram = arch::pager::init().unwrap();
    frames::reset(ram).unwrap();
    arch::pager::enable(boot3)
}

/// Map the physical address range from the given virtual base address
pub fn _range_map(_range: PhysAddrRange, _base: *const ()) -> Result<(), u64> {
    todo!();
}
