use crate::pager::{
    frames,
    virt_addr::{AddrOffsetUp, VirtAddr},
    PhysAddr,
};

pub enum Mapper {
    OffsetUp(AddrOffsetUp),
    OnDemand,
    Fulfil,
}

impl Mapper {
    pub fn identity() -> Self {
        Mapper::OffsetUp(AddrOffsetUp::id_map())
    }

    pub fn reverse_translation(phys_addr: PhysAddr, virt_base: VirtAddr) -> Self {
        unsafe { Mapper::OffsetUp(AddrOffsetUp::reverse_translation(phys_addr, virt_base)) }
    }

    pub fn demand() -> Self {
        Mapper::OnDemand
    }

    pub fn fulfil() -> Self {
        Mapper::Fulfil
    }

    pub fn translate(&self, virt_addr: VirtAddr) -> Result<PhysAddr, u64> {
        use Mapper::*;
        let result = match self {
            OffsetUp(offset) => offset.translate(virt_addr),
            OnDemand => PhysAddr::null(),
            Fulfil => frames::find(),
        };
        Ok(result)
    }
}
