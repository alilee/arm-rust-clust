// SPDX-License-Identifier: Unlicense

use cortex_a::{regs::*, asm};

#[link_section = ".startup"]
#[no_mangle]
#[naked]
/// Entry point for OS
///
/// Positioned at magic address by linker.ld.
///
/// Gets a stack and calls boot2 for the first core, and parks the rest in a WFE loop.
///
/// NOTE: must not use stack before SP set.
///
/// TODO: CPACR to enable FP in EL1
pub unsafe extern "C" fn _reset(_pdtb: *const u8) -> ! {
    extern "C" {
        static STACK_TOP: u64; // defined in linker.ld
    }

    const CORE_0: u64 = 0;
    const AFF0_CORE_MASK: u64 = 0xFF;

    if CORE_0 == MPIDR_EL1.get() & AFF0_CORE_MASK {
        SP.set(&STACK_TOP as *const u64 as u64);
        // device_tree::set(pdtb);
        crate::kernel_init()
    }

    loop {
        asm::wfe();
    }
}
