//! 3cd406's inline dual binarization candidate scan (PREDUALCONVTOBIN, 0x3b697d–0x3b723e, docs/m2/m2_spine_b.md §5.4).
use crate::state::{State, NIL};
use crate::Params;

impl State {
    /// Columns (not binary, bounded within ±1e6) that dual arguments could move to a bound: every row they sit in
    /// is either slack in their cost direction or a set-packing-like row of binaries with the same |coefficient|.
    /// shortcut: the conversion `3c2f36` is not ported (0 calls on the 38 models: no candidate survives), and the
    /// SOS big-M values of `57b1ce` (PreSOS1BigM/PreSOS2BigM, env+0x2640/+0x2648) are taken at their default 1000
    /// since SOS columns (flags 1/2) are not ported.
    pub fn dual_conv_bin(&mut self, p: &Params) -> i32 {
        if self.nbin == self.n as i32 {
            return 0;
        }
        let mu = self.work_mult;
        let list = self.cols_list();
        let (m1, m2) = (1000.0, 1000.0);
        let mut rmark = vec![false; self.m];
        let mut marked: Vec<usize> = Vec::new();
        let mut cand: Vec<usize> = Vec::new();
        for &j in &list {
            let j = j as usize;
            if self.col_cnt[j] < 0 {
                continue;
            }
            let f = self.col_flag[j];
            if (self.col_cnt[j] == 0 && f & 3 == 0) || f & !3 != 0 || self.vtype[j] == b'B' {
                continue;
            }
            let (l, u) = (self.lb[j], self.ub[j]);
            if !(l >= -1e6) || !(u <= 1e6) {
                continue;
            }
            let d = u - l;
            if !(d > 1e-6) {
                continue;
            }
            let (mut dn, mut up) = (true, true);
            if f & 3 != 0 {
                dn = !(1e-10 < l.abs()) && !(d <= 1e-5);
                up = !(1e-10 < u.abs()) && !(d <= 1e-5);
                if (f & 1 != 0 && d > m1) || (f & 2 != 0 && d > m2) {
                    dn = false;
                    up = false;
                }
            }
            let cj = self.obj[j];
            if cj > 0.0 { dn = false } else { up = up && cj >= 0.0 }
            let mut s = dn || up;
            let mut v = 0;
            let mut q = self.col_head[j];
            while q != NIL && s {
                let nd = self.nd[q as usize];
                if nd.col >= 0 {
                    let (a, r) = (nd.val, nd.row as usize);
                    let da = (d * a).abs();
                    if da < 1e-10 || (d > 1.0 && da > 1e6) {
                        s = false;
                        break;
                    }
                    let is_eq = self.sense[r] == b'=';
                    if is_eq || (a > 0.0 && dn) || (a < 0.0 && up) {
                        let mut eq = is_eq;
                        let mut bnd = true;
                        let marked_row = rmark[r];
                        if marked_row {
                            eq = false;
                        } else {
                            let k1 = (self.row_cnt[r] - 1) as f64;
                            if !(self.inf_max[r] <= 0 && (self.act_max[r] - da).abs() <= 1e-10 && self.inf_min[r] <= 0
                                && (k1 * da + self.act_min[r]).abs() <= 1e-10)
                            {
                                bnd = false;
                            }
                            if eq && !(self.inf_min[r] <= 0 && (self.act_min[r] + da).abs() <= 1e-10
                                && self.inf_max[r] <= 0 && (self.act_max[r] - k1 * da).abs() <= 1e-10)
                            {
                                eq = false;
                            }
                            if bnd || eq {
                                let mut cnt = 0;
                                let mut o = self.row_head[r];
                                while o != NIL {
                                    let x = self.nd[o as usize];
                                    if x.col >= 0 && x.col as usize != j
                                        && ((x.val.abs() - da).abs() > 1e-10 || self.vtype[x.col as usize] != b'B')
                                    {
                                        bnd = false;
                                        eq = false;
                                        break;
                                    }
                                    cnt += 1;
                                    o = x.nr;
                                }
                                self.work = (mu * 4.0) * cnt as f64 + self.work;
                                rmark[r] = true;
                                marked.push(r);
                            }
                        }
                        if marked_row || !bnd {
                            dn = dn && a <= 0.0;
                            up = up && a > 0.0;
                        }
                        if !eq && is_eq {
                            s = if a < 0.0 { up } else { dn };
                            dn = dn && a >= 0.0;
                            up = up && a < 0.0;
                        } else {
                            s = dn || up;
                        }
                    }
                }
                v += 1;
                q = nd.nc;
            }
            self.work = (mu * 6.0) * v as f64 + self.work;
            if s {
                cand.push(j);
            }
        }
        self.work = (8.0 * mu) * list.len() as f64 + self.work;
        self.work = mu * (marked.len() as f64 + marked.len() as f64) + self.work;
        let conv = match p.predualconvtobin {
            1 => true,
            -1 => (cand.len() as f64) <= self.nint as f64 * 5.0,
            _ => false,
        };
        for &j in &cand {
            if conv || ((self.ub[j] - 1.0) - self.lb[j]).abs() > 1e-10 || self.col_flag[j] != 0 {
                // `3c2f36(env, st, j, …)` would turn j into binaries here (see shortcut)
            }
        }
        0
    }
}
