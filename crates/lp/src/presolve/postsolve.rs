//! LP postsolve (uncrush) `363daa`, bit for bit (docs/m1_postsolve.md). Input: the presolve records and maps, the
//! presolved simplex solution (x, dj, status; x/dj of the rows are slack and dual). Output: basis, x (columns, then
//! slack) and dual (d, then y) of the original model. Only the LP simplex path (spx and basis given) is ported.
//!
//! The original sums with 8-wide SIMD loops whose order depends on whether a record's value array starts on a
//! 16-byte boundary. The records live in a bump arena without alignment, so `Post::new` replays the push sizes to
//! get each array's parity (`aligns`); a caller may override them (the oracle gives the real addresses).
use super::state::{PostRec, State};
use rsolver_model::Model;

pub struct Post {
    /// push order, oldest first (the original walks its list newest first)
    pub recs: Vec<PostRec>,
    /// per record: parity (0 or 8) of the address of up to two value arrays (`ptr & 0xf`)
    pub aligns: Vec<[u8; 2]>,
    /// original column / row -> presolved index, or the state's negative removal code
    pub col_map: Vec<i32>,
    pub row_map: Vec<i32>,
    /// presolved columns without the range columns of split rows, with them, and presolved rows
    pub n_base: usize,
    pub n_pre: usize,
    pub m_pre: usize,
    /// post+0x128: INFBOUND at presolve time
    pub infbound: f64,
}

fn r8(x: i64) -> i64 {
    (x + 7) & !7
}

/// (allocation size, byte offsets of the value arrays) of a record in the arena (`ac68d7` request of its push).
pub fn layout(r: &PostRec) -> (i64, [i64; 2]) {
    match r {
        PostRec::FixCol { entries, .. } => {
            let n = entries.len() as i64;
            (0x48 + r8(4 * n) + 8 * n, [0x48 + r8(4 * n), 0])
        }
        PostRec::Subst { row, col, .. } => {
            let (nr, nc) = (row.len() as i64, col.len() as i64);
            let a = 0x68 + r8(4 * nr);
            let mut size = a + 8 * nr;
            let v2 = size + r8(4 * nc);
            if nc >= 1 {
                size = v2 + 8 * nc;
            }
            (size, [a, v2])
        }
        PostRec::DualFix { rows, terms, .. } => {
            let cnt = rows.len() as i64;
            let k: i64 = terms.iter().map(|t| t.len() as i64).sum();
            let (u20, u22, u27) = ((4 * cnt + 3) & !7, (4 * cnt - 1) & !7, (4 * k - 1) & !7);
            (u20 + u22 + 8 * cnt + u27 + 8 * k + 0x70, [0x70 + u20 + u22 + u27, 0])
        }
        PostRec::SingletonRow { .. } => (0x38, [0, 0]),
        PostRec::Forcing { entries, .. } => {
            let n = entries.len() as i64;
            (r8(4 * n) + 8 * n + 0x38, [0, 0])
        }
        PostRec::ParallelCol { .. } => (0x50, [0, 0]),
        PostRec::ParallelRow { .. } => (0x28, [0, 0]),
        PostRec::SplitRow { entries, .. } => {
            let n = entries.len() as i64;
            (r8(4 * n) + 8 * n + 0x50, [0, 0])
        }
        // aggregation records (docs/m1_presolve_agg.md §9); the value array of type 21 is at 0x90 + Q
        PostRec::AggBound { .. } => (0x50, [0, 0]),
        PostRec::SlackCol { .. } => (0x28, [0, 0]),
        PostRec::PairAgg { n1, n2, .. } => {
            let lt = (n1 + n2) as i64;
            let q = (4 * lt - 1) & !7;
            (q + 8 * lt + 0x90, [0x90 + q, 0])
        }
    }
}

impl PostRec {
    /// the original's record type number
    pub fn kind(&self) -> i32 {
        match self {
            PostRec::FixCol { .. } => 1,
            PostRec::Subst { .. } => 2,
            PostRec::DualFix { .. } => 3,
            PostRec::ParallelCol { .. } => 4,
            PostRec::ParallelRow { .. } => 5,
            PostRec::SingletonRow { .. } => 7,
            PostRec::Forcing { .. } => 8,
            PostRec::SplitRow { .. } => 14,
            PostRec::AggBound { .. } => 6,
            PostRec::SlackCol { .. } => 11,
            PostRec::PairAgg { .. } => 21,
        }
    }
}

