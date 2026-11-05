//! 04 §4 Basis factorization. Singleton phase of the original `1da0b2` (row singletons from a stack, slack rows
//! without structural entries, then column singletons in basis order), a Markowitz phase for what is left, the
//! column-oriented dense solves of `214344`/`2126cd` and Forrest–Tomlin updates (`05b257`). On a triangular basis
//! the bits depend only on the pivot order, so this reproduces the original x_B exactly (60/60 bench LPs with a
//! valid iteration-0 snapshot). `factor` reports an incomplete factorization on a singular basis.
use crate::spx::Spx;

/// Drop tolerance (GURO_PAR_ZERO, spx+0x3a8 / LU+0x10).
pub const ZERO: f64 = 1e-13;

pub struct Lu {
    /// Pivot steps in factor order: (row, basis position, diagonal). FT updates replace the diagonal in place.
    pub steps: Vec<(usize, usize, f64)>,
    /// Step at which each row was pivoted (-1 if not).
    pub prow: Vec<i64>,
    pub head: Vec<usize>,
    /// Step of each basis position.
    kpos: Vec<usize>,
    /// L column of step k: (row i, multiplier) with x_i −= l·x_{r_k}, applied in factor order.
    lcol: Vec<Vec<(usize, f64)>>,
    /// Lᵀ row of step k: (earlier step j, multiplier), descending j (LU+0x70/0x78/0x80).
    lrow: Vec<Vec<(usize, f64)>>,
    /// U off-diagonals: column of step k as (row, u) over steps earlier in `order`, and the same entries by row as
    /// (later step, u).
    ucol: Vec<Vec<(usize, f64)>>,
    urow: Vec<Vec<(usize, f64)>>,
    /// U order of the steps (factor order, then each FT update moves the replaced step to the end).
    order: Vec<usize>,
    /// Forrest–Tomlin row etas in creation order: (row r, [(row i, μ)]) with x_r −= Σ μ·x_i.
    retas: Vec<(usize, Vec<(usize, f64)>)>,
    /// Sparse-solve multiplier LU+0x104: a sparse Lᵀ needs `mult·nnz ≤ m` (036ade L5709).
    sparse_mult: usize,
    /// Lᵀ DAG level of each step (LU+0x130) when BTRAN takes the level path (LU+0x140 ≥ 0, 036ade L5757).
    level: Option<Vec<usize>>,
    /// U has off-diagonals (Markowitz part or updates): BTRAN leaves the iteration-0 emulation.
    general: bool,
    pub n_updates: usize,
}

/// Markowitz threshold (MarkowitzTol default 2⁻⁷, spx+0x390).
pub const MARKOWITZ_TOL: f64 = 0.0078125;

impl Lu {
    pub fn complete(&self) -> bool {
        self.steps.len() == self.head.len()
    }

