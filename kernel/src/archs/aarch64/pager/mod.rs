mod table;
mod virt_addr;

use crate::pager::{frames, PhysAddr, PhysAddrRange, PAGESIZE_BYTES};
use table::Translation;
use virt_addr::{VirtAddr, VirtOffset};

use log::{debug, info};

use core::mem;

/// TODO: Get this from DTB
fn get_ram() -> Result<PhysAddrRange, u64> {
    Ok(PhysAddrRange::new(PhysAddr::new(0x40000000), 0x10000000))
}

pub fn init() -> Result<PhysAddrRange, u64> {
    info!("init");
    get_ram()
}

pub fn enable(boot3: fn() -> !) -> ! {
    info!("enable {:?}", boot3);
    fn translate_fn(a: fn() -> !, offset: VirtOffset) -> fn() -> ! {
        let pa = PhysAddr::from_fn(a);
        let va = offset.offset(pa);
        unsafe { mem::transmute::<*const (), fn() -> !>(va.as_ptr()) }
    }

    fn move_registers(offset: VirtOffset) {
        use cortex_a::regs::*;

        let sp = PhysAddr::new(SP.get() as usize);
        let sp = offset.offset(sp);
        SP.set(sp.addr() as u64);
    }

    let range = unsafe {
        extern "C" {
            static image_base: u8;
            static image_end: u8;
        }
        PhysAddrRange::bounded_by(
            PhysAddr::from_linker_symbol(&image_base),
            PhysAddr::from_linker_symbol(&image_end),
        )
    };

    frames::reserve(range).unwrap();
    let kernel_mem_offset = table::kernel_mem_offset();

    {
        let attributes = Translation::kernel_attributes();
        let ram_offset = VirtOffset::new(0);

        let mut tt0 = Translation::new_lower(ram_offset).unwrap();
        tt0.identity_map(range, attributes).unwrap();
        debug!("ttbr0: {:?}", tt0);

        let mut tt1 = Translation::new_upper(ram_offset).unwrap();
        let reserved_for_ram: usize = 4 * 1024 * 1024 * 1024;
        let kernel_image_location = VirtAddr::new(reserved_for_ram).offset(kernel_mem_offset);
        tt1.absolute_map(range, kernel_image_location, attributes)
            .unwrap();
        debug!("ttbr1: {:?}", tt1);

        enable_paging(tt1.base_register(), tt0.base_register(), 0);
    }

    move_registers(kernel_mem_offset);
    let boot3 = translate_fn(boot3, kernel_mem_offset);
    boot3()
}

fn enable_paging(ttbr1: u64, ttbr0: u64, asid: u16) {
    use cortex_a::{
        barrier,
        regs::{SCTLR_EL1::*, TCR_EL1::*, *},
    };

    debug!(
        "enable_paging(ttrb1: {:#x}, ttbr0: {:#x}, asid: {:#x})",
        ttbr1, ttbr0, asid
    );

    let ttbr0: u64 = ttbr0 | ((asid as u64) << 48);
    TTBR0_EL1.set(ttbr0);
    TTBR1_EL1.set(ttbr1);

    assert!(PAGESIZE_BYTES == 4096);
    //
    // TODO: nTWE, nTWI
    //
    TCR_EL1.modify(
        AS::Bits_16    // 16 bit ASID 
            + IPS::Bits_36  // 36 bits/64GB of physical address space
            + TG1::KiB_4
            + SH1::Outer
            + ORGN1::WriteThrough_ReadAlloc_NoWriteAlloc_Cacheable
            + IRGN1::WriteThrough_ReadAlloc_NoWriteAlloc_Cacheable
            + T1SZ.val(64 - table::kernel_va_bits() as u64)  // 64-t1sz=43 bits of address space in high range
            + TG0::KiB_4
            + SH0::Outer
            + ORGN0::WriteThrough_ReadAlloc_NoWriteAlloc_Cacheable
            + IRGN0::WriteThrough_ReadAlloc_NoWriteAlloc_Cacheable
            + T0SZ.val(64 - table::user_va_bits() as u64), // 64-t0sz=48 bits of address space in low range
    );
    unsafe {
        barrier::isb(barrier::SY);
    }

    SCTLR_EL1.modify(I::SET + C::SET + M::SET);
    unsafe {
        barrier::isb(barrier::SY);
    }
}
