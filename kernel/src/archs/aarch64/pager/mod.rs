use crate::dbg;
use crate::pager::{PhysAddr, PhysAddrRange, VirtAddr, VirtAddrRange};
use log::{debug, trace};

use register::{mmio::*, register_bitfields, LocalRegisterCopy};
use spin::{Mutex, MutexGuard};

use core::mem;

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

const TABLE_ENTRIES: usize = 512;

#[derive(Copy, Clone)]
#[repr(align(4096))]
struct PageTable([u64; TABLE_ENTRIES]);

struct PageTableBranch([ReadWrite<u64, TableDescriptor::Register>; TABLE_ENTRIES]);

struct PageTableLeaf([ReadWrite<u64, PageDescriptor::Register>; TABLE_ENTRIES]);

impl PageTable {
    const fn init() -> Self {
        PageTable([0u64; TABLE_ENTRIES])
    }

    fn to_branch(self: &mut Self) -> &mut PageTableBranch {
        unsafe { mem::transmute::<&mut Self, &mut PageTableBranch>(self) }
    }

    fn to_leaf(self: &mut Self) -> &mut PageTableLeaf {
        unsafe { mem::transmute::<&mut Self, &mut PageTableLeaf>(self) }
    }
}

fn table_entry(
    pt: *const PageTable,
    attributes: u64,
) -> LocalRegisterCopy<u64, TableDescriptor::Register> {
    use TableDescriptor::*;

    let nlta = pt as u64;
    let field = Valid::SET + Type::SET + NextLevelTableAddress.val(nlta >> 12);
    let value = (attributes & !field.mask) | field.value;
    LocalRegisterCopy::<u64, Register>::new(value)
}

fn page_entry(base: VirtAddr, attributes: u64) -> LocalRegisterCopy<u64, PageDescriptor::Register> {
    use PageDescriptor::*;

    let offset = base.addr();
    let field = Valid::SET + Type::SET + OutputAddress.val(offset >> 12);
    let value = (attributes & !field.mask) | field.value;
    LocalRegisterCopy::<u64, Register>::new(value)
}

#[derive(Debug)]
struct PageTableEntries {
    bounds: VirtAddrRange,
    index: usize,
    top: usize,
    span: VirtAddrRange,
}

