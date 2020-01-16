use crate::dbg;
use log::{debug, info, trace};

use super::super::tree::DTBHeader;
use register::{mmio::*, register_bitfields, register_structs};

register_bitfields! {
    u32,
    GICD_CTLR [
        EnableGrp1 OFFSET(1) NUMBITS(1) [],
        EnableGrp0 OFFSET(0) NUMBITS(1) []
    ]
}

register_bitfields! {
    u32,
    GICD_TYPER [
        LSPI OFFSET(11) NUMBITS(5) [],
        SecurityExtn OFFSET(10) NUMBITS(5) [],
        CPUNumber OFFSET(5) NUMBITS(3) [],
        ITLinesNumber OFFSET(0) NUMBITS(5) []
    ]
}

register_structs! {
    #[allow(non_snake_case)]
    DistRegisters {
        (0x0000 => GICD_CTLR: ReadWrite<u32, GICD_CTLR::Register>),
        (0x0004 => GICD_TYPER: ReadWrite<u32, GICD_TYPER::Register>),
        (0x0008 => _reserved),
        (0x0080 => GICD_IGROUPR0: ReadWrite<u32>),
        (0x0084 => _reserved2),
        (0x0100 => GICD_ISENABLER0: ReadWrite<u32>),
        (0x0104 => _reserved3),
        (0x0200 => GICD_ISPENDR0: ReadWrite<u32>),
        (0x0104 => _reserved4),
        (0x0400 => GICD_IPRIORITYR0: ReadWrite<u32>),
        (0x0404 => GICD_IPRIORITYR1: ReadWrite<u32>),
        (0x0408 => GICD_IPRIORITYR2: ReadWrite<u32>),
        (0x040c => GICD_IPRIORITYR3: ReadWrite<u32>),
        (0x0410 => GICD_IPRIORITYR4: ReadWrite<u32>),
        (0x0414 => GICD_IPRIORITYR5: ReadWrite<u32>),
        (0x0418 => GICD_IPRIORITYR6: ReadWrite<u32>),
        (0x041c => GICD_IPRIORITYR7: ReadWrite<u32>),
        (0x0420 => _reserved5),
        (0x0800 => GICD_ITARGETSR0: ReadWrite<u32>),
        (0x0804 => GICD_ITARGETSR1: ReadWrite<u32>),
        (0x0808 => GICD_ITARGETSR2: ReadWrite<u32>),
        (0x080c => GICD_ITARGETSR3: ReadWrite<u32>),
        (0x0810 => GICD_ITARGETSR4: ReadWrite<u32>),
        (0x0804 => GICD_ITARGETSR5: ReadWrite<u32>),
        (0x0818 => GICD_ITARGETSR6: ReadWrite<u32>),
        (0x081c => GICD_ITARGETSR7: ReadWrite<u32>),
        (0x0820 => _reserved_to_end),
        (0x1000 => @END),
    }
}

// IHI0048B_b_gic_architecture_specification Fig 4-24
register_bitfields! {
    u32,
    GICC_CTLR [
        EOImode OFFSET(9) NUMBITS(1) [],
        IRQBypDisGrp1 OFFSET(8) NUMBITS(1) [],
        FIQBypDisGrp1 OFFSET(7) NUMBITS(1) [],
        IRQBypDisGrp0 OFFSET(6) NUMBITS(1) [],
        FIQBypDisGrp0 OFFSET(5) NUMBITS(1) [],
        CBPR OFFSET(4) NUMBITS(1) [],
        FIQEn OFFSET(3) NUMBITS(1) [],
        AckCtl OFFSET(2) NUMBITS(1) [],
        EnableGrp1 OFFSET(1) NUMBITS(1) [],
        EnableGrp0 OFFSET(0) NUMBITS(1) []
    ]
}

register_bitfields! {
    u32,
    GICC_PMR [
        Priority OFFSET(0) NUMBITS(8) []
    ]
}

register_bitfields! {
    u32,
    GICC_BPR [
        BinaryPoint OFFSET(0) NUMBITS(3) []
    ]
}

register_bitfields! {
    u32,
    GICC_IAR [
        InterruptID OFFSET(0) NUMBITS(10) [],
        CPUID OFFSET(10) NUMBITS(3) []
    ]
}

register_bitfields! {
    u32,
    GICC_HPPIR [
        CPUID OFFSET(10) NUMBITS(3) [],
        PENDINTID OFFSET(0) NUMBITS(10) []
    ]
}

register_bitfields! {
    u32,
    GICC_AIAR [
        InterruptID OFFSET(0) NUMBITS(10) [],
        CPUID OFFSET(10) NUMBITS(3) []
    ]
}

register_bitfields! {
    u32,
    GICC_AHPPIR [
        CPUID OFFSET(10) NUMBITS(3) [],
        PENDINTID OFFSET(0) NUMBITS(10) []
    ]
}

register_structs! {
    #[allow(non_snake_case)]
    CPUInterfaceRegisters {
        (0x0000 => GICC_CTLR: ReadWrite<u32, GICC_CTLR::Register>),
        (0x0004 => GICC_PMR: ReadWrite<u32, GICC_PMR::Register>),
        (0x0008 => GICC_BPR: ReadWrite<u32, GICC_BPR::Register>),
        (0x000c => GICC_IAR: ReadWrite<u32, GICC_IAR::Register>),
        (0x0010 => GICC_EOIR: WriteOnly<u32>),
        (0x0014 => _reserved1),
        (0x0018 => GICC_HPPIR: ReadWrite<u32, GICC_HPPIR::Register>),
        (0x001c => _reserved2),
        (0x0020 => GICC_AIAR: ReadWrite<u32, GICC_AIAR::Register>),
        (0x0024 => GICC_AEOIR: WriteOnly<u32>),
        (0x0028 => GICC_AHPPIR: ReadWrite<u32, GICC_AHPPIR::Register>),
        (0x002c => _reserved_to_end),
        (0x1004 => @END),
    }
}

