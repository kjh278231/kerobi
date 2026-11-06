//! `45feaa` two-row reductions with its LP `45d7ec` (docs/m2_red_outer_c.md §2–3): for a row i and each row r that
//! shares enough columns with it, bound r's activity over the shared columns by the LP "shared part of r subject to
//! row i", then find r infeasible, redundant (removed), or use the bound to tighten the bounds of r's other columns
//! (and, for `<` rows, the coefficients of its integer columns at lower bound 0).
use crate::driver::Timer;
use crate::helpers::{maxsd, transpose_csr};
use crate::probe::TimerView;
use crate::state::{State, INF, NIL};
use crate::Params;

const BIG: f64 = 1e100;

/// One side of `45d7ec`: a fractional knapsack in min form, min Σ r·w over taken weight ≥ b (`s` the value at the
/// start point, `rmax` the best ratio of the items that can give weight back, `inf` unbounded). `base`: index of
/// `w[0]` in its 16-byte aligned buffer (the vectorized sums peel one element when it is odd).
struct Side {
    s: f64,
    b: f64,
    rmax: f64,
    inf: bool,
    w: Vec<f64>,
    r: Vec<f64>,
    base: usize,
}

/// `addpd`/`subpd` sum of `p` into `acc` the way 45d7ec's partition step does it: scalar under 8 elements, else a
/// peel to 16-byte alignment, 4 two-lane accumulators over blocks of 8, ((l+m)+(n+o)) lane sum, scalar tail.
fn vsum(mut acc: f64, p: &[f64], addr: usize, sub: bool) -> f64 {
    let f = |x: f64, y: f64| if sub { x - y } else { x + y };
    let (cnt, peel) = (p.len(), addr & 1);
    let mut k0 = 0;
    if cnt >= 8 && cnt >= peel + 8 {
        let vend = cnt - ((cnt - peel) & 7);
        for &x in &p[..peel] {
            acc = f(acc, x);
        }
        let (mut l, mut m, mut n, mut o) = ([acc, 0.0], [0.0; 2], [0.0; 2], [0.0; 2]);
        for k in (peel..vend).step_by(8) {
            for t in 0..2 {
                l[t] = f(l[t], p[k + t]);
                m[t] = f(m[t], p[k + 2 + t]);
                n[t] = f(n[t], p[k + 4 + t]);
                o[t] = f(o[t], p[k + 6 + t]);
            }
        }
        let v = [(l[0] + m[0]) + (n[0] + o[0]), (l[1] + m[1]) + (n[1] + o[1])];
        acc = v[0] + v[1];
        k0 = vend;
    }
    for &x in &p[k0..] {
        acc = f(acc, x);
    }
    acc
}

/// The final sums of 45d7ec: sw += Σ w, sp += Σ w·r, vectorized like `vsum` (products chained in one accumulator).
fn vdot(mut sw: f64, mut sp: f64, w: &[f64], r: &[f64], addr: usize) -> (f64, f64) {
    let (cnt, peel) = (w.len(), addr & 1);
    let mut k0 = 0;
    if cnt >= 8 && cnt >= peel + 8 {
        let vend = cnt - ((cnt - peel) & 7);
        for q in 0..peel {
            sw += w[q];
            sp += w[q] * r[q];
        }
        let (mut a, mut b, mut p) = ([sw, 0.0], [0.0; 2], [sp, 0.0]);
        for k in (peel..vend).step_by(8) {
            for t in 0..2 {
                a[t] += w[k + t];
                p[t] = p[t] + w[k + t] * r[k + t];
                b[t] += w[k + 2 + t];
                p[t] = w[k + 2 + t] * r[k + 2 + t] + p[t];
                a[t] += w[k + 4 + t];
                p[t] = w[k + 4 + t] * r[k + 4 + t] + p[t];
                b[t] += w[k + 6 + t];
                p[t] = w[k + 6 + t] * r[k + 6 + t] + p[t];
            }
        }
        sp = p[0] + p[1];
        sw = (a[0] + b[0]) + (a[1] + b[1]);
        k0 = vend;
    }
    for k in k0..cnt {
        sw += w[k];
        sp += w[k] * r[k];
    }
    (sw, sp)
}

