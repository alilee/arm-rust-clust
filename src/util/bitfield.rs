// SPDX-License-Identifier: Unlicense

//! Page table data structures.

pub use tock_registers::{register_bitfields, UIntLike};

use tock_registers::{
    fields::{Field, FieldValue, TryFromValue},
    RegisterLongName,
};

use core::marker::PhantomData;

/// A in-memory bit struct that fits into integer.
#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct Bitfield<T: UIntLike, R: RegisterLongName = ()> {
    value: T,
    associated_register: PhantomData<R>,
}

impl<T: UIntLike, R: RegisterLongName> Bitfield<T, R> {
    /// New bitfield with given value.
    pub const fn new(value: T) -> Self {
        Self {
            value: value,
            associated_register: PhantomData,
        }
    }

    /// Retrieve the aggregate value.
    #[inline]
    pub fn get(&self) -> T {
        self.value
    }

    /// Set the aggregate value
    #[inline]
    fn set(&mut self, value: T) {
        self.value = value;
    }

    /// Read a field.
    #[inline]
    pub fn read(&self, field: Field<T, R>) -> T {
        (self.get() & (field.mask << field.shift)) >> field.shift
    }

    /// Read a field as the enum value.
    #[inline]
    pub fn read_as_enum<E: TryFromValue<T, EnumType = E>>(&self, field: Field<T, R>) -> Option<E> {
        let val: T = self.read(field);

        E::try_from(val)
    }

    /// Set an individual field value.
    #[inline]
    pub fn write(&mut self, field: FieldValue<T, R>) {
        self.set(field.value);
    }

    /// Change a set of field values.
    #[inline]
    pub fn modify(&mut self, field: FieldValue<T, R>) {
        self.set(field.modify(self.get()));
    }

    /// Determine if a specific flag is set.
    #[inline]
    pub fn is_set(&self, field: Field<T, R>) -> bool {
        self.read(field) != T::zero()
    }

    /// Determine if any of a set of specific flags is set.
    #[inline]
    pub fn matches_any(&self, field: FieldValue<T, R>) -> bool {
        self.get() & field.mask() != T::zero()
    }

    /// Determine if all of a set of specific flags is set.
    #[inline]
    pub fn matches_all(&self, field: FieldValue<T, R>) -> bool {
        self.get() & field.mask() == field.value
    }
}

impl<T: UIntLike, R: RegisterLongName> From<FieldValue<T, R>> for Bitfield<T, R> {
    fn from(field: FieldValue<T, R>) -> Self {
        Self::new(field.value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        register_bitfields! {
            u64,
            pub TestFields [
                flag1 OFFSET(0) NUMBITS(1) [],
                field1 OFFSET(1) NUMBITS(2) [
                   value1a = 0,
                   value1b = 1
                ]
            ]
        }

        type TestBF = Bitfield<u64, TestFields::Register>;

        use TestFields::*;
        let mut v = TestBF::new(0u64);
        v.write(flag1::SET + field1::value1b);
        trace!("{:?}", v.get());
        assert_eq!(v.get(), 0x3);
    }
}
