#![allow(missing_docs)]

/// Processor entry point
pub unsafe extern "C" fn _reset() -> ! {
    crate::boot2()
}

pub mod thread {
    use crate::pager::{Page, PhysAddr};

    pub mod spinlock {
        use core::sync::atomic::AtomicBool;

        pub const fn new() -> AtomicBool {
            AtomicBool::new(false)
        }
        pub fn exclusive<F, T>(_b: &mut AtomicBool, closure: F) -> T
        where
            F: FnOnce() -> T,
        {
            closure()
        }
    }

    #[derive(Copy, Clone, Debug)]
    pub struct ControlBlock {}
    static mut MOCK_CONTROL_BLOCK: ControlBlock = ControlBlock {};

    impl ControlBlock {
        pub const fn new() -> Self {
            Self {}
        }
        pub fn spawn(
            f: fn() -> (),
            stack_top: *const Page,
            user_tt_page: PhysAddr,
        ) -> ControlBlock {
            ControlBlock {}
        }
        pub fn current() -> &'static mut ControlBlock {
            unsafe { &mut MOCK_CONTROL_BLOCK }
        }
        pub fn set_user_stack(&mut self, _stack: &[u64]) -> () {}
        pub fn store_cpu(&mut self) -> () {}
        pub fn restore_cpu(&self) -> () {}
        pub fn resume(&self) -> ! {
            unreachable!()
        }
    }

    pub fn init() {}
}

pub mod handler {
    pub fn init() -> Result<(), u64> {
        Ok(())
    }
    pub fn supervisor(_syndrome: u16) -> () {}
    pub fn resume() -> ! {
        unreachable!()
    }
}

pub mod pager {
    use crate::pager;
    use pager::virt_addr::*;
    use pager::{PhysAddr, PhysAddrRange};

    pub fn init() -> Result<PhysAddrRange, u64> {
        Ok(PhysAddrRange::new_const(
            PhysAddr::new_const(0x40000000),
            0x10000000,
        ))
    }

    pub fn enable(offset: VirtOffset) {}
    pub fn device_map(_range: PhysAddrRange) -> Result<*mut (), u64> {
        Err(0)
    }
}
