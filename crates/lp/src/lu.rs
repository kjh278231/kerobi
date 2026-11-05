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
    /// U order of the steps (LU+0x48 ord: position → step); identity after a factor, permuted by FT updates.
    order: Vec<usize>,
    /// Prefix of U positions with empty U columns as the original tracks it (LU+0x100).
    prefix: usize,
    /// Cache of the highest U position with an empty column (LU+0xe8).
    e8: usize,
    /// Forrest–Tomlin row etas in creation order: (row r, [(row i, μ)]) with x_r −= Σ μ·x_i.
    retas: Vec<(usize, Vec<(usize, f64)>)>,
    /// Sparse-solve multiplier LU+0x104: a sparse Lᵀ needs `mult·nnz ≤ m` (036ade L5709).
    sparse_mult: usize,
    /// Lᵀ DAG level of each step (LU+0x130) when BTRAN takes the level path (LU+0x140 ≥ 0, 036ade L5757).
    level: Option<Vec<usize>>,
    /// U has off-diagonals (Markowitz part or updates): BTRAN leaves the iteration-0 emulation.
    general: bool,
    pub n_updates: usize,
    /// Steps taken by the singleton phases (the rest is the Markowitz kernel).
    pub n_singleton: usize,
    /// U rows of the kernel steps by step: (position, u), moved into `ucol` after the factor.
    kernel_u: Vec<Vec<(usize, f64)>>,
    /// Singular factor: (position, row) pairs of the empty columns and rows in discovery order (tail steps m−1,
    /// m−2, … of `036ade`); 5b257 puts the row's slack at that position.
    pub singular: Vec<(usize, usize)>,
}

/// Count buckets of the `036ade` kernel: circular doubly linked lists; insert goes after the head and becomes the
/// head, walks go head → prv (last inserted first).
struct Buckets {
    head: Vec<i64>,
    nxt: Vec<usize>,
    prv: Vec<usize>,
}

impl Buckets {
    fn new(m: usize) -> Buckets {
        Buckets { head: vec![-1; m + 1], nxt: vec![0; m], prv: vec![0; m] }
    }

    fn ins(&mut self, i: usize, k: usize) {
        let h = self.head[k];
        if h < 0 {
            self.nxt[i] = i;
            self.prv[i] = i;
        } else {
            let h = h as usize;
            self.nxt[i] = self.nxt[h];
            self.nxt[h] = i;
            self.prv[i] = h;
            self.prv[self.nxt[i]] = i;
        }
        self.head[k] = i as i64;
    }

    fn rem(&mut self, i: usize, k: usize) {
        if self.nxt[i] == i {
            self.head[k] = -1;
            return;
        }
        if self.head[k] == i as i64 {
            self.head[k] = self.prv[i] as i64;
        }
        let (n, p) = (self.nxt[i], self.prv[i]);
        self.prv[n] = p;
        self.nxt[p] = n;
    }
}

/// Markowitz threshold (MarkowitzTol default 2⁻⁷, spx+0x390).
pub const MARKOWITZ_TOL: f64 = 0.0078125;

impl Lu {
    /// `9e60b`: estimated nonzeros of row k of B⁻¹ by factor step (P+0x68), for the BTRAN-cost pricing weights.
    /// Each step starts at 1 and absorbs its Lᵀ row, then its U row from the last step down, as `p + q − pq/m`.
    /// Returns the costs (`min(p + 100, 0.1m + 100)`) and false when Σp exceeds m·(0.1m + 100) (cost mode 2,
    /// every cost at the cap).
    pub fn row_cost_estimate(&self) -> (Vec<f64>, bool) {
        let k = self.steps.len();
        let mf = self.head.len() as f64;
        let mut p = vec![1.0; k];
        for s in 0..k {
            for &(t, _) in &self.lrow[s] {
                p[s] = (p[s] + p[t]) - p[t] * p[s] / mf;
            }
        }
        for s in (0..k).rev() {
            for &(t, _) in &self.urow[s] {
                p[s] = (p[s] + p[t]) - p[t] * p[s] / mf;
            }
        }
        let cap = mf * 0.1 + 100.0;
        // shortcut: plain left-to-right sum; the original sums in 8 lanes (only matters at the threshold)
        let total: f64 = p.iter().sum();
        if total <= mf * cap {
            (p.iter().map(|&x| (x + 100.0).min(cap)).collect(), true)
        } else {
            (vec![cap; k], false)
        }
    }

