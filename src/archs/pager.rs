// SPDX-License-Identifier: Unlicense

//! Interface for paging functions.

use crate::pager::{Attributes, FrameAllocator, PhysAddrRange, Translate, VirtAddrRange};
use crate::util::locked::Locked;

/// Methods to maintain a directory of virtual to physical addresses.
pub trait PageDirectory {
    /// Map physical address range at offset
    fn map_translation(
        &mut self,
        phys_range: PhysAddrRange,
        virtual_address_translation: impl Translate,
        attrs: Attributes,
        allocator: &Locked<impl FrameAllocator>,
        mem_access_translation: &impl Translate,
    );
    /// Map physical address range at offset
    fn map_demand(
        &mut self,
        virtual_range: VirtAddrRange,
        attrs: Attributes,
        allocator: &Locked<impl FrameAllocator>,
        mem_access_translation: &impl Translate,
    );
}
