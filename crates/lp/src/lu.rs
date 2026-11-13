//! 04 §4 Basis factorization. Singleton phase of the original `1da0b2` (row singletons from a stack, slack rows
//! without structural entries, then column singletons in basis order), a Markowitz phase for what is left, the
//! column-oriented dense solves of `214344`/`2126cd` and Forrest–Tomlin updates (`05b257`). On a triangular basis
//! the bits depend only on the pivot order, so this reproduces the original x_B exactly (60/60 bench LPs with a
//! valid iteration-0 snapshot). `factor` reports an incomplete factorization on a singular basis.
use crate::spx::Spx;

/// Drop tolerance (GURO_PAR_ZERO, spx+0x3a8 / LU+0x10).
pub const ZERO: f64 = 1e-13;

/// (index, value) entries in storage order.
type Sparse = Vec<(usize, f64)>;

pub struct Lu {
    /// Pivot steps in factor order: (row, basis position, diagonal). FT updates replace the diagonal in place.
    pub steps: Vec<(usize, usize, f64)>,
    /// Step at which each row was pivoted (-1 if not).
    pub prow: Vec<i64>,
    pub head: Vec<usize>,
    /// Step of each basis position.
    kpos: Vec<usize>,
    /// L column of step k: (row i, multiplier) with x_i −= l·x_{r_k}, applied in factor order.
    lcol: Vec<Vec<(usize, f64)>>,
    /// Lᵀ row of step k: (earlier step j, multiplier), descending j (LU+0x70/0x78/0x80).
    lrow: Vec<Vec<(usize, f64)>>,
    /// U off-diagonals: column of step k as (row, u) over steps earlier in `order`, and the same entries by row as
    /// (later step, u).
    ucol: Vec<Vec<(usize, f64)>>,
    urow: Vec<Vec<(usize, f64)>>,
    /// Column k's entries in rows of tail steps come first, `umid[k]` of them, then the prefix rows (U+0x78 of the
    /// `21686f` view; 05b257 keeps the split as rows cross t).
    umid: Vec<usize>,
    /// U order of the steps (LU+0x48 ord: position → step); identity after a factor, permuted by FT updates.
    order: Vec<usize>,
    /// U position of each step (LU+0x40 pos), the inverse of `order`.
    upos: Vec<usize>,
    /// Prefix of U positions with empty U columns as the original tracks it (LU+0x100).
    prefix: usize,
    /// Cache of the highest U position with an empty column (LU+0xe8).
    e8: usize,
    /// End of the slack phase of the singleton factor (LU+0xdc): prefix positions below it start with empty U rows.
    dc: usize,
    /// LU+0x108 + LU+0x110: mean U row length + 1 of the tail rows and of the rows in [0xdc, t) (`036ade`
    /// C:5725–5740), the density factor of the `09d112` U-stage test.
    dsum: f64,
    /// LU+0x108 alone: `21a941` takes its linear path above 5.
    d108: f64,
    /// Prefix rows with a non-empty U row (LU+0xf0, its length LU+0xec) as steps in list order, and each step's slot
    /// in it (LU+0xf8: ≥ 0 slot, −1 prefix with an empty U row, −2 tail).
    f0: Vec<usize>,
    f8: Vec<i64>,
    /// Forrest–Tomlin row etas in creation order: (row r, [(row i, μ)]) with x_r −= Σ μ·x_i.
    retas: Vec<(usize, Vec<(usize, f64)>)>,
    /// Per row eta: its values start at an odd offset of the LU pool, i.e. 8 but not 16-byte aligned.
    reta_odd: Vec<bool>,
    /// LU pool fill LU+0x160: U rows, U columns and row etas share LU+0x148/0x150 (16-byte aligned base in the kb2
    /// capture). 2·nnz(L+U) after a factor, then grown by `update` (05b257).
    pub pool: usize,
    /// Pool slots of the U rows by step: their lengths after a factor, grown by `update`.
    rcap: Vec<usize>,
    /// Sparse-solve multiplier LU+0x104: a sparse Lᵀ needs `mult·nnz ≤ m` (036ade L5709).
    sparse_mult: usize,
    /// Lᵀ DAG level of each step (LU+0x130) when BTRAN takes the level path (LU+0x140 ≥ 0, 036ade L5757).
    level: Option<Vec<usize>>,
    /// L DAG level of each step (LU+0x118) and the deepest level LU+0x128 (−1 without levels), `036ade` C:5764–5830.
    llev: Option<Vec<usize>>,
    lmax: i64,
    /// Lᵀ DAG depth LU+0x140 (−1 without levels), read by BTRAN's sparse Lᵀ test.
    ltmax: i64,
    /// L entries in the columns of steps ≥ `s4` (LU+0xe4): `21b9de`'s path test reads it.
    ltail: usize,
    /// U has off-diagonals (Markowitz part or updates): BTRAN leaves the iteration-0 emulation.
    general: bool,
    pub n_updates: usize,
    /// Steps taken by the singleton phases (the rest is the Markowitz kernel).
    pub n_singleton: usize,
    /// End of the row-singleton stack phase (LU+0xd8) and of the column-singleton phase 3 (LU+0xe0 = LU+0x100 as
    /// `1da0b2` leaves it, C:1514–1532).
    s1: usize,
    s3: usize,
    /// LU+0xe4: steps before the 2×2 block phase when it ran (`local_88`, C:1512–1520), else `n_singleton`.
    s4: usize,
    /// L column (row, l) and U row (position, u) of each step the 2×2 block phase took (steps s4..n_singleton), in
    /// the original's storage order.
    blk: Vec<(Sparse, Sparse)>,
    /// U rows of the kernel steps by step: (position, u), moved into `ucol` after the factor.
    kernel_u: Vec<Vec<(usize, f64)>>,
    /// Singular factor: (position, row) pairs of the empty columns and rows in discovery order (tail steps m−1,
    /// m−2, … of `036ade`); 5b257 puts the row's slack at that position.
    pub singular: Vec<(usize, usize)>,
    /// Work inputs of this factorization (`036ade`, docs/m1/m1_02q_work.md).
    pub fw: FactorWork,
    /// Pool size `param_15[2]` (F+0x158) when the kernel may take the dense switch (`factor_dense`); `None` keeps
    /// the shortcut (no dense switch) of the M1 path.
    dense_cap: Option<i64>,
    /// The dense switch this factor took (`factor_dense`).
    pub dense: Option<DenseTrace>,
    /// M3-04c-21j: `036ade`'s pool positions (`factor_dense` only), their snapshots at the probe points, the
    /// `local_70` mode's pivots, and a pool shortage (`036ade` returns −1).
    fpool: Option<Pool>,
    pub pool_trace: Vec<PoolEvent>,
    pub mode70: Option<Mode70Trace>,
    pub short: Option<Short>,
}

/// `036ade` pool positions at a probe point (M3-04c-21j): `local_1b8` (next L slot, downward), `local_1c0` (next U
/// slot), `local_f8` (first fill), `local_138`, `param_15[2..=4]`; `nodes` the kernel's initial node count
/// (`start`), `n` = m − steps (`mode70`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PoolEvent {
    pub kind: &'static str,
    pub nodes: i64,
    pub n: i64,
    pub b8: i64,
    pub c0: i64,
    pub f8: i64,
    pub x138: i64,
    pub p2: i64,
    pub p3: i64,
    pub p4: i64,
}

/// The `local_70` mode of a factor: the step it took over, its dense size n', and each pivot as the flat index
/// r'·n' + c' of its dense array (asm 0xd9f5b esi).
#[derive(Clone, Debug)]
pub struct Mode70Trace {
    pub step: usize,
    pub n: usize,
    pub pivots: Vec<i64>,
    /// Branch counts (coverage only): full column / row scans, list column / row scans, cost-tie acceptances,
    /// markers set, structural fills (list adds), fills on a marker, list adds skipped (full), new empty rows /
    /// columns, dense gate recounts, list cap iVar67.
    pub stats: [usize; 12],
}

/// A pool shortage: `036ade` returns −1 with F+0x158 = F+0x168 set to a new size (`Grow`, C:2592–2620) or left as
/// it was (`Keep`, the other −1 exits; `05b257` C:1487–1492 then grows it by 1.5).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Short {
    Grow(i64),
    Keep,
}

impl Short {
    /// F+0x158 for the next `036ade` call after this shortage (`05b257` C:1479–1513): the size `036ade` set, or the
    /// old size ×1.5 when it left F+0x158 alone.
    pub fn next_cap(self, cap: i64) -> i64 {
        match self {
            Short::Grow(c) => c,
            Short::Keep => {
                let g = cap as f64 * 1.5;
                if g < 9.223372036854776e18 { g as i64 } else { i64::MAX }
            }
        }
    }
}

/// `045a76` FactSpace: the initial pool size F+0x158 of an LU for m rows, n columns, nnz(A) (spx+0x3d0),
/// `GURO_PAR_SIMPLEXALLOCS` k (spx+0x50) and the model's pool regrowth counter c (asm 0xe3697–0xe3bf0,
/// docs/m1/m1_02w2_factspace.md). shortcut: the first allocation try is assumed to succeed (loop 0xe3a7c), and
/// c ≥ 8 (the paired mulpd path 0xe3a0b) is not modelled.
pub fn factspace(m: usize, n: usize, nnz: usize, k: i32, c: u32) -> i64 {
    assert!(c < 8, "FactSpace c >= 8 not modelled");
    let (fm, fn_, fz) = (m as f64, n as f64, nnz as f64);
    let d0 = if m < n {
        let d = fz / fn_;
        let x = if m < 500 {
            d * d
        } else if m < 3000 {
            d * d.sqrt()
        } else if m < 10000 && fm > d * 4.0 {
            // shortcut: asm 0xe3707–0xe37cb read, no captured case; check before relying on m ≥ 3000
            let q = d * d.sqrt().sqrt();
            if fm * 0.25 > q { q } else { fm * 0.25 }
        } else {
            d
        };
        let half = (m / 2 + 2) as f64;
        (if half < x { half } else { x }) * fm
    } else if m == 0 {
        100.0
    } else {
        let d = fz / fm;
        let x = if n < 500 {
            d * d
        } else if n < 3000 {
            d * d.sqrt()
        } else if n < 10000 {
            d * d.sqrt().sqrt() // shortcut: as above, no captured case
        } else {
            d
        };
        let half = (n / 2) as f64;
        fz * 0.5 + (if half < x { half } else { x }) * fn_
    };
    let r = (d0 as i64 + 1) as f64;
    let mut dd = if fz > fn_ * 10.0 {
        r * 6.0 + fm * 30.0 * k as f64
    } else if m < n {
        r * 6.0 + fm * 3.0 * fz * k as f64 / fn_
    } else {
        r * 6.0 + fm * 3.0 * (m + nnz) as f64 * k as f64 / (m + 1 + n) as f64
    };
    for _ in 0..c {
        dd *= 1.5;
    }
    (dd * 0.75) as i64 + 1
}

/// `036ade`'s element pool bookkeeping (C:296–366, C:541–606, C:2440–2490): U from 0 up, L from F+0x168 down,
/// element nodes (32 bytes = 4 slots) from `local_f8` up in chunks of 4m nodes; each chunk's slots also give the
/// int pool's 16-aligned rest as a later chunk (`local_2f0`).
#[derive(Clone, Debug)]
struct Pool {
    m: i64,
    p2: i64,
    p4: i64,
    f8: i64,
    b8: i64,
    c0: i64,
    p3: i64,
    x138: i64,
    /// nodes left in the current chunk (`local_2d8`) and the pending int-side chunk (`local_2f0`)
    left: i64,
    pend: i64,
}

/// Nodes of the int pool's part of slots [p3, p3 + len): from the 16-byte aligned start to the end, 32 bytes each.
fn int_side(p3: i64, len: i64) -> i64 {
    let q = (p3 + 3 + (((p3 + 3) >> 1) as u64 >> 62) as i64) >> 2;
    (((p3 + len - 4 * q) as u64 & 0x3ffffffffffffff8) >> 3) as i64
}

impl Pool {
    fn snap(&self, kind: &'static str, nodes: i64, n: i64) -> PoolEvent {
        PoolEvent { kind, nodes, n, b8: self.b8, c0: self.c0, f8: self.f8, x138: self.x138, p2: self.p2, p3: self.p3, p4: self.p4 }
    }

    /// A fill node (C:2449–2490, asm 0xd808c): a new chunk of 4m nodes (16m slots) when the current one and the
    /// pending int-side one are used up; false when the pool cannot take it (`LAB_001d81b2`).
    fn node(&mut self) -> bool {
        if self.left < 1 {
            if self.pend < 1 {
                let slots = 16 * self.m;
                if self.b8 < self.p3 || ((self.b8 - self.p3) as f64) * 8.0 <= (slots * 8) as f64 {
                    return false;
                }
                let e = int_side(self.p3, slots);
                if e != 0 {
                    self.pend = e;
                }
                self.p3 += slots;
                self.x138 = self.x138.max(self.p3);
                self.left = 4 * self.m;
            } else {
                self.left = self.pend;
                self.pend = 0;
            }
        }
        self.left -= 1;
        true
    }
}

/// Borrowed internals of a factor (`Lu::parts`), in the factor's own step numbering.
pub struct LuParts<'a> {
    pub steps: &'a [(usize, usize, f64)],
    pub prow: &'a [i64],
    pub lcol: &'a [Vec<(usize, f64)>],
    pub lrow: &'a [Vec<(usize, f64)>],
    pub ucol: &'a [Vec<(usize, f64)>],
    pub urow: &'a [Vec<(usize, f64)>],
    pub umid: &'a [usize],
    pub prefix: usize,
    pub e8: usize,
    pub dc: usize,
    pub f0: &'a [usize],
    pub f8: &'a [i64],
    pub pool: usize,
    pub sparse_mult: usize,
    pub level: Option<&'a [usize]>,
    pub llev: Option<&'a [usize]>,
    pub lmax: i64,
    pub n_singleton: usize,
    pub s1: usize,
    pub s3: usize,
    pub s4: usize,
}

/// The `1da0b2` 2×2 block phase state on the simplex struct: S+0x2c0 (refactor count, raised by `05b257` C:1399
/// before the call), S+0x4e0 (runs of the phase) and S+0x4e4 (LU2X2BLOCK: −1 off, 2e9 always, else the runs that
/// pivoted 20 or more steps; `02f7e1` C:270–281, `1da0b2` C:1494–1510).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BlockState {
    pub x2c0: i32,
    pub x4e0: i32,
    pub x4e4: i32,
}

impl BlockState {
    /// The phase switched off (what the M1 simplex path factors with).
    pub const OFF: BlockState = BlockState { x2c0: 0, x4e0: 0, x4e4: -1 };
}

/// The dense switch of the `036ade` kernel as one factor took it: the step it left the sparse kernel at, the dense
/// order (positions by column-count bucket, rows ascending) and the array before and after the dense LU.
#[derive(Clone, Debug, Default)]
pub struct DenseTrace {
    pub step: usize,
    pub n: usize,
    /// 16-byte offset of the dense array (`param_15[1] + param_15[3]·8`): 8 when the pool fill is odd.
    pub mat16: usize,
    pub rows: Vec<usize>,
    pub cols_in: Vec<usize>,
    pub a_in: Vec<f64>,
    pub cols_out: Vec<usize>,
    pub a_out: Vec<f64>,
}

/// `036ade` C:331–366 (asm 0xd440a–0xd457b): the element pool's first fill `local_f8` from the basis entries `nnz`,
/// m and the pool size (`param_15[2]`, F+0x158). Every later chunk adds a multiple of 4 slots, so its parity is
/// the dense array's 16-byte alignment.
pub fn pool_first_fill(nnz: usize, m: usize, cap: i64) -> i64 {
    let d = nnz as f64 / m as f64;
    let x = if m < 500 {
        d * d
    } else if m < 3000 {
        d * d.sqrt()
    } else if m < 10000 {
        d * d.sqrt().sqrt()
    } else {
        d
    };
    let half = (m as i32 / 2 + 2) as f64;
    let y = if half < x { half } else { x }; // minsd
    let y = (y + y) * m as f64;
    let c5 = (cap / 5) as f64;
    (if y < c5 { c5 } else { y }) as i64 // maxsd, cvttsd2si
}

/// `036ade`'s own dense LU (C:3968–4920, asm 0xdd1b2–0xde68c) on the column-major n×n array `a` (column k holds
/// row `cols[k]`, row i position `rows[i]`): 4-row blocks of left-looking updates (two SSE2 lanes, one peeled
/// element when a column starts off 16 bytes), the largest |a| along the pivot row (first maximum) with column
/// swaps, then single rows (eight lanes against the scratch row) for the last < 4. `first` is the blocked pass;
/// a block pivot below `zero` asks for the unblocked pass (`None`), where a pivot below `zero` sends its row to the
/// end. Returns the rows left without a pivot.
#[allow(clippy::too_many_arguments)]
pub fn dense_lu(
    a: &mut [f64],
    n: usize,
    cols: &mut [usize],
    rows: &mut [usize],
    zero: f64,
    mat16: usize,
    scr16: usize,
    first: bool,
) -> Option<usize> {
    let col_peel = |c: usize| usize::from(!(mat16 + 8 * c * n).is_multiple_of(16));
    let scr_peel = usize::from(!scr16.is_multiple_of(16));
    let mut c0 = 0;
    if first {
        let mut j = 0;
        loop {
            if n - j < 4 {
                break;
            }
            let mut s = [vec![0.0f64; j], vec![0.0f64; j], vec![0.0f64; j], vec![0.0f64; j]];
            for k in 0..j {
                for (r, sr) in s.iter_mut().enumerate() {
                    sr[k] = a[j + r + k * n];
                }
            }
            // C:3995–4158: the block rows against the earlier U rows (columns 1..j), two lanes
            for k in 1..j {
                let col = k * n;
                let mut v = [s[0][k], s[1][k], s[2][k], s[3][k]];
                let mut t0 = 0;
                if k > 1 {
                    let peel = col_peel(k);
                    if peel + 2 <= k {
                        let end = k - ((k - peel) & 1);
                        for t in 0..peel {
                            for r in 0..4 {
                                v[r] -= s[r][t] * a[t + col];
                            }
                        }
                        let (mut l0, mut l1) = (v, [0.0f64; 4]);
                        let mut t = peel;
                        while t < end {
                            for r in 0..4 {
                                l0[r] -= s[r][t] * a[t + col];
                                l1[r] -= s[r][t + 1] * a[t + 1 + col];
                            }
                            t += 2;
                        }
                        for r in 0..4 {
                            v[r] = l0[r] + l1[r];
                        }
                        t0 = end;
                    }
                }
                for t in t0..k {
                    for r in 0..4 {
                        v[r] -= s[r][t] * a[t + col];
                    }
                }
                for r in 0..4 {
                    s[r][k] = v[r];
                }
            }
            for k in 0..j {
                for (r, sr) in s.iter().enumerate() {
                    a[j + r + k * n] = sr[k];
                }
            }
            // C:4175–4307: the block rows of columns j.. against the solved left part
            for c in j..n {
                let col = c * n;
                let mut v = [a[j + col], a[j + 1 + col], a[j + 2 + col], a[j + 3 + col]];
                if j > 0 {
                    let mut t0 = 0;
                    if j >= 2 {
                        let peel = col_peel(c);
                        if peel + 2 <= j {
                            let end = j - ((j - peel) & 1);
                            for t in 0..peel {
                                for r in 0..4 {
                                    v[r] -= s[r][t] * a[t + col];
                                }
                            }
                            let (mut l0, mut l1) = (v, [0.0f64; 4]);
                            let mut t = peel;
                            while t < end {
                                for r in 0..4 {
                                    l0[r] -= s[r][t] * a[t + col];
                                    l1[r] -= s[r][t + 1] * a[t + 1 + col];
                                }
                                t += 2;
                            }
                            for r in 0..4 {
                                v[r] = l0[r] + l1[r];
                            }
                            t0 = end;
                        }
                    }
                    for t in t0..j {
                        for r in 0..4 {
                            v[r] -= s[r][t] * a[t + col];
                        }
                    }
                }
                for (r, &x) in v.iter().enumerate() {
                    a[j + r + col] = x;
                }
            }
            // C:4308–4463: pivots of the block rows
            for p in j..j + 4 {
                let d = p + p * n;
                let (mut best, mut bi) = (a[d].abs(), p);
                for t in p + 1..n {
                    let x = a[p + t * n].abs();
                    if best < x {
                        best = x;
                        bi = t;
                    }
                }
                if best < zero {
                    return None;
                }
                if bi != p {
                    cols.swap(bi, p);
                    for x in 0..n {
                        a.swap(p * n + x, bi * n + x);
                    }
                }
                let piv = a[d];
                let mut l = d + n;
                match p - j {
                    0 => {
                        let (b1, b2, b3) = (a[d + 1], a[d + 2], a[d + 3]);
                        while l < n * n {
                            let q = a[l] / piv;
                            a[l] = q;
                            a[l + 1] -= b1 * q;
                            a[l + 2] -= b2 * q;
                            a[l + 3] -= q * b3;
                            l += n;
                        }
                    }
                    1 => {
                        let (b1, b2) = (a[d + 1], a[d + 2]);
                        while l < n * n {
                            let q = a[l] / piv;
                            a[l] = q;
                            a[l + 1] -= b1 * q;
                            a[l + 2] -= q * b2;
                            l += n;
                        }
                    }
                    2 => {
                        let b1 = a[d + 1];
                        while l < n * n {
                            let q = a[l] / piv;
                            a[l] = q;
                            a[l + 1] -= q * b1;
                            l += n;
                        }
                    }
                    _ => {
                        while l < n * n {
                            a[l] /= piv;
                            l += n;
                        }
                    }
                }
            }
            j += 4;
            if j >= n {
                break;
            }
        }
        c0 = j;
    }
    // C:4470–4909: single rows, eight lanes against the scratch copy of the row's left part
    let dot8 = |a: &[f64], col: usize, s: &[f64], terms: usize, mut v: f64| -> f64 {
        let mut t0 = 0;
        if terms >= 8 && terms >= scr_peel + 8 {
            let end = terms - ((terms - scr_peel) & 7);
            for t in 0..scr_peel {
                v -= a[col + t] * s[t];
            }
            let mut l = [[v, 0.0], [0.0, 0.0], [0.0, 0.0], [0.0, 0.0]];
            let mut t = scr_peel;
            while t < end {
                for (q, lq) in l.iter_mut().enumerate() {
                    lq[0] -= a[col + t + 2 * q] * s[t + 2 * q];
                    lq[1] -= a[col + t + 2 * q + 1] * s[t + 2 * q + 1];
                }
                t += 8;
            }
            // asm 0xde06e: (A + B) + (C + D) lane-wise, then the two lanes
            let x0 = (l[0][0] + l[1][0]) + (l[2][0] + l[3][0]);
            let x1 = (l[0][1] + l[1][1]) + (l[2][1] + l[3][1]);
            v = x0 + x1;
            t0 = end;
        }
        for t in t0..terms {
            v -= a[col + t] * s[t];
        }
        v
    };
    let mut last = n as isize - 1;
    let mut c = c0;
    let mut s = vec![0.0f64; n];
    while (c as isize) <= last {
        if c > 0 {
            for t in 0..c {
                s[t] = a[c + t * n];
            }
            for k in 1..c {
                s[k] = dot8(a, k * n, &s, k, s[k]);
            }
            for t in 0..c {
                a[c + t * n] = s[t];
            }
        }
        for cc in c..n {
            let v = a[c + cc * n];
            a[c + cc * n] = if c > 0 { dot8(a, cc * n, &s, c, v) } else { v };
        }
        let d = c + c * n;
        let (mut best, mut bi) = (a[d].abs(), c);
        for t in c + 1..n {
            let x = a[c + t * n].abs();
            if best < x {
                best = x;
                bi = t;
            }
        }
        if zero <= best {
            if bi != c {
                cols.swap(bi, c);
                for x in 0..n {
                    a.swap(c * n + x, bi * n + x);
                }
            }
            let piv = a[d];
            for t in c + 1..n {
                a[c + t * n] /= piv;
            }
            c += 1;
        } else {
            let l = last as usize;
            rows.swap(l, c);
            for t in 0..n {
                a.swap(c + t * n, l + t * n);
            }
            last -= 1;
        }
    }
    Some((n as isize - 1 - last) as usize)
}

