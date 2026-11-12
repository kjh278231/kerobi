//! Support-round BTRAN `221173` of the crossover push (cache F+0x1d0) on the original factor layout, with the sparse
//! Uᵀ kernel `21a941`: the cache path, dense Uᵀ, dense and sparse row etas, the sparse Lᵀ (heap, level and tail paths
//! with the F+0x20 relabel) and the dense Lᵀ with the F+0x30 gather, and the F+0x200 work in the original's order of
//! additions. The two in/out vectors are the original's four buffers behind two headers whose pointers the original
//! swaps (as in `crossover_ftran`); uncaptured cells (output buffers, the F+0x1a8 list) start unknown. Contract and C
//! references: docs/m3/primal_push_btran_rust.md. `x += y` for the original's `y + x` and `x -= a·b` for its
//! `-(a·b) + x` are bit-identical (IEEE addition commutes, `x + (−p) = x − p`); the association is kept as in C.

use crate::crossover_ftran::{Buffer, Cells, Header, VecSnap};

/// Row eta record of F+0x198 (`{r, len, idx*, val*}`) in creation order.
pub struct BtranEta {
    pub row: usize,
    pub idx: Vec<usize>,
    pub val: Vec<f64>,
}

/// A row of U (F+0xb0/0xb8 over F+0xd0/0xc8) or of L (F+0x70 over F+0x80/0x78) in storage order.
pub struct SparseRow {
    pub cols: Vec<usize>,
    pub vals: Vec<f64>,
}

pub struct BtranFactor {
    pub m: usize,
    /// F+0x10.
    pub zero: f64,
    /// F+0x20 (sparse output relabel `idx := F20[idx]`) and F+0x30 (dense output gather `out[k] = w[F30[k]]`).
    pub relabel: Vec<usize>,
    pub gather: Vec<usize>,
    /// F+0x40 row → U position, F+0x48 U position → row.
    pub upos: Vec<usize>,
    pub uord: Vec<usize>,
    pub diag: Vec<f64>,
    pub urow: Vec<SparseRow>,
    /// F+0xf8 (−1: empty U row, < −1: tail row, ≥ 0: prefix row).
    pub flag: Vec<i32>,
    /// F+0x100, F+0x104, F+0x108, F+0x110.
    pub t: usize,
    pub mult: i32,
    pub f108: f64,
    pub f110: f64,
    /// F+0xd8, F+0xe4 and the L column count F58[m] − F58[e4].
    pub d8: usize,
    pub e4: usize,
    pub ltail: i64,
    pub lrow: Vec<SparseRow>,
    /// F+0x140 (−1 without levels), F+0x130 row → level, F+0x138 level → first slot.
    pub levels: i32,
    pub lev: Vec<usize>,
    pub lst: Vec<usize>,
    /// F+0x1e0, F+0x184.
    pub f1e0: i64,
    pub f184: i32,
    pub etas: Vec<BtranEta>,
}

