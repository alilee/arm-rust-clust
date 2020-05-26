// SPDX-License-Identifier: Unlicense

use crate::util::locked::Locked;
use crate::{Error, Result};

use super::{PhysAddr, PhysAddrRange, Translate, PAGESIZE_BYTES};

use core::fmt::{Debug, Formatter};

/// Ability to provide an unused frame.
pub trait Allocator {
    /// Reserve and return a zero'd frame.
    fn alloc_page(&mut self, mem_access_translation: &impl Translate) -> Result<PhysAddr>;
}

/// Initialise
pub fn init() -> Result<()> {
    log!("MAJOR", "init");
    Ok(())
}

/// Extend the frame table to include a range of physical addresses.
pub fn add_ram_range(range: PhysAddrRange, mem_offset: &impl Translate) -> Result<()> {
    info!(
        "including: {:?} ({} pages)",
        range,
        range.length() / PAGESIZE_BYTES
    );
    assert!(range.aligned(PAGESIZE_BYTES));
    let mut alloc = ALLOCATOR.lock();
    for phys_addr in range.chunks(PAGESIZE_BYTES) {
        alloc.push(phys_addr, mem_offset)?;
    }
    debug!("{:?}", alloc);
    Ok(())
}

/// O(1) allocator of available pages
pub struct StackAllocator {
    top: Option<PhysAddr>,
    count: usize,
}

impl StackAllocator {
    const fn new() -> Self {
        Self {
            top: None,
            count: 0,
        }
    }

    #[cfg(test)]
    fn reset(&mut self) {
        self.top = None;
        self.count = 0;
    }

    fn push(&mut self, phys_addr: PhysAddr, mem_offset: &impl Translate) -> Result<()> {
        let top: *mut Option<PhysAddr> = mem_offset.translate_phys(phys_addr).unwrap().into();
        unsafe {
            *top = self.top;
            self.top = Some(phys_addr);
        };
        self.count += 1;
        Ok(())
    }
}

impl Debug for StackAllocator {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "StackAllocator(top: {:?}, count: {})",
            self.top, self.count
        )
    }
}

impl Allocator for StackAllocator {
    fn alloc_page(&mut self, mem_offset: &impl Translate) -> Result<PhysAddr> {
        let phys_addr = self.top.ok_or(Error::OutOfMemory)?;
        unsafe {
            let top: *mut Option<PhysAddr> = mem_offset.translate_phys(phys_addr).unwrap().into();
            self.top = *top;
            *top = None;
        };
        self.count -= 1;
        debug!(
            "Allocating {:?} - {} pages remaining",
            phys_addr, self.count
        );
        trace!("{:?}", self);
        Ok(phys_addr)
    }
}

pub static ALLOCATOR: Locked<StackAllocator> = Locked::new(StackAllocator::new());

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pager::{Identity, Page};

    #[test]
    fn empty() {
        let mut alloc = ALLOCATOR.lock();
        alloc.reset();
        assert_err!(alloc.alloc_page(&Identity::new()));
    }

    #[test]
    fn extending() {
        let mem_offset = &Identity::new();
        static mut PAGES: [Page; 3] = [Page::new(); 3];
        let mock_ram_range = unsafe {
            PhysAddrRange::between(
                PhysAddr::at(&PAGES[0] as *const Page as usize),
                PhysAddr::at(&PAGES[2] as *const Page as usize),
            )
        };
        {
            ALLOCATOR.lock().reset();
        }
        assert_ok!(add_ram_range(mock_ram_range, mem_offset));
        {
            let mut alloc = ALLOCATOR.lock();
            assert_ok!(alloc.alloc_page(mem_offset));
            assert_ok!(alloc.alloc_page(mem_offset));
            assert_err!(alloc.alloc_page(mem_offset));
        }
    }
}
