pub mod attrs;
pub mod layout;
mod page_bump;

use crate::pager::virt_addr::VirtAddrRange;
use crate::util::locked::Locked;

use log::{debug, info};

pub use page_bump::PageBumpAllocator;

static DEVICE_PAGE_ALLOC: Locked<PageBumpAllocator> = Locked::new(PageBumpAllocator::new());
static MEM_PAGE_ALLOC: Locked<PageBumpAllocator> = Locked::new(PageBumpAllocator::new());

pub fn init() -> Result<(), u64> {
    {
        let mut lock = DEVICE_PAGE_ALLOC.lock();
        lock.reset(layout::device());
        debug!("DEVICE_PAGE_ALLOC {:?}", *lock);
    }
    {
        let mut lock = MEM_PAGE_ALLOC.lock();
        lock.reset(layout::page_pool());
        debug!("MEM_PAGE_ALLOC {:?}", *lock);
    }
    Ok(())
}

pub fn alloc_device(pages: usize) -> Result<VirtAddrRange, u64> {
    info!("alloc_device {} pages", pages);
    let mut lock = DEVICE_PAGE_ALLOC.lock();
    let result = lock.alloc(pages)?;
    debug!(
        "DEVICE_PAGE_ALLOC alloc'ed {:?} leaving {:?}",
        result, *lock
    );
    Ok(result)
}

pub fn alloc_pool(pages: usize) -> Result<VirtAddrRange, u64> {
    info!("alloc_mem {} pages", pages);
    let mut lock = MEM_PAGE_ALLOC.lock();
    let result = lock.alloc(pages)?;
    debug!("MEM_PAGE_ALLOC alloc'ed {:?} leaving {:?}", result, *lock);
    Ok(result)
}
