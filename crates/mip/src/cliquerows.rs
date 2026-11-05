//! `46a62d` with `4694fe` (docs/m2_red_clique.md §6): binary rows whose entries all fill the row alone are
//! normalized to ±1 clique rows; rows two hops away are re-linked (binaries by |a| descending first); then each short
//! clique row is removed when another clique row holds it, or extended by the literals that conflict with all of it,
//! and clique rows inside the extended row are removed.
use crate::state::{Node, State, NIL};
use crate::Params;

/// Per-call arrays: row class (0 none, 1 out, 2 `<` clique, 3 `=` clique), extension counters, literal bits and
/// conflict masks, row counters (state+0x3a8), scan work per row, `=` fixings.
struct Cx {
    cls: Vec<i32>,
    f0: Vec<i32>,
    bit: Vec<u32>,
    mask: Vec<u32>,
    cnt: Vec<i32>,
    wacc: Vec<f64>,
    fixflag: Vec<u8>,
    fixlist: Vec<usize>,
}

/// Candidate literals of the row being processed (`Lc`).
struct Cand {
    l: Vec<usize>,
}

enum End {
    Done,
    More(u32),
}

impl State {
    /// Row `i`'s first live node, unlinking dead ones from the row list (counts unchanged).
    fn row_first(&mut self, i: usize) -> u32 {
        let mut h = self.row_head[i];
        while h != NIL && self.nd[h as usize].col < 0 {
            h = self.nd[h as usize].nr;
            self.row_head[i] = h;
        }
        h
    }

    fn row_after(&mut self, p: u32) -> u32 {
        let mut q = self.nd[p as usize].nr;
        while q != NIL && self.nd[q as usize].col < 0 {
            q = self.nd[q as usize].nr;
            self.nd[p as usize].nr = q;
        }
        q
    }

    fn live_nodes(&mut self, i: usize) -> Vec<u32> {
        let mut out = Vec::new();
        let mut p = self.row_first(i);
        while p != NIL {
            out.push(p);
            p = self.row_after(p);
        }
        out
    }

    /// A dead node leaving its column list goes to the pending list (+0x188, +0xd8 − 1).
    fn push_pending(&mut self, at: u32) {
        self.nd[at as usize].nc = self.pend;
        self.pend = at;
        self.npend += 1;
        self.nnz -= 1;
    }

    fn col_first(&mut self, j: usize) -> u32 {
        let mut h = self.col_head[j];
        while h != NIL && self.nd[h as usize].col < 0 {
            let nx = self.nd[h as usize].nc;
            self.push_pending(h);
            h = nx;
            self.col_head[j] = h;
        }
        h
    }

    fn col_after(&mut self, p: u32) -> u32 {
        let mut q = self.nd[p as usize].nc;
        while q != NIL && self.nd[q as usize].col < 0 {
            let nx = self.nd[q as usize].nc;
            self.push_pending(q);
            q = nx;
            self.nd[p as usize].nc = q;
        }
        q
    }

    /// Coefficients to ±1, rhs = 1 − (negatives), min / max activity − rhs = −1 / count − 1 (+0x3fc not counted).
    fn normalize_clique(&mut self, i: usize) {
        let nodes = self.live_nodes(i);
        let mut neg = 0;
        for &p in &nodes {
            let v = &mut self.nd[p as usize].val;
            if *v <= 0.0 {
                *v = -1.0;
                neg += 1;
            } else {
                *v = 1.0;
            }
        }
        self.work += (nodes.len() as f64 + nodes.len() as f64) * self.work_mult;
        self.rhs[i] = 1.0 - neg as f64;
        self.act_min[i] = -1.0;
        self.act_max[i] = self.row_cnt[i] as f64 - 1.0;
    }

    /// A new node {v, r, c} from the free list at the head of column c and after `tail` in row r.
    /// shortcut: an empty free list (`3b85ea` growing the pool) is not ported; the spare nodes never run out here.
    fn append_node(&mut self, r: usize, c: usize, v: f64, tail: u32) -> u32 {
        assert!(self.free != NIL, "46a62d: node pool growth (3b85ea) not ported");
        let at = self.free;
        self.free = self.nd[at as usize].nc;
        self.nfree -= 1;
        self.nd[at as usize] = Node { val: v, row: r as i32, col: c as i32, nr: NIL, nc: self.col_head[c] };
        self.col_head[c] = at;
        self.nnz += 1;
        self.col_cnt[c] += 1;
        self.nd[tail as usize].nr = at;
        at
    }

