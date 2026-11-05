//! `3ff744` on the CSR state (state+0xe0 = 1, the LP path): parallel columns (mode 1: `508461` hash, `508879`
//! scale, `508f07` compare, `509fd0` pair), parallel rows (mode 0: `50866d`, `508bcd`, `509742`, `50b4e5`) and,
//! from model generation, the split of opposite parallel rows into a range (mode 2: same row callbacks, `50c736`).
//! Candidates are sorted by a pattern hash, regrouped by a value hash, then split by an exact comparison; every
//! class is handed to the pair routine in an order that decides which member survives, so the loops, tolerances
//! and sort tie order follow the original.
use super::reduce::{ColTols, INFEASIBLE, UNBOUNDED};
use super::sort::sort_by_key;
use super::state::{PostRec, State};

/// `ab9789`: sort `idx` by `key[idx]`. Below 32 entries it Shell-sorts through the keys, otherwise it copies the
/// keys and runs `aaedd9`; both give `aaedd9`'s order for keys without NaN.
fn sort_indirect(key: &[f64], idx: &mut [i32]) {
    let mut k: Vec<f64> = idx.iter().map(|&i| key[i as usize]).collect();
    sort_by_key(&mut k, idx);
}

/// Rank of a row sense when picking which row of a large parallel class leads (`3ff744`).
fn sense_rank(c: u8) -> i32 {
    match c {
        b'T' => 1,
        b'<' => 2,
        b'=' => 3,
        _ => 0,
    }
}

impl State {
    /// `3ff744(mode)`. Mode 1 (columns, outer pass): of two columns with proportional coefficients, one is fixed,
    /// merged into the other, or proves unboundedness (`509fd0`). Mode 0 (rows, outer pass): of two rows with
    /// proportional coefficients, one is dropped, two `<` rows facing each other become one `=` row, or they prove
    /// infeasibility (`50b4e5`); facing pairs with room between them are counted in +0x42c. Mode 2 (model
    /// generation, `3b0917`): each such pair becomes one `=` row plus a range column (`50c736`, `splits`).
    /// Returns 0, 3 or 5.
    pub fn parallel(&mut self, mode: i32, t: &ColTols) -> i32 {
        let saved = if mode == 2 { 0 } else { self.antipar };
        let rc = self.parallel_classes(mode, t);
        self.antipar = saved.max(self.antipar);
        rc
    }

