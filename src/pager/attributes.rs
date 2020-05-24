// SPDX-License-Identifier: Unlicense

use core::fmt::{Debug, Formatter};

/// Flags for page attributes.
#[derive(Copy, Clone)]
pub enum AttributeField {
    /// Readable by user-space threads
    UserRead,
    /// Writeable by user-space threads
    UserWrite,
    /// Executable by user-space threads
    UserExec,
    /// Readable by privileged threads
    KernelRead,
    /// Writeable by privileged threads
    KernelWrite,
    /// Executable by privileged threads
    KernelExec,
    /// Strongly-ordered memory consistency
    Device,
    /// Don't allocate cache on writes
    StreamOut,
    /// Don't allocate cache on reads
    StreamIn,
    /// Allocate memory in blocks because pages can't fault individually
    Block,
    /// Back mapping with memory only when accessed
    OnDemand,
}

/// Bit flags for page attributes.
#[derive(Copy, Clone)]
pub struct Attributes(u64);

impl Debug for Attributes {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        const FIELD_DISPLAY: &[(AttributeField, &str)] = &[
            (OnDemand, "Dmd "),
            (Block, "Blk "),
            (StreamIn, "StmI "),
            (StreamOut, "StmO "),
            (Device, "Dev "),
            (KernelRead, "R"),
            (KernelWrite, "W"),
            (KernelExec, "X"),
            (UserRead, "R"),
            (UserWrite, "W"),
            (UserExec, "X"),
        ];

        write!(f, "Attributes(").unwrap();
        for (field, display) in FIELD_DISPLAY {
            match field {
                KernelRead => {
                    write!(f, "K:").unwrap();
                }
                UserRead => {
                    write!(f, " U:").unwrap();
                }
                _ => (),
            };
            if self.is_set(*field) {
                write!(f, "{}", *display).unwrap();
            }
        }
        write!(f, ")")
    }
}

impl Attributes {
    /// Construct empty attributes.
    pub const fn new() -> Self {
        Self(0)
    }

    /// Read the presence of a specific attribute flag.
    pub const fn is_set(self, field: AttributeField) -> bool {
        0 != (self.0 & (1 << (field as u64)))
    }

    /// Set a specific attribute flag so it will be read as present.
    pub const fn set(self, field: AttributeField) -> Self {
        Self(self.0 | (1 << (field as u64)))
    }
}

impl core::ops::BitOr<AttributeField> for Attributes {
    type Output = Self;

    fn bitor(self, field: AttributeField) -> Attributes {
        self.set(field)
    }
}

use AttributeField::*;

impl Attributes {
    /// For kernel-mapped copy of RAM
    pub const RAM: Attributes = Attributes::new()
        .set(KernelRead)
        .set(KernelWrite)
        .set(Block);
    /// For kernel code
    pub const KERNEL_EXEC: Attributes = Attributes::new()
        .set(KernelExec);
    /// For kernel static
    pub const KERNEL_STATIC: Attributes = Attributes::new()
        .set(KernelRead);
    /// For kernel stack and heap
    pub const KERNEL_DYNAMIC: Attributes = Attributes::new()
        .set(KernelRead)
        .set(KernelWrite)
        .set(OnDemand);
    /// For memory mapped devices
    pub const DEVICE: Attributes = Attributes::new()
        .set(KernelRead)
        .set(KernelWrite)
        .set(Device)
        .set(Block);
    /// For user process code
    pub const USER_EXEC: Attributes = Attributes::new()
        .set(KernelRead)
        .set(KernelWrite)
        .set(UserExec);
    /// For user process data
    pub const USER_STATIC: Attributes = Attributes::new()
        .set(UserRead);
    /// For user process data
    pub const USER_DYNAMIC: Attributes = Attributes::new()
        .set(UserRead)
        .set(UserWrite)
        .set(OnDemand);
}
