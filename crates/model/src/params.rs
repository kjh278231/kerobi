//! Parameter table (docs/m1_driver.md §2): `ac81d7` binds each name to its env slot; names, types, ranges and
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

/// Definition of a parameter by name, ignoring case as the original lookup does.
pub fn find(name: &str) -> Option<&'static Def> {
    PARAMS.iter().find(|d| d.name.eq_ignore_ascii_case(name))
}

/// Default of a string parameter. All are empty in the table; NodefileDir gets "." outside it (gurobipy reports ".").
pub fn default_str(d: &Def) -> &'static str {
    if d.name == "NodefileDir" {
        "."
    } else {
        ""
    }
}
