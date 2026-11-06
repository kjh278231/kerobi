//! `4030c9` dominated columns (docs/m2_red_outer_b.md §7): on a column snapshot (CSR of the `53c422` columns and
//! its row transpose `7722f`), a column j that can move in one direction only against the rows it shares with
//! column k fixes k (phase 1) or itself (phase 2) at a bound, checked by `40a768`.
use crate::driver::Timer;
use crate::probe::TimerView;
use crate::state::{State, NIL};
use crate::Params;

/// The column snapshot: column CSR (`local_60` starts, `lVar8` counts, `local_c0`/`local_c8`), row CSR from
/// `7722f` (`lVar34` starts, `local_a8` ends, `local_b0`/`local_b8`), row-sign signatures (`local_120` `=`,
/// `local_118` a > 0, `local_110` a <= 0; bit row % 31), flags (`local_80`), scratch (`local_a0`).
struct Snap {
    cs: Vec<usize>,
    cn: Vec<usize>,
    crow: Vec<usize>,
    cval: Vec<f64>,
    rs: Vec<usize>,
    re: Vec<usize>,
    rcol: Vec<usize>,
    rval: Vec<f64>,
    eq: Vec<u32>,
    pos: Vec<u32>,
    neg: Vec<u32>,
    fl: Vec<u32>,
    scr: Vec<f64>,
}

/// Large-row scratch (`local_50` counts, `local_d0` ends, `local_f0` touched rows, `local_d8`/`local_e0` subset
/// row CSR, `local_e8` active bits, `local_58` subset).
struct Big {
    cnt: Vec<i32>,
    end: Vec<i32>,
    touched: Vec<usize>,
    sc: Vec<usize>,
    sv: Vec<f64>,
    act: Vec<u32>,
    sub: Vec<usize>,
}

enum Stop {
    /// timer or budget: leave the phase
    Phase,
    /// PRELIMIT < 0 after `3ba9de`: leave at once
    Exit,
}

