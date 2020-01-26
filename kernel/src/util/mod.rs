pub mod locked;

pub const fn set_above_bits(n: u32) -> usize {
    !((1 << (n as usize + 1)) - 1)
}
pub const fn set_below_bits(n: u32) -> usize {
    !((1 << (n as usize + 1)) - 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_above() {
        assert_eq!(set_above_bits(0), max_value::<usize>());
        assert_eq!(set_above_bits(32), !(0xFFFF_FFFFusize));
        assert_eq!(set_above_bits(64), 0);
    }

    #[test]
    fn test_set_below() {
        assert_eq!(set_below_bits(0), 0);
        assert_eq!(set_below_bits(32), 0xFFFF_FFFF);
        assert_eq!(set_below_bits(64), !0);
    }
}
