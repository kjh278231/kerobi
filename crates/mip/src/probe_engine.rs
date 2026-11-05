//! Probing engine `4e48f7(st, ctx, cand, ncand, W)` and its helpers (docs/m2_red_probe.md §3.4): candidates scored
//! and sorted (`4e28ef`, first call only), each binary tried at 0 and 1 with propagation (`4eba40` column step,
//! `4ed528` row step, `4eb08d` queue, IntSet `ac3c8b` family), then common implications, aggregations, implication
//! groups and redundant-row records written into the context. Reads the state (objective, cutoff, counts, scale
//! ranges) and the conflict graph; writes only the context, the candidate order and the work.
// `x = a + x` keeps the original's operand order in sight; col_step mirrors 4eba40's argument list.
#![allow(clippy::assign_op_pattern, clippy::too_many_arguments)]
use crate::conflict::ConflictGraph;
use crate::probe::{gdn, gup, sort_desc, Ctx};
use crate::state::State;
use crate::Params;

const BIG: f64 = 2.5822498780869086e120;
/// one ulp below 1e-4 (0x3f1a36e2eb1c432c)
const TOL4: f64 = 9.999999999999999e-05;

/// `ac3c8b` integer set over [lo, hi): LIFO items, position −1 absent, −2 blocked.
struct IntSet {
    lo: i32,
    items: Vec<i32>,
    pos: Vec<i32>,
}

impl IntSet {
    fn new(lo: i32, hi: i32, w: &mut f64) -> IntSet {
        *w += (hi - lo) as f64;
        IntSet { lo, items: Vec::new(), pos: vec![-1; (hi - lo) as usize] }
    }
    fn at(&mut self, x: i32) -> &mut i32 {
        &mut self.pos[(x - self.lo) as usize]
    }
    /// `ac3f7f`
    fn push(&mut self, x: i32) {
        if *self.at(x) == -1 {
            let k = self.items.len() as i32;
            *self.at(x) = k;
            self.items.push(x);
        }
    }
    /// `ac4062`
    fn pop(&mut self) -> i32 {
        match self.items.pop() {
            Some(x) => {
                *self.at(x) = -1;
                x
            }
            None => -2_000_000_000,
        }
    }
    fn cnt(&self) -> usize {
        self.items.len()
    }
    /// `ac426e`: work += 2·popped (no μ)
    fn clear(&mut self, w: &mut f64) {
        let k = self.items.len();
        while self.cnt() > 0 {
            self.pop();
        }
        *w += k as f64 + k as f64;
    }
    /// `ac4326`
    fn has(&self, x: i32) -> bool {
        self.pos[(x - self.lo) as usize] >= 0
    }
    /// `ac40d4` / `ac4168`
    fn block(&mut self, x: i32) {
        if *self.at(x) == -1 {
            *self.at(x) = -2;
        }
    }
    fn unblock(&mut self, x: i32) {
        if *self.at(x) == -2 {
            *self.at(x) = -1;
        }
    }
}

/// `a7b54f` (crates/lp/src/common.rs `Rng`)
struct Rng(u64);
impl Rng {
    fn f64(&mut self) -> f64 {
        self.0 = self.0.wrapping_mul(0x5851f42d4c957f2d).wrapping_add(0x14057b7ef767814f);
        let mut y = (self.0 >> 32) as u32;
        y ^= y >> 11;
        y ^= (y << 7) & 0x9d2c5680;
        y ^= (y << 15) & 0xefc60000;
        y ^= y >> 18;
        (y & 0x7fff_ffff) as i32 as f64 * 4.656612873077393e-10
    }
}

/// One scenario's scratch: bounds slot, changed columns (mark, list), touched rows (shared marks), activity copies.
struct Scn {
    slot: usize,
    rl: Vec<usize>,
}

struct Eng<'a> {
    c: &'a mut Ctx,
    st: &'a State,
    g: &'a ConflictGraph,
    /// FeasibilityTol, GURO_PAR_AGGMULTLIMIT (env+0x26c0)
    ftol: f64,
    aggm: f64,
    w: f64,
    t0: f64,
    a: IntSet,
    b: IntSet,
    lits: IntSet,
    /// two scenario slots: bounds, column marks and lists
    lbs: [Vec<f64>; 2],
    ubs: [Vec<f64>; 2],
    cm: [Vec<u8>; 2],
    cl: [Vec<usize>; 2],
    rm: Vec<u8>,
    amin: Vec<f64>,
    amax: Vec<f64>,
}

/// What ended the engine early (none at defaults on the 38 models).
#[derive(Debug, PartialEq)]
pub enum EngineEnd {
    Done,
    /// both scenarios of a candidate failed: infeasible (ctx lb[x] = 1 > ub[x] = 0 marks it)
    Infeasible(usize),
    /// the 1e7-work check found the presolve budget spent (`3ba9de` would set PRELIMIT −1) or PRELIMIT < 0
    Budget,
}

impl<'a> Eng<'a> {
    fn elapsed(&self) -> f64 {
        self.t0 + self.w
    }

