//! 3cd406's inline blocks of the final stage: dual fixing of empty columns and implied-integer scaling
//! (docs/m2_spine_d.md §5.2, §5.3; docs/m2_red_final.md).
use crate::iint::gcd_f64;
use crate::state::{State, INF, NIL};
use crate::Params;

impl State {
    /// Row `i`'s live nodes as (column, value) in list order.
    fn row_live(&self, i: usize) -> Vec<(usize, f64)> {
        let mut out = Vec::new();
        let mut p = self.row_head[i];
        while p != NIL {
            let q = self.nd[p as usize];
            p = q.nr;
            if q.col >= 0 {
                out.push((q.col as usize, q.val));
            }
        }
        out
    }

    /// The gcd input of row `i` relative to its first live column `c2` with x = 0 (value a): |a_k·x_k / a| over the
    /// other live nodes, then |rhs / a| when |rhs| > 1e-10.
    fn iscale_ratios(&self, i: usize, x: &[f64]) -> (usize, Vec<f64>) {
        let nodes = self.row_live(i);
        let &(c2, a) = nodes.iter().find(|&&(c, _)| x[c] == 0.0).expect("row has an open continuous column");
        let mut v: Vec<f64> = nodes.iter().filter(|&&(c, _)| c != c2).map(|&(c, val)| ((val * x[c]) / a).abs()).collect();
        if self.rhs[i].abs() > 1e-10 {
            v.push((self.rhs[i] / a).abs());
        }
        (c2, v)
    }

    /// Final dual fixing (0x3b9f0e): empty unflagged columns go to the bound their cost prefers (type 1 record), then
    /// nearly fixed live columns are fixed (`3df916`). 3 = crossed bounds, 5 = unbounded (st+0x470 = the column).
    pub fn dual_fix_empty_cols(&mut self, p: &Params) -> i32 {
        let (ft, ift, ot) = (p.feastol, p.intfeastol, p.opttol);
        for j in 0..self.n {
            if self.col_cnt[j] != 0 || self.col_flag[j] != 0 {
                continue;
            }
            let (lj, uj) = (self.lb[j], self.ub[j]);
            let d = lj - uj;
            if (self.vtype[j] != b'C' && d > ift) || d > ft {
                self.bad_col = j as i32;
                return 3;
            }
            let cj = self.obj[j];
            let v = if cj > ot {
                if -INF >= lj {
                    self.bad_col = j as i32;
                    return 5;
                }
                lj
            } else if cj < -ot {
                if uj >= INF {
                    self.bad_col = j as i32;
                    return 5;
                }
                uj
            } else if cj >= 0.0 {
                if lj > -INF { lj } else if INF > uj { uj } else { 0.0 }
            } else if INF > uj {
                uj
            } else if lj > -INF {
                lj
            } else {
                0.0
            };
            self.push_fix(j, v);
            self.objcon = v * self.obj[j] + self.objcon;
            self.col_cnt[j] = -1;
            self.obj[j] = 0.0;
        }
        for j in 0..self.n {
            if self.col_cnt[j] > 0 && self.lb[j] + 1e-10 > self.ub[j] {
                let r = self.fix_near(j, ft);
                if r != 0 {
                    return r;
                }
            }
        }
        0
    }

