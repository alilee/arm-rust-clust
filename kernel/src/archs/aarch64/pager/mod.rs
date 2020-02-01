mod table;
mod virt_addr;

use table::{Translation, TranslationAttributes};
use virt_addr::{VirtAddr, VirtOffset};

use crate::debug;
use crate::device;
use crate::pager::{range::attrs, frames, range::layout, range, Page, PhysAddr, PhysAddrRange, PAGESIZE_BYTES};

use log::{debug, info};

use core::mem;

pub const KERNEL_BASE: *const Page = table::UPPER_VA_BASE as *const Page;

pub fn init() -> Result<(), u64> {
    info!("init");
    table::init();

    Ok(())
}

pub fn enable(boot3: fn() -> !) -> ! {
    info!("enable {:?}", boot3);
    fn translate_fn(a: fn() -> !, offset: VirtOffset) -> fn() -> ! {
        let pa = PhysAddr::from_fn(a);
        let va = offset.increment(pa);
        unsafe { mem::transmute::<*const (), fn() -> !>(va.as_ptr()) }
    }

    fn move_registers(offset: VirtOffset) {
        // in asm so that sp doesn't move between get and set
        unsafe {
            asm!("add sp, sp, $0"
                 : 
                 : "r"(offset.get()));
        }
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

    let image_offset = {
        let ram_offset = VirtOffset::new(0);
        let kernel_attrs = TranslationAttributes::from(attrs::kernel());

        let mut tt0 = Translation::new_lower(ram_offset).unwrap();
        tt0.identity_map(range, kernel_attrs).unwrap();
        debug!("ttbr0: {:?}", tt0);

        let mut tt1 = Translation::new_upper(ram_offset).unwrap();
        let reserved_for_ram: usize = 4 * 1024 * 1024 * 1024;
        let kernel_image_location = VirtAddr::new(reserved_for_ram).offset(kernel_mem_offset);
        tt1.absolute_map(range, kernel_image_location, kernel_attrs)
            .unwrap();
        debug!("ttbr1: {:?}", tt1);

        tt1.absolute_map(
            device::ram::range(),
            VirtAddr::from(layout::ram().base()),
            TranslationAttributes::from(attrs::ram()),
        )
        .unwrap();
        debug!("ttbr1: {:?}", tt1);

        enable_paging(tt1.base_register(), tt0.base_register(), 0);

        debug::uart_logger::reset().unwrap();

        VirtOffset::between(range.base(), kernel_image_location)
    };

    move_registers(image_offset);
    let boot3 = translate_fn(boot3, image_offset);
    boot3()
}

pub fn device_map(range: PhysAddrRange) -> Result<*mut (), u64> {
    let (base, offset) = range.base().align_down(PAGESIZE_BYTES);
    let top = range.top().align_up(PAGESIZE_BYTES);
    let range = PhysAddrRange::bounded_by(base, top);
    let page_addr = VirtAddr::from(range::device(range.pages())?);
    let mut tt1 = Translation::ttbr1()?;
    tt1.absolute_map(
        range,
        page_addr,
        TranslationAttributes::from(attrs::device()),
    )?;
    Ok(offset.offset_mut(page_addr.as_mut_ptr()))
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

    assert_eq!(crate::pager::PAGESIZE_BYTES, 4096);
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
