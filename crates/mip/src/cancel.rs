//! `40d2bc` nonzero cancellation (docs/m2_red_outer_b.md §5): `414c03` (mode 0) adds multiples of an `=` row to
//! rows sharing most of its columns, `414c53` (mode 1) the column counterpart (an implied-free column substituted
//! into columns sharing most of its rows). Names follow mode 0: the pivot and the rows it cancels in are "majors"
//! (rows; columns in mode 1), their entries "minors".
use crate::driver::Timer;
use crate::probe::{sort_desc, TimerView};
use crate::state::{State, INF, NIL};
use crate::Params;

/// The two multiplier hypotheses of a candidate major (`40c0b2` and its inline copy): ratio `r` against the
/// slot of its sign. Counts are minus the agreeing entries; `poison` marks a near miss.
fn ratio_vote(r: f64, slot: &mut f64, cnt: &mut i32, tol: f64, poison: i32) {
    let d = (r - *slot).abs();
    if d <= 1e-13 {
        *cnt -= 1;
    } else if tol < d {
        if r.abs() < slot.abs() {
            *cnt = -1;
            *slot = r;
        }
    } else {
        *cnt = poison;
        if r.abs() < slot.abs() {
            *slot = r;
        }
    }
}

impl State {
    fn mj_head(&self, m0: bool, i: usize) -> u32 {
        if m0 { self.row_head[i] } else { self.col_head[i] }
    }
    fn mn_head(&self, m0: bool, q: usize) -> u32 {
        if m0 { self.col_head[q] } else { self.row_head[q] }
    }
    /// Sign class of row `r` (cached in `sgn`): 1 | 2 all live coefficients > 0 | 4 all < 0. W += (μ·4)·nodes.
    fn sign_class(&mut self, sgn: &mut [u32], r: usize) -> u32 {
        if sgn[r] == 0 {
            let (mut pos, mut neg, mut walked) = (true, true, 0);
            let mut q = self.row_head[r];
            while q != NIL {
                let nd = self.nd[q as usize];
                if nd.col >= 0 {
                    pos = pos && nd.val > 0.0;
                    neg = neg && !(0.0 < nd.val || nd.val == 0.0);
                }
                walked += 1;
                q = nd.nr;
            }
            self.work += self.work_mult * 4.0 * walked as f64;
            sgn[r] = 1 + if pos { 2 } else { 0 } + if neg { 4 } else { 0 };
        }
        sgn[r]
    }

    /// `40c618(d, env, st, mode, node)`: node value += d; mode 1 also moves its row's activities from the old
    /// value to the new one (bounds of its column). A result below 1e-10 kills the node (live counts −1).
    fn cancel_add(&mut self, m0: bool, at: usize, d: f64) {
        let nd = self.nd[at];
        let v = d + nd.val;
        if !m0 {
            let (r, c) = (nd.row as usize, nd.col as usize);
            let (l, u) = (self.lb[c], self.ub[c]);
            self.contrib(r, nd.val, l, false, false);
            self.contrib(r, nd.val, u, true, false);
            self.contrib(r, v, l, false, true);
            self.contrib(r, v, u, true, true);
        }
        self.nd[at].val = v;
        if v.abs() < 1e-10 {
            self.nd[at].col = -1;
            self.row_cnt[nd.row as usize] -= 1;
            self.col_cnt[nd.col as usize] -= 1;
        }
    }

    /// `40cc92(v, env, st, mode, i, q)`: fill-in node v at major i, minor q (`3bea24`); mode 1 adds it to row q's
    /// activities first.
    fn cancel_fill(&mut self, m0: bool, i: usize, q: usize, v: f64) {
        if m0 {
            self.new_node(v, i, q);
        } else {
            let (l, u) = (self.lb[i], self.ub[i]);
            self.contrib(q, v, l, false, true);
            self.contrib(q, v, u, true, true);
            self.new_node(v, q, i);
        }
    }

    /// `414c03(st, env, mode, T15)`: row cancellation (`40d2bc` mode 0).
    pub fn cancel_rows(&mut self, p: &Params, mode: i32, t: Timer) -> i32 {
        self.cancel(p, true, mode, t)
    }

