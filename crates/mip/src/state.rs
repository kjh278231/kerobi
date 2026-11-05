//! MIP presolve state: `3a484d(model, mipctx, 1, work)`, the branch with state+0xfc = 1 and state+0xe0 = 0, which
//! keeps the matrix as linked lists of 0x20-byte nodes instead of the CSR of `5118e6` (docs/m2_presolve_state.md).
//! Field comments give the original state offset. Removal is lazy as in the LP port: a removed node stays in both
//! lists with column -1, removed rows and columns get a negative live count. Model generation (`3b0917`) walks the
//! column lists, so node order is what decides the presolved model's entry order.
use crate::post::PostRec;
use crate::tracker::Tracker;
use crate::Params;
use rsolver_model::Model;

/// Null link.
pub const NIL: u32 = u32::MAX;
/// Index of +0x40c (types changed) in `State::ctr`; +0x410 is 1, ..., +0x428 is 7.
pub const CTR_TYPES: usize = 0;

/// `3c546c` snapshot: n, m, +0x1f8 SOS, +0x27c QC (both 0 for linear models), +0x408, +0x404, +0x400, +0x3fc,
/// +0x3f8, then +0x40c..+0x428.
#[derive(Clone, Copy, Debug, Default)]
pub struct Snapshot([i32; 17]);
/// env+0x2458 GURO_PAR_INFBOUND at its default (as in the LP port)
pub const INF: f64 = rsolver_model::INFINITY;

/// One matrix node (0x20 bytes): value +0x00, row +0x08, column +0x0c (-1 = removed), next in the row +0x10, next in
/// the column (or in the free / pending list) +0x18. Links are arena indices.
#[derive(Clone, Copy, Debug)]
pub struct Node {
    pub val: f64,
    pub row: i32,
    pub col: i32,
    pub nr: u32,
    pub nc: u32,
}

