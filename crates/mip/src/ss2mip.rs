//! `594c0c` PRESS2MIP (docs/m2/m2_red_roundend.md §6): column sets that appear in many rows with one coefficient
//! (up to sign) become a new continuous column s = Σ sign·x with a defining `=` row; the occurrences keep one entry
//! on s. Work outside `self.work` is gathered in `l70` (local_70) and added where the original flushes it.
use crate::state::{State, NIL};
use crate::subst::sort_asc;
use crate::probe::sort_desc;
use crate::Params;
use std::collections::HashMap;

/// `frexp` for finite values: (mantissa in [0.5, 1), exponent).
fn frexp(v: f64) -> (f64, i32) {
    if v == 0.0 || !v.is_finite() {
        return (v, 0);
    }
    let b = v.to_bits();
    let e = ((b >> 52) & 0x7ff) as i32;
    if e == 0 {
        let (m, k) = frexp(v * 18014398509481984.0);
        return (m, k - 54);
    }
    (f64::from_bits((b & !(0x7ffu64 << 52)) | (1022u64 << 52)), e - 1022)
}

/// `591f99`: hash of an occurrence (row, |value|).
fn occ_hash(r: i32, v: f64) -> u32 {
    let (m, e) = frexp(v);
    let k = if e <= 0 { e } else { 0 } + 30;
    let x = ((m + 1e-14) * 2f64.powi(k)).trunc();
    (!r).wrapping_add(x as i64 as i32) as u32
}

/// The pattern store of `5941d6` (ctx[1..6]) with its hash table (`5919f4` / `591ba8`).
#[derive(Default)]
struct Pats {
    start: Vec<usize>,
    cols: Vec<i32>,
    signs: Vec<i8>,
    count: Vec<i32>,
    head: Vec<i32>,
    table: HashMap<u32, Vec<usize>>,
    /// occurrences {value, next, row}
    occ: Vec<(f64, i32, i32)>,
}

impl Pats {
    fn len(&self, id: usize) -> usize {
        self.start[id + 1] - self.start[id]
    }

    /// `5919f4`; W (l70) += 2·len.
    fn hash(cols: &[i32], signs: &[i8], l70: &mut f64) -> u32 {
        let mut h: i32 = 0;
        for (&c, &s) in cols.iter().zip(signs) {
            h = h.wrapping_mul(7).wrapping_add(!c).wrapping_add(if s > 0 { 0x100000 } else { 0x400 });
        }
        *l70 = cols.len() as f64 + cols.len() as f64 + *l70;
        h as u32
    }

    /// `5941d6(v, env, ctx, row, start, len)` on the sorted-by-|a| buffer: the run's columns (sorted) and signs
    /// (first positive) are a pattern; record the occurrence.
    fn add(&mut self, v: f64, row: i32, cols: &mut [i32], vals: &mut [f64], l70: &mut f64) {
        let len = cols.len();
        sort_asc(cols, vals); // `a9f78f`
        let mut val = v;
        if !(0.0 <= vals[0]) {
            for x in vals.iter_mut() {
                *x = -*x;
            }
            *l70 += len as f64;
            val = -v;
        }
        let signs: Vec<i8> = vals.iter().map(|&x| if x <= 0.0 { -1 } else { 1 }).collect();
        *l70 += len as f64 * 3.0;
        let h = Self::hash(cols, &signs, l70);
        let mut found = None;
        if let Some(ids) = self.table.get(&h) {
            for &id in ids.iter().rev() {
                let (s, l) = (self.start[id], self.len(id));
                if l != len {
                    continue;
                }
                let k = (0..len).find(|&t| self.cols[s + t] != cols[t] || self.signs[s + t] != signs[t]).unwrap_or(len);
                *l70 = k as f64 + k as f64 + *l70;
                if k == len {
                    found = Some(id);
                    break;
                }
            }
        }
        let id = match found {
            Some(id) => id,
            None => {
                let id = self.count.len();
                if self.start.is_empty() {
                    self.start.push(0);
                }
                self.cols.extend_from_slice(cols);
                self.signs.extend_from_slice(&signs);
                self.start.push(self.cols.len());
                self.count.push(0);
                self.head.push(-1);
                *l70 = len as f64 + len as f64 + *l70;
                Self::hash(cols, &signs, l70);
                self.table.entry(h).or_default().push(id);
                id
            }
        };
        self.occ.push((val, self.head[id], row));
        self.head[id] = (self.occ.len() - 1) as i32;
        self.count[id] += 1;
    }
}

