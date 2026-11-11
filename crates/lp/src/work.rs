//! Dual simplex work counter (`06268b` param_2, docs/m1/m1_02p_work.md). Each original store is `w = amount + w`
//! with the amount rounded first (no FMA); where the original skips a store it keeps the partial sum in a register,
//! which rounds the same way (`carry`). Sites are the original trap pcs (instruction after the store) so a dump
//! lines up with the M1-02p watchpoint capture. shortcut: only the factor, FT update, refactor-return, start and
//! post-update (`1bae9c`, `1b7106`, `1ca57`, `16b0a`) sites are ported (M1-02q, M1-02r, M1-02s); the iteration
//! kernels and LU solves follow in M1-02v, M1-02t and M1-02u, so the absolute value is not the original's yet and
//! nothing decides on it.

/// spx+0x148 (double precision; 1.95 in quad); LU+0x1f8 is its copy in the factor.
pub const X148: f64 = 1.3;

/// `RSOLVER_WORK` log entry: (site, value after, amounts folded since the previous store, k).
pub type Store = (u32, f64, Vec<f64>, usize);

pub struct Work {
    pub w: f64,
    /// spx+0x150: work at the start of the last factorization (`036ade` C:531–539).
    pub w150: f64,
    /// spx+0x158: moving average of the factorization's own work (`036ade` C:5753–5772).
    pub w158: f64,
    carried: Vec<f64>,
    /// Iteration the stores belong to (the original's `8b473` count k), set by the caller.
    pub k: usize,
    pub log: Option<Vec<Store>>,
    /// `RSOLVER_WORK_SEED`: work at each refactor entry, taken from an original capture (verification input).
    seed: std::collections::VecDeque<f64>,
}

impl Work {
    pub fn new() -> Work {
        let seed = std::env::var("RSOLVER_WORK_SEED")
            .ok()
            .map(|p| std::fs::read_to_string(p).unwrap_or_default())
            .unwrap_or_default()
            .split_whitespace()
            .map(|h| f64::from_bits(u64::from_str_radix(h, 16).expect("RSOLVER_WORK_SEED: hex bits per line")))
            .collect();
        Work { w: 0.0, w150: 0.0, w158: 0.0, carried: Vec::new(), k: 0, log: std::env::var("RSOLVER_WORK").ok().map(|_| Vec::new()), seed }
    }

    /// The original keeps this partial sum in a register; a later `add` stores it.
    pub fn carry(&mut self, amount: f64) {
        self.w += amount; // = amount + w: IEEE addition commutes
        if self.log.is_some() {
            self.carried.push(amount);
        }
    }

    pub fn add(&mut self, site: u32, amount: f64) {
        self.carry(amount);
        if let Some(l) = &mut self.log {
            l.push((site, self.w, std::mem::take(&mut self.carried), self.k));
        }
    }

    /// Refactor entry (`05b257`): the verification seed replaces the work value when one is given.
    pub fn enter_refactor(&mut self) {
        if let Some(v) = self.seed.pop_front() {
            self.w = v;
        }
    }

    /// One `036ade` call: spx+0x150, its stores, then spx+0x158. shortcut: SIMPLEXENDFAC (env+0x2454) ≠ 0 and
    /// env+0x2728 < 2 are assumed (defaults); otherwise the original leaves 0x158 alone.
    pub fn factor(&mut self, amounts: &[(u32, f64)]) {
        self.w150 = self.w;
        for &(site, a) in amounts {
            self.add(site, a);
        }
        let d = self.w - self.w150;
        self.w158 = if self.w158 <= 0.0 { d } else { (d + self.w158) * 0.5 };
    }
}

impl Default for Work {
    fn default() -> Work {
        Work::new()
    }
}
