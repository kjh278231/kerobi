//! Serial numeric backend of the crossover push on the original factor layout: the factor `F` (F+0x18 … F+0x1d8)
//! with the shared U/eta pool (F+0x148/0x150) at the original's positions, carried through the push's factor
//! operations in order — the Forrest–Tomlin update of `05b257` (request 0, C:144–1335) with its row eta `09e47c`,
//! the push-state renumbering after a refactor (C:2006–2090, 2765–2770), and the one-RHS FTRAN (`crossover_ftran`,
//! behind the supplier wrapper's spike clear `09d008`) and support BTRAN (`crossover_btran`) on this state. The
//! refactor's factor itself (`036ade` and C:2570–2760) is an input of this backend. Contract and C references:
//! docs/m3/primal_push_lu_rust.md.

use crate::crossover_btran::{self, BtranEta, BtranFactor, BtranState, Cache, SparseRow};
use crate::crossover_ftran::{self, eta_dot, Cells, EtaRecord, FtranError, FtranFactor, FtranState, Spike, UColumn};

/// A row eta record of F+0x198: its row, length and pool offset (F+0x148/0x150 index).
#[derive(Clone)]
pub struct EtaRec {
    pub row: i32,
    pub len: usize,
    pub off: i64,
}

/// Scalars of F that the backend carries (F+0xec and F+0x180 are the lengths of `f0` and `etas`).
#[derive(Clone)]
pub struct Scalars {
    pub xd8: i32,
    pub xdc: i32,
    pub xe4: i32,
    pub xe8: i32,
    pub x100: i32,
    pub x104: i32,
    pub x108: f64,
    pub x110: f64,
    pub x128: i32,
    pub x184: i32,
    pub x188: i32,
    pub x18c: i32,
    pub x158: i64,
    pub x160: i64,
    pub x168: i64,
    pub x1f8: f64,
    pub pool_mod16: [i32; 2],
    pub x140: i32,
    pub x1e0: i64,
}

