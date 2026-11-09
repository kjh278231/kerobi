//! `564ce0(st, env, map, sc, of, upd, &out)`: substitute columns x_j = sc_j·x_map[j] + of_j, grouped by
//! representative (`55e383` per group, type 2 records through `push_agg`), docs/m2/m2_red_probe.md §3.5. Probing calls
//! it for its aggregations; other reductions (5466a7, 5577ba, 522aa2) share it.
use crate::state::{State, INF, NIL};

/// `aa6825` / `aaedd9`: ascending sort of `key`, payload moved along with the original's exact element movement
/// (crates/lp/src/presolve/sort.rs `sort_by_key`).
pub fn sort_asc<K: PartialOrd + Copy, T: Copy>(a: &mut [K], ix: &mut [T]) {
    quick(0, a.len(), 0, a, ix);
}

fn quick<K: PartialOrd + Copy, T: Copy>(mut lo: usize, mut hi: usize, mut depth: i32, a: &mut [K], ix: &mut [T]) {
    while depth < 50 {
        let len = hi - lo;
        if len < 100 {
            break;
        }
        let (d2, d3, d4) = (a[lo], a[hi - 1], a[len / 2 + lo]);
        let mut piv = d2;
        if (d2 < d4 || d3 < d2) && (d4 < d2 || d2 < d3) {
            piv = d4;
            if (d4 < d2 || d3 < d4) && (d2 < d4 || d4 < d3) {
                piv = d3;
            }
        }
        let (mut i, mut j, mut k) = (lo, hi, lo);
        if !(hi <= lo || piv <= d2) {
            loop {
                i += 1;
                k = i;
                if hi <= i || !(a[i] < piv) {
                    break;
                }
            }
        }
        while i < j {
            let x = a[i];
            if piv <= x {
                if x == piv {
                    i += 1;
                    continue;
                }
                j -= 1;
                let mut y = a[j];
                let mut done = false;
                while piv <= y {
                    if y == piv {
                        if i < j {
                            a[i] = y;
                            a[j] = x;
                            ix.swap(i, j);
                            i += 1;
                        }
                        done = true;
                        break;
                    }
                    if j <= i {
                        done = true;
                        break;
                    }
                    j -= 1;
                    y = a[j];
                }
                if done {
                    continue;
                }
                if i < j {
                    let (xi, xk, xj) = (ix[i], ix[k], ix[j]);
                    a[i] = a[k];
                    ix[i] = xk;
                    i += 1;
                    a[k] = a[j];
                    ix[k] = xj;
                    k += 1;
                    a[j] = x;
                    ix[j] = xi;
                }
            } else {
                a[i] = a[k];
                ix.swap(i, k);
                i += 1;
                a[k] = x;
                k += 1;
            }
        }
        depth += 1;
        if hi - j < k - lo {
            quick(j, hi, depth, a, ix);
            hi = k;
        } else {
            quick(lo, k, depth, a, ix);
            lo = j;
        }
    }
    let mut g = (hi - lo) / 2;
    while g > 0 {
        loop {
            let mut i = lo + g;
            while i < hi {
                let (key, id) = (a[i], ix[i]);
                let mut p = i;
                while lo + g <= p && a[p - g] > key {
                    a[p] = a[p - g];
                    ix[p] = ix[p - g];
                    p -= g;
                }
                a[p] = key;
                ix[p] = id;
                i += 1;
            }
            if g == 2 && lo + 1 < hi {
                g = 1;
                continue;
            }
            break;
        }
        g = (g as f64 / 2.2) as usize;
    }
}

impl State {
    /// One bound's term in row r's activities, plain (no 1-ulp guard): `add` adds a·b, else subtracts it.
    fn plain_term(&mut self, r: usize, a: f64, b: f64, upper: bool, add: bool) {
        if a == 0.0 {
            return;
        }
        let finite = if upper { b < INF } else { -INF < b };
        let max_side = if a > 0.0 { upper } else { !upper };
        if !finite {
            let c = if max_side { &mut self.inf_max[r] } else { &mut self.inf_min[r] };
            *c += if add { 1 } else { -1 };
            return;
        }
        let x = if max_side { &mut self.act_max[r] } else { &mut self.act_min[r] };
        *x = if add { b * a + *x } else { *x - b * a };
    }

    /// `remove` / `add` of `55e383`: the column's two bound terms (lb first).
    fn plain_bounds(&mut self, r: usize, a: f64, l: f64, u: f64, add: bool) {
        self.plain_term(r, a, l, false, add);
        self.plain_term(r, a, u, true, add);
    }

