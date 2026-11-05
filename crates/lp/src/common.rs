//! Shared library routines of the original (`aabbe3` sort) with the exact element movement, so ties come out in
//! the original order.

/// `aabbe3`/`aabbe3_quick`: sort `key` descending, moving `idx` along. Ranges of 100 or more get a three-way
/// partition around a median-of-three pivot (at most 50 levels), the rest a shell sort with gaps n/2, /2.2, 2→1.
pub fn sort_desc(key: &mut [f64], idx: &mut [i32]) {
    quick(0, key.len(), 0, key, idx);
}

fn quick(mut lo: usize, mut hi: usize, mut depth: i32, key: &mut [f64], idx: &mut [i32]) {
    while depth < 50 {
        let len = hi - lo;
        if len < 100 {
            break;
        }
        let (d1, d2, d3) = (key[lo], key[hi - 1], key[lo + len / 2]);
        let mut p = d1;
        if ((d3 < d1) || (d1 < d2)) && ((d1 < d3) || (d2 < d1)) {
            p = d3;
            if ((d1 < d3) || (d3 < d2)) && ((d3 < d1) || (d2 < d3)) {
                p = d2;
            }
        }
        // [lo, g) > p, [g, i) == p, [i, j) unscanned, [j, hi) < p
        let (mut i, mut g, mut j) = (lo, lo, hi);
        if key[lo] > p {
            while i < hi && key[i] > p {
                i += 1;
            }
            g = i;
        }
        while i < j {
            let x = key[i];
            if x > p {
                key.swap(g, i);
                idx.swap(g, i);
                g += 1;
                i += 1;
            } else if x == p {
                i += 1;
            } else {
                j -= 1;
                let mut y = key[j];
                let mut done = false;
                while y <= p {
                    if y == p {
                        if i < j {
                            key.swap(i, j);
                            idx.swap(i, j);
                            i += 1;
                        }
                        done = true;
                        break;
                    }
                    if j <= i {
                        done = true;
                        break;
                    }
                    j -= 1;
                    y = key[j];
                }
                if !done && i < j {
                    // y > p: y joins the greater part, the element at g moves to i, x to j
                    let (xi, gi, ji) = (idx[i], idx[g], idx[j]);
                    key[i] = key[g];
                    idx[i] = gi;
                    key[g] = y;
                    idx[g] = ji;
                    key[j] = x;
                    idx[j] = xi;
                    g += 1;
                    i += 1;
                }
            }
        }
        depth += 1;
        if hi - j < g - lo {
            quick(j, hi, depth, key, idx);
            hi = g;
        } else {
            quick(lo, g, depth, key, idx);
            lo = j;
        }
    }
    shell(lo, hi, key, idx);
}

fn shell(lo: usize, hi: usize, key: &mut [f64], idx: &mut [i32]) {
    let mut gap = (hi - lo) / 2;
    while gap > 0 {
        loop {
            let mut k = lo + gap;
            while k < hi {
                let (x, xi) = (key[k], idx[k]);
                let mut t = k;
                while t >= lo + gap && x > key[t - gap] {
                    key[t] = key[t - gap];
                    idx[t] = idx[t - gap];
                    t -= gap;
                }
                key[t] = x;
                idx[t] = xi;
                k += 1;
            }
            if gap == 2 && lo + 1 < hi {
                gap = 1;
            } else {
                break;
            }
        }
        gap = (gap as f64 / 2.2) as usize;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sorts_descending_with_index() {
        for n in [0usize, 1, 5, 99, 100, 101, 1000, 5000] {
            let mut key: Vec<f64> = (0..n).map(|i| ((i * 7919) % 1013) as f64 / 7.0).collect();
            let orig = key.clone();
            let mut idx: Vec<i32> = (0..n as i32).collect();
            sort_desc(&mut key, &mut idx);
            assert!(key.windows(2).all(|w| w[0] >= w[1]), "n {n}");
            assert!(idx.iter().zip(&key).all(|(&i, &k)| orig[i as usize] == k), "n {n}");
        }
    }
}

/// The original's random stream (`a7b4d6` integer, `a7b54f` double): a 64-bit LCG state whose high word goes through
/// a Mersenne-Twister style temper. State lives at env+0xb0; `895c5` reseeds it.
#[derive(Clone, Copy, Debug)]
pub struct Rng(pub u64);

impl Rng {
    /// `a7b4d6`: next value in [0, 2^31).
    pub fn next_int(&mut self) -> i32 {
        self.0 = self.0.wrapping_mul(0x5851f42d4c957f2d).wrapping_add(0x14057b7ef767814f);
        let mut y = (self.0 >> 32) as u32;
        y ^= y >> 11;
        y ^= (y << 7) & 0x9d2c5680;
        y ^= (y << 15) & 0xefc60000;
        y ^= y >> 18;
        (y & 0x7fff_ffff) as i32
    }

    /// `a7b54f`: next value in [0, 1).
    pub fn next_f64(&mut self) -> f64 {
        self.next_int() as f64 * 4.656612873077393e-10
    }

    /// `895c5`: state = seed + offset (offset = the Seed parameter); returns the previous state minus the offset.
    pub fn reseed(&mut self, seed: u64, offset: i64) -> u64 {
        let old = self.0.wrapping_sub(offset as u64);
        self.0 = seed.wrapping_add(offset as u64);
        old
    }
}

#[cfg(test)]
mod rng_tests {
    use super::Rng;

    /// Values of libgurobi's own a7b4d6/a7b54f from state 12345 (tools/proto/rng_oracle.py).
    #[test]
    fn rng_matches_original() {
        let mut r = Rng(12345);
        let ints: Vec<i32> = (0..4).map(|_| r.next_int()).collect();
        assert_eq!(ints, RNG_INTS);
        assert_eq!(r.next_f64().to_bits(), RNG_F64_BITS);
    }

    const RNG_INTS: [i32; 4] = [1885523651, 1555173835, 2062167607, 1850098887];
    const RNG_F64_BITS: u64 = 0x3fd9596e8a800000;
}