impl State {
    /// `594c0c(env, st, T20)`. Returns 0 or `36c5cf`'s status.
    /// shortcut: the timer test (`adf2be`) is left out (never reached on the 38 models); `56b360` (doubleton `=`
    /// rows left by the substitution) is skipped: those rows stay for later reductions (0 calls on the 38 models;
    /// port it when a model reaches it).
    pub fn ss2mip(&mut self, p: &Params) -> i32 {
        if self.nnz > 2_000_000_000 {
            return 0;
        }
        let (n, m, mu) = (self.n, self.m, self.work_mult);
        let mut rcnt = self.row_cnt.clone();
        let mut ccnt = self.col_cnt.clone();
        let tot: i64 = rcnt.iter().map(|&c| c.max(0) as i64).sum();
        if tot == 0 {
            return 0;
        }
        let mut rord: Vec<i32> = (0..m as i32).collect();
        let mut cord: Vec<i32> = (0..n as i32).collect();
        let mut l70 = ((n as f64 + m as f64) + m as f64) * 2.0;
        // 1. peel columns with <= 3 and rows with <= 32 entries (marks in node.row) until nothing changes
        let (mut pc, mut pr) = (0, 0);
        loop {
            let mut changed = false;
            let mut k: Vec<i32> = cord[pc..].iter().map(|&j| ccnt[j as usize]).collect();
            sort_asc(&mut k, &mut cord[pc..]);
            let mut cnt = 0;
            while pc < n {
                let j = cord[pc] as usize;
                if ccnt[j] > 3 {
                    break;
                }
                let mut q = self.col_head[j];
                while q != NIL {
                    let nd = self.nd[q as usize];
                    if nd.row >= 0 && nd.col >= 0 {
                        self.nd[q as usize].row = -1 - nd.row;
                        changed = true;
                        rcnt[nd.row as usize] -= 1;
                        ccnt[j] -= 1;
                    }
                    cnt += 1;
                    q = nd.nc;
                }
                pc += 1;
            }
            l70 = (mu + 2.0) * cnt as f64 + l70;
            let mut k: Vec<i32> = rord[pr..].iter().map(|&i| rcnt[i as usize]).collect();
            sort_asc(&mut k, &mut rord[pr..]);
            let mut cnt = 0;
            while pr < m {
                let i = rord[pr] as usize;
                if rcnt[i] > 32 {
                    break;
                }
                let mut q = self.row_head[i];
                while q != NIL {
                    let nd = self.nd[q as usize];
                    if nd.row >= 0 && nd.col >= 0 {
                        self.nd[q as usize].row = -1 - nd.row;
                        changed = true;
                        rcnt[i] -= 1;
                        ccnt[nd.col as usize] -= 1;
                    }
                    cnt += 1;
                    q = nd.nr;
                }
                pr += 1;
            }
            l70 = (2.0 + mu) * cnt as f64 + l70;
            if !changed {
                break;
            }
        }
        let mut cnt = 0;
        for (lists, k, by_row) in [(&rord, pr, true), (&cord, pc, false)] {
            for &x in &lists[..k] {
                let mut q = if by_row { self.row_head[x as usize] } else { self.col_head[x as usize] };
                while q != NIL {
                    let nd = &mut self.nd[q as usize];
                    if nd.row < 0 {
                        nd.row = -1 - nd.row;
                    }
                    cnt += 1;
                    q = if by_row { nd.nr } else { nd.nc };
                }
            }
        }
        self.work = (l70 + mu * cnt as f64) + self.work;
        self.budget_check();
        // 2. per remaining row: entries of bounded columns kept by the peel, by |a| descending; runs of >= 3 equal
        // |a| (1e-13) are pattern occurrences
        let mut pats = Pats::default();
        l70 = 0.0;
        let mut pp = pr;
        while pp < m {
            let i = rord[pp] as usize;
            let (mut cols, mut vals) = (Vec::new(), Vec::new());
            let mut walked = 0;
            let mut q = self.row_head[i];
            while q != NIL {
                let nd = self.nd[q as usize];
                if nd.col >= 0 {
                    let j = nd.col as usize;
                    if -1e30 < self.lb[j] && self.ub[j] < 1e30 && ccnt[j] > 3 {
                        cols.push(nd.col);
                        vals.push(nd.val);
                    }
                }
                walked += 1;
                q = nd.nr;
            }
            // `ac3523`: |a| descending (ties are resorted by column inside each run, so their order does not matter)
            let mut ix: Vec<usize> = (0..cols.len()).collect();
            ix.sort_by(|&a, &b| vals[b].abs().total_cmp(&vals[a].abs()));
            let mut cols: Vec<i32> = ix.iter().map(|&t| cols[t]).collect();
            let mut vals: Vec<f64> = ix.iter().map(|&t| vals[t]).collect();
            let k = cols.len();
            let (mut prev, mut run, mut st, mut t) = (1e100, 0usize, 0usize, 0usize);
            while t < k {
                let a = vals[t].abs();
                if 1e-13 < (a - prev).abs() {
                    if run >= 3 {
                        pats.add(prev, i as i32, &mut cols[st..st + run], &mut vals[st..st + run], &mut l70);
                    }
                    run = 0;
                    st = t;
                    prev = a;
                    if 1e-7 > a {
                        break;
                    }
                }
                t += 1;
                run += 1;
            }
            l70 = (((walked as f64 + walked as f64) * mu + k as f64) + k as f64) + t as f64 + l70;
            if run >= 3 {
                pats.add(prev, i as i32, &mut cols[st..st + run], &mut vals[st..st + run], &mut l70);
            }
            pp += 1;
        }
        self.work = self.work + ((m - pp) as f64 * 3.0 + l70);
        self.budget_check();
        let npat = pats.count.len();
        if npat == 0 {
            return 0;
        }
        // 3. candidates: score (len − 1)·count − (len + 1) > 47 with count > 2, best first
        let (mut cand, mut score) = (Vec::new(), Vec::new());
        for id in 0..npat {
            let len = pats.len(id) as i32;
            let s = (len - 1).wrapping_mul(pats.count[id]) - (len + 1);
            if s >= 48 && pats.count[id] >= 3 {
                cand.push(id as i32);
                score.push(s as f64);
            }
        }
        self.work = self.work + ((npat as f64 * 3.0 + 0.0) + cand.len() as f64);
        self.budget_check();
        if cand.is_empty() {
            return 0;
        }
        sort_desc(&mut score, &mut cand);
        let ncand = {
            let c = cand.len() as f64;
            let x = m as f64 * 0.2;
            (if c < x { c } else { x }) as usize
        };
        let mut best = score[0] as i32;
        // occurrence table (`591f99` / `5920f1`): (row, |value|) of the top candidates' occurrences
        let (mut orow, mut oval): (Vec<i32>, Vec<f64>) = (Vec::new(), Vec::new());
        let mut otab: HashMap<u32, Vec<usize>> = HashMap::new();
        let put = |otab: &mut HashMap<u32, Vec<usize>>, orow: &mut Vec<i32>, oval: &mut Vec<f64>, r: i32, v: f64| {
            otab.entry(occ_hash(r, v)).or_default().push(orow.len());
            orow.push(r);
            oval.push(v);
        };
        for &id in &cand[..ncand] {
            let mut k = pats.head[id as usize];
            while k >= 0 {
                let (v, next, r) = pats.occ[k as usize];
                put(&mut otab, &mut orow, &mut oval, r, v.abs());
                k = next;
            }
        }
        l70 = ((ncand as f64 + ncand as f64) + orow.len() as f64 * 11.0) + 0.0;
        self.work = self.work + l70;
        self.budget_check();
        // 4. extend: rows holding all of a pattern's columns with one value (up to the signs) become occurrences
        let mut mark = vec![0.0f64; m];
        let mut csign = vec![0i8; n];
        let mut seen = 0i64; // local_1f0
        l70 = 0.0;
        let mut t = 0;
        while t < ncand {
            if !(best as f64 * 0.1125 <= score[t]) {
                break;
            }
            let id = cand[t] as usize;
            let (s0, plen) = (pats.start[id], pats.len(id));
            let mut k = pats.head[id];
            while k >= 0 {
                let (v, next, r) = pats.occ[k as usize];
                mark[r as usize] = v;
                k = next;
            }
            let (mut amin, mut cmin) = (-1i32, 2_000_000_000);
            for c in 0..plen {
                let col = pats.cols[s0 + c];
                csign[col as usize] = pats.signs[s0 + c];
                if self.col_cnt[col as usize] < cmin {
                    cmin = self.col_cnt[col as usize];
                    amin = col;
                }
            }
            let mut q = self.col_head[amin as usize];
            while q != NIL {
                let nd = self.nd[q as usize];
                q = nd.nc;
                seen += 1;
                let r = nd.row as usize;
                if nd.col < 0 || mark[r] != 0.0 {
                    continue;
                }
                let rc = self.row_cnt[r];
                let lim = if plen as i32 * 10 > 0x2000 { plen as i32 * 10 } else { 0x2000 };
                if rc < plen as i32 || rc > lim {
                    continue;
                }
                let v = nd.val / csign[amin as usize] as f64;
                let av = v.abs();
                if 1e-7 > av {
                    continue;
                }
                let h = occ_hash(r as i32, av);
                if otab.get(&h).is_some_and(|ids| ids.iter().any(|&e| orow[e] == r as i32 && !(1e-13 <= (oval[e] - av).abs()))) {
                    continue;
                }
                let (mut first, mut got, mut ok) = (1e101, 0usize, false);
                let mut w = self.row_head[r];
                while w != NIL {
                    let e = self.nd[w as usize];
                    if e.col >= 0 && csign[e.col as usize] != 0 {
                        if e.col == pats.cols[s0] {
                            first = e.val;
                        }
                        got += 1;
                        if (v - e.val / csign[e.col as usize] as f64).abs() > 1e-13 {
                            break;
                        }
                        if got == plen {
                            ok = true;
                            break;
                        }
                    }
                    seen += 1;
                    w = e.nr;
                }
                if !ok {
                    continue;
                }
                put(&mut otab, &mut orow, &mut oval, r as i32, av);
                pats.occ.push((first, pats.head[id], r as i32));
                pats.head[id] = (pats.occ.len() - 1) as i32;
                pats.count[id] += 1;
                score[t] += (got - 1) as f64;
            }
            for c in 0..plen {
                csign[pats.cols[s0 + c] as usize] = 0;
            }
            let mut k = pats.head[id];
            while k >= 0 {
                let (_, next, r) = pats.occ[k as usize];
                mark[r as usize] = 0.0;
                k = next;
            }
            l70 = plen as f64 * 12.0 + l70;
            if t % 100 == 99 {
                self.work = self.work + l70;
                l70 = 0.0;
                self.budget_check();
            }
            t += 1;
        }
        l70 = ((seen as f64 * mu + t as f64) * 3.0) + l70;
        sort_desc(&mut score[..ncand], &mut cand[..ncand]);
        best = score[0] as i32;
        let (mut big, mut sum) = (0, 0i64);
        for i in 0..m {
            if self.row_cnt[i] >= 3 {
                big += 1;
            }
            sum += self.row_cnt[i].max(0) as i64;
        }
        let lim = {
            let x = (big as f64 * 0.1) as i32;
            if x > 0x200 { x } else { 0x200 }
        };
        l70 += m as f64;
        let nnew = if (ncand as i32) < lim { ncand } else { lim as usize };
        let c0 = self.add_cols(nnew);
        let r0 = self.add_rows(nnew);
        // 5. substitute
        let (mut singles, mut doubles): (Vec<i32>, Vec<i32>) = (Vec::new(), Vec::new());
        let mut gain = 0.0; // local_98
        let mut t = 0;
        while t < nnew {
            if best as f64 * 0.125 > score[t] {
                break;
            }
            gain += score[t];
            let id = cand[t] as usize;
            let (s0, plen) = (pats.start[id], pats.len(id));
            let (nc, nr) = (c0 + t, r0 + t);
            let mut nocc = 0;
            let mut k = pats.head[id];
            while k >= 0 {
                let (v, next, r) = pats.occ[k as usize];
                mark[r as usize] = v;
                nocc += 1;
                k = next;
            }
            let mut hits = 0;
            let mut visited = 0i64;
            for c in 0..plen {
                let mut q = self.col_head[pats.cols[s0 + c] as usize];
                while q != NIL {
                    let nd = self.nd[q as usize];
                    if nd.col >= 0 && mark[nd.row as usize] != 0.0 {
                        hits += 1;
                    }
                    visited += 1;
                    q = nd.nc;
                }
            }
            if hits != nocc * plen {
                let mut k = pats.head[id];
                while k >= 0 {
                    let (_, next, r) = pats.occ[k as usize];
                    mark[r as usize] = 0.0;
                    k = next;
                }
                t += 1;
                continue;
            }
            // first column's entries in occurrence rows move to the new column (value = the occurrence's)
            let c1 = pats.cols[s0] as usize;
            let mut prev = NIL;
            let mut q = self.col_head[c1];
            while q != NIL {
                let nd = self.nd[q as usize];
                let next = nd.nc;
                if nd.col >= 0 && mark[nd.row as usize] != 0.0 {
                    if prev == NIL { self.col_head[c1] = next } else { self.nd[prev as usize].nc = next }
                    let e = &mut self.nd[q as usize];
                    e.col = nc as i32;
                    e.val = mark[nd.row as usize];
                    e.nc = self.col_head[nc];
                    self.col_head[nc] = q;
                    self.col_cnt[nc] += 1;
                    self.col_cnt[c1] -= 1;
                } else {
                    prev = q;
                }
                visited += 1;
                q = next;
            }
            for c in 1..plen {
                let col = pats.cols[s0 + c] as usize;
                let mut q = self.col_head[col];
                while q != NIL {
                    let nd = self.nd[q as usize];
                    let r = nd.row as usize;
                    if nd.col >= 0 && mark[r] != 0.0 {
                        self.col_cnt[col] -= 1;
                        self.row_cnt[r] -= 1;
                        self.nd[q as usize].col = -1;
                        if self.row_cnt[r] == 1 {
                            singles.push(r as i32);
                        } else if self.row_cnt[r] == 2 && self.sense[r] == b'=' {
                            doubles.push(r as i32);
                        }
                    }
                    visited += 1;
                    q = nd.nc;
                }
            }
            let mut k = pats.head[id];
            while k >= 0 {
                let (_, next, r) = pats.occ[k as usize];
                mark[r as usize] = 0.0;
                self.tracker.mark_row(r);
                k = next;
            }
            self.sense[nr] = b'=';
            let (mut lo, mut up, mut nlo, mut nup) = (0.0f64, 0.0f64, 0, 0);
            for c in 0..plen {
                let (col, sg) = (pats.cols[s0 + c] as usize, pats.signs[s0 + c]);
                self.add_node(col, nr, sg as f64);
                let (l, u) = (self.lb[col], self.ub[col]);
                if sg > 0 {
                    if -1e30 < l { lo += l } else { nlo += 1 }
                    if u < 1e30 { up += u } else { nup += 1 }
                } else {
                    if -1e30 < l { up -= l } else { nup += 1 }
                    if u < 1e30 { lo -= u } else { nlo += 1 }
                }
                self.tracker.mark_col(col as i32);
            }
            self.add_node(nc, nr, -1.0);
            self.lb[nc] = if nlo != 0 || !(-1e30 < lo) { -1e30 } else { lo };
            self.ub[nc] = if nup != 0 || !(up < 1e30) { 1e30 } else { up };
            self.activity(Some(&[nr as i32]));
            let mut k = pats.head[id];
            while k >= 0 {
                let (_, next, r) = pats.occ[k as usize];
                self.activity(Some(&[r]));
                k = next;
            }
            l70 = ((plen as f64 * 6.0 + l70) + pats.count[id] as f64 * 10.0) + (mu * 3.0) * visited as f64;
            if t % 100 == 99 {
                self.work = self.work + l70;
                l70 = 0.0;
                self.budget_check();
            }
            t += 1;
        }
        l70 = t as f64 * 4.0 + l70;
        if 1024.0 < gain && sum as f64 * 0.1 < gain {
            self.gc(1);
        }
        if !singles.is_empty() {
            let mut g = 0;
            let rc = self.singleton_rows_in(p, &singles, &mut g);
            if rc != 0 {
                return rc;
            }
        }
        let _ = doubles; // shortcut: `56b360` skipped, see the doc comment
        self.work += l70;
        0
    }
}