    /// `414c53(st, env, mode, T16)`: column cancellation (`40d2bc` mode 1).
    pub fn cancel_cols(&mut self, p: &Params, mode: i32, t: Timer) -> i32 {
        self.cancel(p, false, mode, t)
    }

    /// `40d2bc(st, env, mode, M, T, W)`, `m0` = mode 0. `t` is the reduction timer before the driver starts it.
    fn cancel(&mut self, p: &Params, m0: bool, mm: i32, t: Timer) -> i32 {
        let mut t = t;
        t.start(self.work);
        let tv = TimerView { acc: t.acc(), running: t.running(), limit: t.limit };
        let mu = self.work_mult;
        let (nmaj, nmin) = if m0 { (self.m, self.n) } else { (self.n, self.m) };
        let w0 = self.work;
        let mut wchk = w0;
        let f = match p.numeric_focus { 0 => 0.1, 1 => 10.0, 2 => 100.0, _ => 1000.0 };
        let tol = f * p.feastol;
        let poison = -(nmin as i32) - 1;
        let mut cnt = vec![0i32; nmaj]; // matches per candidate major (CNT); <0 while voting, >0 selected
        let mut miss = vec![0i32; nmaj]; // 1 + the pivot minor it lacks, -1 none, 0 unknown
        let mut stamp = vec![0i32; nmin];
        let mut pc = vec![0.0f64; nmin]; // pivot coefficient per minor
        let mut cp = vec![0i32; nmaj];
        let mut cn = vec![0i32; nmaj];
        let mut mult = vec![0.0f64; nmaj]; // positive-ratio slot, then the chosen multiplier (st+0x3a0)
        let mut mn = vec![0.0f64; nmaj];
        let mut sgn = vec![0u32; self.m];
        let mut rmark = vec![false; self.m];
        let mut rlist: Vec<i32> = Vec::new();
        let mut ncancel = 0;
        let list = self.adjacent_list(!m0);
        let maj_cnt = |s: &State, i: usize| if m0 { s.row_cnt[i] } else { s.col_cnt[i] };
        let min_cnt = |s: &State, q: usize| if m0 { s.col_cnt[q] } else { s.row_cnt[q] };
        let maj_of = |nd: &crate::state::Node| if m0 { nd.row } else { nd.col } as usize;
        let min_of = |nd: &crate::state::Node| if m0 { nd.col } else { nd.row } as usize;
        let mj_next = |nd: &crate::state::Node| if m0 { nd.nr } else { nd.nc };
        let mn_next = |nd: &crate::state::Node| if m0 { nd.nc } else { nd.nr };
        let mut idx = 0;
        while idx < list.len() {
            let pv = list[idx] as usize;
            let eligible = maj_cnt(self, pv) >= 2
                && if m0 {
                    self.sense[pv] == b'='
                } else {
                    self.col_flag[pv] == 0 && self.col_cnt[pv] <= 500 && self.ub_implied(p, pv) && self.lb_implied(p, pv)
                };
            if !eligible {
                idx += 1;
                continue;
            }
            let two = mm != 0 && maj_cnt(self, pv) >= 3;
            let mut maxc = 0;
            let mut marked = false;
            if tv.over(self.work) {
                break;
            }
            if self.work - wchk > 1e7 {
                // `3ba5ac` (callback) and `3ba9de` (limits): only the budget stop is modelled
                self.budget_check();
                if self.prelimit < 0 {
                    return 0;
                }
                wchk = self.work;
                if self.work - w0 > self.budget / 5.0 {
                    break;
                }
            }
            // the pivot's minors, shortest first
            let mut lst: Vec<i32> = Vec::new();
            let mut key: Vec<f64> = Vec::new();
            let mut tot: i64 = 0;
            let mut walked = 0;
            let mut q = self.mj_head(m0, pv);
            while q != NIL {
                let nd = self.nd[q as usize];
                if nd.col >= 0 {
                    let mi = min_of(&nd);
                    pc[mi] = nd.val;
                    lst.push(mi as i32);
                    let c = min_cnt(self, mi);
                    key.push(-c as f64);
                    tot += c as i64;
                }
                walked += 1;
                q = mj_next(&nd);
            }
            self.work += (mu * 6.0) * walked as f64;
            sort_desc(&mut key, &mut lst);
            let k = lst.len();
            if 200.0 < -key[0] {
                idx += 1;
                continue;
            }
            let s = if !(200.0 < -key[1]) && two { 1usize } else { 0 };
            let need = (k - s) as i32;
            // rows met through the seed minors (the first s+1)
            let mut sum: i64 = 0;
            for tt in 0..=s {
                let mut walked = 0;
                let mut q = self.mn_head(m0, lst[tt] as usize);
                while q != NIL {
                    let nd = self.nd[q as usize];
                    if nd.col >= 0 {
                        sum += maj_cnt(self, maj_of(&nd)) as i64;
                    }
                    walked += 1;
                    q = mn_next(&nd);
                }
                self.work = walked as f64 * mu + self.work;
                if tt == s {
                    self.work = self.work + mu * ((tt + 1) as f64 + (tt + 1) as f64);
                }
            }
            let mut cand_len: i64 = 0; // local_230
            if tot / 4 < sum {
                // count matches minor by minor, keeping only majors that can still reach k − s
                let full = need - 1;
                let mut tt = 0usize;
                loop {
                    let mut walked = 0;
                    let mut q = self.mn_head(m0, lst[tt] as usize);
                    while q != NIL {
                        let nd = self.nd[q as usize];
                        q = mn_next(&nd);
                        walked += 1;
                        if nd.col < 0 {
                            continue;
                        }
                        let i = maj_of(&nd);
                        let mut c = cnt[i];
                        if tt <= s {
                            if !(maj_cnt(self, i) >= need && i != pv) {
                                continue;
                            }
                            if tt == 1 && c == 0 {
                                if mm < 2 && (k as i32) < maj_cnt(self, i) {
                                    continue;
                                }
                                miss[i] = lst[0] + 1;
                                c = cnt[i];
                            }
                        } else {
                            if !((tt - s) as i32 <= c) {
                                continue;
                            }
                            if c == tt as i32 - 1 && miss[i] == 0 {
                                if mm < 2 && (k as i32) < maj_cnt(self, i) {
                                    cnt[i] = 0;
                                    continue;
                                }
                                miss[i] = lst[tt - 1] + 1;
                                c = cnt[i];
                            }
                        }
                        c += 1;
                        cnt[i] = c;
                        maxc = maxc.max(c);
                        if c == full {
                            cand_len += maj_cnt(self, i) as i64;
                        }
                    }
                    tt += 1;
                    let w = if tt <= s + 1 { (mu * 5.0) * walked as f64 } else { mu * 4.0 * walked as f64 };
                    let mut w1 = w + self.work;
                    if tt == s + 1 {
                        w1 = w1 + (tt as f64 + tt as f64) * mu;
                    }
                    if tt > s && (k <= tt || maxc < (tt - s) as i32) {
                        self.work = w1 + mu * 2.0 * ((tt - s) as f64 - 1.0);
                        break;
                    }
                    self.work = w1;
                }
            } else {
                // mark the pivot's minors, then count each seed-reached major's overlap from its own list
                for &mi in &lst {
                    stamp[mi as usize] = 1;
                }
                self.work = (k as f64 + k as f64) * mu + self.work;
                marked = true;
                let mut st_no = 1;
                for tt in 0..=s {
                    let mut walked = 0;
                    let mut q = self.mn_head(m0, lst[tt] as usize);
                    while q != NIL {
                        let nd = self.nd[q as usize];
                        q = mn_next(&nd);
                        walked += 1;
                        if nd.col < 0 {
                            continue;
                        }
                        let i = maj_of(&nd);
                        let mc = maj_cnt(self, i);
                        if i == pv || !(mc >= need) || (!m0 && mc > 500) || cnt[i] != 0 {
                            continue;
                        }
                        if tt == 1 {
                            if mm < 2 && (k as i32) < mc {
                                continue;
                            }
                            miss[i] = lst[0] + 1;
                        }
                        st_no += 1;
                        let mut w2 = 0;
                        let mut r = self.mj_head(m0, i);
                        while r != NIL {
                            let n2 = self.nd[r as usize];
                            if n2.col >= 0 {
                                let mi = min_of(&n2);
                                if stamp[mi] != 0 {
                                    stamp[mi] = st_no;
                                    cnt[i] += 1;
                                }
                            }
                            w2 += 1;
                            r = mj_next(&n2);
                        }
                        self.work = (w2 as f64 + w2 as f64) * mu + self.work;
                        let c = cnt[i];
                        if s != 0 && tt == 0 && c == k as i32 - 1 {
                            if mm < 2 && (k as i32) < mc {
                                cnt[i] = 0;
                                continue;
                            }
                            let mut u = 1;
                            while u < k {
                                if stamp[lst[u] as usize] != st_no {
                                    miss[i] = lst[u] + 1;
                                    break;
                                }
                                u += 1;
                            }
                            self.work = u as f64 * mu + self.work;
                        }
                        maxc = maxc.max(cnt[i]);
                        cand_len += mc as i64;
                    }
                    self.work = mu * 4.0 * walked as f64 + self.work;
                    if tt == s {
                        self.work = self.work + mu * ((tt + 1) as f64 + (tt + 1) as f64);
                    }
                }
            }
            if need <= maxc {
                let mut sel: Vec<usize> = Vec::new();
                let mut score = 0i32;
                if tot < cand_len {
                    // C1: votes minor by minor (seeds start the candidates)
                    let mut tt = 0usize;
                    loop {
                        let mut walked = 0;
                        let a = pc[lst[tt] as usize];
                        let mut q = self.mn_head(m0, lst[tt] as usize);
                        while q != NIL {
                            let nd = self.nd[q as usize];
                            q = mn_next(&nd);
                            walked += 1;
                            if nd.col < 0 {
                                continue;
                            }
                            let i = maj_of(&nd);
                            let c = cnt[i];
                            if c < need {
                                if c < 0 {
                                    let r = -nd.val / a;
                                    if r <= 0.0 {
                                        ratio_vote(r, &mut mn[i], &mut cn[i], tol, poison);
                                    } else {
                                        ratio_vote(r, &mut mult[i], &mut cp[i], tol, poison);
                                    }
                                } else if tt <= s {
                                    cnt[i] = 0;
                                    miss[i] = 0;
                                }
                            } else if tt <= s {
                                let r = -nd.val / a;
                                if r <= 0.0 {
                                    mult[i] = 1e100;
                                    cp[i] = 0;
                                    mn[i] = r;
                                } else {
                                    mn[i] = 1e100;
                                    cn[i] = 0;
                                    mult[i] = r;
                                }
                                if c == k as i32 {
                                    miss[i] = -1;
                                } else if miss[i] == 0 {
                                    if mm < 2 && (k as i32) < maj_cnt(self, i) {
                                        cnt[i] = 0;
                                        continue;
                                    }
                                    miss[i] = lst[k - 1] + 1;
                                }
                                cnt[i] = -1;
                                if r <= 0.0 { cn[i] = -1 } else { cp[i] = -1 }
                            }
                        }
                        tt += 1;
                        let mut w1 = mu * 5.0 * walked as f64 + self.work;
                        if tt == s + 1 {
                            w1 = w1 + (tt as f64 + tt as f64) * mu;
                        }
                        if tt > s && k <= tt {
                            self.work = w1 + mu * 2.0 * ((tt - s) as f64 - 1.0);
                            break;
                        }
                        self.work = w1;
                    }
                } else {
                    // C2: each candidate votes over its own list
                    if !marked {
                        for &mi in &lst {
                            stamp[mi as usize] = 1;
                        }
                        self.work = (k as f64 + k as f64) * mu + self.work;
                        marked = true;
                    }
                    for tt in 0..=s {
                        let mut walked = 0;
                        let mut q = self.mn_head(m0, lst[tt] as usize);
                        while q != NIL {
                            let nd = self.nd[q as usize];
                            q = mn_next(&nd);
                            walked += 1;
                            if nd.col < 0 {
                                continue;
                            }
                            let i = maj_of(&nd);
                            let c = cnt[i];
                            if c < 0 {
                                continue;
                            }
                            if c < need {
                                cnt[i] = 0;
                                miss[i] = 0;
                                continue;
                            }
                            if c == k as i32 {
                                miss[i] = -1;
                            } else if miss[i] == 0 {
                                if mm < 2 && (k as i32) < maj_cnt(self, i) {
                                    cnt[i] = 0;
                                    continue;
                                }
                                miss[i] = lst[k - 1] + 1;
                            }
                            cnt[i] = -1;
                            cp[i] = 0;
                            mult[i] = 1e100;
                            cn[i] = 0;
                            mn[i] = 1e100;
                            let mut w2 = 0;
                            let mut r = self.mj_head(m0, i);
                            while r != NIL {
                                let n2 = self.nd[r as usize];
                                if n2.col >= 0 {
                                    let mi = min_of(&n2);
                                    if stamp[mi] != 0 {
                                        let x = -n2.val / pc[mi];
                                        if 0.0 < x {
                                            ratio_vote(x, &mut mult[i], &mut cp[i], tol, poison);
                                        } else {
                                            ratio_vote(x, &mut mn[i], &mut cn[i], tol, poison);
                                        }
                                    }
                                }
                                w2 += 1;
                                r = mj_next(&n2);
                            }
                            self.work = mu * 5.0 * w2 as f64 + self.work;
                        }
                        self.work = mu * 5.0 * walked as f64 + self.work;
                        if tt == s {
                            self.work = self.work + mu * ((tt + 1) as f64 + (tt + 1) as f64);
                        }
                    }
                }
                // selection over the seed-reached majors
                for tt in 0..=s {
                    let mut walked = 0;
                    let mut q = self.mn_head(m0, lst[tt] as usize);
                    while q != NIL {
                        let nd = self.nd[q as usize];
                        q = mn_next(&nd);
                        walked += 1;
                        if nd.col < 0 {
                            continue;
                        }
                        let i = maj_of(&nd);
                        let c = cnt[i];
                        if c >= 1 {
                            continue;
                        }
                        let ok = c < 0 && self.cancel_pick(p, m0, mm, pv, i, k, &pc, &mut cnt, &miss, &cp, &cn,
                                                           &mut mult, &mn, &mut sgn, poison);
                        if ok {
                            cnt[i] = -cnt[i];
                            sel.push(i);
                            score += cnt[i] - (0 < miss[i]) as i32;
                        } else {
                            miss[i] = 0;
                            cnt[i] = 0;
                        }
                    }
                    self.work = mu * 4.0 * walked as f64 + self.work;
                    if tt == s {
                        self.work = self.work + mu * ((tt + 1) as f64 + (tt + 1) as f64);
                    }
                }
                if !sel.is_empty() {
                    let mut skip = false;
                    if m0 {
                        self.mark_cols_of_row(pv);
                    } else if score < 5 {
                        for &i in &sel {
                            cnt[i] = 0;
                            miss[i] = 0;
                        }
                        self.work = mu * 3.0 * sel.len() as f64 + self.work;
                        skip = true;
                    } else {
                        self.mark_rows_of_col(pv);
                    }
                    if !skip {
                        let rc = self.cancel_apply(m0, pv, &sel, k, tot, &lst, &pc, &mut cnt, &mut miss, &mut stamp,
                                                   &mut marked, &mult, &mut rmark, &mut rlist);
                        if rc != 0 {
                            return rc;
                        }
                        ncancel += sel.len() as i32;
                    }
                }
            } else {
                for tt in 0..=s {
                    let mut walked = 0;
                    let mut q = self.mn_head(m0, lst[tt] as usize);
                    while q != NIL {
                        let nd = self.nd[q as usize];
                        if nd.col >= 0 {
                            let i = maj_of(&nd);
                            cnt[i] = 0;
                            miss[i] = 0;
                        }
                        walked += 1;
                        q = mn_next(&nd);
                    }
                    self.work = mu * 4.0 * walked as f64 + self.work;
                    if tt == s {
                        self.work = self.work + mu * ((tt + 1) as f64 + (tt + 1) as f64);
                    }
                }
            }
            if marked {
                for &mi in &lst {
                    stamp[mi as usize] = 0;
                }
                self.work = (k as f64 + k as f64) * mu + self.work;
            }
            idx += 1;
        }
        self.work = self.work + 3.0 * idx as f64 * mu;
        self.coefs_changed += ncancel;
        for &r in &rlist {
            rmark[r as usize] = false;
        }
        self.work = (rlist.len() as f64 + rlist.len() as f64) * mu + self.work;
        self.activity(Some(&rlist));
        0
    }

