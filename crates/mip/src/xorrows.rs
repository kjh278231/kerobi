//! `3e5363` XOR / parity rows (docs/m2_red_outer_a.md §2): a 3- or 4-entry `<` row with rhs 2 (±1 coefficients) or
//! −1 (all −1) over binaries (one continuous [0,1] column allowed) seeds a parity system; with a pivot column whose
//! other rows all fit the pattern, the system's rows are removed (type 25 record) and the pivot goes (or, when the
//! pivot also sits in other rows with one sign, the rows where its sign agrees go).
use crate::post::PostRec;
use crate::state::{State, NIL};
use crate::Params;

impl State {
    /// `3e5363`. Returns the last `3de18e` status (0).
    pub fn xor_rows(&mut self, p: &Params) -> i32 {
        let mu = self.work_mult;
        if self.nbin < 3 || p.dual_reductions == 0 {
            return 0;
        }
        let lv = self.tracker.level;
        let list: Vec<i32> = self.tracker.rows.list[lv][..self.tracker.rows.a[lv]].to_vec();
        let mut ret = 0;
        for &ii in &list {
            let i = ii as usize;
            let rc = self.row_cnt[i];
            if (rc != 3 && rc != 4) || self.sense[i] == b'=' {
                continue;
            }
            let rhs = self.rhs[i];
            let e8 = if (rhs - 2.0).abs() <= 1e-13 {
                0
            } else if (rhs + 1.0).abs() <= 1e-13 {
                1
            } else {
                continue;
            };
            // seed row: columns, the pivot (non-binary column, else the smallest count, ties to a later free one)
            let (mut nb, mut best, mut piv, mut npos) = (-1i32, 2000000000, -1i32, 0);
            let mut cols: Vec<i32> = Vec::new();
            let (mut vis, mut ok) = (0, true);
            let mut q = self.row_head[i];
            while q != NIL {
                let x = self.nd[q as usize];
                if x.col >= 0 {
                    let c = x.col as usize;
                    if self.vtype[c] != b'B' {
                        if 1e-13 < self.lb[c].abs() || 1e-13 < (self.ub[c] - 1.0).abs() || nb >= 0 {
                            ok = false;
                            break;
                        }
                        nb = x.col;
                    }
                    let a = x.val;
                    if (e8 == 0 && 1e-13 < (a.abs() - 1.0).abs()) || (e8 == 1 && 1e-13 < (a + 1.0).abs()) {
                        ok = false;
                        break;
                    }
                    cols.push(x.col);
                    npos += (0.0 < a) as i32;
                    if !(0.5 > self.ub[c] - self.lb[c]) {
                        if nb == x.col {
                            best = self.col_cnt[c];
                            piv = x.col;
                        } else {
                            let k = self.col_cnt[c];
                            if k < best {
                                if nb == -1 {
                                    piv = x.col;
                                    best = k;
                                }
                            } else if k == best && self.obj[c] == 0.0 && nb == -1 {
                                piv = x.col;
                            }
                        }
                    }
                }
                vis += 1;
                q = x.nr;
            }
            self.work += vis as f64 * 6.0 * mu;
            if !ok || piv < 0 || best < 2 {
                continue;
            }
            let pv = piv as usize;
            if !(self.obj[pv] == 0.0) || self.col_flag[pv] != 0 || (rc == 4 && e8 == 0 && npos != 3) {
                continue;
            }
            // pivot column: the system's other rows, and the sign of the pivot in rows outside it
            let mut rows = vec![i];
            let (mut sgn, mut vis) = (0i32, 0);
            let mut ok = true;
            let mut q = self.col_head[pv];
            while q != NIL {
                let pn = self.nd[q as usize];
                q = pn.nc;
                if pn.col >= 0 && pn.row as usize != i {
                    let r = pn.row as usize;
                    if self.sense[r] == b'=' {
                        ok = false;
                        break;
                    }
                    let d = self.rhs[r];
                    let cand = rc == self.row_cnt[r]
                        && match (e8, rc) {
                            (0, 3) => d.abs() <= 1e-13,
                            (0, _) => d.abs() <= 1e-13 || (d - 2.0).abs() <= 1e-13,
                            (_, 3) => (d - 1.0).abs() <= 1e-13,
                            _ => (d - 1.0).abs() <= 1e-13 || (d - 3.0).abs() <= 1e-13,
                        };
                    let s = if cand {
                        let ps = if pn.val < 0.0 { -1 } else { 1 };
                        let (mut bad, mut np, mut v2) = (false, 0, 0);
                        let mut q2 = self.row_head[r];
                        while q2 != NIL {
                            let x = self.nd[q2 as usize];
                            if x.col >= 0 {
                                let c = x.col;
                                if c != piv && !(c == cols[0] || c == cols[1] || c == cols[2] || (rc != 3 && c == cols[3])) {
                                    bad = true;
                                }
                                if 1e-13 < (x.val.abs() - 1.0).abs() {
                                    bad = true;
                                }
                                if x.val > 0.0 {
                                    np += 1;
                                }
                            }
                            v2 += 1;
                            q2 = x.nr;
                        }
                        self.work += mu * 3.0 * v2 as f64;
                        let fit = !bad
                            && match (e8, rc) {
                                (0, 3) => np == 1,
                                (0, _) => (d.abs() > 1e-13 || np == 1) && ((d - 2.0).abs() > 1e-13 || np == 3),
                                (_, 3) => np == 2,
                                _ => ((d - 1.0).abs() > 1e-13 || np == 2) && ((d - 3.0).abs() > 1e-13 || np == 4),
                            };
                        if fit {
                            rows.push(r);
                            vis += 1;
                            continue;
                        }
                        ps
                    } else {
                        let a = pn.val;
                        if sgn == 0 {
                            if a < 0.0 { -1 } else { 1 }
                        } else if (sgn == -1 && a > 0.0) || (sgn == 1 && 0.0 > a) {
                            ok = false;
                            break;
                        } else {
                            sgn
                        }
                    };
                    if sgn == 0 {
                        sgn = s;
                    } else if s != sgn {
                        ok = false;
                        break;
                    }
                }
                vis += 1;
            }
            self.work += vis as f64 * 7.0 * mu;
            if !ok {
                continue;
            }
            for k in 1..4 {
                if k == 3 && self.row_cnt[i] != 4 {
                    break;
                }
                if piv == cols[k] {
                    cols.swap(0, k);
                    break;
                }
            }
            if sgn != 0 {
                let mut keep = Vec::new();
                for &r in &rows {
                    let mut v = 0;
                    let mut q = self.row_head[r];
                    while q != NIL {
                        let x = self.nd[q as usize];
                        if x.col >= 0 && x.col == piv {
                            if 0.0 < x.val * sgn as f64 {
                                keep.push(r);
                            }
                            break;
                        }
                        v += 1;
                        q = x.nr;
                    }
                    self.work += v as f64 * mu;
                }
                self.work += rows.len() as f64 * 3.0 * mu;
                if !keep.is_empty() {
                    self.push_xor(e8, &cols, &keep, (piv + 1) * sgn);
                    self.drop_xor_rows(&keep);
                } else {
                    self.work += 0.0 * 3.0 * mu;
                }
                for &c in &cols {
                    if self.col_cnt[c as usize] >= 0 {
                        self.tracker.mark_col(c);
                    }
                }
                self.work += cols.len() as f64 * mu;
                continue;
            }
            self.push_xor(e8, &cols, &rows, 0);
            self.drop_xor_rows(&rows);
            // row_cnt[i] is −1 by now, so a fourth column is never fixed here (as in the original)
            let last = if self.row_cnt[i] == 4 { 4 } else { 3 };
            for &c in &cols[1..last] {
                let c = c as usize;
                if self.col_cnt[c] == 0 && self.col_flag[c] & 0xfffff7bf == 0 {
                    let v = if self.obj[c] >= 0.0 { 0.0 } else { 1.0 };
                    ret = self.fix_col(c, v, true);
                    if ret != 0 {
                        return ret;
                    }
                }
            }
            self.col_cnt[pv] = -3;
            self.cols_removed += 1;
            let mut t = 1;
            while t < cols.len() {
                if self.col_cnt[cols[t] as usize] >= 0 {
                    self.tracker.mark_col(cols[t]);
                }
                t += 1;
            }
            self.work += t as f64 * mu;
        }
        self.work += list.len() as f64 * 3.0 * mu;
        ret
    }

