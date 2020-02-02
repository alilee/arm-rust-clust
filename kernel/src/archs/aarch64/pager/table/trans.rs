use super::attrs;
use super::desc;
use super::{
    PageTable, PageTableEntry, LEVEL_OFFSETS, LEVEL_WIDTH, LOWER_TABLE_LEVEL, UPPER_TABLE_LEVEL,
    UPPER_VA_BASE,
};
use attrs::TranslationAttributes;
use desc::{PageBlockDescriptor, TableDescriptor};

use crate::arch::pager::virt_addr::{PhysOffset, VirtAddr, VirtAddrRange, VirtOffset};
use crate::device;
use crate::pager::{frames, range::layout, PhysAddr, PhysAddrRange, PAGESIZE_BYTES};

use log::{debug, trace};

use core::fmt::{Debug, Error, Formatter};

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
        let ram_offset = VirtOffset::between(
            device::ram::range().base(),
            VirtAddr::from(layout::ram().base()),
        );
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
        let ram_offset = VirtOffset::between(
            device::ram::range().base(),
            VirtAddr::from(layout::ram().base()),
        );
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
    unsafe { core::intrinsics::volatile_set_memory(pt, 0, 1) };
}

fn map_level(
    target_range: VirtAddrRange,
    phys_offset: PhysOffset,
    level: u8,
    pt: *mut PageTable,
    table_base: VirtAddr,
    attributes: TranslationAttributes,
    ram_offset: VirtOffset,
) -> Result<(), u64> {
    trace!(
        "id_map_level(table_range: {:?}, phys_offset: {:?}, level: {}, pt: 0x{:08x}, table_base: {:?}, _, ram_offset: {:?})",
        target_range,
        phys_offset,
        level,
        pt as u64,
        table_base,
        ram_offset
    );
    for (index, entry_target_range, entry_range) in table_entries(target_range, level, table_base) {
        // pt: a pointer to the physical address of the page table, offset by ram_offset, which can
        //     be used to read or modify the page table.
        // pt[index]: the relevant entry inside the page table
        // phys_offset: the offset from a va to the intended output address
        // entry_range: the sub-part of the range which this entry covers
        // entry_target_range: the all or part of this entry to map pages for
        // target_range: the span of entries in this table to map pages for
        // table_base: the va of pt[0]
        trace!(
            "  index: {}, entry_target_range: {:?}, entry_range: {:?}",
            index,
            entry_target_range,
            entry_range,
        );
        let table = unsafe { &mut (*pt) };
        if level == 3
            || ((1u8..=2u8).contains(&level) && attributes.pageblock_desc().is_contiguous())
        {
            // if the entire entry_range is inside the virt_range
            if level == 3 || entry_target_range.covers(&entry_range) {
                // level 1: 1GB block
                // level 2: 2MB block
                // level 3: 4KB page
                let pte = table.get(index);
                assert!(!pte.is_valid());
                let output_addr = phys_offset.translate(entry_range.base());
                trace!("{:?}+{:?}={:?}", entry_range, phys_offset, output_addr);
                let pte = PageBlockDescriptor::new_entry(level, output_addr, attributes);
                table.set(index, PageTableEntry::from(pte));
                trace!("{:?}", pte);
                continue;
            }
        }
        let pte = TableDescriptor::from(table.get(index));
        let pt = if pte.is_valid() {
            pte.next_level_table_address()
        } else {
            // need a new table
            let pt = frames::find()?;
            let table_entry = TableDescriptor::new_entry(pt, attributes);
            table.set(index, PageTableEntry::from(table_entry));
            let ppt = ram_offset.increment(pt).as_ptr() as *mut PageTable;
            unsafe { core::intrinsics::volatile_set_memory(ppt, 0, 1) };
            pt
        };
        let pt = ram_offset.increment(pt).as_ptr() as *mut PageTable;
        map_level(
            entry_target_range,
            phys_offset,
            level + 1,
            pt,
            entry_range.base(),
            attributes,
            ram_offset,
        )?;
    }
    Ok(())
}

struct PageTableEntries {
    bounds: VirtAddrRange,
    index: usize,
    top: usize,
    entry_span: VirtAddrRange,
}

impl Iterator for PageTableEntries {
    type Item = (usize, VirtAddrRange, VirtAddrRange); // (index, virt_range, entry_range)

    fn next(&mut self) -> Option<Self::Item> {
        trace!("PageTableEntries::next(&mut self) {:?}", self);
        if self.index < self.top {
            let result = (
                self.index,
                self.entry_span.intersection(&self.bounds),
                self.entry_span,
            );
            self.index += 1;
            self.entry_span = self.entry_span.step();
            Some(result)
        } else {
            None
        }
    }
}

impl Debug for PageTableEntries {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(
            f,
            "{:?} ({}..{}) {:?}",
            self.bounds, self.index, self.top, self.entry_span
        )
    }
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
        "  index@level: bits[{}..{}] mask: {:#016x}",
        level_offset,
        level_offset + LEVEL_WIDTH,
        ((1usize << LEVEL_WIDTH) - 1usize) << level_offset,
    );

    let first = (virt_range.base.addr() >> level_offset) & ((1 << LEVEL_WIDTH) - 1);
    let entries = (virt_range.length + ((1 << level_offset) - 1)) >> level_offset;

    let span = VirtAddrRange {
        base: base.increment(first << level_offset),
        length: 1usize << level_offset,
    };

    let result = PageTableEntries {
        bounds: virt_range,
        index: first,
        top: first + entries,
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
            let table = unsafe { *pt };
            for (i, pte) in table.entries().enumerate() {
                if pte.is_valid() {
                    if pte.is_table(level) {
                        //                        writeln!(f, "{} istable! at level: {} {:#b}", i, level, pte.get())?;
                        let pte = TableDescriptor::from(*pte);
                        writeln!(f, "      {}{:03}: {:?}", LEVEL_BUFFERS[level], i, pte)?;
                        let pt = pte.next_level_table_address();
                        let table_base = table_base.increment(i << LEVEL_OFFSETS[level]);
                        let pt = offset.increment(pt).as_ptr() as *const PageTable;
                        print_level(level + 1, pt, table_base, offset, f)?;
                    } else {
                        let pte = PageBlockDescriptor::from(*pte);
                        writeln!(
                            f,
                            "                         {}{:08x}: {:?}",
                            LEVEL_BUFFERS[level],
                            i << LEVEL_OFFSETS[level],
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
