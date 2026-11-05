//! Inner-loop row reductions of `3cd406` (docs/m2_red_inner.md §3–6, §8): `36ddf2` row cleanup, `37168e` activity
//! bound tightening, `36c5cf` singleton rows, `56f793` coefficient strengthening, `36e6b6` node GC, and the
//! round's inline implied-bound block (m2_spine_b.md §5.3). Comparisons keep the original's form and operand
//! order (bit-exact results and work).
use crate::helpers::{maxsd, minsd};
use crate::state::{guarded_add, State, INF, NIL};
use crate::Params;

impl State {
    /// `36ddf2`: over `538f30`'s rows, empty rows go (-1; infeasible if the rhs is violated), rows that can never
    /// be violated go (-2, their columns marked), and in the others coefficients too small to matter
    /// (`cnt·(u−l)·|a| < 0.01·ft`, |a| < 0.001) move to the rhs at the column's lower bound; those rows get their
    /// activities recomputed. Returns 0 or 3 (`bad_row`).
    pub fn row_clean(&mut self, p: &Params) -> i32 {
        let ft = p.feastol;
        let small = 0.001; // MIP state (+0xfc = 1); OptimalityTol·0.1 belongs to LP-like states
        let rows = self.rows_list();
        let (mut removed, mut changed) = (0, Vec::new());
        for &ii in &rows {
            let i = ii as usize;
            let cnt = self.row_cnt[i];
            if cnt == 0 {
                let b = self.rhs[i];
                if b < -ft || (ft < b && self.sense[i] == b'=') {
                    self.bad_row = ii;
                    return 3;
                }
                self.row_cnt[i] = -1;
                removed += 1;
            } else if cnt > 0 {
                let keep = self.inf_max[i] != 0
                    || ft <= self.act_max[i]
                    || (self.sense[i] == b'=' && (self.inf_min[i] != 0 || self.act_min[i] <= -ft));
                if keep {
                    let mut k = 0;
                    let mut q = self.row_head[i];
                    while q != NIL {
                        let at = q as usize;
                        let nd = self.nd[at];
                        q = nd.nr;
                        if nd.col < 0 {
                            continue;
                        }
                        let j = nd.col as usize;
                        let aa = nd.val.abs();
                        if (self.row_cnt[i] as f64) * (self.ub[j] - self.lb[j]) * aa < ft * 0.01 && aa < small {
                            self.tracker.mark_col(nd.col);
                            k += 1;
                            self.row_cnt[i] -= 1;
                            self.col_cnt[j] -= 1;
                            self.rhs[i] -= nd.val * self.lb[j];
                            self.nd[at].col = -1;
                        }
                    }
                    self.work += (self.row_cnt[i] as f64 * 6.0) * self.work_mult;
                    if k > 0 {
                        self.tracker.mark_row(ii);
                        changed.push(ii);
                        self.coefs_changed += k;
                    }
                } else {
                    removed += 1;
                    self.mark_cols_of_row(i);
                    self.kill_row_nodes(i);
                    self.work += (self.row_cnt[i] as f64 * 3.0) * self.work_mult;
                    self.row_cnt[i] = -2;
                }
            }
        }
        self.work += (rows.len() as f64 * 9.0) * self.work_mult;
        self.rows_removed += removed;
        if !changed.is_empty() {
            self.activity(Some(&changed));
        }
        0
    }

    /// Kill row `i`'s live nodes (their columns lose one entry); the row's own count is the caller's.
    fn kill_row_nodes(&mut self, i: usize) {
        let mut q = self.row_head[i];
        while q != NIL {
            let at = q as usize;
            q = self.nd[at].nr;
            let c = self.nd[at].col;
            if c >= 0 {
                self.col_cnt[c as usize] -= 1;
                self.nd[at].col = -1;
            }
        }
    }