impl Post {
    pub fn new(st: &State, infbound: f64, nnz0: i64) -> Post {
        let map = |cnt: &[i32]| {
            let mut k = 0;
            cnt.iter()
                .map(|&c| {
                    if c < 0 {
                        c
                    } else {
                        k += 1;
                        k - 1
                    }
                })
                .collect::<Vec<i32>>()
        };
        let (col_map, row_map) = (map(&st.col_cnt), map(&st.row_cnt));
        let n_base = col_map.iter().filter(|&&c| c >= 0).count();
        let m_pre = row_map.iter().filter(|&&c| c >= 0).count();
        // The records come from a bump arena (`ac68d7`) with no alignment: chunk bases are 16-aligned, the first
        // chunk has `nnz/5 + 10` bytes (3a484d), a request that does not fit starts a chunk of
        // max(next, request) bytes and next = max(first, min(64K, 2 * that)). Only the records allocate from it
        // (checked against the oracle's record addresses for the 58 Netlib models that reach uncrush).
        let first = nnz0 / 5 + 10;
        let (mut cap, mut used, mut next) = (first, 0i64, first);
        let mut aligns = Vec::with_capacity(st.post.len());
        for r in &st.post {
            let (size, off) = layout(r);
            if size > cap - used {
                cap = next.max(size);
                next = first.max((2 * cap).min(0x10000));
                used = 0;
            }
            aligns.push([((used + off[0]) & 8) as u8, ((used + off[1]) & 8) as u8]);
            used += size;
        }
        Post { recs: st.post.clone(), aligns, col_map, row_map, n_base, n_pre: n_base + st.splits.len(), m_pre, infbound }
    }
}

/// `dot` of the original: `init + sum p[k]` in the order of its SIMD loop. `odd`: value array at 8 mod 16.
pub fn simd_sum(init: f64, odd: bool, p: &[f64]) -> f64 {
    let n = p.len();
    let u = odd as usize;
    let mut s = init;
    if n < 8 || n < u + 8 {
        for &v in p {
            s += v;
        }
        return s;
    }
    let end = n - ((n - u) & 7);
    if u == 1 {
        s += p[0];
    }
    let mut l = [0f64; 8];
    l[0] = s;
    let mut k = u;
    while k < end {
        for (i, li) in l.iter_mut().enumerate() {
            *li += p[k + i];
        }
        k += 8;
    }
    s = ((l[0] + l[2]) + (l[4] + l[6])) + ((l[1] + l[3]) + (l[5] + l[7]));
    for &v in &p[end..] {
        s += v;
    }
    s
}

/// Presolved solution as `363daa` reads it after its own unscaling: x, dj and status of the columns, then rows.
pub struct PreSol<'a> {
    pub x: &'a [f64],
    pub dj: &'a [f64],
    pub st: &'a [i32],
}

pub struct Uncrushed {
    pub basis: Vec<i32>,
    pub x: Vec<f64>,
    pub dual: Vec<f64>,
}

