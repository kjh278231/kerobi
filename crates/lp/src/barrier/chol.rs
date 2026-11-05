//! 05 §2.6 Supernodal LDLᵀ of `A Θ Aᵀ + diag(θ_slack)` and the solve `2ee6c5`, bit for bit as Gurobi computes
//! them with Threads=1 (docs/m3_chol.md; prototype tools/proto/bar_chol.py, checked against `tools/bar_oracle.py
//! --chol --solve`). Same three calls as `normal::Factor`.
//!
//! Factor, per block c (width w, rows R, `2d8ee5`): assembly (`32d72f` or `32e116` order) into a dense w×w buffer
//! and the (len−w)×w rectangle; updates from the descendant supernodes in ascending order, then from earlier blocks
//! of the same supernode (`32c49d`: A = −(L∘d) rows, MKL DGEMM); dense left-looking LDLᵀ with 8-lane dots and the
//! pivot rule |d| ≤ 1e-16·|diag| → d = 0, 1/d = 0; MKL DTRSM of the rectangle, then rectangle ×= 1/d (1 if dropped).
//! Storage: one array, blocks in order, each a packed lower triangle by rows then the rectangle by rows.
//!
//! Vectorised SSE2 loops of Gurobi sum in 8 lanes (2 in the 4-row solve groups) and peel one term when the address
//! is 8 mod 16. All value arrays (A by columns and rows, the factor storage) start 16-byte aligned (oracle), so the
//! peel is the parity of the element offset.
#![allow(clippy::needless_range_loop)] // index loops over parallel arrays, as in the original
use super::mkl;
use super::ordering::Symbolic;
use super::setup::Lp;

/// `acc + Σ prods` in the order of Gurobi's SSE2 reductions: sequential below `lanes + peel` terms (and below 8 for
/// 8 lanes); else `peel` terms sequentially, `lanes` accumulators with lane 0 seeded by `acc`, reduced
/// ((l0+l2)+(l4+l6)) + ((l1+l3)+(l5+l7)) (2 lanes: l0+l1), then the tail sequentially. `a − p` is passed as −p.
pub(crate) fn vsum(mut acc: f64, prods: &[f64], peel: usize, lanes: usize) -> f64 {
    let n = prods.len();
    if n < peel + lanes {
        return prods.iter().fold(acc, |a, &p| a + p);
    }
    for &p in &prods[..peel] {
        acc += p;
    }
    let nv = n - (n - peel) % lanes;
    let mut l = [0.0f64; 8];
    l[0] = acc;
    for k in peel..nv {
        l[(k - peel) % lanes] += prods[k];
    }
    acc = if lanes == 8 { ((l[0] + l[2]) + (l[4] + l[6])) + ((l[1] + l[3]) + (l[5] + l[7])) } else { l[0] + l[1] };
    prods[nv..].iter().fold(acc, |a, &p| a + p)
}

pub struct Factor<'a> {
    lp: &'a Lp,
    /// block starts, widths (F+0x78 at the start, 0 elsewhere), supernode start of each column
    starts: Vec<usize>,
    w: Vec<usize>,
    snode: Vec<i32>,
    /// rows of the block starting at c: `ri[rp[c]..rp[c] + len[c]]`
    ri: Vec<usize>,
    rp: Vec<usize>,
    len: Vec<usize>,
    /// descendant supernodes of each block (F+0x88/0x90/0x98): (supernode start, first row position ≥ c)
    desc: Vec<Vec<(usize, usize)>>,
    /// storage offset of each block, factor values
    off: Vec<usize>,
    lx: Vec<f64>,
    /// F+0xe0 d, F+0xe8 1/d (0 when dropped), F+0xf0 assembled diagonal
    d: Vec<f64>,
    e: Vec<f64>,
    diag0: Vec<f64>,
    /// F+0x80 = max(480, max w², m): row chunk numerator of the rectangle updates
    f80: usize,
    th: Vec<f64>,
    /// row → local index in the block being factored (usize::MAX elsewhere)
    mp: Vec<usize>,
}

