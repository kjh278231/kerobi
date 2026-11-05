//! 03 LP presolve (`390989`), ported stage by stage against the presolve oracle (tools/presolve_oracle.py,
//! docs/m1_presolve.md). Entry from `Model.presolve()` is `999702`, which names the result `<name>_pre`.
use rsolver_model::Model;

/// The Gurobi parameters LP presolve reads. Values as gurobipy takes them; names match case-insensitively.
#[derive(Clone, Debug)]
pub struct Params {
    /// env+0x2424 Presolve (-1 auto, 0 off, 1 conservative, 2 aggressive)
    pub presolve: i32,
}

impl Default for Params {
    fn default() -> Self {
        Params { presolve: -1 }
    }
}

impl Params {
    pub fn from_pairs(pairs: &[(String, f64)]) -> Params {
        let mut p = Params::default();
        for (k, v) in pairs {
            if k.eq_ignore_ascii_case("Presolve") {
                p.presolve = *v as i32;
            }
        }
        p
    }
}

pub fn run(m: &Model, p: &Params) -> Model {
    let mut q = m.clone();
    q.name = format!("{}_pre", m.name);
    // State setup (`3a484d`) keeps `>` rows as `<` rows times -1 (12 §4.1); the presolved model inherits that.
    for (k, &i) in m.col_ind.iter().enumerate() {
        if m.row_sense[i as usize] == b'>' {
            q.col_val[k] = -q.col_val[k];
        }
    }
    for (i, s) in q.row_sense.iter_mut().enumerate() {
        if *s == b'>' {
            *s = b'<';
            q.rhs[i] = -q.rhs[i];
        }
    }
    if p.presolve == 0 {
        return q;
    }
    q
}
