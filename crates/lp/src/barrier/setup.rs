//! 05 §2 Barrier internal LP (docs/m3_ipm.md §1): `2ca086` → `07d3ce` (scaling) → `2e9086` (row permutation) →
//! `30f5b9` (free-column bounds, BARLOOSE, types, boxed u = ub − lb) → `2eb47b` (rows ascending in each column) →
//! `312a1e` (shift by lb). Port of `lp_from_model` in tools/proto/bar_ipm.py (64/64 bit-equal to the oracle there).
#![allow(clippy::needless_range_loop)] // index loops over parallel arrays, as in the original
use crate::scaling::scale_flag3;
use rsolver_model::Model;

const BIG: f64 = 1e30;

/// Internal LP of the barrier workspace B: n columns then m slacks, rows in ordering order, `A x + s = b`.
/// The slack of a non-'=' row has coefficient +1 and lower bound 0; '=' rows have no slack (type 0).
#[derive(Clone, Debug)]
pub struct Lp {
    pub m: usize,
    pub n: usize,
    /// A by columns, rows ascending (new numbering).
    pub beg: Vec<usize>,
    pub ind: Vec<usize>,
    pub val: Vec<f64>,
    /// A by rows, columns ascending (the `30f5b9` row loops and `A x`).
    rbeg: Vec<usize>,
    rind: Vec<usize>,
    rval: Vec<f64>,
    /// position of each row-wise entry in the column arrays (B+0x120)
    rpos: Vec<usize>,
    /// b after the lb shift, c (minimisation: sense · obj), lb (shift), ub (u = ub − lb for boxed columns).
    pub b: Vec<f64>,
    pub c: Vec<f64>,
    pub lb: Vec<f64>,
    pub ub: Vec<f64>,
    /// B+0x640 types (m+n): bit0 lower bound, bit1 finite upper bound; slack 1, or 0 on '=' rows.
    pub t: Vec<u8>,
    pub eq: Vec<bool>,
    /// T&1 (m+n) and T&2 (n).
    pub lo: Vec<bool>,
    pub up: Vec<bool>,
    pub objscale: f64,
    /// D+0x60 times sense (the internal problem minimises sense · objective).
    pub objcon: f64,
    pub sense: f64,
    /// B+0x618 = Σ c_j lb_j from `312a1e`.
    pub cshift: f64,
    /// B+0x210
    pub ncomp: usize,
    /// nonzeros of A (B+0x5a8 = 2 nnz drives the work counter)
    pub nnz: usize,
    /// B+0x168, B+0x178
    pub bnorminf: f64,
    pub cnorminf: f64,
    /// B+0x260 = (√(m+1) + ‖b‖₂)·tol, B+0x268 = (√(n+1) + ‖c‖₂)·tol
    pub ptol: f64,
    pub dtol: f64,
    col_scale: Vec<f64>,
    /// row factors and the ordering position of each model row (for the dual values in the model's space)
    row_scale: Vec<f64>,
    perm: Vec<usize>,
}

