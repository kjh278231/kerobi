//! DynamicRows dive row selection of `5fb8ec` (C:1620–1842, after the gate at `0x513078` up to `0x513951`): the rows the
//! dive node deactivates through `10e5fc` (docs/m2/mip_dive_rows.md, docs/m2/mip_dive_rows_rust.md).
//! `RowList` keeps the deactivated list X = `D+0x140` (`10e5fc`, `10e662`, `edd71`) and swaps the reduced columns into D
//! (`10f448`/`10f5ae`) (docs/m2/mip_dive_list_rust.md).
//! shortcut: the gate (DynamicRows, dive depth, node state, `a7f0ad`/`a7f923`) is not ported and nothing calls this from
//! the B&B yet; upgrade with M2-03d-4.
use crate::knapsack::act_sum;

/// What the selection reads. Model rows are the cut-free MIP LP `(mip+8)->D` in row-major form; cut j is row nrows+j.
pub struct Sel<'a> {
    /// Row i entries are `ind/val[beg[i]..end[i]]` (`D+0x118/0x120/0x130/0x138`).
    pub beg: &'a [usize],
    pub end: &'a [usize],
    pub ind: &'a [i32],
    pub val: &'a [f64],
    /// `D+0x380`, `D+0x388` (`<`, `>`, `=` as bytes), `mip+0x650`.
    pub rhs: &'a [f64],
    pub sense: &'a [u8],
    pub vtype: &'a [u8],
    /// Node bounds `I+0x80/0x88`.
    pub lb: &'a [f64],
    pub ub: &'a [f64],
    /// Node row flags `I+0xa0`, nrows + ncuts.
    pub flags: &'a [i32],
    /// Node LP slack `(I+8)->+0x20` (read at nrows+j) and reduced costs `(I+8)->+0x28`.
    pub slack: &'a [f64],
    pub rc: &'a [f64],
    /// `7889a5(mip+0x32e8, i)` array: row i is excluded when `i < len` and the entry is ≥ 0. Empty without the object.
    pub b7: &'a [i32],
    /// Per cut: type `+0x2c` and whether `+0x40` is set.
    pub cuts: &'a [(i32, bool)],
    /// FeasTol env+0x2460, DualReductions env+0x2430.
    pub feastol: f64,
    pub dual_reductions: i32,
    /// `5e7204(mip)` (1e100 without an incumbent) and `I+0x10`.
    pub cutoff: f64,
    pub obj_off: f64,
}

/// Selected rows in order (model rows ascending, then cuts). `calls` receives every `66661c` call as (row, result).
/// `w` is the work counter `param_3` (C only, not captured).
pub fn select(s: &Sel, calls: &mut Vec<(i32, bool)>, w: &mut f64) -> Vec<i32> {
    let nrows = s.rhs.len();
    let mut co = s.cutoff;
    if co < 1e100 {
        co = co - s.obj_off;
    }
    let mut out = Vec::new();
    for i in 0..nrows {
        if s.flags[i] != 0 || s.b7.get(i).is_some_and(|&v| v >= 0) {
            continue;
        }
        let (ind, val) = (&s.ind[s.beg[i]..s.end[i]], &s.val[s.beg[i]..s.end[i]]);
        let rhs = s.rhs[i];
        let maxact = -act_sum(ind, val, s.lb, s.ub, -1.0, w);
        let cutoff_path = if !(s.feastol * 0.1 + rhs >= maxact) {
            true
        } else {
            let minact = if s.sense[i] == b'=' { act_sum(ind, val, s.lb, s.ub, 1.0, w) } else { 1e100 };
            !(minact >= rhs - s.feastol * 0.1)
        };
        if cutoff_path {
            if s.dual_reductions == 0 || !(1e100 > co) || s.sense[i] == b'=' {
                continue;
            }
            let r = f66661c(rhs, co, ind, val, s.vtype, s.lb, s.ub, s.rc, w);
            calls.push((i as i32, r));
            if !r {
                continue;
            }
        }
        out.push(i as i32);
    }
    *w = nrows as f64 + nrows as f64 + *w;
    for (j, &(kind, attr40)) in s.cuts.iter().enumerate() {
        let k = nrows + j;
        if s.flags[k] == 0 && kind != 0x14 && !attr40 && 1e-05 < s.slack[k] {
            out.push(k as i32);
        }
    }
    *w = *w + s.cuts.len() as f64 * 3.0;
    out
}

