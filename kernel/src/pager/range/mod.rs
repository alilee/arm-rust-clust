pub mod attrs;
pub mod layout;
mod page_bump;

use super::Page;

use crate::util::locked::Locked;

use log::{debug, info};

use page_bump::PageBumpAllocator;

pub struct PageRange(*const Page, *const Page);

impl PageRange {
    pub fn new(range: (*const Page, *const Page)) -> Self {
        PageRange(range.0, range.1)
    }
    pub fn base(&self) -> *const Page {
        self.0
    }
    pub fn top(&self) -> *const Page {
        self.1
    }
    pub fn length_in_pages(&self) -> usize {
        unsafe { self.1.offset_from(self.0) as usize }
    }
}

static DEVICE_PAGE_ALLOC: Locked<PageBumpAllocator> = Locked::new(PageBumpAllocator::new());

pub fn init() -> Result<(), u64> {
    let mut lock = DEVICE_PAGE_ALLOC.lock();
    lock.reset(layout::device());
    debug!("DEVICE_PAGE_ALLOC {:?}", *lock);
    Ok(())
}

pub fn device(pages: usize) -> Result<*const Page, u64> {
    info!("device {} pages", pages);
    let mut lock = DEVICE_PAGE_ALLOC.lock();
    let result = lock.alloc(pages)? as *const Page;
    debug!("DEVICE_PAGE_ALLOC {:?}", *lock);
    Ok(result)
}
