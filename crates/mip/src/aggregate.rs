//! `37f484` aggregation on the MIP state (docs/m2_red_outer_b.md §6). Copied from the LP port
//! (crates/lp/src/presolve/agg.rs, docs/m1_presolve_agg.md; LP modules are private and crates/lp stays as it is)
//! with the MIP branches: the `53af0f` candidate list, integer columns in stage A (`377cb6`/`3773aa` with the
//! integer row cache `local_170`, the binary VUB pattern), stage B's cls-0 columns, two-term rows and slack upper
//! bound, stage C's integer guards and pair folding, MIP records (`39b9e0` MIP branch, type 11), no stage D.
use crate::state::{guarded_add, Node, State, NIL};
use crate::subst::sort_asc;
use crate::post::PostRec;
use crate::Params;

/// GURO_PAR_ZERO (env+0x2498)
const ZERO: f64 = 1e-13;

/// The tolerances and parameters `37f484` reads (LP port's `ColTols` / `AggPar` names).
struct ColTols {
    infbound: f64,
    feastol: f64,
    numeric_focus: i32,
    dual_reductions: bool,
    precrush: i32,
    agg_mult_limit: f64,
    pair_agg: i32,
    /// AggFill as `3db19d` leaves it (-1 becomes 0)
    fill: i32,
}

/// state+0x160..+0x1a0 taken out of the state while `37f484` runs: node arena, row and column heads, free list
/// (+0x1a0, count +0x198) and pending list (+0x190, count +0x188), plus the +0x1b0 scatter (cleared after use).
struct Mat {
    nd: Vec<Node>,
    rh: Vec<u32>,
    ch: Vec<u32>,
    free: u32,
    nfree: i64,
    pend: u32,
    npend: i64,
    scat: Vec<u32>,
}

/// Per-row caches of `37f484` (`local_1e0`, `local_1d8`, `local_1e8`, `local_180`, `local_178`, `local_170`);
/// hi == 0 (ints == 0) means stale.
struct RowCache {
    hi: Vec<f64>,
    lo: Vec<f64>,
    amax: Vec<f64>,
    amin: Vec<f64>,
    rowmax: Vec<f64>,
    ints: Vec<f64>,
}

impl Mat {
    /// `3b85ea`: grab `k` more nodes for the free list.
    fn grow(&mut self, k: usize) {
        for _ in 0..k {
            let p = self.nd.len() as u32;
            self.nd.push(Node { val: 0.0, row: 0, col: -1, nr: NIL, nc: self.free });
            self.free = p;
        }
        self.nfree += k as i64;
    }
}

impl State {

    /// Add (`add`) or remove the contribution `a * bound` of a column with bounds [l, u] to row i's activities
    /// (max side first), as the inline updates of `37f484`/`3791fd` do.
    fn ag_contrib(&mut self, i: usize, a: f64, l: f64, u: f64, add: bool, inf: f64) {
        let s = if add { 1 } else { -1 };
        let term = |x: f64| if add { x * a } else { -(x * a) };
        let (xmax, fmax, xmin, fmin) = if a <= 0.0 { (l, -inf < l, u, u < inf) } else { (u, u < inf, l, -inf < l) };
        if fmax {
            self.act_max[i] = guarded_add(self.act_max[i], term(xmax), true);
        } else {
            self.inf_max[i] += s;
        }
        if fmin {
            self.act_min[i] = guarded_add(self.act_min[i], term(xmin), false);
        } else {
            self.inf_min[i] += s;
        }
    }

    /// `53d18c`: mark (tracker + 0x30) every live column of row r.
    fn ag_mark_row(&mut self, mat: &Mat, r: usize) {
        let mut p = mat.rh[r];
        let mut k = 0;
        while p != NIL {
            let q = mat.nd[p as usize];
            if q.col >= 0 {
                self.tracker.mark_col(q.col);
                self.tracker.cols.flag[q.col as usize] |= 0x30;
            }
            p = q.nr;
            k += 1;
        }
        self.work += self.work_mult * 4.0 * k as f64;
    }

    /// `53d2b0`: mark (tracker + 0x30) the row of every live node of column j.
    fn ag_mark_col(&mut self, mat: &Mat, j: usize) {
        let mut p = mat.ch[j];
        let mut k = 0;
        while p != NIL {
            let q = mat.nd[p as usize];
            if q.col >= 0 {
                self.tracker.mark_row(q.row);
                self.tracker.rows.flag[q.row as usize] |= 0x30;
            }
            p = q.nc;
            k += 1;
        }
        self.work += self.work_mult * 5.0 * k as f64;
    }

    /// `36f5c4` (also inlined in `3791fd`): dead nodes leave column j for the pending list.
    fn ag_compact_col(&mut self, mat: &mut Mat, j: usize) {
        let mut p = mat.ch[j];
        let mut prev = NIL;
        let mut k = 0.0;
        while p != NIL {
            k += 1.0;
            let next = mat.nd[p as usize].nc;
            if mat.nd[p as usize].col < 0 {
                mat.nd[p as usize].nc = mat.pend;
                mat.pend = p;
                mat.npend += 1;
                self.nnz -= 1;
            } else {
                if prev == NIL { mat.ch[j] = p } else { mat.nd[prev as usize].nc = p }
                prev = p;
            }
            p = next;
        }
        if prev == NIL { mat.ch[j] = NIL } else { mat.nd[prev as usize].nc = NIL }
        self.work += (k + k) * self.work_mult;
    }

    /// Inline row compaction: dead nodes are only unlinked from row r.
    fn ag_compact_row(&mut self, mat: &mut Mat, r: usize) {
        let mut p = mat.rh[r];
        let mut prev = NIL;
        let mut k = 0.0;
        while p != NIL {
            k += 1.0;
            let next = mat.nd[p as usize].nr;
            if mat.nd[p as usize].col >= 0 {
                if prev == NIL { mat.rh[r] = p } else { mat.nd[prev as usize].nr = p }
                prev = p;
            }
            p = next;
        }
        if prev == NIL { mat.rh[r] = NIL } else { mat.nd[prev as usize].nr = NIL }
        self.work += (k + k) * self.work_mult;
    }

    /// Kill every node of row r and drop the row's list. Returns the nodes visited (work).
    fn ag_kill_row(mat: &mut Mat, r: usize) -> f64 {
        let mut p = mat.rh[r];
        let mut k = 0.0;
        while p != NIL {
            mat.nd[p as usize].col = -1;
            p = mat.nd[p as usize].nr;
            k += 1.0;
        }
        mat.rh[r] = NIL;
        k
    }