/// The cache vector F+0x1d0 (its buffers are the original's own, length m).
#[derive(Clone)]
pub struct Cache {
    pub count: i32,
    pub idx: Cells<i32>,
    pub val: Cells<f64>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CacheSnap {
    pub count: i32,
    pub idx: Vec<Option<i32>>,
    pub val: Vec<Option<f64>>,
}

/// Non-zero cells of the scratch F+0x1b0 (marks) and F+0x1b8 (values).
#[derive(Clone, Debug, PartialEq)]
pub struct Scratch {
    pub marks: Vec<(usize, i32)>,
    pub vals: Vec<(usize, f64)>,
}

#[derive(Clone, Debug)]
pub struct BtranKernelEvent {
    pub leave: bool,
    pub input: VecSnap,
    pub output: VecSnap,
    pub alias: [(Buffer, Buffer); 2],
    pub scratch: Scratch,
    pub cache: Option<CacheSnap>,
    pub x1d8: i32,
    pub work: f64,
}

#[derive(Clone, Debug, Default)]
pub struct BtranBranches {
    pub entry: &'static str,
    pub u: &'static str,
    pub kernel: &'static str,
    pub eta: &'static str,
    pub lt: &'static str,
}

pub struct BtranState {
    pub idx: [Cells<i32>; 2],
    pub val: [Cells<f64>; 2],
    pub input: Header,
    pub output: Header,
    pub mark: Vec<i32>,
    pub vals: Vec<f64>,
    /// F+0x1a8[0..m) (level lists, sparse eta list) and F+0x1a8[m..m+F140] (level bucket counts).
    pub list: Cells<i32>,
    pub bucket: Option<Vec<i32>>,
    pub cache: Option<Cache>,
    pub x1d8: i32,
    /// F+0x200.
    pub work: f64,
    pub events: Vec<BtranKernelEvent>,
    pub branches: BtranBranches,
}

fn b(x: Buffer) -> usize {
    x as usize
}

fn big(zero: f64, v: f64) -> bool {
    zero <= v || v <= -zero
}

impl BtranState {
    /// Entry state: the input header owns the caller's input buffers with `count` entries (m when dense), the output
    /// header has the captured count and unknown buffers.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        m: usize,
        count: i32,
        idx: &[i32],
        val: &[f64],
        out_count: i32,
        cache: Option<Cache>,
        x1d8: i32,
        work: f64,
        bucket: Option<Vec<i32>>,
    ) -> BtranState {
        let mut s = BtranState {
            idx: [Cells::unknown(m), Cells::unknown(m)],
            val: [Cells::unknown(m), Cells::unknown(m)],
            input: Header { count: Some(count), idx: Buffer::In, val: Buffer::In },
            output: Header { count: Some(out_count), idx: Buffer::Out, val: Buffer::Out },
            mark: vec![0; m],
            vals: vec![0.0; m],
            list: Cells::unknown(m),
            bucket,
            cache,
            x1d8,
            work,
            events: Vec::new(),
            branches: BtranBranches::default(),
        };
        for (k, &i) in idx.iter().enumerate() {
            s.idx[0].set(k, i);
        }
        for (k, &v) in val.iter().enumerate() {
            s.val[0].set(k, v);
        }
        s
    }

    fn snap(&self, h: Header, m: usize) -> VecSnap {
        let Some(c) = h.count else { return VecSnap { count: None, idx: None, val: Vec::new() } };
        let n = if c < 0 { m } else { c as usize };
        VecSnap {
            count: Some(c),
            idx: (c >= 0).then(|| (0..n).map(|k| self.idx[b(h.idx)].peek(k)).collect()),
            val: (0..n).map(|k| self.val[b(h.val)].peek(k)).collect(),
        }
    }

    pub fn snapshot(&self, m: usize) -> (VecSnap, VecSnap) {
        (self.snap(self.output, m), self.snap(self.input, m))
    }

    pub fn alias(&self) -> [(Buffer, Buffer); 2] {
        [(self.input.idx, self.input.val), (self.output.idx, self.output.val)]
    }

    pub fn scratch(&self) -> Scratch {
        Scratch {
            marks: self.mark.iter().enumerate().filter(|(_, &v)| v != 0).map(|(i, &v)| (i, v)).collect(),
            vals: self.vals.iter().enumerate().filter(|(_, &v)| v.to_bits() != 0).map(|(i, &v)| (i, v)).collect(),
        }
    }

    pub fn cache_snap(&self) -> Option<CacheSnap> {
        self.cache.as_ref().map(|c| {
            let n = c.count.max(0) as usize;
            CacheSnap { count: c.count, idx: (0..n).map(|k| c.idx.peek(k)).collect(), val: (0..n).map(|k| c.val.peek(k)).collect() }
        })
    }

    fn event(&mut self, leave: bool, m: usize) {
        let ev = BtranKernelEvent {
            leave,
            input: self.snap(self.input, m),
            output: if leave { self.snap(self.output, m) } else { VecSnap { count: self.output.count, idx: None, val: Vec::new() } },
            alias: self.alias(),
            scratch: self.scratch(),
            cache: self.cache_snap(),
            x1d8: self.x1d8,
            work: self.work,
        };
        self.events.push(ev);
    }

    fn count(h: Header) -> i32 {
        h.count.expect("read of an unknown count")
    }
}

