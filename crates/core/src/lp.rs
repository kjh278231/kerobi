//! Dense two-phase primal simplex with Bland's rule on a standard-form copy.
//! shortcut: dense tableau, O(m·n) memory; switch to a sparse revised simplex
//! with LU updates (notes 13) when models exceed a few thousand rows.
use crate::{Problem, Solution, Status, INF};

const EPS: f64 = 1e-9;

/// Standard form: min c·y, T y = b, y >= 0. Keeps how to map y back to x.
struct Standard {
    tab: Vec<Vec<f64>>, // m rows of n+1 (last = rhs), b >= 0
    c: Vec<f64>,
    ncols: usize,
    // x_j = shift_j + y[pos_j] - y[neg_j] (neg_j None if x_j bounded below)
    map: Vec<(f64, usize, Option<usize>)>,
}

fn to_standard(p: &Problem) -> Option<Standard> {
    let n = p.obj.len();
    let mut map = Vec::with_capacity(n);
    let mut ncols = 0;
    // extra rows for finite upper bounds / ranged rows
    let mut rows: Vec<(Vec<(usize, f64)>, f64)> = Vec::new(); // (coefs over y, rhs), equality
    let mut c = Vec::new();
    for j in 0..n {
        let (lb, ub) = (p.lb[j], p.ub[j]);
        if lb > ub + EPS {
            return None;
        }
        if lb > -INF {
            map.push((lb, ncols, None));
            c.push(p.obj[j]);
            ncols += 1;
            if ub < INF {
                // y_j + s = ub - lb
                c.push(0.0);
                rows.push((vec![(ncols - 1, 1.0), (ncols, 1.0)], ub - lb));
                ncols += 1;
            }
        } else if ub < INF {
            // x = ub - y
            map.push((ub, usize::MAX, Some(ncols)));
            c.push(-p.obj[j]);
            ncols += 1;
        } else {
            map.push((0.0, ncols, Some(ncols + 1)));
            c.push(p.obj[j]);
            c.push(-p.obj[j]);
            ncols += 2;
        }
    }
    // substitute x into each constraint row
    for (coefs, lo, hi) in &p.rows {
        let mut ycoefs = Vec::new();
        let mut shift = 0.0;
        for &(j, a) in coefs {
            let (s, pos, neg) = map[j];
            shift += a * s;
            if pos != usize::MAX {
                ycoefs.push((pos, a));
            }
            if let Some(ng) = neg {
                ycoefs.push((ng, -a));
            }
        }
        let (lo, hi) = (lo - shift, hi - shift);
        if lo > hi + EPS {
            return None;
        }
        if (lo - hi).abs() <= EPS {
            rows.push((ycoefs, lo));
        } else {
            if hi < INF {
                let mut r = ycoefs.clone();
                r.push((ncols, 1.0));
                c.push(0.0);
                ncols += 1;
                rows.push((r, hi));
            }
            if lo > -INF {
                let mut r = ycoefs;
                r.push((ncols, -1.0));
                c.push(0.0);
                ncols += 1;
                rows.push((r, lo));
            }
        }
    }
    let mut tab = Vec::with_capacity(rows.len());
    for (coefs, rhs) in rows {
        let mut row = vec![0.0; ncols + 1];
        for (j, a) in coefs {
            row[j] += a;
        }
        row[ncols] = rhs;
        if rhs < 0.0 {
            row.iter_mut().for_each(|v| *v = -*v);
        }
        tab.push(row);
    }
    Some(Standard { tab, c, ncols, map })
}

fn pivot(tab: &mut [Vec<f64>], cost: &mut [f64], r: usize, s: usize) {
    let pv = tab[r][s];
    tab[r].iter_mut().for_each(|v| *v /= pv);
    let prow = tab[r].clone();
    for (i, row) in tab.iter_mut().enumerate() {
        if i != r && row[s].abs() > EPS {
            let f = row[s];
            row.iter_mut().zip(&prow).for_each(|(v, p)| *v -= f * p);
        }
    }
    let f = cost[s];
    if f.abs() > EPS {
        cost.iter_mut().zip(&prow).for_each(|(v, p)| *v -= f * p);
    }
}

/// Runs simplex on `tab` with reduced costs `cost` (last entry = -objective).
/// `basis[i]` = basic column of row i. Returns false if unbounded.
fn run(tab: &mut [Vec<f64>], cost: &mut [f64], basis: &mut [usize], ncols: usize) -> bool {
    loop {
        // Bland: first column with negative reduced cost
        let Some(s) = (0..ncols).find(|&j| cost[j] < -EPS) else { return true };
        let mut r = None;
        let mut best = INF;
        for (i, row) in tab.iter().enumerate() {
            if row[s] > EPS {
                let ratio = row[ncols] / row[s];
                if ratio < best - EPS || (ratio < best + EPS && r.is_some_and(|rr| basis[i] < basis[rr])) {
                    best = ratio;
                    r = Some(i);
                }
            }
        }
        let Some(r) = r else { return false };
        pivot(tab, cost, r, s);
        basis[r] = s;
    }
}

