mod logger;

use log::info;

#[test]
fn it_works() {
    logger::init();

    assert!(true);
    info!("hello logger");
    dbg!("sandwich");
}
