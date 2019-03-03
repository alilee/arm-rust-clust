/// kernel entry points from userspace.
///
/// TODO: needs to be linked in user-executable memory.

// use crate::handler::supervisor;

pub mod thread {
    use crate::arch::handler::supervisor;

    pub fn spawn(_f: fn() -> ()) -> Result<u64, u64> {
        supervisor(1);
        Ok(1)
    }

    /// Shut down a user thread.
    pub fn terminate() -> ! {
        supervisor(99);
        unreachable!()
    }
}


// 1: ssel el0
//    mrs elr_el1, x30
//    bl spawn
//    eret
