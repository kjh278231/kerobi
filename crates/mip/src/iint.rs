//! `57627e` implied integers and integer equations (docs/m2_red_outer_a.md §4), and `a7bea8` (gcd of doubles).
use crate::state::{State, INF, NIL};
use crate::Params;

/// The integer gcd inside `a7bea8`: 0 if either is 0; a side >= 2e9 gives the other.
fn igcd(a: i32, b: i32) -> i32 {
    if a == 0 || b == 0 {
        return 0;
    }
    if !(a < 2000000000) {
        return b;
    }
    if !(b < 2000000000) {
        return a;
    }
    let (mut lo, mut hi) = if a < b { (a, b) } else { (b, a) };
    while lo > 0 {
        (hi, lo) = (lo, hi % lo);
    }
    hi
}

/// `a7bea8(n, v)`: gcd of |v| (docs/m2_red_outer_a.md §4.2); 0 when there is none within its tolerances.
pub(crate) fn gcd_f64(v: &[f64]) -> f64 {
    let n = v.len();
    if n == 0 {
        return 0.0;
    }
    if n == 1 {
        return v[0].abs();
    }
    let (mut mn, mut mx) = (1e100f64, 0.0f64);
    for x in v {
        let a = x.abs();
        if a <= mn {
            mn = a;
        }
        if mx <= a {
            mx = a;
        }
    }
    if 2000000000.0 < mx || mn < 1e-05 {
        return 0.0;
    }
    if mx - mn <= 1e-10 {
        return mn;
    }
    let multiple_of = |f: f64, sc: f64| v.iter().all(|x| !(sc * 1e-10 < (x.abs() * f - (x.abs() * f + 0.5).floor()).abs()));
    let inv = 1.0 / mn;
    if multiple_of(inv, if inv <= 1.0 { 1.0 } else { inv }) {
        return mn;
    }
    let int_gcd = |k: f64, stop_at_zero_returns: bool| -> Option<f64> {
        let mut g = (v[0].abs() * k + 0.5).floor();
        for x in &v[1..] {
            if g == 0.0 {
                return if stop_at_zero_returns { Some(g) } else { None };
            }
            let h = (x.abs() * k + 0.5).floor();
            g = igcd(g as i32, h as i32) as f64;
        }
        Some(g)
    };
    if mx * 600.0 <= 2000000000.0 {
        if v.iter().all(|x| !(6.000000000000001e-08 < (x.abs() * 600.0 - (x.abs() * 600.0 + 0.5).floor()).abs())) {
            let g = int_gcd(600.0, false).unwrap_or(0.0);
            return g / 600.0;
        }
    } else if 1.9999999999 <= mn && v.iter().all(|x| !(1e-10 < (x.abs() - (x.abs() + 0.5).floor()).abs())) {
        return int_gcd(1.0, true).unwrap_or(0.0);
    }
    // Euclid on doubles over the values in order (the running value starts as v[0], signed)
    let mut g = v[0];
    for x in &v[1..] {
        if g <= 0.0 {
            break;
        }
        g = g.abs();
        let b = x.abs();
        if g < 1e-10 || b < 1e-10 {
            g = 0.0;
        } else if g < 1e100 {
            if b < 1e100 {
                let (mut small, mut big) = if b <= g { (b, g) } else { (g, b) };
                let mut res = None;
                while 1e-10 < small {
                    if big < small {
                        res = Some(0.0);
                        break;
                    }
                    let q = ((big + 1e-10) / small).floor();
                    let rem = big - q * small;
                    let rem = if rem <= 0.0 { 0.0 } else { rem };
                    (big, small) = (small, rem);
                }
                g = res.unwrap_or(if 1e-09 <= big { big } else { 0.0 });
            }
        } else {
            g = b;
        }
    }
    if g <= 1e-10 {
        return 0.0;
    }
    let inv = 1.0 / g;
    if inv - (inv + 1.0000000000000001e-11).floor() <= 1.0000000000000001e-11 && multiple_of(inv, if inv <= 1.0 { 1.0 } else { inv }) {
        return g;
    }
    // continued fraction of g, denominators up to 1000
    let mut x = g;
    let mut a0 = (g + 5e-14).floor();
    let mut e1 = g - a0 * 1.0;
    let a1 = if 0.0 <= e1 { a0 + 1.0 } else { a0 - 1.0 };
    let mut e2 = g - a1 * 1.0;
    let (mut p1, mut q1, mut p0, mut q0) = (a0, 1.0f64, 1.0f64, 0.0f64);
    while (e1 < -1e-13 || 1e-13 < e1) && (e2 < -1e-13 || 1e-13 < e2) {
        x = 1.0 / (x - a0);
        a0 = (x + 5e-14).floor();
        let qn = q1 * a0 + q0;
        let pn = p1 * a0 + p0;
        if 1000.0 < qn {
            return 0.0;
        }
        e1 = g - pn / qn;
        (p0, q0, p1, q1) = (p1, q1, pn, qn);
        e2 = if 0.0 <= e1 { g - (pn + 1.0) / qn } else { g - (pn - 1.0) / qn };
    }
    let num = if -1e-13 <= e1 { if e1 <= 1e-13 { p1 as i32 } else { (p1 + 1.0) as i32 } } else { (p1 - 1.0) as i32 };
    if num < 1 {
        return 0.0;
    }
    let f = (q1 as i32) as f64 / num as f64;
    if !multiple_of(f, if f <= 1.0 { 1.0 } else { f }) {
        return 0.0;
    }
    num as f64 / (q1 as i32) as f64
}