    /// `4e7515`: min activity of row r after a column moved from `old` to `new` (coefficient part c > 0).
    fn upd_min(&mut self, cf: f64, old: f64, new: f64, slot: Option<usize>, r: usize, global: bool) {
        let mu = self.c.mu;
        let a = if global { &mut self.c.amin } else { &mut self.amin };
        if old <= -1e30 {
            let v = BIG + a[r];
            a[r] = v;
            if v != -BIG {
                return;
            }
            let (l, u) = bounds(self.c, &self.lbs, &self.ubs, slot);
            let mut acc = -self.c.rhs[r];
            let (b, e) = (self.c.rbeg[r] as usize, self.c.rbeg[r + 1] as usize);
            for t in b..e {
                let (v, j) = (self.c.rval[t], self.c.rcol[t] as usize);
                acc = gdn(acc, if v <= 0.0 { u[j] * v } else { l[j] * v });
            }
            self.w += mu * 3.0 * (e - b) as f64;
            let a = if global { &mut self.c.amin } else { &mut self.amin };
            a[r] = acc;
            return;
        }
        if a[r] <= -BIG {
            return;
        }
        let d = new - old;
        if 1e10 <= d {
            let s = self.a7d0ab(1.0, r, slot);
            let a = if global { &mut self.c.amin } else { &mut self.amin };
            a[r] = s - self.c.rhs[r];
            return;
        }
        a[r] = gdn(a[r], cf * d);
    }

    /// `4e7b3e`: max activity counterpart.
    fn upd_max(&mut self, cf: f64, old: f64, new: f64, slot: Option<usize>, r: usize, global: bool) {
        let mu = self.c.mu;
        let a = if global { &mut self.c.amax } else { &mut self.amax };
        if old >= 1e30 {
            let v = a[r] - BIG;
            a[r] = v;
            if v != BIG {
                return;
            }
            let (l, u) = bounds(self.c, &self.lbs, &self.ubs, slot);
            let mut acc = -self.c.rhs[r];
            let (b, e) = (self.c.rbeg[r] as usize, self.c.rbeg[r + 1] as usize);
            for t in b..e {
                let (v, j) = (self.c.rval[t], self.c.rcol[t] as usize);
                acc = gup(acc, if 0.0 <= v { u[j] * v } else { l[j] * v });
            }
            self.w += mu * 3.0 * (e - b) as f64;
            let a = if global { &mut self.c.amax } else { &mut self.amax };
            a[r] = acc;
            return;
        }
        if BIG <= a[r] {
            return;
        }
        let d = old - new;
        if 1e10 <= d {
            let s = self.a7d0ab(-1.0, r, slot);
            let a = if global { &mut self.c.amax } else { &mut self.amax };
            a[r] = -self.c.rhs[r] - s;
            return;
        }
        a[r] = gup(a[r], -(cf * d));
    }

    /// `a7d0ab`: min of sg·(row r) over the bounds (−1e100 when unbounded); work 3 per term, no μ.
    fn a7d0ab(&mut self, sg: f64, r: usize, slot: Option<usize>) -> f64 {
        let (l, u) = bounds(self.c, &self.lbs, &self.ubs, slot);
        let mut acc = 0.0;
        let mut cnt = 0;
        for t in self.c.rbeg[r] as usize..self.c.rbeg[r + 1] as usize {
            let (cf, j) = (self.c.rval[t] * sg, self.c.rcol[t] as usize);
            let b = if cf <= 0.0 { u[j] } else { l[j] };
            if (cf <= 0.0 && !(b < 1e30)) || (cf > 0.0 && b <= -1e30) {
                acc = -1e100;
                break;
            }
            acc = gdn(acc, cf * b);
            cnt += 1;
        }
        self.w += cnt as f64 * 3.0;
        acc
    }

