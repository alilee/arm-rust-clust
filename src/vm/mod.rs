use log;

pub mod frame;
pub mod page;
pub mod range;

const PAGESIZE_BYTES: u32 = 4096;
const PAGESIZE_WORDS: u32 = PAGESIZE_BYTES/4;


/// Initialise the system by initialising the submodules and mapping initial memory contents. 
pub fn init() {

    info!("initialising");
    
    const p_frame_table: *mut u32 = (200 * 4096) as *mut u32; // 1 page
    const p_range_table: *mut u32 = (201 * 4096) as *mut u32; // 1 page
    const p_page_table: *mut u32 = (202 * 4096) as *mut u32; // 2 pages

    frame::Table::init(p_frame_table);
    range::Table::init(p_range_table);
    // page::Table::init(page_table_page);
    
    // id_map(&start, 6);
    // id_map(frame::table, 1);
    // id_map(page::table, 2);
    
    // enable_vmmu();
        
}


/// Map a sequence of pages  
pub fn map(npages: u8) {
    
}

pub fn id_map(page: u32, npages: u8) {
    
}