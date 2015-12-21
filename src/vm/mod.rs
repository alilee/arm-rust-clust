use log;

pub mod frame;
pub mod page;
pub mod range;

const PAGESIZE_BYTES: u32 = 4096;
const PAGESIZE_WORDS: u32 = PAGESIZE_BYTES/4;


/// Initialise the system by initialising the submodules and mapping initial memory contents. 
pub fn init() {

    info!("initialising");
    
    const P_FRAME_TABLE: *mut u32 = (200 * 4096) as *mut u32; // 1 page
    const P_RANGE_TABLE: *mut u32 = (201 * 4096) as *mut u32; // 1 page
    // const P_PAGE_TABLE: *mut u32 = (202 * 4096) as *mut u32; // 2 pages

    frame::Table::init(P_FRAME_TABLE);
    range::Table::init(P_RANGE_TABLE);
    // page::Table::init(page_table_page);
    
    // id_map(&start, 6);
    // id_map(frame::table, 1);
    // id_map(page::table, 2);
    
    // enable_vmmu();
        
}


/// Map a sequence of pages  
pub fn map(_: u8) {
    
}

pub fn id_map(_: u32, _: u8) {
    
}