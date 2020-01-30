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
