mod gicv2;

use crate::arch::device_tree::DTBHeader;

use log::info;

pub type IRQHandler = fn(u32, u64);

fn hang(int: u32, _data: u64) {
    use log::error;
    error!("unrequested interrupt {:?}", int);
}

static mut HANDLER_TABLE: [(IRQHandler, u64); 1024] = [(hang, 0); 1024];

// FIXME: Remove trait
pub trait GIC {
    fn reset(self: &mut Self);
    fn request_irq(self: &mut Self, irq: u32, handler: IRQHandler, data: u64) {
        assert!(irq < 1024);
        unsafe {
            HANDLER_TABLE[irq as usize] = (handler, data);
        }
    }
    fn enable_irq(self: &mut Self, irq: u32);
    fn ack_int(self: &mut Self) -> u32;
    fn dispatch(self: &mut Self, int: u32) {
        unsafe {
            let (handler, data) = HANDLER_TABLE[int as usize & 0x3FF];
            handler(int, data);
        }
    }
    fn end_int(self: &mut Self, int: u32);
    fn print_state(self: &mut Self);
}

pub fn init(pdtb: *const DTBHeader) -> Result<(), u64> {
    info!("init");
    gicv2::init(pdtb)
}

fn get_gic() -> impl GIC {
    gicv2::get_gic()
}

pub fn reset() {
    info!("reset");
    let mut gic = get_gic();
    gic.reset();
}

pub fn request_irq(irq: u32, handler: IRQHandler, data: u64) {
    info!("request_irq");
    let mut gic = get_gic();
    gic.request_irq(irq, handler, data);
}

pub fn enable_irq(irq: u32) {
    info!("enable_irq");
    let mut gic = get_gic();
    gic.enable_irq(irq);
}

pub fn ack_int() -> u32 {
    let mut gic = get_gic();
    gic.ack_int()
}

pub fn dispatch(int: u32) {
    let mut gic = get_gic();
    gic.dispatch(int)
}

pub fn end_int(int: u32) {
    let mut gic = get_gic();
    gic.end_int(int);
}

pub fn print_state() {
    let mut gic = get_gic();
    gic.print_state();
}
