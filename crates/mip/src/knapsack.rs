//! `578fd2` knapsack row strengthening (docs/m2_red_outer_a.md §5): each `<` row of the level-1 list goes through
//! `6aec61` (the knapsack routine shared with the cut code; mode 2, PREPOWER), which may tighten coefficients and
//! rhs, round with Chvátal–Gomory multipliers (`6b33dd`) and drop columns. Its work goes to W unscaled.
use crate::helpers::minsd;
use crate::iint::gcd_f64;
use crate::state::{State, NIL};
use crate::Params;

/// Cached activity not computed (`6aec61`'s 1e101 marker).
const STALE: f64 = 1e101;

/// `6aec61`'s row and the locals its helpers share.
struct Kn<'a> {
    lb: &'a [f64],
    ub: &'a [f64],
    vt: &'a [u8],
    ft: f64,
    w: &'a mut f64,
    ind: Vec<i32>,
    val: Vec<f64>,
    rhs: f64,
    /// local_50 / local_48: min and max activity (STALE when not known)
    minact: f64,
    maxact: f64,
    /// local_38: 1 = has a continuous column, 0 = none, -1 = not known
    cont: i32,
    /// local_ac: the last step changed the row
    ac: bool,
}

impl State {
    /// `578fd2(st, env)`. Returns a status (0).
    pub fn knapsack(&mut self, p: &Params) -> i32 {
        let mu = self.work_mult;
        let rows = self.rows_list();
        let mut chg: Vec<i32> = Vec::new();
        let mut tmp = vec![0.0f64; self.n];
        for &ii in &rows {
            let i = ii as usize;
            if !(0 < self.row_cnt[i] && self.sense[i] != b'=') {
                continue;
            }
            let (mut ind, mut val) = (Vec::new(), Vec::new());
            let mut q = self.row_head[i];
            while q != NIL {
                let x = self.nd[q as usize];
                if x.col >= 0 {
                    ind.push(x.col);
                    val.push(x.val);
                }
                q = x.nr;
            }
            self.work += mu * 5.0 * self.row_cnt[i] as f64;
            let mut kn = Kn {
                lb: &self.lb,
                ub: &self.ub,
                vt: &self.vtype,
                ft: p.feastol,
                w: &mut self.work,
                ind,
                val,
                rhs: self.rhs[i],
                minact: STALE,
                maxact: STALE,
                cont: -1,
                ac: false,
            };
            let mut changed = false;
            let rc = kn.lift(p.prepower, p.preknapliftlen, &mut changed);
            let (ind, val, rhs) = (kn.ind, kn.val, kn.rhs);
            self.rhs[i] = rhs;
            if rc != 0 {
                return rc;
            }
            if !changed {
                continue;
            }
            chg.push(ii);
            self.tracker.mark_row(ii);
            for (&j, &a) in ind.iter().zip(&val) {
                tmp[j as usize] = a;
            }
            self.work += mu * 3.0 * ind.len() as f64;
            self.row_cnt[i] = ind.len() as i32;
            let mut q = self.row_head[i];
            while q != NIL {
                let x = self.nd[q as usize];
                if x.col >= 0 {
                    let c = x.col as usize;
                    let nv = tmp[c];
                    tmp[c] = 0.0;
                    if 1e-10 < (x.val - nv).abs() {
                        self.tracker.mark_col(x.col);
                    }
                    if nv == 0.0 {
                        self.col_cnt[c] -= 1;
                        self.nd[q as usize].col = -1;
                    } else {
                        self.nd[q as usize].val = nv;
                    }
                }
                q = x.nr;
            }
            self.work += mu * 4.0 * self.row_cnt[i] as f64;
        }
        self.work += rows.len() as f64 * 6.0 * mu;
        if !chg.is_empty() {
            self.coefs_changed += chg.len() as i32;
            self.activity(Some(&chg));
        }
        0
    }
}

