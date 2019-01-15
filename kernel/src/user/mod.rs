use crate::handler::supervisor;

mod thread {
    pub fn spawn(f: fn() -> ()) -> Result<u64, u64> {
        crate::handler::supervisor!(1);
    }

    /// Shut down a user thread.
    pub fn terminate() -> ! {
        crate::handler::supervisor!(99);
    }
}


// 1: ssel el0
//    mrs elr_el1, x30
//    bl spawn
//    eret