    pub fn factor(spx: &Spx, head: &[usize]) -> Lu {
        let (m, n) = (spx.m, spx.n);
        let mut rc = vec![0usize; m];
        for &j in head {
            if j < n {
                for (i, _) in spx.col(j) {
                    rc[i] += 1;
                }
            } else {
                rc[j - n] += 1 + m; // slack-basic row marker
            }
        }
        let mut rows: Vec<Vec<(usize, f64)>> = vec![Vec::new(); m];
        for (pos, &j) in head.iter().enumerate() {
            if j < n {
                for (i, a) in spx.col(j) {
                    if rc[i] <= m {
                        rows[i].push((pos, a));
                    }
                }
            }
        }
        let mut lu = Lu {
            steps: Vec::with_capacity(m),
            prow: vec![-1; m],
            head: head.to_vec(),
            kpos: Vec::new(),
            lcol: Vec::new(),
            lrow: Vec::new(),
            ucol: Vec::new(),
            urow: Vec::new(),
            order: Vec::new(),
            retas: Vec::new(),
            sparse_mult: 2,
            level: None,
            general: false,
            n_updates: 0,
        };
        let mut ppos = vec![-1i64; m];
        let pivot = |lu: &mut Lu, ppos: &mut Vec<i64>, r: usize, pos: usize, d: f64| {
            let k = lu.steps.len() as i64;
            lu.prow[r] = k;
            ppos[pos] = k;
            lu.steps.push((r, pos, d));
        };
        // row singletons, LIFO stack seeded in ascending row order
        let mut stack: Vec<usize> = (0..m).filter(|&i| rc[i] == 1).collect();
        while let Some(r) = stack.pop() {
            if rc[r] == 0 {
                continue;
            }
            rc[r] = 0;
            let &(pos, d) = rows[r].iter().find(|(p, _)| ppos[*p] == -1).expect("row singleton has a free column");
            pivot(&mut lu, &mut ppos, r, pos, d);
            for (i, _) in spx.col(head[pos]) {
                if lu.prow[i] == -1 {
                    rc[i] -= 1;
                    if rc[i] == 1 {
                        stack.push(i);
                    }
                }
            }
        }
        // slacks of rows without structural entries
        for (pos, &j) in head.iter().enumerate() {
            if j >= n && rc[j - n] == m + 1 {
                rc[j - n] = 1;
                pivot(&mut lu, &mut ppos, j - n, pos, 1.0);
            }
        }
        // remaining column singletons and slacks, in basis order
        for (pos, &j) in head.iter().enumerate() {
            if ppos[pos] >= 0 {
                continue;
            }
            if j < n {
                if spx.len[j] == 1 {
                    let (i, a) = spx.col(j).next().unwrap();
                    if rc[i] <= m && lu.prow[i] < 0 {
                        pivot(&mut lu, &mut ppos, i, pos, a);
                    }
                }
            } else {
                let i = j - n;
                rc[i] -= m;
                pivot(&mut lu, &mut ppos, i, pos, 1.0);
            }
        }
        let ns = lu.steps.len();
        lu.lcol = vec![Vec::new(); m];
        lu.ucol = vec![Vec::new(); m];
        if ns < m && !lu.markowitz(spx, &mut ppos) {
            return lu; // singular: incomplete
        }
        // L and U of the singleton steps come straight from the columns (their pivot rows have no other active
        // entries, so the Markowitz values are unaffected); taken after Markowitz so every row has its step
        for k in 0..ns {
            let (_, pos, d) = lu.steps[k];
            for (i, a) in spx.col(head[pos]) {
                let ki = lu.prow[i];
                if ki > k as i64 {
                    lu.lcol[k].push((i, a / d));
                } else if ki < k as i64 {
                    lu.ucol[k].push((i, a));
                }
            }
        }
        lu.finish();
        lu
    }

