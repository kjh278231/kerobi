//! Symmetry detection in MIP presolve: `5577ba` (linked-list state, sixth argument 1) and `551cc0` with variable
//! types (docs/m2_red_roundend.md §7). Copied from the LP port (crates/lp/src/presolve/symmetry.rs, LP modules are
//! private) and extended with the MIP branches: the column CSR is built from the column lists, columns start split by
//! type, integer columns and rows are split by their integer entry counts, and when the hash refinement stalls with
//! integer columns `5507cb` may individualize them. Cell ids, queue order, sort tie order, the random stream and the
//! work follow the original.
use crate::state::{State, NIL};
use crate::subst::sort_asc;
use crate::Params;
use std::cell::Cell;

/// `a7b4d6`: the shared random number step (LCG state, tempered high word), 31 bits.
fn rand31(s: &mut u64) -> i64 {
    *s = s.wrapping_mul(0x5851f42d4c957f2d).wrapping_add(0x14057b7ef767814f);
    let mut x = (*s >> 32) as u32;
    x ^= x >> 11;
    x ^= (x & 0x13a58ad) << 7;
    x ^= (x & 0x1df8c) << 15;
    ((x ^ (x >> 18)) & 0x7fffffff) as i64
}

/// `ac3c8b` list of cell ids: queued ids in `list` (taken from the end), per id its index in `list`, -1 when not
/// queued, -2 when fixed (a singleton cell, never queued again).
struct Stack {
    list: Vec<i32>,
    pos: Vec<i32>,
}

impl Stack {
    fn new(n: usize, w: &mut f64) -> Stack {
        *w += n as f64;
        Stack { list: Vec::with_capacity(n), pos: vec![-1; n] }
    }

    /// `ac3f7f`
    fn push(&mut self, c: i32) {
        if self.pos[c as usize] == -1 {
            self.pos[c as usize] = self.list.len() as i32;
            self.list.push(c);
        }
    }

    /// `ac4062`
    fn pop(&mut self) -> Option<i32> {
        let c = self.list.pop()?;
        self.pos[c as usize] = -1;
        Some(c)
    }

    /// `ac40d4`
    fn fix(&mut self, c: i32) {
        if self.pos[c as usize] == -1 {
            self.pos[c as usize] = -2;
        }
    }

    /// `ac41fc`
    fn fixed(&self, c: i32) -> bool {
        self.pos[c as usize] == -2
    }

    /// `ac4326`
    fn queued(&self, c: i32) -> bool {
        self.pos[c as usize] >= 0
    }

    /// `ac4040`
    fn len(&self) -> usize {
        self.list.len()
    }
}

/// What `54e5eb` splits a cell by: the side's hashes, other per-item keys (`5507cb`'s marks), or nothing (every item
/// its own cell, no sort).
#[derive(Clone, Copy)]
enum By<'a> {
    Hash,
    Keys(&'a [i64]),
    Each,
}

/// One side (rows or columns): its partition and its sparse lines for the hash refinement.
struct Side {
    /// cell id per item, first position per cell, items ordered by cell
    cell: Vec<i32>,
    start: Vec<i32>,
    perm: Vec<i32>,
    /// cells so far, singleton cells, removed items (each in a singleton cell at the end of `perm`)
    ncell: i32,
    single: i32,
    removed: i32,
    stack: Stack,
    /// the lines (columns: the column CSR; rows: its transpose `790e3`) and their values
    beg: Vec<i64>,
    ind: Vec<i32>,
    val: Vec<f64>,
    /// integer keys of `val` (`54ad3b`), compacted along with `own`
    key: Vec<i64>,
    /// `54e0c4`'s copy without the other side's fixed cells; `cur_own` = the refinement reads it
    own: Option<(Vec<i64>, Vec<i32>)>,
    cur_own: bool,
    hash: Vec<i64>,
    /// entries of this side's fixed cells since the other side was last compacted (local_178 / local_170)
    dnnz: i64,
}

impl Side {
    fn new(n: usize, w: &mut f64) -> Side {
        Side {
            cell: vec![0; n],
            start: vec![0; n + 1],
            perm: vec![0; n],
            ncell: 0,
            single: 0,
            removed: 0,
            stack: Stack::new(n, w),
            beg: Vec::new(),
            ind: Vec::new(),
            val: Vec::new(),
            key: Vec::new(),
            own: None,
            cur_own: false,
            hash: vec![0; n],
            dnnz: 0,
        }
    }

    /// current lines (`*param_6`, `*param_7` of `54e0c4`)
    fn cur(&self) -> (&[i64], &[i32]) {
        match (&self.own, self.cur_own) {
            (Some((b, i)), true) => (b, i),
            _ => (&self.beg, &self.ind),
        }
    }

    /// `549df5` (also inlined for the bounds, and with i32 keys and `ab6a5f` for the integer counts): split every
    /// cell among positions [0, len) by `key`; values `same` as a group's first (smallest) stay together, the first
    /// group keeps the cell's id.
    fn refine<K: Copy + PartialOrd>(&mut self, len: usize, key: &[K], same: impl Fn(K, K) -> bool, w: &mut f64) {
        let mut s = 0;
        while s < len {
            let first = self.perm[s] as usize;
            let c = self.cell[first];
            let mut e = s + 1;
            let mut differ = false;
            while e < len {
                let j = self.perm[e] as usize;
                if self.cell[j] != c {
                    break;
                }
                differ = differ || !same(key[j], key[first]);
                e += 1;
            }
            *w += (e - s) as f64 * 3.0;
            if differ {
                let seg = &mut self.perm[s..e];
                let mut k: Vec<K> = seg.iter().map(|&j| key[j as usize]).collect();
                sort_asc(&mut k, seg); // `ab9789` / `ab6a5f`
                let mut g = s;
                while g < e {
                    let mut t = g + 1;
                    while t < e && same(key[self.perm[t] as usize], key[self.perm[g] as usize]) {
                        t += 1;
                    }
                    *w += (t - 1 - g) as f64 * 2.0;
                    if t - g == 1 {
                        self.single += 1;
                        self.stack.fix(if g == s { c } else { self.ncell });
                    }
                    if g != s {
                        let id = self.ncell;
                        self.start[id as usize] = g as i32;
                        for q in g..t {
                            self.cell[self.perm[q] as usize] = id;
                        }
                        *w += (t - g) as f64 * 2.0;
                        self.ncell += 1;
                    }
                    g = t;
                }
            }
            s = e;
        }
    }