    /// `4eba40(s, old, new, …, depth, flag, j, …)`: column j's bound (already written into the slot) moved;
    /// update its rows' activities (last entry first), test them, queue rows to propagate; then follow the conflict
    /// graph from the new value of a binary (depth −1: collect literals in C, ≥ 0: apply, up to 1000 levels).
    fn col_step(&mut self, s: f64, old: f64, new: f64, depth: i32, flag: bool, j: usize, sc: &mut Scn) -> bool {
        let k_ = sc.slot;
        if self.cm[k_][j] == 0 {
            self.cl[k_].push(j);
            self.cm[k_][j] = 1;
        }
        let beg = self.c.cbeg[j] as usize;
        let ln = self.c.cbeg[j + 1] as usize - beg;
        let (so, sn) = (s * old, s * new);
        let mut k = ln as i64 - 1;
        let mut ret = false;
        while k >= 0 {
            let q = beg + k as usize;
            let cf = self.c.cval[q] * s;
            let r = self.c.crow[q] as usize;
            if self.rm[r] & 1 == 0 {
                sc.rl.push(r);
                self.rm[r] |= 1;
            }
            if cf < 0.0 {
                self.upd_min(-cf, -so, -sn, Some(k_), r, false);
                if self.amin[r] > TOL4 {
                    ret = true;
                    break;
                }
                if flag && 1e-06 < self.c.range[r] + self.amin[r] {
                    if self.amin[r] >= -1e-10 { self.a.push(r as i32) } else { self.b.push(r as i32) }
                }
            } else {
                self.upd_max(cf, so, sn, Some(k_), r, false);
                if self.c.sense[r] == b'=' {
                    if self.amax[r] < -TOL4 {
                        ret = true;
                        break;
                    }
                } else if self.c.sense[r] == b'<' && self.amax[r] < -0.001 {
                    self.rm[r] |= 8;
                }
                if flag && self.c.sense[r] == b'=' && self.amax[r] - self.c.range[r] < -1e-06 {
                    let it = -1 - r as i32;
                    if -self.amax[r] >= -1e-10 { self.a.push(it) } else { self.b.push(it) }
                }
            }
            k -= 1;
        }
        self.w += ((ln as i64 - k - 1) as f64 * 6.0 + 5.0) * self.c.mu;
        if ret {
            return true;
        }
        if j >= self.c.n {
            return false;
        }
        // shortcut: SOS (st+0x1f8) and QC (`4e818b`) steps are skipped; both counts are 0 for linear models
        let g = self.g;
        if ((depth + 1) as u32) <= 1000 && self.c.vt[j] == b'B' && !g.ptr.is_empty() {
            let l2 = if new > 0.5 { 2 * j } else { 2 * j + 1 };
            let (s0, s1) = (g.ptr[l2], g.ptr[l2 + 1]);
            let mut e = s0;
            while e < s1 {
                let v = g.adj[e as usize];
                if v >= 0 {
                    let kk = v as usize;
                    let (l, u) = (self.lbs[k_][kk], self.ubs[k_][kk]);
                    if self.st.col_cnt[kk] >= 0 && u > 0.5 {
                        if l > 0.5 {
                            ret = true;
                            break;
                        }
                        if depth == -1 {
                            if self.lits.has(-1 - kk as i32) {
                                ret = true;
                                break;
                            }
                            self.lits.push(kk as i32);
                        } else {
                            self.ubs[k_][kk] = 0.0;
                            if self.col_step(1.0, 1.0, 0.0, depth + 1, flag, kk, sc) {
                                ret = true;
                                break;
                            }
                        }
                    }
                } else {
                    let kk = (-1 - v) as usize;
                    let (l, u) = (self.lbs[k_][kk], self.ubs[k_][kk]);
                    if self.st.col_cnt[kk] >= 0 && l < 0.5 {
                        if u < 0.5 {
                            ret = true;
                            break;
                        }
                        if depth == -1 {
                            if self.lits.has(kk as i32) {
                                ret = true;
                                break;
                            }
                            self.lits.push(-1 - kk as i32);
                        } else {
                            self.lbs[k_][kk] = 1.0;
                            if self.col_step(-1.0, 0.0, 1.0, depth + 1, flag, kk, sc) {
                                ret = true;
                                break;
                            }
                        }
                    }
                }
                e += 1;
            }
            self.w += 3.0 * self.c.mu * (e - s0) as f64;
        }
        ret
    }