/// `76fa2`: the smallest 6k±1 number ≥ p that passes its trial division (a prime for the sizes used here).
fn next_prime(p: i32) -> i32 {
    let test = |x: i32| -> bool {
        if (x / 3) * 3 == x {
            return false;
        }
        let s = (x as f64).sqrt() as i32;
        let mut i2 = 5;
        if s < 5 || (x / 5) * 5 == x {
            return s < 5;
        }
        loop {
            let i1 = i2 + 2;
            if x % i1 == 0 {
                return s < i1;
            }
            i2 += 6;
            if s < i2 {
                return true;
            }
            if x % i2 == 0 {
                return false;
            }
        }
    };
    if !(0..0x773593e6).contains(&p) {
        return 0x7735940b;
    }
    if p < 2 {
        return p;
    }
    if p < 4 {
        return 3;
    }
    let mut v = ((p as f64 / 6.0).ceil() * 6.0 - 1.0) as i32;
    if v < p {
        if test(v + 2) {
            return v + 2;
        }
        v += 6;
    }
    loop {
        if v > 0x773593e4 {
            return 0x773593e5;
        }
        if test(v) {
            return v;
        }
        if test(v + 2) {
            return v + 2;
        }
        v += 6;
    }
}

/// What `036ade` adds to the dual work counter: q = singleton-phase L + U entries ((P0 + local_100) − local_110 after
/// `1da0b2`), nnzB, act = active entries after the singleton phases (at least 1), the Markowitz accumulator and its
/// terms (kind `c`/`r` singleton count, `g` general pivot (ns, Σ column counts along the pivot row, Σ row counts
/// down the pivot column, bound)), and folds of acc > 1e8 before a pivot.
#[derive(Default)]
pub struct FactorWork {
    pub q: usize,
    pub nnz_b: usize,
    pub act: usize,
    pub acc: f64,
    pub early: Vec<f64>,
    pub terms: Vec<(char, [i64; 3], f64)>,
}

/// Count buckets of the `036ade` kernel: circular doubly linked lists; insert goes after the head and becomes the
/// head, walks go head → prv (last inserted first).
struct Buckets {
    head: Vec<i64>,
    nxt: Vec<usize>,
    prv: Vec<usize>,
}

impl Buckets {
    fn new(m: usize) -> Buckets {
        Buckets { head: vec![-1; m + 1], nxt: vec![0; m], prv: vec![0; m] }
    }

    fn ins(&mut self, i: usize, k: usize) {
        let h = self.head[k];
        if h < 0 {
            self.nxt[i] = i;
            self.prv[i] = i;
        } else {
            let h = h as usize;
            self.nxt[i] = self.nxt[h];
            self.nxt[h] = i;
            self.prv[i] = h;
            self.prv[self.nxt[i]] = i;
        }
        self.head[k] = i as i64;
    }

    fn rem(&mut self, i: usize, k: usize) {
        if self.nxt[i] == i {
            self.head[k] = -1;
            return;
        }
        if self.head[k] == i as i64 {
            self.head[k] = self.prv[i] as i64;
        }
        let (n, p) = (self.nxt[i], self.prv[i]);
        self.prv[n] = p;
        self.nxt[p] = n;
    }
}

/// The sparse kernel's state the `local_70` mode starts from (live elements, their lists in reversed original
/// order, counts − 1, the empty row/column stacks and the fill estimate).
struct Mode70In<'a> {
    ev: &'a [f64],
    ec: &'a [usize],
    rl: &'a [Vec<usize>],
    rcs: &'a [i64],
    ccs: &'a [i64],
    erows: &'a mut Vec<usize>,
    ecols: &'a mut Vec<usize>,
    est: f64,
}

/// Markowitz threshold (MarkowitzTol default 2⁻⁷, spx+0x390).
pub const MARKOWITZ_TOL: f64 = 0.0078125;

impl Lu {
    /// `9e60b`: estimated nonzeros of row k of B⁻¹ by factor step (P+0x68), for the BTRAN-cost pricing weights.
    /// Each step starts at 1 and absorbs its Lᵀ row, then its U row from the last step down, as `p + q − pq/m`.
    /// Returns the costs (`min(p + 100, 0.1m + 100)`) and false when Σp exceeds m·(0.1m + 100) (cost mode 2,
    /// every cost at the cap).
    pub fn row_cost_estimate(&self) -> (Vec<f64>, bool) {
        let k = self.steps.len();
        let mf = self.head.len() as f64;
        let mut p = vec![1.0; k];
        for s in 0..k {
            for &(t, _) in &self.lrow[s] {
                p[s] = (p[s] + p[t]) - p[t] * p[s] / mf;
            }
        }
        for s in (0..k).rev() {
            for &(t, _) in &self.urow[s] {
                p[s] = (p[s] + p[t]) - p[t] * p[s] / mf;
            }
        }
        let cap = mf * 0.1 + 100.0;
        // shortcut: plain left-to-right sum; the original sums in 8 lanes (only matters at the threshold)
        let total: f64 = p.iter().sum();
        if total <= mf * cap {
            (p.iter().map(|&x| (x + 100.0).min(cap)).collect(), true)
        } else {
            (vec![cap; k], false)
        }
    }

    /// (nnz of L, nnz of U) of the factorization including the diagonals (LU+0x1e0, LU+0x1e8 after a factor).
    pub fn factor_nnz(&self) -> (usize, usize) {
        let m = self.head.len();
        (m + self.lcol.iter().map(|c| c.len()).sum::<usize>(), m + self.ucol.iter().map(|c| c.len()).sum::<usize>())
    }

    /// Entries of all Forrest–Tomlin row etas (LU+0x184).
    pub fn eta_nnz(&self) -> usize {
        self.retas.iter().map(|e| e.1.len()).sum()
    }

    /// Row eta count LU+0x180.
    pub(crate) fn n_etas(&self) -> usize {
        self.retas.len()
    }

    /// Basis positions after a refactor (5b257 LAB_001f474e): the new position of a basic variable is its factor
    /// step. Returns the old position of each new one.
    pub fn renumbered(&self) -> Vec<usize> {
        self.steps.iter().map(|s| s.1).collect()
    }

    /// Relabel basis positions: `old_of_new[p]` is the old position of new position p.
    pub fn relabel(&mut self, old_of_new: &[usize]) {
        let m = self.head.len();
        let mut new_of_old = vec![0; m];
        for (np, &op) in old_of_new.iter().enumerate() {
            new_of_old[op] = np;
        }
        self.head = old_of_new.iter().map(|&op| self.head[op]).collect();
        self.kpos = old_of_new.iter().map(|&op| self.kpos[op]).collect();
        for st in &mut self.steps {
            st.1 = new_of_old[st.1];
        }
    }

    /// Row-wise U by basis position as (column position, u) in storage order, and the prefix t (LU+0x100), for
    /// snapshots.
    pub fn u_rows(&self) -> (Vec<Vec<(usize, f64)>>, usize) {
        let rows = self.kpos.iter().map(|&k| self.urow[k].iter().map(|&(k2, u)| (self.steps[k2].1, u)).collect()).collect();
        (rows, self.prefix)
    }

    /// LU+0xe8 and the U positions whose column is empty (snapshots).
    pub fn empty_cols(&self) -> (usize, Vec<usize>) {
        (self.e8, (0..self.order.len()).filter(|&p| self.ucol[self.order[p]].is_empty()).collect())
    }

    /// Tail U columns by descending U position as (position, prefix-row positions in storage order) (snapshots).
    pub fn u_prefix_parts(&self) -> Vec<(usize, Vec<usize>)> {
        let t = self.prefix;
        self.order[t..].iter().rev().map(|&k| (self.steps[k].1, self.prefix_part(k).map(|j| self.steps[j].1).collect())).collect()
    }