/// `221173(F, in, out, cache)` with the cache F+0x1d0 given as `s.cache` (None: a null cache argument).
pub fn btran(f: &BtranFactor, s: &mut BtranState) {
    let m = f.m;
    let mut cache_on = s.cache.is_some();
    let mut kernel_path = false;
    if cache_on {
        // C:1197–1208: cache.count := 0; r = in.idx[0]; an empty U row r takes the general path without the cache
        let r = s.idx[b(s.input.idx)].get(0) as usize;
        s.cache.as_mut().unwrap().count = 0;
        if f.urow[r].cols.is_empty() {
            cache_on = false;
            s.branches.entry = "cache-empty-row";
        } else {
            s.x1d8 = r as i32;
            kernel_path = true;
            s.branches.entry = "cache";
        }
    } else {
        s.branches.entry = "general";
    }
    if !kernel_path {
        // C:86–90 (asm 0x12dee6–0x12df33): (int)(F108 + F110)·count ≤ m
        let c = BtranState::count(s.input);
        kernel_path = c >= 0 && ((f.f108 + f.f110) as i32).wrapping_mul(c) <= m as i32;
    }
    if !kernel_path {
        u_dense(f, s);
        return;
    }
    s.branches.u = "21a941";
    kernel(f, s);
    let oc = BtranState::count(s.output);
    if oc == 0 {
        // C:1209–1211
        s.branches.eta = "kernel-out0";
        return;
    }
    if cache_on {
        // C:1213–1227
        let (oi, ov) = (b(s.output.idx), b(s.output.val));
        let mut n = 0;
        if oc > 0 {
            for k in 0..oc as usize {
                let (i, v) = (s.idx[oi].get(k), s.val[ov].get(k));
                let c = s.cache.as_mut().unwrap();
                c.idx.set(k, i);
                c.val.set(k, v);
            }
            n = oc;
        }
        s.cache.as_mut().unwrap().count = n;
    }
    if f.etas.is_empty() {
        s.branches.eta = "none";
        let (i37, v44, v26) = (s.input.idx, s.input.val, s.output.val);
        swap_ec37(f, s, i37, v44, v26, oc);
        return;
    }
    if oc >= 0 {
        if (m as i32) < f.mult * oc {
            // C:1237–1352: scatter into the input's values, dense etas, dense Lᵀ
            s.branches.eta = "dense";
            let (v26, i35, v44) = (s.input.val, s.output.idx, s.output.val);
            if oc > 0 {
                for i in 0..m {
                    s.val[b(v26)].set(i, 0.0);
                }
                for k in 0..oc as usize {
                    let i = s.idx[b(i35)].get(k) as usize;
                    let v = s.val[b(v44)].get(k);
                    s.val[b(v26)].set(i, v);
                }
            }
            dense_etas(f, s, v26);
            s.input.count = Some(-1);
            let i16 = s.input.idx;
            let dv = f.f184 as f64 * 2.0 + s.work;
            l_dense(f, s, -1, i16, v26, i35, v44, dv);
        } else {
            sparse_etas(f, s, oc);
        }
    } else {
        s.branches.eta = "dense";
        let (i37, v44, v26) = (s.input.idx, s.input.val, s.output.val);
        swap_e394(f, s, i37, v44, v26, oc);
    }
}

/// `LAB_0022ec37`: the output header takes (input count, i37, v44), the input header (count, old output idx, v26);
/// then Lᵀ from the input header.
fn swap_ec37(f: &BtranFactor, s: &mut BtranState, i37: Buffer, v44: Buffer, v26: Buffer, count: i32) {
    let i16 = s.output.idx;
    s.output = Header { count: s.input.count, idx: i37, val: v44 };
    s.input = Header { count: Some(count), idx: i16, val: v26 };
    l_transpose(f, s, count, i16, v26);
}

/// `LAB_0022e394`: header exchange of the dense eta entry, dense etas on v26, dense Lᵀ.
fn swap_e394(f: &BtranFactor, s: &mut BtranState, i37: Buffer, v44: Buffer, v26: Buffer, count: i32) {
    s.input.count = Some(count);
    let i16 = s.output.idx;
    s.output.idx = i37;
    s.output.val = v44;
    s.input.idx = i16;
    s.input.val = v26;
    s.output.count = Some(-1);
    dense_etas(f, s, v26);
    s.input.count = Some(-1);
    let (i35, v44) = (s.output.idx, s.output.val);
    let dv = f.f184 as f64 * 2.0 + s.work;
    l_dense(f, s, -1, i16, v26, i35, v44, dv);
}

