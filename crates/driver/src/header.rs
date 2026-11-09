//! Optimize header log (`c23877` step C, docs/m1/m1_driver.md §3): "Optimize a model with …", fingerprint, `a8aa6a`
//! ("Model has …"), `a8a3b4` ("Variable types …", MIP only) and `a9a78e` (coefficient statistics from `a71757`).
use rsolver_model::mps_write::{c_exp, fmt_num};
use rsolver_model::Model;

/// `a71757` (the model+0x2d8 statistics array, linear part): (max, min) of the nonzero |values|, (0, 0) if none.
pub struct Stats {
    pub matrix: (f64, f64),
    pub obj: (f64, f64),
    /// finite bounds only (|v| < 1e30)
    pub bounds: (f64, f64),
    pub rhs: (f64, f64),
}

fn range(vals: impl Iterator<Item = f64>) -> (f64, f64) {
    let (mut hi, mut lo) = (0f64, 1e100f64);
    for a in vals.map(f64::abs).filter(|&a| a != 0.0) {
        hi = hi.max(a);
        lo = lo.min(a);
    }
    (hi, lo.min(hi))
}

pub fn stats(m: &Model) -> Stats {
    let bnd = m.lb.iter().chain(&m.ub).copied().filter(|v| v.abs() < 1e30);
    Stats {
        matrix: range(m.col_val.iter().copied()),
        obj: range(m.obj.iter().copied()),
        bounds: range(bnd),
        rhs: range(m.rhs.iter().copied()),
    }
}

/// Header lines of a top-level first call, up to the blank line that follows them. `numeric_focus` = 0 adds the
/// "Consider reformulating" advice after a warning.
pub fn header(m: &Model, numeric_focus: i32) -> String {
    let mut s = format!(
        "Optimize a model with {} rows, {} columns and {} nonzeros ({})\n",
        m.num_constrs(),
        m.num_vars(),
        m.num_nz(),
        if m.sense == -1 { "Max" } else { "Min" }
    );
    s += &format!("Model fingerprint: 0x{:08x}\n", rsolver_model::fingerprint::fingerprint(m));
    let k = m.obj.iter().filter(|c| c.abs() > 1e-13).count();
    s += &format!("Model has {k} linear objective coefficients");
    if m.objcon.abs() > 1e-13 {
        s += &format!(" and an objective constant of {}", fmt_num(m.objcon));
    }
    s += "\n";
    if m.vtype.iter().any(|&t| t != b'C') {
        let n = |t: u8| m.vtype.iter().filter(|&&v| v == t).count();
        let (b, i) = (n(b'B'), n(b'I'));
        s += &format!("Variable types: {} continuous, {} integer ({b} binary)\n", n(b'C'), i + b);
        let (sc, si) = (n(b'S'), n(b'N'));
        if sc + si > 0 {
            s += &format!("Semi-Variable types: {sc} continuous, {si} integer\n");
        }
    }
    let st = stats(m);
    let r = |name: &str, (hi, lo): (f64, f64)| format!("  {name:<16} [{}, {}]\n", c_exp(lo, 0), c_exp(hi, 0));
    s += "Coefficient statistics:\n";
    s += &r("Matrix range", st.matrix);
    s += &r("Objective range", st.obj);
    s += &r("Bounds range", st.bounds);
    s += &r("RHS range", st.rhs);
    let mut warn = false;
    let mut w = |cond: bool, text: &str| {
        if cond {
            warn = true;
            s += &format!("Warning: Model contains {text}\n");
        }
    };
    if (st.matrix.0 + 1e-10).log10() - (st.matrix.1 + 1e-10).log10() >= 12.0 {
        w(true, "large matrix coefficient range");
    } else {
        w(st.matrix.0 > 1e8, "large matrix coefficients");
    }
    w(st.obj.0 > 1e9, "large objective coefficients");
    w(st.rhs.0 > 1e9, "large rhs");
    w(st.bounds.0 > 1e9, "large bounds");
    if warn && numeric_focus == 0 {
        s += "         Consider reformulating model or setting NumericFocus parameter\n         to avoid numerical issues.\n";
    }
    s
}
