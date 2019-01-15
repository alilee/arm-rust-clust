//! The thread module manages the set of local TCBs.
//!
//! A TCB is a space for thread state to be stored when not executing, to
//! pass messages for IPC, and a stack.
//!

use log::info;
use super::arch;


struct ThreadID(u64);


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Terminated = 0,
    Ready,
    Running,
    Blocked,
}


trait ControlBlock {

    pub fn spawn(f: fn() -> ()) -> Result<&mut ControlBlock, u64> {
        arch::thread::ControlBlock::spawn(f)
    }
    pub fn current() -> Result<&mut ControlBlock, u64> {
        arch::thread::ControlBlock::current()
    }
    pub fn yield() -> ! {
        arch::thread::ControlBlock::yield();
    }

    pub fn set_stack(self: &mut ControlBlock, stack: &[u64]) -> Result<&mut ControlBlock, u64>;

    pub fn thread_id(self: &ControlBlock) -> ThreadID;

    pub fn state(self: &ControlBlock) -> Result<State, u64>;
    pub fn terminate(self: &mut ControlBlock) -> Result<State, u64>;
    pub fn ready(self: &mut ControlBlock) -> Result<State, u64>;
    pub fn block(self: &mut ControlBlock) -> Result<State, u64>;
    pub fn run(self: &mut ControlBlock) -> Result<State, u64>;

}


/// Initialise the thread system on boot.
pub fn init() -> () {
    info!("init");
    arch::thread::init();
}
