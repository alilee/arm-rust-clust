
// pub mod pager;
// pub mod handler;

/// Loop forever, saving power
pub fn loop_forever() -> ! {
    use cortex_a::asm;

    loop {
        asm::wfe()
    }
}


// pub fn drop_to_userspace() {
//     // we need a stack
//     unsafe {
//         asm!("      adr x0, foo
//                     msr elr_el1, x0
//                     eret
//               foo:  nop" ::: "x0");
//     }
// }


#[link_section = ".startup"]
#[no_mangle]
#[naked]
/// Entry point for OS
///
/// Positioned at magic address in linker.ld.
///
/// NOTE: must not use stack before SP set.
///
/// TODO: CPACR to enable FP in EL1
pub unsafe extern "C" fn _reset() -> ! {
    use cortex_a::{asm, regs::*};

    extern {
        static stack_top: *const usize; // defined in linker.ld
    }

    const CORE_0: u64 = 0;
    const AFF0_CORE_MASK: u64 = 0xFF;

    if CORE_0 == MPIDR_EL1.get() & AFF0_CORE_MASK {
        SP.set(stack_top as u64);
        ::boot2();
    }

    loop {
        asm::wfe();
    }
}