#[derive(Clone, Debug)]
pub struct State {
    /// +0x08 problem type from the constructor scans: 1 (all ±1, more than half the rows `=`), 4 (network: columns of
    /// at most two entries, as many +1 as -1, all rows `=`), else 0. shortcut: 11 (needs the objective gcd `a7bea8`)
    /// is never set; Model.presolve() turns it back into 0 before any reduction reads it (3cd406 L1668), port
    /// `a7bea8` when a MIP context (restart presolve) is ported.
    pub ptype: i32,
    /// +0x0c / +0x10 columns and rows now; +0xe8 / +0xe4 at construction
    pub n: usize,
    pub m: usize,
    pub n0: usize,
    pub m0: usize,
    /// +0x60 / +0x68 live entries per row / column; negative = removed (-1 empty row, -2 free row)
    pub row_cnt: Vec<i32>,
    pub col_cnt: Vec<i32>,
    /// +0x70 / +0x90: infinite contributions to, and finite part of, max activity - rhs
    pub inf_max: Vec<i32>,
    pub act_max: Vec<f64>,
    /// +0x78 / +0x98: same for min activity - rhs
    pub inf_min: Vec<i32>,
    pub act_min: Vec<f64>,
    /// +0x80 / +0x88: per-column growth bounds of substitution multipliers (start at 1.0)
    pub mult_lo: Vec<f64>,
    pub mult_hi: Vec<f64>,
    /// +0xa0 / +0xa8 / +0xb0 (objective in minimization form)
    pub lb: Vec<f64>,
    pub ub: Vec<f64>,
    pub obj: Vec<f64>,
    /// +0xb8 / +0xc0 ('<' or '='; '>' rows are stored negated, as in the LP port)
    pub rhs: Vec<f64>,
    pub sense: Vec<u8>,
    /// +0xc8 objective constant (minimization form)
    pub objcon: f64,
    /// +0xd8 model nonzeros, tiny ones included
    pub nnz: i64,
    /// +0xf0 / +0xf4 binary and integer columns (counted by 3cd406's bound rounding, 0 until then)
    pub nbin: i32,
    pub nint: i32,
    /// +0xf8 set by 3cd406 L938-1014: >= 99% binaries, m > 1.2 n and more than m/1000 `=` rows
    pub eq_rows: bool,
    /// +0x150 column types (a copy of the model's; 3cd406 turns [0,1] integers into 'B')
    pub vtype: Vec<u8>,
    /// +0x170 per-column flags: 0x40 MINBPFORBID, 1/2 SOS member, 4 semicontinuous; all 0 for the models handled
    pub col_flag: Vec<u32>,
    /// +0x160 node arena (+0x158 its capacity: spare + nnz; positions of tiny entries stay unused),
    /// +0x178 / +0x180 row and column list heads
    pub nd: Vec<Node>,
    pub node_cap: i64,
    pub row_head: Vec<u32>,
    pub col_head: Vec<u32>,
    /// +0x1a0 / +0x198 free list (nodes nnz.. in order) and its count, +0x190 / +0x188 pending list (nodes out of
    /// their column that may still sit in a row list), +0x1a8 size of the next chunk `3b85ea` grabs (= spare)
    pub free: u32,
    pub nfree: i64,
    pub pend: u32,
    pub npend: i64,
    pub chunk: i64,
    /// +0x1dc GURO_PAR_PRELIMIT (env+0x26ec), +0x1e0 PreCrush, +0x1e4 OptimalityTarget != 1, +0x1e8 IntFeasTol
    pub prelimit: i32,
    pub precrush: i32,
    pub opt_target_off: bool,
    pub intfeastol: f64,
    /// +0x1f0 the `3ba9de` budget: 1e100 here (3cd406 limits work through its reduction timers instead)
    pub budget: f64,
    /// +0x3d8
    pub tracker: Tracker,
    /// +0x3f8 / +0x3fc / +0x400 / +0x404 / +0x408
    pub bounds_tightened: i32,
    pub coefs_changed: i32,
    pub ineq_to_eq: i32,
    pub rows_removed: i32,
    pub cols_removed: i32,
    /// +0x40c types changed, +0x410 SOS removed, +0x414, +0x418 QC removed, +0x41c, +0x420, +0x424, +0x428: the
    /// other progress counters of `3c5595` (see `CTR_*`)
    pub ctr: [i32; 8],
    /// +0x3a8 per-index scratch marks of the list getters (sized for rows and columns)
    pub mark: Vec<bool>,
    /// *(+0x458) deterministic work, +0x448 its multiplier, +0x450 the work when 3cd406 started
    pub work: f64,
    pub work_mult: f64,
    pub work_base: f64,
    /// +0x470 / +0x478 column / row proving infeasibility (+0x474.. +0x484 also start at -1)
    pub bad_col: i32,
    pub bad_row: i32,
    /// +0x490 objective cutoff: 1e100 without a MIP context or incumbent
    pub cutoff: f64,
    /// +0x370 postsolve records, in push order
    pub post: Vec<PostRec>,
    /// +0x3f0 conflict graph (`5b84b1`): probing's implications
    pub cg: crate::conflict::ConflictGraph,
    /// +0x3b0 per-row scratch of `3e39a6`'s integrality check (0.0 = not computed; reset at the end of each call)
    pub row_int: Vec<f64>,
}

/// Rounding guard of `4f79a1`/`36fcb5`: if `s = old + t` was inexact, push `s` one ulp in the safe direction.
pub fn guarded_add(old: f64, t: f64, up: bool) -> f64 {
    let s = old + t;
    let inexact = if old.abs() < t.abs() { s - t != old } else { s - old != t };
    if !inexact {
        return s;
    }
    let grow = if up { s > 0.0 } else { s < 0.0 };
    s * if grow { 1.0000000000000002 } else { 0.9999999999999998 }
}

impl State {
    /// `3c546c`.
    pub fn snapshot(&self) -> Snapshot {
        let mut s = [self.n as i32, self.m as i32, 0, 0, self.cols_removed, self.rows_removed, self.ineq_to_eq,
                     self.coefs_changed, self.bounds_tightened, 0, 0, 0, 0, 0, 0, 0, 0];
        s[9..].copy_from_slice(&self.ctr);
        Snapshot(s)
    }

