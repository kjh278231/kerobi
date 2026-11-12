//! One-RHS FTRAN `2207de` of the crossover push on the original factor layout (F+0x20 … F+0x1c8), with the kernels
//! it calls: dense L `214344`, sparse L `21b9de` (Kahn path A1), the row etas, the FT spike, dense U `2126cd` and
//! sparse U `21686f`. The two in/out vectors are modelled as the original's four buffers (index and value of the
//! caller's input and output) behind two headers whose pointers the original swaps, so the buffer aliasing and the
//! stale contents the original leaves behind are part of the result. Cells the original writes but this model does
//! not reproduce, and the output's entry state (not captured), are unknown; reading one in arithmetic panics and a
//! snapshot reports it as `None`. Contract and C references: docs/m3/primal_push_ftran_rust.md.

/// One row eta record of F+0x198 (`{r, len, idx*, val*}`) in creation order; `peel` when its values start 8 but
/// not 16-byte aligned (the 8-lane dot takes the first term alone, asm 0x120bf4).
pub struct EtaRecord {
    pub row: usize,
    pub idx: Vec<usize>,
    pub val: Vec<f64>,
    pub peel: bool,
}

/// U column of a row (F+0x88/0x90/0x98 window over F+0xa0/0xa8): `tail` entries in rows of U positions ≥ t first
/// (to F+0x98), then the prefix part.
pub struct UColumn {
    pub rows: Vec<usize>,
    pub vals: Vec<f64>,
    pub tail: usize,
}