/// `363daa` on the LP simplex path. `orig` is the model presolve ran on (for the final slack and the `>` sign
/// of y). `aligns` overrides `post.aligns` when given.
pub fn uncrush(post: &Post, orig: &Model, pre: &PreSol, aligns: Option<&[[u8; 2]]>) -> Uncrushed {
    let (n, m) = (orig.num_vars(), orig.num_constrs());
    let al = aligns.unwrap_or(&post.aligns);
    let (n_pre, n_base) = (post.n_pre, post.n_base);
    let mut x = vec![0f64; n + m];
    let mut dual = vec![0f64; n + m];
    let mut bs = vec![0i32; n + m];
    // basis positions of the presolved model are named by the original row they sit in
    let mut pos = vec![0i32; post.m_pre];
    for (i, &k) in post.row_map.iter().enumerate() {
        if k >= 0 {
            pos[k as usize] = i as i32;
        }
    }
    let conv = |s: i32| if s == -6 { -3 } else if s < 0 { s } else { pos[s as usize] };
    for j in 0..n {
        let cm = post.col_map[j];
        if cm < 0 {
            bs[j] = -3 - cm.min(-2);
        } else {
            let c = cm as usize;
            x[j] = pre.x[c];
            dual[j] = pre.dj[c];
            bs[j] = conv(pre.st[c]);
        }
    }
    for i in 0..m {
        let rm = post.row_map[i];
        if rm < 0 {
            if rm < -2 {
                bs[n + i] = -1;
            } else {
                bs[n + i] = i as i32;
            }
        } else {
            let c = n_pre + rm as usize;
            dual[n + i] = pre.dj[c];
            let s = pre.st[c];
            if s < 0 {
                bs[n + i] = s;
            } else {
                bs[n + i] = pos[s as usize];
                x[n + i] = pre.x[c];
            }
        }
    }
    for (ri, r) in post.recs.iter().enumerate().rev() {
        let a = al[ri];
        match r {
            PostRec::FixCol { j, v, obj, status, entries } => {
                let j = *j as usize;
                x[j] = *v;
                let p: Vec<f64> = entries.iter().map(|&(i, a)| dual[n + i as usize] * a).collect();
                let d = simd_sum(*obj, a[0] != 0, &p);
                dual[j] = d;
                bs[j] = if *status == -4 { if d >= 0.0 { -1 } else { -2 } } else { *status };
            }
            PostRec::Subst { j, r, c0, a: av, obj, mode, row, col } => {
                let (j, r) = (*j as usize, *r as usize);
                let p: Vec<f64> = row.iter().map(|&(k, v)| v * x[k as usize]).collect();
                x[j] = simd_sum(*c0, a[0] != 0, &p);
                dual[j] = 0.0;
                if *mode < 0 {
                    if bs[n + r] >= 0 {
                        bs[j] = bs[n + r];
                        bs[n + r] = -1;
                        continue;
                    }
                    bs[j] = *mode;
                    let flip = if *mode == -1 { *av < 0.0 } else { *mode == -2 && *av > 0.0 };
                    if flip {
                        dual[n + r] = -dual[n + r];
                    }
                    dual[j] = *av * dual[n + r];
                } else {
                    let p: Vec<f64> = col.iter().map(|&(k, v)| dual[n + k as usize] * v).collect();
                    let s = simd_sum(*obj, a[1] != 0, &p);
                    dual[n + r] = -s / *av;
                }
            }
            PostRec::DualFix { j, dir, bound, rows, terms } => {
                let j = *j as usize;
                let (mut v, mut best, mut start) = (*bound, -1i32, 0usize);
                for (t, &(row, c)) in rows.iter().enumerate() {
                    let p: Vec<f64> = terms[t].iter().map(|&(k, w)| w * x[k as usize]).collect();
                    let odd = (a[0] as usize + 8 * start) & 8 != 0;
                    let val = simd_sum(c, odd, &p);
                    start += terms[t].len();
                    if if *dir == 0 { val < v } else { v < val } {
                        best = row;
                        v = val;
                    }
                }
                x[j] = v;
                if best < 0 {
                    bs[j] = if *dir != 0 { -1 } else { -2 };
                } else {
                    let b = best as usize;
                    bs[j] = bs[n + b];
                    bs[n + b] = -1;
                    x[n + b] = 0.0;
                }
            }
            PostRec::SingletonRow { j, r, code, a: av, x: xr } => {
                let (j, r) = (*j as usize, *r as usize);
                let s = bs[j];
                if code & 8 != 0 && (x[j] - xr).abs() > 1e-6 {
                    continue;
                }
                let hit = match s {
                    -1 => code & 1 != 0,
                    -2 => code & 2 != 0,
                    _ => false,
                };
                if hit {
                    dual[n + r] = -dual[j] / *av;
                    dual[j] = 0.0;
                    bs[j] = bs[n + r];
                    bs[n + r] = -1;
                }
            }
            PostRec::Forcing { r, dir, entries } => {
                let r = *r as usize;
                let (mut t, mut best) = (0.0f64, -1i32);
                for &(k, v) in entries {
                    let p = v * *dir;
                    let k = k as usize;
                    let dk = dual[k];
                    if p <= 0.0 {
                        bs[k] = -2;
                        if dk > 0.0 && t * p < dk {
                            t = dk / p;
                            best = k as i32;
                        }
                    } else {
                        bs[k] = -1;
                        if dk < 0.0 && dk < t * p {
                            t = dk / p;
                            best = k as i32;
                        }
                    }
                }
                if best >= 0 {
                    bs[best as usize] = bs[n + r];
                    bs[n + r] = -1;
                    dual[n + r] = -(*dir * t);
                    let td = t * *dir;
                    let n4 = entries.len() / 4 * 4;
                    for (q, &(k, v)) in entries.iter().enumerate() {
                        dual[k as usize] -= if q < n4 { v * td } else { v * t * *dir };
                    }
                }
            }
            PostRec::ParallelRow { a: ra, b, ratio } => {
                let (ra, b) = (*ra as usize, *b as usize);
                let y = dual[n + ra];
                if y < 0.0 {
                    bs.swap(n + ra, n + b);
                    dual[n + b] = y / *ratio;
                    dual[n + ra] = 0.0;
                }
            }
            PostRec::SplitRow { a: ra, b, k, ratio, .. } => {
                let (ra, b) = (*ra as usize, *b as usize);
                let s = pre.st[n_base + *k as usize];
                if s >= 0 {
                    bs[n + ra] = 1;
                    bs[n + b] = 1;
                    dual[n + ra] = 0.0;
                    dual[n + b] = 0.0;
                } else if s == -1 {
                    bs[n + b] = 1;
                    dual[n + b] = 0.0;
                } else {
                    dual[n + b] = dual[n + ra] * *ratio;
                    dual[n + ra] = 0.0;
                    if bs[n + ra] < 0 {
                        bs[n + b] = -1;
                    }
                    bs[n + ra] = 1;
                }
            }
            PostRec::ParallelCol { x: xc, y: yc, lx, ly, ux, uy, r } => {
                parallel_col(&mut x, &mut dual, &mut bs, (*xc as usize, *yc as usize), [*lx, *ly, *ux, *uy, *r], post.infbound)
            }
            PostRec::AggBound { code, e, k, r: rr, ae, ak, .. } => {
                // `363daa` case 6, basis path. shortcut: code bit 8 (col_flag 0x80, PWL) is not ported.
                let (e, k, rr) = (*e as usize, *k as usize, *rr as usize);
                let s = bs[k];
                let hit = match s {
                    -1 => code & 1 != 0,
                    -2 => code & 2 != 0,
                    _ => false,
                };
                if hit {
                    dual[n + rr] -= dual[k] / *ak;
                    dual[e] = (-dual[k] * *ae) / *ak;
                    dual[k] = 0.0;
                    bs[k] = bs[e];
                    let prod = *ae * *ak;
                    let to_upper = if prod <= 0.0 { s != -1 } else { s == -1 };
                    bs[e] = if to_upper { -2 } else { -1 };
                }
            }
            PostRec::SlackCol { r: rr, j, a: av } => {
                let (rr, j) = (*rr as usize, *j as usize);
                x[n + rr] = *av * x[j];
                bs[n + rr] = bs[j];
                bs[j] = 0;
            }
            PostRec::PairAgg { r: rr, e1, e2, k1, k2, n1, n2, f1, f2, a1, a2, obj1, obj2, ub1, ub2, entries } => {
                let (rr, e1, e2, k1, k2) = (*rr as usize, *e1 as usize, *e2 as usize, *k1 as usize, *k2 as usize);
                x[e1] = *f1 * x[k1];
                x[e2] = *f2 * x[k2];
                let prods: Vec<f64> = entries.iter().map(|&(i, v)| dual[n + i as usize] * v).collect();
                let n1 = *n1 as usize;
                let _ = n2;
                let s1 = simd_sum(*obj1, a[0] != 0, &prods[..n1]);
                let s2 = simd_sum(*obj2, (a[0] as usize + 8 * n1) & 8 != 0, &prods[n1..]);
                let (x1, x2) = (x[e1], x[e2]);
                let which: i32 = if x1 >= 1e-6 || x2 >= 1e-6 {
                    if a2.abs() * x2 <= a1.abs() * x1 { 1 } else { -1 }
                } else if dual[k1] >= 1e-6 {
                    -((dual[k2] < 1e-6) as i32)
                } else {
                    1
                };
                let status = |v: f64, ub: f64| if v < 1e-6 { -1 } else if ub - 1e-6 < v { -2 } else { -3 };
                if which == 1 {
                    dual[n + rr] = -s1 / *a1;
                    dual[e1] = 0.0;
                    let d2 = s2 + *a2 * dual[n + rr];
                    dual[e2] = d2;
                    dual[k2] -= *f2 * d2;
                    bs[e2] = status(x[e2], *ub2);
                } else if which == -1 {
                    dual[n + rr] = -s2 / *a2;
                    dual[e2] = 0.0;
                    let d1 = s1 + *a1 * dual[n + rr];
                    dual[e1] = d1;
                    dual[k1] -= *f1 * d1;
                    bs[e1] = status(x[e1], *ub1);
                } else {
                    let (t1, t2) = (s1 / a1.abs(), s2 / a2.abs());
                    let (mut pick, mut best) = if t1 <= t2 { (e1, t1) } else { (e2, t2) };
                    let v3 = (dual[k1] - s1 * *f1) / *f1;
                    let t3 = v3 / a1.abs();
                    if t3 < best {
                        pick = k1;
                        best = t3;
                    }
                    let v4 = (dual[k2] - s2 * *f2) / *f2;
                    if v4 / a2.abs() < best {
                        pick = k2;
                    }
                    let y = if pick == e1 {
                        -s1 / *a1
                    } else if pick == e2 {
                        -s2 / *a2
                    } else if pick == k1 {
                        v3 / *a1
                    } else {
                        v4 / *a2
                    };
                    dual[n + rr] = y;
                    dual[e1] = *a1 * y + s1;
                    dual[e2] = *a2 * y + s2;
                    dual[k1] += (-s1 - *a1 * y) * *f1;
                    dual[k2] += (-s2 - *a2 * y) * *f2;
                    bs[e1] = -1;
                    bs[e2] = -1;
                    bs[pick] = rr as i32;
                }
            }
        }
    }
    // `a8ff5e`: slack = rhs - A x, accumulated column by column (multiply, then add)
    let mut sl = vec![0f64; m];
    for (j, &xj) in x.iter().enumerate().take(n) {
        let b = orig.col_beg[j] as usize;
        for k in b..b + orig.col_len[j] as usize {
            let i = orig.col_ind[k] as usize;
            sl[i] += xj * orig.col_val[k];
        }
    }
    for i in 0..m {
        if orig.row_sense[i] == b'>' {
            // the original holds `>` rows negated and flips the slack back; this only matters for the sign of 0
            let s_neg = if sl[i] == 0.0 { 0.0 } else { -sl[i] };
            x[n + i] = -((-orig.rhs[i]) - s_neg);
            dual[n + i] = -dual[n + i];
        } else {
            x[n + i] = orig.rhs[i] - sl[i];
        }
    }
    Uncrushed { basis: bs, x, dual }
}

