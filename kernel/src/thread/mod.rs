
pub struct JoinHandle<T>(T);

pub fn spawn<F, T>(f: F) -> JoinHandle<T>
where
    F: FnOnce() -> T,
    F: Send + 'static,
    T: Send + 'static,
{
    JoinHandle(f())
}

pub fn init() -> () {}

pub fn discard_boot() -> ! {

    unreachable!()
}