    fn parallel_classes(&mut self, mode: i32, t: &ColTols) -> i32 {
        let cols = mode == 1;
        let dim = if cols { self.n } else { self.m };
        // candidate list (state+0x3b8)
        let mut list: Vec<i32> = match mode {
            1 => {
                let l = self.adjacent_list(true);
                l.into_iter().filter(|&j| self.col_cnt[j as usize] > 0).collect()
            }
            0 => {
                self.antipar = 0;
                let l = self.adjacent_list(false);
                l.into_iter().filter(|&i| self.row_cnt[i as usize] > 1 && !self.row_slack(i as usize, t.feastol)).collect()
            }
            _ => {
                self.antipar = 0;
                self.splits.clear();
                (0..self.m as i32)
                    .filter(|&i| {
                        let i = i as usize;
                        self.row_cnt[i] > 1 && self.sense[i] != b'=' && !self.row_slack(i, t.feastol)
                    })
                    .collect()
            }
        };
        let n = list.len();
        if n < 2 {
            return 0;
        }
        // state+0x3a0 sort keys and +0x398 scales, by item; +0x3b0 scratch for the comparison
        let mut key = vec![0.0f64; dim];
        let mut scale = vec![0.0f64; dim];
        let mut tmp = vec![0.0f64; if cols { self.m } else { self.n }];
        self.work += ((n as f64 * 20.0 * self.nnz as f64) / (dim as f64 + 1.0)) * self.work_mult;
        for &x in &list {
            key[x as usize] = self.par_hash(cols, x as usize);
        }
        sort_indirect(&key, &mut list);

        // pass 1: classes of equal pattern hash (1e-13), each split by the value hash (1e-10); grp[p] = start of
        // p's subclass (state+0x390)
        let mut grp = vec![0usize; n + 1];
        grp[n] = n;
        let mut start = 0;
        let mut prev = list[0] as usize;
        for end in 1..n {
            let cur = list[end] as usize;
            if 1e-13 < key[cur] - key[prev] {
                if end - start > 1 {
                    self.par_split(cols, &mut list[start..end], &mut key, &mut scale, &mut grp[start..end], start);
                }
                prev = cur;
                start = end;
            }
            grp[end] = start;
        }
        if n - start > 1 {
            self.par_split(cols, &mut list[start..n], &mut key, &mut scale, &mut grp[start..n], start);
        }

        // pass 2: exact comparison within each subclass
        let mut s = 0;
        for e in 1..=n {
            if grp[e] == grp[s] {
                continue;
            }
            let len = e - s;
            if len == 2 {
                let (a, b) = (list[s] as usize, list[s + 1] as usize);
                if self.par_cmp(cols, scale[a], scale[b], a, b, &mut tmp) != 0 {
                    grp[s + 1] = s + 1;
                }
            } else if len > 2 {
                let seg = &mut list[s..e];
                let mut gap = len / 2;
                while gap > 0 {
                    loop {
                        for i in gap..len {
                            let x = seg[i] as usize;
                            let mut p = i;
                            while gap <= p {
                                let y = seg[p - gap] as usize;
                                if self.par_cmp(cols, scale[x], scale[y], x, y, &mut tmp) >= 0 {
                                    break;
                                }
                                seg[p] = y as i32;
                                p -= gap;
                            }
                            seg[p] = x as i32;
                        }
                        if gap != 2 {
                            break;
                        }
                        gap = 1;
                    }
                    gap = (gap as f64 / 2.2) as usize;
                }
                let mut r = list[s] as usize;
                let mut id = s;
                for q in s + 1..e {
                    let x = list[q] as usize;
                    if self.par_cmp(cols, scale[r], scale[x], r, x, &mut tmp) != 0 {
                        id = q;
                        r = x;
                    }
                    grp[q] = id;
                }
            }
            s = e;
        }

        // pass 3: class bounds; classes of 3 or more are ordered by cost (rhs) over scale, ties within 1e-10 of a
        // block's first by |scale|
        let cost = if cols { &self.obj } else { &self.rhs };
        let mut bounds = vec![0usize];
        let mut s = 0;
        for e in 1..=n {
            if grp[e] == grp[s] {
                continue;
            }
            bounds.push(e);
            if e - s > 2 {
                for &x in &list[s..e] {
                    key[x as usize] = cost[x as usize] / scale[x as usize];
                }
                sort_indirect(&key, &mut list[s..e]);
                let mut b = s;
                loop {
                    let mut q = b + 1;
                    while q < e && !(key[list[b] as usize] + 1e-10 < key[list[q] as usize]) {
                        q += 1;
                    }
                    if q - b > 2 {
                        for &x in &list[b..q] {
                            key[x as usize] = scale[x as usize].abs();
                        }
                        sort_indirect(&key, &mut list[b..q]);
                    }
                    b = q;
                    if q + 2 >= e {
                        break;
                    }
                }
            }
            s = e;
        }

        // pairs: classes of 2 once, up to 6 every pair, larger ones a forward and a backward sweep with a leader
        // (mode 2: classes of 2 only)
        // `3ba9de` after a class once 10000 more pairs were tried since the last check
        let alive = |st: &State, x: usize| st.par_alive(cols, x);
        let (mut pairs, mut checked) = (0i64, 0i64);
        for g in 1..bounds.len() {
            if 9999 < pairs - checked {
                checked = pairs;
                self.budget_check();
            }
            let (b, e) = (bounds[g - 1], bounds[g]);
            let len = e - b;
            if len < 2 {
                continue;
            }
            if len == 2 {
                let (x, y) = (list[b] as usize, list[b + 1] as usize);
                pairs += 1;
                let rc = self.par_pair(mode, scale[x], scale[y], x, y, t);
                if rc != 0 {
                    return rc;
                }
            } else if mode == 2 {
                continue;
            } else if len < 7 {
                for i in b..e - 1 {
                    let x = list[i] as usize;
                    if !alive(self, x) {
                        continue;
                    }
                    for &y in &list[i + 1..e] {
                        let y = y as usize;
                        if !alive(self, y) {
                            continue;
                        }
                        pairs += 1;
                        let rc = self.par_pair(mode, scale[x], scale[y], x, y, t);
                        if rc != 0 {
                            return rc;
                        }
                        if !alive(self, x) {
                            break;
                        }
                    }
                }
            } else {
                let mut lead = list[b] as usize;
                for &y in &list[b + 1..e] {
                    pairs += alive(self, y as usize) as i64;
                    match self.par_lead_step(mode, lead, y as usize, &scale, t) {
                        Ok(next) => lead = next,
                        Err(rc) => return rc,
                    }
                }
                let mut q = e - 1;
                while b + 2 <= q && !alive(self, list[q] as usize) {
                    q -= 1;
                }
                if q < b + 2 {
                    continue;
                }
                let mut lead = list[q] as usize;
                for &y in list[b + 1..q].iter().rev() {
                    pairs += alive(self, y as usize) as i64;
                    match self.par_lead_step(mode, lead, y as usize, &scale, t) {
                        Ok(next) => lead = next,
                        Err(rc) => return rc,
                    }
                }
            }
        }
        if 9999 < pairs - checked {
            self.budget_check();
        }
        0
    }