    /// Markowitz threshold pivoting on the part the singleton phases leave (rows and positions not yet pivoted).
    /// shortcut: a plain lowest-cost search, not `036ade`'s candidate lists, tie rules, 2x2 blocks or dense dgetrf
    /// switch; the pivot sequence (so the bits) differs from Gurobi's on non-triangular bases.
    fn markowitz(&mut self, spx: &Spx, ppos: &mut [i64]) -> bool {
        let m = self.head.len();
        // active rows by position with current values; U entries of the column in rows already pivoted
        let mut arow: Vec<Vec<(usize, f64)>> = vec![Vec::new(); m];
        let mut acol: Vec<Vec<usize>> = vec![Vec::new(); m];
        let mut early: Vec<Vec<(usize, f64)>> = vec![Vec::new(); m]; // by position
        for pos in 0..m {
            if ppos[pos] >= 0 {
                continue;
            }
            for (i, a) in spx.col(self.head[pos]) {
                if self.prow[i] < 0 {
                    arow[i].push((pos, a));
                    acol[pos].push(i);
                } else {
                    early[pos].push((i, a));
                }
            }
        }
        let mut upos: Vec<Vec<(usize, f64)>> = vec![Vec::new(); m]; // U row of a Markowitz step, by position
        let get = |row: &Vec<(usize, f64)>, p: usize| row.iter().find(|e| e.0 == p).map(|e| e.1);
        use std::collections::BTreeSet;
        let mut cols: BTreeSet<(usize, usize)> = (0..m).filter(|&p| ppos[p] < 0).map(|p| (acol[p].len(), p)).collect();
        let mut rset: BTreeSet<(usize, usize)> = (0..m).filter(|&i| self.prow[i] < 0).map(|i| (arow[i].len(), i)).collect();
        while !cols.is_empty() {
            // lowest Markowitz cost (r−1)(c−1) by count order; stop once no smaller count can beat the best
            let mut best: Option<(usize, usize, f64, usize)> = None; // (row, pos, value, cost)
            let mut seen = 0;
            let cmax = |p: usize, arow: &Vec<Vec<(usize, f64)>>, acol: &Vec<Vec<usize>>| {
                acol[p].iter().filter_map(|&i| get(&arow[i], p)).fold(0.0f64, |a, v| a.max(v.abs()))
            };
            let maxc = cols.iter().next_back().map_or(0, |e| e.0).max(rset.iter().next_back().map_or(0, |e| e.0));
            for k in 1..=maxc {
                if best.is_some_and(|b| b.3 <= (k - 1) * (k - 1)) || seen >= 4 {
                    break;
                }
                for &(_, p) in cols.range((k, 0)..(k + 1, 0)) {
                    let cm = cmax(p, &arow, &acol);
                    for &i in &acol[p] {
                        let v = get(&arow[i], p).unwrap();
                        if v.abs() >= MARKOWITZ_TOL * cm && v.abs() > ZERO {
                            let cost = (arow[i].len() - 1) * (k - 1);
                            if best.is_none_or(|b| cost < b.3 || (cost == b.3 && v.abs() > b.2.abs())) {
                                best = Some((i, p, v, cost));
                            }
                        }
                    }
                    seen += 1;
                    if seen >= 4 {
                        break;
                    }
                }
                for &(_, i) in rset.range((k, 0)..(k + 1, 0)) {
                    for &(p, v) in &arow[i] {
                        let cm = cmax(p, &arow, &acol);
                        if v.abs() >= MARKOWITZ_TOL * cm && v.abs() > ZERO {
                            let cost = (k - 1) * (acol[p].len() - 1);
                            if best.is_none_or(|b| cost < b.3 || (cost == b.3 && v.abs() > b.2.abs())) {
                                best = Some((i, p, v, cost));
                            }
                        }
                    }
                    seen += 1;
                    if seen >= 4 {
                        break;
                    }
                }
            }
            let Some((r, p, v, _)) = best else { return false };
            let k = self.steps.len();
            self.prow[r] = k as i64;
            ppos[p] = k as i64;
            self.steps.push((r, p, v));
            self.ucol[k] = std::mem::take(&mut early[p]);
            cols.remove(&(acol[p].len(), p));
            rset.remove(&(arow[r].len(), r));
            let prow_r: Vec<(usize, f64)> = arow[r].iter().copied().filter(|e| e.0 != p).collect();
            for &(p2, _) in &prow_r {
                cols.remove(&(acol[p2].len(), p2));
                acol[p2].retain(|&i| i != r);
            }
            for &i in &acol[p].clone() {
                if i == r {
                    continue;
                }
                rset.remove(&(arow[i].len(), i));
                let l = get(&arow[i], p).unwrap() / v;
                self.lcol[k].push((i, l));
                arow[i].retain(|e| e.0 != p);
                for &(p2, u) in &prow_r {
                    if let Some(e) = arow[i].iter_mut().find(|e| e.0 == p2) {
                        e.1 -= l * u;
                    } else {
                        arow[i].push((p2, -(l * u)));
                        acol[p2].push(i);
                    }
                }
                rset.insert((arow[i].len(), i));
            }
            for &(p2, _) in &prow_r {
                cols.insert((acol[p2].len(), p2));
            }
            acol[p].clear();
            upos[k] = prow_r;
            arow[r].clear();
        }
        // Markowitz rows' U entries become entries of the later columns
        for k in 0..self.steps.len() {
            for &(p2, u) in &std::mem::take(&mut upos[k]) {
                let k2 = ppos[p2] as usize;
                self.ucol[k2].push((self.steps[k].0, u));
            }
        }
        true
    }

