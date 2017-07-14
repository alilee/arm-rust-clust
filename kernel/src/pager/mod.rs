use log;

mod frames;
mod range;
mod page;

extern "C" {
    static frame_table: *mut usize;
    static page_table: *mut usize;
}

const PAGESIZE_BYTES: u32 = 4096;
const PAGESIZE_WORDS: u32 = PAGESIZE_BYTES / 4;

/// A cluster-wide virtual address
struct VirtualAddress {
    address: usize,
}

/// A local physical address
struct PhysicalAddress {
    address: usize,
}

/// Initialise the system by initialising the submodules and mapping initial memory contents.
pub fn init() {

    info!("initialising");

    frames::FrameTable::init(unsafe { frame_table });
    // pages::PageTree::init(unsafe { page_table });

    // range::Table::init(P_RANGE_TABLE);

    // id_map(&start, 6);
    // id_map(frame::table, 1);
    // id_map(page::table, 2);

}


/// Map a sequence of pages
pub fn map(_: u8) {}

pub fn id_map(_: u32, _: u8) {}