/// `35d235` for type 4 (x' = x + r y was merged into column x; split the merged value back into x and y). The
/// integer-column branches (record flag at +0x30 = 1 or 2) are not ported: LP columns carry flag 0.
/// shortcut: the basis argument is assumed to be passed (the decompile of the call drops it); fit1d confirms.
fn parallel_col(x: &mut [f64], dual: &mut [f64], bs: &mut [i32], (xc, yc): (usize, usize), rec: [f64; 5], tol: f64) {
    let [a, b, c, d, r] = rec;
    let xv = x[xc];
    let mut best = 1e100f64;
    let mut u = 0i32;
    // 0 y at 0, -1 y at b, -2 y at d, -3 x at a, -4 x at c
    let tail = |u: i32| if u != 0 { -u >> 1 } else { 0 };
    // `L572`: x at its upper bound c
    let at_c = |best: f64, u: i32| -> i32 {
        let y = (xv - c) / r;
        if y < b || d < y {
            if b <= y || best <= b - y {
                if y <= d || best <= y - d {
                    return tail(u);
                }
                return tail(8);
            }
            return tail(7);
        }
        -4
    };
    // `L56c`
    let l56c = |best: f64, u: i32| if tol <= c { tail(u) } else { at_c(best, u) };
    if b > -tol {
        let y = xv - r * b;
        if y < a || c < y {
            if a <= y {
                best = y - c;
                u = 2;
            } else {
                u = 1;
                best = a - y;
            }
        } else {
            return finish(x, dual, bs, (xc, yc), rec, -1);
        }
    }
    // `L480`
    let mut early = None;
    if d < tol {
        let y = xv - r * d;
        if a <= y && y <= c {
            early = Some(-2);
        } else if a <= y || best <= a - y {
            if c < y && y - c < best {
                u = 4;
                best = y - c;
            }
        } else {
            u = 3;
            best = a - y;
        }
    }
    let code = if let Some(e) = early {
        e
    } else if a <= -tol {
        l56c(best, u)
    } else {
        let y = (xv - a) / r;
        if y < b || d < y {
            let mut resolved = None;
            let mut w = 0.0;
            if b <= y || {
                w = b - y;
                best <= w
            } {
                if y <= d || {
                    w = y - d;
                    best <= w
                } {
                    resolved = Some(l56c(best, u));
                } else {
                    u = 6;
                }
            } else {
                u = 5;
            }
            match resolved {
                Some(c0) => c0,
                None => {
                    best = w;
                    if c < tol { at_c(best, u) } else { tail(u) }
                }
            }
        } else {
            -3
        }
    };
    finish(x, dual, bs, (xc, yc), rec, code)
}