    /// `3c5595`: progress since snapshot `s` (12 §3.4), in the original's operation order.
    pub fn progress(&self, s: &Snapshot) -> f64 {
        let s = &s.0;
        let c = &self.ctr;
        let cols = (s[0] - s[4]).max(1) as f64;
        let rows = ((s[1] - s[5]) + (s[2] - s[10]) + (s[3] - s[12])).max(1) as f64;
        let nnz = self.nnz.max(1) as f64;
        let d_cols = ((self.cols_removed - s[4]) - (self.n as i32 - s[0])).max(0);
        let d_rows = ((self.rows_removed - s[5]) - (self.m as i32 - s[1])).max(0);
        let d_sos = ((c[1] - s[10]) - (0 - s[2])).max(0);
        let d_qc = ((c[3] - s[12]) - (0 - s[3])).max(0);
        ((self.coefs_changed - s[7]) as f64 * 0.1) / nnz
            + d_cols as f64 / cols
            + ((((self.bounds_tightened - s[8]) + (c[0] - s[9])) + (c[4] - s[13]) + (c[5] - s[14])) as f64 * 0.0001) / cols
            + (d_rows + d_sos + d_qc + (c[6] - s[15]) + (c[7] - s[16])) as f64 / rows
            + ((self.ineq_to_eq - s[6]) as f64 * 0.5) / rows
            + ((c[2] - s[11]) as f64 * 0.001) / rows
    }

    /// `3ba9de` without time limits: past the budget, PRELIMIT becomes -1 so the next count ends presolve.
    pub fn budget_check(&mut self) {
        if self.work - self.work_base > self.budget {
            self.prelimit = -1;
        }
    }

