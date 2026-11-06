//! `5a4d8b` / `5a5eff` (PREDERIVED): the MIP versions of the LP `5b488d` (crates/lp/src/presolve/derived.rs). A
//! column whose rows hold for every point of its neighbours' box leaves with those rows (`5a4d8b`); with binary
//! neighbours only, the bad points become no-good rows instead (`5a5eff`). docs/m2_red_roundend_b.md §7.
//! Work is added unscaled (no μ) except inside the shared `537da8`.
use crate::post::PostRec;
use crate::state::{State, NIL};
use crate::Params;

/// `5b488d`'s constants, not INFBOUND
const INF: f64 = 1e30;
const BIG: f64 = 1e100;

/// Column j's rows as `5b2ea6` writes them: x_j (sense) v + Σ c·x_p over the enumerated neighbours p, the
/// (row, c) entries grouped by neighbour (`seg` has nk + 1 starts).
struct Sys {
    v: Vec<f64>,
    sense: Vec<u8>,
    seg: Vec<usize>,
    ent_r: Vec<usize>,
    ent_c: Vec<f64>,
}

impl State {
    /// `5a4d8b(st, env)`. Returns 0 (or `3a008f`'s status).
    pub fn derived_a(&mut self, p: &Params) -> i32 {
        let n = self.n;
        let (mut slot, mut neg, mut pos) = (vec![-1i32; n], vec![0i32; n], vec![0i32; n]);
        for j in 0..n {
            let Some(piv) = self.derived_col(j) else { continue };
            let (mut list, _) = self.derived_nbrs(j, &mut slot, &mut neg, &mut pos);
            if list.len() >= 9 {
                continue;
            }
            // size of the box to enumerate (two-sign neighbours only), starting at the column count
            let mut prod = self.col_cnt[j] as f64;
            let cj = self.vtype[j] == b'C';
            for &q in &list {
                if neg[q] != 0 && pos[q] != 0 {
                    let (l, u) = (self.lb[q], self.ub[q]);
                    if (-INF < l && l < -1e6) || (u < INF && 1e6 < u) {
                        prod = BIG;
                    } else if self.vtype[q] == b'C' {
                        prod += prod;
                        if !cj {
                            prod = BIG;
                        }
                    } else if cj {
                        prod += prod;
                    } else {
                        prod *= (u + 1.0) - l;
                    }
                }
                if 1024.0 < prod {
                    break;
                }
            }
            self.work += list.len() as f64 * 5.0;
            if !(prod <= 1024.0) {
                continue;
            }
            // two-sign neighbours to the front; one-sign ones sit at their worst bound
            let mut nk = 0;
            for t in 0..list.len() {
                let q = list[t];
                if neg[q] > 0 && pos[q] > 0 {
                    list.swap(t, nk);
                    nk += 1;
                }
            }
            self.work += list.len() as f64 + list.len() as f64;
            list.truncate(nk);
            let s = self.derived_sys(j, &list, &mut slot, Some((&neg, &pos)));
            // each row alone: its bound's range over the box must meet [lb_j, ub_j] (1e-6)
            let nr = self.col_cnt[j] as usize;
            let (mut lo, mut hi) = (s.v.clone(), s.v.clone());
            for (t, &q) in list.iter().enumerate() {
                let (l, u) = (self.lb[q], self.ub[q]);
                for e in s.seg[t]..s.seg[t + 1] {
                    let (r, c) = (s.ent_r[e], s.ent_c[e]);
                    if c <= 0.0 {
                        if -INF < lo[r] {
                            lo[r] = if INF <= u { -INF } else { lo[r] + u * c };
                        }
                        if hi[r] < INF {
                            hi[r] = if l <= -INF { INF } else { hi[r] + l * c };
                        }
                    } else {
                        if -INF < lo[r] {
                            lo[r] = if l <= -INF { -INF } else { lo[r] + l * c };
                        }
                        if hi[r] < INF {
                            hi[r] = if INF <= u { INF } else { hi[r] + c * u };
                        }
                    }
                }
                self.work += (s.seg[t + 1] - s.seg[t]) as f64 * 7.0 + 2.0;
            }
            let (lj, uj) = (self.lb[j], self.ub[j]);
            let ok = (0..nr).all(|r| {
                let up_ok = !(uj + 1e-6 < hi[r]);
                match s.sense[r] {
                    b'>' => up_ok,
                    t => !(lo[r] < lj - 1e-6) && (t == b'<' || up_ok),
                }
            });
            if !ok {
                continue;
            }
            self.work += nr as f64 * 5.0;
            if self.derived_points(j, &list, &s, p.intfeastol, None) != 0 {
                continue;
            }
            self.derived_drop(j, piv);
            self.col_cnt[j] = -3;
            self.cols_removed += 1;
        }
        self.work += n as f64 * 5.0;
        0
    }

