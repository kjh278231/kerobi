//! 05 §2 Barrier iterations for LP (`2bf1ea` L455–1030), port of tools/proto/bar_ipm.py (docs/m3_ipm.md §2–§5):
//! starting point `2d0663`/`2d252f`/`2d8838`, residuals `2d4977`, directions `2f59f1`/`2ee6c5` + refinement
//! `2f9699`, step `2f15ba`/`2f4fe9`, correctors `2d64b4`, corrector count from the work timers, update `2f2e95`,
//! termination `b2aecf`. Sums follow Gurobi's SSE2 lane order and x87 Kahan sums (docs/m3_chol.md §4), so the
//! iterates are bit-equal to Gurobi on the oracle instances. Restart (`2bf1ea` L956–994, perold and pilot4) is ported.
//! shortcut: no homogeneous switch (`2d35d7`, `2f8f0e`), no status-13 stall rules and no best-iterate
//! choice (`2c4689`); the last iterate is returned. None of them fires on the 64 bench LPs (docs/m3_ipm.md §8).
#![allow(clippy::needless_range_loop)] // index loops over parallel arrays, as in the original
use super::chol::{vsum, Factor};
use super::f80::{kahan_dot, F80};
use super::ordering::Symbolic;
use super::setup::{dot, Lp};
use super::{BarResult, IterInfo, Params};

struct State {
    x: Vec<f64>,
    z: Vec<f64>,
    y: Vec<f64>,
    w: Vec<f64>,
    s: Vec<f64>,
    tau: f64,
    kappa: f64,
}

/// `2f9515`: Σ a·b with Gurobi's 8-lane order; `peel` = 1 when b starts at an 8-mod-16 address (B arrays are 16-byte
/// aligned, so it is the parity of the start offset).
fn dot8(a: &[f64], b: &[f64], peel: usize) -> f64 {
    let p: Vec<f64> = a.iter().zip(b).map(|(x, y)| x * y).collect();
    vsum(0.0, &p, peel, 8)
}

/// Debug aid: with BARDUMP=<file>, append one JSON line (iterates, steps; doubles as bit patterns) for
/// tools/proto/bar_cmp.py / bar_cmp_step.py, which compare them bit for bit with the oracle.
fn dump(line: impl FnOnce() -> String) {
    if let Ok(path) = std::env::var("BARDUMP") {
        use std::io::Write;
        let mut f = std::fs::OpenOptions::new().create(true).append(true).open(path).expect("BARDUMP file");
        writeln!(f, "{}", line()).expect("BARDUMP write");
    }
}

fn hx(v: &[f64]) -> String {
    format!("[{}]", v.iter().map(|x| format!("\"{:016x}\"", x.to_bits())).collect::<Vec<_>>().join(","))
}

/// `2f28c2` slack/box sums: 4 lanes (lane 0 seeded by `acc`) over 8-element blocks after a `peel`, reduced
/// (l0+l2)+(l1+l3), the tail sequentially; sequential below peel + 8 terms.
fn vsum4(mut acc: f64, p: &[f64], peel: usize) -> f64 {
    let n = p.len();
    if n < peel + 8 {
        return p.iter().fold(acc, |a, &v| a + v);
    }
    for &v in &p[..peel] {
        acc += v;
    }
    let nv = n - (n - peel) % 8;
    let mut l = [acc, 0.0, 0.0, 0.0];
    for k in peel..nv {
        l[(k - peel) % 4] += p[k];
    }
    p[nv..].iter().fold((l[0] + l[2]) + (l[1] + l[3]), |a, &v| a + v)
}

/// `2d4977` output: residual vectors, norms, objectives, compl and μ.
struct Res {
    rp: Vec<f64>,
    rd: Vec<f64>,
    ru: Vec<f64>,
    rp2: f64,
    rd2: f64,
    ru2: f64,
    rp_rel: f64,
    rd_rel: f64,
    compl: f64,
    mu: f64,
    ctx: f64,
    pobj: f64,
    dobj: f64,
}

/// Search direction; `res` = (B+0x770, B+0x780) solve residual norms from the refinement.
#[derive(Clone)]
struct Dir {
    dx: Vec<f64>,
    dy: Vec<f64>,
    dz: Vec<f64>,
    dw: Vec<f64>,
    ds: Vec<f64>,
    res: [f64; 2],
}

