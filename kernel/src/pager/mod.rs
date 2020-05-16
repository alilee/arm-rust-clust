pub mod attrs;
pub mod frames;
pub mod layout;
mod phys_addr;
pub mod virt_addr;

use crate::arch;
use crate::debug;
use crate::device;
use attrs::Attributes;
pub use layout::kernel_mem_offset;
pub use phys_addr::{MemOffset, PhysAddr, PhysAddrRange};
use virt_addr::*;

use log::info;

use core::mem;

pub const PAGESIZE_BYTES: usize = 4096;

#[repr(align(4096))]
pub struct Page([u8; PAGESIZE_BYTES]);

pub struct PageRange(*const Page, *const Page);

impl PageRange {
    pub fn base(&self) -> *const Page {
        self.0
    }
    pub fn base_mut(&self) -> *mut Page {
        self.0 as *mut Page
    }
    pub fn top(&self) -> *const Page {
        self.1
    }
}

impl From<VirtAddrRange> for PageRange {
    fn from(range: VirtAddrRange) -> Self {
        unsafe {
            PageRange(
                range.base().as_ptr() as *const Page,
                range.top().as_ptr() as *const Page,
            )
        }
    }
}

/// Initialise the system by initialising the submodules and mapping initial memory contents.
pub fn init(boot3: fn() -> !) -> ! {
    info!("init");

    let ram = device::ram::range();
    frames::init(ram).expect("pager::frames::init");
    layout::init().expect("pager::layout::init");
    arch::pager::init().expect("arch::pager::init");

    let image_range = unsafe {
        extern "C" {
            static image_base: u8;
            static image_end: u8;
        }
        PhysAddrRange::pages_bounding(
            PhysAddr::from_linker_symbol(&image_base),
            PhysAddr::from_linker_symbol(&image_end),
        )
    };
    frames::reserve(image_range).expect("pager::frames::reserve image");

    let mem_offset = MemOffset::identity();

    arch::pager::identity_map(image_range, attrs::kernel(), mem_offset).unwrap();

    let kernel_base = layout::image().base();
    arch::pager::absolute_map(image_range, kernel_base, attrs::kernel(), mem_offset).unwrap();

    let ram_base = layout::ram().base();
    arch::pager::absolute_map(device::ram::range(), ram_base, attrs::ram(), mem_offset).unwrap();

    let uart_registers = debug::uart_logger::device_range();
    arch::pager::identity_map(uart_registers, attrs::device(), mem_offset).unwrap();

    let kernel_image_offset = AddrOffsetUp::reverse_translation(image_range.base(), kernel_base);
    let boot3 = kernel_image_offset.reverse_translate_fn(boot3);
    arch::pager::enable(kernel_image_offset);
    boot3()
}

pub fn device_map<T>(base: PhysAddr) -> Result<*mut T, u64> {
    let length = mem::size_of::<T>();
    let phys_range = PhysAddrRange::new(base, length);
    let aligned_range = phys_range.extend_to_align_to(PAGESIZE_BYTES);
    info!("device map pages@{:?}", aligned_range);
    let device_range = layout::alloc_device(aligned_range.pages())?;
    arch::pager::absolute_map(
        phys_range,
        device_range.base(),
        attrs::device(),
        layout::kernel_mem_offset(),
    )?;
    let align_offset = AddrOffsetDown::new_phys_offset(base, aligned_range.base());
    let mapped_addr = align_offset.reverse_offset_virt_addr(device_range.base());
    unsafe { Ok(mapped_addr.as_ptr() as *mut T) }
}

pub fn alloc_pool(pages: usize) -> Result<PageRange, u64> {
    let alloc_range = layout::alloc_pool(pages)?;
    arch::pager::demand_map(alloc_range, attrs::kernel_read_write(), kernel_mem_offset())?;
    Ok(PageRange::from(alloc_range))
}

pub fn allocate(virt_range: VirtAddrRange, attributes: Attributes) -> Result<*const Page, u64> {
    let result = arch::pager::fulfil_map(virt_range, attributes, kernel_mem_offset())?;
    Ok(result.base().as_ptr())
}

// FIXME: When we can rely on frames to issue zero'd frames
pub fn clear_page(page: *mut Page) {
    unsafe { core::intrinsics::volatile_set_memory(page, 0, 1) };
}
