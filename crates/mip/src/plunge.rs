//! Plunge-list selection of `6e8eb3` (docs/m2/mip_plunge_rust.md): the rule number from the heap pop count k, the
//! local RNG slot seeded from the entry k and `W+0x28`, the rule bodies that pick an entry, and the tail cleanup that
//! keeps only the siblings under the same ancestor. Float comparisons are the original `comisd`/`minsd`/`maxsd` selects.
use rsolver_lp::common::Rng;

/// One plunge-list entry (`W+0x3fb0`): node, est `I+0x20`, depth `I+0x2c`, excluded `I+0x41`, bound `I+0x18`.
#[derive(Clone, Copy, Debug)]
pub struct Item<T> {
    pub node: T,
    pub est: f64,
    pub depth: i32,
    pub excluded: bool,
    pub bound: f64,
}

/// Inputs of the rule step.
pub struct RuleIn {
    /// `W+0x30` after the heap pops.
    pub k: i32,
    /// `Mx+0x40` (objective gap, -1e100 when it is constant).
    pub gap: f64,
    /// lp0 columns are all integer.
    pub all_int: bool,
    /// r8b: `env+0x2e2c == 0`, `M+0x3560 != 0` and the entry k below 10.
    pub hint: bool,
    /// `M+0x848 != 0`.
    pub m848: bool,
    /// BranchDir `env+0x27e0`.
    pub branch_dir: i32,
    /// `Mx+0x808 != 0`.
    pub mx808: bool,
    /// `5e70c1(M)`.
    pub s: i32,
    /// MIPFocus `env+0x2710`.
    pub focus: i32,
}

/// Inputs of the tail.
pub struct TailIn {
    /// `W+0x30` at the rule step.
    pub k: i32,
    /// `M+0x2c`.
    pub m2c: i32,
    pub focus: i32,
    /// `T+0x138`.
    pub t138: i64,
    /// `T+0x120` (global heap size).
    pub t120: i64,
}

/// Values the tail reads only when the selected node has an ancestor at depth d'.
pub struct Thr {
    /// `5f9a41(M)`.
    pub f9a41: i32,
    /// `5e7796(M)`: objective of the first pool solution.
    pub pool: f64,
    /// `5e8239(M, 2)`.
    pub v8239: f64,
    /// `5e7204(M)`: cutoff.
    pub cut: f64,
}

pub struct Tail<T> {
    pub sel: Item<T>,
    /// d' = depth - delta.
    pub d: i32,
    pub anc: Option<T>,
    /// Threshold before the excluded-node override; None when the ancestor is null (no cleanup).
    pub thr: Option<f64>,
}

fn minsd(a: f64, b: f64) -> f64 {
    if a < b {
        a
    } else {
        b
    }
}

fn maxsd(a: f64, b: f64) -> f64 {
    if a > b {
        a
    } else {
        b
    }
}

/// Local RNG slot `0x70(%rsp)` at entry: `cvttsd2si` of k_entry + `W+0x28`, with the 2^63 split.
pub fn seed(k_entry: i32, w28: f64) -> Rng {
    const TWO63: f64 = 9.223372036854776e18;
    let x = k_entry as f64 + w28;
    Rng(if x < TWO63 { x as i64 as u64 } else { (x - TWO63) as i64 as u64 | 1 << 63 })
}

/// Rule number (asm 0x5b3326-0x5b3553). `f6efd` is `5e6efd(M)`, asked only on the k % 7 clause; the draw uses `rng`.
pub fn rule(p: &RuleIn, f6efd: impl FnOnce() -> bool, rng: &mut Rng) -> u8 {
    let k = p.k;
    let r = if p.gap == -1e100 && p.all_int {
        11
    } else if p.hint {
        3
    } else if p.m848 {
        6
    } else if p.branch_dir != 0 {
        9
    } else if k <= 8 {
        if p.mx808 { 1 } else { 6 }
    } else if k <= 16 || p.s == 0 || k % 11 == 0 {
        2
    } else if k % 7 == 0 {
        if f6efd() { 4 } else { 2 }
    } else if k % 13 == 0 {
        5
    } else if k % 19 == 0 {
        if p.mx808 { 1 } else { 6 }
    } else if k % 43 == 9 {
        7
    } else if k % 2 != 0 {
        if rng.next_f64() < 0.9 { 3 } else { 12 }
    } else {
        3
    };
    if p.focus == 4 { 5 } else { r }
}

/// `5fb13c`: the ancestor of `node` (at `depth`) at depth d, walking `parent` depth - d times; null stops the walk.
pub fn ancestor<T: Copy>(node: T, depth: i32, d: i32, parent: &mut impl FnMut(T) -> Option<T>) -> Option<T> {
    let steps = depth.wrapping_sub(d);
    if (d | steps) < 0 {
        return None;
    }
    let mut a = node;
    for _ in 0..steps {
        a = parent(a)?;
    }
    Some(a)
}

