//! Pseudocost table A (`M+0x3250`): per column and row a down and an up count and sum (docs/m2/mip_pseudocost_rust.md).
//! Comparisons keep the original float comparisons, NaN included; no `f64::max`.

/// Direction 0 is down (branch `'<'`), 1 is up.
#[derive(Clone, Debug, PartialEq)]
pub struct Pseudocost {
    pub cnt: [Vec<i32>; 2],
    pub sum: [Vec<f64>; 2],
}

/// One processed node as read by the node update (`5fb8ec`) and the decay gate (`5fe9cd`).
pub struct NodeLp {
    pub has_parent: bool,
    /// Node status `I+0x28`.
    pub status: i32,
    pub depth: i32,
    /// Branching variable, negative when none.
    pub j: i32,
    /// Branch direction is `'<'`.
    pub down: bool,
    pub continuous: bool,
    /// Branching value.
    pub x: f64,
    /// Node LP objective.
    pub lp: f64,
    /// Parent LP objective `I+0x78`.
    pub p: f64,
}

/// LP result fields the decay reads. `alt` is set only when the result has both its `+0x18` and `+0x28` parts.
pub struct DecayLp<'a> {
    /// `L+0x38`.
    pub status: i32,
    /// `L+0x58`.
    pub k: i32,
    /// `L+0x54`.
    pub s: i32,
    /// `L+0x28`.
    pub alt: Option<&'a [f64]>,
    /// `L+0x70`.
    pub vec: Option<&'a [f64]>,
}

/// Depth weight of a node update.
pub fn node_weight(depth: i32, cutoff: f64) -> f64 {
    let e = if 5e99 > cutoff { 0.1 } else { 0.2 };
    (0.8 / (depth as f64 + 1.0)).powf(e)
}

/// `5fe9cd` gate in front of the decay; `n.status` is read after the cutoff check set it to 5.
pub fn decay_fires(n: &NodeLp) -> bool {
    if !n.has_parent || n.j < 0 || n.status == 9 || -1e100 >= n.p {
        return false;
    }
    n.status & !1 == 4 || (((n.status - 1) as u32) < 2 && n.lp - n.p > 1e-10)
}

impl Pseudocost {
    /// Fresh table of `n` entries (columns and rows).
    pub fn new(n: usize) -> Self {
        Self { cnt: [vec![0; n], vec![0; n]], sum: [vec![0.0; n], vec![0.0; n]] }
    }

    /// `6ec3da` seed path: column j takes the average of `src[map[j]]` per direction, count 0; stops at the first
    /// negative map entry. `map` has one entry per column.
    pub fn seeded(n: usize, src: &Pseudocost, map: &[i32]) -> Self {
        let mut t = Self::new(n);
        for (j, &m) in map.iter().enumerate() {
            if m < 0 {
                break;
            }
            for d in 0..2 {
                let c = src.cnt[d][m as usize];
                if c != 0 {
                    t.sum[d][j] = src.sum[d][m as usize] / c as f64;
                }
            }
        }
        t
    }

    /// `6edfcf`: gain `max(delta, 1e-10) / (|f| + 0.001)` into the direction of f; the first one replaces a smaller
    /// sum, later ones add.
    pub fn update(&mut self, f: f64, delta: f64, j: usize) {
        let g = if delta > 1e-10 { delta } else { 1e-10 };
        let (d, g) = if f > 0.0 {
            (1, g / (f + 0.001))
        } else if f < 0.0 {
            (0, g / (0.001 - f))
        } else {
            return;
        };
        let s = self.sum[d][j];
        self.sum[d][j] = if self.cnt[d][j] == 0 { if s > g { s } else { g } } else { g + s };
        self.cnt[d][j] += 1;
    }

    /// `5fb8ec` node update after the node LP; returns whether it called `update`.
    pub fn node_update(&mut self, n: &NodeLp, cutoff: f64) -> bool {
        if !n.has_parent || n.status == 7 || n.j < 0 {
            return false;
        }
        let w = node_weight(n.depth, cutoff);
        let f = match (n.down, n.continuous) {
            (true, true) => -0.5,
            (true, false) => n.x.floor() - n.x,
            (false, true) => 0.5,
            (false, false) => n.x.ceil() - n.x,
        };
        if -1e100 >= n.p {
            return false;
        }
        let delta = if n.status & !1 == 4 {
            if !(5e99 > cutoff) {
                return false;
            }
            w * (cutoff - n.p)
        } else if ((n.status - 1) as u32) < 2 {
            (n.lp - n.p) * w
        } else {
            return false;
        };
        self.update(f, delta, n.j as usize);
        true
    }

    /// `6ee1d3`: shrinks the sums of the ancestors' branches, (variable, down) from the parent up to the first ancestor
    /// without node information, whose LP vector entry passes the sign test.
    pub fn decay(&mut self, lp: Option<&DecayLp>, anc: impl IntoIterator<Item = (i32, bool)>) {
        let Some(l) = lp else { return };
        let v = if l.status == 6 || l.k >= 0 { l.vec } else { l.alt };
        let Some(v) = v else { return };
        let (mut w, s) = if l.k < 0 { (0.5, 1.0) } else { (1.0, l.s as f64) };
        for (j, down) in anc {
            if !(w > 0.001) {
                break;
            }
            w *= 0.5;
            if j < 0 {
                continue;
            }
            let j = j as usize;
            let s2 = if down { s } else { -s };
            if -1e-10 > s2 * v[j] {
                continue;
            }
            let d = usize::from(!down);
            self.sum[d][j] *= 1.0 - w / (self.cnt[d][j] as f64 + 1.0);
        }
    }

    /// `6ef22e`: per direction, every column of an orbit class takes the count and sum of the class column with the
    /// largest average (first column on ties). `cls` has one class index per column.
    pub fn symmetry(&mut self, cls: &[i32]) {
        let nc = cls.len();
        let mut best = vec![0.0; nc];
        let mut rep = vec![-1i32; nc];
        for d in 0..2 {
            rep.fill(-1);
            for (j, &c) in cls.iter().enumerate() {
                let cn = self.cnt[d][j];
                if cn > 0 {
                    let c = c as usize;
                    let v = self.sum[d][j] / cn as f64;
                    if rep[c] == -1 || best[c] < v {
                        best[c] = v;
                        rep[c] = j as i32;
                    }
                }
            }
            for (j, &c) in cls.iter().enumerate() {
                let r = rep[c as usize];
                if r >= 0 {
                    self.cnt[d][j] = self.cnt[d][r as usize];
                    self.sum[d][j] = self.sum[d][r as usize];
                }
            }
        }
    }
}
