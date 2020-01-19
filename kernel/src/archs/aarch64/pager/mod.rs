use crate::pager::{PhysAddrRange, VirtAddr, VirtAddrRange};
use core::ops::Range;
use core::slice::Iter;
use register::{mmio::*, register_bitfields, LocalRegisterCopy};

register_bitfields! {
    u64,
    VirtualAddress [
        L0Index OFFSET(39) NUMBITS(9) [],
        L1Index OFFSET(30) NUMBITS(9) [],
        L2Index OFFSET(21) NUMBITS(9) [],
        L3Index OFFSET(12) NUMBITS(9) [],
        PageOffset OFFSET(0) NUMBITS(12) []
    ]
}

register_bitfields! {
    u64,
    TableDescriptor [
        NSTable OFFSET(63) NUMBITS(1) [],
        APTable OFFSET(61) NUMBITS(2) [],
        XNTable OFFSET(60) NUMBITS(1) [],
        PXNTable OFFSET(59) NUMBITS(1) [],
        NextLevelTableAddress OFFSET(12) NUMBITS(35) [],
        Type OFFSET(1) NUMBITS(1) [],
        Valid OFFSET(0) NUMBITS(1) []
    ]
}

register_bitfields! {
    u64,
    PageDescriptor [
        Available OFFSET(55) NUMBITS(9) [],
        UXN OFFSET(54) NUMBITS(1) [],                      // Unprivileged Execute Never
        PXN OFFSET(53) NUMBITS(1) [],                      // Privileged Execute Never
        Contiguous OFFSET(52) NUMBITS(1) [],               // One of a contiguous set of entries
        OutputAddress OFFSET(12) NUMBITS(35) [],
        nG OFFSET(11) NUMBITS(1) [],                       // Not Global - all or current ASID
        AF OFFSET(10) NUMBITS(1) [],                       // Access flag
        SH OFFSET(8) NUMBITS(2) [],                        // Shareability
        AP OFFSET(6) NUMBITS(2) [],                        // Data access permissions
        AttrIndx OFFSET(2) NUMBITS(3) [],                  // Memory attributes index for MAIR_ELx
        Type OFFSET(1) NUMBITS(1) [],
        Valid OFFSET(0) NUMBITS(1) []
    ]
}

#[repr(align(4096))]
struct PageTable([u64; 512]);

struct PageTableBranch([ReadWrite<u64, TableDescriptor::Register>; 512]);

struct PageTableLeaf([ReadWrite<u64, PageDescriptor::Register>; 512]);

impl PageTable {
    const fn init() -> Self {
        PageTable([0u64; 512])
    }
}

const MAX_TABLES: usize = 4;
static mut PAGE_TABLES: [PageTable; MAX_TABLES] = [PageTable([0; 512]); MAX_TABLES];
static mut NEXT_TABLE: usize = 1usize;

fn table_entry(
    pt: &PageTable,
    attributes: u64,
) -> LocalRegisterCopy<u64, TableDescriptor::Register> {
    use TableDescriptor::*;

    let nlta = pt as *const PageTable as u64;
    let pte = LocalRegisterCopy::<u64, VirtualAddress::Register>::new(attributes);
    pte.modify(Valid::SET + Type::SET + NextLevelTableAddress.val(nlta >> 12));
    pte
}

fn page_entry(base: VirtAddr, attributes: u64) -> LocalRegisterCopy<u64, PageDescriptor::Register> {
    use PageDescriptor::*;

    let offset = base.addr();
    let pte = LocalRegisterCopy::<u64, PageDescriptor::Register>::new(attributes);
    pte.modify(Valid::SET + Type::SET + OutputAddress.val(offset >> 12));
    pte
}

struct PageTableEntries {
    bounds: VirtAddrRange,
    index: usize,
    top: usize,
    span: VirtAddrRange,
}

impl Iterator for PageTableEntries {
    type Item = (usize, VirtAddrRange);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.top {
            let result = (self.index, self.span.intersection(&self.bounds));
            self.index += 1;
            self.span = self.span.step();
            Some(result)
        } else {
            None
        }
    }
}

fn extract_bitslice(base: u64, offset: usize, width: usize) -> u64 {
    (base >> (offset as u64)) & ((1 << (width as u64)) - 1)
}

fn table_entries(range: VirtAddrRange, level: usize) -> PageTableEntries {
    const LEVEL_OFFSETS: [usize; 4] = [39, 30, 21, 12];
    const LEVEL_WIDTH: usize = 9;
    let level_offset = LEVEL_OFFSETS[level];

    let first = extract_bitslice(range.base.addr(), level_offset, LEVEL_WIDTH);
    let entries = if range.length > 0 {
        extract_bitslice(range.length as u64, level_offset, LEVEL_WIDTH)
    } else {
        0
    };

    let span = VirtAddrRange {
        base: VirtAddr(first << (level_offset as u64)),
        length: 1usize << level_offset,
    };

    PageTableEntries {
        bounds: range.clone(),
        index: first as usize,
        top: (first + entries) as usize,
        span,
    }
}

pub fn init() {}

fn id_map_level(level: usize, pt: &mut PageTable, range: VirtAddrRange, attributes: u64) {
    for (index, range) in table_entries(range, level) {
        if level != 3 {
            let table = pt as &mut PageTableBranch;
            let pte = table[index].extract();
            let pt = if !pte.is_set(TableDescriptor::Valid) {
                // need a new table
                unsafe {
                    let pt = unsafe { &mut PAGE_TABLES[NEXT_TABLE] };
                    NEXT_TABLE += 1;
                    let table_entry = table_entry(pt, attributes);
                    pt[index].set(table_entry.get());
                    pt
                }
            } else {
                let pt = pte.read(TableDescriptor::NextLevelTableAddress) << 12u64;
                pt as &mut PageTable
            };
            id_map_level(level + 1, pt, range, attributes)
        } else {
            let table = pt as &mut PageTableLeaf;
            let pte = table[index].extract();
            assert!(!pte.is_set(PageDescriptor::Valid));
            let page_entry = page_entry(range.base, attributes);
            pt[index].set(page_entry.get());
        }
    }
}

pub fn id_map(range: PhysAddrRange) {
    let range = VirtAddrRange::id_map(range);
    let l0 = unsafe { &mut PAGE_TABLES[0] };

    id_map_level(0, l0, range, 0u64);
}

pub fn enable_paging() {
    use cortex_a::regs::*;

    // TCR_EL1.modify(TCR_EL1::IPS.val(0b101) + TCR_EL1::TG1.val(0b10) + TCR_EL1::)
}

pub fn print_state() {}