fn residuals(lp: &Lp, st: &State) -> Res {
    let (m, n, tau) = (lp.m, lp.n, st.tau);
    let ax = lp.ax(&st.x);
    let rp: Vec<f64> = (0..m).map(|i| lp.b[i] * tau - ax[i]).collect();
    let aty = lp.aty(&st.y);
    // oracle: columns in the 8-wide SIMD body cτ − ((Aᵀy + z) − s), in the scalar tail ((cτ − Aᵀy) − z) + s
    // (fffff800 cols 848–853, 25fv47 col 1570); slacks −y − z
    let nv = if n >= 8 { n - n % 8 } else { 0 };
    let rd: Vec<f64> = (0..m + n)
        .map(|j| match j {
            _ if j < nv => lp.c[j] * tau - ((aty[j] + st.z[j]) - st.s[j]),
            _ if j < n => ((lp.c[j] * tau - aty[j]) - st.z[j]) + st.s[j],
            _ => -aty[j] - st.z[j],
        })
        .collect();
    let ru: Vec<f64> = (0..n).map(|j| if lp.up[j] { lp.ub[j] * tau - st.x[j] - st.w[j] } else { 0.0 }).collect();
    let amax = |v: &[f64]| v.iter().fold(0.0f64, |a, x| a.max(x.abs()));
    let mut dinf = 0.0f64;
    for j in 0..n {
        let dc = rd[j];
        dinf = dinf.max(if lp.lo[j] && dc < 0.0 { (-(dc + st.z[j])).max(0.0) } else { dc.abs() });
    }
    for i in 0..m {
        if !lp.eq[i] {
            dinf = dinf.max(st.y[i]);
        }
    }
    // 2d4977 asm 0x2ee7e4: x87 Kahan sums (3439d5) of x·z over all m+n and of w·s over the finite-ub column
    // range B+0x630..0x634, each rounded to double when combined: τκ + dbl(S_ws + dbl(S_xz))
    let (lo, hi) = lp.ub_range();
    let sxz = F80::from_f64(kahan_dot(&st.x, &st.z).to_f64());
    let xz = (kahan_dot(&st.w[lo..hi], &st.s[lo..hi]) + sxz).to_f64();
    let compl = (tau * st.kappa + xz) / lp.ncomp as f64;
    // 2d4977 L1333-1352: 2f9515 dots; the s·u dot runs over the finite-ub range
    let ctx = (dot8(&st.x[..n], &lp.c, 0) + 0.0) + lp.cshift * tau;
    let su = dot8(&st.s[lo..hi], &lp.ub[lo..hi], lo & 1);
    let bty = ((0.0 + (dot8(&st.y, &lp.b, 0) - su)) - 0.0) + tau * lp.cshift;
    Res {
        rp2: dot8(&rp, &rp, 0).sqrt() / tau,
        rd2: dot8(&rd, &rd, 0).sqrt() / tau,
        ru2: dot8(&ru[lo..hi], &ru[lo..hi], lo & 1).sqrt() / tau, // oracle: over the finite-ub range only
        rp_rel: amax(&rp).max(amax(&ru)) / tau,
        rd_rel: dinf / tau,
        compl,
        mu: compl / tau / tau,
        ctx,
        pobj: ctx / lp.objscale / tau + lp.objcon,
        dobj: bty / lp.objscale / tau + lp.objcon,
        rp,
        rd,
        ru,
    }
}

/// B+0x800: z/x (T&1) + s/w (T&2), 0 for free columns and '=' slacks.
fn theta_inv(lp: &Lp, st: &State) -> Vec<f64> {
    let mut d = vec![0.0; lp.m + lp.n];
    for j in 0..lp.m + lp.n {
        if lp.lo[j] {
            d[j] = st.z[j] / st.x[j];
        }
        if j < lp.n && lp.up[j] {
            d[j] += st.s[j] / st.w[j];
        }
    }
    d
}

/// `2d4977` tail: θ = min(1/d, 1e12); free columns get the BARFREEPERT perturbation; '=' slacks 0.
fn theta(lp: &Lp, st: &State, pobj: f64) -> Vec<f64> {
    let n = lp.n;
    let mut th = theta_inv(lp, st);
    for (j, t) in th.iter_mut().enumerate() {
        *t = if *t > 0.0 {
            (1.0 / *t).min(1e12)
        } else if j < n {
            let xm = st.x[j].abs().max(1e-10);
            1.0 / ((pobj.abs() + 1.0) * 1e-8 / (xm * xm)).clamp(1e-12, 1e-6)
        } else {
            0.0
        };
        if j >= n && lp.eq[j - n] {
            *t = 0.0;
        }
    }
    th
}

/// `2d8838` (and `2d4977` when μ stalls): move z toward μ/x and s toward μ/w, by at most what r_d allows.
fn recenter(lp: &Lp, st: &mut State, rd: &mut [f64], mu: f64) {
    let step = |t: f64, cap: f64| if t > 0.0 { t.min(cap.max(1e-20)) } else { t.max(cap.min(-1e-20)) };
    for j in 0..lp.n {
        if lp.lo[j] {
            let t = step(mu / st.x[j] - st.z[j], rd[j]);
            st.z[j] += t;
            rd[j] -= t;
        }
        if lp.up[j] {
            let t = step(mu / st.w[j] - st.s[j], -rd[j]);
            st.s[j] += t;
            rd[j] += t;
        }
    }
    for i in lp.n..lp.n + lp.m {
        if lp.t[i] == 1 {
            let t = step(mu / st.x[i] - st.z[i], rd[i]);
            st.z[i] += t;
            rd[i] -= t;
        }
    }
}

