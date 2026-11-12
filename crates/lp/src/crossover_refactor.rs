//! The push refactor's factor on the original layout: `036ade` (singleton phase `1da0b2` and the Markowitz kernel,
//! through M1 `Lu::factor_tol`) and the setup after it in `05b257` (C:2540–2760), written into the `crossover_lu`
//! state the push backend carries. The allocation the factor does not own (pool size F+0x158, its alignment, the
//! update limit F+0x188, the work scale F+0x1f8) and the push-state parts (spike, cache, scratch) come from the
//! state before the call. Contract and C references: docs/m3/primal_push_refactor_rust.md.

use crate::crossover_btran::Cache;
use crate::crossover_ftran::{Cells, Spike};
use crate::crossover_lu::{LuState, Scalars};
use crate::lu::{BlockState, Lu};
use crate::scaling::Scaling;
use crate::spx::Spx;

/// The constraint matrix the push factors (S+0x64/0x68, CSC S+0x70/0x78/0x80/0x88; slack j = n+i is +e_i).
pub struct Matrix {
    pub m: usize,
    pub n: usize,
    pub beg: Vec<i64>,
    pub len: Vec<i32>,
    pub ind: Vec<i32>,
    pub val: Vec<f64>,
}

/// F+0x1a8 after `036ade` holds the factor's scratch (seen: halves of 1.0 and counts); every reader zeroes the counts
/// it uses first (`21b9de` C:262–280, `221173` level path), so these cells are unknown.
pub const BUCKET_UNKNOWN: i32 = i32::MIN;

/// The allocation state a refactor keeps: pool size F+0x158 (F+0x168 = F+0x158 − 1 is set before the call,
/// `05b257` C:1431), the pool's alignment, the update limit F+0x188 and the work scale F+0x1f8.
#[derive(Clone, Copy)]
pub struct Alloc {
    pub x158: i64,
    pub pool_mod16: [i32; 2],
    pub x188: i32,
    pub x1f8: f64,
}

impl Alloc {
    pub fn of(s: &LuState) -> Alloc {
        Alloc { x158: s.sc.x158, pool_mod16: s.sc.pool_mod16, x188: s.sc.x188, x1f8: s.sc.x1f8 }
    }
}