    /// `54e0c4`: once the other side's fixed cells hold more than a fifth of this side's current entries, drop
    /// those entries (and their keys) from this side's own copy, made from the original lines the first time.
    /// Later calls keep compacting the copy even when the refinement reads the original again (pass 3).
    fn drop_fixed(&mut self, other_cell: &[i32], other_stack: &Stack, other_dnnz: &mut i64, w: &mut f64) {
        let nn = self.beg.len() - 1;
        if (*other_dnnz as f64) <= self.cur().0[nn] as f64 * 0.2 {
            return;
        }
        if self.own.is_none() {
            self.own = Some((self.beg.clone(), self.ind.clone()));
            self.cur_own = true;
        }
        let (ob, oi) = self.own.as_mut().unwrap();
        let mut p = 0;
        for j in 0..nn {
            let (b, e) = (ob[j], ob[j + 1]);
            ob[j] = p;
            for k in b as usize..e as usize {
                let i = oi[k];
                if !other_stack.fixed(other_cell[i as usize]) {
                    oi[p as usize] = i;
                    self.key[p as usize] = self.key[k];
                    p += 1;
                }
            }
            *w += (e - b) as f64 * 3.0;
        }
        *w += nn as f64;
        ob[nn] = p;
        *other_dnnz = 0;
    }

    /// `54ad3b` for both sides at once: keys = round(x * scale), scale 1 when every |x| is integral, else 1 /
    /// `a7bea8`(distinct |x|) when that finds a divisor among fewer than 100000 values, else 2^48 / max|x|; then
    /// shifted right by the trailing zero bits all column keys share.
    fn keys(cols: &mut Side, rows: &mut Side, w: &mut f64) {
        let nnz = cols.val.len();
        if nnz == 0 {
            return;
        }
        let mut amax = 1e-9;
        let mut integral = true;
        let mut a: Vec<f64> = Vec::with_capacity(nnz);
        for &x in &cols.val {
            let v = x.abs();
            if amax <= v {
                amax = v;
            }
            integral = integral && v - (v + 1e-9).floor() <= 1e-9;
            a.push(v);
        }
        *w += nnz as f64 + nnz as f64;
        let mut scale = 281474976710656.0 / amax;
        if integral {
            scale = 1.0;
        } else {
            // `aaa891` sorts the values descending; a value is kept when it is 1e-9 below the last one kept
            a.sort_by(|x, y| y.total_cmp(x));
            let mut d = 1;
            for i in 1..nnz {
                if 1e-9 < a[d - 1] - a[i] {
                    a[d] = a[i];
                    d += 1;
                }
            }
            *w += nnz as f64 + nnz as f64;
            if d < 100000 {
                let g = real_gcd(&a[..d]);
                if g != 0.0 {
                    scale = 1.0 / g;
                }
            }
        }
        // cvttsd2si: out of range (or NaN) gives i64::MIN
        let to_key = |x: f64| {
            let r = (scale * x).round();
            if (-9.223372036854776e18..9.223372036854776e18).contains(&r) { r as i64 } else { i64::MIN }
        };
        let mut or = 0u64;
        for k in 0..nnz {
            cols.key[k] = to_key(cols.val[k]);
            or |= cols.key[k] as u64;
            rows.key[k] = to_key(rows.val[k]);
        }
        *w += nnz as f64 * 4.0;
        let tz = if or == 0 { 0 } else { or.trailing_zeros() };
        if tz > 0 {
            for k in 0..nnz {
                cols.key[k] >>= tz;
                rows.key[k] >>= tz;
            }
            *w += nnz as f64 + nnz as f64;
        }
    }

    /// hash of every item: its cell id plus, over its current entries, key * rand[cell of the partner]
    fn hash_all(&mut self, other_cell: &[i32], rand: &[i64], w: &mut f64) {
        let n = self.cell.len();
        for i in 0..n {
            let (b, ind) = self.cur();
            let (s, e) = (b[i] as usize, b[i + 1] as usize);
            let mut h = self.cell[i] as i64;
            for k in s..e {
                h = h.wrapping_add(rand[other_cell[ind[k] as usize] as usize].wrapping_mul(self.key[k]));
            }
            self.hash[i] = h;
            *w += (e - s) as f64 * 4.0;
        }
        *w += n as f64;
    }

    /// Loop B's queue pass for one side: take cells off the queue (last first) and split those whose items hash
    /// differently.
    fn settle(&mut self, o: &mut Side, rand: &[i64], runs: &mut Vec<usize>, ext: &mut [Option<&mut Stack>; 2], w: &mut f64) {
        let nn = self.cell.len();
        while let Some(c) = self.stack.pop() {
            let s = self.start[c as usize] as usize;
            let h0 = self.hash[self.perm[s] as usize];
            let mut e = s + 1;
            let mut differ = false;
            while e < nn {
                let j = self.perm[e] as usize;
                if self.cell[j] != c {
                    break;
                }
                differ = differ || h0 != self.hash[j];
                e += 1;
            }
            *w += (e - s) as f64 * 3.0;
            if differ {
                self.split(o, rand, c, e, By::Hash, runs, ext, w);
            }
        }
    }

    /// `54e5eb`: split cell `c` (positions [start, e)) by hash. Groups of equal hash, sorted unsigned (`ab848c`),
    /// get new ids in order, except the first group with the most entries, which keeps `c`. Each entry of a moved
    /// item adds key * (rand[new] - rand[c]) to its partner's hash and queues the partner's cell. The extra stacks
    /// (integer mode) fix a new cell when `c` was fixed there or the cell is a singleton, else queue it when `c` was.
    #[allow(clippy::too_many_arguments)]
    fn split(&mut self, o: &mut Side, rand: &[i64], c: i32, e: usize, by: By, runs: &mut Vec<usize>,
             ext: &mut [Option<&mut Stack>; 2], w: &mut f64) {
        let s = self.start[c as usize] as usize;
        let rc = rand[c as usize];
        let flags = ext.each_ref().map(|x| x.as_ref().map(|t| (t.queued(c), t.fixed(c))));
        let len = |b: &[i64], j: i32| b[j as usize + 1] - b[j as usize];
        runs.clear();
        runs.push(s);
        let mut best = 0;
        let mut k: Vec<u64> = match by {
            By::Hash => self.perm[s..e].iter().map(|&j| self.hash[j as usize] as u64).collect(),
            By::Keys(h) => self.perm[s..e].iter().map(|&j| h[j as usize] as u64).collect(),
            By::Each => Vec::new(),
        };
        if let By::Each = by {
            // no keys: every item its own group in order, the first keeps `c`
            runs.extend(s + 1..=e);
        } else {
            sort_asc(&mut k, &mut self.perm[s..e]);
            let mut most = 0;
            let mut g = s;
            while g < e {
                let mut tot = len(self.cur().0, self.perm[g]);
                let mut t = g + 1;
                while t < e && k[t - s] == k[g - s] {
                    tot += len(self.cur().0, self.perm[t]);
                    t += 1;
                }
                *w += (t - g) as f64 * 4.0;
                runs.push(t);
                if most < tot {
                    most = tot;
                    best = runs.len() - 2;
                }
                g = t;
            }
        }
        for r in 0..runs.len() - 1 {
            let (a, b) = (runs[r], runs[r + 1]);
            if b - a == 1 {
                let id = if r == best { c } else { self.ncell };
                let l = len(self.cur().0, self.perm[a]);
                self.single += 1;
                self.dnnz += l;
                self.stack.fix(id);
            }
            if r == best {
                self.start[c as usize] = a as i32;
                continue;
            }
            let u = self.ncell;
            for (x, f) in ext.iter_mut().zip(flags) {
                if let (Some(t), Some((queued, fixed))) = (x, f) {
                    if fixed || b - a == 1 {
                        t.fix(u);
                    } else if queued {
                        t.push(u);
                    }
                }
            }
            let d = rand[u as usize].wrapping_sub(rc);
            self.start[u as usize] = a as i32;
            for q in a..b {
                let j = self.perm[q] as usize;
                let (cb, ci) = match (&self.own, self.cur_own) {
                    (Some((cb, ci)), true) => (cb, ci),
                    _ => (&self.beg, &self.ind),
                };
                for k in cb[j] as usize..cb[j + 1] as usize {
                    let i = ci[k] as usize;
                    o.hash[i] = o.hash[i].wrapping_add(self.key[k].wrapping_mul(d));
                    o.stack.push(o.cell[i]);
                }
                *w += (cb[j + 1] - cb[j]) as f64 * 4.0;
                self.cell[j] = u;
            }
            *w += (b - a) as f64 * 3.0;
            self.ncell = u + 1;
        }
    }