/// `2d252f`: Mehrotra-type shifts (δx, δz) of the starting point; sums run over the pairs in column order (lower,
/// upper), then the slacks. `floor` = B+0x30 (0 → 0.1 for LP).
fn shifts(lp: &Lp, st: &State, floor: f64) -> (f64, f64) {
    let (n, mn) = (lp.n, lp.m + lp.n);
    let mut pairs = vec![]; // (x or w, z or s)
    for j in 0..mn {
        if lp.lo[j] {
            pairs.push((st.x[j], st.z[j]));
        }
        if j < n && lp.up[j] {
            pairs.push((st.w[j], st.s[j]));
        }
    }
    let mx = pairs.iter().fold(1e100f64, |a, p| a.min(p.0));
    let mz = pairs.iter().fold(1e100f64, |a, p| a.min(p.1));
    let (dx, dz) = ((mx * -1.5).max(1e-4), (mz * -1.5).max(1e-4));
    let (mut sx, mut sz, mut sxz) = (0.0, 0.0, 0.0);
    for &(x, z) in &pairs {
        sx += x + dx;
        sz += z + dz;
        sxz += (x + dx) * (z + dz);
    }
    let mut ex = if sz > 0.0 { (sxz * 0.5) / sz + dx } else { 100.0 };
    let mut ez = if sx > 0.0 { dz + (sxz * 0.5) / sx } else { 100.0 };
    // asm 0x2ebe4e: both products are (‖·‖∞ · 1.0001) · shift
    let tx = lp.bnorminf * 1.0001 * ex;
    if tx.sqrt() >= ex {
        ex = tx.sqrt();
    }
    let tz = lp.cnorminf * 1.0001 * ez;
    if tz.sqrt() >= ez {
        ez = tz.sqrt();
    }
    let fl = if floor == 0.0 { 0.1 } else { floor };
    (ex.max(fl), ez.max(fl))
}

/// `2f15ba`: the step shrinks only when the current one crosses the bound of that entry (index order).
fn ratio(a0: f64, vd: impl Iterator<Item = (f64, f64)>) -> f64 {
    vd.fold(a0, |a, (v, d)| if a * d + v < 0.0 { -(v + -1e-30) / d - 1e-13 } else { a })
}

/// `2f15ba` / `2f4fe9` inner test: largest step (cap 1.001) along d (+ ω d1) for x, w (primal) and z, s (dual).
fn step_lengths(lp: &Lp, st: &State, d: &Dir, d1: Option<(&Dir, f64)>) -> (f64, f64) {
    let g = |v0: &[f64], f: fn(&Dir) -> &Vec<f64>, k: usize| d1.map_or(v0[k], |(e, om)| v0[k] + om * f(e)[k]);
    let lo = || (0..lp.m + lp.n).filter(|&j| lp.lo[j]);
    let up = || (0..lp.n).filter(|&j| lp.up[j]);
    let ap = ratio(1.001, lo().map(|j| (st.x[j], g(&d.dx, |e| &e.dx, j))));
    let ap = ratio(ap, up().map(|j| (st.w[j], g(&d.dw, |e| &e.dw, j)))).max(0.0);
    let ad = ratio(1.001, lo().map(|j| (st.z[j], g(&d.dz, |e| &e.dz, j))));
    let ad = ratio(ad, up().map(|j| (st.s[j], g(&d.ds, |e| &e.ds, j)))).max(0.0);
    (ap, ad)
}

/// `2bf1ea` L780–835: corrector count from the work ratio t(extra correctors) / t(factor + predictor + first
/// corrector + update).
fn adjust_k(k: i32, ratio: f64, avg: f64, base: i32) -> i32 {
    if k < 20 {
        if ratio < 0.25 {
            return k + 1;
        }
        if k < base {
            if ratio < 0.5 && avg < 0.1 {
                return k + 1;
            }
        } else if base < k && ratio > 0.25 {
            return k - 1;
        }
        if k > 1 && ratio > 0.25 && avg > 0.1 {
            return k - 1;
        }
        return k;
    }
    if k < base {
        if ratio < 0.5 && avg < 0.1 {
            return k + 1;
        }
        if ratio > 0.25 && avg > 0.1 {
            return k - 1;
        }
        return k;
    }
    if k <= base || ratio <= 0.25 {
        if ratio <= 0.25 {
            return k;
        }
        return if avg > 0.1 { k - 1 } else { k };
    }
    k - 1
}

struct Ipm<'a> {
    lp: &'a Lp,
    fac: Factor<'a>,
    st: State,
    /// F+0x178 = 1: free columns remain or F+0x100 < 1e-10 → `2f59f1` refines with `2f9699`.
    mode1: bool,
    /// B+0x800 (unperturbed θ⁻¹) and B+0x280 for the refinement; refinement solves per `2f59f1` call.
    dtrue: Vec<f64>,
    reg: f64,
    reflog: Vec<usize>,
    /// B+0x298 corrector count K, B+0x29c line-search grid N, B+0xf4, last three α_p·α_d.
    k: i32,
    ngrid: usize,
    f4: usize,
    ab: [f64; 3],
    /// B+0x30 shift floor (0 until a restart), B+0x38 = min(δx, δz) of the last start, B+0xf8 = iteration after the
    /// restart (0: none yet)
    floor: f64,
    shift_min: f64,
    f8: usize,
    /// (B+0x770, B+0x780) of the last combined direction
    last_res: [f64; 2],
    /// debug: (ω_p, ω_d) of the last line search
    last_om: (f64, f64),
}

