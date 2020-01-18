mod frames;
mod page;
mod range;

use frames::FrameTable;

use log::info;

const PAGESIZE_BYTES: u32 = 4096;
const PAGESIZE_WORDS: u32 = PAGESIZE_BYTES / 4;

/// A cluster-wide virtual address
#[derive(Debug)]
pub struct VirtAddr(u64);

/// A local physical address
#[derive(Debug, Copy, Clone, PartialOrd, PartialEq)]
pub struct PhysAddr(u64);

impl PhysAddr {
    pub fn from_linker_symbol(sym: &u8) -> Self {
        Self(sym as *const u8 as u64)
    }

    pub fn align_down(self: &Self, align: usize) -> Self {
        assert!(align.is_power_of_two(), "`align` must be a power of two");
        Self(self.0 & !(align as u64 - 1))
    }

    pub fn as_ptr(self: &Self) -> *const u8 {
        self.0 as *const u8
    }
}

impl From<*const u8> for PhysAddr {
    fn from(p: *const u8) -> Self {
        Self(p as u64)
    }
}

/// A range in the VA space
#[derive(Debug, Copy, Clone)]
pub struct PhysAddrRange {
    base: PhysAddr,
    length: usize,
}

impl PhysAddrRange {
    fn bounded_by(base: PhysAddr, top: PhysAddr) -> Self {
        assert!(base.0 < top.0);
        unsafe {
            let length = top.as_ptr().offset_from(base.as_ptr()) as usize;
            Self { base, length }
        }
    }

    fn pages(self: &Self, page_size: usize) -> usize {
        self.length / page_size
    }

    fn top(self: &Self) -> PhysAddr {
        // FIXME: Wrapping around?
        PhysAddr(self.base.0 + self.length as u64)
    }

    fn outside(self: &Self, other: &Self) -> bool {
        self.base < other.base || self.top() > other.top()
    }
}

static mut FRAME_TABLE: FrameTable = FrameTable::init();

/// Initialise the system by initialising the submodules and mapping initial memory contents.
pub fn init() {
    info!("initialising");

    unsafe {
        let ram = PhysAddrRange {
            base: PhysAddr(0x40000000),
            length: 0x10000000,
        };
        FRAME_TABLE.reset(ram);
    }

    // pages::PageTree::init(unsafe { page_table });

    extern "C" {
        static image_base: u8;
        static image_end: u8;
    }

    unsafe {
        let base = PhysAddr::from_linker_symbol(&image_base);
        let top = PhysAddr::from_linker_symbol(&image_end);

        let range = PhysAddrRange::bounded_by(base, top);
        info!("image: {:?}", range);

        // identity-map ram for kernel
        FRAME_TABLE.reserve(range).unwrap();
        // kernel_page_table.id_map(range).unwrap();
    }

    // switch on VM
}