impl Iterator for PageTableEntries {
    type Item = (usize, VirtAddrRange, VirtAddr);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.top {
            let result = (
                self.index,
                self.span.intersection(&self.bounds),
                self.span.base,
            );
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

/// Generate an iterator for the page table at specific level covering from a specific base
/// where it overlaps with target range
fn table_entries(range: VirtAddrRange, level: usize, base: VirtAddr) -> PageTableEntries {
    const LEVEL_OFFSETS: [usize; 4] = [39, 30, 21, 12];
    const LEVEL_WIDTH: usize = 9;
    let level_offset = LEVEL_OFFSETS[level];

    trace!("table_entries:");
    trace!("range: {:?}", range);
    trace!("level: {}", level);
    trace!("base: {:?}", base);
    trace!("level_offset: {}", level_offset);

    let first = extract_bitslice(range.base.addr(), level_offset, LEVEL_WIDTH);
    trace!("first: {}", first);
    let entries = if range.length > 0 {
        extract_bitslice(range.length as u64, level_offset, LEVEL_WIDTH) + 1
    } else {
        0
    };
    trace!("entries: {}", entries);

    let span = VirtAddrRange {
        base: base.offset(first << (level_offset as u64)),
        length: 1usize << level_offset,
    };
    trace!("span {:?}", span);

    let result = PageTableEntries {
        bounds: range.clone(),
        index: first as usize,
        top: (first + entries) as usize,
        span,
    };
    trace!("result: {:?}", result);
    result
}

pub fn init() {
    let ram = unsafe {
        extern "C" {
            static image_base: u8;
            static image_end: u8;
        }
        PhysAddrRange::bounded_by(
            PhysAddr::from_linker_symbol(&image_base),
            PhysAddr::from_linker_symbol(&image_end),
        )
    };
    id_map(ram);
    print_state();
}

const MAX_TABLES: usize = 4;

#[repr(align(4096))]
#[repr(C)]
struct Allocator {
    tables: [PageTable; MAX_TABLES],
    next: usize,
    top: usize,
}

struct Locked<A> {
    inner: Mutex<A>,
}

impl<A> Locked<A> {
    pub const fn new(inner: A) -> Self {
        Locked {
            inner: Mutex::new(inner),
        }
    }

    pub fn lock(&self) -> MutexGuard<A> {
        self.inner.lock()
    }
}

impl Locked<Allocator> {
    pub fn get_page(&self) -> Result<*mut PageTable, u64> {
        let mut lock = self.lock();

        if lock.next >= lock.top {
            return Err(0);
        }
        let next = lock.next;
        let result = &mut lock.tables[next] as *mut PageTable;
        lock.next += 1;
        Ok(result)
    }
}

static STATIC_ALLOCATOR: Locked<Allocator> = Locked::<Allocator>::new(Allocator {
    tables: [PageTable([0; TABLE_ENTRIES]); MAX_TABLES],
    next: 0,
    top: MAX_TABLES,
});

fn id_map_level(
    level: usize,
    pt: *mut PageTable,
    table_base: VirtAddr,
    range: VirtAddrRange,
    attributes: u64,
    allocator: &Locked<Allocator>,
) -> Result<(), u64> {
    debug!("id_map_level");
    trace!("{}, 0x{:08x}, {:?}", level, pt as u64, range);
    for (index, range, table_base) in table_entries(range, level, table_base) {
        if level != 3 {
            let table = pt as *mut PageTableBranch;
            let pte = unsafe { &(*table).0[index] };
            let pt = if !pte.is_set(TableDescriptor::Valid) {
                // need a new table
                let pt = allocator.get_page()?;
                let table_entry = table_entry(pt, attributes);
                unsafe { (*table).0[index].set(table_entry.get()) };
                pt
            } else {
                let pt = pte.read(TableDescriptor::NextLevelTableAddress) << 12u64;
                unsafe { mem::transmute::<u64, &mut PageTable>(pt) }
            };
            id_map_level(level + 1, pt, table_base, range, attributes, allocator)?;
        } else {
            let table = pt as *mut PageTableLeaf;
            let pte = unsafe { &(*table).0[index] };
            assert!(!pte.is_set(PageDescriptor::Valid));
            let page_entry = page_entry(range.base, attributes);
            unsafe { (*pt).0[index] = page_entry.get() };
        }
    }
    Ok(())
}

pub fn id_map(range: PhysAddrRange) {
    let range = VirtAddrRange::id_map(range);
    let pt = STATIC_ALLOCATOR.get_page().unwrap();
    id_map_level(0, pt, VirtAddr::init(0), range, 0u64, &STATIC_ALLOCATOR);
}

pub fn enable_paging() {
    // use cortex_a::regs::*;

    // TCR_EL1.modify(TCR_EL1::IPS.val(0b101) + TCR_EL1::TG1.val(0b10) + TCR_EL1::)
}

pub fn print_state() {
    fn print_level(level: usize, pt: *const PageTable, table_base: VirtAddr) {
        const LEVEL_OFFSETS: [usize; 4] = [39, 30, 21, 12];
        const LEVEL_WIDTH: usize = 9;
        const LEVEL_BUFFERS: [&str; 4] = ["", " ", "  ", "   "];
        debug!(
            "{:?}: level {} ============================= (0x{:8x})",
            table_base, level, pt as u64
        );
        for (i, pte) in unsafe { (*pt).0.iter().enumerate() } {
            if *pte != 0 {
                if level != 3 {
                    debug!("{}{:03}: 0b{:064b}", LEVEL_BUFFERS[level], i, pte);
                    let pte = LocalRegisterCopy::<u64, TableDescriptor::Register>::new(*pte);
                    let pt = pte.read(TableDescriptor::NextLevelTableAddress) << 12u64;
                    let pt = unsafe { mem::transmute::<u64, *const PageTable>(pt) };
                    let table_base = table_base.offset((i << LEVEL_OFFSETS[level]) as u64);
                    print_level(level + 1, pt, table_base);
                } else {
                    debug!("{}{:03}: 0b{:064b}", LEVEL_BUFFERS[level], i, pte);
                }
            }
        }
    }

    let tables = &STATIC_ALLOCATOR.lock().tables;
    print_level(0, &tables[0], VirtAddr::init(0));
}
