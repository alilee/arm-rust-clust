// SPDX-License-Identifier: Unlicense

#![no_std]
#![feature(naked_functions)]  // for _reset

pub mod archs;

#[allow(unused_imports)]
use archs::{ArchTrait, arch::{Arch, _reset}};

fn kernel_init() -> ! {
    kernel_main()
}

fn kernel_main() -> ! {
    Arch::init_handler();
    unreachable!()
}

#[cfg(test)]
mod tests {
    #[test]
    fn lib_works() {
        assert_eq!(2 + 2, 4);
    }
}
