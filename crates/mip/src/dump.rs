//! Loading a state from `tools/mip_state_dump.js` output and running one reduction on it, so each reduction can be
//! compared with Gurobi's state at its return (tools/mip_dump_check.py). Node arena positions are not in the dump:
//! live nodes are matched between the row and column lists by (row, column); dead entries, the pending list and the
//! free list get fresh nodes (only list order and counts are observable).
use crate::state::{Node, State, NIL};
use crate::tracker::{Queue, Tracker};
use crate::Params;
use std::collections::HashMap;

/// One kind of the change tracker as dumped.
#[derive(Clone, Debug, Default)]
pub struct QueueParts {
    pub flag: Vec<u8>,
    pub a: [usize; 3],
    pub b: [usize; 3],
    pub list: [Vec<i32>; 3],
    pub cache: [Option<Vec<i32>>; 3],
    pub cache2: [Option<Vec<i32>>; 3],
}

/// The dumped state (field names as in the dump).
#[derive(Clone, Debug, Default)]
pub struct Parts {
    pub n: usize,
    pub m: usize,
    pub lb: Vec<f64>,
    pub ub: Vec<f64>,
    pub obj: Vec<f64>,
    pub rhs: Vec<f64>,
    pub sense: Vec<u8>,
    pub vtype: Vec<u8>,
    pub row_cnt: Vec<i32>,
    pub col_cnt: Vec<i32>,
    pub inf_max: Vec<i32>,
    pub inf_min: Vec<i32>,
    pub act_max: Vec<f64>,
    pub act_min: Vec<f64>,
    pub mult_lo: Vec<f64>,
    pub mult_hi: Vec<f64>,
    pub col_flag: Vec<u32>,
    pub objcon: f64,
    pub nnz: i64,
    pub work: f64,
    pub work_mult: f64,
    /// +0x3f8 .. +0x428 (13 counters)
    pub counters: Vec<i32>,
    pub nfree: i64,
    pub npend: i64,
    /// [row, column, value] per list, in list order
    pub rows: Vec<Vec<(i32, i32, f64)>>,
    pub cols: Vec<Vec<(i32, i32, f64)>>,
    pub level: usize,
    pub active: [bool; 3],
    pub r: QueueParts,
    pub c: QueueParts,
    /// +0xf0 when dumped (-1: count the 'B' columns; Gurobi keeps the count from bound rounding)
    pub nbin: i32,
}

fn queue(q: &QueueParts) -> Queue {
    Queue { flag: q.flag.clone(), a: q.a, b: q.b, list: q.list.clone(), cache: q.cache.clone(), cache2: q.cache2.clone() }
}

fn queue_parts(q: &Queue) -> QueueParts {
    let list = [0, 1, 2].map(|l| q.list[l][..q.b[l].min(q.list[l].len())].to_vec());
    QueueParts { flag: q.flag.clone(), a: q.a, b: q.b, list, cache: q.cache.clone(), cache2: q.cache2.clone() }
}

impl State {
    /// A state from dumped parts, with default parameters for what the dump does not hold.
    pub fn from_parts(d: &Parts, p: &Params) -> State {
        let (n, m) = (d.n, d.m);
        let mut nd: Vec<Node> = Vec::new();
        let new = |nd: &mut Vec<Node>, v: f64, r: i32, c: i32| {
            nd.push(Node { val: v, row: r, col: c, nr: NIL, nc: NIL });
            (nd.len() - 1) as u32
        };
        let mut key = HashMap::new();
        let mut row_head = vec![NIL; m];
        for (i, l) in d.rows.iter().enumerate() {
            let mut tail = NIL;
            for &(r, c, v) in l {
                let at = new(&mut nd, v, r, c);
                if c >= 0 {
                    key.insert((r, c), at);
                }
                if tail == NIL { row_head[i] = at } else { nd[tail as usize].nr = at }
                tail = at;
            }
        }
        let mut col_head = vec![NIL; n];
        for (j, l) in d.cols.iter().enumerate() {
            let mut tail = NIL;
            for &(r, c, v) in l {
                let at = if c >= 0 { key[&(r, c)] } else { new(&mut nd, v, r, c) };
                if tail == NIL { col_head[j] = at } else { nd[tail as usize].nc = at }
                tail = at;
            }
        }
        let chain = |nd: &mut Vec<Node>, k: i64| {
            let mut head = NIL;
            for _ in 0..k {
                let at = new(nd, 0.0, 0, -1);
                nd[at as usize].nc = head;
                head = at;
            }
            head
        };
        let pend = chain(&mut nd, d.npend);
        let free = chain(&mut nd, d.nfree);
        let cnt = |t: u8| d.vtype.iter().filter(|&&x| x == t).count() as i32;
        let ctr = |k: usize| d.counters.get(k).copied().unwrap_or(0);
        let mut tracker = Tracker::new(m, n);
        tracker.level = d.level;
        tracker.active = d.active;
        tracker.rows = queue(&d.r);
        tracker.cols = queue(&d.c);
        State {
            ptype: 0,
            n,
            m,
            n0: n,
            m0: m,
            row_cnt: d.row_cnt.clone(),
            col_cnt: d.col_cnt.clone(),
            inf_max: d.inf_max.clone(),
            act_max: d.act_max.clone(),
            inf_min: d.inf_min.clone(),
            act_min: d.act_min.clone(),
            mult_lo: d.mult_lo.clone(),
            mult_hi: d.mult_hi.clone(),
            lb: d.lb.clone(),
            ub: d.ub.clone(),
            obj: d.obj.clone(),
            rhs: d.rhs.clone(),
            sense: d.sense.clone(),
            objcon: d.objcon,
            nnz: d.nnz,
            nbin: if d.nbin >= 0 { d.nbin } else { cnt(b'B') },
            nint: cnt(b'B') + cnt(b'I'),
            eq_rows: false,
            vtype: d.vtype.clone(),
            col_flag: d.col_flag.clone(),
            node_cap: nd.len() as i64,
            nd,
            row_head,
            col_head,
            free,
            nfree: d.nfree,
            pend,
            npend: d.npend,
            chunk: 10000,
            prelimit: p.prelimit,
            precrush: p.precrush,
            opt_target_off: true,
            intfeastol: p.intfeastol,
            budget: 1e100,
            tracker,
            bounds_tightened: ctr(0),
            coefs_changed: ctr(1),
            ineq_to_eq: ctr(2),
            rows_removed: ctr(3),
            cols_removed: ctr(4),
            ctr: [5, 6, 7, 8, 9, 10, 11, 12].map(ctr),
            mark: vec![false; n.max(m)],
            work: d.work,
            work_mult: d.work_mult,
            work_base: 0.0,
            bad_col: -1,
            bad_row: -1,
            cutoff: 1e100,
            post: Vec::new(),
            cg: Default::default(),
            row_int: vec![0.0; m],
        }
    }