/// Internal LP from the model, rows placed at `perm[i]` (ordering result).
pub fn lp_from_model(md: &Model, perm: &[i32], tol: f64) -> Lp {
    let (m, n) = (md.num_constrs(), md.num_vars());
    let sc = scale_flag3(md);
    let sense = md.sense as f64;
    let sg: Vec<f64> = md.row_sense.iter().map(|&s| if s == b'>' { -1.0 } else { 1.0 }).collect();
    let p: Vec<usize> = perm.iter().map(|&k| k as usize).collect();
    let (mut beg, mut ind, mut val) = (vec![0], vec![], vec![]);
    for j in 0..n {
        let b0 = md.col_beg[j] as usize;
        let mut e: Vec<(usize, f64)> = (b0..b0 + md.col_len[j] as usize)
            .map(|k| {
                let i = md.col_ind[k] as usize;
                (p[i], sg[i] * md.col_val[k] * sc.row[i] * sc.col[j])
            })
            .collect();
        e.sort_by_key(|x| x.0);
        ind.extend(e.iter().map(|x| x.0));
        val.extend(e.iter().map(|x| x.1));
        beg.push(ind.len());
    }
    let (mut cnt, mut rbeg) = (vec![0usize; m], vec![0usize; m + 1]);
    for &i in &ind {
        cnt[i] += 1;
    }
    for i in 0..m {
        rbeg[i + 1] = rbeg[i] + cnt[i];
    }
    let (mut rind, mut rval, mut at) = (vec![0; ind.len()], vec![0.0; ind.len()], rbeg[..m].to_vec());
    let mut rpos = vec![0; ind.len()];
    for j in 0..n {
        for k in beg[j]..beg[j + 1] {
            let i = ind[k];
            rind[at[i]] = j;
            rval[at[i]] = val[k];
            rpos[at[i]] = k;
            at[i] += 1;
        }
    }
    let (mut b0, mut eq) = (vec![0.0; m], vec![false; m]);
    for i in 0..m {
        b0[p[i]] = sg[i] * md.rhs[i] * sc.row[i];
        eq[p[i]] = md.row_sense[i] == b'=';
    }
    let c: Vec<f64> = (0..n).map(|j| sense * md.obj[j] * sc.col[j] * sc.obj).collect();
    let lbd: Vec<f64> = (0..n).map(|j| if md.lb[j] <= -BIG { md.lb[j] } else { md.lb[j] / sc.col[j] }).collect();
    let ubd: Vec<f64> = (0..n).map(|j| if md.ub[j] >= BIG { md.ub[j] } else { md.ub[j] / sc.col[j] }).collect();
    // 2ca086 takes ncomp and the norms before 30f5b9 changes bounds and 312a1e shifts b
    let ncomp = lbd.iter().filter(|&&v| v > -BIG).count() + ubd.iter().filter(|&&v| v < BIG).count()
        + eq.iter().filter(|&&e| !e).count();
    let mut b: Vec<f64> = (0..m).map(|i| if eq[i] { b0[i] } else { b0[i].min(BIG) }).collect();
    let mut lp = Lp {
        m,
        n,
        beg,
        ind,
        val,
        rbeg,
        rind,
        rval,
        rpos,
        b: vec![],
        c,
        lb: lbd.clone(),
        ub: ubd.clone(),
        t: vec![],
        eq,
        lo: vec![],
        up: vec![],
        objscale: sc.obj,
        objcon: sense * md.objcon,
        sense,
        cshift: 0.0,
        ncomp,
        nnz: md.num_nz(),
        bnorminf: b0.iter().fold(0.0, |a: f64, v| a.max(v.abs())),
        cnorminf: 0.0,
        ptol: 0.0,
        dtol: 0.0,
        col_scale: sc.col,
        row_scale: sc.row,
        perm: p,
    };
    lp.cnorminf = lp.c.iter().fold(0.0, |a: f64, v| a.max(v.abs()));
    lp.ptol = ((m as f64 + 1.0).sqrt() + dot(&b0, &b0).sqrt()) * tol;
    lp.dtol = ((n as f64 + 1.0).sqrt() + dot(&lp.c, &lp.c).sqrt()) * tol;
    let free: Vec<usize> = (0..n).filter(|&j| lbd[j] <= -BIG && ubd[j] >= BIG).collect();
    if !free.is_empty() {
        let (plb, pub_) = lp.propagate(&b0);
        for j in free {
            if plb[j].abs().min(pub_[j].abs()) < 1e6 {
                if pub_[j].abs() <= plb[j].abs() {
                    lp.ub[j] = pub_[j] + 0.01;
                } else {
                    lp.lb[j] = plb[j] - 0.01;
                }
            }
        }
    }
    lp.loosen(&b0);
    let (lb, ub) = (&lp.lb, &mut lp.ub);
    lp.t = (0..n).map(|j| (lb[j] > -BIG) as u8 + 2 * (ub[j] < BIG) as u8).collect();
    lp.t.extend(lp.eq.iter().map(|&e| (!e) as u8));
    for j in 0..n {
        if lp.t[j] == 3 {
            ub[j] -= lb[j];
        }
    }
    for j in 0..n {
        // 312a1e: b −= A_j lb_j and c·lb in column order
        if lb[j] > -BIG && lb[j] != 0.0 {
            for k in lp.beg[j]..lp.beg[j + 1] {
                b[lp.ind[k]] -= lp.val[k] * lb[j];
            }
            lp.cshift += lp.c[j] * lb[j];
        }
    }
    lp.b = b;
    lp.lo = lp.t.iter().map(|&t| t & 1 != 0).collect();
    lp.up = lp.t[..n].iter().map(|&t| t & 2 != 0).collect();
    lp
}