    /// Selection of candidate `i` (count < 0): pick the multiplier hypothesis, then the fill-in and integrality
    /// rules. On success the count stays negative (the caller flips it); `mult[i]` holds the multiplier.
    #[allow(clippy::too_many_arguments)]
    fn cancel_pick(&mut self, p: &Params, m0: bool, mm: i32, pv: usize, i: usize, k: usize, pc: &[f64],
                   cnt: &mut [i32], miss: &[i32], cp: &[i32], cn: &[i32], mult: &mut [f64], mn: &[f64],
                   sgn: &mut [u32], poison: i32) -> bool {
        let (m_n, c_n) = (mn[i], cn[i]);
        let neg = if m_n == 1e100 || c_n <= poison {
            false
        } else {
            let (m_p, c_p) = (mult[i], cp[i]);
            if m_p == 1e100 || c_p <= poison {
                true
            } else {
                let by_counts = |tie: bool| if c_p < c_n { false } else if c_n < c_p { true } else { tie };
                if p.precancelselect == -1 {
                    if -m_n < m_p * 10.0 {
                        if m_p < m_n * -10.0 { by_counts(-m_n < m_p) } else { true }
                    } else {
                        false
                    }
                } else if p.precancelselect != 0 {
                    by_counts(-m_n < m_p)
                } else {
                    -m_n < m_p
                }
            }
        };
        if neg {
            cnt[i] = c_n;
            mult[i] = m_n;
        } else {
            cnt[i] = cp[i];
        }
        let c = cnt[i];
        if !(c < 0 && -(self.nd_dim(m0) as i32) <= c) {
            return false;
        }
        let mc = if m0 { self.row_cnt[i] } else { self.col_cnt[i] };
        if 0 < miss[i] {
            if mm == 1 {
                if 1 < mc + c {
                    return false;
                }
            } else if mm == 2 && 1 < c + mc && (k < 21 || 1 < k as i32 + c) {
                return false;
            }
            if c == -1 {
                return false;
            }
        }
        let x = mult[i];
        if !(x.abs() <= 1000.0 && 0.001 <= x.abs()) {
            return false;
        }
        let qm = miss[i] - 1;
        if m0 {
            if 2 < self.row_cnt[i] + c + (qm >= 0) as i32 && 1 < self.row_cnt[pv] + c && qm >= 0 {
                let q = qm as usize;
                if self.vtype[q] != b'B' {
                    return false;
                }
                let g = self.sign_class(sgn, i);
                let y = pc[q] * x;
                return if g & 2 != 0 { 0.0 < y } else if g & 4 != 0 { y < 0.0 } else { true };
            }
            return true;
        }
        // mode 1: majors are columns, the missing minor is a row. Steps: 0 reject, 1 accept, 2 sign rule on the
        // fill row, 3 binary pivot rule (LAB_004dc622), 4 short fill rows (LAB_004dc1ee)
        let tp = self.vtype[pv];
        let mut step = if tp != b'C' {
            if !(self.vtype[i] != b'C' && x - (x + 1e-10).floor() <= 1e-10) {
                return false;
            }
            if mm < 3 { 4 } else if qm < 0 { 1 } else if tp != b'B' { 2 } else { 3 }
        } else if mm < 3 {
            4
        } else if qm >= 0 {
            2
        } else {
            1
        };
        if step == 4 {
            step = if qm < 0 { 1 } else if self.row_cnt[qm as usize] < 3 { 0 } else if tp == b'B' { 3 } else { 2 };
        }
        if step == 3 {
            let q = qm as usize;
            step = if pc[q].abs() + self.act_min[q] < -1e-10 { 2 } else { 0 };
        }
        if step == 2 {
            let q = qm as usize;
            let g = self.sign_class(sgn, q);
            let y = pc[q] * x;
            step = if g & 2 != 0 { (!(y <= 0.0)) as i32 } else { (!(g & 4 != 0 && 0.0 <= y)) as i32 };
        }
        if step == 0 {
            return false;
        }
        if tp != b'C' {
            mult[i] = (mult[i] + 0.5).floor();
        }
        true
    }