    /// `46a62d(st, env, timer, mode)`; mode 2 (Presolve or PRECLIQUE > 1) allows longer rows. Returns 0 or a status
    /// (3 with `bad_col` when an `=` clique row needs a column at 0 and at 1).
    /// shortcut: the reduction timer is not passed in, so the scan budget B = (limit − used)·0.01 + 1e5 is taken as
    /// unbounded and `adf2be` never stops; neither binds on the 38 models (B 1.7e5–2.6e5 there, never reached).
    pub fn clique_rows(&mut self, p: &Params, mode: i32) -> i32 {
        let (n, m, mu, ft) = (self.n, self.m, self.work_mult, p.feastol);
        let ft100 = ft * 100.0;
        let mut w_last = self.work;
        if 2e9 < n as f64 + n as f64 {
            return 0;
        }
        let (cap, lim) = if mode == 2 {
            (3200, m as i32)
        } else {
            let x = m.min(n.max(1_000_000));
            (32, ((x as f64).powf(0.4) as i32 * 4).max(32))
        };
        let list = self.rows_list();
        // 1: candidate rows
        let mut kept_all = Vec::new();
        for &i in &list {
            let i = i as usize;
            if !(self.row_cnt[i] >= 2 && !(self.act_min[i] > -ft)) {
                continue;
            }
            let (mut k, mut ok) = (0, true);
            let mut q = self.row_first(i);
            if q == NIL {
                k = 1;
            }
            while q != NIL {
                let node = self.nd[q as usize];
                let j = node.col as usize;
                if self.ub[j] == self.lb[j] {
                    let rc = self.fix_col(j, self.lb[j], true);
                    if rc != 0 {
                        return rc;
                    }
                    if self.row_cnt[i] < 2 {
                        ok = false;
                        break;
                    }
                } else if lim < self.col_cnt[j] || self.vtype[j] != b'B' || 1e-10 < (node.val.abs() + self.act_min[i]).abs() {
                    ok = false;
                    break;
                }
                let nx = self.row_after(q);
                k += 1;
                q = nx;
            }
            self.work += (mu * 8.0) * k as f64;
            if ok {
                kept_all.push(i);
            }
        }
        self.work += (list.len() as f64 * 5.0) * mu;
        if kept_all.is_empty() {
            return 0;
        }
        let mut c = Cx { cls: vec![0; m], f0: vec![-1; m], bit: vec![0; 2 * n], mask: vec![0; 2 * n], cnt: vec![0; m],
                         wacc: vec![0.0; m], fixflag: vec![0; n], fixlist: Vec::new() };
        let mut kept = Vec::new();
        for &i in &kept_all {
            c.cls[i] = if self.sense[i] == b'<' { 2 } else { 3 };
            c.f0[i] = 0;
            self.normalize_clique(i);
            if self.row_cnt[i] <= cap {
                kept.push(i);
            }
        }
        self.work += (kept_all.len() as f64 * 6.0) * mu;
        // 2: classify the rows two hops away (bit[..n] marks columns, cnt marks rows)
        let (mut cols, mut rows): (Vec<usize>, Vec<usize>) = (Vec::new(), Vec::new());
        for &r in &kept {
            let mut q = self.row_first(r);
            let mut t = 0;
            while q != NIL {
                if cols.len() >= n || rows.len() >= m {
                    break;
                }
                let j = self.nd[q as usize].col as usize;
                if c.bit[j] == 0 {
                    c.bit[j] = 1;
                    cols.push(j);
                    let (mut e, mut walked) = (self.col_first(j), 0);
                    while e != NIL {
                        if m <= rows.len() {
                            break;
                        }
                        let i = self.nd[e as usize].row as usize;
                        if c.cnt[i] == 0 {
                            c.cnt[i] = 1;
                            rows.push(i);
                            if self.row_cnt[i] > 0 && c.cls[i] == 0 {
                                let mut ok = self.row_cnt[i] >= 2 && !(1e-9 >= self.act_min[i].abs());
                                let nodes = self.live_nodes(i);
                                for &x in &nodes {
                                    let (k2, v) = (self.nd[x as usize].col as usize, self.nd[x as usize].val);
                                    if self.ub[k2] == self.lb[k2] {
                                        let rc = self.fix_col(k2, self.lb[k2], true);
                                        if rc != 0 {
                                            return rc;
                                        }
                                    } else if !(ok && self.vtype[k2] == b'B' && (v.abs() + self.act_min[i]).abs() < 1e-10) {
                                        ok = false;
                                    }
                                }
                                self.work += (nodes.len() as f64 * 8.0) * mu;
                                if self.row_cnt[i] > 1 && ok {
                                    c.cls[i] = if self.sense[i] == b'<' { 2 } else { 3 };
                                    c.f0[i] = 0;
                                    self.normalize_clique(i);
                                }
                            }
                        }
                        e = self.col_after(e);
                        walked += 1;
                    }
                    self.work += (walked as f64 * 6.0) * mu;
                }
                q = self.row_after(q);
                t += 1;
            }
            self.work += (t as f64 * 4.0) * mu;
        }
        self.work += (kept.len() as f64 + kept.len() as f64) * mu;
        // 3: clear marks, re-link each row (binaries by |a| descending, then the others), classify
        if cols.len() > n / 4 {
            c.bit[..n].iter_mut().for_each(|b| *b = 0);
            self.work += (0.5 * n as f64) * mu;
        } else {
            for &j in &cols {
                c.bit[j] = 0;
            }
            self.work += (cols.len() as f64 + cols.len() as f64) * mu;
        }
        for &i in &rows {
            c.cnt[i] = 0;
            if self.row_cnt[i] < 2 || (self.inf_min[i] != 0 && (self.inf_max[i] != 0 || self.sense[i] == b'<')) {
                c.cls[i] = 1;
                continue;
            }
            self.relink_row(i);
            let p0 = self.row_head[i];
            let q0 = self.nd[p0 as usize].nr;
            let (a, b) = (self.nd[p0 as usize], self.nd[q0 as usize]);
            if self.vtype[b.col as usize] != b'B' {
                c.cls[i] = 1;
            } else {
                let sm = a.val.abs() + b.val.abs();
                if ft100 > self.act_min[i] + sm && (self.sense[i] == b'<' || ft100 > sm - self.act_max[i]) {
                    c.cls[i] = 1;
                }
            }
        }
        self.work += (rows.len() as f64 * 11.0) * mu;
        // 4: process the clique rows
        let (mut it, mut added) = (0, 0);
        for &r in &kept {
            if self.work - w_last > 1e7 {
                self.budget_check(); // `3ba5ac`, `3ba9de`
                if self.prelimit < 0 {
                    return 0;
                }
                w_last = self.work;
            }
            if c.cls[r] & !1 == 2 {
                added += self.clique_row(r, &mut c, ft100);
            }
            it += 1;
        }
        self.work += (it as f64 * 3.0) * mu;
        // 5: columns an `=` clique row forces to 0 (never reached on the 38 models)
        let mut touched = Vec::new();
        for &j in &c.fixlist {
            let (mut q, mut walked) = (self.col_first(j), 0);
            while q != NIL {
                let i = self.nd[q as usize].row as usize;
                if c.cnt[i] == 0 {
                    c.cnt[i] = 1;
                    touched.push(i as i32);
                }
                q = self.col_after(q);
                walked += 1;
            }
            self.work += (walked as f64 * 3.0) * mu;
            match c.fixflag[j] {
                3 => {
                    self.bad_col = j as i32;
                    return 3;
                }
                1 => self.ub[j] = 0.0,
                _ => self.lb[j] = 1.0,
            }
            let rc = self.fix_near(j, ft);
            if rc != 0 {
                return rc;
            }
            c.fixflag[j] = 0;
        }
        self.work += (c.fixlist.len() as f64 * 4.0) * mu;
        for &i in &touched {
            c.cnt[i as usize] = 0;
        }
        self.work += (touched.len() as f64 + touched.len() as f64) * mu;
        if !touched.is_empty() {
            self.activity(Some(&touched));
        }
        self.coefs_changed += added;
        0
    }

