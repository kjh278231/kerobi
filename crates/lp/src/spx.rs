//! 04 §3 Simplex workspace arrays (original spx at model+0xe0): scaled column-wise matrix with slack columns implied,
//! costs, bounds and right-hand side. Conventions verified bit-for-bit on 64 LPs (docs/m1/m1_notes.md §4).
use crate::scaling::{scale_flag3, Scaling};
use rsolver_model::{Model, INFINITY};

#[derive(Clone, Debug)]
pub struct Spx {
    pub m: usize,
    pub n: usize,
    /// Scaled structural columns (spx+0x70/0x78/0x80/0x88). Slack j = n+i is the unit column e_i.
    pub beg: Vec<i64>,
    pub len: Vec<i32>,
    pub ind: Vec<i32>,
    pub val: Vec<f64>,
    /// Original row sense characters (spx+0x90); '>' rows are stored negated.
    pub sense: Vec<u8>,
    /// Structural column types (spx+0xb8), retained when solving a MIP's LP relaxation.
    pub vtype: Vec<u8>,
    /// n+m costs, n+m bounds (slacks: [0, INF] or [0, 0] on '=' rows), m right-hand sides.
    pub c: Vec<f64>,
    pub l: Vec<f64>,
    pub u: Vec<f64>,
    pub b: Vec<f64>,
    pub scaling: Scaling,
}

impl Spx {
    pub fn new(md: &Model) -> Spx {
        Spx::with_scaling(md, scale_flag3(md))
    }

    /// Workspace without any scaling (`2e9ae` re-solve: row and column scales 1, objective scale 1).
    pub fn unscaled(md: &Model) -> Spx {
        let sc = Scaling { col: vec![1.0; md.num_vars()], row: vec![1.0; md.num_constrs()], obj: 1.0 };
        Spx::with_scaling(md, sc)
    }

    pub fn with_scaling(md: &Model, sc: Scaling) -> Spx {
        let (m, n) = (md.num_constrs(), md.num_vars());
        let sigma: Vec<f64> = md.row_sense.iter().map(|&s| if s == b'>' { -1.0 } else { 1.0 }).collect();
        let mut val = Vec::with_capacity(md.col_val.len());
        for j in 0..n {
            let b = md.col_beg[j] as usize;
            for k in b..b + md.col_len[j] as usize {
                let i = md.col_ind[k] as usize;
                val.push(sigma[i] * md.col_val[k] * sc.row[i] * sc.col[j]);
            }
        }
        let unscale = |x: f64, s: f64| if x.abs() >= INFINITY { x } else { x / s };
        let mut c: Vec<f64> = (0..n).map(|j| md.obj[j] * sc.col[j] * sc.obj).collect();
        let mut l: Vec<f64> = (0..n).map(|j| unscale(md.lb[j], sc.col[j])).collect();
        let mut u: Vec<f64> = (0..n).map(|j| unscale(md.ub[j], sc.col[j])).collect();
        for &s in &md.row_sense {
            c.push(0.0);
            l.push(0.0);
            u.push(if s == b'=' { 0.0 } else { INFINITY });
        }
        let b = (0..m).map(|i| sigma[i] * md.rhs[i] * sc.row[i]).collect();
        Spx {
            m,
            n,
            beg: md.col_beg.clone(),
            len: md.col_len.clone(),
            ind: md.col_ind.clone(),
            val,
            sense: md.row_sense.clone(),
            vtype: md.vtype.clone(),
            c,
            l,
            u,
            b,
            scaling: sc,
        }
    }

    /// Integer-column preference shared by pricing, BFRT and cost perturbation.
    pub fn var_factor(&self, j: usize) -> f64 {
        if j < self.n && self.vtype.get(j).is_some_and(|&t| t != b'C') { 1.5 } else { 1.0 }
    }

    /// Entries (row, value) of column j over structural and slack columns.
    pub fn col(&self, j: usize) -> impl Iterator<Item = (usize, f64)> + '_ {
        let (b, e, slack) = if j < self.n {
            (self.beg[j] as usize, self.beg[j] as usize + self.len[j] as usize, None)
        } else {
            (0, 0, Some(j - self.n))
        };
        (b..e).map(move |k| (self.ind[k] as usize, self.val[k])).chain(slack.map(|i| (i, 1.0)))
    }
}
