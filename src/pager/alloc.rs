// SPDX-License-Identifier: Unlicense

//! A kernel heap.

use crate::pager::layout::RangeContent;
use crate::pager::{Addr, AddrRange};
use crate::Result;

use linked_list_allocator::LockedHeap;

/// Allocator for kernel heap. Must be initialised.
#[global_allocator]
pub static ALLOCATOR: LockedHeap = LockedHeap::empty();

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    error!("alloc_error_handler");
    panic!("allocation error: {:?}", layout)
}

/// Initialise the heap.
///
/// Note: Memory must be accessible.
#[cfg(not(test))]
pub fn init() -> Result<()> {
    info!("init");

    let heap_range = super::layout::get_range(RangeContent::KernelHeap)?;

    unsafe {
        let mut lock = ALLOCATOR.lock();
        lock.init(heap_range.base().get(), heap_range.length());
    }
    Ok(())
}

#[cfg(test)]
pub fn init() -> Result<()> {
    Ok(())
}