    /// whether the refinement should go on: #cells * ratio < #items, counting live items only
    fn open(&self, live: usize, flags: i32, ratio: f64) -> bool {
        (((self.ncell - flags - self.removed) as f64) * ratio) < live as f64
    }
}

/// `a7bea8`: a common divisor of `p` (0 when none is found): the smallest value when all are its multiples, the
/// gcd of the values times 600 or of integral values, else a Euclid on the doubles and a continued fraction of
/// its result with denominators up to 1000.
fn real_gcd(p: &[f64]) -> f64 {
    let n = p.len();
    if n == 0 {
        return 0.0;
    }
    if n == 1 {
        return p[0].abs();
    }
    let (mut lo, mut hi) = (1e100f64, 0.0f64);
    for &x in p {
        let v = x.abs();
        if v <= lo {
            lo = v;
        }
        if hi <= v {
            hi = v;
        }
    }
    if 2e9 < hi || lo < 1e-5 {
        return 0.0;
    }
    if hi - lo <= 1e-10 {
        return lo;
    }
    // every value within tol of an integer multiple of 1 / r
    let multiples = |r: f64, tol: f64| p.iter().all(|&x| !(tol < (x.abs() * r - (x.abs() * r + 0.5).floor()).abs()));
    let r = 1.0 / lo;
    if multiples(r, if r <= 1.0 { 1.0 } else { r } * 1e-10) {
        return lo;
    }
    let int_gcd = |a: f64, b: f64| -> f64 {
        let (a, b) = (a as i32, b as i32);
        if a == 0 || b == 0 {
            return 0.0;
        }
        if 2_000_000_000 <= a {
            return b as f64;
        }
        if 2_000_000_000 <= b {
            return a as f64;
        }
        let (mut big, mut small) = if a < b { (b, a) } else { (a, b) };
        let mut g = small;
        while 0 < small {
            g = small;
            let r = big % small;
            big = small;
            small = r;
        }
        g as f64
    };
    if hi * 600.0 <= 2e9 {
        if multiples(600.0, 6.000000000000001e-08) {
            let mut g = (p[0].abs() * 600.0 + 0.5).floor();
            for &x in &p[1..] {
                if g == 0.0 {
                    break;
                }
                g = int_gcd(g, (x.abs() * 600.0 + 0.5).floor());
            }
            return g / 600.0;
        }
    } else if 1.9999999999 <= lo && multiples(1.0, 1e-10) {
        let mut g = (p[0].abs() + 0.5).floor();
        for &x in &p[1..] {
            if g == 0.0 {
                return g;
            }
            g = int_gcd(g, (x.abs() + 0.5).floor());
        }
        return g;
    }
    let mut g = p[0];
    for &x in &p[1..] {
        if g <= 0.0 {
            break;
        }
        let (a, b) = (g.abs(), x.abs());
        g = if a < 1e-10 || b < 1e-10 {
            0.0
        } else if 1e100 <= a {
            b
        } else if 1e100 <= b {
            a
        } else {
            let (mut small, mut big) = if b <= a { (b, a) } else { (a, b) };
            loop {
                if !(1e-10 < small) {
                    break if 1e-9 <= big { big } else { 0.0 };
                }
                if big < small {
                    break 0.0;
                }
                let q = ((big + 1e-10) / small).floor();
                let rem = big - q * small;
                (big, small) = (small, if rem <= 0.0 { 0.0 } else { rem });
            }
        };
    }
    if g <= 1e-10 {
        return 0.0;
    }
    let inv = 1.0 / g;
    if inv - (inv + 1.0000000000000001e-11).floor() <= 1.0000000000000001e-11
        && multiples(inv, if inv <= 1.0 { 1.0 } else { inv } * 1e-10)
    {
        return g;
    }
    // continued fraction of g: p/q, stop when g is p/q or (p +- 1)/q, or q > 1000
    let mut x = g;
    let mut a = (g + 5e-14).floor();
    let mut e1 = g - a;
    let mut e2 = g - if 0.0 <= e1 { a + 1.0 } else { a - 1.0 };
    let (mut p1, mut q1, mut p0, mut q0) = (a, 1.0, 1.0, 0.0);
    while (e1 < -1e-13 || 1e-13 < e1) && (e2 < -1e-13 || 1e-13 < e2) {
        x = 1.0 / (x - a);
        a = (x + 5e-14).floor();
        let q = q1 * a + q0;
        let pn = p1 * a + p0;
        if 1000.0 < q {
            return 0.0;
        }
        e1 = g - pn / q;
        (p0, q0, p1, q1) = (p1, q1, pn, q);
        e2 = g - if 0.0 <= e1 { (pn + 1.0) / q } else { (pn - 1.0) / q };
    }
    let num = if e1 < -1e-13 {
        (p1 - 1.0) as i32
    } else if e1 <= 1e-13 {
        p1 as i32
    } else {
        (p1 + 1.0) as i32
    };
    if num < 1 {
        return 0.0;
    }
    let r = (q1 as i32) as f64 / num as f64;
    if !multiples(r, if r <= 1.0 { 1.0 } else { r } * 1e-10) {
        return 0.0;
    }
    num as f64 / (q1 as i32) as f64
}

