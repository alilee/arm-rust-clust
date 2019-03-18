

#[repr(C)]
struct GICD {
    CTLR: u32, // Distributor Control Register
}

pub fn init() -> () {
    info!("initialising");

    unsafe {
        const GICDIST = 0x08000000usize as *const u32;
        let a = ptr::read_volatile(GICDIST);
        info!("{}", a);

        // set GICD_CTLR.ARE[4]
        // set GICD_CTLR.EnableGrp0[0]
        // Clear GICR_WAKER.ProcessorSleep[1]
        // Poll GICR_WAKER.ChildrenAsleep[2] until it reads 0
        // Set ICC_SRE_EL1.SRE[0]
        // Set priority mask and binary point registers
        // Set EOI mode
        // Enable signalling of each interrupt group

        loop {}
    }
}
