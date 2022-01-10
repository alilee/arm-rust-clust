// SPDX-License-Identifier: Unlicense

use crate::util::locked::Locked;
use crate::{Error, Result};

use super::{
    mem_translation, Addr, AddrRange, PhysAddr, PhysAddrRange, Translate, VirtAddr, PAGESIZE_BYTES,
    {layout, layout::RangeContent},
};

use enum_map::{Enum, EnumMap};

use core::default::Default;
use core::fmt::{Debug, Formatter};

/// The use for an allocated page.
///
/// Determines the list the page is on, and enables scanning tasks, for example page table
/// Access Flag reset.
#[derive(Copy, Clone, Debug)]
pub enum Purpose {
    /// Page is for a user process.
    User,
    /// Page is for kernel code or data.
    Kernel,
    /// Page is for a leaf page table (which should be access scanned).
    LeafPageTable,
    /// Page is for a branch page table.
    BranchPageTable,
    /// Page is for DMA (and should be nailed)..
    DirectMemoryAccess,
}

/// Ability to provide an unused frame.
pub trait Allocator {
    /// Reserve and return a zeroed frame for a given purpose.
    fn alloc_zeroed(&mut self, purpose: Purpose) -> Result<PhysAddr>;

    /// Reserve and return a frame for a given purpose, which must be overwritten.
    fn alloc_for_overwrite(&mut self, purpose: Purpose) -> Result<PhysAddr> {
        self.alloc_zeroed(purpose)
    }
}

/// Threaded double-ended queues with O(1) for all operations, including detach.
///
/// O(n) for: is element in deque.
struct FrameDeque {
    head: Option<usize>,
    tail: Option<usize>,
    count: usize,
}

impl Default for FrameDeque {
    fn default() -> Self {
        Self {
            head: None,
            tail: None,
            count: 0,
        }
    }
}

impl Debug for FrameDeque {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "FrameDeque {{ head: {:?}, tail: {:?}, count: {} }}",
            self.head, self.tail, self.count
        )
    }
}

impl FrameDeque {
    pub fn reset(&mut self, table: &mut [FrameTableNode]) -> Result<()> {
        let last = table.len() - 1;
        for (i, entry) in table.iter_mut().enumerate() {
            if i == 0 {
                entry.clear_prev();
            } else {
                entry.set_prev(i - 1);
            }
            if i == last {
                entry.clear_next();
            } else {
                entry.set_next(i + 1);
            }
        }
        self.head = Some(0);
        self.tail = Some(last);
        self.count = table.len();
        Ok(())
    }

    pub fn remove_section(&mut self, i: usize, count: usize, table: &mut [FrameTableNode]) -> () {
        debug!("remove_section: {:?}, {:?}", i, count);

        let before = table[i].prev();
        let end = i + count - 1;
        let after = table[end].next();

        if self.head != Some(i) {
            table[before].set_next(after);
            table[i].clear_prev();
        } else {
            if self.tail == Some(end) {
                self.head = None;
            } else {
                self.head = Some(after);
                table[after].clear_prev();
            }
        }

        if self.tail != Some(end) {
            table[after].set_prev(before);
            table[end].clear_next();
        } else {
            if self.head == Some(i) {
                self.tail = None;
            } else {
                self.tail = Some(before);
                table[before].clear_next();
            }
        }

        self.count -= count;

        assert!(FrameTableNode::is_clear(table[i].prev()));
        assert!(FrameTableNode::is_clear(table[end].next()));
        assert!(self.head.is_none() || FrameTableNode::is_clear(table[self.head.unwrap()].prev()));
        assert!(self.tail.is_none() || FrameTableNode::is_clear(table[self.tail.unwrap()].next()));
    }

    pub fn append_section(&mut self, i: usize, count: usize, table: &mut [FrameTableNode]) -> () {
        debug!("append_section: {:?}, {:?}", i, count);

        let end = i + count - 1;
        if self.head.is_none() {
            self.head = Some(i);
            self.tail = Some(end);
        } else {
            let old_head = self.head.unwrap();
            table[old_head].set_prev(end);
            table[end].set_next(old_head);
            self.head = Some(i);
        }
        self.count += count;
    }