    /// Steps of the prefix rows in column k in storage order (`21686f` [+0x78, +0x70)).
    fn prefix_part(&self, k: usize) -> impl Iterator<Item = usize> + '_ {
        self.ucol[k][self.umid[k]..].iter().map(|&(i, _)| self.prow[i] as usize)
    }

    pub fn complete(&self) -> bool {
        self.steps.len() == self.head.len()
    }

    /// Read-only view of a fresh factor for adapters that rebuild the original's array layout (the crossover push
    /// backend, docs/m3/primal_push_refactor_rust.md). Nothing here changes the factor.
    pub fn parts(&self) -> LuParts<'_> {
        LuParts {
            steps: &self.steps,
            prow: &self.prow,
            lcol: &self.lcol,
            lrow: &self.lrow,
            ucol: &self.ucol,
            urow: &self.urow,
            umid: &self.umid,
            prefix: self.prefix,
            e8: self.e8,
            dc: self.dc,
            f0: &self.f0,
            f8: &self.f8,
            pool: self.pool,
            sparse_mult: self.sparse_mult,
            level: self.level.as_deref(),
            llev: self.llev.as_deref(),
            lmax: self.lmax,
            n_singleton: self.n_singleton,
            s1: self.s1,
            s3: self.s3,
            s4: self.s4,
        }
    }

    pub fn factor(spx: &Spx, head: &[usize]) -> Lu {
        Lu::factor_tol(spx, head, MARKOWITZ_TOL)
    }

    /// LU+0xd8, 0xdc, 0xe0, 0xe4 and 0x100 (t) as the factor leaves them (M1-01e probe).
    pub fn block_fields(&self) -> [usize; 5] {
        [self.s1, self.dc, self.s3, self.s4, self.prefix]
    }

    /// Factor with Markowitz threshold `tol` (MarkowitzTol, raised by the numerics ladder).
    pub fn factor_tol(spx: &Spx, head: &[usize], tol: f64) -> Lu {
        let mut st = BlockState::OFF;
        Lu::factor_block(spx, head, tol, &mut st)
    }

    /// Factor with the `1da0b2` 2×2 block phase state `st` (read and updated as the original does on S).
    pub fn factor_block(spx: &Spx, head: &[usize], tol: f64, st: &mut BlockState) -> Lu {
        Lu::factor_with(spx, head, tol, st, None)
    }

    /// `factor_block` with the kernel's dense switch (`036ade` C:1526–1626, C:3394–3470, C:3968–5300) for an element
    /// pool of `cap` slots (`param_15[2]`, F+0x158), whose first fill fixes the dense array's alignment.
    pub fn factor_dense(spx: &Spx, head: &[usize], tol: f64, st: &mut BlockState, cap: i64) -> Lu {
        Lu::factor_with(spx, head, tol, st, Some(cap))
    }

    fn factor_with(spx: &Spx, head: &[usize], tol: f64, st: &mut BlockState, cap: Option<i64>) -> Lu {
        let (m, n) = (spx.m, spx.n);
        let mut rc = vec![0usize; m];
        for &j in head {
            if j < n {
                for (i, _) in spx.col(j) {
                    rc[i] += 1;
                }
            } else {
                rc[j - n] += 1 + m; // slack-basic row marker
            }
        }
        // rows of the basic structurals (rows without a basic slack), position order, as one flat array
        let mut rstart = vec![0usize; m + 1];
        for i in 0..m {
            rstart[i + 1] = rstart[i] + if rc[i] <= m { rc[i] } else { 0 };
        }
        let mut rent = vec![(0usize, 0.0f64); rstart[m]];
        let mut fill = rstart.clone();
        for (pos, &j) in head.iter().enumerate() {
            if j < n {
                for (i, a) in spx.col(j) {
                    if rc[i] <= m {
                        rent[fill[i]] = (pos, a);
                        fill[i] += 1;
                    }
                }
            }
        }
        let mut lu = Lu {
            steps: Vec::with_capacity(m),
            prow: vec![-1; m],
            head: head.to_vec(),
            kpos: Vec::new(),
            lcol: Vec::new(),
            lrow: Vec::new(),
            ucol: Vec::new(),
            urow: Vec::new(),
            umid: Vec::new(),
            order: Vec::new(),
            upos: Vec::new(),
            prefix: 0,
            e8: 0,
            dc: 0,
            dsum: 3.0,
            d108: 1.5,
            f0: Vec::new(),
            f8: Vec::new(),
            retas: Vec::new(),
            reta_odd: Vec::new(),
            pool: 0,
            rcap: Vec::new(),
            sparse_mult: 2,
            level: None,
            llev: None,
            lmax: -1,
            ltmax: -1,
            ltail: 0,
            general: false,
            n_updates: 0,
            n_singleton: 0,
            s1: 0,
            s3: 0,
            s4: 0,
            blk: Vec::new(),
            kernel_u: Vec::new(),
            singular: Vec::new(),
            fw: FactorWork::default(),
            dense_cap: cap,
            dense: None,
            fpool: None,
            pool_trace: Vec::new(),
            mode70: None,
            short: None,
        };
        // 036ade C:296–330: basis entries, a slack counts 1
        lu.fw.nnz_b = head.iter().map(|&j| if j < n { spx.len[j] as usize } else { 1 }).sum();
        let mut ppos = vec![-1i64; m];
        let pivot = |lu: &mut Lu, ppos: &mut Vec<i64>, r: usize, pos: usize, d: f64| {
            let k = lu.steps.len() as i64;
            lu.prow[r] = k;
            ppos[pos] = k;
            lu.steps.push((r, pos, d));
        };
        // row singletons, LIFO stack seeded in ascending row order
        let mut stack: Vec<usize> = (0..m).filter(|&i| rc[i] == 1).collect();
        while let Some(r) = stack.pop() {
            if rc[r] == 0 {
                continue;
            }
            rc[r] = 0;
            let &(pos, d) = rent[rstart[r]..rstart[r + 1]].iter().find(|(p, _)| ppos[*p] == -1).expect("row singleton has a free column");
            pivot(&mut lu, &mut ppos, r, pos, d);
            for (i, _) in spx.col(head[pos]) {
                if lu.prow[i] == -1 {
                    rc[i] -= 1;
                    if rc[i] == 1 {
                        stack.push(i);
                    }
                }
            }
        }

        let s1 = lu.steps.len();
        lu.s1 = s1;
        // slacks of rows without structural entries
        for (pos, &j) in head.iter().enumerate() {
            if j >= n && rc[j - n] == m + 1 {
                rc[j - n] = 1;
                pivot(&mut lu, &mut ppos, j - n, pos, 1.0);
            }
        }
        lu.dc = lu.steps.len();
        // remaining column singletons and slacks, in basis order
        for (pos, &j) in head.iter().enumerate() {
            if ppos[pos] >= 0 {
                continue;
            }
            if j < n {
                if spx.len[j] == 1 {
                    let (i, a) = spx.col(j).next().unwrap();
                    if rc[i] <= m && lu.prow[i] < 0 {
                        pivot(&mut lu, &mut ppos, i, pos, a);
                    }
                }
            } else {
                let i = j - n;
                rc[i] -= m;
                pivot(&mut lu, &mut ppos, i, pos, 1.0);
            }
        }
        // phase 4 (1da0b2 L530–690): column singletons of the active part from a stack seeded in ascending position
        // order; skipped when phases 2–3 pivoted nothing

        let s3 = lu.steps.len();
        lu.s3 = s3;
        // C:470–560: the active row copy (rows by index, their entries in the unpivoted columns in ascending
        // position with values) and the column counts over the active rows
        let mut cc = vec![0i64; m];
        let mut cstart = vec![0usize; m + 1];
        for c in 0..m {
            if ppos[c] == -1 {
                for (i, _) in spx.col(head[c]) {
                    if lu.prow[i] < 0 {
                        cc[c] += 1;
                        cstart[i + 1] += 1;
                    }
                }
            }
        }
        for i in 0..m {
            cstart[i + 1] += cstart[i];
        }
        let mut rcopy = vec![0usize; cstart[m]];
        let mut rval = vec![0.0f64; cstart[m]];
        let mut fill = cstart.clone();
        for c in 0..m {
            if ppos[c] == -1 {
                for (i, a) in spx.col(head[c]) {
                    if lu.prow[i] < 0 {
                        rcopy[fill[i]] = c;
                        rval[fill[i]] = a;
                        fill[i] += 1;
                    }
                }
            }
        }
        let mut stack: Vec<usize> = if s3 != s1 { (0..m).filter(|&c| cc[c] == 1).collect() } else { Vec::new() };
        while let Some(c) = stack.pop() {
            if cc[c] == 0 {
                continue;
            }
            let (r, d) = spx.col(head[c]).find(|&(i, _)| lu.prow[i] == -1).expect("active entry");
            pivot(&mut lu, &mut ppos, r, c, d);
            for &c2 in &rcopy[cstart[r]..cstart[r + 1]] {
                if ppos[c2] == -1 {
                    cc[c2] -= 1;
                    if cc[c2] == 1 {
                        stack.push(c2);
                    }
                }
            }
        }
        lu.s4 = lu.steps.len();
        // C:681–1510: 2×2 blocks, skipped with the phases 2–4 when phases 2–3 pivoted nothing (C:440)
        if s3 != s1 {
            let rcnt: Vec<i64> = rc.iter().map(|&x| x as i64).collect();
            lu.block_phase(spx, &mut ppos, rcnt, cc, cstart, rcopy, rval, st);
        }

        let ns = lu.steps.len();
        lu.n_singleton = ns;
        // 036ade C:543–585: entries of the unpivoted columns in unpivoted rows (any value), slacks 1, at least 1
        lu.fw.act = (0..m)
            .filter(|&p| ppos[p] < 0)
            .map(|p| if head[p] < n { spx.col(head[p]).filter(|&(i, _)| lu.prow[i] < 0).count() } else { 1 })
            .sum::<usize>()
            .max(1);
        lu.lcol = vec![Vec::new(); m];
        lu.ucol = vec![Vec::new(); m];
        if ns < m && !lu.kernel(spx, &mut ppos, tol) {
            return lu; // singular: incomplete
        }

        // L and U of the singleton steps come straight from the columns (their pivot rows have no other active
        // entries, so the kernel values are unaffected); kernel columns keep their entries in singleton rows as U
        let s4 = lu.s4;
        for k in 0..m {
            let (_, pos, d) = lu.steps[k];
            for (i, a) in spx.col(head[pos]) {
                let ki = lu.prow[i] as usize;
                if k < s4 && ki > k {
                    lu.lcol[k].push((i, a / d));
                } else if ki < k && ki < s4 {
                    lu.ucol[k].push((i, a));
                }
            }
        }
        // the block phase's steps keep the L columns and U rows it built (C:806–1495)
        for (b, (l, _)) in lu.blk.iter().enumerate() {
            lu.lcol[s4 + b] = l.clone();
        }
        // row-wise U in the original's storage order (LU+0xb0/0xd0/0xc8): singleton rows by column position
        // (1da0b2 C:530–560, C:657–677), kernel rows in their list order (036ade C:600–690, C:2366–2393)
        lu.urow = vec![Vec::new(); m];
        for pos in 0..m {
            let k = ppos[pos] as usize;
            for (i, a) in spx.col(head[pos]) {
                let ki = lu.prow[i] as usize;
                if ki < k && ki < s4 {
                    lu.urow[ki].push((k, a));
                }
            }
        }
        for (b, (_, u)) in std::mem::take(&mut lu.blk).into_iter().enumerate() {
            let (k, r) = (s4 + b, lu.steps[s4 + b].0);
            for (p2, v) in u {
                lu.ucol[ppos[p2] as usize].push((r, v));
                lu.urow[k].push((ppos[p2] as usize, v));
            }
        }
        for (k, ku) in std::mem::take(&mut lu.kernel_u).into_iter().enumerate() {
            let r = lu.steps[k].0;
            for (p2, u) in ku {
                lu.ucol[ppos[p2] as usize].push((r, u));
                lu.urow[k].push((ppos[p2] as usize, u));
            }
        }

        // 1da0b2 emits the singleton steps' L columns and U rows (including entries in kernel columns)
        lu.fw.q = (0..ns).map(|k| lu.lcol[k].len() + lu.urow[k].len()).sum();
        lu.finish();
        lu
    }

    /// `036ade` work stores in order: 0xd4ded, 0xd5300, early acc folds 0xd5e39, 0xd85e6, 0xdc7b2 and with L levels
    /// 0xdcb7b (C:535–541, C:762–764, C:813–822, C:1531–1533, C:5741–5754, C:5934–5936).
    pub fn factor_work(&self) -> Vec<(u32, f64)> {
        use crate::work::X148;
        let nb = self.head.len() as f64;
        let fw = &self.fw;
        let (l, u) = self.factor_nnz();
        let (l, u) = (l - self.head.len(), u - self.head.len());
        let mut v = vec![
            (0xd4ded, ((nb * 5.0 + 100.0) + fw.q as f64 * 3.0) * X148),
            (0xd5300, ((nb * 10.0 + fw.nnz_b as f64 * 3.0) + fw.act as f64 * 7.0) * X148),
        ];
        v.extend(fw.early.iter().map(|&a| (0xd5e39, a * X148)));
        v.push((0xd85e6, fw.acc * X148));
        let big = (l + u) as f64 * 24.0;
        let f = if big <= 2_000_000.0 { 1.0 } else if big < 4_000_000.0 { ((big - 2_000_000.0) * 9.0) / 2_000_000.0 + 1.0 } else { 10.0 };
        v.push((0xdc7b2, ((((nb + nb) + l as f64) + u as f64) * 2.0 * f) * X148));
        if self.level.is_some() {
            v.push((0xdcb7b, ((l + self.head.len()) as f64 * 2.0) * X148));
        }
        v
    }

    /// `05b257` after a refactor (C:2545–2729) with `9e60b` (C:246–251, C:345–347) when the cost mode is on:
    /// the stores the original skips carry their partial sum to the next one.
    pub fn refactor_work(&self, w: &mut crate::work::Work, cost_mode: i32) {
        use crate::work::X148;
        let (m, f1f8) = (self.head.len(), X148);
        let mf = m as f64;
        w.add(0xf48ae, (mf * 3.0) * f1f8);
        if cost_mode >= 1 {
            // shortcut: 05b257 turns the cost mode off instead when env+0xe8 = 4, env+0x2504 = −1 and iterations > 2m
            let unnz: usize = self.urow.iter().map(Vec::len).sum();
            w.add(if cost_mode == 2 { 0x1329e6 } else { 0x1327dc }, unnz as f64 * f1f8);
            w.add(0x1329c1, (f1f8 * 3.0) * mf);
        }
        let a = (mf * 9.0) * X148;
        if self.dc > 0 { w.add(0xf4f9b, a) } else { w.carry(a) }
        let (i, t) = (self.dc, self.prefix);
        let a = X148 * i as f64;
        if i < t { w.add(0xf4ffd, a) } else { w.carry(a) }
        let d = i.max(t);
        let a = (d as f64 + d as f64) * X148;
        if d < m {
            w.add(0xf507a, a);
        } else if t + 1 < m {
            w.add(0xf515b, a);
        } else {
            w.carry(a);
        }
        w.add(0xf51af, m.saturating_sub(self.e8 + 1) as f64 * X148);
    }

    /// `1da0b2` C:681–1510: the 2×2 block phase after the column singletons of phase 4. It runs when 4 ≤ active ≤
    /// 5·10⁸ and S+0x4e4 ≥ 0, and with S+0x4e4 < 2e9 only for active ≥ 50 (C:681–684). Phase A pairs active rows of
    /// count 2 on the same two columns (`1d9af1`, chained hash of `76fa2(2·active)` buckets, pairs taken LIFO), pivots
    /// the row with the larger |a| in the first column and then the other row on the Schur value, accepted when
    /// |s| ≥ 1e-10; row singletons it leaves follow at once. Phase B does the same for active columns of count 2 on
    /// the same two rows, with column singletons after each block. The blocks change no other active entry, so the
    /// kernel sees the original values. `rcnt`/`cc`: active row/column counts (local_e0/local_108), `rbeg`/`rpos`/
    /// `rval`: the active row copy (local_b0/98/a0), which `1d9af1` compacts in place.
    /// shortcut: the pool checks that return −1 (C:765, 806, 977, 1062, 1338; `036ade` then retries) are not ported.
    #[allow(clippy::too_many_arguments)]
    fn block_phase(&mut self, spx: &Spx, ppos: &mut [i64], mut rcnt: Vec<i64>, mut cc: Vec<i64>, rbeg: Vec<usize>,
                   mut rpos: Vec<usize>, mut rval: Vec<f64>, st: &mut BlockState) {
        let m = self.head.len();
        let s4 = self.steps.len();
        let active = m as i64 - s4 as i64;
        if (active - 4) as u32 > 0x1dcd64fc || st.x4e4 < 0 || (st.x4e4 < 2_000_000_000 && active < 50) {
            return;
        }
        st.x4e0 += 1;
        let prime = next_prime((active * 2) as i32) as i64;
        let mi = m as i64;
        let head = self.head.clone();
        let pivot = |lu: &mut Lu, ppos: &mut [i64], r: usize, pos: usize, d: f64, l: Vec<(usize, f64)>, u: Vec<(usize, f64)>| {
            let k = lu.steps.len() as i64;
            lu.prow[r] = k;
            ppos[pos] = k;
            lu.steps.push((r, pos, d));
            lu.blk.push((l, u));
        };
        // the chained hash of `1d9af1` and C:1040–1100: items (line, key), bucket heads, next links, pair list
        struct Hash {
            line: Vec<usize>,
            key: Vec<i64>,
            next: Vec<i64>,
            head: Vec<i64>,
            pairs: Vec<usize>,
        }
        impl Hash {
            fn new(n: usize) -> Hash {
                Hash { line: Vec::new(), key: Vec::new(), next: Vec::new(), head: vec![-1; n], pairs: Vec::new() }
            }
            fn insert(&mut self, line: usize, key: i64) {
                let item = self.line.len();
                self.line.push(line);
                self.key.push(key);
                self.next.push(-1);
                let b = (key % self.head.len() as i64) as usize;
                let h = self.head[b];
                if h == -1 {
                    self.head[b] = item as i64;
                    self.next[item] = -1;
                    return;
                }
                let (mut cur, mut prev) = (h, -1i64);
                loop {
                    if h >= 0 && self.key[cur as usize] == key {
                        self.pairs.push(cur as usize);
                        self.pairs.push(item);
                        let nx = self.next[cur as usize];
                        if prev == -1 { self.head[b] = nx } else { self.next[prev as usize] = nx }
                        return;
                    }
                    let nx = if h >= 0 { self.next[cur as usize] } else { -1 };
                    if nx < 0 {
                        self.next[item] = h;
                        self.head[b] = item as i64;
                        return;
                    }
                    prev = cur;
                    cur = nx;
                }
            }
            fn pop(&mut self) -> Option<(usize, usize)> {
                let b = self.pairs.pop()?;
                let a = self.pairs.pop()?;
                Some((a, b))
            }
        }
        // `1d9af1`: move the row's active entries to the front (positions swap, values copy one way), order the two
        // active columns by (count, index), key = first + m·second
        let hash_row = |h: &mut Hash, r: usize, rpos: &mut [usize], rval: &mut [f64], cc: &[i64]| {
            let (lo, hi) = (rbeg[r], rbeg[r + 1]);
            let mut w = lo;
            for j in lo..hi {
                if cc[rpos[j]] > 0 {
                    if w < j {
                        rpos.swap(w, j);
                        rval[w] = rval[j];
                    }
                    w += 1;
                }
            }
            let (a, b) = (rpos[w - 2], rpos[w - 1]);
            let key = if cc[b] < cc[a] || (cc[b] == cc[a] && b < a) {
                rpos[w - 2] = b;
                rpos[w - 1] = a;
                rval.swap(w - 2, w - 1);
                b as i64 + mi * a as i64
            } else {
                a as i64 + mi * b as i64
            };
            h.insert(r, key);
        };
        // C:1043–1066 inline: the column's first two active rows ordered by (count, index), key = first + m·second
        let hash_col = |h: &mut Hash, c: usize, rcnt: &[i64]| {
            let (mut first, mut second) = (-1i64, -1i64);
            for (i, _) in spx.col(head[c]) {
                if rcnt[i] > 0 {
                    if first == -1 {
                        first = i as i64;
                    } else {
                        second = i as i64;
                        break;
                    }
                }
            }
            let (c2, c1) = (rcnt[second as usize], rcnt[first as usize]);
            let (a, b) = if c2 < c1 || (c2 == c1 && second < first) { (second, first) } else { (first, second) };
            h.insert(c, a + b * mi);
        };

        // phase A (C:686–1031)
        let mut h = Hash::new(prime as usize);
        for idx in 0..m {
            if ppos[idx] != -1 {
                cc[idx] = -1;
            }
            if self.prow[idx] == -1 {
                if rcnt[idx] == 2 {
                    hash_row(&mut h, idx, &mut rpos, &mut rval, &cc);
                }
            } else {
                rcnt[idx] = -1;
            }
        }
        let mut w = vec![0.0f64; m];
        let mut stack: Vec<usize> = Vec::new();
        while let Some((i0, i1)) = h.pop() {
            let (r6, r10) = (h.line[i0], h.line[i1]);
            if !(rcnt[r6] > 1 && rcnt[r10] > 1) {
                continue;
            }
            let (p, q) = (rbeg[r6], rbeg[r10]);
            let (c1, c2) = (rpos[p], rpos[p + 1]);
            let (v39, mut v42, v41, mut v43) = (rval[p], rval[q], rval[p + 1], rval[q + 1]);
            let (mut v38, mut v40, mut piv, mut oth) = (v41, v39, r6, r10);
            if v39.abs() < v42.abs() {
                v38 = v43;
                v40 = v42;
                v42 = v39;
                v43 = v41;
                piv = r10;
                oth = r6;
            }
            let s = v43 - (v42 * v38) / v40;
            if s.abs() < 1e-10 {
                continue;
            }
            rcnt[piv] = 0;
            cc[c1] = 0;
            let mut l = Vec::new();
            for (i, a) in spx.col(head[c1]) {
                if rcnt[i] > 0 {
                    w[i] = (-a * v38) / v40;
                    l.push((i, a / v40));
                    rcnt[i] -= 1;
                    if rcnt[i] == 1 {
                        stack.push(i);
                    } else if rcnt[i] == 2 {
                        hash_row(&mut h, i, &mut rpos, &mut rval, &cc);
                    }
                }
            }
            let lk = l.clone();
            pivot(self, ppos, piv, c1, v40, l, vec![(c2, v38)]);
            rcnt[oth] = 0;
            cc[c2] = 0;
            w[oth] = 0.0;
            let mut l = Vec::new();
            for (i, a) in spx.col(head[c2]) {
                if rcnt[i] > 0 {
                    let v = (a + w[i]) / s;
                    w[i] = 0.0;
                    if ZERO < v.abs() {
                        l.push((i, v));
                    }
                    rcnt[i] -= 1;
                    if rcnt[i] == 1 {
                        stack.push(i);
                    } else if rcnt[i] == 2 {
                        hash_row(&mut h, i, &mut rpos, &mut rval, &cc);
                    }
                }
            }
            for &(i, _) in &lk {
                if ZERO < (w[i] / s).abs() {
                    l.push((i, w[i] / s));
                }
                w[i] = 0.0;
            }
            pivot(self, ppos, oth, c2, s, l, Vec::new());
            // row singletons (C:959–1027)
            while let Some(r) = stack.pop() {
                if rcnt[r] <= 0 {
                    continue;
                }
                let mut j = rbeg[r];
                while j < rbeg[r + 1] && cc[rpos[j]] < 1 {
                    j += 1;
                }
                let (c, d) = (rpos[j], rval[j]);
                rcnt[r] = 0;
                cc[c] = 0;
                let mut l = Vec::new();
                for (i, a) in spx.col(head[c]) {
                    if rcnt[i] > 0 {
                        l.push((i, a / d));
                        rcnt[i] -= 1;
                        if rcnt[i] == 1 {
                            stack.push(i);
                        } else if rcnt[i] == 2 {
                            hash_row(&mut h, i, &mut rpos, &mut rval, &cc);
                        }
                    }
                }
                pivot(self, ppos, r, c, d, l, Vec::new());
            }
        }

        // phase B (C:1032–1493)
        let mut h = Hash::new(prime as usize);
        for c in 0..m {
            if cc[c] == 2 {
                hash_col(&mut h, c, &rcnt);
            }
        }
        let mut w = vec![0.0f64; m];
        let mut stack: Vec<usize> = Vec::new();
        while let Some((i0, i1)) = h.pop() {
            let (ca, cb, key) = (h.line[i0], h.line[i1], h.key[i0]);
            if !(cc[ca] > 1 && cc[cb] > 1) {
                continue;
            }
            let (ra, rb) = ((key % mi) as usize, (key / mi) as usize);
            let (mut va, mut vb) = (0.0f64, 0.0f64);
            let mut uk = Vec::new();
            for j in rbeg[ra]..rbeg[ra + 1] {
                let c = rpos[j];
                if cc[c] > 0 {
                    if c == ca {
                        va = rval[j];
                    } else if c == cb {
                        vb = rval[j];
                    } else {
                        uk.push((c, rval[j]));
                    }
                }
            }
            let (mut vp, mut vo, mut piv, mut oth) = (va, vb, ca, cb);
            if va.abs() < vb.abs() {
                vo = va;
                piv = cb;
                vp = vb;
                oth = ca;
            }
            uk.push((oth, vo));
            let (mut bp, mut bo) = (0.0f64, 0.0f64);
            let mut uk1 = Vec::new();
            for j in rbeg[rb]..rbeg[rb + 1] {
                let c = rpos[j];
                if cc[c] > 0 {
                    if c == piv {
                        bp = rval[j];
                    } else if c == oth {
                        bo = rval[j];
                    } else {
                        uk1.push((c, rval[j]));
                    }
                }
            }
            let s = bo - (vo * bp) / vp;
            if s.abs() < 1e-10 {
                continue;
            }
            rcnt[ra] = 0;
            cc[piv] = 0;
            for &(c, u) in &uk[..uk.len() - 1] {
                w[c] = (-bp * u) / vp;
                cc[c] -= 1;
                if cc[c] == 1 {
                    stack.push(c);
                } else if cc[c] == 2 {
                    hash_col(&mut h, c, &rcnt);
                }
            }
            let ua: Vec<usize> = uk[..uk.len() - 1].iter().map(|e| e.0).collect();
            pivot(self, ppos, ra, piv, vp, vec![(rb, bp / vp)], uk);
            rcnt[rb] = 0;
            cc[oth] = 0;
            let mut u = Vec::new();
            for (c, x) in uk1 {
                cc[c] -= 1;
                if cc[c] == 1 {
                    stack.push(c);
                } else if cc[c] == 2 {
                    hash_col(&mut h, c, &rcnt);
                }
                if ZERO < (x + w[c]).abs() {
                    u.push((c, x + w[c]));
                }
                w[c] = 0.0;
            }
            for c in ua {
                if ZERO < w[c].abs() {
                    u.push((c, w[c]));
                }
                w[c] = 0.0;
            }
            pivot(self, ppos, rb, oth, s, Vec::new(), u);
            // column singletons (C:1384–1493)
            while let Some(c) = stack.pop() {
                if cc[c] <= 0 {
                    continue;
                }
                let (r, d) = spx.col(head[c]).find(|&(i, _)| rcnt[i] >= 1).expect("active row");
                cc[c] = 0;
                rcnt[r] = 0;
                let mut u = Vec::new();
                for j in rbeg[r]..rbeg[r + 1] {
                    let c2 = rpos[j];
                    if cc[c2] > 0 {
                        u.push((c2, rval[j]));
                        cc[c2] -= 1;
                        if cc[c2] == 1 {
                            stack.push(c2);
                        } else if cc[c2] == 2 {
                            hash_col(&mut h, c2, &rcnt);
                        }
                    }
                }
                pivot(self, ppos, r, c, d, Vec::new(), u);
            }
        }

        // C:1494–1510
        if st.x4e4 < 2_000_000_000 {
            if self.steps.len() as i64 - s4 as i64 > 19 {
                st.x4e4 += 1;
            }
            if st.x2c0 > 20 {
                st.x4e4 = if st.x4e4 * 2 < st.x4e0 { -1 } else { 2_000_000_000 };
            }
        }
    }

    /// Sparse Markowitz kernel of `036ade` (L546–2570, docs/m1/m1_notes.md §6.7) on the rows and positions the singleton
    /// phases leave: column singletons, then row singletons, then a Markowitz search over count buckets (LIFO
    /// circular lists, rotated heads), threshold `tol·max < |a|` on the searched line's max, ties to the larger
    /// |a|/max; multipliers a/v, updates a − l·u and fill −(l·u) dropped under ZERO, fill prepended to its lists.
    /// Empty rows and columns pair up in discovery order as the singular tail (`self.singular`).
    /// The 2×2 block phase runs before it in `block_phase` (only with a `BlockState` that allows it).
    /// With `dense_cap` the kernel takes the dense switch (C:1526–1626: estimated fill > 0.36·n², 4m < n², n > 50,
    /// m + n² < 2e9, then the recounted fill > 0.325·n²) into `dense_lu`; without it the M1 path stays sparse.
    /// shortcut: no `local_70` dense-array Markowitz mode (C:1762: 201 ≤ n ≤ 9999 with a costly search), no MKL
    /// `dgetrf` (n ≥ 1001 with n² > m), no dense switch with empty columns, no 1e99 pivot cap.
    fn kernel(&mut self, spx: &Spx, ppos: &mut [i64], tol: f64) -> bool {
        let m = self.head.len();
        // element pool: (value, row, position; position usize::MAX = dead). Lists are stored reversed so that
        // pushing prepends; iterate with .rev() for the original order.
        let mut ev: Vec<f64> = Vec::new();
        let mut er: Vec<usize> = Vec::new();
        let mut ec: Vec<usize> = Vec::new();
        const DEAD: usize = usize::MAX;
        let mut rl: Vec<Vec<usize>> = vec![Vec::new(); m];
        let mut cl: Vec<Vec<usize>> = vec![Vec::new(); m];
        let mut rcs = vec![-1i64; m]; // count − 1
        let mut ccs = vec![-1i64; m];
        for pos in 0..m {
            if ppos[pos] >= 0 {
                continue;
            }
            for (i, a) in spx.col(self.head[pos]) {
                if self.prow[i] < 0 && ZERO <= a.abs() {
                    let e = ev.len();
                    ev.push(a);
                    er.push(i);
                    ec.push(pos);
                    rl[i].push(e);
                    cl[pos].push(e);
                    rcs[i] += 1;
                    ccs[pos] += 1;
                }
            }
        }
        let mut rb = Buckets::new(m);
        let mut cb = Buckets::new(m);
        let (mut erows, mut ecols) = (Vec::new(), Vec::new());
        for i in 0..m {
            if self.prow[i] < 0 {
                if rcs[i] < 0 {
                    erows.push(i);
                } else {
                    rb.ins(i, rcs[i] as usize);
                }
            }
            if ppos[i] < 0 {
                if ccs[i] < 0 {
                    ecols.push(i);
                } else {
                    cb.ins(i, ccs[i] as usize);
                }
            }
        }
        if let Some(pcap) = self.dense_cap {
            if !self.pool_start(spx, ppos, pcap, ev.len() as i64) {
                return false;
            }
        }
        // a finished line: its live elements in list order (the list is taken, nothing else walks it again)
        let take_alive = |l: &mut Vec<usize>, ec: &Vec<usize>| -> Vec<usize> {
            let mut v = std::mem::take(l);
            v.retain(|&e| ec[e] != DEAD);
            v.reverse();
            v
        };
        let first_alive = |l: &Vec<usize>, ec: &Vec<usize>| -> usize { *l.iter().rev().find(|&&e| ec[e] != DEAD).expect("live") };
        self.kernel_u = vec![Vec::new(); m];
        let mut w = vec![0.0f64; m];
        let mut seen = vec![false; m];
        let pivot = |lu: &mut Lu, ppos: &mut [i64], r: usize, pos: usize, d: f64| {
            let k = lu.steps.len() as i64;
            lu.prow[r] = k;
            ppos[pos] = k;
            lu.steps.push((r, pos, d));
        };
        let mut acc = 0.0f64;
        // C:296–329 / C:2023: the fill estimate local_128
        let mut est = self.fw.nnz_b as f64;
        loop {
            if 100000000.0 < acc {
                self.fw.early.push(acc);
                acc = 0.0;
            }
            if cb.head[0] >= 0 {
                // column singleton (L1003–1140): no threshold
                let c = cb.head[0] as usize;
                cb.rem(c, 0);
                let e = first_alive(&cl[c], &ec);
                let r = er[e];
                acc += (rcs[r] as f64 + 1.0) * 10.0;
                self.fw.terms.push(('c', [rcs[r], 0, 0], 0.0));
                rb.rem(r, rcs[r] as usize);
                ec[e] = DEAD;
                if !self.pool_u(rcs[r]) {
                    return false;
                }
                pivot(self, ppos, r, c, ev[e]);
                let k = self.steps.len() - 1;
                for f in take_alive(&mut rl[r], &ec) {
                    let j = ec[f];
                    self.pool_put_u();
                    self.kernel_u[k].push((j, ev[f]));
                    ec[f] = DEAD;
                    cb.rem(j, ccs[j] as usize);
                    ccs[j] -= 1;
                    if ccs[j] < 0 { ecols.push(j) } else { cb.ins(j, ccs[j] as usize) }
                }
                rcs[r] = -1;
                ccs[c] = -1;
            } else if rb.head[0] >= 0 {
                // row singleton (L821–1002): no threshold
                let r = rb.head[0] as usize;
                rb.rem(r, 0);
                let e = first_alive(&rl[r], &ec);
                let c = ec[e];
                acc += (ccs[c] as f64 + 1.0) * 10.0;
                self.fw.terms.push(('r', [ccs[c], 0, 0], 0.0));
                cb.rem(c, ccs[c] as usize);
                let d = ev[e];
                ec[e] = DEAD;
                if !self.pool_l(ccs[c]) {
                    return false;
                }
                pivot(self, ppos, r, c, d);
                let k = self.steps.len() - 1;
                for f in take_alive(&mut cl[c], &ec) {
                    let i = er[f];
                    self.pool_put_l();
                    self.lcol[k].push((i, ev[f] / d));
                    ec[f] = DEAD;
                    rb.rem(i, rcs[i] as usize);
                    rcs[i] -= 1;
                    if rcs[i] < 0 { erows.push(i) } else { rb.ins(i, rcs[i] as usize) }
                }
                rcs[r] = -1;
                ccs[c] = -1;
            } else {
                let nact = (m - self.steps.len()) - ecols.len();
                if nact == 0 {
                    self.pool_event("end", 0, 0);
                    break;
                }
                if let Some(cap) = self.dense_cap {
                    // C:1526–1626 (asm 0xd6695): the gate, then the exact count Σ(count) of the active columns
                    let (n2, mf) = (nact as f64 * nact as f64, m as f64);
                    if n2 * 0.36 < est && mf * 4.0 < n2 && nact > 50 && mf + n2 < 2000000000.0 {
                        est = ccs.iter().filter(|&&c| c >= 0).map(|&c| (c + 1) as f64).sum();
                        if n2 * 0.325 < est {
                            assert!(ecols.is_empty(), "036ade dense switch with empty columns (local_f0 > 0): not ported");
                            if !self.pool_dense() {
                                return false;
                            }
                            self.dense_switch(ppos, nact, cap, (&ev, &ec), &rl, &cb, &rcs);
                            break;
                        }
                    }
                }
                let cidx = (1..m).find(|&k| cb.head[k] >= 0).unwrap_or(m);
                let ridx = (1..m).find(|&k| rb.head[k] >= 0).unwrap_or(m);
                // M3-04c-21j: C:1762 (asm 0xd6a82–0xd6ab6): 201 ≤ n ≤ 9999, cidx·ridx > 0.1·n and 0.25·n² > the
                // fill estimate take the local_70 mode for the rest of the factor
                if self.dense_cap.is_some() {
                    let nf = nact as f64;
                    if ((nact as i64 - 201) as u32) <= 0x2646 && nf * 0.1 < cidx as f64 * ridx as f64 && est < nf * nf * 0.25 {
                        let mut st = Mode70In { ev: &ev, ec: &ec, rl: &rl, rcs: &rcs, ccs: &ccs, erows: &mut erows, ecols: &mut ecols, est };
                        if !self.mode70(ppos, tol, &mut st) {
                            return false;
                        }
                        break;
                    }
                }
                let k0 = cidx.min(ridx);
                if k0 >= m {
                    break; // 0x2732
                }
                let limit = ((k0 as f64 * 0.5) as usize).max(4).min(nact);
                let mut best: Option<usize> = None;
                let (mut bcost, mut bratio, mut ns) = (m as f64 * m as f64, 0.0f64, 0usize);
                let mut bound = cidx as f64 * ridx as f64;
                let mut k = k0;
                'search: while k < m {
                    let fk = k as f64;
                    bound = bound.max(fk * fk);
                    for side in 0..2 {
                        let bk = if side == 0 { &mut cb } else { &mut rb };
                        let h = bk.head[k];
                        if h < 0 {
                            if bcost <= bound {
                                break 'search;
                            }
                            continue;
                        }
                        let h = h as usize;
                        let mut x = h;
                        loop {
                            let line = if side == 0 { &mut cl[x] } else { &mut rl[x] };
                            line.retain(|&e| ec[e] != DEAD); // the original compacts while it walks
                            let line = &*line;
                            let mx = line.iter().rev().fold(0.0f64, |a, &f| if ev[f].abs() <= a { a } else { ev[f].abs() });
                            for &f in line.iter().rev() {
                                let cost = if side == 0 { rcs[er[f]] as f64 * fk } else { ccs[ec[f]] as f64 * fk };
                                let a = ev[f].abs();
                                if (cost < bcost && tol * mx < a) || (cost == bcost && bratio * mx < a) {
                                    best = Some(f);
                                    bcost = cost;
                                    bratio = a / mx;
                                }
                            }
                            ns += 1;
                            if ns < limit && bound < bcost {
                                x = bk.prv[x];
                                if x == h {
                                    break;
                                }
                                continue;
                            }
                            // rotate the bucket head (asm debdb / d8380)
                            let on = best.is_some_and(|f| if side == 0 { ec[f] == x } else { er[f] == x });
                            bk.head[k] = if on { x as i64 } else { bk.prv[x] as i64 };
                            break 'search;
                        }
                        if bcost <= bound {
                            break 'search;
                        }
                    }
                    k += 1;
                }
                let Some(best) = best else { break };
                let (r, c, v) = (er[best], ec[best], ev[best]);
                est += (rcs[r] as f64 - 1.0) * (ccs[c] as f64 - 1.0) - 2.0;
                cb.rem(c, ccs[c] as usize);
                rb.rem(r, rcs[r] as usize);
                ec[best] = DEAD;
                // asm 0xd7157: ecx = column counts along the pivot row, r10d = row counts down the pivot column
                // (other live entries, counts as in the search); the smaller one enters
                let rsum: i64 = rl[r].iter().filter(|&&f| ec[f] != DEAD).map(|&f| ccs[ec[f]]).sum();
                let csum: i64 = cl[c].iter().filter(|&&f| ec[f] != DEAD).map(|&f| rcs[er[f]]).sum();
                acc = (acc + (ns as f64 * 6.0) * bound) + rsum.min(csum) as f64 * 10.0;
                self.fw.terms.push(('g', [ns as i64, rsum, csum], bound));
                if !self.pool_l(ccs[c]) || !self.pool_u(rcs[r]) {
                    return false;
                }
                pivot(self, ppos, r, c, v);
                let sk = self.steps.len() - 1;
                let mut u_row = Vec::new();
                for f in take_alive(&mut rl[r], &ec) {
                    let j = ec[f];
                    self.pool_put_u();
                    u_row.push((j, ev[f]));
                    w[j] = ev[f];
                    ec[f] = DEAD;
                    cb.rem(j, ccs[j] as usize);
                    ccs[j] -= 1;
                }
                for f in take_alive(&mut cl[c], &ec) {
                    let i = er[f];
                    let l = ev[f] / v;
                    self.pool_put_l();
                    self.lcol[sk].push((i, l));
                    ec[f] = DEAD;
                    rb.rem(i, rcs[i] as usize);
                    rcs[i] -= 1;
                    rl[i].retain(|&e| ec[e] != DEAD);
                    for t in (0..rl[i].len()).rev() {
                        let g = rl[i][t];
                        let j = ec[g];
                        if w[j] != 0.0 {
                            seen[j] = true;
                            ev[g] -= l * w[j];
                            if ev[g].is_nan() || ev[g].abs() < ZERO {
                                ec[g] = DEAD;
                                rcs[i] -= 1;
                                ccs[j] -= 1;
                            }
                        }
                    }
                    for &(j, u) in &u_row {
                        if !seen[j] {
                            let p = -(l * u);
                            if ZERO <= p.abs() {
                                if self.fpool.as_mut().is_some_and(|pl| !pl.node()) {
                                    self.short = Some(Short::Keep);
                                    return false;
                                }
                                let e = ev.len();
                                ev.push(p);
                                er.push(i);
                                ec.push(j);
                                rl[i].push(e);
                                cl[j].push(e);
                                rcs[i] += 1;
                                ccs[j] += 1;
                            }
                        }
                        seen[j] = false;
                    }
                    if rcs[i] < 0 { erows.push(i) } else { rb.ins(i, rcs[i] as usize) }
                }
                for &(j, _) in &u_row {
                    w[j] = 0.0;
                    if ccs[j] < 0 { ecols.push(j) } else { cb.ins(j, ccs[j] as usize) }
                }
                rcs[r] = -1;
                ccs[c] = -1;
                self.kernel_u[sk] = u_row;
            }
        }
        self.fw.acc = acc;
        if self.steps.len() < m {
            self.singular = ecols.into_iter().zip(erows).collect();
            return false;
        }
        true
    }

    /// `036ade` pool positions at the kernel's start (asm 0xd5010): F+0x168 = F+0x158 − 1 (`05b257` C:1432), the
    /// first fill `local_f8` (C:331–366), U and L of the singleton and block phases (`1da0b2`), then the initial
    /// nodes (C:584–606: `nodes` slots ×4 from `local_f8`, the int side's rest pending). `created` nodes are used.
    fn pool_start(&mut self, spx: &Spx, ppos: &[i64], cap: i64, created: i64) -> bool {
        let m = self.head.len();
        let s4 = self.s4 as i64;
        let (mut u, mut l) = (0i64, 0i64);
        for pos in 0..m {
            let kp = ppos[pos];
            for (i, _) in spx.col(self.head[pos]) {
                let ki = self.prow[i];
                if ki >= 0 && ki < s4 && (kp < 0 || ki < kp) {
                    u += 1;
                }
                if kp >= 0 && kp < s4 && (ki < 0 || ki > kp) {
                    l += 1;
                }
            }
        }
        for (lb, ub) in &self.blk {
            l += lb.len() as i64;
            u += ub.len() as i64;
        }
        let f8 = pool_first_fill(self.fw.nnz_b, m, cap);
        let nodes = self.fw.act as i64;
        let mut pl = Pool { m: m as i64, p2: cap, p4: cap - 1, f8, b8: cap - 1 - l, c0: u, p3: f8, x138: 0, left: 0, pend: 0 };
        // C:586–591: the initial nodes must fit below L
        if pl.b8 < pl.p3 || ((pl.b8 - pl.p3) as f64) * 8.0 <= (nodes * 32) as f64 {
            self.fpool = Some(pl);
            self.short = Some(Short::Keep);
            return false;
        }
        let e = int_side(pl.p3, 4 * nodes);
        if e != 0 {
            pl.pend = e;
        }
        pl.p3 += 4 * nodes;
        pl.x138 = pl.p3.max(pl.f8);
        pl.left = nodes - created;
        self.pool_trace.push(pl.snap("start", nodes, 0));
        self.fpool = Some(pl);
        true
    }

    fn pool_event(&mut self, kind: &'static str, nodes: i64, n: i64) {
        if let Some(pl) = &self.fpool {
            let e = pl.snap(kind, nodes, n);
            self.pool_trace.push(e);
        }
    }

    /// Room for `cnt` U entries (`local_f8 − local_1c0`, C:2289 / C:3734), else −1 with F+0x158 kept.
    fn pool_u(&mut self, cnt: i64) -> bool {
        if self.fpool.as_ref().is_some_and(|pl| pl.f8 - pl.c0 < cnt) {
            self.short = Some(Short::Keep);
            return false;
        }
        true
    }

    /// Room for `cnt` L entries (`local_1b8 − local_138`, C:928 / C:2140 / C:2331 / C:3814), else −1.
    fn pool_l(&mut self, cnt: i64) -> bool {
        if self.fpool.as_ref().is_some_and(|pl| pl.b8 - pl.x138 < cnt) {
            self.short = Some(Short::Keep);
            return false;
        }
        true
    }

    fn pool_put_u(&mut self) {
        if let Some(pl) = self.fpool.as_mut() {
            pl.c0 += 1;
        }
    }

    fn pool_put_l(&mut self) {
        if let Some(pl) = self.fpool.as_mut() {
            pl.b8 -= 1;
        }
    }

    /// The dense switch's room (C:3394–3398, asm 0xd88d6): 1.5·n(n+1) slots below L and n(n+1)/2 for U with
    /// n = m − steps, else −1.
    fn pool_dense(&mut self) -> bool {
        let n = (self.head.len() - self.steps.len()) as i64;
        self.pool_event("dense", 0, 0);
        let Some(pl) = &self.fpool else { return true };
        let t = (n + 1) as i32 as f64 * 0.5 * (n as i32 as f64);
        if !(t * 3.0 <= (pl.b8 - pl.p3) as f64 && t <= (pl.f8 - pl.c0) as f64) {
            self.short = Some(Short::Keep);
            return false;
        }
        true
    }

    /// `036ade`'s `local_70` mode (M3-04c-21j): the rest of the factor on a dense n'×n' array (row r', column c' at
    /// r'·n' + c') with per-line index lists of at most `iVar67` entries (a longer line is scanned in full). Pool
    /// test and new size C:2572–2620; layout C:2621–2810 (columns in position order, rows in row order, each row's
    /// live elements in list order); search C:2813–3300 (dense switch gate, lowest buckets from 0, at most 4 lines,
    /// columns then rows per count, `1e-100 < |a|`, threshold and cost ties as the sparse kernel, head rotation);
    /// pivot C:3686–3966 (U row, then each L row with its update: a structural zero becoming ≥ ZERO joins both
    /// lists, a value falling below ZERO becomes the marker 0x2b2bff2ee48e0530 and leaves the counts).
    fn mode70(&mut self, ppos: &mut [i64], tol: f64, st: &mut Mode70In) -> bool {
        const DEAD: usize = usize::MAX;
        const TINY: f64 = 1e-100;
        let sent = f64::from_bits(0x2b2bff2ee48e0530);
        let m = self.head.len();
        let steps0 = self.steps.len();
        let l238 = (m - steps0) as i64;
        let (e60, ef0) = (st.erows.len() as i64, st.ecols.len() as i64);
        self.pool_event("mode70", 0, l238);
        let n2 = l238 - e60.min(ef0);
        let nn = (n2 * n2) as i32;
        let mut iv67 = (nn - 3 * n2 as i32) / (2 * n2 as i32);
        let iv65 = ((n2 as i32 + 2) / 3).max(3);
        let mut l40 = nn as i64;
        if iv67 < iv65 {
            iv67 = iv65;
            l40 = 3 * n2 + 2 * n2 * iv65 as i64;
        }
        if let Some(pl) = self.fpool.as_mut() {
            let q = nn as f64 * 0.3;
            let avail = pl.b8 - pl.p3;
            if (avail as f64) < l40 as i32 as f64 + q || ((pl.f8 - pl.c0) as f64) < q || avail < m as i64 {
                let e = pl.snap("short70", 0, 0);
                let cap = if pl.p2 < i64::MAX {
                    let d1 = (l40 as i32 as f64 + q) - avail as f64;
                    let q2 = q - (pl.f8 - pl.c0) as f64;
                    let d = if 0.0 <= d1 { d1 } else { 0.0 };
                    let mut x = d as i64 as f64;
                    if x <= q2 {
                        x = q2;
                    }
                    let l30 = (m as i64 - avail).max(x as i64);
                    let g = (pl.p2 + l30) as f64 * 1.4;
                    if g < 9.223372036854776e18 { g as i64 } else { i64::MAX }
                } else {
                    i64::MAX
                };
                self.pool_trace.push(e);
                self.short = Some(Short::Grow(cap));
                return false;
            }
            pl.p3 += l40;
            pl.x138 += l40;
        }
        let n = n2 as usize;
        let cap = iv67 as usize;
        let ni = n2;
        // C:2621–2700: dense columns in position order with their count, list flag and cached max
        let mut colmax = vec![0.0f64; m];
        for e in 0..st.ev.len() {
            if st.ec[e] != DEAD {
                let v = st.ev[e].abs();
                let c = st.ec[e];
                if colmax[c] <= v {
                    colmax[c] = v;
                }
            }
        }
        let mut colmap = vec![0usize; n];
        let mut cof = vec![usize::MAX; m];
        let mut cnt_c = vec![-1i64; n];
        let mut clen = vec![0usize; n];
        let mut clist = vec![0usize; n * cap];
        let mut cache = vec![0.0f64; n];
        let mut cb = Buckets::new(n);
        let mut k = 0;
        for pos in 0..m {
            if st.ccs[pos] >= 0 {
                colmap[k] = pos;
                cof[pos] = k;
                cnt_c[k] = st.ccs[pos];
                clen[k] = if st.ccs[pos] < iv67 as i64 - 1 { 0 } else { cap };
                cb.ins(k, st.ccs[pos] as usize);
                cache[k] = colmax[pos];
                k += 1;
            }
        }
        for j in 0..n - k {
            colmap[k + j] = st.ecols[st.ecols.len() - 1 - j];
        }
        // C:2700–2810: dense rows in row order, their live elements in list order into the array and the lists
        let mut rowmap = vec![0usize; n];
        let mut cnt_r = vec![-1i64; n];
        let mut rlen = vec![0usize; n];
        let mut rlist = vec![0usize; n * cap];
        let mut rb = Buckets::new(n);
        let mut a = vec![0.0f64; n * n];
        let mut r = 0;
        for i in 0..m {
            if st.rcs[i] < 0 {
                continue;
            }
            rowmap[r] = i;
            cnt_r[r] = st.rcs[i];
            rb.ins(r, st.rcs[i] as usize);
            let listed = st.rcs[i] < iv67 as i64 - 1;
            rlen[r] = if listed { 0 } else { cap };
            for &e in st.rl[i].iter().rev() {
                if st.ec[e] == DEAD {
                    continue;
                }
                let c = cof[st.ec[e]];
                if clen[c] < cap {
                    clist[c * cap + clen[c]] = r;
                    clen[c] += 1;
                }
                if listed {
                    rlist[r * cap + rlen[r]] = c;
                    rlen[r] += 1;
                }
                a[r * n + c] = st.ev[e];
            }
            r += 1;
        }
        for j in 0..n - r {
            rowmap[r + j] = st.erows[st.erows.len() - 1 - j];
        }
        let mut est = st.est;
        let mut pivots = Vec::new();
        let mut cov = [0usize; 12];
        cov[11] = cap;
        loop {
            let steps = self.steps.len();
            let nact = (m - steps) as i64 - st.ecols.len() as i64;
            if nact == 0 {
                self.pool_event("end", 0, 0);
                break;
            }
            let nsq = nact as f64 * nact as f64;
            let limit = nact.min(4);
            if est > nsq * 0.36 && (m as f64) * 4.0 < nsq && nact >= 51 && (m as f64) + nsq < 2000000000.0 {
                // C:3301–3392: the exact count over the dense columns replaces the estimate
                est = cnt_c.iter().filter(|&&c| c >= 0).map(|&c| (c + 1) as f64).sum();
                cov[10] += 1;
                if est > nsq * 0.325 {
                    // C:3405 → LAB_001d88bf with local_70 set: the dense switch of the remaining n = m − steps
                    if !self.pool_dense() {
                        return false;
                    }
                    assert!(st.ecols.is_empty() && st.erows.is_empty(), "036ade local_70 dense switch with empty lines: not ported");
                    let nd = m - steps;
                    // C:3460–3480: positions in column-count bucket order (local_1f0 heads, +4 link), local_1e0 their index
                    let (mut rows, mut didx) = (Vec::with_capacity(nd), vec![usize::MAX; n]);
                    for k in 0..n {
                        let Ok(h) = usize::try_from(cb.head[k]) else { continue };
                        let mut x = h;
                        loop {
                            didx[x] = rows.len();
                            rows.push(colmap[x]);
                            x = cb.prv[x];
                            if x == h {
                                break;
                            }
                        }
                    }
                    // C:3482–3525: the live rows in local_70 row order, values |v| ≥ ZERO (local_88) of the old array
                    let mut cols = Vec::with_capacity(nd);
                    let mut ad = vec![0.0f64; nd * nd];
                    for r in (0..n).filter(|&r| cnt_r[r] >= 0) {
                        let kk = cols.len();
                        cols.push(rowmap[r]);
                        for c in 0..n {
                            let v = a[r * n + c];
                            if ZERO <= v.abs() {
                                // a live value only sits in a column still in the buckets (C writes local_1e0 there)
                                assert!(didx[c] != usize::MAX, "036ade local_70 dense switch: value in a finished column");
                                ad[kk * nd + didx[c]] = v;
                            }
                        }
                    }
                    assert!(rows.len() == nd && cols.len() == nd, "036ade local_70 dense switch size");
                    // the new array starts at param_15[1] + param_15[3]·8 (C:3409), after the local_70 array
                    let mat16 = if self.fpool.as_ref().map_or(0, |pl| pl.p3) % 2 != 0 { 8 } else { 0 };
                    self.mode70 = Some(Mode70Trace { step: steps0, n, pivots, stats: cov });
                    self.dense_steps(ppos, nd, ad, rows, cols, mat16);
                    return true;
                }
            }
            let cmin = (0..n).find(|&k| cb.head[k] >= 0).unwrap_or(n);
            let rmin = (0..n).find(|&k| rb.head[k] >= 0).unwrap_or(n);
            let k0 = cmin.min(rmin);
            assert!(k0 < n, "036ade local_70 mode without a pivot (0x271e): not ported");
            let mut bound = cmin as f64 * rmin as f64;
            let mut bcost = n as f64 * n as f64;
            let (mut ratio, mut best, mut ns) = (0.0f64, -1i64, 0i64);
            let mut k = k0;
            'search: loop {
                let fk = k as f64;
                let b2 = fk * fk;
                if bound < b2 {
                    bound = b2;
                }
                let h = cb.head[k];
                if h < 0 {
                    if limit <= ns {
                        break 'search;
                    }
                } else {
                    let h = h as usize;
                    let mut x = h;
                    loop {
                        if clen[x] >= cap {
                            cov[0] += 1;
                            if cache[x] == 0.0 {
                                let mut t = 0.0f64;
                                for rr in 0..n {
                                    let v = a[rr * n + x].abs();
                                    if t <= v {
                                        t = v;
                                    }
                                }
                                cache[x] = if t <= TINY { 0.0 } else { t };
                            }
                            let mx = cache[x];
                            assert!(ZERO <= mx, "036ade local_70 column below ZERO (C:3030): not ported");
                            for rr in 0..n {
                                let v = a[rr * n + x].abs();
                                let cost = cnt_r[rr] as f64 * fk;
                                if TINY < v && ((cost < bcost && tol * mx < v) || (cost == bcost && ratio * mx < v)) {
                                    cov[4] += usize::from(cost == bcost);
                                    best = (x + rr * n) as i64;
                                    ratio = v / mx;
                                    bcost = cost;
                                }
                            }
                        } else {
                            cov[2] += 1;
                            let recompute = cache[x] == 0.0;
                            let (mut kept, mut t) = (0usize, 0.0f64);
                            for j in 0..clen[x] {
                                let rr = clist[x * cap + j];
                                let v = a[rr * n + x];
                                if v != 0.0 {
                                    clist[x * cap + kept] = rr;
                                    kept += 1;
                                    if t <= v.abs() {
                                        t = v.abs();
                                    }
                                }
                            }
                            clen[x] = kept;
                            if recompute {
                                cache[x] = t;
                            }
                            let mx = cache[x];
                            assert!(ZERO <= mx, "036ade local_70 column below ZERO (C:3030): not ported");
                            for j in 0..clen[x] {
                                let rr = clist[x * cap + j];
                                let v = a[rr * n + x].abs();
                                let cost = cnt_r[rr] as f64 * fk;
                                if TINY < v && ((cost < bcost && tol * mx < v) || (cost == bcost && ratio * mx < v)) {
                                    cov[4] += usize::from(cost == bcost);
                                    best = (x + n * rr) as i64;
                                    ratio = v / mx;
                                    bcost = cost;
                                }
                            }
                        }
                        ns += 1;
                        if limit <= ns || bcost <= bound {
                            assert!(best >= 0, "036ade local_70 mode without a pivot (0x271e): not ported");
                            let nx = cb.prv[x];
                            cb.head[k] = if (best % ni) as usize == x { x as i64 } else { nx as i64 };
                            break 'search;
                        }
                        x = cb.prv[x];
                        if x == h {
                            break;
                        }
                    }
                }
                if bcost <= bound {
                    break 'search;
                }
                let h = rb.head[k];
                if h >= 0 {
                    let h = h as usize;
                    let mut y = h;
                    loop {
                        if rlen[y] >= cap {
                            cov[1] += 1;
                            let mut mx = 0.0f64;
                            for c in 0..n {
                                let v = a[y * n + c].abs();
                                if mx <= v {
                                    mx = v;
                                }
                            }
                            assert!(ZERO <= mx, "036ade local_70 row below ZERO (C:3199): not ported");
                            for c in 0..n {
                                let v = a[y * n + c].abs();
                                let cost = cnt_c[c] as f64 * fk;
                                if TINY < v && ((cost < bcost && mx * tol < v) || (cost == bcost && mx * ratio < v)) {
                                    cov[4] += usize::from(cost == bcost);
                                    best = (y * n + c) as i64;
                                    ratio = v / mx;
                                    bcost = cost;
                                }
                            }
                        } else {
                            cov[3] += 1;
                            let (mut kept, mut mx) = (0usize, 0.0f64);
                            for j in 0..rlen[y] {
                                let c = rlist[y * cap + j];
                                let v = a[y * n + c];
                                if v != 0.0 {
                                    rlist[y * cap + kept] = c;
                                    kept += 1;
                                    if mx <= v.abs() {
                                        mx = v.abs();
                                    }
                                }
                            }
                            rlen[y] = kept;
                            assert!(ZERO <= mx, "036ade local_70 row below ZERO (C:3199): not ported");
                            for j in 0..rlen[y] {
                                let c = rlist[y * cap + j];
                                let v = a[y * n + c].abs();
                                let cost = cnt_c[c] as f64 * fk;
                                if TINY < v && ((cost < bcost && mx * tol < v) || (cost == bcost && mx * ratio < v)) {
                                    cov[4] += usize::from(cost == bcost);
                                    best = (c + n * y) as i64;
                                    ratio = v / mx;
                                    bcost = cost;
                                }
                            }
                        }
                        ns += 1;
                        if limit <= ns || bcost <= bound {
                            assert!(best >= 0, "036ade local_70 mode without a pivot (0x271e): not ported");
                            let nx = rb.prv[y];
                            rb.head[k] = if (best / ni) as usize == y { y as i64 } else { nx as i64 };
                            break 'search;
                        }
                        y = rb.prv[y];
                        if y == h {
                            break;
                        }
                    }
                }
                if bcost <= bound {
                    break 'search;
                }
                k += 1;
                if k >= n {
                    break 'search;
                }
            }
            assert!(best >= 0, "036ade local_70 mode without a pivot (0x271e): not ported");
            pivots.push(best);
            // C:3692–3735: the pivot
            let (r, c) = ((best / ni) as usize, (best % ni) as usize);
            est += (cnt_r[r] - 1) as f64 * (cnt_c[c] - 1) as f64 - 2.0;
            cb.rem(c, cnt_c[c] as usize);
            rb.rem(r, cnt_r[r] as usize);
            let pv = a[r * n + c];
            a[r * n + c] = 0.0;
            if !self.pool_u(cnt_r[r]) {
                return false;
            }
            let sk = self.steps.len();
            self.prow[rowmap[r]] = sk as i64;
            ppos[colmap[c]] = sk as i64;
            self.steps.push((rowmap[r], colmap[c], pv));
            // C:3736–3806: the U row
            let mut urow: Vec<(usize, f64)> = Vec::new();
            let full = rlen[r] >= cap;
            let len = if full { n } else { rlen[r] };
            for j in 0..len {
                let c2 = if full { j } else { rlist[r * cap + j] };
                let v = a[r * n + c2];
                if TINY < v.abs() {
                    cb.rem(c2, cnt_c[c2] as usize);
                    urow.push((c2, v));
                    self.pool_put_u();
                    cnt_c[c2] -= 1;
                    a[r * n + c2] = 0.0;
                }
            }
            rlen[r] = 0;
            cnt_r[r] = -1;
            if !self.pool_l(cnt_c[c]) {
                return false;
            }
            // C:3814–3923: each L row with its update
            let mut lc: Vec<(usize, f64)> = Vec::new();
            let full = clen[c] >= cap;
            let len = if full { n } else { clen[c] };
            for j in 0..len {
                let r2 = if full { j } else { clist[c * cap + j] };
                let v = a[r2 * n + c];
                if v.abs() <= TINY {
                    continue;
                }
                rb.rem(r2, cnt_r[r2] as usize);
                let l = v / pv;
                lc.push((rowmap[r2], l));
                self.pool_put_l();
                a[r2 * n + c] = 0.0;
                cnt_r[r2] -= 1;
                for &(c2, u) in &urow {
                    let old = a[r2 * n + c2];
                    let new = u * -l + old;
                    if old.abs() <= TINY {
                        if ZERO <= new.abs() {
                            cov[if old == 0.0 { 6 } else { 7 }] += 1;
                            if old == 0.0 {
                                cov[8] += usize::from(rlen[r2] >= cap) + usize::from(clen[c2] >= cap);
                                if rlen[r2] < cap {
                                    rlist[r2 * cap + rlen[r2]] = c2;
                                    rlen[r2] += 1;
                                }
                                if clen[c2] < cap {
                                    clist[c2 * cap + clen[c2]] = r2;
                                    clen[c2] += 1;
                                }
                            }
                            a[r2 * n + c2] = new;
                            cnt_r[r2] += 1;
                            cnt_c[c2] += 1;
                        }
                    } else if new.abs() < ZERO {
                        cov[5] += 1;
                        a[r2 * n + c2] = sent;
                        cnt_r[r2] -= 1;
                        cnt_c[c2] -= 1;
                    } else {
                        a[r2 * n + c2] = new;
                    }
                }
                if cnt_r[r2] < 0 {
                    cov[9] += 1;
                    st.erows.push(rowmap[r2]);
                } else {
                    rb.ins(r2, cnt_r[r2] as usize);
                }
            }
            // C:3929–3966: the U row's columns back into their buckets, their cached max cleared
            for &(c2, _) in &urow {
                cache[c2] = 0.0;
                if cnt_c[c2] < 0 {
                    cov[9] += 1;
                    st.ecols.push(colmap[c2]);
                } else {
                    cb.ins(c2, cnt_c[c2] as usize);
                }
            }
            self.kernel_u[sk] = urow.iter().map(|&(c2, v)| (colmap[c2], v)).collect();
            self.lcol[sk] = lc;
            clen[c] = 0;
            cnt_c[c] = -1;
        }
        self.mode70 = Some(Mode70Trace { step: steps0, n, pivots, stats: cov });
        true
    }

    /// `036ade` dense switch (C:3394–3470, own LU C:3968–4920, conversion C:4921–5300): the active positions in
    /// column-count bucket order (each circular list from its head through the C's +4 link, Rust `prv`) and the
    /// active rows ascending, as a column-major array of the live elements; `dense_lu` on it; then step s0 + k
    /// pivots position `rows[k]` and row `cols[k]` with the L column (row `cols[j]`, a[j·n + k]) and U row
    /// (position `rows[i]`, a[k·n + i]) above j, i > k ascending, after entries |v| < ZERO become 0 and drop.
    #[allow(clippy::too_many_arguments)]
    fn dense_switch(
        &mut self,
        ppos: &mut [i64],
        n: usize,
        cap: i64,
        (ev, ec): (&[f64], &[usize]),
        rl: &[Vec<usize>],
        cb: &Buckets,
        rcs: &[i64],
    ) {
        const DEAD: usize = usize::MAX;
        let m = self.head.len();
        let mut rows = Vec::with_capacity(n);
        for k in 0..=m {
            if rows.len() == n {
                break;
            }
            let h = cb.head[k];
            if h < 0 {
                continue;
            }
            let h = h as usize;
            let mut x = h;
            loop {
                rows.push(x);
                x = cb.prv[x];
                if x == h {
                    break;
                }
            }
        }
        assert_eq!(rows.len(), n, "dense rows");
        let mut didx = vec![usize::MAX; m];
        for (i, &p) in rows.iter().enumerate() {
            didx[p] = i;
        }
        let cols: Vec<usize> = (0..m).filter(|&i| rcs[i] >= 0).collect();
        assert_eq!(cols.len(), n, "dense columns (active rows with entries)");
        let mut a = vec![0.0f64; n * n];
        for (kk, &i) in cols.iter().enumerate() {
            for &e in &rl[i] {
                if ec[e] != DEAD {
                    a[kk * n + didx[ec[e]]] = ev[e];
                }
            }
        }
        let mat16 = if pool_first_fill(self.fw.nnz_b, m, cap) % 2 != 0 { 8 } else { 0 };
        self.dense_steps(ppos, n, a, rows, cols, mat16);
    }

    /// The dense switch's own LU and conversion (C:3968–5300) on the gathered column-major array `a` (row `cols[kk]`
    /// at kk·n, position `rows[i]` at + i) whose 16-byte offset is `mat16`.
    fn dense_steps(&mut self, ppos: &mut [i64], n: usize, mut a: Vec<f64>, rows: Vec<usize>, cols: Vec<usize>, mat16: usize) {
        let (a_in, cols_in) = (a.clone(), cols.clone());
        let (mut cols, mut rows2) = (cols, rows.clone());
        let deficits = match dense_lu(&mut a, n, &mut cols, &mut rows2, ZERO, mat16, 0, true) {
            Some(d) => d,
            None => {
                // C:4331–4336: the unblocked pass from a fresh array (bVar23 = false)
                a.copy_from_slice(&a_in);
                cols.copy_from_slice(&cols_in);
                rows2.copy_from_slice(&rows);
                dense_lu(&mut a, n, &mut cols, &mut rows2, ZERO, mat16, 0, false).expect("unblocked pass")
            }
        };
        assert_eq!(deficits, 0, "036ade dense LU left rows without a pivot: not ported");
        self.dense = Some(DenseTrace {
            step: self.steps.len(),
            n,
            mat16,
            rows: rows.clone(),
            cols_in,
            a_in,
            cols_out: cols.clone(),
            a_out: a.clone(),
        });
        // C:4921–5180: the diagonal moves out, then |v| < ZERO becomes 0 over the array
        let diag: Vec<f64> = (0..n).map(|k| a[k * n + k]).collect();
        for k in 0..n {
            a[k * n + k] = 0.0;
        }
        for v in a.iter_mut() {
            if v.abs() < ZERO {
                *v = 0.0;
            }
        }
        let s0 = self.steps.len();
        for k in 0..n {
            let (r, pos) = (cols[k], rows2[k]);
            self.prow[r] = (s0 + k) as i64;
            ppos[pos] = (s0 + k) as i64;
            self.steps.push((r, pos, diag[k]));
            self.lcol[s0 + k] = (k + 1..n).filter(|&j| a[j * n + k] != 0.0).map(|j| (cols[j], a[j * n + k])).collect();
            self.kernel_u[s0 + k] = (k + 1..n).filter(|&i| a[k * n + i] != 0.0).map(|i| (rows2[i], a[k * n + i])).collect();
        }
    }

    /// Positions, Lᵀ rows, LU+0x104 and the Lᵀ levels (036ade L5700-5912).
    fn finish(&mut self) {
        let m = self.head.len();
        self.kpos = vec![0; m];
        for (k, &(_, pos, _)) in self.steps.iter().enumerate() {
            self.kpos[pos] = k;
        }
        self.order = (0..m).collect();
        self.upos = (0..m).collect();
        // t = LU+0x100 as `1da0b2` leaves it: the end of column-singleton phase 3 (C:1514–1532); 05b257 L2595–2733
        // only reads it. LU+0xe8 = highest empty column above t
        self.prefix = self.s3;
        self.e8 = m.saturating_sub(1);
        while self.e8 > self.prefix && !self.ucol[self.e8].is_empty() {
            self.e8 -= 1;
        }
        self.general = self.ucol.iter().any(|c| !c.is_empty());
        // U columns after a factor hold their rows by descending U position (so the tail rows lead): all degen2
        // refactors captured in M1-02m. shortcut: there U position = basis position; a model where they differ
        // after a factor (kb2 refactors have no capture at 0 updates) would tell the two orders apart
        let (t, prow) = (self.prefix, &self.prow);
        self.umid = self
            .ucol
            .iter_mut()
            .map(|c| {
                c.sort_by_key(|e| std::cmp::Reverse(prow[e.0]));
                c.iter().filter(|e| prow[e.0] >= t as i64).count()
            })
            .collect();
        // 05b257 C:2595–2690: LU+0xf8 = −1 below 0xdc, the prefix rows [0xdc, t) listed in position order, −2 in the
        // tail; 036ade C:5725–5740: LU+0x108/0x110 (1.5 for an empty range)
        let (t, dc) = (self.prefix, self.dc);
        self.f8 = (0..m).map(|k| if k < dc { -1 } else if k < t { (k - dc) as i64 } else { -2 }).collect();
        self.f0 = (dc..t.max(dc)).collect();
        let mean = |r: std::ops::Range<usize>| {
            let n = r.len();
            if n == 0 { 1.5 } else { r.map(|k| self.urow[k].len()).sum::<usize>() as f64 / n as f64 + 1.0 }
        };
        self.d108 = mean(t..m);
        self.dsum = self.d108 + mean(dc..t.max(dc));
        self.lrow = vec![Vec::new(); m];
        for k in (0..m).rev() {
            for &(i, l) in &self.lcol[k] {
                self.lrow[self.prow[i] as usize].push((k, l));
            }
        }
        let lnnz: usize = self.lrow.iter().map(Vec::len).sum();
        // pool fill after a factor: 2·nnz(L+U) (kb2 capture, 4 factors), every U row slot as long as the row
        self.rcap = self.urow.iter().map(Vec::len).collect();
        self.pool = 2 * (lnnz + self.rcap.iter().sum::<usize>());
        let t = lnnz as f64 / m as f64 + 1.0;
        let t = t * t * 5.0;
        self.sparse_mult = if lnnz <= m { 2 } else if m as f64 <= t { m } else { t as usize };
        if m >= 250 && self.sparse_mult < m {
            let mut lev = vec![0; m];
            for k in (0..m).rev() {
                for &(j, _) in &self.lrow[k] {
                    lev[j] = lev[j].max(lev[k] + 1);
                }
            }
            self.ltmax = lev.iter().copied().max().unwrap_or(0) as i64;
            self.level = Some(lev);
            // 036ade C:5764–5830: L levels forward over the columns, LU+0x128 = the deepest
            let mut lev = vec![0; m];
            for k in 0..m {
                for &(i, _) in &self.lcol[k] {
                    let j = self.prow[i] as usize;
                    lev[j] = lev[j].max(lev[k] + 1);
                }
            }
            self.lmax = lev.iter().copied().max().unwrap_or(0) as i64;
            self.llev = Some(lev);
        }
        self.ltail = self.lcol[self.s4.min(m)..].iter().map(Vec::len).sum();
    }

    /// BTRAN `221173`: yᵀ = xᵀB⁻¹ for x given as (basis position, value) in input order. Returns (row, value) in the
    /// original's output order: sparse when it stays sparse, else all m rows in row order (output nnz = −1).
    /// Callers sum in this order, so the order is part of the result. shortcut: the Lᵀ linear-scan path (L956) is not
    /// ported, and with U off-diagonals or row etas a plain dense solve stands in for the original's paths.
    pub fn btran(&self, x: &[(usize, f64)]) -> Vec<(usize, f64)> {
        self.btran_dense(x).0
    }

    /// `btran` and whether the output is dense (the original's output nnz < 0), which `28c93` reads (`9e288`).
    pub fn btran_dense(&self, x: &[(usize, f64)]) -> (Vec<(usize, f64)>, bool) {
        self.btran_ut(x, self.ut_dense(x.len()))
    }

    /// BTRAN of e_r as `9e376` hands it (dual ρ_r): a non-empty U row r forces the sparse Uᵀ kernel, an empty one
    /// takes the general test (`221173` C:1196–1206).
    pub fn btran_row(&self, r: usize) -> (Vec<(usize, f64)>, bool) {
        self.btran_ut(&[(r, 1.0)], self.urow_len(r) == 0 && self.ut_dense(1))
    }

    /// Length of the U row of basis position r (LU+0xb8[r] − LU+0xb0[r]).
    pub(crate) fn urow_len(&self, r: usize) -> usize {
        self.urow.get(self.kpos[r]).map_or(0, Vec::len)
    }

    /// `221173` C:88–90: the Uᵀ phase is dense unless (int)(LU+0x108 + LU+0x110)·nnz ≤ m (32-bit product).
    pub(crate) fn ut_dense(&self, nnz: usize) -> bool {
        (self.dsum as i32).wrapping_mul(nnz as i32) > self.head.len() as i32
    }

    /// LU+0x108 + LU+0x110 (for the M1-02k probe).
    pub(crate) fn dsum(&self) -> f64 {
        self.dsum
    }

    /// BTRAN leaves the diagonal-U emulation (Markowitz kernel or FT updates).
    pub(crate) fn updated(&self) -> bool {
        self.n_updates > 0 || self.general
    }

    /// LU+0x108 and the prefix t = LU+0x100 (for the M1-02k probe).
    pub(crate) fn d108(&self) -> f64 {
        self.d108
    }

    /// Step and diagonal of basis position `pos` (for the M1-02y probe after an update).
    pub(crate) fn diag_of(&self, pos: usize) -> (usize, f64) {
        let k = self.kpos[pos];
        (k, self.steps[k].2)
    }

    /// Lᵀ DAG depth (LU+0x140), −1 without levels (for the M1-02k probe).
    pub(crate) fn lt_depth(&self) -> i64 {
        self.ltmax
    }

    /// `221173` C:263–268: Lᵀ goes sparse when 0x104·nnz ≤ m and LU+0x140 < 0.1m; otherwise the dense Lᵀ (asm
    /// 0x12fee6), which hands an empty input back as nnz 0 (C:1580), the same as the sparse path.
    fn lt_sparse_ok(&self, nnz: usize) -> bool {
        let m = self.head.len();
        self.sparse_mult * nnz <= m && (nnz == 0 || (self.ltmax as f64) < m as f64 * 0.1)
    }

    pub(crate) fn prefix_len(&self) -> usize {
        self.prefix
    }

    /// BTRAN with the Uᵀ path given: dense Uᵀ outputs nnz −1, so the row etas and Lᵀ are dense too.
    pub(crate) fn btran_ut(&self, x: &[(usize, f64)], ut_dense: bool) -> (Vec<(usize, f64)>, bool) {
        assert!(self.complete(), "singular basis");
        if self.updated() {
            // Markowitz kernel or FT updates: U is no longer diagonal, the same phase order applies
            return self.btran_updated(x, ut_dense);
        }
        let m = self.head.len();
        // Uᵀ is the diagonal: sparse (21a941, inputs in order) or dense (C:91–260)
        let sparse_u = !ut_dense;
        let xs = if sparse_u { self.ut_sparse(x) } else { Vec::new() };
        let mut dense = vec![0.0; m]; // by step
        for &(pos, v) in x.iter().filter(|_| !sparse_u) {
            let k = self.kpos[pos];
            let t = v / self.steps[k].2;
            if !(v.abs() <= ZERO || t.abs() <= ZERO) {
                dense[k] = t;
            }
        }
        if !sparse_u || !self.lt_sparse_ok(xs.len()) {
            for &(k, t) in &xs {
                dense[k] = t;
            }
            for k in (0..m).rev() {
                // dense Lᵀ (L1576): values at or below ZERO are kept but not propagated
                let v = dense[k];
                if ZERO < v.abs() {
                    for &(j, l) in &self.lrow[k] {
                        dense[j] -= l * v;
                    }
                }
            }
            let mut y = vec![(0, 0.0); m];
            for (k, &(r, _, _)) in self.steps.iter().enumerate() {
                y[r] = (r, dense[k]);
            }
            return (y, true);
        }
        (self.lt_sparse(&xs), false)
    }

    /// Sparse Uᵀ kernel `21a941` on (basis position, value) inputs in input order; returns (step, x/d) in output
    /// order. LU+0x108 ≤ 5: an input in the prefix (U position < t) goes out at once and scatters its U row, a tail
    /// input is added to w (`w += x`, whatever its size); the touched positions then leave a min-heap in ascending
    /// U position, scattering `w − u·t`, until (m − top) ≤ 6·heap size switches to a linear scan from top.
    /// LU+0x108 > 5: inputs by LU+0xf8 slot (−1 out at once, ≥ 0 out and scatter, tail added to w when |x/d| > ZERO)
    /// in input order, then a linear scan from the lowest touched tail position. Output keeps |x/d| > ZERO.
    fn ut_sparse(&self, x: &[(usize, f64)]) -> Vec<(usize, f64)> {
        let m = self.head.len();
        let d = |k: usize| self.steps[k].2;
        let mut w = vec![0.0; m]; // by step
        let mut out = Vec::new();
        let linear = |from: usize, w: &mut Vec<f64>, out: &mut Vec<(usize, f64)>| {
            for p in from..m {
                let k = self.order[p];
                let t = w[k] / d(k);
                if ZERO < t.abs() {
                    out.push((k, t));
                    for &(j, u) in &self.urow[k] {
                        w[j] -= u * t; // = −(u·t) + w (21a941), same bits
                    }
                }
                w[k] = 0.0;
            }
        };
        if self.d108 > 5.0 {
            let mut lo = m;
            for &(pos, v) in x {
                let k = self.kpos[pos];
                let t = v / d(k);
                if ZERO < t.abs() {
                    match self.f8[k] {
                        -1 => out.push((k, t)),
                        s if s < 0 => {
                            lo = lo.min(self.upos[k]);
                            w[k] += v;
                        }
                        _ => {
                            out.push((k, t));
                            for &(j, u) in &self.urow[k] {
                                w[j] -= u * t; // = −(u·t) + w (21a941), same bits
                                lo = lo.min(self.upos[j]);
                            }
                        }
                    }
                }
            }
            linear(lo, &mut w, &mut out);
            return out;
        }
        let mut mark = vec![false; m];
        let mut heap = std::collections::BinaryHeap::new(); // Reverse(U position): min-heap
        for &(pos, v) in x {
            let k = self.kpos[pos];
            if self.upos[k] < self.prefix {
                let t = v / d(k);
                if ZERO < t.abs() {
                    out.push((k, t));
                    for &(j, u) in &self.urow[k] {
                        w[j] -= u * t; // = −(u·t) + w (21a941), same bits
                        if !mark[j] {
                            mark[j] = true;
                            heap.push(std::cmp::Reverse(self.upos[j]));
                        }
                    }
                }
            } else {
                w[k] += v;
                if !mark[k] {
                    mark[k] = true;
                    heap.push(std::cmp::Reverse(self.upos[k]));
                }
            }
        }
        while let Some(&std::cmp::Reverse(top)) = heap.peek() {
            if (m - top) as f64 <= heap.len() as f64 * 6.0 {
                linear(top, &mut w, &mut out);
                break;
            }
            heap.pop();
            let k = self.order[top];
            let t = w[k] / d(k);
            if ZERO < t.abs() {
                out.push((k, t));
                for &(j, u) in &self.urow[k] {
                    w[j] -= u * t;
                    if !mark[j] {
                        mark[j] = true;
                        heap.push(std::cmp::Reverse(self.upos[j]));
                    }
                }
            }
            mark[k] = false;
            w[k] = 0.0;
        }
        out
    }

    /// Sparse Lᵀ of BTRAN (`221173` L262–616) on step-indexed inputs in arrival order: the descending-step heap
    /// when LU+0x140 < 0 (m < 250 among others), else level buckets; output (row, value) in visit order.
    fn lt_sparse(&self, xs: &[(usize, f64)]) -> Vec<(usize, f64)> {
        let m = self.head.len();
        let mut val: Vec<Option<f64>> = vec![None; m]; // touched steps
        for &(k, t) in xs {
            val[k] = Some(t);
        }
        let mut out = Vec::new();
        let mut visit = |k: usize, val: &mut Vec<Option<f64>>, touched: &mut dyn FnMut(usize)| {
            let v = val[k].take().unwrap();
            if v.abs() >= ZERO {
                out.push((self.steps[k].0, v));
                for &(j, l) in &self.lrow[k] {
                    if val[j].is_none() {
                        val[j] = Some(0.0);
                        touched(j);
                    }
                    val[j] = Some(val[j].unwrap() - l * v);
                }
            }
        };
        match &self.level {
            None => {
                // heap path: descending step
                let mut heap: std::collections::BinaryHeap<usize> = xs.iter().map(|&(k, _)| k).collect();
                while let Some(k) = heap.pop() {
                    visit(k, &mut val, &mut |j| heap.push(j));
                }
            }
            Some(lev) => {
                // level path (L619): level buckets from the lowest, each in insertion order; fills go to higher levels
                let depth = lev.iter().copied().max().unwrap_or(0);
                let mut buckets: Vec<Vec<usize>> = vec![Vec::new(); depth + 1];
                for &(k, _) in xs {
                    buckets[lev[k]].push(k);
                }
                for l in 0..=depth {
                    let bucket = std::mem::take(&mut buckets[l]);
                    for k in bucket {
                        visit(k, &mut val, &mut |j| buckets[lev[j]].push(j));
                    }
                }
            }
        }
        out
    }

    /// BTRAN with Forrest–Tomlin updates (m1_notes §8.10): Uᵀ in U order, row etas newest first, then Lᵀ. With etas
    /// and m < 0x104·nnz(Uᵀ output) the etas are dense (`221173` C:1237–1572: |v| ≤ ZERO clears w_r and skips the
    /// eta) and so is Lᵀ (C:1573–1766); else sparse etas (v = 0.0 skips, |w| ≥ ZERO kept) and a sparse Lᵀ
    /// (`lt_sparse`, the original's output order) when `lt_sparse_ok`, else dense in row order.
    /// A dense Uᵀ (`ut_dense`, C:91–260) zeroes |x| ≤ ZERO as well as |x/d| ≤ ZERO and hands nnz −1 on, so the etas
    /// and Lᵀ are dense; a sparse one is `ut_sparse` (`21a941`).
    fn btran_updated(&self, x: &[(usize, f64)], ut_dense: bool) -> (Vec<(usize, f64)>, bool) {
        let m = self.head.len();
        let mut w = vec![0.0; m]; // by step
        for &(pos, v) in x {
            w[self.kpos[pos]] += v;
        }
        let mut y = vec![0.0; m]; // by row
        let mut list = Vec::new(); // rows in arrival order
        let mut seen = vec![false; m];
        let mut put = |k: usize, t: f64, y: &mut Vec<f64>| {
            let r = self.steps[k].0;
            y[r] = t;
            if !seen[r] {
                seen[r] = true;
                list.push(r);
            }
        };
        if ut_dense {
            for &k in &self.order {
                let t = w[k] / self.steps[k].2;
                if t.abs() <= ZERO || w[k].abs() <= ZERO {
                    continue;
                }
                put(k, t, &mut y);
                for &(j, u) in &self.urow[k] {
                    w[j] -= u * t;
                }
            }
        } else {
            for (k, t) in self.ut_sparse(x) {
                put(k, t, &mut y);
            }
        }
        let dense_eta = !self.retas.is_empty() && (ut_dense || m < self.sparse_mult * list.len());
        for (r, eta) in self.retas.iter().rev() {
            // 221173 eta phase (newest first): v = w_r; w_r = η_r·v, w_i = η_i·v + w_i
            let v = y[*r];
            if dense_eta && v.abs() <= ZERO {
                y[*r] = 0.0;
            } else if v != 0.0 {
                for &(i, e) in eta {
                    y[i] = if i == *r { e * v } else { e * v + y[i] };
                    if !seen[i] {
                        seen[i] = true;
                        list.push(i);
                    }
                }
            }
        }
        let xs: Vec<(usize, f64)> =
            list.iter().filter(|&&r| y[r].abs() >= ZERO).map(|&r| (self.prow[r] as usize, y[r])).collect();
        if !dense_eta && !ut_dense && self.lt_sparse_ok(xs.len()) {
            return (self.lt_sparse(&xs), false);
        }
        let mut dense = vec![0.0; m]; // by step
        if dense_eta || ut_dense {
            // the dense eta output goes to Lᵀ whole, values below ZERO included
            for (r, &v) in y.iter().enumerate() {
                dense[self.prow[r] as usize] = v;
            }
        } else {
            for &(k, v) in &xs {
                dense[k] = v;
            }
        }
        for k in (0..m).rev() {
            let v = dense[k];
            if ZERO < v.abs() {
                for &(j, l) in &self.lrow[k] {
                    dense[j] -= l * v;
                }
            }
        }
        let mut out = vec![(0, 0.0); m];
        for (k, &(r, _, _)) in self.steps.iter().enumerate() {
            out[r] = (r, dense[k]);
        }
        (out, true)
    }

    /// FTRAN through L and the row etas (x by row); the result is the FT spike of the column.
    pub fn ftran_lower(&self, x: &mut [f64]) {
        self.ftran_l(x);
        self.ftran_r(x);
    }

    /// FTRAN through L (x by row).
    pub fn ftran_l(&self, x: &mut [f64]) {
        for (k, &(r, _, _)) in self.steps.iter().enumerate() {
            let xr = x[r];
            if xr.abs() > ZERO {
                for &(i, l) in &self.lcol[k] {
                    x[i] -= l * xr;
                }
            }
        }
    }

    /// FTRAN through the row etas (x by row).
    pub fn ftran_r(&self, x: &mut [f64]) {
        self.ftran_r_list(x, None);
    }

    /// `ftran_r` keeping a sparse side's row list as `09d112` C:1376–1553 (and `21f83a`) does: an eta row that gets
    /// a value above ZERO joins the end of the list, and rows below ZERO leave it at the end.
    fn ftran_r_list(&self, x: &mut [f64], mut list: Option<&mut Vec<usize>>) {
        let mut on = vec![false; if list.is_some() { x.len() } else { 0 }];
        if let Some(l) = list.as_deref() {
            l.iter().for_each(|&i| on[i] = true);
        }
        for ((r, eta), &odd) in self.retas.iter().zip(&self.reta_odd) {
            // 09d112 R phase (C:1136–1355): x_r = η·x in 8 lanes, one element peeled for 8-byte aligned values;
            // dropped at or below ZERO
            let v = dot8_peel(eta.iter().map(|&(i, e)| x[i] * e), odd);
            x[*r] = if v.abs() <= ZERO { 0.0 } else { v };
            if let Some(l) = list.as_deref_mut() {
                if v.abs() > ZERO && !on[*r] {
                    on[*r] = true;
                    l.push(*r);
                }
            }
        }
        if let Some(l) = list {
            if !self.retas.is_empty() {
                l.retain(|&i| x[i].abs() >= ZERO);
            }
        }
    }

    /// L and row etas of one `09d112` right-hand side (`input`: rows and values in input order, None when the side
    /// comes in dense). The L phase is the sparse kernel `21b9de` when 0x104·nnz ≤ m and 0.1m > LU+0x128
    /// (C:659–700), else dense; the row etas keep a sparse side's list while 0x104·nnz ≤ m. Returns the side's rows
    /// in the order the U phase gets them, None for a dense side.
    pub fn ftran_lr(&self, x: &mut [f64], input: Option<&[(usize, f64)]>) -> Option<Vec<usize>> {
        let m = self.head.len();
        let fits = |n: usize| self.sparse_mult * n <= m;
        let mut list = match input.filter(|i| fits(i.len()) && 0.1 * m as f64 > self.lmax as f64) {
            Some(input) => match self.l_sparse(input) {
                Some(out) => {
                    for &(i, _) in input {
                        x[i] = 0.0;
                    }
                    for &(i, v) in &out {
                        x[i] = v;
                    }
                    Some(out.into_iter().map(|(i, _)| i).collect::<Vec<_>>())
                }
                None => {
                    // shortcut: path B keeps the side sparse in the M1-02h order (input rows, then the other nonzeros
                    // of the dense solve in position order); it is taken on maros-r7, pilot, pilot87 and wood1p
                    // (outside the license, no capture). Port C:607–893 when a capture takes it.
                    self.ftran_l(x);
                    let mut seen = vec![false; m];
                    let mut l: Vec<usize> = input.iter().map(|e| e.0).filter(|&i| x[i].abs() > ZERO).collect();
                    input.iter().for_each(|e| seen[e.0] = true);
                    l.extend(self.dense_order(x).into_iter().filter(|&i| !seen[i]));
                    Some(l)
                }
            },
            None => {
                self.ftran_l(x);
                None
            }
        };
        if !self.retas.is_empty() && list.as_ref().is_some_and(|l| !fits(l.len())) {
            list = None;
        }
        self.ftran_r_list(x, list.as_mut());
        list
    }

    /// The sparse L kernel `21b9de` on (row, value) inputs in input order: (row, value) in its output order, values
    /// as it computes them. LU+0x128 < 0 (no levels): Kahn FIFO over the reached steps (C:81–257); else level
    /// buckets from the lowest input level, each in insertion order (C:259–605). Values below ZERO are dropped and
    /// not propagated.
    /// Path B (Kahn below LU+0xe4, then the tail swept in step order, C:607–893) is not ported: None (see
    /// `ftran_lr`); 0 calls on the degen2/kb2/boeing1 captures (M1-02j).
    fn l_sparse(&self, input: &[(usize, f64)]) -> Option<Vec<(usize, f64)>> {
        let m = self.head.len();
        let dm = (m - self.s4.min(m)) as f64;
        if !(dm < 6.0 || dm * (10.0 * dm) <= m as f64 || self.ltail as f64 <= 10.0 * dm) {
            return None;
        }
        let row = |k: usize| self.steps[k].0;
        let mut x = vec![0.0; m]; // by step
        let mut out = Vec::new();
        match &self.llev {
            None => {
                let mut cnt = vec![0u32; m];
                let mut lst: Vec<usize> = input.iter().map(|&(i, _)| self.prow[i] as usize).collect();
                lst.iter().for_each(|&k| cnt[k] += 1);
                let mut p = 0;
                while p < lst.len() {
                    for &(i, _) in &self.lcol[lst[p]] {
                        let j = self.prow[i] as usize;
                        if cnt[j] == 0 {
                            lst.push(j);
                        }
                        cnt[j] += 1;
                    }
                    p += 1;
                }
                let mut q = Vec::with_capacity(lst.len());
                for &(i, v) in input {
                    let k = self.prow[i] as usize;
                    x[k] = v;
                    cnt[k] -= 1;
                    if cnt[k] == 0 {
                        q.push(k);
                    }
                }
                let mut p = 0;
                while p < q.len() {
                    let k = q[p];
                    p += 1;
                    let v = std::mem::take(&mut x[k]);
                    let big = v.abs() >= ZERO;
                    if big {
                        out.push((row(k), v));
                    }
                    for &(i, l) in &self.lcol[k] {
                        let j = self.prow[i] as usize;
                        cnt[j] -= 1;
                        if big {
                            x[j] -= l * v;
                        }
                        if cnt[j] == 0 {
                            q.push(j);
                        }
                    }
                }
            }
            Some(lev) => {
                let mut bk: Vec<Vec<usize>> = vec![Vec::new(); self.lmax as usize + 1];
                let mut mark = vec![false; m];
                let mut lo = bk.len();
                for &(i, v) in input {
                    let k = self.prow[i] as usize;
                    bk[lev[k]].push(k);
                    mark[k] = true;
                    lo = lo.min(lev[k]);
                    x[k] = v;
                }
                for l in lo..bk.len() {
                    let mut t = 0;
                    while t < bk[l].len() {
                        let k = bk[l][t];
                        t += 1;
                        mark[k] = false;
                        let v = std::mem::take(&mut x[k]);
                        if v.abs() >= ZERO {
                            out.push((row(k), v));
                            for &(i, lv) in &self.lcol[k] {
                                let j = self.prow[i] as usize;
                                if !mark[j] {
                                    mark[j] = true;
                                    bk[lev[j]].push(j);
                                }
                                x[j] -= lv * v;
                            }
                        }
                    }
                }
            }
        }
        Some(out)
    }

    /// FTRAN through U (x by row in, by row out: x_{r_k} holds the value of basis position p_k).
    pub fn ftran_upper(&self, x: &mut [f64]) {
        for &k in self.order.iter().rev() {
            let (r, _, d) = self.steps[k];
            if x[r].abs() <= ZERO || (x[r] / d).abs() <= ZERO {
                x[r] = 0.0;
                continue;
            }
            x[r] /= d;
            let xr = x[r];
            for &(i, u) in &self.ucol[k] {
                x[i] -= u * xr;
            }
        }
    }

    /// U phase of the DSE τ = B⁻¹ρ as `09d112` computes it (two-RHS path C:2086–2402), x by row after
    /// `ftran_lower`: tail steps (U position ≥ LU+0x100) divide in descending U order and scatter into tail rows only;
    /// each basis position of `need` (the α_q support) in the prefix then gathers its U row in storage order,
    /// (x_i − Σ u·τ_j)/d_i, or x_i/d_i for an empty row, with no drop test. Returns τ by basis position on `need`, 0
    /// elsewhere.
    /// shortcut: the gather rule runs on every pivot; the original also has sparse and kernel-call paths, and the rule
    /// reproduced all of them only on kb2 pivots 0–21 (M1-02a-1). Upgrade when another capture disagrees.
    pub fn ftran_tau(&self, mut x: Vec<f64>, need: &[usize]) -> Vec<f64> {
        let t = self.prefix;
        for &k in self.order[t..].iter().rev() {
            let (r, _, d) = self.steps[k];
            if x[r].abs() <= ZERO || (x[r] / d).abs() <= ZERO {
                x[r] = 0.0;
                continue;
            }
            x[r] /= d;
            let xr = x[r];
            for &(i, u) in &self.ucol[k] {
                if self.upos[self.prow[i] as usize] >= t {
                    x[i] -= u * xr;
                }
            }
        }
        let mut tau = vec![0.0; self.head.len()];
        for &p in need {
            let k = self.kpos[p];
            let (r, _, d) = self.steps[k];
            tau[p] = if self.upos[k] >= t { x[r] } else { gather(x[r], &self.urow[k], |j| x[self.steps[j].0]) / d };
        }
        tau
    }

    /// Bound-flip FTRAN (`2207de`): keep the L/R list to distinguish a sparse U output from a dense one.
    pub fn ftran_flip(&self, x: Vec<f64>, input: Option<&[(usize, f64)]>) -> (Vec<f64>, bool) {
        assert!(self.complete(), "singular basis");
        let m = self.head.len();
        let te = m - self.prefix + self.f0.len();
        let d2 = self.dsum * self.dsum;
        let sparse = input.is_some_and(|i| self.sparse_mult * i.len() <= m && 0.1 * m as f64 > self.lmax as f64)
            && {
                // shortcut: evaluate the L/R list separately to preserve the existing dense arithmetic; M1-02t
                // should share the verified numerical kernel and its representation instead.
                let lr = self.ftran_lr(&mut x.clone(), input);
                lr.is_some_and(|l| te + l.len() < m || d2 * (l.len() as f64) < 2.0 * m as f64)
            };
        let xb = self.ftran_dense(x);
        (xb, !sparse)
    }

    /// Dense FTRAN: x (indexed by row) := B⁻¹x, returned by basis position.
    pub fn ftran_dense(&self, mut x: Vec<f64>) -> Vec<f64> {
        assert!(self.complete(), "singular basis");
        self.ftran_lower(&mut x);
        self.ftran_upper(&mut x);
        let mut xb = vec![0.0; self.head.len()];
        for &(r, pos, _) in &self.steps {
            xb[pos] = x[r];
        }
        xb
    }

    /// Take step k out of the LU+0xf0 list (the last entry fills its slot) and set its LU+0xf8 to `mark`.
    fn unlist(&mut self, k: usize, mark: i64) {
        if self.f8[k] >= 0 {
            let (i, last) = (self.f8[k] as usize, self.f0.pop().expect("listed row"));
            if last != k {
                self.f0[i] = last;
                self.f8[last] = i as i64;
            }
        }
        self.f8[k] = mark;
    }

    /// Rows of x (by row) above ZERO in basis position order: the `09d112` dense collection order (C:1744–1786).
    fn dense_order(&self, x: &[f64]) -> Vec<usize> {
        let mut rows: Vec<usize> = (0..x.len()).filter(|&i| x[i].abs() > ZERO).collect();
        rows.sort_by_key(|&i| self.steps[self.prow[i] as usize].1);
        rows
    }

    /// The two-RHS FTRAN `09d112` of the entering column for the dual DSE (C:150–1884) after `ftran_lr` on both
    /// sides: `lr` the α_q side's rows in the order the U phase gets them (None: dense), `spike` that side after L and
    /// the row etas (by row), `nb` the ρ side's count (−1 dense), `aq` α_q by position, `r` the pivot position.
    /// Returns the U kernel ("21686f", "2126cd", "21841f" or "inline"), the α_q output order (positions; M1-02g P3 on
    /// the sparse-U kernels: rows with LU+0xf8 = −1 in `lr` order, tail rows by U position descending, listed prefix
    /// rows in section 2 or LU+0xf0 order (M1-02m); else position order with the pivot last, C:1812–1832) and the
    /// spike order for `update`.
    pub fn ftran2(&self, lr: Option<&[usize]>, spike: &[f64], nb: i64, aq: &[f64], r: usize) -> (&'static str, Vec<usize>, Vec<usize>) {
        let m = self.head.len() as i64;
        let na = lr.map_or(-1, |l| l.len() as i64);
        let te = m - self.prefix as i64 + self.f0.len() as i64;
        let (d2, m2) = (self.dsum * self.dsum, 2.0 * m as f64);
        let sparse_u = |n: i64| n >= 0 && (te + n < m || d2 * (n as f64) < m2);
        let kernel = match (sparse_u(nb), sparse_u(na)) {
            (true, true) => "21686f",
            (true, false) => "2126cd",
            (false, true) => "21841f",
            (false, false) => "inline",
        };
        let sorder = lr.map_or_else(|| self.dense_order(spike), <[usize]>::to_vec);
        let supp: Vec<usize> = (0..aq.len()).filter(|&p| aq[p] != 0.0).collect();
        if !matches!(kernel, "21686f" | "21841f") {
            let mut out: Vec<usize> = supp.into_iter().filter(|&p| p != r).collect();
            out.push(r);
            return (kernel, out, sorder);
        }
        let t = self.prefix;
        let pos = |k: usize| self.steps[k].1;
        let mut out: Vec<usize> =
            sorder.iter().map(|&i| self.prow[i] as usize).filter(|&k| self.upos[k] < t && self.f8[k] < 0 && aq[pos(k)] != 0.0).collect();
        let mut tail: Vec<usize> = supp.iter().map(|&p| self.kpos[p]).filter(|&k| self.upos[k] >= t).collect();
        tail.sort_by_key(|&k| std::cmp::Reverse(self.upos[k]));
        // prefix rows (21686f C:89–507, 21841f C:201–552): in "section 2" order (listed input rows in input order, then
        // rows first reached from the tail outputs' prefix parts) when the input is sparse and those parts are short,
        // else in LU+0xf0 list order
        let lr: Vec<usize> = lr.expect("sparse-U input").iter().map(|&i| self.prow[i] as usize).collect();
        let (np, nt) = (lr.iter().filter(|&&k| self.f8[k] >= 0).count(), lr.iter().filter(|&&k| self.f8[k] == -2).count());
        let sec2 = m as f64 > lr.len() as f64 * d2 && d2 * ((np + nt) as f64) <= te as f64 && {
            let b8 = np + tail.iter().map(|&k| self.ucol[k].len() - self.umid[k]).sum::<usize>();
            b8 as f64 * 2.5 <= self.f0.len() as f64
        };
        let pre: Vec<usize> = if sec2 {
            let mut seen = vec![false; self.steps.len()];
            let first = lr.iter().copied().filter(|&k| self.f8[k] >= 0).chain(tail.iter().flat_map(|&k| self.prefix_part(k)));
            first.filter(|&k| !std::mem::replace(&mut seen[k], true)).collect()
        } else {
            self.f0.clone()
        };
        out.extend(tail);
        out.extend(pre.into_iter().filter(|&k| aq[pos(k)] != 0.0));
        debug_assert_eq!(out.len(), supp.len(), "sparse-U output covers the support");
        (kernel, out.into_iter().map(pos).collect(), sorder)
    }

    /// LU+0xf0 list as basis positions (snapshots).
    pub fn listed(&self) -> Vec<usize> {
        self.f0.iter().map(|&k| self.steps[k].1).collect()
    }

    /// Forrest–Tomlin update (05b257): basis position `pos` takes the column whose spike (L and row etas applied,
    /// by row) is `spike`. The replaced step moves to the end of the U order; its old U row is eliminated into a row
    /// eta. Returns false when the new diagonal is below ZERO (caller refactors). `sorder`: the spike rows in the
    /// order `09d112` hands them on (`ftran2`); empty for the dense collection order (position order).
    /// shortcut: callers outside the dual DSE pass an empty `sorder` (the `2207de` sparse paths are not modelled).
    /// `interval`: the update limit LU+0x188 (LUINTERVAL), which caps the room a moved U row gets.
    /// shortcut: the pool never outgrows its capacity LU+0x168 (05b257 C:800–860 reallocates; kb2 stays far below).
    /// shortcut: the pool growth rules are checked on kb2 (34/34 updates) and maros pivots 0–388 only.
    /// `w`: the work stores of 05b257 (docs/m1/m1_02r_ft_work.md), gathered where Rust computes them and stored at the
    /// end in the original's order. shortcut: of the early exits only the small diagonal is modelled (0xf1beb only).
    /// `diag_peel`: the η values the new diagonal reads (LU+0x1d0) are 8 but not 16-byte aligned (`Engine::new`).
    #[allow(clippy::too_many_arguments)]
    pub fn update(&mut self, pos: usize, var: usize, spike: &[f64], sorder: &[usize], interval: usize, diag_peel: bool,
        w: &mut crate::work::Work) -> bool {
        // Forrest–Tomlin as 5b257 stores it (m1_notes §8.10): r = the leaving index, q its U position, t the prefix
        // LU+0x100, p = max(q, t−1), l = max(p, U positions of the spike entries ≠ r). With l ≠ p and a non-empty U
        // row r the row eta is η = e_rᵀU⁻¹ (old U, r first then U order), the new diagonal η·S and row r leaves U;
        // otherwise the diagonal is S_r and U row r stays. r ends at position l (swap with ord[p], then the ord[l]
        // chain of L1101–1218 and a rotation of [p, l]).
        let kp = self.kpos[pos];
        let off = self.pool; // a new row eta takes the pool end (05b257 C:906–918)
        let rp = self.steps[kp].0;
        let mut at = std::mem::take(&mut self.upos);
        let q = at[kp];
        let t = self.prefix;
        let p0 = q.max(t.saturating_sub(1));
        let others: Vec<usize> = (0..spike.len()).filter(|&i| i != rp && spike[i].abs() > ZERO).collect();
        let l = others.iter().map(|&i| at[self.prow[i] as usize]).fold(p0, usize::max);
        // 05b257 C:445–449: the leaving row's place in the spike list takes its last entry
        let mut sp = if sorder.is_empty() { self.dense_order(spike) } else { sorder.to_vec() };
        if let Some(i) = sp.iter().position(|&i| i == rp) {
            sp.swap_remove(i);
        }
        // work amounts by site group (05b257 C:316–1316): η length (0xf1beb), LU+0xe8 scan (0xf1d02), column r's rows
        // (0xf1f37 when a length-1 row at a position ≤ local_40 leaves the list, else 0xf1f82 = place of r in the row),
        // row r's columns (0xf3f71 / 0xf2969 / 0xf4073 = place of r in the column section, then 0xf2a51), the moved p's
        // row (0xf3eac), the cycle (0xf3ca4, 0xf3d83)
        let (mut w_eta, mut w_e8) = (None, None);
        let mut w_col: Vec<(bool, usize)> = Vec::new();
        let (mut w_row, mut w_urow): (Vec<(u32, usize)>, Option<usize>) = (Vec::new(), None);
        let (mut w_moved, mut w_cyc): (Vec<usize>, Option<(f64, usize)>) = (Vec::new(), None);
        let old_len = self.ucol[kp].len(); // 05b257 C:529 local_150: the new column fits in place up to this
        // old column kp out of U; rows it leaves with an empty U row leave the LU+0xf0 list below (C:535–551)
        let mut emptied = Vec::new();
        for &(i, _) in &std::mem::take(&mut self.ucol[kp]) {
            // 05b257 C:556–567: the row's last entry takes the removed one's place
            let ki = self.prow[i] as usize;
            let at_kp = self.urow[ki].iter().position(|e| e.0 == kp).expect("U row entry");
            w_col.push(if self.urow[ki].len() == 1 { (true, at[ki]) } else { (false, at_kp) });
            self.urow[ki].swap_remove(at_kp);
            if self.urow[ki].is_empty() {
                emptied.push((ki, at[ki]));
            }
        }
        // new prefix (L463–512): local_40, with an empty column above p taking the place of p when q < p. It reads
        // LU+0xe8 and the column ends before row r leaves U below (C:979–1019 raise LU+0xe8 after it)
        let (tm1, nz) = (t as i64 - 1, !others.is_empty());
        let (pi, qi, li) = (p0 as i64, q as i64, l as i64);
        let mut p = p0;
        let local40 = if li == pi {
            if qi < pi || pi == tm1 { tm1 - nz as i64 } else { tm1 }
        } else if qi < pi {
            let mut e = self.e8 as i64;
            let mut found = false;
            if pi < e {
                loop {
                    if self.ucol[self.order[e as usize]].is_empty() {
                        found = true;
                        break;
                    }
                    e -= 1;
                    if pi >= e {
                        break;
                    }
                }
            }
            w_e8 = Some(self.e8 as i64 - e);
            self.e8 = e as usize;
            if found {
                p = e as usize;
                tm1
            } else if pi < li {
                t as i64 - 2
            } else {
                tm1
            }
        } else if pi < li && pi == tm1 {
            t as i64 - 2 + (!nz) as i64
        } else {
            tm1
        };
        let mut diag = spike[rp];
        if l != p0 && !self.urow[kp].is_empty() {
            // η = e_rᵀU⁻¹ by Uᵀ in U order (the 21a941 output order) on the U before the update
            let mut w = vec![0.0; self.steps.len()];
            w[kp] = 1.0;
            let mut eta: Vec<(usize, f64)> = Vec::new(); // (row, value)
            for &k in &self.order[q..] {
                let (r, _, d) = self.steps[k];
                let z = w[k] / d;
                if z.abs() <= ZERO {
                    continue;
                }
                eta.push((r, z));
                for &(j, u) in &self.urow[k] {
                    w[j] -= u * z;
                }
            }
            // 05b257 C:321–372 reads the LU+0x1d0 copy of η, whose alignment `diag_peel` gives
            diag = dot8_peel(eta.iter().map(|&(i, e)| spike[i] * e), diag_peel);
            w_eta = Some(eta.len());
            w_urow = Some(self.urow[kp].len());
            // row r out of U; a column left empty raises the LU+0xe8 cache (L1023, position before the permutation)
            for &(j, _) in &std::mem::take(&mut self.urow[kp]) {
                // C:979–1019: from a prefix part the column's last entry fills the hole; from a tail part the last
                // tail entry does, and the last prefix entry moves to the front of the prefix part
                let (v, mid) = (&mut self.ucol[j], &mut self.umid[j]);
                let h = v.iter().position(|e| e.0 == rp).expect("U column entry");
                w_row.push(if q < t { (0xf3f71, h - *mid) } else { (0xf2969, h) });
                if q < t {
                    v.swap_remove(h);
                } else {
                    *mid -= 1;
                    v[h] = v[*mid];
                    let last = v.pop().expect("U column entry");
                    if *mid < v.len() {
                        v[*mid] = last;
                    }
                }
                if self.ucol[j].is_empty() {
                    self.e8 = self.e8.max(at[j]);
                }
            }
            self.pool += eta.len();
            self.reta_odd.push(off % 2 == 1);
            self.retas.push((rp, eta));
        } else if !others.is_empty() && q < t {
            // C:871–899 (no row eta, r leaves the prefix): r swaps with the first prefix entry of its columns. The
            // original stores work here only with l = p (local_80 = 3); with l ≠ p U row r is empty
            if l == p0 {
                w_urow = Some(self.urow[kp].len());
            }
            for &(j, _) in &self.urow[kp] {
                let mid = self.umid[j];
                let h = mid + self.ucol[j][mid..].iter().position(|e| e.0 == rp).expect("U column entry");
                w_row.push((0xf4073, h - mid));
                self.ucol[j].swap(h, mid);
                self.umid[j] += 1;
            }
        }
        // LU+0xf0 list (05b257 C:535–551, C:1040–1059, C:1085–1097)
        for (k, at_k) in emptied {
            if at_k as i64 <= local40 {
                self.unlist(k, -1);
            }
        }
        if p != p0 {
            // the empty column at p takes the place of p0: its row enters the prefix
            let k = self.order[p];
            // C:1040–1069: its entries swap with the last tail entry of their columns
            let rk = self.steps[k].0;
            for &(j, _) in &self.urow[k] {
                let h = self.ucol[j][..self.umid[j]].iter().position(|e| e.0 == rk).expect("U column entry");
                w_moved.push(h);
                self.umid[j] -= 1;
                self.ucol[j].swap(h, self.umid[j]);
            }
            if self.urow[k].is_empty() {
                self.f8[k] = -1;
            } else {
                self.f8[k] = self.f0.len() as i64;
                self.f0.push(k);
            }
        }
        if l != p || p + 1 != t || nz {
            self.unlist(kp, -2);
        }
        // permutation (L1101–1218)
        let swap = |order: &mut Vec<usize>, at: &mut Vec<usize>, a: usize, b: usize| {
            order.swap(a, b);
            at[order[a]] = a;
            at[order[b]] = b;
        };
        if l <= p {
            if q < p {
                swap(&mut self.order, &mut at, q, p);
            }
        } else {
            if q < p {
                swap(&mut self.order, &mut at, q, p);
            }
            let base = p;
            let mut ll = l;
            let mut acc = 0.0;
            loop {
                let x = self.order[ll];
                if ll - base <= self.ucol[x].len() {
                    break;
                }
                acc = (acc + self.ucol[x].len() as f64 * 2.0) + 5.0;
                let m2 = self.ucol[x]
                    .iter()
                    .map(|&(i, _)| at[self.prow[i] as usize])
                    .filter(|&pp| base < pp && pp < ll)
                    .fold(base, usize::max);
                swap(&mut self.order, &mut at, base, ll);
                ll = m2;
            }
            w_cyc = Some((acc, ll - base));
            self.order[base..=ll].rotate_left(1);
            for i in base..=ll {
                at[self.order[i]] = i;
            }
        }
        self.prefix = (local40 + 1) as usize;
        self.upos = at;
        // 05b257 C:1240–1262: spike rows that land in the prefix join the list in spike order
        for &i in &sp {
            let k = self.prow[i] as usize;
            if self.upos[k] as i64 <= local40 && self.f8[k] < 0 {
                self.f8[k] = self.f0.len() as i64;
                self.f0.push(k);
            }
        }
        // pool growth: a new column longer than the old one goes to the end (C:1224–1227), a full U row that gets
        // an entry moves to the end with min(max(4, len/2), LU+0x188 − LU+0x18c) free places (C:1264–1282)
        if old_len < others.len() {
            self.pool += others.len();
        }
        let room = interval.saturating_sub(self.n_updates);
        for &i in &others {
            let k = self.prow[i] as usize;
            let n = self.urow[k].len();
            if n == self.rcap[k] {
                let extra = (n / 2).max(4).min(room);
                self.pool += n + extra;
                self.rcap[k] = n + extra;
            }
            self.urow[k].push((kp, spike[i]));
        }
        // C:1228–1260: the new column takes the spike rows in list order, tail rows from the front and prefix rows
        // from the back
        let (tl, pre): (Vec<usize>, Vec<usize>) =
            sp.iter().filter(|&&i| spike[i].abs() > ZERO).partition(|&&i| self.upos[self.prow[i] as usize] as i64 > local40);
        debug_assert_eq!(tl.len() + pre.len(), others.len(), "spike list = spike support");
        self.umid[kp] = tl.len();
        self.ucol[kp] = tl.iter().chain(pre.iter().rev()).map(|&i| (i, spike[i])).collect();
        if others.is_empty() {
            self.e8 = self.e8.max(self.upos[kp]);
        }
        // work stores in the original's order (05b257 C:417 … C:1316, docs/m1/m1_02r_ft_work.md §2)
        use crate::work::X148;
        if let Some(n) = w_eta.filter(|&n| n > 0) {
            w.add(0xf1beb, (X148 * 3.0) * n as f64);
        }
        if diag.abs() <= ZERO || diag.is_nan() {
            return false; // C:445–455: 05b257 leaves before any other store and the caller refactors
        }
        if let Some(d) = w_e8 {
            w.add(0xf1d02, (d as f64 * 3.0) * X148);
        }
        for (len1, v) in w_col {
            if !len1 {
                w.add(0xf1f82, v as f64 * X148);
            } else if v as i64 <= local40 {
                w.add(0xf1f37, X148 * 3.0);
            }
        }
        let ns = sp.len();
        let a = (old_len as f64 * 5.0) * X148;
        if old_len < ns {
            w.add(0xf2044, a);
        } else if ns > 0 {
            w.add(0xf207a, a);
        } else {
            w.carry(a);
        }
        w.add(0xf2521, (ns as f64 * 3.0) * X148);
        for (site, v) in w_row {
            w.add(site, v as f64 * X148);
        }
        if let Some(n) = w_urow {
            w.add(0xf2a51, (n as f64 * 3.0) * X148);
        }
        for v in w_moved {
            w.add(0xf3eac, v as f64 * X148);
        }
        if let Some((acc, k)) = w_cyc {
            w.add(0xf3ca4, acc * X148);
            w.add(0xf3d83, (k as f64 * 2.0) * X148);
        }
        w.add(0xf2e3c, (ns as f64 * 5.0) * X148);
        self.steps[kp].2 = diag;
        self.head[pos] = var;
        self.general = true;
        self.n_updates += 1;
        // the LU+0xf0 list stays in step with U: listed rows in the prefix at their slot, −2 exactly in the tail, −1
        // only on empty U rows (all three hold in the kb2 capture)
        debug_assert!(self.f0.iter().enumerate().all(|(i, &k)| self.f8[k] == i as i64 && self.upos[k] < self.prefix));
        debug_assert!((0..self.f8.len()).all(|k| (self.f8[k] == -2) == (self.upos[k] >= self.prefix)));
        debug_assert!((0..self.f8.len()).all(|k| self.f8[k] != -1 || self.urow[k].is_empty()));
        // and every column is [tail rows | prefix rows]
        debug_assert!((0..self.ucol.len()).all(|k| self.ucol[k]
            .iter()
            .enumerate()
            .all(|(h, e)| (h < self.umid[k]) == (self.upos[self.prow[e.0] as usize] >= self.prefix))));
        true
    }
}

