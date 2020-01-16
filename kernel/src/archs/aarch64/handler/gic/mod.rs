use crate::arch::tree::DTBHeader;

mod gicv2;

pub type IRQHandler = fn(u32, u64);

fn hang(int: u32, _data: u64) {
    use log::error;
    error!("unrequested interrupt {:?}", int);
}

static mut HANDLER_TABLE: [(IRQHandler, u64); 1024] = [(hang, 0); 1024];

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

pub fn init(pdtb: *const DTBHeader) -> impl GIC {
    gicv2::init(pdtb)
}

pub fn get_gic() -> impl GIC {
    gicv2::get_gic()
}
