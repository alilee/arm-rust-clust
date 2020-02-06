use crate::arch;
use crate::device;
use crate::pager::{
    phys_addr::MemOffset,
    virt_addr::{VirtAddr, VirtAddrRange},
};

const GB: usize = 1024 * 1024 * 1024;

pub const LAYOUT: [(&str, usize); 4] = [
    ("ram", 4 * GB),
    ("image", 1 * GB),
    ("device", 1 * GB),
    ("pages", 8 * GB),
];

fn find(section: &str) -> Result<VirtAddrRange, u64> {
    let mut base: VirtAddr = arch::pager::KERNEL_BASE;
    for (this_section, length) in LAYOUT.iter() {
        let next = base.increment(*length);
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
    find("image").unwrap()
}

pub fn page_pool() -> VirtAddrRange {
    find("pages").unwrap()
}

pub fn kernel_mem_offset() -> MemOffset {
    MemOffset::new(device::ram::range(), ram().base())
}
