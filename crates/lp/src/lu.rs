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
    /// Column k's entries in rows of tail steps come first, `umid[k]` of them, then the prefix rows (U+0x78 of the
    /// `21686f` view; 05b257 keeps the split as rows cross t).
    umid: Vec<usize>,
    /// U order of the steps (LU+0x48 ord: position → step); identity after a factor, permuted by FT updates.
    order: Vec<usize>,
    /// U position of each step (LU+0x40 pos), the inverse of `order`.
    upos: Vec<usize>,
    /// Prefix of U positions with empty U columns as the original tracks it (LU+0x100).
    prefix: usize,
    /// Cache of the highest U position with an empty column (LU+0xe8).
    e8: usize,
    /// End of the slack phase of the singleton factor (LU+0xdc): prefix positions below it start with empty U rows.
    dc: usize,
    /// LU+0x108 + LU+0x110: mean U row length + 1 of the tail rows and of the rows in [0xdc, t) (`036ade`
    /// C:5725–5740), the density factor of the `09d112` U-stage test.
    dsum: f64,
    /// LU+0x108 alone: `21a941` takes its linear path above 5.
    d108: f64,
    /// Prefix rows with a non-empty U row (LU+0xf0, its length LU+0xec) as steps in list order, and each step's slot
    /// in it (LU+0xf8: ≥ 0 slot, −1 prefix with an empty U row, −2 tail).
    f0: Vec<usize>,
    f8: Vec<i64>,
    /// Forrest–Tomlin row etas in creation order: (row r, [(row i, μ)]) with x_r −= Σ μ·x_i.
    retas: Vec<(usize, Vec<(usize, f64)>)>,
    /// Per row eta: its values start at an odd offset of the LU pool, i.e. 8 but not 16-byte aligned.
    reta_odd: Vec<bool>,
    /// LU pool fill LU+0x160: U rows, U columns and row etas share LU+0x148/0x150 (16-byte aligned base in the kb2
    /// capture). 2·nnz(L+U) after a factor, then grown by `update` (05b257).
    pub pool: usize,
    /// Pool slots of the U rows by step: their lengths after a factor, grown by `update`.
    rcap: Vec<usize>,
    /// Sparse-solve multiplier LU+0x104: a sparse Lᵀ needs `mult·nnz ≤ m` (036ade L5709).
    sparse_mult: usize,
    /// Lᵀ DAG level of each step (LU+0x130) when BTRAN takes the level path (LU+0x140 ≥ 0, 036ade L5757).
    level: Option<Vec<usize>>,
    /// L DAG level of each step (LU+0x118) and the deepest level LU+0x128 (−1 without levels), `036ade` C:5764–5830.
    llev: Option<Vec<usize>>,
    lmax: i64,
    /// Lᵀ DAG depth LU+0x140 (−1 without levels), read by BTRAN's sparse Lᵀ test.
    ltmax: i64,
    /// L entries in the columns of steps ≥ `n_singleton` (LU+0xe4): `21b9de`'s path test reads it.
    ltail: usize,
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
    /// Work inputs of this factorization (`036ade`, docs/m1/m1_02q_work.md).
    pub fw: FactorWork,
}

/// What `036ade` adds to the dual work counter: q = singleton-phase L + U entries ((P0 + local_100) − local_110 after
/// `1da0b2`), nnzB, act = active entries after the singleton phases (at least 1), the Markowitz accumulator and its
/// terms (kind `c`/`r` singleton count, `g` general pivot (ns, Σ column counts along the pivot row, Σ row counts
/// down the pivot column, bound)), and folds of acc > 1e8 before a pivot.
#[derive(Default)]
pub struct FactorWork {
    pub q: usize,
    pub nnz_b: usize,
    pub act: usize,
    pub acc: f64,
    pub early: Vec<f64>,
    pub terms: Vec<(char, [i64; 3], f64)>,
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