    pub fn push(&mut self, i: usize, table: &mut [FrameTableNode]) -> Result<()> {
        assert!(FrameTableNode::is_clear(table[i].prev()));
        assert!(FrameTableNode::is_clear(table[i].next()));
        match self.head {
            None => {
                assert!(self.tail.is_none());
                self.tail = Some(i);
            }
            Some(next) => {
                table[i].set_next(next);
                table[next].set_prev(i);
            }
        }
        self.head = Some(i);
        self.count += 1;
        Ok(())
    }

    pub fn pop(&mut self, table: &mut [FrameTableNode]) -> Result<usize> {
        let result = self.head.ok_or(Error::OutOfPages)?;
        assert!(FrameTableNode::is_clear(table[result].prev()));

        if self.count == 1 {
            self.head = None;
            self.tail = None;
        } else {
            let next = table[result].next();
            table[next].clear_prev();
            self.head = Some(next);
            table[result].clear_next();
        }
        self.count -= 1;
        Ok(result)
    }
}

type FrameTableNodes = &'static mut [FrameTableNode];

pub struct FrameTableInner {
    frame_lists: EnumMap<FrameUse, FrameDeque>,
    table: FrameTableNodes,
    ram_range: PhysAddrRange, // range of ram to be managed
}

pub struct FrameTable(Option<FrameTableInner>);

impl FrameTable {
    fn inner(&mut self) -> Result<&mut FrameTableInner> {
        self.0.as_mut().ok_or(Error::UnInitialised)
    }
}

impl Debug for FrameTable {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self.0.as_ref())
    }
}

impl Debug for FrameTableInner {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        use crate::debug::BUFFER;
        use core::mem::size_of_val;

        writeln!(f, "FrameTable {{ ").unwrap();
        writeln!(f, "{}    RAM range: {:?}", BUFFER, self.ram_range).unwrap();
        writeln!(
            f,
            "{}    table address: {:?}",
            BUFFER, &self.table[0] as *const FrameTableNode
        )
        .unwrap();
        writeln!(
            f,
            "{}    size_of(table): {:?} pages",
            BUFFER,
            (PAGESIZE_BYTES + size_of_val(self.table) - 1) / PAGESIZE_BYTES
        )
        .unwrap();
        for (frame_use, _) in self.frame_lists.iter() {
            let frame_list = &self.frame_lists[frame_use];
            if frame_list.count > 0 {
                writeln!(
                    f,
                    "{}    {:>15?} (h: {}, t: {}, c: {}):",
                    BUFFER,
                    frame_use,
                    frame_list.head.unwrap(),
                    frame_list.tail.unwrap(),
                    frame_list.count
                )
                .unwrap();
                write!(f, "{}        [", BUFFER).unwrap();
                let mut i = frame_list.head.unwrap_or(u32::MAX as usize);
                let mut seq_length = 0;
                let mut count_check = 0usize;
                while i != u32::MAX as usize {
                    let next = self.table[i].next();
                    if next == i + 1 {
                        seq_length += 1;
                    }
                    if seq_length < 4 || next != i + 1 {
                        write!(f, "{}, ", i).unwrap();
                    } else if seq_length == 4 {
                        write!(f, "..., ").unwrap();
                    }
                    if next != i + 1 {
                        seq_length = 0;
                        if next == u32::MAX as usize {
                            // assert_eq!(frame_list.tail.unwrap(), i);
                        }
                    }
                    i = next;
                    count_check += 1;
                }
                assert_eq!(count_check, frame_list.count);
                writeln!(f, "]").unwrap();
            }
        }
        write!(f, "{}}}", BUFFER)
    }
}

static mut ALLOCATOR: Locked<FrameTable> = Locked::new(FrameTable(None));

#[derive(Copy, Clone, Debug, Enum)]
enum FrameUse {
    Zeroed,
    Zeroing,
    Free,
    UserHot,
    UserWarm,
    UserCold,
    KernelHot,
    KernelWarm,
    KernelCold,
    Nailed,
    FrameTable,
    LeafPageTable,
    BranchPageTable,
    DirectMemoryAccess,
}

impl From<Purpose> for FrameUse {
    fn from(purpose: Purpose) -> Self {
        match purpose {
            Purpose::User => FrameUse::UserHot,
            Purpose::Kernel => FrameUse::KernelHot,
            Purpose::LeafPageTable => FrameUse::LeafPageTable,
            Purpose::BranchPageTable => FrameUse::BranchPageTable,
            Purpose::DirectMemoryAccess => FrameUse::DirectMemoryAccess,
        }
    }
}

