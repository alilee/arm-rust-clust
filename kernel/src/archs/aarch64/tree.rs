#[derive(Debug)]
#[repr(C)]
pub struct DTBHeader {
    magic: u32,
    pub totalsize: u32,
    off_dt_struct: u32,
    off_dt_strings: u32,
    off_mem_rsvmap: u32,
    version: u32,
    last_comp_version: u32,
    boot_cpuid_phys: u32,
    size_dt_strings: u32,
    size_dt_struct: u32,
}

static mut DTB: *const DTBHeader = 0 as *const DTBHeader;

pub fn set(pdtb: *const DTBHeader) {
    unsafe {
        DTB = pdtb;
    }
}

pub fn get_dtb() -> *const DTBHeader {
    unsafe { DTB }
}
