// SPDX-License-Identifier: Unlicense

//! Paging trait for aarch64

mod layout;
mod mair;
mod table;

pub use layout::kernel_offset;
pub use table::TABLE_ENTRIES;

use table::{PageBlockDescriptor, PageTable, TableDescriptor, LEVEL_OFFSETS, LEVEL_WIDTH};

use super::{hal, Arch};

use crate::archs::{DeviceTrait, PagerTrait};
use crate::pager::AttributeField::OnDemand;
use crate::pager::{
    Addr, AddrRange, AttributeField, Attributes, FixedOffset, FrameAllocator, FramePurpose,
    PhysAddr, PhysAddrRange, Translate, VirtAddr, VirtAddrRange,
};
use crate::util::locked::Locked;
use crate::{Error, Result};

use core::any::Any;

impl PagerTrait for Arch {
    fn ram_range() -> Result<PhysAddrRange> {
        // FIXME: Collect from DTB
        Ok(PhysAddrRange::between(
            PhysAddr::at(0x4000_0000),
            PhysAddr::at(0x5000_0000),
        ))
    }
    fn kernel_base() -> VirtAddr {
        let result = VirtAddr::at(!((1 << super::UPPER_VA_BITS) - 1));
        result
    }

    fn kernel_offset() -> FixedOffset {
        layout::kernel_offset()
    }

    fn boot_image() -> PhysAddrRange {
        layout::boot_image()
    }

    fn text_image() -> PhysAddrRange {
        layout::text_image()
    }

    fn static_image() -> PhysAddrRange {
        layout::static_image()
    }

    fn bss_image() -> PhysAddrRange {
        layout::bss_image()
    }

    fn data_image() -> PhysAddrRange {
        layout::data_image()
    }

    fn pager_init() -> Result<()> {
        info!("init");
        mair::init()
    }

    fn enable_paging(page_directory: &impl crate::archs::PageDirectory) -> Result<()> {
        info!("enable");

        let page_directory = page_directory
            .as_any()
            .downcast_ref::<PageDirectory>()
            .expect("PageDirectory downcast");

        let ttb1 = page_directory.ttb1().unwrap().get() as u64;
        let ttb0 = page_directory.ttb0().unwrap().get() as u64;

        debug!("ttb1: {:?}", ttb1);
        debug!("ttb0: {:?}", ttb0);

        hal::enable_paging(ttb1, ttb0, 0)
    }
}

/// Starting level of kernel range.
const TTB0_FIRST_LEVEL: u8 = 0;

/// Starting level of user range.
const TTB1_FIRST_LEVEL: u8 = 1;

/// Aarch64 implementation of a page directory
pub struct PageDirectory {
    ttb0: Option<PhysAddr>,
    // physical address of the root table for user space
    ttb1: Option<PhysAddr>, // physical address of the root table for kernel space
}

impl PageDirectory {
    fn new() -> Self {
        Self {
            ttb0: None,
            ttb1: None,
        }
    }

    /// Physical address of kernel address space directory.
    fn ttb0(&self) -> Option<PhysAddr> {
        self.ttb0
    }

    /// Physical address of kernel address space directory.
    fn ttb1(&self) -> Option<PhysAddr> {
        self.ttb1
    }

    /// Return the virtual address range for the required span for a page table entry to be contiguous.
    fn contiguous_virt_range(
        level: u8,
        index: usize,
        page_table_virt_addr_range_base: VirtAddr,
    ) -> VirtAddrRange {
        const CONTIG_SPAN: usize = 16;

        let level_offset = LEVEL_OFFSETS[level as usize];
        let entry_size = 1usize << level_offset;
        let index = index - index % CONTIG_SPAN;
        let base = page_table_virt_addr_range_base.increment(index * entry_size);
        let length = CONTIG_SPAN * entry_size;
        VirtAddrRange::new(base, length)
    }