#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub(crate) struct FrameTableNode {
    _prev: u32,
    _next: u32,
}

impl FrameTableNode {
    const CLEAR: u32 = u32::MAX;

    #[allow(dead_code)]
    pub const fn null() -> Self {
        Self { _prev: 0, _next: 0 }
    }

    pub const fn prev(&self) -> usize {
        self._prev as usize
    }

    pub fn set_prev(&mut self, prev: usize) {
        self._prev = prev as u32;
    }

    pub fn clear_prev(&mut self) {
        self._prev = Self::CLEAR;
    }

    pub const fn next(&self) -> usize {
        self._next as usize
    }

    pub fn set_next(&mut self, next: usize) {
        self._next = next as u32;
    }

    pub fn clear_next(&mut self) {
        self._next = Self::CLEAR;
    }

    pub fn is_clear(entry: usize) -> bool {
        Self::CLEAR == entry as u32
    }
}

/// Initialise the frame table
///
/// Take the first n pages after the kernel image as a frame table.
pub fn init() -> Result<()> {
    use crate::archs::{arch::Arch, PagerTrait};

    major!("init");

    // paging is not enabled yet
    let frame_table_range = layout::get_phys_range(RangeContent::FrameTable)?;

    debug!("frame_table_range: {:?}", frame_table_range);

    let ram_range: PhysAddrRange = Arch::ram_range()?;
    let len = ram_range.length_in_pages();

    let mut frame_table = unsafe {
        let frame_table_ptr: *mut FrameTableNode =
            VirtAddr::identity_mapped(frame_table_range.base()).into();
        let result = FrameTableInner::new(
            core::slice::from_raw_parts_mut(frame_table_ptr, len),
            ram_range,
        );
        result
    };

    let reset_stack_range = layout::get_phys_range(RangeContent::ResetStack)?;
    frame_table.move_free_range(reset_stack_range, FrameUse::KernelHot)?;

    let image_range: PhysAddrRange = Arch::boot_image();
    frame_table.move_free_range(image_range, FrameUse::KernelHot)?;

    frame_table.move_free_range(frame_table_range, FrameUse::FrameTable)?;

    unsafe {
        let mut lock = ALLOCATOR.lock();
        *lock = FrameTable(Some(frame_table));

        debug!("{:?}", *lock);
    }

    Ok(())
}

/// Get the frame table allocator.
pub fn allocator() -> &'static Locked<FrameTable> {
    unsafe { &ALLOCATOR }
}

/// Align frame table pointer with re-mapped frame table.
///
/// After reset, the frame table is placed just after the data section. Layout will
/// put it somewhere else in kernel memory so the pointers need to be updated.
pub(in crate::pager) fn repoint_frame_table() -> Result<()> {
    info!("repoint_frame_table");

    let virt_addr = layout::get_range(RangeContent::FrameTable)?;
    let mut lock = allocator().lock();

    let frame_table_ptr: *mut FrameTableNode = virt_addr.base().into();
    let inner = lock.inner()?;
    let len = inner.ram_range.length_in_pages();
    inner.table = unsafe { core::slice::from_raw_parts_mut(frame_table_ptr, len) };

    Ok(())
}

impl FrameTableInner {
    fn new(table: FrameTableNodes, ram_range: PhysAddrRange) -> Self {
        let mut frame_lists: EnumMap<FrameUse, FrameDeque> = EnumMap::default();
        frame_lists[FrameUse::Free]
            .reset(table)
            .expect("FrameDeque::reset");
        Self {
            table,
            frame_lists,
            ram_range,
        }
    }

    fn move_free_range(
        &mut self,
        phys_addr_range: PhysAddrRange,
        frame_use: FrameUse,
    ) -> Result<()> {
        debug!("move_free_range: {:?}, {:?}", phys_addr_range, frame_use);
        let count = phys_addr_range.length_in_pages();
        let phys_addr = phys_addr_range.base();
        let i = phys_addr.offset_above(self.ram_range.base()) / PAGESIZE_BYTES;

        assert_ne!(0, count);
        self.frame_lists[FrameUse::Free].remove_section(i, count, self.table);
        self.frame_lists[frame_use].append_section(i, count, self.table);
        Ok(())
    }