pub(crate) fn dot(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b).fold(0.0, |s, (x, y)| s + x * y)
}

impl Lp {
    /// `2d3b3e`: A x[..n] + slacks (row-wise, columns ascending).
    pub fn ax(&self, x: &[f64]) -> Vec<f64> {
        (0..self.m)
            .map(|i| {
                let r = (self.rbeg[i]..self.rbeg[i + 1]).fold(0.0, |s, k| s + self.rval[k] * x[self.rind[k]]);
                if self.eq[i] { r } else { r + x[self.n + i] }
            })
            .collect()
    }

    /// B+0x630..B+0x634: first column with a finite upper bound and one past the last (empty: (n, n)).
    pub(crate) fn ub_range(&self) -> (usize, usize) {
        match (self.up.iter().position(|&u| u), self.up.iter().rposition(|&u| u)) {
            (Some(a), Some(b)) => (a, b + 1),
            _ => (self.n, self.n),
        }
    }

    /// Row i of A as (column, position in the column arrays), columns ascending.
    pub(crate) fn row_entries(&self, i: usize) -> impl Iterator<Item = (usize, usize)> + '_ {
        (self.rbeg[i]..self.rbeg[i + 1]).map(|k| (self.rind[k], self.rpos[k]))
    }

    /// Offset of row i's first entry in the row-wise arrays (its 16-byte phase decides the `2d3b3e` peel).
    pub(crate) fn row_start(&self, i: usize) -> usize {
        self.rbeg[i]
    }

    /// `2d3d91`: [Aᵀy; y_i on non-'=' rows, 0 on '=' rows].
    pub fn aty(&self, y: &[f64]) -> Vec<f64> {
        let mut v: Vec<f64> =
            (0..self.n).map(|j| (self.beg[j]..self.beg[j + 1]).fold(0.0, |s, k| s + self.val[k] * y[self.ind[k]])).collect();
        v.extend((0..self.m).map(|i| if self.eq[i] { 0.0 } else { y[i] }));
        v
    }

    /// Internal column values back to the model's space: undo the lb shift and the column scale.
    /// (slack, d, y) of the model rows / columns as `7a069` unscales them: slack_i/r_i, d_j/(s_j·ω), −(r_i/ω)·y_i,
    /// with d = z − s (lower minus upper bound dual) and the slack of row i its slack column; `>` rows negated.
    pub fn duals_orig(&self, x: &[f64], z: &[f64], s: &[f64], y: &[f64]) -> (Vec<f64>, Vec<f64>, Vec<f64>) {
        let w = self.objscale;
        let slack = (0..self.m).map(|i| if self.eq[self.perm[i]] { 0.0 } else { x[self.n + self.perm[i]] } / self.row_scale[i]).collect();
        let d = (0..self.n).map(|j| (z[j] - s[j]) / (self.col_scale[j] * w)).collect();
        let yo = (0..self.m).map(|i| -((self.row_scale[i] / w) * y[self.perm[i]])).collect(); // stored negated
        (slack, d, yo)
    }

    pub fn x_orig(&self, x: &[f64]) -> Vec<f64> {
        (0..self.n).map(|j| (if self.lb[j] > -BIG { x[j] + self.lb[j] } else { x[j] }) * self.col_scale[j]).collect()
    }

    /// `30f5b9` free-column branch: two Gauss–Seidel passes of row activity bounds (`a7d29c`) on copies of the
    /// bounds; '=' rows also run with s = −1.
    fn propagate(&self, rhs: &[f64]) -> (Vec<f64>, Vec<f64>) {
        let (mut lb, mut ub) = (self.lb.clone(), self.ub.clone());
        for _ in 0..2 {
            for i in 0..self.m {
                let (cj, av) = (&self.rind[self.rbeg[i]..self.rbeg[i + 1]], &self.rval[self.rbeg[i]..self.rbeg[i + 1]]);
                for sgn in if self.eq[i] { &[-1.0, 1.0][..] } else { &[1.0][..] } {
                    let (mut ninf, mut act) = (0, 0.0);
                    for (&j, &v) in cj.iter().zip(av) {
                        let a = v * sgn;
                        if a <= 0.0 {
                            if ub[j] >= BIG {
                                ninf += 1;
                            } else {
                                act += a * ub[j];
                            }
                        } else if lb[j] <= -BIG {
                            ninf += 1;
                        } else {
                            act += lb[j] * a;
                        }
                    }
                    let slack = sgn * rhs[i] - act;
                    if slack < -1e-6 {
                        continue;
                    }
                    if ninf == 0 {
                        for (&j, &v) in cj.iter().zip(av) {
                            let a = v * sgn;
                            if a > 1e-6 {
                                ub[j] = ub[j].min(slack / a + lb[j]);
                            } else if a < -1e-6 {
                                lb[j] = lb[j].max(slack / a + ub[j]);
                            }
                        }
                    } else if ninf == 1 {
                        // the infinite term (or the last entry when none matches, as the prototype's loop leaves it)
                        let k = (0..cj.len())
                            .find(|&k| {
                                let a = av[k] * sgn;
                                (a > 0.0 && lb[cj[k]] <= -BIG) || (a < 0.0 && ub[cj[k]] >= BIG)
                            })
                            .unwrap_or(cj.len() - 1);
                        let (j, a) = (cj[k], av[k] * sgn);
                        if a > 1e-6 {
                            ub[j] = slack / a;
                        } else if a < -1e-6 {
                            lb[j] = slack / a;
                        }
                    }
                }
            }
        }
        (lb, ub)
    }

    /// `30f5b9` BARLOOSE (env+0x2bdc, default 1): drop (ub = 1e100) the upper bound of a boxed column when the row
    /// already implies it; a row drops at most one bound per side.
    fn loosen(&mut self, rhs: &[f64]) {
        let (lb, ub) = (&self.lb, &mut self.ub);
        for i in 0..self.m {
            let r = self.rbeg[i]..self.rbeg[i + 1];
            let (mut neg, mut pos, mut mx, mut mn) = (0, 0, 0.0, 0.0);
            for k in r.clone() {
                let (j, a) = (self.rind[k], self.rval[k]);
                let (lv, uv) = (lb[j] * a, ub[j] * a);
                if a <= 0.0 {
                    neg += 1;
                    mx += lv;
                    mn += uv;
                } else {
                    pos += 1;
                    mx += uv;
                    mn += lv;
                }
            }
            let mut hi = if self.eq[i] { mx - rhs[i] } else { 1e100 };
            let mut lo = rhs[i] - mn;
            for k in r {
                let (j, a) = (self.rind[k], self.rval[k]);
                if lb[j] > -BIG && ub[j] < BIG {
                    let rng = ub[j] - lb[j];
                    if a <= 1e-4 {
                        if a < -1e-4 && (hi < a * 0.9999 * rng || (pos == 0 && hi - 1e-10 < a * rng)) {
                            ub[j] = 1e100;
                            lo = 1e100;
                        }
                    } else if lo < a * 0.9999 * rng || (neg == 0 && lo - 1e-10 < a * rng) {
                        ub[j] = 1e100;
                        hi = 1e100;
                    }
                }
            }
        }
    }
}