    /// `46a62d` phase 3: rebuild row `i`'s list from its live nodes, binaries first by |a| descending (Gurobi's Shell
    /// sort, so ties keep its order), then the others in list order.
    fn relink_row(&mut self, i: usize) {
        let mu = self.work_mult;
        let (mut arr, mut other, mut walked) = (Vec::new(), Vec::new(), 0);
        let mut p = self.row_head[i];
        while p != NIL {
            walked += 1;
            let node = self.nd[p as usize];
            if node.col >= 0 {
                if self.vtype[node.col as usize] == b'B' { arr.push(p) } else { other.push(p) }
            }
            p = node.nr;
        }
        self.work += (walked as f64 * 5.0) * mu;
        let mut head = NIL;
        for &q in other.iter().rev() {
            self.nd[q as usize].nr = head;
            head = q;
        }
        if !arr.is_empty() {
            let ln = arr.len();
            let key = |st: &State, q: u32| st.nd[q as usize].val.abs();
            let mut h = ln / 2;
            while h > 0 {
                for j in h..ln {
                    let k = arr[j];
                    let (mut t, mut sh) = (j, 0);
                    while t >= h && key(self, k) > key(self, arr[t - h]) {
                        arr[t] = arr[t - h];
                        t -= h;
                        sh += 1;
                    }
                    arr[t] = k;
                    self.work += sh as f64 * (4.0 * mu);
                }
                self.work += (ln.max(h) - h) as f64 * (mu * 2.0);
                h = if h == 2 { 1 } else { (h as f64 / 2.2) as usize };
            }
            for w in 1..ln {
                self.nd[arr[w - 1] as usize].nr = arr[w];
            }
            self.work += (ln as f64 * 3.0) * mu;
            self.nd[arr[ln - 1] as usize].nr = head;
            head = arr[0];
            self.work += (ln as f64 * 0.5) * mu;
        }
        self.row_head[i] = head;
    }

