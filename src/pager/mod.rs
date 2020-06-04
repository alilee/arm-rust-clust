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

use crate::archs::{arch, arch::Arch, DeviceTrait, PageDirectory, PagerTrait};
use crate::debug;
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

    map_ranges(&mut (*page_directory), &frames::ALLOCATOR).expect("pager::map_ranges");

    debug!("{:?}", *frames::ALLOCATOR.lock());
    // FIXME: Access to logging enabled.
    if crate::debug::logger::_is_enabled("TRACE", module_path!()) {
        page_directory.dump(&Identity::new());
    }

    let image_base = PhysAddrRange::text_image().base();
    let kernel_image_offset = FixedOffset::new(image_base, Arch::kernel_base());
    let next: fn() -> ! = unsafe {
        kernel_image_offset
            .translate_phys(PhysAddr::from_ptr(next as *const u8))
            .unwrap()
            .into()
    };

    Arch::enable_paging(&(*page_directory));
    debug::logger::offset(kernel_image_offset).expect("debug::logger::offset");

    next()
}

fn map_ranges(
    page_directory: &mut impl PageDirectory,
    allocator: &Locked<impl FrameAllocator>,
) -> Result<()> {
    let mem_access_translation = &Identity::new();

    for kernel_range in layout::layout().expect("layout::layout") {
        use layout::RangeContent::*;

        debug!("{:?}", kernel_range);

        match kernel_range.content {
            RAM | KernelText | KernelStatic | KernelData | FrameTable => {
                let phys_addr_range = kernel_range
                    .phys_addr_range
                    .expect("kernel_range.phys_addr_range");
                page_directory.map_translation(
                    kernel_range
                        .virt_addr_range
                        .resize(phys_addr_range.length()),
                    FixedOffset::new(phys_addr_range.base(), kernel_range.virt_addr_range.base()),
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
            L3PageTables | Heap => {}
        };
    }

    // Kernel text identity-mapped
    page_directory.map_translation(
        unsafe { VirtAddrRange::identity_mapped(PhysAddrRange::text_image()) },
        Identity::new(),
        Attributes::KERNEL_EXEC,
        allocator,
        mem_access_translation,
    )?;

    // Debug output device identity-mapped
    page_directory.map_translation(
        unsafe { VirtAddrRange::identity_mapped(Arch::debug_uart()?) },
        Identity::new(),
        Attributes::DEVICE,
        allocator,
        mem_access_translation,
    )?;

    Ok(())
}
