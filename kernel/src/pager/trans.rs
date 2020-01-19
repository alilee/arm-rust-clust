use super::{PhysAddrRange, VirtAddr, VirtAddrRange};
use crate::arch;

pub fn init() {
    arch::pager::init();
}

pub fn id_map(range: PhysAddrRange) {
    arch::pager::id_map(range);
}

pub fn print_state() {
    arch::pager::print_state();
}
