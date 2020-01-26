extern crate kernel;
mod test_logger;

use log::trace;

#[test]
fn it_works() {
    test_logger::init();

    assert!(true);
    trace!("hello");
}