#[derive(Debug, Clone)]
pub struct GICv2 {
    dist_base: *mut DistRegisters,
    cpu_intf_base: *mut CPUInterfaceRegisters,
}

static mut THE_GIC: GICv2 = GICv2 {
    dist_base: 0 as *mut DistRegisters,
    cpu_intf_base: 0 as *mut CPUInterfaceRegisters,
};

impl GICv2 {
    fn dist(self: &mut Self) -> &mut DistRegisters {
        unsafe { &mut (*self.dist_base) }
    }
    fn cpu_intf(self: &mut Self) -> &mut CPUInterfaceRegisters {
        unsafe { &mut (*self.cpu_intf_base) }
    }
}

unsafe impl Sync for GICv2 {}

/// Find the GICD base address from the DTB
pub fn init(_pdtb: *const DTBHeader) -> impl super::GIC {
    info!("finding GICD");

    let dist_base = 0x8000000usize as *mut DistRegisters;
    let cpu_intf_base = 0x8010000usize as *mut CPUInterfaceRegisters;

    unsafe {
        THE_GIC = GICv2 {
            dist_base,
            cpu_intf_base,
        };
        THE_GIC.clone()
    }
}

pub fn get_gic() -> impl super::GIC {
    unsafe { THE_GIC.clone() }
}

impl super::GIC for GICv2 {
    fn reset(self: &mut Self) {
        info!("resetting GIC");
        // IHI0048B_b_gic_architecture_specification s4.1.5
        {
            let dist = { self.dist() };
            debug!("GICD_TYPER {:b}", dist.GICD_TYPER.get());
            dist.GICD_CTLR.modify(GICD_CTLR::EnableGrp1::SET);
            debug!("GICD_CTLR {:b}", dist.GICD_CTLR.get());
            dist.GICD_IGROUPR0.set(0xFFFFFFFF);
        }
        {
            let cpu_intf = { self.cpu_intf() };
            cpu_intf
                .GICC_CTLR
                .modify(GICC_CTLR::EnableGrp1::SET + GICC_CTLR::AckCtl::SET);
            cpu_intf.GICC_PMR.modify(GICC_PMR::Priority.val(0xFF));
            //            cpu_intf.GICC_BPR.modify(GICC_BPR::BinaryPoint.val(4));
            debug!("GICC_CTLR {:b}", cpu_intf.GICC_CTLR.get());
            debug!("GICC_PMR {:b}", cpu_intf.GICC_PMR.get());
            debug!("GICC_BPR {:b}", cpu_intf.GICC_BPR.get());
        }
    }

    fn enable_irq(self: &mut Self, irq: u32) {
        assert!(irq < 32);
        let dist = self.dist();

        let igroupr0 = dist.GICD_IGROUPR0.get() | (1 << irq);
        dist.GICD_IGROUPR0.set(igroupr0);

        let ipriorityr7 = dist.GICD_IPRIORITYR7.get() | 0xFEFEFEFE;
        dist.GICD_IPRIORITYR7.set(ipriorityr7);

        let ienabler0 = dist.GICD_ISENABLER0.get() | (1 << irq);
        dist.GICD_ISENABLER0.set(ienabler0);
    }

    fn ack_int(self: &mut Self) -> u32 {
        let cpu_intf = self.cpu_intf();
        cpu_intf.GICC_IAR.get()
    }

    fn end_int(self: &mut Self, int: u32) {
        let cpu_intf = self.cpu_intf();
        cpu_intf.GICC_EOIR.set(int);
    }

    fn print_state(self: &mut Self) {
        {
            let dist = self.dist();
            info!("GICD_CTLR        0b{:32b}", dist.GICD_CTLR.get());
            info!("GICD_ISPENDR0    0b{:32b}", dist.GICD_ISPENDR0.get());
            info!("GICD_IGROUPR0    0b{:32b}", dist.GICD_IGROUPR0.get());
            info!("GICD_ISENABLER0  0b{:32b}", dist.GICD_ISENABLER0.get());
            info!("GICD_ITARGETSR7  0x{:x}", dist.GICD_ITARGETSR7.get());
            info!("GICD_IPRIORITYR7 0x{:x}", dist.GICD_IPRIORITYR7.get());
        }
        {
            let cpu_intf = self.cpu_intf();
            info!("GICC_PMR         0b{:32b}", cpu_intf.GICC_PMR.get());
            info!("GICC_BPR         0b{:32b}", cpu_intf.GICC_BPR.get());
            info!(
                "GICC_HPPIR.CPUID {:?}",
                cpu_intf.GICC_HPPIR.read(GICC_HPPIR::CPUID)
            );
            info!(
                "GICC_HPPIR.PENDINTID {:?}",
                cpu_intf.GICC_HPPIR.read(GICC_HPPIR::PENDINTID)
            );
            info!(
                "GICC_AHPPIR.CPUID {:?}",
                cpu_intf.GICC_AHPPIR.read(GICC_AHPPIR::CPUID)
            );
            info!(
                "GICC_AHPPIR.PENDINTID {:?}",
                cpu_intf.GICC_AHPPIR.read(GICC_AHPPIR::PENDINTID)
            );
        }
    }
}
