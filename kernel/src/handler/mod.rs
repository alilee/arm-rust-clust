
use log::info;

use super::arch;

pub fn init() {
    info!("init");
    arch::handler::init();
}

pub fn supervisor(syndrome: u16) -> () {
    arch::handler::supervisor(syndrome);
}