    fn map_level(
        &mut self,
        target_range: VirtAddrRange,
        translation: &(impl Translate + core::fmt::Debug),
        level: u8,
        phys_addr_table: PhysAddr,
        page_table_virt_addr_range_base: VirtAddr,
        attributes: Attributes,
        allocator: &Locked<impl FrameAllocator>,
        mem_access_translation: &impl Translate,
    ) -> Result<VirtAddrRange> {
        trace!(
            "map_level(target_range: {:?}, level: {}, page_table: {:?}, pt_base: {:?}, ...)",
            target_range,
            level,
            phys_addr_table,
            page_table_virt_addr_range_base,
        );
        let page_table = unsafe {
            mem_access_translation
                .translate_phys(phys_addr_table)?
                .as_mut_ref::<PageTable>()
        };
        for (index, entry_target_range, entry_range) in
            table_entries(target_range, level, page_table_virt_addr_range_base)
        {
            // page_table: a pointer to the physical address of the page table, offset
            //     by mem_access_translation, which can be used to read or modify the page table.
            // pt[index]: the relevant entry inside the page table
            // mapper: generates an output address from a va
            // entry_range: the sub-part of the range which this entry covers
            // entry_target_range: the all or part of this entry to map pages for
            // target_range: the span of entries in this table to map pages for
            // page_table_virt_addr_range_base: the va of pt[0]
            debug!(
                "  index: {:03}, entry_target_range: {:?}, entry_range: {:?}",
                index, entry_target_range, entry_range,
            );
            if level == 3
                || ((1u8..=2u8).contains(&level) && attributes.is_set(AttributeField::Block))
            {
                // if the entry range is inside the 16-entry span contig range
                let contiguous_range =
                    Self::contiguous_virt_range(level, index, page_table_virt_addr_range_base);
                let contiguous = attributes.is_set(AttributeField::Block)
                    && target_range.covers(&contiguous_range);
                // if the entire entry_range is inside the virt_range
                if level == 3 || entry_target_range.covers(&entry_range) {
                    // level 1: 1GB block
                    // level 2: 2MB block
                    // level 3: 4KB page
                    assert!(!page_table[index].is_valid()); // FIXME: re-mapping
                    let maybe_output_addr = translation.translate_maybe(entry_range.base());
                    trace!(
                        "{:?}+{:?}={:?}",
                        entry_range,
                        translation,
                        maybe_output_addr
                    );
                    page_table[index] = PageBlockDescriptor::new_entry(
                        level,
                        maybe_output_addr,
                        attributes,
                        contiguous,
                    )
                    .into();
                    trace!("{:?}", page_table[index]);
                    continue;
                }
            }
            let maybe_phys_addr_table = if page_table[index].is_valid() {
                // FIXME: check that new attributes are compatible with parent tables
                Some(page_table[index].next_level_table_address())
            } else {
                // FIXME: PageTable for the entry may have just been paged out
                let maybe_phys_addr =
                    if attributes.is_set(OnDemand) && target_range.covers(&entry_range) {
                        // No frame needed for next level table yet.
                        None
                    } else {
                        let purpose = match level {
                            2 => FramePurpose::LeafPageTable,
                            _ => FramePurpose::BranchPageTable,
                        };
                        Some(allocator.lock().alloc_zeroed(purpose)?)
                    };
                page_table[index] = TableDescriptor::new_entry(maybe_phys_addr, attributes).into();
                maybe_phys_addr
            };
            if let Some(phys_addr_table) = maybe_phys_addr_table {
                self.map_level(
                    entry_target_range,
                    translation,
                    level + 1,
                    phys_addr_table,
                    entry_range.base(),
                    attributes,
                    allocator,
                    mem_access_translation,
                )?;
            }
        }
        Ok(target_range)
    }
}

impl crate::archs::PageDirectory for PageDirectory {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn map_translation(
        &mut self,
        target_range: VirtAddrRange,
        translation: impl Translate + core::fmt::Debug,
        attributes: Attributes,
        allocator: &Locked<impl FrameAllocator>,
        mem_access_translation: &impl Translate,
    ) -> Result<VirtAddrRange> {
        info!(
            "map_translation(&mut self, va_range: {:?}, {:?}, {:?}, ...)",
            target_range, translation, attributes
        );

        let (phys_addr_table, page_table_virt_addr_range_base, first_level) =
            if target_range.base() < Arch::kernel_base() {
                self.ttb0 = self.ttb0.or_else(|| {
                    allocator
                        .lock()
                        .alloc_zeroed(FramePurpose::BranchPageTable)
                        .ok()
                });
                (
                    self.ttb0.ok_or(Error::OutOfMemory)?,
                    VirtAddr::null(),
                    TTB0_FIRST_LEVEL,
                )
            } else {
                self.ttb1 = self.ttb1.or_else(|| {
                    allocator
                        .lock()
                        .alloc_zeroed(FramePurpose::BranchPageTable)
                        .ok()
                });
                (
                    self.ttb1.ok_or(Error::OutOfMemory)?,
                    Arch::kernel_base(),
                    TTB1_FIRST_LEVEL,
                )
            };

        self.map_level(
            target_range,
            &translation,
            first_level,
            phys_addr_table,
            page_table_virt_addr_range_base,
            attributes,
            allocator,
            mem_access_translation,
        )
    }

