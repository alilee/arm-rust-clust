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
    pub fn length_in_pages(&self) -> usize {
        unsafe { self.1.offset_from(self.0) as usize }
    }
}

impl From<VirtAddrRange> for PageRange {
    fn from(range: VirtAddrRange) -> Self {
        PageRange(
            range.base().as_ptr() as *const Page,
            range.top().as_ptr() as *const Page,
        )
    }
}

/// Initialise the system by initialising the submodules and mapping initial memory contents.
pub fn init(boot3: fn() -> !) -> ! {
    info!("init");

    let ram = device::ram::range();
    frames::init(ram);
    range::init();
    arch::pager::init();

    let image_range = unsafe {
        extern "C" {
            static image_base: u8;
            static image_end: u8;
        }
        PhysAddrRange::bounded_by(
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

    let kernel_image_offset = XVirtOffset::between(
        VirtAddr::id_map(image_range.base()),
        VirtAddr::from(kernel_base),
    );
    let boot3 = kernel_image_offset.offset_fn(boot3);
    arch::pager::enable(boot3, kernel_image_offset)
}

pub fn device_map<T>(base: PhysAddr) -> Result<*mut T, u64> {
    let length = mem::size_of::<T>();
    let range = PhysAddrRange::new(base, length);
    let range = range.align_to_pages();
    info!("device map @ {:?}", range);
    let device_base = range::alloc_device(range.pages())?;
    arch::pager::absolute_map(
        range,
        device_base.base(),
        attrs::device(),
        layout::kernel_mem_offset(),
    )?;
    Ok(device_base.base().as_ptr() as *mut T)
}

pub fn alloc(pages: usize, guard: bool) -> Result<PageRange, u64> {
    let phys_range = frames::find_contiguous(pages)?;
    let pages = if guard { pages + 1 } else { pages };
    let alloc_range = range::alloc_pool(pages)?;
    let alloc_range = if guard {
        alloc_range.trim_left_pages(1)
    } else {
        alloc_range
    };
    arch::pager::absolute_map(phys_range, alloc_range.base(), range::attrs::kernel())?;
    Ok(PageRange::from(alloc_range))
}

pub fn clear_page(page: *mut Page) {
    unsafe { core::intrinsics::volatile_set_memory(page, 0, 1) };
}
