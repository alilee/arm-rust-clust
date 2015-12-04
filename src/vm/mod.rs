pub mod frame;
pub mod page;

extern {
    static mut page_table: [u32; 1024];
    static mut frame_table: [u32; 1024];
}

pub unsafe fn init() {
    frame::init(frame_table.as_mut());
    // page::init(page_table.as_mut());
}