    /// `3a484d` with the MIP flag. Work terms are added in the original order (asm checked where the
    /// multiplication order matters for μ = +0x448 != 1).
    /// shortcut: linear models only (no SOS, semicontinuous, Q, QC, general constraints, PWL, MINBPFORBID marks,
    /// model+0x258 row marks, MIP context / cutoff); add those branches when such a model is compared.
    pub fn new(md: &Model, p: &Params) -> State {
        let (n, m) = (md.num_vars(), md.num_constrs());
        assert!(md.vtype.iter().all(|&t| t != b'S' && t != b'N'), "semicontinuous columns are not ported");
        let neg: Vec<bool> = md.row_sense.iter().map(|&s| s == b'>').collect();
        let raw: usize = md.col_len.iter().map(|&l| l as usize).sum();
        let spare = if raw / 10 <= 2_000_000_000 { (raw as i64 / 10).max(n.max(m).max(10000) as i64) } else { 2_000_000_000 };
        let mu = if raw <= 100_000 { 1.0 } else { (raw as f64 / 1e4).log10().sqrt() };
        let (nf, mf) = (n as f64, m as f64);
        let mut w = nf; // L837 'S'/'N' scan
        w += nf; // L962 nonzero count
        w += (nf * 1.5 + mf) * mu; // L1397
        w += (nf + nf) * mu; // L1562 +0x80/+0x88
        w += (nf * 0.5) * mu; // L1594 vtype copy
        w += (spare - 1) as f64 * mu; // L4004 free list
        let s = md.sense as f64;
        // presolve works on the minimization form; `to_model` turns it back
        let obj: Vec<f64> = md.obj.iter().map(|&c| c * s).collect();
        let objcon = md.objcon * s;
        let rhs: Vec<f64> = md.rhs.iter().zip(&neg).map(|(&r, &g)| if g { -r } else { r }).collect();
        let col = |j: usize| md.col_beg[j] as usize..md.col_beg[j] as usize + md.col_len[j] as usize;

        // L4021: with dual reductions, an objective on a continuous column asks which rows keep a continuous
        // column integral (+0x390 scratch: 0 integral rhs, then +1 per continuous ±1 entry; n = no)
        let scan = p.dual_reductions != 0 && n > 1; // +0x250 Q, env+0x2728 sub-MIP and semicontinuous count are 0 here
        let mut int_rows = vec![0i32; m];
        let mut check = false;
        if scan {
            let j0 = (0..n).position(|j| 1e-10 < obj[j].abs() && md.vtype[j] == b'C').unwrap_or(n);
            w += (j0 as f64 + j0 as f64) * mu;
            if j0 < n {
                for i in 0..m {
                    int_rows[i] = if rhs[i] - (rhs[i] + 1e-10).floor() <= 1e-10 { 0 } else { n as i32 };
                }
                w += (mf + mf) * mu;
                check = true;
            }
        }
        // L4193: entries per row (|a| > 1e-13), then row starts; nodes are laid out row by row
        let mut start = vec![0usize; m + 2];
        for j in 0..n {
            for k in col(j) {
                if 1e-13 < md.col_val[k].abs() {
                    start[md.col_ind[k] as usize + 2] += 1;
                }
            }
            w += md.col_len[j] as f64 * (mu * 3.0);
        }
        w += (nf + nf) * mu;
        for i in 0..m {
            start[i + 2] += start[i + 1];
        }
        w += (m + 1) as f64 * mu;
        // L4347: columns in order fill their rows' slots, so a row list is in column order and a column list in
        // the model's entry order
        let cap = spare as usize + raw;
        let mut nd = vec![Node { val: 0.0, row: 0, col: -1, nr: NIL, nc: NIL }; cap];
        let mut col_head = vec![NIL; n];
        let (mut maxlen, mut plus, mut minus, mut other) = (0, 0, 0, 0);
        for j in 0..n {
            maxlen = maxlen.max(md.col_len[j]);
            let mut tail = NIL;
            for k in col(j) {
                let a = md.col_val[k];
                if !(1e-13 < a.abs()) {
                    continue;
                }
                let i = md.col_ind[k] as usize;
                let v = if neg[i] { -a } else { a };
                if v == 1.0 {
                    plus += 1;
                } else if v == -1.0 {
                    minus += 1;
                } else {
                    other += 1;
                }
                let at = start[i + 1];
                start[i + 1] += 1;
                nd[at] = Node { val: v, row: i as i32, col: j as i32, nr: NIL, nc: NIL };
                if tail == NIL {
                    col_head[j] = at as u32;
                } else {
                    nd[tail as usize].nc = at as u32;
                }
                tail = at as u32;
                if check {
                    if 1e-10 < v - (1e-10 + v).floor() {
                        int_rows[i] = n as i32;
                    } else if md.vtype[j] == b'C' {
                        if 1e-10 > (v.abs() - 1.0).abs() {
                            int_rows[i] += 1;
                        } else {
                            int_rows[i] = n as i32;
                        }
                    }
                }
            }
            w += md.col_len[j] as f64 * (mu * 6.0);
        }
        w += (nf + nf) * mu;
        let mut row_head = vec![NIL; m];
        for i in 0..m {
            let (b, e) = (start[i], start[i + 1]);
            if b == e {
                continue;
            }
            row_head[i] = b as u32;
            for at in b..e - 1 {
                nd[at].nr = (at + 1) as u32;
            }
            w += (e - 1 - b) as f64 * mu;
        }
        w += (mf * 3.0) * mu;
        // L4437 problem type scans (D's coefficients: '>' rows negated)
        let mut ptype = 0;
        if maxlen >= 5 {
            if plus != 0 && minus != 0 && other == 0 {
                let eq = md.row_sense.iter().filter(|&&c| c == b'=').count();
                w += mf * mu;
                if m < 2 * eq {
                    ptype = 1;
                }
            }
        } else if maxlen <= 2 && other == 0 && plus == minus {
            let c = md.row_sense.iter().take_while(|&&c| c == b'=').count();
            w += c as f64 * mu;
            if c == m {
                ptype = 4;
            }
        }
        if scan {
            objective_scan(md, &obj, &int_rows, &mut w, mu);
        }
        // L4752 common tail: '>' rows become '<', live counts, tracker
        let sense: Vec<u8> = md.row_sense.iter().map(|&c| if c == b'>' { b'<' } else { c }).collect();
        w += mf * mu;
        let row_cnt: Vec<i32> = (0..m).map(|i| (start[i + 1] - start[i]) as i32).collect();
        w += (mf + mf) * mu;
        let mut col_cnt = vec![0i32; n];
        for j in 0..n {
            col_cnt[j] = col(j).filter(|&k| 1e-13 < md.col_val[k].abs()).count() as i32;
            w += (md.col_len[j] as f64 * mu) * 2.0;
        }
        w += (nf * 3.0) * mu;
        // PREINFBND / PREINFOBJ (env+0x2668 / +0x2670) are infinite by default: no bound clipping
        for _ in 0..3 {
            w += mf * (mu * 3.0);
            w += (mu * 3.0) * nf;
        }
        w += 48.0 * mu;
        // free list: nodes raw.. linked in order (L3951)
        for at in raw..cap {
            nd[at].nc = if at + 1 < cap { (at + 1) as u32 } else { NIL };
        }
        State {
            ptype,
            n,
            m,
            n0: n,
            m0: m,
            row_cnt,
            col_cnt,
            inf_max: vec![0; m],
            act_max: vec![0.0; m],
            inf_min: vec![0; m],
            act_min: vec![0.0; m],
            mult_lo: vec![1.0; n],
            mult_hi: vec![1.0; n],
            lb: md.lb.clone(),
            ub: md.ub.clone(),
            obj,
            rhs,
            sense,
            objcon,
            nnz: raw as i64,
            nbin: 0,
            nint: 0,
            eq_rows: false,
            vtype: md.vtype.clone(),
            col_flag: vec![0; n],
            nd,
            node_cap: cap as i64,
            row_head,
            col_head,
            free: if spare > 0 { raw as u32 } else { NIL },
            nfree: spare,
            pend: NIL,
            npend: 0,
            chunk: spare,
            prelimit: p.prelimit,
            precrush: p.precrush,
            opt_target_off: true, // OptimalityTarget (env+0x2af0) -1 by default
            intfeastol: p.intfeastol,
            budget: 1e100,
            tracker: Tracker::new(m, n),
            bounds_tightened: 0,
            coefs_changed: 0,
            ineq_to_eq: 0,
            rows_removed: 0,
            cols_removed: 0,
            ctr: [0; 8],
            mark: vec![false; n.max(m)],
            work: w,
            work_mult: mu,
            work_base: 0.0,
            bad_col: -1,
            bad_row: -1,
            cutoff: 1e100,
            post: Vec::new(),
            cg: Default::default(),
            row_int: vec![0.0; m],
        }
    }