    /// (nnz of L, nnz of U) of the factorization including the diagonals (LU+0x1e0, LU+0x1e8 after a factor).
    pub fn factor_nnz(&self) -> (usize, usize) {
        let m = self.head.len();
        (m + self.lcol.iter().map(|c| c.len()).sum::<usize>(), m + self.ucol.iter().map(|c| c.len()).sum::<usize>())
    }

    /// Entries of all Forrest–Tomlin row etas (LU+0x184).
    pub fn eta_nnz(&self) -> usize {
        self.retas.iter().map(|e| e.1.len()).sum()
    }

    /// Basis positions after a refactor (5b257 LAB_001f474e): the new position of a basic variable is its factor
    /// step. Returns the old position of each new one.
    pub fn renumbered(&self) -> Vec<usize> {
        self.steps.iter().map(|s| s.1).collect()
    }

    /// Relabel basis positions: `old_of_new[p]` is the old position of new position p.
    pub fn relabel(&mut self, old_of_new: &[usize]) {
        let m = self.head.len();
        let mut new_of_old = vec![0; m];
        for (np, &op) in old_of_new.iter().enumerate() {
            new_of_old[op] = np;
        }
        self.head = old_of_new.iter().map(|&op| self.head[op]).collect();
        self.kpos = old_of_new.iter().map(|&op| self.kpos[op]).collect();
        for st in &mut self.steps {
            st.1 = new_of_old[st.1];
        }
    }

    pub fn complete(&self) -> bool {
        self.steps.len() == self.head.len()
    }

    pub fn factor(spx: &Spx, head: &[usize]) -> Lu {
        Lu::factor_tol(spx, head, MARKOWITZ_TOL)
    }

    /// Factor with Markowitz threshold `tol` (MarkowitzTol, raised by the numerics ladder).
    pub fn factor_tol(spx: &Spx, head: &[usize], tol: f64) -> Lu {
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
            prefix: 0,
            e8: 0,
            retas: Vec::new(),
            sparse_mult: 2,
            level: None,
            general: false,
            n_updates: 0,
            n_singleton: 0,
            kernel_u: Vec::new(),
            singular: Vec::new(),
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

        let s1 = lu.steps.len();
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
        // phase 4 (1da0b2 L530–690): column singletons of the active part from a stack seeded in ascending position
        // order; skipped when phases 2–3 pivoted nothing

        let s3 = lu.steps.len();
        {
            let mut cc = vec![0usize; m];
            let mut rcopy: Vec<Vec<usize>> = vec![Vec::new(); m]; // active row copy, ascending position
            for c in 0..m {
                if ppos[c] == -1 {
                    for (i, _) in spx.col(head[c]) {
                        if lu.prow[i] < 0 {
                            cc[c] += 1;
                            rcopy[i].push(c);
                        }
                    }
                }
            }
            let mut stack: Vec<usize> = if s3 != s1 { (0..m).filter(|&c| cc[c] == 1).collect() } else { Vec::new() };
            while let Some(c) = stack.pop() {
                if cc[c] == 0 {
                    continue;
                }
                let (r, d) = spx.col(head[c]).find(|&(i, _)| lu.prow[i] == -1).expect("active entry");
                pivot(&mut lu, &mut ppos, r, c, d);
                for &c2 in &rcopy[r] {
                    if ppos[c2] == -1 {
                        cc[c2] -= 1;
                        if cc[c2] == 1 {
                            stack.push(c2);
                        }
                    }
                }
            }
        }

        let ns = lu.steps.len();
        lu.n_singleton = ns;
        lu.lcol = vec![Vec::new(); m];
        lu.ucol = vec![Vec::new(); m];
        if ns < m && !lu.kernel(spx, &mut ppos, tol) {
            return lu; // singular: incomplete
        }

        // L and U of the singleton steps come straight from the columns (their pivot rows have no other active
        // entries, so the kernel values are unaffected); kernel columns keep their entries in singleton rows as U
        for k in 0..m {
            let (_, pos, d) = lu.steps[k];
            for (i, a) in spx.col(head[pos]) {
                let ki = lu.prow[i] as usize;
                if k < ns && ki > k {
                    lu.lcol[k].push((i, a / d));
                } else if ki < k && ki < ns {
                    lu.ucol[k].push((i, a));
                }
            }
        }
        for (k, ku) in std::mem::take(&mut lu.kernel_u).into_iter().enumerate() {
            let r = lu.steps[k].0;
            for (p2, u) in ku {
                lu.ucol[ppos[p2] as usize].push((r, u));
            }
        }

        lu.finish();
        lu
    }