    /// Positions, U by row, Lᵀ rows, LU+0x104 and the Lᵀ levels (036ade L5700-5912).
    fn finish(&mut self) {
        let m = self.head.len();
        self.kpos = vec![0; m];
        for (k, &(_, pos, _)) in self.steps.iter().enumerate() {
            self.kpos[pos] = k;
        }
        self.order = (0..m).collect();
        self.urow = vec![Vec::new(); m];
        for k in 0..m {
            for &(i, u) in &self.ucol[k] {
                self.urow[self.prow[i] as usize].push((k, u));
            }
        }
        self.general = self.ucol.iter().any(|c| !c.is_empty());
        self.lrow = vec![Vec::new(); m];
        for k in (0..m).rev() {
            for &(i, l) in &self.lcol[k] {
                self.lrow[self.prow[i] as usize].push((k, l));
            }
        }
        let lnnz: usize = self.lrow.iter().map(Vec::len).sum();
        let t = lnnz as f64 / m as f64 + 1.0;
        let t = t * t * 5.0;
        self.sparse_mult = if lnnz <= m { 2 } else if m as f64 <= t { m } else { t as usize };
        if m >= 250 && self.sparse_mult < m {
            let mut lev = vec![0; m];
            for k in (0..m).rev() {
                for &(j, _) in &self.lrow[k] {
                    lev[j] = lev[j].max(lev[k] + 1);
                }
            }
            self.level = Some(lev);
        }
    }

    /// BTRAN `221173`: yᵀ = xᵀB⁻¹ for x given as (basis position, value) in input order. Returns (row, value) in the
    /// original's output order: sparse when it stays sparse, else all m rows in row order (output nnz = −1).
    /// Callers sum in this order, so the order is part of the result. shortcut: the Lᵀ linear-scan path (L956) is not
    /// ported, and with U off-diagonals or row etas a plain dense solve stands in for the original's paths.
    pub fn btran(&self, x: &[(usize, f64)]) -> Vec<(usize, f64)> {
        assert!(self.complete(), "singular basis");
        if self.general || !self.retas.is_empty() {
            return self.btran_general(x);
        }
        let m = self.head.len();
        // Uᵀ is the diagonal: sparse (21a941, inputs in order) when (int)(LU+0x108 + LU+0x110)·nnz ≤ m, both 1.5 here
        let sparse_u = 3 * x.len() <= m;
        let mut xs: Vec<(usize, f64)> = Vec::with_capacity(x.len());
        let mut dense = vec![0.0; m]; // by step
        for &(pos, v) in x {
            let k = self.kpos[pos];
            let t = v / self.steps[k].2;
            if sparse_u {
                if ZERO < t.abs() {
                    xs.push((k, t));
                }
            } else if !(v.abs() <= ZERO || t.abs() <= ZERO) {
                dense[k] = t;
            }
        }
        if !sparse_u || self.sparse_mult * xs.len() > m {
            for &(k, t) in &xs {
                dense[k] = t;
            }
            for k in (0..m).rev() {
                // dense Lᵀ (L1576): values at or below ZERO are kept but not propagated
                let v = dense[k];
                if ZERO < v.abs() {
                    for &(j, l) in &self.lrow[k] {
                        dense[j] -= l * v;
                    }
                }
            }
            let mut y = vec![(0, 0.0); m];
            for (k, &(r, _, _)) in self.steps.iter().enumerate() {
                y[r] = (r, dense[k]);
            }
            return y;
        }
        let mut val: Vec<Option<f64>> = vec![None; m]; // touched steps
        for &(k, t) in &xs {
            val[k] = Some(t);
        }
        let mut out = Vec::new();
        let mut visit = |k: usize, val: &mut Vec<Option<f64>>, touched: &mut dyn FnMut(usize)| {
            let v = val[k].take().unwrap();
            if v.abs() >= ZERO {
                out.push((self.steps[k].0, v));
                for &(j, l) in &self.lrow[k] {
                    if val[j].is_none() {
                        val[j] = Some(0.0);
                        touched(j);
                    }
                    val[j] = Some(val[j].unwrap() - l * v);
                }
            }
        };
        match &self.level {
            None => {
                // heap path: descending step
                let mut heap: std::collections::BinaryHeap<usize> = xs.iter().map(|&(k, _)| k).collect();
                while let Some(k) = heap.pop() {
                    visit(k, &mut val, &mut |j| heap.push(j));
                }
            }
            Some(lev) => {
                // level path (L619): level buckets from the lowest, each in insertion order; fills go to higher levels
                let depth = lev.iter().copied().max().unwrap_or(0);
                let mut buckets: Vec<Vec<usize>> = vec![Vec::new(); depth + 1];
                for &(k, _) in &xs {
                    buckets[lev[k]].push(k);
                }
                for l in 0..=depth {
                    let bucket = std::mem::take(&mut buckets[l]);
                    for k in bucket {
                        visit(k, &mut val, &mut |j| buckets[lev[j]].push(j));
                    }
                }
            }
        }
        out
    }