    /// `370047` + `36fcb5`: activities of the given rows (all when None) from the row lists, summed from 0, then
    /// -rhs added (as the LP port's aggregation does).
    pub fn activity(&mut self, rows: Option<&[i32]>) {
        let all: Vec<i32>;
        let rows = match rows {
            Some(r) => r,
            None => {
                all = (0..self.m as i32).collect();
                &all
            }
        };
        for &i in rows {
            let i = i as usize;
            if self.row_cnt[i] < 0 {
                continue;
            }
            let (mut mx, mut mn, mut imx, mut imn) = (0.0, 0.0, 0, 0);
            let mut p = self.row_head[i];
            let mut visited = 0;
            while p != NIL {
                let q = self.nd[p as usize];
                p = q.nr;
                visited += 1;
                if q.col < 0 {
                    continue;
                }
                let (k, a) = (q.col as usize, q.val);
                let (l, u) = (self.lb[k], self.ub[k]);
                if a <= 0.0 {
                    if -INF < l { mx = guarded_add(mx, l * a, true) } else { imx += 1 }
                    if u < INF { mn = guarded_add(mn, a * u, false) } else { imn += 1 }
                } else {
                    if u < INF { mx = guarded_add(mx, u * a, true) } else { imx += 1 }
                    if -INF < l { mn = guarded_add(mn, a * l, false) } else { imn += 1 }
                }
            }
            self.work += self.work_mult * 7.0 * visited as f64;
            self.inf_max[i] = imx;
            self.inf_min[i] = imn;
            self.act_max[i] = guarded_add(mx, -self.rhs[i], true);
            self.act_min[i] = guarded_add(mn, -self.rhs[i], false);
            let tiny = self.rhs[i].abs() * 1e-13;
            if 0.0 < self.act_min[i] && self.act_min[i] < tiny {
                self.act_min[i] = 0.0;
            }
            if self.act_max[i] < 0.0 && -self.act_max[i] < tiny {
                self.act_max[i] = 0.0;
            }
        }
        self.work += self.work_mult * 8.0 * rows.len() as f64;
    }

