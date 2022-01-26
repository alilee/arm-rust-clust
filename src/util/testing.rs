// SPDX-License-Identifier: Unlicense

//! Support for integration testing, shared between tests.

/// Quit the kernel and emulator signalling success.
#[allow(unreachable_code)]
pub fn exit_success() -> ! {
    #[cfg(target_arch = "aarch64")]
    {
        use qemu_exit::QEMUExit;
        qemu_exit::AArch64::new().exit_success();
    }

    // #[cfg(target_arch = "x86_64")]
    // qemu_exit::x86::exit::<100u16>(0u32);

    loop {}
}

/// The runner for integration tests.
///
/// NOTE: This is not used for unit tests.
pub fn test_runner(tests: &[&test_types::UnitTest]) {
    major!("running {} tests", tests.len());
    for test in tests {
        major!("testing {}", test.name);
        (test.test_func)();
    }
    major!("test result: ok.");

    exit_success()
}

#[linkage = "weak"]
#[no_mangle]
fn core_init() {
    use core::sync::atomic::{AtomicBool, Ordering};

    static ACCESS: AtomicBool = AtomicBool::new(true);

    while ACCESS.swap(false, Ordering::Relaxed) {
        // handler::core().expect("handler::core");
        // pager::core(core_main).expect("pager::join_core");
        info!("sandwich");

        ACCESS.store(true, Ordering::Relaxed);
    }

    loop {}
}

#[linkage = "weak"]
#[no_mangle]
fn kernel_init() -> ! {
    use crate::{handler, pager};

    fn next() -> ! {
        major!("paging initialised for testing");

        extern "C" {
            fn collect_tests() -> ();
        }
        unsafe { collect_tests() };
        unreachable!()
    }

    handler::init().expect("handler::init");
    pager::init(next)
}

#[linkage = "weak"]
#[no_mangle]
fn collect_tests() -> () {
    panic!("Override collect_tests() -> () to call test_main()")
}
