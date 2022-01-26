// SPDX-License-Identifier: Unlicense

//! Page table walks
//!
//! Cannot use heap, because this is used before kernel heap is mapped.

use super::{PageTable, PageTableEntry, LEVEL_OFFSETS, LEVEL_WIDTH, MAX_LEVELS, TABLE_ENTRIES};

use crate::pager::{Addr, AddrRange, FixedOffset, PhysAddr, Translate, VirtAddr, VirtAddrRange};

use core::ptr::NonNull;

#[derive(Debug)]
pub enum TraversalOrder {
    Preorder,
    Postorder,
}

/// Iterate over a span of entries in Page Table
#[derive(Clone, Copy, Debug)]
struct PageTableWalk {
    level: u8,
    page_table: NonNull<PageTable>,
    index: u16,
    top: u16,
    next_virt_addr_range: Option<VirtAddrRange>, // step this along until disjoint with target
}

impl PageTableWalk {
    fn new(
        level: usize,
        phys_addr_table: PhysAddr,
        target_range: VirtAddrRange,
        mem_access_translation: &impl Translate,
    ) -> Self {
        let page_table = NonNull::new(
            mem_access_translation
                .translate_phys(phys_addr_table)
                .expect("mem access translation")
                .into(),
        )
        .expect("NonNull phys_addr_table");

        let level_offset = LEVEL_OFFSETS[level];

        let target_range_base = target_range.base().get();
        let target_chunk = target_range_base >> level_offset;
        let first = target_chunk & ((1 << LEVEL_WIDTH) - 1);
        let entries = (target_range.length() + ((1 << level_offset) - 1)) >> level_offset;

        let next_virt_addr_range = Some(VirtAddrRange::new(
            VirtAddr::at(target_chunk << level_offset),
            1usize << level_offset,
        ));

        Self {
            level: level as u8,
            page_table,
            index: first as u16,
            top: (first + entries) as u16,
            next_virt_addr_range,
        }
    }

    fn within(
        level: u8,
        target_range: VirtAddrRange,
        pte: &PageTableEntry,
        mem_access_translation: &impl Translate,
    ) -> Self {
        assert!(pte.is_valid() && pte.is_table(level));
        let phys_addr_table = pte.next_level_table_address();
        Self::new(
            level as usize + 1,
            phys_addr_table,
            target_range,
            mem_access_translation,
        )
    }

    fn peek(&mut self) -> Option<(u8, VirtAddrRange, &'static mut PageTableEntry)> {
        if self.index < self.top {
            debug!("next entry: {}", self.index);
            assert_lt!(self.index, TABLE_ENTRIES as u16);
            let next_virt_addr_range = self.next_virt_addr_range.unwrap();
            let pt = unsafe { self.page_table.as_mut() };
            let result = Some((
                self.level,
                next_virt_addr_range,
                &mut pt[self.index as usize],
            ));
            result
        } else {
            debug!("no more entries");
            None
        }
    }
}

impl Iterator for PageTableWalk {
    type Item = (u8, VirtAddrRange, &'static mut PageTableEntry); // (level, entry_range, pte)

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.top {
            debug!("next entry: {}", self.index);
            assert_lt!(self.index, TABLE_ENTRIES as u16);
            let next_virt_addr_range = self.next_virt_addr_range.unwrap();
            let pt = unsafe { self.page_table.as_mut() };
            let result = Some((
                self.level,
                next_virt_addr_range,
                &mut pt[self.index as usize],
            ));
            self.index += 1;
            self.next_virt_addr_range = Some(next_virt_addr_range.step());
            result
        } else {
            debug!("no more entries");
            None
        }
    }
}

/// Iterate through a tree-walk through the page directory.
pub struct PageDirectoryWalk<'a> {
    virt_addr_range: VirtAddrRange,
    traversal_order: TraversalOrder,
    first_level: u8,
    current_level: u8,
    stack: [Option<PageTableWalk>; MAX_LEVELS],
    target_range: VirtAddrRange,
    pte: Option<NonNull<PageTableEntry>>,
    mem_access_translation: &'a FixedOffset,
}