    /// Sparse Markowitz kernel of `036ade` (L546–2570, docs/m1_notes.md §6.7) on the rows and positions the singleton
    /// phases leave: column singletons, then row singletons, then a Markowitz search over count buckets (LIFO
    /// circular lists, rotated heads), threshold `tol·max < |a|` on the searched line's max, ties to the larger
    /// |a|/max; multipliers a/v, updates a − l·u and fill −(l·u) dropped under ZERO, fill prepended to its lists.
    /// Empty rows and columns pair up in discovery order as the singular tail (`self.singular`).
    /// shortcut: no 2x2 block phase, no dense switches (036ade L1542/L1762: fill > 0.325·n² with n > 50, or
    /// 201 ≤ n ≤ 9999 with a costly search) and no 1e99 pivot cap; those cases stay sparse.
    fn kernel(&mut self, spx: &Spx, ppos: &mut [i64], tol: f64) -> bool {
        let m = self.head.len();
        // element pool: (value, row, position; position usize::MAX = dead). Lists are stored reversed so that
        // pushing prepends; iterate with .rev() for the original order.
        let mut ev: Vec<f64> = Vec::new();
        let mut er: Vec<usize> = Vec::new();
        let mut ec: Vec<usize> = Vec::new();
        const DEAD: usize = usize::MAX;
        let mut rl: Vec<Vec<usize>> = vec![Vec::new(); m];
        let mut cl: Vec<Vec<usize>> = vec![Vec::new(); m];
        let mut rcs = vec![-1i64; m]; // count − 1
        let mut ccs = vec![-1i64; m];
        for pos in 0..m {
            if ppos[pos] >= 0 {
                continue;
            }
            for (i, a) in spx.col(self.head[pos]) {
                if self.prow[i] < 0 && ZERO <= a.abs() {
                    let e = ev.len();
                    ev.push(a);
                    er.push(i);
                    ec.push(pos);
                    rl[i].push(e);
                    cl[pos].push(e);
                    rcs[i] += 1;
                    ccs[pos] += 1;
                }
            }
        }
        let mut rb = Buckets::new(m);
        let mut cb = Buckets::new(m);
        let (mut erows, mut ecols) = (Vec::new(), Vec::new());
        for i in 0..m {
            if self.prow[i] < 0 {
                if rcs[i] < 0 {
                    erows.push(i);
                } else {
                    rb.ins(i, rcs[i] as usize);
                }
            }
            if ppos[i] < 0 {
                if ccs[i] < 0 {
                    ecols.push(i);
                } else {
                    cb.ins(i, ccs[i] as usize);
                }
            }
        }
        // a finished line: its live elements in list order (the list is taken, nothing else walks it again)
        let take_alive = |l: &mut Vec<usize>, ec: &Vec<usize>| -> Vec<usize> {
            let mut v = std::mem::take(l);
            v.retain(|&e| ec[e] != DEAD);
            v.reverse();
            v
        };
        let first_alive = |l: &Vec<usize>, ec: &Vec<usize>| -> usize { *l.iter().rev().find(|&&e| ec[e] != DEAD).expect("live") };
        self.kernel_u = vec![Vec::new(); m];
        let mut w = vec![0.0f64; m];
        let mut seen = vec![false; m];
        let pivot = |lu: &mut Lu, ppos: &mut [i64], r: usize, pos: usize, d: f64| {
            let k = lu.steps.len() as i64;
            lu.prow[r] = k;
            ppos[pos] = k;
            lu.steps.push((r, pos, d));
        };
        loop {
            if cb.head[0] >= 0 {
                // column singleton (L1003–1140): no threshold
                let c = cb.head[0] as usize;
                cb.rem(c, 0);
                let e = first_alive(&cl[c], &ec);
                let r = er[e];
                rb.rem(r, rcs[r] as usize);
                ec[e] = DEAD;
                pivot(self, ppos, r, c, ev[e]);
                let k = self.steps.len() - 1;
                for f in take_alive(&mut rl[r], &ec) {
                    let j = ec[f];
                    self.kernel_u[k].push((j, ev[f]));
                    ec[f] = DEAD;
                    cb.rem(j, ccs[j] as usize);
                    ccs[j] -= 1;
                    if ccs[j] < 0 { ecols.push(j) } else { cb.ins(j, ccs[j] as usize) }
                }
                rcs[r] = -1;
                ccs[c] = -1;
            } else if rb.head[0] >= 0 {
                // row singleton (L821–1002): no threshold
                let r = rb.head[0] as usize;
                rb.rem(r, 0);
                let e = first_alive(&rl[r], &ec);
                let c = ec[e];
                cb.rem(c, ccs[c] as usize);
                let d = ev[e];
                ec[e] = DEAD;
                pivot(self, ppos, r, c, d);
                let k = self.steps.len() - 1;
                for f in take_alive(&mut cl[c], &ec) {
                    let i = er[f];
                    self.lcol[k].push((i, ev[f] / d));
                    ec[f] = DEAD;
                    rb.rem(i, rcs[i] as usize);
                    rcs[i] -= 1;
                    if rcs[i] < 0 { erows.push(i) } else { rb.ins(i, rcs[i] as usize) }
                }
                rcs[r] = -1;
                ccs[c] = -1;
            } else {
                let nact = (m - self.steps.len()) - ecols.len();
                if nact == 0 {
                    break;
                }
                let cidx = (1..m).find(|&k| cb.head[k] >= 0).unwrap_or(m);
                let ridx = (1..m).find(|&k| rb.head[k] >= 0).unwrap_or(m);
                let k0 = cidx.min(ridx);
                if k0 >= m {
                    break; // 0x2732
                }
                let limit = ((k0 as f64 * 0.5) as usize).max(4).min(nact);
                let mut best: Option<usize> = None;
                let (mut bcost, mut bratio, mut ns) = (m as f64 * m as f64, 0.0f64, 0usize);
                let mut bound = cidx as f64 * ridx as f64;
                let mut k = k0;
                'search: while k < m {
                    let fk = k as f64;
                    bound = bound.max(fk * fk);
                    for side in 0..2 {
                        let bk = if side == 0 { &mut cb } else { &mut rb };
                        let h = bk.head[k];
                        if h < 0 {
                            if bcost <= bound {
                                break 'search;
                            }
                            continue;
                        }
                        let h = h as usize;
                        let mut x = h;
                        loop {
                            let line = if side == 0 { &mut cl[x] } else { &mut rl[x] };
                            line.retain(|&e| ec[e] != DEAD); // the original compacts while it walks
                            let line = &*line;
                            let mx = line.iter().rev().fold(0.0f64, |a, &f| if ev[f].abs() <= a { a } else { ev[f].abs() });
                            for &f in line.iter().rev() {
                                let cost = if side == 0 { rcs[er[f]] as f64 * fk } else { ccs[ec[f]] as f64 * fk };
                                let a = ev[f].abs();
                                if (cost < bcost && tol * mx < a) || (cost == bcost && bratio * mx < a) {
                                    best = Some(f);
                                    bcost = cost;
                                    bratio = a / mx;
                                }
                            }
                            ns += 1;
                            if ns < limit && bound < bcost {
                                x = bk.prv[x];
                                if x == h {
                                    break;
                                }
                                continue;
                            }
                            // rotate the bucket head (asm debdb / d8380)
                            let on = best.is_some_and(|f| if side == 0 { ec[f] == x } else { er[f] == x });
                            bk.head[k] = if on { x as i64 } else { bk.prv[x] as i64 };
                            break 'search;
                        }
                        if bcost <= bound {
                            break 'search;
                        }
                    }
                    k += 1;
                }
                let Some(best) = best else { break };
                let (r, c, v) = (er[best], ec[best], ev[best]);
                cb.rem(c, ccs[c] as usize);
                rb.rem(r, rcs[r] as usize);
                ec[best] = DEAD;
                pivot(self, ppos, r, c, v);
                let sk = self.steps.len() - 1;
                let mut u_row = Vec::new();
                for f in take_alive(&mut rl[r], &ec) {
                    let j = ec[f];
                    u_row.push((j, ev[f]));
                    w[j] = ev[f];
                    ec[f] = DEAD;
                    cb.rem(j, ccs[j] as usize);
                    ccs[j] -= 1;
                }
                for f in take_alive(&mut cl[c], &ec) {
                    let i = er[f];
                    let l = ev[f] / v;
                    self.lcol[sk].push((i, l));
                    ec[f] = DEAD;
                    rb.rem(i, rcs[i] as usize);
                    rcs[i] -= 1;
                    rl[i].retain(|&e| ec[e] != DEAD);
                    for t in (0..rl[i].len()).rev() {
                        let g = rl[i][t];
                        let j = ec[g];
                        if w[j] != 0.0 {
                            seen[j] = true;
                            ev[g] -= l * w[j];
                            if ev[g].is_nan() || ev[g].abs() < ZERO {
                                ec[g] = DEAD;
                                rcs[i] -= 1;
                                ccs[j] -= 1;
                            }
                        }
                    }
                    for &(j, u) in &u_row {
                        if !seen[j] {
                            let p = -(l * u);
                            if ZERO <= p.abs() {
                                let e = ev.len();
                                ev.push(p);
                                er.push(i);
                                ec.push(j);
                                rl[i].push(e);
                                cl[j].push(e);
                                rcs[i] += 1;
                                ccs[j] += 1;
                            }
                        }
                        seen[j] = false;
                    }
                    if rcs[i] < 0 { erows.push(i) } else { rb.ins(i, rcs[i] as usize) }
                }
                for &(j, _) in &u_row {
                    w[j] = 0.0;
                    if ccs[j] < 0 { ecols.push(j) } else { cb.ins(j, ccs[j] as usize) }
                }
                rcs[r] = -1;
                ccs[c] = -1;
                self.kernel_u[sk] = u_row;
            }
        }
        if self.steps.len() < m {
            self.singular = ecols.into_iter().zip(erows).collect();
            return false;
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
        // 5b257 L2595–2733 after a factor: t = leading empty U columns, LU+0xe8 = highest empty column above t
        self.prefix = self.ucol.iter().take_while(|c| c.is_empty()).count();
        self.e8 = m.saturating_sub(1);
        while self.e8 > self.prefix && !self.ucol[self.e8].is_empty() {
            self.e8 -= 1;
        }
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
        if self.n_updates > 0 || self.general {
            // Markowitz kernel or FT updates: U is no longer diagonal, the same phase order applies
            return self.btran_updated(x);
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
        self.lt_sparse(&xs)
    }

    /// Sparse Lᵀ of BTRAN (`221173` L262–616) on step-indexed inputs in arrival order: the descending-step heap
    /// when LU+0x140 < 0 (m < 250 among others), else level buckets; output (row, value) in visit order.
    fn lt_sparse(&self, xs: &[(usize, f64)]) -> Vec<(usize, f64)> {
        let m = self.head.len();
        let mut val: Vec<Option<f64>> = vec![None; m]; // touched steps
        for &(k, t) in xs {
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
                for &(k, _) in xs {
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

    /// BTRAN with Forrest–Tomlin updates (m1_notes §8.10): Uᵀ in U order, row etas newest first, then Lᵀ — sparse
    /// (`lt_sparse`, so the output order is the original's) when 0x104·nnz ≤ m, else dense in row order.
    /// shortcut: the etas are textbook FT multipliers, not the original η = Uᵀ⁻¹e_r, so values can differ in the last
    /// bits; the Uᵀ phase is computed densely.
    fn btran_updated(&self, x: &[(usize, f64)]) -> Vec<(usize, f64)> {
        let m = self.head.len();
        let mut w = vec![0.0; m]; // by step
        for &(pos, v) in x {
            w[self.kpos[pos]] += v;
        }
        let mut y = vec![0.0; m]; // by row
        let mut list = Vec::new(); // rows in arrival order
        let mut seen = vec![false; m];
        for &k in &self.order {
            let (r, _, d) = self.steps[k];
            let t = w[k] / d;
            if t.abs() <= ZERO {
                continue;
            }
            y[r] = t;
            seen[r] = true;
            list.push(r);
            for &(j, u) in &self.urow[k] {
                w[j] -= u * t;
            }
        }
        for (r, eta) in self.retas.iter().rev() {
            // 221173 eta phase (newest first): v = w_r; w_r = η_r·v, w_i = η_i·v + w_i
            let v = y[*r];
            if v != 0.0 {
                for &(i, e) in eta {
                    y[i] = if i == *r { e * v } else { e * v + y[i] };
                    if !seen[i] {
                        seen[i] = true;
                        list.push(i);
                    }
                }
            }
        }
        let xs: Vec<(usize, f64)> =
            list.iter().filter(|&&r| y[r].abs() >= ZERO).map(|&r| (self.prow[r] as usize, y[r])).collect();
        if self.sparse_mult * xs.len() <= m {
            return self.lt_sparse(&xs);
        }
        let mut dense = vec![0.0; m]; // by step
        for &(k, v) in &xs {
            dense[k] = v;
        }
        for k in (0..m).rev() {
            let v = dense[k];
            if ZERO < v.abs() {
                for &(j, l) in &self.lrow[k] {
                    dense[j] -= l * v;
                }
            }
        }
        let mut out = vec![(0, 0.0); m];
        for (k, &(r, _, _)) in self.steps.iter().enumerate() {
            out[r] = (r, dense[k]);
        }
        out
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
        for (r, eta) in &self.retas {
            // 2207de R phase: x_r = η·x (8 lanes), dropped at or below ZERO
            let v = dot8(eta.iter().map(|&(i, e)| x[i] * e));
            x[*r] = if v.abs() <= ZERO { 0.0 } else { v };
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
        // Forrest–Tomlin as 5b257 stores it (m1_notes §8.10): r = the leaving index, q its U position, t the prefix
        // LU+0x100, p = max(q, t−1), l = max(p, U positions of the spike entries ≠ r). With l ≠ p and a non-empty U
        // row r the row eta is η = e_rᵀU⁻¹ (old U, r first then U order), the new diagonal η·S and row r leaves U;
        // otherwise the diagonal is S_r and U row r stays. r ends at position l (swap with ord[p], then the ord[l]
        // chain of L1101–1218 and a rotation of [p, l]).
        // shortcut: the dot product never peels an element (pool alignment unknown)
        let kp = self.kpos[pos];
        let rp = self.steps[kp].0;
        let mut at = vec![0; self.order.len()];
        for (i, &k) in self.order.iter().enumerate() {
            at[k] = i;
        }
        let q = at[kp];
        let t = self.prefix;
        let p0 = q.max(t.saturating_sub(1));
        let others: Vec<usize> = (0..spike.len()).filter(|&i| i != rp && spike[i].abs() > ZERO).collect();
        let l = others.iter().map(|&i| at[self.prow[i] as usize]).fold(p0, usize::max);
        // old column kp out of U
        for &(i, _) in &std::mem::take(&mut self.ucol[kp]) {
            let ki = self.prow[i] as usize;
            self.urow[ki].retain(|e| e.0 != kp);
        }
        let mut diag = spike[rp];
        if l != p0 && !self.urow[kp].is_empty() {
            // η = e_rᵀU⁻¹ by Uᵀ in U order (the 21a941 output order) on the U before the update
            let mut w = vec![0.0; self.steps.len()];
            w[kp] = 1.0;
            let mut eta: Vec<(usize, f64)> = Vec::new(); // (row, value)
            for &k in &self.order[q..] {
                let (r, _, d) = self.steps[k];
                let z = w[k] / d;
                if z.abs() <= ZERO {
                    continue;
                }
                eta.push((r, z));
                for &(j, u) in &self.urow[k] {
                    w[j] -= u * z;
                }
            }
            diag = dot8(eta.iter().map(|&(i, e)| spike[i] * e));
            // row r out of U; a column left empty raises the LU+0xe8 cache (L1023, position before the permutation)
            for &(j, _) in &std::mem::take(&mut self.urow[kp]) {
                self.ucol[j].retain(|e| e.0 != rp);
                if self.ucol[j].is_empty() {
                    self.e8 = self.e8.max(at[j]);
                }
            }
            self.retas.push((rp, eta));
        }
        // new prefix (L463–512): local_40, with an empty column above p taking the place of p when q < p
        let (tm1, nz) = (t as i64 - 1, !others.is_empty());
        let (pi, qi, li) = (p0 as i64, q as i64, l as i64);
        let mut p = p0;
        let local40 = if li == pi {
            if qi < pi || pi == tm1 { tm1 - nz as i64 } else { tm1 }
        } else if qi < pi {
            let mut e = self.e8 as i64;
            let mut found = false;
            if pi < e {
                loop {
                    if self.ucol[self.order[e as usize]].is_empty() {
                        found = true;
                        break;
                    }
                    e -= 1;
                    if pi >= e {
                        break;
                    }
                }
            }
            self.e8 = e as usize;
            if found {
                p = e as usize;
                tm1
            } else if pi < li {
                t as i64 - 2
            } else {
                tm1
            }
        } else if pi < li && pi == tm1 {
            t as i64 - 2 + (!nz) as i64
        } else {
            tm1
        };
        // permutation (L1101–1218)
        let swap = |order: &mut Vec<usize>, at: &mut Vec<usize>, a: usize, b: usize| {
            order.swap(a, b);
            at[order[a]] = a;
            at[order[b]] = b;
        };
        if l <= p {
            if q < p {
                swap(&mut self.order, &mut at, q, p);
            }
        } else {
            if q < p {
                swap(&mut self.order, &mut at, q, p);
            }
            let base = p;
            let mut ll = l;
            loop {
                let x = self.order[ll];
                if ll - base <= self.ucol[x].len() {
                    break;
                }
                let m2 = self.ucol[x]
                    .iter()
                    .map(|&(i, _)| at[self.prow[i] as usize])
                    .filter(|&pp| base < pp && pp < ll)
                    .fold(base, usize::max);
                swap(&mut self.order, &mut at, base, ll);
                ll = m2;
            }
            self.order[base..=ll].rotate_left(1);
            for i in base..=ll {
                at[self.order[i]] = i;
            }
        }
        self.prefix = (local40 + 1) as usize;
        for &i in &others {
            self.ucol[kp].push((i, spike[i]));
            self.urow[self.prow[i] as usize].push((kp, spike[i]));
        }
        if others.is_empty() {
            self.e8 = self.e8.max(at[kp]);
        }
        self.steps[kp].2 = diag;
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

pub fn basis_head(m: usize, vstat: &[i32]) -> Vec<usize> {
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

/// Σ of products in the original 8-lane order (5b257 L318–420, 2207de R phase): in sequence below 8 terms, else
/// lanes over the 8-aligned head combined ((0+2)+(4+6)) + ((1+3)+(5+7)), then the tail in sequence.
pub fn dot8(terms: impl Iterator<Item = f64>) -> f64 {
    let v: Vec<f64> = terms.collect();
    if v.len() < 8 {
        return v.iter().fold(0.0, |a, &t| a + t);
    }
    let (head, tail) = v.as_chunks::<8>();
    let mut a = [0.0; 8];
    for c in head {
        for q in 0..8 {
            a[q] += c[q];
        }
    }
    let w = ((a[0] + a[2]) + (a[4] + a[6])) + ((a[1] + a[3]) + (a[5] + a[7]));
    tail.iter().fold(w, |w, t| w + t)
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