impl<'a> Ipm<'a> {
    /// `2d0663` (BARSTART 0): least-squares x, two dual candidates, shifts, then the `2d8838` recentring.
    fn initial_point(&mut self) {
        let lp = self.lp;
        let (m, n) = (lp.m, lp.n);
        let mut th: Vec<f64> = (0..n).map(|j| match lp.t[j] {
            0 => 1e-6,
            3 => 0.5,
            _ => 1.0,
        }).collect();
        th.extend(lp.eq.iter().map(|&e| if e { 0.0 } else { 1.0 }));
        self.fac.factor(&th[..n], &th[n..]);
        let mut rx = vec![0.0; m + n];
        for j in 0..n {
            if lp.up[j] {
                rx[j] = -lp.ub[j];
            }
        }
        let st = &mut self.st;
        st.x = self.fac.solve(&rx, &lp.b).0;
        dump(|| format!("{{\"iter\":-1,\"x\":{}}}", hx(&st.x)));
        st.w = (0..n).map(|j| if lp.up[j] { lp.ub[j] - st.x[j] } else { 0.0 }).collect();
        // candidate 1: y = 0, c split between z (lower) and s (upper) by column type
        st.y = vec![0.0; m];
        st.z = vec![0.0; m + n];
        st.s = vec![0.0; n];
        for j in 0..n {
            let c = lp.c[j];
            match lp.t[j] {
                3 => {
                    st.z[j] = c * 0.5;
                    st.s[j] = -(c * 0.5);
                }
                1 => st.z[j] = c,
                2 => st.s[j] = -c,
                _ => {}
            }
        }
        let mut da = shifts(lp, st, self.floor);
        // candidate 2: (v, y) from the solve with rhs_x = [c; 0], z = −v (T&1), s = v (T&2)
        let mut cx = lp.c.clone();
        cx.resize(m + n, 0.0);
        let (v, yb) = self.fac.solve(&cx, &vec![0.0; m]);
        let zb = (0..m + n).map(|j| if lp.lo[j] { -v[j] } else { 0.0 }).collect();
        let sb = (0..n).map(|j| if lp.up[j] { v[j] } else { 0.0 }).collect();
        let old = (std::mem::replace(&mut st.y, yb), std::mem::replace(&mut st.z, zb), std::mem::replace(&mut st.s, sb));
        let db = shifts(lp, st, self.floor);
        if db.1 < da.1 {
            da = db;
        } else {
            (st.y, st.z, st.s) = old;
        }
        let (ex, ez) = da;
        self.shift_min = ex.min(ez);
        for j in 0..m + n {
            if lp.lo[j] {
                st.x[j] = (st.x[j] + ex).min(1e10);
                st.z[j] = (st.z[j] + ez).min(1e10);
            } else {
                st.x[j] = st.x[j].clamp(-1e10, 1e10);
            }
        }
        for j in 0..n {
            if lp.up[j] {
                st.w[j] = (st.w[j] + ex).min(1e10);
                st.s[j] = (st.s[j] + ez).min(1e10);
            }
        }
        st.y.iter_mut().for_each(|v| *v = v.clamp(-1e10, 1e10));
        let mut r = residuals(lp, st);
        recenter(lp, st, &mut r.rd, r.compl);
    }

    /// `2f59f1`: solve (+ `2f9699` refinement in mode 1), then dw = r_u − dx, ds = (rws − s dw)/w (T&2),
    /// dz = (rxz − z dx)/x (T&1).
    fn direction(&mut self, rhs_x: &[f64], rhs_y: &[f64], ru: &[f64], rxz: &[f64], rws: &[f64]) -> Dir {
        let lp = self.lp;
        let (mut dx, mut dy) = self.fac.solve(rhs_x, rhs_y);
        let mut res = [0.0; 2]; // 2f59f1 zeroes B+0x770/0x780; only the refinement writes them
        if self.mode1 {
            let (rdx, rdy, steps, rres) = self.refine(rhs_x, rhs_y, dx, dy);
            (dx, dy, res) = (rdx, rdy, rres);
            self.reflog.push(steps);
        }
        let st = &self.st;
        let dw: Vec<f64> = (0..lp.n).map(|j| if lp.up[j] { ru[j] - dx[j] } else { 0.0 }).collect();
        let ds = (0..lp.n).map(|j| if lp.up[j] { (rws[j] - st.s[j] * dw[j]) / st.w[j] } else { 0.0 }).collect();
        let dz =
            (0..lp.m + lp.n).map(|j| if lp.lo[j] { (rxz[j] - st.z[j] * dx[j]) / st.x[j] } else { 0.0 }).collect();
        Dir { dx, dy, dz, dw, ds, res }
    }

    /// `2f9699`: iterative refinement on the unperturbed system −D dx + Aᵀdy = rx, A dx = ry while ‖r‖² ≥ B+0x280²;
    /// a correction is kept only if it lowers ‖r‖² below 0.99×, at most 5 kept.
    fn refine(&self, rx: &[f64], ry: &[f64], mut dx: Vec<f64>, mut dy: Vec<f64>) -> (Vec<f64>, Vec<f64>, usize, [f64; 2]) {
        let lp = self.lp;
        let res = |dx: &[f64], dy: &[f64]| {
            let ry_: Vec<f64> = ry.iter().zip(lp.ax(dx)).map(|(r, a)| r - a).collect();
            let rx_: Vec<f64> = (0..lp.m + lp.n).zip(lp.aty(dy)).map(|(j, a)| rx[j] - a + self.dtrue[j] * dx[j]).collect();
            (ry_, rx_)
        };
        let (mut r_y, mut r_x) = res(&dx, &dy);
        let mut nrm = dot(&r_y, &r_y) + dot(&r_x, &r_x);
        let mut resn = [dot(&r_y, &r_y).sqrt(), dot(&r_x, &r_x).sqrt()];
        let (mut kept, mut steps) = (0, 0);
        while nrm >= self.reg * self.reg {
            let (cx, cy) = self.fac.solve(&r_x, &r_y);
            steps += 1;
            let ndx: Vec<f64> = dx.iter().zip(&cx).map(|(a, b)| a + b).collect();
            let ndy: Vec<f64> = dy.iter().zip(&cy).map(|(a, b)| a + b).collect();
            let (n_y, n_x) = res(&ndx, &ndy);
            let new = dot(&n_y, &n_y) + dot(&n_x, &n_x);
            if nrm * 0.99 <= new {
                break;
            }
            resn = [dot(&n_y, &n_y).sqrt(), dot(&n_x, &n_x).sqrt()];
            (dx, dy, r_y, r_x, nrm) = (ndx, ndy, n_y, n_x, new);
            kept += 1;
            if kept > 4 {
                break;
            }
        }
        (dx, dy, steps, resn)
    }

