pub unsafe extern "C" fn _reset() -> ! {
    crate::boot2();
}

pub mod thread {
    pub mod spinlock {
        
    }
}
