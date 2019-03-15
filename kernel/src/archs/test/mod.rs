pub unsafe extern "C" fn _reset() -> ! {
    crate::boot2();
}

pub mod thread {
    pub mod spinlock {

    }
}

pub mod handler {
    pub fn supervisor(syndrome: u16) -> () {}
}
