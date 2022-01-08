// SPDX-License-Identifier: Unlicense

#![no_std]
#![no_main]
#![feature(format_args_nl)] // for debug macros

extern crate libkernel; // yes, this is needed

use libkernel::*;

/// Kernel entry point, called from architecture-specific reset.
#[no_mangle]
extern "C" fn kernel_init() -> ! {
    major!("starting");

    handler::init().expect("handler::init");
    pager::init().expect("pager::init");

    #[cfg(not(test))]
    pager::alloc::init().expect("pager::alloc::init");

    kernel_main()
}

/// Kernel in high memory, initialise rest of kernel.
fn kernel_main() -> ! {
    major!("kernel_main");

    device::init().expect("device::init");
    // thread::init().expect("thread::init");

    // release_cores();

    // let ta = thread::spawn(workload_a).unwrap();
    // thread::ready(ta);
    // let tb = thread::spawn(workload_b).unwrap();
    // thread::ready(tb);
    //
    // thread::show_state();
    //
    // // clean up boot thread and yield to ready workload
    // thread::terminate()

    major!("looping");
    loop {}
}

/// Core entry point, called from architecture-specific reset.
///
/// Note: stack pointer is at top of small, temporary, shared,
/// start-up stack and memory manager is not yet enabled so is
/// running directed mapped in low memory.
#[no_mangle]
extern "C" fn core_init() -> ! {
    use core::sync::atomic::{AtomicBool, Ordering};

    static ACCESS: AtomicBool = AtomicBool::new(true);

    while ACCESS.swap(false, Ordering::Relaxed) {
        handler::init_core().expect("handler::init_core");
        pager::init_core().expect("pager::init_core");

        ACCESS.store(true, Ordering::Relaxed);
    }

    loop {}
}
