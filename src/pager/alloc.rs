// SPDX-License-Identifier: Unlicense

//! Allocators to parcel up chunks of kernel memory.

use crate::pager::VirtAddrRange;
use crate::Result;

pub fn _add_heap_range(_virt_addr_range: VirtAddrRange) -> Result<()> {
    unimplemented!()
}
