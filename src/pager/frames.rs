// SPDX-License-Identifier: Unlicense

use crate::util::locked::Locked;
use crate::{Error, Result};

use super::{Addr, AddrRange, PhysAddr, PhysAddrRange, VirtAddr, PAGESIZE_BYTES};

use enum_map::{Enum, EnumMap};

use core::default::Default;
use core::fmt::{Debug, Formatter};

/// The use for an allocated page.
///
/// Determines the list the page is on, and enables scanning tasks, for example page table
/// Access Flag reset.
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
        write!(f, "FrameDeque {{ head: {:?}, tail: {:?}, count: {} }}", self.head, self.tail, self.count)
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

    pub fn move_range(
        &mut self,
        i: usize,
        count: usize,
        table: &mut [FrameTableNode],
    ) -> Result<()> {
        assert!(!FrameTableNode::is_clear(table[i].prev()));
        let prev = table[i].prev();
        let next = table[i + count].next();
        table[prev].set_next(next);
        table[next].set_prev(prev);
        match self.tail {
            None => {
                assert_eq!(self.count, 0);
                assert_none!(self.head);
                table[i].clear_prev();
                self.head = Some(i);
            }
            Some(tail) => {
                assert_ne!(self.count, 0);
                assert!(FrameTableNode::is_clear(table[tail].next()));
                table[tail].set_next(i);
                table[i].set_prev(tail);
            }
        }
        self.tail = Some(i + count);
        self.count += count;
        Ok(())
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

#[repr(C)]
pub struct FrameTable {
    ram_base: PhysAddr,
    maybe_frame_lists: Option<EnumMap<FrameUse, FrameDeque>>,
    maybe_table: Option<FrameTableNodes>,
}

impl Debug for FrameTable {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        use core::mem::size_of_val;

        writeln!(f, "FrameTable {{ ").unwrap();
        writeln!(f, "                                                                   ram_base: {:?},", self.ram_base).unwrap();
        match &self.maybe_frame_lists {
            None => writeln!(f, "                                                                   lists: {:?}", self.maybe_frame_lists).unwrap(),
            Some(frame_lists) => {
                for (frame_use, _) in frame_lists.iter() {
                    let table = self.maybe_table.as_deref().unwrap();
                    let frame_lists = self.maybe_frame_lists.as_ref().unwrap();
                    let frame_list = &frame_lists[frame_use];
                    if frame_list.count > 0 {
                        writeln!(f, "                                                                   {:>15?} ({}):", frame_use, frame_list.count).unwrap();
                        write!(f, "                                                                       [").unwrap();
                        let mut i = frame_list.head.unwrap_or(u32::MAX as usize);
                        let mut count = 20;
                        while i != u32::MAX as usize {
                            if count == 0 {
                                write!(f, "...").unwrap();
                                break;
                            }
                            write!(f, "{}, ", i).unwrap();
                            i = table[i].next();
                            count -= 1;
                        }
                        writeln!(f, "]").unwrap();
                    }
                }
            },
        }
        if let Some(_) = self.maybe_table {
            writeln!(f, "                                                                   size_of(table): {:?} bytes", size_of_val(*self.maybe_table.as_ref().unwrap())).unwrap();
        }
        writeln!(f, "                                                               }}")
    }
}

pub static ALLOCATOR: Locked<FrameTable> = Locked::new(FrameTable::null());

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
struct FrameTableNode {
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

/// Initialise
///
/// Take the first n pages as a frame table.
pub fn init() -> Result<()> {
    use crate::archs::{arch::Arch, ArchTrait};

    log!("MAJOR", "init");

    let ram_range = Arch::ram_range()?;
    let len = ram_range.length_in_pages();
    let image_range = PhysAddrRange::boot_image();
    let frame_table_range = PhysAddrRange::new(
        image_range.top(),
        len * core::mem::size_of::<FrameTableNode>() / PAGESIZE_BYTES,
    );

    let mut lock = ALLOCATOR.lock();

    unsafe {
        let frame_table_ptr: *mut FrameTableNode =
            VirtAddr::identity_mapped(frame_table_range.base()).into();
        *lock = FrameTable::new(
            core::slice::from_raw_parts_mut(frame_table_ptr, len),
            ram_range.base(),
        );
    }

    (*lock).move_range(image_range, FrameUse::KernelHot)?;
    (*lock).move_range(frame_table_range, FrameUse::KernelHot)?;

    Ok(())
}

impl FrameTable {
    const fn null() -> Self {
        let result = Self {
            ram_base: PhysAddr::null(),
            maybe_table: None,
            maybe_frame_lists: None,
        };
        result
    }

