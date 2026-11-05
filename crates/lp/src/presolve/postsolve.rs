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
}

fn r8(x: i64) -> i64 {
    (x + 7) & !7
}

/// (allocation size, byte offsets of the value arrays) of a record in the arena (`ac68d7` request of its push).
/// shortcut: `ParallelCol` (type 4, `509fd0` -> `4f9fc2`) is not decoded yet; size 0.
fn layout(r: &PostRec) -> (i64, [i64; 2]) {
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
        PostRec::ParallelCol { .. } => (0, [0, 0]),
        PostRec::ParallelRow { .. } => (0x28, [0, 0]),
        PostRec::SplitRow { entries, .. } => {
            let n = entries.len() as i64;
            (r8(4 * n) + 8 * n + 0x50, [0, 0])
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
        }
    }
}

impl Post {
    pub fn new(st: &State) -> Post {
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
        // shortcut: the arena start parity is assumed 0 and no foreign allocation is modelled; the oracle's
        // `ptr` values show where that fails (docs/m1_postsolve.md §6).
        let mut base = 0i64;
        let mut aligns = Vec::with_capacity(st.post.len());
        for r in &st.post {
            let (size, off) = layout(r);
            aligns.push([((base + off[0]) & 8) as u8, ((base + off[1]) & 8) as u8]);
            base += size;
        }
        Post { recs: st.post.clone(), aligns, col_map, row_map, n_base, n_pre: n_base + st.splits.len(), m_pre }
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
                if y <= 0.0 && y != 0.0 {
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
            PostRec::ParallelCol { .. } => panic!("uncrush of type 4 (parallel columns) is not ported yet"),
        }
    }
    // `a8ff5e`: slack = rhs - A x, accumulated column by column (multiply, then add)
    let mut sl = vec![0f64; m];
    for j in 0..n {
        let b = orig.col_beg[j] as usize;
        for k in b..b + orig.col_len[j] as usize {
            let i = orig.col_ind[k] as usize;
            sl[i] = x[j] * orig.col_val[k] + sl[i];
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
