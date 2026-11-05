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