impl<'a> Iterator for PageDirectoryWalk<'a> {
    type Item = (u8, VirtAddrRange, &'a mut PageTableEntry); // (level, entry_range, pte)

    fn next(&mut self) -> Option<Self::Item> {
        info!(
            "next: {:?}, {:?}",
            self.stack[self.current_level as usize], self.virt_addr_range
        );
        match &self.traversal_order {
            TraversalOrder::Preorder => {
                dbg!(self.current_level);
                dbg!(self.first_level);
                dbg!(self.pte);

                if let Some(pte) = self.pte {
                    let pte = unsafe { pte.as_ref() };
                    dbg!(pte);
                    if pte.is_valid() && pte.is_table(self.current_level) {
                        info!("entry is table");
                        dbg!(self.target_range);
                        dbg!(self.virt_addr_range);
                        let target_range = self
                            .target_range
                            .intersection(&self.virt_addr_range)
                            .expect("entry range within target range");
                        dbg!(target_range);
                        let ptw = PageTableWalk::within(
                            self.current_level,
                            target_range,
                            pte,
                            self.mem_access_translation,
                        );
                        self.current_level += 1;
                        self.stack[self.current_level as usize] = Some(ptw);
                    }
                }

                let mut next = self.stack[self.current_level as usize]
                    .as_mut()
                    .unwrap()
                    .next();
                debug!("next in stack[current]: {:?}", next);

                while next.is_none() {
                    debug!("end of table at level: {}", self.current_level);
                    if self.current_level == self.first_level {
                        debug!("end of first table");
                        return None;
                    }
                    self.stack[self.current_level as usize] = None;
                    self.current_level -= 1;
                    next = self.stack[self.current_level as usize]
                        .as_mut()
                        .unwrap()
                        .next();
                }

                let (level, target_range, pte) = next.unwrap();
                self.pte = NonNull::new(pte);
                self.target_range = target_range;

                Some((level, target_range, pte))
            }
            TraversalOrder::Postorder => {
                dbg!(self.current_level);
                dbg!(self.first_level);
                dbg!(self.pte);

                loop {
                    let peek = self.stack[self.current_level as usize]
                        .as_mut()
                        .unwrap()
                        .peek();
                    debug!("next in stack[current]: {:?}", peek);

                    match peek {
                        None => {
                            self.stack[self.current_level as usize] = None;
                            if self.current_level == self.first_level {
                                return None;
                            }
                            self.current_level -= 1;
                            break;
                        }
                        Some((level, entry_range, pte)) => {
                            if pte.is_valid() && pte.is_table(level) {
                                info!("entry is table");
                                dbg!(self.target_range);
                                dbg!(self.virt_addr_range);
                                let target_range = self
                                    .target_range
                                    .intersection(&self.virt_addr_range)
                                    .expect("entry range within target range");
                                dbg!(target_range);
                                let ptw = PageTableWalk::within(
                                    self.current_level,
                                    target_range,
                                    pte,
                                    self.mem_access_translation,
                                );
                                self.current_level += 1;
                                self.stack[self.current_level as usize] = Some(ptw);
                                continue;
                            }
                            break;
                        }
                    }
                }
                self.stack[self.current_level as usize]
                    .as_mut()
                    .unwrap()
                    .next()
            }
        }
    }
}

impl PageDirectoryWalk<'_> {
    pub fn new(
        traversal_order: TraversalOrder,
        virt_addr_range: VirtAddrRange,
        first_level: u8,
        phys_addr_table: PhysAddr,
        mem_access_translation: &FixedOffset,
    ) -> PageDirectoryWalk {
        info!(
            "new: {:?}, {:?}, {}, {:?}",
            traversal_order, virt_addr_range, first_level, phys_addr_table
        );

        let mut stack = [None; 4];

        stack[first_level as usize] = Some(PageTableWalk::new(
            first_level as usize,
            phys_addr_table,
            virt_addr_range,
            mem_access_translation,
        ));

        PageDirectoryWalk {
            virt_addr_range,
            traversal_order,
            first_level,
            current_level: first_level,
            stack,
            target_range: VirtAddrRange::new(VirtAddr::null(), 0),
            pte: None,
            mem_access_translation,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::archs::aarch64::PageBlockDescriptor;
    use crate::archs::aarch64::{pager::table::TableDescriptor, pager::PagerTrait, Arch};
    use crate::pager::{Attributes, Page, PAGESIZE_BYTES};

    const IDENTITY: &FixedOffset = &FixedOffset::identity();

    #[test]
    fn test_compile() {
        assert!(true);
    }

    #[test]
    fn test_table_first() {
        let mut page = Page::new();
        let phys_addr_table = IDENTITY.translate(VirtAddr::from(&page));
        page[0] = 1;

        let one_page = VirtAddrRange::new(VirtAddr::null(), PAGESIZE_BYTES);

        let ptw = PageTableWalk::new(3, phys_addr_table, one_page, IDENTITY);

        for (level, entry_range, pte) in ptw {
            assert_eq!(1, pte.get());
            assert_eq!(3, level);
            assert_eq!(PAGESIZE_BYTES, entry_range.length());
            assert_eq!(one_page, entry_range);
            assert_eq!(
                pte as *const PageTableEntry as *const (),
                &page[0] as *const u64 as *const ()
            );
        }
    }

    #[test]
    fn test_table_last() {
        let mut page = Page::new();
        let phys_addr_table = IDENTITY.translate(VirtAddr::from(&page));
        page[511] = 1;

        let one_page = VirtAddrRange::new(VirtAddr::at(511 * PAGESIZE_BYTES), PAGESIZE_BYTES);

        let ptw = PageTableWalk::new(3, phys_addr_table, one_page, IDENTITY);

        for (level, entry_range, pte) in ptw {
            assert_eq!(1, pte.get());
            assert_eq!(3, level);
            assert_eq!(PAGESIZE_BYTES, entry_range.length());
            assert_eq!(one_page, entry_range);
            assert_eq!(
                pte as *const PageTableEntry as *const (),
                &page[511] as *const u64 as *const ()
            );
        }
    }

    #[test]
    fn test_table_span() {
        let mut page = Page::new();
        let phys_addr_table = IDENTITY.translate(VirtAddr::from(&page));
        page[200] = 1;
        page[201] = 1;
        page[202] = 1;
        page[203] = 1;

        let page_range = VirtAddrRange::new(VirtAddr::at(200 * PAGESIZE_BYTES), 4 * PAGESIZE_BYTES);

        let ptw = PageTableWalk::new(3, phys_addr_table, page_range, IDENTITY);

        for (level, entry_range, pte) in ptw {
            assert_eq!(1, pte.get());
            assert_eq!(3, level);
            assert_eq!(PAGESIZE_BYTES, entry_range.length());
            assert!(page_range.covers(&entry_range));
        }
    }

    #[test]
    fn test_table_complete() {
        let mut page = Page::new();
        let phys_addr_table = IDENTITY.translate(VirtAddr::from(&page));
        for i in 0..512 {
            page[i] = 1;
        }

        let page_range = VirtAddrRange::new(VirtAddr::null(), 512 * PAGESIZE_BYTES);

        let ptw = PageTableWalk::new(3, phys_addr_table, page_range, IDENTITY);

        for (level, entry_range, pte) in ptw {
            assert_eq!(1, pte.get());
            assert_eq!(3, level);
            assert_eq!(PAGESIZE_BYTES, entry_range.length());
            assert!(page_range.covers(&entry_range));
        }
    }

    #[test]
    fn test_branch_table_span() {
        let mut page = Page::new();
        let phys_addr_table = IDENTITY.translate(VirtAddr::from(&page));

        for target_level in 0..LEVEL_OFFSETS.len() {
            dbg!(target_level);
            let level_offset = LEVEL_OFFSETS[target_level];
            dbg!(level_offset);
            let page_range =
                VirtAddrRange::new(VirtAddr::at(0x123456 << level_offset), 4 << level_offset);
            dbg!(page_range);

            let ptw = PageTableWalk::new(target_level, phys_addr_table, page_range, IDENTITY);
            dbg!(ptw);

            let mut count = 0;
            for (level, entry_range, pte) in ptw {
                dbg!(pte as *const PageTableEntry);
                pte.set(pte.get() + 1);
                unsafe {
                    dbg!(*(pte as *const PageTableEntry));
                }
                assert_eq!(level as usize, target_level);
                assert_eq!(1 << level_offset, entry_range.length());
                dbg!(entry_range);
                assert!(page_range.covers(&entry_range));
                count += 1;
            }
            assert_eq!(4, count);
        }

        for i in 0..512 {
            if i >= 86 && i <= 89 {
                dbg!(i);
                dbg!(page[i]);
                assert_eq!(4, page[i]);
            } else {
                assert_eq!(0, page[i]);
            }
        }
    }

    #[test]
    fn test_sub_table_span() {
        let mut page = Page::new();
        let phys_addr_table = IDENTITY.translate(VirtAddr::from(&page));
        dbg!(phys_addr_table);

        let branch_level = 2;
        let level_offset = LEVEL_OFFSETS[branch_level + 1];
        dbg!(level_offset);
        let target_range =
            VirtAddrRange::new(VirtAddr::at(0x123456 << level_offset), 4 * PAGESIZE_BYTES);
        dbg!(target_range);

        let pte = TableDescriptor::new_entry(
            false,
            branch_level as u8,
            Some(phys_addr_table),
            Attributes::KERNEL_DATA,
        )
        .into();
        let ptw = PageTableWalk::within(branch_level as u8, target_range, &pte, IDENTITY);
        dbg!(ptw);

        let mut count = 0;
        for (level, entry_range, pte) in ptw {
            dbg!(pte as *const PageTableEntry);
            pte.set(pte.get() + 1);
            unsafe {
                dbg!(*(pte as *const PageTableEntry));
            }
            assert_eq!(branch_level + 1, level as usize);
            assert_eq!(PAGESIZE_BYTES, entry_range.length());
            dbg!(entry_range);
            assert!(target_range.covers(&entry_range));
            count += 1;
        }
        assert_eq!(4, count);

        for i in 0..512 {
            if i >= 86 && i <= 89 {
                dbg!(i);
                dbg!(page[i]);
                assert_eq!(1, page[i]);
            } else {
                assert_eq!(0, page[i]);
            }
        }
    }

    #[test]
    fn test_branch_table_span_kernel() {
        let mut page = Page::new();
        let phys_addr_table = IDENTITY.translate(VirtAddr::from(&page));

        for target_level in 1..LEVEL_OFFSETS.len() {
            dbg!(target_level);
            let level_offset = LEVEL_OFFSETS[target_level];
            dbg!(level_offset);
            dbg!(Arch::kernel_base());
            let page_range = VirtAddrRange::new(
                Arch::kernel_base().increment(200 << level_offset),
                4 << level_offset,
            );
            dbg!(page_range);

            let ptw = PageTableWalk::new(target_level, phys_addr_table, page_range, IDENTITY);
            dbg!(ptw);

            let mut count = 0;
            for (level, entry_range, pte) in ptw {
                dbg!(pte as *const PageTableEntry);
                pte.set(pte.get() + 1);
                unsafe {
                    dbg!(*(pte as *const PageTableEntry));
                }
                assert_eq!(level as usize, target_level);
                assert_eq!(1 << level_offset, entry_range.length());
                dbg!(entry_range);
                assert!(page_range.covers(&entry_range));
                count += 1;
            }
            assert_eq!(4, count);
        }

        for i in 0..512 {
            if i >= 200 && i <= 203 {
                dbg!(i);
                dbg!(page[i]);
                assert_eq!(3, page[i]);
            } else {
                assert_eq!(0, page[i]);
            }
        }
    }

    #[test]
    fn test_left() {
        let one_page = VirtAddrRange::new(VirtAddr::null(), PAGESIZE_BYTES);

        let pages = [PageTable::new(); 4];
        let mut allocated = 0;
        let phys_addr_table = IDENTITY.translate(VirtAddr::from(&pages[allocated]));
        allocated += 1;

        for (level, virt_addr_range, pte) in PageDirectoryWalk::new(
            TraversalOrder::Preorder,
            one_page,
            0,
            phys_addr_table,
            &IDENTITY,
        ) {
            info!(
                "visiting: ({:?}, {:?}, {:?}@{:?})",
                level, virt_addr_range, pte, pte as *const PageTableEntry
            );
            if level < 3 {
                let new_page_table = IDENTITY.translate(VirtAddr::from(&pages[allocated]));
                allocated += 1;
                let td = TableDescriptor::new_entry(
                    false,
                    level,
                    Some(new_page_table),
                    Attributes::USER_DATA,
                );
                dbg!(td);
                *pte = td.into();
            } else {
                *pte =
                    PageBlockDescriptor::new_entry(level, None, Attributes::USER_RWX, false).into();
            }
            dbg!(*pte);
        }

        for i in 0..4 {
            dbg!(i);
            let pte = pages[i as usize][0];
            dbg!(pte);
            if i < 3 {
                assert!(pte.is_valid());
                assert!(pte.is_table(i));
                assert!(pte.next_level_table_address() != PhysAddr::null());
            } else {
                assert!(!pte.is_null());
                assert!(!pte.is_valid());
                assert!(!pte.is_table(i));
            }
        }
    }

    #[test]
    fn test_right() {
        let top = 1usize << (LEVEL_OFFSETS[0] + LEVEL_WIDTH);
        dbg!(top as *const ());
        let one_page =
            VirtAddrRange::new(VirtAddr::at(top).decrement(PAGESIZE_BYTES), PAGESIZE_BYTES);
        dbg!(one_page);

        let pages = [PageTable::new(); 4];
        let mut allocated = 0;
        let root_page_table = IDENTITY.translate(VirtAddr::from(&pages[allocated]));
        allocated += 1;

        for (level, virt_addr_range, pte) in PageDirectoryWalk::new(
            TraversalOrder::Preorder,
            one_page,
            0,
            root_page_table,
            &IDENTITY,
        ) {
            info!(
                "visiting: ({:?}, {:?}, {:?}@{:?})",
                level, virt_addr_range, pte, pte as *const PageTableEntry
            );
            if level < 3 {
                let new_page_table = IDENTITY.translate(VirtAddr::from(&pages[allocated]));
                allocated += 1;
                let td = TableDescriptor::new_entry(
                    false,
                    level,
                    Some(new_page_table),
                    Attributes::USER_DATA,
                );
                dbg!(td);
                *pte = td.into();
            } else {
                *pte =
                    PageBlockDescriptor::new_entry(level, None, Attributes::USER_RWX, false).into();
            }
            dbg!(*pte);
        }

        for i in 0..4 {
            dbg!(i);
            let pte = &pages[i as usize][511];
            dbg!(pte);
            if i < 3 {
                assert!(pte.is_valid());
                assert!(pte.is_table(i));
                assert!(pte.next_level_table_address() != PhysAddr::null());
            } else {
                assert!(!pte.is_null());
                assert!(!pte.is_valid());
                assert!(!pte.is_table(i));
            }
        }
    }

    #[test]
    fn test_right_kernel() {
        let top = 1u128 << 64;
        let one_page = VirtAddrRange::new(
            VirtAddr::at((top - PAGESIZE_BYTES as u128) as usize),
            PAGESIZE_BYTES,
        );
        dbg!(one_page);

        let pages = [PageTable::new(); 4];
        let mut allocated = 1;
        let root_page_table = IDENTITY.translate(VirtAddr::from(&pages[allocated]));
        allocated += 1;

        for (level, virt_addr_range, pte) in PageDirectoryWalk::new(
            TraversalOrder::Preorder,
            one_page,
            1,
            root_page_table,
            &IDENTITY,
        ) {
            info!(
                "visiting: ({:?}, {:?}, {:?}@{:?})",
                level, virt_addr_range, pte, pte as *const PageTableEntry
            );
            if level < 3 {
                let new_page_table = IDENTITY.translate(VirtAddr::from(&pages[allocated]));
                allocated += 1;
                let td = TableDescriptor::new_entry(
                    true,
                    level,
                    Some(new_page_table),
                    Attributes::USER_DATA,
                );
                dbg!(td);
                *pte = td.into();
            } else {
                *pte =
                    PageBlockDescriptor::new_entry(level, None, Attributes::USER_RWX, false).into();
            }
            dbg!(*pte);
        }

        for i in 1..4 {
            dbg!(i);
            let pte = &pages[i as usize][511];
            dbg!(pte as *const PageTableEntry);
            dbg!(pte);
            if i < 3 {
                assert!(pte.is_valid());
                assert!(pte.is_table(i));
                assert!(pte.next_level_table_address() != PhysAddr::null());
            } else {
                assert!(!pte.is_null());
                assert!(!pte.is_valid());
                assert!(!pte.is_table(i));
            }
        }
    }
}
