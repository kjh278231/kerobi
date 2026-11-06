//! MIP postsolve records (state+0x370, pushed newest first; `State::post` keeps push order), in the MIP layouts of
//! docs/m2_red_inner.md §1.6, §2.2.5, §7.2.9. A record is {i32 type, data pointer, next} followed by its data in
//! a bump arena; only the data part is compared with Gurobi (`layout`).

/// One postsolve record.
#[derive(Clone, Debug, PartialEq)]
pub enum PostRec {
    /// type 1 (`39ab33`, MIP branch): column `j` fixed at `v`.
    Fix { j: i32, v: f64 },
    /// type 2 (`39b9e0` MIP branch, `39d9e1`): x_j = c0 + Σ val[k]·x_ind[k] (`39d9e1`: one term, the aggregation
    /// root and its multiplier).
    Subst { j: i32, c0: f64, ind: Vec<i32>, val: Vec<f64> },
    /// type 9 (`3a061e`): {i32 i, f64 d1, f64 d2}.
    T9 { i: i32, d1: f64, d2: f64 },
    /// type 4 (`39a1e9`): parallel columns, `y` merged into `x` (flag 0, x = x + r·y) or two integer columns into
    /// one integer (flag 1, `3ff744` mode 3); the bounds are those before the merge.
    Par { x: i32, y: i32, lx: f64, ly: f64, ux: f64, uy: f64, r: f64, flag: i32 },
    /// type 25 (`39ba81`): XOR rows removed by `3e5363`: the columns (pivot first), the rows with their live
    /// entries (`beg` has rows + 1 entries) and rhs, the parity kind `e8` (0: rhs 2, 1: rhs −1) and the pivot code.
    Xor { e8: i32, code: i32, cols: Vec<i32>, rows: Vec<i32>, beg: Vec<i32>, ind: Vec<i32>, val: Vec<f64>, rhs: Vec<f64> },
    /// type 14 (`399cbe`): opposite parallel rows split at model generation (`3ff744` mode 2): row `a` (now `=`,
    /// rhs `rhs`, range column number `k`, lower side `lo`), row `b` removed, ratio s_a/s_b, a's live entries.
    Split { a: i32, b: i32, k: i32, rhs: f64, lo: f64, ratio: f64, ind: Vec<i32>, val: Vec<f64> },
}

/// Data bytes of a record as Gurobi lays them out: padding bytes are 0 here (Gurobi leaves arena garbage) and
/// pointer fields hold their target's offset from the data start (Gurobi: an address).
pub struct Layout {
    pub bytes: Vec<u8>,
    /// [start, end) byte ranges that are padding
    pub pads: Vec<(usize, usize)>,
    /// offsets of 8-byte pointer fields
    pub ptrs: Vec<usize>,
}

impl PostRec {
    pub fn kind(&self) -> i32 {
        match self {
            PostRec::Fix { .. } => 1,
            PostRec::Subst { .. } => 2,
            PostRec::T9 { .. } => 9,
            PostRec::Par { .. } => 4,
            PostRec::Xor { .. } => 25,
            PostRec::Split { .. } => 14,
        }
    }

