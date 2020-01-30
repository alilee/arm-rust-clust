pub mod entry;

use super::layout;
use super::virt_addr::{PhysOffset, VirtAddr, VirtAddrRange, VirtOffset};
use crate::device;
use crate::pager::{frames, PhysAddr, PhysAddrRange, PAGESIZE_BYTES};
use crate::util::set_above_bits;
use entry::{PageDescriptor, TableDescriptor};

#[allow(unused_imports)]
use log::{debug, info, trace};

use core::cmp::max;
use core::fmt::{Debug, Error, Formatter};
use core::intrinsics::volatile_set_memory;
use core::mem;
use core::slice::Iter;

type PageTableEntry = u64;

const TABLE_ENTRIES: usize = PAGESIZE_BYTES / mem::size_of::<PageTableEntry>();
const LOWER_VA_BITS: u32 = 48; // 256 TB
const UPPER_VA_BITS: u32 = 39; // 512 GB
pub const UPPER_VA_BASE: usize = set_above_bits(UPPER_VA_BITS);
const UPPER_TABLE_LEVEL: u8 = 1;
const LOWER_TABLE_LEVEL: u8 = 0;
const LEVEL_OFFSETS: [usize; 4] = [39, 30, 21, 12];
const LEVEL_WIDTH: usize = 9;

pub const fn kernel_mem_offset() -> VirtOffset {
    VirtOffset::new_const(UPPER_VA_BASE)
}

pub const fn kernel_va_bits() -> u32 {
    UPPER_VA_BITS
}

pub const fn user_va_bits() -> u32 {
    LOWER_VA_BITS
}

#[derive(Copy, Clone)]
pub struct TranslationAttributes(TableDescriptor, PageDescriptor);

impl TranslationAttributes {
    pub const fn new(table_desc: TableDescriptor, page_desc: PageDescriptor) -> Self {
        Self(table_desc, page_desc)
    }

    pub fn table(&self) -> TableDescriptor {
        self.0
    }

    pub fn page(&self) -> PageDescriptor {
        self.1
    }
}

impl From<&TableDescriptor> for TranslationAttributes {
    fn from(pte: &TableDescriptor) -> Self {
        use entry::PageDescriptorFields::Valid;
        Self(*pte, PageDescriptor::new_bitfield(Valid::CLEAR))
    }
}

impl From<&PageDescriptor> for TranslationAttributes {
    fn from(pte: &PageDescriptor) -> Self {
        use entry::TableDescriptorFields::Valid;
        Self(TableDescriptor::new_bitfield(Valid::CLEAR), *pte)
    }
}

impl Debug for TranslationAttributes {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{:?}/{:?}", self.0, self.1)?;
        Ok(())
    }
}

pub struct Translation {
    /// Bottom of table's VA range
    va_range_base: VirtAddr,
    /// Which table level address translation starts from (normally 0)
    first_level: u8,
    /// PA of first-level page table
    page_table: PhysAddr,
    /// Offset for pointer to table base
    ram_offset: VirtOffset,
}

impl Translation {
    pub fn new_upper(ram_offset: VirtOffset) -> Result<Translation, u64> {
        debug!("Translation::new_upper(ram_offset: {:?})", ram_offset);
        let page_table = frames::find()?;
        clear_page_table(page_table, ram_offset);
        Ok(Translation {
            va_range_base: VirtAddr::new(UPPER_VA_BASE),
            first_level: UPPER_TABLE_LEVEL,
            page_table,
            ram_offset,
        })
    }

    pub fn new_lower(ram_offset: VirtOffset) -> Result<Translation, u64> {
        debug!("Translation::new_lower(ram_offset: {:?})", ram_offset);
        let page_table = frames::find()?;
        clear_page_table(page_table, ram_offset);
        Ok(Translation {
            va_range_base: VirtAddr::new(0),
            first_level: LOWER_TABLE_LEVEL,
            page_table,
            ram_offset,
        })
    }

    pub fn ttbr1() -> Result<Translation, u64> {
        use cortex_a::regs::{RegisterReadWrite, TTBR1_EL1};
        debug!("Translation::ttbr1()");
        let page_table = PhysAddr::new(TTBR1_EL1.get() as usize);
        let ram_offset = VirtOffset::between(device::ram::range().base(), layout::RAM.base());
        Ok(Translation {
            va_range_base: VirtAddr::new(UPPER_VA_BASE),
            first_level: UPPER_TABLE_LEVEL,
            page_table,
            ram_offset,
        })
    }

    pub fn _ttbr0() -> Result<Translation, u64> {
        use cortex_a::regs::{RegisterReadWrite, TTBR0_EL1};
        debug!("Translation::ttbr0()");
        let page_table = PhysAddr::new(TTBR0_EL1.get() as usize);
        let ram_offset = VirtOffset::between(device::ram::range().base(), layout::RAM.base());
        Ok(Translation {
            va_range_base: VirtAddr::new(0),
            first_level: LOWER_TABLE_LEVEL,
            page_table,
            ram_offset,
        })
    }

