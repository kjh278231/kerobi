//! Parameter table (docs/m1/m1_driver.md §2): `ac81d7` binds each name to its env slot; names, types, ranges and
//! defaults come from the static definition table, ported as `param_table::PARAMS`.
pub use crate::param_table::PARAMS;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Int,
    Dbl,
    Str,
}

#[derive(Debug)]
pub struct Def {
    pub name: &'static str,
    pub kind: Kind,
    pub min: f64,
    pub max: f64,
    /// numeric default; string parameters keep 0 here (see `default_str`)
    pub default: f64,
}

fn index(name: &str) -> Option<usize> {
    PARAMS.iter().position(|d| d.name.eq_ignore_ascii_case(name))
}

/// Definition of a parameter by name, ignoring case as the original lookup does.
pub fn find(name: &str) -> Option<&'static Def> {
    index(name).map(|i| &PARAMS[i])
}

/// Numeric parameter values of one env, in `PARAMS` order (string parameters stay in the Python layer).
#[derive(Clone, Debug)]
pub struct Params(Vec<f64>);

impl Default for Params {
    fn default() -> Self {
        Params(PARAMS.iter().map(|d| d.default).collect())
    }
}

impl Params {
    /// Defaults overridden by (name, value) pairs; names the table lacks are ignored (the caller validates).
    pub fn from_pairs(pairs: &[(String, f64)]) -> Params {
        let mut p = Params::default();
        for (k, v) in pairs {
            if let Some(i) = index(k) {
                p.0[i] = *v;
            }
        }
        p
    }
    pub fn get(&self, name: &str) -> f64 {
        self.0[index(name).unwrap_or_else(|| panic!("unknown parameter {name}"))]
    }
    pub fn int(&self, name: &str) -> i32 {
        self.get(name) as i32
    }
    pub fn set(&mut self, name: &str, v: f64) {
        let i = index(name).unwrap_or_else(|| panic!("unknown parameter {name}"));
        self.0[i] = v;
    }
    /// Values that differ from the default, as (name, value) pairs for the engines' own parameter readers.
    pub fn pairs(&self) -> Vec<(String, f64)> {
        PARAMS.iter().zip(&self.0).filter(|(d, &v)| v != d.default).map(|(d, &v)| (d.name.to_string(), v)).collect()
    }
}

/// Default of a string parameter. All are empty in the table; NodefileDir gets "." outside it (gurobipy reports ".").
pub fn default_str(d: &Def) -> &'static str {
    if d.name == "NodefileDir" {
        "."
    } else {
        ""
    }
}
