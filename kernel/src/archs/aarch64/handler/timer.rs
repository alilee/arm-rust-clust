use log::info;

use super::gic;
use gic::IRQHandler;

pub fn set(duration: i32, handler: IRQHandler) -> Result<u32, u64> {
    use cortex_a::regs::*;

    let freq = CNTFRQ_EL0.get();
    info!("timer frequency is {} Hz", freq);
    info!("setting timer for {} ticks", duration);
    info!("  which is {} secs", duration as f32 / freq as f32);

    CNTP_TVAL_EL0.set(duration as u32);
    CNTP_CTL_EL0.modify(CNTP_CTL_EL0::IMASK::CLEAR + CNTP_CTL_EL0::ENABLE::SET);

    let timer_irq = 30;
    gic::request_irq(timer_irq, handler, duration as u64);
    Ok(timer_irq)
}