    /// `37168e`: over `538f30`'s rows, each column's bounds implied by the row's activities (both directions of an
    /// `=` row), rounded for integer columns, become the column's bounds when they tighten enough (`3dd6f1`
    /// updates the activities as it goes). Returns 0 or 3 (`bad_row` / `bad_col`).
    pub fn bound_tighten(&mut self, p: &Params) -> i32 {
        let (ft, nf, ift) = (p.feastol, p.numeric_focus, p.intfeastol);
        let rows = self.rows_list();
        let mut tight = 0;
        for &ii in &rows {
            let i = ii as usize;
            let cnt = self.row_cnt[i];
            if cnt <= 0 {
                continue;
            }
            if (self.inf_min[i] == 0 && ft < self.act_min[i])
                || (self.sense[i] == b'=' && self.inf_max[i] == 0 && self.act_max[i] < -ft)
            {
                self.bad_row = ii;
                return 3;
            }
            if nf >= 1 && cnt == 1 {
                match self.forcing_singleton(i, ft) {
                    Some(0) => continue,
                    Some(r) => return r,
                    None => {}
                }
            }
            let mut r = self.act_max[i] - self.act_min[i];
            r = if r > 1.0 { r } else { 1.0 };
            r = if 1e6 < r { 1e6 } else { r };
            if nf == 1 {
                r *= 10.0;
            } else if nf >= 2 {
                r *= 100.0;
            }
            let thr = r * 1e-10;
            let mut q = self.row_head[i];
            while q != NIL {
                let nd = self.nd[q as usize];
                q = nd.nr;
                if nd.col < 0 {
                    continue;
                }
                let j = nd.col as usize;
                let (a, aa) = (nd.val, nd.val.abs());
                if thr > aa {
                    continue;
                }
                let (ub, lb) = (self.ub[j], self.lb[j]);
                let (am, im) = (self.act_min[i], self.inf_min[i]);
                let (mut l, mut u) = (lb, ub);
                let neg0 = |x: f64| if 0.0 <= x { 0.0 } else { x };
                if a > 1e-8 {
                    if im == 0 {
                        u = lb - neg0(am) / a;
                    } else if im == 1 && lb <= -INF {
                        u = -am / a;
                    }
                } else if a < -1e-8 {
                    if im == 0 {
                        l = ub - neg0(am) / a;
                    } else if im == 1 && INF <= ub {
                        l = -am / a;
                    }
                }
                if self.sense[i] == b'=' {
                    // the row negated: coefficient -a, min activity - rhs = -act_max
                    let (a2, ax, ix) = (-a, self.act_max[i], self.inf_max[i]);
                    let (mut l2, mut u2) = (lb, ub);
                    if a2 > 1e-8 {
                        if ix == 0 {
                            u2 = lb - neg0(-ax) / a2;
                        } else if ix == 1 && lb <= -INF {
                            u2 = ax / a2;
                        }
                    } else if a2 < -1e-8 {
                        if ix == 0 {
                            l2 = ub - neg0(-ax) / a2;
                        } else if ix == 1 && INF <= ub {
                            l2 = ax / a2;
                        }
                    }
                    l = if l2 <= l { l } else { l2 };
                    u = if u <= u2 { u } else { u2 };
                }
                if self.vtype[j] == b'C' {
                    if nf > 1 {
                        continue;
                    }
                    if l <= ft + ub {
                        l = minsd(ub, l);
                    }
                    if lb - ft <= u {
                        u = maxsd(lb, u);
                    }
                    if nf == 1 {
                        if l != u && u - l < ft * 100.0 {
                            continue;
                        }
                    } else if p.prebound < 2 && l != u && u - l < ft * 10.0 {
                        // nearly fixed continuous column: round outward (usually rejected below)
                        l = (l + 1e-13).floor();
                        u = (u - 1e-13).ceil();
                    }
                } else {
                    let (mut lc, mut uf) = ((l - ift).ceil(), (ift + u).floor());
                    let (fl, cu) = (l.floor(), u.ceil());
                    if fl < lc && !(0.1 * ft < (l - fl) * aa) {
                        lc = fl;
                    }
                    if uf < cu && !(ft * 0.1 < aa * (cu - u)) {
                        uf = cu;
                    }
                    l = lc;
                    u = uf;
                }
                let tol = if self.col_flag[j] & 0x18 != 0 { ft * 10.0 } else { ft * 1000.0 };
                let lb_ok = 1e12 > l.abs() && l > -1e8 && l > lb && (1e-13 >= u - l || l > lb + tol);
                let ub_pre = 1e12 > (-u).abs() && -u > -1e8 && -u > -ub;
                if lb_ok {
                    tight += 1;
                } else {
                    l = lb;
                    if !ub_pre {
                        continue;
                    }
                }
                if !ub_pre {
                    u = ub;
                } else if 1e-13 >= u - l || -u > tol - ub {
                    tight += 1;
                } else {
                    u = ub;
                    if !lb_ok {
                        continue;
                    }
                }
                self.bound_change(j, lb, ub, l, u);
                self.lb[j] = l;
                self.ub[j] = u;
                self.tracker.mark_col(j as i32);
                if u + ft < l {
                    self.bad_col = j as i32;
                    return 3;
                }
            }
            self.work += (self.row_cnt[i] as f64 * 11.0) * self.work_mult;
        }
        self.work += (rows.len() as f64 * 8.0) * self.work_mult;
        self.bounds_tightened += tight;
        0
    }