    /// `53d3d9`: pending nodes join the free list; columns and rows flagged 0x10 in the current level's b-lists
    /// are compacted (dead column nodes go straight to free) and lose the flag.
    fn ag_gc(&mut self, mat: &mut Mat) {
        let mu = self.work_mult;
        if mat.pend != NIL {
            let mut tail = mat.pend;
            let mut k = 0;
            while mat.nd[tail as usize].nc != NIL {
                tail = mat.nd[tail as usize].nc;
                k += 1;
            }
            self.work += k as f64 * mu;
            mat.nd[tail as usize].nc = mat.free;
            mat.free = mat.pend;
            mat.nfree += mat.npend;
            mat.pend = NIL;
            mat.npend = 0;
        }
        let lv = self.tracker.level;
        for idx in 0..self.tracker.cols.b[lv] {
            let j = self.tracker.cols.list[lv][idx] as usize;
            if self.tracker.cols.flag[j] & 0x10 == 0 {
                continue;
            }
            self.tracker.cols.flag[j] &= !0x10;
            let mut p = mat.ch[j];
            let mut prev = NIL;
            let mut k = 0.0;
            while p != NIL {
                k += 1.0;
                let next = mat.nd[p as usize].nc;
                if mat.nd[p as usize].col < 0 {
                    mat.nd[p as usize].nc = mat.free;
                    mat.free = p;
                    mat.nfree += 1;
                    self.nnz -= 1;
                } else {
                    if prev == NIL { mat.ch[j] = p } else { mat.nd[prev as usize].nc = p }
                    prev = p;
                }
                p = next;
            }
            if prev == NIL { mat.ch[j] = NIL } else { mat.nd[prev as usize].nc = NIL }
            self.work += (k + k) * mu;
        }
        self.work += self.tracker.cols.b[lv] as f64 * 3.0 * mu;
        for idx in 0..self.tracker.rows.b[lv] {
            let i = self.tracker.rows.list[lv][idx] as usize;
            if self.tracker.rows.flag[i] & 0x10 == 0 {
                continue;
            }
            self.tracker.rows.flag[i] &= !0x10;
            // keeps the first row_cnt live nodes and cuts the list there
            let mut left = self.row_cnt[i];
            let mut p = mat.rh[i];
            let mut prev = NIL;
            let mut k = 0.0;
            while p != NIL {
                let next = mat.nd[p as usize].nr;
                if mat.nd[p as usize].col >= 0 {
                    if prev == NIL { mat.rh[i] = p } else { mat.nd[prev as usize].nr = p }
                    prev = p;
                    left -= 1;
                    if left == 0 {
                        break;
                    }
                }
                p = next;
                k += 1.0;
            }
            if prev == NIL { mat.rh[i] = NIL } else { mat.nd[prev as usize].nr = NIL }
            self.work += (k + k) * mu;
        }
        self.work += self.tracker.rows.b[lv] as f64 * 3.0 * mu;
    }

    /// A node for (i, k) with value v at the front of column k: the reused node of the eliminated column (it keeps
    /// its row slot) or a free one put at the front of row i.
    fn ag_new_node(&mut self, mat: &mut Mat, i: usize, k: usize, v: f64, reuse: &mut u32) {
        let p = if *reuse != NIL {
            std::mem::replace(reuse, NIL)
        } else {
            if mat.nfree == 0 {
                mat.grow(self.m.max(3000)); // 3b85ea
            }
            let p = mat.free;
            mat.free = mat.nd[p as usize].nc;
            mat.nfree -= 1;
            self.nnz += 1;
            mat.nd[p as usize].row = i as i32;
            mat.nd[p as usize].nr = mat.rh[i];
            mat.rh[i] = p;
            self.row_cnt[i] += 1;
            p
        };
        let q = &mut mat.nd[p as usize];
        q.col = k as i32;
        q.val = v;
        q.nc = mat.ch[k];
        mat.ch[k] = p;
        self.col_cnt[k] += 1;
    }

    /// One fill update: add v to entry (i, k) (`e`, NIL if none). A new entry needs |v| >= ZERO; an entry whose
    /// new value is within max(1, 10|v|) * ZERO dies. With `act`, row i's activities follow (k's bounds).
    #[allow(clippy::too_many_arguments)]
    fn ag_upd(&mut self, mat: &mut Mat, e: u32, i: usize, k: usize, v: f64, act: bool, reuse: &mut u32, inf: f64) {
        let (l, u) = (self.lb[k], self.ub[k]);
        if e == NIL {
            if ZERO <= v.abs() {
                self.ag_new_node(mat, i, k, v, reuse);
                if act {
                    self.ag_contrib(i, v, l, u, true, inf);
                }
            }
            return;
        }
        let old = mat.nd[e as usize].val;
        if act {
            self.ag_contrib(i, old, l, u, false, inf);
        }
        let new = old + v;
        mat.nd[e as usize].val = new;
        let tol = if 1.0 <= v.abs() * 10.0 { v.abs() * 10.0 } else { 1.0 };
        if new.abs() <= tol * ZERO {
            mat.nd[e as usize].col = -1;
            self.row_cnt[i] -= 1;
            self.col_cnt[k] -= 1;
        } else if act {
            self.ag_contrib(i, new, l, u, true, inf);
        }
    }

    /// `3773aa(st, env, r, …)`: row r's cache: max mult_hi, min mult_lo, max |a|, min |a| over live nodes (a column
    /// past AGGMULTLIMIT stops the walk: 2L / 1/(2L), integer reference −1), and the integer reference `ints`: the
    /// row's smallest |a| when all its columns are integer and rhs and coefficients are multiples of it, else −1.
    fn ag_row_cache(&mut self, mat: &Mat, rc: &mut RowCache, r: usize, lim: f64) {
        rc.amax[r] = 0.0;
        rc.amin[r] = 1e101;
        let (mut lo, mut hi) = (1.0f64, 1.0f64);
        let mut d = 1e100f64;
        let mut bad = false;
        let mut s = mat.rh[r];
        let mut visited = 0;
        while s != NIL {
            let x = mat.nd[s as usize];
            s = x.nr;
            visited += 1;
            if x.col < 0 {
                continue;
            }
            let k = x.col as usize;
            if self.vtype[k] == b'C' {
                d = -1.0;
            }
            let a = x.val.abs();
            if a <= d {
                d = a;
            }
            if rc.amax[r] <= a {
                rc.amax[r] = a;
            }
            if rc.amin[r] >= a {
                rc.amin[r] = a;
            }
            let ml = self.mult_lo[k];
            if (ml < lo && {
                lo = ml;
                ml < 1.0 / lim
            }) || {
                let mh = self.mult_hi[k];
                hi < mh && {
                    hi = mh;
                    lim < mh
                }
            } {
                bad = true;
                visited -= 1;
                break;
            }
        }
        self.work = self.work_mult * 4.0 * visited as f64 + self.work;
        if bad {
            rc.hi[r] = lim + lim;
            rc.lo[r] = 1.0 / (lim + lim);
            rc.ints[r] = -1.0;
            return;
        }
        rc.hi[r] = hi;
        rc.lo[r] = lo;
        if 0.0 < d {
            let q = self.rhs[r] / d;
            if 1e-10 < q - (q + 1e-10).floor() {
                d = -1.0;
            }
        }
        if 0.0 < d {
            let mut s = mat.rh[r];
            let mut k = 0;
            while s != NIL {
                let x = mat.nd[s as usize];
                if x.col >= 0 {
                    let q = x.val / d;
                    if 1e-10 < q - (q + 1e-10).floor() {
                        d = -1.0;
                    }
                }
                s = x.nr;
                k += 1;
            }
            self.work = self.work_mult * 3.0 * k as f64 + self.work;
        }
        rc.ints[r] = d;
    }

    /// `377cb6(st, env, node, isInt, …)`: may the pivot node p's column be substituted out of its row without the
    /// multiplier growth bounds (state+0x80/+0x88) passing AGGMULTLIMIT? An integer column also needs |a| to be the
    /// row's integer reference (`ints`). Returns 0 / 1 (2: the two-term integer case below).
    fn ag_mult_ok(&mut self, mat: &Mat, rc: &mut RowCache, p: u32, lim: f64, int: bool) -> i32 {
        let q = mat.nd[p as usize];
        let r = q.row as usize;
        if rc.hi[r] == 0.0 {
            self.ag_row_cache(mat, rc, r, lim);
        }
        let a = q.val.abs();
        let j = q.col as usize;
        let inv = 1.0 / lim;
        if lim < rc.hi[r] || inv > rc.lo[r] || lim < (rc.amax[r] * self.mult_hi[j]) / a || inv > (rc.amin[r] * self.mult_lo[j]) / a {
            return 0;
        }
        if int {
            if rc.ints[r] == 0.0 {
                self.ag_row_cache(mat, rc, r, lim);
            }
            if 1e-10 < (a - rc.ints[r]).abs() {
                if self.row_cnt[r] != 2 || !(1e-10 > self.rhs[r].abs()) {
                    return 0;
                }
                let mut s = mat.rh[r];
                let mut k = 0;
                while s != NIL && (s == p || mat.nd[s as usize].col < 0) {
                    s = mat.nd[s as usize].nr;
                    k += 1;
                }
                self.work = (k as f64 + k as f64) * self.work_mult + self.work;
                if s == NIL || self.col_flag[mat.nd[s as usize].col as usize] == 0
                    || !(1e-10 > ((mat.nd[s as usize].val / q.val).abs() - 1.0).abs()) {
                    return 0;
                }
                return 2;
            }
        }
        1
    }

