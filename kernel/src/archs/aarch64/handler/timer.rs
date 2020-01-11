use log::info;

pub fn set(duration: i32) -> Result<(), u64> {
    use cortex_a::regs::*;

    let freq = CNTFRQ_EL0.get();
    info!("timer frequency is {} Hz", freq);
    info!("setting timer for {} ticks", duration);
    info!("  which is {} secs", duration as f32 / freq as f32);

    CNTP_TVAL_EL0.set(duration as u32);
    CNTP_CTL_EL0.modify(
        CNTP_CTL_EL0::ISTATUS::CLEAR + CNTP_CTL_EL0::IMASK::CLEAR + CNTP_CTL_EL0::ENABLE::SET,
    );

    Ok(())
}