/// Solve one side: items whose ratio is at most `rmax` are taken outright, then a quickselect (Hoare partition,
/// middle pivot) finds the item where the taken weight reaches b. ±1e100: unbounded / infeasible.
fn knap(k: Side, work: &mut f64, mu: f64) -> f64 {
    if k.inf {
        return -BIG;
    }
    let (mut s, mut b, rmax) = (k.s, k.b, k.rmax);
    let (mut w, mut r) = (k.w, k.r);
    if rmax > 0.0 {
        let mut dst = 0;
        for src in 0..w.len() {
            let (rr, ww) = (r[src], w[src]);
            if rmax >= rr {
                if !(ww > 5e99) {
                    b -= ww;
                    s += rr * ww;
                    continue;
                }
                if rmax > rr {
                    *work = (src + src) as f64 * mu + *work;
                    return -BIG;
                }
            }
            w[dst] = ww;
            r[dst] = rr;
            dst += 1;
        }
        *work = (w.len() + w.len()) as f64 * mu + *work;
        if 1e-10 > b {
            return s + rmax * b;
        }
        w.truncate(dst);
        r.truncate(dst);
    } else if 1e-10 > b {
        return s;
    }
    let n = w.len() as i64;
    let mut acc = -b;
    let (mut lo, mut hi) = (0i64, n - 1);
    while lo < hi {
        let piv = r[((lo + hi) / 2) as usize];
        let (mut i, mut j) = (lo - 1, hi + 1);
        loop {
            j -= 1;
            while r[j as usize] > piv {
                j -= 1;
            }
            i += 1;
            while piv > r[i as usize] {
                i += 1;
            }
            if i >= j {
                break;
            }
            r.swap(i as usize, j as usize);
            w.swap(i as usize, j as usize);
        }
        if 0.0 > acc {
            if lo <= j {
                acc = vsum(acc, &w[lo as usize..=j as usize], k.base + lo as usize, false);
            }
        } else if j < hi {
            acc = vsum(acc, &w[(j + 1) as usize..=hi as usize], k.base + (j + 1) as usize, true);
        }
        if 0.0 > acc { lo = j + 1 } else { hi = j }
    }
    *work = (n + n) as f64 * mu + *work;
    let lo = lo as usize;
    if -1e-6 > acc {
        if lo >= w.len() {
            return BIG;
        }
        acc += w[lo];
        if -1e-6 > acc {
            return BIG;
        }
    }
    let (sw, mut sp) = vdot(-b, s, &w[..lo], &r[..lo], k.base);
    *work = *work + mu * (lo + lo) as f64;
    if lo < w.len() {
        sp -= sw * r[lo];
    }
    sp
}

/// Per-row CSR snapshot of `45feaa` (rows of the list with more than one entry) and its column transpose.
struct Csr {
    rb: Vec<usize>,
    re: Vec<usize>,
    ci: Vec<usize>,
    cv: Vec<f64>,
    cb: Vec<usize>,
    cn: Vec<usize>,
    ri: Vec<usize>,
    rv: Vec<f64>,
}

/// Pending bound candidates of the current row i (`local_100`/`local_108`, list `local_110`, marks st+0x3a8).
#[derive(Default)]
struct Bnds {
    lo: Vec<f64>,
    up: Vec<f64>,
    list: Vec<usize>,
    mark: Vec<bool>,
}

impl State {
    /// `45d7ec(b, INF, st, k, c, a, cols, lb, ub, …, n, &hi, &lo)`: max and min of c·x over the columns `cols`
    /// subject to a·x ≥ b and their bounds (1e100 / -1e100 for unbounded, the opposite for infeasible).
    fn two_row_lp(&mut self, b: f64, c: &[f64], a: &[f64], cols: &[usize]) -> (f64, f64) {
        let (mut s1, mut b1, mut s2, mut b2) = (0.0, b, 0.0, b);
        let (mut rmax1, mut rmax2, mut inf1, mut inf2) = (0.0, 0.0, false, false);
        let (mut w1, mut r1, mut w2, mut r2) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
        for t in 0..c.len() {
            let (cc, aa, j) = (c[t], a[t], cols[t]);
            let (l, u) = (self.lb[j], self.ub[j]);
            if cc > 0.0 {
                if aa > 0.0 {
                    if l > -INF {
                        b1 -= aa * l;
                        s1 += cc * l;
                        w1.push(if u >= INF { BIG } else { (u - l) * aa });
                        r1.push(cc / aa);
                    } else {
                        if u >= INF { inf1 = true } else { b1 -= aa * u; s1 += u * cc }
                        rmax1 = maxsd(cc / aa, rmax1);
                    }
                    if INF > u { s2 += cc * u; b2 -= aa * u } else { inf2 = true }
                } else {
                    if l > -INF { s1 += cc * l; b1 -= aa * l } else { inf1 = true }
                    if INF > u {
                        s2 += u * cc;
                        b2 -= aa * u;
                        w2.push(if -INF < l { (-(u - l)) * aa } else { BIG });
                        r2.push((-cc) / aa);
                    } else {
                        if -INF < l { s2 += cc * l; b2 -= l * aa } else { inf2 = true }
                        rmax2 = maxsd((-cc) / aa, rmax2);
                    }
                }
            } else if aa > 0.0 {
                if INF > u { s1 += u * cc; b1 -= aa * u } else { inf1 = true }
                if l > -INF {
                    s2 += cc * l;
                    b2 -= aa * l;
                    w2.push(if u < INF { (u - l) * aa } else { BIG });
                    r2.push((-cc) / aa);
                } else {
                    if u < INF { s2 += u * cc; b2 -= u * aa } else { inf2 = true }
                    rmax2 = maxsd((-cc) / aa, rmax2);
                }
            } else {
                if INF > u {
                    s1 += u * cc;
                    b1 -= aa * u;
                    w1.push(if -INF < l { (-(u - l)) * aa } else { BIG });
                    r1.push(cc / aa);
                } else {
                    if -INF < l { s1 += cc * l; b1 -= aa * l } else { inf1 = true }
                    rmax1 = maxsd(cc / aa, rmax1);
                }
                if l > -INF { s2 += cc * l; b2 -= aa * l } else { inf2 = true }
            }
        }
        let mu = self.work_mult;
        self.work = (7.0 * mu) * c.len() as f64 + self.work;
        // the second side fills its buffer from the end down
        let nb = self.n - w2.len();
        w2.reverse();
        r2.reverse();
        let lo = knap(Side { s: s1, b: b1, rmax: rmax1, inf: inf1, w: w1, r: r1, base: 0 }, &mut self.work, mu);
        let hi = -knap(Side { s: -s2, b: b2, rmax: rmax2, inf: inf2, w: w2, r: r2, base: nb }, &mut self.work, mu);
        (hi, lo)
    }