    /// Dense BTRAN through Uᵀ (in U order), the row etas transposed (newest first) and Lᵀ; all m rows in row order.
    fn btran_general(&self, x: &[(usize, f64)]) -> Vec<(usize, f64)> {
        let m = self.head.len();
        let mut w = vec![0.0; m]; // by step
        for &(pos, v) in x {
            w[self.kpos[pos]] += v;
        }
        let mut y = vec![0.0; m]; // by row
        for &k in &self.order {
            let (r, _, d) = self.steps[k];
            let t = w[k] / d;
            if t.abs() <= ZERO {
                continue;
            }
            y[r] = t;
            for &(j, u) in &self.urow[k] {
                w[j] -= u * t;
            }
        }
        for (r, mu) in self.retas.iter().rev() {
            let yr = y[*r];
            if yr != 0.0 {
                for &(i, v) in mu {
                    y[i] -= v * yr;
                }
            }
        }
        for k in (0..m).rev() {
            let r = self.steps[k].0;
            let v = y[r];
            if ZERO < v.abs() {
                for &(j, l) in &self.lrow[k] {
                    let rj = self.steps[j].0;
                    y[rj] -= l * v;
                }
            }
        }
        y.into_iter().enumerate().collect()
    }

    /// FTRAN through L and the row etas (x by row); the result is the FT spike of the column.
    pub fn ftran_lower(&self, x: &mut [f64]) {
        for (k, &(r, _, _)) in self.steps.iter().enumerate() {
            let xr = x[r];
            if xr.abs() > ZERO {
                for &(i, l) in &self.lcol[k] {
                    x[i] -= l * xr;
                }
            }
        }
        for (r, mu) in &self.retas {
            let mut v = x[*r];
            for &(i, f) in mu {
                v -= f * x[i];
            }
            x[*r] = v;
        }
    }

    /// FTRAN through U (x by row in, by row out: x_{r_k} holds the value of basis position p_k).
    pub fn ftran_upper(&self, x: &mut [f64]) {
        for &k in self.order.iter().rev() {
            let (r, _, d) = self.steps[k];
            if x[r].abs() <= ZERO || (x[r] / d).abs() <= ZERO {
                x[r] = 0.0;
                continue;
            }
            x[r] /= d;
            let xr = x[r];
            for &(i, u) in &self.ucol[k] {
                x[i] -= u * xr;
            }
        }
    }

