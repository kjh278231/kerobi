//! 04 §5 Dual ratio test with bound flipping (`1c35c5`), rules reproduced on Gurobi's own inputs/outputs
//! (docs/m1/m1_notes.md §8.6, docs/m1/m1_02e_bfrt.md: entering column 3108/3112, flip list 3112/3112).
//! Candidates come in the original α-vector order: structural columns ("front"), then slacks ("back").
//! shortcut: the front order is ascending column index; the original scatters the row-wise copy (`0a4109`), so
//! exact ties can still break differently.

pub const OPT_TOL: f64 = 1e-6;
const PIVOT_TOL: f64 = 3e-8;

/// One nonbasic column of the pivot row in the original sign convention: α < 0 marks a candidate, d ≥ 0 is the
/// normalized reduced cost, `range` = u − l.
#[derive(Clone, Copy, Debug)]
pub struct Cand {
    pub j: usize,
    pub a: f64,
    pub d: f64,
    pub range: f64,
    pub back: bool,
    /// Column count (structural) or 0.5 (slack).
    pub cnt: f64,
    /// 0.3·f + 0.7 (f = 1.5 for integer columns).
    pub fac: f64,
}

impl Cand {
    /// Selection weight of the main pass.
    fn w(&self) -> f64 {
        self.cnt * self.fac
    }
}

/// run ± Σxs in the original order (asm 0xcba7f): one element first when the start index is odd (16-byte aligned
/// buffer), fewer than 8 in sequence, else 8 lanes with run in lane 0 combined ((0+2)+(4+6))+((1+3)+(5+7)), then
/// the tail in sequence.
fn lane_sum(mut run: f64, xs: impl Iterator<Item = f64>, start: usize, sign: f64) -> f64 {
    let xs: Vec<f64> = xs.map(|x| sign * x).collect();
    let k = start % 2;
    if xs.len() < k + 8 {
        return xs.iter().fold(run, |r, &x| r + x);
    }
    for &x in &xs[..k] {
        run += x;
    }
    let nv = k + (xs.len() - k) / 8 * 8;
    let mut l = [0.0f64; 8];
    l[0] = run;
    for b in (k..nv).step_by(8) {
        for t in 0..8 {
            l[t] += xs[b + t];
        }
    }
    run = ((l[0] + l[2]) + (l[4] + l[6])) + ((l[1] + l[3]) + (l[5] + l[7]));
    xs[nv..].iter().fold(run, |r, &x| r + x)
}

/// Hoare partitions with the first element as pivot (ratio descending, contribution ascending), narrowing to the
/// breakpoint where δ + Σ contributions turns non-negative.
fn quickselect(c: &mut [(usize, f64, f64)], mut run: f64) -> usize {
    let (mut lo, mut hi) = (0usize, c.len() - 1);
    while lo < hi {
        let (pr, pc) = (c[lo].2, c[lo].1);
        let (mut i, mut j) = (hi + 1, lo as isize - 1);
        loop {
            i -= 1;
            while c[i].2 < pr || (c[i].2 == pr && c[i].1 > pc) {
                i -= 1;
            }
            j += 1;
            while c[j as usize].2 > pr || (c[j as usize].2 == pr && c[j as usize].1 < pc) {
                j += 1;
            }
            let ju = j as usize;
            if i <= ju {
                break;
            }
            if c[ju].2 < c[i].2 || c[i].1 < c[ju].1 {
                c.swap(i, ju);
            }
        }
        run = if run >= 0.0 {
            lane_sum(run, c[i + 1..=hi].iter().map(|x| x.1), i + 1, -1.0)
        } else {
            lane_sum(run, c[lo..=i].iter().map(|x| x.1), lo, 1.0)
        };
        if run >= 0.0 {
            hi = i;
        } else {
            lo = i + 1;
        }
    }
    lo
}