impl<'a> Factor<'a> {
    pub fn new(lp: &'a Lp, sym: &Symbolic) -> Factor<'a> {
        let m = lp.m;
        let w: Vec<usize> = sym.blocks.iter().map(|&b| b.max(0) as usize).collect();
        let starts: Vec<usize> = (0..m).filter(|&c| w[c] > 0).collect();
        let len: Vec<usize> = sym.col_len.iter().map(|&l| l as usize).collect();
        let ri: Vec<usize> = sym.row_idx.iter().map(|&r| r as usize).collect();
        let (mut off, mut tot) = (vec![0; m], 0);
        for &c in &starts {
            off[c] = tot;
            tot += w[c] * (w[c] + 1) / 2 + (len[c] - w[c]) * w[c];
        }
        // block of each row; each supernode adds itself to the blocks its rows fall in, ascending
        let mut blk = vec![0; m];
        for &c in &starts {
            blk[c..c + w[c]].fill(c);
        }
        let mut desc = vec![vec![]; m];
        for s0 in (0..m).filter(|&c| sym.snode[c] > 0) {
            let rows = &ri[sym.col_ptr[s0]..sym.col_ptr[s0] + len[s0]];
            let mut last = usize::MAX;
            for (p, &r) in rows.iter().enumerate().skip(sym.snode[s0] as usize) {
                if blk[r] != last {
                    last = blk[r];
                    desc[last].push((s0, p));
                }
            }
        }
        let maxw = w.iter().copied().max().unwrap_or(0);
        Factor {
            lp,
            starts,
            w,
            snode: sym.snode.clone(),
            ri,
            rp: sym.col_ptr.clone(),
            len,
            desc,
            off,
            lx: vec![0.0; tot],
            d: vec![0.0; m],
            e: vec![0.0; m],
            diag0: vec![0.0; m],
            f80: 480.max(maxw * maxw).max(m),
            th: vec![],
            mp: vec![usize::MAX; m],
        }
    }

    /// Factor with column scaling `theta` (n) and slack scaling `theta_slack` (m, 0 on '=' rows). Returns
    /// (F+0x100 = min |d/diag| before dropping, 0 on NaN; number of dropped pivots).
    pub fn factor(&mut self, theta: &[f64], theta_slack: &[f64]) -> (f64, usize) {
        self.th = [theta, theta_slack].concat();
        let mut min_ratio = 1e100f64;
        let mut dropped = 0;
        for bi in 0..self.starts.len() {
            let c = self.starts[bi];
            let (w, ln) = (self.w[c], self.len[c]);
            for q in 0..ln {
                self.mp[self.ri[self.rp[c] + q]] = q;
            }
            let mut dd = vec![0.0; w * w];
            self.assemble(c, &mut dd);
            for q in 0..w {
                self.diag0[c + q] = dd[q * w + q];
            }
            let mut srcs = vec![];
            for &(s0, p) in &self.desc[c] {
                let mut s = s0;
                while s < s0 + self.snode[s0] as usize {
                    srcs.push((s, p + s0 - s));
                    s += self.w[s];
                }
            }
            let mut s = (c as i64 + self.snode[c].min(0) as i64) as usize;
            while s != c {
                srcs.push((s, c - s));
                s += self.w[s];
            }
            for (s, p0) in srcs {
                self.update(s, p0, c, &mut dd);
            }
            // dense kernel (2d8ee5): the general path only if the buffer has an off-diagonal nonzero
            let dense = (0..w).any(|i| (0..i).any(|k| dd[i * w + k] != 0.0));
            let mut wv = vec![0.0; w];
            let mut prods = vec![0.0; w];
            for j in 0..w {
                if dense {
                    for p in 0..j {
                        wv[p] = -dd[j * w + p] * self.d[c + p];
                    }
                    for i in j..w {
                        for p in 0..j {
                            prods[p] = dd[i * w + p] * wv[p];
                        }
                        let v = vsum(dd[i * w + j], &prods[..j], 0, 8);
                        dd[i * w + j] = if i > j { v * self.e[c + j] } else { v };
                        if i == j {
                            dropped += self.pivot(c + j, v, &mut min_ratio);
                            dd[j * w + j] = self.d[c + j];
                        }
                    }
                } else {
                    let v = dd[j * w + j];
                    dropped += self.pivot(c + j, v, &mut min_ratio);
                    dd[j * w + j] = self.d[c + j];
                }
            }
            let (o, tri) = (self.off[c], w * (w + 1) / 2);
            let rect = &mut self.lx[o + tri..o + tri + (ln - w) * w];
            if dense && ln > w {
                mkl::trsm_lutu(w, ln - w, &dd, w, rect, w);
            }
            for r in 0..ln - w {
                for q in 0..w {
                    let ev = self.e[c + q];
                    rect[r * w + q] *= if ev != 0.0 { ev } else { 1.0 };
                }
            }
            let mut k = o;
            for i in 0..w {
                self.lx[k..k + i + 1].copy_from_slice(&dd[i * w..i * w + i + 1]);
                k += i + 1;
            }
            for q in 0..ln {
                self.mp[self.ri[self.rp[c] + q]] = usize::MAX;
            }
        }
        (min_ratio, dropped)
    }

    fn pivot(&mut self, k: usize, dk: f64, min_ratio: &mut f64) -> usize {
        let d0 = self.diag0[k];
        if dk.is_nan() {
            *min_ratio = 0.0;
        } else if d0 != 0.0 {
            *min_ratio = min_ratio.min((dk / d0).abs());
        }
        if d0.abs() * 1e-16 < dk.abs() {
            (self.d[k], self.e[k]) = (dk, 1.0 / dk);
            0
        } else {
            (self.d[k], self.e[k]) = (0.0, 0.0);
            1
        }
    }

    /// `32d72f` (w < 3 or w·len > 30000: per column k, the columns of row k in order) or `32e116` (a column of A
    /// is added whole at its first row ≥ c); a_i (a_k θ) products, the slack θ after row k's columns.
    fn assemble(&mut self, c: usize, dd: &mut [f64]) {
        let (lp, w, ln) = (self.lp, self.w[c], self.len[c]);
        let (n, tri, o) = (lp.n, w * (w + 1) / 2, self.off[c]);
        let (mp, th) = (&self.mp, &self.th);
        let rect = &mut self.lx[o + tri..o + tri + (ln - w) * w];
        rect.fill(0.0);
        let mut add = |i: usize, k: usize, v: f64| {
            let (qi, qk) = (mp[i], mp[k]);
            if qi < w {
                dd[qi * w + qk] += v;
            } else {
                rect[(qi - w) * w + qk] += v;
            }
        };
        let (beg, ind, val) = (&lp.beg, &lp.ind, &lp.val);
        let by_row = w < 3 || w * ln > 30000;
        for k in c..c + w {
            for (j, p) in lp.row_entries(k) {
                if by_row {
                    let t = th[j] * val[p];
                    for q in p..beg[j + 1] {
                        add(ind[q], k, val[q] * t);
                    }
                } else if p == beg[j] || ind[p - 1] < c {
                    let mut e = p;
                    while e < beg[j + 1] && mp[ind[e]] < w {
                        e += 1;
                    }
                    for a in p..e {
                        let t = val[a] * th[j];
                        for q in a..beg[j + 1] {
                            add(ind[q], ind[a], val[q] * t);
                        }
                    }
                }
            }
            if !lp.eq[k] {
                add(k, k, th[n + k]);
            }
        }
    }

    /// `32c49d`: block s (factored, rows from position p0 inside the target) into target t.
    fn update(&mut self, s: usize, p0: usize, t: usize, dd: &mut [f64]) {
        let (ws, lens, wt) = (self.w[s], self.len[s], self.w[t]);
        let rs = &self.ri[self.rp[s]..self.rp[s] + lens];
        let mp = &self.mp;
        let (os, ot) = (self.off[s], self.off[t]);
        let (lo, hi) = self.lx.split_at_mut(ot);
        let ls = &lo[os..];
        let rect = &mut hi[wt * (wt + 1) / 2..];
        let tri = ws * (ws + 1) / 2;
        let row = |q: usize| tri + (q - ws) * ws; // storage offset of row position q ≥ ws
        let mut p1 = p0 + 1;
        while p1 < lens && rs[p1] < t + wt {
            p1 += 1;
        }
        let nr = p1 - p0;
        let ds = &self.d[s..s + ws];
        if ws == 1 {
            // scalar path: target w < 5 multiplies L[row] by −(L[col] d), w ≥ 5 multiplies L[col] by −(L[row] d)
            for r in p0..p1 {
                let col = mp[rs[r]];
                for q in r..lens {
                    let g = mp[rs[q]];
                    let v = if wt < 5 { ls[q] * (-ls[r] * ds[0]) } else { ls[r] * (-ls[q] * ds[0]) };
                    if g < wt {
                        dd[g * wt + col] += v;
                    } else {
                        rect[(g - wt) * wt + col] += v;
                    }
                }
            }
            return;
        }
        let mut ta = vec![0.0; nr * ws];
        for i in 0..nr {
            let o = row(p0 + i);
            for q in 0..ws {
                ta[i * ws + q] = -ls[o + q] * ds[q];
            }
        }
        let j0 = mp[rs[p0]];
        let contig = mp[rs[p1 - 1]] == j0 + nr - 1;
        if contig {
            for a in (0..nr).step_by(24) {
                let k = 24.min(nr - a);
                mkl::gemm_tn(a + k, k, ws, &ta, ws, &ls[row(p0 + a)..], ws, &mut dd[j0 * wt + j0 + a * wt..], wt);
            }
        } else {
            let mut cb = vec![0.0; nr * nr];
            for a in (0..nr).step_by(24) {
                let k = 24.min(nr - a);
                mkl::gemm_tn(a + k, k, ws, &ta, ws, &ls[row(p0 + a)..], ws, &mut cb[a * nr..], nr);
            }
            for r in 0..nr {
                let g = mp[rs[p0 + r]];
                for q in 0..=r {
                    dd[g * wt + mp[rs[p0 + q]]] += cb[r * nr + q];
                }
            }
        }
        let mut a = p1;
        while a < lens {
            let mut b = (a + (self.f80 / nr).min(192)).min(lens);
            let g0 = mp[rs[a]];
            if contig && mp[rs[b - 1]] == g0 + (b - a) - 1 {
                while b < lens && mp[rs[b]] == g0 + (b - a) {
                    b += 1;
                }
                mkl::gemm_tn(nr, b - a, ws, &ta, ws, &ls[row(a)..], ws, &mut rect[(g0 - wt) * wt + j0..], wt);
            } else {
                let mut cb = vec![0.0; nr * (b - a)];
                mkl::gemm_tn(nr, b - a, ws, &ta, ws, &ls[row(a)..], ws, &mut cb, nr);
                for r in a..b {
                    let g = mp[rs[r]];
                    for q in p0..p1 {
                        rect[(g - wt) * wt + mp[rs[q]]] += cb[(r - a) * nr + (q - p0)];
                    }
                }
            }
            a = b;
        }
    }

    /// `2ee6c5`: −Θ⁻¹dx + Aᵀdy = rhs_x (m+n), A dx = rhs_y (m), i.e. dy = (LDLᵀ)⁻¹ (rhs_y + A Θ rhs_x) and
    /// dx = Θ(Aᵀdy − rhs_x). Threads=1: the block kernels run once on [0, m) (`31cfeb`/`31d3c7`).
    pub fn solve(&self, rhs_x: &[f64], rhs_y: &[f64]) -> (Vec<f64>, Vec<f64>) {
        let (lp, th) = (self.lp, &self.th);
        let (m, n) = (lp.m, lp.n);
        let v: Vec<f64> = rhs_x.iter().zip(th).map(|(r, t)| r * t).collect();
        let mut pr = vec![];
        let mut y: Vec<f64> = (0..m)
            .map(|i| {
                // 2d3b3e: row-wise, peel by the parity of the row's first value
                pr.clear();
                let r0 = lp.row_start(i);
                pr.extend(lp.row_entries(i).map(|(j, p)| lp.val[p] * v[j]));
                let t = vsum(0.0, &pr, r0 & 1, 8);
                rhs_y[i] + if lp.eq[i] { t } else { t + v[n + i] }
            })
            .collect();
        self.forward(&mut y);
        for (yi, ei) in y.iter_mut().zip(&self.e) {
            *yi *= ei;
        }
        self.backward(&mut y);
        let mut dx = Vec::with_capacity(m + n);
        for j in 0..n {
            // 2d3d91: column-wise, peel by the parity of the column's first value
            let (b0, b1) = (lp.beg[j], lp.beg[j + 1]);
            pr.clear();
            pr.extend((b0..b1).map(|k| lp.val[k] * y[lp.ind[k]]));
            dx.push((vsum(0.0, &pr, b0 & 1, 8) - rhs_x[j]) * th[j]);
        }
        for i in 0..m {
            dx.push(((if lp.eq[i] { 0.0 } else { y[i] }) - rhs_x[n + i]) * th[n + i]);
        }
        (dx, y)
    }

    /// `2efdb8` on one range: per block the in-block substitution (dropped pivots give 0), then the rectangle rows
    /// are updated right-looking, 4 rows at a time with 2 lanes (peel from the 4th row) and then singly with 8.
    fn forward(&self, x: &mut [f64]) {
        let mut pr = vec![];
        for &c in &self.starts {
            let (w, ln, o) = (self.w[c], self.len[c], self.off[c]);
            let l = &self.lx[o..];
            for i in 0..w {
                if i > 0 {
                    let t = i * (i + 1) / 2;
                    pr.clear();
                    pr.extend((0..i).map(|p| -(l[t + p] * x[c + p])));
                    x[c + i] = vsum(x[c + i], &pr, (o + t) & 1, 8);
                }
                if self.d[c + i] == 0.0 && self.e[c + i] == 0.0 {
                    x[c + i] = 0.0;
                }
            }
            let rs = &self.ri[self.rp[c]..self.rp[c] + ln];
            let tri = w * (w + 1) / 2;
            let row = |p: usize| tri + (p - w) * w;
            let mut p = w;
            if w + 3 < ln {
                while p + 4 <= ln {
                    let pe = (o + row(p + 3)) & 1;
                    for k in p..p + 4 {
                        pr.clear();
                        pr.extend((0..w).map(|q| -(l[row(k) + q] * x[c + q])));
                        x[rs[k]] = vsum(x[rs[k]], &pr, pe, 2);
                    }
                    p += 4;
                }
            }
            for k in p..ln {
                pr.clear();
                pr.extend((0..w).map(|q| -(l[row(k) + q] * x[c + q])));
                x[rs[k]] = vsum(x[rs[k]], &pr, (o + row(k)) & 1, 8);
            }
        }
    }

    /// `2efe1f` on one range: blocks in reverse; the rectangle rows from the top in groups of 4
    /// (x −= ((p0+p1)+p2)+p3, rows ascending in the group), the rest singly, then the in-block substitution.
    fn backward(&self, x: &mut [f64]) {
        for &c in self.starts.iter().rev() {
            let (w, ln, o) = (self.w[c], self.len[c], self.off[c]);
            let l = &self.lx[o..];
            let rs = &self.ri[self.rp[c]..self.rp[c] + ln];
            let tri = w * (w + 1) / 2;
            let row = |p: usize| tri + (p - w) * w;
            let mut p = ln; // one past the next row to use
            while p >= w + 4 {
                let (a, b, cc, d) = (row(p - 4), row(p - 3), row(p - 2), row(p - 1));
                let (xa, xb, xc, xd) = (x[rs[p - 4]], x[rs[p - 3]], x[rs[p - 2]], x[rs[p - 1]]);
                for q in 0..w {
                    let s = ((l[a + q] * xa + l[b + q] * xb) + l[cc + q] * xc) + l[d + q] * xd;
                    x[c + q] -= s;
                }
                p -= 4;
            }
            while p > w {
                p -= 1;
                let (r, xr) = (row(p), x[rs[p]]);
                for q in 0..w {
                    x[c + q] -= l[r + q] * xr;
                }
            }
            for i in (1..w).rev() {
                let t = i * (i + 1) / 2;
                let xi = x[c + i];
                for q in 0..i {
                    x[c + q] -= l[t + q] * xi;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::barrier::{normal, ordering, setup};

    /// The supernodal factor and the scalar LDLᵀ (normal.rs) solve the same systems (bit equality with Gurobi is
    /// checked against the oracle: tools/proto/bar_chol.py, docs/m3_chol.md §2).
    #[test]
    fn agrees_with_scalar_ldl() {
        for nm in ["afiro", "sc50a", "adlittle", "fit1p"] { // full row rank (degen2: dropped pivots, other null-space part)
            let p = format!("{}/../../../re/bench/data/netlib/{nm}.mps.gz", env!("CARGO_MANIFEST_DIR"));
            let md = rsolver_model::mps::read_file(&p).unwrap();
            let sym = ordering::analyze(&md);
            let lp = setup::lp_from_model(&md, &sym.perm, 1e-8);
            let (mut a, mut b) = (Factor::new(&lp, &sym), normal::Factor::new(&lp, &sym));
            let th: Vec<f64> = (0..lp.n).map(|j| 0.5 + (j % 7) as f64).collect();
            let ts: Vec<f64> = (0..lp.m).map(|i| if lp.eq[i] { 0.0 } else { 1.0 + (i % 3) as f64 }).collect();
            assert_eq!(a.factor(&th, &ts).1, b.factor(&th, &ts).1, "{nm}");
            let rx: Vec<f64> = (0..lp.m + lp.n).map(|j| (j % 5) as f64 - 2.0).collect();
            let (dx, dy) = a.solve(&rx, &lp.b);
            let (ex, ey) = b.solve(&rx, &lp.b);
            let err = dx.iter().zip(&ex).chain(dy.iter().zip(&ey)).fold(0.0f64, |e, (u, v)| e.max((u - v).abs() / v.abs().max(1.0)));
            assert!(err < 1e-6, "{nm}: {err}");
        }
    }

    #[test]
    fn vsum_orders() {
        // 1 + 2^-53 terms: the order decides whether the small ones survive
        let e = 2f64.powi(-53);
        let p = [e; 9];
        assert_eq!(vsum(1.0, &p[..7], 0, 8), 1.0); // sequential: each e is lost against 1
        // 8 lanes, peel 0: lane 0 = fl(1 + e) = 1, lanes 1..7 = e; ((1 + e) + 2e) + 4e = 1 + 6e
        assert_eq!(vsum(1.0, &p[..8], 0, 8), 1.0 + 6.0 * e);
        assert_eq!(vsum(1.0, &p[..2], 0, 2), 1.0);
    }
}
