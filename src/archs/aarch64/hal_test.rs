// SPDX-License-Identifier: Unlicense

use crate::pager::VirtAddr;
use crate::Result;

pub fn init_mair() {}

pub fn enable_paging(_: u64, _: u64, _: u16) -> Result<()> {
    unimplemented!()
}

pub fn move_stack(_: usize, _next: fn() -> !) -> ! {
    unimplemented!()
}

pub fn set_vbar() -> Result<()> {
    unimplemented!()
}

pub fn core_id() -> u8 {
    1
}

pub fn invalidate_tlb(_virt_addr: VirtAddr) -> Result<()> {
    unimplemented!()
}