    /// `37168e`'s forcing branch (NumericFocus >= 1, one live entry): when the row's min activity (or, for `=`,
    /// its max activity) sits at the rhs and no coefficient is below 1000·ft, the column goes to the bound giving
    /// that activity (`3de18e` with a record) and the row is dropped (-2). Some(status) when it acted, None to
    /// go on with the plain bound tightening.
    /// shortcut: not checked against Gurobi (NumericFocus 0 by default).
    fn forcing_singleton(&mut self, i: usize, ft: f64) -> Option<i32> {
        let no_small = |st: &mut State| {
            let mut q = st.row_head[i];
            let mut walked = 0;
            let mut found = false;
            while q != NIL {
                let nd = st.nd[q as usize];
                if nd.col >= 0 && nd.val.abs() < ft * 1000.0 {
                    found = true;
                    break;
                }
                walked += 1;
                q = nd.nr;
            }
            st.work += (walked as f64 * 5.0) * st.work_mult;
            !found
        };
        let am = self.act_min[i];
        let mut s = 0.0;
        if self.inf_min[i] == 0 && !(am <= -1e-10 || ft / 100.0 <= am) {
            if no_small(self) {
                s = 1.0;
            }
        } else if self.sense[i] == b'=' && self.inf_max[i] == 0 {
            let ax = self.act_max[i];
            if ax < 1e-10 && -ft / 100.0 < ax && no_small(self) {
                s = -1.0;
            }
        }
        if s == 0.0 {
            return None;
        }
        let mut q = self.row_head[i];
        while q != NIL {
            let at = q as usize;
            let nd = self.nd[at];
            q = nd.nr;
            if nd.col < 0 {
                continue;
            }
            let j = nd.col as usize;
            self.nd[at].col = -1;
            self.col_cnt[j] -= 1;
            let v = if s * nd.val <= 0.0 { self.ub[j] } else { self.lb[j] };
            let r = self.fix_col(j, v, true);
            if r != 0 {
                return Some(r);
            }
        }
        self.work += (self.row_cnt[i] as f64 * 5.0) * self.work_mult;
        self.row_cnt[i] = -2;
        self.rows_removed += 1;
        Some(0)
    }

