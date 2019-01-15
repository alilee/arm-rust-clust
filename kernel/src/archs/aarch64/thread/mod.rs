use log::info;
use core::sync::atomic::{AtomicBool, ATOMIC_BOOL_INIT};

mod spinlock;

use crate::thread::State;
use crate::thread::ControlBlock as IControlBlock;

#[derive(Clone, Copy, Debug)]
struct ControlBlock {
    tpid: u64,
    slot: u64,
    el1_sp: u64,
    el0_sp: u64,
    elr: u64,
    spsr: u64,
}

const U64_SIZE: u64 = core::mem::size_of::<u64>() as u64;
const MAX_THREADS: usize = 10;
const KERNEL_STACK_LEN: usize = 100;

type KernelStack = [u64; KERNEL_STACK_LEN];
static mut STACKS: [KernelStack; MAX_THREADS] = [[0; KERNEL_STACK_LEN]; MAX_THREADS];
static mut STATES: [State; MAX_THREADS] = [State::Terminated; MAX_THREADS];
static mut THREADS: [ControlBlock; MAX_THREADS] =
    [ ControlBlock {
        tpid: 99,
        slot: 0,
        el1_sp: 0,
        el0_sp: 0,
        elr: 0,
        spsr: 0, }; MAX_THREADS];
static mut THREADS_SL: AtomicBool = ATOMIC_BOOL_INIT;


fn find_state(states: &[State], state: State) -> Result<usize, u64> {
    states.iter().position(|s| s == state ).ok_or(0)
}


impl IControlBlock for ControlBlock {

    pub fn spawn(f: fn() -> ()) -> Result<&mut IControlBlock, u64> {
        unsafe {
            let slot = spinlock::exclusive(&mut THREADS_SL, || {
                find_state(&STATES[..], State::Terminated).and_then(|slot| {
                    STATES[slot] = State::Blocked;
                    Ok(slot)
                })
            })?;

            let kernel_stack: &[u64] = &STACKS[slot][..];
            THREADS[slot] = ControlBlock {
                tpid: &THREADS[slot] as *const ControlBlock as u64,
                slot: slot,
                el1_sp: &kernel_stack[kernel_stack.len()-33] as *const u64 + U64_SIZE,
                el0_sp: 0,
                elr: f as *const () as u64,
                spsr: 0,
            };
            // LR
            *(THREADS[slot].el1_sp as *const u64) = &crate::user::thread::terminate as *const () as u64;
            Ok(&THREADS[slot])
        }
    }

    fn find(thread_id: ThreadID) -> Result<&mut IControlBlock, u64> {
        unsafe {
            let ptcb = thread_id as *mut ControlBlock;
            Ok(&(*ptcb))
        }
    }

    fn next_ready() -> Result<&mut IControlBlock, u64> {
        unsafe {
            let slot = spinlock::exclusive(&mut THREADS_SL, || {
                let slot = find_state(&STATES[..], State::Ready)?;
                THREADS[slot].running();
                Ok(slot)
            })?;
            Ok(&THREADS[slot])
        }
    }

    fn set_user_stack(self: &mut IControlBlock, stack: &[u64]) -> Result<&mut IControlBlock, u64> {
        self.el0_sp = &stack[stack.len()-1] as *const u64 as u64 + U64_SIZE;
        Ok(self)
    }

    fn thread_id(self: &IControlBlock) -> ThreadID {
        ThreadID(self.tpid)
    }

    fn state(self: &IControlBlock) -> State {
        STATES[self.slot]
    }

    fn terminate(self: &mut IControlBlock) -> Result<State, u64> {
        STATES[self.slot] = State::Terminated;
        Ok(State::Terminated)
    }

    fn ready(self: &mut IControlBlock) -> Result<State, u64> {
        STATES[self.slot] = State::Ready;
        Ok(State::Ready)
    }

    fn block(self: &mut IControlBlock) -> Result<State, u64> {
        STATES[self.slot] = State::Blocked;
        Ok(State::Blocked)
    }

    fn run(self: &mut IControlBlock) -> Result<State, u64> {
        STATES[self.slot] = State::Running;
        Ok(State::Running)
    }
}


/// Initialise the thread system on boot.
///
/// Assimilates the boot2 thread as already running.
pub fn init() -> Result<u64, u64> {
    info!("init");

    unsafe {
        let tcb = spinlock::exclusive(&mut THREADS_SL, || {
            STATES[0] = State::Running;
            THREADS[0] = ControlBlock {
                tpid: &THREADS[0] as *const ControlBlock as u64,
                el1_sp: 0,
                el0_sp: 0,
                elr: 0,
                spsr: 0,
            }?;
            Ok(&THREADS[0])
        })?;
        TDIPR_EL0.set(tcb.tpid);
    }
}


pub fn current() -> ThreadID {
    let ptcb: *const ControlBlock = TPID_EL1.get() as *const ControlBlock;
    ThreadID(ptcb as u64)
}


pub fn terminate() -> () {
    use cortex_a::svc;
    svc!(99);
}


pub fn svc_terminate() -> () {
    let tcb = TPID_EL1.get() as *mut ControlBlock;
    let slot = spinlock::exclusive(&mut THREADS_SL, || {
        *tcb.state = State::Terminated;
    })?;
}


pub fn yield() -> ! {
    let slot = spinlock::exclusive(&mut THREADS_SL, || {
        *tcb.state = State::Terminated;
    }
    match ControlBlock::next_ready() {
        Err(e) => loop_forever(),
        Ok(tcb) => switch_context(tcb),
    }
}