impl State {
    /// `4030c9(st, env, T17)`. Returns 0 (`3de18e` never fails here).
    pub fn dominated(&mut self, _p: &Params, t: Timer) -> i32 {
        let mut t = t;
        t.start(self.work);
        let tv = TimerView { acc: t.acc(), running: t.running(), limit: t.limit };
        let (n, m, mu) = (self.n, self.m, self.work_mult);
        let w0 = self.work;
        let mut i = 0;
        while i < m && !(0 < self.row_cnt[i] && self.sense[i] != b'=') {
            i += 1;
        }
        self.work = (i as f64 + i as f64) * mu + self.work;
        if i == m {
            return 0;
        }
        let list = self.adjacent2_cols();
        let mut cand = Vec::new();
        let mut sum: i64 = 0;
        for &j in &list {
            let c = self.col_cnt[j as usize];
            if 0 < c && self.col_flag[j as usize] == 0 {
                cand.push(j as usize);
                sum += c as i64;
            }
        }
        self.work = mu * 5.0 * list.len() as f64 + self.work;
        if 2_000_000_000 < sum || (10 < cand.len() && (cand.len() as f64) * 5000.0 < sum as f64) {
            return 0;
        }
        let mut s = Snap { cs: vec![0; n + 1], cn: vec![0; n], crow: Vec::new(), cval: Vec::new(), rs: vec![0; m + 1],
                           re: vec![0; m], rcol: Vec::new(), rval: Vec::new(), eq: vec![0; n], pos: vec![0; n],
                           neg: vec![0; n], fl: vec![0; n], scr: vec![0.0; m] };
        for &j in &cand {
            s.cn[j] = self.col_cnt[j] as usize;
        }
        self.work = mu * 4.0 * cand.len() as f64 + self.work;
        for j in 0..n {
            s.cs[j] = s.crow.len();
            if s.cn[j] > 0 {
                let mut q = self.col_head[j];
                let mut walked = 0;
                while q != NIL {
                    let nd = self.nd[q as usize];
                    if nd.col >= 0 {
                        s.crow.push(nd.row as usize);
                        s.cval.push(nd.val);
                    }
                    walked += 1;
                    q = nd.nc;
                }
                self.work = mu * 6.0 * walked as f64 + self.work;
            }
        }
        s.cs[n] = s.crow.len();
        self.work = self.work + mu * 3.0 * n as f64;
        self.transpose(&mut s);
        self.work = mu * 3.0 * m as f64 + self.work;
        for &j in &cand {
            for t in s.cs[j]..s.cs[j] + s.cn[j] {
                let r = s.crow[t];
                let bit = 1u32 << (r % 31);
                if self.sense[r] == b'=' {
                    s.eq[j] |= bit;
                } else if s.cval[t] <= 0.0 {
                    s.neg[j] |= bit;
                } else {
                    s.pos[j] |= bit;
                }
            }
            self.work = mu * 4.0 * s.cn[j] as f64 + self.work;
        }
        self.work = self.work + mu * 3.0 * cand.len() as f64;
        for &j in &cand {
            let (lo, hi) = self.implied_col_bounds(j, 3, None);
            if -1e-9 <= lo - self.lb[j] {
                s.fl[j] |= 1;
            }
            if hi - self.ub[j] <= 1e-9 {
                s.fl[j] |= 2;
            }
        }
        self.work = self.work + mu * 4.0 * cand.len() as f64;
        let lim = (n + 1).min(1000);
        let mut big = Big { cnt: vec![0; m], end: vec![0; m], touched: Vec::new(), sc: Vec::new(), sv: Vec::new(),
                            act: vec![0; n], sub: Vec::new() };
        let mut wchk = w0;
        // phase 1: j fixes the columns it dominates
        let mut idx = 0;
        while idx < cand.len() {
            match self.dom_poll(&tv, &mut wchk, w0) {
                Some(Stop::Exit) => return 0,
                Some(Stop::Phase) => break,
                None => {}
            }
            let j = cand[idx];
            for d in [-1.0, 1.0] {
                if self.col_cnt[j] < 1 || (d == 1.0 && s.fl[j] & 2 == 0) || (d == -1.0 && s.fl[j] & 1 == 0) {
                    continue;
                }
                let (mut r, mut v, mut best) = (usize::MAX, 0.0, n as i32 + 1);
                for t in s.cs[j]..s.cs[j] + s.cn[j] {
                    let ri = s.crow[t];
                    let x = d * s.cval[t];
                    if (self.sense[ri] == b'=' || 1e-10 < x) && self.row_cnt[ri] < best {
                        v = x;
                        r = ri;
                        best = self.row_cnt[ri];
                    }
                }
                self.work = mu * 3.0 * s.cn[j] as f64 + self.work;
                if lim as i32 <= best {
                    continue;
                }
                let (rs, re) = (s.rs[r], s.re[r]);
                let eq = self.sense[r] == b'=';
                let small = if best < 51 {
                    true
                } else {
                    let nact = self.dom_subset(&mut s, &mut big, r, |rv, f| if eq { f } else if rv <= 0.0 { f & 1 } else { f & 2 });
                    self.work = mu * 7.0 * (re - rs) as f64 + self.work;
                    nact < 6
                };
                if small {
                    for t in rs..re {
                        let k = s.rcol[t];
                        if 0 < self.col_cnt[k] && s.fl[k] & 0x10 == 0 && k != j {
                            let ak = s.rval[t];
                            self.dom_fix_other(&mut s, j, k, d, v, ak, eq);
                        }
                    }
                    self.work = mu * 5.0 * (re - rs) as f64 + self.work;
                    s.fl[j] &= if d == 1.0 { !2 } else { !1 };
                    continue;
                }
                self.dom_build(&mut s, &mut big);
                let sub = big.sub.clone();
                for &k in &sub {
                    for d2 in [-1.0, 1.0] {
                        if !(0 < self.col_cnt[k] && (d2 != 1.0 || big.act[k] & 2 != 0) && (d2 != -1.0 || big.act[k] & 1 != 0)) {
                            continue;
                        }
                        let (mut r2, mut v2, mut b2) = (usize::MAX, 0.0, n as i32 + 1);
                        for t in s.cs[k]..s.cs[k] + s.cn[k] {
                            let ri = s.crow[t];
                            let x = d2 * s.cval[t];
                            let len = big.end[ri] - big.cnt[ri];
                            if (self.sense[ri] == b'=' || 1e-10 < x) && len < b2 {
                                v2 = x;
                                r2 = ri;
                                b2 = len;
                            }
                        }
                        self.work = mu * 5.0 * s.cn[k] as f64 + self.work;
                        if r2 == usize::MAX {
                            continue;
                        }
                        let (a, b) = (big.cnt[r2] as usize, big.end[r2] as usize);
                        let eq2 = self.sense[r2] == b'=';
                        for t in a..b {
                            let m2 = big.sc[t];
                            if 0 < self.col_cnt[m2] && m2 != k {
                                self.dom_fix_other(&mut s, k, m2, d2, v2, big.sv[t], eq2);
                            }
                        }
                        self.work = mu * 6.0 * (b - a) as f64 + self.work;
                    }
                }
                self.work = mu * 7.0 * sub.len() as f64 + self.work;
                if eq {
                    for &k in &sub {
                        s.fl[k] |= 0x10;
                    }
                    self.work = (sub.len() as f64 + sub.len() as f64) * mu + self.work;
                } else {
                    for &k in &sub {
                        s.fl[k] &= !big.act[k];
                    }
                    self.work = mu * 3.0 * sub.len() as f64 + self.work;
                }
                self.dom_clear(&mut big);
            }
            idx += 1;
        }
        self.work = mu * 10.0 * idx as f64 + self.work;
        // phase 2: j fixes itself when a column of one of its rows dominates it
        for &j in &cand {
            if self.col_cnt[j] < 1 || s.fl[j] & 0x10 != 0 {
                continue;
            }
            let mut f = s.fl[j] | 0xc;
            if 1e8 < self.lb[j].abs() {
                f &= !8;
            }
            s.fl[j] = if self.ub[j].abs() <= 1e8 { f } else { f & !4 };
            let mut t = s.cs[j];
            while t < s.cs[j] + s.cn[j] {
                if self.sense[s.crow[t]] == b'=' {
                    s.fl[j] = 0x10;
                    break;
                }
                t += 1;
            }
            self.work = mu * 3.0 * (t - s.cs[j]) as f64 + self.work;
        }
        self.work = self.work + mu * 5.0 * cand.len() as f64;
        let mut idx = 0;
        while idx < cand.len() {
            match self.dom_poll(&tv, &mut wchk, w0) {
                Some(Stop::Exit) => return 0,
                Some(Stop::Phase) => break,
                None => {}
            }
            let j = cand[idx];
            for d in [-1.0, 1.0] {
                if self.col_cnt[j] < 1 || s.fl[j] & 0x10 != 0 || (d == 1.0 && s.fl[j] & 8 == 0) || (d == -1.0 && s.fl[j] & 4 == 0) {
                    continue;
                }
                let fixv = if d == 1.0 { self.lb[j] } else { self.ub[j] };
                let (mut r, mut v, mut best) = (usize::MAX, 0.0, n as i32 + 1);
                for t in s.cs[j]..s.cs[j] + s.cn[j] {
                    let x = d * s.cval[t];
                    let ri = s.crow[t];
                    if x < -1e-10 && self.row_cnt[ri] < best {
                        v = x;
                        r = ri;
                        best = self.row_cnt[ri];
                    }
                }
                self.work = mu * 3.0 * s.cn[j] as f64 + self.work;
                if lim as i32 <= best {
                    continue;
                }
                let (rs, re) = (s.rs[r], s.re[r]);
                let small = if best <= 50 {
                    true
                } else {
                    let nact = self.dom_subset(&mut s, &mut big, r, |rv, f| if rv <= 0.0 { f & 8 } else { f & 4 });
                    self.work = mu * 6.0 * (re - rs) as f64 + self.work;
                    nact < 6
                };
                if small {
                    let mut t = rs;
                    while t < re {
                        let k = s.rcol[t];
                        let ak = s.rval[t];
                        if 0 < self.col_cnt[k] && k != j && s.fl[k] & 0x10 == 0 && self.dom_fix_self(&mut s, k, j, d, v, ak, fixv) {
                            break;
                        }
                        t += 1;
                    }
                    self.work = mu * 5.0 * (t - rs) as f64 + self.work;
                    continue;
                }
                self.dom_build(&mut s, &mut big);
                let sub = big.sub.clone();
                for &k in &sub {
                    for d2 in [-1.0, 1.0] {
                        if !(0 < self.col_cnt[k] && (d2 != 1.0 || big.act[k] & 8 != 0) && (d2 != -1.0 || big.act[k] & 4 != 0)) {
                            continue;
                        }
                        let fixv2 = if d2 == 1.0 { self.lb[k] } else { self.ub[k] };
                        let (mut r2, mut v2, mut b2) = (usize::MAX, 0.0, n as i32 + 1);
                        for t in s.cs[k]..s.cs[k] + s.cn[k] {
                            let ri = s.crow[t];
                            let x = d2 * s.cval[t];
                            let len = big.end[ri] - big.cnt[ri];
                            if x < -1e-10 && len < b2 {
                                v2 = x;
                                r2 = ri;
                                b2 = len;
                            }
                        }
                        self.work = mu * 4.0 * s.cn[k] as f64 + self.work;
                        if r2 == usize::MAX {
                            continue;
                        }
                        let (a, b) = (big.cnt[r2] as usize, big.end[r2] as usize);
                        let mut t = a;
                        while t < b {
                            let m2 = big.sc[t];
                            if 0 < self.col_cnt[m2] && k != m2 && self.dom_fix_self(&mut s, m2, k, d2, v2, big.sv[t], fixv2) {
                                break;
                            }
                            t += 1;
                        }
                        self.work = mu * 4.0 * (t - a) as f64 + self.work;
                    }
                }
                self.work = mu * 9.0 * sub.len() as f64 + self.work;
                for &k in &sub {
                    s.fl[k] &= !big.act[k];
                }
                self.work = mu * 3.0 * sub.len() as f64 + self.work;
                self.dom_clear(&mut big);
            }
            idx += 1;
        }
        self.work = self.work + mu * 10.0 * idx as f64;
        0
    }