    /// `4ed528(s, …, r, …, mode, …)`: one side of row r (s = +1 the ≤ side, −1 the ≥ side of an `=` row): tighten
    /// general columns by the slack (mode 0), fix binaries whose |a| exceeds it (mode 1: collect literals in C).
    fn row_step(&mut self, s: f64, r: usize, mode: bool, sc: &mut Scn) -> bool {
        let mu = self.c.mu;
        let k_ = sc.slot;
        let beg = self.c.rbeg[r] as usize;
        let ln = self.c.rbeg[r + 1] as usize - beg;
        let ng = self.c.rnb[r] as usize - beg;
        let (amin, amax) = (self.amin[r], self.amax[r]);
        let (mut slack, act);
        if s > 0.0 {
            slack = -amin;
            act = amax > 1e-06;
            if !act {
                self.rm[r] |= 2;
            }
        } else {
            slack = amax;
            act = -1e-06 > amin;
            if !act {
                self.rm[r] |= 4;
            }
        }
        self.w += 10.0 * mu;
        if -TOL4 > slack {
            return true;
        }
        if !act {
            return false;
        }
        let mut ret = false;
        if slack == 2.0 * BIG {
            // exactly one infinite term
            if mode {
                return false;
            }
            let (mut sm, mut inf) = (0.0, 0usize);
            for e in 0..ng {
                let (cf, j) = (self.c.rval[beg + e] * s, self.c.rcol[beg + e] as usize);
                let (l, u) = (self.lbs[k_][j], self.ubs[k_][j]);
                if cf > 0.0 {
                    if -1e30 < l { sm = sm + cf * l } else { inf = e }
                } else if u >= 1e30 {
                    inf = e
                } else {
                    sm = sm + cf * u
                }
            }
            for e in ng..ln {
                let (cf, j) = (self.c.rval[beg + e] * s, self.c.rcol[beg + e] as usize);
                sm = sm + if cf > 0.0 { cf * self.lbs[k_][j] } else { cf * self.ubs[k_][j] };
            }
            self.w += 4.0 * mu * ng as f64;
            self.w += 3.0 * mu * (ln - ng) as f64;
            let cf = self.c.rval[beg + inf] * s;
            let rest = self.c.rhs[r] * s - sm;
            let j = self.c.rcol[beg + inf] as usize;
            let (l, u) = (self.lbs[k_][j], self.ubs[k_][j]);
            let mut v = rest / cf;
            if cf > 0.0 {
                let um = u - 1e-06;
                if v < um {
                    if self.c.vt[j] == b'C' {
                        if l > v && v > l - 1e-06 {
                            v = l;
                        }
                    } else {
                        v = (v + self.c.itol).floor();
                    }
                }
                if v < um && (1e30 > u || 1e8 > v) && v > -1e8 {
                    let fl = maxsd(minsd(u.abs(), v.abs()), 0.0001) < u - v;
                    self.ubs[k_][j] = v;
                    ret = self.col_step(1.0, u, v, -2, fl, j, sc);
                }
            } else {
                let lp = 1e-06 + l;
                if v > lp {
                    if self.c.vt[j] == b'C' {
                        if v > u && u + 1e-06 > v {
                            v = u;
                        }
                    } else {
                        v = (v - self.c.itol).ceil();
                    }
                }
                if v > lp && (l > -1e30 || v > -1e8) && 1e8 > v {
                    let fl = maxsd(minsd(l.abs(), v.abs()), 0.0001) < v - l;
                    self.lbs[k_][j] = v;
                    ret = self.col_step(-1.0, l, v, -2, fl, j, sc);
                }
            }
            self.w += 7.0 * mu;
            return ret;
        }
        slack = if slack > 0.0 { slack } else { 0.0 };
        let mut e = 0;
        if !mode {
            while e < ng {
                let (cf, j) = (self.c.rval[beg + e] * s, self.c.rcol[beg + e] as usize);
                let (l, u) = (self.lbs[k_][j], self.ubs[k_][j]);
                let um = u - 1e-06;
                let t = (um - l) * cf.abs();
                if !(t > slack) {
                    if slack >= ((self.c.ub0[j] - 1e-06) - self.c.lb0[j]) * cf.abs() {
                        break;
                    }
                } else if cf > 0.0 {
                    let mut v = slack / cf + l;
                    if um > v {
                        if self.c.vt[j] == b'C' {
                            if l > v && v > l - 1e-06 {
                                v = l;
                            }
                        } else {
                            v = (v + self.c.itol).floor();
                        }
                    }
                    if um > v && (1e30 > u || 1e8 > v) {
                        let fl = (u - v) > (-(l - 1e-06) + u) * 0.01 || (u - v) > maxsd(minsd(u.abs(), v.abs()), 0.0001);
                        self.ubs[k_][j] = v;
                        ret = self.col_step(1.0, u, v, -2, fl, j, sc);
                    }
                } else {
                    let mut v = slack / cf + u;
                    let lp = 1e-06 + l;
                    if v > lp {
                        if self.c.vt[j] == b'C' {
                            if v > u && 1e-06 + u > v {
                                v = u;
                            }
                        } else {
                            v = (v - self.c.itol).ceil();
                        }
                    }
                    if v > lp && (l > -1e30 || v > -1e8) {
                        let fl = (v - l) > (-(l - 1e-06) + u) * 0.01 || (v - l) > maxsd(minsd(l.abs(), v.abs()), 0.0001);
                        self.lbs[k_][j] = v;
                        ret = self.col_step(-1.0, l, v, -2, fl, j, sc);
                    }
                }
                e += 1;
                if e >= ng || ret {
                    break;
                }
            }
            self.w += 7.0 * mu * e as f64;
        }
        let mut e = ng;
        let thr = 1e-06 + slack;
        while e < ln {
            if ret {
                break;
            }
            let cf = self.c.rval[beg + e] * s;
            if thr > cf.abs() {
                break;
            }
            let j = self.c.rcol[beg + e] as usize;
            if self.lbs[k_][j] != self.ubs[k_][j] {
                if cf > 0.0 {
                    if mode {
                        if self.lits.has(-1 - j as i32) {
                            ret = true;
                            break;
                        }
                        self.lits.push(j as i32);
                    } else {
                        self.ubs[k_][j] = 0.0;
                        ret = self.col_step(1.0, 1.0, 0.0, -2, true, j, sc);
                    }
                } else if mode {
                    if self.lits.has(j as i32) {
                        ret = true;
                        break;
                    }
                    self.lits.push(-1 - j as i32);
                } else {
                    self.lbs[k_][j] = 1.0;
                    ret = self.col_step(-1.0, 0.0, 1.0, -2, true, j, sc);
                }
            }
            e += 1;
        }
        self.w += 4.0 * mu * (e - ng) as f64;
        ret
    }

    /// `4eb08d`: propagate queued rows (A first) until both queues are empty, a row fails or the budget is spent;
    /// mode 1 defers rows starting with a general column and requeues them at the end.
    fn queue(&mut self, mode: bool, sc: &mut Scn) -> bool {
        let limit = self.c.fac * self.c.budget;
        let mut dfr = Vec::new();
        let mut st = false;
        if self.a.cnt() > 0 || self.b.cnt() > 0 {
            loop {
                if !(limit > self.elapsed()) {
                    break;
                }
                self.w += self.c.mu * 2.0;
                let item = if self.a.cnt() > 0 { self.a.pop() } else { self.b.pop() };
                let r = if item >= 0 { item } else { -1 - item } as usize;
                self.b.block(item);
                self.a.block(item);
                let mut d = false;
                st = false;
                let side = if item >= 0 { 2 } else { 4 };
                if self.rm[r] & side == 0 {
                    st = self.row_step(if item >= 0 { 1.0 } else { -1.0 }, r, mode, sc);
                    if !st && mode {
                        d = self.c.vt[self.c.rcol[self.c.rbeg[r] as usize] as usize] != b'B';
                    }
                }
                self.b.unblock(item);
                self.a.unblock(item);
                if d {
                    dfr.push(item);
                }
                if self.a.cnt() > 0 || self.b.cnt() > 0 {
                    if st {
                        return st;
                    }
                    continue;
                }
                break;
            }
        }
        if limit > self.elapsed() {
            for item in dfr {
                let r = if item >= 0 { item } else { -1 - item } as usize;
                let tight = if item < 0 { -self.amax[r] >= -1e-10 } else { self.amin[r] >= -1e-10 };
                if tight { self.a.push(item) } else { self.b.push(item) }
            }
        }
        st
    }

