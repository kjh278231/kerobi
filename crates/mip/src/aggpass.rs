//! `55a779` repeated coefficient patterns (docs/m2/m2_red_roundend.md §5): long rows are cut by column class (binary,
//! integer, continuous × coefficient sign); a class part of 20 or more entries, scaled by its smallest |a|, is a
//! pattern. Patterns met more than once would be replaced by new aggregate columns (part 2).
use crate::state::{State, NIL};
use crate::Params;
use std::collections::HashMap;

/// `55866f`: hash of a pattern (columns and values rounded to 1e-6), i32 arithmetic with wrap-around.
fn pattern_hash(cols: &[i32], vals: &[f64]) -> u32 {
    let mut h: i32 = 0;
    for (&c, &v) in cols.iter().zip(vals) {
        let f = (v * 1000000.0 + 0.5).floor();
        let f = if -4.611686018427388e18 <= f { if f <= 4.611686018427388e18 { f } else { 4.611686018427388e18 } } else { -4.611686018427388e18 };
        h = (h.wrapping_mul(31).wrapping_add(c)).wrapping_mul(31).wrapping_add(f as i64 as i32);
    }
    h as u32
}

/// `55880a`: same columns, values within 1e-10.
fn pattern_eq(c1: &[i32], v1: &[f64], c2: &[i32], v2: &[f64]) -> bool {
    c1 == c2 && v1.iter().zip(v2).all(|(a, b)| !(1e-10 < (a - b).abs()))
}

/// `coefclasssign`: sign of each class (B+, B−, I+, I−, C+, C−).
const CLASS_SIGN: [f64; 6] = [1.0, -1.0, 1.0, -1.0, 1.0, -1.0];

impl State {
    /// `55a779(env, st, T20)`. Returns 0.
    /// shortcut: part 2 (a pattern met twice: new columns and rows through `3bbf87`/`3bde19`/`3bea24`) is not
    /// ported; no default run of the 38 models finds a repeated pattern (`55880a` never runs). Port it with a model
    /// that does.
    pub fn agg_pass(&mut self, _p: &Params) -> i32 {
        if self.nnz > 2_000_000_000 {
            return 0;
        }
        let mu = self.work_mult;
        let rows = self.adjacent_list(false);
        let class = |st: &State, j: usize, a: f64| {
            (a <= 0.0) as usize + match st.vtype[j] {
                b'I' => 2,
                b'C' => 4,
                _ => 0,
            }
        };
        // stored patterns (columns, values) and the hash table (hash -> ids, newest last)
        let (mut pcols, mut pvals): (Vec<Vec<i32>>, Vec<Vec<f64>>) = (Vec::new(), Vec::new());
        let mut table: HashMap<u32, Vec<usize>> = HashMap::new();
        for &ri in &rows {
            let r = ri as usize;
            if !(19 < self.row_cnt[r]) {
                continue;
            }
            let mut cnt = [0usize; 6];
            let mut walked = 0;
            let mut q = self.row_head[r];
            while q != NIL {
                let nd = self.nd[q as usize];
                if nd.col >= 0 && 0.001 < nd.val.abs() {
                    cnt[class(self, nd.col as usize, nd.val)] += 1;
                }
                walked += 1;
                q = nd.nr;
            }
            self.work += mu * 4.0 * walked as f64;
            let mut end = [0usize; 7];
            for k in 0..6 {
                end[k + 1] = end[k] + cnt[k];
            }
            // filled from the back of each class: `end[k + 1]` counts down to the class start
            let mut pos = end;
            let total = end[6];
            let (mut cols, mut vals) = (vec![0i32; total], vec![0f64; total]);
            let mut min = [1e100f64; 6];
            let mut walked = 0;
            let mut q = self.row_head[r];
            while q != NIL {
                let nd = self.nd[q as usize];
                let a = nd.val.abs();
                if nd.col >= 0 && 0.001 < a {
                    let k = class(self, nd.col as usize, nd.val);
                    min[k] = if a <= min[k] { a } else { min[k] };
                    pos[k + 1] -= 1;
                    cols[pos[k + 1]] = nd.col;
                    vals[pos[k + 1]] = nd.val;
                }
                walked += 1;
                q = nd.nr;
            }
            self.work += mu * 5.0 * walked as f64;
            for k in 0..6 {
                let (s, e) = (end[k], end[k + 1]);
                if !(19 < e - s) {
                    continue;
                }
                let scale = CLASS_SIGN[k] * min[k];
                for v in &mut vals[s..e] {
                    *v /= scale;
                }
                self.work = (e - s) as f64 * mu + self.work;
                // `a9f78f`: by column (distinct in a row, so any sort gives this order)
                let mut seg: Vec<(i32, f64)> = cols[s..e].iter().copied().zip(vals[s..e].iter().copied()).collect();
                seg.sort_by_key(|x| x.0);
                let (sc, sv): (Vec<i32>, Vec<f64>) = seg.into_iter().unzip();
                let h = pattern_hash(&sc, &sv);
                let found = table
                    .get(&h)
                    .and_then(|ids| ids.iter().rev().find(|&&id| pattern_eq(&pcols[id], &pvals[id], &sc, &sv)).copied());
                // a pattern seen before would count toward part 2 (not ported, see above)
                if found.is_none() {
                    let len = sc.len() as f64;
                    self.work = ((len + 5.0) + len) * mu + self.work;
                    table.entry(h).or_default().push(pcols.len());
                    pcols.push(sc);
                    pvals.push(sv);
                }
            }
        }
        self.work = (rows.len() as f64 + rows.len() as f64) * mu + self.work;
        0
    }
}
