// SPDX-License-Identifier: Unlicense

//! Managing virtual address space, address translation and page faults.

mod addr;
mod attributes;
mod bump;
mod frames;
mod handlers;
mod layout;
mod owned;
mod page;
mod phys_addr;
mod translation;
mod virt_addr;

#[cfg(not(test))]
mod alloc;

pub use addr::*;
pub use attributes::*;
pub use handlers::*;
pub use owned::*;
pub use page::*;
pub use phys_addr::*;
pub use translation::*;
pub use virt_addr::*;

pub use frames::allocator as frame_allocator;
pub use frames::Allocator as FrameAllocator;
pub use frames::Purpose as FramePurpose;

pub use layout::{get_range, mem_fixed_offset, mem_translation, RangeContent};

use crate::archs::{arch, arch::Arch, DeviceTrait, PageDirectory, PagerTrait};
use crate::debug::Level;
use crate::pager::bump::PageBumpAllocator;
use crate::pager::frames::Purpose;
use crate::util::locked::Locked;
use crate::Result;

use ::alloc::sync::Arc;

/// Major interface trait of paging module.
pub trait Paging {
    /// Map a memory-mapped IO range to an unused kernel address range in the
    /// device range, with access attributes for device memory.
    fn map_device(phys_addr_range: PhysAddrRange) -> Result<Arc<OwnedMapping>>;
    /// Map contiguous physical pages to an unused device memory range in the
    /// with access attributes for device memory.
    fn map_dma(contiguous_pages: u8) -> Result<Arc<OwnedMapping>>;
    /// Return the current physical address for a virtual address
    fn maps_to(virt_addr: VirtAddr) -> Result<PhysAddr>;
}

/// Implements the Paging interface trait.
pub struct Pager {}

impl Paging for Pager {
    fn map_device(phys_addr_range: PhysAddrRange) -> Result<Arc<OwnedMapping>> {
        info!("map_device");
        let virt_addr_range = DEVICE_MEM_ALLOCATOR
            .lock()
            .alloc(phys_addr_range.length_in_pages())?;
        let translation = FixedOffset::new(phys_addr_range.base(), virt_addr_range.base());
        let mut page_directory = KERNEL_PAGE_DIRECTORY.lock();
        let virt_addr_range = page_directory.map_translation(
            virt_addr_range,
            translation,
            Attributes::DEVICE,
            frames::allocator(),
            mem_translation(),
        )?;
        Ok(Arc::new(OwnedMapping::new(virt_addr_range)))
    }

    fn map_dma(contiguous_pages: u8) -> Result<Arc<OwnedMapping>> {
        major!("map_dma");
        if contiguous_pages > 1 {
            unimplemented!()
        }
        let virt_addr_range = DEVICE_MEM_ALLOCATOR
            .lock()
            .alloc(contiguous_pages as usize)?;
        let phys_addr_page = frames::allocator()
            .lock()
            .alloc_zeroed(Purpose::DirectMemoryAccess)?;
        info!("phys_addr_page: {:?}", phys_addr_page);
        let translation = FixedOffset::new(phys_addr_page, virt_addr_range.base());
        let mut page_directory = KERNEL_PAGE_DIRECTORY.lock();
        let virt_addr_range = page_directory.map_translation(
            virt_addr_range,
            translation,
            Attributes::DEVICE,
            frames::allocator(),
            mem_translation(),
        )?;
        Ok(Arc::new(OwnedMapping::new(virt_addr_range)))
    }

    fn maps_to(virt_addr: VirtAddr) -> Result<PhysAddr> {
        KERNEL_PAGE_DIRECTORY
            .lock()
            .maps_to(virt_addr, mem_fixed_offset())
    }
}

/// Number of bytes in a cluster-wide atomic page.
pub const PAGESIZE_BYTES: usize = 4096;

/// Available virtual memory within device range.
static DEVICE_MEM_ALLOCATOR: Locked<PageBumpAllocator> = Locked::new(PageBumpAllocator::new());

/// Available virtual memory for handler stacks (1 per CPU).
static KERNEL_STACK_ALLOCATOR: Locked<PageBumpAllocator> = Locked::new(PageBumpAllocator::new());

/// Pointers to kernel page directory.
static KERNEL_PAGE_DIRECTORY: Locked<arch::PageDirectory> = Locked::new(arch::PageDirectory::new());