/// BFRT for primal infeasibility `delta` (< 0) of the leaving row. Returns the entering column (None if no
/// candidate passes the pivot tolerances) and the columns to flip, in the original order. `nondeg_flip`:
/// NONDEGENFLIP (spx+0x3fc) is on, so the passed breakpoints are re-scanned after the choice.
pub fn bfrt(cands: &[Cand], delta: f64, feas_tol: f64, nondeg_flip: bool) -> (Option<usize>, Vec<usize>) {
    let mut bound = -1e100;
    // (index into cands, contribution, ratio)
    let mut c: Vec<(usize, f64, f64)> = Vec::new();
    let mut best = usize::MAX;
    let mut cbest = 0.0;
    let mut amin = 0.0f64;
    let order = cands.iter().enumerate().filter(|(_, x)| x.back).chain(cands.iter().enumerate().filter(|(_, x)| !x.back));
    for (t, x) in order {
        if x.a >= -1e-10 {
            continue;
        }
        amin = amin.min(x.a);
        let ratio = x.d / x.a;
        if ratio < bound {
            continue;
        }
        let (contrib, upd) = if x.back {
            (delta * -2.0, true)
        } else {
            let raw = -x.range * x.a;
            (raw.min(delta * -2.0), feas_tol - delta < raw)
        };
        let nb = if upd && x.d >= 0.0 { bound.max(OPT_TOL / x.a + ratio) } else { bound };
        if best == usize::MAX || ratio > c[best].2 + 1e-10 {
            best = c.len();
            cbest = contrib;
        } else if ratio >= c[best].2 - 1e-10 {
            cbest = cbest.min(contrib);
            if ratio > c[best].2 {
                best = c.len();
            }
        }
        c.push((t, contrib, ratio));
        bound = nb;
    }
    if c.is_empty() {
        return (None, Vec::new());
    }
    let mut lo = if delta + cbest < 0.0 {
        quickselect(&mut c, delta)
    } else {
        c.swap(0, best);
        0
    };
    if lo > 0 && lo < c.len() && cands[c[lo].0].d <= OPT_TOL {
        lo = 0;
    }
    let thr = (-amin * 1e-13).max(PIVOT_TOL);
    let (mut theta, mut hmax) = (-1e100f64, -1e100f64);
    let mut rec: Vec<usize> = Vec::new();
    for &(t, _, _) in c.iter().skip(lo) {
        let x = &cands[t];
        if -x.a >= thr && x.d < theta * x.a {
            rec.push(t);
            let h = (OPT_TOL + x.d) / x.a;
            hmax = hmax.max(h);
            if x.d >= 0.0 {
                theta = theta.max(h);
            }
        }
    }
    let mut q: Option<usize> = rec.first().copied();
    let mut best_s = 0.0;
    for &t in &rec {
        let x = &cands[t];
        if x.d <= hmax * x.a && x.a < best_s * x.w() {
            best_s = x.a / x.w();
            q = Some(t);
        }
    }
    if let (Some(qt), true) = (q, nondeg_flip) {
        // C:1500-1610: a passed breakpoint t with 0 <= d_t <= θ_q·α_t − 1000·OptTol and a larger |α|/w' enters,
        // w' = (count + ((θ_q·α_t − d_t)/OptTol)^0.25)·(0.3f + 0.7); the flips are then re-filtered with θ_t.
        // shortcut: the pow term is added only while spx+0x178 < 10; Rust keeps no spx+0x178, taken as 0.
        let thq = cands[qt].d / cands[qt].a;
        let mut score = best_s;
        let mut new = None;
        for &(t, _, _) in &c[..lo] {
            let x = &cands[t];
            if x.d < 0.0 {
                continue;
            }
            let dv = thq * x.a;
            if x.d <= dv - 1000.0 * OPT_TOL {
                let w = (x.cnt + ((dv - x.d) / OPT_TOL).powf(0.25)) * x.fac;
                if x.a < score * w {
                    score = x.a / w;
                    new = Some(t);
                }
            }
        }
        if let Some(nt) = new {
            let th = cands[nt].d / cands[nt].a;
            let flips = c[..lo].iter().filter(|&&(t, _, _)| t != nt && cands[t].d <= cands[t].a * th).map(|&(t, _, _)| cands[t].j);
            return (Some(cands[nt].j), flips.collect());
        }
    }
    let flips = match q {
        Some(qt) => c[..lo]
            .iter()
            .filter(|&&(t, _, _)| t != qt && cands[t].d <= cands[t].a * hmax)
            .map(|&(t, _, _)| cands[t].j)
            .collect(),
        None => c[..lo].iter().map(|&(t, _, _)| cands[t].j).collect(),
    };
    (q.map(|t| cands[t].j), flips)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn cand(j: usize, a: f64, d: f64, range: f64) -> Cand {
        Cand { j, a, d, range, back: false, cnt: 1.0, fac: 1.0 }
    }
    #[test]
    fn passes_cheap_breakpoints_then_enters() {
        // slope 10: column 0 (ratio -1, cost 2) and 1 (ratio -2, cost 3) are passed, column 2 (cost 20) enters
        let c = [cand(0, -1.0, 1.0, 2.0), cand(1, -1.0, 2.0, 3.0), cand(2, -2.0, 6.0, 10.0), cand(3, -1.0, 9.0, 1.0)];
        let (q, flips) = bfrt(&c, -10.0, 1e-6, false);
        assert_eq!(q, Some(2));
        assert_eq!(flips, vec![0, 1]);
    }
    #[test]
    fn sparse_column_wins_harris_tie() {
        let mut c = [cand(0, -2.0, 0.0, 1e30), cand(1, -1.5, 0.0, 1e30)];
        c[0].cnt = 4.0;
        assert_eq!(bfrt(&c, -1.0, 1e-6, false).0, Some(1));
    }
}