    /// `46a62d` main loop for clique row r, in chunks of at most 32 nodes (later chunks 31: bit 1 stands for the
    /// literals carried over). Returns the nodes added to r.
    fn clique_row(&mut self, r: usize, c: &mut Cx, ft100: f64) -> i32 {
        let (n, mu) = (self.n, self.work_mult);
        let (mut mask, mut bit, mut cf, mut esi) = (0u32, 1u32, 0, 0);
        let mut sup: Vec<usize> = Vec::new();
        let mut cand = Cand { l: Vec::new() };
        let mut added = 0;
        let mut best_rows = Vec::new();
        let mut p = self.row_head[r];
        let mut chunk = p;
        let mut tail;
        loop {
            let mut edi = 0;
            let mut rl: Vec<usize> = Vec::new();
            loop {
                let node = self.nd[p as usize];
                let (v, col) = (node.val, node.col as usize);
                esi += 1;
                let lit = if v > 0.0 { col } else { n + col };
                mask |= bit;
                edi += 1;
                c.bit[lit] = bit;
                bit <<= 1;
                let (mut q, mut walked) = (self.col_first(col), 0);
                while q != NIL {
                    let x = self.nd[q as usize];
                    let i2 = x.row as usize;
                    if i2 != r && c.cls[i2] != 1 && c.cls[i2] & 4 == 0 && ((0.0 < x.val) == (0.0 < v)) {
                        if c.cnt[i2] == 0 {
                            rl.push(i2);
                        }
                        c.cnt[i2] += 1;
                    }
                    q = self.col_after(q);
                    walked += 1;
                }
                self.work += (walked as f64 * 5.0) * mu;
                tail = p;
                p = self.nd[p as usize].nr;
                if p == NIL || esi >= 32 {
                    break;
                }
            }
            self.work += ((esi - cf) as f64 * 4.0) * mu;
            if cf == 0 {
                for &i2 in &rl {
                    if c.cnt[i2] == edi && c.cls[i2] & !1 == 2 {
                        sup.push(i2);
                    }
                    c.cnt[i2] = 0;
                }
            } else {
                let k = sup.len();
                if k > 0 {
                    sup.retain(|&i2| c.cnt[i2] == edi);
                }
                self.work += (k as f64 + k as f64) * mu;
                for &i2 in &rl {
                    c.cnt[i2] = 0;
                }
            }
            self.work += (rl.len() as f64 + rl.len() as f64) * mu;
            if p == NIL && !sup.is_empty() {
                // another clique row holds every literal of r
                let mut e = 0;
                let mut x = self.row_head[r];
                while x != NIL {
                    let col = self.nd[x as usize].col as usize;
                    c.bit[col] = 0;
                    c.bit[n + col] = 0;
                    e += 1;
                    x = self.nd[x as usize].nr;
                }
                self.work += (e as f64 * 4.0) * mu;
                if self.sense[r] == b'<' {
                    let mut e = 0;
                    let mut x = self.row_head[r];
                    while x != NIL {
                        let col = self.nd[x as usize].col;
                        self.tracker.mark_col(col);
                        self.col_cnt[col as usize] -= 1;
                        self.nd[x as usize].col = -1;
                        e += 1;
                        x = self.nd[x as usize].nr;
                    }
                    self.work += (e as f64 * 3.0) * mu;
                    self.row_cnt[r] = -2;
                    self.rows_removed += 1;
                }
                c.cls[r] = 1;
                c.f0[r] = -1;
                if cf != 0 {
                    for &l in &cand.l {
                        c.mask[l] = 0;
                    }
                    self.work += (cand.l.len() as f64 + cand.l.len() as f64) * mu;
                }
                break;
            }
            // literals conflicting with this chunk
            let first = cf == 0;
            if first {
                cand.l.clear();
            }
            for &i2 in &rl {
                let d = c.wacc[i2];
                c.wacc[i2] = d - self.work;
                if c.cls[i2] & !1 == 2 {
                    let nodes = self.live_nodes(i2);
                    let mut bs = 0u32;
                    for &x in &nodes {
                        let (xv, xc) = (self.nd[x as usize].val, self.nd[x as usize].col as usize);
                        bs |= if xv > 0.0 { c.bit[xc] } else { c.bit[n + xc] };
                    }
                    self.work += (nodes.len() as f64 * 4.0) * mu;
                    let mut e = 0;
                    let mut x = self.row_head[i2];
                    while x != NIL {
                        let (xv, xc) = (self.nd[x as usize].val, self.nd[x as usize].col as usize);
                        let l = if xv > 0.0 { xc } else { n + xc };
                        let u = c.mask[l];
                        if first {
                            if u == 0 {
                                cand.l.push(l);
                            }
                            c.mask[l] = u | bs;
                        } else if u != 0 {
                            c.mask[l] = u | bs;
                        }
                        e += 1;
                        x = self.nd[x as usize].nr;
                    }
                    self.work += (e as f64 * 5.0) * mu;
                } else {
                    if self.inf_min[i2] == 0 {
                        self.implied_conflicts(i2, 1.0, self.act_min[i2] - ft100, c, &mut cand, first);
                    }
                    if self.sense[i2] == b'=' && self.inf_max[i2] == 0 {
                        self.implied_conflicts(i2, -1.0, -self.act_max[i2] - ft100, c, &mut cand, first);
                    }
                }
                c.wacc[i2] = self.work + c.wacc[i2];
            }
            self.work += (rl.len() as f64 * 3.0) * mu;
            let k = cand.l.len();
            if k > 0 {
                let mut keep = Vec::with_capacity(k);
                for &l in &cand.l {
                    let col = if l < n { l } else { l - n };
                    if c.bit[col] != 0 || c.bit[n + col] != 0 || c.mask[l] != mask {
                        c.mask[l] = 0;
                    } else {
                        keep.push(l);
                    }
                }
                cand.l = keep;
            }
            self.work += (k as f64 * 5.0) * mu;
            if p == NIL {
                match self.extend_row(r, c, &mut cand, mask, &mut added, &mut best_rows, &mut tail) {
                    End::Done => {
                        let mut x = chunk;
                        while x != NIL {
                            let col = self.nd[x as usize].col as usize;
                            c.bit[col] = 0;
                            c.bit[n + col] = 0;
                            x = self.nd[x as usize].nr;
                        }
                        self.work += ((esi - cf) as f64 * 4.0) * mu;
                        break;
                    }
                    End::More(q) => p = q,
                }
            }
            // next chunk: the literals so far carry bit 1
            for &l in &cand.l {
                c.mask[l] = 1;
            }
            self.work += (cand.l.len() as f64 + cand.l.len() as f64) * mu;
            mask = 1;
            bit = 2;
            cf = 1;
            let mut x = chunk;
            while x != p {
                let col = self.nd[x as usize].col as usize;
                c.bit[col] = 0;
                c.bit[n + col] = 0;
                x = self.nd[x as usize].nr;
            }
            self.work += ((esi - 1) as f64 * 4.0) * mu;
            if cand.l.is_empty() {
                break;
            }
            esi = 1;
            chunk = p;
        }
        for &b in &best_rows {
            c.cls[b] &= !4; // asm 0x416ab9 set bit 1, not 4: a no-op kept as is
        }
        added
    }

