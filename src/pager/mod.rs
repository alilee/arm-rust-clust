// SPDX-License-Identifier: Unlicense

//! Managing virtual address space, address translation and page faults.

mod addr;
mod attributes;
mod bump;
mod frames;
mod layout;
mod page;
mod phys_addr;
mod translation;
mod virt_addr;

#[cfg(not(test))]
pub mod alloc;

pub use addr::*;
pub use attributes::*;
pub use page::*;
pub use phys_addr::*;
pub use translation::*;
pub use virt_addr::*;

pub use frames::allocator;
pub use frames::Allocator as FrameAllocator;
pub use frames::Purpose as FramePurpose;

use crate::archs::{arch, arch::Arch, DeviceTrait, PageDirectory, PagerTrait};
use crate::debug::Level;
use crate::pager::bump::PageBumpAllocator;
use crate::util::locked::Locked;
use crate::Result;

/// Number of bytes in a cluster-wide atomic page.
pub const PAGESIZE_BYTES: usize = 4096;

/// Available virtual memory within device range.
pub static DEVICE_MEM_ALLOCATOR: Locked<PageBumpAllocator> = Locked::new(PageBumpAllocator::new());

/// Available virtual memory for handler stacks (1 per CPU).
pub static KERNEL_STACK_ALLOCATOR: Locked<PageBumpAllocator> =
    Locked::new(PageBumpAllocator::new());

static mut MEM_FIXED_OFFSET: FixedOffset = FixedOffset::identity();

/// Get the offset of real RAM from the kernel-mapped area.
#[inline(always)]
pub fn mem_translation() -> &'static impl Translate {
    unsafe { &MEM_FIXED_OFFSET }
}

/// Initialise the virtual memory manager and jump to the kernel in high memory.
///
/// Can only reference debug, arch and self. Other modules not initialised.
pub fn init(next: fn() -> !) -> ! {
    info!("init");

    let frame_table_range = frames::init().expect("pager::frames::init");
    layout::init(frame_table_range).expect("pager::layout::init");

    Arch::pager_init().expect("arch::pager::init");

    let page_directory = Locked::new(arch::new_page_directory());
    let mut page_directory = page_directory.lock();

    let (mem_translation_found, kernel_image_offset, heap_range, stack_pointer) =
        map_ranges(&mut (*page_directory), &frames::allocator()).expect("pager::map_ranges");
    trace!("kernel_image_offset: {:?}", kernel_image_offset);
    debug!("{:?}", *frames::allocator().lock());

    unsafe {
        MEM_FIXED_OFFSET = mem_translation_found;
    }

    // FIXME: Access to logging enabled.
    if log_enabled!(Level::Trace) {
        page_directory.dump(&Identity::new());
    }

    Arch::enable_paging(&(*page_directory)).expect("Arch::enable-paging");

    debug!("heap_range: {:?}", heap_range);
    #[cfg(not(test))]
    alloc::init(heap_range).expect("alloc::init");

    Arch::move_stack(stack_pointer, next)
}

fn map_ranges(
    page_directory: &mut impl PageDirectory,
    allocator: &Locked<impl FrameAllocator>,
) -> Result<(FixedOffset, FixedOffset, VirtAddrRange, VirtAddr)> {
    use crate::Error;

    let mem_access_translation = &Identity::new();
    let mut mem_translation = Err(Error::UnInitialised); // layout mem
    let mut kernel_offset = Err(Error::UnInitialised); // layout should contain KernelText
    let mut heap_range_result = Err(Error::UnInitialised); // layout should yield heap
    let mut stack_pointer = Err(Error::UnInitialised); // layout should yield stack

    for kernel_range in layout::layout().expect("layout::layout") {
        use layout::RangeContent::*;

        debug!("{:?}", kernel_range);

        match kernel_range.content {
            RAM | KernelText | KernelStatic | KernelData | FrameTable => {
                let attributes = kernel_range.attributes.set(AttributeField::Accessed);
                let phys_addr_range = kernel_range
                    .phys_addr_range
                    .expect("kernel_range.phys_addr_range");
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

                match kernel_range.content {
                    RAM => mem_translation = Ok(translation),
                    KernelText => {
                        kernel_offset = Ok(translation);
                    }
                    _ => {}
                };

                assert!(
                    kernel_range.content != RAM
                        || kernel_range.virt_addr_range.base() == Arch::kernel_base()
                );
            }
            Device => {
                DEVICE_MEM_ALLOCATOR
                    .lock()
                    .reset(kernel_range.virt_addr_range)?;
            }
            L3PageTables => {}
            Heap => {
                use AttributeField::*;
                let attributes = Attributes::new()
                    .set(KernelRead)
                    .set(KernelWrite)
                    .set(Accessed);

                let backing = {
                    let mut frame_table = frames::allocator().lock();
                    frame_table.alloc_zeroed(FramePurpose::Kernel)?
                };

                let phys_addr_range = PhysAddrRange::new(backing, PAGESIZE_BYTES);
                let translation =
                    FixedOffset::new(phys_addr_range.base(), kernel_range.virt_addr_range.base());
                let heap_range = kernel_range
                    .virt_addr_range
                    .resize(phys_addr_range.length());

                page_directory.map_translation(
                    heap_range,
                    translation,
                    attributes,
                    allocator,
                    mem_access_translation,
                )?;

                heap_range_result = Ok(heap_range);
            }
            KernelStack => {
                use frames::Purpose;
                use AttributeField::*;

                // core 0 kernel stack
                const KERNEL_STACK_LEN_PAGES: usize = 6;

                let kernel_stack = {
                    let mut lock = KERNEL_STACK_ALLOCATOR.lock();
                    lock.reset(kernel_range.virt_addr_range)?;
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
                    let phys_addr = allocator.lock().alloc_for_overwrite(Purpose::Kernel)?;
                    let translation = FixedOffset::new(phys_addr, kernel_stack_page.base());
                    page_directory.map_translation(
                        kernel_stack_page,
                        translation,
                        attributes,
                        allocator,
                        mem_access_translation,
                    )?;
                    kernel_stack_page = kernel_stack_page.step();
                }
                stack_pointer = Ok(kernel_stack_page.base());
            }
        };
    }

    debug!("Debug output device identity-mapped");
    page_directory.map_translation(
        unsafe { VirtAddrRange::identity_mapped(Arch::debug_uart()?) },
        Identity::new(),
        Attributes::DEVICE,
        allocator,
        mem_access_translation,
    )?;

    Ok((
        mem_translation?,
        kernel_offset?,
        heap_range_result?,
        stack_pointer?,
    ))
}
