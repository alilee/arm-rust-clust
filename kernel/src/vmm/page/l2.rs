#![allow(dead_code)]

/// Page descriptor
#[derive(Debug, Clone, Copy)]
pub struct Entry {
    value: u32,
}

const FAULT_MASK: u32 = 0b11;

impl Entry {
    
    pub fn init(value: u32) -> Entry {
        Entry { value: value }
    }

    pub fn id_map(&mut self, page: u32, access: u32) {
        self.value = page << 12 | access; 
    }
    
    pub fn is_fault(&self) -> bool {
        0 == self.value & FAULT_MASK
    }
    
}

/// Set of 4096 level 2 page tables (covers entire 32-bit address range for 4k tables)
pub struct Table {
    pub entries: [Entry; 1024],
}

impl Table {

    pub fn init(value: u32) -> Table {
        Table { entries: [ Entry { value: value }; 1024] }
    }
    
    pub fn find_l2_entry(&mut self, page: u32) -> &mut Entry {
        &mut self.entries[0]
    }
     
}

// impl Copy for Table {}
// impl Clone for Table {
//     fn clone(&self) -> Table {
//         *self
//     }
// }

const NG_BITS: u8 = 11;
const S_BITS: u8 = 10;
const APX_BITS: u8 = 9;
const TEX_BITS: u8 = 6;
const AP1_BITS: u8 = 5;
const C_BITS: u8 = 3;
const B_BITS: u8 = 2;

///
///
/// ng      non-global
/// s       shareable, memory-region attribute
/// tex,c,b memory-region attributes
/// apx     disable write access
/// ap      enable unprivileged access
/// access  accessed flag
/// xn      execute never
///
pub fn build_access(ng: bool, s: bool, apx: bool, tex: u8, ap: bool, c: bool, b: bool, xn: bool) -> u32 {
    (ng as u32) << NG_BITS |
    (s as u32) << S_BITS |
    (apx as u32) << APX_BITS |
    (tex as u32) << TEX_BITS |
    (ap as u32) << AP1_BITS |
    (c as u32) << C_BITS |
    (b as u32) << B_BITS |
    (xn as u32)
}