    /// `5a5eff(st, env)`: binary neighbours (at most 8) only; the bad points of their box, merged into cubes, must
    /// fit in j's rows of two or more entries, which then become one no-good row per cube. Returns 0 (or a status).
    pub fn derived_b(&mut self, p: &Params) -> i32 {
        let n = self.n;
        let (mut slot, mut neg, mut pos) = (vec![-1i32; n], vec![0i32; n], vec![0i32; n]);
        for j in 0..n {
            let Some(piv) = self.derived_col(j) else { continue };
            let (list, rows2) = self.derived_nbrs(j, &mut slot, &mut neg, &mut pos);
            if list.len() >= 9 {
                continue;
            }
            let k = list.iter().position(|&q| self.vtype[q] != b'B').unwrap_or(list.len());
            self.work += k as f64 + k as f64;
            if k < list.len() {
                continue;
            }
            let nk = list.len();
            let s = self.derived_sys(j, &list, &mut slot, None);
            let mut pt = Vec::new();
            self.derived_points(j, &list, &s, p.intfeastol, Some(&mut pt));
            let (mut care, mut cnt) = (vec![u32::MAX; pt.len()], pt.len());
            if cnt > 1 {
                // Quine-McCluskey style: equal care masks one bit apart merge; merged cubes are parked at the end
                // of the live range and compared among themselves on the next pass
                let mut i = 0;
                loop {
                    let (mut k, mut a) = (0, i);
                    while a < cnt - k {
                        let mut b = a + 1;
                        let mut merged = false;
                        while b < cnt - k {
                            let d = pt[a] ^ pt[b];
                            if care[a] == care[b] && d.count_ones() == 1 {
                                pt[a] &= !d;
                                care[a] &= !d;
                                let (c0, l7) = (cnt, cnt - k);
                                pt[b] = pt[l7 - 1];
                                care[b] = care[l7 - 1];
                                care[l7 - 1] = care[c0 - 1];
                                pt[l7 - 1] = pt[c0 - 1];
                                cnt -= 1;
                                let l26 = cnt - k;
                                k += 1;
                                pt.swap(l26 - 1, a);
                                care.swap(l26 - 1, a);
                                merged = true;
                                break;
                            }
                            b += 1;
                        }
                        self.work += (b - a) as f64 * 4.0 + nk as f64;
                        if !merged {
                            a += 1;
                            i += 1;
                        }
                    }
                    if i >= cnt {
                        break;
                    }
                }
            }
            if (rows2 as usize) < cnt {
                continue;
            }
            self.derived_drop(j, piv);
            // the first `cnt` dropped rows come back as no-good rows: Σ_{set} x − Σ_{clear} x ≤ |set| − 1
            let (mut at, mut c) = (self.col_head[j], 0);
            let mut walked = 0;
            while at != NIL {
                let r = self.nd[at as usize].row as usize;
                if self.row_cnt[r] < 0 && c < cnt {
                    self.row_cnt[r] = 0;
                    self.sense[r] = b'<';
                    self.rhs[r] = -1.0;
                    for (t, &q) in list.iter().enumerate() {
                        let bit = 1u32 << ((nk + 31 - t) & 31);
                        if care[c] & bit != 0 {
                            if pt[c] & bit == 0 {
                                self.new_node(-1.0, r, q);
                            } else {
                                self.new_node(1.0, r, q);
                                self.rhs[r] += 1.0;
                            }
                        }
                    }
                    self.work += nk as f64 + nk as f64;
                    self.rows_removed -= 1;
                    self.activity(Some(&[r as i32]));
                    self.tracker.mark_row(r as i32);
                    c += 1;
                }
                walked += 1;
                at = self.nd[at as usize].nc;
            }
            self.work += walked as f64 * 4.0;
            self.col_cnt[j] = -3;
            self.cols_removed += 1;
        }
        self.work += n as f64 * 5.0;
        0
    }