    /// `378c30__0` (`upper`) / `__1`: is column j's upper (lower) bound implied by one of its rows?
    fn ag_implied(&mut self, mat: &Mat, j: usize, upper: bool, tol_i: f64, t: &ColTols) -> bool {
        let mut visited = 0;
        let found = self.ag_implied_walk(mat, j, upper, tol_i, t, &mut visited);
        self.work += self.work_mult * 8.0 * visited as f64;
        found
    }

    /// `agg_implied` without the work: `visited` gets the nodes passed before the one that implies the bound.
    fn ag_implied_walk(&self, mat: &Mat, j: usize, upper: bool, tol_i: f64, t: &ColTols, visited: &mut i32) -> bool {
        let inf = t.infbound;
        let mut p = mat.ch[j];
        let (base, f) = if upper {
            let l = self.lb[j];
            (if -l < inf { -l } else { 0.0 }, inf <= -l)
        } else {
            let u = self.ub[j];
            (if u < inf { u } else { 0.0 }, inf <= u)
        };
        while p != NIL {
            let q = mat.nd[p as usize];
            p = q.nc;
            *visited += 1;
            if q.col < 0 || q.val.abs() < tol_i {
                continue;
            }
            let (i, a) = (q.row as usize, q.val);
            let eq = self.sense[i] == b'=';
            if upper {
                let v = if -a >= 0.0 {
                    if !(eq && (f as i32) == self.inf_max[i]) {
                        continue;
                    }
                    self.act_max[i] / -a
                } else {
                    if (f as i32) != self.inf_min[i] {
                        continue;
                    }
                    self.act_min[i] / -a
                };
                if -self.ub[j] - t.feastol <= base - v {
                    *visited -= 1;
                    return true;
                }
            } else {
                let v = if a >= 0.0 {
                    if !(eq && (f as i32) == self.inf_max[i]) {
                        continue;
                    }
                    self.act_max[i]
                } else {
                    if (f as i32) != self.inf_min[i] {
                        continue;
                    }
                    self.act_min[i]
                };
                if self.lb[j] - t.feastol <= base - v / a {
                    *visited -= 1;
                    return true;
                }
            }
        }
        false
    }

    /// `37f484(st, env)` on the MIP state (+0xfc = 1, +0xe0 = 0): no prescan (+0x488 = −1, tolA = 1e-6, tolI =
    /// FeasibilityTol), candidates from `53af0f` classified by bound finiteness only, stages A, B and C (stage D
    /// needs +0xfc == 0), then `53d3d9` when rows or columns went. Returns a status (0x2711 never here).
    pub fn aggregate(&mut self, p: &Params) -> i32 {
        let t = ColTols {
            infbound: crate::state::INF,
            feastol: p.feastol,
            numeric_focus: p.numeric_focus,
            dual_reductions: p.dual_reductions != 0,
            precrush: p.precrush,
            agg_mult_limit: p.agg_mult_limit,
            pair_agg: p.prepairagg,
            fill: if p.aggfill == -1 { 0 } else { p.aggfill },
        };
        let (n, m) = (self.n, self.m);
        let inf = t.infbound;
        let rho = if t.numeric_focus == 1 { 0.5 } else if t.numeric_focus < 2 { 0.01 } else { 0.9 };
        let rho2 = rho * 0.01;
        let pmin = 1e-6;
        let rows0 = self.rows_removed;
        let cols0 = self.cols_removed;
        let lv = self.tracker.level;
        let rlist: Vec<i32> = self.tracker.rows.list[lv][..self.tracker.rows.a[lv]].to_vec();
        let (tol_a, tol_i) = (pmin, t.feastol);
        let mut rc = RowCache { hi: vec![0.0; m], lo: vec![0.0; m], amax: vec![0.0; m], amin: vec![0.0; m],
                                rowmax: vec![0.0; m], ints: vec![0.0; m] };
        let mu = self.work_mult;
        // candidates (0 both bounds finite, 1 upper only, 2 lower only, 3 free)
        let cand = self.adjacent_list(true);
        let mut cls = vec![0u8; n];
        for &j in &cand {
            let j = j as usize;
            if self.col_cnt[j] > 0 && self.col_flag[j] == 0 {
                let (l, u) = (self.lb[j], self.ub[j]);
                cls[j] = if -inf < l { if u < inf { 0 } else { 2 } } else if u < inf { 1 } else { 3 };
            }
        }
        self.work = cand.len() as f64 * 6.0 * mu + self.work;
        let mut mat = Mat {
            nd: std::mem::take(&mut self.nd),
            rh: std::mem::take(&mut self.row_head),
            ch: std::mem::take(&mut self.col_head),
            free: self.free,
            nfree: self.nfree,
            pend: self.pend,
            npend: self.npend,
            scat: vec![NIL; n.max(m)],
        };
        let rc_ = self.ag_stage_a(&mut mat, &mut rc, &cls, &cand, tol_a, tol_i, rho, rho2, &t, t.fill);
        if rc_.is_ok() {
            if t.dual_reductions && t.precrush < 2 {
                self.ag_stage_b(&mut mat, &mut rc, &cls, &cand, tol_a, tol_i, rho, &t, t.fill);
            }
            // stage C needs DualReductions and PREPAIRAGG 2 or 3 in MIP (1 is LP only)
            if t.dual_reductions && (t.pair_agg == 2 || t.pair_agg == 3) {
                self.ag_stage_c(&mut mat, &rlist, pmin, &t);
            }
            if rows0 < self.rows_removed || cols0 < self.cols_removed {
                self.ag_gc(&mut mat);
            }
        }
        self.nd = mat.nd;
        self.row_head = mat.rh;
        self.col_head = mat.ch;
        self.free = mat.free;
        self.nfree = mat.nfree;
        self.pend = mat.pend;
        self.npend = mat.npend;
        rc_.err().unwrap_or(0)
    }

