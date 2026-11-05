//! PDHG evaluation `354824`, restart test `350614`, restart and primal weight `350723`, power iteration `351b1f`
//! (docs/m3_pdhg.md §6). One thread: every cross-thread reduction is `0.0 + v` (sum) or `maxsd(0.0, v)` (max).
use super::kern::{dot, red8, Mat, Vecs};
use super::scale::Scaled;

const INF: f64 = 1e30;

#[inline]
fn maxsd(a: f64, b: f64) -> f64 {
    if a > b { a } else { b }
}

/// `354f42`: Σ (a − b)² with b deciding the peel.
fn dist2(a: &[f64], b: &[f64], peel_b: usize) -> f64 {
    0.0 + red8(a.len(), peel_b, |k| {
        let d = a[k] - b[k];
        d * d
    })
}

/// The scalar part of the PDHG state P (offsets in docs/m3_pdhg.md §6) plus its vectors.
#[derive(Clone, Debug, Default)]
pub struct State {
    pub it: i64,      // P[0]   all iterations incl. power iterations (PDHG_ITRCNT, PDHGIterCount)
    pub it_pdhg: i64, // P[1]   PDHG iterations (evaluation cadence, restart test)
    pub k: i64,       // P[2]   iterations since the last restart
    pub pw_it: i64,   // P[3]   accumulated power iterations
    pub pobj: f64,    // +0x38
    pub dobj: f64,    // +0x40
    pub compl: f64,   // +0x48
    pub rp_inf: f64,  // +0x50
    pub rd_inf: f64,  // +0x58
    pub rp2: f64,     // +0x60
    pub rd2: f64,     // +0x68
    pub rp_rel: f64,  // +0x70
    pub rd_rel: f64,  // +0x78
    pub gap: f64,     // +0x80
    pub eta: f64,     // +0xe0
    pub omega: f64,   // +0xe8
    pub omega0: f64,  // +0xf0 "best balance" ω; never replaced on a fresh solve (+0xf8 stays 0)
    pub integ: f64,   // +0x100
    pub err: f64,     // +0x108
    pub g: f64,       // +0x110
    pub g_prev: f64,  // +0x118
    pub g0: f64,      // +0x120
    pub cnt: i64,     // +0x168 coordinates with a nontrivial projection
    pub bnorm: f64,   // +0x170 ‖b⊙wr‖₂
    pub cnorm: f64,   // +0x178 ‖c⊙wc‖₂
    pub eps: f64,     // +0x188 power-iteration residual
    pub tol: f64,     // +0x190 its tolerance
    pub v: Vecs,
    pub wc: Vec<f64>, // +0x158 1/s
    pub wr: Vec<f64>, // +0x160 1/r
    pub pv: Vec<f64>, // +0x180 power-iteration vector (A Aᵀ u of the last step; start: RNG)
}

