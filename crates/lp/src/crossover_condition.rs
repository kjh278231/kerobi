//! Condition estimate `116fd7` (double path of `12f282`): ‖B‖₁ times a Hager-style estimate of ‖B⁻¹‖₁ on the
//! push factor, with the FTRAN `09cf9b`→`2207de` and the BTRAN `9e288`→`221173` of [`LuState`]. The sums follow the
//! original's 8-lane SIMD order, which depends on the 16-byte alignment of the summed buffers ([`CondEntry`]); the
//! FTRAN/BTRAN may exchange the headers of the two work vectors S+0x220/0x228, so which physical buffer x points at is
//! tracked. Contract and C/asm references: docs/m3/primal_push_condition.md.
use crate::crossover_ftran::{eta_dot, Buffer, VecSnap};
use crate::crossover_lu::LuState;
use crate::crossover_refactor::Matrix;

/// What `116fd7` reads besides the factor and the matrix: F+0xc and the alignment (address mod 16) of the matrix value
/// array S+0x88 and of the value buffers of x = S+0x220 and y = S+0x228 at entry.
#[derive(Clone, Copy, Debug)]
pub struct CondEntry {
    pub f0c: i32,
    pub a_val_mod16: u8,
    pub x_mod16: u8,
    pub y_mod16: u8,
}

impl CondEntry {
    /// F+0xc = 0 (just refactored, `05b257` C:1977) and every buffer 16-byte aligned (all 64 captured calls).
    pub const ALIGNED: CondEntry = CondEntry { f0c: 0, a_val_mod16: 0, x_mod16: 0, y_mod16: 0 };
}

/// The two physical value buffers: the ones x and y held at entry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phys {
    X,
    Y,
}