/// What the refactor returns besides the factor: F+0x28 (old position of each new one) and F+0x38 (its inverse) at
/// the `036ade` return, before `05b257` overwrites them with the identity.
pub struct Refactor {
    pub state: LuState,
    pub o: Vec<i32>,
    pub n: Vec<i32>,
    /// The kernel's dense switch when the factor took it (M3-04c-21i).
    pub dense: Option<crate::lu::DenseTrace>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum RefactorError {
    /// The factor did not pivot every position (singular basis, `036ade` returns non-zero; not in this range).
    Singular,
}

/// Level arrays of `036ade` C:5764–5912: counts per level go to lst[level + 1], then lst[0..=levels] is summed
/// (lst[l] = first slot of level l) and lst[levels + 1] keeps the count of the deepest level.
fn level_starts(lev: &[usize]) -> (i32, Vec<i32>) {
    let levels = lev.iter().copied().max().unwrap_or(0);
    let mut lst = vec![0i32; levels + 2];
    for &l in lev {
        lst[l + 1] += 1;
    }
    for l in 1..=levels {
        lst[l] += lst[l - 1];
    }
    (levels as i32, lst)
}

/// Refactor `header` (variable of each basis position) with ZERO `zero` and Markowitz threshold `tol`.
/// `alloc` is the allocation before the call; `prev` the backend state before it (its spike, cache and scratch
/// are kept; None at the push's first factor call, where they are unknown). `block` is the `1da0b2` 2×2 block phase
/// state on S (S+0x2c0/0x4e0/0x4e4), updated as the original does.
pub fn refactor(
    a: &Matrix,
    header: &[i32],
    zero: f64,
    tol: f64,
    alloc: Alloc,
    prev: Option<&LuState>,
    block: &mut BlockState,
) -> Result<Refactor, RefactorError> {
    let (m, n) = (a.m, a.n);
    assert_eq!(zero.to_bits(), crate::lu::ZERO.to_bits(), "ZERO other than the M1 factor's");
    let spx = Spx {
        m,
        n,
        beg: a.beg.clone(),
        len: a.len.clone(),
        ind: a.ind.clone(),
        val: a.val.clone(),
        sense: vec![b'<'; m],
        vtype: vec![b'C'; n],
        c: vec![0.0; n + m],
        l: vec![0.0; n + m],
        u: vec![0.0; n + m],
        b: vec![0.0; m],
        scaling: Scaling { col: vec![1.0; n], row: vec![1.0; m], obj: 1.0 },
    };
    let head: Vec<usize> = header.iter().map(|&j| j as usize).collect();
    // M3-04c-21i: the kernel's dense switch on, its array alignment from the pool size F+0x158
    let mut lu = Lu::factor_dense(&spx, &head, tol, block, alloc.x158);
    let dense = lu.dense.take();
    if !lu.complete() {
        return Err(RefactorError::Singular);
    }
    let p = lu.parts();
    let step = |i: usize| p.prow[i] as i32;
    let x20: Vec<i32> = p.steps.iter().map(|s| s.0 as i32).collect();
    let mut x30 = vec![0i32; m];
    for (k, s) in p.steps.iter().enumerate() {
        x30[s.0] = k as i32;
    }
    let o: Vec<i32> = p.steps.iter().map(|s| s.1 as i32).collect();
    let mut nn = vec![0i32; m];
    for (k, &q) in o.iter().enumerate() {
        nn[q as usize] = k as i32;
    }
    let ident: Vec<i32> = (0..m as i32).collect();
    let diag: Vec<f64> = p.steps.iter().map(|s| s.2).collect();

    // L columns by step (F+0x58/0x68/0x60), rows as steps
    let mut lbeg = vec![0i64; m + 1];
    let (mut lrows, mut lvals) = (Vec::new(), Vec::new());
    for k in 0..m {
        for &(i, l) in &p.lcol[k] {
            lrows.push(step(i));
            lvals.push(l);
        }
        lbeg[k + 1] = lrows.len() as i64;
    }
    // L rows (F+0x70/0x80/0x78)
    let mut rbeg = vec![0i64; m + 1];
    let (mut rcols, mut rvals) = (Vec::new(), Vec::new());
    for k in 0..m {
        for &(j, l) in &p.lrow[k] {
            rcols.push(j as i32);
            rvals.push(l);
        }
        rbeg[k + 1] = rcols.len() as i64;
    }
    let lnnz = lrows.len();
    let (x128, llev, llst) = match p.llev {
        Some(lev) => {
            let (lv, lst) = level_starts(lev);
            (lv, Some(lev.iter().map(|&x| x as i32).collect::<Vec<_>>()), Some(lst))
        }
        None => (-1, None, None),
    };
    let (x140, rlev, rlst) = match p.level {
        Some(lev) => {
            let (lv, mut lst) = level_starts(lev);
            // the backend keeps F+0x138 up to the deepest level (what `221173` reads and the capture holds)
            lst.pop();
            (lv, Some(lev.iter().map(|&x| x as i32).collect::<Vec<_>>()), Some(lst))
        }
        None => (-1, None, None),
    };
    let bucket = (x140 >= 0).then(|| vec![BUCKET_UNKNOWN; (x140 as usize + 1).max(x128.max(0) as usize + 1)]);

    // U: rows from the pool start, columns from pool + F+0x160/2 (036ade C:362, C:5378)
    let unnz: usize = p.urow.iter().map(Vec::len).sum();
    let x160 = 2 * (lnnz + unnz) as i64;
    let colbase = x160 / 2;
    let size = alloc.x158 as usize;
    let mut pidx = Cells::unknown(size);
    let mut pval = Cells::unknown(size);
    let (mut rs, mut re) = (vec![0i64; m], vec![0i64; m]);
    let mut at = 0i64;
    for k in 0..m {
        rs[k] = at;
        for &(j, u) in &p.urow[k] {
            pidx.set(at as usize, j as i32);
            pval.set(at as usize, u);
            at += 1;
        }
        re[k] = at;
    }
    // t = F+0x100 as `1da0b2` leaves it (end of the column singletons of phase 3, C:1514–1532); M1 counts the
    // leading empty U columns instead, which runs past it when a phase-4 column happens to be empty
    let (t, dc) = (p.s3, p.dc);
    let (mut cs, mut ce, mut cb) = (vec![0i64; m], vec![0i64; m], vec![0i64; m]);
    let mut at = 0i64;
    for k in 0..m {
        cs[k] = at;
        // U columns hold their rows by descending U position: [tail rows (≥ t) | prefix rows]
        cb[k] = at + p.ucol[k].iter().filter(|e| p.prow[e.0] >= t as i64).count() as i64;
        for &(i, u) in &p.ucol[k] {
            pidx.set((colbase + at) as usize, step(i));
            pval.set((colbase + at) as usize, u);
            at += 1;
        }
        ce[k] = at;
    }

    // 05b257 C:2700–2715: F+0xe8 = the highest empty U column above t (t when none)
    let mut e8 = m - 1;
    while e8 > t && ce[e8] != cs[e8] {
        e8 -= 1;
    }
    // 05b257 C:2595–2690: F+0xf8 = −1 below F+0xdc, the rows [F+0xdc, t) listed in position order, −2 in the tail
    let f8: Vec<i32> = (0..m).map(|k| if k < dc { -1 } else if k < t { (k - dc) as i32 } else { -2 }).collect();
    let f0: Vec<i32> = (dc..t.max(dc)).map(|k| k as i32).collect();
    // 036ade C:5725–5740: mean U row length + 1 of the tail rows and of the rows in [F+0xdc, t), 1.5 when empty
    let mean = |a: usize, b: usize| if a < b { (re[b - 1] - rs[a]) as f64 / (b - a) as f64 + 1.0 } else { 1.5 };
    let sc = Scalars {
        xd8: p.s1 as i32,
        xdc: dc as i32,
        xe4: p.s4 as i32,
        xe8: e8 as i32,
        x100: t as i32,
        x104: p.sparse_mult as i32,
        x108: mean(t, m),
        x110: mean(dc, t),
        x128,
        x184: 0,
        x188: alloc.x188,
        x18c: 0,
        x158: alloc.x158,
        x160,
        x168: alloc.x158 - 1,
        x1f8: alloc.x1f8,
        pool_mod16: alloc.pool_mod16,
        x140,
        x1e0: (m + lnnz) as i64,
    };
    let state = LuState {
        m,
        zero,
        header: header.to_vec(),
        x20,
        x28: ident.clone(),
        x30,
        x38: ident.clone(),
        x40: ident.clone(),
        x48: ident,
        x40_is_x28: true,
        x48_is_x38: true,
        diag,
        lbeg,
        lrows,
        lvals,
        llev,
        llst,
        rbeg,
        rcols,
        rvals,
        rlev,
        rlst,
        bucket,
        rc: re.clone(),
        cs,
        ce,
        cb,
        rs,
        re,
        colbase,
        pidx,
        pval,
        sc,
        f0,
        f8,
        oldest: vec![2_000_000_000; m],
        etas: Vec::new(),
        // the spike value buffer is zero off its list (the C invariant of docs/m3/primal_push_lu_rust.md §2)
        spike: prev.map_or_else(
            || {
                let mut val = Cells::unknown(m);
                for i in 0..m {
                    val.set(i, 0.0);
                }
                Spike { count: 0, idx: Cells::unknown(m), val }
            },
            |s| s.spike.clone(),
        ),
        cache: prev.map_or_else(|| Cache { count: 0, idx: Cells::unknown(m), val: Cells::unknown(m) }, |s| s.cache.clone()),
        x1d8: prev.map_or(-1, |s| s.x1d8),
        mark: prev.map_or_else(|| vec![0; m], |s| s.mark.clone()),
        vals: prev.map_or_else(|| vec![0.0; m], |s| s.vals.clone()),
    };
    Ok(Refactor { state, o, n: nn, dense })
}