/// `10e5fc`/`10e662` return code for a row outside [0, D rows).
pub const ERR_ROW: i32 = 0x2716;
const DEACT: u32 = 0x1000;
const REACT: u32 = 0x2000;

/// Deactivated dive rows of one LP: pending row flags (`lp+0x230`→`+0xb0`) and the list X = `D+0x140`.
#[derive(Default)]
pub struct RowList {
    /// Pending flags per D row; empty when no `edd71` is due.
    flags: Vec<u32>,
    x: Option<X>,
}

#[derive(Default)]
struct X {
    /// X+0x08 (count X+0), X+0x10 row → list position or −1.
    list: Vec<i32>,
    pos: Vec<i32>,
    /// X+0x20 rhs and X+0x28 column lengths: the reduced copies, or D's own arrays while swapped in.
    rhs: Vec<f64>,
    len: Vec<i32>,
    /// X+0x18 copies are current, X+0x1c swap depth.
    built: bool,
    refs: i32,
}

impl RowList {
    pub fn list(&self) -> &[i32] {
        self.x.as_ref().map_or(&[], |x| &x.list)
    }

    fn mark(&mut self, nrows: usize, rows: &[i32], on: u32) -> Result<(), i32> {
        if self.flags.len() < nrows {
            self.flags.resize(nrows, 0);
        }
        for &r in rows {
            if r < 0 || r as usize >= nrows {
                return Err(ERR_ROW);
            }
            let f = &mut self.flags[r as usize];
            *f = *f & !(DEACT | REACT) | on;
        }
        Ok(())
    }

    /// `10e5fc`: request deactivation; the caller runs `update` next (`5fb8ec` C:1856).
    pub fn deactivate(&mut self, nrows: usize, rows: &[i32]) -> Result<(), i32> {
        self.mark(nrows, rows, DEACT)
    }

    /// `10e662`: flush pending requests, then request reactivation of every listed row.
    pub fn reactivate_all(&mut self, nrows: usize) -> Result<(), i32> {
        if self.list().is_empty() {
            return Ok(());
        }
        self.update();
        let list = self.list().to_vec();
        self.mark(nrows, &list, REACT)
    }

    /// `edd71` list block (C:8020–8417): rows in increasing order; a deactivated row is appended when not listed, a
    /// reactivated listed row is replaced by the last one.
    pub fn update(&mut self) {
        if self.flags.is_empty() {
            return;
        }
        let flags = std::mem::take(&mut self.flags);
        let x = self.x.get_or_insert_with(X::default);
        if x.pos.len() < flags.len() {
            x.pos.resize(flags.len(), -1);
        }
        for (i, &f) in flags.iter().enumerate() {
            if f & DEACT != 0 {
                if x.pos[i] < 0 {
                    x.pos[i] = x.list.len() as i32;
                    x.list.push(i as i32);
                }
            } else if f & REACT != 0 && x.pos[i] >= 0 {
                let (p, last) = (x.pos[i] as usize, *x.list.last().unwrap());
                x.list[p] = last;
                x.pos[last as usize] = p as i32;
                x.list.pop();
                x.pos[i] = -1;
            }
        }
        x.built = false;
    }

    /// D changed (rows, rhs, coefficients): `dcd29`/`db102`/`fbf68`/`edd71` C:3454 clear X+0x18; new rows are unlisted.
    pub fn changed(&mut self, nrows: usize) {
        let Some(x) = self.x.as_mut() else { return };
        x.built = false;
        // shortcut: row deletion with listed rows (renumbering) is not ported; never seen in the M2-03d-8 runs
        assert!(x.list.iter().all(|&r| (r as usize) < nrows), "row deletion with a non-empty dive list");
        x.pos.resize(nrows.max(x.pos.len()), -1);
    }