/// `54c63b`: whether some cell of positions [0, n) (perm/own_cell) has items that differ in the `d` values (by
/// more than 1e-9) or `ch`, or whose coefficient sums per cell of the other side differ from its first item's.
#[allow(clippy::too_many_arguments)]
fn unequal(
    n: usize, beg: &[i64], ind: &[i32], val: &[f64], d: [Option<&[f64]>; 3], ch: Option<&[u8]>, other_cell: &[i32],
    own_cell: &[i32], perm: &[i32], buf: &mut Buf, w: &mut f64,
) -> bool {
    let Buf { sa, sb, list, pos } = buf;
    // slot of the other side's cell `oc` in `list`, opened with a zero sum in `sum`
    fn slot(oc: i32, cnt: &mut usize, sum: &mut [f64], list: &mut [i32], pos: &mut [i32]) -> usize {
        if pos[oc as usize] == -1 {
            pos[oc as usize] = *cnt as i32;
            list[*cnt] = oc;
            sum[*cnt] = 0.0;
            *cnt += 1;
        }
        pos[oc as usize] as usize
    }
    let mut fail = false;
    let mut k = 0;
    while k < n && !fail {
        let s = k;
        let first = perm[k] as usize;
        k += 1;
        while k < n && own_cell[first] == own_cell[perm[k] as usize] {
            k += 1;
        }
        *w += (k - s) as f64 * 2.0;
        if k - s < 2 {
            continue;
        }
        let refd = d.map(|v| v.map_or(0.0, |v| v[first]));
        let refc = ch.map_or(b' ', |c| c[first]);
        let mut nref = 0;
        let (b0, e0) = (beg[first] as usize, beg[first + 1] as usize);
        for q in b0..e0 {
            let t = slot(other_cell[ind[q] as usize], &mut nref, sa, list, pos);
            sa[t] += val[q];
        }
        *w += (e0 - b0) as f64 * 7.0;
        let mut cnt_prev = nref;
        let mut t = s + 1;
        let mut d19;
        loop {
            let j = perm[t] as usize;
            if (0..3).any(|x| d[x].is_some_and(|v| 1e-9 < (v[j] - refd[x]).abs())) || ch.is_some_and(|c| c[j] != refc) {
                fail = true;
                d19 = *w;
                break;
            }
            sb[..nref].fill(0.0);
            let mut cnt = nref;
            let (b, e) = (beg[j] as usize, beg[j + 1] as usize);
            for q in b..e {
                let x = slot(other_cell[ind[q] as usize], &mut cnt, sb, list, pos);
                sb[x] += val[q];
            }
            cnt_prev = cnt;
            d19 = (e - b) as f64 * 5.0 + *w;
            let mut s13 = 0;
            if 0 < nref {
                *w = d19;
                if let Some(x) = (0..nref).find(|&x| 1e-9 < (sa[x] - sb[x]).abs()) {
                    fail = true;
                    d19 = x as f64 + x as f64 + *w;
                    break;
                }
                s13 = nref;
                d19 = *w;
            }
            d19 = d19 + s13 as f64 + s13 as f64;
            let mut s14 = nref;
            if nref < cnt {
                *w = d19;
                let mut bad = None;
                for x in nref..cnt {
                    pos[list[x] as usize] = -1;
                    if 1e-9 < sb[x].abs() {
                        bad = Some(x);
                        break;
                    }
                }
                if let Some(x) = bad {
                    fail = true;
                    d19 = (x - nref) as f64 + *w;
                    break;
                }
                d19 = *w;
                s14 = cnt;
            }
            t += 1;
            d19 += (s14 - nref) as f64;
            if k <= t {
                break;
            }
            *w = d19;
        }
        d19 += (t - s) as f64 * 5.0;
        *w = d19;
        if 0 < cnt_prev {
            for &oc in &list[..cnt_prev] {
                pos[oc as usize] = -1;
            }
            *w = *w + cnt_prev as f64 + cnt_prev as f64;
        }
    }
    fail
}

/// `54d420`'s work arrays: per slot the first item's sums and the current item's, the slot's cell, per cell its slot
struct Buf {
    sa: Vec<f64>,
    sb: Vec<f64>,
    list: Vec<i32>,
    pos: Vec<i32>,
}

/// The timer of `551cc0` (`ade736`, `adf00e`, `adf2be`): work since the start plus 1.0, against the limit.
struct Timer {
    t0: f64,
    last: f64,
    limit: f64,
    /// state+0x1f0 and the work when presolve started (`3ba9de`'s budget)
    budget: f64,
    base: f64,
}

impl Timer {
    /// every 1e7 work units `551cc0` calls `3ba5ac` (log) and `3ba9de`, which sets PRELIMIT (state+0x1dc) to -1
    /// once the presolve work passes the budget; a negative PRELIMIT ends the detection with what it has.
    fn quit(&mut self, w: f64, prelimit: &Cell<i32>) -> bool {
        let e = self.t0 + w;
        if 1e7 < e - self.last {
            self.last = e;
            if w - self.base > self.budget {
                prelimit.set(-1);
            }
            return prelimit.get() < 0;
        }
        false
    }

    /// `adf2be`
    fn over(&self, w: f64) -> bool {
        if self.limit < 0.0 {
            return true;
        }
        0.0 < self.limit && 1.0 <= (self.t0 + w) / self.limit
    }
}

/// The integer mode of `551cc0` (local_a8): the extra cell stacks and arrays `5507cb` works with.
struct IntMode {
    /// local_1a8 / local_1a0 (columns), local_198 (rows), local_190 (columns), local_188 (rows)
    s1a8: Stack,
    s1a0: Stack,
    s198: Stack,
    s190: Stack,
    s188: Stack,
    /// local_168 (per column, -1), local_150 (per row, 0), local_158 (per column: bits row & 63), local_160 (per column)
    d168: Vec<i32>,
    d150: Vec<i32>,
    mask: Vec<u64>,
    d160: Vec<i64>,
    /// local_d0 as marks per component (column index)
    marks: Vec<i32>,
    /// local_88: integer columns
    nint: i32,
}

impl State {
    /// `5577ba(env, st, nodes, 1, &f4; 1.01, limit)`: the column CSR from the column lists (live nodes, list order,
    /// no work), then `551cc0`. Returns the status (0); `f4` gets local_f4: 1 or 2 for an equitable partition, else
    /// 0, -1 or -2.
    /// shortcut: with f4 > 0 the original merges the columns of each cell (type 19 records `3a0a17`, `564ce0`) and
    /// drops all but one row per cell (`397538`); no default run of the 38 models gets f4 > 0 (`54d420` never
    /// runs), so that block is not ported.
    pub fn symmetry(&mut self, _p: &Params, nodes: i32, limit: f64, f4: &mut i32) -> i32 {
        let n = self.n;
        let mut beg = vec![0i64; n + 1];
        let (mut ind, mut val) = (Vec::new(), Vec::new());
        for j in 0..n {
            beg[j] = ind.len() as i64;
            let mut q = self.col_head[j];
            while q != NIL {
                let nd = self.nd[q as usize];
                if nd.col >= 0 {
                    ind.push(nd.row);
                    val.push(nd.val);
                }
                q = nd.nc;
            }
        }
        beg[n] = ind.len() as i64;
        let mut w = self.work;
        let prelimit = Cell::new(self.prelimit);
        *f4 = self.detect(beg, ind, val, nodes, limit, 1.01, &prelimit, &mut w);
        self.work = w;
        self.prelimit = prelimit.get();
        0
    }

