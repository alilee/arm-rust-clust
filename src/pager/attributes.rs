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
    /// Ensure access does not fault due to page entry flags
    Accessed,
    /// Do not mark the underlying page as mapped because the mapping is outside the frame table
    ///
    /// Specifically, the RAM mapping at the bottom of kernel memory bypasses the frame table and
    /// device memory is not backed by physical ram.
    SuppressMapCount,
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
            (Accessed, " A"),
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
        .set(Block)
        .set(SuppressMapCount);
    /// For kernel code
    pub const KERNEL_EXEC: Attributes = Attributes::new().set(KernelExec);
    /// For kernel read-only data
    pub const KERNEL_RO_DATA: Attributes = Attributes::new().set(KernelRead);
    /// For kernel stack and heap
    pub const KERNEL_DATA: Attributes = Attributes::new()
        .set(KernelRead)
        .set(KernelWrite)
        .set(OnDemand);
    /// For kernel identity map on init, and Branch tables
    pub const KERNEL_RWX: Attributes = Attributes::new()
        .set(KernelRead)
        .set(KernelWrite)
        .set(KernelExec)
        .set(Block);
    /// For memory mapped devices
    pub const DEVICE: Attributes = Attributes::new()
        .set(KernelRead)
        .set(KernelWrite)
        .set(Device)
        .set(Block)
        .set(SuppressMapCount);
    /// For user-space branch tables
    pub const USER_RWX: Attributes = Attributes::new().set(UserRead).set(UserWrite).set(UserExec);
    /// For user process code
    pub const USER_EXEC: Attributes = Attributes::new().set(UserExec);
    /// For user process data
    pub const USER_RO_DATA: Attributes = Attributes::new().set(UserRead);
    /// For user process data
    pub const USER_DATA: Attributes = Attributes::new().set(UserRead).set(UserWrite);
}