/// Initialise the virtual memory manager and jump to the kernel in high memory.
///
/// Can only reference debug, arch and self. Other modules not initialised.
pub fn init(next: fn() -> !) -> ! {
    fn do_init() -> Result<VirtAddr> {
        major!("init");

        Arch::pager_init()?;

        layout::init()?;
        frames::init()?;

        {
            let mut page_directory = KERNEL_PAGE_DIRECTORY.lock();

            map_ranges(&mut (*page_directory), &frames::allocator())?;

            // FIXME: Access to logging enabled.
            if log_enabled!(Level::Trace) {
                page_directory.dump(&Identity::new());
            }
        }

        frames::repoint_frame_table()?;
        layout::update_mem_translation()?;

        {
            let page_directory = KERNEL_PAGE_DIRECTORY.lock();
            Arch::enable_paging(&(*page_directory))?;
        }

        #[cfg(not(test))]
        alloc::init()?;

        allocate_core_stack()
    }

    let stack_pointer = do_init().expect("pager::init");
    Arch::move_stack(stack_pointer, next)
}

fn allocate_core_stack() -> Result<VirtAddr> {
    use AttributeField::*;

    major!("allocate_core_stack");

    // core 0 kernel stack
    const KERNEL_STACK_LEN_PAGES: usize = 6;

    let kernel_stack = {
        let mut lock = KERNEL_STACK_ALLOCATOR.lock();
        // include a guard page
        lock.alloc(KERNEL_STACK_LEN_PAGES + 1)?
    };

    let attributes = Attributes::new()
        .set(KernelRead)
        .set(KernelWrite)
        .set(Accessed);

    // first mapped page, after guard
    let mut kernel_stack_page = kernel_stack.resize(PAGESIZE_BYTES).step();

    for _ in 0..KERNEL_STACK_LEN_PAGES {
        let phys_addr = frames::allocator().lock().alloc_zeroed(Purpose::Kernel)?;
        let translation = FixedOffset::new(phys_addr, kernel_stack_page.base());
        KERNEL_PAGE_DIRECTORY.lock().map_translation(
            kernel_stack_page,
            translation,
            attributes,
            frames::allocator(),
            mem_translation(),
        )?;
        kernel_stack_page = kernel_stack_page.step();
    }
    Ok(kernel_stack_page.base())
}

/// Enable paging and use dedicated stack for current core.
pub fn init_core(next: fn() -> !) -> ! {
    major!("init_core");
    {
        let page_directory = KERNEL_PAGE_DIRECTORY.lock();
        Arch::enable_paging(&(*page_directory)).expect("Arch::enable_paging");
    }
    let stack_pointer = allocate_core_stack().expect("pager::allocate_core_stack");
    Arch::move_stack(stack_pointer, next)
}

fn map_ranges(
    page_directory: &mut impl PageDirectory,
    allocator: &Locked<impl FrameAllocator>,
) -> Result<()> {
    let mem_access_translation = &Identity::new();

    for kernel_range in layout::layout()? {
        use layout::RangeContent::*;

        major!("{:?}", kernel_range);

        if let Some(phys_addr_range) = kernel_range.phys_addr_range {
            let attributes = kernel_range.attributes.set(AttributeField::Accessed);
            let translation =
                FixedOffset::new(phys_addr_range.base(), kernel_range.virt_addr_range.base());

            page_directory.map_translation(
                kernel_range
                    .virt_addr_range
                    .resize(phys_addr_range.length()),
                translation,
                attributes,
                allocator,
                mem_access_translation,
            )?;
        }

        match kernel_range.content {
            KernelStack => {
                KERNEL_STACK_ALLOCATOR
                    .lock()
                    .reset(kernel_range.virt_addr_range)?;
            }
            KernelHeap => {
                page_directory.map_translation(
                    kernel_range.virt_addr_range,
                    NullTranslation::new(),
                    kernel_range.attributes,
                    allocator,
                    mem_access_translation,
                )?;
            }
            Device => {
                DEVICE_MEM_ALLOCATOR
                    .lock()
                    .reset(kernel_range.virt_addr_range)?;
            }
            _ => {}
        };
    }

    error!("Debug output device identity-mapped");
    page_directory.map_translation(
        unsafe { VirtAddrRange::identity_mapped(Arch::debug_uart()?) },
        Identity::new(),
        Attributes::DEVICE,
        allocator,
        mem_access_translation,
    )?;

    Ok(())
}
