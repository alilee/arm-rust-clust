// SPDX-License-Identifier: Unlicense

mod pager;

/// Live hardware abstraction layer for integration tests and releases.
#[cfg(not(test))]
mod hal;

/// Mock hardware abstraction layer for unit tests.
#[cfg(test)]
mod hal_test;

/// Publish hardware abstraction layer for unit tests.
#[cfg(test)]
use hal_test as hal;

use crate::pager::{Addr, AddrRange, PhysAddr, PhysAddrRange, VirtAddr};
use crate::Result;

/// Materialise empty struct implementing Arch trait.
pub struct Arch {}

impl super::ArchTrait for Arch {
    fn ram_range() -> Result<PhysAddrRange> {
        // FIXME: Collect from DTB
        Ok(PhysAddrRange::between(
            PhysAddr::at(0x4000_0000),
            PhysAddr::at(0x5000_0000),
        ))
    }
    fn kernel_base() -> VirtAddr {
        const UPPER_VA_BITS: usize = 39;
        let result = VirtAddr::at(!((1 << (UPPER_VA_BITS + 1)) - 1));
        result
    }

    fn pager_init() -> Result<()> {
        pager::init()
    }

    fn enable_paging(_page_directory: &impl super::PageDirectory) {
        unimplemented!()
    }

    fn handler_init() -> Result<()> {
        unimplemented!()
    }

    fn thread_init() -> Result<()> {
        unimplemented!()
    }

    fn wait_forever() -> ! {
        unimplemented!()
    }
}

#[cfg(not(test))]
pub use hal::_reset;

/// Construct an empty page directory.
pub fn new_page_directory() -> impl super::PageDirectory {
    pager::new_page_directory()
}

#[cfg(test)]
mod tests {
    extern crate std;

    #[test]
    fn it_works() {
        info!("marker")
    }
}
