//! `3ff744` parallel rows / columns, MIP linked-list branch (docs/m2_red_outer_a.md §3; the LP CSR branch is
//! crates/lp/src/presolve/parallel.rs). Mode 0 rows (singleton slack columns hidden: rows 'P'/'T'), mode 1 columns,
//! mode 3 integer columns. Candidates are grouped by a pattern hash, a value hash and an exact comparison; each
//! class goes to the pair routine (`3ee2bb` rows, `3eaf1d` columns) in the order that decides the survivor.
use crate::helpers::{maxsd, minsd};
use crate::iint::gcd_f64;
use crate::post::PostRec;
use crate::state::{guarded_add, State, INF, NIL};
use crate::subst::sort_asc;
use crate::Params;

/// `ab9789`: sort `idx` by `key[idx]` (same order as `aaedd9` on the gathered keys).
fn sort_by(key: &[f64], idx: &mut [i32]) {
    let mut k: Vec<f64> = idx.iter().map(|&i| key[i as usize]).collect();
    sort_asc(&mut k, idx);
}

fn frac(v: f64) -> f64 {
    v - (v + 1e-10).floor()
}

/// Tolerances of one call.
struct Tol {
    ft: f64,
    ift: f64,
    ot: f64,
    nf: i32,
    precrush: i32,
    preparallel: i32,
    /// AggFill as 3cd406 sees it (-1 is 0)
    fill: i32,
}

impl State {
    /// `3ff744(st, env, T6, mode, flag, 0)` for modes 0, 1, 3. Returns 0, 3 or 5.
    pub fn parallel(&mut self, p: &Params, mode: i32, flag: i32) -> i32 {
        let t = Tol { ft: p.feastol, ift: self.intfeastol, ot: p.opttol, nf: p.numeric_focus, precrush: self.precrush,
                      preparallel: p.preparallel, fill: p.agg_fill.max(0) };
        let cols = mode == 1 || mode == 3;
        let saved = self.antipar;
        let (mut pwl, mut objpwl) = (0, 0);
        let gate = flag != 0 && p.precrush == 0 && p.dual_reductions != 0;
        if mode == 0 {
            self.antipar = 0;
            if gate {
                pwl = p.prepwlred;
            }
        } else if gate {
            objpwl = p.preobjpwlred;
        }
        let dim = if cols { self.n } else { self.m };
        let mut list: Vec<i32> = match mode {
            0 => {
                let l = self.adjacent_list(false);
                l.into_iter().filter(|&i| self.row_cnt[i as usize] > 1 && !self.row_slack(i as usize, t.ft)).collect()
            }
            1 => self.adjacent_list(true).into_iter().filter(|&j| self.col_cnt[j as usize] > 0).collect(),
            _ => self
                .adjacent_list(true)
                .into_iter()
                .filter(|&j| self.col_cnt[j as usize] > 0 && self.vtype[j as usize] != b'C')
                .collect(),
        };
        let mut rc = 0;
        if list.len() > 1 {
            if mode == 0 {
                list = self.hide_slacks(&list, pwl, p.dual_reductions != 0);
            }
            self.work += ((list.len() as f64 * 20.0 * self.nnz as f64) / (dim as f64 + 1.0)) * self.work_mult;
            rc = self.par_classes(mode, cols, &mut list, &t, pwl != 0, objpwl != 0);
        }
        self.antipar = if saved >= self.antipar { saved } else { self.antipar };
        if mode == 0 {
            // the hidden slack entries come back, 'T' → '=', 'P' → '<'
            for &ii in &list {
                let i = ii as usize;
                if self.sense[i] == b'P' || self.sense[i] == b'T' {
                    let mut q = self.row_head[i];
                    while q != NIL {
                        let x = &mut self.nd[q as usize];
                        if x.col < -1 {
                            x.col = -2 - x.col;
                            self.row_cnt[i] += 1;
                        }
                        q = x.nr;
                    }
                    self.sense[i] = if self.sense[i] == b'T' { b'=' } else { b'<' };
                }
            }
        }
        rc
    }

    /// `36dce1`: the row cannot be violated.
    fn row_slack(&self, i: usize, ft: f64) -> bool {
        self.inf_max[i] == 0
            && self.act_max[i] < ft
            && (self.sense[i] != b'=' || (self.inf_min[i] == 0 && -ft < self.act_min[i]))
    }

    /// Mode 0: entries of columns that live only in this row (count 1) are hidden (col = −2 − col) and the row
    /// becomes 'P' (`<`) or 'T' (`=`) when it has one such column, or several continuous ones with PREPWLRED, and
    /// (DualReductions or `=`); rows with a flagged slack, or more slacks than allowed, leave the candidates.
    fn hide_slacks(&mut self, list: &[i32], pwl: i32, dual: bool) -> Vec<i32> {
        let mut keep = Vec::new();
        for &ii in list {
            let i = ii as usize;
            let (mut k1, mut anyint, mut first, mut excl) = (0, false, NIL, false);
            let mut q = self.row_head[i];
            while q != NIL {
                let x = self.nd[q as usize];
                if x.col >= 0 && self.col_cnt[x.col as usize] == 1 {
                    let c = x.col as usize;
                    if self.col_flag[c] != 0 {
                        excl = true;
                        break;
                    }
                    anyint = anyint || self.vtype[c] != b'C';
                    k1 += 1;
                    if pwl == 0 && 1 < k1 {
                        excl = true;
                        break;
                    }
                    if first == NIL {
                        first = q;
                    }
                }
                q = x.nr;
            }
            if excl {
                continue;
            }
            if k1 == 0 {
                keep.push(ii);
                continue;
            }
            if (k1 == 1 || !anyint) && (dual || self.sense[i] == b'=') {
                let mut q = first;
                while q != NIL {
                    let x = self.nd[q as usize];
                    if x.col >= 0 && self.col_cnt[x.col as usize] == 1 {
                        self.nd[q as usize].col = -2 - x.col;
                        self.row_cnt[i] -= 1;
                        k1 -= 1;
                        if k1 == 0 {
                            break;
                        }
                    }
                    q = x.nr;
                }
                self.sense[i] = if self.sense[i] == b'=' { b'T' } else { b'P' };
                keep.push(ii);
            }
        }
        keep
    }

