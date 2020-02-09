use register::{register_bitfields, LocalRegisterCopy};

register_bitfields! {
    u64,
    pub AttributeFields [
        Block 16,
        StreamIn 10,
        StreamOut 9,
        Device 8,
        KernelExec 7,
        KernelWrite 6,
        KernelRead 5,
        UserExec 3,
        UserWrite 2,
        UserRead 1
    ]
}

pub type Attributes = LocalRegisterCopy<u64, AttributeFields::Register>;

pub fn kernel() -> Attributes {
    use AttributeFields::*;
    let field = KernelExec::SET + KernelRead::SET + KernelWrite::SET;
    Attributes::new(field.value)
}

pub fn device() -> Attributes {
    use AttributeFields::*;
    let field = KernelRead::SET + KernelWrite::SET + Device::SET + Block::SET;
    Attributes::new(field.value)
}

pub fn ram() -> Attributes {
    use AttributeFields::*;
    let field = KernelRead::SET + KernelWrite::SET + Block::SET;
    Attributes::new(field.value)
}

pub fn user_read_write_prov() -> Attributes {
    use AttributeFields::*;
    let field = UserRead::SET + UserWrite::SET;
    Attributes::new(field.value)
}

pub fn user_read_write_exec() -> Attributes {
    use AttributeFields::*;
    let field = UserRead::SET + UserWrite::SET + UserExec::SET;
    Attributes::new(field.value)
}

pub fn kernel_read_write() -> Attributes {
    use AttributeFields::*;
    let field = KernelRead::SET + KernelWrite::SET;
    Attributes::new(field.value)
}