    /// `4e28ef`: candidate scores (rows: −(length + random), columns: per row a weight by how many other binaries
    /// can't be 1 with it, times 1 + 1e-4·random), then conflict-graph degrees.
    fn scores(&mut self, cand: &[i32], seed: i32) -> Vec<f64> {
        let mu = self.c.mu;
        let c = &*self.c;
        let mut rng = Rng(seed as i64 as u64);
        let mut mx = 0.0f64;
        let mut cache = vec![0i32; c.nrow0];
        let mut score = vec![0.0f64; cand.len()];
        let mut w = self.w;
        for k in 0..cand.len() {
            let j = cand[k];
            if j as usize >= c.nc {
                let u = rng.f64();
                let rr = j as usize - c.nc;
                score[k] = -(u + (c.rbeg[rr + 1] - c.rbeg[rr]) as f64);
                continue;
            }
            let j = j as usize;
            score[k] = 1.0;
            let mut e = c.cbeg[j] as usize;
            while e < c.cbeg[j + 1] as usize {
                let (a, r) = (c.cval[e], c.crow[e] as usize);
                let wv = if 0.0 > a { 150.0 } else { 100.0 };
                if r < c.nrow0 {
                    let mut cc = cache[r];
                    if cc == 0 {
                        let (b, en) = (c.rbeg[r] as usize, c.rbeg[r + 1] as usize);
                        let am = c.amin[r];
                        let mut dmin = -1.0;
                        let mut valid = true;
                        if en - b < 2 || am > -self.ftol || c.vt[c.rcol[b] as usize] != b'B' {
                            cache[r] = -1;
                            e += 1;
                            continue;
                        }
                        let x0 = a.abs() + am;
                        let mut t = b;
                        let mut n1 = 0;
                        while t < en {
                            if c.rcol[t] as usize != j {
                                let v = c.rval[t].abs();
                                if 0.0 > dmin {
                                    dmin = v;
                                }
                                if v + x0 > 1e-10 {
                                    n1 += 1;
                                } else {
                                    valid = false;
                                    if n1 == 0 && (t == en - 1 || -1e-10 > (am + c.rval[t + 1].abs()) + dmin) {
                                        cache[r] = -1;
                                    }
                                    break;
                                }
                            }
                            t += 1;
                        }
                        w += mu * 6.0 * (t - b) as f64;
                        if valid {
                            cache[r] = n1;
                        }
                        cc = n1;
                    }
                    if cc > 0 {
                        score[k] = if c.sense[r] == b'=' { (wv * wv) / cc as f64 + score[k] } else { wv * (cc as f64).ln() + score[k] };
                    }
                }
                e += 1;
            }
            w += mu * 10.0 * (e - c.cbeg[j] as usize) as f64;
            let u = rng.f64();
            score[k] = (u * 0.0001 + 1.0) * score[k];
            mx = if mx > score[k] { mx } else { score[k] };
        }
        w += cand.len() as f64 * (3.0 * mu);
        let g = self.g;
        if !g.ptr.is_empty() {
            for k in 0..cand.len() {
                let j = cand[k] as usize;
                if j < c.nc {
                    let (s0, s1, s2) = (g.ptr[2 * j], g.ptr[2 * j + 1], g.ptr[2 * j + 2]);
                    let (d1, d2) = (s1 - s0, s2 - s1);
                    if d1 > 0 {
                        score[k] = (d1 as f64 * 100.0 + mx) + score[k];
                    }
                    if d2 > 0 {
                        score[k] = (d2 as f64 * 100.0 + mx) + score[k];
                    }
                }
            }
            w += (3.0 * mu) * cand.len() as f64;
        }
        self.w = w;
        score
    }
}

fn bounds<'b>(c: &'b Ctx, lbs: &'b [Vec<f64>; 2], ubs: &'b [Vec<f64>; 2], slot: Option<usize>) -> (&'b [f64], &'b [f64]) {
    match slot {
        Some(k) => (&lbs[k], &ubs[k]),
        None => (&c.lb, &c.ub),
    }
}

/// `minsd` / `maxsd`: the second operand when the comparison fails (NaN)
fn minsd(a: f64, b: f64) -> f64 {
    if a < b { a } else { b }
}
fn maxsd(a: f64, b: f64) -> f64 {
    if a > b { a } else { b }
}

