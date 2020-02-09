pub mod locked;
pub mod page_bump;

pub const fn set_above_bits(n: u32) -> usize {
    !((1 << (n as usize)) - 1)
}
pub const fn set_below_bits(n: u32) -> usize {
    ((1 << (n as usize)) - 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_above() {
        assert_eq!(set_above_bits(0), !0usize);
        assert_eq!(set_above_bits(32), !(0xFFFF_FFFFusize));
        assert_eq!(set_above_bits(63), 0x8000_0000_0000_0000usize);
    }

    #[test]
    fn test_set_below() {
        assert_eq!(set_below_bits(0), 0usize);
        assert_eq!(set_below_bits(32), 0xFFFF_FFFFusize);
        assert_eq!(set_below_bits(63), 0x7FFF_FFFF_FFFF_FFFFusize);
    }
}