    fn nd_dim(&self, m0: bool) -> usize {
        if m0 { self.n } else { self.m }
    }

    /// Apply the cancellations of pivot `pv` to the selected majors: rhs (objective) updates and marks, mode 1
    /// bound shifts of the pivot column with records, then the node updates and fill-ins.
    #[allow(clippy::too_many_arguments)]
    fn cancel_apply(&mut self, m0: bool, pv: usize, sel: &[usize], k: usize, tot: i64, lst: &[i32],
                    pc: &[f64], cnt: &mut [i32], miss: &mut [i32], stamp: &mut [i32], marked: &mut bool,
                    mult: &[f64], rmark: &mut [bool], rlist: &mut Vec<i32>) -> i32 {
        let mu = self.work_mult;
        let mut sumlen: i64 = 0;
        for &i in sel {
            let x = mult[i];
            sumlen += if m0 { self.row_cnt[i] } else { self.col_cnt[i] } as i64;
            if m0 {
                self.rhs[i] = self.rhs[pv] * x + self.rhs[i];
                if !rmark[i] {
                    rmark[i] = true;
                    rlist.push(i as i32);
                }
                self.tracker.mark_row(i as i32);
                continue;
            }
            self.obj[i] = self.obj[pv] * x + self.obj[i];
            if self.vtype[pv] == b'B' {
                // `5b91ad`: the pivot's conflict-graph edges go at the next merge
                if self.cg.flag.len() != self.n {
                    self.cg.flag.resize(self.n, 0);
                }
                self.cg.flag[pv] = 1;
            }
            let (l, u) = (self.lb[pv], self.ub[pv]);
            let (nl, nu);
            if x <= 0.0 {
                nl = if l <= -1e30 || self.lb[i] <= -1e30 { -1e30 } else { l - self.lb[i] * x };
                nu = if 1e30 <= u || 1e30 <= self.ub[i] { 1e30 } else { u - self.ub[i] * x };
            } else {
                nl = if l <= -1e30 || 1e30 <= self.ub[i] { -1e30 } else { l - self.ub[i] * x };
                nu = if 1e30 <= u || !(-1e30 < self.lb[i]) { 1e30 } else { u - self.lb[i] * x };
            }
            self.lb[pv] = nl;
            self.ub[pv] = nu;
            let mut walked = 0;
            let mut q = self.col_head[pv];
            while q != NIL {
                let nd = self.nd[q as usize];
                q = nd.nc;
                walked += 1;
                if nd.col < 0 {
                    continue;
                }
                let r = nd.row as usize;
                if !(l == nl && u == nu) {
                    let a = nd.val;
                    self.contrib(r, a, l, false, false);
                    self.contrib(r, a, u, true, false);
                    self.contrib(r, a, nl, false, true);
                    self.contrib(r, a, nu, true, true);
                }
                if !rmark[r] {
                    rmark[r] = true;
                    rlist.push(r as i32);
                }
            }
            self.work = mu * 6.0 * walked as f64 + self.work;
            if self.vtype[pv] != b'C' {
                self.vtype[pv] = if self.lb[pv] == 0.0 && self.ub[pv] == 1.0 { b'B' } else { b'I' };
            }
            self.push_t10(pv as i32, i as i32, 1.0, x, 0.0);
            self.tracker.mark_col(i as i32);
        }
        self.work = mu * 5.0 * sel.len() as f64 + self.work;
        if sumlen as f64 * 6.0 <= k as f64 * 3.0 + tot as f64 * 6.0 {
            if !*marked {
                for &mi in lst {
                    stamp[mi as usize] = 1;
                }
                self.work = (k as f64 + k as f64) * mu + self.work;
                *marked = true;
            }
            for &i in sel {
                let mut walked = 0;
                let mut q = self.mj_head(m0, i);
                while q != NIL {
                    let at = q as usize;
                    let nd = self.nd[at];
                    q = if m0 { nd.nr } else { nd.nc };
                    if nd.col >= 0 {
                        let mi = if m0 { nd.col } else { nd.row } as usize;
                        if stamp[mi] != 0 {
                            self.cancel_add(m0, at, mult[i] * pc[mi]);
                        }
                    }
                    walked += 1;
                }
                self.work = mu * 6.0 * walked as f64 + self.work;
                cnt[i] = 0;
                if 0 < miss[i] {
                    let q = (miss[i] - 1) as usize;
                    self.cancel_fill(m0, i, q, mult[i] * pc[q]);
                }
                miss[i] = 0;
            }
            self.work = mu * 3.0 * sel.len() as f64 + self.work;
        } else {
            for &mi in lst {
                let mi = mi as usize;
                let a = pc[mi];
                let mut walked = 0;
                let mut q = self.mn_head(m0, mi);
                while q != NIL {
                    let at = q as usize;
                    let nd = self.nd[at];
                    q = if m0 { nd.nc } else { nd.nr };
                    if nd.col >= 0 {
                        let i = if m0 { nd.row } else { nd.col } as usize;
                        if cnt[i] != 0 {
                            self.cancel_add(m0, at, mult[i] * a);
                        }
                    }
                    walked += 1;
                }
                self.work = mu * 6.0 * walked as f64 + self.work;
            }
            self.work = self.work + k as f64 * (mu * 3.0);
            for &i in sel {
                cnt[i] = 0;
                if 0 < miss[i] {
                    let q = (miss[i] - 1) as usize;
                    self.cancel_fill(m0, i, q, mult[i] * pc[q]);
                }
                miss[i] = 0;
            }
            self.work = self.work + (mu * 3.0) * sel.len() as f64;
        }
        0
    }