    /// `3e7ec1` / `3e7e33`: Σ 1/(index + 1) over the live entries (columns of a row, rows of a column).
    fn par_hash(&self, cols: bool, x: usize) -> f64 {
        let mut h = 0.0;
        let mut q = if cols { self.col_head[x] } else { self.row_head[x] };
        while q != NIL {
            let y = self.nd[q as usize];
            if y.col >= 0 {
                h += 1.0 / ((if cols { y.row } else { y.col }) as f64 + 1.0);
            }
            q = if cols { y.nc } else { y.nr };
        }
        h
    }

    /// `3e8260` (rows: min |a|) / `3e80fd` (columns: max |a|): (Σ |a|/(index+1) / scale, scale signed like the
    /// entry of smallest index).
    fn par_scale(&self, cols: bool, x: usize) -> (f64, f64) {
        let (mut acc, mut sgn) = (0.0f64, 1.0f64);
        let mut first = if cols { self.m } else { self.n } as i32 + 1;
        let mut ext = if cols { 0.0f64 } else { 1e100f64 };
        let mut q = if cols { self.col_head[x] } else { self.row_head[x] };
        while q != NIL {
            let y = self.nd[q as usize];
            q = if cols { y.nc } else { y.nr };
            if y.col < 0 {
                continue;
            }
            let k = if cols { y.row } else { y.col };
            let a = y.val;
            if k < first {
                first = k;
                sgn = if a <= 0.0 { -1.0 } else { 1.0 };
            }
            if cols {
                if !(a.abs() <= ext) {
                    ext = a.abs();
                }
            } else if !(ext <= a.abs()) {
                ext = a.abs();
            }
            acc += a.abs() / (k as f64 + 1.0);
        }
        (acc / ext, sgn * ext)
    }