    /// The poll at each phase step: `adf2be` timer, then every 1e7 work `3ba5ac`/`3ba9de` (budget only) and the
    /// budget/5 cap.
    fn dom_poll(&mut self, tv: &TimerView, wchk: &mut f64, w0: f64) -> Option<Stop> {
        if tv.over(self.work) {
            return Some(Stop::Phase);
        }
        if 1e7 < self.work - *wchk {
            self.budget_check();
            if self.prelimit < 0 {
                return Some(Stop::Exit);
            }
            *wchk = self.work;
            if self.budget / 5.0 < *wchk - w0 {
                return Some(Stop::Phase);
            }
        }
        None
    }

    /// Phase 1 test of column k against j (direction d, j's coefficient v in the shared row, k's ak): k goes to its
    /// lower bound, else its upper bound, when `40a768` confirms.
    #[allow(clippy::too_many_arguments)]
    fn dom_fix_other(&mut self, s: &mut Snap, j: usize, k: usize, d: f64, v: f64, ak: f64, eq: bool) {
        let low_ok = self.lb[k].abs() < 1e8 && if eq { (ak - v).abs() < 1e-10 } else { v - 1e-10 < ak };
        if low_ok && self.dom_check(s, d, 1.0, j, k) {
            let x = self.lb[k];
            self.fix_col(k, x, true);
            return;
        }
        let up_ok = self.ub[k].abs() < 1e8 && if eq { (-(ak + v)).abs() < 1e-10 } else { v - 1e-10 < -ak };
        if up_ok && self.dom_check(s, d, -1.0, j, k) {
            let x = self.ub[k];
            self.fix_col(k, x, true);
        }
    }