impl Kn<'_> {
    fn act(&mut self, s: f64) -> f64 {
        act_sum(&self.ind, &self.val, self.lb, self.ub, s, self.w)
    }

    fn max_act(&mut self) -> f64 {
        if self.maxact == STALE {
            self.maxact = -self.act(-1.0);
        }
        self.maxact
    }

    fn min_act(&mut self) -> f64 {
        if self.minact == STALE {
            self.minact = self.act(1.0);
        }
        self.minact
    }

    fn is_c(&self, t: usize) -> bool {
        self.vt[self.ind[t] as usize] == b'C'
    }

    /// Fixed columns (range < 1e-10) and negligible ones (range·|a|·len < ft/100) go into the rhs at the bound
    /// that minimizes their term, zero coefficients go. W += 4 per entry. Returns whether any went.
    fn drop_fixed(&mut self) -> bool {
        let n = self.ind.len();
        let nf = n as f64;
        let (mut k, mut any) = (0, false);
        for t in 0..n {
            let j = self.ind[t] as usize;
            let a = self.val[t];
            let (u, l) = (self.ub[j], self.lb[j]);
            let d = u - l;
            if d < 1e-10 || !(self.ft * 0.01 <= d * a.abs() * nf) {
                any = true;
                self.rhs = self.rhs - a * if 0.0 < a { l } else { u };
            } else if a.abs() < 1e-10 {
                any = true;
            } else {
                self.ind[k] = j as i32;
                self.val[k] = a;
                k += 1;
            }
        }
        *self.w += n as f64 * 4.0;
        self.ind.truncate(k);
        self.val.truncate(k);
        any
    }

    /// Zero coefficients (|a| <= 1e-10) go. W += 3 per entry.
    fn compact(&mut self) {
        let n = self.ind.len();
        let mut k = 0;
        for t in 0..n {
            if 1e-10 < self.val[t].abs() {
                self.ind[k] = self.ind[t];
                self.val[k] = self.val[t];
                k += 1;
            }
        }
        *self.w += n as f64 * 3.0;
        self.ind.truncate(k);
        self.val.truncate(k);
    }

    /// rhs = c + Σ a·(a <= 0 ? lo_neg : hi_pos) with `pos_ub` choosing the bound for positive coefficients.
    /// W += 3 per entry.
    fn rhs_from(&mut self, c: f64, pos_ub: bool) {
        let mut acc = c;
        for t in 0..self.ind.len() {
            let j = self.ind[t] as usize;
            let a = self.val[t];
            let b = if (a <= 0.0) == pos_ub { self.lb[j] } else { self.ub[j] };
            acc = acc + a * b;
        }
        *self.w += self.ind.len() as f64 * 3.0;
        self.rhs = acc;
    }

    /// `6b33dd(f, sum_c, ...)`: Chvátal–Gomory rounding of f·row (continuous range sum `sum_c`) when the fractional
    /// part of the rhs stays in (0.001, 0.999) after the columns that fit; sets `ac`.
    fn cg_round(&mut self, f: f64, sum_c: f64) {
        self.ac = false;
        let n = self.ind.len();
        if n < 2 || f < 1e-4 || 1000.0 < f || 1.0 <= sum_c * f {
            return;
        }
        let act = if f <= 0.0 { self.max_act() } else { self.min_act() };
        let r = self.rhs * f - act * f;
        if r < self.ft || 1e8 < r {
            return;
        }
        let fl = (r + 1e-10).floor();
        let frac0 = r - fl;
        let fpart = |aa: f64| {
            let x = aa - (aa + 1e-10).floor();
            if x <= 0.0 { 0.0 } else { x }
        };
        let (mut fr, mut sum, mut cur, mut t) = (frac0, 0.0, 0.0, 0);
        while t < n {
            let aa = (self.val[t] * f).abs();
            let fa = fpart(aa);
            let j = self.ind[t] as usize;
            let d = self.ub[j] - self.lb[j];
            cur = if self.vt[j] == b'C' {
                sum + aa * d
            } else if fr + 1e-10 <= fa {
                let d2 = (1.0 - fa) * d;
                fr += d2;
                sum + d2
            } else {
                sum + d * fa
            };
            if !(cur < 1.0) {
                break;
            }
            sum = cur;
            t += 1;
        }
        *self.w += t as f64 * 5.0;
        if fr <= 0.001 || 0.999 <= fr {
            return;
        }
        let m = if f <= 1.0 { f } else { 1.0 };
        if fr - cur <= m * -self.ft {
            return;
        }
        self.rhs = fl;
        self.ac = true;
        let mut fr = frac0;
        for t in 0..n {
            let a = self.val[t];
            let aa = (f * a).abs();
            let fa = fpart(aa);
            let j = self.ind[t] as usize;
            self.val[t] = if self.vt[j] == b'C' {
                0.0
            } else if fr + 1e-10 <= fa {
                fr += (1.0 - fa) * (self.ub[j] - self.lb[j]);
                let v = (aa + 1.0) - fa;
                if a <= 0.0 { -v } else { v }
            } else if a <= 0.0 {
                -(aa - fa)
            } else {
                aa - fa
            };
        }
        *self.w += n as f64 * 5.0;
        self.rhs_from(self.rhs, false);
        self.compact();
        self.cont = 0;
        self.minact = STALE;
        self.maxact = STALE;
    }

    /// The row is redundant: emptied, rhs 0.
    fn redundant(&mut self, changed: &mut bool) -> i32 {
        self.rhs = 0.0;
        *changed = true;
        self.ind.clear();
        self.val.clear();
        0
    }

    /// `6aec61(env, &k, ind, val, &rhs, lb, ub, vtype, p9, p10, &changed, W)` with p10 >= 1 (578fd2 passes 2).
    /// shortcut: the PREPOWER != 0 block is not ported (default 0), nor the exact binary knapsack `6b5e11` for rows
    /// of 11..PREKNAPLIFTLEN binaries (ran14x18, nidda, csched008, gmu-35-40/50 reach it; docs §5.3): such rows skip
    /// that step. Port it before trusting those models.
    fn lift(&mut self, p9: i32, knaplen: i32, changed: &mut bool) -> i32 {
        let ft = self.ft;
        *changed = false;
        let mut mx = -self.act(-1.0);
        if -ft <= self.rhs - mx {
            return self.redundant(changed);
        }
        if self.drop_fixed() {
            mx = STALE;
            *changed = true;
        }
        if mx == STALE {
            mx = -self.act(-1.0);
        }
        if -ft <= self.rhs - mx {
            return self.redundant(changed);
        }
        let n = self.ind.len();
        let mut c = 0;
        while c < n && self.is_c(c) {
            c += 1;
            self.cont = 1;
        }
        *self.w += c as f64 * 3.0;
        if c == n {
            return 0;
        }
        let mut hint = self.cont;
        let r = self.rhs;
        if r - (r + 1e-10).floor() <= 1e-10 && hint != 1 {
            // all integer with one |a|: nothing to do when that divides the rhs
            let (mut d, mut t) = (-1.0f64, 0);
            while t < n {
                if self.is_c(t) {
                    self.cont = 1;
                    hint = 1;
                    break;
                }
                let aa = self.val[t].abs();
                if 0.0 <= d {
                    if ft * d < (aa - d).abs() {
                        break;
                    }
                } else {
                    d = aa;
                }
                t += 1;
            }
            *self.w += t as f64 + t as f64;
            if t == n {
                let q = r / d;
                hint = 0;
                if q - (q + 1e-10).floor() <= 1e-10 {
                    return 0;
                }
                self.cont = 0;
            }
        }
        self.maxact = mx;
        let lim = if knaplen == -1 { 512 } else { knaplen };
        if 2 < n && n as i32 <= lim && hint != 1 {
            let (mut all01, mut neg, mut t) = (true, 0.0f64, 0);
            while t < n {
                if self.is_c(t) {
                    self.cont = 1;
                    break;
                }
                let j = self.ind[t] as usize;
                if self.lb[j] != 0.0 || self.ub[j] != 1.0 {
                    all01 = false;
                }
                if self.val[t] < 0.0 {
                    neg = neg - self.val[t];
                }
                t += 1;
            }
            *self.w += t as f64 * 5.0;
            if t == n {
                self.cont = 0;
                if all01 && n < 11 {
                    self.ac = false;
                    self.knap_small(neg + self.rhs);
                    if self.ac {
                        *changed = true;
                    }
                    return 0;
                }
                if all01 {
                    // shortcut: 6b5e11 (11..PREKNAPLIFTLEN binaries) not ported, see above
                }
            }
        }
        if !self.ind.is_empty() {
            self.rounds(changed);
        }
        self.gcd_others(changed);
        if p9 == 0 {
            if *changed && self.drop_fixed() {
                if self.cont == 1 {
                    self.cont = -1;
                }
                self.minact = STALE;
                self.maxact = STALE;
            }
        } else {
            // shortcut: the PREPOWER block (p9 != 0) is not ported; GURO_PAR_PREPOWER is 0 by default
        }
        let s = self.max_act() - self.rhs;
        if s <= ft || 1e10 <= s {
            return 0;
        }
        self.cg_round(1.0 / s, 0.0);
        if self.ac {
            *changed = true;
        }
        0
    }

    /// The main loop of `6aec61` (LAB_00684830): up to 10 rounds while something changes.
    fn rounds(&mut self, changed: &mut bool) {
        let ft = self.ft;
        let mut iter = 0;
        loop {
            let mut any = false;
            let n = self.ind.len();
            // (A) coefficients above the max-activity slack
            self.ac = false;
            let s = self.max_act() - self.rhs;
            if 0.0 < s {
                let d = if self.cont == 1 { 0.0 } else { self.min_abs_int() };
                for t in 0..n {
                    if self.is_c(t) {
                        continue;
                    }
                    let j = self.ind[t] as usize;
                    let a = self.val[t];
                    let aa = a.abs();
                    let delta = if aa <= s {
                        if s - ft * 100.0 <= aa || d + aa <= s {
                            0.0
                        } else {
                            let (d1, d2) = (aa - s * 0.5, aa + (d - s));
                            if d1 <= d2 { d1 } else { d2 }
                        }
                    } else {
                        aa - s
                    };
                    if 1e-10 < delta {
                        self.ac = true;
                        if a <= 0.0 {
                            self.val[t] = a + delta;
                            self.rhs = self.lb[j] * delta + self.rhs;
                        } else {
                            self.val[t] = a - delta;
                            self.rhs = self.rhs - self.ub[j] * delta;
                        }
                    }
                }
                *self.w += n as f64 * 3.0;
                if self.ac {
                    self.minact = STALE;
                    self.maxact = STALE;
                    any = true;
                }
            }
            // (B) coefficients against the min-activity slack
            self.ac = false;
            if 2 < n && self.step_b() {
                any = true;
            }
            // (C) integer rows: divide by the coefficient gcd, round the rhs down
            self.ac = false;
            if self.cont == -1 {
                let mut t = 0;
                while t < n && !self.is_c(t) {
                    t += 1;
                }
                *self.w += t as f64 + t as f64;
                self.cont = if t < n { 1 } else { 0 };
            }
            if self.cont == 0 && self.step_c() {
                any = true;
            }
            // (D) rows over [0, ·] integers: rounding with multipliers -(i + c)/slack
            if self.step_d() {
                any = true;
            }
            // (E) Chvátal–Gomory rounding with multipliers 1, i/amax, i/(2·amin)
            self.ac = false;
            if self.min_act().abs() < 1e6 {
                let (mut amin, mut amax, mut sum_c) = (1e100f64, 0.0f64, 0.0f64);
                let n = self.ind.len();
                for t in 0..n {
                    let aa = self.val[t].abs();
                    if aa <= amin {
                        amin = aa;
                    }
                    if amax <= aa {
                        amax = aa;
                    }
                    if self.is_c(t) {
                        let j = self.ind[t] as usize;
                        sum_c = sum_c + (self.val[t] * (self.ub[j] - self.lb[j])).abs();
                    }
                }
                *self.w += n as f64;
                self.cg_round(1.0, sum_c);
                if 0.0 < amax {
                    for i in 1..6 {
                        if self.ac {
                            break;
                        }
                        self.cg_round(i as f64 / amax, sum_c);
                    }
                }
                if 0.0 < amin {
                    for i in 1..11 {
                        if self.ac {
                            break;
                        }
                        self.cg_round(i as f64 / (amin + amin), sum_c);
                    }
                }
            }
            if self.ac {
                any = true;
            }
            if any {
                *changed = true;
            }
            iter += 1;
            if !(iter < 10 && !self.ind.is_empty() && any) {
                break;
            }
        }
    }

    /// Smallest |a| over the columns when none is continuous (sets `cont` 0), else 0 (sets `cont` 1). W += 3 per
    /// entry looked at.
    fn min_abs_int(&mut self) -> f64 {
        let mut d = 1e100f64;
        let n = self.ind.len();
        let mut t = 0;
        while t < n {
            if self.is_c(t) {
                *self.w += t as f64 * 3.0;
                self.cont = 1;
                return 0.0;
            }
            let aa = self.val[t].abs();
            t += 1;
            if aa <= d {
                d = aa;
            }
        }
        *self.w += n as f64 * 3.0;
        self.cont = 0;
        d
    }

    /// Step B of the main loop (`6aec61` LAB_00684af9). Returns whether the row changed.
    fn step_b(&mut self) -> bool {
        let ft = self.ft;
        let n = self.ind.len();
        let mn = self.min_act();
        if mn < -1e10 {
            return false;
        }
        let s2 = self.rhs - mn;
        let dl = ft * 1000.0;
        if s2 < dl || 1e10 < s2 {
            return false;
        }
        let (dmin, mut m2) = if self.cont == 1 {
            (0.0, 0.0)
        } else {
            let d = self.min_abs_int();
            let m = if d == 0.0 && self.cont == 1 { 0.0 } else { 1e100 };
            (d, if d + d <= m { d + d } else { m })
        };
        if 0.0 < dmin && dmin < dl {
            return false;
        }
        if 0.0 < m2 {
            let lo = dmin + ft;
            for t in 0..n {
                let aa = self.val[t].abs();
                if lo < aa && aa <= m2 {
                    m2 = aa;
                }
            }
            *self.w += n as f64;
        }
        let d38 = ft * 100.0 + s2;
        let (mut sum_c, mut b3) = (0.0f64, false);
        for t in 0..n {
            let j = self.ind[t] as usize;
            let a = self.val[t];
            let mut aa = a.abs();
            if self.vt[j] != b'C' {
                if aa <= s2 - ft {
                    if dmin + aa <= d38 {
                        if d38 < a.abs() + m2 && dmin + aa < s2 - ft * 100.0 && m2 + ft < aa {
                            self.ac = true;
                            let delta = (s2 - dmin) - aa;
                            if a <= 0.0 {
                                self.val[t] = a - delta;
                                self.rhs = self.rhs - self.ub[j] * delta;
                            } else {
                                self.val[t] = a + delta;
                                self.rhs = self.lb[j] * delta + self.rhs;
                            }
                            aa = self.val[t].abs();
                        }
                    } else {
                        self.ac = true;
                        let delta = s2 - aa;
                        if a <= 0.0 {
                            self.val[t] = a - delta;
                            self.rhs = self.rhs - self.ub[j] * delta;
                        } else {
                            self.val[t] = a + delta;
                            self.rhs = self.lb[j] * delta + self.rhs;
                        }
                        continue;
                    }
                } else {
                    if ft + s2 < aa && aa < dl + s2 {
                        b3 = true;
                    }
                    continue;
                }
            }
            sum_c = sum_c + (self.ub[j] - self.lb[j]) * aa;
        }
        *self.w += n as f64 * 5.0;
        if !b3 && !(sum_c <= ft * 10000.0 || s2 - ft * 10.0 <= sum_c) {
            // the min-activity slack shrinks to the continuous part
            for t in 0..n {
                if self.is_c(t) {
                    continue;
                }
                let a = self.val[t];
                if a.abs() <= ft * 10.0 + s2 {
                    if s2 - ft < a.abs() {
                        self.val[t] = if a <= 0.0 { -sum_c } else { sum_c };
                    }
                } else {
                    self.val[t] = if a <= 0.0 { sum_c * -2.0 } else { sum_c + sum_c };
                }
            }
            self.rhs_from(sum_c, false);
            self.minact = STALE;
            self.maxact = STALE;
            return true;
        }
        if self.ac {
            self.minact = STALE;
            self.maxact = STALE;
            return true;
        }
        false
    }

    /// Step C (all integer): gcd division and rhs rounding. Returns whether the row changed.
    fn step_c(&mut self) -> bool {
        let ft = self.ft;
        let n = self.ind.len();
        let mut g = gcd_f64(&self.val);
        if g < 0.001 {
            return false;
        }
        if 10000.0 < g {
            let mut t = 0;
            while t < n && !(1e-10 < self.val[t] - (self.val[t] + 1e-10).floor()) {
                t += 1;
            }
            *self.w += t as f64;
            if t < n {
                return false;
            }
            g = 1.0;
        }
        let q = self.rhs / g;
        let f = (ft * 100.0 + q).floor();
        if q - ft <= f {
            return false;
        }
        self.ac = true;
        self.rhs = f;
        if 1e-10 < (g - 1.0).abs() {
            let inv = 1.0 / g;
            for v in self.val.iter_mut() {
                *v = (*v * inv + 0.5).floor();
            }
            *self.w += n as f64;
        }
        self.minact = STALE;
        self.maxact = STALE;
        true
    }

    /// Step D (`6aec61` LAB_0068542a): integer columns with lb >= 0, multipliers -(i + c)/slack, i = 1..9.
    /// Returns whether the row changed.
    fn step_d(&mut self) -> bool {
        let ft = self.ft;
        if self.cont == 1 {
            return false;
        }
        let n = self.ind.len();
        let mut t = 0;
        while t < n {
            if self.is_c(t) {
                self.cont = 1;
                break;
            }
            if self.lb[self.ind[t] as usize] < 0.0 {
                break;
            }
            t += 1;
        }
        *self.w += t as f64 * 3.0;
        if t < n {
            return false;
        }
        self.cont = 0;
        let s = self.max_act() - self.rhs;
        if !(0.0 < s) {
            return false;
        }
        for i in 1..10 {
            let c = if ft * 1000.0 <= 0.01 { ft * 1000.0 } else { 0.01 };
            let f = -(i as f64 + c) / s;
            if n < 2 || -f < 0.0001 || 1000.0 < -f {
                continue;
            }
            let act = if f <= 0.0 { self.max_act() } else { self.min_act() };
            let r = self.rhs * f - act * f;
            if !(ft <= r && r <= 1e8) {
                continue;
            }
            let c2 = r.ceil();
            let (mut b3, mut cnt) = (false, 0);
            while cnt < n {
                let aa = (self.val[cnt] * f).abs();
                let cc = (aa - 1e-10).ceil();
                if ft + aa < (r / c2) * cc {
                    break;
                }
                if cc * (r / c2) < aa - ft {
                    b3 = true;
                }
                cnt += 1;
            }
            *self.w += cnt as f64;
            if cnt == n && b3 {
                for v in self.val.iter_mut() {
                    let cc = (f * *v).abs() - 1e-10;
                    let cc = cc.ceil();
                    *v = if 0.0 <= *v { cc } else { -cc };
                }
                *self.w += n as f64;
                self.rhs_from(-c2, true);
                self.compact();
                self.minact = STALE;
                self.maxact = STALE;
                return true;
            }
        }
        false
    }

    /// `6b4738(c, ...)` (rows of 3..10 binaries, c = rhs + Σ|a<0|): per coefficient i (not marked, |a_i| > 1e-13),
    /// the sorted subset sums (<= c + 100ft, deduplicated within 1e-10) of the other coefficients decide a decrease
    /// of |a_i| (when no sum reaches c − 100ft) and an increase (when no sum + |a_i| is within 100ft of c); later
    /// coefficients with the same |a| are skipped. W += 5 per merge step. Sets `ac`; leaves the row compacted.
    fn knap_small(&mut self, mut c: f64) {
        let n = self.ind.len();
        let tol = 100.0 * self.ft;
        let mut done = vec![false; n];
        for i in 0..n {
            if done[i] {
                continue;
            }
            let mut ai = self.val[i].abs();
            if 1e-13 >= ai {
                continue;
            }
            let (lo, hi) = (c - tol, c + tol);
            let mut sums = vec![0.0f64];
            let (mut below, mut far, mut quit) = (true, true, false);
            for j in 0..n {
                let aj = self.val[j].abs();
                if j == i || 1e-13 >= aj {
                    continue;
                }
                let mut add = Vec::new();
                for &x in &sums {
                    let y = x + aj;
                    if y > hi {
                        break;
                    }
                    add.push(y);
                    far = far && tol < ((y - c) + ai).abs();
                }
                let Some(&last) = add.last() else { continue };
                if below && lo > last {
                    below = true;
                } else {
                    below = false;
                    if !far {
                        quit = true;
                        break;
                    }
                }
                // merge (new sums equal to an old one within 1e-10 are dropped)
                let (mut out, mut a, mut b, mut new) = (Vec::new(), 0, 0, false);
                while a < sums.len() && b < add.len() {
                    if 1e-10 > (sums[a] - add[b]).abs() {
                        b += 1;
                    } else if add[b] <= sums[a] {
                        out.push(add[b]);
                        new = true;
                        b += 1;
                    } else {
                        out.push(sums[a]);
                        a += 1;
                    }
                }
                if b < add.len() {
                    out.extend_from_slice(&add[b..]);
                    b = add.len();
                    new = true;
                }
                if new && a < sums.len() {
                    out.extend_from_slice(&sums[a..]);
                    a = sums.len();
                }
                *self.w += (b + a) as f64 * 5.0;
                if new {
                    sums = out;
                }
            }
            if !quit {
                let phases = if far { 1 } else { 0 };
                let mut ph = if below { 0 } else { 1 };
                while ph <= phases {
                    let base = if ph == 0 { c } else { c - ai };
                    if ph == 1 {
                        while sums.last().is_some_and(|&x| x > tol + base) {
                            sums.pop();
                        }
                    }
                    let mx = sums.last().copied().unwrap_or(0.0);
                    let mut d = base - mx;
                    if ph == 0 {
                        d = minsd(d, ai);
                    }
                    if d > tol {
                        self.ac = true;
                        let v = self.val[i];
                        if (ph == 0 && v < 0.0) || (ph == 1 && v > 0.0) {
                            self.val[i] = v + d;
                        } else {
                            self.val[i] = v - d;
                            self.rhs = self.rhs - d;
                        }
                        if ph == 0 {
                            c = c - d;
                            if 1e-13 > self.val[i].abs() {
                                self.val[i] = 0.0;
                                ai = 0.0;
                                break;
                            }
                        }
                        ai = self.val[i].abs();
                    }
                    ph += 1;
                }
            }
            if 1e-13 < ai {
                for t in i + 1..n {
                    if 1e-10 > (ai - self.val[t].abs()).abs() {
                        done[t] = true;
                    }
                }
            }
        }
        if self.ac {
            let mut k = 0;
            for t in 0..n {
                if 1e-10 < self.val[t].abs() {
                    self.ind[k] = self.ind[t];
                    self.val[k] = self.val[t];
                    k += 1;
                }
            }
            self.ind.truncate(k);
            self.val.truncate(k);
            self.minact = STALE;
            self.maxact = STALE;
        }
    }

    /// After the main loop (p10 >= 1): with integral coefficients (|a| <= 2e7, not all equal), `a7c752` gives each
    /// coefficient the gcd of the others; the first column whose value is > 1 gets `6b33dd(1/value, 0)`.
    fn gcd_others(&mut self, changed: &mut bool) {
        let n = self.ind.len();
        self.ac = false;
        if !(2 < n && self.cont != 1) {
            return;
        }
        let a0 = self.val[0].abs();
        let mut eq = true;
        let mut t = 0;
        while t < n {
            if self.is_c(t) {
                self.cont = 1;
                break;
            }
            let a = self.val[t];
            if 1e-10 < a - (a + 1e-10).floor() || 2e7 < a.abs() {
                break;
            }
            t += 1;
            if eq && 1e-10 < (a0 - a.abs()).abs() {
                eq = false;
            }
        }
        *self.w += t as f64 * 3.0;
        if t < n || eq {
            return;
        }
        self.cont = 0;
        let g = gcd_excl(&self.val, self.w);
        let mut t = 0;
        while t < n {
            if !self.is_c(t) && 1.0 < g[t] {
                self.cg_round(1.0 / g[t], 0.0);
                break;
            }
            t += 1;
        }
        *self.w += t as f64 * 3.0;
        if self.ac {
            *changed = true;
        }
    }
}