    /// Candidate test of both: 2..19 entries, no flags, every row at most 9 long, and a cost of at most 1e-10 or an
    /// `=` row (|a| > 0.001, the largest) to move it through. Returns that row (-1 none).
    fn derived_col(&mut self, j: usize) -> Option<i32> {
        let cnt = self.col_cnt[j];
        if !(2..=19).contains(&cnt) || self.col_flag[j] != 0 {
            return None;
        }
        let (mut best, mut piv, mut walked) = (0.001f64, -1i32, 0);
        let mut at = self.col_head[j];
        while at != NIL {
            let x = self.nd[at as usize];
            if x.col >= 0 {
                let r = x.row as usize;
                if self.row_cnt[r] > 9 {
                    self.work += walked as f64 + walked as f64;
                    return None;
                }
                if self.sense[r] == b'=' && !(x.val.abs() <= best) {
                    best = x.val.abs();
                    piv = x.row;
                }
            }
            walked += 1;
            at = x.nc;
        }
        self.work += walked as f64 + walked as f64;
        if 1e-10 < self.obj[j].abs() && piv == -1 {
            return None;
        }
        Some(piv)
    }

    /// Neighbours through j's rows in first-seen order, stopping at 9, with their sign counts (`=` rows count
    /// both), and the number of j's rows with two or more entries. Slots are left at -1.
    fn derived_nbrs(&mut self, j: usize, slot: &mut [i32], neg: &mut [i32], pos: &mut [i32]) -> (Vec<usize>, i32) {
        slot[j] = self.n as i32;
        neg[j] = 0;
        pos[j] = 0;
        let (mut list, mut rows2) = (Vec::new(), 0);
        let mut at = self.col_head[j];
        while at != NIL {
            let x = self.nd[at as usize];
            if x.col >= 0 {
                let r = x.row as usize;
                if self.row_cnt[r] > 1 {
                    rows2 += 1;
                }
                let eq = self.sense[r] == b'=';
                let (mut e, mut walked) = (self.row_head[r], 0);
                while e != NIL {
                    let y = self.nd[e as usize];
                    if y.col >= 0 {
                        let q = y.col as usize;
                        if slot[q] < 0 {
                            slot[q] = list.len() as i32;
                            list.push(q);
                            neg[q] = 0;
                            pos[q] = 0;
                            if list.len() > 8 {
                                break;
                            }
                        }
                        if y.val < 0.0 || eq {
                            neg[q] += 1;
                        }
                        if 0.0 < y.val || eq {
                            pos[q] += 1;
                        }
                    }
                    walked += 1;
                    e = y.nr;
                }
                self.work += walked as f64 * 4.0 + 3.0;
            }
            at = x.nc;
            if list.len() >= 9 {
                break;
            }
        }
        slot[j] = -1;
        for &q in &list {
            slot[q] = -1;
        }
        self.work += list.len() as f64 + list.len() as f64;
        (list, rows2)
    }