    /// Phase 2 test: column k (coefficient ak) dominates j (direction d, coefficient v): j goes to `fixv`.
    #[allow(clippy::too_many_arguments)]
    fn dom_fix_self(&mut self, s: &mut Snap, k: usize, j: usize, d: f64, v: f64, ak: f64, fixv: f64) -> bool {
        if ak - 1e-10 < v && s.fl[k] & 2 != 0 && self.dom_check(s, 1.0, d, k, j) {
            self.fix_col(j, fixv, true);
            return true;
        }
        if -(ak + 1e-10) < v && s.fl[k] & 1 != 0 && self.dom_check(s, -1.0, d, k, j) {
            self.fix_col(j, fixv, true);
            return true;
        }
        false
    }

    /// `40a768(d1, d2, st, …, j, k, scratch)`: does d1·x_j dominate d2·x_k (costs, integrality, row signatures,
    /// then every shared row's scaled coefficients both ways)?
    fn dom_check(&mut self, s: &mut Snap, d1: f64, d2: f64, j: usize, k: usize) -> bool {
        let mu = self.work_mult;
        if 1_000_000 < self.n {
            self.work += 10.0;
        }
        if self.obj[j] * d1 > self.obj[k] * d2 + 1e-10 {
            return false;
        }
        if self.vtype[j] != b'C' && self.vtype[k] == b'C' {
            return false;
        }
        if s.eq[j] != s.eq[k] {
            return false;
        }
        let (sj, tj) = if 0.0 > d1 { (s.neg[j], s.pos[j]) } else { (s.pos[j], s.neg[j]) };
        let (sk, tk) = if 0.0 > d2 { (s.neg[k], s.pos[k]) } else { (s.pos[k], s.neg[k]) };
        if sj & !sk != 0 || tk & !tj != 0 || (sj == 0 && sk == 0 && tj == 0 && tk == 0) {
            return false;
        }
        let (jr, kr) = (s.cs[j]..s.cs[j] + s.cn[j], s.cs[k]..s.cs[k] + s.cn[k]);
        for t in kr.clone() {
            s.scr[s.crow[t]] = 0.0;
        }
        self.work = mu * 2.0 * kr.len() as f64 + self.work;
        for t in jr.clone() {
            s.scr[s.crow[t]] = s.cval[t] * d1;
        }
        self.work = mu * 3.0 * jr.len() as f64 + self.work;
        let mut t = kr.start;
        while t < kr.end {
            let r = s.crow[t];
            let (x, y) = (s.cval[t] * d2, s.scr[r]);
            if x < y - 1e-10 || (self.sense[r] == b'=' && 1e-10 < (x - y).abs()) {
                break;
            }
            t += 1;
        }
        self.work = mu * 4.0 * (t - kr.start) as f64 + self.work;
        if t < kr.end {
            return false;
        }
        for t in jr.clone() {
            s.scr[s.crow[t]] = 0.0;
        }
        self.work = mu * 2.0 * jr.len() as f64 + self.work;
        for t in kr.clone() {
            s.scr[s.crow[t]] = s.cval[t] * d2;
        }
        self.work = mu * 3.0 * kr.len() as f64 + self.work;
        let mut t = jr.start;
        while t < jr.end {
            let r = s.crow[t];
            let (x, y) = (s.cval[t] * d1, s.scr[r]);
            if y + 1e-10 < x || (self.sense[r] == b'=' && 1e-10 < (x - y).abs()) {
                break;
            }
            t += 1;
        }
        self.work = mu * 4.0 * (t - jr.start) as f64 + self.work;
        jr.end <= t
    }