    /// `2d64b4`: k = 1 Mehrotra (μ_t − dx dz of the affine direction), k ≥ 2 Gondzio box [μ_t/10, 10μ_t] at the trial
    /// point with steps 1.5α + 0.3 (cap 1.001). Returns (rxz, rws, rhs_x); correctors carry no r_p, r_d, r_u terms.
    fn corrector_rhs(&self, d: &Dir, mu_t: f64, k: i32, ap: f64, ad: f64) -> (Vec<f64>, Vec<f64>, Vec<f64>) {
        let (lp, st) = (self.lp, &self.st);
        let (n, mn) = (lp.n, lp.m + lp.n);
        let (mut rxz, mut rws, mut rhs) = (vec![0.0; mn], vec![0.0; n], vec![0.0; mn]);
        let (tp, td) = ((1.5 * ap + 0.3).min(1.001), (1.5 * ad + 0.3).min(1.001));
        let boxed = |v: f64| {
            if v < mu_t / 10.0 {
                mu_t / 10.0 - v
            } else if v > 10.0 * mu_t {
                (10.0 * mu_t - v).max(-10.0 * mu_t)
            } else {
                0.0
            }
        };
        for j in 0..mn {
            if lp.lo[j] {
                rxz[j] = if k == 1 {
                    mu_t - d.dx[j] * d.dz[j]
                } else {
                    boxed((st.x[j] + tp * d.dx[j]) * (st.z[j] + td * d.dz[j]))
                };
                rhs[j] -= rxz[j] / st.x[j];
            }
        }
        for j in 0..n {
            if lp.up[j] {
                rws[j] = if k == 1 {
                    mu_t - d.dw[j] * d.ds[j]
                } else {
                    boxed((st.w[j] + tp * d.dw[j]) * (st.s[j] + td * d.ds[j]))
                };
                rhs[j] += rws[j] / st.w[j];
            }
        }
        (rxz, rws, rhs)
    }

    /// `2f4fe9`: grid over ω from 1 down to lo (clamp(α_pα_d, 0.01, 1) for k = 1, 0 for k ≥ 2); primal and dual pick
    /// their own ω (first strict 1.0001× improvement); d0 += ω d1. Returns (α_p, α_d, both ω = 0).
    fn line_search(&mut self, d0: &mut Dir, d1: &Dir, k: i32, ap: f64, ad: f64) -> (f64, f64, bool) {
        let lo = if k < 2 { (ap * ad).clamp(0.01, 1.0) } else { 0.0 };
        let nn = self.ngrid;
        let (mut bp, mut bd, mut wp, mut wd) = (-1e100, -1e100, 0.0, 0.0);
        for i in 0..nn {
            let om = ((nn - 1 - i) as f64 * (1.0 - lo)) / (nn - 1) as f64 + lo;
            let (a, b) = if om != 0.0 { step_lengths(self.lp, &self.st, d0, Some((d1, om))) } else { (ap, ad) };
            if bp * 1.0001 < a {
                (bp, wp) = (a, om);
            }
            if bd * 1.0001 < b {
                (bd, wd) = (b, om);
            }
        }
        let add = |v: &mut Vec<f64>, u: &[f64], w: f64| v.iter_mut().zip(u).for_each(|(a, b)| *a += w * b);
        add(&mut d0.dx, &d1.dx, wp);
        add(&mut d0.dw, &d1.dw, wp);
        add(&mut d0.dz, &d1.dz, wd);
        add(&mut d0.ds, &d1.ds, wd);
        add(&mut d0.dy, &d1.dy, wd);
        d0.res = [d0.res[0].max(wp * d1.res[0]), d0.res[1].max(wd * d1.res[1])];
        self.last_om = (wp, wd);
        (bp, bd, wp == 0.0 && wd == 0.0)
    }

    /// `2f2e95`: damped step (1 − η) with η from the worst trial product, then x, z, w, s floored at 1e-30.
    fn update(&mut self, d: &Dir, alpha_p: f64, alpha_d: f64, it: usize) {
        let (lp, st) = (self.lp, &mut self.st);
        let eta0 = 1e-6;
        let (ap, ad) = ((alpha_p * (1.0 - eta0)).min(1.0), ((1.0 - eta0) * alpha_d).min(1.0));
        if ap * ad > 5e-5 {
            self.f4 = it + 1;
        }
        let mut p = vec![];
        for j in (0..lp.m + lp.n).filter(|&j| lp.lo[j]) {
            p.push((d.dx[j] * ap + st.x[j]) * (d.dz[j] * ad + st.z[j]));
        }
        for j in (0..lp.n).filter(|&j| lp.up[j]) {
            p.push((d.dw[j] * ap + st.w[j]) * (d.ds[j] * ad + st.s[j]));
        }
        let mn = p.iter().fold(1e100f64, |a, &v| a.min(v));
        let avg = (p.iter().sum::<f64>() + st.kappa * st.tau) / lp.ncomp as f64;
        let eta = if mn < 0.01 * avg { eta0.max((0.01 * eta0 * avg) / mn) } else { eta0 }.min(0.5);
        let (ap, ad) = ((alpha_p * (1.0 - eta)).min(1.0), ((1.0 - eta) * alpha_d).min(1.0));
        for j in 0..lp.m + lp.n {
            let (x, z) = (st.x[j] + ap * d.dx[j], st.z[j] + ad * d.dz[j]);
            (st.x[j], st.z[j]) = if lp.lo[j] { (x.max(1e-30), z.max(1e-30)) } else { (x, z) };
        }
        for j in 0..lp.n {
            if lp.up[j] {
                st.w[j] = (st.w[j] + ap * d.dw[j]).max(1e-30);
                st.s[j] = (st.s[j] + ad * d.ds[j]).max(1e-30);
            }
        }
        st.y.iter_mut().zip(&d.dy).for_each(|(y, dy)| *y += ad * dy);
    }