    fn new(frame_table: FrameTableNodes, ram_base: PhysAddr) -> Self {
        let mut frame_lists: EnumMap<FrameUse, FrameDeque> = EnumMap::new();
        frame_lists[FrameUse::Free].reset(frame_table).expect("FrameDeque::reset");
        let result = Self {
            ram_base,
            maybe_table: Some(frame_table),
            maybe_frame_lists: Some(frame_lists),
        };
        result
    }

    /// Avoids E0499 by getting two mutable references from sub-parts.
    ///
    /// Note: https://stackoverflow.com/questions/31281155/cannot-borrow-x-as-mutable-more-than-once-at-a-time
    fn table_frame_list(
        &mut self,
        frame_use: FrameUse,
    ) -> Result<(&mut [FrameTableNode], &mut FrameDeque)> {
        let table = self.maybe_table.as_deref_mut().ok_or(Error::UnInitialised)?;
        let frame_lists = self.maybe_frame_lists.as_mut().ok_or(Error::UnInitialised)?;
        let frame_list = &mut frame_lists[frame_use];

        Ok((table, frame_list))
    }

    fn move_range(&mut self, phys_addr_range: PhysAddrRange, frame_use: FrameUse) -> Result<()> {
        let count = phys_addr_range.length_in_pages();
        let i = phys_addr_range.base().offset_above(self.ram_base) / PAGESIZE_BYTES;

        let (table, frame_list) = self.table_frame_list(frame_use)?;
        frame_list.move_range(i, count, table)
    }

    fn push(&mut self, i: usize, frame_use: FrameUse) -> Result<()> {
        let (table, frame_list) = self.table_frame_list(frame_use)?;
        frame_list.push(i, table)
    }

    fn pop(&mut self, frame_use: FrameUse) -> Result<usize> {
        let (table, frame_list) = self.table_frame_list(frame_use)?;
        frame_list.pop(table)
    }
}

impl Allocator for FrameTable {
    fn alloc_zeroed(&mut self, purpose: Purpose) -> Result<PhysAddr> {
        let i = self.pop(FrameUse::Zeroed).or(Err(Error::OutOfMemory))?;
        self.push(i, purpose.into())?;

        let phys_addr = self.ram_base.increment(i * PAGESIZE_BYTES);
        Ok(phys_addr)
    }

    fn alloc_for_overwrite(&mut self, purpose: Purpose) -> Result<PhysAddr> {
        let i = self.pop(FrameUse::Free).or(Err(Error::OutOfMemory))?;
        self.push(i, purpose.into())?;

        let phys_addr = self.ram_base.increment(i * PAGESIZE_BYTES);
        Ok(phys_addr)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() {
        let mut alloc = ALLOCATOR.lock();
        trace!("{:?}", *alloc);
        assert_err!(alloc.alloc_zeroed(Purpose::User));
    }

    const FRAME_TABLE_LENGTH: usize = 3;
    static mut FRAME_TABLE_NODES: [FrameTableNode; FRAME_TABLE_LENGTH] = [FrameTableNode::null(); FRAME_TABLE_LENGTH];

    #[test]
    fn allocating_same() {
        {
            let mut lock = ALLOCATOR.lock();
            unsafe {
                *lock = FrameTable::new(&mut FRAME_TABLE_NODES, PhysAddr::at(0x4000_0000));
            }
        }
        unsafe {
            assert_eq!((&mut FRAME_TABLE_NODES).len(), FRAME_TABLE_LENGTH);
        }
        {
            use Purpose::*;
            let mut alloc = ALLOCATOR.lock();
            assert_err!(alloc.alloc_zeroed(User));
            assert_ok!(alloc.alloc_for_overwrite(Kernel));
            assert_ok!(alloc.alloc_for_overwrite(Kernel));
            assert_ok!(alloc.alloc_for_overwrite(Kernel));
            assert_err!(alloc.alloc_for_overwrite(LeafPageTable));
            trace!("{:?}", *alloc);
        }
    }
}