    /// The state in dump form.
    pub fn to_parts(&self) -> Parts {
        let walk = |mut q: u32, row: bool| {
            let mut v = Vec::new();
            while q != NIL {
                let x = self.nd[q as usize];
                v.push((x.row, x.col, x.val));
                q = if row { x.nr } else { x.nc };
            }
            v
        };
        let mut counters = vec![self.bounds_tightened, self.coefs_changed, self.ineq_to_eq, self.rows_removed, self.cols_removed];
        counters.extend_from_slice(&self.ctr);
        Parts {
            nbin: self.nbin,
            n: self.n,
            m: self.m,
            lb: self.lb.clone(),
            ub: self.ub.clone(),
            obj: self.obj.clone(),
            rhs: self.rhs.clone(),
            sense: self.sense.clone(),
            vtype: self.vtype.clone(),
            row_cnt: self.row_cnt.clone(),
            col_cnt: self.col_cnt.clone(),
            inf_max: self.inf_max.clone(),
            inf_min: self.inf_min.clone(),
            act_max: self.act_max.clone(),
            act_min: self.act_min.clone(),
            mult_lo: self.mult_lo.clone(),
            mult_hi: self.mult_hi.clone(),
            col_flag: self.col_flag.clone(),
            objcon: self.objcon,
            nnz: self.nnz,
            work: self.work,
            work_mult: self.work_mult,
            counters,
            nfree: self.nfree,
            npend: self.npend,
            rows: (0..self.m).map(|i| walk(self.row_head[i], true)).collect(),
            cols: (0..self.n).map(|j| walk(self.col_head[j], false)).collect(),
            level: self.tracker.level,
            active: self.tracker.active,
            r: queue_parts(&self.tracker.rows),
            c: queue_parts(&self.tracker.cols),
        }
    }

    /// Run one reduction by Gurobi function id (`36e6b6:<mode>` for the GC; `implied_bounds` for the inline block,
    /// `370047` for an all-row activity recompute). Returns (status, extra output: `36c5cf`'s garbage count or
    /// `56c80e`'s status); None for an unknown name.
    pub fn run_named(&mut self, name: &str, p: &Params) -> Option<(i32, i64)> {
        Some(match name {
            "3e39a6" => (self.col_pass(p), 0),
            "36ddf2" => (self.row_clean(p), 0),
            "37168e" => (self.bound_tighten(p), 0),
            "36c5cf" => {
                let mut gc = 0;
                (self.singleton_rows(p, &mut gc), gc)
            }
            "56f793" => (self.coef_strengthen(p), 0),
            "56c80e" => {
                let mut s = 0;
                (self.doubleton_agg(p, &mut s), s as i64)
            }
            "36e6b6:0" => {
                self.gc(0);
                (0, 0)
            }
            "36e6b6:1" => {
                self.gc(1);
                (0, 0)
            }
            "implied_bounds" => {
                self.implied_bounds(p);
                (0, 0)
            }
            "45885d" => (self.clique(p), 0),
            "52be10" => (self.set_packing(p), 0),
            "46a62d" => (self.clique_rows(p, if p.presolve > 1 || p.preclique > 1 { 2 } else { 1 }), 0),
            "5a9282" => (self.r5a9282(p), 0),
            "5ab429" => (self.r5ab429(p), 0),
            "59c193" => (self.vub_lift(p), 0),
            "55a779" => (self.agg_pass(p), 0),
            "5577ba" => {
                // nodes 20000 (Symmetry < 2); the timer-24 work limit is not in the dump: none
                let mut f4 = 0;
                (self.symmetry(p, 20000, 1e100, &mut f4), f4 as i64)
            }
            "370047" => {
                self.activity(None);
                (0, 0)
            }
            _ => return None,
        })
    }
}
