/// Kernel heap
///
///
///
use crate::arch;

use alloc::alloc::{GlobalAlloc, Layout};
use core::panic;
use core::ptr::null_mut;
use linked_list_allocator::LockedHeap;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}

pub fn init() -> Result<(), u64> {
    use crate::pager::{attrs, layout};

    let heap_range = layout::heap();
    let heap_range = arch::pager::provisional_map(
        heap_range,
        attrs::kernel_read_write(),
        layout::kernel_mem_offset(),
    )?;

    unsafe {
        ALLOCATOR.lock().init(heap_range.base(), heap_range.top());
    }

    Ok(())
}
