
extern crate core;
use core::intrinsics as i;

extern {
    static page_table: *mut u32;
}

pub fn init() {
    // empty page table
    // clear 4 pages from page_table
    unsafe {
        i::write_bytes(page_table, 0, 0x1000);
    }
}

pub fn id_map(a: *const u32, pages: u8) {
    pages;
}

