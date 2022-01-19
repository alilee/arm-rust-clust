// SPDX-License-Identifier: Unlicense

mod deque;

use crate::archs::{arch::Arch, PagerTrait};
use crate::device;
use crate::util::locked::Locked;
use crate::{Error, Error::Unimplemented, Result};

use super::{
    layout, layout::RangeContent, mem_translation, AddrRange, PhysAddr, PhysAddrRange, Translate,
    VirtAddr, PAGESIZE_BYTES,
};

use core::default::Default;
use core::fmt::{Debug, Formatter};
use core::num::NonZeroU64;
use core::ptr::NonNull;

/// The use for an allocated page.
///
/// Determines the list the page is on, and enables scanning tasks, for example page table
/// Access Flag reset.
#[derive(Copy, Clone, PartialEq, Debug)]
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

#[derive(Copy, Clone, Debug)]
#[repr(u8)]
#[allow(dead_code)]
enum FrameUse {
    Zero,               // blank read-only page of zeros for all processes to shared, ie. bss.
    Zeroed,   // reservoir of zero pages for when processes write to pages for the first time.
    Zeroing,  // owned by page cleaner while cleaning
    Free,     // returned after use when no longer needed
    UserWarm, // recently-accessed user pages
    UserCold, // less recently-accessed user pages
    Kernel,   // kernel pages (always resident)
    Nailed,   // special user pages that must not be evicted, ie. shared with device
    FrameTable, // holds the frame table
    LeafPageTable, // the L3 page tables
    BranchPageTable, // the L0-2 page tables
    DirectMemoryAccess, // pool to supply contiguous pages of physical memory
}

impl From<u8> for FrameUse {
    fn from(i: u8) -> Self {
        match i {
            i if i == FrameUse::Zero as u8 => FrameUse::Zero,
            i if i == FrameUse::Zeroed as u8 => FrameUse::Zeroed,
            i if i == FrameUse::Zeroing as u8 => FrameUse::Zeroing,
            i if i == FrameUse::Free as u8 => FrameUse::Free,
            i if i == FrameUse::UserWarm as u8 => FrameUse::UserWarm,
            i if i == FrameUse::UserCold as u8 => FrameUse::UserCold,
            i if i == FrameUse::Kernel as u8 => FrameUse::Kernel,
            i if i == FrameUse::Nailed as u8 => FrameUse::Nailed,
            i if i == FrameUse::FrameTable as u8 => FrameUse::FrameTable,
            i if i == FrameUse::LeafPageTable as u8 => FrameUse::LeafPageTable,
            i if i == FrameUse::BranchPageTable as u8 => FrameUse::BranchPageTable,
            i if i == FrameUse::DirectMemoryAccess as u8 => FrameUse::DirectMemoryAccess,
            _ => unreachable!(),
        }
    }
}

impl Into<u8> for FrameUse {
    fn into(self) -> u8 {
        self as u8
    }
}

#[derive(Debug)]
pub struct FrameTableEntry {
    _persisted: Option<NonZeroU64>,
    _page_block_descriptor: Option<NonNull<crate::archs::arch::PageBlockDescriptor>>,
}

impl Default for FrameTableEntry {
    fn default() -> Self {
        Self {
            _persisted: None,
            _page_block_descriptor: None,
        }
    }
}

#[derive(Debug)]
pub struct FrameTableInner {
    table: deque::Deque<FrameTableEntry, FrameUse>,
    user_count: u32,
    user_warm_count: u32,
    ram_range: PhysAddrRange, // range of ram to be managed
}

impl FrameTableInner {
    fn move_contiguous_range(
        &mut self,
        phys_addr_range: PhysAddrRange,
        frame_use: FrameUse,
    ) -> Result<()> {
        let i = PhysAddrRange::between(Arch::ram_range()?.base(), phys_addr_range.base())
            .length_in_pages() as u32;
        let j = i + phys_addr_range.length_in_pages() as u32 - 1;
        self.table.remove_seq_to(i, j, frame_use)?;
        Ok(())
    }
}

impl Allocator for FrameTableInner {
    fn alloc_zeroed(&mut self, purpose: Purpose) -> Result<PhysAddr> {
        self.table
            .drip_to(FrameUse::Zeroed, purpose.into())
            .map(|i| {
                if purpose == Purpose::User {
                    self.user_count += 1;
                    self.user_warm_count += 1;
                }
                i
            })
            .map(|i| PhysAddr::ram_page(i as usize))
            .or_else(|_| {
                self.alloc_for_overwrite(purpose).and_then(|phys_addr| {
                    let page: *mut u8 = mem_translation().translate_phys(phys_addr)?.into();
                    unsafe {
                        core::ptr::write_bytes(page, 0, PAGESIZE_BYTES);
                    }
                    Ok(phys_addr)
                })
            })
    }

