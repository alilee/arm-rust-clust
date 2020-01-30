use super::virt_addr::{VirtAddr, VirtAddrRange, VirtOffset};
use crate::pager::PAGESIZE_BYTES;

pub struct PageBump {
    range: VirtAddrRange,
    base: VirtAddr,
}

impl PageBump {
    pub const fn new(range: VirtAddrRange) -> Self {
        Self {
            range,
            base: range.top(),
        }
    }

    pub fn alloc(&mut self, pages: usize) -> Result<VirtAddr, u64> {
        let span = VirtOffset::new(pages * PAGESIZE_BYTES);
        let new_base = span.decrement(self.base);
        if new_base < self.range.base {
            Err(0)
        } else {
            self.base = new_base;
            Ok(self.base)
        }
    }
}
