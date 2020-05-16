use crate::arch;
use crate::device;
use crate::pager::{
    phys_addr::MemOffset,
    virt_addr::{VirtAddr, VirtAddrRange},
    PAGESIZE_BYTES,
};
use crate::util::{locked::Locked, page_bump::PageBumpAllocator};

use log::{debug, info};

const GB: usize = 1024 * 1024 * 1024;

pub const LAYOUT: [(&str, usize); 5] = [
    ("ram", 4 * GB),
    ("image", 1 * GB),
    ("device", 1 * GB),
    ("pages", 8 * GB),
    ("heap", 8 * GB),
];

fn find(section: &str) -> Result<VirtAddrRange, u64> {
    let mut base: VirtAddr = arch::pager::KERNEL_BASE;
    for (this_section, length) in LAYOUT.iter() {
        let next = unsafe { base.increment(*length) };
        if *this_section == section {
            return Ok(VirtAddrRange::new(base, *length));
        }
        base = next;
    }
    Err(0)
}

pub fn ram() -> VirtAddrRange {
    find("ram").unwrap()
}

pub fn image() -> VirtAddrRange {
    find("image").unwrap()
}

pub fn device() -> VirtAddrRange {
    find("device").unwrap()
}

pub fn page_pool() -> VirtAddrRange {
    find("pages").unwrap()
}

pub fn heap() -> VirtAddrRange {
    find("heap").unwrap()
}

pub fn kernel_mem_offset() -> MemOffset {
    MemOffset::new(device::ram::range().base(), ram().base())
}

pub fn user_space() -> VirtAddrRange {
    VirtAddrRange::between(VirtAddr(PAGESIZE_BYTES), arch::pager::USER_TOP)
}

static DEVICE_PAGE_ALLOC: Locked<PageBumpAllocator> = Locked::new(PageBumpAllocator::new());
static MEM_PAGE_ALLOC: Locked<PageBumpAllocator> = Locked::new(PageBumpAllocator::new());

pub fn init() -> Result<(), u64> {
    {
        let mut lock = DEVICE_PAGE_ALLOC.lock();
        lock.reset(device());
        debug!("DEVICE_PAGE_ALLOC {:?}", *lock);
    }
    {
        let mut lock = MEM_PAGE_ALLOC.lock();
        lock.reset(page_pool());
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