fn finish(x: &mut [f64], dual: &mut [f64], bs: &mut [i32], (xc, yc): (usize, usize), rec: [f64; 5], code: i32) {
    let [a, b, c, d, r] = rec;
    let xv = x[xc];
    match code {
        0 => x[yc] = 0.0,
        -4 => {
            x[yc] = (xv - c) / r;
            x[xc] = c;
        }
        -3 => {
            x[yc] = (xv - a) / r;
            x[xc] = a;
        }
        -2 => {
            x[yc] = d;
            x[xc] -= r * d;
        }
        -1 => {
            x[yc] = b;
            x[xc] -= r * b;
        }
        _ => unreachable!(),
    }
    dual[yc] = dual[xc] * r;
    match code {
        0 => bs[yc] = -3,
        -1 | -2 => {
            bs[yc] = if code == -1 { -1 } else { -2 };
            if bs[xc] == -3 {
                let v = x[xc];
                if (v - a).abs() < 1e-13 {
                    bs[xc] = -1;
                } else if (v - c).abs() < 1e-13 {
                    bs[xc] = -2;
                }
            }
        }
        _ => {
            let s = bs[xc];
            bs[yc] = if s >= 0 || r > 0.0 {
                s
            } else if s == -1 {
                -2
            } else if s != -2 {
                s
            } else {
                -1
            };
            bs[xc] = if code == -3 { -1 } else { -2 };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::simd_sum;

    #[test]
    fn simd_sum_order() {
        let p: Vec<f64> = (1..=9).map(|k| 1.0 / k as f64).collect();
        // fewer than 8 terms: plain left to right
        assert_eq!(simd_sum(0.5, false, &p[..7]), p[..7].iter().fold(0.5, |s, v| s + v));
        // 8 terms, aligned: ((s0+s2)+(s4+s6)) + ((s1+s3)+(s5+s7)), s0 carrying init
        let s0 = 0.5 + p[0];
        assert_eq!(simd_sum(0.5, false, &p[..8]), ((s0 + p[2]) + (p[4] + p[6])) + ((p[1] + p[3]) + (p[5] + p[7])));
        // 9 terms at 8 mod 16: the first term goes in alone, then 8 in the vector loop
        let s0 = 0.5 + p[0] + p[1];
        assert_eq!(simd_sum(0.5, true, &p), ((s0 + p[3]) + (p[5] + p[7])) + ((p[2] + p[4]) + (p[6] + p[8])));
    }
}
