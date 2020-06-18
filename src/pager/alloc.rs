// SPDX-License-Identifier: Unlicense

//! A kernel heap.

use crate::pager::{Addr, AddrRange, VirtAddrRange};
use crate::Result;

use linked_list_allocator::LockedHeap;

/// Allocator for kernel heap. Must be initialised.
#[global_allocator]
pub static ALLOCATOR: LockedHeap = LockedHeap::empty();

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}

/// Initialise the heap.
///
/// Note: Memory must be accessible.
#[cfg(not(test))]
pub fn init(heap_range: VirtAddrRange) -> Result<()> {
    info!("init");
    unsafe {
        let mut lock = ALLOCATOR.lock();
        lock.init(heap_range.base().get(), heap_range.length());
    }
    Ok(())
}

#[cfg(test)]
pub fn init(_heap_range: VirtAddrRange) -> Result<()> {
    Ok(())
}