    /// Large row r: the live unflagged columns (`local_58`) with their active bits; returns how many are active.
    fn dom_subset(&mut self, s: &mut Snap, big: &mut Big, r: usize, bits: impl Fn(f64, u32) -> u32) -> usize {
        big.touched.clear();
        big.sc.clear();
        let mut nact = 0;
        let mut sub = Vec::new();
        for t in s.rs[r]..s.re[r] {
            let k = s.rcol[t];
            if 0 < self.col_cnt[k] && s.fl[k] & 0x10 == 0 {
                sub.push(k);
                big.act[k] = bits(s.rval[t], s.fl[k]);
                nact += (big.act[k] != 0) as usize;
            }
        }
        big.sub = sub;
        nact
    }

    /// Subset row CSR: rows touched by the subset's columns, counted, prefix-summed and filled column by column
    /// (each row's entries in reverse subset order).
    fn dom_build(&mut self, s: &mut Snap, big: &mut Big) {
        let mu = self.work_mult;
        let sub = std::mem::take(&mut big.sub);
        for &k in &sub {
            for t in s.cs[k]..s.cs[k] + s.cn[k] {
                let ri = s.crow[t];
                if big.cnt[ri] == 0 {
                    big.touched.push(ri);
                }
                big.cnt[ri] += 1;
            }
            self.work = s.cn[k] as f64 * (mu * 3.0) + self.work;
        }
        self.work = self.work + mu * 3.0 * sub.len() as f64;
        let t0 = big.touched[0];
        big.end[t0] = big.cnt[t0];
        for w in 1..big.touched.len() {
            let (a, b) = (big.touched[w - 1], big.touched[w]);
            big.cnt[b] += big.cnt[a];
            big.end[b] = big.cnt[b];
        }
        self.work = mu * 3.0 * big.touched.len() as f64 + self.work;
        let total = big.end[*big.touched.last().unwrap()] as usize;
        let mut sc = vec![0usize; total];
        let mut sv = vec![0.0f64; total];
        for &k in &sub {
            for t in s.cs[k]..s.cs[k] + s.cn[k] {
                let ri = s.crow[t];
                big.cnt[ri] -= 1;
                sc[big.cnt[ri] as usize] = k;
                sv[big.cnt[ri] as usize] = s.cval[t];
            }
            self.work = mu * 5.0 * s.cn[k] as f64 + self.work;
        }
        self.work = self.work + mu * 3.0 * sub.len() as f64;
        big.sc = sc;
        big.sv = sv;
        big.sub = sub;
    }