    pub fn page_table(&self) -> *mut PageTable {
        let p = VirtAddr::id_map(self.page_table, self.ram_offset).as_ptr();
        p as *mut PageTable
    }

    pub fn base_register(&self) -> u64 {
        self.page_table.get() as u64
    }

    pub fn identity_map(
        &mut self,
        phys_range: PhysAddrRange,
        attributes: TranslationAttributes,
    ) -> Result<(), u64> {
        debug!(
            "Translation::identity_map(&mut self, phys_range: {:?}, attributes: <{:?}>)",
            phys_range, attributes
        );
        let va_range_base = self.va_range_base;
        let pt = self.page_table();
        let virt_range = VirtAddrRange::id_map(phys_range);
        let phys_offset = PhysOffset::id_map();
        map_level(
            virt_range,
            phys_offset,
            self.first_level,
            pt,
            va_range_base,
            attributes,
            self.ram_offset,
        )
    }

    pub fn absolute_map(
        &mut self,
        phys_range: PhysAddrRange,
        virt_base: VirtAddr,
        attributes: TranslationAttributes,
    ) -> Result<(), u64> {
        debug!(
            "Translation::absolute_map(&mut self, phys_range: {:?}, virt_base: {:?}, attributes: <{:?}>)",
            phys_range, virt_base, attributes
        );
        let va_range_base = self.va_range_base;
        let pt = self.page_table();
        let (virt_range, phys_offset) = VirtAddrRange::target_map(phys_range, virt_base);
        map_level(
            virt_range,
            phys_offset,
            self.first_level,
            pt,
            va_range_base,
            attributes,
            self.ram_offset,
        )
    }
}

fn clear_page_table(page_table: PhysAddr, ram_offset: VirtOffset) {
    let pt = ram_offset.increment(page_table).as_ptr() as *mut PageTable;
    unsafe { volatile_set_memory(pt, 0, 1) };
}

fn map_level(
    virt_range: VirtAddrRange,
    phys_offset: PhysOffset,
    level: u8,
    pt: *mut PageTable,
    va_range_base: VirtAddr,
    attributes: TranslationAttributes,
    ram_offset: VirtOffset,
) -> Result<(), u64> {
    trace!(
        "id_map_level(virt_range: {:?}, phys_offset: {:?}, level: {}, pt: 0x{:08x}, va_range_base: {:?}, _, ram_offset: {:?})",
        virt_range,
        phys_offset,
        level,
        pt as u64,
        va_range_base,
        ram_offset
    );
    for (index, virt_range, va_range_base) in table_entries(virt_range, level, va_range_base) {
        trace!(
            "  index: {}, virt_range: {:?}, va_range_base: {:?}",
            index,
            virt_range,
            va_range_base
        );
        if level != 3 {
            let table = PageTableBranch::from_page_table(pt);
            let pte = table[index];
            let pt = if pte.is_valid() {
                pte.next_level_table_address()
            } else {
                // need a new table
                let pt = frames::find()?;
                let table_entry = TableDescriptor::new_entry(pt, attributes);
                table[index] = table_entry;
                let ppt = ram_offset.increment(pt).as_ptr() as *mut PageTable;
                unsafe { volatile_set_memory(ppt, 0, 1) };
                pt
            };
            let pt = ram_offset.increment(pt).as_ptr() as *mut PageTable;
            map_level(
                virt_range,
                phys_offset,
                level + 1,
                pt,
                va_range_base,
                attributes,
                ram_offset,
            )?;
        } else {
            let table = PageTableLeaf::from_page_table(pt);
            let pte = table[index];
            assert!(!pte.is_valid());
            let output_addr = phys_offset.translate(virt_range.base());
            let pte = PageDescriptor::new_entry(output_addr, attributes);
            table[index] = pte;
        }
    }
    Ok(())
}

#[derive(Debug)]
struct PageTableEntries {
    bounds: VirtAddrRange,
    index: usize,
    top: usize,
    entry_span: VirtAddrRange,
}

impl Iterator for PageTableEntries {
    type Item = (usize, VirtAddrRange, VirtAddr); // (index, va_sub_range, pt_entry_base)

    fn next(&mut self) -> Option<Self::Item> {
        trace!("PageTableEntries::next(&mut self) {:?}", self);
        if self.index < self.top {
            let result = (
                self.index,
                self.entry_span.intersection(&self.bounds),
                self.entry_span.base,
            );
            self.index += 1;
            self.entry_span = self.entry_span.step();
            Some(result)
        } else {
            None
        }
    }
}

fn extract_va_index(base: usize, offset: usize, width: usize) -> usize {
    trace!("{:#b}", base >> offset);
    trace!("{:#b}", (1 << width) - 1);
    trace!("{:#b}", (base >> offset) & ((1 << width) - 1));
    (base >> offset) & ((1 << width) - 1)
}