/// Dense Uᵀ (C:30–160): U positions 0 → m−1, x := +0 when |x| ≤ ZERO or |x/d| ≤ ZERO, else x/d and its U row.
fn u_dense(f: &BtranFactor, s: &mut BtranState) {
    let m = f.m;
    s.branches.u = "dense";
    let c = BtranState::count(s.input);
    let (i37, v44, v26);
    if c < 1 {
        std::mem::swap(&mut s.input, &mut s.output);
        i37 = s.input.idx;
        v26 = s.output.val;
        v44 = s.input.val;
    } else {
        i37 = s.input.idx;
        s.work += c as f64 * 2.0 + m as f64;
        let (ii, iv, ov) = (b(s.input.idx), b(s.input.val), b(s.output.val));
        for i in 0..m {
            s.val[ov].set(i, 0.0);
        }
        for k in 0..c as usize {
            let i = s.idx[ii].get(k) as usize;
            let v = s.val[iv].get(k);
            s.val[ov].set(i, v);
        }
        v26 = s.output.val;
        v44 = s.input.val;
    }
    s.work += m as f64 + m as f64;
    let w = b(v26);
    for p in 0..m {
        let r = f.uord[p];
        let x = s.val[w].get(r);
        let d = f.diag[r];
        if x.abs() <= f.zero || (x / d).abs() <= f.zero {
            s.val[w].set(r, 0.0);
            continue;
        }
        let row = &f.urow[r];
        s.work += row.cols.len() as f64 * 2.0;
        let y = s.val[w].get(r) / d;
        s.val[w].set(r, y);
        for (&col, &u) in row.cols.iter().zip(&row.vals) {
            let x = s.val[w].get(col);
            s.val[w].set(col, x - u * y);
        }
    }
    s.output.count = Some(-1);
    if f.etas.is_empty() {
        s.branches.eta = "none";
        swap_ec37(f, s, i37, v44, v26, -1);
    } else {
        s.branches.eta = "dense";
        swap_e394(f, s, i37, v44, v26, -1);
    }
}

/// Dense etas (newest first) on the values of buffer `w` (C:1316–1347, 1508–1566): v = w[r]; |v| ≤ ZERO clears w[r]
/// and skips the eta, else w[r] = η_r·v and w[i] = η_i·v + w[i].
fn dense_etas(f: &BtranFactor, s: &mut BtranState, w: Buffer) {
    let w = b(w);
    for e in f.etas.iter().rev() {
        let v = s.val[w].get(e.row);
        if v.abs() <= f.zero {
            s.val[w].set(e.row, 0.0);
            continue;
        }
        for (&i, &x) in e.idx.iter().zip(&e.val) {
            let y = if i == e.row { x * v } else { x * v + s.val[w].get(i) };
            s.val[w].set(i, y);
        }
    }
}

/// Sparse etas (C:1384–1494) from the kernel output (output header, `oc` > 0) into the input header, then Lᵀ.
fn sparse_etas(f: &BtranFactor, s: &mut BtranState, oc: i32) {
    s.branches.eta = "sparse";
    let (oi, ov) = (b(s.output.idx), b(s.output.val));
    let mut n = oc as usize;
    for k in 0..n {
        let r = s.idx[oi].get(k);
        s.list.set(k, r);
        s.mark[r as usize] = 1;
        s.vals[r as usize] = s.val[ov].get(k);
    }
    let ne = f.etas.len() as f64;
    let mut dv = (ne + ne) + oc as f64 * 5.0;
    for e in f.etas.iter().rev() {
        let v = s.vals[e.row];
        if v == 0.0 {
            continue;
        }
        dv += e.idx.len() as f64 * 3.0;
        for (&i, &x) in e.idx.iter().zip(&e.val) {
            if i == e.row {
                s.vals[i] = x * v;
            } else {
                let mk = s.mark[i];
                s.vals[i] += x * v;
                if mk == 0 {
                    s.mark[i] = 1;
                    s.list.set(n, i as i32);
                    n += 1;
                }
            }
        }
    }
    let (pi, pv) = (b(s.input.idx), b(s.input.val));
    let mut cnt = 0;
    for k in 0..n {
        let r = s.list.get(k) as usize;
        let v = s.vals[r];
        if f.zero <= v.abs() {
            s.idx[pi].set(cnt, r as i32);
            s.val[pv].set(cnt, v);
            cnt += 1;
        }
        s.vals[r] = 0.0;
        s.mark[r] = 0;
    }
    s.input.count = Some(cnt as i32);
    s.work += (dv + (n as i32 - oc) as f64 * 2.0) + n as f64 * 3.0;
    if cnt == 0 {
        s.output.count = Some(0);
        return;
    }
    let (i16, v26) = (s.input.idx, s.input.val);
    l_transpose(f, s, cnt as i32, i16, v26);
}

