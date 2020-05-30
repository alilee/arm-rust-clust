// SPDX-License-Identifier: Unlicense

//! Managing virtual address space, address translation and page faults.

mod alloc;
mod attributes;
mod bump;
mod frames;
mod layout;
mod page;
mod phys_addr;
mod translation;
mod virt_addr;

pub use attributes::*;
pub use page::*;
pub use phys_addr::*;
pub use translation::*;
pub use virt_addr::*;

pub use frames::Allocator as FrameAllocator;

use crate::archs::{arch, arch::Arch, ArchTrait, PageDirectory};
use crate::debug;
use crate::util::locked::Locked;
use crate::Result;
use crate::pager::bump::PageBumpAllocator;

/// Number of bytes in a cluster-wide atomic page.
pub const PAGESIZE_BYTES: usize = 4096;

/// Available virtual memory within device range.
pub static DEVICE_MEM_ALLOCATOR: Locked<PageBumpAllocator> = Locked::new(PageBumpAllocator::new());

/// Initialise the virtual memory manager and jump to the kernel in high memory.
///
/// Can only reference debug, arch and self. Other modules not initialised.
pub fn init(next: fn() -> !) -> ! {
    info!("init");

    frames::init().expect("pager::frames::init");
    layout::init().expect("pager::layout::init");

    Arch::pager_init().expect("arch::pager::init");
    let pd = Locked::new(arch::new_page_directory());
    let mut page_directory = pd.lock();

    let ram_range = Arch::ram_range().expect("arch::ram_range");
    let image_base = PhysAddrRange::text_image().base();
    let mem_offset = &Identity::new();

    // TODO: put all available RAM into frame table
    let low_ram = PhysAddrRange::between(ram_range.base(), image_base);
    frames::add_ram_range(low_ram, mem_offset).expect("pager::frames::include low_ram");

    map_ranges(&mut (*page_directory), &frames::ALLOCATOR).expect("pager::map_ranges");

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
            RAM | KernelImage => {
                let phys_addr_range = kernel_range.phys_addr_range.expect("kernel_range.phys_addr_range");
                page_directory.map_translation(
                    kernel_range.virt_addr_range.resize(phys_addr_range.length()),
                    FixedOffset::new(phys_addr_range.base(), kernel_range.virt_addr_range.base()),
                    kernel_range.attributes,
                    allocator,
                    mem_access_translation,
                )?;
            },
            Device => {
                DEVICE_MEM_ALLOCATOR.lock().reset(kernel_range.virt_addr_range)?;
            }
            L3PageTables | Heap => {
            }
        };
    }
    Ok(())
}