/// Generate an iterator for the page table at specific level covering from a specific base
/// where it overlaps with target range
fn table_entries(virt_range: VirtAddrRange, level: u8, base: VirtAddr) -> PageTableEntries {
    trace!(
        "table_entries(virt_range: {:?}), level: {}, base: {:?})",
        virt_range,
        level,
        base,
    );

    let level = level as usize;
    let level_offset = LEVEL_OFFSETS[level];

    trace!(
        "  index@level: bits[{}..{}] mask: 0x{:08x}",
        level_offset,
        level_offset + LEVEL_WIDTH,
        0, //((1usize << (LEVEL_WIDTH + 1usize)) - 1usize) << level_offset,
    );

    let first = extract_va_index(virt_range.base.addr(), level_offset, LEVEL_WIDTH);

    let entries = (virt_range.length + ((1 << level_offset) - 1)) >> level_offset;

    //    let entries = if virt_range.length > 0 {
    //        max(
    //            1,
    //            extract_va_index(virt_range.length, level_offset, LEVEL_WIDTH),
    //        )
    //    } else {
    //        0
    //    };

    let span = VirtAddrRange {
        base: base.increment(first << level_offset),
        length: 1usize << level_offset,
    };

    let result = PageTableEntries {
        bounds: virt_range,
        index: first as usize,
        top: (first + entries) as usize,
        entry_span: span,
    };
    trace!(
        "  result: {}..{} starting: {:?}",
        result.index,
        result.top,
        result.entry_span
    );
    result
}

#[derive(Copy, Clone)]
#[repr(align(4096))]
pub struct PageTable([u64; TABLE_ENTRIES]);

struct PageTableBranch([TableDescriptor; TABLE_ENTRIES]);

impl PageTableBranch {
    pub fn from_page_table(pt: *mut PageTable) -> &'static mut Self {
        unsafe { mem::transmute::<&mut PageTable, &mut Self>(&mut (*pt)) }
    }
    pub fn from_page_table_const(pt: *const PageTable) -> &'static Self {
        unsafe { mem::transmute::<&PageTable, &Self>(&(*pt)) }
    }
    pub fn entries(&self) -> Iter<'_, TableDescriptor> {
        self.0.iter()
    }
}

impl core::ops::Index<usize> for PageTableBranch {
    type Output = TableDescriptor;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl core::ops::IndexMut<usize> for PageTableBranch {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

struct PageTableLeaf([PageDescriptor; TABLE_ENTRIES]);

impl PageTableLeaf {
    pub fn from_page_table(pt: *mut PageTable) -> &'static mut Self {
        unsafe { mem::transmute::<&mut PageTable, &mut Self>(&mut (*pt)) }
    }
    pub fn from_page_table_const(pt: *const PageTable) -> &'static Self {
        unsafe { mem::transmute::<&PageTable, &Self>(&(*pt)) }
    }
    pub fn entries(&self) -> Iter<'_, PageDescriptor> {
        self.0.iter()
    }
}

impl core::ops::Index<usize> for PageTableLeaf {
    type Output = PageDescriptor;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl core::ops::IndexMut<usize> for PageTableLeaf {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

impl Debug for Translation {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        fn print_level(
            level: usize,
            pt: *const PageTable,
            table_base: VirtAddr,
            offset: VirtOffset,
            f: &mut Formatter<'_>,
        ) -> Result<(), Error> {
            const LEVEL_BUFFERS: [&str; 4] = ["", " ", "  ", "   "];
            writeln!(
                f,
                "      {}{:?}: level {} ============================= (0x{:8x})",
                LEVEL_BUFFERS[level], table_base, level, pt as u64
            )?;
            if level != 3 {
                let table = PageTableBranch::from_page_table_const(pt);
                for (i, pte) in table.entries().enumerate() {
                    if pte.is_valid() {
                        writeln!(f, "      {}{:03}: {:?}", LEVEL_BUFFERS[level], i, pte)?;
                        let pt = pte.next_level_table_address();
                        let table_base = table_base.increment(i << LEVEL_OFFSETS[level]);
                        let pt = offset.increment(pt).as_ptr() as *const PageTable;
                        print_level(level + 1, pt, table_base, offset, f)?;
                    }
                }
            } else {
                let table = PageTableLeaf::from_page_table_const(pt);
                for (i, pte) in table.entries().enumerate() {
                    if pte.is_valid() {
                        writeln!(
                            f,
                            "                            {}{:05x}: {:?}",
                            LEVEL_BUFFERS[level],
                            i * PAGESIZE_BYTES,
                            pte
                        )?;
                    }
                }
            }
            Ok(())
        }

        writeln!(
            f,
            "Translation: based at {:?} starting level {} (accessing ram through {:?})",
            self.va_range_base, self.first_level, self.ram_offset
        )?;
        writeln!(f, "      ######")?;
        let result = print_level(
            self.first_level as usize,
            self.page_table(),
            self.va_range_base,
            self.ram_offset,
            f,
        );
        write!(f, "      ######")?;
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consts() {
        assert_eq!(TABLE_ENTRIES, 512);
        assert_eq!(LOWER_VA_BITS, 48);
        assert_eq!(UPPER_VA_BITS, 43);
        assert_eq!(UPPER_VA_BASE, 0xFFFF_FE00_0000_0000);
        assert_eq!(UPPER_TABLE_LEVEL, 1);
        assert_eq!(LOWER_TABLE_LEVEL, 0);
    }
}