    fn par_alive(&self, cols: bool, x: usize) -> bool {
        if cols { self.col_cnt[x] > 0 } else { self.row_cnt[x] > 0 }
    }

    /// One step of a sweep over a class of 7 or more: pair the leader with `y` if `y` is live; `y` leads afterwards
    /// unless it is gone or, for rows, the leader is preferred (higher sense rank, else smaller |scale|, and
    /// |scale| <= 1e13). Returns the next leader or a status.
    fn par_lead_step(&mut self, mode: i32, lead: usize, y: usize, scale: &[f64], t: &ColTols) -> Result<usize, i32> {
        let cols = mode == 1;
        if !self.par_alive(cols, y) {
            return Ok(lead);
        }
        let rc = self.par_pair(mode, scale[lead], scale[y], lead, y, t);
        if rc != 0 {
            return Err(rc);
        }
        if !self.par_alive(cols, y) {
            return Ok(lead);
        }
        if mode == 0 && self.par_alive(cols, lead) {
            let (sl, sy) = (scale[lead].abs(), scale[y].abs());
            let (rl, ry) = (sense_rank(self.sense[lead]), sense_rank(self.sense[y]));
            if sl <= 1e13 && (1e13 < sy || (ry <= rl && (ry < rl || sl < sy))) {
                return Ok(lead);
            }
        }
        Ok(y)
    }

    fn par_pair(&mut self, mode: i32, s1: f64, s2: f64, x: usize, y: usize, t: &ColTols) -> i32 {
        match mode {
            1 => self.parallel_cols_pair(s1, s2, x, y, t),
            0 => self.parallel_rows_pair(s1, s2, x, y, t),
            _ => self.split_rows_pair(s1, s2, x, y, t.feastol),
        }
    }