    /// Dense FTRAN: x (indexed by row) := B⁻¹x, returned by basis position.
    pub fn ftran_dense(&self, mut x: Vec<f64>) -> Vec<f64> {
        assert!(self.complete(), "singular basis");
        self.ftran_lower(&mut x);
        self.ftran_upper(&mut x);
        let mut xb = vec![0.0; self.head.len()];
        for &(r, pos, _) in &self.steps {
            xb[pos] = x[r];
        }
        xb
    }

    /// Forrest–Tomlin update (05b257): basis position `pos` takes the column whose spike (L and row etas applied,
    /// by row) is `spike`. The replaced step moves to the end of the U order; its old U row is eliminated into a row
    /// eta. Returns false when the new diagonal is below ZERO (caller refactors).
    pub fn update(&mut self, pos: usize, var: usize, spike: &[f64]) -> bool {
        let kp = self.kpos[pos];
        let rp = self.steps[kp].0;
        for &(i, _) in &std::mem::take(&mut self.ucol[kp]) {
            let ki = self.prow[i] as usize;
            self.urow[ki].retain(|e| e.0 != kp);
        }
        // eliminate row rp over the steps after kp in U order
        let rank: std::collections::HashMap<usize, usize> = self.order.iter().enumerate().map(|(a, &k)| (k, a)).collect();
        let mut row: std::collections::BTreeMap<usize, (usize, f64)> = std::collections::BTreeMap::new(); // rank -> (step, value)
        for &(j, u) in &std::mem::take(&mut self.urow[kp]) {
            row.insert(rank[&j], (j, u));
            self.ucol[j].retain(|e| e.0 != rp);
        }
        let mut mu = Vec::new();
        let mut diag = spike[rp];
        while let Some((_, (j, v))) = row.pop_first() {
            let (rj, _, dj) = self.steps[j];
            let f = v / dj;
            if f == 0.0 {
                continue;
            }
            mu.push((rj, f));
            diag -= f * spike[rj];
            for &(j2, u2) in &self.urow[j] {
                let e = row.entry(rank[&j2]).or_insert((j2, 0.0));
                e.1 -= f * u2;
            }
        }
        self.order.retain(|&k| k != kp);
        self.order.push(kp);
        for (i, &s) in spike.iter().enumerate() {
            if i != rp && s != 0.0 {
                self.ucol[kp].push((i, s));
                self.urow[self.prow[i] as usize].push((kp, s));
            }
        }
        self.steps[kp].2 = diag;
        if !mu.is_empty() {
            self.retas.push((rp, mu));
        }
        self.head[pos] = var;
        self.general = true;
        self.n_updates += 1;
        diag.abs() > ZERO
    }
}

/// Basic solution x_B = B⁻¹(b − N·x_N) with nonbasic values from the status array
/// (−1 lower, −2 upper, −4 fixed at lower, others 0); columns subtracted in order 0..n+m.
pub fn basic_solution(spx: &Spx, vstat: &[i32]) -> Vec<f64> {
    let head = basis_head(spx.m, vstat);
    let mut rhs = spx.b.clone();
    for (j, &s) in vstat.iter().enumerate() {
        let xj = match s {
            -2 => spx.u[j],
            -1 | -4 => spx.l[j],
            _ => 0.0,
        };
        if s < 0 && xj != 0.0 {
            for (i, a) in spx.col(j) {
                rhs[i] -= a * xj;
            }
        }
    }
    Lu::factor(spx, &head).ftran_dense(rhs)
}

fn basis_head(m: usize, vstat: &[i32]) -> Vec<usize> {
    let mut head = vec![usize::MAX; m];
    for (j, &s) in vstat.iter().enumerate() {
        if s >= 0 {
            head[s as usize] = j;
        }
    }
    head
}