    /// `39ba81(env, st, e8, cols, rows, code)`: type 25 record of the rows' live entries and rhs; rows removed
    /// += len(rows).
    fn push_xor(&mut self, e8: i32, cols: &[i32], rows: &[usize], code: i32) {
        let mu = self.work_mult;
        let nr = rows.len() as f64;
        self.work += (nr + nr) * mu;
        self.rows_removed += rows.len() as i32;
        self.work += ((cols.len() as f64 + nr) * 0.5) * mu;
        let (mut beg, mut ind, mut val, mut rhs) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
        for &r in rows {
            beg.push(ind.len() as i32);
            let mut q = self.row_head[r];
            while q != NIL {
                let x = self.nd[q as usize];
                if x.col >= 0 {
                    ind.push(x.col);
                    val.push(x.val);
                }
                q = x.nr;
            }
            rhs.push(self.rhs[r]);
        }
        beg.push(ind.len() as i32);
        self.work += mu * (nr * 5.0 + ind.len() as f64 * 3.0);
        self.post.push(PostRec::Xor { e8, code, cols: cols.to_vec(), rows: rows.iter().map(|&r| r as i32).collect(), beg, ind, val, rhs });
    }

    /// Rows of the parity system leave: live nodes die (column counts −1), row count −1; no marks.
    fn drop_xor_rows(&mut self, rows: &[usize]) {
        let mu = self.work_mult;
        for &r in rows {
            let mut v = 0;
            let mut q = self.row_head[r];
            while q != NIL {
                let x = self.nd[q as usize];
                if x.col >= 0 {
                    self.col_cnt[x.col as usize] -= 1;
                    self.nd[q as usize].col = -1;
                }
                v += 1;
                q = x.nr;
            }
            self.row_cnt[r] = -1;
            self.work += (v as f64 + v as f64) * mu;
        }
        self.work += rows.len() as f64 * 3.0 * mu;
    }
}
