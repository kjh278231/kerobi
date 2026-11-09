//! `419ffc` (PRECOVERS, timer 23): ±1 binary `<` rows with rhs = (positive terms) − 1 say "not all of these literals
//! are true" (literal x_j for a > 0, its complement for a < 0). A row whose literal set holds another's is dropped;
//! one that holds another's with one literal flipped loses that literal (resolution). docs/m2/m2_red_roundend_b.md §7.
use crate::state::{State, NIL};
use crate::subst::sort_asc;
use crate::Params;

/// `77aab(nmin, nmaj, beg, len, ind, 0, obeg, olen, oind, 0, 0, W)` without values: transpose (minor lines list
/// their majors in increasing order). W += 2·len per major, 3·nmaj, 2·nmin, 3·len per major, 3·nmaj.
fn transpose(nmin: usize, beg: &[usize], len: &[usize], ind: &[usize], w: &mut f64) -> (Vec<usize>, Vec<usize>, Vec<usize>) {
    let mut cnt = vec![0usize; nmin];
    for (b, l) in beg.iter().zip(len) {
        for &k in &ind[*b..b + l] {
            cnt[k] += 1;
        }
        *w += *l as f64 * 2.0;
    }
    *w += len.len() as f64 * 3.0;
    let mut ob = vec![0usize; nmin + 1];
    for i in 0..nmin {
        ob[i + 1] = ob[i] + cnt[i];
    }
    *w += nmin as f64 + nmin as f64;
    let mut oind = vec![0usize; ob[nmin]];
    let mut fill = ob.clone();
    for (t, (b, l)) in beg.iter().zip(len).enumerate() {
        for &k in &ind[*b..b + l] {
            oind[fill[k]] = t;
            fill[k] += 1;
        }
        *w += *l as f64 * 3.0;
    }
    *w += len.len() as f64 * 3.0;
    (ob, cnt, oind)
}

/// The two 64-bit literal signatures: bit (l mod 64) and bit (l mod 63).
fn sigs(lits: &[usize]) -> (u64, u64) {
    lits.iter().fold((0, 0), |(a, b), &l| (a | 1 << (l & 63), b | 1 << (l % 63)))
}