impl State {
    /// `4e48f7(st, ctx, cand, ncand, W)` on one thread. `cand` is sorted in place on the first call (p7 = 0).
    /// shortcut: the `a83b2f` branch of the 1e7 check (ctx+0xec / no state) and the SOS / QC / GUB steps are not
    /// ported; none runs from Model.presolve() on linear models.
    pub fn probe_engine(&self, p: &Params, g: &ConflictGraph, c: &mut Ctx, cand: &mut [i32], w: &mut f64) -> EngineEnd {
        let (nr, nc, n) = (c.nr as i32, c.nc, c.n);
        let mu = c.mu;
        let mut e = Eng {
            st: self,
            g,
            ftol: p.feastol,
            aggm: p.aggmultlimit,
            w: *w,
            t0: (0.0 - *w) + 1.0,
            a: IntSet { lo: 0, items: Vec::new(), pos: Vec::new() },
            b: IntSet { lo: 0, items: Vec::new(), pos: Vec::new() },
            lits: IntSet { lo: 0, items: Vec::new(), pos: Vec::new() },
            lbs: [c.lb.clone(), c.lb.clone()],
            ubs: [c.ub.clone(), c.ub.clone()],
            cm: [vec![0; nc], vec![0; nc]],
            cl: [Vec::new(), Vec::new()],
            rm: vec![0; c.nr],
            amin: c.amin.clone(),
            amax: c.amax.clone(),
            c,
        };
        e.c.nchg = 0;
        e.c.nbin = 0;
        e.c.neq = 0;
        e.a = IntSet::new(-nr, nr, &mut e.w);
        e.b = IntSet::new(-nr, nr, &mut e.w);
        if e.c.imp {
            e.lits = IntSet::new(-(n as i32), n as i32, &mut e.w);
        }
        if e.c.rec {
            e.c.rhead = vec![-1; e.c.nrow0];
        }
        if !e.c.p7 {
            let mut score = e.scores(cand, 0); // env+0x2cd0 (not in the parameter table) is 0
            sort_desc(&mut score, cand);
        }
        // objective cutoff (st+0x490; 1e100 from Model.presolve(), so `use` is off)
        let mut usec = p.dual_reductions != 0;
        let mut om = self.objcon;
        if self.cutoff < 5e99 && usec {
            let mut cnt = 0;
            let mut ok = true;
            for j in 0..n {
                let cf = self.obj[j];
                if cf > 0.0 {
                    if e.c.lb[j] <= -1e30 {
                        om = -1e100;
                        ok = false;
                        break;
                    }
                    om = gdn(om, cf * e.c.lb[j]);
                    cnt += 1;
                } else if cf < 0.0 {
                    if e.c.ub[j] >= 1e30 {
                        om = -1e100;
                        ok = false;
                        break;
                    }
                    om = gdn(om, cf * e.c.ub[j]);
                    cnt += 1;
                }
            }
            if !(ok && -1e100 < om && cnt != 0) {
                usec = false;
            }
        } else {
            usec = false;
        }
        let tolc = (self.cutoff.abs() + 1.0) * 1e-06;
        let mut last = 0.0;
        let mut used = 0i32;
        let mut i = 0;
        let mut nss = [0usize; 2];
        while i < cand.len() {
            let t = e.elapsed();
            let f = ((e.c.nbin as f64 * 20.0 + 100.0) + e.c.nagg as f64 * 10.0 + e.c.nchg as f64) / (i as f64 + 100.0);
            let f = minsd(f, 100.0);
            let f = maxsd(f, 0.01);
            e.c.fac = f;
            if t > f * e.c.budget {
                e.c.stop = true;
                break;
            }
            if t - last > 1e7 {
                last = t;
                // `3ba5ac` (callback) and `3ba9de` (presolve budget): over budget sets PRELIMIT −1, then stop
                if e.w - self.work_base > self.budget || self.prelimit < 0 {
                    *w = e.w;
                    return EngineEnd::Budget;
                }
            }
            if e.c.imp {
                let g = e.c.gbeg.len() / 2;
                e.c.gbeg.truncate(2 * g);
                e.c.gbeg.extend_from_slice(&[used, used, used]);
            }
            let x = cand[i];
            if x as usize >= nc {
                e.row_probe(x as usize - nc);
                i += 1;
                continue;
            }
            let x = x as usize;
            if !(e.c.lb[x] == 0.0 && e.c.ub[x] == 1.0 && e.c.aggto[x] < 0) {
                i += 1;
                continue;
            }
            // both scenarios
            let mut nf = 0usize;
            for side in 0..2 {
                let mut sc = Scn { slot: nf, rl: Vec::new() };
                e.a.clear(&mut e.w);
                e.b.clear(&mut e.w);
                if e.c.imp {
                    e.lits.clear(&mut e.w);
                }
                let depth = if e.c.imp { -1 } else { 0 };
                let mut ret = if side == 0 {
                    e.ubs[nf][x] = 0.0;
                    e.col_step(1.0, 1.0, 0.0, depth, true, x, &mut sc)
                } else {
                    e.lbs[nf][x] = 1.0;
                    e.col_step(-1.0, 0.0, 1.0, depth, true, x, &mut sc)
                };
                let mut obj_ok = true;
                if !ret {
                    if e.c.imp {
                        ret = e.queue(true, &mut sc);
                        while e.lits.cnt() > 0 {
                            if ret {
                                obj_ok = false;
                                break;
                            }
                            if !(e.c.fac * e.c.budget > e.elapsed()) {
                                break;
                            }
                            let l = e.lits.pop();
                            if l < 0 {
                                let k = (-1 - l) as usize;
                                if e.lbs[nf][k] == 0.0 {
                                    if e.ubs[nf][k] == 0.0 {
                                        ret = true;
                                    } else {
                                        e.lbs[nf][k] = 1.0;
                                        ret = e.col_step(-1.0, 0.0, 1.0, -2, true, k, &mut sc);
                                    }
                                }
                            } else {
                                let k = l as usize;
                                if e.ubs[nf][k] == 1.0 {
                                    if e.lbs[nf][k] == 1.0 {
                                        ret = true;
                                    } else {
                                        e.ubs[nf][k] = 0.0;
                                        ret = e.col_step(1.0, 1.0, 0.0, -2, true, k, &mut sc);
                                    }
                                }
                            }
                        }
                        nss[nf] = e.cl[nf].len();
                        if ret {
                            obj_ok = false;
                        }
                    }
                    if obj_ok {
                        if !(e.c.fac * e.c.budget <= e.elapsed()) {
                            ret = e.queue(false, &mut sc);
                        }
                        if !ret && usec {
                            let mut ob = om;
                            for &j in &e.cl[nf] {
                                if j >= n || self.obj[j] == 0.0 {
                                    continue;
                                }
                                let cf = self.obj[j];
                                if cf > 0.0 {
                                    if e.lbs[nf][j] > e.c.lb[j] {
                                        ob = gdn(ob, (e.lbs[nf][j] - e.c.lb[j]) * cf);
                                    }
                                } else if e.c.ub[j] > e.ubs[nf][j] {
                                    ob = gdn(ob, (e.ubs[nf][j] - e.c.ub[j]) * cf);
                                }
                            }
                            if tolc < ob - self.cutoff {
                                ret = true;
                            }
                        }
                    }
                }
                for &q in &sc.rl {
                    if !ret && e.c.rec && e.rm[q] & 8 != 0 {
                        let gq = e.c.rlit.len() as i32;
                        if gq != 1_000_000 {
                            e.c.rlit.push(if side == 1 { -1 - x as i32 } else { x as i32 });
                            e.c.rslack.push(-e.amax[q]);
                            e.c.rnext.push(e.c.rhead[q]);
                            e.c.rhead[q] = gq;
                        }
                    }
                    e.rm[q] = 0;
                    e.amin[q] = e.c.amin[q];
                    e.amax[q] = e.c.amax[q];
                }
                e.w += sc.rl.len() as f64 * 6.0 * mu;
                if !ret {
                    nf += 1;
                } else {
                    for &j in &e.cl[nf] {
                        e.lbs[nf][j] = e.c.lb[j];
                        e.ubs[nf][j] = e.c.ub[j];
                        e.cm[nf][j] = 0;
                    }
                    e.w += e.cl[nf].len() as f64 * 6.0 * mu;
                    e.cl[nf].clear();
                    nss[nf] = 0;
                }
            }
            if nf == 0 {
                e.c.ub[x] = 0.0;
                e.c.lb[x] = 1.0;
                *w = e.w;
                return EngineEnd::Infeasible(x);
            }
            e.common(x, nf, &mut nss, &mut used);
            i += 1;
        }
        e.w += i as f64 * 12.0 * mu;
        *w = e.w;
        EngineEnd::Done
    }
}