    /// Implied-integer scaling (0x3ba255): find for continuous columns a step s with x_j = s·x'_j and x'_j integral
    /// (from `=` rows, then from dual arguments), record x_j = 0 + s·x'_j (type 9) and scale the column, then divide
    /// rows whose terms share a gcd. Column types stay; no tracker marks, counters or work.
    pub fn implied_int_scale(&mut self, p: &Params) -> i32 {
        let (n, m) = (self.n, self.m);
        let open_flag = |f: u32| f & 0xffff_ffbc != 0;
        let mut x: Vec<f64> =
            (0..n).map(|j| if self.col_cnt[j] > 0 && self.vtype[j] != b'C' { 1.0 } else { 0.0 }).collect();
        let mut kc = vec![0i32; m];
        let mut d = vec![0.0f64; m];
        let mut q1 = Vec::new();
        for i in 0..m {
            if self.row_cnt[i] > 0 {
                kc[i] = self.row_live(i).iter().filter(|&&(c, _)| self.vtype[c] == b'C').count() as i32;
                if self.sense[i] == b'=' && kc[i] == 1 {
                    q1.push(i);
                }
            }
        }
        // A: `=` rows with one open continuous column (FIFO; the queue grows while it is read)
        let mut h = 0;
        while h < q1.len() {
            let i = q1[h];
            h += 1;
            if kc[i] != 1 {
                continue;
            }
            let (j, v) = self.iscale_ratios(i, &x);
            if open_flag(self.col_flag[j]) {
                continue;
            }
            let mut g = gcd_f64(&v);
            if !(g > 0.001) {
                continue;
            }
            while g > 1000.0 {
                g /= 10.0;
            }
            x[j] = g;
            for r in self.col_rows(j) {
                kc[r] -= 1;
                if self.sense[r] == b'=' && kc[r] == 1 {
                    q1.push(r);
                }
            }
        }
        // B: dual arguments
        if p.dual_reductions != 0 {
            let row_scale = |s: &Self, i: usize, x: &[f64], d: &mut [f64], kc: &mut [i32]| -> usize {
                let (c2, v) = s.iscale_ratios(i, x);
                d[i] = gcd_f64(&v);
                if d[i] == 0.0 {
                    kc[i] = -1;
                }
                c2
            };
            for i in 0..m {
                if self.sense[i] != b'=' && kc[i] == 1 {
                    row_scale(self, i, &x, &mut d, &mut kc);
                }
            }
            let mut q2: Vec<usize> = (0..n).filter(|&j| x[j] == 0.0 && self.col_cnt[j] > 0).collect();
            let mut h = 0;
            while h < q2.len() {
                let j = q2[h];
                h += 1;
                if x[j] > 0.0 || self.col_cnt[j] <= 0 || open_flag(self.col_flag[j]) {
                    continue;
                }
                let (cj, lj, uj) = (self.obj[j], self.lb[j], self.ub[j]);
                let mut s_lo = if cj > -1e-10 { if lj > -INF && lj.abs() > 1e-10 { lj.abs() } else { 1e100 } } else { 0.0 };
                let mut s_up = if cj < 1e-10 { if INF > uj && uj.abs() > 1e-10 { uj.abs() } else { 1e100 } } else { 0.0 };
                let mut eq = false;
                for (r, a) in self.col_live(j) {
                    if self.sense[r] == b'=' {
                        eq = true;
                        break;
                    }
                    if a < -1e-10 {
                        s_lo = if s_lo > 1e99 { d[r] } else { gcd_f64(&[d[r], s_lo]) };
                    } else if a > 1e-10 {
                        s_up = if s_up > 1e99 { d[r] } else { gcd_f64(&[d[r], s_up]) };
                    }
                }
                if eq {
                    s_lo = 0.0;
                    s_up = 0.0;
                }
                if !(s_up > 0.001 || s_lo > 0.001) {
                    continue;
                }
                let mut g = if s_up > s_lo { s_up } else { s_lo };
                if g > 1e99 {
                    g = 1.0;
                }
                while g > 1000.0 {
                    g /= 10.0;
                }
                x[j] = g;
                for r in self.col_rows(j) {
                    kc[r] -= 1;
                    if kc[r] == 1 {
                        q2.push(row_scale(self, r, &x, &mut d, &mut kc));
                    }
                }
            }
        }
        // C: row gcds over rows without open continuous columns
        for i in 0..m {
            d[i] = 0.0;
            if kc[i] != 0 || self.row_cnt[i] <= 0 {
                continue;
            }
            let mut v: Vec<f64> = self.row_live(i).iter().map(|&(c, a)| (a * x[c]).abs()).collect();
            if self.rhs[i].abs() > 1e-10 {
                v.push(self.rhs[i].abs());
            }
            let mut g = gcd_f64(&v);
            if g > 0.001 {
                while g > 1000.0 {
                    g /= 10.0;
                }
                d[i] = g;
            }
        }
        // D: column scaling (dead nodes too)
        for j in 0..n {
            let s = x[j];
            if !(s > 0.0) || !((s - 1.0).abs() > 1e-10) {
                continue;
            }
            self.push_t9(j as i32, 0.0, s);
            let mut q = self.col_head[j];
            while q != NIL {
                let nd = &mut self.nd[q as usize];
                nd.val *= s;
                q = nd.nc;
            }
            self.obj[j] *= s;
            if self.lb[j] > -INF {
                self.lb[j] /= s;
            }
            if INF > self.ub[j] {
                self.ub[j] /= s;
            }
        }
        // E: row division (no record; dead nodes too)
        for i in 0..m {
            let g = d[i];
            if !(g > 0.0) || !((g - 1.0).abs() > 1e-10) {
                continue;
            }
            let mut q = self.row_head[i];
            while q != NIL {
                let nd = &mut self.nd[q as usize];
                nd.val /= g;
                q = nd.nr;
            }
            self.rhs[i] /= g;
            self.act_min[i] /= g;
            self.act_max[i] /= g;
        }
        0
    }

    /// Column `j`'s live nodes as (row, value) in list order.
    fn col_live(&self, j: usize) -> Vec<(usize, f64)> {
        let mut out = Vec::new();
        let mut p = self.col_head[j];
        while p != NIL {
            let q = self.nd[p as usize];
            p = q.nc;
            if q.col >= 0 {
                out.push((q.row as usize, q.val));
            }
        }
        out
    }

    fn col_rows(&self, j: usize) -> Vec<usize> {
        self.col_live(j).into_iter().map(|(r, _)| r).collect()
    }
}
