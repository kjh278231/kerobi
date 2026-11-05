//! `451bed`: the clique table built from rows (docs/m2_red_clique.md §2). `45885d` calls it on the `53a487` rows with
//! `bin_only` false; probing (`4d3a1d`) calls it on all rows with `bin_only` true.
use crate::state::{State, NIL};
use crate::Params;

/// Output of `451bed`. A literal `l >= 0` is x_l, `l < 0` the complement of x_{-1-l}; a literal is "on" when its
/// column leaves the bound that gives the row its minimum activity. At most one literal of a clique is on.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Cliques {
    /// clique c is `lits[start[c]..start[c + 1]]` (`start` has one more entry than there are cliques)
    pub start: Vec<i32>,
    pub lits: Vec<i32>,
    /// per clique: b'=' exactly one literal is on, b'<' at most one
    pub sense: Vec<u8>,
    /// per clique: 1 when it has fewer than two non-binary literals (`out_flag`)
    pub flag: Vec<i32>,
    /// `cidx[colstart[j]..colstart[j + 1]]`: the cliques holding column j in increasing order, c for x_j and -1-c for
    /// its complement (`colstart` has n + 1 entries)
    pub colstart: Vec<i32>,
    pub cidx: Vec<i32>,
    /// rows not wholly one clique (written to state+0x390), whether each gave a clique, and their longest live count
    pub rec_rows: Vec<i32>,
    pub rec_flag: Vec<u8>,
    pub maxlen: i32,
}

impl Cliques {
    pub fn len(&self) -> usize {
        self.sense.len()
    }
    pub fn is_empty(&self) -> bool {
        self.sense.is_empty()
    }
}

impl State {
    /// `451bed(env, st, nlist, list, bin_only, ...)`: `list` None = rows 0..m. With no clique everything stays empty
    /// (Gurobi then leaves the row outputs at 0). Err = status (3: column `bad_col` has lb > ub + feastol).
    /// shortcut: the SOS1 cliques (+0x1f8 > 0, `a85f71`) are not ported (linear models only), nor the 2e9 caps on the
    /// literal and clique buffers.
    pub fn clique_table(&mut self, p: &Params, list: Option<&[i32]>, bin_only: bool) -> Result<Cliques, i32> {
        let (ft, mu) = (p.feastol, self.work_mult);
        let all: Vec<i32>;
        let rows = match list {
            Some(l) => l,
            None => {
                all = (0..self.m as i32).collect();
                &all
            }
        };
        let mut t = Cliques::default();
        for &r in rows {
            let r = r as usize;
            if self.row_cnt[r] <= 0 {
                continue;
            }
            let mut so = self.sense[r];
            let (mut s, mut nb, mut produced) = (0i32, 0, 0u8);
            let mark = t.lits.len();
            if self.inf_min[r] == 0 && self.inf_max[r] == 0 {
                let mut ent = 0;
                let mut q = self.row_head[r];
                while q != NIL {
                    let node = self.nd[q as usize];
                    q = node.nr;
                    ent += 1;
                    if node.col < 0 {
                        continue;
                    }
                    let j = node.col as usize;
                    let w = self.ub[j] - self.lb[j];
                    if w <= 1e-10 {
                        if self.ub[j] + ft < self.lb[j] {
                            self.bad_col = j as i32;
                            return Err(3);
                        }
                        let mut mx = 0.0f64;
                        if w > 1e-13 {
                            let mut c = self.col_head[j];
                            while c != NIL {
                                let e = self.nd[c as usize];
                                if e.col >= 0 && mx <= e.val.abs() {
                                    mx = e.val.abs();
                                }
                                c = e.nc;
                            }
                        }
                        if w * mx < ft {
                            // shortcut: `3de18e(v, INF, env, st, j, 1)` fixes the column here (v = 0 when lb, ub are
                            // within 1e-10 of it, else lb if obj > 0, else ub) and the node is skipped; it is the
                            // inner-loop port's helper, wire it in with `45885d` (only neos-860300 gets here).
                            unimplemented!("451bed: 3de18e column fix (col {j}) not merged yet");
                        }
                    }
                    if bin_only && self.vtype[j] != b'B' {
                        so = b'<';
                        continue;
                    }
                    let mut a = node.val;
                    let aa = a.abs();
                    if aa < 1e-4 {
                        so = b'<';
                        continue;
                    }
                    let d = w * aa + self.act_min[r]; // read only when s >= 0
                    if s < 0 || !(d >= -1e-10) {
                        // max-activity mode, `=` rows only
                        if s > 0 || self.sense[r] != b'=' || 1e-10 < self.act_max[r] - w * aa {
                            so = b'<';
                            continue;
                        }
                        a = -a;
                        s -= 1;
                    } else {
                        s += 1;
                        if so == b'=' && d > 1e-10 {
                            so = b'<';
                        }
                    }
                    if self.vtype[j] != b'B' {
                        nb += 1;
                    }
                    t.lits.push(if 0.0 <= a { j as i32 } else { -1 - j as i32 });
                }
                self.work += (ent as f64 * 8.0) * mu;
            }
            let rc = self.row_cnt[r];
            if !(s < 2 && -2 < s) {
                t.start.push(mark as i32);
                t.sense.push(if s == rc { so } else { b'<' });
                t.flag.push((nb < 2) as i32);
                produced = 1;
            } else {
                t.lits.truncate(mark);
            }
            if s != rc && -s != rc {
                t.rec_rows.push(r as i32);
                t.rec_flag.push(produced);
                t.maxlen = t.maxlen.max(rc);
            }
        }
        self.work += (8.0 * rows.len() as f64) * mu;
        if !bin_only && 1 <= self.nbin && self.nbin <= self.n as i32 - 2 {
            self.implied_pairs(&mut t);
        }
        if t.sense.is_empty() {
            return Ok(Cliques::default());
        }
        self.index_cliques(&mut t);
        Ok(t)
    }