    /// `46a62d` end of row r: an `=` row fixes its candidates at 0; a `<` row takes them from the row holding most
    /// of the first ten (or all of them), then drops the clique rows the extended row contains.
    #[allow(clippy::too_many_arguments)]
    fn extend_row(&mut self, r: usize, c: &mut Cx, cand: &mut Cand, mask: u32, added: &mut i32, best_rows: &mut Vec<usize>,
                  tail: &mut u32) -> End {
        let (n, mu) = (self.n, self.work_mult);
        'ext: {
            let lc = cand.l.len();
            if self.sense[r] == b'=' {
                for &l in &cand.l {
                    c.mask[l] = 0;
                    let col = if l < n { l } else { l - n };
                    if c.fixflag[col] == 0 {
                        c.fixlist.push(col);
                    }
                    c.fixflag[col] |= if l >= n { 2 } else { 1 };
                }
                self.work += (lc as f64 * 4.0) * mu;
                return End::Done;
            }
            if lc == 0 {
                break 'ext;
            }
            let saved = *added;
            let t = lc.min(10);
            let mut f8 = Vec::new();
            for &l in &cand.l[..t] {
                let col = if l < n { l } else { l - n };
                let (mut q, mut walked) = (self.col_first(col), 0);
                while q != NIL {
                    let x = self.nd[q as usize];
                    let i2 = x.row as usize;
                    if c.f0[i2] >= 0 {
                        let v = x.val;
                        if (v > 0.0 && l < n) || (!(v > 0.0) && (!(0.0 > v) || l >= n)) {
                            if c.f0[i2] == 0 {
                                f8.push(i2);
                            }
                            c.f0[i2] += 1;
                        }
                    }
                    q = self.col_after(q);
                    walked += 1;
                }
                self.work += (walked as f64 * 5.0) * mu;
            }
            self.work += (t as f64 + t as f64) * mu;
            let (mut best, mut bc) = (usize::MAX, 0);
            for &i2 in &f8 {
                let x = c.f0[i2];
                c.f0[i2] = 0;
                if x > bc {
                    bc = x;
                    best = i2;
                }
            }
            self.work += (f8.len() as f64 + f8.len() as f64) * mu;
            let mut newfirst = NIL;
            if best != usize::MAX {
                let (mut rd, mut md, mut e) = (0.0f64, 0.0f64, 0);
                for x in self.live_nodes(best) {
                    let (v, col) = (self.nd[x as usize].val, self.nd[x as usize].col as usize);
                    e += 1;
                    if v > 0.0 && c.mask[col] == mask {
                        *tail = self.append_node(r, col, 1.0, *tail);
                    } else if 0.0 > v && c.mask[n + col] == mask {
                        *tail = self.append_node(r, col, -1.0, *tail);
                        rd -= 1.0;
                    } else {
                        continue;
                    }
                    self.row_cnt[r] += 1;
                    *added += 1;
                    self.tracker.mark_col(col as i32);
                    if newfirst == NIL {
                        newfirst = *tail;
                    }
                    md += 1.0;
                }
                self.work += (e as f64 * 3.0) * mu;
                self.act_max[r] = md + self.act_max[r];
                self.rhs[r] = rd + self.rhs[r];
                self.tracker.mark_row(r as i32);
                c.cls[best] |= 1;
                best_rows.push(best);
            } else {
                for &l in &cand.l {
                    let (col, v) = if l < n { (l, 1.0) } else { (l - n, -1.0) };
                    *tail = self.append_node(r, col, v, *tail);
                    self.row_cnt[r] += 1;
                    *added += 1;
                    self.tracker.mark_row(r as i32);
                    self.tracker.mark_col(col as i32);
                    self.act_max[r] += 1.0;
                    if 0.0 > v {
                        self.rhs[r] -= 1.0;
                    }
                }
                self.work += (lc as f64 + lc as f64) * mu;
            }
            if ((*added - saved) as usize) < lc {
                if newfirst != NIL {
                    return End::More(newfirst);
                }
                break 'ext; // the masks of the candidates stay set (as in Gurobi; only after a failed allocation)
            }
            for &l in &cand.l {
                c.mask[l] = 0;
            }
            self.work += (lc as f64 + lc as f64) * mu;
        }
        if *added > 0 {
            let (mut rl, mut er) = (Vec::new(), 0);
            let mut x = self.row_head[r];
            while x != NIL {
                let (v, col) = (self.nd[x as usize].val, self.nd[x as usize].col as usize);
                let (mut q, mut walked) = (self.col_first(col), 0);
                while q != NIL {
                    let y = self.nd[q as usize];
                    if y.row as usize != r && (0.0 < y.val) == (0.0 < v) {
                        if c.cnt[y.row as usize] == 0 {
                            rl.push(y.row as usize);
                        }
                        c.cnt[y.row as usize] += 1;
                    }
                    q = self.col_after(q);
                    walked += 1;
                }
                self.work += (walked as f64 * 4.0) * mu;
                er += 1;
                x = self.nd[x as usize].nr;
            }
            self.work += (er as f64 * 3.0) * mu;
            for &i2 in &rl {
                if c.cnt[i2] == self.row_cnt[i2] {
                    if c.cls[i2] == 2 {
                        let nodes = self.live_nodes(i2);
                        for &y in &nodes {
                            let col = self.nd[y as usize].col;
                            self.tracker.mark_col(col);
                            self.col_cnt[col as usize] -= 1;
                            self.nd[y as usize].col = -1;
                        }
                        self.work += (nodes.len() as f64 + nodes.len() as f64) * mu;
                        self.row_cnt[i2] = -2;
                        self.rows_removed += 1;
                    }
                    c.cls[i2] = 1;
                    c.f0[i2] = -1;
                }
                c.cnt[i2] = 0;
            }
            self.work += (rl.len() as f64 * 4.0) * mu;
        }
        End::Done
    }

    /// `4694fe`: in a non-clique row i2 (binaries first, |a| descending), literal x conflicts with the r literals
    /// ahead of it when, side `s` on, x plus each of them overfills the row (min activity − rhs − 100·ft = `d`).
    fn implied_conflicts(&mut self, i2: usize, s: f64, d: f64, c: &mut Cx, cand: &mut Cand, first: bool) {
        let (n, mu) = (self.n, self.work_mult);
        let (mut acc, mut masks, mut thr) = (0u32, Vec::new(), Vec::new());
        let mut p = self.row_first(i2);
        let mut cnt1 = 0;
        while p != NIL && self.vtype[self.nd[p as usize].col as usize] == b'B' {
            let (v, col) = (self.nd[p as usize].val * s, self.nd[p as usize].col as usize);
            let u = if v > 0.0 { c.bit[col] } else { c.bit[n + col] };
            if u != 0 {
                acc |= u;
                masks.push(acc);
                thr.push(if v > 0.0 { v + d } else { d - v });
            }
            p = self.row_after(p);
            cnt1 += 1;
        }
        let k = masks.len();
        self.work += (k as f64 + k as f64 + cnt1 as f64 * 5.0) * mu;
        let (mut cnt2, mut pp) = (0, k as isize - 1);
        let mut x = self.row_head[i2];
        while x != NIL {
            let col = self.nd[x as usize].col as usize;
            if self.vtype[col] != b'B' {
                break;
            }
            let v = self.nd[x as usize].val * s;
            let av = v.abs();
            while pp >= 0 && !(0.0 <= thr[pp as usize] + av) {
                pp -= 1;
            }
            if pp < 0 {
                break;
            }
            let l = if v > 0.0 { col } else { n + col };
            let u = c.mask[l];
            if first {
                if u == 0 {
                    cand.l.push(l);
                }
                c.mask[l] = u | masks[pp as usize];
            } else if u != 0 {
                c.mask[l] = u | masks[pp as usize];
            }
            cnt2 += 1;
            x = self.nd[x as usize].nr;
        }
        self.work += (k as f64 + k as f64 + cnt2 as f64 * 6.0) * mu;
    }
}
