// SPDX-License-Identifier: Unlicense

//! A kernel heap.

use crate::pager::layout::RangeContent;
use crate::pager::{Addr, AddrRange};
use crate::Result;

use linked_list_allocator::LockedHeap;

/// Allocator for kernel heap. Must be initialised.
#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

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
    major!("init");

    let heap_range = super::layout::get_range(RangeContent::KernelHeap)?;
    info!("heap_range: {:?}", heap_range);

    unsafe {
        let mut lock = ALLOCATOR.lock();
        lock.init(heap_range.base().get(), heap_range.length());
        info!(
            "Kernel Heap: 0x{:x}...0x{:x}, used: 0x{:x}, free: 0x{:x}",
            lock.bottom(),
            lock.top(),
            lock.used(),
            lock.free()
        );
    }

    Ok(())
}

#[cfg(test)]
pub fn init() -> Result<()> {
    Ok(())
}