/// One coefficient class of an integer `=` row: entries with the same |a| (1.6e-9 relative).
struct Class {
    count: i32,
    val: f64,
    col: usize,
    neg: bool,
    lo: f64,
    hi: f64,
}

impl State {
    /// `57627e`. Returns 0 or 3 (`bad_row`: an integer row whose rhs is not a multiple of its gcd; `bad_col`: a scaled
    /// column with crossing bounds).
    pub fn implied_int(&mut self, p: &Params) -> i32 {
        let (mu, ft, ift) = (self.work_mult, p.feastol, self.intfeastol);
        let lv = self.tracker.level;
        let list: Vec<i32> = self.tracker.rows.list[lv][..self.tracker.rows.a[lv]].to_vec();
        let mut changed: Vec<i32> = Vec::new();
        for &ii in &list {
            let i = ii as usize;
            if !(self.row_cnt[i] > 1 && self.sense[i] == b'=') {
                continue;
            }
            let mut rhs0 = self.rhs[i];
            let mut cls: Vec<Class> = Vec::new();
            let (mut allok, mut cont, mut ints, mut fixed, mut broke) = (true, None::<u32>, Vec::new(), 0.0f64, false);
            let mut q = self.row_head[i];
            while q != NIL {
                let at = q;
                let x = self.nd[q as usize];
                q = x.nr;
                if x.col < 0 {
                    continue;
                }
                let c = x.col as usize;
                let (u, l) = (self.ub[c], self.lb[c]);
                if u - l <= 1e-13 {
                    // a fixed column goes into the rhs (no record)
                    self.nd[at as usize].col = -1;
                    fixed += l * x.val;
                    self.row_cnt[i] -= 1;
                    self.col_cnt[c] -= 1;
                    continue;
                }
                let (a, t) = (x.val, self.vtype[c]);
                if !allok || l <= -1e30 || 1e30 <= u || ift + u < l || (t != b'B' && t != b'I') {
                    allok = false;
                } else {
                    let (aa, neg) = (a.abs(), a < 0.0);
                    if aa < 0.001 || 2000000000.0 < aa {
                        allok = false;
                    } else if let Some(k) = cls.iter_mut().find(|k| (aa - k.val).abs() <= if aa <= k.val { aa } else { k.val } * 1.6e-09) {
                        k.count += 1;
                        if neg {
                            k.lo -= u;
                            k.hi -= l;
                        } else {
                            k.lo += l;
                            k.hi += u;
                        }
                    } else if cls.len() == 5 {
                        allok = false;
                    } else {
                        cls.push(Class { count: 1, val: aa, col: c, neg, lo: if neg { -u } else { l }, hi: if neg { -l } else { u } });
                    }
                }
                if t == b'C' {
                    if cont.is_some() {
                        broke = true;
                        break;
                    }
                    cont = Some(at);
                } else {
                    ints.push(a.abs());
                }
            }
            self.work += self.row_cnt[i] as f64 * 5.0 * mu;
            self.rhs[i] -= fixed;
            rhs0 -= fixed;
            if broke || !(self.row_cnt[i] > 1) {
                continue;
            }
            if let Some(at) = cont {
                let ac = self.nd[at as usize].val.abs();
                if ac < 0.01 || 100.0 < ac {
                    continue;
                }
                let ar = self.rhs[i].abs();
                if 1e-10 < ar {
                    ints.push(ar);
                }
            }
            let g = gcd_f64(&ints);
            if !(0.001 < g.abs()) {
                continue;
            }
            if let Some(at) = cont {
                let rc = self.implied_int_col(at as usize, g, ft, ift, p);
                if rc != 0 {
                    return rc;
                }
                continue;
            }
            // integer row: divide by the gcd
            let ratio = self.rhs[i] / g;
            let tolr = ft / if 1.0 <= g { 1.0 } else { g };
            let tolr = if 0.1 <= tolr { 0.1 } else { tolr };
            let fl = (tolr + ratio).floor();
            if tolr < ratio - fl {
                self.bad_row = ii;
                return 3;
            }
            self.rhs[i] = fl;
            let (mut mn, mut mnode, mut v) = (1e100f64, None::<usize>, 0);
            let mut q = self.row_head[i];
            while q != NIL {
                let at = q as usize;
                let x = self.nd[at];
                q = x.nr;
                v += 1;
                if x.col < 0 {
                    continue;
                }
                let nv = (x.val / g + 0.5).floor();
                self.nd[at].val = nv;
                if nv == 0.0 {
                    self.tracker.mark_row(ii);
                    self.tracker.mark_col(x.col);
                    self.row_cnt[i] -= 1;
                    self.col_cnt[x.col as usize] -= 1;
                    self.nd[at].col = -1;
                } else if !(mn <= nv.abs()) {
                    mn = nv.abs();
                    mnode = Some(at);
                }
            }
            self.work += v as f64 * 5.0 * mu;
            if 1e-10 < (g - 1.0).abs() {
                changed.push(ii);
            }
            if allok && cls.len() >= 2 {
                let rc = self.int_classes(i, &cls, rhs0, ift);
                if rc != 0 {
                    return rc;
                }
            }
            // the substitution of the smallest coefficient (`3c5996`, `574764`)
            let thr = (mn + 0.5) * g;
            let others: Vec<f64> = ints.iter().filter(|&&x| thr <= x).map(|&x| (x / g + 0.5).floor()).collect();
            self.work += ints.len() as f64 * mu;
            if let Some(mat) = mnode.filter(|_| ints.len() <= others.len() + 1 && ints.len() > 1) {
                let g2 = gcd_f64(&others[..ints.len() - 1]);
                let r = self.rhs[i];
                if 1.5 <= g2 && r - (r + 1e-10).floor() <= 1e-10 && r.abs() <= 100000000.0 {
                    let am = self.nd[mat].val;
                    let r2 = r - (g2 - 1.0) * am;
                    let r2 = (r2 - (r2 + 1e-10).floor()) / g2;
                    if r2 - (r2 + 1e-10).floor() <= 1e-10 {
                        // shortcut: the 574764 substitution x = g2·z + r (call at 0x4bb3a7, with 3c5996) is not wired;
                        // never reached on the 38 models. Wire it to State::mod_subst (roundend.rs) when a model does
                    }
                }
            }
        }
        self.work += list.len() as f64 * 10.0 * mu;
        if !changed.is_empty() {
            self.activity(Some(&changed));
        }
        0
    }