/// `a7cec0(s, n, ind, val, lb, ub, W)`: Σ bound·(s·a), the bound that minimizes each term; -1e100 at the first
/// infinite one. W += 3 per entry visited.
pub(crate) fn act_sum(ind: &[i32], val: &[f64], lb: &[f64], ub: &[f64], s: f64, w: &mut f64) -> f64 {
    let (mut sum, mut t) = (0.0, 0);
    while t < ind.len() {
        let v = val[t] * s;
        let j = ind[t] as usize;
        let b = if v <= 0.0 { ub[j] } else { lb[j] };
        if (v <= 0.0 && 1e30 <= b) || (0.0 < v && b <= -1e30) {
            sum = -1e100;
            break;
        }
        sum += b * v;
        t += 1;
    }
    *w += t as f64 * 3.0;
    sum
}

/// `a7c752(env, n, v, out, W)`: for each i the gcd of the other |v| (rounded; 2e9 = none), by a segment tree;
/// all 0 when some |v| is not integral.
pub(crate) fn gcd_excl(v: &[f64], w: &mut f64) -> Vec<f64> {
    let n = v.len();
    let mut out = vec![0.0; n];
    if n == 1 {
        out[0] = v[0];
    }
    if n < 2 {
        return out;
    }
    let mut size = 1;
    loop {
        size *= 2;
        if size >= n {
            break;
        }
    }
    let g = |a: i32, b: i32| -> i32 {
        if a == 0 || b == 0 {
            return 0;
        }
        if a >= 2000000000 {
            return b;
        }
        if b >= 2000000000 {
            return a;
        }
        let (mut x, mut y) = (a.max(b), a.min(b));
        while 0 < y {
            let r = x % y;
            x = y;
            y = r;
        }
        x
    };
    let mut tr = vec![0i32; 2 * size];
    for i in 0..n {
        let r = (v[i].abs() + 0.5).floor() as i32;
        tr[size + i] = r;
        if 1e-10 < (r as f64 - v[i].abs()).abs() {
            return out;
        }
    }
    for x in &mut tr[size + n..] {
        *x = 2000000000;
    }
    *w = (size as f64 + size as f64) + *w;
    for k in (1..size).rev() {
        tr[k] = g(tr[2 * k], tr[2 * k + 1]);
    }
    *w = *w + size as f64 + size as f64;
    let mut ex = vec![0i32; 2 * size];
    ex[1] = 2000000000;
    let mut k = 2;
    while k < 2 * size {
        ex[k] = g(ex[k / 2], tr[k + 1]);
        ex[k + 1] = g(ex[k / 2], tr[k]);
        k += 2;
    }
    *w = (k as f64 + k as f64) + *w;
    for i in 0..n {
        out[i] = ex[size + i] as f64;
    }
    out
}

#[cfg(test)]
mod tests {
    #[test]
    fn gcd_excl_segment_tree() {
        let mut w = 0.0;
        assert_eq!(super::gcd_excl(&[4.0, 6.0, 9.0], &mut w), vec![3.0, 1.0, 2.0]);
        assert_eq!(w, 8.0 + 8.0 + 16.0);
        assert_eq!(super::gcd_excl(&[4.0, 6.5], &mut w), vec![0.0, 0.0]);
    }
}