    /// Stage A (`37f484` L757-1287): an `=` row substitutes out a column whose bounds its rows imply.
    #[allow(clippy::too_many_arguments)]
    fn ag_stage_a(&mut self, mat: &mut Mat, rc: &mut RowCache, cls: &[u8], cand: &[i32], tol_a: f64, tol_i: f64, rho: f64,
                   rho2: f64, t: &ColTols, fill: i32) -> Result<(), i32> {
        let (n, m) = (self.n, self.m);
        let lim_l = t.agg_mult_limit;
        let ff = fill as f64;
        let mut est = vec![-1i32; n];
        let mut bucket = vec![0i32; m + 2];
        for &j in cand {
            let j = j as usize;
            est[j] = -1;
            let c = self.col_cnt[j];
            if !(0 < c && self.col_flag[j] == 0 && 1.0 / lim_l <= self.mult_lo[j] && self.mult_hi[j] <= lim_l) {
                continue;
            }
            let mut lim = if 2 < c {
                let x = (ff + 2.0) / ((c - 2) as f64) + 3.0001;
                (if x <= (n + 1) as f64 { x } else { (n + 1) as f64 }) as i32
            } else {
                n as i32 + 1
            };
            let int = self.vtype[j] != b'C';
            let mut best = lim;
            let mut p = mat.ch[j];
            while p != NIL {
                let cur = p;
                let q = mat.nd[cur as usize];
                p = q.nc;
                if q.col < 0 || q.val.abs() < tol_a {
                    continue;
                }
                let r = q.row as usize;
                let a = q.val;
                if self.sense[r] != b'=' || best <= self.row_cnt[r] {
                    // a binary in a variable upper bound row x_k + a·x_j <= 0 caps the estimate at 3
                    let vub = 1 <= self.nbin
                        && self.nbin < n as i32
                        && 3 <= best
                        && a < 0.0
                        && self.vtype[j] == b'B'
                        && self.sense[r] == b'<'
                        && self.row_cnt[r] == 2
                        && !(1e-10 < self.rhs[r].abs())
                        && self.inf_min[r] == 0
                        && self.inf_max[r] == 0
                        && !(self.act_min[r] - a < -1e-10)
                        && !(1e-10 < self.act_max[r] + a);
                    if vub {
                        let mut cap = true;
                        if (a + 1.0).abs() <= 1e-10 {
                            let mut x = mat.rh[r];
                            let mut k = 0;
                            while x != NIL {
                                let y = mat.nd[x as usize];
                                if y.col >= 0 && y.col as usize != j {
                                    cap = self.vtype[y.col as usize] != b'B';
                                    break;
                                }
                                k += 1;
                                x = y.nr;
                            }
                            self.work = (k as f64 + k as f64) * self.work_mult + self.work;
                        }
                        if cap {
                            lim = 3;
                            best = 3;
                        }
                    }
                    continue;
                }
                if self.row_cnt[r] == 1 {
                    best = lim;
                    break;
                }
                if self.ag_mult_ok(mat, rc, cur, lim_l, int) != 0 {
                    best = self.row_cnt[r];
                }
            }
            self.work += c as f64 * 7.0 * self.work_mult;
            if best != lim {
                let c2 = c - 2;
                if (c2 as f64) * ((best - 2) as f64) <= (fill + 2) as f64 {
                    let e = if c < 2 {
                        0.0
                    } else if int {
                        (best * c2 + 1) as f64
                    } else {
                        (c2 * (best - 1) + 1) as f64
                    };
                    if e <= m as f64 {
                        est[j] = e as i32;
                        bucket[e as usize + 1] += 1;
                    }
                }
            }
        }
        let mu = self.work_mult;
        self.work += cand.len() as f64 * 6.0 * mu;
        for i in 1..m + 2 {
            bucket[i] += bucket[i - 1];
        }
        self.work += (m + 1) as f64 * mu;
        let mut order = vec![0i32; bucket[m + 1] as usize];
        let mut saved = vec![0i32; n];
        for &j in cand {
            let j = j as usize;
            if est[j] >= 0 {
                let b = &mut bucket[est[j] as usize];
                order[*b as usize] = j as i32;
                *b += 1;
                saved[j] = self.col_cnt[j];
            }
        }
        self.work += cand.len() as f64 * 5.0 * mu;
        let total = bucket[m] as usize;
        let mut keys: Vec<f64> = Vec::new();
        let mut nodes: Vec<i32> = Vec::new();
        for &j in &order[..total] {
            let j = j as usize;
            let c = self.col_cnt[j];
            if !(0 < c && self.col_flag[j] == 0 && c - saved[j] < saved[j] * 2 + 10000) {
                continue;
            }
            let ok = match cls[j] {
                0 => self.ag_implied(mat, j, true, tol_i, t) && self.ag_implied(mat, j, false, tol_i, t),
                1 => self.ag_implied(mat, j, true, tol_i, t),
                2 => self.ag_implied(mat, j, false, tol_i, t),
                _ => true,
            };
            if !ok {
                continue;
            }
            let lim = if 2 < c { ((ff + 2.0) / ((c - 2) as f64) + 3.0001) as i32 } else { n as i32 + 1 };
            let int = self.vtype[j] != b'C';
            let mut colmax = 0.0f64;
            keys.clear();
            nodes.clear();
            let mut give_up = false;
            let mut p = mat.ch[j];
            let mut visited = 0.0;
            while p != NIL {
                let q = mat.nd[p as usize];
                if q.col >= 0 {
                    let a = q.val.abs();
                    if colmax <= a {
                        colmax = a;
                    }
                    let r = q.row as usize;
                    if tol_a <= a && self.sense[r] == b'=' && self.row_cnt[r] <= lim {
                        if self.row_cnt[r] == 1 {
                            give_up = true;
                            break;
                        }
                        keys.push(self.row_cnt[r] as f64);
                        nodes.push(p as i32);
                    }
                }
                p = q.nc;
                visited += 1.0;
            }
            if give_up {
                keys.clear();
            }
            let kk = keys.len() as f64;
            self.work += (visited * 6.0 + kk + kk) * mu;
            if keys.is_empty() {
                continue;
            }
            if keys.len() >= 2 {
                sort_asc(&mut keys, &mut nodes);
            }
            let thr = colmax * rho;
            let (mut ratio_best, mut cnt_best, mut piv) = (0.0f64, lim, NIL);
            let mut pivot_kind = 0;
            let mut seen = 0;
            for (&key, &p) in keys.iter().zip(nodes.iter()) {
                let cnt = key as i32;
                if cnt_best < cnt {
                    break;
                }
                seen += 1;
                let q = mat.nd[p as usize];
                let r = q.row as usize;
                if rc.rowmax[r] == 0.0 {
                    let mut mx = 0.0f64;
                    let mut s = mat.rh[r];
                    let mut k = 0;
                    while s != NIL {
                        let x = mat.nd[s as usize];
                        if x.col >= 0 && mx <= x.val.abs() {
                            mx = x.val.abs();
                        }
                        s = x.nr;
                        k += 1;
                    }
                    self.work += k as f64 * 3.0 * mu;
                    rc.rowmax[r] = mx;
                }
                let rmax = rc.rowmax[r];
                let a = q.val.abs();
                if (cnt != cnt_best || rmax / a < ratio_best)
                    && ((thr <= a && !int) || rho * rmax <= a || (rmax * rho2 <= a && cnt < 3 && !int))
                {
                    let kind = self.ag_mult_ok(mat, rc, p as u32, lim_l, int);
                    if kind != 0 {
                        ratio_best = rmax / a;
                        piv = p as u32;
                        cnt_best = cnt;
                        pivot_kind = kind;
                    }
                }
            }
            self.work += seen as f64 * 4.0 * mu;
            if piv == NIL {
                continue;
            }
            let need = (cnt_best + fill) as i64;
            if mat.nfree <= self.col_cnt[j] as i64 + need {
                self.ag_gc(mat);
            }
            if !(need + (self.col_cnt[j] as i64) < mat.nfree) {
                continue;
            }
            let r = mat.nd[piv as usize].row as usize;
            let a = mat.nd[piv as usize].val;
            mat.nd[piv as usize].col = -1;
            self.ag_compact_row(mat, r);
            self.ag_compact_col(mat, j);
            if pivot_kind == 2 {
                self.ag_integer_pair(mat, rc, j, r, a);
            }
            self.ag_subst(mat, rc, a, r, j, t.infbound);
            let k = Self::ag_kill_row(mat, r);
            self.work += (k + k) * mu;
        }
        self.work += total as f64 * 10.0 * mu;
        Ok(())
    }