pub fn solve(p: &Problem) -> Solution {
    let n = p.obj.len();
    let infeasible = Solution { status: Status::Infeasible, obj_val: INF, x: vec![0.0; n] };
    let Some(Standard { mut tab, c, ncols, map }) = to_standard(p) else { return infeasible };
    let m = tab.len();
    // Phase 1: artificial column per row, minimise their sum.
    let total = ncols + m;
    for (i, row) in tab.iter_mut().enumerate() {
        let rhs = row[ncols];
        row.truncate(ncols);
        row.extend((0..m).map(|k| if k == i { 1.0 } else { 0.0 }));
        row.push(rhs);
    }
    let mut basis: Vec<usize> = (ncols..total).collect();
    let mut cost = vec![0.0; total + 1];
    for row in &tab {
        for j in 0..=total {
            cost[j] -= row[j];
        }
    }
    cost[ncols..total].fill(0.0);
    run(&mut tab, &mut cost, &mut basis, total);
    if -cost[total] > 1e-7 {
        return infeasible;
    }
    // Drive remaining artificials out of the basis (degenerate rows), then drop them.
    for i in 0..m {
        if basis[i] >= ncols {
            if let Some(s) = (0..ncols).find(|&j| tab[i][j].abs() > EPS) {
                pivot(&mut tab, &mut cost, i, s);
                basis[i] = s;
            }
        }
    }
    let keep: Vec<usize> = (0..m).filter(|&i| basis[i] < ncols).collect();
    let mut tab: Vec<Vec<f64>> = keep.iter().map(|&i| {
        let mut row = tab[i][..ncols].to_vec();
        row.push(tab[i][total]);
        row
    }).collect();
    let mut basis: Vec<usize> = keep.iter().map(|&i| basis[i]).collect();
    // Phase 2: original costs, priced out against the current basis.
    let mut cost = c.clone();
    cost.push(0.0);
    for (i, &bj) in basis.iter().enumerate() {
        let f = cost[bj];
        if f.abs() > EPS {
            let row = tab[i].clone();
            cost.iter_mut().zip(&row).for_each(|(v, p)| *v -= f * p);
        }
    }
    if !run(&mut tab, &mut cost, &mut basis, ncols) {
        return Solution { status: Status::Unbounded, obj_val: -INF, x: vec![0.0; n] };
    }
    let mut y = vec![0.0; ncols];
    for (i, &bj) in basis.iter().enumerate() {
        y[bj] = tab[i][ncols];
    }
    let x: Vec<f64> = map.iter().map(|&(s, pos, neg)| {
        s + if pos != usize::MAX { y[pos] } else { 0.0 } - neg.map_or(0.0, |k| y[k])
    }).collect();
    let obj_val = x.iter().zip(&p.obj).map(|(a, b)| a * b).sum();
    Solution { status: Status::Optimal, obj_val, x }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn textbook_lp() {
        // max 3x + 2y  s.t. x + y <= 4, x + 3y <= 6, x,y >= 0  -> (4, 0), obj 12
        let p = Problem {
            obj: vec![-3.0, -2.0],
            rows: vec![(vec![(0, 1.0), (1, 1.0)], -INF, 4.0), (vec![(0, 1.0), (1, 3.0)], -INF, 6.0)],
            lb: vec![0.0, 0.0],
            ub: vec![INF, INF],
            integer: vec![false, false],
        };
        let s = solve(&p);
        assert_eq!(s.status, Status::Optimal);
        assert!((s.obj_val + 12.0).abs() < 1e-7, "{s:?}");
    }

    #[test]
    fn free_var_and_ranged_row() {
        // min x, 1 <= x <= 3 via a ranged row, x free -> 1
        let p = Problem {
            obj: vec![1.0],
            rows: vec![(vec![(0, 1.0)], 1.0, 3.0)],
            lb: vec![-INF],
            ub: vec![INF],
            integer: vec![false],
        };
        let s = solve(&p);
        assert_eq!(s.status, Status::Optimal);
        assert!((s.x[0] - 1.0).abs() < 1e-7, "{s:?}");
    }

    #[test]
    fn infeasible_and_unbounded() {
        let p = Problem { obj: vec![1.0], rows: vec![(vec![(0, 1.0)], 5.0, INF)], lb: vec![0.0], ub: vec![1.0], integer: vec![false] };
        assert_eq!(solve(&p).status, Status::Infeasible);
        let p = Problem { obj: vec![-1.0], rows: vec![], lb: vec![0.0], ub: vec![INF], integer: vec![false] };
        assert_eq!(solve(&p).status, Status::Unbounded);
    }
}
