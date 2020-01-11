use crate::arch::tree::DTBHeader;

use dtb;
use log::info;
use register::{mmio::*, register_bitfields};

register_bitfields! {
    u32,
    GICD_CTLR [
        RWP OFFSET(31) NUMBITS(1) [],
        E1NWF OFFSET(7) NUMBITS(1) [],
        DS OFFSET(6) NUMBITS(1) [],
        ARE_NS OFFSET(5) NUMBITS(1) [],
        ARE_S OFFSET(4) NUMBITS(1) [],
        EnableGrp1S OFFSET(2) NUMBITS(1) [],
        EnableGrp1NS OFFSET(1) NUMBITS(1) [],
        EnableGrp0 OFFSET(0) NUMBITS(1) []
    ]
}

#[allow(non_snake_case)]
#[repr(C)]
struct GICDRegisters {
    GICD_CTLR: ReadWrite<u32, GICD_CTLR::Register>, // Distributor Control Register
}

pub struct GICD {
    base: *mut GICDRegisters,
}

impl GICD {
    pub fn regs(self: &mut Self) -> &mut GICDRegisters {
        unsafe { &mut (*self.base) }
    }
}

unsafe impl Sync for GICD {}

impl GICD {
    pub fn init(pdtb: *const DTBHeader) -> GICD {
        info!("initialising");

        unsafe {
            // TODO: should be /interrupt-parent -> find(phandle)
            // Address is node.path_struct_items("reg").value_u32_list()[2..3] as u64
            let dtb = dtb::Reader::read_from_address(pdtb as usize).unwrap();
            let root = dtb.struct_items();
            let (node, _) = root.path_struct_items("/intc").next().unwrap();
            let node_name = node.name().unwrap();
            info!("node_name: {:?}", node_name);
            let mut split = node_name.split('@');
            split.next();
            let address_str = split.next().unwrap();
            let address: usize = usize::from_str_radix(address_str, 16).unwrap();
            info!("address: {:?}", address);
            GICD {
                base: address as *mut GICDRegisters,
            }
        }
    }

    pub fn enable(self: &mut GICD) -> () {
        let regs = self.regs();
        let ctlr = regs.GICD_CTLR.get();
        info!("CTLR before: {:?}", ctlr);

        regs.GICD_CTLR.modify(
            GICD_CTLR::ARE_NS::SET + GICD_CTLR::EnableGrp0::SET + GICD_CTLR::EnableGrp1NS::SET,
        );

        // Clear GICR_WAKER.ProcessorSleep[1]
        // Poll GICR_WAKER.ChildrenAsleep[2] until it reads 0
        // Set ICC_SRE_EL1.SRE[0]
        // Set priority mask and binary point registers
        // Set EOI mode
        // Enable signalling of each interrupt group

        let ctlr = regs.GICD_CTLR.get();
        info!("CTLR after: {:?}", ctlr);

        //            loop {}
    }
}
