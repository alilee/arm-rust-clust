#![allow(dead_code)]

/// Page table entry
#[derive(Debug, Clone, Copy)]
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
        (address_offset / 1024) as usize
    }

    pub fn is_fault(&self) -> bool {
        0 == self.value & FAULT_MASK
    }
}

pub struct Table {
    pub entries: [Entry; 1024],
}

impl Table {
    pub fn init(value: u32) -> Table {
        Table { entries: [Entry { value: value }; 1024] }
    }

    pub fn reset(&mut self) {
        for e in self.entries.iter_mut() {
            e.value = 0;
        }
    }

    pub fn entry(&mut self, i: usize) -> &mut Entry {
        &mut self.entries[i]
    }
}
