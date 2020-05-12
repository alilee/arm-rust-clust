// SPDX-License-Identifier: Unlicense

/// Mock hardware abstraction layer for unit tests.
#[cfg(test)]
pub mod hal_test;

/// Publish hardware abstration layer for unit tests.
#[cfg(test)]
pub use hal_test as hal;

/// Live hardware abstraction layer for integration tests and releases.
#[cfg(not(test))]
pub mod hal_live;

/// Publish hardware abstration layer for integration tests and releases.
#[cfg(not(test))]
pub use hal_live as hal;

use crate::Result;
use crate::pager::{VirtAddr, FrameAllocator};

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

    fn map_translation(_phys_range: PhysAddrRange, _virtual_address_translation: impl Translate, _attrs: Attributes, _allocator: &Locked<impl FrameAllocator>, _mem_access_translation: impl Translate) {
        unimplemented!()
    }

    fn map_demand(_virtual_range: VirtAddrRange, _attrs: Attributes, _allocator: &Locked<impl FrameAllocator>, _mem_access_translation: impl Translate) {
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
}

#[cfg(not(test))]
pub use hal::_reset;
use crate::util::locked::Locked;
use crate::pager::{PhysAddrRange, Attributes, VirtAddrRange, Translate};

#[cfg(test)]
mod tests {
    extern crate std;
    use std::dbg;

    #[test]
    fn sandwich() {
        dbg!("hello");
        assert!(true)
    }
}