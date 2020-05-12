// SPDX-License-Identifier: Unlicense

use crate::pager::{Attributes, FrameAllocator, PhysAddrRange, Translate, VirtAddr, VirtAddrRange};
use crate::util::locked::Locked;
use crate::Result;

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
}

#[cfg(not(test))]
#[no_mangle]
pub unsafe extern "C" fn _reset() -> ! {
    crate::kernel_init()
}