/// `LAB_0022ec5a`: Lᵀ of the input (count, i16, v26) into the output header.
fn l_transpose(f: &BtranFactor, s: &mut BtranState, count: i32, i16: Buffer, v26: Buffer) {
    let m = f.m;
    if count >= 0 && f.mult * count <= m as i32 && (f.levels as f64) < m as f64 * 0.1 {
        l_sparse(f, s, count, i16, v26);
        return;
    }
    let (i35, v44) = (s.output.idx, s.output.val);
    let dv = s.work;
    l_dense(f, s, count, i16, v26, i35, v44, dv);
}

fn emit(s: &mut BtranState, oi: usize, ov: usize, n: &mut usize, r: usize, v: f64) {
    s.idx[oi].set(*n, r as i32);
    s.val[ov].set(*n, v);
    *n += 1;
}

fn scatter(s: &mut BtranState, row: &SparseRow, v: f64) {
    for (&c, &x) in row.cols.iter().zip(&row.vals) {
        s.vals[c] -= x * v;
    }
}

/// Sparse Lᵀ (C:241–1186): heap (F140 < 0), level (F140 ≥ 0) or tail path B, then the F+0x20 relabel.
fn l_sparse(f: &BtranFactor, s: &mut BtranState, c: i32, i16: Buffer, v26: Buffer) {
    let m = f.m;
    if c == 0 {
        s.output.count = Some(0);
        return;
    }
    let zero = f.zero;
    let (ii, iv, oi, ov) = (b(i16), b(v26), b(s.output.idx), b(s.output.val));
    let c = c as usize;
    let d = m as i32 - f.e4 as i32;
    let tail = f.ltail as f64;
    let (mut cnt_ge, d53, d55) = (0i32, 0.0f64, 0.0f64);
    let (d53, d55) = if d < 6 {
        if d != 0 { (tail * 0.5 / d as f64, d as f64) } else { (d53, d55) }
    } else {
        let d10 = d as f64 * 10.0;
        if (m as f64) < d10 * d as f64 && (f.d8 as i32) < d * 2 && d10 < tail {
            cnt_ge = (0..c).filter(|&k| s.idx[ii].get(k) >= f.e4 as i32).count() as i32;
        }
        (tail * 0.5 / d as f64, d as f64)
    };
    let mut n = 0usize;
    let dv;
    if cnt_ge as f64 * d53 * d53 <= d55 {
        if f.levels < 0 {
            s.branches.lt = "heap";
            // input rows into the max-heap at the top of the output index buffer, h[j] = idx[m−1−j]
            for k in 0..c {
                let r = s.idx[ii].get(k);
                s.idx[oi].set(m - 1 - k, r);
                s.mark[r as usize] = 1;
                s.vals[r as usize] = s.val[iv].get(k);
            }
            let h = |s: &BtranState, j: usize| s.idx[oi].get(m - 1 - j);
            let hs = |s: &mut BtranState, j: usize, x: i32| s.idx[oi].set(m - 1 - j, x);
            let mut hn = c;
            for i in 1..hn {
                let x = h(s, i);
                let mut hole = i;
                while hole > 0 {
                    let p = (hole - 1) / 2;
                    if x < h(s, p) {
                        break;
                    }
                    let y = h(s, p);
                    hs(s, hole, y);
                    hole = p;
                }
                hs(s, hole, x);
            }
            let mut seq = false;
            while hn > 0 {
                let top = h(s, 0);
                if hn != 1 && (top as f64) <= hn as f64 * 6.0 {
                    seq = true;
                    break;
                }
                hn -= 1;
                let last = h(s, hn);
                hs(s, 0, last);
                let mut hole = 0;
                let mut child = 1;
                if 1 < hn {
                    loop {
                        let mut c2 = child;
                        if child + 1 < hn && h(s, child) < h(s, child + 1) {
                            c2 = child + 1;
                        }
                        let y = h(s, c2);
                        if y < last {
                            break;
                        }
                        hs(s, hole, y);
                        child = c2 * 2 + 1;
                        hole = c2;
                        if child >= hn {
                            break;
                        }
                    }
                }
                hs(s, hole, last);
                let top = top as usize;
                let v = std::mem::take(&mut s.vals[top]);
                s.mark[top] = 0;
                if big(zero, v) {
                    emit(s, oi, ov, &mut n, top, v);
                    let row = &f.lrow[top];
                    for (&col, &x) in row.cols.iter().zip(&row.vals) {
                        let mk = s.mark[col];
                        s.vals[col] -= x * v;
                        if mk == 0 {
                            s.mark[col] = 1;
                            let col = col as i32;
                            hs(s, hn, col);
                            let mut hole = hn;
                            while hole > 0 {
                                let p = (hole - 1) / 2;
                                if col < h(s, p) {
                                    break;
                                }
                                let y = h(s, p);
                                hs(s, hole, y);
                                hole = p;
                            }
                            hs(s, hole, col);
                            hn += 1;
                        }
                    }
                }
            }
            if seq {
                s.branches.lt = "heap+seq";
                for j in 0..hn {
                    let r = h(s, j) as usize;
                    s.mark[r] = 0;
                }
                let top = h(s, 0) as usize;
                for i in (0..=top).rev() {
                    let v = std::mem::take(&mut s.vals[i]);
                    if big(zero, v) {
                        emit(s, oi, ov, &mut n, i, v);
                        scatter(s, &f.lrow[i], v);
                    }
                }
            }
            dv = (n as f64 * 7.0 * ((m as f64 + m as f64) + f.f1e0 as f64)) / (m + 1) as f64;
        } else {
            s.branches.lt = "level";
            let levels = f.levels as usize;
            let bucket = s.bucket.as_mut().expect("level path without a bucket");
            for x in bucket.iter_mut().take(levels + 1) {
                *x = 0;
            }
            let mut low = f.levels + 1;
            for k in 0..c {
                let r = s.idx[ii].get(k) as usize;
                let l = f.lev[r];
                let bucket = s.bucket.as_mut().unwrap();
                let slot = f.lst[l] + bucket[l] as usize;
                bucket[l] += 1;
                s.list.set(slot, r as i32);
                s.mark[r] = 1;
                s.vals[r] = s.val[iv].get(k);
                low = low.min(l as i32);
            }
            let mut pending = c as i64;
            let mut j = 0usize;
            loop {
                let l = low as usize + j;
                if s.bucket.as_ref().unwrap()[l] != 0 {
                    let mut pos = f.lst[l];
                    while pos < s.bucket.as_ref().unwrap()[l] as usize + f.lst[l] {
                        let r = s.list.get(pos) as usize;
                        pending -= 1;
                        let v = std::mem::take(&mut s.vals[r]);
                        s.mark[r] = 0;
                        if zero <= v.abs() {
                            emit(s, oi, ov, &mut n, r, v);
                            let row = &f.lrow[r];
                            for (&col, &x) in row.cols.iter().zip(&row.vals) {
                                if s.mark[col] == 0 {
                                    let l2 = f.lev[col];
                                    pending += 1;
                                    s.mark[col] = 1;
                                    let bucket = s.bucket.as_mut().unwrap();
                                    let slot = f.lst[l2] + bucket[l2] as usize;
                                    bucket[l2] += 1;
                                    s.list.set(slot, col as i32);
                                }
                                s.vals[col] -= x * v;
                            }
                        }
                        pos += 1;
                    }
                }
                j += 1;
                if pending <= 0 {
                    break;
                }
            }
            dv = (n as f64 * 5.0 * (f.f1e0 + m as i64 * 2) as f64) / (m + 1) as f64;
        }
    } else {
        // C:957–1146
        s.branches.lt = "B";
        let mut w = c as f64 * 3.0;
        let mut maxr = 0i32;
        let (d8, e4) = (f.d8, f.e4);
        for k in 0..c {
            let r = s.idx[ii].get(k);
            let v = s.val[iv].get(k);
            let ru = r as usize;
            if ru < e4 {
                if ru < d8 {
                    s.vals[ru] += v;
                } else {
                    let row = &f.lrow[ru];
                    w += row.cols.len() as f64 * 2.0;
                    emit(s, oi, ov, &mut n, ru, v);
                    scatter(s, row, v);
                }
            } else {
                maxr = maxr.max(r);
                s.vals[ru] = v;
            }
        }
        w += (((d8 as i32 - e4 as i32) + maxr) as f64) * 2.0;
        if e4 as i32 <= maxr {
            for i in (e4..=maxr as usize).rev() {
                let v = s.vals[i];
                if zero < v.abs() {
                    let row = &f.lrow[i];
                    w += row.cols.len() as f64 * 2.0;
                    emit(s, oi, ov, &mut n, i, v);
                    scatter(s, row, v);
                }
                s.vals[i] = 0.0;
            }
        }
        for i in (0..d8).rev() {
            let v = s.vals[i];
            if zero < v.abs() {
                let row = &f.lrow[i];
                w += row.cols.len() as f64 * 2.0;
                emit(s, oi, ov, &mut n, i, v);
                scatter(s, row, v);
            }
            s.vals[i] = 0.0;
        }
        dv = w;
    }
    s.output.count = Some(n as i32);
    s.work += dv;
    for k in 0..n {
        let i = s.idx[oi].get(k) as usize;
        s.idx[oi].set(k, f.relabel[i] as i32);
    }
}

