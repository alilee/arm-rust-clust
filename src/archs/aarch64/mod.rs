// SPDX-License-Identifier: Unlicense

/// Mock hardware abstraction layer for unit tests.
#[cfg(test)]
pub mod hal_test;

/// Publish hardware abstraction layer for unit tests.
#[cfg(test)]
pub use hal_test as hal;

/// Live hardware abstraction layer for integration tests and releases.
#[cfg(not(test))]
pub mod hal_live;

/// Publish hardware abstraction layer for integration tests and releases.
#[cfg(not(test))]
pub use hal_live as hal;

use crate::pager::{FrameAllocator, VirtAddr};
use crate::Result;

/// Materialise empty struct implementating Arch trait.
pub struct Arch {}

impl super::ArchTrait for Arch {
    fn ram_range() -> Result<PhysAddrRange> {
        unimplemented!()
    }
    fn kernel_base() -> VirtAddr {
        unimplemented!()
    }

    fn pager_init() -> Result<()> {
        unimplemented!()
    }

    fn map_translation(
        _phys_range: PhysAddrRange,
        _virtual_address_translation: impl Translate,
        _attrs: Attributes,
        _allocator: &Locked<impl FrameAllocator>,
        _mem_access_translation: impl Translate,
    ) {
        unimplemented!()
    }

    fn map_demand(
        _virtual_range: VirtAddrRange,
        _attrs: Attributes,
        _allocator: &Locked<impl FrameAllocator>,
        _mem_access_translation: impl Translate,
    ) {
        unimplemented!()
    }

    fn enable_paging() {
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

use crate::pager::{Attributes, PhysAddrRange, Translate, VirtAddrRange};
use crate::util::locked::Locked;
#[cfg(not(test))]
pub use hal::_reset;

#[cfg(test)]
mod tests {
    extern crate std;

    #[test]
    fn it_works() {
        info!("marker")
    }
}