    fn push(&mut self, i: usize, frame_use: FrameUse) -> Result<()> {
        self.frame_lists[frame_use].push(i, self.table)
    }

    fn pop(&mut self, frame_use: FrameUse) -> Result<usize> {
        self.frame_lists[frame_use].pop(self.table)
    }

    fn ram_page(&self, i: usize) -> PhysAddr {
        self.ram_range.base().increment(i * PAGESIZE_BYTES)
    }
}

impl Allocator for FrameTable {
    fn alloc_zeroed(&mut self, purpose: Purpose) -> Result<PhysAddr> {
        info!("alloc_zeroed: {:?}", purpose);
        let inner = self.inner()?;
        let mut zero_required = false;
        let i = inner.pop(FrameUse::Zeroed).or_else(|e| match e {
            Error::OutOfPages => {
                let result = inner.pop(FrameUse::Free);
                zero_required = true;
                result
            }
            _ => Err(e),
        })?;

        let phys_addr = inner.ram_page(i);

        if zero_required {
            debug!("No zeroed free memory. Just-in-time zeroing on alloc.");
            let page: *mut u8 = mem_translation().translate_phys(phys_addr)?.into();
            unsafe {
                core::ptr::write_bytes(page, 0, PAGESIZE_BYTES);
            }
        }

        inner.push(i, purpose.into())?;
        Ok(phys_addr)
    }

    fn alloc_for_overwrite(&mut self, purpose: Purpose) -> Result<PhysAddr> {
        info!("alloc_for_overwrite: {:?}", purpose);

        let inner = self.inner()?;
        let i = inner.pop(FrameUse::Free).or(Err(Error::OutOfMemory))?;
        inner.push(i, purpose.into())?;

        let phys_addr = inner.ram_page(i);
        Ok(phys_addr)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::size_of_val;

    #[test]
    fn empty() {
        let mut alloc = FrameTable(None);
        trace!("{:?}", alloc);
        assert_err!(alloc.alloc_zeroed(Purpose::User));
    }

    #[test]
    fn moving() {
        use crate::pager::Page;

        const FRAME_TABLE_LENGTH: usize = 3;
        static mut FRAME_TABLE_NODES: [FrameTableNode; FRAME_TABLE_LENGTH] =
            [FrameTableNode::null(); FRAME_TABLE_LENGTH];

        let pages = [Page::new(); 3];
        let pages_base = unsafe { PhysAddr::from_ptr(&pages) };
        let ram_range = PhysAddrRange::new(pages_base, size_of_val(&pages));
        trace!("{:?}", ram_range);

        let mut inner = unsafe { FrameTableInner::new(&mut FRAME_TABLE_NODES, ram_range) };
        dbg!(&inner);

        inner
            .move_free_range(
                PhysAddrRange::new(pages_base.increment(0x1000), 0x1000),
                FrameUse::KernelHot,
            )
            .unwrap();
        dbg!(&inner);
    }

    #[test]
    fn allocating_same() {
        use crate::pager::Page;
        use Purpose::*;

        const FRAME_TABLE_LENGTH: usize = 3;
        static mut FRAME_TABLE_NODES: [FrameTableNode; FRAME_TABLE_LENGTH] =
            [FrameTableNode::null(); FRAME_TABLE_LENGTH];

        let pages = [Page::new(); 3];
        let ram_range =
            unsafe { PhysAddrRange::new(PhysAddr::from_ptr(&pages), size_of_val(&pages)) };
        trace!("{:?}", ram_range);

        let alloc = unsafe { FrameTableInner::new(&mut FRAME_TABLE_NODES, ram_range) };

        unsafe {
            assert_eq!((&FRAME_TABLE_NODES).len(), FRAME_TABLE_LENGTH);
        }

        let mut alloc: FrameTable = FrameTable(Some(alloc));

        assert_ok!(alloc.alloc_for_overwrite(Kernel));
        assert_ok!(alloc.alloc_zeroed(User));
        assert_ok!(alloc.alloc_for_overwrite(Kernel));
        assert_err!(alloc.alloc_for_overwrite(LeafPageTable));
        assert_err!(alloc.alloc_zeroed(LeafPageTable));
        trace!("{:?}", alloc);
    }
}