    /// Row eta count LU+0x180.
    pub(crate) fn n_etas(&self) -> usize {
        self.retas.len()
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

    /// Row-wise U by basis position as (column position, u) in storage order, and the prefix t (LU+0x100), for
    /// snapshots.
    pub fn u_rows(&self) -> (Vec<Vec<(usize, f64)>>, usize) {
        let rows = self.kpos.iter().map(|&k| self.urow[k].iter().map(|&(k2, u)| (self.steps[k2].1, u)).collect()).collect();
        (rows, self.prefix)
    }

    /// LU+0xe8 and the U positions whose column is empty (snapshots).
    pub fn empty_cols(&self) -> (usize, Vec<usize>) {
        (self.e8, (0..self.order.len()).filter(|&p| self.ucol[self.order[p]].is_empty()).collect())
    }

    /// Tail U columns by descending U position as (position, prefix-row positions in storage order) (snapshots).
    pub fn u_prefix_parts(&self) -> Vec<(usize, Vec<usize>)> {
        let t = self.prefix;
        self.order[t..].iter().rev().map(|&k| (self.steps[k].1, self.prefix_part(k).map(|j| self.steps[j].1).collect())).collect()
    }

    /// Steps of the prefix rows in column k in storage order (`21686f` [+0x78, +0x70)).
    fn prefix_part(&self, k: usize) -> impl Iterator<Item = usize> + '_ {
        self.ucol[k][self.umid[k]..].iter().map(|&(i, _)| self.prow[i] as usize)
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
        // rows of the basic structurals (rows without a basic slack), position order, as one flat array
        let mut rstart = vec![0usize; m + 1];
        for i in 0..m {
            rstart[i + 1] = rstart[i] + if rc[i] <= m { rc[i] } else { 0 };
        }
        let mut rent = vec![(0usize, 0.0f64); rstart[m]];
        let mut fill = rstart.clone();
        for (pos, &j) in head.iter().enumerate() {
            if j < n {
                for (i, a) in spx.col(j) {
                    if rc[i] <= m {
                        rent[fill[i]] = (pos, a);
                        fill[i] += 1;
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
            umid: Vec::new(),
            order: Vec::new(),
            upos: Vec::new(),
            prefix: 0,
            e8: 0,
            dc: 0,
            dsum: 3.0,
            d108: 1.5,
            f0: Vec::new(),
            f8: Vec::new(),
            retas: Vec::new(),
            reta_odd: Vec::new(),
            pool: 0,
            rcap: Vec::new(),
            sparse_mult: 2,
            level: None,
            llev: None,
            lmax: -1,
            ltmax: -1,
            ltail: 0,
            general: false,
            n_updates: 0,
            n_singleton: 0,
            kernel_u: Vec::new(),
            singular: Vec::new(),
            fw: FactorWork::default(),
        };
        // 036ade C:296–330: basis entries, a slack counts 1
        lu.fw.nnz_b = head.iter().map(|&j| if j < n { spx.len[j] as usize } else { 1 }).sum();
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
            let &(pos, d) = rent[rstart[r]..rstart[r + 1]].iter().find(|(p, _)| ppos[*p] == -1).expect("row singleton has a free column");
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
        lu.dc = lu.steps.len();
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
            // active row copy, ascending position, flat
            let mut cstart = vec![0usize; m + 1];
            for c in 0..m {
                if ppos[c] == -1 {
                    for (i, _) in spx.col(head[c]) {
                        if lu.prow[i] < 0 {
                            cc[c] += 1;
                            cstart[i + 1] += 1;
                        }
                    }
                }
            }
            for i in 0..m {
                cstart[i + 1] += cstart[i];
            }
            let mut rcopy = vec![0usize; cstart[m]];
            let mut fill = cstart.clone();
            for c in 0..m {
                if ppos[c] == -1 {
                    for (i, _) in spx.col(head[c]) {
                        if lu.prow[i] < 0 {
                            rcopy[fill[i]] = c;
                            fill[i] += 1;
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
                for &c2 in &rcopy[cstart[r]..cstart[r + 1]] {
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
        // 036ade C:543–585: entries of the unpivoted columns in unpivoted rows (any value), slacks 1, at least 1
        lu.fw.act = (0..m)
            .filter(|&p| ppos[p] < 0)
            .map(|p| if head[p] < n { spx.col(head[p]).filter(|&(i, _)| lu.prow[i] < 0).count() } else { 1 })
            .sum::<usize>()
            .max(1);
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
        // row-wise U in the original's storage order (LU+0xb0/0xd0/0xc8): singleton rows by column position
        // (1da0b2 C:530–560, C:657–677), kernel rows in their list order (036ade C:600–690, C:2366–2393)
        lu.urow = vec![Vec::new(); m];
        for pos in 0..m {
            let k = ppos[pos] as usize;
            for (i, a) in spx.col(head[pos]) {
                let ki = lu.prow[i] as usize;
                if ki < k && ki < ns {
                    lu.urow[ki].push((k, a));
                }
            }
        }
        for (k, ku) in std::mem::take(&mut lu.kernel_u).into_iter().enumerate() {
            let r = lu.steps[k].0;
            for (p2, u) in ku {
                lu.ucol[ppos[p2] as usize].push((r, u));
                lu.urow[k].push((ppos[p2] as usize, u));
            }
        }

        // 1da0b2 emits the singleton steps' L columns and U rows (including entries in kernel columns)
        lu.fw.q = (0..ns).map(|k| lu.lcol[k].len() + lu.urow[k].len()).sum();
        lu.finish();
        lu
    }

    /// `036ade` work stores in order: 0xd4ded, 0xd5300, early acc folds 0xd5e39, 0xd85e6, 0xdc7b2 and with L levels
    /// 0xdcb7b (C:535–541, C:762–764, C:813–822, C:1531–1533, C:5741–5754, C:5934–5936).
    pub fn factor_work(&self) -> Vec<(u32, f64)> {
        use crate::work::X148;
        let nb = self.head.len() as f64;
        let fw = &self.fw;
        let (l, u) = self.factor_nnz();
        let (l, u) = (l - self.head.len(), u - self.head.len());
        let mut v = vec![
            (0xd4ded, ((nb * 5.0 + 100.0) + fw.q as f64 * 3.0) * X148),
            (0xd5300, ((nb * 10.0 + fw.nnz_b as f64 * 3.0) + fw.act as f64 * 7.0) * X148),
        ];
        v.extend(fw.early.iter().map(|&a| (0xd5e39, a * X148)));
        v.push((0xd85e6, fw.acc * X148));
        let big = (l + u) as f64 * 24.0;
        let f = if big <= 2_000_000.0 { 1.0 } else if big < 4_000_000.0 { ((big - 2_000_000.0) * 9.0) / 2_000_000.0 + 1.0 } else { 10.0 };
        v.push((0xdc7b2, ((((nb + nb) + l as f64) + u as f64) * 2.0 * f) * X148));
        if self.level.is_some() {
            v.push((0xdcb7b, ((l + self.head.len()) as f64 * 2.0) * X148));
        }
        v
    }

    /// `05b257` after a refactor (C:2545–2729) with `9e60b` (C:246–251, C:345–347) when the cost mode is on:
    /// the stores the original skips carry their partial sum to the next one.
    pub fn refactor_work(&self, w: &mut crate::work::Work, cost_mode: i32) {
        use crate::work::X148;
        let (m, f1f8) = (self.head.len(), X148);
        let mf = m as f64;
        w.add(0xf48ae, (mf * 3.0) * f1f8);
        if cost_mode >= 1 {
            // shortcut: 05b257 turns the cost mode off instead when env+0xe8 = 4, env+0x2504 = −1 and iterations > 2m
            let unnz: usize = self.urow.iter().map(Vec::len).sum();
            w.add(if cost_mode == 2 { 0x1329e6 } else { 0x1327dc }, unnz as f64 * f1f8);
            w.add(0x1329c1, (f1f8 * 3.0) * mf);
        }
        let a = (mf * 9.0) * X148;
        if self.dc > 0 { w.add(0xf4f9b, a) } else { w.carry(a) }
        let (i, t) = (self.dc, self.prefix);
        let a = X148 * i as f64;
        if i < t { w.add(0xf4ffd, a) } else { w.carry(a) }
        let d = i.max(t);
        let a = (d as f64 + d as f64) * X148;
        if d < m {
            w.add(0xf507a, a);
        } else if t + 1 < m {
            w.add(0xf515b, a);
        } else {
            w.carry(a);
        }
        w.add(0xf51af, m.saturating_sub(self.e8 + 1) as f64 * X148);
    }

    /// Sparse Markowitz kernel of `036ade` (L546–2570, docs/m1/m1_notes.md §6.7) on the rows and positions the singleton
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
        let mut acc = 0.0f64;
        loop {
            if 100000000.0 < acc {
                self.fw.early.push(acc);
                acc = 0.0;
            }
            if cb.head[0] >= 0 {
                // column singleton (L1003–1140): no threshold
                let c = cb.head[0] as usize;
                cb.rem(c, 0);
                let e = first_alive(&cl[c], &ec);
                let r = er[e];
                acc += (rcs[r] as f64 + 1.0) * 10.0;
                self.fw.terms.push(('c', [rcs[r], 0, 0], 0.0));
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
                acc += (ccs[c] as f64 + 1.0) * 10.0;
                self.fw.terms.push(('r', [ccs[c], 0, 0], 0.0));
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
                // asm 0xd7157: ecx = column counts along the pivot row, r10d = row counts down the pivot column
                // (other live entries, counts as in the search); the smaller one enters
                let rsum: i64 = rl[r].iter().filter(|&&f| ec[f] != DEAD).map(|&f| ccs[ec[f]]).sum();
                let csum: i64 = cl[c].iter().filter(|&&f| ec[f] != DEAD).map(|&f| rcs[er[f]]).sum();
                acc = (acc + (ns as f64 * 6.0) * bound) + rsum.min(csum) as f64 * 10.0;
                self.fw.terms.push(('g', [ns as i64, rsum, csum], bound));
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
        self.fw.acc = acc;
        if self.steps.len() < m {
            self.singular = ecols.into_iter().zip(erows).collect();
            return false;
        }
        true
    }

    /// Positions, Lᵀ rows, LU+0x104 and the Lᵀ levels (036ade L5700-5912).
    fn finish(&mut self) {
        let m = self.head.len();
        self.kpos = vec![0; m];
        for (k, &(_, pos, _)) in self.steps.iter().enumerate() {
            self.kpos[pos] = k;
        }
        self.order = (0..m).collect();
        self.upos = (0..m).collect();
        // 5b257 L2595–2733 after a factor: t = leading empty U columns, LU+0xe8 = highest empty column above t
        self.prefix = self.ucol.iter().take_while(|c| c.is_empty()).count();
        self.e8 = m.saturating_sub(1);
        while self.e8 > self.prefix && !self.ucol[self.e8].is_empty() {
            self.e8 -= 1;
        }
        self.general = self.ucol.iter().any(|c| !c.is_empty());
        // U columns after a factor hold their rows by descending U position (so the tail rows lead): all degen2
        // refactors captured in M1-02m. shortcut: there U position = basis position; a model where they differ
        // after a factor (kb2 refactors have no capture at 0 updates) would tell the two orders apart
        let (t, prow) = (self.prefix, &self.prow);
        self.umid = self
            .ucol
            .iter_mut()
            .map(|c| {
                c.sort_by_key(|e| std::cmp::Reverse(prow[e.0]));
                c.iter().filter(|e| prow[e.0] >= t as i64).count()
            })
            .collect();
        // 05b257 C:2595–2690: LU+0xf8 = −1 below 0xdc, the prefix rows [0xdc, t) listed in position order, −2 in the
        // tail; 036ade C:5725–5740: LU+0x108/0x110 (1.5 for an empty range)
        let (t, dc) = (self.prefix, self.dc);
        self.f8 = (0..m).map(|k| if k < dc { -1 } else if k < t { (k - dc) as i64 } else { -2 }).collect();
        self.f0 = (dc..t.max(dc)).collect();
        let mean = |r: std::ops::Range<usize>| {
            let n = r.len();
            if n == 0 { 1.5 } else { r.map(|k| self.urow[k].len()).sum::<usize>() as f64 / n as f64 + 1.0 }
        };
        self.d108 = mean(t..m);
        self.dsum = self.d108 + mean(dc..t.max(dc));
        self.lrow = vec![Vec::new(); m];
        for k in (0..m).rev() {
            for &(i, l) in &self.lcol[k] {
                self.lrow[self.prow[i] as usize].push((k, l));
            }
        }
        let lnnz: usize = self.lrow.iter().map(Vec::len).sum();
        // pool fill after a factor: 2·nnz(L+U) (kb2 capture, 4 factors), every U row slot as long as the row
        self.rcap = self.urow.iter().map(Vec::len).collect();
        self.pool = 2 * (lnnz + self.rcap.iter().sum::<usize>());
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
            self.ltmax = lev.iter().copied().max().unwrap_or(0) as i64;
            self.level = Some(lev);
            // 036ade C:5764–5830: L levels forward over the columns, LU+0x128 = the deepest
            let mut lev = vec![0; m];
            for k in 0..m {
                for &(i, _) in &self.lcol[k] {
                    let j = self.prow[i] as usize;
                    lev[j] = lev[j].max(lev[k] + 1);
                }
            }
            self.lmax = lev.iter().copied().max().unwrap_or(0) as i64;
            self.llev = Some(lev);
        }
        self.ltail = self.lcol[self.n_singleton.min(m)..].iter().map(Vec::len).sum();
    }

    /// BTRAN `221173`: yᵀ = xᵀB⁻¹ for x given as (basis position, value) in input order. Returns (row, value) in the
    /// original's output order: sparse when it stays sparse, else all m rows in row order (output nnz = −1).
    /// Callers sum in this order, so the order is part of the result. shortcut: the Lᵀ linear-scan path (L956) is not
    /// ported, and with U off-diagonals or row etas a plain dense solve stands in for the original's paths.
    pub fn btran(&self, x: &[(usize, f64)]) -> Vec<(usize, f64)> {
        self.btran_dense(x).0
    }

    /// `btran` and whether the output is dense (the original's output nnz < 0), which `28c93` reads (`9e288`).
    pub fn btran_dense(&self, x: &[(usize, f64)]) -> (Vec<(usize, f64)>, bool) {
        self.btran_ut(x, self.ut_dense(x.len()))
    }

    /// BTRAN of e_r as `9e376` hands it (dual ρ_r): a non-empty U row r forces the sparse Uᵀ kernel, an empty one
    /// takes the general test (`221173` C:1196–1206).
    pub fn btran_row(&self, r: usize) -> (Vec<(usize, f64)>, bool) {
        self.btran_ut(&[(r, 1.0)], self.urow_len(r) == 0 && self.ut_dense(1))
    }

    /// Length of the U row of basis position r (LU+0xb8[r] − LU+0xb0[r]).
    pub(crate) fn urow_len(&self, r: usize) -> usize {
        self.urow.get(self.kpos[r]).map_or(0, Vec::len)
    }

    /// `221173` C:88–90: the Uᵀ phase is dense unless (int)(LU+0x108 + LU+0x110)·nnz ≤ m (32-bit product).
    pub(crate) fn ut_dense(&self, nnz: usize) -> bool {
        (self.dsum as i32).wrapping_mul(nnz as i32) > self.head.len() as i32
    }

    /// LU+0x108 + LU+0x110 (for the M1-02k probe).
    pub(crate) fn dsum(&self) -> f64 {
        self.dsum
    }

    /// BTRAN leaves the diagonal-U emulation (Markowitz kernel or FT updates).
    pub(crate) fn updated(&self) -> bool {
        self.n_updates > 0 || self.general
    }

    /// LU+0x108 and the prefix t = LU+0x100 (for the M1-02k probe).
    pub(crate) fn d108(&self) -> f64 {
        self.d108
    }

    /// Lᵀ DAG depth (LU+0x140), −1 without levels (for the M1-02k probe).
    pub(crate) fn lt_depth(&self) -> i64 {
        self.ltmax
    }

    /// `221173` C:263–268: Lᵀ goes sparse when 0x104·nnz ≤ m and LU+0x140 < 0.1m; otherwise the dense Lᵀ (asm
    /// 0x12fee6), which hands an empty input back as nnz 0 (C:1580), the same as the sparse path.
    fn lt_sparse_ok(&self, nnz: usize) -> bool {
        let m = self.head.len();
        self.sparse_mult * nnz <= m && (nnz == 0 || (self.ltmax as f64) < m as f64 * 0.1)
    }

    pub(crate) fn prefix_len(&self) -> usize {
        self.prefix
    }

    /// BTRAN with the Uᵀ path given: dense Uᵀ outputs nnz −1, so the row etas and Lᵀ are dense too.
    pub(crate) fn btran_ut(&self, x: &[(usize, f64)], ut_dense: bool) -> (Vec<(usize, f64)>, bool) {
        assert!(self.complete(), "singular basis");
        if self.updated() {
            // Markowitz kernel or FT updates: U is no longer diagonal, the same phase order applies
            return self.btran_updated(x, ut_dense);
        }
        let m = self.head.len();
        // Uᵀ is the diagonal: sparse (21a941, inputs in order) or dense (C:91–260)
        let sparse_u = !ut_dense;
        let xs = if sparse_u { self.ut_sparse(x) } else { Vec::new() };
        let mut dense = vec![0.0; m]; // by step
        for &(pos, v) in x.iter().filter(|_| !sparse_u) {
            let k = self.kpos[pos];
            let t = v / self.steps[k].2;
            if !(v.abs() <= ZERO || t.abs() <= ZERO) {
                dense[k] = t;
            }
        }
        if !sparse_u || !self.lt_sparse_ok(xs.len()) {
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
            return (y, true);
        }
        (self.lt_sparse(&xs), false)
    }

    /// Sparse Uᵀ kernel `21a941` on (basis position, value) inputs in input order; returns (step, x/d) in output
    /// order. LU+0x108 ≤ 5: an input in the prefix (U position < t) goes out at once and scatters its U row, a tail
    /// input is added to w (`w += x`, whatever its size); the touched positions then leave a min-heap in ascending
    /// U position, scattering `w − u·t`, until (m − top) ≤ 6·heap size switches to a linear scan from top.
    /// LU+0x108 > 5: inputs by LU+0xf8 slot (−1 out at once, ≥ 0 out and scatter, tail added to w when |x/d| > ZERO)
    /// in input order, then a linear scan from the lowest touched tail position. Output keeps |x/d| > ZERO.
    fn ut_sparse(&self, x: &[(usize, f64)]) -> Vec<(usize, f64)> {
        let m = self.head.len();
        let d = |k: usize| self.steps[k].2;
        let mut w = vec![0.0; m]; // by step
        let mut out = Vec::new();
        let linear = |from: usize, w: &mut Vec<f64>, out: &mut Vec<(usize, f64)>| {
            for p in from..m {
                let k = self.order[p];
                let t = w[k] / d(k);
                if ZERO < t.abs() {
                    out.push((k, t));
                    for &(j, u) in &self.urow[k] {
                        w[j] -= u * t; // = −(u·t) + w (21a941), same bits
                    }
                }
                w[k] = 0.0;
            }
        };
        if self.d108 > 5.0 {
            let mut lo = m;
            for &(pos, v) in x {
                let k = self.kpos[pos];
                let t = v / d(k);
                if ZERO < t.abs() {
                    match self.f8[k] {
                        -1 => out.push((k, t)),
                        s if s < 0 => {
                            lo = lo.min(self.upos[k]);
                            w[k] += v;
                        }
                        _ => {
                            out.push((k, t));
                            for &(j, u) in &self.urow[k] {
                                w[j] -= u * t; // = −(u·t) + w (21a941), same bits
                                lo = lo.min(self.upos[j]);
                            }
                        }
                    }
                }
            }
            linear(lo, &mut w, &mut out);
            return out;
        }
        let mut mark = vec![false; m];
        let mut heap = std::collections::BinaryHeap::new(); // Reverse(U position): min-heap
        for &(pos, v) in x {
            let k = self.kpos[pos];
            if self.upos[k] < self.prefix {
                let t = v / d(k);
                if ZERO < t.abs() {
                    out.push((k, t));
                    for &(j, u) in &self.urow[k] {
                        w[j] -= u * t; // = −(u·t) + w (21a941), same bits
                        if !mark[j] {
                            mark[j] = true;
                            heap.push(std::cmp::Reverse(self.upos[j]));
                        }
                    }
                }
            } else {
                w[k] += v;
                if !mark[k] {
                    mark[k] = true;
                    heap.push(std::cmp::Reverse(self.upos[k]));
                }
            }
        }
        while let Some(&std::cmp::Reverse(top)) = heap.peek() {
            if (m - top) as f64 <= heap.len() as f64 * 6.0 {
                linear(top, &mut w, &mut out);
                break;
            }
            heap.pop();
            let k = self.order[top];
            let t = w[k] / d(k);
            if ZERO < t.abs() {
                out.push((k, t));
                for &(j, u) in &self.urow[k] {
                    w[j] -= u * t;
                    if !mark[j] {
                        mark[j] = true;
                        heap.push(std::cmp::Reverse(self.upos[j]));
                    }
                }
            }
            mark[k] = false;
            w[k] = 0.0;
        }
        out
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

    /// BTRAN with Forrest–Tomlin updates (m1_notes §8.10): Uᵀ in U order, row etas newest first, then Lᵀ. With etas
    /// and m < 0x104·nnz(Uᵀ output) the etas are dense (`221173` C:1237–1572: |v| ≤ ZERO clears w_r and skips the
    /// eta) and so is Lᵀ (C:1573–1766); else sparse etas (v = 0.0 skips, |w| ≥ ZERO kept) and a sparse Lᵀ
    /// (`lt_sparse`, the original's output order) when `lt_sparse_ok`, else dense in row order.
    /// A dense Uᵀ (`ut_dense`, C:91–260) zeroes |x| ≤ ZERO as well as |x/d| ≤ ZERO and hands nnz −1 on, so the etas
    /// and Lᵀ are dense; a sparse one is `ut_sparse` (`21a941`).
    fn btran_updated(&self, x: &[(usize, f64)], ut_dense: bool) -> (Vec<(usize, f64)>, bool) {
        let m = self.head.len();
        let mut w = vec![0.0; m]; // by step
        for &(pos, v) in x {
            w[self.kpos[pos]] += v;
        }
        let mut y = vec![0.0; m]; // by row
        let mut list = Vec::new(); // rows in arrival order
        let mut seen = vec![false; m];
        let mut put = |k: usize, t: f64, y: &mut Vec<f64>| {
            let r = self.steps[k].0;
            y[r] = t;
            if !seen[r] {
                seen[r] = true;
                list.push(r);
            }
        };
        if ut_dense {
            for &k in &self.order {
                let t = w[k] / self.steps[k].2;
                if t.abs() <= ZERO || w[k].abs() <= ZERO {
                    continue;
                }
                put(k, t, &mut y);
                for &(j, u) in &self.urow[k] {
                    w[j] -= u * t;
                }
            }
        } else {
            for (k, t) in self.ut_sparse(x) {
                put(k, t, &mut y);
            }
        }
        let dense_eta = !self.retas.is_empty() && (ut_dense || m < self.sparse_mult * list.len());
        for (r, eta) in self.retas.iter().rev() {
            // 221173 eta phase (newest first): v = w_r; w_r = η_r·v, w_i = η_i·v + w_i
            let v = y[*r];
            if dense_eta && v.abs() <= ZERO {
                y[*r] = 0.0;
            } else if v != 0.0 {
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
        if !dense_eta && !ut_dense && self.lt_sparse_ok(xs.len()) {
            return (self.lt_sparse(&xs), false);
        }
        let mut dense = vec![0.0; m]; // by step
        if dense_eta || ut_dense {
            // the dense eta output goes to Lᵀ whole, values below ZERO included
            for (r, &v) in y.iter().enumerate() {
                dense[self.prow[r] as usize] = v;
            }
        } else {
            for &(k, v) in &xs {
                dense[k] = v;
            }
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
        (out, true)
    }

    /// FTRAN through L and the row etas (x by row); the result is the FT spike of the column.
    pub fn ftran_lower(&self, x: &mut [f64]) {
        self.ftran_l(x);
        self.ftran_r(x);
    }

    /// FTRAN through L (x by row).
    pub fn ftran_l(&self, x: &mut [f64]) {
        for (k, &(r, _, _)) in self.steps.iter().enumerate() {
            let xr = x[r];
            if xr.abs() > ZERO {
                for &(i, l) in &self.lcol[k] {
                    x[i] -= l * xr;
                }
            }
        }
    }

    /// FTRAN through the row etas (x by row).
    pub fn ftran_r(&self, x: &mut [f64]) {
        self.ftran_r_list(x, None);
    }

    /// `ftran_r` keeping a sparse side's row list as `09d112` C:1376–1553 (and `21f83a`) does: an eta row that gets
    /// a value above ZERO joins the end of the list, and rows below ZERO leave it at the end.
    fn ftran_r_list(&self, x: &mut [f64], mut list: Option<&mut Vec<usize>>) {
        let mut on = vec![false; if list.is_some() { x.len() } else { 0 }];
        if let Some(l) = list.as_deref() {
            l.iter().for_each(|&i| on[i] = true);
        }
        for ((r, eta), &odd) in self.retas.iter().zip(&self.reta_odd) {
            // 09d112 R phase (C:1136–1355): x_r = η·x in 8 lanes, one element peeled for 8-byte aligned values;
            // dropped at or below ZERO
            let v = dot8_peel(eta.iter().map(|&(i, e)| x[i] * e), odd);
            x[*r] = if v.abs() <= ZERO { 0.0 } else { v };
            if let Some(l) = list.as_deref_mut() {
                if v.abs() > ZERO && !on[*r] {
                    on[*r] = true;
                    l.push(*r);
                }
            }
        }
        if let Some(l) = list {
            if !self.retas.is_empty() {
                l.retain(|&i| x[i].abs() >= ZERO);
            }
        }
    }

    /// L and row etas of one `09d112` right-hand side (`input`: rows and values in input order, None when the side
    /// comes in dense). The L phase is the sparse kernel `21b9de` when 0x104·nnz ≤ m and 0.1m > LU+0x128
    /// (C:659–700), else dense; the row etas keep a sparse side's list while 0x104·nnz ≤ m. Returns the side's rows
    /// in the order the U phase gets them, None for a dense side.
    pub fn ftran_lr(&self, x: &mut [f64], input: Option<&[(usize, f64)]>) -> Option<Vec<usize>> {
        let m = self.head.len();
        let fits = |n: usize| self.sparse_mult * n <= m;
        let mut list = match input.filter(|i| fits(i.len()) && 0.1 * m as f64 > self.lmax as f64) {
            Some(input) => match self.l_sparse(input) {
                Some(out) => {
                    for &(i, _) in input {
                        x[i] = 0.0;
                    }
                    for &(i, v) in &out {
                        x[i] = v;
                    }
                    Some(out.into_iter().map(|(i, _)| i).collect::<Vec<_>>())
                }
                None => {
                    // shortcut: path B keeps the side sparse in the M1-02h order (input rows, then the other nonzeros
                    // of the dense solve in position order); it is taken on maros-r7, pilot, pilot87 and wood1p
                    // (outside the license, no capture). Port C:607–893 when a capture takes it.
                    self.ftran_l(x);
                    let mut seen = vec![false; m];
                    let mut l: Vec<usize> = input.iter().map(|e| e.0).filter(|&i| x[i].abs() > ZERO).collect();
                    input.iter().for_each(|e| seen[e.0] = true);
                    l.extend(self.dense_order(x).into_iter().filter(|&i| !seen[i]));
                    Some(l)
                }
            },
            None => {
                self.ftran_l(x);
                None
            }
        };
        if !self.retas.is_empty() && list.as_ref().is_some_and(|l| !fits(l.len())) {
            list = None;
        }
        self.ftran_r_list(x, list.as_mut());
        list
    }

    /// The sparse L kernel `21b9de` on (row, value) inputs in input order: (row, value) in its output order, values
    /// as it computes them. LU+0x128 < 0 (no levels): Kahn FIFO over the reached steps (C:81–257); else level
    /// buckets from the lowest input level, each in insertion order (C:259–605). Values below ZERO are dropped and
    /// not propagated.
    /// Path B (Kahn below LU+0xe4, then the tail swept in step order, C:607–893) is not ported: None (see
    /// `ftran_lr`); 0 calls on the degen2/kb2/boeing1 captures (M1-02j).
    fn l_sparse(&self, input: &[(usize, f64)]) -> Option<Vec<(usize, f64)>> {
        let m = self.head.len();
        let dm = (m - self.n_singleton.min(m)) as f64;
        if !(dm < 6.0 || dm * (10.0 * dm) <= m as f64 || self.ltail as f64 <= 10.0 * dm) {
            return None;
        }
        let row = |k: usize| self.steps[k].0;
        let mut x = vec![0.0; m]; // by step
        let mut out = Vec::new();
        match &self.llev {
            None => {
                let mut cnt = vec![0u32; m];
                let mut lst: Vec<usize> = input.iter().map(|&(i, _)| self.prow[i] as usize).collect();
                lst.iter().for_each(|&k| cnt[k] += 1);
                let mut p = 0;
                while p < lst.len() {
                    for &(i, _) in &self.lcol[lst[p]] {
                        let j = self.prow[i] as usize;
                        if cnt[j] == 0 {
                            lst.push(j);
                        }
                        cnt[j] += 1;
                    }
                    p += 1;
                }
                let mut q = Vec::with_capacity(lst.len());
                for &(i, v) in input {
                    let k = self.prow[i] as usize;
                    x[k] = v;
                    cnt[k] -= 1;
                    if cnt[k] == 0 {
                        q.push(k);
                    }
                }
                let mut p = 0;
                while p < q.len() {
                    let k = q[p];
                    p += 1;
                    let v = std::mem::take(&mut x[k]);
                    let big = v.abs() >= ZERO;
                    if big {
                        out.push((row(k), v));
                    }
                    for &(i, l) in &self.lcol[k] {
                        let j = self.prow[i] as usize;
                        cnt[j] -= 1;
                        if big {
                            x[j] -= l * v;
                        }
                        if cnt[j] == 0 {
                            q.push(j);
                        }
                    }
                }
            }
            Some(lev) => {
                let mut bk: Vec<Vec<usize>> = vec![Vec::new(); self.lmax as usize + 1];
                let mut mark = vec![false; m];
                let mut lo = bk.len();
                for &(i, v) in input {
                    let k = self.prow[i] as usize;
                    bk[lev[k]].push(k);
                    mark[k] = true;
                    lo = lo.min(lev[k]);
                    x[k] = v;
                }
                for l in lo..bk.len() {
                    let mut t = 0;
                    while t < bk[l].len() {
                        let k = bk[l][t];
                        t += 1;
                        mark[k] = false;
                        let v = std::mem::take(&mut x[k]);
                        if v.abs() >= ZERO {
                            out.push((row(k), v));
                            for &(i, lv) in &self.lcol[k] {
                                let j = self.prow[i] as usize;
                                if !mark[j] {
                                    mark[j] = true;
                                    bk[lev[j]].push(j);
                                }
                                x[j] -= lv * v;
                            }
                        }
                    }
                }
            }
        }
        Some(out)
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

    /// U phase of the DSE τ = B⁻¹ρ as `09d112` computes it (two-RHS path C:2086–2402), x by row after
    /// `ftran_lower`: tail steps (U position ≥ LU+0x100) divide in descending U order and scatter into tail rows only;
    /// each basis position of `need` (the α_q support) in the prefix then gathers its U row in storage order,
    /// (x_i − Σ u·τ_j)/d_i, or x_i/d_i for an empty row, with no drop test. Returns τ by basis position on `need`, 0
    /// elsewhere.
    /// shortcut: the gather rule runs on every pivot; the original also has sparse and kernel-call paths, and the rule
    /// reproduced all of them only on kb2 pivots 0–21 (M1-02a-1). Upgrade when another capture disagrees.
    pub fn ftran_tau(&self, mut x: Vec<f64>, need: &[usize]) -> Vec<f64> {
        let t = self.prefix;
        for &k in self.order[t..].iter().rev() {
            let (r, _, d) = self.steps[k];
            if x[r].abs() <= ZERO || (x[r] / d).abs() <= ZERO {
                x[r] = 0.0;
                continue;
            }
            x[r] /= d;
            let xr = x[r];
            for &(i, u) in &self.ucol[k] {
                if self.upos[self.prow[i] as usize] >= t {
                    x[i] -= u * xr;
                }
            }
        }
        let mut tau = vec![0.0; self.head.len()];
        for &p in need {
            let k = self.kpos[p];
            let (r, _, d) = self.steps[k];
            tau[p] = if self.upos[k] >= t { x[r] } else { gather(x[r], &self.urow[k], |j| x[self.steps[j].0]) / d };
        }
        tau
    }

    /// Bound-flip FTRAN (`2207de`): keep the L/R list to distinguish a sparse U output from a dense one.
    pub fn ftran_flip(&self, x: Vec<f64>, input: Option<&[(usize, f64)]>) -> (Vec<f64>, bool) {
        assert!(self.complete(), "singular basis");
        let m = self.head.len();
        let te = m - self.prefix + self.f0.len();
        let d2 = self.dsum * self.dsum;
        let sparse = input.is_some_and(|i| self.sparse_mult * i.len() <= m && 0.1 * m as f64 > self.lmax as f64)
            && {
                // shortcut: evaluate the L/R list separately to preserve the existing dense arithmetic; M1-02t
                // should share the verified numerical kernel and its representation instead.
                let lr = self.ftran_lr(&mut x.clone(), input);
                lr.is_some_and(|l| te + l.len() < m || d2 * (l.len() as f64) < 2.0 * m as f64)
            };
        let xb = self.ftran_dense(x);
        (xb, !sparse)
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

    /// Take step k out of the LU+0xf0 list (the last entry fills its slot) and set its LU+0xf8 to `mark`.
    fn unlist(&mut self, k: usize, mark: i64) {
        if self.f8[k] >= 0 {
            let (i, last) = (self.f8[k] as usize, self.f0.pop().expect("listed row"));
            if last != k {
                self.f0[i] = last;
                self.f8[last] = i as i64;
            }
        }
        self.f8[k] = mark;
    }

    /// Rows of x (by row) above ZERO in basis position order: the `09d112` dense collection order (C:1744–1786).
    fn dense_order(&self, x: &[f64]) -> Vec<usize> {
        let mut rows: Vec<usize> = (0..x.len()).filter(|&i| x[i].abs() > ZERO).collect();
        rows.sort_by_key(|&i| self.steps[self.prow[i] as usize].1);
        rows
    }

    /// The two-RHS FTRAN `09d112` of the entering column for the dual DSE (C:150–1884) after `ftran_lr` on both
    /// sides: `lr` the α_q side's rows in the order the U phase gets them (None: dense), `spike` that side after L and
    /// the row etas (by row), `nb` the ρ side's count (−1 dense), `aq` α_q by position, `r` the pivot position.
    /// Returns the U kernel ("21686f", "2126cd", "21841f" or "inline"), the α_q output order (positions; M1-02g P3 on
    /// the sparse-U kernels: rows with LU+0xf8 = −1 in `lr` order, tail rows by U position descending, listed prefix
    /// rows in section 2 or LU+0xf0 order (M1-02m); else position order with the pivot last, C:1812–1832) and the
    /// spike order for `update`.
    pub fn ftran2(&self, lr: Option<&[usize]>, spike: &[f64], nb: i64, aq: &[f64], r: usize) -> (&'static str, Vec<usize>, Vec<usize>) {
        let m = self.head.len() as i64;
        let na = lr.map_or(-1, |l| l.len() as i64);
        let te = m - self.prefix as i64 + self.f0.len() as i64;
        let (d2, m2) = (self.dsum * self.dsum, 2.0 * m as f64);
        let sparse_u = |n: i64| n >= 0 && (te + n < m || d2 * (n as f64) < m2);
        let kernel = match (sparse_u(nb), sparse_u(na)) {
            (true, true) => "21686f",
            (true, false) => "2126cd",
            (false, true) => "21841f",
            (false, false) => "inline",
        };
        let sorder = lr.map_or_else(|| self.dense_order(spike), <[usize]>::to_vec);
        let supp: Vec<usize> = (0..aq.len()).filter(|&p| aq[p] != 0.0).collect();
        if !matches!(kernel, "21686f" | "21841f") {
            let mut out: Vec<usize> = supp.into_iter().filter(|&p| p != r).collect();
            out.push(r);
            return (kernel, out, sorder);
        }
        let t = self.prefix;
        let pos = |k: usize| self.steps[k].1;
        let mut out: Vec<usize> =
            sorder.iter().map(|&i| self.prow[i] as usize).filter(|&k| self.upos[k] < t && self.f8[k] < 0 && aq[pos(k)] != 0.0).collect();
        let mut tail: Vec<usize> = supp.iter().map(|&p| self.kpos[p]).filter(|&k| self.upos[k] >= t).collect();
        tail.sort_by_key(|&k| std::cmp::Reverse(self.upos[k]));
        // prefix rows (21686f C:89–507, 21841f C:201–552): in "section 2" order (listed input rows in input order, then
        // rows first reached from the tail outputs' prefix parts) when the input is sparse and those parts are short,
        // else in LU+0xf0 list order
        let lr: Vec<usize> = lr.expect("sparse-U input").iter().map(|&i| self.prow[i] as usize).collect();
        let (np, nt) = (lr.iter().filter(|&&k| self.f8[k] >= 0).count(), lr.iter().filter(|&&k| self.f8[k] == -2).count());
        let sec2 = m as f64 > lr.len() as f64 * d2 && d2 * ((np + nt) as f64) <= te as f64 && {
            let b8 = np + tail.iter().map(|&k| self.ucol[k].len() - self.umid[k]).sum::<usize>();
            b8 as f64 * 2.5 <= self.f0.len() as f64
        };
        let pre: Vec<usize> = if sec2 {
            let mut seen = vec![false; self.steps.len()];
            let first = lr.iter().copied().filter(|&k| self.f8[k] >= 0).chain(tail.iter().flat_map(|&k| self.prefix_part(k)));
            first.filter(|&k| !std::mem::replace(&mut seen[k], true)).collect()
        } else {
            self.f0.clone()
        };
        out.extend(tail);
        out.extend(pre.into_iter().filter(|&k| aq[pos(k)] != 0.0));
        debug_assert_eq!(out.len(), supp.len(), "sparse-U output covers the support");
        (kernel, out.into_iter().map(pos).collect(), sorder)
    }

    /// LU+0xf0 list as basis positions (snapshots).
    pub fn listed(&self) -> Vec<usize> {
        self.f0.iter().map(|&k| self.steps[k].1).collect()
    }

    /// Forrest–Tomlin update (05b257): basis position `pos` takes the column whose spike (L and row etas applied,
    /// by row) is `spike`. The replaced step moves to the end of the U order; its old U row is eliminated into a row
    /// eta. Returns false when the new diagonal is below ZERO (caller refactors). `sorder`: the spike rows in the
    /// order `09d112` hands them on (`ftran2`); empty for the dense collection order (position order).
    /// shortcut: callers outside the dual DSE pass an empty `sorder` (the `2207de` sparse paths are not modelled).
    /// `interval`: the update limit LU+0x188 (LUINTERVAL), which caps the room a moved U row gets.
    /// shortcut: the pool never outgrows its capacity LU+0x168 (05b257 C:800–860 reallocates; kb2 stays far below).
    /// shortcut: the pool growth rules are checked on kb2 (34/34 updates) and maros pivots 0–388 only.
    /// `w`: the work stores of 05b257 (docs/m1/m1_02r_ft_work.md), gathered where Rust computes them and stored at the
    /// end in the original's order. shortcut: of the early exits only the small diagonal is modelled (0xf1beb only).
    pub fn update(&mut self, pos: usize, var: usize, spike: &[f64], sorder: &[usize], interval: usize, w: &mut crate::work::Work) -> bool {
        // Forrest–Tomlin as 5b257 stores it (m1_notes §8.10): r = the leaving index, q its U position, t the prefix
        // LU+0x100, p = max(q, t−1), l = max(p, U positions of the spike entries ≠ r). With l ≠ p and a non-empty U
        // row r the row eta is η = e_rᵀU⁻¹ (old U, r first then U order), the new diagonal η·S and row r leaves U;
        // otherwise the diagonal is S_r and U row r stays. r ends at position l (swap with ord[p], then the ord[l]
        // chain of L1101–1218 and a rotation of [p, l]).
        let kp = self.kpos[pos];
        let off = self.pool; // a new row eta takes the pool end (05b257 C:906–918)
        let rp = self.steps[kp].0;
        let mut at = std::mem::take(&mut self.upos);
        let q = at[kp];
        let t = self.prefix;
        let p0 = q.max(t.saturating_sub(1));
        let others: Vec<usize> = (0..spike.len()).filter(|&i| i != rp && spike[i].abs() > ZERO).collect();
        let l = others.iter().map(|&i| at[self.prow[i] as usize]).fold(p0, usize::max);
        // 05b257 C:445–449: the leaving row's place in the spike list takes its last entry
        let mut sp = if sorder.is_empty() { self.dense_order(spike) } else { sorder.to_vec() };
        if let Some(i) = sp.iter().position(|&i| i == rp) {
            sp.swap_remove(i);
        }
        // work amounts by site group (05b257 C:316–1316): η length (0xf1beb), LU+0xe8 scan (0xf1d02), column r's rows
        // (0xf1f37 when a length-1 row at a position ≤ local_40 leaves the list, else 0xf1f82 = place of r in the row),
        // row r's columns (0xf3f71 / 0xf2969 / 0xf4073 = place of r in the column section, then 0xf2a51), the moved p's
        // row (0xf3eac), the cycle (0xf3ca4, 0xf3d83)
        let (mut w_eta, mut w_e8) = (None, None);
        let mut w_col: Vec<(bool, usize)> = Vec::new();
        let (mut w_row, mut w_urow): (Vec<(u32, usize)>, Option<usize>) = (Vec::new(), None);
        let (mut w_moved, mut w_cyc): (Vec<usize>, Option<(f64, usize)>) = (Vec::new(), None);
        let old_len = self.ucol[kp].len(); // 05b257 C:529 local_150: the new column fits in place up to this
        // old column kp out of U; rows it leaves with an empty U row leave the LU+0xf0 list below (C:535–551)
        let mut emptied = Vec::new();
        for &(i, _) in &std::mem::take(&mut self.ucol[kp]) {
            // 05b257 C:556–567: the row's last entry takes the removed one's place
            let ki = self.prow[i] as usize;
            let at_kp = self.urow[ki].iter().position(|e| e.0 == kp).expect("U row entry");
            w_col.push(if self.urow[ki].len() == 1 { (true, at[ki]) } else { (false, at_kp) });
            self.urow[ki].swap_remove(at_kp);
            if self.urow[ki].is_empty() {
                emptied.push((ki, at[ki]));
            }
        }
        // new prefix (L463–512): local_40, with an empty column above p taking the place of p when q < p. It reads
        // LU+0xe8 and the column ends before row r leaves U below (C:979–1019 raise LU+0xe8 after it)
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
            w_e8 = Some(self.e8 as i64 - e);
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
            // 05b257 C:321–372 reads the LU+0x1d0 copy of η. shortcut: those values are taken as 16-byte aligned (kb2
            // capture, 38/38 pivots), the peel of an 8-byte aligned buffer is not modelled
            diag = dot8(eta.iter().map(|&(i, e)| spike[i] * e));
            w_eta = Some(eta.len());
            w_urow = Some(self.urow[kp].len());
            // row r out of U; a column left empty raises the LU+0xe8 cache (L1023, position before the permutation)
            for &(j, _) in &std::mem::take(&mut self.urow[kp]) {
                // C:979–1019: from a prefix part the column's last entry fills the hole; from a tail part the last
                // tail entry does, and the last prefix entry moves to the front of the prefix part
                let (v, mid) = (&mut self.ucol[j], &mut self.umid[j]);
                let h = v.iter().position(|e| e.0 == rp).expect("U column entry");
                w_row.push(if q < t { (0xf3f71, h - *mid) } else { (0xf2969, h) });
                if q < t {
                    v.swap_remove(h);
                } else {
                    *mid -= 1;
                    v[h] = v[*mid];
                    let last = v.pop().expect("U column entry");
                    if *mid < v.len() {
                        v[*mid] = last;
                    }
                }
                if self.ucol[j].is_empty() {
                    self.e8 = self.e8.max(at[j]);
                }
            }
            self.pool += eta.len();
            self.reta_odd.push(off % 2 == 1);
            self.retas.push((rp, eta));
        } else if !others.is_empty() && q < t {
            // C:871–899 (no row eta, r leaves the prefix): r swaps with the first prefix entry of its columns. The
            // original stores work here only with l = p (local_80 = 3); with l ≠ p U row r is empty
            if l == p0 {
                w_urow = Some(self.urow[kp].len());
            }
            for &(j, _) in &self.urow[kp] {
                let mid = self.umid[j];
                let h = mid + self.ucol[j][mid..].iter().position(|e| e.0 == rp).expect("U column entry");
                w_row.push((0xf4073, h - mid));
                self.ucol[j].swap(h, mid);
                self.umid[j] += 1;
            }
        }
        // LU+0xf0 list (05b257 C:535–551, C:1040–1059, C:1085–1097)
        for (k, at_k) in emptied {
            if at_k as i64 <= local40 {
                self.unlist(k, -1);
            }
        }
        if p != p0 {
            // the empty column at p takes the place of p0: its row enters the prefix
            let k = self.order[p];
            // C:1040–1069: its entries swap with the last tail entry of their columns
            let rk = self.steps[k].0;
            for &(j, _) in &self.urow[k] {
                let h = self.ucol[j][..self.umid[j]].iter().position(|e| e.0 == rk).expect("U column entry");
                w_moved.push(h);
                self.umid[j] -= 1;
                self.ucol[j].swap(h, self.umid[j]);
            }
            if self.urow[k].is_empty() {
                self.f8[k] = -1;
            } else {
                self.f8[k] = self.f0.len() as i64;
                self.f0.push(k);
            }
        }
        if l != p || p + 1 != t || nz {
            self.unlist(kp, -2);
        }
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
            let mut acc = 0.0;
            loop {
                let x = self.order[ll];
                if ll - base <= self.ucol[x].len() {
                    break;
                }
                acc = (acc + self.ucol[x].len() as f64 * 2.0) + 5.0;
                let m2 = self.ucol[x]
                    .iter()
                    .map(|&(i, _)| at[self.prow[i] as usize])
                    .filter(|&pp| base < pp && pp < ll)
                    .fold(base, usize::max);
                swap(&mut self.order, &mut at, base, ll);
                ll = m2;
            }
            w_cyc = Some((acc, ll - base));
            self.order[base..=ll].rotate_left(1);
            for i in base..=ll {
                at[self.order[i]] = i;
            }
        }
        self.prefix = (local40 + 1) as usize;
        self.upos = at;
        // 05b257 C:1240–1262: spike rows that land in the prefix join the list in spike order
        for &i in &sp {
            let k = self.prow[i] as usize;
            if self.upos[k] as i64 <= local40 && self.f8[k] < 0 {
                self.f8[k] = self.f0.len() as i64;
                self.f0.push(k);
            }
        }
        // pool growth: a new column longer than the old one goes to the end (C:1224–1227), a full U row that gets
        // an entry moves to the end with min(max(4, len/2), LU+0x188 − LU+0x18c) free places (C:1264–1282)
        if old_len < others.len() {
            self.pool += others.len();
        }
        let room = interval.saturating_sub(self.n_updates);
        for &i in &others {
            let k = self.prow[i] as usize;
            let n = self.urow[k].len();
            if n == self.rcap[k] {
                let extra = (n / 2).max(4).min(room);
                self.pool += n + extra;
                self.rcap[k] = n + extra;
            }
            self.urow[k].push((kp, spike[i]));
        }
        // C:1228–1260: the new column takes the spike rows in list order, tail rows from the front and prefix rows
        // from the back
        let (tl, pre): (Vec<usize>, Vec<usize>) =
            sp.iter().filter(|&&i| spike[i].abs() > ZERO).partition(|&&i| self.upos[self.prow[i] as usize] as i64 > local40);
        debug_assert_eq!(tl.len() + pre.len(), others.len(), "spike list = spike support");
        self.umid[kp] = tl.len();
        self.ucol[kp] = tl.iter().chain(pre.iter().rev()).map(|&i| (i, spike[i])).collect();
        if others.is_empty() {
            self.e8 = self.e8.max(self.upos[kp]);
        }
        // work stores in the original's order (05b257 C:417 … C:1316, docs/m1/m1_02r_ft_work.md §2)
        use crate::work::X148;
        if let Some(n) = w_eta.filter(|&n| n > 0) {
            w.add(0xf1beb, (X148 * 3.0) * n as f64);
        }
        if diag.abs() <= ZERO || diag.is_nan() {
            return false; // C:445–455: 05b257 leaves before any other store and the caller refactors
        }
        if let Some(d) = w_e8 {
            w.add(0xf1d02, (d as f64 * 3.0) * X148);
        }
        for (len1, v) in w_col {
            if !len1 {
                w.add(0xf1f82, v as f64 * X148);
            } else if v as i64 <= local40 {
                w.add(0xf1f37, X148 * 3.0);
            }
        }
        let ns = sp.len();
        let a = (old_len as f64 * 5.0) * X148;
        if old_len < ns {
            w.add(0xf2044, a);
        } else if ns > 0 {
            w.add(0xf207a, a);
        } else {
            w.carry(a);
        }
        w.add(0xf2521, (ns as f64 * 3.0) * X148);
        for (site, v) in w_row {
            w.add(site, v as f64 * X148);
        }
        if let Some(n) = w_urow {
            w.add(0xf2a51, (n as f64 * 3.0) * X148);
        }
        for v in w_moved {
            w.add(0xf3eac, v as f64 * X148);
        }
        if let Some((acc, k)) = w_cyc {
            w.add(0xf3ca4, acc * X148);
            w.add(0xf3d83, (k as f64 * 2.0) * X148);
        }
        w.add(0xf2e3c, (ns as f64 * 5.0) * X148);
        self.steps[kp].2 = diag;
        self.head[pos] = var;
        self.general = true;
        self.n_updates += 1;
        // the LU+0xf0 list stays in step with U: listed rows in the prefix at their slot, −2 exactly in the tail, −1
        // only on empty U rows (all three hold in the kb2 capture)
        debug_assert!(self.f0.iter().enumerate().all(|(i, &k)| self.f8[k] == i as i64 && self.upos[k] < self.prefix));
        debug_assert!((0..self.f8.len()).all(|k| (self.f8[k] == -2) == (self.upos[k] >= self.prefix)));
        debug_assert!((0..self.f8.len()).all(|k| self.f8[k] != -1 || self.urow[k].is_empty()));
        // and every column is [tail rows | prefix rows]
        debug_assert!((0..self.ucol.len()).all(|k| self.ucol[k]
            .iter()
            .enumerate()
            .all(|(h, e)| (h < self.umid[k]) == (self.upos[self.prow[e.0] as usize] >= self.prefix))));
        true
    }
}

/// `09d112` gather v − Σ u·x_j over a U row in storage order (asm 0x12925f–0x129465): in sequence below 8 terms;
/// else lane 0 starts at v, eight lanes subtract and combine as ((l0+l2)+(l4+l6)) + ((l1+l3)+(l5+l7)), the tail in
/// sequence.
/// shortcut: no one-element peel when the row's value array is 8- but not 16-byte aligned (the original's row storage
/// offsets are not modelled; kb2 has no 8+ term row).
fn gather(v: f64, row: &[(usize, f64)], x: impl Fn(usize) -> f64) -> f64 {
    if row.len() < 8 {
        return row.iter().fold(v, |a, &(j, u)| a - u * x(j));
    }
    let (chunks, rest) = row.as_chunks::<8>();
    let mut a = [v, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
    for c in chunks {
        for (q, &(j, u)) in c.iter().enumerate() {
            a[q] -= u * x(j);
        }
    }
    let w = ((a[0] + a[2]) + (a[4] + a[6])) + ((a[1] + a[3]) + (a[5] + a[7]));
    rest.iter().fold(w, |a, &(j, u)| a - u * x(j))
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
    dot8_peel(terms, false)
}

/// `dot8` over an array that may be 8 but not 16-byte aligned (`09d112` C:1149–1260, asm 0x1272d7): with `peel` the
/// first term goes to lane 0 first, and below 9 terms the whole sum is sequential.
fn dot8_peel(terms: impl Iterator<Item = f64>, peel: bool) -> f64 {
    let v: Vec<f64> = terms.collect();
    let p = peel as usize;
    if v.len() < 8 + p {
        return v.iter().fold(0.0, |a, &t| a + t);
    }
    let (head, tail) = v[p..].as_chunks::<8>();
    let mut a = [v[..p].iter().fold(0.0, |a, &t| a + t), 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
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
    use super::{dot8_peel, gather, sum_squares, Lu};
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
            vtype: vec![b'C'; n],
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
            assert!(lu.update(pos, j, &spike, &[], usize::MAX, &mut crate::work::Work::new()));
            check(&spx, &lu);
            done += 1;
        }
        assert!(done >= 4);
    }

    #[test]
    fn dot8_peel_order() {
        // 8 terms: lanes without a peel, in sequence with one (kb2 pivot 30, row 17)
        let t = [1e16, 1.0, 1.0, 1.0, -1e16, 1.0, 1.0, 1.0];
        assert_eq!(dot8_peel(t.into_iter(), false), ((1e16 + 1.0) + (-1e16 + 1.0)) + ((1.0 + 1.0) + (1.0 + 1.0)));
        assert_eq!(dot8_peel(t.into_iter(), true), t.iter().fold(0.0, |a, &x| a + x));
        // 9 terms with a peel: the first starts lane 0, the other 8 fill the lanes
        let t9 = [1.0, 1e16, 1.0, 1.0, 1.0, -1e16, 1.0, 1.0, 1.0];
        assert_eq!(dot8_peel(t9.into_iter(), true), ((1.0 + 1e16 + 1.0) + (-1e16 + 1.0)) + ((1.0 + 1.0) + (1.0 + 1.0)));
    }

    #[test]
    fn sum_squares_lane_order() {
        // 9 values: lanes take 0..8, the 9th is added last; (1e16 + 1) + ... shows the grouping
        let v = [1e8, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0];
        let lanes = ((1e16 + 1.0) + (1.0 + 1.0)) + ((1.0 + 1.0) + (1.0 + 1.0));
        assert_eq!(sum_squares(&v), lanes + 1.0);
        assert_eq!(sum_squares(&v[..7]), v[..7].iter().fold(0.0, |w, t| w + t * t));
    }

    #[test]
    fn gather_lane_order() {
        // 9 terms u·1: v and terms 0..8 in lanes (v seeds lane 0), the 9th subtracted last; 1e16 shows the grouping
        let row: Vec<(usize, f64)> = (0..9).map(|_| (0, 1.0)).collect();
        let lanes = (((1e16 - 1.0) + -1.0) + (-1.0 + -1.0)) + ((-1.0 + -1.0) + (-1.0 + -1.0));
        assert_eq!(gather(1e16, &row, |_| 1.0), lanes - 1.0);
        assert_ne!(lanes - 1.0, (0..9).fold(1e16, |a, _| a - 1.0));
        assert_eq!(gather(1e16, &row[..7], |_| 1.0), (0..7).fold(1e16, |a, _| a - 1.0));
    }
}