    /// Zero the touched rows' counts (the first one is already 0). W += (μ·2)·(touched − 1).
    fn dom_clear(&mut self, big: &mut Big) {
        for &ri in &big.touched[1..] {
            big.cnt[ri] = 0;
        }
        self.work = self.work_mult * 2.0 * (big.touched.len() - 1) as f64 + self.work;
    }

    /// `7722f(m, n, …, W)`: column CSR to row CSR (rows' entries in column order); work without the multiplier.
    fn transpose(&mut self, s: &mut Snap) {
        let (m, n) = (self.m, self.n);
        let mut cnt = vec![0usize; m];
        for j in 0..n {
            for t in s.cs[j]..s.cs[j] + s.cn[j] {
                cnt[s.crow[t]] += 1;
            }
            self.work = s.cn[j] as f64 * 2.0 + self.work;
        }
        self.work = self.work + n as f64 * 3.0;
        s.rs[0] = 0;
        for i in 0..m {
            s.rs[i + 1] = s.rs[i] + cnt[i];
        }
        self.work = (self.work + m as f64) + m as f64;
        let total = s.rs[m];
        s.rcol = vec![0; total];
        s.rval = vec![0.0; total];
        let mut fill = s.rs.clone();
        for j in 0..n {
            for t in s.cs[j]..s.cs[j] + s.cn[j] {
                let r = s.crow[t];
                s.rcol[fill[r]] = j;
                s.rval[fill[r]] = s.cval[t];
                fill[r] += 1;
            }
            self.work = s.cn[j] as f64 * 5.0 + self.work;
        }
        self.work = self.work + n as f64 * 3.0;
        for i in 0..m {
            s.re[i] = cnt[i] + s.rs[i];
        }
    }
}
