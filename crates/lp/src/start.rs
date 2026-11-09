//! 04 §4 Dual simplex start basis (original `1e7531` crash L993-1290, called from `0534f7`). From the slack basis,
//! equality-row slacks are swapped for structural columns in two passes, violated inequality rows take a structural by
//! a dual ratio test, and the first factorization renumbers basis positions: structurals in pivot order, then slacks
//! in row order. Nonbasic free columns are then marked −3 for the free-column pivot loop. Matches Gurobi's iteration-0
//! vstat on 64/64 bench LPs (the 4 with free columns are snapshotted before their first free pivot).
//! shortcut: not ported, none triggers on the bench LPs: the free-column crash (L470-870: n > 100, many free columns),
//! the huge-rhs slack bound rule (0534f7 L650) and the free-column pivot loop itself (0534f7 L700-1300, needs FTRAN
//! with LU updates).
use crate::lu::Lu;
use crate::spx::Spx;
use rsolver_model::INFINITY;

/// Primal feasibility tolerance in the scaled workspace (spx+0x380).
const FEAS_TOL: f64 = 1e-6;

/// Status array of the dual start basis: −1 lower, −2 upper, −3 free, −4 fixed, ≥ 0 basis position.
pub fn start_basis(spx: &Spx) -> Vec<i32> {
    let (m, n) = (spx.m, spx.n);
    let mut rows: Vec<Vec<(usize, f64)>> = vec![Vec::new(); m];
    for j in 0..n {
        for (i, a) in spx.col(j) {
            rows[i].push((j, a));
        }
    }
    let clen = |j: usize| spx.len[j] as usize;
    let (l, u) = (&spx.l, &spx.u);
    let mut vstat: Vec<i32> = (0..n)
        .map(|j| if u[j] - l[j] > 0.0 { if spx.c[j] < 0.0 { -2 } else { -1 } } else { -4 }) // FIXTOL = 0
        .chain(0..m as i32)
        .collect();
    let mut head: Vec<usize> = (n..n + m).collect(); // by row
    let mut cw: Vec<f64> = spx.c[..n].to_vec();
    cw.resize(n + m, 0.0);
    // columns sitting at an infinite bound cannot enter in pass 1
    let mut mark: Vec<bool> =
        (0..n).map(|j| (l[j] <= -INFINITY && vstat[j] == -1) || (INFINITY <= u[j] && vstat[j] == -2)).collect();
    let mut active: Vec<bool> = spx.sense.iter().map(|&s| s == b'=').collect();
    let mut eq: Vec<usize> = (0..m).filter(|&i| active[i]).collect();
    let rlen: Vec<i32> = rows.iter().map(|r| r.len() as i32).collect();
    sort_by_key(&mut eq, &rlen);
    let enter = |j: usize, row: usize, old: i32, vstat: &mut Vec<i32>, head: &mut Vec<usize>, active: &mut Vec<bool>| {
        vstat[j] = row as i32;
        vstat[head[row]] = old;
        head[row] = j;
        for (i, _) in spx.col(j) {
            active[i] = false;
        }
    };
    // pass 1: rows whose entries are all unmarked and >= 0.1; least c/a, within 1e-10 the shorter column
    for &row in &eq {
        if !active[row] || rows[row].iter().any(|&(j, a)| mark[j] || a < 0.1) {
            continue;
        }
        let (mut best, mut bcnt, mut ch) = (1e100, m, None);
        for &(j, a) in &rows[row] {
            let r = cw[j] / a;
            if best - 1e-10 > r || (best + 1e-10 > r && clen(j) < bcnt) {
                (best, bcnt, ch) = (r, clen(j), Some((j, a)));
            }
        }
        if let Some((j0, a0)) = ch {
            // 1e7531 L1075: reject the chosen integer column without trying the runner-up.
            if spx.vtype.get(j0).is_some_and(|&t| t != b'C') {
                continue;
            }
            let c0 = cw[j0];
            enter(j0, row, -4, &mut vstat, &mut head, &mut active);
            cw[j0] = 0.0;
            for &(j, a) in &rows[row] {
                mark[j] = true;
                if j != j0 {
                    cw[j] -= (a * c0) / a0;
                }
            }
        }
    }
    // pass 2: zero-cost or free nonbasic columns shorter than 6, free first, then the larger |a|
    for &row in &eq {
        if !active[row] {
            continue;
        }
        let (mut bk, mut babs, mut ch) = (5i64, 0.0, None);
        for &(j, a) in &rows[row] {
            if (cw[j].abs() <= 1e-10 || l[j] <= -INFINITY)
                && !spx.vtype.get(j).is_some_and(|&t| t != b'C')
                && vstat[j] < 0 && a.abs() >= 1e-6 && clen(j) < 6 {
                let key = clen(j) as i64 - if l[j] <= -INFINITY { 2 * m as i64 } else { 0 };
                if key < bk || (key == bk && babs < a.abs()) {
                    (bk, babs, ch) = (clen(j) as i64, a.abs(), Some(j)); // the original keeps the length, not the key
                }
            }
        }
        if let Some(j) = ch {
            enter(j, row, -4, &mut vstat, &mut head, &mut active);
        }
    }
    // pass 3: inequality rows with a basic slack violated at the nonbasic bounds take a column by a dual ratio test
    for row in 0..m {
        if spx.sense[row] == b'=' || head[row] < n {
            continue;
        }
        let (mut act, mut zr, mut zbest, mut nr, mut nbest) = (0.0, 1e100, None, 0.0, None);
        let mut complete = true;
        for &(j, a) in &rows[row] {
            let s = vstat[j];
            let x = match s {
                -4 | -1 => l[j],
                -2 => u[j],
                -3 => 0.0,
                _ => f64::INFINITY,
            };
            if x.abs() >= 1e30 {
                complete = false;
                break;
            }
            act += x * a;
            if s == -4 {
                continue;
            }
            let w = cw[j];
            if x.abs() <= 1e-10 {
                if w == 0.0 && ((s == -1 && a < 0.0) || (s == -2 && 0.0 < a) || s == -3) {
                    complete = false;
                    break;
                }
                if a * w < 0.0 && -w / a < zr {
                    (zr, zbest) = (-w / a, Some(j));
                }
            } else if a * w < 0.0 && -w / a > nr {
                (nr, nbest) = (-w / a, Some(j));
            }
        }
        if !complete || act - spx.b[row] <= FEAS_TOL {
            continue;
        }
        if nr < zr {
            (zr, zbest) = (nr, nbest);
        }
        let Some(jin) = zbest else { continue };
        for &(j, a) in &rows[row] {
            if j == jin {
                vstat[head[row]] = -1;
                vstat[j] = row as i32;
                head[row] = j;
                cw[j] = 0.0;
                cw[n + row] = zr;
            } else {
                cw[j] += a * zr;
                if vstat[j] != -4 {
                    if cw[j] < 0.0 && vstat[j] == -1 {
                        vstat[j] = -2;
                    } else if cw[j] >= 0.0 && vstat[j] == -2 {
                        vstat[j] = -1;
                    }
                }
            }
        }
    }
    // first factorization: structurals in pivot order, then slacks in row order
    let lu = Lu::factor(spx, &head);
    assert!(lu.complete(), "start basis needs the Markowitz phase (not ported)");
    let slacks: Vec<usize> = (n..n + m).filter(|&j| vstat[j] >= 0).collect();
    let order = lu.steps.iter().map(|&(_, pos, _)| head[pos]).filter(|&j| j < n).chain(slacks);
    for (p, j) in order.enumerate() {
        vstat[j] = p as i32;
    }
    for j in 0..n {
        if vstat[j] < 0 && l[j] <= -INFINITY && INFINITY <= u[j] {
            vstat[j] = -3;
        }
    }
    vstat
}