    /// `36d89a`: drop empty rows (infeasible if their rhs is violated) and `<` rows with rhs >= 1e20 (their nodes
    /// leave the columns, which get marked). Returns 3 with `bad_row` set when infeasible, else 0.
    pub fn empty_and_free_rows(&mut self, feastol: f64) -> i32 {
        let mut removed = 0;
        let mut rc = 0;
        for i in 0..self.m {
            if self.row_cnt[i] == 0 {
                let r = self.rhs[i];
                if (self.sense[i] == b'=' && feastol < r.abs()) || (self.sense[i] != b'=' && r < -feastol) {
                    self.bad_row = i as i32;
                    rc = 3;
                    break;
                }
                self.row_cnt[i] = -1;
                removed += 1;
            } else if self.row_cnt[i] > 0 && self.sense[i] != b'=' && 1e20 <= self.rhs[i] {
                removed += 1;
                let mut p = self.row_head[i];
                while p != NIL {
                    let q = &mut self.nd[p as usize];
                    if q.col >= 0 {
                        let j = q.col;
                        q.col = -1;
                        self.tracker.mark_col(j);
                        self.col_cnt[j as usize] -= 1;
                    }
                    p = self.nd[p as usize].nr;
                }
                self.work += (self.row_cnt[i] as f64 * 3.0) * self.work_mult;
                self.row_cnt[i] = -2;
            }
        }
        if rc == 0 {
            self.work += (self.m as f64 * 3.0) * self.work_mult;
        }
        self.rows_removed += removed;
        rc
    }

    /// Presolved model (`3b0917`, MIP state): live rows and columns in original order, each column's live nodes in
    /// list order, types from +0x150. Gurobi names it `<name>_pre`; with nothing left `999702` makes the empty model
    /// `Presolved` (minimize, minimization-form constant), as in the LP port.
    /// shortcut: no split-row range columns or other appended columns (none before reductions run).
    pub fn to_model(&self, md: &Model) -> Model {
        let mut row_map = vec![-1i32; self.m];
        let s = md.sense as f64;
        let mut q = Model { name: format!("{}_pre", md.name), sense: md.sense, objcon: self.objcon * s, ..Model::default() };
        for i in 0..self.m {
            if self.row_cnt[i] >= 0 {
                row_map[i] = q.rhs.len() as i32;
                q.rhs.push(self.rhs[i]);
                q.row_sense.push(self.sense[i]);
                q.constr_names.push(md.constr_names[i].clone());
            }
        }
        for j in 0..self.n {
            if self.col_cnt[j] < 0 {
                continue;
            }
            q.col_beg.push(q.col_ind.len() as i64);
            let mut p = self.col_head[j];
            while p != NIL {
                let node = self.nd[p as usize];
                if node.col >= 0 {
                    q.col_ind.push(row_map[node.row as usize]);
                    q.col_val.push(node.val);
                }
                p = node.nc;
            }
            debug_assert_eq!(q.col_ind.len() as i64 - q.col_beg.last().unwrap(), self.col_cnt[j] as i64, "col {j}");
            q.col_len.push(self.col_cnt[j]);
            q.obj.push(self.obj[j] * s);
            q.lb.push(self.lb[j]);
            q.ub.push(self.ub[j]);
            // an integer column left at [0, 1] comes out binary (never 'I' with these bounds in Gurobi's presolved
            // models, e.g. enlight_hard at PRELIMIT 6 after `37168e`; fixed integers stay 'I')
            let int01 = self.vtype[j] == b'I' && self.lb[j] == 0.0 && self.ub[j] == 1.0;
            q.vtype.push(if int01 { b'B' } else { self.vtype[j] });
            q.var_names.push(md.var_names[j].clone());
        }
        if q.rhs.is_empty() && q.obj.is_empty() {
            q.name = "Presolved".into();
            q.sense = 1;
            q.objcon = self.objcon;
        }
        q
    }
}

