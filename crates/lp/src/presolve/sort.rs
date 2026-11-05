//! `aaedd9`: sort `keys` ascending, permuting `idx` alongside. Ties keep whatever order this exact algorithm
//! leaves, which decides reduction order downstream (10 §4), so it is ported as is: median-of-3 three-way
//! quicksort for parts of 100 or more (depth limit 50), then Shell sort with gaps n/2, /2.2, ..., 2 -> 1.

pub fn sort_by_key(keys: &mut [f64], idx: &mut [i32]) {
    quick(0, keys.len(), 0, keys, idx);
}

fn quick(mut lo: usize, mut hi: usize, mut depth: i32, a: &mut [f64], ix: &mut [i32]) {
    while depth < 50 {
        let len = hi - lo;
        if len < 100 {
            break;
        }
        let (d2, d3, d4) = (a[lo], a[hi - 1], a[len / 2 + lo]);
        let mut piv = d2;
        if (d2 < d4 || d3 < d2) && (d4 < d2 || d2 < d3) {
            piv = d4;
            if (d4 < d2 || d3 < d4) && (d2 < d4 || d4 < d3) {
                piv = d3;
            }
        }
        // [lo, k) < piv, [k, i) == piv, [j, hi) > piv
        let (mut i, mut j, mut k) = (lo, hi, lo);
        if !(hi <= lo || piv <= d2) {
            loop {
                i += 1;
                k = i;
                if hi <= i {
                    break;
                }
                if !(a[i] < piv) {
                    break;
                }
            }
        }
        while i < j {
            let x = a[i];
            if piv <= x {
                if x == piv {
                    i += 1;
                    continue;
                }
                j -= 1;
                let mut y = a[j];
                let mut done = false;
                while piv <= y {
                    if y == piv {
                        if i < j {
                            a[i] = y;
                            a[j] = x;
                            ix.swap(i, j);
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
                    y = a[j];
                }
                if done {
                    continue;
                }
                if i < j {
                    let (xi, xk, xj) = (ix[i], ix[k], ix[j]);
                    a[i] = a[k];
                    ix[i] = xk;
                    i += 1;
                    a[k] = a[j];
                    ix[k] = xj;
                    k += 1;
                    a[j] = x;
                    ix[j] = xi;
                }
            } else {
                a[i] = a[k];
                ix.swap(i, k);
                i += 1;
                a[k] = x;
                k += 1;
            }
        }
        depth += 1;
        if hi - j < k - lo {
            quick(j, hi, depth, a, ix);
            hi = k;
        } else {
            quick(lo, k, depth, a, ix);
            lo = j;
        }
    }
    shell(lo, hi, a, ix);
}

fn shell(lo: usize, hi: usize, a: &mut [f64], ix: &mut [i32]) {
    let mut g = (hi - lo) / 2;
    while g > 0 {
        loop {
            let mut i = lo + g;
            while i < hi {
                let (key, id) = (a[i], ix[i]);
                let mut p = i;
                while lo + g <= p && a[p - g] > key {
                    a[p] = a[p - g];
                    ix[p] = ix[p - g];
                    p -= g;
                }
                a[p] = key;
                ix[p] = id;
                i += 1;
            }
            if g == 2 && lo + 1 < hi {
                g = 1;
                continue;
            }
            break;
        }
        g = (g as f64 / 2.2) as usize;
    }
}

#[cfg(test)]
mod tests {
    use super::sort_by_key;

    #[test]
    fn sorts_and_carries_index() {
        for n in [0usize, 1, 2, 5, 37, 99, 100, 101, 250, 1000] {
            let mut keys: Vec<f64> = (0..n).map(|i| ((i * 7919) % 97) as f64 - 40.0).collect();
            let orig = keys.clone();
            let mut idx: Vec<i32> = (0..n as i32).collect();
            sort_by_key(&mut keys, &mut idx);
            assert!(keys.windows(2).all(|w| w[0] <= w[1]), "n={n}");
            assert!(idx.iter().zip(&keys).all(|(&i, &k)| orig[i as usize] == k), "n={n}");
        }
    }
}
