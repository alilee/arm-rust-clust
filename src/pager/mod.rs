// SPDX-License-Identifier: Unlicense

//! Managing virtual address space, address translation and page faults.

mod addr;
mod alloc;
mod attributes;
mod bump;
mod frames;
mod layout;
mod page;
mod phys_addr;
mod translation;
mod virt_addr;

pub use addr::*;
pub use attributes::*;
pub use page::*;
pub use phys_addr::*;
pub use translation::*;
pub use virt_addr::*;

pub use frames::Allocator as FrameAllocator;
pub use frames::Purpose as FramePurpose;

use crate::archs::{arch, arch::Arch, DeviceTrait, HandlerTrait, PageDirectory, PagerTrait};
use crate::debug::Level;
use crate::pager::bump::PageBumpAllocator;
use crate::util::locked::Locked;
use crate::Result;

/// Number of bytes in a cluster-wide atomic page.
pub const PAGESIZE_BYTES: usize = 4096;

/// Available virtual memory within device range.
pub static DEVICE_MEM_ALLOCATOR: Locked<PageBumpAllocator> = Locked::new(PageBumpAllocator::new());

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

    let kernel_image_offset =
        map_ranges(&mut (*page_directory), &frames::allocator()).expect("pager::map_ranges");
    trace!("kernel_image_offset: {:?}", kernel_image_offset);
    debug!("{:?}", *frames::allocator().lock());

    // FIXME: Access to logging enabled.
    if log_enabled!(Level::Trace) {
        page_directory.dump(&Identity::new());
    }

    Arch::handler_init().expect("handler_init");
    Arch::enable_paging(&(*page_directory)).expect("Arch::enable-paging");

    next()
}

fn map_ranges(
    page_directory: &mut impl PageDirectory,
    allocator: &Locked<impl FrameAllocator>,
) -> Result<FixedOffset> {
    use crate::Error;

    let mem_access_translation = &Identity::new();
    let mut result = Err(Error::UnInitialised); // layout should contain KernelText

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

                if kernel_range.content == KernelText {
                    result = Ok(translation);
                }
            }
            Device => {
                DEVICE_MEM_ALLOCATOR
                    .lock()
                    .reset(kernel_range.virt_addr_range)?;
            }
            L3PageTables | Heap => {}
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

    result
}