impl<'a> Eng<'a> {
    /// Row candidate rr: put every column at the bound giving the row's min activity; if that already fails or
    /// propagation fails, the row must be tight (`=`).
    fn row_probe(&mut self, rr: usize) {
        let mu = self.c.mu;
        let mut sc = Scn { slot: 0, rl: Vec::new() };
        self.a.clear(&mut self.w);
        self.b.clear(&mut self.w);
        if self.c.imp {
            self.lits.clear(&mut self.w);
        }
        let (b0, e1) = (self.c.rbeg[rr] as usize, self.c.rbeg[rr + 1] as usize);
        let mut e = b0;
        let mut ret = false;
        let mut cntf = 0;
        while e < e1 {
            let j = self.c.rcol[e] as usize;
            let (u, l) = (self.ubs[0][j], self.lbs[0][j]);
            if !(0.5 > u - l) {
                if self.c.rval[e] > 0.0 {
                    self.ubs[0][j] = l;
                    ret = self.col_step(1.0, u, l, 0, true, j, &mut sc);
                } else {
                    self.lbs[0][j] = u;
                    ret = self.col_step(-1.0, l, u, 0, true, j, &mut sc);
                }
                cntf += 1;
            }
            e += 1;
            if ret {
                break;
            }
        }
        self.w += (e - b0) as f64 * 4.0 * mu;
        if ret || (cntf >= 2 && self.queue(false, &mut sc)) {
            self.c.sense[rr] = b'=';
            self.c.neq += 1;
        }
        for &q in &sc.rl {
            self.rm[q] = 0;
            self.amin[q] = self.c.amin[q];
            self.amax[q] = self.c.amax[q];
        }
        self.w += sc.rl.len() as f64 * 6.0 * mu;
        for &j in &self.cl[0] {
            self.lbs[0][j] = self.c.lb[j];
            self.ubs[0][j] = self.c.ub[j];
            self.cm[0][j] = 0;
        }
        self.w += self.cl[0].len() as f64 * 6.0 * mu;
        self.cl[0].clear();
    }