    /// Pass-1 class `seg` (positions `at..`) with equal pattern hash: value hash (`508879`/`508bcd`) into `key`,
    /// sort, and chain into subclasses whose keys stay within 1e-10 of the subclass's first.
    fn par_split(&mut self, cols: bool, seg: &mut [i32], key: &mut [f64], scale: &mut [f64], grp: &mut [usize], at: usize) {
        for &x in seg.iter() {
            let x = x as usize;
            key[x] = self.par_scale(cols, x, &mut scale[x]);
        }
        sort_indirect(key, seg);
        let mut r = seg[0] as usize;
        let mut id = at;
        for p in 1..seg.len() {
            let x = seg[p] as usize;
            if 1e-10 < key[x] - key[r] {
                id = at + p;
                r = x;
            }
            grp[p] = id;
        }
    }

    /// The CSR of the kind being compared: (start, slots, index, value, own live counts, other kind's live counts).
    #[allow(clippy::type_complexity)]
    fn par_csr(&self, cols: bool) -> (&[i64], &[i32], &[i32], &[f64], &[i32], &[i32]) {
        if cols {
            (&self.col_beg, &self.col_len, &self.col_ind, &self.col_val, &self.col_cnt, &self.row_cnt)
        } else {
            (&self.row_beg, &self.row_len, &self.row_ind, &self.row_val, &self.row_cnt, &self.col_cnt)
        }
    }

    /// `508461` (columns) / `50866d` (rows): sum of 1/(i+1) over the live entries' indices, in two lanes (even and
    /// odd slots) added at the end, then the odd last slot.
    fn par_hash(&mut self, cols: bool, x: usize) -> f64 {
        let (beg, len, ind, _, cnt, other) = self.par_csr(cols);
        if cnt[x] < 0 {
            return 0.0;
        }
        let (b, l) = (beg[x] as usize, len[x]);
        let term = |k: usize| {
            let i = ind[k];
            if i >= 0 && other[i as usize] >= 0 { 1.0 / (i as f64 + 1.0) } else { 0.0 }
        };
        let mut s = 0.0f64;
        let pairs = if l >= 2 { l as usize / 2 } else { 0 };
        if pairs > 0 {
            let (mut s0, mut s1) = (0.0f64, 0.0f64);
            for p in 0..pairs {
                s0 += term(b + 2 * p);
                s1 += term(b + 2 * p + 1);
            }
            s = s0 + s1;
        }
        if l > 0 && (l as usize) > 2 * pairs {
            let k = b + l as usize - 1;
            let i = ind[k];
            if i >= 0 && other[i as usize] >= 0 {
                s += 1.0 / (i as f64 + 1.0);
            }
        }
        self.work += l as f64 * 2.0;
        s
    }

    /// `508879` (columns) / `508bcd` (rows): sum of |a|/(i+1) over the live entries divided by max |a|; `scale`
    /// gets max |a| with the sign of the entry of smallest index.
    fn par_scale(&mut self, cols: bool, x: usize, scale: &mut f64) -> f64 {
        let (beg, len, ind, val, cnt, other) = self.par_csr(cols);
        if cnt[x] < 0 || (cols && len[x] < 0) {
            return 0.0;
        }
        let mut first = (if cols { self.m } else { self.n }) as i32 + 1;
        let (mut sign, mut amax, mut s) = (1.0f64, 0.0f64, 0.0f64);
        let b = beg[x] as usize;
        for k in b..b + len[x].max(0) as usize {
            let i = ind[k];
            if i < 0 || other[i as usize] < 0 {
                continue;
            }
            let a = val[k];
            if i < first {
                first = i;
                sign = if a <= 0.0 { -1.0 } else { 1.0 };
            }
            if !(a.abs() <= amax) {
                amax = a.abs();
            }
            s += a.abs() / (i as f64 + 1.0);
        }
        self.work += len[x] as f64 * if cols { 3.0 } else { 4.0 };
        *scale = sign * amax;
        s / amax
    }