    /// `45feaa(st, env, T7, mode)`.
    /// shortcut: the MIP flag st+0xfc is taken as set and the `3ba5ac` callback as a no-op (both always so here).
    pub fn two_rows(&mut self, p: &Params, mode: i32, mut t: Timer) -> i32 {
        t.start(self.work);
        let tv = TimerView { acc: t.acc(), running: t.running(), limit: t.limit };
        let (n, m, mu) = (self.n, self.m, self.work_mult);
        let mut wchk = self.work;
        let list = self.adjacent2_rows();
        let lim = if mode == 1 {
            if m < 257 { 32 } else { let s = (m as f64).sqrt(); (s + s) as i32 }
        } else {
            let s = (self.nnz as f64).sqrt();
            (s + s) as i32 + 32
        };
        let mut rlen = vec![0usize; m];
        let mut total = 0i64;
        for &i in &list {
            let c = self.row_cnt[i as usize];
            if c > 1 {
                rlen[i as usize] = c as usize;
                total += c as i64;
            }
        }
        self.work = (list.len() as f64 * 3.0) * mu + self.work;
        if !list.is_empty() && total > 2_000_000_000 {
            return 0;
        }
        // row CSR of the listed rows (live nodes in row order), then its transpose (`7722f`)
        let (mut rb, mut ci, mut cv) = (vec![0usize; m + 1], Vec::new(), Vec::new());
        for i in 0..m {
            rb[i] = ci.len();
            if rlen[i] < 1 {
                continue;
            }
            let mut q = self.row_head[i];
            let mut walked = 0;
            while q != NIL {
                let nd = self.nd[q as usize];
                if nd.col >= 0 {
                    ci.push(nd.col as usize);
                    cv.push(nd.val);
                }
                walked += 1;
                q = nd.nr;
            }
            self.work = (walked as f64 * 5.0) * mu + self.work;
        }
        rb[m] = ci.len();
        self.work = self.work + (m as f64 * 3.0) * mu;
        let (cb, cn, ri, rv) = transpose_csr(n, &rb[..m], &rlen, &ci, &cv, &mut self.work);
        let re: Vec<usize> = (0..m).map(|i| rb[i] + rlen[i]).collect();
        self.work = (m as f64 * 3.0) * mu + self.work;
        let mut g = Csr { rb, re, ci, cv, cb, cn, ri, rv };
        // (45feaa sizes its shared-entry buffers by the largest Σ col_cnt over a row; only the work stays)
        for &i in &list {
            let i = i as usize;
            if self.row_cnt[i] > 1 {
                self.work = ((g.re[i] - g.rb[i]) as f64 * 2.0) * mu + self.work;
            }
        }
        self.work = self.work + (list.len() as f64 * 4.0) * mu;
        let mut ovl = vec![0i32; m];
        let mut score = if mode <= 1 { Some(vec![0.0f64; m]) } else { None };
        let mut dense = vec![0.0f64; n];
        let mut bn = Bnds { lo: vec![0.0; n], up: vec![0.0; n], list: Vec::new(), mark: vec![false; n] };
        let mut newc = vec![0.0f64; n];
        let mut nrem = 0;
        let mut rc = 0;
        'rows: for &i in &list {
            if tv.over(self.work) {
                break;
            }
            if 1e7 < self.work - wchk {
                self.budget_check();
                if self.prelimit < 0 {
                    break;
                }
                wchk = self.work;
            }
            let i = i as usize;
            if !(((self.row_cnt[i] - 2) as u32) < 999_999) {
                continue;
            }
            // drop columns removed since the snapshot; a tiny coefficient skips the row
            let mut k = g.rb[i];
            let mut bad = false;
            for e in g.rb[i]..g.re[i] {
                let c = g.ci[e];
                if self.col_cnt[c] > 0 {
                    g.cv[k] = g.cv[e];
                    g.ci[k] = c;
                    if g.cv[e].abs() < 1e-6 {
                        bad = true;
                    }
                    k += 1;
                }
            }
            self.work = ((g.re[i] - g.rb[i]) as f64 * 3.0) * mu + self.work;
            g.re[i] = k;
            if bad {
                continue;
            }
            match self.two_rows_row(p, i, lim, &mut g, &mut ovl, &mut score, &mut dense, &mut bn, &mut newc, &mut nrem) {
                0 => {}
                r => {
                    rc = r;
                    break 'rows;
                }
            }
        }
        self.rows_removed += nrem;
        rc
    }

    /// The work of `45feaa` for one row i (after its CSR was compacted). Nonzero = status (3 infeasible).
    #[allow(clippy::too_many_arguments)]
    fn two_rows_row(&mut self, p: &Params, i: usize, lim: i32, g: &mut Csr, ovl: &mut [i32],
                    score: &mut Option<Vec<f64>>, dense: &mut [f64], bn: &mut Bnds, newc: &mut [f64],
                    nrem: &mut i32) -> i32 {
        let mu = self.work_mult;
        let (ft, tol) = (p.feastol, p.feastol * 100.0);
        let (rbi, rei) = (g.rb[i], g.re[i]);
        let len = rei - rbi;
        // candidate rows: rows sharing a column of i whose column is short enough
        let mut cand: Vec<usize> = Vec::new();
        let mut csum = 0i64;
        for e in rbi..rei {
            let c = g.ci[e];
            csum += self.col_cnt[c] as i64;
            dense[c] = g.cv[e];
            if self.col_cnt[c] > lim {
                continue;
            }
            let (l, u) = (self.lb[c], self.ub[c]);
            let range = if -INF < l {
                if u < INF { u - l } else { INF }
            } else if u < INF {
                INF
            } else {
                INF + INF
            };
            let (b0, b1) = (g.cb[c], g.cb[c] + g.cn[c]);
            for q in b0..b1 {
                let r = g.ri[q];
                if r != i && 1 < self.row_cnt[r] && self.row_cnt[r] < 1_000_001 {
                    if ovl[r] == 0 {
                        cand.push(r);
                    }
                    ovl[r] += 1;
                    if let Some(sc) = score.as_mut() {
                        if range < INF {
                            sc[r] = g.rv[q].abs() * range + sc[r];
                        } else {
                            sc[r] = sc[r] + (range / INF) * 1e100;
                        }
                    }
                }
            }
            self.work = ((b1 - b0) as f64 * 6.0) * mu + self.work;
        }
        self.work = self.work + (len as f64 * 8.0) * mu;
        // keep rows sharing at least two columns and half of the shorter row, whose shared columns can explain
        // enough of the row's activity range
        let mut kept: Vec<usize> = Vec::new();
        let mut sumcnt = 0i64;
        if !cand.is_empty() {
            for &r in &cand {
                let o = ovl[r];
                let mut drop = o < 2 || o * 2 < self.row_cnt[r].min(self.row_cnt[i]);
                if !drop {
                    if let Some(sc) = score.as_ref() {
                        if o < self.row_cnt[r] - 1 {
                            let s = sc[r];
                            if (self.inf_min[r] > 0 || self.act_min[r] + s < -1e-10)
                                && (self.sense[r] != b'=' || self.inf_max[r] > 0 || 1e-10 < self.act_max[r] - s)
                            {
                                let ni = self.inf_min[r] + self.inf_max[r];
                                let slack = if ni < 1 { self.act_max[r] - self.act_min[r] } else { ni as f64 * 1e100 };
                                drop = s < slack * 0.8;
                            }
                        }
                    }
                }
                if drop {
                    ovl[r] = 0;
                    if let Some(sc) = score.as_mut() {
                        sc[r] = 0.0;
                    }
                } else {
                    sumcnt += self.row_cnt[r] as i64;
                    kept.push(r);
                }
            }
            self.work = self.work + (cand.len() as f64 * 10.0) * mu;
        }
        // shared entries per kept row, in reverse fill order (only when the kept rows are long)
        let bucketed = !kept.is_empty() && sumcnt > csum * 2;
        let (mut bcol, mut bvr, mut bvi) = (Vec::new(), Vec::new(), Vec::new());
        let mut seg = vec![0usize; 0];
        if bucketed {
            let mut bucket = vec![0usize; self.m];
            for e in rbi..rei {
                let c = g.ci[e];
                let (b0, b1) = (g.cb[c], g.cb[c] + g.cn[c]);
                for q in b0..b1 {
                    if ovl[g.ri[q]] != 0 {
                        bucket[g.ri[q]] += 1;
                    }
                }
                self.work = ((b1 - b0) as f64 * 3.0) * mu + self.work;
            }
            self.work = (len as f64 * 3.0) * mu + self.work;
            let mut acc = bucket[kept[0]];
            for &r in &kept[1..] {
                acc += bucket[r];
                bucket[r] = acc;
            }
            self.work = (kept.len() as f64 + kept.len() as f64) * mu + self.work;
            let total = acc;
            bcol = vec![0usize; total];
            bvr = vec![0.0; total];
            bvi = vec![0.0; total];
            for e in rbi..rei {
                let c = g.ci[e];
                let (b0, b1) = (g.cb[c], g.cb[c] + g.cn[c]);
                for q in b0..b1 {
                    let r = g.ri[q];
                    if ovl[r] != 0 {
                        bucket[r] -= 1;
                        let at = bucket[r];
                        bvr[at] = g.rv[q];
                        bvi[at] = -dense[c];
                        bcol[at] = c;
                    }
                }
                self.work = ((b1 - b0) as f64 * 8.0) * mu + self.work;
            }
            self.work = (len as f64 * 3.0) * mu + self.work;
            seg = kept.iter().map(|&r| bucket[r]).collect();
            seg.push(total);
        }
        let mut modrows: Vec<usize> = Vec::new();
        for (t, &r) in kept.iter().enumerate() {
            ovl[r] = 0;
            if let Some(sc) = score.as_mut() {
                sc[r] = 0.0;
            }
            // activities of the shared part: r's (minR, maxR) and i's (minI, maxI), with infinite counts
            let (mut min_r, mut max_r, mut min_i, mut max_i) = (0.0f64, 0.0f64, 0.0f64, 0.0f64);
            let (mut nmin_r, mut nmax_r, mut nmin_i, mut nmax_i) = (0, 0, 0, 0);
            let (mut sc, mut sr, mut si): (Vec<usize>, Vec<f64>, Vec<f64>);
            let mut nonov: Option<(Vec<usize>, Vec<f64>)> = None;
            if !bucketed {
                (sc, sr, si) = (Vec::new(), Vec::new(), Vec::new());
                let (mut oc, mut ov) = (Vec::new(), Vec::new());
                let mut e = g.rb[r];
                while e < g.re[r] {
                    let c = g.ci[e];
                    if self.col_cnt[c] > 0 {
                        let a = g.cv[e];
                        if dense[c] == 0.0 {
                            oc.push(c);
                            ov.push(a);
                        } else {
                            sc.push(c);
                            sr.push(a);
                            si.push(-dense[c]);
                            add_act(self.lb[c], self.ub[c], a, dense[c], &mut [&mut min_r, &mut max_r, &mut min_i, &mut max_i],
                                    &mut [&mut nmin_r, &mut nmax_r, &mut nmin_i, &mut nmax_i]);
                        }
                    }
                    e += 1;
                }
                self.work = ((e - g.rb[r]) as f64 * 9.0) * mu + self.work;
                nonov = Some((oc, ov));
            } else {
                let (s0, s1) = (seg[t], seg[t + 1]);
                (sc, sr, si) = (bcol[s0..s1].to_vec(), bvr[s0..s1].to_vec(), bvi[s0..s1].to_vec());
                for q in 0..sc.len() {
                    let c = sc[q];
                    add_act(self.lb[c], self.ub[c], sr[q], -si[q], &mut [&mut min_r, &mut max_r, &mut min_i, &mut max_i],
                            &mut [&mut nmin_r, &mut nmax_r, &mut nmin_i, &mut nmax_i]);
                }
                self.work = (sc.len() as f64 * 5.0) * mu + self.work;
            }
            let mut removed = false;
            let mut nchg = 0;
            for dir in 0..2 {
                let b = if dir == 0 {
                    if nmin_i != self.inf_min[i] {
                        continue;
                    }
                    if nmax_i == 0 && nmin_i == 0 && !((tol - self.act_min[i]) + min_i <= max_i) {
                        continue;
                    }
                    self.act_min[i] - min_i
                } else {
                    if !(self.sense[i] == b'=' && !removed && nchg == 0 && nmax_i == self.inf_max[i]) {
                        continue;
                    }
                    if nmax_i == 0 && nmin_i == 0 && (max_i - tol) - self.act_max[i] < min_i {
                        continue;
                    }
                    for x in si.iter_mut() {
                        *x = -*x;
                    }
                    self.work = si.len() as f64 * mu + self.work;
                    max_i - self.act_max[i]
                };
                let (hi, lo) = self.two_row_lp(b, &sr, &si, &sc);
                if !(-5e99 <= hi && lo <= 5e99) {
                    continue;
                }
                let eq = self.sense[r] == b'=';
                if eq && nmax_r == self.inf_max[r] && (self.act_max[r] - max_r) + hi < -tol {
                    self.bad_row = i as i32; // +0x47c gets r (not kept: nothing reads it here)
                    return 3;
                }
                if nmin_r == self.inf_min[r] && tol < (self.act_min[r] - min_r) + lo {
                    self.bad_row = i as i32;
                    return 3;
                }
                if !eq && nmax_r == self.inf_max[r] && (self.act_max[r] - max_r) + hi < 1e-10 {
                    // r is redundant given i
                    let keep = !(p.numeric_focus < 1 || sc.len() > 2) && {
                        let mut q = 0;
                        while q < sc.len() {
                            let a = sr[q].abs();
                            let c = sc[q];
                            if -1e-10 <= a / si[q].abs() - 100000.00000000001 && 1e-6 < a * (self.ub[c] - self.lb[c]) {
                                break;
                            }
                            q += 1;
                        }
                        self.work = (q as f64 * 5.0) * mu + self.work;
                        q < sc.len()
                    };
                    if !keep {
                        let mut q = self.row_head[r];
                        let mut walked = 0;
                        while q != NIL {
                            let nd = self.nd[q as usize];
                            if nd.col >= 0 {
                                self.tracker.mark_col(nd.col);
                                self.col_cnt[nd.col as usize] -= 1;
                                self.nd[q as usize].col = -1;
                            }
                            walked += 1;
                            q = nd.nr;
                        }
                        self.work = (walked as f64 * 3.0) * mu + self.work;
                        *nrem += 1;
                        self.row_cnt[r] = -2;
                        g.re[r] = g.rb[r];
                        removed = true;
                    }
                    continue;
                }
                // bounds of r's other columns
                let tol_u = maxsd(1.0, self.act_max[r]) * tol;
                if nmin_r == self.inf_min[r] {
                    let tol_l = maxsd(1.0, -self.act_min[r]);
                    if !(lo - tol_l * tol <= min_r) {
                        let (oc, ov) = self.nonoverlap(&mut nonov, g, r, dense);
                        let d = zap((min_r - self.act_min[r]) - lo);
                        for q in 0..oc.len() {
                            self.cand_bound(bn, oc[q], ov[q], d, true, ft);
                        }
                        self.work = (oc.len() as f64 * 9.0) * mu + self.work;
                    }
                } else if self.inf_min[r] == nmin_r + 1 {
                    let (oc, ov) = self.nonoverlap(&mut nonov, g, r, dense);
                    let d = zap((min_r - self.act_min[r]) - lo);
                    for q in 0..oc.len() {
                        self.cand_bound(bn, oc[q], ov[q], d, false, ft);
                    }
                    self.work = (oc.len() as f64 * 9.0) * mu + self.work;
                }
                if eq {
                    let one = self.inf_max[r] == nmax_r + 1;
                    if (nmax_r == self.inf_max[r] && !(max_r - tol_u <= hi)) || one {
                        let (oc, ov) = self.nonoverlap(&mut nonov, g, r, dense);
                        let d = zap((self.act_max[r] - max_r) + hi);
                        for q in 0..oc.len() {
                            self.cand_bound(bn, oc[q], -ov[q], d, !one, ft);
                        }
                        self.work = (oc.len() as f64 * 9.0) * mu + self.work;
                    }
                } else if nmax_r == self.inf_max[r] && hi < max_r - tol_u {
                    // coefficients of r's integer columns at lower bound 0
                    let (oc, mut ov) = self.nonoverlap(&mut nonov, g, r, dense);
                    let s = (self.act_max[r] - max_r) + hi;
                    for q in 0..oc.len() {
                        let c = oc[q];
                        if self.vtype[c] != b'C' && self.lb[c] == 0.0 {
                            let a = ov[q];
                            if a <= tol + s || self.ub[c] != 1.0 {
                                if tol + s < -a {
                                    ov[q] = a + -(s + a);
                                    newc[c] = if ov[q] == 0.0 { 1e-200 } else { ov[q] };
                                    nchg += 1;
                                }
                            } else {
                                ov[q] = a - (a - s);
                                self.rhs[r] = self.rhs[r] - (a - s);
                                newc[c] = if ov[q] == 0.0 { 1e-200 } else { ov[q] };
                                nchg += 1;
                            }
                        }
                    }
                    self.work = (oc.len() as f64 * 7.0) * mu + self.work;
                    nonov = Some((oc, ov));
                    if nchg != 0 {
                        modrows.push(r);
                        self.tracker.mark_row(r as i32);
                        self.rewrite_row(g, r, newc);
                    }
                    self.coefs_changed += nchg;
                }
            }
        }
        self.work = (kept.len() as f64 * 9.0) * mu + self.work;
        for e in rbi..rei {
            dense[g.ci[e]] = 0.0;
        }
        self.work = (len as f64 * 2.0) * mu + self.work;
        // apply the bound candidates; rows of changed columns get their activities recomputed
        let mut inmod = vec![false; if modrows.is_empty() && bn.list.is_empty() { 0 } else { self.m }];
        for &r in &modrows {
            inmod[r] = true;
        }
        self.work = (modrows.len() as f64 + modrows.len() as f64) * mu + self.work;
        let nb = bn.list.len();
        for q in 0..nb {
            let c = bn.list[q];
            let (nu, nl) = (bn.up[c], bn.lo[c]);
            bn.mark[c] = false;
            if tol + nu < nl {
                self.bad_col = c as i32;
                return 3;
            }
            let mut l = self.lb[c];
            let mut changed = false;
            if nl <= tol + l || nl <= -1e8 {
                if nu < self.ub[c] - tol && nu < 1e8 {
                    self.ub[c] = if l <= nu { nu } else { l };
                    changed = true;
                }
            } else {
                l = if self.ub[c] <= nl { self.ub[c] } else { nl };
                self.lb[c] = l;
                let nu = bn.up[c];
                if nu < self.ub[c] - tol && nu < 1e8 {
                    self.ub[c] = if l <= nu { nu } else { l };
                }
                changed = true;
            }
            if changed {
                self.bounds_tightened += 1;
                self.tracker.mark_col(c as i32);
                let mut qq = self.col_head[c];
                let mut walked = 0;
                while qq != NIL {
                    let nd = self.nd[qq as usize];
                    if nd.col >= 0 && !inmod[nd.row as usize] {
                        inmod[nd.row as usize] = true;
                        modrows.push(nd.row as usize);
                    }
                    walked += 1;
                    qq = nd.nc;
                }
                self.work = (walked as f64 * 5.0) * mu + self.work;
            }
            if self.ub[c] - 1e-10 < self.lb[c] {
                let r = self.fix_near(c, ft);
                if r != 0 {
                    return r;
                }
            }
        }
        bn.list.clear();
        self.work = (nb as f64 * 6.0) * mu + self.work;
        if !modrows.is_empty() {
            let rows: Vec<i32> = modrows.iter().map(|&r| r as i32).collect();
            self.activity(Some(&rows));
            self.work = (modrows.len() as f64 + modrows.len() as f64) * mu + self.work;
        }
        0
    }

    /// r's entries outside the shared columns (computed on first use in the bucketed mode, W += (μ·7)·entries).
    fn nonoverlap(&mut self, cache: &mut Option<(Vec<usize>, Vec<f64>)>, g: &Csr, r: usize, dense: &[f64])
        -> (Vec<usize>, Vec<f64>) {
        if let Some(x) = cache {
            return x.clone();
        }
        let (mut oc, mut ov) = (Vec::new(), Vec::new());
        for e in g.rb[r]..g.re[r] {
            let c = g.ci[e];
            if self.col_cnt[c] > 0 && dense[c] == 0.0 {
                oc.push(c);
                ov.push(g.cv[e]);
            }
        }
        self.work = ((g.re[r] - g.rb[r]) as f64 * 7.0) * self.work_mult + self.work;
        *cache = Some((oc.clone(), ov.clone()));
        (oc, ov)
    }

    /// A bound candidate for column c from `a·x_c ≤ d` (+ the other bound when `fin`, else only when that bound is
    /// infinite); integers rounded by FeasibilityTol.
    fn cand_bound(&self, bn: &mut Bnds, c: usize, a: f64, d: f64, fin: bool, ft: f64) {
        let int = self.vtype[c] != b'C';
        let new = |bn: &mut Bnds, c: usize| {
            if !bn.mark[c] {
                bn.list.push(c);
                bn.lo[c] = self.lb[c];
                bn.up[c] = self.ub[c];
                bn.mark[c] = true;
            }
        };
        if a > 1e-8 && (fin || !(-INF < self.lb[c])) {
            let mut v = if fin { d / a + self.lb[c] } else { d / a };
            if int {
                v = (v + ft).floor();
            }
            new(bn, c);
            bn.up[c] = if bn.up[c] <= v { bn.up[c] } else { v };
        } else if a < -1e-8 && (fin || INF <= self.ub[c]) {
            let mut v = if fin { d / a + self.ub[c] } else { d / a };
            if int {
                v = (v - ft).ceil();
            }
            new(bn, c);
            bn.lo[c] = if v <= bn.lo[c] { bn.lo[c] } else { v };
        }
    }

    /// Write r's changed coefficients (`newc`, 1e-200 = became zero) into its nodes and the snapshot; nodes that
    /// became (near) zero die. The snapshot row keeps only the entries looked at with `newc` set or equal.
    fn rewrite_row(&mut self, g: &mut Csr, r: usize, newc: &mut [f64]) {
        let mu = self.work_mult;
        let mut k = g.rb[r];
        let mut q = self.row_head[r];
        let mut walked = 0;
        while q != NIL {
            let nd = self.nd[q as usize];
            if nd.col >= 0 {
                let c = nd.col as usize;
                let (v, nc) = (nd.val, newc[c]);
                if (v - nc).abs() < 1e-13 {
                    g.ci[k] = c;
                    g.cv[k] = v;
                    k += 1;
                } else if nc.abs() <= 1e-10 {
                    if nc != 0.0 {
                        self.tracker.mark_col(c as i32);
                        self.nd[q as usize].col = -1;
                        self.col_cnt[c] -= 1;
                        self.row_cnt[r] -= 1;
                        let (b0, e0) = (g.cb[c], g.cb[c] + g.cn[c]);
                        let mut x = b0;
                        while x < e0 {
                            if g.ri[x] == r {
                                g.ri[x] = g.ri[e0 - 1];
                                g.rv[x] = g.rv[e0 - 1];
                                g.cn[c] -= 1;
                                break;
                            }
                            x += 1;
                        }
                        self.work = (x - b0) as f64 * mu + self.work;
                    }
                } else {
                    self.tracker.mark_col(c as i32);
                    g.ci[k] = c;
                    k += 1;
                    self.nd[q as usize].val = nc;
                    g.cv[k - 1] = nc;
                    let (b0, e0) = (g.cb[c], g.cb[c] + g.cn[c]);
                    let mut x = b0;
                    while x < e0 {
                        if g.ri[x] == r {
                            g.rv[x] = nc;
                            break;
                        }
                        x += 1;
                    }
                    self.work = (x - b0) as f64 * mu + self.work;
                }
                newc[c] = 0.0;
            }
            walked += 1;
            q = nd.nr;
        }
        self.work = (walked as f64 * 8.0) * mu + self.work;
        g.re[r] = k;
    }
}