    #[allow(dead_code)]
    fn dump(&self, mem_access_translation: &impl Translate) {
        fn dump_level(phys_addr: PhysAddr, level: usize, mem_access_translation: &impl Translate) {
            const LEVEL_BUFFERS: [&str; 4] = ["", " ", "  ", "   "];

            info!("dumping table at {:?} being level {}", phys_addr, level);
            let page_table = unsafe {
                mem_access_translation
                    .translate_phys(phys_addr)
                    .unwrap()
                    .as_ref::<PageTable>()
            };
            let mut null_count = 0u16;
            for i in 0..512 {
                let entry = page_table[i];
                if entry.is_null() {
                    null_count += 1;
                } else {
                    if null_count > 0 {
                        info!("{} [...] {} null entries", LEVEL_BUFFERS[level], null_count);
                        null_count = 0;
                    }
                    if !entry.is_table(level as u8) {
                        let entry = PageBlockDescriptor::from(entry);
                        info!("{} [{:>3}] {:?}", LEVEL_BUFFERS[level], i, entry);
                    } else {
                        let table = TableDescriptor::from(entry);
                        info!("{} [{:>3}] {:?}", LEVEL_BUFFERS[level], i, table);
                        if level < 3 && entry.is_valid() {
                            dump_level(
                                entry.next_level_table_address(),
                                level + 1,
                                mem_access_translation,
                            );
                        }
                    }
                }
            }
            if null_count > 0 {
                info!("{} [...] {} null entries", LEVEL_BUFFERS[level], null_count);
            }
        }

        info!("PageDirectory");

        if self.ttb0.is_some() {
            info!("TTBO");
            dump_level(
                self.ttb0.unwrap(),
                TTB0_FIRST_LEVEL.into(),
                mem_access_translation,
            );
        }

        if self.ttb1.is_some() {
            info!("TTB1");
            dump_level(
                self.ttb1.unwrap(),
                TTB1_FIRST_LEVEL.into(),
                mem_access_translation,
            );
        }
        info!("PageDirectory ends");
    }
}

pub fn new_page_directory() -> impl crate::archs::PageDirectory {
    PageDirectory::new()
}

const GB: usize = 1024 * 1024 * 1024;

/// Create a level 1 block descriptor to map first GB of physical RAM
///
/// TODO: Make const
#[allow(dead_code)]
fn make_boot_ram_descriptor() -> u64 {
    let phys_addr = Arch::ram_range().expect("Arch::ram_range").base();
    assert!(phys_addr.is_aligned(1 * GB));
    let level = 1;
    let contiguous = false;
    PageBlockDescriptor::new_entry(
        level,
        Some(phys_addr),
        Attributes::KERNEL_RWX.set(AttributeField::Accessed),
        contiguous,
    )
    .get()
}

pub const BOOT_RAM_DESCRIPTOR: u64 = 0x40000040000605;

/// Create a level 1 block descriptor to map the device
///
/// TODO: Make const
#[allow(dead_code)]
fn make_boot_device_descriptor() -> u64 {
    let phys_addr = Arch::debug_uart()
        .expect("Arch::debug_uart")
        .base()
        .align_down(1 * GB);
    let level = 1;
    let contiguous = false;
    PageBlockDescriptor::new_entry(level, Some(phys_addr), Attributes::DEVICE, contiguous).get()
}

pub const BOOT_DEVICE_DESCRIPTOR: u64 = 0x60000000000601;

/// Iterator over the virtual address ranges implied by entries in a page table.
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

impl core::fmt::Debug for PageTableEntries {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{:?} ({}..{}) {:?}",
            self.bounds, self.index, self.top, self.entry_span
        )
    }
}

