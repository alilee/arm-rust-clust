extern "C" {
    static vector_table_el1: *mut u64;
}

pub fn init() {
    unsafe {
        init_vbar(vector_table_el1);
    }
}

fn init_vbar(vba: *mut u64) {
    // we need a stack
    unsafe {
        asm!("      msr vbar_el1, $0" :: "r"(vba) );
    }
}

#[no_mangle]
pub extern "C" fn handler() -> u64 {
    loop {}
}

pub fn svc() {
    unsafe {
        asm!("svc 1");
    }
}