    /// `508f07` (columns) / `509742` (rows): order two candidates by live count, then by the largest index where
    /// their scaled entries differ by more than 1e-10 (`tmp`, all zero on entry and exit). 0 = parallel.
    fn par_cmp(&mut self, cols: bool, s1: f64, s2: f64, x: usize, y: usize, tmp: &mut [f64]) -> i32 {
        let (beg, len, ind, val, cnt, other) = self.par_csr(cols);
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
        let (mut hi, mut lo) = (-1i32, -1i32);
        let live = |k: usize| {
            let i = ind[k];
            if i >= 0 && other[i as usize] >= 0 { Some(i) } else { None }
        };
        let (bx, lx) = (beg[x] as usize, len[x]);
        let (by, ly) = (beg[y] as usize, len[y]);
        for k in bx..bx + lx.max(0) as usize {
            if let Some(i) = live(k) {
                tmp[i as usize] = val[k] / s1;
            }
        }
        for k in by..by + ly.max(0) as usize {
            if let Some(i) = live(k) {
                let (u, v) = (tmp[i as usize], val[k] / s2);
                if u == 0.0 {
                    if 0.0 < v { hi = hi.max(i) } else { lo = lo.max(i) }
                } else {
                    tmp[i as usize] = 0.0;
                    if u + 1e-10 < v {
                        hi = hi.max(i);
                    } else if v < u - 1e-10 {
                        lo = lo.max(i);
                    }
                }
            }
        }
        for k in bx..bx + lx.max(0) as usize {
            if let Some(i) = live(k) {
                let u = tmp[i as usize];
                if u != 0.0 {
                    if u <= 0.0 { hi = hi.max(i) } else { lo = lo.max(i) }
                    tmp[i as usize] = 0.0;
                }
            }
        }
        let (w1, w2, w3) = if cols { (3.0, 6.0, 5.0) } else { (4.0, 4.0, 4.0) };
        self.work += lx as f64 * w1;
        self.work += ly as f64 * w2;
        self.work += lx as f64 * w3;
        if hi < lo {
            1
        } else if hi == lo {
            0
        } else {
            -1
        }
    }

    /// `36dce1`: row `i` cannot be violated (max activity below rhs, and for `=` rows min activity above).
    fn row_slack(&self, i: usize, feastol: f64) -> bool {
        self.inf_max[i] == 0
            && self.act_max[i] < feastol
            && (self.sense[i] != b'=' || (self.inf_min[i] == 0 && -feastol < self.act_min[i]))
    }

