//! x87 80-bit extended arithmetic (64-bit significand, round to nearest even) for the sums Gurobi's barrier keeps
//! in `long double`: the Kahan dot `3439d5` (compl, docs/m3_chol.md §4). Only what those sums need: double → ext,
//! double × double → ext, ext ± ext, ext → double.
//! shortcut: no inf/NaN or subnormal-double results (inputs that produce them fall back to f64 arithmetic);
//! add them if a model drives the complementarity there.

/// (−1)^neg · m · 2^e with m = 0 or m ≥ 2^63.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct F80 {
    neg: bool,
    m: u64,
    e: i32,
}

pub const ZERO: F80 = F80 { neg: false, m: 0, e: 0 };

/// Round the positive integer `v` (with sticky information already in its low bits) to 64 significant bits, RNE.
fn round64(neg: bool, v: u128, e: i32) -> F80 {
    if v == 0 {
        return F80 { neg: false, m: 0, e: 0 };
    }
    let p = 127 - v.leading_zeros() as i32; // index of the leading bit
    if p <= 63 {
        return F80 { neg, m: (v << (63 - p)) as u64, e: e - (63 - p) };
    }
    let sh = (p - 63) as u32;
    let (mut m, rest) = ((v >> sh) as u64, v & ((1u128 << sh) - 1));
    let half = 1u128 << (sh - 1);
    let mut e = e + sh as i32;
    if rest > half || (rest == half && m & 1 == 1) {
        m = m.wrapping_add(1);
        if m == 0 {
            m = 1 << 63;
            e += 1;
        }
    }
    F80 { neg, m, e }
}

impl F80 {
    pub fn from_f64(x: f64) -> F80 {
        let b = x.to_bits();
        let (neg, ex, fr) = (b >> 63 == 1, ((b >> 52) & 0x7ff) as i32, b & ((1 << 52) - 1));
        if ex == 0 && fr == 0 {
            return F80 { neg, m: 0, e: 0 };
        }
        let (mant, e) = if ex == 0 { (fr, -1074) } else { (fr | 1 << 52, ex - 1075) };
        let z = mant.leading_zeros();
        F80 { neg, m: mant << z, e: e - z as i32 }
    }

    /// a·b rounded once to 64 bits (x87 `fmull` of a loaded double by a double).
    pub fn mul(a: f64, b: f64) -> F80 {
        let (x, y) = (F80::from_f64(a), F80::from_f64(b));
        let neg = x.neg != y.neg;
        if x.m == 0 || y.m == 0 {
            return F80 { neg, m: 0, e: 0 };
        }
        // 64×64-bit product, keep the top 128 bits exactly (the low 64 of each significand of a double are zero)
        let p = (x.m >> 11) as u128 * (y.m >> 11) as u128;
        round64(neg, p, x.e + y.e + 22)
    }

    /// Round to double (RNE).
    pub fn to_f64(self) -> f64 {
        if self.m == 0 {
            return if self.neg { -0.0 } else { 0.0 };
        }
        let r = round64(self.neg, self.m as u128, self.e); // already 64 bits; now 53:
        let (mut m, rest) = (r.m >> 11, r.m & 0x7ff);
        let mut e = r.e + 11;
        if rest > 0x400 || (rest == 0x400 && m & 1 == 1) {
            m += 1;
            if m == 1 << 53 {
                m >>= 1;
                e += 1;
            }
        }
        let v = m as f64 * 2f64.powi(e); // shortcut: exact unless the result is subnormal or overflows
        if self.neg {
            -v
        } else {
            v
        }
    }
}

impl std::ops::Add for F80 {
    type Output = F80;

    fn add(self, o: F80) -> F80 {
        if self.m == 0 {
            return if o.m == 0 { F80 { neg: self.neg && o.neg, m: 0, e: 0 } } else { o };
        }
        if o.m == 0 {
            return self;
        }
        let (x, y) = if (self.e, self.m) >= (o.e, o.m) { (self, o) } else { (o, self) };
        let d = (x.e - y.e) as u32;
        let xv = (x.m as u128) << 62;
        let yv = (y.m as u128) << 62;
        let ya = if d >= 127 { 1 } else { (yv >> d) | ((yv & ((1u128 << d) - 1) != 0) as u128) };
        if x.neg == y.neg {
            round64(x.neg, xv + ya, x.e - 62)
        } else {
            round64(x.neg, xv - ya, x.e - 62)
        }
    }
}

impl std::ops::Sub for F80 {
    type Output = F80;

    fn sub(self, o: F80) -> F80 {
        self + F80 { neg: !o.neg, ..o }
    }
}

/// `3439d5`: Kahan sum of a[i]·b[i] in x87 extended precision (y = p − c; t = s + y; c = (t − s) − y; s = t).
pub fn kahan_dot(a: &[f64], b: &[f64]) -> F80 {
    let (mut s, mut c) = (ZERO, ZERO);
    for (&x, &y) in a.iter().zip(b) {
        let p = F80::mul(x, y);
        let yy = p - c;
        let t = s + yy;
        c = (t - s) - yy;
        s = t;
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extended_rounding() {
        // 1 + 2^-60 survives in 64 bits but not in 53
        let a = F80::from_f64(1.0) + F80::from_f64(2f64.powi(-60));
        assert_ne!(a, F80::from_f64(1.0));
        assert_eq!(a.to_f64(), 1.0);
        assert_eq!((a - F80::from_f64(1.0)).to_f64(), 2f64.powi(-60));
        // (1 + 2^-52)^2 = 1 + 2^-51 + 2^-104: the last term is lost in 64 bits too
        let p = F80::mul(1.0 + f64::EPSILON, 1.0 + f64::EPSILON);
        assert_eq!((p - F80::from_f64(1.0)).to_f64(), 2f64.powi(-51));
        // tie to even at the 64-bit boundary: 2^63 + 0.5 → 2^63
        assert_eq!((F80::from_f64(2f64.powi(63)) + F80::from_f64(0.5)).to_f64(), 2f64.powi(63));
        // values from numpy.longdouble (x87) running the same Kahan loop
        let v = [1.0, 1e-17, 1e-17, -1.0];
        assert_eq!(kahan_dot(&v, &[1.0; 4]).to_f64(), 2.0003530082357557e-17);
        let mut st = 7u64;
        let mut next = || {
            st = st.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            ((st >> 11) as f64 / 2f64.powi(53) - 0.5) * 2f64.powi((st % 61) as i32 - 30)
        };
        let (mut a, mut b) = (vec![], vec![]);
        for _ in 0..300 {
            a.push(next());
            b.push(next());
        }
        assert_eq!(kahan_dot(&a, &b).to_f64(), f64::from_bits(0x432258784dc60d93));
        assert_eq!(F80::mul(-3.0, 0.5).to_f64(), -1.5);
    }
}