    /// `5b2ea6`: per live row r of j (column-list order) 1/a, the sense of x_j's bound ('<' upper, '>' lower, '='
    /// both) and v_r = rhs/a less the other neighbours at their worst bound (`sign` given; else they are ignored),
    /// then the entries of the enumerated neighbours `list`, c = -a_q/a_j.
    fn derived_sys(&mut self, j: usize, list: &[usize], slot: &mut [i32], sign: Option<(&[i32], &[i32])>) -> Sys {
        let nk = list.len();
        for (t, &q) in list.iter().enumerate() {
            slot[q] = t as i32;
        }
        self.work += nk as f64 + nk as f64;
        let mut seg = vec![0usize; nk + 1];
        let (mut inv_r, mut sense, mut v) = (Vec::new(), Vec::new(), Vec::new());
        let mut at = self.col_head[j];
        while at != NIL {
            let x = self.nd[at as usize];
            at = x.nc;
            if x.col < 0 {
                continue;
            }
            let r = x.row as usize;
            let inv = 1.0 / x.val;
            inv_r.push(inv);
            sense.push(if self.sense[r] == b'<' && !(0.0 <= inv) { b'>' } else { self.sense[r] });
            let mut vr = self.rhs[r] * inv;
            let (mut e, mut walked) = (self.row_head[r], 0);
            while e != NIL {
                let y = self.nd[e as usize];
                if y.col >= 0 && y.col as usize != j {
                    let (q, a) = (y.col as usize, y.val);
                    if slot[q] >= 0 {
                        seg[slot[q] as usize] += 1;
                    } else if let Some((neg, pos)) = sign {
                        if neg[q] != 0 {
                            if pos[q] == 0 {
                                let l = self.lb[q];
                                vr = if l <= -INF { if 0.0 < a * inv { BIG } else { -BIG } } else { vr - l * inv * a };
                            }
                        } else {
                            let u = self.ub[q];
                            vr = if INF <= u { if a * inv <= 0.0 { BIG } else { -BIG } } else { vr - u * inv * a };
                        }
                    }
                }
                walked += 1;
                e = y.nr;
            }
            v.push(vr);
            self.work += walked as f64 * 6.0 + 6.0;
        }
        let mut sum = 0;
        for c in seg.iter_mut() {
            let here = *c;
            *c = sum;
            sum += here;
        }
        self.work += nk as f64;
        let (mut ent_r, mut ent_c) = (vec![0usize; sum], vec![0f64; sum]);
        let mut fill = seg.clone();
        let (mut at, mut r) = (self.col_head[j], 0);
        while at != NIL {
            let x = self.nd[at as usize];
            at = x.nc;
            if x.col < 0 {
                continue;
            }
            let (mut e, mut walked) = (self.row_head[x.row as usize], 0);
            while e != NIL {
                let y = self.nd[e as usize];
                if y.col >= 0 && y.col as usize != j && slot[y.col as usize] >= 0 {
                    let t = slot[y.col as usize] as usize;
                    ent_r[fill[t]] = r;
                    ent_c[fill[t]] = -inv_r[r] * y.val;
                    fill[t] += 1;
                }
                walked += 1;
                e = y.nr;
            }
            r += 1;
            self.work += walked as f64 * 8.0 + 3.0;
        }
        self.work += nk as f64;
        for &q in list {
            slot[q] = -1;
        }
        self.work += nk as f64 + nk as f64;
        Sys { v, sense, seg, ent_r, ent_c }
    }

