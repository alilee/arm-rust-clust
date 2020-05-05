// SPDX-License-Identifier: Unlicense

pub struct Arch {}

impl super::ArchTrait for Arch {
    fn init_handler() {
        1;
    }
    fn init_pager() {
        2;
    }
    fn init_thread() {
        3;
    }
}

#[no_mangle]
pub unsafe extern "C" fn _reset() -> ! {
    crate::kernel_init()
}