    /// After both scenarios of x: bounds common to the feasible ones become global (activities updated), columns
    /// on a line with x are aggregated, new binary implications grouped, slots reset.
    fn common(&mut self, x: usize, nf: usize, nss: &mut [usize; 2], used: &mut i32) {
        let mu = self.c.mu;
        let n = self.c.n;
        if nf == 2 {
            self.c.cnt0[x] = self.cl[0].len() as i32;
            self.c.cnt1[x] = self.cl[1].len() as i32;
        }
        let pick = if nf > 1 && self.cl[1].len() < self.cl[0].len() { 1 } else { 0 };
        let lst = self.cl[pick].clone();
        let mut rl: Vec<usize> = Vec::new();
        let aggm = self.aggm;
        for &j in &lst {
            let (mut lo, mut hi) = (1e30f64, -1e30f64);
            for s in 0..nf {
                lo = minsd(lo, self.lbs[s][j]);
                hi = maxsd(hi, self.ubs[s][j]);
            }
            self.w += (nf as f64 + nf as f64) * mu;
            let (mut l, u) = (self.c.lb[j], self.c.ub[j]);
            let oldl = l;
            let (mut lch, mut uch) = (false, false);
            if lo > 1e-06 + l {
                if lo > u && 1e-06 + u > lo {
                    lo = u;
                }
                lch = true;
                self.c.lb[j] = lo;
                self.lbs[1][j] = lo;
                self.lbs[0][j] = lo;
                if j < n {
                    self.c.nchg += 1;
                    if self.c.vt[j] == b'B' {
                        self.c.nbin += 1;
                    }
                }
                l = lo;
            }
            if hi < u - 1e-06 {
                if hi < l && l - 1e-06 < hi {
                    hi = l;
                }
                uch = true;
                self.c.ub[j] = hi;
                self.ubs[1][j] = hi;
                self.ubs[0][j] = hi;
                if j < n {
                    self.c.nchg += 1;
                    if self.c.vt[j] == b'B' {
                        self.c.nbin += 1;
                    }
                }
            }
            if lch || uch {
                let (beg, end) = (self.c.cbeg[j] as usize, self.c.cbeg[j + 1] as usize);
                for q in beg..end {
                    let (r, a) = (self.c.crow[q] as usize, self.c.cval[q]);
                    if self.rm[r] & 1 == 0 {
                        rl.push(r);
                        self.rm[r] |= 1;
                    }
                    if r < self.c.nrow0 {
                        if lch {
                            if a > 0.0 { self.upd_min(a, oldl, lo, None, r, true) } else { self.upd_max(-a, -oldl, -lo, None, r, true) }
                        }
                        if uch {
                            if a > 0.0 { self.upd_max(a, u, hi, None, r, true) } else { self.upd_min(-a, -u, -hi, None, r, true) }
                        }
                    }
                }
                self.w += (end - beg) as f64 * 6.0 * mu;
            }
            // aggregation: j on a line with x
            if nf > 1 && j != x && j < n && x < n && self.c.aggto[j] < 0
                && aggm >= self.st.mult_hi[x] && self.st.mult_lo[x] >= 1.0 / aggm
            {
                let (l, u) = (self.c.lb[j], self.c.ub[j]);
                let dlu = l - u;
                let wd = -dlu;
                if aggm >= wd * self.st.mult_hi[j] && wd * self.st.mult_lo[j] >= 1.0 / aggm {
                    if self.lbs[0][j] > u - 1e-10 && 1e-10 + l > self.ubs[1][j] {
                        self.c.aggto[j] = x as i32;
                        if 1e-12 >= u.abs() {
                            self.c.aggmul[j] = l;
                            self.c.aggoff[j] = 0.0;
                        } else {
                            self.c.aggmul[j] = dlu;
                            self.c.aggoff[j] = u;
                        }
                        self.c.nagg += 1;
                    } else if 1e-10 + l > self.ubs[0][j] && self.lbs[1][j] > u - 1e-10 {
                        self.c.aggto[j] = x as i32;
                        if 1e-12 >= l.abs() {
                            self.c.aggmul[j] = u;
                            self.c.aggoff[j] = 0.0;
                        } else {
                            self.c.aggmul[j] = wd;
                            self.c.aggoff[j] = l;
                        }
                        self.c.nagg += 1;
                    }
                }
            }
        }
        self.w += lst.len() as f64 * 10.0 * mu;
        for &q in &rl {
            self.rm[q] = 0;
            self.amin[q] = self.c.amin[q];
            self.amax[q] = self.c.amax[q];
        }
        self.w += rl.len() as f64 * 6.0 * mu;
        if nf > 1 && self.c.imp {
            let need = (((self.cl[0].len() - nss[0]) + self.cl[1].len()) - nss[1]) as i64 + *used as i64;
            let mut ok = true;
            if (self.c.cap as i64) < need {
                if (self.c.lim as i64) < need || (self.c.lim as f64) < self.c.cap as f64 * 1.5 {
                    self.c.imp = false;
                    ok = false;
                } else {
                    let grown = (self.c.cap as f64 * 1.5) as i64;
                    self.c.cap = need.max(grown) as i32;
                }
            }
            if ok {
                let mut iv = *used;
                let g = self.c.gbeg.len() / 2; // groups so far (this candidate's included)
                for (s0, s1) in [(0usize, 1usize), (1, 0)] {
                    let st0 = nss[s0];
                    for q in nss[s0]..self.cl[s0].len() {
                        let j = self.cl[s0][q];
                        if j < n && self.c.vt[j] == b'B' && self.c.aggto[j] < 0 {
                            if self.lbs[s0][j] == 1.0 && self.lbs[s1][j] == 0.0 {
                                self.c.imps.push(-1 - j as i32);
                                iv += 1;
                            } else if self.ubs[s0][j] == 0.0 && self.ubs[s1][j] == 1.0 {
                                self.c.imps.push(j as i32);
                                iv += 1;
                            }
                        }
                    }
                    self.w += ((self.cl[s0].len() - st0) as f64 * 6.0 + 1.0) * mu;
                    let at = if s0 == 0 { 2 * g - 1 } else { 2 * g };
                    self.c.gbeg[at] = iv;
                }
                *used = iv;
            }
        }
        for s in 0..nf {
            for q in 0..self.cl[s].len() {
                let j = self.cl[s][q];
                self.lbs[s][j] = self.c.lb[j];
                self.ubs[s][j] = self.c.ub[j];
                self.cm[s][j] = 0;
            }
            self.w += self.cl[s].len() as f64 * 6.0 * mu;
            self.cl[s].clear();
            nss[s] = 0;
        }
        self.w += nf as f64 * mu;
    }
}