/// minEst and the rule body (asm 0x5b3541-0x5b44a0): adds the work and returns the picked index, or None for the
/// rules whose score is not ported (4, 7, 9, 11; their work is added).
pub fn pick<T: Copy + PartialEq>(
    rule: u8,
    items: &[Item<T>],
    rng: &mut Rng,
    parent: &mut impl FnMut(T) -> Option<T>,
    work: &mut f64,
) -> Option<usize> {
    let n = items.len();
    let nf = n as f64;
    // minEst over every entry, excluded ones included, two at a time: minsd, then the second only if not above
    let mut m = 1e100;
    let mut j = 0;
    while j + 1 < n {
        m = minsd(m, items[j].est);
        let b = items[j + 1].est;
        if !(b > m) {
            m = b;
        }
        j += 2;
    }
    if j < n {
        m = minsd(m, items[j].est);
    }
    *work = nf * 3.0 + *work;
    let live = || items.iter().enumerate().filter(|(_, it)| !it.excluded);
    let mut idx = None;
    let mut best = -1e100;
    match rule {
        2 => {
            for (j, it) in live() {
                let v = -it.est;
                if v > best {
                    idx = Some(j);
                }
                best = maxsd(v, best);
            }
            *work = nf * 3.0 + *work;
        }
        3 | 12 => {
            for (j, it) in live() {
                let q = (it.est - m) / 1e8;
                let dep = it.depth as f64;
                let sc = if rule == 12 { dep + q } else { dep - q };
                if sc > best {
                    idx = Some(j);
                }
                best = maxsd(sc, best);
            }
            *work = nf * 4.0 + *work;
        }
        1 | 6 => {
            // the asm asks 5fb13c at every new maximum and keeps the last; it is pure, so ask once for the first deepest
            let (mut maxd, mut top) = (0, None);
            for (_, it) in live() {
                if it.depth > maxd {
                    maxd = it.depth;
                    top = Some(it);
                }
            }
            let d = maxd - 2;
            let top_anc = top.and_then(|it| ancestor(it.node, it.depth, d, parent));
            *work = nf * 3.0 + *work;
            for (j, it) in live() {
                let sc = if rule == 6 { -it.est } else { (-it.depth) as f64 - (it.est - m) / 1e8 };
                if sc > best && ancestor(it.node, it.depth, d, parent) == top_anc {
                    idx = Some(j);
                    best = sc;
                }
            }
            *work = nf * if rule == 6 { 3.0 } else { 4.0 } + *work;
        }
        5 => {
            let s = rng.next_int() % n as i32;
            idx = (0..n).map(|i| (s as usize + i) % n).find(|&j| !items[j].excluded);
        }
        8 | 10 => {}
        4 => {
            *work = nf * 7.0 + *work;
            return None;
        }
        7 | 9 => {
            *work = nf * 5.0 + *work;
            return None;
        }
        11 => {
            *work = nf * 3.0 + *work;
            return None;
        }
        _ => panic!("plunge rule {rule}"),
    }
    Some(idx.unwrap_or(0))
}

/// Tail (asm 0x5b44a9-0x5b49d6): take `list[idx]` out (the last entry fills its slot), then, when the selected node
/// has an ancestor at d', keep in order only the entries under the same ancestor with bound not above the threshold.
/// Each other entry goes to `removed(node, to_heap)`: to_heap = stage 4 + global heap insert, else stage 0 (dropped).
/// shortcut: hint sibling swap (r8b) and the parallel worker heap (5e2919 != 0) are not ported; single thread, no hint.
pub fn tail<T: Copy + PartialEq>(
    list: &mut Vec<Item<T>>,
    idx: usize,
    p: &TailIn,
    thr: impl FnOnce() -> Thr,
    parent: &mut impl FnMut(T) -> Option<T>,
    mut removed: impl FnMut(T, bool),
    work: &mut f64,
) -> Tail<T> {
    let sel = list.swap_remove(idx);
    let delta = if p.k <= p.m2c {
        2
    } else if p.focus == 4 {
        3
    } else if p.t138 > 0 && p.t120 >= 10000 && p.k >= 1000 {
        10
    } else if p.k % 23 == 7 {
        6
    } else {
        let x = (sel.depth as f64 * 0.2 + 1.0).ln() / 2f64.ln();
        (x as i32).clamp(3, 10)
    };
    let d = sel.depth - delta;
    let anc = ancestor(sel.node, sel.depth, d, parent);
    if anc.is_none() {
        return Tail { sel, d, anc, thr: None };
    }
    let t = thr();
    let mut thr0 = 1e100;
    if t.f9a41 != 0 && t.pool > t.v8239 - 1e-10 {
        thr0 = minsd(thr0, t.pool);
    }
    let lim = if sel.excluded { -1e100 } else { thr0 };
    let n = list.len();
    let mut kept = 0;
    for j in 0..n {
        let it = list[j];
        let same = ancestor(it.node, it.depth, d, parent) == anc;
        if same && !(it.bound > lim) {
            list[kept] = it;
            kept += 1;
        } else {
            removed(it.node, t.cut > it.bound);
        }
    }
    list.truncate(kept);
    *work = n as f64 * 4.0 + *work;
    Tail { sel, d, anc, thr: Some(thr0) }
}