    /// Stage A's protected integer pair: transfer integrality and intersect the other column's bounds.
    fn ag_integer_pair(&mut self, mat: &Mat, rc: &mut RowCache, j: usize, r: usize, a: f64) {
        let mut walked = 0;
        if self.row_cnt[r] == 2 && self.rhs[r].abs() < 1e-10 {
            let q = mat.nd[mat.rh[r] as usize];
            let k = q.col as usize;
            let old = self.vtype[k];
            let ratio = q.val / a;
            if (ratio - 1.0).abs() < 1e-10 {
                if old == b'C' { self.vtype[k] = b'I'; }
                self.lb[k] = crate::helpers::maxsd((self.lb[k] + self.intfeastol).floor(), -self.ub[j]);
                self.ub[k] = crate::helpers::minsd((self.ub[k] - self.intfeastol).ceil(), -self.lb[j]);
            } else if (ratio + 1.0).abs() < 1e-10 {
                self.vtype[k] = self.vtype[j];
                self.lb[k] = crate::helpers::maxsd((self.lb[k] + self.intfeastol).floor(), self.lb[j]);
                self.ub[k] = crate::helpers::minsd((self.ub[k] - self.intfeastol).ceil(), self.ub[j]);
            }
            if old != self.vtype[k] {
                let mut p = mat.ch[k];
                while p != NIL {
                    let q = mat.nd[p as usize];
                    if q.col >= 0 {
                        rc.ints[q.row as usize] = 0.0;
                        rc.hi[q.row as usize] = 0.0;
                    }
                    walked += 1;
                    p = q.nc;
                }
            }
            self.work = walked as f64 * 3.0 * self.work_mult + self.work;
        }
        self.work += self.work_mult * 20.0;
    }

    /// `39b9e0(c0, a, env, st, 0, j, r, rowhead[r], cnt)`, MIP branch (as `helpers::push_subst_row`, on the
    /// taken-out lists): x_j = c0 + Σ (−a_rk/a)·x_k over row r's live nodes; the other columns' multiplier growth
    /// bounds follow j's; rows and columns removed +1. W += (5·μ)·nodes walked.
    fn ag_push_subst(&mut self, mat: &Mat, j: usize, r: usize, c0: f64, a: f64) {
        self.cols_removed += 1;
        self.rows_removed += 1;
        let (mut ind, mut val) = (Vec::new(), Vec::new());
        let mut p = mat.rh[r];
        let mut walked = 0;
        while p != NIL {
            let q = mat.nd[p as usize];
            p = q.nr;
            walked += 1;
            if q.col < 0 {
                continue;
            }
            let k = q.col as usize;
            ind.push(q.col);
            val.push((-q.val) / a);
            if k != j {
                let t = (q.val / a).abs();
                self.mult_lo[k] = crate::helpers::minsd(self.mult_lo[k], self.mult_lo[j] * t);
                self.mult_hi[k] = crate::helpers::maxsd(self.mult_hi[k], self.mult_hi[j] * t);
            }
        }
        self.work += (5.0 * self.work_mult) * walked as f64;
        self.post.push(PostRec::Subst { j: j as i32, c0, ind, val });
    }

    /// `3791fd`: substitute column j out through `=` row r (pivot value a, already taken out of both lists).
    fn ag_subst(&mut self, mat: &mut Mat, rc: &mut RowCache, a: f64, r: usize, j: usize, inf: f64) {
        let rhs_r = self.rhs[r];
        self.ag_push_subst(mat, j, r, rhs_r / a, a);
        rc.ints[r] = 0.0;
        rc.hi[r] = 0.0;
        rc.rowmax[r] = 0.0;
        self.objcon += self.obj[j] * (rhs_r / a);
        let mu = self.work_mult;
        let mut s_row = 0.0f64;
        let mut len_r = 0;
        let mut p = mat.rh[r];
        while p != NIL {
            let q = mat.nd[p as usize];
            p = q.nr;
            let k = q.col as usize;
            self.col_cnt[k] -= 1;
            self.obj[k] -= (self.obj[j] * q.val) / a;
            s_row += self.col_cnt[k] as f64;
            len_r += 1;
        }
        self.work += len_r as f64 * 4.0 * mu;
        self.ag_mark_row(mat, r);
        self.ag_mark_col(mat, j);
        let (lj, uj) = (self.lb[j], self.ub[j]);
        let mut s_col = 0.0f64;
        let (mut rows, mut vals, mut slot) = (Vec::new(), Vec::new(), Vec::new());
        let mut p = mat.ch[j];
        while p != NIL {
            let q = mat.nd[p as usize];
            let i = q.row as usize;
            let d = (rhs_r * q.val) / a;
            s_col += self.row_cnt[i] as f64;
            self.rhs[i] -= d;
            rc.ints[i] = 0.0;
            rc.hi[i] = 0.0;
            rc.rowmax[i] = 0.0;
            self.ag_contrib(i, q.val, lj, uj, false, inf);
            self.act_min[i] = guarded_add(self.act_min[i], d, false);
            self.act_max[i] = guarded_add(self.act_max[i], d, true);
            rows.push(i);
            vals.push(q.val);
            slot.push(p);
            p = q.nc;
        }
        let u = rows.len() as f64;
        self.work += u * 5.0 * mu;
        self.work += u * 6.0 * mu;
        self.row_cnt[r] = -3;
        self.obj[j] = 0.0;
        self.col_cnt[j] = -3 - r as i32;
        mat.ch[j] = NIL;
        if s_row <= s_col {
            // B1: row r's columns outside, j's entries inside
            let mut p = mat.rh[r];
            while p != NIL {
                let qk = mat.nd[p as usize];
                p = qk.nr;
                let k = qk.col as usize;
                if self.tracker.cols.flag[k] & 0x10 != 0 {
                    self.ag_compact_col(mat, k);
                }
                let mut s = mat.ch[k];
                let mut ns = 0.0;
                while s != NIL {
                    mat.scat[mat.nd[s as usize].row as usize] = s;
                    s = mat.nd[s as usize].nc;
                    ns += 1.0;
                }
                self.work += (ns + ns) * mu;
                for tt in 0..rows.len() {
                    let i = rows[tt];
                    let v = (-vals[tt] * qk.val) / a;
                    let e = mat.scat[i];
                    let mut re = slot[tt];
                    self.ag_upd(mat, e, i, k, v, true, &mut re, inf);
                    slot[tt] = re;
                    if e != NIL {
                        mat.scat[i] = NIL;
                    }
                }
                self.work += ns * 16.0 * mu;
                let mut s = mat.ch[k];
                let mut nc = 0.0;
                while s != NIL {
                    let x = mat.nd[s as usize];
                    if x.col >= 0 {
                        mat.scat[x.row as usize] = NIL;
                    }
                    s = x.nc;
                    nc += 1.0;
                }
                self.work += (nc + nc) * mu;
            }
            self.work += len_r as f64 * 5.0 * mu;
        } else {
            // B2: j's entries outside, row r's columns inside (scatter by column)
            for tt in 0..rows.len() {
                let i = rows[tt];
                if self.tracker.rows.flag[i] & 0x10 != 0 {
                    self.ag_compact_row(mat, i);
                }
                let mut s = mat.rh[i];
                let mut ns = 0.0;
                while s != NIL {
                    let x = mat.nd[s as usize];
                    if x.col >= 0 {
                        mat.scat[x.col as usize] = s;
                    }
                    s = x.nr;
                    ns += 1.0;
                }
                self.work += (ns + ns) * mu;
                let mut p = mat.rh[r];
                let mut ni = 0;
                while p != NIL {
                    let qk = mat.nd[p as usize];
                    p = qk.nr;
                    ni += 1;
                    let k = qk.col as usize;
                    let v = (-qk.val * vals[tt]) / a;
                    let e = mat.scat[k];
                    let mut re = slot[tt];
                    self.ag_upd(mat, e, i, k, v, true, &mut re, inf);
                    if re != slot[tt] {
                        mat.scat[j] = NIL;
                    }
                    slot[tt] = re;
                    if e != NIL {
                        mat.scat[k] = NIL;
                    }
                }
                self.work += ni as f64 * 16.0 * mu;
                let mut s = mat.rh[i];
                let mut nc = 0.0;
                while s != NIL {
                    let x = mat.nd[s as usize];
                    if x.col >= 0 {
                        mat.scat[x.col as usize] = NIL;
                    }
                    s = x.nr;
                    nc += 1.0;
                }
                self.work += (nc + nc) * mu;
            }
            self.work += u * 5.0 * mu;
        }
        // nodes of j not reused for fill leave their rows
        for &s in &slot {
            if s != NIL {
                let i = mat.nd[s as usize].row as usize;
                self.row_cnt[i] -= 1;
                mat.nd[s as usize].col = -1;
                mat.nd[s as usize].nc = mat.pend;
                mat.pend = s;
                mat.npend += 1;
                self.nnz -= 1;
            }
        }
        self.work += u * 3.0 * mu;
    }