/// `ab6a5f`: sort `idx` ascending by `key[idx]`, unstable as the original (3-way quicksort on ranges of 100 or more,
/// median-of-3 pivot; shell sort below with gaps n/2, then /2.2, 2 → 1). Tie order is part of the crash result.
pub fn sort_by_key(a: &mut [usize], key: &[i32]) {
    let k = |a: &[usize], i: usize| key[a[i]];
    let shell = |a: &mut [usize], lo: usize, hi: usize| {
        let mut g = (hi - lo) / 2;
        while g > 0 {
            loop {
                for i in lo + g..hi {
                    let (v, kv) = (a[i], key[a[i]]);
                    let mut j = i;
                    while j >= lo + g && key[a[j - g]] > kv {
                        a[j] = a[j - g];
                        j -= g;
                    }
                    a[j] = v;
                }
                if g == 2 && lo + 1 < hi {
                    g = 1;
                    continue;
                }
                break;
            }
            g = (g as f64 / 2.2) as usize;
        }
    };
    fn quick(a: &mut [usize], mut lo: usize, mut hi: usize, mut depth: u32, k: &dyn Fn(&[usize], usize) -> i32, shell: &dyn Fn(&mut [usize], usize, usize)) {
        while depth < 50 && hi - lo >= 100 {
            let (k0, k1, k2) = (k(a, lo), k(a, hi - 1), k(a, lo + (hi - lo) / 2));
            let piv = if k2 < k0 {
                if k1 < k0 { k1.max(k2) } else { k0 }
            } else if k2 != k0 && k0 < k1 {
                k1.min(k2)
            } else {
                k0
            };
            let mut i = lo;
            while i < hi && k(a, i) < piv {
                i += 1;
            }
            let (mut lt, mut j) = (i, hi);
            while i < j {
                let (v, kv) = (a[i], k(a, i));
                if kv < piv {
                    a[i] = a[lt];
                    a[lt] = v;
                    lt += 1;
                    i += 1;
                } else if kv == piv {
                    i += 1;
                } else {
                    while i < j {
                        j -= 1;
                        let kw = k(a, j);
                        if kw < piv {
                            if i < j {
                                a[i] = a[lt];
                                a[lt] = a[j];
                                a[j] = v;
                                lt += 1;
                                i += 1;
                            }
                            break;
                        }
                        if kw == piv {
                            if i < j {
                                a[i] = a[j];
                                a[j] = v;
                                i += 1;
                            }
                            break;
                        }
                    }
                }
            }
            depth += 1;
            if hi - j < lt - lo {
                quick(a, j, hi, depth, k, shell);
                hi = lt;
            } else {
                quick(a, lo, lt, depth, k, shell);
                lo = j;
            }
        }
        shell(a, lo, hi);
    }
    let n = a.len();
    quick(a, 0, n, 0, &k, &shell);
}

#[cfg(test)]
mod tests {
    use super::sort_by_key;

    #[test]
    fn sort_orders_keys_and_keeps_the_original_tie_order() {
        // the gap-2 pass swaps 1 and 3, so equal keys 2 and 3 come out reversed (a stable sort gives [0, 2, 3, 1])
        let mut a = vec![0, 1, 2, 3];
        sort_by_key(&mut a, &[0, 1, 0, 0]);
        assert_eq!(a, vec![0, 3, 2, 1]);
        let key: Vec<i32> = (0..300).map(|i| i * 7919 % 13).collect();
        let mut a: Vec<usize> = (0..300).collect();
        sort_by_key(&mut a, &key);
        assert!(a.windows(2).all(|w| key[w[0]] <= key[w[1]]));
        a.sort();
        assert_eq!(a, (0..300).collect::<Vec<_>>());
    }
}