    /// `509fd0`: columns `x` and `y` with a_y = r a_x. With `x` the one of larger |scale| and r = s_y / s_x, the
    /// first case that holds decides: (1) x's lower bound is implied (or -inf) and c_x >= c_y/r, (2) x's upper
    /// bound implied and c_x <= c_y/r, (3) y's lower bound implied and c_y >= r c_x, (4) y's upper bound implied and
    /// c_y <= r c_x, (0) otherwise. Each case fixes one column at a bound, merges y into x when the costs agree
    /// within 1e-10 (type-4 record), proves unboundedness, or does nothing. Returns 0 or 5.
    fn parallel_cols_pair(&mut self, s1: f64, s2: f64, x: usize, y: usize, t: &ColTols) -> i32 {
        let (inf, opttol) = (t.infbound, t.opttol);
        if 1e13 < s1.abs() || 1e13 < s2.abs() {
            return 0;
        }
        let (sx, sy, x, y) = if s1.abs() < s2.abs() { (s2, s1, y, x) } else { (s1, s2, x, y) };
        let (lx, ly, ux, uy, cx, cy) = (self.lb[x], self.lb[y], self.ub[x], self.ub[y], self.obj[x], self.obj[y]);
        let r = sy / sx;
        let (mut tx, mut ty) = (1e-9, 1e-9);
        let (mut lox, mut hix, mut loy, mut hiy) = (-inf, inf, -inf, inf);
        // state+0x100 == 0 (see dual_bound_rows)
        if t.numeric_focus <= 0 {
            tx = 1e-9 / if 1.0 < sx.abs() { sx.abs() } else { 1.0 };
            ty = 1e-9 / if 1.0 < sy.abs() { sy.abs() } else { 1.0 };
            if -inf < lx || ux < inf {
                (lox, hix) = self.implied_bounds(x, inf);
            }
            if -inf < ly || uy < inf {
                (loy, hiy) = self.implied_bounds(y, inf);
            }
        }
        let q = cy / r;
        let p = cx * r;
        let case = if (-inf >= lx || lox - lx >= -tx) && cx > q - 1e-10 {
            1
        } else if (ux >= inf || tx >= hix - ux) && q + 1e-10 > cx {
            2
        } else if (-inf >= ly || loy - ly >= -ty) && cy > p - 1e-10 {
            3
        } else if (uy >= inf || ty >= hiy - uy) && cy < p + 1e-10 {
            4
        } else {
            0
        };
        // fix: (column, value); merge: costs agree; unbounded: the scale that must be nonzero
        enum Act {
            Fix(usize, f64),
            Merge,
            Unbounded(f64),
            None,
        }
        let act = match case {
            4 if (0.0 > r && inf > ux) || (r > 0.0 && lx > -inf) => Act::Fix(x, if r < 0.0 { ux } else { lx }),
            3 if (r > 0.0 && inf > ux) || (0.0 > r && lx > -inf) => Act::Fix(x, if 0.0 < r { ux } else { lx }),
            2 if (0.0 > r && inf > uy) || (r > 0.0 && ly > -inf) => Act::Fix(y, if r < 0.0 { uy } else { ly }),
            1 if (r > 0.0 && inf > uy) || (0.0 > r && ly > -inf) => Act::Fix(y, if 0.0 < r { uy } else { ly }),
            4 if cy > p - 1e-10 => Act::Merge,
            4 if p - 100.0 * opttol > cy => Act::Unbounded(sx),
            3 if 1e-10 + p > cy => Act::Merge,
            3 if cy > 100.0 * opttol + p => Act::Unbounded(sx),
            2 if cx > q - 1e-10 => Act::Merge,
            2 if q - 100.0 * opttol > cx => Act::Unbounded(sy),
            1 if 1e-10 + q > cx => Act::Merge,
            1 if cx > 100.0 * opttol + q => Act::Unbounded(sy),
            0 if 1e-10 > (cx - q).abs() => Act::Merge,
            _ => Act::None,
        };
        match act {
            Act::Fix(j, v) => {
                self.fix_at(j, v, inf, true);
                0
            }
            Act::Unbounded(s) if 1e-9 < s.abs() => {
                self.bad_col = x as i32;
                UNBOUNDED
            }
            Act::Merge if t.precrush == 0 => {
                self.cols_removed += 1;
                self.post.push(PostRec::ParallelCol { x: x as i32, y: y as i32, lx, ly, ux, uy, r });
                self.tracker.mark_col(x as i32);
                if r > 0.0 {
                    self.lb[x] = if -inf >= ly || -inf >= self.lb[x] { -inf } else { self.lb[x] + r * ly };
                    self.ub[x] = if uy >= inf || self.ub[x] >= inf { inf } else { self.ub[x] + r * uy };
                } else {
                    self.lb[x] = if uy >= inf || -inf >= self.lb[x] { -inf } else { self.lb[x] + r * uy };
                    self.ub[x] = if -inf >= ly || self.ub[x] >= inf { inf } else { self.ub[x] + r * ly };
                }
                self.bound_change(y, ly, uy, 0.0, 0.0, inf);
                let (nl, nu) = (self.lb[x], self.ub[x]);
                self.bound_change(x, lx, ux, nl, nu, inf);
                self.obj[y] = 0.0;
                self.col_cnt[y] = -1;
                let b = self.col_beg[y] as usize;
                let mut k = b;
                while k < b + self.col_len[y].max(0) as usize {
                    let i = self.col_ind[k];
                    if i >= 0 && self.row_cnt[i as usize] >= 0 {
                        self.tracker.mark_row(i);
                        self.row_cnt[i as usize] -= 1;
                        self.col_ind[k] = -1;
                    }
                    k += 1;
                }
                self.work += self.col_len[y] as f64 * 5.0;
                self.col_len[y] = 0;
                0
            }
            _ => 0,
        }
    }

