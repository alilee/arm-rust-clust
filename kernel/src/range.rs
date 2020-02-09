/// Keeps track of which ranges of cluster address space have been allocated.
///
/// Chicken and egg: does capability minting needs to check this?
///
/// This needs to be distributed. Each node would have a section of the address
/// space it can issue reservations for without coordinating.
/// TODO: Distributed approach to range allocation.
use crate::pager::{attrs::Attributes, layout, virt_addr::VirtAddrRange, Page};
use crate::thread::cap::Capability;
use crate::util::locked::Locked;
use crate::util::page_bump::PageBumpAllocator;

static RANGES: Locked<PageBumpAllocator> = Locked::new(PageBumpAllocator::new());

/// Identify unused section of cluster memory and create capability
pub fn reserve(pages: usize, guard_page: bool, attributes: Attributes) -> Result<Capability, u64> {
    let mut lock = RANGES.lock();
    let virt_range = lock.alloc(pages)?;
    Ok(Capability::AddrRange(
        virt_range,
        Capability::AddrRangeRights::from(attributes),
    ))
}

pub fn null() -> *const Page {
    use core::ptr;
    ptr::null() as *const Page
}

pub fn init() -> Result<(), u64> {
    info!("init");
    RANGES.lock().reset(layout::user_space());
    Ok(())
}