    pub fn layout(&self) -> Layout {
        let mut b = Vec::new();
        let mut pads = Vec::new();
        let mut ptrs = Vec::new();
        let i32b = |b: &mut Vec<u8>, x: i32| b.extend_from_slice(&x.to_le_bytes());
        let f64b = |b: &mut Vec<u8>, x: f64| b.extend_from_slice(&x.to_le_bytes());
        let pad = |b: &mut Vec<u8>, pads: &mut Vec<(usize, usize)>, k: usize| {
            pads.push((b.len(), b.len() + k));
            b.resize(b.len() + k, 0);
        };
        match self {
            PostRec::Fix { j, v } | PostRec::T9 { i: j, d1: v, .. } => {
                i32b(&mut b, *j);
                pad(&mut b, &mut pads, 4);
                f64b(&mut b, *v);
                if let PostRec::T9 { d2, .. } = self {
                    f64b(&mut b, *d2);
                }
            }
            PostRec::Par { x, y, lx, ly, ux, uy, r, flag } => {
                i32b(&mut b, *x);
                i32b(&mut b, *y);
                for v in [lx, ly, ux, uy, r] {
                    f64b(&mut b, *v);
                }
                i32b(&mut b, *flag);
                pad(&mut b, &mut pads, 4);
            }
            PostRec::Split { a, b: rb, k, rhs, lo, ratio, ind, val } => {
                // ind at +0x38, val at +0x40 + ((4·cnt − 1) & !7)
                let cnt = ind.len();
                let o_val = 0x40 + ((4 * cnt).saturating_sub(1) & !7);
                for x in [*a, *rb, *k, cnt as i32] {
                    i32b(&mut b, x);
                }
                for v in [rhs, lo, ratio] {
                    f64b(&mut b, *v);
                }
                ptrs.push(b.len());
                b.extend_from_slice(&0x38u64.to_le_bytes());
                ptrs.push(b.len());
                b.extend_from_slice(&(o_val as u64).to_le_bytes());
                for &x in ind {
                    i32b(&mut b, x);
                }
                if b.len() < o_val {
                    pads.push((b.len(), o_val));
                    b.resize(o_val, 0);
                }
                for &v in val {
                    f64b(&mut b, v);
                }
            }
            PostRec::Xor { e8, code, cols, rows, beg, ind, val, rhs } => {
                // `39ba81`: cols at +0x48, rows at +0x50+Pc, beg at +0x58+Pc+Pr, ind at +0x60+Pc+Pr+Pb, val at
                // +0x68+Pc+Pr+Pb+Pi, rhs after val; P = (4k − 1) & !7, Pb = (4·rows + 3) & !7
                let pk = |k: usize| (4 * k).saturating_sub(1) & !7;
                let (nc, nr, nz) = (cols.len(), rows.len(), ind.len());
                let o_rows = 0x50 + pk(nc);
                let o_beg = o_rows + 8 + pk(nr);
                let o_ind = o_beg + 8 + ((4 * nr + 3) & !7);
                let o_val = o_ind + 8 + pk(nz);
                let o_rhs = o_val + 8 * nz;
                ptrs.push(0);
                b.extend_from_slice(&0x48u64.to_le_bytes());
                i32b(&mut b, nc as i32);
                pad(&mut b, &mut pads, 4);
                ptrs.push(b.len());
                b.extend_from_slice(&(o_rows as u64).to_le_bytes());
                i32b(&mut b, nr as i32);
                i32b(&mut b, *e8);
                i32b(&mut b, *code);
                pad(&mut b, &mut pads, 4);
                for o in [o_beg, o_ind, o_val, o_rhs] {
                    ptrs.push(b.len());
                    b.extend_from_slice(&(o as u64).to_le_bytes());
                }
                let fill = |b: &mut Vec<u8>, pads: &mut Vec<(usize, usize)>, to: usize| {
                    if b.len() < to {
                        pads.push((b.len(), to));
                        b.resize(to, 0);
                    }
                };
                for &k in cols {
                    i32b(&mut b, k);
                }
                fill(&mut b, &mut pads, o_rows);
                for &k in rows {
                    i32b(&mut b, k);
                }
                fill(&mut b, &mut pads, o_beg);
                for &k in beg {
                    i32b(&mut b, k);
                }
                fill(&mut b, &mut pads, o_ind);
                for &k in ind {
                    i32b(&mut b, k);
                }
                fill(&mut b, &mut pads, o_val);
                for &x in val {
                    f64b(&mut b, x);
                }
                for &x in rhs {
                    f64b(&mut b, x);
                }
            }
            PostRec::Subst { j, c0, ind, val } => {
                let cnt = ind.len();
                // `39b9e0`: values start at data+0x30+P, P = (4·cnt − 1) & !7
                let p = (4 * cnt).saturating_sub(1) & !7;
                i32b(&mut b, *j);
                pad(&mut b, &mut pads, 4);
                f64b(&mut b, *c0);
                i32b(&mut b, cnt as i32);
                pad(&mut b, &mut pads, 4);
                ptrs.push(b.len());
                b.extend_from_slice(&0x28u64.to_le_bytes());
                ptrs.push(b.len());
                b.extend_from_slice(&(0x30 + p as u64).to_le_bytes());
                for &k in ind {
                    i32b(&mut b, k);
                }
                let gap = 0x30 + p - b.len();
                pad(&mut b, &mut pads, gap);
                for &x in val {
                    f64b(&mut b, x);
                }
            }
        }
        Layout { bytes: b, pads, ptrs }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn xor_layout_offsets() {
        // 3 columns, 4 rows of 3 entries: rows at 0x58, beg at 0x68, ind at 0x80, val at 0xb0, rhs at 0x110
        let l = PostRec::Xor { e8: 0, code: 0, cols: vec![5, 1, 2], rows: vec![0, 1, 2, 3], beg: vec![0, 3, 6, 9, 12],
                               ind: vec![0; 12], val: vec![1.0; 12], rhs: vec![2.0, 0.0, 0.0, 0.0] }.layout();
        let ptr = |o: usize| u64::from_le_bytes(l.bytes[o..o + 8].try_into().unwrap());
        assert_eq!([ptr(0), ptr(0x10), ptr(0x28), ptr(0x30), ptr(0x38), ptr(0x40)], [0x48, 0x58, 0x68, 0x80, 0xb0, 0x110]);
        assert_eq!(l.bytes.len(), 0x110 + 32);
        let sp = PostRec::Split { a: 1, b: 2, k: 0, rhs: 1.0, lo: 0.0, ratio: -1.0, ind: vec![3, 4, 5], val: vec![1.0; 3] };
        assert_eq!((sp.layout().bytes.len(), sp.layout().ptrs), (0x48 + 24, vec![0x28, 0x30]));
        assert_eq!(PostRec::Par { x: 1, y: 2, lx: 0.0, ly: 0.0, ux: 1.0, uy: 1.0, r: 1.0, flag: 0 }.layout().bytes.len(), 0x38);
    }

    #[test]
    fn subst_layout_matches_39d9e1() {
        // 39d9e1: 0x50-byte record, data {j, pad, o, 1, pad, ind ptr = +0x28, val ptr = +0x30, root, pad, m}
        let l = PostRec::Subst { j: 7, c0: -0.0, ind: vec![3], val: vec![2.5] }.layout();
        assert_eq!(l.bytes.len(), 0x38);
        assert_eq!(l.ptrs, [0x18, 0x20]);
        assert_eq!(l.pads, [(4, 8), (0x14, 0x18), (0x2c, 0x30)]);
        assert_eq!(&l.bytes[0x28..0x2c], &3i32.to_le_bytes());
        assert_eq!(&l.bytes[0x30..0x38], &2.5f64.to_le_bytes());
        // three terms: P = 8, values at 0x38
        let l = PostRec::Subst { j: 1, c0: 1.0, ind: vec![0, 2, 4], val: vec![1.0, -1.0, 0.5] }.layout();
        assert_eq!(&l.bytes[0x20..0x28], &0x38u64.to_le_bytes());
        assert_eq!(l.bytes.len(), 0x38 + 24);
    }
}
