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
    /// type 11 (`3a0e1b`): row r became the slack column j (coefficient |a|).
    T11 { r: i32, j: i32, a: f64 },
    /// type 10 (`3a07b5`): x_j = d1·x_j' + d2·x_k + d3 (`40d2bc` mode 1: column k cancelled with pivot j).
    T10 { j: i32, k: i32, d1: f64, d2: f64, d3: f64 },
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
            PostRec::T10 { .. } => 10,
            PostRec::T11 { .. } => 11,
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
            PostRec::T11 { r, j, a } => {
                i32b(&mut b, *r);
                i32b(&mut b, *j);
                f64b(&mut b, *a);
            }
            PostRec::T10 { j, k, d1, d2, d3 } => {
                i32b(&mut b, *j);
                i32b(&mut b, *k);
                for x in [d1, d2, d3] {
                    f64b(&mut b, *x);
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

    #[test]
    fn t10_t11_layouts() {
        // 3a07b5: {p, i, 1.0, mult, 0.0}, no padding (neos-860300 414c53 record bytes)
        let l = PostRec::T10 { j: 3, k: 2, d1: 1.0, d2: -1.0, d3: 0.0 }.layout();
        assert_eq!(l.bytes, hex("0300000002000000000000000000f03f000000000000f0bf0000000000000000"));
        // 3a0e1b: {r, j, |a|} (graphdraw-domain 37f484 record bytes)
        let l = PostRec::T11 { r: 0x2d, j: 0x14, a: 1.0 }.layout();
        assert_eq!(l.bytes, hex("2d00000014000000000000000000f03f"));
    }

    fn hex(s: &str) -> Vec<u8> {
        (0..s.len()).step_by(2).map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap()).collect()
    }
}