    /// `379173`: column j's upper bound is implied by one of its rows (or infinite). W += (μ·8)·nodes before it.
    fn ub_implied(&mut self, p: &Params, j: usize) -> bool {
        let u = self.ub[j];
        if !(u < INF) {
            return true;
        }
        let l = self.lb[j];
        let d8 = if -l <= INF && INF != -l { -l } else { 0.0 };
        let lfree = (INF <= -l) as i32;
        let mut walked = 0;
        let mut q = self.col_head[j];
        while q != NIL {
            let nd = self.nd[q as usize];
            if nd.col >= 0 && p.feastol <= nd.val.abs() {
                let r = nd.row as usize;
                let a = nd.val;
                let v = if 0.0 < -a || -a == 0.0 {
                    if self.sense[r] == b'=' && lfree == self.inf_max[r] { Some(self.act_max[r] / -a) } else { None }
                } else if lfree == self.inf_min[r] {
                    Some(self.act_min[r] / -a)
                } else {
                    None
                };
                if let Some(v) = v {
                    if -u - p.feastol <= d8 - v {
                        self.work = self.work_mult * 8.0 * walked as f64 + self.work;
                        return true;
                    }
                }
            }
            walked += 1;
            q = nd.nc;
        }
        self.work = self.work_mult * 8.0 * walked as f64 + self.work;
        false
    }