    /// `36c5cf(st, env, -1, NULL, &gc)`: rows with one live entry among the tracker level's raw rows become bounds
    /// on their column (rounded for integer columns; an `=` row fixes it via `3de18e`), or prove infeasibility; the
    /// row goes (-1). `gc` gets the number of entries this removed. Returns 0, 3 or `3de18e`'s status.
    pub fn singleton_rows(&mut self, p: &Params, gc: &mut i64) -> i32 {
        let (ft, ift) = (p.feastol, p.intfeastol);
        let lv = self.tracker.level;
        let rows = self.tracker.rows.list[lv][..self.tracker.rows.a[lv]].to_vec();
        let (mut nnz, mut nrem) = (0i64, 0);
        let rc = 'rows: {
            for &ii in &rows {
                let i = ii as usize;
                if self.row_cnt[i] != 1 {
                    continue;
                }
                let mut q = self.row_head[i];
                let mut skipped = 0;
                while self.nd[q as usize].col < 0 {
                    q = self.nd[q as usize].nr;
                    skipped += 1;
                }
                self.work += (skipped as f64 + skipped as f64) * self.work_mult;
                let at = q as usize;
                let (a, j) = (self.nd[at].val, self.nd[at].col as usize);
                let aa = a.abs();
                if !(1e-8 <= aa) {
                    continue;
                }
                let b = self.rhs[i];
                let mut x = b / a;
                let kind = if self.sense[i] == b'=' {
                    3
                } else if a <= 0.0 {
                    (self.lb[j] < x) as i32
                } else if x < self.ub[j] {
                    2
                } else {
                    0
                };
                let mut conflict = false;
                if kind & 1 != 0 && !(x <= self.ub[j]) {
                    let ub = self.ub[j];
                    let t = if a <= 0.0 { ft + b } else { b - ft };
                    x = if t / a <= ub { ub } else { t / a };
                    if x <= ft + ub {
                        if ft + b < ub * a {
                            continue;
                        }
                    } else {
                        conflict = true;
                    }
                }
                if !conflict && kind & 2 != 0 && x < self.lb[j] {
                    let lb = self.lb[j];
                    let t = if a <= 0.0 { b - ft } else { ft + b };
                    x = if lb <= t / a { lb } else { t / a };
                    if x < lb - ft {
                        conflict = true;
                    } else if lb * a < b - ft {
                        continue;
                    }
                }
                if conflict {
                    if ft <= self.act_min[i] || (!(-ft < self.act_max[i]) && self.sense[i] == b'=') {
                        self.bad_row = ii;
                        self.bad_col = j as i32;
                        break 'rows 3;
                    }
                    continue;
                }
                self.nd[at].col = -1;
                if kind != 0 {
                    let (lb, ub) = (self.lb[j], self.ub[j]);
                    let (l, u);
                    if self.vtype[j] == b'C' {
                        l = if kind & 1 != 0 { if ub <= x { ub } else { x } } else { lb };
                        u = if kind & 2 != 0 { if x <= l { l } else { x } } else { ub };
                    } else {
                        let l0 = if kind & 1 != 0 { x } else { lb };
                        let u0 = if kind & 2 != 0 { x } else { ub };
                        let (mut lc, mut uf) = ((l0 - ift).ceil(), (ift + u0).floor());
                        let (fl, cu) = (l0.floor(), u0.ceil());
                        if fl < lc && !(ft * 0.1 < (l0 - fl) * aa) {
                            lc = fl;
                        }
                        if uf < cu && !(ft * 0.1 < aa * (cu - u0)) {
                            uf = cu;
                        }
                        if uf < lc {
                            self.bad_row = ii;
                            self.bad_col = j as i32;
                            break 'rows 3;
                        }
                        if x != x.floor() && lc <= lb && ub <= uf {
                            // rounding does not tighten: keep the row
                            self.nd[at].col = j as i32;
                            continue;
                        }
                        if kind == 3 {
                            x = lc;
                        }
                        (l, u) = (lc, uf);
                    }
                    self.bound_change(j, lb, ub, l, u);
                    self.lb[j] = l;
                    self.ub[j] = u;
                }
                self.tracker.mark_col(j as i32);
                self.row_cnt[i] = -1;
                nnz += 1;
                self.col_cnt[j] -= 1;
                if kind == 3 {
                    nnz += self.col_cnt[j] as i64;
                    let r = self.fix_col(j, x, true);
                    if r != 0 {
                        break 'rows r;
                    }
                }
                nrem += 1;
            }
            self.work += (rows.len() as f64 * 11.0) * self.work_mult;
            self.rows_removed += nrem;
            0
        };
        *gc = nnz;
        rc
    }

    /// `56f793`: in `<` rows with a finite max activity above the rhs (d = max activity − rhs >= ft), an integer
    /// column's coefficient larger than d in magnitude (by more than ft) becomes ±d, the rhs moving by the
    /// difference at the column's bound; min activity follows (`370047` when a coefficient was beyond 1e10).
    /// Returns the number of coefficients changed (ignored by 3cd406).
    pub fn coef_strengthen(&mut self, p: &Params) -> i32 {
        let ft = p.feastol;
        let rows = self.rows_list();
        let mut total = 0;
        for &ii in &rows {
            let i = ii as usize;
            if !(self.row_cnt[i] > 0 && self.sense[i] != b'=' && self.inf_max[i] == 0 && ft <= self.act_max[i]) {
                continue;
            }
            let (mut k, mut drhs, mut dmin, mut big) = (0, 0.0, 0.0, false);
            let mut q = self.row_head[i];
            while q != NIL {
                let at = q as usize;
                let nd = self.nd[at];
                q = nd.nr;
                if nd.col < 0 || self.vtype[nd.col as usize] == b'C' {
                    continue;
                }
                let j = nd.col as usize;
                if !(0.5 <= self.ub[j] - self.lb[j]) {
                    continue;
                }
                let (a, d) = (nd.val, self.act_max[i]);
                if a <= 0.0 {
                    let e = -(a + d);
                    if !(ft < e && (-1e10 <= a || self.lb[j] == 0.0)) {
                        continue;
                    }
                    big = big || a < -1e10;
                    self.nd[at].val = -d;
                    let (l, u) = (self.lb[j], self.ub[j]);
                    drhs += e * l;
                    if INF <= u {
                        dmin = guarded_add(dmin, a * l, false);
                        dmin = guarded_add(dmin, l * self.act_max[i], false);
                    } else {
                        let w = u - l;
                        dmin = guarded_add(dmin, -(a * w), false);
                        dmin = guarded_add(dmin, -(w * self.act_max[i]), false);
                    }
                } else {
                    let e = a - d;
                    if !(ft < e && (a <= 1e10 || self.ub[j] == 0.0)) {
                        continue;
                    }
                    big = big || 1e10 < a;
                    self.nd[at].val = d;
                    let (u, l) = (self.ub[j], self.lb[j]);
                    drhs -= e * u;
                    if l <= -INF {
                        dmin = guarded_add(dmin, a * u, false);
                        dmin = guarded_add(dmin, -(u * self.act_max[i]), false);
                    } else {
                        let w = u - l;
                        dmin = guarded_add(dmin, a * w, false);
                        dmin = guarded_add(dmin, -(w * self.act_max[i]), false);
                    }
                }
                self.tracker.mark_col(j as i32);
                k += 1;
            }
            self.work += (self.row_cnt[i] as f64 * 7.0) * self.work_mult;
            if k > 0 {
                self.tracker.mark_row(ii);
                self.rhs[i] += drhs;
                if big {
                    self.activity(Some(&[ii]));
                } else {
                    self.act_min[i] = guarded_add(self.act_min[i], dmin, false);
                }
                total += k;
            }
        }
        self.work += (rows.len() as f64 * 8.0) * self.work_mult;
        self.coefs_changed += total;
        total
    }

    /// `36e6b6(st, mode)`: drop dead nodes from the lists. Mode 0 cleans the tracker level's raw column and row
    /// lists and sends the dead column nodes to the pending list (they may still sit in other rows); otherwise the
    /// pending list joins the free list first, every column and row is cleaned, dead column nodes go to the free
    /// list and +0xd8 becomes the live node count. +0xd8 drops by one per node leaving a column.
    pub fn gc(&mut self, mode: i32) {
        let full = mode != 0;
        let mu = self.work_mult;
        let (rows, cols): (Vec<i32>, Vec<i32>) = if full {
            if self.pend != NIL {
                let (mut t, mut links) = (self.pend, 0);
                while self.nd[t as usize].nc != NIL {
                    t = self.nd[t as usize].nc;
                    links += 1;
                }
                self.work += (links as f64 * 5.0) * mu;
                self.nd[t as usize].nc = self.free;
                self.nfree += self.npend;
                self.free = self.pend;
                self.pend = NIL;
                self.npend = 0;
            }
            ((0..self.m as i32).collect(), (0..self.n as i32).collect())
        } else {
            let lv = self.tracker.level;
            (self.tracker.rows.list[lv][..self.tracker.rows.a[lv]].to_vec(),
             self.tracker.cols.list[lv][..self.tracker.cols.a[lv]].to_vec())
        };
        let mut live = 0i64;
        for &j in &cols {
            let j = j as usize;
            let mut q = self.col_head[j];
            if q == NIL {
                continue;
            }
            let (mut tail, mut walked) = (NIL, 0);
            while q != NIL {
                let at = q;
                q = self.nd[at as usize].nc;
                walked += 1;
                if self.nd[at as usize].col < 0 {
                    if full {
                        self.nd[at as usize].nc = self.free;
                        self.free = at;
                        self.nfree += 1;
                    } else {
                        self.nd[at as usize].nc = self.pend;
                        self.pend = at;
                        self.npend += 1;
                    }
                    self.nnz -= 1;
                } else {
                    if tail == NIL { self.col_head[j] = at } else { self.nd[tail as usize].nc = at }
                    tail = at;
                    live += 1;
                }
            }
            if tail == NIL { self.col_head[j] = NIL } else { self.nd[tail as usize].nc = NIL }
            self.work += (walked as f64 * mu) * 8.0;
        }
        self.work += (cols.len() as f64 * 3.0) * mu;
        for &i in &rows {
            let i = i as usize;
            let mut q = self.row_head[i];
            if q == NIL {
                continue;
            }
            let (mut tail, mut walked) = (NIL, 0);
            while q != NIL {
                let at = q;
                q = self.nd[at as usize].nr;
                walked += 1;
                if self.nd[at as usize].col >= 0 {
                    if tail == NIL { self.row_head[i] = at } else { self.nd[tail as usize].nr = at }
                    tail = at;
                }
            }
            if tail == NIL { self.row_head[i] = NIL } else { self.nd[tail as usize].nr = NIL }
            self.work += (walked as f64 * mu) * 8.0;
        }
        self.work += (rows.len() as f64 * 3.0) * mu;
        if full {
            self.nnz = live;
        }
    }

    /// 3cd406 inline implied bounds (PREBOUND && DualReductions, m2_spine_b.md §5.3): for the columns of `53af0f`,
    /// `3dcc78` mode 2 bounds tighten lb / ub when beyond 1000·ft and below 1e8 in magnitude.
    pub fn implied_bounds(&mut self, p: &Params) {
        let tol = p.feastol * 1000.0;
        let cols = self.adjacent_list(true);
        let mut nchg = 0;
        for &jj in &cols {
            let j = jj as usize;
            if self.col_cnt[j] <= 0 {
                continue;
            }
            let (lo, hi) = self.implied_col_bounds(j, 2, None);
            let lbt = lo.abs() < 1e8 && lo > tol + self.lb[j];
            if lbt {
                nchg += 1;
                let (l, u) = (self.lb[j], self.ub[j]);
                self.bound_change(j, l, u, lo, u);
                self.lb[j] = lo;
            }
            let ubt = hi.abs() < 1e8 && self.ub[j] - tol > hi;
            if ubt {
                nchg += 1;
                let (l, u) = (self.lb[j], self.ub[j]);
                self.bound_change(j, l, u, l, hi);
                self.ub[j] = hi;
            }
            if lbt || ubt {
                self.tracker.mark_col(jj);
            }
        }
        self.bounds_tightened += nchg;
    }
}