    /// `3e92c4` / `3e8a37`: order two candidates by live count, then by the largest index where their scaled
    /// entries differ (rows 1e-10; columns a tolerance from the bound ranges). 0 = parallel.
    fn par_cmp(&self, cols: bool, s1: f64, s2: f64, x: usize, y: usize, tmp: &mut [f64]) -> i32 {
        let cnt = if cols { &self.col_cnt } else { &self.row_cnt };
        if cnt[y] < cnt[x] {
            return 1;
        }
        if cnt[x] < cnt[y] {
            return -1;
        }
        let (mut s1, mut s2) = (s1, s2);
        let d = if s1.abs() <= s2.abs() { s2.abs() } else { s1.abs() };
        if 1000.0 < d {
            s1 *= 1000.0 / d;
            s2 *= 1000.0 / d;
        }
        let tol = if cols {
            let (rx, ry) = (self.ub[x] - self.lb[x], self.ub[y] - self.lb[y]);
            let t = if s2.abs() < s1.abs() { rx + ry * (s2 / s1) } else { ry + rx * (s1 / s2) };
            let t = t.abs() * 1e-06;
            let t = if t <= 1e-13 { 1e-13 } else { t };
            if 1e-10 <= t { 1e-10 } else { t }
        } else {
            1e-10
        };
        let head = |k: usize| if cols { self.col_head[k] } else { self.row_head[k] };
        let next = |n: &crate::state::Node| if cols { n.nc } else { n.nr };
        let key = |n: &crate::state::Node| if cols { n.row } else { n.col } as usize;
        let (mut hi, mut lo) = (-1i32, -1i32);
        let mut q = head(x);
        while q != NIL {
            let n = self.nd[q as usize];
            if n.col >= 0 {
                tmp[key(&n)] = n.val / s1;
            }
            q = next(&n);
        }
        let mut q = head(y);
        while q != NIL {
            let n = self.nd[q as usize];
            q = next(&n);
            if n.col < 0 {
                continue;
            }
            let k = key(&n);
            let (u, v) = (tmp[k], n.val / s2);
            if u == 0.0 {
                if 0.0 < v { hi = hi.max(k as i32) } else { lo = lo.max(k as i32) }
            } else {
                tmp[k] = 0.0;
                if cols {
                    let dd = v - u;
                    if tol < dd {
                        hi = hi.max(k as i32);
                    } else if dd < -tol {
                        lo = lo.max(k as i32);
                    }
                } else if u + 1e-10 < v {
                    hi = hi.max(k as i32);
                } else if v < u - 1e-10 {
                    lo = lo.max(k as i32);
                }
            }
        }
        let mut q = head(x);
        while q != NIL {
            let n = self.nd[q as usize];
            q = next(&n);
            if n.col < 0 {
                continue;
            }
            let k = key(&n);
            let u = tmp[k];
            if u != 0.0 {
                if u <= 0.0 { hi = hi.max(k as i32) } else { lo = lo.max(k as i32) }
                tmp[k] = 0.0;
            }
        }
        if hi < lo {
            1
        } else if hi <= lo {
            0
        } else {
            -1
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn par_classes(&mut self, mode: i32, cols: bool, list: &mut [i32], t: &Tol, pwl: bool, objpwl: bool) -> i32 {
        let n = list.len();
        if n == 0 {
            return 0;
        }
        let dim = if cols { self.n } else { self.m };
        let mut key = vec![0.0f64; dim];
        let mut scale = vec![0.0f64; dim];
        let mut tmp = vec![0.0f64; if cols { self.m } else { self.n }];
        for &x in list.iter() {
            key[x as usize] = self.par_hash(cols, x as usize);
        }
        sort_by(&key, list);
        // pass 1: equal pattern hash (1e-13 from the class's first), split by the value hash (1e-10)
        let mut grp = vec![0usize; n + 1];
        grp[n] = n;
        let split = |st: &State, seg: &mut [i32], key: &mut [f64], scale: &mut [f64], grp: &mut [usize], at: usize| {
            for &x in seg.iter() {
                let (k, s) = st.par_scale(cols, x as usize);
                key[x as usize] = k;
                scale[x as usize] = s;
            }
            sort_by(key, seg);
            let (mut r, mut id) = (seg[0] as usize, at);
            for q in 1..seg.len() {
                let x = seg[q] as usize;
                if 1e-10 < key[x] - key[r] {
                    id = at + q;
                    r = x;
                }
                grp[q] = id;
            }
        };
        let (mut start, mut prev) = (0, list[0] as usize);
        for end in 1..n {
            let cur = list[end] as usize;
            if 1e-13 < key[cur] - key[prev] {
                if end - start > 1 {
                    split(self, &mut list[start..end], &mut key, &mut scale, &mut grp[start..end], start);
                }
                prev = cur;
                start = end;
            }
            grp[end] = start;
        }
        if n - start > 1 {
            split(self, &mut list[start..n], &mut key, &mut scale, &mut grp[start..n], start);
        }
        // pass 2: exact comparison
        let mut s0 = 0;
        for e in 1..=n {
            if grp[e] == grp[s0] {
                continue;
            }
            let len = e - s0;
            if len == 2 {
                let (a, b) = (list[s0] as usize, list[s0 + 1] as usize);
                if self.par_cmp(cols, scale[a], scale[b], a, b, &mut tmp) != 0 {
                    grp[s0 + 1] = s0 + 1;
                }
            } else if len > 2 {
                let seg = &mut list[s0..e];
                let mut gap = len / 2;
                while gap > 0 {
                    loop {
                        for i in gap..len {
                            let x = seg[i] as usize;
                            let mut q = i;
                            while gap <= q {
                                let y = seg[q - gap] as usize;
                                if self.par_cmp(cols, scale[x], scale[y], x, y, &mut tmp) >= 0 {
                                    break;
                                }
                                seg[q] = y as i32;
                                q -= gap;
                            }
                            seg[q] = x as i32;
                        }
                        if gap != 2 {
                            break;
                        }
                        gap = 1;
                    }
                    gap = (gap as f64 / 2.2) as usize;
                }
                let (mut r, mut id) = (list[s0] as usize, s0);
                for q in s0 + 1..e {
                    let x = list[q] as usize;
                    if self.par_cmp(cols, scale[r], scale[x], r, x, &mut tmp) != 0 {
                        id = q;
                        r = x;
                    }
                    grp[q] = id;
                }
            }
            s0 = e;
        }
        // pass 3: class bounds; classes of 3+ sorted by cost/scale (all-integer column classes: |scale|), then
        // blocks within 1e-10 by |scale| (all-integer, mode != 3: by |cost|)
        let mut bounds = vec![0usize];
        let mut s0 = 0;
        for e in 1..=n {
            if grp[e] == grp[s0] {
                continue;
            }
            bounds.push(e);
            let len = e - s0;
            if len > 2 {
                let ints = if cols { list[s0..e].iter().filter(|&&x| self.vtype[x as usize] != b'C').count() } else { 0 };
                for &x in &list[s0..e] {
                    let x = x as usize;
                    let cost = if cols { self.obj[x] } else { self.rhs[x] };
                    key[x] = if ints == len { scale[x].abs() } else { cost / scale[x] };
                }
                sort_by(&key, &mut list[s0..e]);
                if ints < len || mode != 3 {
                    let mut b = s0;
                    loop {
                        let mut q = b + 1;
                        while q < e && !(key[list[b] as usize] + 1e-10 < key[list[q] as usize]) {
                            q += 1;
                        }
                        if q - b > 2 {
                            for &x in &list[b..q] {
                                let x = x as usize;
                                let cost = if cols { self.obj[x] } else { self.rhs[x] };
                                key[x] = if ints < len { scale[x].abs() } else { cost.abs() };
                            }
                            sort_by(&key, &mut list[b..q]);
                        }
                        b = q;
                        if !(q + 2 < e) {
                            break;
                        }
                    }
                }
            }
            s0 = e;
        }
        // pairs
        let alive = |st: &State, x: i32| if cols { st.col_cnt[x as usize] > 0 } else { st.row_cnt[x as usize] > 0 };
        let rank = |c: u8| match c {
            b'T' => 1,
            b'<' => 2,
            b'=' => 3,
            _ => 0,
        };
        for g in 1..bounds.len() {
            let (b, e) = (bounds[g - 1], bounds[g]);
            let len = e - b;
            if len < 2 {
                continue;
            }
            if len == 2 {
                let rc = self.par_pair(mode, &scale, list[b], list[b + 1], t);
                if rc != 0 {
                    return rc;
                }
            } else if len < 7 {
                for i in b..e - 1 {
                    let x = list[i];
                    if !alive(self, x) {
                        continue;
                    }
                    for q in i + 1..e {
                        let y = list[q];
                        if !alive(self, y) {
                            continue;
                        }
                        let rc = self.par_pair(mode, &scale, x, y, t);
                        if rc != 0 {
                            return rc;
                        }
                        if !alive(self, x) {
                            break;
                        }
                    }
                }
            } else {
                let step = |st: &mut State, lead: i32, y: i32| -> Result<i32, i32> {
                    if !alive(st, y) {
                        return Ok(lead);
                    }
                    let rc = st.par_pair(mode, &scale, lead, y, t);
                    if rc != 0 {
                        return Err(rc);
                    }
                    if !alive(st, y) {
                        return Ok(lead);
                    }
                    if alive(st, lead) && mode == 0 {
                        let (sl, sy) = (scale[lead as usize].abs(), scale[y as usize].abs());
                        let (rl, ry) = (rank(st.sense[lead as usize]), rank(st.sense[y as usize]));
                        if sl <= 1e13 && (1e13 < sy || (ry <= rl && (ry < rl || sl < sy))) {
                            return Ok(lead);
                        }
                    }
                    Ok(y)
                };
                let mut lead = list[b];
                for q in b + 1..e {
                    match step(self, lead, list[q]) {
                        Ok(l) => lead = l,
                        Err(rc) => return rc,
                    }
                }
                let mut q = e - 1;
                while b + 2 <= q && !alive(self, list[q]) {
                    q -= 1;
                }
                if q < b + 2 {
                    continue;
                }
                let mut lead = list[q];
                for k in (b + 1..q).rev() {
                    match step(self, lead, list[k]) {
                        Ok(l) => lead = l,
                        Err(rc) => return rc,
                    }
                }
            }
        }
        if mode == 0 && pwl {
            // shortcut: 3fa2f6 (classes with 'P'/'T' rows after the pairs) is not decoded; no such class occurs on
            // the 38 models
        }
        if objpwl {
            return self.par_col_aggregation(list, &bounds, &scale, t);
        }
        0
    }

    fn par_pair(&mut self, mode: i32, scale: &[f64], x: i32, y: i32, t: &Tol) -> i32 {
        let (x, y) = (x as usize, y as usize);
        if mode == 0 { self.pair_rows(scale[x], scale[y], x, y, t) } else { self.pair_cols(scale[x], scale[y], x, y, mode, t) }
    }

    /// The single hidden slack node of a 'P'/'T' row; Err when it has more than one.
    fn hidden_slack(&self, r: usize) -> Result<Option<usize>, ()> {
        let mut h = None;
        let mut q = self.row_head[r];
        while q != NIL {
            let x = self.nd[q as usize];
            if x.col < -1 {
                if h.is_some() {
                    return Err(());
                }
                h = Some(q as usize);
            }
            q = x.nr;
        }
        Ok(h)
    }

    /// Row removal inside `3ee2bb`: live columns marked and decremented, nodes die, row −2, rows removed +1.
    fn par_drop_row(&mut self, r: usize) {
        let mut q = self.row_head[r];
        while q != NIL {
            let x = self.nd[q as usize];
            if x.col >= 0 {
                self.tracker.mark_col(x.col);
                self.col_cnt[x.col as usize] -= 1;
                self.nd[q as usize].col = -1;
            }
            q = x.nr;
        }
        self.row_cnt[r] = -2;
        self.rows_removed += 1;
    }

    /// `39df2d(v, c0, env, st, j, k)`: x_j = c0 + v·x_k (type 2, one term), columns removed +1, no growth bounds.
    fn push_subst1(&mut self, j: usize, k: usize, c0: f64, v: f64) {
        self.cols_removed += 1;
        self.post.push(PostRec::Subst { j: j as i32, c0, ind: vec![k as i32], val: vec![v] });
    }

    /// `3ee2bb(s1, s2, st, env, r1, r2)`: two parallel rows (no work). Returns 0 or 3 (`bad_row` = a).
    fn pair_rows(&mut self, p1: f64, p2: f64, r1: usize, r2: usize, t: &Tol) -> i32 {
        let (ft, ift) = (t.ft, t.ift);
        let a1 = p1.abs();
        if 1e13 < a1 || 1e13 < p2.abs() {
            return 0;
        }
        let a2 = p2.abs();
        let mut smin = if a2 <= a1 { a2 } else { a1 };
        let mut smax = if !(a1 <= a2) { a1 } else { a2 };
        let ftc = if ft <= 1e-06 { ft } else { 1e-06 };
        let tol0 = if 1e-06 <= ft { ft } else { 1e-06 };
        if t.nf < 1 {
            smax = if smax <= 1.0 { 1.0 } else { smax };
            smin = if 0.1 <= smin { 0.1 } else { smin };
        } else if t.nf == 1 {
            smax = if smax <= 10.0 { 10.0 } else { smax };
            smin = if 0.01 <= smin { 0.01 } else { smin };
        } else {
            smax *= 10.0;
            smin *= 0.1;
            smax = if smax <= 100.0 { 100.0 } else { smax };
            smin = if 0.001 <= smin { 0.001 } else { smin };
        }
        let tol2 = ftc / smax;
        let tol = tol0 / smin;
        let (mut a, mut b, mut sa, mut sb) = (r1, r2, p1, p2);
        if self.sense[b] == b'T' {
            if self.sense[a] != b'T' {
                (a, b, sa, sb) = (r2, r1, p2, p1);
            }
        } else if self.sense[b] == b'=' && self.sense[a] != b'T' && self.sense[a] != b'=' {
            (a, b, sa, sb) = (r2, r1, p2, p1);
        }
        let (ca, cb) = (self.sense[a], self.sense[b]);
        let (ra, rb) = (self.rhs[a], self.rhs[b]);
        let r = sa / sb;
        let ha = if ca == b'P' || ca == b'T' { match self.hidden_slack(a) { Ok(h) => h, Err(()) => return 0 } } else { None };
        let hb = if cb == b'P' || cb == b'T' { match self.hidden_slack(b) { Ok(h) => h, Err(()) => return 0 } } else { None };
        let infeasible = |st: &mut State| {
            st.bad_row = a as i32;
            3
        };
        if ca == b'P' {
            if cb == b'P' && (r - 1.0).abs() <= 1e-10 {
                let (ha, hb) = (ha.unwrap(), hb.unwrap());
                let (na, nb) = (self.nd[ha], self.nd[hb]);
                let (xa, xb) = ((-2 - na.col) as usize, (-2 - nb.col) as usize);
                if (na.val - nb.val).abs() <= 1e-10
                    && self.vtype[xa] == self.vtype[xb]
                    && 0.0 <= self.obj[xa] * self.obj[xb]
                    && (ra - rb).abs() <= 1e-10
                    && (self.lb[xa] - self.lb[xb]).abs() <= 1e-10
                    && (self.ub[xa] - self.ub[xb]).abs() <= 1e-10
                {
                    self.push_subst1(xb, xa, 0.0, 1.0);
                    self.obj[xa] += self.obj[xb];
                    self.obj[xb] = 0.0;
                    self.tracker.mark_col(xa as i32);
                    self.nd[hb].col = -2 - self.nd[hb].col;
                    self.par_drop_row(b);
                    self.col_cnt[xb] = -3;
                }
            }
            return 0;
        }
        if ca == b'T' {
            return self.pair_t(a, b, r, ha.unwrap(), hb, t);
        }
        if ca != b'=' {
            // a is `<`
            if cb == b'P' {
                return 0;
            }
            let qa = ra / sa;
            let code = if sa <= 0.0 {
                if sb < 0.0 {
                    if rb / sb < qa { 2 } else { 1 }
                } else {
                    let qb = rb / sb;
                    if qa <= tol + qb {
                        if ft * 10.0 + qb < qa {
                            return 0;
                        }
                        if qb - tol2 < qa { 3 } else { 0 }
                    } else {
                        -1
                    }
                }
            } else if 0.0 < sb {
                if qa < rb / sb { 2 } else { 1 }
            } else {
                let qb = rb / sb;
                if qb <= tol + qa {
                    if ft * 10.0 + qa < qb {
                        return 0;
                    }
                    if qa - tol2 < qb { 3 } else { 0 }
                } else {
                    -1
                }
            };
            match code {
                -1 => return infeasible(self),
                0 => self.antipar += 1,
                1 => self.par_drop_row(a),
                2 => self.par_drop_row(b),
                _ => {
                    // facing rows within the tolerance: a becomes `=` at the midpoint (MIP: no type 5 record)
                    let new = ((sa * rb) / sb + ra) * 0.5;
                    self.sense[a] = b'=';
                    let d = ra - new;
                    self.rhs[a] = new;
                    if d != 0.0 {
                        self.act_max[a] = guarded_add(self.act_max[a], d, true);
                        self.act_min[a] = guarded_add(self.act_min[a], ra - self.rhs[a], false);
                    }
                    self.tracker.mark_row(a as i32);
                    self.par_drop_row(b);
                }
            }
            return 0;
        }
        // a is `=`
        let ok = if cb == b'P' {
            let hb = hb.unwrap();
            let nb = self.nd[hb];
            let xb = (-2 - nb.col) as usize;
            let v = (rb * r - ra) / (r * nb.val);
            let (l, u) = (self.lb[xb], self.ub[xb]);
            let (mut nl, mut nu) = (l, u);
            if nb.val <= 0.0 {
                nl = if l <= v { v } else { l };
            } else {
                nu = if v <= u { v } else { u };
            }
            if self.vtype[xb] != b'C' {
                nl = (nl - ift).ceil();
                nu = (nu + ift).floor();
            }
            if nl <= ft + u && l - ft <= nu {
                if l < nl || nu < u {
                    self.lb[xb] = nl;
                    self.ub[xb] = nu;
                    self.tracker.mark_col(xb as i32);
                }
                if 1e-06 <= r.abs() {
                    self.nd[hb].col = -2 - self.nd[hb].col;
                    self.sense[b] = b'<';
                    self.par_drop_row(b);
                }
                return 0;
            }
            false
        } else if cb == b'=' {
            (ra / sa - rb / sb).abs() <= ft
        } else {
            (sb <= 0.0 || ra / sa - ft <= rb / sb) && (0.0 <= sb || rb / sb <= ft + ra / sa)
        };
        if ok {
            if 1e-06 <= r.abs() {
                self.par_drop_row(b);
            }
            return 0;
        }
        infeasible(self)
    }

    /// `3ee2bb` with a 'T' row `a` (hidden slack node `ha`).
    fn pair_t(&mut self, a: usize, b: usize, r: f64, ha: usize, hb: Option<usize>, t: &Tol) -> i32 {
        let (ft, ift) = (t.ft, t.ift);
        let (ra, rb) = (self.rhs[a], self.rhs[b]);
        let na = self.nd[ha];
        let xa = (-2 - na.col) as usize;
        let cb = self.sense[b];
        let infeasible = |st: &mut State| {
            st.bad_row = a as i32;
            3
        };
        if cb != b'T' {
            if cb == b'P' {
                return 0;
            }
            let v = (ra - rb * r) / na.val;
            if cb == b'=' {
                if !(self.lb[xa] - ft <= v && v <= ft + self.ub[xa]
                    && (self.vtype[xa] == b'C' || !(ift < v - (ift + v).floor())))
                {
                    return infeasible(self);
                }
                self.nd[ha].col = xa as i32;
                let rc = self.fix_col(xa, v, true);
                if rc != 0 {
                    return rc;
                }
                self.nd[ha].col = -2 - self.nd[ha].col;
                self.row_cnt[a] += 1;
                self.sense[a] = b'=';
                self.tracker.mark_row(a as i32);
                self.par_drop_row(b);
                return 0;
            }
            let d = na.val;
            let (l, u) = (self.lb[xa], self.ub[xa]);
            let (mut nl, mut nu) = (l, u);
            if !(r * d <= 0.0) {
                nl = if l > v { l } else { v };
            } else {
                nu = if u < v { u } else { v };
            }
            let (mut lo, mut hi, mut good) = (nl, nu, true);
            if self.vtype[xa] != b'C' {
                lo = (nl - ift).ceil();
                hi = (nu + ift).floor();
                if ft < (nl - lo) * d.abs() || ft < d.abs() * (hi - nu) {
                    good = false;
                }
            }
            if lo <= ft + u && l - ft <= hi {
                if l < lo || hi < u {
                    self.nd[ha].col = xa as i32;
                    self.bound_change(xa, l, u, lo, hi);
                    self.lb[xa] = lo;
                    self.ub[xa] = hi;
                    self.nd[ha].col = -2 - xa as i32;
                    self.tracker.mark_col(xa as i32);
                }
                if good {
                    self.par_drop_row(b);
                }
                return 0;
            }
            return infeasible(self);
        }
        let hb = hb.unwrap();
        let nb = self.nd[hb];
        let xb = (-2 - nb.col) as usize;
        let k = (nb.val * r) / na.val;
        let c = (ra - rb * r) / na.val;
        let mut flag = 0;
        if self.vtype[xa] != b'C' {
            if self.vtype[xb] == b'C' {
                flag = 1;
            } else if 1e-10 < frac(k) || 1e-10 < frac(c) {
                flag = if !(1e-10 < frac(1.0 / k)) && !(1e-10 < frac(c / k)) { 1 } else { 2 };
            }
        }
        if flag == 2 {
            return 0;
        }
        if flag == 1 {
            // x_b = (x_a − c)/k: x_a's bounds take x_b's, row b and x_b go
            let (l, u) = (self.lb[xa], self.ub[xa]);
            let (mut nl, mut nu) = (l, u);
            let (lbb, ubb) = (self.lb[xb], self.ub[xb]);
            let tl = |v: f64, nl: f64| if v > nl { v } else { nl };
            let tu = |v: f64, nu: f64| if v < nu { v } else { nu };
            if k > 0.0 {
                if lbb > -INF {
                    nl = tl(lbb * k + c, l);
                }
                if INF > ubb {
                    nu = tu(ubb * k + c, u);
                }
            } else {
                if INF > ubb {
                    nl = tl(ubb * k + c, l);
                }
                if lbb > -INF {
                    nu = tu(lbb * k + c, u);
                }
            }
            if self.vtype[xa] != b'C' {
                nl = (nl - ift).ceil();
                nu = (nu + ift).floor();
            }
            if nl > ft + nu {
                return infeasible(self);
            }
            if nl > l || u > nu {
                self.nd[ha].col = xa as i32;
                self.bound_change(xa, l, u, nl, nu);
                self.lb[xa] = nl;
                self.ub[xa] = nu;
                self.nd[ha].col = -2 - xa as i32;
            }
            self.push_subst1(xb, xa, -c / k, 1.0 / k);
            self.obj[xa] = self.obj[xb] / k + self.obj[xa];
            self.objcon -= (c * self.obj[xb]) / k;
            self.obj[xb] = 0.0;
            self.tracker.mark_col(xa as i32);
            self.nd[hb].col = xb as i32;
            self.par_drop_row(b);
            self.col_cnt[xb] = -3;
            return 0;
        }
        // x_a = c + k·x_b: x_b's bounds take x_a's, row a and x_a go
        let (k1, c1) = (1.0 / k, -c / k);
        let (l, u) = (self.lb[xb], self.ub[xb]);
        let (mut nl, mut nu) = (l, u);
        let (lba, uba) = (self.lb[xa], self.ub[xa]);
        let tl = |v: f64, l: f64| if v <= l { l } else { v };
        let tu = |v: f64, u: f64| if u <= v { u } else { v };
        if k1 <= 0.0 {
            if uba < INF {
                nl = tl(uba * k1 + c1, l);
            }
            if -INF < lba {
                nu = tu(lba * k1 + c1, u);
            }
        } else {
            if -INF < lba {
                nl = tl(lba * k1 + c1, l);
            }
            if uba < INF {
                nu = tu(uba * k1 + c1, u);
            }
        }
        if self.vtype[xb] != b'C' {
            nl = (nl - ift).ceil();
            nu = (nu + ift).floor();
        }
        let mut ubn = u;
        if l < nl || nu < u {
            self.nd[hb].col = xb as i32;
            self.bound_change(xb, l, u, nl, nu);
            self.lb[xb] = nl;
            self.ub[xb] = nu;
            self.nd[hb].col = -2 - xb as i32;
            ubn = nu;
        }
        if nl <= ubn + ft && self.lb[xb] - ft <= nu {
            self.push_subst1(xa, xb, -c1 / k1, 1.0 / k1);
            self.obj[xb] = self.obj[xa] / k1 + self.obj[xb];
            self.objcon -= (c1 * self.obj[xa]) / k1;
            self.obj[xa] = 0.0;
            self.tracker.mark_col(xb as i32);
            self.nd[ha].col = xa as i32;
            self.par_drop_row(a);
            self.col_cnt[xa] = -3;
            return 0;
        }
        infeasible(self)
    }

    /// `3eaf1d(s1, s2, st, env, x, y, mode)`: two parallel columns. Returns 0 or 5 (`bad_col` = x).
    #[allow(clippy::too_many_arguments)]
    fn pair_cols(&mut self, p1: f64, p2: f64, x: usize, y: usize, mode: i32, t: &Tol) -> i32 {
        if 1e13 < p1.abs() || 1e13 < p2.abs() {
            return 0;
        }
        let (sx, sy, x, y) = if p1.abs() < p2.abs() { (p2, p1, y, x) } else { (p1, p2, x, y) };
        let (lx, ly, ux, uy, cx, cy) = (self.lb[x], self.lb[y], self.ub[x], self.ub[y], self.obj[x], self.obj[y]);
        let r = sy / sx;
        let (xint, yint) = (self.vtype[x] != b'C', self.vtype[y] != b'C');
        let (mut tx, mut ty) = (1e-09, 1e-09);
        let (mut lox, mut hix, mut loy, mut hiy) = (-INF, INF, -INF, INF);
        // shortcut: state+0x100 taken as (NumericFocus > 0), see dual_bounds
        if t.nf <= 0 {
            tx = 1e-09 / if !(sx.abs() > 1.0) { 1.0 } else { sx.abs() };
            ty = 1e-09 / if !(sy.abs() > 1.0) { 1.0 } else { sy.abs() };
            if (-INF < lx || ux < INF) && tx < ux - lx {
                (lox, hix) = self.implied_col_bounds(x, 1, None);
            }
            if (-INF < ly || uy < INF) && ty < uy - ly {
                (loy, hiy) = self.implied_col_bounds(y, 1, None);
            }
        }
        let q = cy / r;
        let unit = !(1e-10 < (r.abs() - 1.0).abs());
        let pair_ok = yint && unit;
        let (mut case, mut dispatch) = (0, false);
        if (lx <= -INF || -tx <= lox - lx) && !(cx <= q - 1e-10) {
            case = 1;
            if !xint || pair_ok { dispatch = true } else { case = 0 }
        }
        if !dispatch && (INF <= ux || hix - ux <= tx) {
            if !(q + 1e-10 <= cx) {
                case = 2;
            }
            if xint && !pair_ok {
                case = 0;
            }
            dispatch = case != 0;
        }
        if !dispatch && (ly <= -INF || -ty <= loy - ly) {
            if !(cy <= cx * r - 1e-10) {
                case = 3;
            }
            if yint && (!xint || !unit) {
                case = 0;
            }
            dispatch = case != 0;
        }
        if !dispatch {
            if INF <= uy || hiy - uy <= ty {
                if cy < cx * r + 1e-10 {
                    case = 4;
                }
                if !(!yint || (xint && unit)) {
                    case = 0;
                }
            }
            if case == 0 && mode == 3 {
                return self.pair_cols5(sx, sy, x, y, t);
            }
        }
        let ot = t.ot;
        let fix = |st: &mut State, j: usize, v: f64| st.fix_col(j, v, true);
        let unbounded = |st: &mut State, s: f64| {
            if 1e-09 < s.abs() {
                st.bad_col = x as i32;
                5
            } else {
                0
            }
        };
        match case {
            4 => {
                if (r < 0.0 && INF > ux) || (r > 0.0 && lx > -INF) {
                    return fix(self, x, if r < 0.0 { ux } else { lx });
                }
                let p = r * cx;
                if cy > p - 1e-10 {
                    return self.merge_test(x, y, r, t);
                }
                if p - 100.0 * ot > cy {
                    return unbounded(self, sx);
                }
                0
            }
            3 => {
                if (r > 0.0 && INF > ux) || (r < 0.0 && lx > -INF) {
                    return fix(self, x, if 0.0 < r { ux } else { lx });
                }
                let p = r * cx;
                if 1e-10 + p > cy {
                    return self.merge_test(x, y, r, t);
                }
                if cy > 100.0 * ot + p {
                    return unbounded(self, sx);
                }
                0
            }
            2 => {
                if (r < 0.0 && INF > uy) || (r > 0.0 && ly > -INF) {
                    return fix(self, y, if r < 0.0 { uy } else { ly });
                }
                if cx > q - 1e-10 {
                    return self.merge_test(x, y, r, t);
                }
                if q - 100.0 * ot > cx {
                    return unbounded(self, sy);
                }
                0
            }
            1 => {
                if (r > 0.0 && INF > uy) || (r < 0.0 && ly > -INF) {
                    return fix(self, y, if 0.0 < r { uy } else { ly });
                }
                if 1e-10 + q > cx {
                    return self.merge_test(x, y, r, t);
                }
                if cx > 100.0 * ot + q {
                    return unbounded(self, sy);
                }
                0
            }
            _ => self.merge_test(x, y, r, t),
        }
    }

    /// The merge test of `3eaf1d` and the merge of y into x (x = x + r·y; type 4 record, flag 0).
    fn merge_test(&mut self, x: usize, y: usize, r: f64, t: &Tol) -> i32 {
        let (lx, ly, ux, uy, cx, cy) = (self.lb[x], self.lb[y], self.ub[x], self.ub[y], self.obj[x], self.obj[y]);
        let (xc, yc) = (self.vtype[x] == b'C', self.vtype[y] == b'C');
        let q = cy / r;
        let w = (((uy - ly) / r.abs()) + (ux - lx)) * (cx - q).abs();
        if w > 1e-06 || t.precrush != 0 {
            return 0;
        }
        if !(1e-10 > (cx - q).abs() && xc == yc && (xc || 1e-10 > (r.abs() - 1.0).abs())) {
            // shortcut: the PREPARALLEL = 2 merge variant is not decoded (default 1)
            let _ = t.preparallel;
            return 0;
        }
        self.cols_removed += 1;
        self.post.push(PostRec::Par { x: x as i32, y: y as i32, lx, ly, ux, uy, r, flag: 0 });
        self.tracker.mark_col(x as i32);
        // shortcut: 5b91ad (a 'B' column marked in the st+0x3f0 block) is not kept; nothing reads it here
        if r > 0.0 {
            self.lb[x] = if ly <= -INF || self.lb[x] <= -INF { -INF } else { self.lb[x] + r * ly };
            self.ub[x] = if uy >= INF || self.ub[x] >= INF { INF } else { self.ub[x] + r * uy };
        } else {
            self.lb[x] = if uy >= INF || self.lb[x] <= -INF { -INF } else { self.lb[x] + uy * r };
            self.ub[x] = if ly <= -INF || self.ub[x] >= INF { INF } else { self.ub[x] + r * ly };
        }
        if self.vtype[x] != b'C' {
            self.lb[x] = (self.lb[x] - t.ift).ceil();
            self.ub[x] = (self.ub[x] + t.ift).floor();
            self.vtype[x] = if self.lb[x] >= 0.0 && 1.0 >= self.ub[x] { b'B' } else { b'I' };
        }
        self.bound_change(y, ly, uy, 0.0, 0.0);
        let (nl, nu) = (self.lb[x], self.ub[x]);
        self.bound_change(x, lx, ux, nl, nu);
        self.obj[y] = 0.0;
        self.col_cnt[y] = -1;
        self.par_drop_col(y);
        0
    }

    /// The removed column's live nodes die; their rows are marked and lose a count.
    fn par_drop_col(&mut self, y: usize) {
        let mut q = self.col_head[y];
        while q != NIL {
            let n = self.nd[q as usize];
            if n.col >= 0 {
                self.tracker.mark_row(n.row);
                self.row_cnt[n.row as usize] -= 1;
                self.nd[q as usize].col = -1;
            }
            q = n.nc;
        }
    }

    /// `3eaf1d` case 5 (mode 3): two integer columns with a_x = k·a_y (k integral) and equal cost per unit become
    /// one integer z = y + k·x (type 4 record, flag 1); activities need no change. Else the merge test.
    fn pair_cols5(&mut self, sx: f64, sy: f64, x: usize, y: usize, t: &Tol) -> i32 {
        let (lx, ly, ux, uy, cx, cy) = (self.lb[x], self.lb[y], self.ub[x], self.ub[y], self.obj[x], self.obj[y]);
        let r = sy / sx;
        let (ax, ay) = (sx.abs(), sy.abs());
        let mut ok = t.precrush == 0 && self.vtype[x] != b'C' && self.vtype[y] != b'C';
        if ok {
            ok = if ay <= ax { frac(sx / sy) <= 1e-10 } else { frac(r) <= 1e-10 };
        }
        ok = ok && (cx / sx - cy / sy).abs() <= 1e-10 && lx + 0.5 < ux && ly + 0.5 < uy && -INF < lx && -INF < ly
            && ux < INF && uy < INF;
        if ok {
            let (mut f0, f8, lo, hi, c128, c120) = if ay < ax {
                (if 0.0 <= sy { ly } else { -uy }, if 0.0 <= sy { uy } else { -ly }, ay, ax,
                 if 0.0 <= sx { ux } else { -lx }, if 0.0 <= sx { lx } else { -ux })
            } else {
                (if 0.0 <= sx { lx } else { -ux }, if 0.0 <= sx { ux } else { -lx }, ax, ay,
                 if 0.0 <= sy { uy } else { -ly }, if 0.0 <= sy { ly } else { -uy })
            };
            let k = (hi / lo + 1e-10).floor();
            if (hi - lo).abs() <= 1e-10 || k <= (1.0 - f0) + f8 {
                f0 += c120 * k;
                let d = f8 + k * c128;
                if INF < f0 || d < INF {
                    let mut f8 = d;
                    let (keep, elim, rat, rec) = if ay < ax {
                        if sy < 0.0 {
                            (f0, f8) = (-d, -f0);
                        }
                        (y, x, sx / sy, (ly, lx, uy, ux))
                    } else {
                        if sx < 0.0 {
                            (f0, f8) = (-d, -f0);
                        }
                        (x, y, r, (lx, ly, ux, uy))
                    };
                    self.cols_removed += 1;
                    self.post.push(PostRec::Par { x: keep as i32, y: elim as i32, lx: rec.0, ly: rec.1, ux: rec.2, uy: rec.3,
                                                  r: rat, flag: 1 });
                    self.tracker.mark_col(keep as i32);
                    self.lb[keep] = f0;
                    self.ub[keep] = f8;
                    self.vtype[keep] = b'I';
                    self.obj[elim] = 0.0;
                    self.col_cnt[elim] = -1;
                    self.par_drop_col(elim);
                    return 0;
                }
            }
        }
        self.merge_test(x, y, r, t)
    }

    /// The column aggregation phase of `3ff744` (PREOBJPWLRED, docs/m2_red_outer_a.md §3.5): the live members of a
    /// class of parallel columns (all in two or more rows) become one new column z with coefficient base/s_best in
    /// best's rows, and a new row `-f·z + Σ f·(s_k/base)·x_k = 0` ties them to it. Returns a status.
    fn par_col_aggregation(&mut self, list: &[i32], bounds: &[usize], scale: &[f64], t: &Tol) -> i32 {
        let mu = self.work_mult;
        let mut sets: Vec<Vec<i32>> = Vec::new();
        for g in 1..bounds.len() {
            let (b, e) = (bounds[g - 1], bounds[g]);
            if e - b < 2 {
                continue;
            }
            let (mut set, mut tt) = (Vec::new(), b);
            while tt < e {
                let c = self.col_cnt[list[tt] as usize];
                if 0 < c {
                    if c < 2 {
                        set.clear();
                        break;
                    }
                    set.push(list[tt]);
                }
                tt += 1;
            }
            self.work += (tt - b) as f64 * (3.0 * mu);
            if set.len() > 1 {
                sets.push(set);
            }
        }
        self.work += (bounds.len() - 1) as f64 * 2.0 * mu;
        if sets.is_empty() {
            return 0;
        }
        self.work += (self.n as f64 * 0.5) * mu;
        for set in &sets {
            let l = set.len();
            let lf = l as f64;
            if l < 3 || (t.fill + 10) as f64 >= (lf - 1.0) * self.col_cnt[set[0] as usize] as f64 - 1.0 - lf {
                continue;
            }
            // best: the largest |scale| (ties within 1e-13 to a positive one); all within a factor 1e6
            let (mut smax, mut sbest, mut best, mut smin, mut allint) = (0.0f64, 0.0f64, -1i32, 1e100f64, true);
            let mut absv = vec![0.0; l];
            for (k, &c) in set.iter().enumerate() {
                let s = scale[c as usize];
                let a = s.abs();
                let d = a - smax;
                if 1e-13 < d || (-1e-13 <= d && 0.0 < s) {
                    (smax, sbest, best) = (a, s, c);
                }
                absv[k] = a;
                smin = minsd(smin, a);
                allint = allint && self.vtype[c as usize] != b'C';
            }
            self.work += (lf + lf) * mu;
            if best < 0 || 1e-10 > smax || smax > smin * 1e6 {
                continue;
            }
            let mut base = sbest;
            if allint {
                let g = gcd_f64(&absv);
                if (0.001..=1000.0).contains(&g) {
                    base = if 0.0 < sbest { g } else { -g };
                }
            }
            let z = self.add_cols(1);
            // best's nodes move to z (scaled by base/s_best); the other members' nodes die
            let (bst, ratio) = (best as usize, base / sbest);
            let (mut v, mut q) = (0, self.col_head[bst]);
            while q != NIL {
                let at = q as usize;
                let x = self.nd[at];
                if x.col >= 0 {
                    let new = ratio * x.val;
                    self.nd[at].col = z as i32;
                    self.nd[at].val = new;
                    self.agg_tiny(x.row as usize, at, bst, new, x.val);
                    self.tracker.mark_row(x.row);
                    self.row_cnt[x.row as usize] -= l as i32 - 1;
                }
                v += 1;
                q = x.nc;
            }
            self.work += (v as f64 * 3.0) * mu;
            self.col_head[z] = self.col_head[bst];
            self.col_head[bst] = NIL;
            self.col_cnt[z] += self.col_cnt[bst];
            self.col_cnt[bst] = 0;
            for &c in set {
                let (mut v, mut q) = (0, self.col_head[c as usize]);
                while q != NIL {
                    self.nd[q as usize].col = -1;
                    v += 1;
                    q = self.nd[q as usize].nc;
                }
                self.col_cnt[c as usize] = 0;
                self.work += (v as f64 + v as f64) * mu;
            }
            self.work += (lf * 3.0) * mu;
            let mut f = 1.0f64;
            for &c in set {
                f = maxsd(f, 1.0 / (scale[c as usize] / base).abs());
            }
            f = minsd(f, base.abs() * 10.0);
            let r = self.add_rows(1);
            self.sense[r] = b'=';
            self.new_node(-f, r, z);
            // z's bounds from the members' (guarded sums), integral when every t_k is
            let (mut int, mut lo, mut hi) = (true, 0.0f64, 0.0f64);
            for &c in set {
                let cu = c as usize;
                let tk = scale[cu] / base;
                let (lb, ub) = (self.lb[cu], self.ub[cu]);
                int = int && self.vtype[cu] != b'C' && tk - (tk + 1e-10).floor() <= 1e-10;
                self.new_node(f * tk, r, cu);
                let (lo_b, hi_b) = if 0.0 < tk { (lb, ub) } else { (ub, lb) };
                if -INF < lo {
                    let inf = if 0.0 < tk { -INF >= lo_b } else { lo_b >= INF };
                    lo = if inf { -INF } else { guarded_add(lo, lo_b * tk, false) };
                }
                if hi < INF {
                    let inf = if 0.0 < tk { hi_b >= INF } else { -INF >= hi_b };
                    hi = if inf { INF } else { guarded_add(hi, hi_b * tk, true) };
                }
                self.tracker.mark_col(c);
            }
            self.work += (lf * 12.0) * mu;
            if int {
                self.vtype[z] = b'I';
                lo = (lo - t.ift).ceil();
                hi = (hi + t.ift).floor();
            }
            self.lb[z] = lo;
            self.ub[z] = hi;
            self.activity(Some(&[r as i32]));
            if -INF >= lo || hi >= INF {
                let (mut v, mut q) = (0, self.col_head[z]);
                while q != NIL {
                    let x = self.nd[q as usize];
                    if x.col >= 0 {
                        self.activity(Some(&[x.row]));
                    }
                    v += 1;
                    q = x.nc;
                }
                self.work += (v as f64 + v as f64) * mu;
            }
        }
        self.work += (sets.len() as f64 + sets.len() as f64) * mu;
        0
    }
}
