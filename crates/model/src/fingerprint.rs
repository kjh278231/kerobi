//! 01 §1.1 Model fingerprint: rolling hash over the model-data arrays in their stored order.
//! Covers the parts an LP/MIP from a file has; Q, QC, SOS, general constraints, multi-objective, starts,
//! hints and attribute arrays enter the hash only when present, which this model type cannot hold yet.
use crate::Model;

const SCALE: f64 = 1.9554777523817772;

fn dh(h: u64, x: f64) -> u64 {
    let x = if x == 0.0 { 0.0 } else { x }; // -0.0 hashes like 0.0
    // The raw bit pattern is added (`add rax, [scaled double]` at 0x895a50), not a converted integer.
    h.wrapping_mul(31).wrapping_add(h >> 52).wrapping_add((x * SCALE).to_bits())
}

fn ih(h: u64, v: i64) -> u64 {
    h.wrapping_mul(31).wrapping_add(v as u64)
}

pub fn fingerprint(m: &Model) -> u32 {
    let (n, nrows) = (m.num_vars(), m.num_constrs());
    let num_obj = 0i64; // D+0x7c: multi-objective count, 0 for a single objective
    let mut h = num_obj as u64;
    h = dh(h, m.objcon);
    h = ih(h, m.sense as i64);
    h = dh(h, 0.0); // D+0x318
    for v in [0, 0, n as i64, nrows as i64] {
        // PWL objective count, Q nonzeros, columns, rows
        h = ih(h, v);
    }
    h = dh(h, m.num_nz() as f64); // D+0x10
    for v in [0, 0, 0, 0, 0, 0] {
        // QC, D+0x208, D+0x20, SOS, general constraints, D+0xc0
        h = ih(h, v);
    }
    for &c in &m.obj {
        h = dh(h, c);
    }
    for j in 0..n {
        let (b, l) = (m.col_beg[j] as usize, m.col_len[j] as usize);
        if l == 0 {
            h = ih(h, 0);
        }
        for k in b..b + l {
            h = ih(dh(h, m.col_val[k]), m.col_ind[k] as i64);
        }
    }
    for i in 0..nrows {
        h = ih(h, m.row_sense[i] as i8 as i64);
        h = dh(h, m.rhs[i]);
    }
    for j in 0..n {
        h = dh(h, m.lb[j]);
        h = dh(h, m.ub[j]);
        h = ih(h, m.vtype[j] as i8 as i64);
    }
    let v = ((h >> 32) as u32).wrapping_add(h as u32);
    if v == 0 {
        1
    } else {
        v
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::INFINITY;

    // Values reported by Gurobi 13.0.3 for the same models built in gurobipy.
    #[test]
    fn matches_original_on_tiny_models() {
        let empty = Model { sense: 1, ..Default::default() };
        assert_eq!(fingerprint(&empty), 4184956321);
        assert_eq!(fingerprint(&Model { sense: -1, ..Default::default() }), 3173427354);
        assert_eq!(fingerprint(&Model { sense: 1, objcon: 1.0, ..Default::default() }), 1511821735);
        let one_var = Model {
            sense: 1,
            obj: vec![0.0],
            lb: vec![0.0],
            ub: vec![INFINITY],
            vtype: vec![b'C'],
            col_beg: vec![0],
            col_len: vec![0],
            ..Default::default()
        };
        assert_eq!(fingerprint(&one_var), 2710021817);
    }
}