    /// `3790d9`: column j's lower bound is implied by one of its rows (or infinite). W += (μ·8)·nodes before it.
    fn lb_implied(&mut self, p: &Params, j: usize) -> bool {
        let l = self.lb[j];
        if !(-INF < l) {
            return true;
        }
        let u = self.ub[j];
        let d10 = if u < INF { u } else { 0.0 };
        let ufree = (INF <= u) as i32;
        let mut walked = 0;
        let mut q = self.col_head[j];
        while q != NIL {
            let nd = self.nd[q as usize];
            if nd.col >= 0 && p.feastol <= nd.val.abs() {
                let r = nd.row as usize;
                let a = nd.val;
                let v = if 0.0 <= a {
                    if self.sense[r] == b'=' && ufree == self.inf_max[r] { Some(self.act_max[r]) } else { None }
                } else if ufree == self.inf_min[r] {
                    Some(self.act_min[r])
                } else {
                    None
                };
                if let Some(v) = v {
                    if l - p.feastol <= d10 - v / a {
                        self.work = self.work_mult * 8.0 * walked as f64 + self.work;
                        return true;
                    }
                }
            }
            walked += 1;
            q = nd.nc;
        }
        self.work = self.work_mult * 8.0 * walked as f64 + self.work;
        false
    }
}

