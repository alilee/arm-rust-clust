
#[allow(unused_imports)]
use crate::archs::{ArchTrait, arch::Arch};

pub fn init() {
    Arch::init_handler();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        init();
        assert!(true)
    }
}