    /// `5b208f(itol, st, j, nk, list, &out, masks, ...)`: walk the points of the neighbours' box (integers step by
    /// one when j is integer too, else lower/upper bound; last neighbour fastest) and count those where no x_j in
    /// [lb_j, ub_j] satisfies all rows (rounded to integers for an integer j). Without `masks` it stops at the
    /// first; with them each bad point is kept as a bit set (neighbour t at bit nk-1-t when x_t > 0.5).
    fn derived_points(&mut self, j: usize, list: &[usize], s: &Sys, itol: f64, mut masks: Option<&mut Vec<u32>>) -> i32 {
        let nk = list.len();
        let nr = self.col_cnt[j] as usize;
        debug_assert_eq!(nr, s.v.len(), "5b208f: live rows of column {j}");
        let mut x: Vec<f64> = list.iter().map(|&q| self.lb[q]).collect();
        self.work += nk as f64 * 3.0;
        let mut val = s.v.clone();
        let (mut plus, mut minus) = (vec![0i32; nr], vec![0i32; nr]);
        self.work += nr as f64 * 3.0;
        for r in 0..nr {
            if !(s.v[r] < INF) {
                val[r] = 0.0;
                plus[r] += 1;
            } else if s.v[r] <= -INF {
                val[r] = 0.0;
                minus[r] += 1;
            }
        }
        self.work += nr as f64 * 5.0;
        for t in 0..nk {
            let xt = x[t];
            for e in s.seg[t]..s.seg[t + 1] {
                let (r, c) = (s.ent_r[e], s.ent_c[e]);
                if xt <= -INF {
                    if c * xt <= 0.0 {
                        minus[r] += 1;
                    } else {
                        plus[r] += 1;
                    }
                } else {
                    val[r] += c * xt;
                }
            }
        }
        self.work += s.seg[nk] as f64 * 5.0;
        let cj = self.vtype[j] == b'C';
        let mut bad = 0;
        'point: loop {
            let (lj, uj) = (self.lb[j], self.ub[j]);
            let (mut hi, mut lo) = (INF, -INF);
            for r in 0..nr {
                if s.sense[r] != b'>' {
                    if plus[r] == 0 && minus[r] == 0 {
                        if val[r] <= hi {
                            hi = val[r];
                        }
                    } else if minus[r] > 0 {
                        hi = -BIG;
                    }
                }
                if s.sense[r] != b'<' {
                    if plus[r] == 0 {
                        if minus[r] == 0 && lo <= val[r] {
                            lo = val[r];
                        }
                    } else if plus[r] > 0 {
                        lo = BIG;
                    }
                }
            }
            if !cj {
                if -INF < lo {
                    lo = (lo - itol).ceil();
                }
                if hi < INF {
                    hi = (hi + itol).floor();
                }
            }
            self.work += nr as f64 + nr as f64 + 5.0;
            if 1e-6 < lo - hi || hi - lj < -1e-6 || 1e-6 < lo - uj {
                bad += 1;
                let Some(m) = masks.as_deref_mut() else { return bad };
                let mut mask = 0u32;
                for (t, &xt) in x.iter().enumerate() {
                    if 0.5 < xt {
                        mask |= 1 << ((nk + 31 - t) & 31);
                    }
                }
                m.push(mask);
                self.work += nk as f64 * 1.5;
            }
            let mut t = nk;
            loop {
                if t == 0 {
                    return bad;
                }
                t -= 1;
                let q = list[t];
                let u = self.ub[q];
                let new = if u - 1e-6 <= x[t] {
                    self.lb[q]
                } else if self.vtype[q] != b'C' && !cj {
                    x[t] + 1.0
                } else {
                    u
                };
                let cnt = (s.seg[t + 1] - s.seg[t]) as f64;
                for (pass, d) in [(0, -x[t]), (1, new)] {
                    if INF <= d.abs() {
                        for e in s.seg[t]..s.seg[t + 1] {
                            let r = s.ent_r[e];
                            match (d * s.ent_c[e] <= 0.0, pass) {
                                (true, 0) => plus[r] -= 1,
                                (true, _) => minus[r] += 1,
                                (false, 0) => minus[r] -= 1,
                                (false, _) => plus[r] += 1,
                            }
                        }
                    } else {
                        for e in s.seg[t]..s.seg[t + 1] {
                            val[s.ent_r[e]] += s.ent_c[e] * d;
                        }
                    }
                    self.work += cnt * 3.0 + 6.0;
                }
                x[t] = new;
                if new != self.lb[q] {
                    continue 'point;
                }
            }
        }
    }

    /// Success tail of both: type-24 record (`3a008f`), j's cost moved onto the pivot row (x_j = (b - Σ a_k x_k) /
    /// a_j), then j's rows go (`537da8` marks, `415f4f`). The caller sets col_cnt[j] = -3.
    fn derived_drop(&mut self, j: usize, piv: i32) {
        let (mut beg, mut sense, mut rhs, mut ind, mut val) = (vec![0], Vec::new(), Vec::new(), Vec::new(), Vec::new());
        let mut at = self.col_head[j];
        while at != NIL {
            let x = self.nd[at as usize];
            at = x.nc;
            if x.col < 0 {
                continue;
            }
            let r = x.row as usize;
            let mut e = self.row_head[r];
            while e != NIL {
                let y = self.nd[e as usize];
                if y.col >= 0 {
                    ind.push(y.col);
                    val.push(y.val);
                }
                e = y.nr;
            }
            beg.push(ind.len() as i32);
            sense.push(self.sense[r]);
            rhs.push(self.rhs[r]);
        }
        let (lb, ub, vt) = (self.lb[j], self.ub[j], self.vtype[j]);
        self.post.push(PostRec::T24 { j: j as i32, lb, ub, vt, beg, sense, rhs, ind, val });
        if 1e-10 < self.obj[j].abs() {
            let pr = piv as usize;
            let (mut mult, mut walked) = (1.0, 0);
            let mut e = self.row_head[pr];
            while e != NIL {
                let y = self.nd[e as usize];
                if y.col == j as i32 {
                    mult = -self.obj[j] / y.val;
                    break;
                }
                walked += 1;
                e = y.nr;
            }
            self.work += walked as f64 + walked as f64;
            let (mut e, mut walked) = (self.row_head[pr], 0);
            while e != NIL {
                let y = self.nd[e as usize];
                if y.col >= 0 {
                    self.obj[y.col as usize] += mult * y.val;
                }
                walked += 1;
                e = y.nr;
            }
            self.obj[j] = 0.0;
            self.work += walked as f64 + walked as f64;
            self.objcon -= mult * self.rhs[pr];
        }
        let (mut at, mut walked) = (self.col_head[j], 0);
        while at != NIL {
            let x = self.nd[at as usize];
            if x.col >= 0 {
                self.mark_cols_of_row(x.row as usize);
                self.drop_row_silent(x.row as usize);
            }
            walked += 1;
            at = x.nc;
        }
        self.work += walked as f64;
    }
}