/// Iteration-0 reduced costs d (n+m) and exact DSE weights w (m, by basis position) for the basis in `vstat`.
/// d: basic 0, upper-bounded nonbasic (−2) negated, fixed structurals (−4) keep c_j (they are not in the row-wise
/// nonbasic copy); y = c_Bᵀ B⁻¹ from a dense BTRAN. w_p = ‖e_pᵀB⁻¹‖² summed in BTRAN output order (1bae9c EXACTNORM).
pub fn dual_start(spx: &Spx, vstat: &[i32]) -> (Vec<f64>, Vec<f64>) {
    let (m, n) = (spx.m, spx.n);
    let lu = Lu::factor(spx, &basis_head(m, vstat));
    let cb: Vec<(usize, f64)> = (0..m).map(|p| (p, spx.c[lu.head[p]])).collect();
    let mut y = vec![0.0; m];
    for (r, v) in lu.btran(&cb) {
        y[r] = v;
    }
    let d = (0..n + m)
        .map(|j| match vstat[j] {
            s if s >= 0 => 0.0,
            -4 if j < n => spx.c[j],
            s => {
                let t = spx.col(j).fold(spx.c[j], |t, (i, a)| t - a * y[i]);
                if s == -2 { -t } else { t }
            }
        })
        .collect();
    let w = (0..m).map(|p| sum_squares(&lu.btran(&[(p, 1.0)]).iter().map(|e| e.1).collect::<Vec<_>>())).collect();
    (d, w)
}

/// Σv² as 1bae9c sums it (L1971-2110): in sequence below 8 values, else 8 lanes over the 8-aligned head combined
/// ((0+2)+(4+6)) + ((1+3)+(5+7)) (asm 0xc3a26; the decompiled order is wrong), then the tail in sequence.
/// The vector buffer is 16-byte aligned, so no element is peeled first.
pub fn sum_squares(v: &[f64]) -> f64 {
    let (head, tail) = v.as_chunks::<8>();
    let mut a = [0.0; 8];
    for c in head {
        for q in 0..8 {
            a[q] += c[q] * c[q];
        }
    }
    let w = ((a[0] + a[2]) + (a[4] + a[6])) + ((a[1] + a[3]) + (a[5] + a[7])); // 0.0 when there is no full chunk
    tail.iter().fold(w, |w, t| w + t * t)
}

#[cfg(test)]
mod tests {
    use super::{sum_squares, Lu};
    use crate::scaling::Scaling;
    use crate::spx::Spx;

    /// m×n dense matrix as an Spx (slack columns implied).
    fn spx_of(a: &[Vec<f64>]) -> Spx {
        let (m, n) = (a.len(), a[0].len());
        let (mut beg, mut len, mut ind, mut val) = (vec![0i64], vec![], vec![], vec![]);
        for j in 0..n {
            let mut k = 0;
            for (i, row) in a.iter().enumerate() {
                if row[j] != 0.0 {
                    ind.push(i as i32);
                    val.push(row[j]);
                    k += 1;
                }
            }
            len.push(k);
            beg.push(beg[j] + k as i64);
        }
        Spx {
            m,
            n,
            beg,
            len,
            ind,
            val,
            sense: vec![b'<'; m],
            c: vec![0.0; n + m],
            l: vec![0.0; n + m],
            u: vec![1.0; n + m],
            b: vec![0.0; m],
            scaling: Scaling { col: vec![1.0; n], row: vec![1.0; m], obj: 1.0 },
        }
    }

    fn column(spx: &Spx, j: usize) -> Vec<f64> {
        let mut x = vec![0.0; spx.m];
        for (i, a) in spx.col(j) {
            x[i] = a;
        }
        x
    }

    /// B·x for x by position.
    fn times(spx: &Spx, head: &[usize], x: &[f64]) -> Vec<f64> {
        let mut y = vec![0.0; spx.m];
        for (p, &j) in head.iter().enumerate() {
            for (i, a) in spx.col(j) {
                y[i] += a * x[p];
            }
        }
        y
    }

