//! `5b488d` (GURO_PAR_PREDERIVED): a column whose rows some value of it always satisfies, whatever its
//! neighbours do within their bounds, leaves together with those rows (docs/m1/m1_presolve_derived.md).
use super::reduce::ColTols;
use super::state::{ColRow, PostRec, State};

/// `5b488d` uses these constants, not INFBOUND.
const INF: f64 = 1e30;
const BIG: f64 = 1e100;

impl State {
    /// `5b488d`: one pass over the columns. A candidate has 1..9 live entries, no row longer than 21, and a cost
    /// of at most 1e-10 or an `=` row (|a| > 0.001) to move the cost through. Its rows' other columns (at most
    /// 20) that appear with one sign only sit at their worst bound; the rest (at most 8) are enumerated over
    /// the corners of their box. Returns 0 (the original's allocation failure code 10001 cannot happen here).
    pub fn derived_bounds(&mut self, _t: &ColTols) -> i32 {
        let n = self.n;
        // slot: position in the neighbour list (-1 none); neg/pos: entries with a < 0 / a > 0 (`=` rows: both)
        let mut slot = vec![-1i32; n];
        let mut neg = vec![0i32; n];
        let mut pos = vec![0i32; n];
        for j in 0..n {
            let cnt = self.col_cnt[j];
            if !(0 < cnt && cnt < 10 && self.col_flag[j] == 0) {
                continue;
            }
            let (b, len) = (self.col_beg[j] as usize, self.col_len[j] as usize);
            let (mut best, mut piv, mut k) = (0.001f64, -1i32, 0);
            while k < len {
                let i = self.col_ind[b + k];
                if i >= 0 && self.row_cnt[i as usize] >= 0 {
                    if self.row_cnt[i as usize] > 21 {
                        break;
                    }
                    let a = self.col_val[b + k].abs();
                    if self.sense[i as usize] == b'=' && !(a <= best) {
                        best = a;
                        piv = i;
                    }
                }
                k += 1;
            }
            if k < len || !(self.obj[j].abs() <= 1e-10 || piv != -1) {
                continue;
            }
            // neighbours through j's rows, with their sign counts
            slot[j] = 1;
            pos[j] = 0;
            neg[j] = 0;
            let mut list: Vec<usize> = Vec::new();
            for k in 0..len {
                let i = self.col_ind[b + k];
                if i < 0 || self.row_cnt[i as usize] < 0 {
                    continue;
                }
                let i = i as usize;
                let eq = self.sense[i] == b'=';
                let rb = self.row_beg[i] as usize;
                let mut t = 0;
                while t < self.row_len[i] as usize {
                    let (q, a) = (self.row_ind[rb + t], self.row_val[rb + t]);
                    if q >= 0 && self.col_cnt[q as usize] >= 0 {
                        let q = q as usize;
                        if slot[q] < 0 {
                            list.push(q);
                            slot[q] = 1;
                            pos[q] = 0;
                            neg[q] = 0;
                            if list.len() > 20 {
                                break;
                            }
                        }
                        if a < 0.0 || eq {
                            neg[q] += 1;
                        }
                        if 0.0 < a || eq {
                            pos[q] += 1;
                        }
                    }
                    t += 1;
                }
                self.work += t as f64 * 6.0 + 3.0;
                if list.len() > 20 {
                    break;
                }
            }
            slot[j] = -1;
            for &q in &list {
                slot[q] = -1;
            }
            self.work += list.len() as f64 + list.len() as f64;
            if list.len() > 20 {
                continue;
            }
            // two-sign neighbours to the front (swap partition, as the original)
            let mut nk = 0;
            for t in 0..list.len() {
                let q = list[t];
                if neg[q] > 0 && pos[q] > 0 {
                    list.swap(t, nk);
                    nk += 1;
                }
            }
            self.work += list.len() as f64 * 4.0;
            if nk >= 9 {
                continue;
            }
            list.truncate(nk);
            for (t, &q) in list.iter().enumerate() {
                slot[q] = t as i32;
            }
            self.work += nk as f64 * 3.0;
            // per live row r of j: x_j (sense_r) v_r + sum c x_p over the kept neighbours p; sense_r '<' upper,
            // '>' lower (a < 0 flips a `<` row), '=' both
            let mut seg = vec![0usize; nk + 1];
            let (mut inv_r, mut sense_r, mut v_r) = (Vec::new(), Vec::new(), Vec::new());
            for k in 0..len {
                let i = self.col_ind[b + k];
                if i < 0 || self.row_cnt[i as usize] < 0 {
                    continue;
                }
                let i = i as usize;
                let inv = 1.0 / self.col_val[b + k];
                let s = self.sense[i];
                inv_r.push(inv);
                sense_r.push(if s == b'<' && !(0.0 <= inv) { b'>' } else { s });
                let mut v = self.rhs[i] * inv;
                let (rb, rl) = (self.row_beg[i] as usize, self.row_len[i] as usize);
                for t in rb..rb + rl {
                    let q = self.row_ind[t];
                    if q < 0 || self.col_cnt[q as usize] < 0 || q as usize == j {
                        continue;
                    }
                    let (q, a) = (q as usize, self.row_val[t]);
                    if slot[q] >= 0 {
                        seg[slot[q] as usize] += 1;
                    } else if neg[q] != 0 {
                        if pos[q] == 0 {
                            let l = self.lb[q];
                            v = if l <= -INF { if 0.0 < a * inv { BIG } else { -BIG } } else { v - l * inv * a };
                        }
                    } else {
                        let u = self.ub[q];
                        v = if INF <= u { if a * inv <= 0.0 { BIG } else { -BIG } } else { v - u * inv * a };
                    }
                }
                v_r.push(v);
                self.work += rl as f64 * 9.0 + 6.0;
            }
            let mut s = 0;
            for c in seg.iter_mut() {
                let here = *c;
                *c = s;
                s += here;
            }
            self.work += nk as f64;
            // (row, coefficient) of each kept neighbour, grouped by neighbour
            let (mut ent_r, mut ent_c) = (vec![0usize; s], vec![0f64; s]);
            let mut fill = seg.clone();
            let mut r = 0;
            for k in 0..len {
                let i = self.col_ind[b + k];
                if i < 0 || self.row_cnt[i as usize] < 0 {
                    continue;
                }
                let i = i as usize;
                let (rb, rl) = (self.row_beg[i] as usize, self.row_len[i] as usize);
                for t in rb..rb + rl {
                    let q = self.row_ind[t];
                    if q < 0 || self.col_cnt[q as usize] < 0 || q as usize == j || slot[q as usize] < 0 {
                        continue;
                    }
                    let p = slot[q as usize] as usize;
                    ent_r[fill[p]] = r;
                    ent_c[fill[p]] = -inv_r[r] * self.row_val[t];
                    fill[p] += 1;
                }
                r += 1;
                self.work += rl as f64 * 11.0 + 3.0;
            }
            self.work += nk as f64;
            for &q in &list {
                slot[q] = -1;
            }
            self.work = self.work + nk as f64 + nk as f64;
            let nr = self.col_cnt[j] as usize;
            debug_assert_eq!(nr, v_r.len(), "5b488d: live rows of column {j}");
            // each row alone: the range of its bound over the box must meet [lb_j, ub_j] (1e-6)
            let (mut lo_r, mut hi_r) = (v_r.clone(), v_r.clone());
            for p in 0..nk {
                let q = list[p];
                for e in seg[p]..seg[p + 1] {
                    let (r, c) = (ent_r[e], ent_c[e]);
                    let (l, u) = (self.lb[q], self.ub[q]);
                    if c <= 0.0 {
                        if -INF < lo_r[r] {
                            lo_r[r] = if INF <= u { -INF } else { lo_r[r] + u * c };
                        }
                        if hi_r[r] < INF {
                            hi_r[r] = if l <= -INF { INF } else { hi_r[r] + l * c };
                        }
                    } else {
                        if -INF < lo_r[r] {
                            lo_r[r] = if l <= -INF { -INF } else { lo_r[r] + l * c };
                        }
                        if hi_r[r] < INF {
                            hi_r[r] = if INF <= u { INF } else { hi_r[r] + c * u };
                        }
                    }
                }
                self.work += (seg[p + 1] - seg[p]) as f64 * 7.0 + 2.0;
            }
            let (lj, uj) = (self.lb[j], self.ub[j]);
            let rows_ok = (0..nr).all(|r| {
                let up_ok = !(uj + 1e-6 < hi_r[r]);
                match sense_r[r] {
                    b'>' => up_ok,
                    s => !(lo_r[r] < lj - 1e-6) && (s == b'<' || up_ok),
                }
            });
            if !rows_ok {
                continue;
            }
            // all rows together, corner by corner (binary counting, last neighbour fastest; values updated in place)
            let mut x: Vec<f64> = list.iter().map(|&q| self.lb[q]).collect();
            let mut val = v_r.clone();
            let (mut plus, mut minus) = (vec![0i32; nr], vec![0i32; nr]);
            self.work += nr as f64 * 3.0;
            for r in 0..nr {
                if !(v_r[r] < INF) {
                    val[r] = 0.0;
                    plus[r] += 1;
                } else if v_r[r] <= -INF {
                    val[r] = 0.0;
                    minus[r] += 1;
                }
            }
            self.work += nr as f64 * 3.0;
            for p in 0..nk {
                let xp = x[p];
                for e in seg[p]..seg[p + 1] {
                    let (r, c) = (ent_r[e], ent_c[e]);
                    if xp <= -INF {
                        if c * xp <= 0.0 {
                            minus[r] += 1;
                        } else {
                            plus[r] += 1;
                        }
                    } else {
                        val[r] += c * xp;
                    }
                }
            }
            self.work += nk as f64 * 5.0;
            let feasible = 'corners: loop {
                let (mut hi, mut lo) = (INF, -INF);
                let mut r = 0;
                while r < nr {
                    let finite = plus[r] == 0 && minus[r] == 0;
                    if sense_r[r] != b'>' {
                        if finite {
                            hi = if hi < val[r] { hi } else { val[r] }; // minsd
                        } else if minus[r] > 0 {
                            hi = -BIG;
                        }
                    }
                    if sense_r[r] != b'<' {
                        if finite {
                            lo = if lo > val[r] { lo } else { val[r] }; // maxsd
                        } else if plus[r] > 0 {
                            lo = BIG;
                        }
                    }
                    if hi + 1e-6 < lo {
                        break;
                    }
                    r += 1;
                }
                self.work += r as f64 * 4.0;
                if r != nr {
                    break false;
                }
                let mut p = nk;
                loop {
                    if p == 0 {
                        break 'corners true;
                    }
                    p -= 1;
                    let (l, u) = (self.lb[list[p]], self.ub[list[p]]);
                    if l == u {
                        continue;
                    }
                    let (old, new) = if x[p] == l { (l, u) } else { (u, l) };
                    let cnt = (seg[p + 1] - seg[p]) as f64;
                    for e in seg[p]..seg[p + 1] {
                        let (r, c) = (ent_r[e], ent_c[e]);
                        if INF <= old.abs() {
                            if c * old > 0.0 {
                                plus[r] -= 1;
                            } else {
                                minus[r] -= 1;
                            }
                        } else {
                            val[r] -= c * old;
                        }
                    }
                    self.work += cnt * 3.0 + 6.0;
                    for e in seg[p]..seg[p + 1] {
                        let (r, c) = (ent_r[e], ent_c[e]);
                        if INF <= new.abs() {
                            if c * new > 0.0 {
                                plus[r] += 1;
                            } else {
                                minus[r] += 1;
                            }
                        } else {
                            val[r] += c * new;
                        }
                    }
                    self.work += cnt * 3.0 + 6.0;
                    x[p] = new;
                    if new == u {
                        continue 'corners;
                    }
                }
            };
            if !feasible {
                continue;
            }
            // the cost moves onto the pivot row's columns (x_j = (b - sum a_k x_k) / a_j)
            let (mut y_idx, mut y_add) = (-1i32, 0.0f64);
            if !(self.obj[j].abs() <= 1e-10) {
                let pr = piv as usize;
                let (rb, rl) = (self.row_beg[pr] as usize, self.row_len[pr] as usize);
                let mut mult = 1.0;
                for t in rb..rb + rl {
                    if self.row_ind[t] == j as i32 {
                        mult = -self.obj[j] / self.row_val[t];
                        break;
                    }
                }
                for t in rb..rb + rl {
                    let q = self.row_ind[t];
                    if q >= 0 && self.col_cnt[q as usize] >= 0 {
                        self.obj[q as usize] += mult * self.row_val[t];
                    }
                }
                y_idx = piv;
                y_add = mult;
                self.obj[j] = 0.0;
                self.objcon -= mult * self.rhs[pr];
            }
            let mut rows = Vec::with_capacity(nr);
            for k in 0..len {
                let i = self.col_ind[b + k];
                if i < 0 || self.row_cnt[i as usize] < 0 {
                    continue;
                }
                let i = i as usize;
                let rb = self.row_beg[i] as usize;
                let terms = (rb..rb + self.row_len[i] as usize)
                    .filter(|&t| self.row_ind[t] >= 0 && self.col_cnt[self.row_ind[t] as usize] >= 0)
                    .map(|t| (self.row_ind[t], self.row_val[t]))
                    .collect();
                rows.push(ColRow { row: i as i32, sense: self.sense[i], rhs: self.rhs[i], terms });
            }
            self.post.push(PostRec::ColRows { j: j as i32, lb: lj, ub: uj, vtype: b'C', y_idx, y_add, rows });
            // drop j's rows and j itself; no tracker marks, `+0xd8` unchanged (as the original)
            for k in 0..len {
                let i = self.col_ind[b + k];
                if i < 0 || self.row_cnt[i as usize] < 0 {
                    continue;
                }
                let i = i as usize;
                self.row_cnt[i] = -1;
                let rb = self.row_beg[i] as usize;
                for t in rb..rb + self.row_len[i] as usize {
                    let q = self.row_ind[t];
                    if q >= 0 && self.col_cnt[q as usize] >= 0 {
                        self.col_cnt[q as usize] -= 1;
                        self.row_ind[t] = -1;
                    }
                }
                self.col_ind[b + k] = -1;
                self.rows_removed += 1;
            }
            self.col_len[j] = -1;
            self.col_cnt[j] = -2;
            self.cols_removed += 1;
        }
        0
    }
}