/// The factor state `2207de` and its kernels read. Indices are the original's (rows after the F+0x30 relabel).
pub struct FtranFactor {
    pub m: usize,
    /// F+0x10.
    pub zero: f64,
    /// F+0x20 (dense input gather `out[i] = in[F20[i]]`) and F+0x30 (sparse input relabel).
    pub gather: Vec<usize>,
    pub relabel: Vec<usize>,
    /// F+0x40 row → U position, F+0x48 U position → row.
    pub upos: Vec<usize>,
    pub uord: Vec<usize>,
    /// F+0x50.
    pub diag: Vec<f64>,
    /// L columns F+0x58 (m+1 starts, relative to the first) over F+0x60 values and F+0x68 rows.
    pub lbeg: Vec<usize>,
    pub lrow: Vec<usize>,
    pub lval: Vec<f64>,
    pub ucol: Vec<UColumn>,
    /// F+0xd8 (columns before it are swept by `214344`) and F+0xe4 (from it to m).
    pub d8: usize,
    pub e4: usize,
    /// F+0x100 (t), F+0x104 (sparse multiplier), F+0x108 + F+0x110, F+0x128 (deepest L level, −1 without).
    pub t: usize,
    pub mult: i64,
    pub dsum: f64,
    pub levels: i64,
    /// F+0xf0 (length F+0xec) and F+0xf8.
    pub f0: Vec<usize>,
    pub f8: Vec<i32>,
    /// F+0x1a0: per row the first eta reading it (a large sentinel when none).
    pub oldest: Vec<i64>,
    pub etas: Vec<EtaRecord>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Buffer {
    In,
    Out,
}

/// A vector header `{i32 count; i32* idx; f64* val}`: which buffers it points at.
#[derive(Clone, Copy, Debug)]
pub struct Header {
    pub count: Option<i32>,
    pub idx: Buffer,
    pub val: Buffer,
}

#[derive(Clone)]
pub struct Cells<T> {
    v: Vec<T>,
    known: Vec<bool>,
}

impl<T: Copy + Default> Cells<T> {
    pub fn unknown(n: usize) -> Self {
        Cells { v: vec![T::default(); n], known: vec![false; n] }
    }
    pub fn get(&self, k: usize) -> T {
        assert!(self.known[k], "read of an unknown cell {k}");
        self.v[k]
    }
    pub fn peek(&self, k: usize) -> Option<T> {
        self.known[k].then(|| self.v[k])
    }
    pub fn set(&mut self, k: usize, x: T) {
        self.v[k] = x;
        self.known[k] = true;
    }
    pub fn forget(&mut self, k: usize) {
        self.known[k] = false;
    }
}

/// The FT spike F+0x1c8: count, row list and values stored at the row.
#[derive(Clone)]
pub struct Spike {
    pub count: i32,
    pub idx: Cells<i32>,
    pub val: Cells<f64>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct VecSnap {
    pub count: Option<i32>,
    pub idx: Option<Vec<Option<i32>>>,
    pub val: Vec<Option<f64>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SpikeSnap {
    pub count: i32,
    pub indices: Vec<Option<i32>>,
    pub values: Vec<Option<f64>>,
}

/// A kernel boundary as the 21a hooks record it: the input header at entry, both headers at return.
#[derive(Clone, Debug)]
pub struct KernelEvent {
    pub kernel: &'static str,
    pub leave: bool,
    pub input: VecSnap,
    pub output: Option<VecSnap>,
    pub alias: [(Buffer, Buffer); 2],
    pub spike: Option<SpikeSnap>,
}

/// Branches taken, for the coverage count of the comparison.
#[derive(Clone, Debug, Default)]
pub struct Branches {
    pub input: &'static str,
    pub l: &'static str,
    pub r: &'static str,
    pub spike: &'static str,
    pub u: &'static str,
}

pub struct FtranState {
    pub idx: [Cells<i32>; 2],
    pub val: [Cells<f64>; 2],
    pub input: Header,
    pub output: Header,
    pub spike: Option<Spike>,
    /// Scratch F+0x1b0 (i32 marks) and F+0x1b8 (f64 values), all zero between calls.
    pub mark: Vec<i32>,
    pub vals: Vec<f64>,
    pub events: Vec<KernelEvent>,
    pub branches: Branches,
}

#[derive(Debug, PartialEq, Eq)]
pub enum FtranError {
    /// A path this adapter does not implement (named).
    Unsupported(&'static str),
}

fn b(x: Buffer) -> usize {
    x as usize
}

impl FtranState {
    /// Entry state: the input header owns the caller's input buffers with `count` entries (m when dense), the
    /// output header's count and both output buffers are unknown, the scratch is zero.
    pub fn new(m: usize, count: i32, idx: &[i32], val: &[f64], spike: Option<Spike>) -> FtranState {
        let mut s = FtranState {
            idx: [Cells::unknown(m), Cells::unknown(m)],
            val: [Cells::unknown(m), Cells::unknown(m)],
            input: Header { count: Some(count), idx: Buffer::In, val: Buffer::In },
            output: Header { count: None, idx: Buffer::Out, val: Buffer::Out },
            spike,
            mark: vec![0; m],
            vals: vec![0.0; m],
            events: Vec::new(),
            branches: Branches::default(),
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

    pub fn spike_snap(&self) -> Option<SpikeSnap> {
        self.spike.as_ref().map(|s| {
            let indices: Vec<Option<i32>> = (0..s.count.max(0) as usize).map(|k| s.idx.peek(k)).collect();
            let values = indices.iter().map(|i| i.and_then(|i| s.val.peek(i as usize))).collect();
            SpikeSnap { count: s.count, indices, values }
        })
    }

    fn event(&mut self, kernel: &'static str, leave: bool, m: usize) {
        let ev = KernelEvent {
            kernel,
            leave,
            input: self.snap(self.input, m),
            output: leave.then(|| self.snap(self.output, m)),
            alias: [(self.input.idx, self.input.val), (self.output.idx, self.output.val)],
            spike: self.spike_snap(),
        };
        self.events.push(ev);
    }

    pub fn snapshot(&self, m: usize) -> (VecSnap, VecSnap) {
        (self.snap(self.output, m), self.snap(self.input, m))
    }

    fn count(h: Header) -> i32 {
        h.count.expect("read of an unknown count")
    }

    /// Swap everything between the two headers (count, index and value pointers).
    fn swap_headers(&mut self) {
        std::mem::swap(&mut self.input, &mut self.output);
    }
}

/// Σ of products in the 8-lane order of `2207de`'s R phase (asm 0x120c0e–0x120cb3): below 8 terms (or 8 + peel)
/// in sequence, else lanes (0,1)(2,3)(4,5)(6,7) over the aligned head combined ((0+2)+(4+6)) + ((1+3)+(5+7)), the
/// tail after in sequence; `peel` adds the first term to lane 0 first.
fn eta_dot(terms: &[f64], peel: bool) -> f64 {
    let p = peel as usize;
    if terms.len() < 8 || terms.len() < p + 8 {
        return terms.iter().fold(0.0, |a, &t| a + t);
    }
    let body = terms.len() - ((terms.len() - p) & 7);
    let mut a = [terms[..p].iter().fold(0.0, |a, &t| a + t), 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
    for c in terms[p..body].as_chunks::<8>().0 {
        for q in 0..8 {
            a[q] += c[q];
        }
    }
    let w = ((a[0] + a[2]) + (a[4] + a[6])) + ((a[1] + a[3]) + (a[5] + a[7]));
    terms[body..].iter().fold(w, |w, &t| w + t)
}

/// `2207de` with `save` (param_4), no second RHS (k5 = 0, p6 null), the spike F+0x1c8 and the scratch F+0x1b0/0x1b8.
pub fn ftran(f: &FtranFactor, s: &mut FtranState, save: i32) -> Result<(), FtranError> {
    let m = f.m;
    let c = FtranState::count(s.input);
    if c == 0 {
        s.branches.input = "count0";
        s.output.count = Some(0);
        return Ok(());
    }
    if c < 0 {
        s.branches.input = "dense";
        let (iv, ov) = (b(s.input.val), b(s.output.val));
        for i in 0..m {
            let v = s.val[iv].get(f.gather[i]);
            s.val[ov].set(i, v);
        }
        // C:118–125: out count := −1, headers exchange pointers, in count := −1
        s.output.count = Some(c);
        std::mem::swap(&mut s.input.idx, &mut s.output.idx);
        std::mem::swap(&mut s.input.val, &mut s.output.val);
        s.input.count = Some(-1);
        l_dense(f, s);
    } else {
        s.branches.input = "sparse";
        let ii = b(s.input.idx);
        for k in 0..c as usize {
            let i = s.idx[ii].get(k);
            s.idx[ii].set(k, f.relabel[i as usize] as i32);
        }
        if (m as i64) < c as i64 * f.mult || m as f64 * 0.1 <= f.levels as f64 {
            l_dense(f, s);
        } else {
            l_sparse(f, s)?;
        }
    }
    let lc = FtranState::count(s.output);
    if lc == 0 {
        s.branches.r = "L0";
        return Ok(());
    }
    let n = match r_phase(f, s, lc) {
        Some(n) => n,
        None => {
            s.output.count = Some(0);
            return Ok(());
        }
    };
    if save != 0 {
        store_spike(f, s, save, n)?;
    } else {
        s.branches.spike = "save0";
    }
    let n = FtranState::count(s.input);
    let te = (m - f.t) as i64 + f.f0.len() as i64 + n as i64;
    // C:687–699 (asm 0x121c61–0x121c8f): (m − t) + F_ec + n < m or (F108 + F110)²·n < 2m
    if n >= 0 && (te < m as i64 || f.dsum * f.dsum * (n as f64) < m as f64 + m as f64) {
        u_sparse(f, s);
    } else {
        u_dense(f, s);
    }
    Ok(())
}

/// R phase (C:167–685) after an L output of count `lc`; leaves the U input in the input header and returns its count,
/// None when the sparse R output is empty (the caller sets the output count to 0).
fn r_phase(f: &FtranFactor, s: &mut FtranState, lc: i32) -> Option<i32> {
    let m = f.m;
    let ne = f.etas.len();
    if ne == 0 {
        // C:170–181: the L output becomes the input header
        s.branches.r = "none";
        let oc = s.input.count;
        s.swap_headers();
        s.output.count = oc;
        s.input.count = Some(lc);
        return Some(lc);
    }
    let zero = f.zero;
    if lc < 0 {
        s.branches.r = "dense";
        s.swap_headers();
        s.input.count = Some(lc);
        s.output.count = Some(-1);
        dense_etas(f, s, 0);
    } else if lc as i64 * f.mult <= m as i64 {
        s.branches.r = "sparse";
        let (oi, ov) = (b(s.output.idx), b(s.output.val));
        let mut first = ne as i64;
        let mut cnt = lc as usize;
        for k in 0..cnt {
            let r = s.idx[oi].get(k) as usize;
            s.mark[r] = 1;
            s.vals[r] = s.val[ov].get(k);
            first = first.min(f.oldest[r]);
        }
        for e in &f.etas[first.max(0) as usize..] {
            let terms: Vec<f64> = e.idx.iter().zip(&e.val).map(|(&i, &v)| s.vals[i] * v).collect();
            let v = eta_dot(&terms, e.peel);
            if v.abs() <= zero {
                s.vals[e.row] = 0.0;
            } else {
                if s.mark[e.row] == 0 {
                    s.mark[e.row] = 1;
                    s.idx[oi].set(cnt, e.row as i32);
                    cnt += 1;
                }
                s.vals[e.row] = v;
            }
        }
        let (ii, iv) = (b(s.input.idx), b(s.input.val));
        let mut kept = 0;
        for k in 0..cnt {
            let r = s.idx[oi].get(k) as usize;
            let v = s.vals[r];
            if zero <= v.abs() {
                s.idx[ii].set(kept, r as i32);
                s.val[iv].set(kept, v);
                kept += 1;
            }
            s.vals[r] = 0.0;
            s.mark[r] = 0;
        }
        s.input.count = Some(kept as i32);
        if kept == 0 {
            return None;
        }
        return Some(kept as i32);
    } else {
        s.branches.r = "sparse-dense";
        let (oi, ov, iv) = (b(s.output.idx), b(s.output.val), b(s.input.val));
        for i in 0..m {
            s.val[iv].set(i, 0.0);
        }
        let mut first = ne as i64;
        for k in 0..lc as usize {
            let r = s.idx[oi].get(k) as usize;
            first = first.min(f.oldest[r]);
            let v = s.val[ov].get(k);
            s.val[iv].set(r, v);
        }
        dense_etas(f, s, first.max(0) as usize);
    }
    s.input.count = Some(-1);
    Some(-1)
}

/// Dense row etas from record `from` on the input header's values (C:208–296, 576–681).
fn dense_etas(f: &FtranFactor, s: &mut FtranState, from: usize) {
    let iv = b(s.input.val);
    for e in &f.etas[from..] {
        let terms: Vec<f64> = e.idx.iter().zip(&e.val).map(|(&i, &v)| s.val[iv].get(i) * v).collect();
        let v = eta_dot(&terms, e.peel);
        s.val[iv].set(e.row, if v.abs() <= f.zero { 0.0 } else { v });
    }
}

/// Spike store (C:703–778) from the input header of count `n` (after R).
fn store_spike(f: &FtranFactor, s: &mut FtranState, save: i32, n: i32) -> Result<(), FtranError> {
    let m = f.m;
    let sv = save as f64;
    let (ii, iv) = (b(s.input.idx), b(s.input.val));
    let Some(sp) = s.spike.as_mut() else { return Err(FtranError::Unsupported("save without spike")) };
    if n < 1 {
        s.branches.spike = "dense";
        let mut k = 0;
        for i in 0..m {
            let v = s.val[iv].get(i);
            if f.zero < v.abs() {
                sp.idx.set(k, i as i32);
                k += 1;
                sp.val.set(i, v * sv);
            }
        }
        sp.count = k as i32;
    } else {
        s.branches.spike = "sparse";
        sp.count = n;
        for k in 0..n as usize {
            let i = s.idx[ii].get(k);
            sp.idx.set(k, i);
            let v = s.val[iv].get(k);
            sp.val.set(i as usize, v * sv);
        }
    }
    Ok(())
}

/// `214344` (dense L): a dense input takes the output header's place, a sparse one is scattered into the zeroed
/// output values; then L columns [0, F+0xd8) and [F+0xe4, m) with |x_k| > ZERO, x_i −= l·x_k in storage order.
fn l_dense(f: &FtranFactor, s: &mut FtranState) {
    let m = f.m;
    s.branches.l = "214344";
    s.event("214344", false, m);
    let c = FtranState::count(s.input);
    if c < 1 {
        let oc = s.output.count;
        s.swap_headers();
        s.output.count = Some(c);
        s.input.count = oc;
    } else {
        let (ii, iv, ov) = (b(s.input.idx), b(s.input.val), b(s.output.val));
        for i in 0..m {
            s.val[ov].set(i, 0.0);
        }
        for k in 0..c as usize {
            let i = s.idx[ii].get(k) as usize;
            let v = s.val[iv].get(k);
            s.val[ov].set(i, v);
        }
    }
    let ov = b(s.output.val);
    for k in (0..f.d8).chain(f.e4..m) {
        let xk = s.val[ov].get(k);
        if f.zero < xk.abs() {
            for e in f.lbeg[k]..f.lbeg[k + 1] {
                let i = f.lrow[e];
                let x = s.val[ov].get(i);
                s.val[ov].set(i, x - f.lval[e] * xk);
            }
        }
    }
    s.output.count = Some(-1);
    s.event("214344", true, m);
}

/// `21b9de` (sparse L). Only path A1 (Kahn FIFO over the reached columns, C:81–257): the output index buffer holds
/// the reached list and then the queue, the output keeps |v| ≥ ZERO in queue order.
pub fn l_sparse(f: &FtranFactor, s: &mut FtranState) -> Result<(), FtranError> {
    let m = f.m;
    let dm = (m - f.e4) as f64;
    let ltail = (f.lbeg[m] - f.lbeg[f.e4]) as f64;
    if !(m - f.e4 < 6 || dm * (dm * 10.0) <= m as f64 || ltail <= dm * 10.0) {
        return Err(FtranError::Unsupported("21b9de path B"));
    }
    if !(f.levels < 0 || m as f64 * 0.1 <= f.levels as f64) {
        return Err(FtranError::Unsupported("21b9de path A2"));
    }
    s.branches.l = "21b9de";
    s.event("21b9de", false, m);
    let (ii, iv, oi, ov) = (b(s.input.idx), b(s.input.val), b(s.output.idx), b(s.output.val));
    let c = FtranState::count(s.input) as usize;
    let mut n = c;
    for k in 0..c {
        let i = s.idx[ii].get(k);
        s.idx[oi].set(k, i);
        s.mark[i as usize] += 1;
    }
    let mut p = 0;
    while p < n {
        let j = s.idx[oi].get(p) as usize;
        p += 1;
        for e in f.lbeg[j]..f.lbeg[j + 1] {
            let i = f.lrow[e];
            if s.mark[i] == 0 {
                s.idx[oi].set(n, i as i32);
                n += 1;
            }
            s.mark[i] += 1;
        }
    }
    let mut q = 0;
    for k in 0..c {
        let i = s.idx[oi].get(k) as usize;
        s.vals[i] = s.val[iv].get(k);
        s.mark[i] -= 1;
        if s.mark[i] == 0 {
            s.idx[oi].set(q, i as i32);
            q += 1;
        }
    }
    let mut out = 0;
    let mut p = 0;
    while p < q {
        let j = s.idx[oi].get(p) as usize;
        p += 1;
        let v = std::mem::take(&mut s.vals[j]);
        let big = f.zero <= v || v <= -f.zero;
        if big {
            s.idx[oi].set(out, j as i32);
            s.val[ov].set(out, v);
            out += 1;
        }
        for e in f.lbeg[j]..f.lbeg[j + 1] {
            let i = f.lrow[e];
            s.mark[i] -= 1;
            if big {
                s.vals[i] += -(f.lval[e] * v);
            }
            if s.mark[i] == 0 {
                s.idx[oi].set(q, i as i32);
                q += 1;
            }
        }
    }
    s.output.count = Some(out as i32);
    s.event("21b9de", true, m);
    Ok(())
}

/// `2126cd` (dense U): as `214344` for the input; U positions m−1 → t divide and scatter the whole column, t−1 → 0
/// divide; x := +0 when |x| ≤ ZERO or |x/d| ≤ ZERO.
fn u_dense(f: &FtranFactor, s: &mut FtranState) {
    let m = f.m;
    s.branches.u = "2126cd";
    s.event("2126cd", false, m);
    let c = FtranState::count(s.input);
    if c < 1 {
        let oc = s.output.count;
        s.swap_headers();
        s.output.count = Some(c);
        s.input.count = oc;
    } else {
        let (ii, iv, ov) = (b(s.input.idx), b(s.input.val), b(s.output.val));
        for i in 0..m {
            s.val[ov].set(i, 0.0);
        }
        for k in 0..c as usize {
            let i = s.idx[ii].get(k) as usize;
            let v = s.val[iv].get(k);
            s.val[ov].set(i, v);
        }
    }
    let ov = b(s.output.val);
    for p in (0..m).rev() {
        let i = f.uord[p];
        let x = s.val[ov].get(i);
        let d = f.diag[i];
        if x.abs() <= f.zero || (x / d).abs() <= f.zero {
            s.val[ov].set(i, 0.0);
            continue;
        }
        let y = x / d;
        s.val[ov].set(i, y);
        if p >= f.t {
            let col = &f.ucol[i];
            for (&r, &u) in col.rows.iter().zip(&col.vals) {
                let x = s.val[ov].get(r);
                s.val[ov].set(r, x - u * y);
            }
        }
    }
    s.output.count = Some(-1);
    s.event("2126cd", true, m);
}

/// `21686f` (sparse U, docs/m1/m1_02m_usolve.md §2): direct outputs for rows with an empty U row (F+0xf8 = −1),
/// the tail by descending U position, prefix rows in section-2 or F+0xf0 order; outputs |x/d| > ZERO. The
/// heap the original keeps below section 2 of the output index buffer is processed in the same order here and its
/// cells are left unknown.
pub fn u_sparse(f: &FtranFactor, s: &mut FtranState) {
    let m = f.m;
    s.branches.u = "21686f";
    s.event("21686f", false, m);
    let c = FtranState::count(s.input);
    if c == 0 {
        s.output.count = Some(0);
        s.event("21686f", true, m);
        return;
    }
    let zero = f.zero;
    let ec = f.f0.len();
    let d2 = f.dsum * f.dsum;
    let (ii, iv, oi, ov) = (b(s.input.idx), b(s.input.val), b(s.output.idx), b(s.output.val));
    let sec2 = m - ec;
    let mut out = 0usize;
    let emit = |s: &mut FtranState, i: usize, y: f64, out: &mut usize| {
        s.idx[oi].set(*out, i as i32);
        s.val[ov].set(*out, y);
        *out += 1;
    };
    if m as f64 <= c as f64 * d2 {
        s.branches.u = "21686f S1";
        for k in 0..c as usize {
            let i = s.idx[ii].get(k) as usize;
            let v = s.val[iv].get(k);
            if f.f8[i] == -1 {
                let y = v / f.diag[i];
                if zero < y.abs() {
                    emit(s, i, y, &mut out);
                }
            } else {
                s.vals[i] = v;
            }
        }
    } else {
        let (mut ntail, mut npre) = (0usize, 0usize);
        for k in 0..c as usize {
            let i = s.idx[ii].get(k) as usize;
            let v = s.val[iv].get(k);
            if f.f8[i] == -1 {
                let y = v / f.diag[i];
                if zero < y.abs() {
                    emit(s, i, y, &mut out);
                }
            } else if f.f8[i] < 0 {
                s.mark[i] += 1;
                s.vals[i] = v;
                s.idx[ii].set(ntail, i as i32);
                ntail += 1;
            } else {
                s.mark[i] = 1;
                s.idx[oi].set(sec2 + npre, i as i32);
                npre += 1;
                s.vals[i] = v;
            }
        }
        if d2 * ((npre + ntail) as f64) <= (ec + m - f.t) as f64 {
            // H: tail outputs (tail parts only), then the prefix
            let direct = out;
            let mut b8 = npre;
            let mut reached = 0usize;
            for p in (f.t..m).rev() {
                let i = f.uord[p];
                if s.mark[i] != 0 || s.vals[i] != 0.0 {
                    reached += 1;
                }
                let y = s.vals[i] / f.diag[i];
                s.vals[i] = 0.0;
                s.mark[i] = 0;
                if zero < y.abs() {
                    emit(s, i, y, &mut out);
                    let col = &f.ucol[i];
                    b8 += col.rows.len() - col.tail;
                    for (&r, &u) in col.rows[..col.tail].iter().zip(&col.vals[..col.tail]) {
                        s.vals[r] -= u * y;
                    }
                }
            }
            for k in 0..reached.max(ntail).min(sec2) {
                s.idx[oi].forget(sec2 - 1 - k);
            }
            let tail_out: Vec<(usize, f64)> =
                (direct..out).map(|q| (s.idx[oi].get(q) as usize, s.val[ov].get(q))).collect();
            if b8 as f64 * 2.5 <= ec as f64 {
                s.branches.u = "21686f H2";
                for &(i, y) in &tail_out {
                    let col = &f.ucol[i];
                    for (&r, &u) in col.rows[col.tail..].iter().zip(&col.vals[col.tail..]) {
                        if s.mark[r] == 0 {
                            s.idx[oi].set(sec2 + npre, r as i32);
                            npre += 1;
                        }
                        s.mark[r] = 1;
                        s.vals[r] += -(u * y);
                    }
                }
                for k in 0..npre {
                    let r = s.idx[oi].get(sec2 + k) as usize;
                    let y = s.vals[r] / f.diag[r];
                    if zero < y.abs() {
                        emit(s, r, y, &mut out);
                    }
                    s.vals[r] = 0.0;
                    s.mark[r] = 0;
                }
            } else {
                s.branches.u = "21686f H1";
                for k in 0..npre {
                    let r = s.idx[oi].get(sec2 + k) as usize;
                    s.mark[r] = 0;
                }
                for &(i, y) in &tail_out {
                    let col = &f.ucol[i];
                    for (&r, &u) in col.rows[col.tail..].iter().zip(&col.vals[col.tail..]) {
                        s.vals[r] += -(u * y);
                    }
                }
                for &r in &f.f0 {
                    let y = s.vals[r] / f.diag[r];
                    if zero < y.abs() {
                        emit(s, r, y, &mut out);
                    }
                    s.vals[r] = 0.0;
                }
            }
            s.output.count = Some(out as i32);
            s.event("21686f", true, m);
            return;
        }
        s.branches.u = "21686f S2";
        for k in 0..ntail {
            let i = s.idx[ii].get(k) as usize;
            s.mark[i] = 0;
        }
        for k in 0..npre {
            let r = s.idx[oi].get(sec2 + k) as usize;
            s.mark[r] = 0;
        }
    }
    for p in (f.t..m).rev() {
        let i = f.uord[p];
        let y = s.vals[i] / f.diag[i];
        if zero < y.abs() {
            emit(s, i, y, &mut out);
            let col = &f.ucol[i];
            for (&r, &u) in col.rows.iter().zip(&col.vals) {
                s.vals[r] -= u * y;
            }
        }
        s.vals[i] = 0.0;
    }
    for &r in &f.f0 {
        let y = s.vals[r] / f.diag[r];
        if zero < y.abs() {
            emit(s, r, y, &mut out);
        }
        s.vals[r] = 0.0;
    }
    s.output.count = Some(out as i32);
    s.event("21686f", true, m);
}