/// A work vector header: count (−1 dense), packed indices and values.
#[derive(Clone, Debug, PartialEq)]
pub struct CondVec {
    pub count: i32,
    pub idx: Vec<i32>,
    pub val: Vec<f64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CondStop {
    /// m = 0 (0.0) or F+0xc ≠ 0 (1.0).
    Early,
    /// The estimate did not grow (C: `0 < it && est <= old`).
    Estimate,
    /// Five BTRANs.
    Limit,
    /// The new index equals the previous one.
    Repeat,
}

/// Observation points (the M3-04c-1c capture boundaries plus the derived values).
#[derive(Debug)]
pub enum CondEvent<'a> {
    Norm { norm: f64, header: &'a [i32] },
    /// FTRAN k: input x, output y, then the value buffers x and y point at.
    Ftran { k: usize, input: &'a CondVec, output: &'a CondVec, x: Phys, y: Phys },
    Estimate { k: usize, est: f64 },
    Btran { k: usize, input: &'a [f64], output: &'a CondVec, x: Phys, y: Phys },
    Done { stop: CondStop, iterations: usize, condition: f64 },
}

/// Σ|v| in the order of asm 0x1a96e3–0x1a97fd (and 0x1a9387–0x1a948b for the columns): a buffer at 8 mod 16 adds its
/// first term to lane 0 before the aligned 8-lane body.
fn asum(vals: &[f64], mod16: u8) -> f64 {
    assert!(mod16 % 8 == 0, "buffer not 8-byte aligned");
    let a: Vec<f64> = vals.iter().map(|v| v.abs()).collect();
    eta_dot(&a, mod16 == 8)
}

fn column(v: &VecSnap, what: &str) -> CondVec {
    let count = v.count.unwrap_or_else(|| panic!("{what}: unknown count"));
    let val = v.val.iter().map(|x| x.unwrap_or_else(|| panic!("{what}: unknown value"))).collect();
    let idx = v.idx.as_ref().map_or_else(Vec::new, |ix| {
        ix.iter().map(|x| x.unwrap_or_else(|| panic!("{what}: unknown index"))).collect()
    });
    CondVec { count, idx, val }
}

fn phys(b: Buffer, at_in: Phys, at_out: Phys) -> Phys {
    match b {
        Buffer::In => at_in,
        Buffer::Out => at_out,
    }
}

/// `116fd7` on the factor `lu` (header F+0x18 = `lu.header`) and the matrix `a`; returns *param_2.
pub fn condition(lu: &mut LuState, a: &Matrix, e: CondEntry, ev: &mut dyn FnMut(CondEvent<'_>)) -> f64 {
    let m = lu.m;
    // C 98–105 (asm 0x1a9303–0x1a9310)
    if m == 0 || e.f0c != 0 {
        let c = if m == 0 { 0.0 } else { 1.0 };
        ev(CondEvent::Done { stop: CondStop::Early, iterations: 0, condition: c });
        return c;
    }
    // ‖B‖₁: slack positions count 1.0, the maximum by `maxsd new, old` from 0
    let mut norm = 0.0f64;
    for &j in &lu.header {
        let s = if j as usize >= a.n {
            1.0
        } else {
            let j = j as usize;
            let b = a.beg[j];
            let vals = &a.val[b as usize..b as usize + a.len[j] as usize];
            asum(vals, ((i64::from(e.a_val_mod16) + 8 * b) % 16) as u8)
        };
        norm = if s > norm { s } else { norm };
    }
    ev(CondEvent::Norm { norm, header: &lu.header });
    let mod16 = |p: Phys| if p == Phys::X { e.x_mod16 } else { e.y_mod16 };

    let mut x = CondVec { count: -1, idx: Vec::new(), val: vec![1.0 / m as f64; m] };
    let (mut xb, mut yb) = (Phys::X, Phys::Y);
    let (mut est, mut prev, mut it, mut k) = (0.0f64, -1i32, 0usize, 0usize);
    let stop = loop {
        // 09cf9b(F, x, y, 0, work): 2207de with no spike
        let s = lu.ftran(x.count, &x.idx, &x.val, 0, false).unwrap_or_else(|err| panic!("116fd7 FTRAN: {err:?}"));
        let (out, _) = s.snapshot(m);
        let y = column(&out, "116fd7 FTRAN");
        let (nx, ny) = (phys(s.input.val, xb, yb), phys(s.output.val, xb, yb));
        (xb, yb) = (nx, ny);
        ev(CondEvent::Ftran { k, input: &x, output: &y, x: xb, y: yb });
        // x.count = y.count and the values (indices when sparse) copied into x's buffer
        x = y;
        let old = est;
        est = if x.count == 0 { 0.0 } else { asum(&x.val, mod16(xb)) };
        ev(CondEvent::Estimate { k, est });
        if it > 0 && old >= est {
            break CondStop::Estimate;
        }
        // y = sign(x), dense
        let mut sign = vec![1.0; m];
        if x.count < 0 {
            for (i, v) in x.val.iter().enumerate() {
                if *v < 0.0 {
                    sign[i] = -1.0;
                }
            }
        } else {
            for (q, v) in x.val.iter().enumerate() {
                if *v < 0.0 {
                    sign[x.idx[q] as usize] = -1.0;
                }
            }
        }
        // 9e288(F, y, x, work): F+0x200 = 0, 221173 with a null cache
        let s = lu.btran(-1, &[], &sign, x.count, false, 0.0);
        let (out, _) = s.snapshot(m);
        let z = column(&out, "116fd7 BTRAN");
        let (nx, ny) = (phys(s.output.val, yb, xb), phys(s.input.val, yb, xb));
        (xb, yb) = (nx, ny);
        ev(CondEvent::Btran { k, input: &sign, output: &z, x: xb, y: yb });
        it += 1;
        if it >= 5 {
            break CondStop::Limit;
        }
        // first index of the largest |z| (from 0, index −1)
        let (mut best, mut j) = (0.0f64, -1i32);
        for (q, v) in z.val.iter().enumerate() {
            if v.abs() > best {
                best = v.abs();
                j = if z.count < 0 { q as i32 } else { z.idx[q] };
            }
        }
        if j == prev {
            break CondStop::Repeat;
        }
        x = CondVec { count: 1, idx: vec![j], val: vec![1.0] };
        prev = j;
        k += 1;
    };
    let c = norm * est;
    ev(CondEvent::Done { stop, iterations: it, condition: c });
    c
}