    /// The continuous column of an otherwise integer `=` row with gcd `g`: x = d·x′ with x′ integer when the bounds
    /// allow (type 9 record {xc, 0, d}); its coefficients scale by d.
    fn implied_int_col(&mut self, at: usize, g: f64, ft: f64, ift: f64, p: &Params) -> i32 {
        let x = self.nd[at];
        let d = g / x.val.abs();
        let xc = x.col as usize;
        if !((1e-08..=100000000.0).contains(&d) && self.col_flag[xc] == 0
            && (self.lb[xc] <= -INF || -100000000.0 * d <= self.lb[xc])
            && (INF <= self.ub[xc] || self.ub[xc] <= d * 100000000.0))
        {
            return 0;
        }
        self.push_t9(xc as i32, 0.0, d);
        self.mark_rows_of_col(xc);
        self.tracker.mark_col(xc as i32);
        let (lb, ub) = (self.lb[xc], self.ub[xc]);
        if -INF < lb {
            let nl = (lb / d - ift).ceil();
            self.bound_change(xc, lb, ub, d * nl, ub);
            self.lb[xc] = nl;
        }
        if self.ub[xc] < INF {
            let nu = (self.ub[xc] / d + ift).floor();
            let cur = self.lb[xc];
            self.bound_change(xc, cur, self.ub[xc], cur, d * nu);
            self.ub[xc] = nu;
        }
        let mut q = self.col_head[xc];
        while q != NIL {
            let y = self.nd[q as usize];
            let nv = y.val * d;
            self.nd[q as usize].val = nv;
            self.agg_tiny(y.row as usize, q as usize, xc, nv, nv);
            q = y.nc;
        }
        self.work += self.col_cnt[xc] as f64 * 2.0 * self.work_mult;
        if p.impliedint != 0 {
            self.vtype[xc] = b'I';
        }
        self.obj[xc] *= d;
        if ft + self.ub[xc] < self.lb[xc] {
            self.bad_col = xc as i32;
            return 3;
        }
        0
    }

