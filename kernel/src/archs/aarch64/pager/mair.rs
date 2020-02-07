use log::trace;

#[derive(Debug)]
pub enum MAIR {
    DeviceStronglyOrdered = 0,
    MemoryWriteThrough,
}

impl From<u64> for MAIR {
    fn from(i: u64) -> Self {
        use MAIR::*;
        match i {
            0 => DeviceStronglyOrdered,
            1 => MemoryWriteThrough,
            _ => panic!(),
        }
    }
}

pub fn init() {
    use cortex_a::regs::{RegisterReadWrite, MAIR_EL1, MAIR_EL1::*};

    MAIR_EL1.modify(
        Attr0_Device::nGnRnE
            + Attr1_Outer::WriteThrough_NonTransient_AllocRW
            + Attr1_Inner::WriteThrough_NonTransient_AllocRW,
    );

    trace!("init -> MAIR_EL1 {:#b}", MAIR_EL1.get());
}