impl State {
    /// `419ffc(st, env, timer)`. Returns 0, or 3 when a row loses its last literal (`bad_col`, `bad_row`).
    /// shortcut: the reduction timer (`adf2be`) is not passed in and +0x47c (the other row) is not kept; the work cap
    /// below is ported and neither binds on the 38 models.
    pub fn covers(&mut self, _p: &Params) -> i32 {
        let (n, m) = (self.n, self.m);
        if 2e9 < n as f64 + 1.0 + n as f64 {
            return 0;
        }
        let w0 = self.work;
        // candidate rows
        let mut cand = vec![-1i32; m];
        let (mut k, mut nnz) = (0usize, 0usize);
        for i in 0..m {
            if !(2..500).contains(&self.row_cnt[i]) || self.sense[i] != b'<' {
                continue;
            }
            let (mut neg, mut pos, mut ok) = (0.0f64, 0.0f64, true);
            let mut e = self.row_head[i];
            while e != NIL {
                let y = self.nd[e as usize];
                e = y.nr;
                if y.col < 0 {
                    continue;
                }
                if y.val <= 0.0 {
                    neg += 1.0;
                } else {
                    pos += 1.0;
                }
                if self.vtype[y.col as usize] != b'B' || 1e-10 < (y.val.abs() - 1.0).abs() {
                    ok = false;
                    break;
                }
            }
            self.work += (neg + pos) * 3.0;
            if ok && ((self.rhs[i] + 1.0) - pos).abs() <= 1e-10 && 2.0 <= neg + pos {
                cand[i] = k as i32;
                k += 1;
                nnz += self.row_cnt[i] as usize;
            }
        }
        self.work += m as f64;
        if k < 2 {
            return 0;
        }
        let limit = (self.budget / 5.0).min(self.nnz as f64 * 1000.0 + 1e6);
        // literal rows (x_j: j + n, complement: j), then sorted through two transposes
        let (mut beg, mut len, mut lit) = (Vec::with_capacity(k), Vec::with_capacity(k), Vec::with_capacity(nnz));
        for i in 0..m {
            if cand[i] < 0 {
                continue;
            }
            beg.push(lit.len());
            let mut e = self.row_head[i];
            while e != NIL {
                let y = self.nd[e as usize];
                if y.col >= 0 {
                    lit.push(if 0.0 <= y.val { y.col as usize + n } else { y.col as usize });
                }
                e = y.nr;
            }
            len.push(lit.len() - beg[beg.len() - 1]);
        }
        self.work += m as f64 + lit.len() as f64 * 4.0 + k as f64 * 3.0;
        let mut w = self.work;
        let (obeg, olen, oind) = transpose(2 * n, &beg, &len, &lit, &mut w);
        let (_, _, sorted) = transpose(k, &obeg, &olen, &oind, &mut w);
        self.work = w;
        lit = sorted;
        let (mut s1, mut s2) = (vec![0u64; k], vec![0u64; k]);
        for r in 0..k {
            (s1[r], s2[r]) = sigs(&lit[beg[r]..beg[r] + len[r]]);
            self.work += len[r] as f64;
        }
        // rows by (smallest literal count) · length
        let mut key: Vec<f64> = (0..k)
            .map(|r| {
                let mn = lit[beg[r]..beg[r] + len[r]].iter().fold(2 * n as i32 + 1, |a, &l| a.min(olen[l] as i32));
                (mn * len[r] as i32) as f64
            })
            .collect();
        let mut order: Vec<i32> = (0..k as i32).collect();
        sort_asc(&mut key, &mut order);
        self.work += k as f64 * 7.0;
        let mut found_any = 0;
        for &r in &order {
            let r = r as usize;
            if len[r] == 0 {
                continue;
            }
            if limit < self.work - w0 || (found_any == 0 && limit < (self.work - w0) * 10.0) {
                break;
            }
            let lr = len[r];
            let mut tmp = lit[beg[r]..beg[r] + lr].to_vec();
            // literals with the smallest and second smallest counts
            let (mut min, mut amin, mut sec, mut asec) = (k + 1, -1i64, k + 1, -1i64);
            for &l in &tmp {
                if olen[l] < min {
                    (sec, asec, min, amin) = (min, amin, olen[l], l as i64);
                } else if olen[l] < sec {
                    (sec, asec) = (olen[l], l as i64);
                }
            }
            // t = -1: r as it is; then r with literal t flipped
            for t in -1..lr as i64 {
                let (flip, scan, a1, a2) = if t < 0 {
                    (-1i64, amin as usize, s1[r], s2[r])
                } else {
                    let e = tmp[t as usize];
                    let mut at = if e as i64 == amin { asec } else { amin };
                    let f = if e < n { e + n } else { e - n };
                    tmp[t as usize] = f;
                    if at == -1 || olen[f] < olen[at as usize] {
                        at = f as i64;
                    }
                    let (a1, a2) = sigs(&tmp);
                    self.work += lr as f64;
                    (f as i64, at as usize, a1, a2)
                };
                if olen[scan] < 5000 {
                    let ob = obeg[scan];
                    for c in 0..olen[scan] {
                        let q = oind[ob + c];
                        let lq = len[q];
                        if q == r || lq < lr || !s1[q] & a1 != 0 || !s2[q] & a2 != 0 {
                            continue;
                        }
                        // sorted merge: every literal of tmp but the flipped one in q; the flipped one's complement
                        // (t >= 0) must be in q too
                        let qs = &lit[beg[q]..beg[q] + lq];
                        let (mut pos, mut seen, mut sub) = (0usize, flip < 0, true);
                        for &l in &tmp {
                            if l as i64 == flip {
                                continue;
                            }
                            while pos < lq {
                                if qs[pos] as i64 == flip {
                                    seen = true;
                                } else if l <= qs[pos] {
                                    break;
                                }
                                pos += 1;
                            }
                            if pos == lq || qs[pos] != l {
                                sub = false;
                                break;
                            }
                        }
                        if sub {
                            while !seen && pos < lq {
                                seen = qs[pos] as i64 == flip;
                                pos += 1;
                            }
                        }
                        self.work += pos as f64 + pos as f64;
                        if !(sub && seen) {
                            continue;
                        }
                        found_any += 1;
                        if flip < 0 {
                            len[q] = 0;
                            continue;
                        }
                        let b = beg[q];
                        let mut nl = 0;
                        for x in b..b + lq {
                            if lit[x] as i64 != flip {
                                lit[b + nl] = lit[x];
                                nl += 1;
                            }
                        }
                        len[q] = nl;
                        if nl == 0 {
                            // x and its complement both excluded: infeasible
                            let f = flip as usize;
                            self.bad_col = if f < n { f as i32 } else { (f - n) as i32 };
                            self.bad_row = cand.iter().position(|&c| c == q as i32).map_or(-1, |i| i as i32);
                            return 3;
                        }
                        self.work += nl as f64 + nl as f64;
                        (s1[q], s2[q]) = sigs(&lit[b..b + nl]);
                        self.work += nl as f64;
                    }
                    self.work += olen[scan] as f64 * 10.0;
                }
                if t >= 0 {
                    let f = tmp[t as usize];
                    tmp[t as usize] = if f < n { f + n } else { f - n };
                }
            }
        }
        // apply: emptied rows go, shortened rows lose the flipped entries (rhs −1 for a +1 entry)
        let mut keep = vec![false; n];
        for i in 0..m {
            if cand[i] < 0 {
                continue;
            }
            let c = cand[i] as usize;
            if len[c] == 0 {
                self.mark_cols_of_row(i);
                self.drop_row_silent(i);
            } else if len[c] as i32 != self.row_cnt[i] {
                let mut e = self.row_head[i];
                while e != NIL {
                    let y = self.nd[e as usize];
                    if y.col >= 0 {
                        keep[y.col as usize] = false;
                    }
                    e = y.nr;
                }
                for &l in &lit[beg[c]..beg[c] + len[c]] {
                    keep[if l < n { l } else { l - n }] = true;
                }
                let mut e = self.row_head[i];
                while e != NIL {
                    let y = self.nd[e as usize];
                    if y.col >= 0 && !keep[y.col as usize] {
                        self.tracker.mark_col(y.col);
                        if 0.0 < y.val {
                            self.rhs[i] -= 1.0;
                        }
                        let x = &mut self.nd[e as usize];
                        x.col = -1;
                        x.val = 0.0;
                        self.row_cnt[i] -= 1;
                        self.col_cnt[y.col as usize] -= 1;
                    }
                    e = y.nr;
                }
                self.coefs_changed += 1;
                self.tracker.mark_row(i as i32);
                self.activity(Some(&[i as i32]));
            }
        }
        0
    }
}
