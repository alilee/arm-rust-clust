mod logger;

extern crate kernel;

use log::info;

#[test]
fn it_works() {
    logger::init();

    assert!(true);
    info!("hello pager");
    dbg!("pager works");
}