/// |d| < 1e-10 counts as 0.
fn zap(d: f64) -> f64 {
    if d.abs() < 1e-10 { 0.0 } else { d }
}

/// Add column bounds [l, u] times (a_r, a_i) to the shared-part activities: s = [minR, maxR, minI, maxI], k = their
/// infinite counts.
fn add_act(l: f64, u: f64, ar: f64, ai: f64, s: &mut [&mut f64; 4], k: &mut [&mut i32; 4]) {
    if -INF < l {
        if 0.0 < ar { *s[0] += l * ar } else { *s[1] = l * ar + *s[1] }
        if ai <= 0.0 { *s[3] += l * ai } else { *s[2] += l * ai }
    } else {
        if 0.0 < ar { *k[0] += 1 } else { *k[1] += 1 }
        if 0.0 < ai { *k[2] += 1 } else { *k[3] += 1 }
    }
    if u < INF {
        if 0.0 < ar { *s[1] += ar * u } else { *s[0] = ar * u + *s[0] }
        if ai <= 0.0 { *s[2] += u * ai } else { *s[3] += u * ai }
    } else {
        if 0.0 < ar { *k[1] += 1 } else { *k[0] += 1 }
        if ai <= 0.0 { *k[2] += 1 } else { *k[3] += 1 }
    }
}

#[cfg(test)]
mod tests {
    use super::{knap, vsum, Side};

    #[test]
    fn knapsack_takes_cheapest_ratio_first() {
        // need 4: all of w=2 at ratio 1, then 2 of w=3 at ratio 2 -> 2 + 4
        let k = Side { s: 0.0, b: 4.0, rmax: 0.0, inf: false, w: vec![1.0, 2.0, 3.0], r: vec![3.0, 1.0, 2.0], base: 0 };
        let mut w = 0.0;
        assert_eq!(knap(k, &mut w, 1.0), 6.0);
        // infeasible: not enough weight
        let k = Side { s: 0.0, b: 7.0, rmax: 0.0, inf: false, w: vec![1.0, 2.0, 3.0], r: vec![3.0, 1.0, 2.0], base: 0 };
        assert_eq!(knap(k, &mut w, 1.0), 1e100);
    }

    #[test]
    fn vector_sum_matches_scalar_on_exact_values() {
        let p: Vec<f64> = (1..=19).map(|x| x as f64).collect();
        for addr in 0..2 {
            assert_eq!(vsum(0.5, &p, addr, false), 190.5);
            assert_eq!(vsum(0.5, &p, addr, true), -189.5);
        }
    }
}