    /// Stage B (`37f484` L1288-1430, L2555-3890): a `<` row turns a free-ish column into its slack.
    #[allow(clippy::too_many_arguments)]
    fn ag_stage_b(&mut self, mat: &mut Mat, rc: &mut RowCache, cls: &[u8], cand: &[i32], tol_a: f64, tol_i: f64, rho: f64,
                   t: &ColTols, fill: i32) {
        let n = self.n;
        let mu = self.work_mult;
        let lim_l = t.agg_mult_limit;
        let ff = fill as f64;
        let (mut keys, mut nodes) = (Vec::new(), Vec::new());
        for &j in cand {
            let j = j as usize;
            let c = self.col_cnt[j];
            if !(0 < c && self.col_flag[j] == 0 && self.vtype[j] == b'C' && 1.0 / lim_l <= self.mult_lo[j] && self.mult_hi[j] <= lim_l) {
                continue;
            }
            let lim = if c < 3 { n as i32 } else { ((ff + 1.0) / (c as f64 - 2.0) + 1.0001) as u32 as i32 };
            if lim < 2 {
                continue;
            }
            let ok = match cls[j] {
                0 => self.ag_implied(mat, j, true, tol_i, t) && self.ag_implied(mat, j, false, tol_i, t), // MIP only
                1 => self.ag_implied(mat, j, true, tol_i, t),
                2 => self.ag_implied(mat, j, false, tol_i, t),
                _ => true,
            };
            if !ok {
                continue;
            }
            let mut colmax = 0.0f64;
            keys.clear();
            nodes.clear();
            let mut give_up = false;
            let mut p = mat.ch[j];
            let mut visited = 0.0;
            // a two-term row whose other column is binary (or flagged 1) is the only candidate: earlier ones go
            let mut forced = -1i32;
            while p != NIL {
                let q = mat.nd[p as usize];
                if q.col >= 0 {
                    let a = q.val.abs();
                    if colmax <= a {
                        colmax = a;
                    }
                    let i = q.row as usize;
                    if self.row_cnt[i] == 1 {
                        give_up = true;
                        break;
                    }
                    if self.row_cnt[i] == 2 {
                        let mut x = mat.rh[i];
                        while x != NIL && (mat.nd[x as usize].col < 0 || mat.nd[x as usize].col as usize == j) {
                            x = mat.nd[x as usize].nr;
                        }
                        if x != NIL {
                            let o = mat.nd[x as usize].col as usize;
                            if self.vtype[o] == b'B' || self.col_flag[o] & 1 != 0 {
                                keys.clear();
                                nodes.clear();
                                forced = i as i32;
                                if 1e6 < a {
                                    give_up = true;
                                    break;
                                }
                            }
                        }
                    }
                    if self.sense[i] == b'<' {
                        if tol_a <= a && self.row_cnt[i] <= lim {
                            if forced != -1 && i as i32 != forced {
                                visited += 1.0;
                                break;
                            }
                            keys.push(self.row_cnt[i] as f64);
                            nodes.push(p as i32);
                        }
                    } else {
                        give_up = true; // MIP: a column in an `=` row gets no slack
                        break;
                    }
                }
                p = q.nc;
                visited += 1.0;
                if forced != -1 {
                    break;
                }
            }
            if give_up {
                keys.clear();
            }
            self.work += (visited * 3.0 + keys.len() as f64 * 4.0) * mu;
            if keys.is_empty() {
                continue;
            }
            if keys.len() >= 2 {
                sort_asc(&mut keys, &mut nodes);
            }
            let thr = colmax * rho;
            let (mut best, mut cnt_best, mut piv) = (0.0f64, n as i32 + 1, NIL);
            let mut seen = 0;
            for (&key, &p) in keys.iter().zip(nodes.iter()) {
                let cnt = key as i32;
                if cnt_best < cnt {
                    break;
                }
                seen += 1;
                let x = if cnt < cnt_best { thr } else { best };
                let a = mat.nd[p as usize].val.abs();
                if x < a && self.ag_mult_ok(mat, rc, p as u32, lim_l, false) != 0 {
                    best = a;
                    piv = p as u32;
                    cnt_best = cnt;
                }
            }
            self.work += seen as f64 * 3.0 * mu;
            if piv == NIL {
                continue;
            }
            let need = (cnt_best + fill) as i64;
            if mat.nfree <= self.col_cnt[j] as i64 + need + 1 {
                self.ag_gc(mat);
            }
            if mat.nfree <= self.col_cnt[j] as i64 + need + 1 {
                continue;
            }
            self.ag_slack(mat, rc, piv, j, t.infbound);
        }
        self.work += cand.len() as f64 * 9.0 * mu;
    }

