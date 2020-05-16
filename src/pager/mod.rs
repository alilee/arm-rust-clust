// SPDX-License-Identifier: Unlicense

//! Managing virtual address space, address translation and page faults.

mod alloc;
mod attributes;
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

/// Number of bytes in a cluster-wide atomic page.
const PAGESIZE_BYTES: usize = 4096;

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

    let identity_map = Identity::new();
    let ram_range = Arch::ram_range().expect("arch::ram_range");
    let image_range = PhysAddrRange::boot_image();
    let mem_offset = &Identity::new();

    // TODO: put all available RAM into frame table
    let low_ram = PhysAddrRange::between(ram_range.base(), image_range.base());
    frames::add_ram_range(low_ram, mem_offset).expect("pager::frames::include low_ram");

    for kernel_range in layout::layout().expect("layout::layout") {
        use layout::KernelRange::*;

        match kernel_range {
            RAM(phys_range, offset, attributes) => page_directory.map_translation(
                phys_range,
                offset,
                attributes,
                &frames::ALLOCATOR,
                mem_offset,
            ),
            Image(phys_range, offset, attributes) => {
                page_directory.map_translation(
                    phys_range,
                    offset,
                    attributes,
                    &frames::ALLOCATOR,
                    mem_offset,
                );
                page_directory.map_translation(
                    phys_range,
                    identity_map, // identity map kernel image for now
                    attributes,
                    &frames::ALLOCATOR,
                    mem_offset,
                );
            }
            Device(virt_range, _attributes) => {
                alloc::add_device_range(virt_range).expect("alloc::add_device_range")
            }
            L3PageTables(virt_range, attributes) => {
                page_directory.map_demand(virt_range, attributes, &frames::ALLOCATOR, mem_offset)
            }
            Heap(virt_range, attributes) => {
                page_directory.map_demand(virt_range, attributes, &frames::ALLOCATOR, mem_offset);
                alloc::add_heap_range(virt_range).expect("alloc::add_heap_range")
            }
        };
    }

    let kernel_image_offset = FixedOffset::new(image_range.base(), Arch::kernel_base());
    let next: fn() -> ! = unsafe {
        kernel_image_offset
            .translate(PhysAddr::from_ptr(next as *const u8))
            .into()
    };

    Arch::enable_paging(&(*page_directory));
    debug::logger::offset(kernel_image_offset).expect("debug::logger::offset");

    next()
}