/// Generate an iterator for the page table at specific level covering from a specific base
/// where it intersects with target range.
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

    let virt_range_base = virt_range.base().get();
    let first = (virt_range_base >> level_offset) & ((1 << LEVEL_WIDTH) - 1);
    let entries = (virt_range.length() + ((1 << level_offset) - 1)) >> level_offset;

    let span = VirtAddrRange::new(
        base.increment(first << level_offset),
        1usize << level_offset,
    );

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::archs::aarch64::Arch;
    use crate::pager::{FixedOffset, Identity};
    use crate::util::result::Error::OutOfMemory;

    #[test]
    fn test_table_entries() {
        let base = VirtAddr::at(0x10000);
        let range = VirtAddrRange::new(base, 0x10000);
        {
            let mut i = table_entries(range, 1, VirtAddr::at(0));
            let (index, virt_range, entry_range) = i.next().unwrap();
            assert_eq!(0, index);
            assert_eq!(range, virt_range);
            assert_eq!(
                VirtAddrRange::between(VirtAddr::at(0x0), VirtAddr::at(0x4000_0000)),
                entry_range
            );
            assert_none!(i.next());
        }
        {
            let mut i = table_entries(range, 2, VirtAddr::at(0));
            let (index, virt_range, entry_range) = i.next().unwrap();
            assert_eq!(0, index);
            assert_eq!(range, virt_range);
            assert_eq!(
                VirtAddrRange::between(VirtAddr::at(0x0), VirtAddr::at(0x20_0000)),
                entry_range
            );
            assert_none!(i.next());
        }
        {
            let mut i = table_entries(range, 3, VirtAddr::at(0));
            let (index, virt_range, entry_range) = i.next().unwrap();
            let range = VirtAddrRange::between(VirtAddr::at(0x1_0000), VirtAddr::at(0x1_1000));
            assert_eq!(16, index);
            assert_eq!(range, virt_range);
            assert_eq!(range, entry_range);
            let (index, virt_range, entry_range) = i.next().unwrap();
            let range = range.step();
            assert_eq!(17, index);
            assert_eq!(range, virt_range);
            assert_eq!(range, entry_range);
            for _ in 18..32 {
                assert_some!(i.next());
            }
            assert_none!(i.next());
        }
    }

    use crate::archs::pager::PageDirectory;
    use crate::pager::Page;

    struct TestAllocator {
        next: usize,
        pages: [Page; 6],
    }

    impl TestAllocator {
        pub fn new(length: usize) -> Self {
            assert!(length <= 6);
            Self {
                next: 6 - length,
                pages: [Page::new(); 6],
            }
        }
    }

    impl FrameAllocator for TestAllocator {
        fn alloc_zeroed(&mut self, _purpose: FramePurpose) -> Result<PhysAddr> {
            if self.next > 5 {
                return Err(OutOfMemory);
            }
            let result = Identity::new().translate(VirtAddr::from(&self.pages[self.next]));
            self.next += 1;
            Ok(result)
        }
    }

    #[test]
    fn test_mapping() {
        let mut page_dir = super::PageDirectory::new();
        assert_none!(page_dir.ttb0);
        assert_none!(page_dir.ttb1);

        let base = Arch::kernel_base();
        let target_range = VirtAddrRange::new(base, 0x1000);
        let translation = FixedOffset::new(PhysAddr::null(), base);
        let attributes = Attributes::DEVICE;
        let allocator = Locked::new(TestAllocator::new(3));
        let mem_access_translation = Identity::new();

        assert_ok!(page_dir.map_translation(
            target_range,
            translation,
            attributes,
            &allocator,
            &mem_access_translation,
        ));

        page_dir.dump(&mem_access_translation);
    }

    #[test]
    fn test_attribute_compatibility() {
        // TODO: test that later allocations which rely on l1 PTE settings
        // are not incompatible.
    }

    #[test]
    fn test_demand_mapping() {
        let mut page_dir = super::PageDirectory::new();
        let base = Arch::kernel_base();
        let target_range = VirtAddrRange::new(base, 0x10_0000_0000);
        let translation = FixedOffset::new(PhysAddr::null(), base);
        let attributes = Attributes::KERNEL_DATA;
        let allocator = Locked::new(TestAllocator::new(1));
        let mem_access_translation = Identity::new();

        assert_ok!(page_dir.map_translation(
            target_range,
            translation,
            attributes,
            &allocator,
            &mem_access_translation,
        ));
    }

    #[test]
    fn test_boot_descriptors() {
        assert_eq!(make_boot_ram_descriptor(), BOOT_RAM_DESCRIPTOR);
        assert_eq!(make_boot_device_descriptor(), BOOT_DEVICE_DESCRIPTOR);
    }
}
