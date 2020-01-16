use crate::arch::tree::DTBHeader;

mod gicv2;

pub trait GIC {
    fn reset(self: &mut Self);
    fn enable_irq(self: &mut Self, irq: u32);
    fn print_state(self: &mut Self);
}

pub fn init(pdtb: *const DTBHeader) -> impl GIC {
    gicv2::init(pdtb)
}

pub fn get_gic() -> impl GIC {
    gicv2::get_gic()
}