    /// `551cc0` with MIP arguments (variable types, no Q, no rounding, no `54d420` inside the rounds).
    #[allow(clippy::too_many_arguments)]
    fn detect(&self, beg: Vec<i64>, ind: Vec<i32>, val: Vec<f64>, nodes: i32, limit: f64, ratio: f64,
              prelimit: &Cell<i32>, w: &mut f64) -> i32 {
        let (n, m) = (self.n, self.m);
        let close = |a: f64, b: f64| !(1e-9 < (a - b).abs());
        let mut tm = Timer {
            t0: 1.0 - *w,
            last: 0.0,
            limit: if limit < 1e100 { limit } else { 0.0 },
            budget: self.budget,
            base: self.work_base,
        };
        let mut f4 = 0;
        let mut col = Side::new(n, w);
        let mut row = Side::new(m, w);
        // rows: `=` rows, then the other live rows, then each removed row in a singleton cell (last first)
        let mut placed = 0;
        for pass in 0..2 {
            row.start[row.ncell as usize] = placed;
            let mut cnt = 0;
            for i in 0..m {
                if 0 <= self.row_cnt[i] {
                    if (pass == 0) == (self.sense[i] == b'=') {
                        row.cell[i] = row.ncell;
                        row.perm[placed as usize] = i as i32;
                        placed += 1;
                        cnt += 1;
                    }
                } else if pass == 1 {
                    row.removed += 1;
                    row.perm[m - row.removed as usize] = i as i32;
                }
            }
            *w += m as f64 * 3.0;
            if 0 < cnt {
                if cnt == 1 {
                    row.single += 1;
                    row.stack.fix(row.ncell);
                }
                row.ncell += 1;
            }
        }
        if placed == 0 {
            return f4;
        }
        for t in placed as usize..m {
            row.single += 1;
            row.cell[row.perm[t] as usize] = row.ncell;
            row.start[row.ncell as usize] = t as i32;
            row.stack.fix(row.ncell);
            row.ncell += 1;
        }
        let live_rows = m - row.removed as usize;
        row.refine(live_rows, &self.rhs, close, w);
        if !row.open(live_rows, 0, ratio) || tm.quit(*w, prelimit) {
            return f4;
        }
        // columns: live ones by type (continuous cell 0, integer cell 1), flagged and removed ones in singletons
        let (mut flags, mut has_c, mut has_i, mut nint) = (0, false, false, 0);
        placed = 0;
        for j in 0..n {
            if 0 <= self.col_cnt[j] && self.col_flag[j] == 0 {
                if self.vtype[j] == b'C' {
                    col.cell[j] = 0;
                    has_c = true;
                } else {
                    col.cell[j] = 1;
                    has_i = true;
                    nint += 1;
                }
                col.perm[placed as usize] = j as i32;
                placed += 1;
                continue;
            }
            if 0 <= self.col_cnt[j] {
                flags += 1;
            } else {
                col.removed += 1;
            }
            col.perm[n - (col.removed + flags) as usize] = j as i32;
        }
        *w += n as f64 * 4.0;
        if placed <= has_c as i32 + has_i as i32 {
            return f4;
        }
        col.ncell = 1;
        if has_c && has_i {
            // `ab6a5f` on the type keys, then the first integer position (at least 1, at most placed - 1)
            let p = placed as usize;
            let mut k: Vec<i32> = col.perm[..p].iter().map(|&j| col.cell[j as usize]).collect();
            sort_asc(&mut k, &mut col.perm[..p]);
            let mut b = 1;
            while b < p - 1 && col.cell[col.perm[b] as usize] != 1 {
                b += 1;
            }
            col.start[1] = b as i32;
            col.ncell = 2;
            if b == 1 {
                col.single += 1;
                col.stack.fix(0);
            } else if b == p - 1 {
                col.single += 1;
                col.stack.fix(1);
            }
        } else if has_i {
            for t in 0..placed as usize {
                col.cell[col.perm[t] as usize] = 0;
            }
        }
        for t in placed as usize..n {
            col.single += 1;
            col.cell[col.perm[t] as usize] = col.ncell;
            col.start[col.ncell as usize] = t as i32;
            col.stack.fix(col.ncell);
            col.ncell += 1;
        }
        let live_cols = n - (col.removed + flags) as usize;
        for key in [&self.obj, &self.lb, &self.ub] {
            col.refine(live_cols, key, close, w);
            if !col.open(live_cols, flags, ratio) {
                return f4;
            }
        }
        if tm.quit(*w, prelimit) {
            return f4;
        }
        // shortcut: with no entries left the original leaves through a branch not traced; 0 assumed
        let nnz = beg[n];
        if nnz == 0 || tm.quit(*w, prelimit) {
            return f4;
        }
        col.ind = ind;
        col.val = val;
        col.beg = beg;
        transpose(&col, &mut row, w);
        if tm.quit(*w, prelimit) {
            return f4;
        }
        let mut int_mode = has_i;
        if int_mode {
            // integer columns by their entry count, then rows by their entries in integer columns
            let lens: Vec<i32> = (0..n).map(|j| (col.beg[j + 1] - col.beg[j]) as i32).collect();
            let d: Vec<i32> = (0..n).map(|j| if self.vtype[j] == b'C' { 0 } else { lens[j] }).collect();
            *w += n as f64 * 3.0;
            col.refine(live_cols, &d, |a, b| a == b, w);
            if !col.open(live_cols, flags, ratio) {
                return f4;
            }
            let mut r = vec![0i32; m];
            for j in 0..n {
                if self.vtype[j] == b'C' {
                    continue;
                }
                for k in col.beg[j] as usize..col.beg[j + 1] as usize {
                    r[col.ind[k] as usize] += 1;
                }
                *w += lens[j] as f64 * 2.0;
            }
            *w += n as f64;
            row.refine(live_rows, &r, |a, b| a == b, w);
            if !row.open(live_rows, 0, ratio) {
                return f4;
            }
        }
        col.key = vec![0; nnz as usize];
        row.key = vec![0; nnz as usize];
        Side::keys(&mut col, &mut row, w);
        if tm.quit(*w, prelimit) {
            return f4;
        }
        let mut im = if int_mode {
            let s1a8 = Stack::new(n, w);
            let s1a0 = Stack::new(n, w);
            let s198 = Stack::new(m, w);
            let s190 = Stack::new(n, w);
            let s188 = Stack::new(m, w);
            let mut mask = vec![0u64; n];
            for j in 0..n {
                for k in col.beg[j] as usize..col.beg[j + 1] as usize {
                    mask[j] |= 1u64 << (col.ind[k] as u32 & 0x3f);
                }
                *w += (col.beg[j + 1] - col.beg[j]) as f64 * 2.0;
            }
            *w += n as f64 + n as f64;
            Some(IntMode { s1a8, s1a0, s198, s190, s188, d168: vec![-1; n], d150: vec![0; m], mask, d160: vec![0; n],
                           marks: vec![0; n], nint })
        } else {
            None
        };

        let mut rng = 0x1328f02u64;
        let mut rand = vec![0i64; n.max(m)];
        let mut runs = Vec::new();
        let (mut round, mut stable, mut retries) = (0, false, 2);
        'a: loop {
            // loop A: new random numbers, hashes from scratch, every cell queued
            for r in rand.iter_mut() {
                let hi = rand31(&mut rng);
                *r = (hi << 32) + rand31(&mut rng);
            }
            *w += rand.len() as f64;
            if retries == 0 && (col.cur_own || row.cur_own) {
                col.cur_own = false;
                row.cur_own = false;
                Side::keys(&mut col, &mut row, w);
            }
            row.hash_all(&col.cell, &rand, w);
            col.hash_all(&row.cell, &rand, w);
            if tm.quit(*w, prelimit) {
                return f4;
            }
            for c in 0..col.ncell {
                col.stack.push(c);
            }
            for c in 0..row.ncell {
                row.stack.push(c);
            }
            if let Some(im) = &mut im {
                for c in 0..col.ncell {
                    if self.vtype[col.perm[col.start[c as usize] as usize] as usize] != b'C' {
                        im.s1a8.push(c);
                    }
                }
                *w += col.ncell as f64 * 3.0;
            }
            for s in [&mut col, &mut row] {
                if s.removed < s.single {
                    for j in 0..s.cell.len() {
                        if s.stack.fixed(s.cell[j]) {
                            let b = s.cur().0;
                            let d = b[j + 1] - b[j];
                            s.dnnz += d;
                        }
                    }
                }
            }
            if round < nodes {
                // loop B: rounds of columns then rows until no cell splits
                let (mut last_c, mut last_r) = (col.ncell, row.ncell);
                let mut r = round;
                loop {
                    if tm.quit(*w, prelimit) {
                        return f4;
                    }
                    col.drop_fixed(&row.cell, &row.stack, &mut row.dnnz, w);
                    {
                        let mut ext = match &mut im {
                            Some(im) => [Some(&mut im.s1a8), Some(&mut im.s1a0)],
                            None => [None, None],
                        };
                        col.settle(&mut row, &rand, &mut runs, &mut ext, w);
                    }
                    if !col.open(live_cols, flags, ratio) {
                        round = r;
                        if !stable {
                            return f4;
                        }
                        break;
                    }
                    if tm.quit(*w, prelimit) {
                        return f4;
                    }
                    row.drop_fixed(&col.cell, &col.stack, &mut col.dnnz, w);
                    {
                        let mut ext = match &mut im {
                            Some(im) => [Some(&mut im.s198), None],
                            None => [None, None],
                        };
                        row.settle(&mut col, &rand, &mut runs, &mut ext, w);
                    }
                    if !row.open(live_rows, 0, ratio) {
                        round = r;
                        if !stable {
                            return f4;
                        }
                        break;
                    }
                    if col.ncell == last_c && row.ncell == last_r {
                        // `5507cb` may refine the cells itself; true = individualize the integer columns
                        let split = match &mut im {
                            Some(im) => {
                                let want = (live_cols as i32 / 20) > im.nint;
                                self.int_split(&mut col, &mut row, im, &rand, want, w)
                            }
                            None => false,
                        };
                        if !split && col.ncell == last_c && row.ncell == last_r {
                            stable = true;
                            round = r;
                            break;
                        }
                    }
                    if col.ncell == last_c && row.ncell == last_r {
                        // integer columns go to singleton cells; the integer mode ends
                        let d: Vec<i32> = (0..n).map(|j| if self.vtype[j] == b'C' { 0 } else { j as i32 + 1 }).collect();
                        *w += n as f64 + n as f64;
                        col.refine(live_cols, &d, |a, b| a == b, w);
                        im = None;
                        int_mode = false;
                        if !col.open(live_cols, flags, ratio) {
                            return f4;
                        }
                        round = r + 1;
                        continue 'a;
                    }
                    if tm.over(*w) {
                        round = nodes;
                        break;
                    }
                    (last_c, last_r) = (col.ncell, row.ncell);
                    r += 1;
                    if nodes <= r {
                        round = r;
                        break;
                    }
                }
            }
            if col.ncell as usize == n || tm.quit(*w, prelimit) {
                return f4;
            }
            if !stable && int_mode {
                return -2;
            }
            f4 = self.verify(&col, &row, w);
            if f4 == 0 {
                if !stable || nodes - 1 <= round || retries == 0 {
                    return -2;
                }
                retries -= 1;
                round += 1;
                continue;
            }
            let live_c = (n - col.removed as usize) as i32;
            if f4 == 1 && col.ncell < live_c / 5 && row.ncell < live_rows as i32 / 5 {
                f4 = 2;
            }
            return f4;
        }
    }

    /// `5507cb(…)` when the hash refinement stalls in integer mode. It works through three cell queues: integer
    /// column cells (s1a8: are the columns' row sets disjoint? if not, split by sharing or individualize), row cells
    /// (s198: requeue column cells met unevenly or with other keys), and checked column cells (s1a0: are the columns in
    /// distinct components of the graph over fixed cells?). It stops at the first split it makes (`54e5eb`) or when
    /// the queues are empty. Returns the flag (*param_36): true = individualize all integer columns (only when
    /// `want`, i.e. few integer columns, and the cell to split is continuous).
    fn int_split(&self, col: &mut Side, row: &mut Side, im: &mut IntMode, rand: &[i64], want: bool, w: &mut f64) -> bool {
        let mut size: Vec<i32> = Vec::new(); // local_60: cell sizes, allocated on first use
        let mut runs = Vec::new();
        loop {
            if im.s1a8.len() == 0 && im.s198.len() == 0 && im.s1a0.len() == 0 {
                return false;
            }
            while let Some(c) = im.s1a8.pop() {
                if let Some(flag) = self.int_cell(col, row, im, rand, want, c, &mut runs, w) {
                    return flag;
                }
            }
            while let Some(r) = im.s198.pop() {
                self.int_row_cell(col, row, im, r, &mut size, w);
                if im.s1a8.len() > 0 {
                    break;
                }
            }
            if im.s1a8.len() > 0 {
                continue;
            }
            while let Some(c) = im.s1a0.pop() {
                if let Some(flag) = self.int_components(col, row, im, rand, want, c, &mut runs, w) {
                    return flag;
                }
            }
        }
    }

    /// `5507cb` part 1, integer column cell `c`: a cheap test on 64-bit row masks, then an exact one on the original
    /// lines (d150 per row). Disjoint: the cell is done, its rows' cells queued (s198), itself queued for part 3.
    /// Overlapping: split by "shares a row with an earlier column" (d160), or individualize when every column shares
    /// rows with all earlier ones (a chain). Some(flag) = a split was made (or the flag set): stop.
    #[allow(clippy::too_many_arguments)]
    fn int_cell(&self, col: &mut Side, row: &mut Side, im: &mut IntMode, rand: &[i64], want: bool, c: i32,
                runs: &mut Vec<usize>, w: &mut f64) -> Option<bool> {
        let n = self.n;
        let cu = c as usize;
        let st = col.start[cu] as usize;
        if st == n - 1 || col.cell[col.perm[st + 1] as usize] != c {
            im.s1a8.fix(c);
            return None;
        }
        let (mut acc, mut pos, mut conf) = (0u64, st, false);
        while pos < n {
            let j = col.perm[pos] as usize;
            if col.cell[j] != c {
                break;
            }
            if acc & im.mask[j] != 0 {
                conf = true;
                break;
            }
            acc |= im.mask[j];
            pos += 1;
        }
        *w = (pos - st) as f64 * 3.0 + *w;
        if conf {
            let (mut lvl, mut shared) = (0i32, false);
            let mut q = st;
            while q < n {
                let j = col.perm[q] as usize;
                if col.cell[j] != c {
                    break;
                }
                let (b, e) = (col.beg[j] as usize, col.beg[j + 1] as usize);
                let mut k = e;
                if q != st {
                    k = b;
                    while k < e && im.d150[col.ind[k] as usize] == 0 {
                        k += 1;
                    }
                    *w = (k - b) as f64 * 2.0 + *w;
                }
                if k >= e {
                    im.d160[j] = 0;
                    for kk in b..e {
                        im.d150[col.ind[kk] as usize] = 1;
                    }
                    *w = (e - b) as f64 * 2.0 + *w;
                    lvl = 1;
                } else {
                    shared = true;
                    im.d160[j] = 1;
                    let i = (q - st) as i32;
                    if lvl == i {
                        for kk in k..e {
                            let r = col.ind[kk] as usize;
                            if im.d150[r] == i {
                                im.d150[r] = i + 1;
                                lvl = i + 1;
                            }
                        }
                        *w = (e - k) as f64 * 2.0 + *w;
                    }
                }
                q += 1;
            }
            *w = (q - st) as f64 * 2.0 + *w;
            for t in st..q {
                let j = col.perm[t] as usize;
                if im.d160[j] != 0 {
                    continue;
                }
                let (b, e) = (col.beg[j] as usize, col.beg[j + 1] as usize);
                for kk in b..e {
                    im.d150[col.ind[kk] as usize] = 0;
                }
                *w = (e - b) as f64 * 2.0 + *w;
            }
            *w = *w + (q - st) as f64 * 2.0;
            if shared {
                let each = lvl == (q - st) as i32;
                if each {
                    im.s1a8.fix(c);
                } else {
                    im.s1a8.push(c);
                }
                return Some(self.int_split_cell(col, row, im, rand, want, c, each, runs, w));
            }
        }
        im.s1a8.fix(c);
        let st = col.start[cu] as usize;
        let mut p = st;
        while p < n {
            let j = col.perm[p] as usize;
            if col.cell[j] != c {
                break;
            }
            let (b, ind) = col.cur();
            let (s0, e0) = (b[j] as usize, b[j + 1] as usize);
            for k in s0..e0 {
                im.s198.push(row.cell[ind[k] as usize]);
            }
            *w = (e0 - s0) as f64 * 3.0 + *w;
            p += 1;
        }
        *w = *w + (p - st) as f64 * 3.0;
        im.s1a0.push(c);
        None
    }

    /// `5507cb` part 2, row cell `r`: per row, column cells met a number of times other than their size go back to
    /// s1a8 (d160 counts per cell); if some met cell is in no s1a8 state, cells whose keys differ from the first
    /// row's go back to s1a8 too.
    fn int_row_cell(&self, col: &Side, row: &Side, im: &mut IntMode, r: i32, size: &mut Vec<i32>, w: &mut f64) {
        let (n, m) = (self.n, self.m);
        im.s198.fix(r);
        let mut flag = false;
        let rst = row.start[r as usize] as usize;
        let (rb, rind) = row.cur();
        let mut p = rst;
        while p < m {
            let i = row.perm[p] as usize;
            if row.cell[i] != r {
                break;
            }
            let (b, e) = (rb[i] as usize, rb[i + 1] as usize);
            for k in b..e {
                im.d160[col.cell[rind[k] as usize] as usize] = 0;
            }
            *w = (e - b) as f64 * 3.0 + *w;
            for k in b..e {
                im.d160[col.cell[rind[k] as usize] as usize] += 1;
            }
            *w = *w + (e - b) as f64 * 3.0;
            for k in b..e {
                let cc = col.cell[rind[k] as usize];
                let cnt = im.d160[cc as usize];
                if cnt == 0 {
                    continue;
                }
                if size.is_empty() {
                    *size = vec![0; n];
                }
                if size[cc as usize] == 0 {
                    let s0 = col.start[cc as usize] as usize;
                    let mut t = s0 + 1;
                    while t < n && col.cell[col.perm[t] as usize] == cc {
                        t += 1;
                    }
                    size[cc as usize] = (t - s0) as i32;
                }
                if size[cc as usize] as i64 != cnt {
                    im.s1a8.push(cc);
                }
                im.d160[cc as usize] = 0;
                if !flag {
                    flag = !im.s1a8.queued(cc) && !im.s1a8.fixed(cc);
                }
            }
            *w = *w + (e - b) as f64 * 5.0;
            p += 1;
        }
        if !flag {
            return;
        }
        let i0 = row.perm[rst] as usize;
        let (b0, e0) = (rb[i0] as usize, rb[i0 + 1] as usize);
        for k in b0..e0 {
            im.d160[col.cell[rind[k] as usize] as usize] = row.key[k];
        }
        *w = (e0 - b0) as f64 * 4.0 + *w;
        let mut p = rst;
        while p < m {
            let i = row.perm[p] as usize;
            if row.cell[i] != r {
                break;
            }
            let (b, e) = (rb[i] as usize, rb[i + 1] as usize);
            for k in b..e {
                let cc = col.cell[rind[k] as usize];
                if im.d160[cc as usize] != row.key[k] {
                    im.s1a8.push(cc);
                }
            }
            *w = (e - b) as f64 * 4.0 + *w;
            p += 1;
        }
        *w = *w + (p - rst) as f64 * 3.0;
    }

    /// `5507cb` part 3, column cell `c` from s1a0: the columns' components (d168, found by a search through rows and
    /// columns of fixed s1a8 cells, s190/s188) must differ; if two coincide the cell is individualized.
    #[allow(clippy::too_many_arguments)]
    fn int_components(&self, col: &mut Side, row: &mut Side, im: &mut IntMode, rand: &[i64], want: bool, c: i32,
                      runs: &mut Vec<usize>, w: &mut f64) -> Option<bool> {
        let n = self.n;
        let st = col.start[c as usize] as usize;
        if st == n - 1 || col.cell[col.perm[st + 1] as usize] != c {
            im.s1a0.fix(c);
            return None;
        }
        let mut p = st;
        while p < n {
            let j = col.perm[p] as usize;
            if col.cell[j] != c {
                break;
            }
            if im.d168[j] == -1 {
                self.component(col, row, im, j, w);
            }
            im.marks[im.d168[j] as usize] = 0;
            p += 1;
        }
        *w = (p - st) as f64 * 4.0 + *w;
        let mut distinct = true;
        let mut p = st;
        while p < n {
            let j = col.perm[p] as usize;
            if col.cell[j] != c {
                break;
            }
            let k = im.d168[j] as usize;
            if im.marks[k] != 0 {
                distinct = false;
                break;
            }
            im.marks[k] = 1;
            p += 1;
        }
        *w = *w + (p - st) as f64 * 4.0;
        if distinct {
            return None;
        }
        Some(self.int_split_cell(col, row, im, rand, want, c, true, runs, w))
    }

    /// The component of column `j` (`5507cb`'s search): columns reached through their current rows, over columns
    /// whose cell is fixed in s1a8; each gets d168 = j.
    fn component(&self, col: &Side, row: &Side, im: &mut IntMode, j: usize, w: &mut f64) {
        im.s190.push(j as i32);
        let (cb, cind) = col.cur();
        let (rb, rind) = row.cur();
        while im.s190.len() > 0 {
            while let Some(x) = im.s190.pop() {
                let x = x as usize;
                im.s190.fix(x as i32);
                im.d168[x] = j as i32;
                for k in cb[x] as usize..cb[x + 1] as usize {
                    im.s188.push(cind[k]);
                }
                *w = (cb[x + 1] - cb[x]) as f64 * 2.0 + *w;
            }
            while let Some(r) = im.s188.pop() {
                let r = r as usize;
                im.s188.fix(r as i32);
                for k in rb[r] as usize..rb[r + 1] as usize {
                    let cj = rind[k];
                    if im.s1a8.fixed(col.cell[cj as usize]) {
                        im.s190.push(cj);
                    }
                }
                *w = (rb[r + 1] - rb[r]) as f64 * 2.0 + *w;
            }
        }
    }

    /// `5507cb`'s split of column cell `c`: the flag when wanted and the cell is continuous, else `54e5eb` with the
    /// cell's end searched (work 2 per position past the first) by d160 marks or every column apart. Returns the flag.
    #[allow(clippy::too_many_arguments)]
    fn int_split_cell(&self, col: &mut Side, row: &mut Side, im: &mut IntMode, rand: &[i64], want: bool, c: i32,
                      each: bool, runs: &mut Vec<usize>, w: &mut f64) -> bool {
        let n = self.n;
        let st = col.start[c as usize] as usize;
        if want && self.vtype[col.perm[st] as usize] == b'C' {
            return true;
        }
        let mut e = st + 1;
        while e < n && col.cell[col.perm[e] as usize] == c {
            e += 1;
        }
        *w = ((e - st) as f64 - 1.0) * 2.0 + *w;
        let IntMode { s1a8, s1a0, d160, .. } = im;
        let by = if each { By::Each } else { By::Keys(d160) };
        col.split(row, rand, c, e, by, runs, &mut [Some(s1a8), Some(s1a0)], w);
        false
    }

    /// `54d420`: 1 when the partition is equitable on the real data (same rhs and sense, obj and bounds within a
    /// cell, same coefficient sums per cell of the other side), -1 when only the columns fail and both sides are
    /// coarse (at most 100 cells, 1/20 of the items), else 0.
    fn verify(&self, col: &Side, row: &Side, w: &mut f64) -> i32 {
        let k = col.ncell.max(row.ncell) as usize;
        let mut buf = Buf { sa: vec![0.0; k], sb: vec![0.0; k], list: vec![0; k], pos: vec![-1; k] };
        let (n, m) = (self.n, self.m);
        let rows_differ = unequal(m, &row.beg, &row.ind, &row.val, [Some(&self.rhs), None, None], Some(&self.sense),
                                  &col.cell, &row.cell, &row.perm, &mut buf, w);
        if rows_differ {
            return 0;
        }
        let cols_differ = unequal(n, &col.beg, &col.ind, &col.val, [Some(&self.obj), Some(&self.lb), Some(&self.ub)],
                                  None, &row.cell, &col.cell, &col.perm, &mut buf, w);
        if !cols_differ {
            return 1;
        }
        let (rc, cc) = (row.ncell - row.removed, col.ncell - col.removed);
        let coarse = rc as f64 * 20.0 <= (m as i32 - row.removed) as f64
            && cc as f64 * 20.0 <= (n as i32 - col.removed) as f64
            && rc < 101
            && cc < 101;
        if coarse { -1 } else { 0 }
    }
}