    /// `10f448`: on the first swap of a nesting, put the reduced column lengths and rhs into D. Building the copies moves
    /// each column's listed-row entries to its end in place (two-pointer exchange) and zeroes the listed rows' rhs.
    pub fn swap_in(&mut self, beg: &[i64], len: &mut Vec<i32>, ind: &mut [i32], val: &mut [f64], rhs: &mut Vec<f64>) {
        let Some(x) = self.x.as_mut() else { return };
        x.refs += 1;
        if x.refs != 1 {
            return;
        }
        if !x.built {
            x.pos.resize(rhs.len().max(x.pos.len()), -1);
            x.len.resize(len.len(), 0);
            for j in 0..len.len() {
                let s = beg[j] as usize;
                let (mut a, mut b) = (s as i64, s as i64 + len[j] as i64 - 1);
                let listed = |k: i64, ind: &[i32]| x.pos[ind[k as usize] as usize] >= 0;
                while a <= b {
                    if !listed(a, ind) {
                        a += 1;
                        continue;
                    }
                    while a <= b {
                        if !listed(b, ind) {
                            if a < b {
                                ind.swap(a as usize, b as usize);
                                val.swap(a as usize, b as usize);
                                a += 1;
                                b -= 1;
                            }
                            break;
                        }
                        b -= 1;
                    }
                }
                x.len[j] = (a - s as i64) as i32;
            }
            x.rhs.clear();
            x.rhs.extend_from_slice(rhs);
            for &r in &x.list {
                x.rhs[r as usize] = 0.0;
            }
            x.built = true;
        }
        std::mem::swap(len, &mut x.len);
        std::mem::swap(rhs, &mut x.rhs);
    }

    /// `10f5ae`: on the last unswap of a nesting, give D its own lengths and rhs back.
    pub fn swap_out(&mut self, len: &mut Vec<i32>, rhs: &mut Vec<f64>) {
        let Some(x) = self.x.as_mut() else { return };
        x.refs -= 1;
        if x.refs == 0 {
            std::mem::swap(len, &mut x.len);
            std::mem::swap(rhs, &mut x.rhs);
        }
    }

    /// X+0x18 and X+0x1c, for the replay trace.
    pub fn state(&self) -> (bool, i32) {
        self.x.as_ref().map_or((false, 0), |x| (x.built, x.refs))
    }
}

/// `66661c`: true when lifting the row activity to rhs (integral rows: the next integer above rhs) costs more than
/// `gap` in reduced cost, taking the five cheapest |rc/a| columns first. Comparisons follow the asm branches
/// (unordered included), which differ from the decompiled C on NaN; the width cap is `minsd`.
#[allow(clippy::too_many_arguments)]
fn f66661c(rhs: f64, gap: f64, ind: &[i32], val: &[f64], vtype: &[u8], lb: &[f64], ub: &[f64], rc: &[f64], w: &mut f64) -> bool {
    let mut loc = 0.0;
    let mut integral = true;
    // slots 1..=5 by |rc/a| ascending: |rc|, |a|, ub - lb
    let (mut e, mut c, mut wd) = ([0.0f64; 6], [0.0f64; 6], [0.0f64; 6]);
    let mut k = 0usize;
    for (&j, &a) in ind.iter().zip(val) {
        let j = j as usize;
        let (l, u) = (lb[j], ub[j]);
        let d = u - l;
        if d <= 1e-10 {
            loc = loc + a * 0.5 * (l + u);
            continue;
        }
        integral = integral && vtype[j] != b'C' && a - (a + 1e-10).floor() <= 1e-10;
        if a.abs() <= 1e-6 {
            return false;
        }
        let r = rc[j];
        if a > 0.0 && 1e-10 >= r {
            loc = loc + u * a;
            continue;
        }
        if 0.0 < a && l <= -1e30 {
            return false;
        }
        if 0.0 > a && r >= -1e-10 {
            loc = loc + l * a;
            continue;
        }
        if a < 0.0 && 1e30 <= u {
            return false;
        }
        let b = if 0.0 < a { l } else { u };
        loc = loc + b * a;
        let q = (r / a).abs();
        let mut p = k;
        while p > 0 && !(e[p] / c[p] <= q) {
            if p < 5 {
                (e[p + 1], c[p + 1], wd[p + 1]) = (e[p], c[p], wd[p]);
            }
            p -= 1;
        }
        if p < 5 {
            (e[p + 1], c[p + 1], wd[p + 1]) = (r.abs(), a.abs(), d);
            if k < 5 {
                k += 1;
            }
        }
    }
    *w = ind.len() as f64 * 14.0 + *w;
    let rhs = if integral { (rhs + 9.999999999999999e-05).floor() + 1.0 } else { rhs };
    if !(loc < rhs - 1e-06) {
        return false;
    }
    let (mut cost, mut s) = (0.0, 0);
    while s < k {
        let mut step = (rhs - loc) / c[s + 1];
        if s + 1 < k && !(step < wd[s + 1]) {
            step = wd[s + 1];
        }
        loc = loc + c[s + 1] * step;
        cost = cost + e[s + 1] * step;
        if rhs - 1e-06 <= loc {
            break;
        }
        s += 1;
    }
    *w = s as f64 * 4.0 + *w;
    gap < cost
}

