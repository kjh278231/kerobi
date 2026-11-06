//! Small round-end reductions of `3cd406` after the mod-k passes (docs/m2_red_roundend_b.md): `5aaaf7` dual
//! fixing by row dual bounds, `579e67` residues of integer `=` rows.
use crate::iint::gcd_f64;
use crate::knapsack::gcd_excl;
use crate::state::{State, NIL};
use crate::Params;

impl State {
    /// `5aaaf7(st, env)`: a column whose live entries share one sign, in `<` rows only, with c·a < 0 bounds the
    /// dual price of each non-redundant row it is in (|c| · the steps of x that would absorb the row's max
    /// activity); an integer column with c·a > 0 entries in two or more rows whose bounds sum to at most |c| goes to
    /// its cheap bound (`3de18e`). No work of its own.
    pub fn r5aaaf7(&mut self, p: &Params) -> i32 {
        let tol = p.feastol;
        let mut price = vec![1e100; self.m];
        for j in 0..self.n {
            let c = self.obj[j];
            if self.col_cnt[j] <= 0 || c.abs() < 1e-6 || self.col_flag[j] != 0 {
                continue;
            }
            let mut first = None;
            let mut q = self.col_head[j];
            let mut skip = false;
            while q != NIL {
                let nd = self.nd[q as usize];
                if nd.col >= 0 {
                    if self.sense[nd.row as usize] == b'=' || first.is_some_and(|f: f64| f * nd.val < 0.0) {
                        skip = true;
                        break;
                    }
                    first.get_or_insert(nd.val);
                }
                q = nd.nc;
            }
            let Some(a) = first else { continue };
            if skip || !(if 0.0 < a { c <= 1e-10 } else { 0.0 <= a || -1e-10 <= c }) {
                continue;
            }
            let mut q = self.col_head[j];
            while q != NIL {
                let nd = self.nd[q as usize];
                q = nd.nc;
                let r = nd.row as usize;
                if nd.col < 0 || 0 < self.inf_max[r] || self.act_max[r] < tol {
                    continue;
                }
                let mut t = self.act_max[r] / nd.val.abs();
                if self.vtype[j] != b'C' {
                    t = (t - 1e-10).ceil();
                }
                if t <= (self.ub[j] + tol) - self.lb[j] {
                    let t = self.obj[j].abs() * t;
                    if t <= price[r] {
                        price[r] = t;
                    }
                }
            }
        }
        for j in 0..self.n {
            let c = self.obj[j];
            if self.col_cnt[j] <= 1 || self.vtype[j] == b'C' || c.abs() < 1e-6 || self.col_flag[j] != 0 {
                continue;
            }
            let mut s = 0.0;
            let mut q = self.col_head[j];
            while q != NIL {
                let nd = self.nd[q as usize];
                q = nd.nc;
                if nd.col < 0 {
                    continue;
                }
                if self.sense[nd.row as usize] == b'=' {
                    s = 1e100;
                    break;
                }
                if c * nd.val < 0.0 {
                    let b = price[nd.row as usize];
                    s += b;
                    if 1e100 <= b {
                        break;
                    }
                }
            }
            if s < 1e100 && s <= c.abs() + 1e-10 {
                let rc = self.fix_col(j, if c <= 0.0 { self.ub[j] } else { self.lb[j] }, true);
                if rc != 0 {
                    return rc;
                }
            }
        }
        0
    }

    /// `579e67(st, env)`: an `=` row of 3+ integer columns whose |a| are not all equal, with a gcd g in [1e-3, 1e4]
    /// and |rhs/g| <= 1e6, is scaled by 1/g; each column whose coefficient's complement gcd (`a7c752`) d is not 0, 1
    /// or >= 1e8 then has a·x ≡ rhs (mod d): x = r + d·y (inline copy of `574764`). W: 8 per entry (unscaled).
    pub fn r579e67(&mut self, p: &Params) -> i32 {
        for i in 0..self.m {
            if self.row_cnt[i] <= 2 || self.sense[i] != b'=' {
                continue;
            }
            let rhs = self.rhs[i];
            let (mut idx, mut val) = (vec![], vec![]);
            let mut same = -1.0f64;
            let mut q = self.row_head[i];
            let mut cont = false;
            while q != NIL {
                let nd = self.nd[q as usize];
                q = nd.nr;
                if nd.col < 0 {
                    continue;
                }
                let t = self.vtype[nd.col as usize];
                if t != b'B' && t != b'I' {
                    cont = true;
                    break;
                }
                if 0.0 <= same {
                    if same != 1e100 && 1e-10 < (same - nd.val.abs()).abs() {
                        same = 1e100;
                    }
                } else {
                    same = nd.val.abs();
                }
                idx.push(nd.col as usize);
                val.push(nd.val);
            }
            self.work += idx.len() as f64 * 8.0;
            if cont || same != 1e100 {
                continue;
            }
            let g = gcd_f64(&val);
            if !((0.001..=10000.0).contains(&g) && (rhs / g).abs() <= 1e6) {
                continue;
            }
            let gi = 1.0 / g;
            for v in val.iter_mut() {
                *v = (*v * gi + 0.5).floor();
            }
            let out = gcd_excl(&val, &mut self.work);
            let b = (rhs * gi + 0.5).floor();
            for (t, &d) in out.iter().enumerate() {
                let j = idx[t];
                if d == 1.0 || d == 0.0 || 1e8 <= d || self.col_flag[j] != 0 {
                    continue;
                }
                let mut done = 0;
                let rc = self.mod_subst(p, j, val[t] as i64 as i32, d as i32, b as i32, &mut done);
                if rc != 0 {
                    return rc;
                }
            }
        }
        0
    }
}