#[derive(Clone)]
pub struct LuState {
    pub m: usize,
    /// F+0x10.
    pub zero: f64,
    /// F+0x18 (push state: set by the caller before each factor call).
    pub header: Vec<i32>,
    /// F+0x20 … F+0x48 (F+0x40 row → U position, F+0x48 U position → row).
    pub x20: Vec<i32>,
    pub x28: Vec<i32>,
    pub x30: Vec<i32>,
    pub x38: Vec<i32>,
    pub x40: Vec<i32>,
    pub x48: Vec<i32>,
    /// F+0x40 / F+0x48 point at the F+0x28 / F+0x38 buffers (set by the refactor, C:2572–2573), so the update's
    /// permutation shows in both.
    pub x40_is_x28: bool,
    pub x48_is_x38: bool,
    pub diag: Vec<f64>,
    /// L columns (F+0x58 starts as stored, F+0x68 rows, F+0x60 values) and levels F+0x128/0x118/0x120.
    pub lbeg: Vec<i64>,
    pub lrows: Vec<i32>,
    pub lvals: Vec<f64>,
    pub llev: Option<Vec<i32>>,
    pub llst: Option<Vec<i32>>,
    /// L rows (F+0x70 starts, F+0x80 columns, F+0x78 values), levels F+0x140/0x130/0x138 and the bucket F+0x1a8[m..].
    pub rbeg: Vec<i64>,
    pub rcols: Vec<i32>,
    pub rvals: Vec<f64>,
    pub rlev: Option<Vec<i32>>,
    pub rlst: Option<Vec<i32>>,
    pub bucket: Option<Vec<i32>>,
    /// U columns F+0x88/0x90/0x98 (start, end, tail bound) in column coordinates (pool index − `colbase`).
    pub cs: Vec<i64>,
    pub ce: Vec<i64>,
    pub cb: Vec<i64>,
    /// U rows F+0xb0/0xb8/0xc0 (start, end, capacity) in pool coordinates.
    pub rs: Vec<i64>,
    pub re: Vec<i64>,
    pub rc: Vec<i64>,
    /// (F+0xa8 − F+0x148)/4: where the column arrays start in the pool (`036ade` C:5378).
    pub colbase: i64,
    pub pidx: Cells<i32>,
    pub pval: Cells<f64>,
    pub sc: Scalars,
    /// F+0xf0 (prefix rows with a non-empty U row) and F+0xf8 (slot, −1 empty prefix row, −2 tail).
    pub f0: Vec<i32>,
    pub f8: Vec<i32>,
    /// F+0x1a0.
    pub oldest: Vec<i32>,
    pub etas: Vec<EtaRec>,
    /// F+0x1c8 (values by row), F+0x1d0 cache, F+0x1d8.
    pub spike: Spike,
    pub cache: Cache,
    pub x1d8: i32,
    /// F+0x1b0 / F+0x1b8 scratch.
    pub mark: Vec<i32>,
    pub vals: Vec<f64>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum LuError {
    /// A path of `05b257` this backend does not implement (named).
    Unsupported(&'static str),
    Ftran(FtranError),
}

/// What one FT update did, for the comparison and the branch counts.
#[derive(Clone, Debug, Default)]
pub struct FtReport {
    /// `09e47c` was called with this row (cache before → after).
    pub roweta: Option<i32>,
    /// The cache right after `09e47c` (count, rows, values; the exit empties it).
    pub roweta_cache: Option<(i32, Vec<Option<i32>>, Vec<Option<f64>>)>,
    pub cache_reuse: bool,
    pub interval_trivial: bool,
    pub eta: bool,
    pub local80: i32,
    pub promoted: bool,
    pub cycle: bool,
    pub moved_column: bool,
    pub moved_rows: usize,
    pub dot_lanes: bool,
}

/// Push-state outputs of a refactor's renumbering.
pub struct Renumbered {
    pub header: Vec<i32>,
    pub leave: i32,
    pub vstat: Vec<i32>,
}

impl LuState {
    fn pool(&self, k: i64) -> usize {
        assert!(k >= 0 && k < self.sc.x168, "pool position {k} outside [0, F+0x168)");
        k as usize
    }
    fn col_idx(&self, k: i64) -> i32 {
        self.pidx.get(self.pool(k + self.colbase))
    }
    fn col_val(&self, k: i64) -> f64 {
        self.pval.get(self.pool(k + self.colbase))
    }
    fn set_col(&mut self, k: i64, i: i32, v: f64) {
        let p = self.pool(k + self.colbase);
        self.pidx.set(p, i);
        self.pval.set(p, v);
    }
    fn row_idx(&self, k: i64) -> i32 {
        self.pidx.get(self.pool(k))
    }
    fn row_val(&self, k: i64) -> f64 {
        self.pval.get(self.pool(k))
    }
    fn set_row(&mut self, k: i64, i: i32, v: f64) {
        let p = self.pool(k);
        self.pidx.set(p, i);
        self.pval.set(p, v);
    }
    fn upos(&self, i: i32) -> i32 {
        self.x40[i as usize]
    }

    /// F+0x28 and F+0x38 as stored (the F+0x40/0x48 buffers when they alias them).
    pub fn x28_x38(&self) -> (&[i32], &[i32]) {
        (if self.x40_is_x28 { &self.x40 } else { &self.x28 }, if self.x48_is_x38 { &self.x48 } else { &self.x38 })
    }

    /// Remove row `i` from the F+0xf0 list (C:540–549, C:1087–1096).
    fn unlist(&mut self, i: i32, mark: i32) {
        let iu = i as usize;
        if self.f8[iu] >= 0 {
            let last = self.f0.pop().expect("F+0xf0 entry");
            if last != i {
                self.f0[self.f8[iu] as usize] = last;
                self.f8[last as usize] = self.f8[iu];
            }
        }
        self.f8[iu] = mark;
    }

    /// U rows of the current state in storage order.
    pub fn u_rows(&self) -> Vec<SparseRow> {
        (0..self.m)
            .map(|i| {
                let (a, b) = (self.rs[i], self.re[i]);
                SparseRow {
                    cols: (a..b).map(|k| self.row_idx(k) as usize).collect(),
                    vals: (a..b).map(|k| self.row_val(k)).collect(),
                }
            })
            .collect()
    }

    /// U columns of the current state in storage order (tail part first, as F+0x98 splits it).
    pub fn u_cols(&self) -> Vec<(Vec<i32>, Vec<f64>)> {
        (0..self.m)
            .map(|j| {
                let (a, b) = (self.cs[j], self.ce[j]);
                ((a..b).map(|k| self.col_idx(k)).collect(), (a..b).map(|k| self.col_val(k)).collect())
            })
            .collect()
    }

    /// Eta record entries (row, value) from the pool, and whether the values start 8 but not 16-byte aligned.
    pub fn eta_entries(&self, e: &EtaRec) -> (Vec<i32>, Vec<f64>, bool) {
        let idx = (0..e.len as i64).map(|k| self.pidx.get(self.pool(e.off + k))).collect();
        let val = (0..e.len as i64).map(|k| self.pval.get(self.pool(e.off + k))).collect();
        (idx, val, self.eta_mod16(e).1 == 8)
    }

    /// (idx, val) pointer address mod 16 of an eta record (F+0x148/0x150 base alignment + offset).
    pub fn eta_mod16(&self, e: &EtaRec) -> (i64, i64) {
        let [a, b] = self.sc.pool_mod16;
        ((a as i64 + 4 * e.off).rem_euclid(16), (b as i64 + 8 * e.off).rem_euclid(16))
    }

    /// The view `2207de` and its kernels read.
    pub fn ftran_factor(&self) -> FtranFactor {
        let u = |v: &[i32]| v.iter().map(|&x| x as usize).collect::<Vec<_>>();
        let b0 = self.lbeg[0];
        FtranFactor {
            m: self.m,
            zero: self.zero,
            gather: u(&self.x20),
            relabel: u(&self.x30),
            upos: u(&self.x40),
            uord: u(&self.x48),
            diag: self.diag.clone(),
            lbeg: self.lbeg.iter().map(|&x| (x - b0) as usize).collect(),
            lrow: u(&self.lrows),
            lval: self.lvals.clone(),
            ucol: (0..self.m)
                .map(|j| {
                    let (a, b) = (self.cs[j], self.ce[j]);
                    UColumn {
                        rows: (a..b).map(|k| self.col_idx(k) as usize).collect(),
                        vals: (a..b).map(|k| self.col_val(k)).collect(),
                        tail: (self.cb[j] - a) as usize,
                    }
                })
                .collect(),
            d8: self.sc.xd8 as usize,
            e4: self.sc.xe4 as usize,
            t: self.sc.x100 as usize,
            mult: self.sc.x104 as i64,
            dsum: self.sc.x108 + self.sc.x110,
            levels: self.sc.x128 as i64,
            llev: self.llev.as_deref().map(u).unwrap_or_default(),
            llst: self.llst.as_deref().map(u).unwrap_or_default(),
            f0: u(&self.f0),
            f8: self.f8.clone(),
            oldest: self.oldest.iter().map(|&x| x as i64).collect(),
            etas: self
                .etas
                .iter()
                .map(|e| {
                    let (idx, val, peel) = self.eta_entries(e);
                    EtaRecord { row: e.row as usize, idx: u(&idx), val, peel }
                })
                .collect(),
        }
    }

    /// The view `221173` and `21a941` read.
    pub fn btran_factor(&self) -> BtranFactor {
        let u = |v: &[i32]| v.iter().map(|&x| x as usize).collect::<Vec<_>>();
        let m = self.m;
        let r0 = self.rbeg[0];
        BtranFactor {
            m,
            zero: self.zero,
            relabel: u(&self.x20),
            gather: u(&self.x30),
            upos: u(&self.x40),
            uord: u(&self.x48),
            diag: self.diag.clone(),
            urow: self.u_rows(),
            flag: self.f8.clone(),
            t: self.sc.x100 as usize,
            mult: self.sc.x104,
            f108: self.sc.x108,
            f110: self.sc.x110,
            d8: self.sc.xd8 as usize,
            e4: self.sc.xe4 as usize,
            ltail: self.lbeg[m] - self.lbeg[self.sc.xe4 as usize],
            lrow: (0..m)
                .map(|i| {
                    let (a, b) = ((self.rbeg[i] - r0) as usize, (self.rbeg[i + 1] - r0) as usize);
                    SparseRow { cols: u(&self.rcols[a..b]), vals: self.rvals[a..b].to_vec() }
                })
                .collect(),
            levels: self.sc.x140,
            lev: self.rlev.as_deref().map(u).unwrap_or_default(),
            lst: self.rlst.as_deref().map(u).unwrap_or_default(),
            f1e0: self.sc.x1e0,
            f184: self.sc.x184,
            etas: self
                .etas
                .iter()
                .map(|e| {
                    let (idx, val, _) = self.eta_entries(e);
                    BtranEta { row: e.row as usize, idx: u(&idx), val }
                })
                .collect(),
        }
    }

    /// `09e47c(F, S+0x220, r, work)`: e_r Uᵀ⁻¹ into the cache F+0x1d0 by the `21a941` algorithm (same C paths; the
    /// input is count 1, value 1.0), an empty U row r leaves the cache empty. F+0x200 work is not kept.
    fn row_eta(&mut self, r: usize) {
        if self.rs[r] == self.re[r] {
            self.cache.count = 0;
            return;
        }
        let f = self.btran_factor();
        let m = self.m;
        let mut s = BtranState::new(m, 1, &[r as i32], &[1.0], 0, None, self.x1d8, 0.0, None);
        std::mem::swap(&mut s.mark, &mut self.mark);
        std::mem::swap(&mut s.vals, &mut self.vals);
        crossover_btran::kernel(&f, &mut s);
        std::mem::swap(&mut s.mark, &mut self.mark);
        std::mem::swap(&mut s.vals, &mut self.vals);
        let (oi, ov) = (s.output.idx as usize, s.output.val as usize);
        for k in 0..m {
            if let Some(i) = s.idx[oi].peek(k) {
                self.cache.idx.set(k, i);
            }
            if let Some(v) = s.val[ov].peek(k) {
                self.cache.val.set(k, v);
            }
        }
        self.cache.count = s.output.count.expect("kernel count");
    }

    /// `05b257` with request 0 and S+0x290 = 0: the Forrest–Tomlin update of leaving position `r` from the spike
    /// F+0x1c8 (C:144–1335), then the common exit (C:2765–2770).
    pub fn ft_update(&mut self, r: usize) -> Result<FtReport, LuError> {
        let mut rep = FtReport::default();
        let ri = r as i32;
        // C:247–309: i52 = max(F40[r], t − 1), i35 = max over the spike list (r excluded), r's place in the list
        let n = self.spike.count;
        if n < 0 {
            return Err(LuError::Unsupported("dense spike"));
        }
        let t = self.sc.x100;
        let tm1 = t - 1;
        let q = self.upos(ri);
        let mut i52 = q.max(tm1);
        let mut i35 = i52;
        let mut at_r: i32 = -1;
        for k in 0..n as usize {
            let s = self.spike.idx.get(k);
            if s == ri {
                at_r = k as i32;
            } else if i35 < self.upos(s) {
                i35 = self.upos(s);
            }
        }
        // C:311–420: row eta (cache) and the new diagonal η·S
        if i35 != i52 {
            if ri != self.x1d8 {
                self.row_eta(r);
                rep.roweta = Some(ri);
                let n = self.cache.count.max(0) as usize;
                rep.roweta_cache = Some((
                    self.cache.count,
                    (0..n).map(|k| self.cache.idx.peek(k)).collect(),
                    (0..n).map(|k| self.cache.val.peek(k)).collect(),
                ));
            } else {
                rep.cache_reuse = true;
            }
            let c = self.cache.count;
            if c > 0 {
                let terms: Vec<f64> = (0..c as usize)
                    .map(|k| self.spike.val.get(self.cache.idx.get(k) as usize) * self.cache.val.get(k))
                    .collect();
                // shortcut: the cache value buffer is taken as 16-byte aligned (its address is not captured)
                rep.dot_lanes = terms.len() >= 8;
                let d = eta_dot(&terms, false);
                self.spike.val.set(r, d);
            } else if self.re[r] > self.rs[r] {
                return Err(LuError::Unsupported("empty row eta with a non-empty U row (C:403–419)"));
            }
        } else {
            rep.interval_trivial = true;
            self.cache.count = 0;
        }
        // C:421–425
        let mut np = n;
        if at_r >= 0 {
            np = n - 1;
            let last = self.spike.idx.get((n - 1) as usize);
            self.spike.idx.set(at_r as usize, last);
        }
        if self.spike.val.get(r).abs() < 1e-13 {
            return Err(LuError::Unsupported("|S_r| < 1e-13 (C:426–437)"));
        }
        // C:441–503: local_40 and the LU+0xe8 scan
        let mut promote = false;
        let local40;
        if i35 == i52 {
            local40 = if q < i52 || i52 == tm1 { tm1 - (np != 0) as i32 } else { tm1 };
        } else if q < i52 {
            let e8 = self.sc.xe8;
            let mut i17 = e8;
            let mut found = false;
            if i52 < e8 {
                loop {
                    let row = self.x48[i17 as usize] as usize;
                    if self.ce[row] == self.cs[row] {
                        found = true;
                        break;
                    }
                    i17 -= 1;
                    if i52 >= i17 {
                        break;
                    }
                }
            } else {
                found = true;
            }
            if found && i52 < i17 {
                promote = true;
                self.sc.xe8 = i17;
                local40 = tm1;
                i52 = i17;
            } else {
                self.sc.xe8 = i17;
                local40 = if i52 < i35 { t - 2 } else { tm1 };
            }
        } else {
            local40 = if i52 < i35 && i52 == tm1 { t - 2 + (np == 0) as i32 } else { tm1 };
        }
        rep.promoted = promote;
        // C:505–516
        let c = self.cache.count;
        let local80 = if c < 1 {
            if np > 0 && i35 == i52 {
                if tm1 < q {
                    0
                } else {
                    3
                }
            } else {
                0
            }
        } else {
            (tm1 < q) as i32 + 1
        };
        rep.local80 = local80;
        // C:517–568: column r leaves U; rows left with one entry at a position ≤ local_40 leave the F+0xf0 list
        let (c0, c1) = (self.cs[r], self.ce[r]);
        let local150 = c1 - c0;
        for k in c0..c1 {
            let row = self.col_idx(k);
            let ru = row as usize;
            let (a, b) = (self.rs[ru], self.re[ru]);
            let mut pos = a;
            if b == a + 1 {
                if self.upos(row) <= local40 {
                    self.unlist(row, -1);
                }
            } else {
                while pos < b && self.row_idx(pos) != ri {
                    pos += 1;
                }
            }
            self.re[ru] = b - 1;
            let (i, v) = (self.row_idx(b - 1), self.row_val(b - 1));
            self.set_row(pos, i, v);
        }
        // C:569–722: pool room for the eta, the new column and the full rows that get an entry
        let room = self.sc.x188 - self.sc.x18c;
        let mut need = c.max(0) as i64;
        if local150 < np as i64 {
            need += np as i64;
        }
        for k in 0..np as usize {
            let i = self.spike.idx.get(k) as usize;
            if self.re[i] == self.rc[i] {
                let len = self.re[i] - self.rs[i];
                let extra = ((len as i32) / 2).max(4).min(room);
                need += extra as i64 + len;
            }
        }
        if self.sc.x168 - self.sc.x160 < need {
            return Err(LuError::Unsupported("pool reallocation (C:723–828)"));
        }
        if c < 1 {
            // C:829–861: without a row eta, r moves to the tail part of its columns (local_80 = 3)
            if local80 == 3 {
                for h in self.rs[r]..self.re[r] {
                    let col = self.row_idx(h) as usize;
                    let b0 = self.cb[col];
                    let mut p = b0;
                    while p < self.ce[col] && self.col_idx(p) != ri {
                        p += 1;
                    }
                    if b0 != p {
                        let (i1, v1) = (self.col_idx(p), self.col_val(p));
                        let (i0, v0) = (self.col_idx(b0), self.col_val(b0));
                        self.set_col(p, i0, v0);
                        self.set_col(b0, i1, v1);
                    }
                    self.cb[col] = b0 + 1;
                }
            }
        } else {
            // C:862–978: the row eta record at the pool end, F+0x1a0, and row r out of U
            rep.eta = true;
            let off = self.sc.x160;
            let ne = self.etas.len() as i32;
            for k in 0..c as usize {
                let (i, v) = (self.cache.idx.get(k), self.cache.val.get(k));
                let p = self.pool(off + k as i64);
                self.pidx.set(p, i);
                self.pval.set(p, v);
            }
            self.sc.x160 = off + c as i64;
            for k in 0..c as usize {
                let i = self.cache.idx.get(k) as usize;
                self.oldest[i] = self.oldest[i].min(ne);
            }
            self.etas.push(EtaRec { row: ri, len: c as usize, off });
            self.sc.x184 += c;
            for h in self.rs[r]..self.re[r] {
                let col = self.row_idx(h) as usize;
                let end;
                if local80 == 1 {
                    let e = self.ce[col];
                    let mut p = self.cb[col];
                    while p < e && self.col_idx(p) != ri {
                        p += 1;
                    }
                    end = e - 1;
                    self.ce[col] = end;
                    if p < end {
                        let (i, v) = (self.col_idx(end), self.col_val(end));
                        self.set_col(p, i, v);
                    }
                } else {
                    let b = self.cb[col];
                    let mut p = self.cs[col];
                    while p < b && self.col_idx(p) != ri {
                        p += 1;
                    }
                    let b1 = b - 1;
                    self.cb[col] = b1;
                    if p < b1 {
                        let (i, v) = (self.col_idx(b1), self.col_val(b1));
                        self.set_col(p, i, v);
                    }
                    let e1 = self.ce[col] - 1;
                    self.ce[col] = e1;
                    if b1 < e1 {
                        let (i, v) = (self.col_idx(e1), self.col_val(e1));
                        self.set_col(b1, i, v);
                    }
                    end = e1;
                }
                if end == self.cs[col] {
                    self.sc.xe8 = self.sc.xe8.max(self.upos(col as i32));
                }
            }
            self.re[r] = self.rs[r];
        }
        // C:1031–1078: the empty column at the new p joins the prefix
        if promote {
            let k = self.x48[i52 as usize];
            let ku = k as usize;
            if self.rs[ku] < self.re[ku] {
                self.f8[ku] = self.f0.len() as i32;
                self.f0.push(k);
                for h in self.rs[ku]..self.re[ku] {
                    let col = self.row_idx(h) as usize;
                    let b = self.cb[col];
                    let mut p = self.cs[col];
                    while p < b && self.col_idx(p) != k {
                        p += 1;
                    }
                    let b1 = b - 1;
                    self.cb[col] = b1;
                    if b1 != p {
                        let (i1, v1) = (self.col_idx(p), self.col_val(p));
                        let (i0, v0) = (self.col_idx(b1), self.col_val(b1));
                        self.set_col(p, i0, v0);
                        self.set_col(b1, i1, v1);
                    }
                }
            } else {
                self.f8[ku] = -1;
            }
        }
        // C:1079–1098
        if i35 != i52 || i52 != tm1 || np != 0 {
            self.unlist(ri, -2);
        }
        // C:1099–1218: permutation
        let swap = |s: &mut LuState, a: i32, b: i32| {
            let (ka, kb) = (s.x48[a as usize], s.x48[b as usize]);
            s.x48[a as usize] = kb;
            s.x48[b as usize] = ka;
            s.x40[kb as usize] = a;
            s.x40[ka as usize] = b;
        };
        if i35 == i52 || i35 < i52 {
            if q < i52 {
                swap(self, q, i52);
            }
        } else {
            if q < i52 {
                swap(self, q, i52);
            }
            rep.cycle = true;
            let base = i52;
            loop {
                let ll = i35;
                let k = self.x48[ll as usize] as usize;
                let len = self.ce[k] - self.cs[k];
                if (ll - base) as i64 <= len {
                    break;
                }
                let mut m2 = base;
                for h in self.cs[k]..self.ce[k] {
                    let pp = self.upos(self.col_idx(h));
                    if m2 < pp && pp < ll {
                        m2 = pp;
                    }
                }
                swap(self, base, ll);
                i35 = m2;
            }
            let k0 = self.x48[base as usize];
            for p in base..i35 {
                let k = self.x48[(p + 1) as usize];
                self.x48[p as usize] = k;
                self.x40[k as usize] = p;
            }
            self.x48[i35 as usize] = k0;
            self.x40[k0 as usize] = i35;
        }
        // C:1219–1222
        self.sc.x100 = local40 + 1;
        self.diag[r] = self.spike.val.get(r);
        self.spike.val.set(r, 0.0);
        // C:1223–1311: the new column r from the spike list; rows that get an entry and are full move to the pool end
        if local150 < np as i64 {
            rep.moved_column = true;
            self.cs[r] = self.sc.x160 - self.colbase;
            self.sc.x160 += np as i64;
        }
        self.ce[r] = self.cs[r] + np as i64;
        let mut head = self.cs[r];
        let mut tailp = self.ce[r];
        for k in 0..np as usize {
            let i = self.spike.idx.get(k);
            let iu = i as usize;
            let v = self.spike.val.get(iu);
            self.spike.val.set(iu, 0.0);
            if local40 < self.upos(i) {
                self.set_col(head, i, v);
                head += 1;
            } else {
                if self.f8[iu] < 0 {
                    self.f8[iu] = self.f0.len() as i32;
                    self.f0.push(i);
                }
                tailp -= 1;
                self.set_col(tailp, i, v);
            }
            let mut e = self.re[iu];
            if e == self.rc[iu] {
                rep.moved_rows += 1;
                let a = self.rs[iu];
                let len = e - a;
                let extra = (((len as i32) / 2).max(4)).min(room) as i64;
                let ns = self.sc.x160;
                for j in 0..len {
                    let (ci, cv) = (self.row_idx(a + j), self.row_val(a + j));
                    self.set_row(ns + j, ci, cv);
                }
                self.rs[iu] = ns;
                self.re[iu] = ns + len;
                self.rc[iu] = ns + len + extra;
                self.sc.x160 += len + extra;
                e = self.re[iu];
            }
            self.set_row(e, ri, v);
            self.re[iu] = e + 1;
        }
        // C:1313–1329
        if np == 0 {
            self.sc.xe8 = self.sc.xe8.max(self.upos(ri));
        }
        self.sc.x18c += 1;
        self.cb[r] = head;
        self.spike.count = 0;
        self.exit();
        Ok(rep)
    }

    /// Common exit of `05b257` (C:2765–2770).
    fn exit(&mut self) {
        self.cache.count = 0;
        self.x1d8 = -1;
    }

    /// The push-state part of a refactor (C:2006–2090, 2765–2770): spike values at its list cleared, count 0;
    /// r' = n[r]; header'[p] = header[o[p]] written into the spike index buffer, which then swaps with the header
    /// buffer; vstat[header'[p]] = p. `o`, `n` are F+0x28 and F+0x38 at the last `036ade` return.
    pub fn renumber(&mut self, header: &[i32], leave: i32, vstat: &[i32], o: &[i32], n: &[i32]) -> Renumbered {
        let m = self.m;
        for k in 0..self.spike.count.max(0) as usize {
            let i = self.spike.idx.get(k) as usize;
            self.spike.val.set(i, 0.0);
        }
        self.spike.count = 0;
        let leave = if leave >= 0 { n[leave as usize] } else { leave };
        let new: Vec<i32> = (0..m).map(|p| header[o[p] as usize]).collect();
        let mut vstat = vstat.to_vec();
        for p in 0..m {
            vstat[new[p] as usize] = p as i32;
        }
        // the spike index buffer is now the old header buffer
        for (k, &h) in header.iter().enumerate() {
            self.spike.idx.set(k, h);
        }
        self.header = new.clone();
        self.exit();
        Renumbered { header: new, leave, vstat }
    }

    /// The supplier wrapper `09d008` before `2207de` when it passes the spike: the listed values to +0, count 0.
    pub fn wrapper_clear(&mut self) {
        for k in 0..self.spike.count.max(0) as usize {
            let i = self.spike.idx.get(k) as usize;
            self.spike.val.set(i, 0.0);
        }
        self.spike.count = 0;
    }

    /// `2207de` after its wrapper (`09d008` with the spike F+0x1c8 after `wrapper_clear`, or `09cf9b` without).
    pub fn ftran(&mut self, count: i32, idx: &[i32], val: &[f64], save: i32, spike: bool) -> Result<FtranState, LuError> {
        let m = self.m;
        let f = self.ftran_factor();
        let sp = spike.then(|| self.spike.clone());
        let mut s = FtranState::new(m, count, idx, val, sp);
        s.bucket = self.bucket.clone();
        std::mem::swap(&mut s.mark, &mut self.mark);
        std::mem::swap(&mut s.vals, &mut self.vals);
        let r = crossover_ftran::ftran(&f, &mut s, save);
        std::mem::swap(&mut s.mark, &mut self.mark);
        std::mem::swap(&mut s.vals, &mut self.vals);
        r.map_err(LuError::Ftran)?;
        if let Some(bk) = s.bucket.take() {
            self.bucket = Some(bk);
        }
        if let Some(sp) = s.spike.take() {
            self.spike = sp;
        }
        Ok(s)
    }

    /// The support BTRAN `221173` with the cache F+0x1d0 (`cache`) or a null cache.
    pub fn btran(&mut self, count: i32, idx: &[i32], val: &[f64], out_count: i32, cache: bool, work: f64) -> BtranState {
        let m = self.m;
        let f = self.btran_factor();
        let c = cache.then(|| self.cache.clone());
        let mut s = BtranState::new(m, count, idx, val, out_count, c, self.x1d8, work, self.bucket.clone());
        std::mem::swap(&mut s.mark, &mut self.mark);
        std::mem::swap(&mut s.vals, &mut self.vals);
        crossover_btran::btran(&f, &mut s);
        std::mem::swap(&mut s.mark, &mut self.mark);
        std::mem::swap(&mut s.vals, &mut self.vals);
        if let Some(c) = s.cache.take() {
            self.cache = c;
        }
        self.x1d8 = s.x1d8;
        self.bucket = s.bucket.clone();
        s
    }
}
