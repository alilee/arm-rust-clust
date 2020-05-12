// SPDX-License-Identifier: Unlicense

//! Managing virtual address space, address translation and page faults.

mod alloc;
mod attributes;
mod frames;
mod layout;
mod phys_addr;
mod translation;
mod virt_addr;

pub use attributes::*;
pub use phys_addr::*;
pub use translation::*;
pub use virt_addr::*;

pub use frames::Allocator as FrameAllocator;

use crate::archs::{arch::Arch, ArchTrait};
use crate::debug;

/// Initialise the virtual memory manager and jump to the kernel in high memory.
///
/// Can only reference debug, arch and self. Other modules not initialised.
pub fn init(next: fn() -> !) -> ! {
    info!("init");

    frames::init().expect("pager::frames::init");
    layout::init().expect("pager::layout::init");

    Arch::pager_init().expect("arch::pager::init");

    let ram_range = Arch::ram_range().expect("arch::ram_range");
    let image_range = PhysAddrRange::boot_image();

    // TODO: put all available RAM into frame table
    let low_ram = PhysAddrRange::between(ram_range.base(), image_range.base());
    frames::add_ram_range(low_ram).expect("pager::frames::include low_ram");

    for kernel_range in layout::layout().expect("layout::layout") {
        use layout::KernelRange::*;

        match kernel_range {
            RAM(phys_range, offset, attributes) => Arch::map_translation(
                phys_range,
                offset,
                attributes,
                &frames::ALLOCATOR,
                Identity::new(),
            ),
            Image(phys_range, offset, attributes) => Arch::map_translation(
                phys_range,
                offset,
                attributes,
                &frames::ALLOCATOR,
                Identity::new(),
            ),
            Device(virt_range, _attributes) => {
                alloc::add_device_range(virt_range).expect("alloc::add_device_range")
            },
            L3PageTables(virt_range, attributes) => {
                Arch::map_demand(virt_range, attributes, &frames::ALLOCATOR, Identity::new())
            },
            Heap(virt_range, attributes) => {
                Arch::map_demand(virt_range, attributes, &frames::ALLOCATOR, Identity::new());
                alloc::add_heap_range(virt_range).expect("alloc::add_heap_range")
            },
        };
    }

    let kernel_image_offset = FixedOffset::new(image_range.base(), Arch::kernel_base());
    let next: fn() -> ! = unsafe { kernel_image_offset.translate(PhysAddr::from_ptr(next as *const u8)).into() };

    Arch::enable_paging();
    debug::logger::offset(kernel_image_offset).expect("debug::logger::offset");

    next()
}