    fn alloc_for_overwrite(&mut self, purpose: Purpose) -> Result<PhysAddr> {
        self.table
            .drip_to(FrameUse::Free, purpose.into())
            .map(|i| {
                if purpose == Purpose::User {
                    self.user_count += 1;
                }
                i
            })
            .or_else(|_| {
                // evict
                Err(Unimplemented)
            })
            .map(|i| {
                if purpose == Purpose::User {
                    self.user_warm_count += 1;
                }
                i
            })
            .map(|i| PhysAddr::ram_page(i as usize))
    }
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

static mut ALLOCATOR: Locked<FrameTable> = Locked::new(FrameTable(None));

impl From<Purpose> for FrameUse {
    fn from(purpose: Purpose) -> Self {
        match purpose {
            Purpose::User => FrameUse::UserWarm,
            Purpose::Kernel => FrameUse::Kernel,
            Purpose::LeafPageTable => FrameUse::LeafPageTable,
            Purpose::BranchPageTable => FrameUse::BranchPageTable,
            Purpose::DirectMemoryAccess => FrameUse::DirectMemoryAccess,
        }
    }
}

pub fn frame_table_bytes() -> usize {
    let ram_range: PhysAddrRange = Arch::ram_range().expect("Arch::ram_range");
    let entries = ram_range.length_in_pages();

    deque::Deque::<FrameTableEntry, FrameUse>::storage_bytes(entries)
}

/// Initialise the frame table
///
/// Take the first n pages after the kernel image as a frame table.
pub fn init() -> Result<()> {
    major!("init");

    // paging is not enabled yet
    let frame_table_range = layout::get_phys_range(RangeContent::FrameTable)?;

    debug!("frame_table_range: {:?}", frame_table_range);

    let ram_range: PhysAddrRange = Arch::ram_range()?;
    let len = ram_range.length_in_pages() as u32;

    let mut frame_table = unsafe {
        let frame_table_ptr: *mut u8 = VirtAddr::identity_mapped(frame_table_range.base()).into();
        FrameTableInner {
            table: deque::Deque::<FrameTableEntry, FrameUse>::new(
                frame_table_ptr,
                len,
                FrameUse::Free,
            ),
            user_count: 0,
            user_warm_count: 0,
            ram_range,
        }
    };

    frame_table.move_contiguous_range(frame_table_range, FrameUse::FrameTable)?;

    let reset_stack_range = layout::get_phys_range(RangeContent::ResetStack)?;
    frame_table.move_contiguous_range(reset_stack_range, FrameUse::Kernel)?;

    let text_range: PhysAddrRange = Arch::text_image();
    frame_table.move_contiguous_range(text_range, FrameUse::Kernel)?;

    let static_range: PhysAddrRange = Arch::static_image();
    frame_table.move_contiguous_range(static_range, FrameUse::Kernel)?;

    let data_range: PhysAddrRange = Arch::data_image();
    frame_table.move_contiguous_range(data_range, FrameUse::Kernel)?;

    unsafe {
        if let Some(dtb_range) = device::PDTB {
            frame_table.move_contiguous_range(dtb_range, FrameUse::Kernel)?;
        }
    }

    unsafe {
        let mut lock = ALLOCATOR.lock();
        *lock = FrameTable(Some(frame_table));

        info!("{:?}", *lock);
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

    let frame_table_ptr: *mut u8 = virt_addr.base().into();
    let inner = lock.inner()?;
    let len = inner.ram_range.length_in_pages() as u32;
    inner.table.repoint_table(frame_table_ptr, len)
}

impl Allocator for FrameTable {
    fn alloc_zeroed(&mut self, purpose: Purpose) -> Result<PhysAddr> {
        info!("alloc_zeroed: {:?}", purpose);
        self.inner()?.alloc_zeroed(purpose)
    }

    fn alloc_for_overwrite(&mut self, purpose: Purpose) -> Result<PhysAddr> {
        info!("alloc_zeroed: {:?}", purpose);
        self.inner()?.alloc_for_overwrite(purpose)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() {
        let mut alloc = FrameTable(None);
        trace!("{:?}", alloc);
        assert_err!(alloc.alloc_zeroed(Purpose::User));
    }
}