    /// `50b4e5`: rows `r1` and `r2` with row r1 = (s1/s2) row r2. An `=` row (taken as `a`) makes the other row
    /// redundant or the LP infeasible. Two `<` rows facing the same way keep the tighter one; facing each other
    /// they prove infeasibility (gap below -tol), become one `=` row (type-5 record) when the gap is within the
    /// NumericFocus tolerance, or are left alone and counted in +0x42c. Returns 0 or 3.
    fn parallel_rows_pair(&mut self, s1: f64, s2: f64, r1: usize, r2: usize, t: &ColTols) -> i32 {
        let (a1, a2) = (s1.abs(), s2.abs());
        if 1e13 < a1 || 1e13 < a2 {
            return 0;
        }
        let smax = if a1 > a2 { a1 } else { a2 };
        let tol2 = match t.numeric_focus {
            nf if nf <= 0 => 1e-10 / if smax > 1.0 { smax } else { 1.0 },
            1 => 1e-10 / if smax > 10.0 { smax } else { 10.0 },
            _ => 1e-10 / if smax * 10.0 > 100.0 { smax * 10.0 } else { 100.0 },
        };
        let smin = if a1 < a2 { a1 } else { a2 };
        let smin = if smin < 1.0 { smin } else { 1.0 };
        let tol = (if 1e-6 > t.feastol { 1e-6 } else { t.feastol }) / smin;
        let (a, b, sa, sb) = if self.sense[r2] == b'=' && self.sense[r1] == b'<' { (r2, r1, s2, s1) } else { (r1, r2, s1, s2) };
        let (ra, rb) = (self.rhs[a], self.rhs[b]);
        let infeasible = |st: &mut State| {
            st.bad_row = a as i32;
            INFEASIBLE
        };
        if self.sense[a] == b'=' {
            if self.sense[b] == b'=' {
                if tol < (ra / sa - rb / sb).abs() {
                    return infeasible(self);
                }
            } else if (sb > 0.0 && ra / sa - tol > rb / sb) || (0.0 > sb && rb / sb > tol + ra / sa) {
                return infeasible(self);
            }
            self.drop_parallel_row(b);
            return 0;
        }
        let (qa, qb) = (ra / sa, rb / sb);
        let act = if sa > 0.0 {
            if sb > 0.0 {
                if qa < qb { 2 } else { 1 }
            } else {
                let d = qb - qa;
                if d > tol {
                    -1
                } else if d >= -tol2 {
                    3
                } else {
                    0
                }
            }
        } else if 0.0 > sb {
            if qb < qa { 2 } else { 1 }
        } else {
            let d = -(qb - qa);
            if d > tol {
                -1
            } else if d >= -tol2 {
                3
            } else {
                0
            }
        };
        match act {
            -1 => return infeasible(self),
            0 => self.antipar += 1,
            1 => self.drop_parallel_row(a),
            2 => self.drop_parallel_row(b),
            _ => {
                self.post.push(PostRec::ParallelRow { a: a as i32, b: b as i32, ratio: sa / sb });
                self.sense[a] = b'=';
                self.tracker.mark_row(a as i32);
                self.drop_parallel_row(b);
            }
        }
        0
    }