/// L4615 (with dual reductions): the objective block that, when every objective column is integral or forced
/// integral by an `=` row, collects |c| for the objective gcd (+0x498 scale, +0x4a0 constant; `a7bea8`), adding its
/// work to `w`. shortcut: the gcd itself is not ported (+0x498/+0x4a0 stay unset); it only feeds problem type 11 and
/// the presolved model's objective-scale attributes, none of which reach the MPS file.
fn objective_scan(md: &Model, obj: &[f64], int_rows: &[i32], w: &mut f64, mu: f64) {
    let n = md.num_vars();
    let mut j = 0;
    while j < n {
        let c = obj[j];
        let (l, u) = (md.lb[j], md.ub[j]);
        if c.abs() < 1e-10 || u - l < 1e-10 || md.col_len[j] == 0 || md.vtype[j] != b'C' {
            j += 1;
            continue;
        }
        // continuous objective column: every row must hold it integral (+0x390 == 1), or an `=` row does
        let b = md.col_beg[j] as usize;
        let mut ok = 1; // col_flag == 0
        let mut seen = md.col_len[j] as usize;
        for (t, &r) in md.col_ind[b..b + seen].iter().enumerate() {
            if int_rows[r as usize] == 1 {
                if md.row_sense[r as usize] == b'=' {
                    ok = 2;
                    seen = t;
                    break;
                }
            } else {
                ok = 0;
            }
        }
        *w += (seen as f64 * 3.0) * mu;
        if ok == 0 {
            break;
        }
        if ok == 1 {
            let lb_int = !(-INF < l) || l - (l + 1e-10).floor() <= 1e-10;
            if u < INF && 1e-10 < u - (u + 1e-10).floor() || !lb_int {
                break;
            }
        }
        j += 1;
    }
    *w += (j as f64 * 7.0) * mu;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn walk(st: &State, mut p: u32, row: bool) -> Vec<(i32, i32, f64)> {
        let mut v = Vec::new();
        while p != NIL {
            let q = st.nd[p as usize];
            v.push((q.row, q.col, q.val));
            p = if row { q.nr } else { q.nc };
        }
        v
    }

    #[test]
    fn lists_keep_model_order() {
        // row 0 is '>' (stored negated), column 1's 1e-14 entry is dropped
        let md = Model {
            name: "t".into(),
            sense: 1,
            objcon: 0.0,
            obj: vec![1.0, 0.0, 2.0],
            lb: vec![0.0; 3],
            ub: vec![1.0, 5.0, 4.0],
            vtype: b"ICI".to_vec(),
            col_beg: vec![0, 2, 4],
            col_len: vec![2, 2, 1],
            col_ind: vec![1, 0, 0, 1, 0],
            col_val: vec![2.0, 1.0, 1e-14, -1.0, 3.0],
            row_sense: b">=".to_vec(),
            rhs: vec![1.0, 2.0],
            var_names: vec!["a".into(), "b".into(), "c".into()],
            constr_names: vec!["r0".into(), "r1".into()],
        };
        let st = State::new(&md, &Params::default());
        assert_eq!(walk(&st, st.col_head[0], false), [(1, 0, 2.0), (0, 0, -1.0)]);
        assert_eq!(walk(&st, st.col_head[1], false), [(1, 1, -1.0)]);
        assert_eq!(walk(&st, st.row_head[0], true), [(0, 0, -1.0), (0, 2, -3.0)]);
        assert_eq!(walk(&st, st.row_head[1], true), [(1, 0, 2.0), (1, 1, -1.0)]);
        assert_eq!((st.row_cnt.as_slice(), st.col_cnt.as_slice()), ([2, 2].as_slice(), [2, 1, 1].as_slice()));
        assert_eq!((st.sense.as_slice(), st.rhs.as_slice()), (b"<=".as_slice(), [-1.0, 2.0].as_slice()));
        // free list: the 10000 spare nodes after the 5 model entries
        assert_eq!((st.free, st.nfree, st.nd.len()), (5, 10000, 10005));
        let q = st.to_model(&md);
        assert_eq!((q.col_ind, q.col_val), (vec![1, 0, 1, 0], vec![2.0, -1.0, -1.0, -3.0]));
    }
}