impl State {
    /// `354824`: objectives, residuals (unscaled through wc, wr), gap. `first`: the pass at P[0] = 0 and the final
    /// call measure the dual residual by reduced costs against the bounds instead of λ = (x̂ − t)/τ.
    pub fn eval(&mut self, d: &Scaled, first: bool, work: &mut f64) {
        let (a, c, b) = (&d.a, &d.c, &d.b);
        let v = &mut self.v;
        a.ax(&v.xhat, &mut v.ax);
        *work += a.nnz() as f64 * 3.0;
        a.aty(&v.yhat, &mut v.aty);
        *work += a.nnz() as f64 * 3.0;
        let (lb, ub) = (&v.lb, &v.ub);
        let mut acc = 0.0;
        for j in 0..a.n {
            let dj = c[j] - v.aty[j];
            let (lf, uf) = (lb[j] > -INF, ub[j] < INF);
            if lf && dj > 1e-10 {
                acc += (v.xhat[j] - lb[j]) * dj;
            } else if uf && dj < -1e-10 {
                acc += (v.xhat[j] - ub[j]) * dj;
            }
        }
        for i in 0..a.m {
            let s = b[i] - v.ax[i];
            if d.sense[i] != b'=' && s > 1e-10 {
                acc += (-s) * v.yhat[i];
            }
        }
        let mut compl = 0.0 + acc;
        if self.cnt > 0 {
            compl /= self.cnt as f64;
        }
        let compl = maxsd(compl, 0.0);
        let (mut mx, mut sq) = (0.0f64, 0.0f64);
        for i in 0..a.m {
            let r = (b[i] - v.ax[i]) * self.wr[i];
            let mut t = maxsd(0.0, -r);
            if d.sense[i] == b'=' {
                t = maxsd(t, r);
            }
            mx = maxsd(mx, t);
            sq += t * t;
        }
        let rp_inf = maxsd(maxsd(0.0, mx), 0.0);
        let rp2 = maxsd((0.0 + sq).sqrt(), 0.0);
        let pobj = dot(&v.xhat, c, d.peel_c) / d.objscale + d.objcon;
        let (mut mx, mut sq, mut acc) = (0.0f64, 0.0f64, 0.0f64);
        if !first {
            for j in 0..a.n {
                let r = ((c[j] - v.aty[j]) - v.xdel[j]) * self.wc[j];
                sq += r * r;
                mx = maxsd(mx, r.abs());
            }
            for i in 0..a.m {
                acc += v.yhat[i] * b[i];
            }
            for j in 0..a.n {
                let lam = v.xdel[j];
                if lam > 0.0 && lb[j] > -INF {
                    acc += lam * lb[j];
                } else if lam < 0.0 && ub[j] < INF {
                    acc += ub[j] * lam;
                }
            }
        } else {
            for j in 0..a.n {
                let dj = (c[j] - v.aty[j]) * self.wc[j];
                let (lf, uf) = (lb[j] > -INF, ub[j] < INF);
                let t = match (lf, uf) {
                    (false, false) => maxsd(dj, -dj),
                    (true, true) => 0.0,
                    (false, true) => maxsd(0.0, dj),
                    (true, false) => maxsd(0.0, -dj),
                };
                sq += t * t;
                mx = maxsd(mx, t.abs());
            }
            for i in 0..a.m {
                acc += v.yhat[i] * b[i];
            }
            for j in 0..a.n {
                let dj = c[j] - v.aty[j];
                if dj > 0.0 && lb[j] > -INF {
                    acc += dj * lb[j];
                } else if dj < 0.0 && ub[j] < INF {
                    acc += ub[j] * dj;
                }
            }
        }
        let rd_inf = maxsd(0.0, mx);
        let rd2 = (0.0 + sq).sqrt();
        let dobj = (0.0 + acc) / d.objscale + d.objcon;
        self.pobj = pobj;
        self.dobj = dobj;
        self.compl = compl;
        self.rp_inf = rp_inf;
        self.rd_inf = rd_inf.abs();
        self.rp2 = rp2;
        self.rd2 = rd2.abs();
        self.rp_rel = rp2 / (self.bnorm + 1.0);
        self.rd_rel = rd2.abs() / (self.cnorm + 1.0);
        self.gap = (pobj - dobj).abs() / ((1.0 + pobj.abs()) + dobj.abs());
        *work += a.m as f64 * 8.0 + a.n as f64 * 14.0;
    }

    /// `350614`: 0 no restart, 1 forced at 100, 2 sufficient decay, 3 necessary decay and worse, 4 artificial,
    /// 5 g = 0.
    pub fn restart_code(&self) -> i32 {
        let k = self.it_pdhg;
        if k < 100 {
            0
        } else if k == 100 {
            1
        } else if self.g == 0.0 {
            5
        } else if 0.2 * self.g0 >= self.g {
            2
        } else if 0.5 * self.g0 >= self.g && self.g > self.g_prev {
            3
        } else if k as f64 * 0.36 <= self.k as f64 {
            4
        } else {
            0
        }
    }