/// Dense Lᵀ (C:1573–1766): rows m−1 → 0 with ZERO < |w_i|, then `out[k] = w[F30[k]]` and the header exchange.
#[allow(clippy::too_many_arguments)]
fn l_dense(f: &BtranFactor, s: &mut BtranState, c: i32, i16: Buffer, v26: Buffer, i35: Buffer, v44: Buffer, dv: f64) {
    let m = f.m;
    s.branches.lt = "dense";
    let (v15, i39, i35, v44) = if c < 1 {
        s.output.idx = i16;
        s.output.val = v26;
        let c48 = s.output.count;
        s.input.idx = i35;
        s.input.val = v44;
        s.output.count = Some(c);
        s.input.count = c48;
        (s.output.val, s.output.idx, i35, v44)
    } else {
        s.work = (dv + c as f64 * 2.0) + m as f64;
        let (ii, iv, ov) = (b(i16), b(v26), b(v44));
        for i in 0..m {
            s.val[ov].set(i, 0.0);
        }
        for k in 0..c as usize {
            let i = s.idx[ii].get(k) as usize;
            let v = s.val[iv].get(k);
            s.val[ov].set(i, v);
        }
        (v44, i35, i16, v26)
    };
    let dv = if c < 1 { dv } else { s.work };
    s.work = (dv + m as f64) + m as f64;
    let w = b(v15);
    for i in (0..m).rev() {
        let v = s.val[w].get(i);
        if f.zero < v.abs() {
            let row = &f.lrow[i];
            s.work += row.cols.len() as f64 * 2.0;
            for (&col, &x) in row.cols.iter().zip(&row.vals) {
                let y = s.val[w].get(col);
                s.val[w].set(col, y - x * v);
            }
        }
    }
    s.output.count = Some(-1);
    for k in 0..m {
        let v = s.val[w].get(f.gather[k]);
        s.val[b(v44)].set(k, v);
    }
    s.input.count = Some(-1);
    s.output.idx = i35;
    s.output.val = v44;
    s.input.idx = i39;
    s.input.val = v15;
    s.output.count = Some(-1);
}

