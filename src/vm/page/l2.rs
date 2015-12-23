#![allow(dead_code)]

/// Page descriptor
pub struct Entry {
    value: u32,
}

impl Entry {

    pub fn id_map(&mut self, page: u32, access: u32) {
        self.value = page << 12 | access; 
    }
    
}


pub struct Table {
    entries: [Entry; 4096],
}

impl Table {

    pub fn find_l2_entry(&mut self, page: u32) -> &mut Entry {
        &mut self.entries[0]
    }
     
}

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