    /// Stage B substitution: with `<` row r (pivot a on column j), x_j = rhs/a - sum(a_rk/a x_k) - s x'_j where
    /// the new x'_j >= 0 takes j's place (s = sign a). Row r goes away; j's entries become -s a_ij.
    fn ag_slack(&mut self, mat: &mut Mat, rc: &mut RowCache, piv: u32, j: usize, inf: f64) {
        let a = mat.nd[piv as usize].val;
        let r = mat.nd[piv as usize].row as usize;
        let s = if a < 0.0 { -1.0 } else { 1.0 };
        let aa = a.abs();
        let (lj, uj) = (self.lb[j], self.ub[j]);
        let (inf_r, min_r) = (self.inf_min[r], self.act_min[r]);
        let mu = self.work_mult;
        let mut s_col = 0.0f64;
        let mut p = mat.ch[j];
        let mut k = 0;
        while p != NIL {
            let q = mat.nd[p as usize];
            p = q.nc;
            k += 1;
            if q.col < 0 {
                continue;
            }
            let i = q.row as usize;
            s_col += self.row_cnt[i] as f64;
            rc.ints[i] = 0.0;
            rc.hi[i] = 0.0;
            self.ag_contrib(i, q.val, lj, uj, false, inf);
        }
        self.work += k as f64 * 3.0 * mu;
        let mut s_row = 0.0f64;
        let mut p = mat.rh[r];
        let mut k = 0;
        while p != NIL {
            let q = mat.nd[p as usize];
            p = q.nr;
            k += 1;
            if q.col >= 0 {
                s_row += self.col_cnt[q.col as usize] as f64;
            }
        }
        self.work += k as f64 * 3.0 * mu;
        mat.nd[piv as usize].val = aa;
        let rhs_r = self.rhs[r];
        self.ag_push_subst(mat, j, r, rhs_r / a, a);
        self.cols_removed -= 1;
        self.post.push(PostRec::T11 { r: r as i32, j: j as i32, a: aa }); // 3a0e1b
        self.mult_lo[j] = 1.0;
        self.mult_hi[j] = 1.0;
        self.lb[j] = 0.0;
        self.ub[j] = inf;
        rc.ints[r] = 0.0;
        rc.hi[r] = 0.0;
        // MIP: the slack is bounded by the row's old minimum activity
        if inf_r == 0 {
            self.ub[j] = -min_r / aa;
        }
        mat.nd[piv as usize].col = -1;
        self.col_cnt[j] -= 1;
        self.objcon += (rhs_r / a) * self.obj[j];
        let mut p = mat.rh[r];
        let mut k = 0;
        while p != NIL {
            let q = mat.nd[p as usize];
            p = q.nr;
            k += 1;
            if q.col >= 0 {
                let k = q.col as usize;
                self.col_cnt[k] -= 1;
                self.obj[k] -= (self.obj[j] * q.val) / a;
            }
        }
        self.work += k as f64 * 5.0 * mu;
        self.obj[j] = -self.obj[j] * s;
        self.ag_compact_row(mat, r);
        self.ag_compact_col(mat, j);
        self.ag_mark_row(mat, r);
        self.ag_mark_col(mat, j);
        self.row_cnt[r] = -3;
        let mut none = NIL;
        let mut outer = 0;
        if s_row <= s_col {
            let mut p = mat.rh[r];
            while p != NIL {
                let qk = mat.nd[p as usize];
                p = qk.nr;
                outer += 1;
                let k = qk.col as usize;
                if self.tracker.cols.flag[k] & 0x10 != 0 {
                    self.ag_compact_col(mat, k);
                }
                let mut x = mat.ch[k];
                let mut ns = 0;
                while x != NIL {
                    mat.scat[mat.nd[x as usize].row as usize] = x;
                    x = mat.nd[x as usize].nc;
                    ns += 1;
                }
                self.work += ns as f64 * 3.0 * mu;
                let mut y = mat.ch[j];
                let mut ni = 0;
                while y != NIL {
                    let qi = mat.nd[y as usize];
                    y = qi.nc;
                    ni += 1;
                    let i = qi.row as usize;
                    let v = (-qi.val * qk.val) / a;
                    let e = mat.scat[i];
                    self.ag_upd(mat, e, i, k, v, true, &mut none, inf);
                    if e != NIL {
                        mat.scat[i] = NIL;
                    }
                }
                self.work += ni as f64 * 10.0 * mu;
                let mut x = mat.ch[k];
                let mut nc = 0;
                while x != NIL {
                    let q = mat.nd[x as usize];
                    if q.col >= 0 {
                        mat.scat[q.row as usize] = NIL;
                    }
                    x = q.nc;
                    nc += 1;
                }
                self.work += nc as f64 * 3.0 * mu;
            }
        } else {
            let mut y = mat.ch[j];
            while y != NIL {
                let qi = mat.nd[y as usize];
                y = qi.nc;
                outer += 1;
                let i = qi.row as usize;
                if self.tracker.rows.flag[i] & 0x10 != 0 {
                    self.ag_compact_row(mat, i);
                }
                let mut x = mat.rh[i];
                let mut ns = 0;
                while x != NIL {
                    let q = mat.nd[x as usize];
                    if q.col >= 0 {
                        mat.scat[q.col as usize] = x;
                    }
                    x = q.nr;
                    ns += 1;
                }
                self.work += ns as f64 * 3.0 * mu;
                let mut p = mat.rh[r];
                let mut ni = 0;
                while p != NIL {
                    let qk = mat.nd[p as usize];
                    p = qk.nr;
                    ni += 1;
                    let k = qk.col as usize;
                    let v = (-qi.val * qk.val) / a;
                    let e = mat.scat[k];
                    self.ag_upd(mat, e, i, k, v, true, &mut none, inf);
                    if e != NIL {
                        mat.scat[k] = NIL;
                    }
                }
                self.work += ni as f64 * 10.0 * mu;
                let mut x = mat.rh[i];
                let mut nc = 0;
                while x != NIL {
                    let q = mat.nd[x as usize];
                    if q.col >= 0 {
                        mat.scat[q.col as usize] = NIL;
                    }
                    x = q.nr;
                    nc += 1;
                }
                self.work += nc as f64 * 3.0 * mu;
            }
        }
        self.work += outer as f64 * 6.0 * mu;
        self.tracker.mark_col(j as i32); // 53d139
        self.tracker.cols.flag[j] |= 0x30;
        let mut y = mat.ch[j];
        let mut k = 0;
        while y != NIL {
            let cur = y as usize;
            let q = mat.nd[cur];
            y = q.nc;
            k += 1;
            let i = q.row as usize;
            let d = (rhs_r * q.val) / a;
            self.rhs[i] -= d;
            let na = -q.val * s;
            mat.nd[cur].val = na;
            self.ag_contrib(i, na, self.lb[j], self.ub[j], true, inf);
            self.act_min[i] = guarded_add(self.act_min[i], d, false);
            self.act_max[i] = guarded_add(self.act_max[i], d, true);
        }
        self.work += k as f64 * 5.0 * mu;
        let k = Self::ag_kill_row(mat, r);
        self.work += k * mu;
    }

    /// Stage C (`37f484` L1431-1925): a 4-term `=` row with zero rhs, P1(x_p1 - x_n1) + P2(x_p2 - x_n2) = 0, whose
    /// columns can be shifted so that min(x_p, x_n) = 0: two columns are merged into the other two.
    fn ag_stage_c(&mut self, mat: &mut Mat, rlist: &[i32], pmin: f64, t: &ColTols) {
        let mut failed = Vec::new();
        for &r in rlist {
            let r = r as usize;
            if !(self.row_cnt[r] == 4 && self.sense[r] == b'=' && self.rhs[r].abs() <= 1e-10) {
                continue;
            }
            let mut pos: Vec<(usize, f64)> = Vec::new();
            let mut neg: Vec<(usize, f64)> = Vec::new();
            let mut fail = false;
            let mut p = mat.rh[r];
            let mut visited = 0;
            while p != NIL {
                let q = mat.nd[p as usize];
                p = q.nr;
                if q.col >= 0 {
                    let k = q.col as usize;
                    if self.col_flag[k] != 0 || 1e-10 < self.lb[k].abs() || self.ub[k] - self.lb[k] < 1e-10 {
                        fail = true;
                        break;
                    }
                    let side = if q.val <= 0.0 { &mut neg } else { &mut pos };
                    if side.len() == 2 {
                        fail = true;
                        break;
                    }
                    side.push((k, q.val));
                }
                visited += 1;
            }
            self.work += visited as f64 * 4.0 * self.work_mult;
            let mut committed = false;
            if !fail && pos.len() == 2 && neg.len() == 2 {
                let ((p1, pv1), (p2, pv2)) = (pos[0], pos[1]);
                let ((mut n1, mut nv1), (mut n2, mut nv2)) = (neg[0], neg[1]);
                let close = |x: f64| x.abs() <= 1e-10;
                if ![p1, p2, n1, n2].iter().all(|&k| self.vtype[k] == b'B')
                    && (close(nv1 + pv1) || close(pv1 + nv2)) && (close(nv1 + pv1) || close(nv1 + pv2)) {
                    let mut paired = close(nv1 + pv1) && self.ag_pairable(mat, p1, n1);
                    if !paired && close(pv1 + nv2) && self.ag_pairable(mat, p1, n2) {
                        std::mem::swap(&mut n1, &mut n2);
                        std::mem::swap(&mut nv1, &mut nv2);
                        paired = true;
                    }
                    if paired && close(pv2 + nv2) && self.ag_pairable(mat, p2, n2) {
                        committed = true;
                        self.ag_pair(mat, r, [p1, p2, n1, n2], [pv1, pv2, nv1, nv2], pmin, t);
                    }
                }
            }
            if !committed {
                let mut p = mat.rh[r];
                let mut k = 0.0;
                while p != NIL {
                    let q = mat.nd[p as usize];
                    p = q.nr;
                    if q.col >= 0 && self.col_flag[q.col as usize] == 0 {
                        failed.push(q.col as usize);
                        self.col_flag[q.col as usize] = 0x1000;
                    }
                    k += 1.0;
                }
                self.work += (k + k) * self.work_mult;
            }
        }
        let nf = failed.len() as f64;
        self.work += (nf + nf) * self.work_mult;
        for j in failed {
            self.col_flag[j] = 0;
        }
    }

