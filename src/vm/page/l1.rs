#![allow(dead_code)]

/// Page table entry
pub struct Entry {
    value: u32,
}

/// 9 p       or, implementation defined. should be zero
/// 5 domain  0
/// 4 sbz     should be zero
/// 3 ns      non-secure, effective only when walking from secure
/// 2 pxn     privileged execute never

const FAULT_MASK: u32 = 0b11;
const PAGE_TABLE_BASE_ADDR_MASK: u32 = !0x3FF;

impl Entry {

    pub fn init_fault(memo: u32) -> Entry {
        Entry { value: memo & !FAULT_MASK }
    }
    
    pub fn offset(&self, table_base: *const u32) -> usize {
        let page_table_address = (self.value & PAGE_TABLE_BASE_ADDR_MASK) as *const u32;
        let address_offset = (page_table_address as u32) - (table_base as u32);
        (address_offset / 4096) as usize
    }
    
}

pub struct Table {
    entries: [Entry; 4096],
}

impl Table {

    pub fn reset(&mut self) {
        for e in self.entries.iter_mut() {
            e.value = 0;
        } 
    }
    
    pub fn entry(&mut self, i: usize) -> &mut Entry {
        &mut self.entries[i]
    }

}