#[cfg(test)]
mod tests {
    use super::*;

    // self-check of the port on a hand-made node; the original comparison is tests/dive_rows_l1.rs
    #[test]
    fn clauses() {
        // rows: x0 + x1 <= 2 (redundant), x0 + x1 <= 1 (flagged), x0 + x1 <= 1 (cutoff path); cuts: slack 1e-5, 2e-5
        let (beg, end, ind, val) = ([0, 2, 4], [2, 4, 6], [0, 1, 0, 1, 0, 1], [1.0; 6]);
        let (lb, ub, rc) = ([0.0; 2], [1.0; 2], [3.0, 4.0]);
        let mut s = Sel {
            beg: &beg, end: &end, ind: &ind, val: &val, rhs: &[2.0, 1.0, 1.0], sense: b"<<<", vtype: b"BB",
            lb: &lb, ub: &ub, flags: &[0, 1, 0, 0, 0], slack: &[0.0, 0.0, 0.0, 1e-5, 2e-5], rc: &rc, b7: &[],
            cuts: &[(0, false), (0, false)], feastol: 1e-6, dual_reductions: 1, cutoff: 10.0, obj_off: 4.0,
        };
        // row 2: integral, rhs -> 2, cost 3 (x0 to 1) + 4 (x1 to 1) = 7 > gap 6
        let (mut calls, mut w) = (vec![], 0.0);
        assert_eq!(select(&s, &mut calls, &mut w), vec![0, 2, 4]);
        assert_eq!(calls, vec![(2, true)]);
        s.obj_off = 3.0; // gap 7: not more than the cost
        calls.clear();
        assert_eq!(select(&s, &mut calls, &mut w), vec![0, 4]);
        s.dual_reductions = 0;
        calls.clear();
        assert_eq!(select(&s, &mut calls, &mut w), vec![0, 4]);
        assert!(calls.is_empty());
    }

    // self-check on hand-made rows; the original comparison is tests/dive_list_l1.rs
    #[test]
    fn row_list() {
        let mut l = RowList::default();
        assert_eq!(l.reactivate_all(5), Ok(()));
        l.update();
        assert!(l.list().is_empty());
        l.deactivate(5, &[1, 3, 4]).unwrap();
        l.update();
        assert_eq!(l.list(), [1, 3, 4]);
        // reactivate all, deactivate 3 again before edd71: 1 and 4 leave by moving the last entry in
        l.reactivate_all(5).unwrap();
        l.deactivate(5, &[3]).unwrap();
        l.update();
        assert_eq!(l.list(), [3]);
        assert_eq!(l.deactivate(5, &[2, 5]), Err(ERR_ROW));
        l.update();
        assert_eq!(l.list(), [3, 2]);
        // one column over rows 0..4 with rows 1 and 3 listed: two-pointer exchange, not a stable partition
        let mut l = RowList::default();
        l.deactivate(4, &[1, 3]).unwrap();
        l.update();
        let (beg, mut len, mut ind, mut val, mut rhs) = ([0i64], vec![4], vec![1, 0, 2, 3], vec![10.0, 20.0, 30.0, 40.0], vec![1.0; 4]);
        for k in 0..2 {
            l.swap_in(&beg, &mut len, &mut ind, &mut val, &mut rhs);
            l.swap_in(&beg, &mut len, &mut ind, &mut val, &mut rhs);
            assert_eq!((len.as_slice(), ind.as_slice(), val.as_slice()), ([2].as_slice(), [2, 0, 1, 3].as_slice(), [30.0, 20.0, 10.0, 40.0].as_slice()));
            assert_eq!(rhs, [1.0, 0.0, 1.0, 0.0]);
            assert_eq!(l.state(), (true, 2), "pass {k}");
            l.swap_out(&mut len, &mut rhs);
            l.swap_out(&mut len, &mut rhs);
            assert_eq!((len.as_slice(), rhs.as_slice()), ([4].as_slice(), [1.0; 4].as_slice()));
        }
    }
}
