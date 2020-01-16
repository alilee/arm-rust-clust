use dtb;
use log::{debug, info, trace};
use register::{mmio::*, register_bitfields, register_structs};

use core::sync::atomic::{fence, Ordering};
use core::{mem, ptr};

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

register_bitfields! {
    u32,
    GICR_WAKER [
        ProcessorSleep OFFSET(1) NUMBITS(1) [],
        ChildrenAsleep OFFSET(2) NUMBITS(1) []
    ]
}

register_structs! {
    #[allow(non_snake_case)]
    DistRegisters {
        (0x0000 => GICD_CTLR: ReadWrite<u32, GICD_CTLR::Register>),
        (0x0004 => _reserved),
        (0x1000 => @END),
    }
}

register_structs! {
    #[allow(non_snake_case)]
    CPUInterfaceRegisters {
        (0x0000 => GICC_CTLR: ReadWrite<u32>),
        (0x0004 => _reserved),
        (0x1004 => @END),
    }
}

pub struct GIC {
    dist_base: *mut DistRegisters,
    cpu_intf_base: *mut CPUInterfaceRegisters,
}

static mut THE_GIC: GIC = GIC {
    dist_base: 0 as *mut DistRegisters,
    cpu_intf_base: 0 as *mut CPUInterfaceRegisters,
};

impl GIC {
    pub fn dist(self: &mut Self) -> &mut DistRegisters {
        unsafe { &mut (*self.dist) }
    }
    pub fn rd_base(self: &mut Self) -> &mut LPIRedistRegisters {
        unsafe { &mut (*self.rd_base) }
    }
    pub fn sgi_base(self: &mut Self) -> &mut SGIRedistRegisters {
        unsafe { &mut (*self.sgi_base) }
    }
}

unsafe impl Sync for GIC {}

impl GIC {
    /// Find the GICD base address from the DTB
    pub fn init<'a>(pdtb: *const DTBHeader) -> GIC {
        use core::str;
        use dtb::*;

        fn find_item_with_phandle<'a>(
            root: &'a mut StructItems,
            phandle: &'a [u8],
        ) -> Option<StructItem<'a>> {
            let mut parent = None;
            for item in root {
                if item.is_property() {
                    if let Ok(name) = item.name() {
                        if name == "phandle" {
                            if let Ok(value) = item.value() {
                                if value == phandle {
                                    trace!("found phandle value {:?}", phandle);
                                    trace!("returning parent {:?}", &parent);
                                    return parent;
                                }
                            }
                        }
                    }
                } else {
                    if item.is_begin_node() {
                        parent = Some(item);
                    } else {
                        parent = None;
                    }
                }
            }
            parent
        }

        //        fn _find_node_with_phandle<'a>(
        //            root: &'a mut PathStructItems,
        //            phandle: &'a [u8],
        //        ) -> Option<(StructItem<'a>, StructItems<'a>)> {
        //            let mut parent = None;
        //            let current = root.next();
        //
        //            while let Some(_) = current {}
        //
        //            for node in root {
        //                let (item, children) = node;
        //                dbg!(&item.name());
        //                if item.is_property() {
        //                    if let Ok(name) = item.name() {
        //                        if name == "phandle" {
        //                            if let Ok(value) = item.value() {
        //                                if value == phandle {
        //                                    info!("found phandle value {:?}", phandle);
        //                                    info!("returning parent {:?}", &parent);
        //                                    return parent;
        //                                }
        //                            }
        //                        }
        //                    }
        //                } else {
        //                    if item.is_begin_node() {
        //                        info!("parent is now named {:?}", &item.name());
        //                        parent = Some((item, children));
        //                    } else {
        //                        parent = None;
        //                        info!("parent is now None");
        //                    }
        //                }
        //            }
        //            parent
        //        }

        info!("initialising GICD");

        unsafe {
            // TODO: Address is node.path_struct_items("reg").value_u32_list()[2..3] as u64
            let reader = Reader::read_from_address(pdtb as usize).unwrap();
            let mut root = reader.struct_items();
            let (node, _) = root.path_struct_items("/interrupt-parent").next().unwrap();
            let phandle = node.value().unwrap();
            trace!("/interrupt-parent = <{:?}>", phandle);
            let (_, mut children) = reader.struct_items().path_struct_items("/").next().unwrap();
            let item = find_item_with_phandle(&mut children, phandle).unwrap();
            let item_name = item.name().unwrap();
            trace!("item_name: {:?}", item_name);
            let dist = usize::from_str_radix(item.unit_address().unwrap(), 16).unwrap()
                as *mut DistRegisters;
            trace!("address: {:x?}", dist);
            let rd_base = 0x80a0000usize as *mut LPIRedistRegisters;
            let sgi_base = 0x80b0000usize as *mut SGIRedistRegisters;
            THE_GIC = GIC {
                dist,
                rd_base,
                sgi_base,
            };
            GIC {
                dist,
                rd_base,
                sgi_base,
            }
        }
    }

    pub fn enable(self: &mut GIC) -> () {
        use cortex_a::regs::*;

        info!("enabling GIC");
        {
            let dist = { self.dist() };
            let ctlr = dist.GICD_CTLR.get();
            debug!("CTLR before: {:b}", ctlr);

            // GICv3_Software_Overview_Official_Release_B s4.1
            dist.GICD_CTLR.modify(GICD_CTLR::EnableGrp1NS::SET);
        }
        {
            let rd_base = { self.rd_base() };
            // GICv3_Software_Overview_Official_Release_B s4.2
            rd_base.GICR_WAKER.modify(GICR_WAKER::ProcessorSleep::CLEAR);

            while rd_base.GICR_WAKER.read(GICR_WAKER::ChildrenAsleep) != 0 {
                fence(Ordering::SeqCst);
            }

            ICC_SRE_EL1.modify(ICC_SRE_EL1::SRE::SET); // system register enable
            ICC_PMR_EL1.write(ICC_PMR_EL1::Priority.val(0xFF)); // all
            ICC_BPR1_EL1.write(ICC_BPR1_EL1::BinaryPoint.val(4)); // gggg.pppp
            ICC_CTLR_EL1.modify(ICC_CTLR_EL1::EOImode::CLEAR); // End of Intr = simple
            ICC_IGRPEN1_EL1.write(ICC_IGRPEN1_EL1::Enable::SET); // Enable Group 1
        }
    }

    pub fn enable_irq(self: &mut Self, irq: u32) {
        use cortex_a::regs::*;

        let sgi_base = { self.sgi_base() };
        // IHI0069C_gic_architecture_specification s4.7.1
        sgi_base.GICR_ISENABLER0.set(0xFFFF); // 1 << irq);

        DAIF.modify(DAIF::I::CLEAR);
        trace!("{:b}", DAIF.get());
    }
}