/// `790e3`: the row CSR of the columns, entries in column order.
fn transpose(col: &Side, row: &mut Side, w: &mut f64) {
    let (n, m) = (col.cell.len(), row.cell.len());
    let mut rb = vec![0i64; m + 1];
    for j in 0..n {
        for k in col.beg[j]..col.beg[j + 1] {
            rb[col.ind[k as usize] as usize + 1] += 1;
        }
        *w += (col.beg[j + 1] - col.beg[j]) as f64 * 2.0;
    }
    *w += n as f64;
    // starts shifted by one, so filling leaves each row's end in rb[i + 1]
    let mut s = col.beg[n];
    for i in (1..=m).rev() {
        s -= rb[i];
        rb[i] = s;
    }
    let nnz = col.beg[n] as usize;
    let (mut ri, mut rv) = (vec![0i32; nnz], vec![0f64; nnz]);
    for j in 0..n {
        for k in col.beg[j] as usize..col.beg[j + 1] as usize {
            let p = &mut rb[col.ind[k] as usize + 1];
            ri[*p as usize] = j as i32;
            rv[*p as usize] = col.val[k];
            *p += 1;
        }
        *w += (col.beg[j + 1] - col.beg[j]) as f64 * 5.0;
    }
    *w += n as f64 * 3.0;
    row.beg = rb;
    row.ind = ri;
    row.val = rv;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rand31_first_values() {
        let mut s = 0x1328f02u64;
        let a = rand31(&mut s);
        let t = 0x1328f02u64.wrapping_mul(0x5851f42d4c957f2d).wrapping_add(0x14057b7ef767814f);
        assert_eq!(s, t);
        let mut x = (t >> 32) as u32;
        x ^= x >> 11;
        x ^= (x & 0x13a58ad) << 7;
        x ^= (x & 0x1df8c) << 15;
        assert_eq!(a, ((x ^ (x >> 18)) & 0x7fffffff) as i64);
    }

    #[test]
    fn real_gcd_cases() {
        assert_eq!(real_gcd(&[6.0, 4.0]), 2.0);
        assert_eq!(real_gcd(&[1.5, 0.5]), 0.5);
        assert_eq!(real_gcd(&[0.3, 0.2]), 0.1);
        assert_eq!(real_gcd(&[1e-6, 1.0]), 0.0);
    }
}
