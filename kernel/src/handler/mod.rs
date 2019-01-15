
use log::info;

use super::arch;

pub fn init() {
    info!("init");
    arch::handler::init();
}

macro_rules! supervisor
    ($syndrome) => {
        arch::handler::supervisor!(syndrome);
    }
}