    fn check(spx: &Spx, lu: &Lu) {
        let m = spx.m;
        for j in 0..spx.n + m {
            let x = lu.ftran_dense(column(spx, j));
            let y = times(spx, &lu.head, &x);
            let e = column(spx, j);
            assert!(y.iter().zip(&e).all(|(a, b)| (a - b).abs() < 1e-9), "ftran col {j}");
        }
        for p in 0..m {
            let mut y = vec![0.0; m];
            for (r, v) in lu.btran(&[(p, 1.0)]) {
                y[r] = v;
            }
            for (q, &j) in lu.head.iter().enumerate() {
                let dot: f64 = spx.col(j).map(|(i, a)| a * y[i]).sum();
                assert!((dot - if p == q { 1.0 } else { 0.0 }).abs() < 1e-9, "btran row {p}");
            }
        }
    }

    #[test]
    fn mixed_singleton_and_markowitz_basis_of_sc50a() {
        // a basis the dual simplex reaches on sc50a: 8 singleton steps, the rest by Markowitz
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../../re/bench/data/netlib/sc50a.mps.gz");
        let md = rsolver_model::mps::read_file(path).unwrap();
        let spx = Spx::new(&md);
        let head = vec![
            47, 40, 39, 38, 3, 25, 18, 7, 2, 17, 6, 1, 69, 58, 13, 14, 4, 11, 48, 50, 37, 19, 22, 8, 9, 10, 15, 30, 12, 29, 35,
            20, 21, 36, 26, 27, 28, 34, 33, 31, 32, 0, 23, 24, 41, 42, 43, 44, 45, 46,
        ];
        let lu = Lu::factor(&spx, &head);
        assert!(lu.complete());
        check(&spx, &lu);
    }

    #[test]
    fn markowitz_and_forrest_tomlin_reproduce_the_basis_inverse() {
        let mut s = 12345u64;
        let mut rnd = || {
            s ^= s << 13;
            s ^= s >> 7;
            s ^= s << 17;
            s
        };
        let (m, n) = (7, 12);
        let a: Vec<Vec<f64>> =
            (0..m).map(|_| (0..n).map(|_| if rnd() % 3 != 0 { (rnd() % 19) as f64 - 9.0 } else { 0.0 }).collect()).collect();
        let spx = spx_of(&a);
        // a structural-heavy basis needs the Markowitz phase; keep slacks where columns are missing
        let mut head: Vec<usize> = (0..m).map(|i| n + i).collect();
        let mut lu = Lu::factor(&spx, &head);
        for j in 0..n {
            for p in (0..m).filter(|&p| head[p] >= n) {
                let mut h = head.clone();
                h[p] = j;
                let t = Lu::factor(&spx, &h);
                if t.complete() && t.steps.iter().all(|s| s.2.abs() > 1e-6) {
                    head = h;
                    lu = t;
                    break;
                }
            }
        }
        assert!(head.iter().filter(|&&j| j < n).count() >= 4);
        assert!(lu.general, "basis should need the Markowitz phase");
        check(&spx, &lu);
        // Forrest-Tomlin updates against the same basis changes
        let mut done = 0;
        for j in 0..n + m {
            if lu.head.contains(&j) {
                continue;
            }
            let mut x = column(&spx, j);
            lu.ftran_lower(&mut x);
            let spike = x.clone();
            lu.ftran_upper(&mut x);
            let Some(pos) = (0..m).find(|&p| x[lu.steps[lu.kpos[p]].0].abs() > 0.5) else { continue };
            assert!(lu.update(pos, j, &spike));
            check(&spx, &lu);
            done += 1;
        }
        assert!(done >= 4);
    }

    #[test]
    fn sum_squares_lane_order() {
        // 9 values: lanes take 0..8, the 9th is added last; (1e16 + 1) + ... shows the grouping
        let v = [1e8, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0];
        let lanes = ((1e16 + 1.0) + (1.0 + 1.0)) + ((1.0 + 1.0) + (1.0 + 1.0));
        assert_eq!(sum_squares(&v), lanes + 1.0);
        assert_eq!(sum_squares(&v[..7]), v[..7].iter().fold(0.0, |w, t| w + t * t));
    }
}