    /// `50c736`: opposite parallel `<` rows with lo <= hi - feastol between them (row a has the positive scale,
    /// u <= hi = rhs_a/s_a; row b gives u >= lo = rhs_b/s_b): row a becomes `=` and gets a range column added at
    /// model generation (`splits`), row b goes. Type-14 record.
    fn split_rows_pair(&mut self, s1: f64, s2: f64, r1: usize, r2: usize, feastol: f64) -> i32 {
        if s1 * s2 > 0.0 {
            return 0;
        }
        let (a, b, sa, sb) = if 0.0 > s1 { (r2, r1, s2, s1) } else { (r1, r2, s1, s2) };
        let (ra, rb) = (self.rhs[a], self.rhs[b]);
        if rb / sb > ra / sa - feastol {
            return 0;
        }
        let rb_ = self.row_beg[a] as usize;
        let mut entries = Vec::new();
        for k in rb_..rb_ + self.row_len[a].max(0) as usize {
            let j = self.row_ind[k];
            if j >= 0 && self.col_cnt[j as usize] >= 0 {
                entries.push((j, self.row_val[k]));
            }
        }
        self.work += self.row_len[a] as f64 * 5.0;
        let lo = (sa * rb) / sb;
        self.post.push(PostRec::SplitRow { a: a as i32, b: b as i32, k: self.antipar, rhs: ra, lo, ratio: sa / sb, entries });
        self.splits.push((a as i32, lo));
        self.antipar += 1;
        self.tracker.mark_row(a as i32);
        self.sense[a] = b'=';
        // the original leaves row b's slot count alone; nothing reads it once the row is gone
        self.drop_parallel_row(b);
        0
    }

    /// Row removal inlined in `50b4e5`: unlink the row from its live columns (marking them), row count -2.
    fn drop_parallel_row(&mut self, i: usize) {
        let b = self.row_beg[i] as usize;
        let mut k = b;
        while k < b + self.row_len[i].max(0) as usize {
            let j = self.row_ind[k];
            if j >= 0 && self.col_cnt[j as usize] >= 0 {
                self.tracker.mark_col(j);
                self.col_cnt[j as usize] -= 1;
                self.row_ind[k] = -1;
            }
            k += 1;
        }
        self.work += self.row_len[i] as f64 * 4.0;
        self.row_cnt[i] = -2;
        self.row_len[i] = 0;
        self.rows_removed += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::super::reduce::ColTols;
    use super::super::state::State;
    use rsolver_model::Model;

    /// x + 2y <= 4 and -x - 2y <= -1 face each other: mode 0 leaves them (+0x42c), mode 2 makes row 0 `=` with
    /// a range column in [0, 3] and drops row 1.
    #[test]
    fn facing_rows_become_a_range() {
        let md = Model {
            sense: 1,
            obj: vec![1.0, 1.0],
            lb: vec![0.0; 2],
            ub: vec![10.0; 2],
            vtype: vec![b'C'; 2],
            col_beg: vec![0, 2],
            col_len: vec![2, 2],
            col_ind: vec![0, 1, 0, 1],
            col_val: vec![1.0, -1.0, 2.0, -2.0],
            row_sense: vec![b'<'; 2],
            rhs: vec![4.0, -1.0],
            var_names: vec!["x".into(), "y".into()],
            constr_names: vec!["r0".into(), "r1".into()],
            ..Model::default()
        };
        let t = ColTols { infbound: 1e30, feastol: 1e-6, opttol: 1e-6, numeric_focus: 0, agg_mult_limit: 1e14,
                          dual_reductions: true, precrush: 0 };
        let mut st = State::new(&md);
        st.activity(None, 1e30);
        assert_eq!(st.parallel(0, &t), 0);
        assert_eq!((st.antipar, st.rows_removed), (1, 0));
        assert_eq!(st.parallel(2, &t), 0);
        let q = st.to_model(&md);
        assert_eq!((q.row_sense.clone(), q.rhs.clone()), (vec![b'='], vec![4.0]));
        assert_eq!((q.var_names[2].as_str(), q.col_ind[2], q.lb[2], q.ub[2]), ("C2", 0, 0.0, 3.0));
    }
}
