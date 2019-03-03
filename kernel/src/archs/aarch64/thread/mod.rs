/// aarch64-specific thread lifecycle and context switching
///
/// Manages a thread control blocks (TCBs) - one per spawned thread.
/// Does not do context switching - that is handler.
/// State in a separate ringbuffer for simple round-robin scheduling.
/// Array and ringbuffer access behind spinlock.
/// EL0 register state is saved into TCB on exception.

use log::info;

pub mod spinlock;

use cortex_a::regs::*;

#[derive(Clone, Copy, Debug)]
pub struct ControlBlock {
    regs: [u64; 32],  // save registers here on interrupt
    sp_el0: u64,      // saved userspace stack pointer
    elr: u64,         // saved EL0 exception resume address
    spsr: u32,        // saved processor flags state
}


impl ControlBlock {

    pub const fn new() -> ControlBlock {
        ControlBlock {
            regs: [0; 32],
            sp_el0: 0,
            elr: 0,
            spsr: 0,
        }
    }

    pub fn spawn(f: fn() -> ()) -> ControlBlock {
        let mut res = ControlBlock {
            regs: [0; 32],
            sp_el0: 0,
            elr: f as *const fn() -> () as u64,
            spsr: 0,
        };
        // LR
        res.regs[30] = crate::user::thread::terminate as *const fn() -> () as u64;
        res
    }

    pub fn current() -> &'static mut ControlBlock {
        let ptcb = TPIDRRO_EL0.get() as *mut ControlBlock;
        unsafe {
            &mut (*ptcb)
        }
    }

    pub fn set_user_stack(self: &mut ControlBlock, stack: &[u64]) -> () {
        const U64_SIZE: u64 = 8;
        self.sp_el0 = &stack[stack.len()-1] as *const u64 as u64 + U64_SIZE;
    }

    pub fn store_cpu(self: &mut ControlBlock) -> () {
        self.sp_el0 = SP_EL0.get();
        self.elr = ELR_EL1.get();
        self.spsr = SPSR_EL1.get();
    }

    pub fn restore_cpu(self: &ControlBlock) -> () {
        SP_EL0.set(self.sp_el0);
        ELR_EL1.set(self.elr);
        SPSR_EL1.set(self.spsr);
        TPIDRRO_EL0.set(self as *const ControlBlock as u64);
    }
}


pub fn init() -> () {
    info!("init");
}
