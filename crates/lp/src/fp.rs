//! Floating-point helpers with the exact behaviour of the libm calls the original makes.

/// `ilogb` for finite non-zero x: the unbiased binary exponent, i.e. floor(log2 |x|).
pub fn ilogb(x: f64) -> i32 {
    let bits = x.abs().to_bits();
    let e = (bits >> 52) as i32;
    if e == 0 {
        // subnormal: value = 2^(p - 1074) for the leading set bit p
        63 - bits.leading_zeros() as i32 - 1074
    } else {
        e - 1023
    }
}

/// `scalbn(1.0, e)` for exponents whose result is a normal double.
pub fn pow2(e: i32) -> f64 {
    debug_assert!((-1022..=1023).contains(&e));
    f64::from_bits(((e + 1023) as u64) << 52)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ilogb_and_pow2() {
        assert_eq!(ilogb(1.0), 0);
        assert_eq!(ilogb(0.4117), -2);
        assert_eq!(ilogb(3200.0), 11);
        assert_eq!(ilogb(f64::MIN_POSITIVE / 4.0), -1024);
        assert_eq!(pow2(-47), 7.105427357601002e-15);
        assert_eq!(pow2(47), 140737488355328.0);
    }
}
