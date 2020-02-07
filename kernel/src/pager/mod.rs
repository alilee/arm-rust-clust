pub mod frames;
mod phys_addr;
pub mod range;
pub mod virt_addr;

use crate::arch;
use crate::debug;
use crate::device;
pub use layout::kernel_mem_offset;
pub use phys_addr::{MemOffset, PhysAddr, PhysAddrRange};
use range::{attrs, layout};
use virt_addr::*;

use log::info;

use core::mem;

pub const PAGESIZE_BYTES: usize = 4096;

#[repr(align(4096))]
pub struct Page([u8; PAGESIZE_BYTES]);

pub struct PageRange(*const Page, *const Page);

impl PageRange {
    pub fn new(range: (*const Page, *const Page)) -> Self {
        PageRange(range.0, range.1)
    }
    pub fn base(&self) -> *const Page {
        self.0
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
    frames::init(ram).unwrap();
    range::init().unwrap();
    arch::pager::init().unwrap();

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
    frames::reserve(image_range).unwrap();

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
    arch::pager::enable(boot3, kernel_image_offset)
}

pub fn device_map<T>(base: PhysAddr) -> Result<*mut T, u64> {
    let length = mem::size_of::<T>();
    let phys_range = PhysAddrRange::new(base, length);
    let aligned_range = phys_range.extend_to_align_to(PAGESIZE_BYTES);
    info!("device map pages@{:?}", aligned_range);
    let device_range = range::alloc_device(aligned_range.pages())?;
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

pub fn alloc(pages: usize, guard: bool) -> Result<PageRange, u64> {
    let pages = if guard { pages + 1 } else { pages };
    let alloc_range = range::alloc_pool(pages)?;
    let alloc_range = if guard {
        alloc_range.trim_left_pages(1)
    } else {
        alloc_range
    };
    arch::pager::provisional_map(
        alloc_range,
        range::attrs::user_read_write(),
        kernel_mem_offset(),
    )?;
    Ok(PageRange::from(alloc_range))
}

pub fn clear_page(page: *mut Page) {
    unsafe { core::intrinsics::volatile_set_memory(page, 0, 1) };
}
