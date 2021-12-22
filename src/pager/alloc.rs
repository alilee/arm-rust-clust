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
    error!("alloc_error_handler");
    panic!("allocation error: {:?}", layout)
}

#[inline(never)]
#[no_mangle]
fn smol() -> () {
    info!("sandwich");
}

/// Initialise the heap.
///
/// Note: Memory must be accessible.
#[cfg(not(test))]
pub fn init(heap_range: VirtAddrRange) -> Result<()> {
    // info!("init");
    unsafe {
        let mut lock = ALLOCATOR.lock();
        // debug!("locked");
        // let x = heap_range.base().get();
        // let y = heap_range.length();
        // debug!("locked 2 {:?} {:?}", x, y);
        // smol();
        lock.init(heap_range.base().get(), heap_range.length());
        // debug!("init'd");
    }
    Ok(())
}

#[cfg(test)]
pub fn init(_heap_range: VirtAddrRange) -> Result<()> {
    Ok(())
}