    /// `395c97`: can columns p and n be shifted together (rows only allow it if p + n stays put on `=` rows and does
    /// not go down on `<` rows, and rows with only one of them do not mind it going up)?
    fn ag_pairable(&mut self, mat: &mut Mat, pc: usize, nc: usize) -> bool {
        if (self.vtype[pc] == b'C') != (self.vtype[nc] == b'C') || self.obj[pc] + self.obj[nc] < -1e-10 {
            return false;
        }
        let mu = self.work_mult;
        let mut x = mat.ch[nc];
        let mut k = 0;
        while x != NIL {
            let q = mat.nd[x as usize];
            if q.col >= 0 {
                mat.scat[q.row as usize] = x;
            }
            x = q.nc;
            k += 1;
        }
        self.work += mu * 3.0 * k as f64;
        let mut ok = true;
        let mut x = mat.ch[pc];
        let mut k = 0;
        while x != NIL {
            let q = mat.nd[x as usize];
            x = q.nc;
            k += 1;
            if q.col < 0 {
                continue;
            }
            let i = q.row as usize;
            let e = mat.scat[i];
            let eq = self.sense[i] == b'=';
            if e == NIL {
                if eq || q.val < 0.0 {
                    ok = false;
                    k -= 1;
                    break;
                }
            } else {
                let s = q.val + mat.nd[e as usize].val;
                if (eq && 1e-10 < s.abs()) || (!eq && s < -1e-10) {
                    ok = false;
                    k -= 1;
                    break;
                }
                mat.scat[i] = NIL;
            }
        }
        self.work += mu * 4.0 * k as f64;
        // rows of n left in the scatter (no p entry), and cleanup
        let mut x = mat.ch[nc];
        let mut k = 0;
        while x != NIL {
            let q = mat.nd[x as usize];
            x = q.nc;
            k += 1;
            if q.col < 0 {
                continue;
            }
            let i = q.row as usize;
            if mat.scat[i] != NIL {
                if ok && (self.sense[i] == b'=' || q.val < 0.0) {
                    ok = false;
                }
                mat.scat[i] = NIL;
            }
        }
        self.work += mu * 3.0 * k as f64;
        ok
    }

    /// Stage C: MIP `39d14c` emits two one-term type-2 records; then fold both columns.
    fn ag_pair(&mut self, mat: &mut Mat, r: usize, c: [usize; 4], v: [f64; 4], pmin: f64, t: &ColTols) {
        let [p1, p2, n1, n2] = c;
        let [pv1, pv2, _, _] = v;
        let rho = pv2 / pv1;
        if 1e6 < pv1 || pv1 < pmin || 1e6 < pv2 || pv2 < pmin || 1e6 < rho || rho < pmin {
            return;
        }
        let integer = c.iter().any(|&k| self.vtype[k] != b'C');
        if integer && 1e-10 < (rho - 1.0).abs() {
            return;
        }
        let (e1, k1, f1) = if self.ub[n2] * rho < self.ub[p1] { (n2, p1, 1.0 / rho) } else { (p1, n2, rho) };
        let (e2, k2, f2) = if self.ub[p2] * rho <= self.ub[n1] { (p2, n1, 1.0 / rho) } else { (n1, p2, rho) };
        let lim = t.agg_mult_limit;
        let inv = 1.0 / lim;
        let grow_ok = |k: usize, e: usize, f: f64, st: &State| {
            inv <= st.mult_lo[k] && st.mult_hi[k] <= lim && inv <= f * st.mult_lo[e] && f * st.mult_hi[e] <= lim
        };
        if !(grow_ok(k1, e1, f1, self) && grow_ok(k2, e2, f2, self)) {
            return;
        }
        let need = (self.col_cnt[e1] + self.col_cnt[e2]) as i64;
        if mat.nfree < need { self.ag_gc(mat); }
        if mat.nfree < need { return; }
        for (e, k, f) in [(e1, k1, f1), (e2, k2, f2)] {
            self.cols_removed += 1;
            self.rows_removed += 1;
            // The original constructs a one-node synthetic row with value -1, pivot 1/f.
            let pivot = 1.0 / f;
            let value = 1.0 / pivot;
            let growth = (-1.0 / pivot).abs();
            let lo = self.mult_lo[e] * growth;
            if lo <= self.mult_lo[k] { self.mult_lo[k] = lo; }
            let hi = self.mult_hi[e] * growth;
            if self.mult_hi[k] <= hi { self.mult_hi[k] = hi; }
            self.post.push(PostRec::Subst { j: e as i32, c0: 0.0, ind: vec![k as i32], val: vec![value] });
            self.work = 5.0 * self.work_mult + self.work;
        }
        self.rows_removed -= 1;
        self.ag_mark_row(mat, r);
        self.ag_mark_col(mat, e1);
        self.ag_mark_col(mat, e2);
        for (e, k, f) in [(e1, k1, f1), (e2, k2, f2)] {
            if integer {
                if self.vtype[e] == b'B' || self.vtype[k] == b'C' { self.vtype[k] = self.vtype[e]; }
                self.ub[k] = (self.ub[e] + self.intfeastol).floor();
            } else {
                self.ub[k] = if t.infbound <= self.ub[e] { t.infbound } else { self.ub[e] / f };
            }
        }
        let walked = Self::ag_kill_row(mat, r);
        self.work = walked * self.work_mult + self.work;
        self.row_cnt[r] = -3;
        self.obj[k1] = f1 * self.obj[e1] + self.obj[k1];
        self.obj[k2] = f2 * self.obj[e2] + self.obj[k2];
        self.obj[e1] = 0.0;
        self.obj[e2] = 0.0;
        self.col_cnt[k1] -= 1;
        self.col_cnt[k2] -= 1;
        self.col_cnt[e1] = -3 - r as i32;
        self.col_cnt[e2] = -3 - r as i32;
        self.ag_fold(mat, e1, k1, f1);
        self.ag_fold(mat, e2, k2, f2);
    }

    /// Fold column e into k without activity updates; release e to pending and compact k.
    fn ag_fold(&mut self, mat: &mut Mat, e: usize, k: usize, f: f64) {
        let mu = self.work_mult;
        let mut x = mat.ch[k];
        let mut c = 0;
        while x != NIL {
            let q = mat.nd[x as usize];
            if q.col >= 0 {
                mat.scat[q.row as usize] = x;
            }
            x = q.nc;
            c += 1;
        }
        self.work += c as f64 * 3.0 * mu;
        let mut none = NIL;
        let mut y = mat.ch[e];
        let mut c = 0;
        while y != NIL {
            let q = mat.nd[y as usize];
            y = q.nc;
            c += 1;
            if q.col < 0 {
                continue;
            }
            let i = q.row as usize;
            let v = f * q.val;
            let s = mat.scat[i];
            self.ag_upd(mat, s, i, k, v, false, &mut none, 0.0);
            if s != NIL {
                mat.scat[i] = NIL;
            }
        }
        self.work += c as f64 * 10.0 * mu;
        let mut x = mat.ch[k];
        let mut c = 0;
        while x != NIL {
            let q = mat.nd[x as usize];
            if q.col >= 0 {
                mat.scat[q.row as usize] = NIL;
            }
            x = q.nc;
            c += 1;
        }
        self.work += c as f64 * 3.0 * mu;
        self.ag_release_col(mat, e);
        self.ag_compact_col(mat, k);
    }

    /// `36f8e0` + head reset: every node of column e goes to pending; live ones leave their row's count.
    fn ag_release_col(&mut self, mat: &mut Mat, e: usize) {
        let mut p = mat.ch[e];
        let mut k = 0;
        while p != NIL {
            k += 1;
            let q = mat.nd[p as usize];
            if q.col >= 0 {
                self.row_cnt[q.row as usize] -= 1;
            }
            mat.npend += 1;
            self.nnz -= 1;
            mat.nd[p as usize].nc = mat.pend;
            mat.nd[p as usize].col = -1;
            mat.pend = p;
            p = q.nc;
        }
        mat.ch[e] = NIL;
        self.work += k as f64 * 3.0 * self.work_mult;
    }


}
