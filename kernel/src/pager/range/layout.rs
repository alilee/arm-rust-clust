use super::{Page, PageRange};
use crate::arch;
use crate::pager::PAGESIZE_BYTES;

const GB: usize = 1024 * 1024 * 1024 / PAGESIZE_BYTES;

pub const LAYOUT: [(&str, usize); 3] = [("ram", 4 * GB), ("image", 1 * GB), ("device", 1 * GB)];

fn find(section: &str) -> Result<(*const Page, *const Page), u64> {
    let mut result: *const Page = arch::pager::KERNEL_BASE;
    for (this_section, length_pages) in LAYOUT.iter() {
        let next = unsafe { result.offset(*length_pages as isize) };
        if *this_section == section {
            return Ok((result, next));
        }
        result = next;
    }
    Err(0)
}

pub fn ram() -> PageRange {
    PageRange::new(find("ram").unwrap())
}

pub fn image() -> PageRange {
    PageRange::new(find("image").unwrap())
}

pub fn device() -> PageRange {
    PageRange::new(find("image").unwrap())
}