/// `21a941(ZERO, F, m, F+0x20, in, out, F+0x1b0, F+0x1b8)`: sequential when F108 > 5, else a min-heap over U
/// positions (switching to sequential when m − top ≤ 6·n); outputs ZERO < |x/d|; work into F+0x200.
pub fn kernel(f: &BtranFactor, s: &mut BtranState) {
    let m = f.m;
    s.event(false, m);
    let c = BtranState::count(s.input);
    if c == 0 {
        s.output.count = Some(0);
        s.branches.kernel = "in0";
        s.event(true, m);
        return;
    }
    let zero = f.zero;
    let (ii, iv, oi, ov) = (b(s.input.idx), b(s.input.val), b(s.output.idx), b(s.output.val));
    let mut n = 0usize;
    let mut acc;
    if 5.0 < f.f108 {
        s.branches.kernel = "seq5";
        acc = c as f64 * 3.0;
        let mut low = m;
        for k in 0..c.max(0) as usize {
            let r = s.idx[ii].get(k) as usize;
            let v = s.val[iv].get(k);
            let y = v / f.diag[r];
            if zero < y.abs() {
                if f.flag[r] == -1 {
                    emit(s, oi, ov, &mut n, r, y);
                } else if f.flag[r] < 0 {
                    low = low.min(f.upos[r]);
                    s.vals[r] += v;
                } else {
                    emit(s, oi, ov, &mut n, r, y);
                    let row = &f.urow[r];
                    acc += row.cols.len() as f64 * 2.0;
                    for (&col, &u) in row.cols.iter().zip(&row.vals) {
                        s.vals[col] -= u * y;
                        low = low.min(f.upos[col]);
                    }
                }
            }
        }
        acc += (m - low) as f64 * 2.0;
        kernel_seq(f, s, oi, ov, &mut n, &mut acc, low);
    } else {
        s.branches.kernel = "heap";
        acc = 0.0;
        let h = |s: &BtranState, j: usize| s.idx[oi].get(m - 1 - j);
        let hs = |s: &mut BtranState, j: usize, x: i32| s.idx[oi].set(m - 1 - j, x);
        let mut hn = 0usize;
        for k in 0..c.max(0) as usize {
            let r = s.idx[ii].get(k) as usize;
            let v = s.val[iv].get(k);
            if f.upos[r] < f.t {
                let y = v / f.diag[r];
                if zero < y.abs() {
                    emit(s, oi, ov, &mut n, r, y);
                    let row = &f.urow[r];
                    for (&col, &u) in row.cols.iter().zip(&row.vals) {
                        let mk = s.mark[col];
                        s.vals[col] -= u * y;
                        if mk == 0 {
                            s.mark[col] = 1;
                            hs(s, hn, f.upos[col] as i32);
                            hn += 1;
                        }
                    }
                }
            } else {
                let mk = s.mark[r];
                s.vals[r] += v;
                if mk == 0 {
                    s.mark[r] = 1;
                    hs(s, hn, f.upos[r] as i32);
                    hn += 1;
                }
            }
        }
        for i in 1..hn {
            let x = h(s, i);
            let mut hole = i;
            while hole > 0 {
                let p = (hole - 1) / 2;
                let y = h(s, p);
                if y < x {
                    break;
                }
                hs(s, hole, y);
                hole = p;
            }
            hs(s, hole, x);
        }
        let mut from = m;
        while hn > 0 {
            let top = h(s, 0);
            if ((m as i32 - top) as f64) <= hn as f64 * 6.0 {
                s.branches.kernel = "heap+seq";
                for j in 0..hn {
                    let r = f.uord[h(s, j) as usize];
                    s.mark[r] = 0;
                }
                from = top as usize;
                break;
            }
            hn -= 1;
            let last = h(s, hn);
            hs(s, 0, last);
            let mut hole = 0;
            let mut child = 1;
            if 1 < hn {
                loop {
                    let mut c2 = child;
                    if child + 1 < hn && h(s, child + 1) < h(s, child) {
                        c2 = child + 1;
                    }
                    let y = h(s, c2);
                    if last < y {
                        break;
                    }
                    hs(s, hole, y);
                    child = c2 * 2 + 1;
                    hole = c2;
                    if child >= hn {
                        break;
                    }
                }
            }
            hs(s, hole, last);
            let r = f.uord[top as usize];
            let y = s.vals[r] / f.diag[r];
            if zero < y.abs() {
                let row = &f.urow[r];
                acc += row.cols.len() as f64 * 2.0;
                emit(s, oi, ov, &mut n, r, y);
                for (&col, &u) in row.cols.iter().zip(&row.vals) {
                    let mk = s.mark[col];
                    s.vals[col] -= u * y;
                    if mk == 0 {
                        s.mark[col] = 1;
                        let x = f.upos[col] as i32;
                        hs(s, hn, x);
                        let mut hole = hn;
                        while hole > 0 {
                            let p = (hole - 1) / 2;
                            let y = h(s, p);
                            if y < x {
                                break;
                            }
                            hs(s, hole, y);
                            hole = p;
                        }
                        hs(s, hole, x);
                        hn += 1;
                    }
                }
            }
            s.mark[r] = 0;
            s.vals[r] = 0.0;
        }
        kernel_seq(f, s, oi, ov, &mut n, &mut acc, from);
    }
    s.output.count = Some(n as i32);
    s.work += acc;
    s.event(true, m);
}

/// Kernel sequential sweep over U positions `from..m` (C:340–395, 128–155).
fn kernel_seq(f: &BtranFactor, s: &mut BtranState, oi: usize, ov: usize, n: &mut usize, acc: &mut f64, from: usize) {
    for p in from..f.m {
        let r = f.uord[p];
        let y = s.vals[r] / f.diag[r];
        if f.zero < y.abs() {
            let row = &f.urow[r];
            *acc += row.cols.len() as f64 * 2.0;
            emit(s, oi, ov, n, r, y);
            for (&col, &u) in row.cols.iter().zip(&row.vals) {
                s.vals[col] -= u * y;
            }
        }
        s.vals[r] = 0.0;
    }
}