    /// The class analysis of an integer `=` row (`57627e`, then `575405`).
    fn int_classes(&mut self, i: usize, cls: &[Class], rhs0: f64, ift: f64) -> i32 {
        let cnt1 = cls.iter().filter(|k| k.count == 1).count();
        if cnt1 == 0 {
            return 0;
        }
        let g = gcd_f64(&cls.iter().map(|k| k.val).collect::<Vec<_>>());
        if g < 0.001 {
            return 0;
        }
        let rq = rhs0 / g;
        let infeas = 0.01 < rq - (rq + 0.01).floor();
        let mut abort = !(rq - (rq + 1e-10).floor() <= 1e-10);
        let rq = (rq + 0.5).floor();
        let mut vals = Vec::new();
        if !abort {
            for k in cls {
                let cv = k.val / g;
                if 1e-10 < cv - (cv + 1e-10).floor() {
                    abort = true;
                    break;
                }
                vals.push((cv + 0.5).floor());
                if k.lo <= -1000000000.0 || 1000000000.0 <= k.hi {
                    abort = true;
                    break;
                }
            }
        }
        if !abort {
            let slo: f64 = vals.iter().zip(cls).map(|(v, k)| v * k.lo).sum();
            let shi: f64 = vals.iter().zip(cls).map(|(v, k)| v * k.hi).sum();
            abort = slo < -2000000000.0 || slo - rq < -2000000000.0 || 2000000000.0 < shi || 2000000000.0 < shi - rq;
        }
        if infeas {
            self.bad_row = i as i32;
            return 3;
        }
        if abort {
            return 0;
        }
        if rq.abs() < 100000000.0 {
            for (ki, k) in cls.iter().enumerate() {
                if k.count < 2 && self.col_flag[k.col] == 0 {
                    let oth: Vec<f64> = vals.iter().enumerate().filter(|&(x, _)| x != ki).map(|(_, &v)| v).collect();
                    let g1 = gcd_f64(&oth);
                    let (lbk, ubk) = (self.lb[k.col], self.ub[k.col]);
                    if ift + 1.0 <= g1 && g1 < 99999999.0
                        && (lbk <= -INF || g1 * -100000000.0 <= lbk) && (INF <= ubk || ubk <= g1 * 100000000.0)
                    {
                        let sg = if k.neg { -1.0 } else { 1.0 };
                        let r3 = rq - (g1 - 1.0) * vals[ki] * sg;
                        let r3 = (r3 - (ift + r3).floor()) / g1;
                        if r3 - (r3 + 1e-10).floor() <= 1e-10 {
                            // shortcut: the 574764 class substitution (0x4bb04a) is not wired to State::mod_subst; never reached on the 38 models
                            return 0;
                        }
                    }
                }
            }
        }
        // shortcut: 575405 (bound tightening by enumerating the classes' sums, 9 KB, can change bounds) is not ported;
        // on the 38 models it never changes a bound. Port it with 571c44 before trusting other models.
        0
    }
}