    /// `451bed` L1026-1400, `bin_only` false: for each binary b, the non-binary literals of the rows b alone fills
    /// when on (`l0`) and when off (`l1`), at most 16 each, each row used once; every (l0, l1) pair cannot be on
    /// together and becomes a `<` clique (at most 16 per binary).
    fn implied_pairs(&mut self, t: &mut Cliques) {
        let (n, mu) = (self.n, self.work_mult);
        let mut used_row = vec![false; self.m];
        let mut listed = vec![false; n];
        for b in 0..n {
            if self.vtype[b] != b'B' {
                continue;
            }
            let (mut l0, mut l1): (Vec<i32>, Vec<i32>) = (Vec::new(), Vec::new());
            let mut seen = 0;
            let mut q = self.col_head[b];
            while q != NIL {
                if 16 <= l0.len() && 16 <= l1.len() {
                    break;
                }
                let node = self.nd[q as usize];
                q = node.nc;
                seen += 1;
                let r = node.row as usize;
                if node.col < 0 || used_row[r] {
                    continue;
                }
                let ab = node.val;
                let dir = if self.inf_min[r] == 0 && ab.abs() + self.act_min[r] >= -1e-10 {
                    1.0
                } else if self.sense[r] == b'=' && self.inf_max[r] == 0 && 1e-10 >= ab.abs() + self.act_max[r] {
                    -1.0 // asm 0x40b9f6: |a| + (max act - rhs), not max act - rhs - |a|
                } else {
                    continue;
                };
                used_row[r] = true;
                let l = if ab * dir > 0.0 { &mut l0 } else { &mut l1 };
                let mut it = 0;
                let mut e = self.row_head[r];
                while e != NIL && l.len() < 16 {
                    let x = self.nd[e as usize];
                    e = x.nr;
                    it += 1;
                    if x.col >= 0 && self.vtype[x.col as usize] != b'B' && !listed[x.col as usize] {
                        listed[x.col as usize] = true;
                        l.push(if dir * x.val > 0.0 { x.col } else { -1 - x.col });
                    }
                }
                self.work += (it as f64 * 3.0) * mu;
            }
            self.work += (seen as f64 * 7.0) * mu;
            for l in [&l0, &l1] {
                for &x in l {
                    listed[if x < 0 { -1 - x } else { x } as usize] = false;
                }
                self.work += (l.len() as f64 + l.len() as f64) * mu;
            }
            let (mut lo, mut hi) = (l0.len(), l1.len());
            if lo == 0 || hi == 0 {
                continue;
            }
            if 16 < lo * hi {
                if hi < lo {
                    lo = 16 / hi;
                } else {
                    hi = 16 / lo;
                }
            }
            for &x in &l0[..lo] {
                for &y in &l1[..hi] {
                    t.start.push(t.lits.len() as i32);
                    t.sense.push(b'<');
                    t.flag.push(1);
                    t.lits.extend([x, y]);
                }
                self.work += (hi as f64 * 6.0) * mu;
            }
            self.work += lo as f64 * mu;
        }
        self.work += n as f64 * mu;
    }

    /// `451bed` end (L1215-1340): close `start` and build the column index (counting sort, cliques in order).
    fn index_cliques(&mut self, t: &mut Cliques) {
        let (n, mu) = (self.n, self.work_mult);
        let var = |l: i32| if l < 0 { -1 - l } else { l } as usize;
        t.start.push(t.lits.len() as i32);
        self.work += t.lits.len() as f64 * mu;
        let mut cs = vec![0i32; n + 2];
        for &l in &t.lits {
            cs[var(l) + 2] += 1;
        }
        if n >= 1 {
            for j in 0..n {
                cs[j + 2] += cs[j + 1];
            }
            self.work += n as f64 * mu;
        }
        t.cidx = vec![0; t.lits.len()];
        for c in 0..t.sense.len() {
            let (a, b) = (t.start[c] as usize, t.start[c + 1] as usize);
            for &l in &t.lits[a..b] {
                let v = var(l);
                t.cidx[cs[v + 1] as usize] = if l < 0 { -1 - c as i32 } else { c as i32 };
                cs[v + 1] += 1;
            }
            self.work += ((b - a) as f64 * mu) * 3.0;
        }
        self.work += t.sense.len() as f64 * mu;
        cs.truncate(n + 1);
        t.colstart = cs;
    }
}