    /// One pass of the loop body after the termination test: factor, predictor, correctors, step limit, update.
    /// Returns the number of correctors run.
    fn iteration(&mut self, r: &Res, it: usize, barcorr: i32) -> usize {
        let (lp, n) = (self.lp, self.lp.n);
        let th = theta(lp, &self.st, r.pobj);
        // predictor RHS: rxz = −xz (T&1), rws = −ws (T&2), rhs_x = r_d − rxz/x + (rws − r_u s)/w
        let st = &self.st;
        let rxz: Vec<f64> = (0..lp.m + n).map(|j| if lp.lo[j] { -st.x[j] * st.z[j] } else { 0.0 }).collect();
        let rws: Vec<f64> = (0..n).map(|j| if lp.up[j] { -st.w[j] * st.s[j] } else { 0.0 }).collect();
        let mut rhs = r.rd.clone();
        for j in 0..lp.m + n {
            if lp.lo[j] {
                rhs[j] -= rxz[j] / st.x[j];
            }
            if j < n && lp.up[j] {
                rhs[j] += (rws[j] - r.ru[j] * st.s[j]) / st.w[j];
            }
        }
        let (min_ratio, _) = self.fac.factor(&th[..n], &th[n..]);
        self.mode1 = lp.t[..n].contains(&0) || min_ratio < 1e-10;
        self.dtrue = theta_inv(lp, &self.st);
        self.reg = ((r.rp2 * r.rp2 + r.rd2 * r.rd2).sqrt() / 10.0).max(1e-8); // B+0x280, no safe mode
        self.reflog.clear();
        let mut d0 = self.direction(&rhs, &r.rp, &r.ru, &rxz, &rws);
        let (mut ap, mut ad) = step_lengths(lp, &self.st, &d0, None);
        // 2f28c2: μ_aff = complementarity at the trial point / ncomp (floor 1e-30). Columns with a lower bound
        // sequentially, then all slacks and the finite-ub column range w·s with 4 lanes over 8-blocks (asm 0x30ca96)
        let st = &self.st;
        let pxz = |j: usize| (d0.dx[j] * ap + st.x[j]) * (d0.dz[j] * ad + st.z[j]);
        let s = (0..n).filter(|&j| lp.lo[j]).fold(0.0, |a, j| a + pxz(j));
        let s = vsum4(s, &(n..n + lp.m).map(pxz).collect::<Vec<_>>(), n & 1);
        let (lo, hi) = lp.ub_range();
        let pws = |j: usize| (d0.dw[j] * ap + st.w[j]) * (d0.ds[j] * ad + st.s[j]);
        let s = vsum4(s, &(lo..hi).map(pws).collect::<Vec<_>>(), lo & 1);
        let mu_aff = ((st.tau * st.kappa + s) / lp.ncomp as f64).max(1e-30);
        let q = mu_aff / r.compl.max(1e-30);
        let sigma = q * q * q.min(0.1);
        let mu_t = if self.f4 < it { r.compl * 0.9 } else { r.compl * sigma };
        if barcorr >= 0 {
            self.k = barcorr.max(1);
        }
        let mut ncorr = 0;
        let mut last_d1 = None;
        let mut oms = vec![];
        for k in 1..=self.k {
            // k = 1 uses the affine direction, which d0 still is
            let (cxz, cws, crhs) = self.corrector_rhs(&d0, mu_t, k, ap, ad);
            let d1 = self.direction(&crhs, &vec![0.0; lp.m], &vec![0.0; n], &cxz, &cws);
            let (nap, nad, none) = self.line_search(&mut d0, &d1, k, ap, ad);
            oms.push(self.last_om);
            last_d1 = Some(d1.clone());
            ncorr += 1;
            let stop = none
                || (nap >= 1.001 && nad >= 1.001)
                || (k > 1 && nad * nap < 1e-100)
                || (k > 1.max(self.k / 10) && nap * 0.99 * nad < ap * ad)
                || d0.res[0] > 1e30
                || d0.res[1] > 1e30;
            (ap, ad) = (nap, nad);
            if stop {
                break;
            }
        }
        if r.mu < 1e-4 {
            // 2bf1ea L836–880: keep the solve error of the direction below the current residual
            let lim = (r.rp2 * 1.2).max(lp.ptol);
            if d0.res[0] > lim {
                ap = (lim / d0.res[0]).min(ap);
            }
            let lim = (r.rd2 * 1.2).max(lp.dtol);
            if d0.res[1] > lim {
                ad = (lim / d0.res[1]).min(ad);
            }
        }
        dump(|| {
            let d1 = last_d1.as_ref().map_or((vec![], vec![]), |d: &Dir| (d.dx.clone(), d.dy.clone()));
            let oms: Vec<f64> = oms.iter().flat_map(|&(a, b)| [a, b]).collect();
            format!(
                "{{\"step\":{it},\"theta\":{},\"dx\":{},\"dy\":{},\"dz\":{},\"dw\":{},\"ds\":{},\"a\":{},\"ncorr\":{ncorr},\"mu_t\":{},\"d1x\":{},\"d1y\":{},\"oms\":{}}}",
                hx(&th), hx(&d0.dx), hx(&d0.dy), hx(&d0.dz), hx(&d0.dw), hx(&d0.ds), hx(&[ap, ad]), hx(&[mu_t]), hx(&d1.0), hx(&d1.1), hx(&oms)
            )
        });
        self.ab = [ap * ad, self.ab[0], self.ab[1]];
        self.last_res = d0.res;
        self.update(&d0, ap, ad, it);
        ncorr
    }
}