/// `09d112` gather v − Σ u·x_j over a U row in storage order (asm 0x12925f–0x129465): in sequence below 8 terms;
/// else lane 0 starts at v, eight lanes subtract and combine as ((l0+l2)+(l4+l6)) + ((l1+l3)+(l5+l7)), the tail in
/// sequence.
/// shortcut: no one-element peel when the row's value array is 8- but not 16-byte aligned (the original's row storage
/// offsets are not modelled; kb2 has no 8+ term row).
fn gather(v: f64, row: &[(usize, f64)], x: impl Fn(usize) -> f64) -> f64 {
    if row.len() < 8 {
        return row.iter().fold(v, |a, &(j, u)| a - u * x(j));
    }
    let (chunks, rest) = row.as_chunks::<8>();
    let mut a = [v, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
    for c in chunks {
        for (q, &(j, u)) in c.iter().enumerate() {
            a[q] -= u * x(j);
        }
    }
    let w = ((a[0] + a[2]) + (a[4] + a[6])) + ((a[1] + a[3]) + (a[5] + a[7]));
    rest.iter().fold(w, |a, &(j, u)| a - u * x(j))
}

/// Basic solution x_B = B⁻¹(b − N·x_N) with nonbasic values from the status array
/// (−1 lower, −2 upper, −4 fixed at lower, others 0); columns subtracted in order 0..n+m.
pub fn basic_solution(spx: &Spx, vstat: &[i32]) -> Vec<f64> {
    let head = basis_head(spx.m, vstat);
    let mut rhs = spx.b.clone();
    for (j, &s) in vstat.iter().enumerate() {
        let xj = match s {
            -2 => spx.u[j],
            -1 | -4 => spx.l[j],
            _ => 0.0,
        };
        if s < 0 && xj != 0.0 {
            for (i, a) in spx.col(j) {
                rhs[i] -= a * xj;
            }
        }
    }
    Lu::factor(spx, &head).ftran_dense(rhs)
}

pub fn basis_head(m: usize, vstat: &[i32]) -> Vec<usize> {
    let mut head = vec![usize::MAX; m];
    for (j, &s) in vstat.iter().enumerate() {
        if s >= 0 {
            head[s as usize] = j;
        }
    }
    head
}

/// Iteration-0 reduced costs d (n+m) and exact DSE weights w (m, by basis position) for the basis in `vstat`.
/// d: basic 0, upper-bounded nonbasic (−2) negated, fixed structurals (−4) keep c_j (they are not in the row-wise
/// nonbasic copy); y = c_Bᵀ B⁻¹ from a dense BTRAN. w_p = ‖e_pᵀB⁻¹‖² summed in BTRAN output order (1bae9c EXACTNORM).
pub fn dual_start(spx: &Spx, vstat: &[i32]) -> (Vec<f64>, Vec<f64>) {
    let (m, n) = (spx.m, spx.n);
    let lu = Lu::factor(spx, &basis_head(m, vstat));
    let cb: Vec<(usize, f64)> = (0..m).map(|p| (p, spx.c[lu.head[p]])).collect();
    let mut y = vec![0.0; m];
    for (r, v) in lu.btran(&cb) {
        y[r] = v;
    }
    let d = (0..n + m)
        .map(|j| match vstat[j] {
            s if s >= 0 => 0.0,
            -4 if j < n => spx.c[j],
            s => {
                let t = spx.col(j).fold(spx.c[j], |t, (i, a)| t - a * y[i]);
                if s == -2 { -t } else { t }
            }
        })
        .collect();
    let w = (0..m).map(|p| sum_squares(&lu.btran(&[(p, 1.0)]).iter().map(|e| e.1).collect::<Vec<_>>())).collect();
    (d, w)
}

/// Σ of products in the original 8-lane order (5b257 L318–420, 2207de R phase): in sequence below 8 terms, else
/// lanes over the 8-aligned head combined ((0+2)+(4+6)) + ((1+3)+(5+7)), then the tail in sequence.
pub fn dot8(terms: impl Iterator<Item = f64>) -> f64 {
    dot8_peel(terms, false)
}

/// `dot8` over an array that may be 8 but not 16-byte aligned (`09d112` C:1149–1260, asm 0x1272d7): with `peel` the
/// first term goes to lane 0 first, and below 9 terms the whole sum is sequential.
fn dot8_peel(terms: impl Iterator<Item = f64>, peel: bool) -> f64 {
    let v: Vec<f64> = terms.collect();
    let p = peel as usize;
    if v.len() < 8 + p {
        return v.iter().fold(0.0, |a, &t| a + t);
    }
    let (head, tail) = v[p..].as_chunks::<8>();
    let mut a = [v[..p].iter().fold(0.0, |a, &t| a + t), 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
    for c in head {
        for q in 0..8 {
            a[q] += c[q];
        }
    }
    let w = ((a[0] + a[2]) + (a[4] + a[6])) + ((a[1] + a[3]) + (a[5] + a[7]));
    tail.iter().fold(w, |w, t| w + t)
}

/// Σv² as 1bae9c sums it (L1971-2110): in sequence below 8 values, else 8 lanes over the 8-aligned head combined
/// ((0+2)+(4+6)) + ((1+3)+(5+7)) (asm 0xc3a26; the decompiled order is wrong), then the tail in sequence.
/// The vector buffer is 16-byte aligned, so no element is peeled first.
pub fn sum_squares(v: &[f64]) -> f64 {
    let (head, tail) = v.as_chunks::<8>();
    let mut a = [0.0; 8];
    for c in head {
        for q in 0..8 {
            a[q] += c[q] * c[q];
        }
    }
    let w = ((a[0] + a[2]) + (a[4] + a[6])) + ((a[1] + a[3]) + (a[5] + a[7])); // 0.0 when there is no full chunk
    tail.iter().fold(w, |w, t| w + t * t)
}

#[cfg(test)]
mod tests {
    use super::{dot8_peel, factspace, gather, next_prime, sum_squares, BlockState, Lu};

    /// `045a76` FactSpace against original F+0x158 (bench/oracle/m1_02w/pool, m1_02w2/final): one model per branch
    /// set, SIMPLEXALLOCS 7 and c = 1 (warm start).
    #[test]
    fn factspace_cases() {
        for (m, n, z, k, c, want) in [
            (821, 1571, 10400, 5, 0, 124072), // 25fv47 m<n, m<3000
            (56, 97, 383, 5, 0, 6421),        // adlittle m<n, m<500
            (488, 163, 2410, 5, 0, 78215),    // agg m>=n, nnz>10n
            (173, 262, 3375, 5, 0, 87976),    // beaconfd m<n half, nnz>10n
            (166, 162, 1215, 5, 0, 49631),    // boeing2 m>=n, n<500
            (1503, 1818, 24646, 5, 0, 506687), // degen3 m<3000, nnz>10n
            (140, 301, 2612, 5, 0, 59032),    // grow7 m<n half
            (174, 142, 2269, 5, 0, 70052),    // israel m>=n half, nnz>10n
            (43, 41, 286, 5, 0, 6211),        // kb2 m>=n half
            (27, 32, 83, 7, 0, 1922),         // afiro SIMPLEXALLOCS 7
            (300, 645, 5620, 5, 0, 131899),   // grow15
            (300, 645, 5620, 5, 1, 197849),   // grow15 warm start c = 1
            (356, 467, 3856, 5, 0, 142294),   // stair
        ] {
            assert_eq!(factspace(m, n, z, k, c), want, "m {m} n {n} nnz {z} k {k} c {c}");
        }
    }
    use crate::scaling::Scaling;
    use crate::spx::Spx;

    /// m×n dense matrix as an Spx (slack columns implied).
    fn spx_of(a: &[Vec<f64>]) -> Spx {
        let (m, n) = (a.len(), a[0].len());
        let (mut beg, mut len, mut ind, mut val) = (vec![0i64], vec![], vec![], vec![]);
        for j in 0..n {
            let mut k = 0;
            for (i, row) in a.iter().enumerate() {
                if row[j] != 0.0 {
                    ind.push(i as i32);
                    val.push(row[j]);
                    k += 1;
                }
            }
            len.push(k);
            beg.push(beg[j] + k as i64);
        }
        Spx {
            m,
            n,
            beg,
            len,
            ind,
            val,
            sense: vec![b'<'; m],
            vtype: vec![b'C'; n],
            c: vec![0.0; n + m],
            l: vec![0.0; n + m],
            u: vec![1.0; n + m],
            b: vec![0.0; m],
            scaling: Scaling { col: vec![1.0; n], row: vec![1.0; m], obj: 1.0 },
        }
    }

    fn column(spx: &Spx, j: usize) -> Vec<f64> {
        let mut x = vec![0.0; spx.m];
        for (i, a) in spx.col(j) {
            x[i] = a;
        }
        x
    }

    /// B·x for x by position.
    fn times(spx: &Spx, head: &[usize], x: &[f64]) -> Vec<f64> {
        let mut y = vec![0.0; spx.m];
        for (p, &j) in head.iter().enumerate() {
            for (i, a) in spx.col(j) {
                y[i] += a * x[p];
            }
        }
        y
    }

    fn check(spx: &Spx, lu: &Lu) {
        let m = spx.m;
        for j in 0..spx.n + m {
            let x = lu.ftran_dense(column(spx, j));
            let y = times(spx, &lu.head, &x);
            let e = column(spx, j);
            assert!(y.iter().zip(&e).all(|(a, b)| (a - b).abs() < 1e-9), "ftran col {j}");
        }
        for p in 0..m {
            let mut y = vec![0.0; m];
            for (r, v) in lu.btran(&[(p, 1.0)]) {
                y[r] = v;
            }
            for (q, &j) in lu.head.iter().enumerate() {
                let dot: f64 = spx.col(j).map(|(i, a)| a * y[i]).sum();
                assert!((dot - if p == q { 1.0 } else { 0.0 }).abs() < 1e-9, "btran row {p}");
            }
        }
    }

    /// Factor `a` (structural columns 0..n, the slack of the last row basic) with the 2×2 block phase always on.
    fn block_factor(a: &[Vec<f64>]) -> (Spx, Lu, BlockState) {
        let spx = spx_of(a);
        let mut head: Vec<usize> = (0..spx.n).collect();
        head.push(spx.n + spx.m - 1);
        let mut st = BlockState { x2c0: 1, x4e0: 0, x4e4: 2_000_000_000 };
        let lu = Lu::factor_block(&spx, &head, super::MARKOWITZ_TOL, &mut st);
        assert!(lu.complete());
        check(&spx, &lu);
        (spx, lu, st)
    }

    // rows 0 and 1 have their two active entries in columns 0 and 1: phase A pairs them (C:686–1031)
    const PAIR: [[f64; 6]; 7] = [
        [2.0, 1.0, 0.0, 0.0, 0.0, 0.0],
        [1.0, 3.0, 0.0, 0.0, 0.0, 0.0],
        [1.0, 0.0, 4.0, 1.0, 0.0, 0.0],
        [0.0, 1.0, 1.0, 0.0, 2.0, 0.0],
        [0.0, 0.0, 0.0, 1.0, 1.0, 3.0],
        [0.0, 0.0, 1.0, 2.0, 1.0, 1.0],
        [0.0, 0.0, 0.0, 0.0, 0.0, 1.0],
    ];

    /// Synthetic (no original comparison): the block steps keep B = LU and the state update of C:681–1510.
    #[test]
    fn block_phase_a_reproduces_the_basis_inverse() {
        let a: Vec<Vec<f64>> = PAIR.iter().map(|r| r.to_vec()).collect();
        let (_, lu, st) = block_factor(&a);
        let p = lu.parts();
        // slack, then the row block: row 0 (|2| > |1| in column 0) on position 0, row 1 on the Schur value 3 − 1·1/2
        assert_eq!((p.s4, st.x4e0, st.x4e4), (1, 1, 2_000_000_000));
        assert_eq!(&p.steps[1..3], &[(0, 0, 2.0), (1, 1, 2.5)]);
        // without the phase the Markowitz kernel pivots these rows differently
        let off = Lu::factor_tol(&spx_of(&a), &lu.head, super::MARKOWITZ_TOL);
        assert_ne!(&off.parts().steps[1..3], &p.steps[1..3]);
    }

    #[test]
    fn block_phase_b_reproduces_the_basis_inverse() {
        // the transpose of PAIR's 6×6 part: columns 0 and 1 have their two active entries in rows 0 and 1 (C:1032–1493)
        let mut a: Vec<Vec<f64>> = (0..6).map(|i| (0..6).map(|j| PAIR[j][i]).collect()).collect();
        a.push(vec![0.0, 0.0, 0.0, 0.0, 0.0, 1.0]);
        let (_, lu, st) = block_factor(&a);
        let p = lu.parts();
        assert_eq!((p.s4, st.x4e0), (1, 1));
        // row 0 (the smaller (count, index)) pivots on column 0 (|2| > |1|), row 1 on column 1 with 3 − 1·1/2
        assert_eq!(&p.steps[1..3], &[(0, 0, 2.0), (1, 1, 2.5)]);
    }

    #[test]
    fn block_phase_gate_and_state() {
        let a: Vec<Vec<f64>> = PAIR.iter().map(|r| r.to_vec()).collect();
        let spx = spx_of(&a);
        let mut head: Vec<usize> = (0..spx.n).collect();
        head.push(spx.n + spx.m - 1);
        // S+0x4e4 = 0 with 6 active (< 50): the phase does not run, nothing changes
        let mut st = BlockState { x2c0: 21, x4e0: 3, x4e4: 0 };
        let lu = Lu::factor_block(&spx, &head, super::MARKOWITZ_TOL, &mut st);
        assert_eq!(st, BlockState { x2c0: 21, x4e0: 3, x4e4: 0 });
        assert_eq!(lu.parts().s4, lu.n_singleton);
        // off: same factor as factor_tol
        let mut st = BlockState::OFF;
        let a2 = Lu::factor_block(&spx, &head, super::MARKOWITZ_TOL, &mut st);
        assert_eq!(a2.parts().steps, lu.parts().steps);
        assert_eq!(next_prime(12), 13);
        assert_eq!(next_prime(128), 131);
        assert_eq!(next_prime(262), 263);
        assert_eq!(next_prime(25), 29);
    }

    #[test]
    fn mixed_singleton_and_markowitz_basis_of_sc50a() {
        // a basis the dual simplex reaches on sc50a: 8 singleton steps, the rest by Markowitz
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../../re/bench/data/netlib/sc50a.mps.gz");
        let md = rsolver_model::mps::read_file(path).unwrap();
        let spx = Spx::new(&md);
        let head = vec![
            47, 40, 39, 38, 3, 25, 18, 7, 2, 17, 6, 1, 69, 58, 13, 14, 4, 11, 48, 50, 37, 19, 22, 8, 9, 10, 15, 30, 12, 29, 35,
            20, 21, 36, 26, 27, 28, 34, 33, 31, 32, 0, 23, 24, 41, 42, 43, 44, 45, 46,
        ];
        let lu = Lu::factor(&spx, &head);
        assert!(lu.complete());
        check(&spx, &lu);
    }

    #[test]
    fn markowitz_and_forrest_tomlin_reproduce_the_basis_inverse() {
        let mut s = 12345u64;
        let mut rnd = || {
            s ^= s << 13;
            s ^= s >> 7;
            s ^= s << 17;
            s
        };
        let (m, n) = (7, 12);
        let a: Vec<Vec<f64>> =
            (0..m).map(|_| (0..n).map(|_| if rnd() % 3 != 0 { (rnd() % 19) as f64 - 9.0 } else { 0.0 }).collect()).collect();
        let spx = spx_of(&a);
        // a structural-heavy basis needs the Markowitz phase; keep slacks where columns are missing
        let mut head: Vec<usize> = (0..m).map(|i| n + i).collect();
        let mut lu = Lu::factor(&spx, &head);
        for j in 0..n {
            for p in (0..m).filter(|&p| head[p] >= n) {
                let mut h = head.clone();
                h[p] = j;
                let t = Lu::factor(&spx, &h);
                if t.complete() && t.steps.iter().all(|s| s.2.abs() > 1e-6) {
                    head = h;
                    lu = t;
                    break;
                }
            }
        }
        assert!(head.iter().filter(|&&j| j < n).count() >= 4);
        assert!(lu.general, "basis should need the Markowitz phase");
        check(&spx, &lu);
        // Forrest-Tomlin updates against the same basis changes
        let mut done = 0;
        for j in 0..n + m {
            if lu.head.contains(&j) {
                continue;
            }
            let mut x = column(&spx, j);
            lu.ftran_lower(&mut x);
            let spike = x.clone();
            lu.ftran_upper(&mut x);
            let Some(pos) = (0..m).find(|&p| x[lu.steps[lu.kpos[p]].0].abs() > 0.5) else { continue };
            assert!(lu.update(pos, j, &spike, &[], usize::MAX, false, &mut crate::work::Work::new()));
            check(&spx, &lu);
            done += 1;
        }
        assert!(done >= 4);
    }

    #[test]
    fn dot8_peel_order() {
        // 8 terms: lanes without a peel, in sequence with one (kb2 pivot 30, row 17)
        let t = [1e16, 1.0, 1.0, 1.0, -1e16, 1.0, 1.0, 1.0];
        assert_eq!(dot8_peel(t.into_iter(), false), ((1e16 + 1.0) + (-1e16 + 1.0)) + ((1.0 + 1.0) + (1.0 + 1.0)));
        assert_eq!(dot8_peel(t.into_iter(), true), t.iter().fold(0.0, |a, &x| a + x));
        // 9 terms with a peel: the first starts lane 0, the other 8 fill the lanes
        let t9 = [1.0, 1e16, 1.0, 1.0, 1.0, -1e16, 1.0, 1.0, 1.0];
        assert_eq!(dot8_peel(t9.into_iter(), true), ((1.0 + 1e16 + 1.0) + (-1e16 + 1.0)) + ((1.0 + 1.0) + (1.0 + 1.0)));
    }

    #[test]
    fn sum_squares_lane_order() {
        // 9 values: lanes take 0..8, the 9th is added last; (1e16 + 1) + ... shows the grouping
        let v = [1e8, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0];
        let lanes = ((1e16 + 1.0) + (1.0 + 1.0)) + ((1.0 + 1.0) + (1.0 + 1.0));
        assert_eq!(sum_squares(&v), lanes + 1.0);
        assert_eq!(sum_squares(&v[..7]), v[..7].iter().fold(0.0, |w, t| w + t * t));
    }

    #[test]
    fn gather_lane_order() {
        // 9 terms u·1: v and terms 0..8 in lanes (v seeds lane 0), the 9th subtracted last; 1e16 shows the grouping
        let row: Vec<(usize, f64)> = (0..9).map(|_| (0, 1.0)).collect();
        let lanes = (((1e16 - 1.0) + -1.0) + (-1.0 + -1.0)) + ((-1.0 + -1.0) + (-1.0 + -1.0));
        assert_eq!(gather(1e16, &row, |_| 1.0), lanes - 1.0);
        assert_ne!(lanes - 1.0, (0..9).fold(1e16, |a, _| a - 1.0));
        assert_eq!(gather(1e16, &row[..7], |_| 1.0), (0..7).fold(1e16, |a, _| a - 1.0));
    }
}
