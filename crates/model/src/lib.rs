//! 01 §1 Model data. Arrays mirror the original model-data layout (model+0xe8) so ports can read them directly.

pub mod fingerprint;
pub mod mps;
pub mod mps_write;
mod param_table;
pub mod params;

/// Value the original stores internally for an infinite bound (INFBOUND); the attribute API reports it as inf.
pub const INFINITY: f64 = 1e30;

#[derive(Clone, Debug, Default)]
pub struct Model {
    pub name: String,
    /// D+0x04: 1 minimize, -1 maximize.
    pub sense: i32,
    /// D+0x60
    pub objcon: f64,
    /// D+0x50
    pub obj: Vec<f64>,
    /// D+0x390 / D+0x398 / D+0x410
    pub lb: Vec<f64>,
    pub ub: Vec<f64>,
    pub vtype: Vec<u8>,
    /// Column-wise matrix: D+0xf8 start (i64), D+0x100 length, D+0x108 row index, D+0x110 value.
    pub col_beg: Vec<i64>,
    pub col_len: Vec<i32>,
    pub col_ind: Vec<i32>,
    pub col_val: Vec<f64>,
    /// D+0x388 / D+0x380
    pub row_sense: Vec<u8>,
    pub rhs: Vec<f64>,
    pub var_names: Vec<String>,
    pub constr_names: Vec<String>,
    /// model+0xb0: state of the model's RNG (`a7b4d6`); 0 on a fresh model, set by `895c5` for a presolved one.
    pub rng: u64,
}

impl Model {
    pub fn num_vars(&self) -> usize {
        self.obj.len()
    }
    pub fn num_constrs(&self) -> usize {
        self.rhs.len()
    }
    pub fn num_nz(&self) -> usize {
        self.col_len.iter().map(|&l| l as usize).sum()
    }
}