    /// `564ce0` with scale/offset arrays (`sc`/`of`, both given by every caller seen) and activity updates `upd`.
    /// Returns (status, substituted count).
    /// shortcut: column flags (SOS 1, Q 0x08/0x30), PREQSUBST, the SOS block and `560ea1`/`562f34` are not ported;
    /// all flags are 0 on the linear models handled. The 2-cycle branch of the path compression never ran.
    pub fn substitute(&mut self, map: &mut [i32], sc: &mut [f64], of: &mut [f64], upd: bool) -> (i32, usize) {
        let n = self.n;
        let mu = self.work_mult;
        // 1. path compression (work without μ)
        for j in 0..n {
            let mut k = map[j];
            if k < 0 {
                continue;
            }
            let (mut s, mut o) = (sc[j], of[j]);
            let mut steps = 0;
            while map[k as usize] >= 0 && map[k as usize] != k && map[k as usize] != j as i32 {
                let ku = k as usize;
                let q = map[ku];
                let r = map[q as usize];
                steps += 1;
                let next = if r < 0 || r == q || r == j as i32 {
                    q
                } else if r == k {
                    map[q as usize] = -1;
                    map[ku]
                } else {
                    of[ku] = of[q as usize] * sc[ku] + of[ku];
                    sc[ku] *= sc[q as usize];
                    map[ku] = r;
                    r
                };
                o += of[ku] * s;
                s *= sc[ku];
                k = next;
            }
            self.work += steps as f64 * 4.0;
            map[j] = k;
            sc[j] = s;
            of[j] = o;
            if map[k as usize] == j as i32 {
                map[k as usize] = -1;
            }
        }
        self.work += n as f64 * 3.0;
        // 2. candidates (no column flags)
        let (mut rr, mut pay) = (Vec::new(), Vec::new());
        for j in 0..n {
            let r = map[j];
            if r < 0 {
                continue;
            }
            if self.col_flag[j] & 0xffff_ffc6 == 0 && self.col_cnt[j] >= 0 && self.col_cnt[r as usize] >= 0 {
                rr.push(r);
                pay.push((j as i32, sc[j], of[j]));
            } else {
                map[j] = -1;
            }
        }
        self.work += (n as f64 * 3.0) * mu;
        let cnt = rr.len();
        if cnt == 0 {
            return (0, 0);
        }
        sort_asc(&mut rr, &mut pay);
        // 3. groups by representative
        let (mut acc, mut g0) = (0i64, 0);
        while g0 < cnt {
            let mut g1 = g0;
            while g1 < cnt && rr[g1] == rr[g0] {
                acc += self.col_cnt[pay[g1].0 as usize] as i64;
                g1 += 1;
            }
            if self.nnz < acc {
                self.gc(1);
                acc = 0;
            }
            self.subst_group(rr[g0] as usize, &pay[g0..g1], upd);
            g0 = g1;
        }
        self.work += (cnt as f64 * 3.0) * mu;
        (0, cnt)
    }

    /// `55e383(st, env, k, cnt, J, S, O, upd)`: move each column j's live nodes into column k scaled by s (merging
    /// with k's node in the same row), shift the rhs by o·a, remove j (−3, type 2 record), then drop near-zero merged
    /// entries of k and mark its rows.
    fn subst_group(&mut self, k: usize, g: &[(i32, f64, f64)], upd: bool) {
        let mu = self.work_mult;
        let mut pos = vec![NIL; self.m];
        let mut p = self.col_head[k];
        while p != NIL {
            let q = self.nd[p as usize];
            if q.col >= 0 {
                pos[q.row as usize] = p;
                if upd {
                    let (l, u) = (self.lb[k], self.ub[k]);
                    self.plain_bounds(q.row as usize, q.val, l, u, false);
                }
            }
            p = q.nc;
        }
        self.work += (self.col_cnt[k] as f64 * 3.0) * mu;
        for &(j, s, o) in g {
            let j = j as usize;
            let mut node = self.col_head[j];
            self.col_head[j] = NIL;
            while node != NIL {
                let at = node as usize;
                let nxt = self.nd[at].nc;
                self.nd[at].nc = NIL;
                let q = self.nd[at];
                if q.col < 0 {
                    self.nd[at].nc = self.col_head[j];
                    self.col_head[j] = node;
                    node = nxt;
                    continue;
                }
                let (i, a) = (q.row as usize, q.val);
                if upd {
                    let (l, u) = (self.lb[j], self.ub[j]);
                    self.plain_bounds(i, a, l, u, false);
                    self.act_max[i] = a * o + self.act_max[i];
                    self.act_min[i] = a * o + self.act_min[i];
                }
                self.rhs[i] -= o * a;
                if pos[i] == NIL {
                    pos[i] = node;
                    self.nd[at].nc = self.col_head[k];
                    self.nd[at].col = k as i32;
                    self.nd[at].val = s * a;
                    self.col_head[k] = node;
                    self.col_cnt[k] += 1;
                } else {
                    self.nd[at].col = -1;
                    self.nd[at].nc = self.col_head[j];
                    self.col_head[j] = node;
                    self.row_cnt[i] -= 1;
                    let t = pos[i] as usize;
                    self.nd[t].val = s * a + self.nd[t].val;
                }
                node = nxt;
            }
            self.work += (self.col_cnt[j] as f64 * 12.0) * mu;
            let c = self.obj[j];
            self.col_cnt[j] = -3;
            self.cols_removed += 1;
            self.obj[k] = c * s + self.obj[k];
            self.objcon = self.obj[j] * o + self.objcon;
            self.obj[j] = 0.0;
            self.push_agg(j, k, s, o);
        }
        let mut p = self.col_head[k];
        while p != NIL {
            let at = p as usize;
            let q = self.nd[at];
            p = q.nc;
            if q.col < 0 {
                continue;
            }
            let i = q.row as usize;
            pos[i] = NIL;
            if q.val.abs() <= 1e-10 {
                self.nd[at].col = -1;
                self.col_cnt[k] -= 1;
                self.nd[at].val = 0.0;
                self.row_cnt[i] -= 1;
            } else if upd {
                let (l, u) = (self.lb[k], self.ub[k]);
                self.plain_bounds(i, q.val, l, u, true);
            }
            self.tracker.mark_row(i as i32);
        }
        self.work += (self.col_cnt[k] as f64 * 6.0) * mu;
        self.tracker.mark_col(k as i32);
    }
}