    /// `350723`: PI update of the primal weight from the anchor distances, then restart at the evaluation point.
    pub fn restart(&mut self) {
        let v = &mut self.v;
        let dx = dist2(&v.xhat, &v.x0, 0).sqrt();
        let dy = dist2(&v.yhat, &v.y0, 0).sqrt();
        let ratio = if self.rp_rel == 0.0 { 1e100 } else { self.rd_rel / self.rp_rel };
        if dx > 1e-16 && dy > 1e-16 && 1e12 > dx && 1e12 > dy && ratio > 1e-8 && 1e8 > ratio {
            let (ly, lx, lw) = (dy.ln(), dx.ln(), self.omega.ln());
            let e = (ly - lx) - lw;
            let integ = 0.3 * self.integ + e;
            let arg = 0.99 * e + 0.01 * integ;
            self.integ = integ;
            self.omega *= arg.exp();
            self.err = e;
        } else {
            self.omega = self.omega0;
            self.integ = 0.0;
            self.err = 0.0;
        }
        // shortcut: the best-balance ω tracking (+0xf0/+0xf8) can never fire on a fresh solve (+0xf8 = 0 ≤ |log10|);
        // port it with warm starts that set +0xf8.
        self.k = 0;
        self.g_prev = 1e100;
        v.x0.copy_from_slice(&v.xhat);
        v.x.copy_from_slice(&v.xhat);
        v.y0.copy_from_slice(&v.yhat);
        v.y.copy_from_slice(&v.yhat);
    }

    /// `351b1f`: power iteration on A Aᵀ (row space) for ‖A‖₂, then η = ((1 − ε) − ε)/‖A‖ (0.01/‖A‖ if ε ≥ 0.5).
    /// `chunk_done` runs after every chunk of ≤ 100 steps (callback, termination test, log) and returns false to stop.
    pub fn power(&mut self, a: &Mat, iter_limit: f64, work: &mut f64, chunk_done: &mut dyn FnMut(&Self, f64) -> bool) {
        let rem = iter_limit - self.it as f64;
        let mut budget: i64 = if self.pw_it == 0 {
            250
        } else if self.g == 0.0 {
            if self.tol >= self.eps {
                self.tol = maxsd(0.1 * self.tol, 1e-16);
            }
            1000
        } else {
            (self.it as f64 * 0.1 - self.pw_it as f64) as i64
        };
        if !(self.eps > self.tol && budget > 100 && a.m != 0 && a.n != 0) {
            return;
        }
        let (m, n) = (a.m, a.n);
        let (mut anorm, mut eps) = (0.0f64, 1e100f64);
        if (budget as f64) >= rem {
            budget = rem as i64;
        }
        let mut done = 0i64;
        let (mut u, mut w) = (vec![0.0f64; m], vec![0.0f64; n]);
        while budget > 0 && done < budget {
            let chunk = (budget - done).min(100);
            let (mut eps_c, mut lam, mut it) = (1e100f64, 1e100f64, 0i64);
            while it < chunk {
                if self.tol > eps_c {
                    break;
                }
                u.copy_from_slice(&self.pv);
                let nrm = dot(&u, &u, 0).sqrt();
                for x in u.iter_mut() {
                    *x /= nrm;
                }
                a.aty(&u, &mut w);
                *work += a.nnz() as f64 * 3.0;
                a.ax(&w, &mut self.pv);
                *work += a.nnz() as f64 * 3.0;
                lam = dot(&self.pv, &u, 0);
                for (x, p) in u.iter_mut().zip(&self.pv) {
                    *x = p - *x * lam;
                }
                eps_c = dot(&u, &u, 0).sqrt();
                it += 1;
            }
            *work += m as f64 + (m as f64 * 8.0) * it as f64;
            anorm = lam.sqrt();
            eps = eps_c;
            let k = if it != 0 { it } else { 1 };
            done += k;
            self.pw_it += k;
            self.it += k;
            let go = chunk_done(self, *work);
            if !go || self.tol >= eps {
                break;
            }
        }
        if anorm > 0.0 && 1e100 > eps {
            self.eta = if eps >= 0.5 { 0.01 / anorm } else { ((1.0 - eps) - eps) / anorm };
            self.eps = eps;
        }
        *work += (m + n) as f64 * 2.0;
    }
}