/// `2bf1ea` L896–994 restart test before any restart (B+0xf8 = 0; LP, B+0x24 = 0): μ flat over three iterations with
/// tiny steps and an accurate solve, or two zero steps, while the complementarity gap is not small. `ab` = (B+0x5e0,
/// 0x5e8, 0x5f0) step products, `res` = (B+0x770, B+0x780).
fn restart_due(lp: &Lp, r: &Res, it: usize, hmu: &[f64], ab: [f64; 3], res: [f64; 2]) -> bool {
    let mut cg = lp.ncomp as f64 * r.compl / (r.ctx.abs() + 1.0);
    if r.rp_rel < 1e-7 && r.rd_rel < 1e-7 {
        cg *= 0.1;
    }
    let ratio = if it < 3 {
        0.0
    } else if hmu[it - 3] == 0.0 {
        1.0
    } else {
        hmu[it] / hmu[it - 3]
    };
    let slow = ab.iter().all(|&a| a < 0.001) && ratio > 0.98 && res[0] < 0.001 && res[1] < 0.001;
    cg > 1e-4 && (slow || (ab[0] == 0.0 && ab[1] == 0.0))
}

/// `b2aecf` for LP with Crossover=0: 2 optimal, 13 numeric trouble, 7 iteration limit, 1 continue.
fn terminate(lp: &Lp, r: &Res, it: usize, hmu: &[f64], hcompl: &[f64], p: &Params) -> i32 {
    let (tol, nc) = (p.bar_conv_tol, lp.ncomp as f64);
    let gap = (r.pobj - r.dobj).abs();
    let grel = (gap / ((r.pobj - lp.objcon).abs() + 1.0)).max(gap / (100.0 * r.pobj.abs() + 1.0));
    let cg = r.compl * nc / (r.ctx.abs() + 1.0);
    let mg = nc * r.mu / (r.pobj.abs() + 1.0);
    let fast = it > 0 && r.mu > tol && hmu[it] < 0.5 * hmu[it - 1];
    let pfeas = (r.rp2 * r.rp2 + r.ru2 * r.ru2).sqrt() < lp.ptol || r.rp_rel < 10.0 * tol;
    let dfeas = r.rd2 < lp.dtol || r.rd_rel < 10.0 * tol;
    let kc = 0.0 < 100.0 * tol; // κ < 100 tol τ with κ = 0, τ = 1 (no homogeneous mode)
    if r.mu.is_nan() || r.mu > 1e100 || (it > 5 && r.compl > 1e20 && r.compl > 0.99 * hcompl[it - 1]) {
        return 13;
    }
    if (grel < tol && mg < 100.0 * tol && pfeas && dfeas && kc && !fast)
        || (cg < tol && mg < 100.0 * tol && grel < 0.1 && pfeas && dfeas && !fast)
        || (grel < tol && cg < tol && pfeas && dfeas && it > 1 && hmu[it] > 0.5 * hmu[it - 1])
        || (r.mu < 1e-16 && pfeas && dfeas)
        || (it > 10 && kc && grel < 10.0 * tol && pfeas && dfeas && r.compl > 0.8 * hcompl[it - 3])
    {
        return 2;
    }
    if it as i64 >= p.bar_iter_limit {
        return 7;
    }
    1
}

/// Whole barrier loop. `f160` = 2 nnz(L), `f168` = factor work estimate (`2e7f38`).
pub fn run(lp: &Lp, sym: &Symbolic, f160: f64, f168: f64, p: &Params, cb: &mut dyn FnMut(&IterInfo)) -> BarResult {
    let (m, n) = (lp.m as f64, lp.n as f64);
    let fin: Vec<usize> = (0..lp.n).filter(|&j| lp.ub[j] < 1e30).collect();
    let nub = fin.last().map_or(0.0, |&l| (l + 1 - fin[0]) as f64); // B+0x634 − B+0x630
    let nnz2 = 2.0 * lp.nnz as f64; // B+0x5a8
    // work per call, 0.75 units (docs/m3_ipm.md §4.6)
    let w_solve = (2.0 * nnz2 + f160 + 11.0 * m + 4.0 * n) * 0.75; // 2ee6c5 incl. 2d3b3e, 2d3d91
    let w_dir = w_solve + (5.0 * (m + n) + 9.0 * nub) * 0.75; // 2f59f1
    let w_ratio = (7.0 * (m + n) + 6.0 * nub) * 0.75; // 2f15ba, one 2f4fe9 grid point
    let w_crhs = (3.0 * (m + nub) + 13.0 * n) * 0.75; // 2d64b4
    // timers B+0x2a8 (factor), 0x2e8 (predictor), 0x328 (first corrector), 0x368 (other correctors), 0x3a8 (update):
    // work + 1 per start
    let (mut tf, mut tp, mut tc1, mut tcx, mut tu) = (1.0 + 0.75 * f168, 0.0, 0.0, 0.0, 0.0);
    // 2e7f38: initial K and grid N from solve vs factor work
    let r0 = if lp.m == 0 { 1.0 } else { (2.0 * nnz2 + f160) / f168 };
    let mut ipm = Ipm {
        lp,
        fac: Factor::new(lp, sym),
        st: State { x: vec![], z: vec![], y: vec![], w: vec![], s: vec![], tau: 1.0, kappa: 0.0 },
        mode1: false,
        dtrue: vec![],
        reg: 0.0,
        reflog: vec![],
        k: (0.1 / r0).clamp(1.0, 20.0) as i32,
        ngrid: if r0 > 0.2 { 3 } else { 5 },
        f4: 0,
        ab: [1e100; 3],
        floor: 0.0,
        shift_min: 0.0,
        f8: 0,
        last_res: [0.0; 2],
        last_om: (0.0, 0.0),
    };
    ipm.initial_point();
    let base = 1.max((n.max(1.0)).log10().floor() as i32 - 2);
    let (mut hmu, mut hcompl) = (vec![], vec![]);
    let mut it = 0;
    let (status, r) = loop {
        let mut r = residuals(lp, &ipm.st);
        // μ stalled (never NaN here: terminate returned 13 on it)
        if it >= 11 && hmu[it - 1] > hmu[it - 2] * 0.99 {
            recenter(lp, &mut ipm.st, &mut r.rd, r.compl);
        }
        dump(|| {
            let (st, sc) = (&ipm.st, [r.compl, r.mu, r.rp2, r.rd2, r.ru2, r.pobj, r.dobj, r.rp_rel, r.rd_rel, r.ctx]);
            format!(
                "{{\"iter\":{it},\"x\":{},\"y\":{},\"z\":{},\"w\":{},\"s\":{},\"sc\":{},\"rd\":{},\"rp\":{}}}",
                hx(&st.x), hx(&st.y), hx(&st.z), hx(&st.w), hx(&st.s), hx(&sc), hx(&r.rd), hx(&r.rp)
            )
        });
        cb(&IterInfo {
            iter: it,
            pobj: lp.sense * r.pobj,
            dobj: lp.sense * r.dobj,
            priminf: (r.rp2 * r.rp2 + r.ru2 * r.ru2).sqrt(),
            dualinf: r.rd2,
            compl: r.compl,
        });
        hmu.push(r.mu);
        hcompl.push(r.compl);
        let status = terminate(lp, &r, it, &hmu, &hcompl, p);
        if status != 1 {
            break (status, r);
        }
        tf += 1.0 + 0.75 * f168;
        let (kcur, ab, ngrid) = (ipm.k, ipm.ab, ipm.ngrid as f64);
        let nc = ipm.iteration(&r, it, p.bar_correctors);
        let w_ref: Vec<f64> = if ipm.mode1 {
            ipm.reflog.iter().map(|&k| 1.5 * nnz2 + k as f64 * (w_solve + 1.5 * nnz2)).collect() // 2f9699
        } else {
            vec![0.0; nc + 1]
        };
        tp += 1.0 + w_dir + w_ratio + w_ref[0];
        let wc = 1.0 + w_crhs + w_dir;
        if nc >= 1 {
            tc1 += wc + w_ratio * ngrid + w_ref[1];
        }
        tcx += (2..=nc).fold(0.0, |a, k| a + (wc + w_ratio * (ngrid - 1.0) + w_ref[k])); // ω = 0 skips the RT
        if p.bar_correctors < 0 {
            let avg = if it < 3 { 1.0 } else { (ab[0] + ab[1] + ab[2]) * 0.3333333333333333 };
            ipm.k = adjust_k(kcur, tcx / (tf + tp + tc1 + tu), avg, base);
        }
        ipm.ngrid = 3.max((ipm.k + 1).min(10)) as usize;
        tu += 1.0;
        if ipm.f8 == 0 && restart_due(lp, &r, it, &hmu, ipm.ab, ipm.last_res) {
            // 2bf1ea L956–994: restart from 2d0663 with the shift floor raised, factor timed as in the loop
            ipm.floor = 1000f64.max(10.0 * ipm.shift_min);
            tf += 1.0 + 0.75 * f168;
            ipm.initial_point();
            ipm.f8 = it + 1;
        } else if ipm.f4 < it && p.bar_correctors == -1 {
            if ipm.k < 11 {
                ipm.k = 10;
            }
            ipm.ngrid = 10;
        }
        it += 1;
    };
    BarResult {
        status,
        obj: lp.sense * r.pobj,
        bar_iters: it,
        x: lp.x_orig(&ipm.st.x),
        // shortcut: barrier-loop timers only (no ordering/setup work), units of 1e9 like the Work attribute
        work: (tf + tp + tc1 + tcx + tu) * 1e-9,
    }
}
