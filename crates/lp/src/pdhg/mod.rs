//! 05 §4 PDHG (Method=6) body `34c004`, ported against the PDHG-state oracle `tools/pdhg_oracle.py`
//! (docs/m3_pdhg.md). Flow: preconditioning `07d3ce(m, 4)` (`scale`) → restarted reflected Halpern PDHG (`kern`).
//! shortcut: Presolve=0, Crossover=0 and one thread are assumed (no presolve, no crossover, no worker threads).

pub mod eval;
pub mod kern;
pub mod scale;

use eval::State;
use kern::{red8, Vecs};
use rsolver_model::Model;

/// Parameters the PDHG reads (Gurobi names: PDHGIterLimit, PDHGRelTol, PDHGAbsTol, PDHGConvTol, WorkLimit).
#[derive(Clone, Debug)]
pub struct Params {
    pub iter_limit: f64,
    pub rel_tol: f64,
    pub abs_tol: f64,
    pub conv_tol: f64,
    pub work_limit: f64,
    /// the optimize's work timer (1e-9 units) when `34c004` starts, e.g. the presolve work; WorkLimit counts it
    pub work0: f64,
}

impl Default for Params {
    fn default() -> Self {
        Params { iter_limit: 1e100, rel_tol: 1e-6, abs_tol: 1e-6, conv_tol: 1e-6, work_limit: 1e100, work0: 0.0 }
    }
}

/// PDHG callback values (where = 10): P[0], P+0x38, P+0x40, P+0x60, P+0x68, P+0x48.
#[derive(Clone, Debug)]
pub struct IterInfo {
    pub iter: i64,
    pub pobj: f64,
    pub dobj: f64,
    pub priminf: f64,
    pub dualinf: f64,
    pub compl: f64,
}

#[derive(Clone, Debug)]
pub struct PdhgResult {
    /// Gurobi status code: 2 optimal, 7 iteration limit, 12 numeric, 16 work limit
    pub status: i32,
    pub obj: f64,
    /// PDHGIterCount = P[0] (PDHG and power iterations)
    pub iters: i64,
    /// column values in the model's space (`7a069`: x̂·s)
    pub x: Vec<f64>,
    /// work counter W[0] (1e-9 work units) at the end of `34c004`
    pub work: f64,
}

/// `355602` (one thread): 2 optimal, 7 iteration limit, 16 work limit, 12 numeric, 1 running.
fn status(st: &State, p: &Params, work: f64) -> i32 {
    if (st.rp_inf < p.abs_tol && st.rd_inf < p.abs_tol && st.gap < p.conv_tol)
        || (p.rel_tol > 0.0 && st.rp_rel < p.rel_tol && st.rd_rel < p.rel_tol && st.gap < p.conv_tol)
    {
        2
    } else if p.iter_limit <= st.it as f64 {
        7
    } else if p.work_limit < 1e100 && p.work_limit * 1e9 < work {
        16
    } else if st.pobj.abs() <= 1e100 && st.dobj.abs() <= 1e100 && st.compl.abs() <= 1e100 {
        1
    } else {
        12
    }
}

/// `a7b4d6`/`a7b54f`: the model RNG (state model+0xb0, 0 on a fresh model) as a double in [0, 1).
fn rand01(seed: &mut u64) -> f64 {
    *seed = seed.wrapping_mul(0x5851f42d4c957f2d).wrapping_add(0x14057b7ef767814f);
    let mut u = (*seed >> 32) as u32;
    u ^= u >> 11;
    u ^= (u & 0x13a58ad) << 7;
    u ^= (u & 0x1df8c) << 15;
    ((u ^ (u >> 18)) & 0x7fffffff) as f64 * 4.656612873077393e-10
}

/// Debug aid: with PDHGDUMP=<file>, append the state at every PDHG callback as one JSON line (doubles as bit
/// patterns, keyed by their P offset) for tools/pdhg_cmp.py, which compares it with tools/pdhg_oracle.py output.
fn dump(st: &State) {
    let Ok(path) = std::env::var("PDHGDUMP") else { return };
    use std::io::Write;
    let b = |v: &[f64]| v.iter().map(|x| x.to_bits().to_string()).collect::<Vec<_>>().join(",");
    let sc = [(0x38, st.pobj), (0x40, st.dobj), (0x48, st.compl), (0x50, st.rp_inf), (0x58, st.rd_inf), (0x60, st.rp2),
        (0x68, st.rd2), (0x70, st.rp_rel), (0x78, st.rd_rel), (0x80, st.gap), (0xe0, st.eta), (0xe8, st.omega),
        (0x100, st.integ), (0x108, st.err), (0x110, st.g), (0x118, st.g_prev), (0x120, st.g0), (0x170, st.bnorm),
        (0x178, st.cnorm), (0x188, st.eps), (0x190, st.tol)];
    let sc: Vec<String> = sc.iter().map(|(o, x)| format!("\"{o:#x}\":{}", x.to_bits())).collect();
    let v = &st.v;
    let vecs = [(0x28, &v.x), (0x30, &v.y), (0xb8, &v.xhat), (0xc0, &v.yhat), (0x128, &v.x0), (0x130, &v.y0)];
    let vecs: Vec<String> = vecs.iter().map(|(o, x)| format!("\"{o:#x}\":[{}]", b(x))).collect();
    let mut f = std::fs::OpenOptions::new().create(true).append(true).open(path).expect("PDHGDUMP file");
    writeln!(f, "{{\"it\":{},\"cnt\":[{},{},{},{}],\"d\":{{{}}},\"vec\":{{{}}}}}", st.it, st.it, st.it_pdhg, st.k,
             st.pw_it, sc.join(","), vecs.join(",")).expect("PDHGDUMP write");
}

/// Solve the LP `m` with PDHG (`34c004`); `cb` runs at every PDHG callback (every 100 PDHG iterations and after
/// every power-iteration chunk).
pub fn solve(md: &Model, p: &Params, cb: &mut dyn FnMut(&IterInfo)) -> PdhgResult {
    let d = scale::scale_flag4(md);
    let (n, m, nnz) = (d.a.n, d.a.m, d.a.nnz());
    let info = |st: &State| IterInfo {
        iter: st.it,
        pobj: st.pobj,
        dobj: st.dobj,
        priminf: st.rp2,
        dualinf: st.rd2,
        compl: st.compl,
    };
    let mut st = State { tol: 1e-4, ..Default::default() };
    st.wc = d.s.iter().map(|s| 1.0 / s).collect();
    st.wr = d.r.iter().map(|r| 1.0 / r).collect();
    let norm = |v: &[f64], w: &[f64]| {
        red8(v.len(), 0, |i| {
            let t = v[i] * w[i];
            t * t
        })
        .sqrt()
    };
    st.cnorm = norm(&d.c, &st.wc);
    st.bnorm = norm(&d.b, &st.wr);
    st.cnt = (0..n).filter(|&j| d.lb[j] > -1e30 || d.ub[j] < 1e30).count() as i64
        + d.sense.iter().filter(|&&s| s != b'=').count() as i64; // `3531c6`
    st.omega = if st.bnorm > 0.0 && st.cnorm > 0.0 { st.cnorm / st.bnorm } else { 1.0 };
    st.omega0 = st.omega;
    (st.eps, st.eta) = if m == 0 || n == 0 { (0.0, 1.0) } else { (1e100, 0.0) };
    let mut work = p.work0 + ((nnz as f64 + m as f64 * 5.0) + n as f64 * 9.0);
    // start point: x = clamp(0, l, u) (ties keep the bound), y = 0
    let x0: Vec<f64> = (0..n)
        .map(|j| {
            let z = if 0.0 <= d.lb[j] { d.lb[j] } else { 0.0 };
            if d.ub[j] <= z { d.ub[j] } else { z }
        })
        .collect();
    let y0 = vec![0.0f64; m];
    st.v = Vecs {
        x: x0.clone(),
        y: y0.clone(),
        xhat: x0.clone(),
        yhat: y0.clone(),
        xdel: vec![0.0; n],
        xref: vec![0.0; n],
        yref: vec![0.0; m],
        x0,
        y0,
        ax: vec![0.0; m],
        aty: vec![0.0; n],
        lb: d.lb.clone(),
        ub: d.ub.clone(),
        dz: vec![0.0; n + m],
    };
    d.a.ax(&st.v.x, &mut st.v.ax);
    work += nnz as f64 * 3.0;
    d.a.aty(&st.v.y, &mut st.v.aty);
    work += nnz as f64 * 3.0;
    let mut seed = md.rng;
    st.pv = (0..m).map(|_| rand01(&mut seed)).collect();

    let eq: Vec<bool> = d.sense.iter().map(|&s| s == b'=').collect();
    let mut ev = true;
    let code = loop {
        let mut rs = 0;
        if ev || st.it as f64 >= p.iter_limit {
            let first = st.it == 0;
            st.eval(&d, first, &mut work);
            if st.it_pdhg % 100 == 0 {
                cb(&info(&st));
                dump(&st);
            }
            let s = status(&st, p, work);
            if s != 1 {
                break s;
            }
            rs = st.restart_code();
            st.g_prev = st.g;
            if rs != 0 || st.it == 0 {
                if rs != 0 {
                    st.restart();
                }
                let mut s = 1;
                st.power(&d.a, p.iter_limit, &mut work, &mut |st, w| {
                    cb(&info(st));
                    dump(st);
                    s = status(st, p, w);
                    s == 1
                });
                if s != 1 {
                    break s;
                }
            }
            work += (m + n) as f64 * 3.0;
        }
        ev = (st.it_pdhg + 1) % 100 == 0;
        kern::primal_step(&d.a, &d.c, &mut st.v, st.eta, st.omega, ev, &mut work);
        kern::dual_step(&d.a, &d.b, &eq, &mut st.v, st.eta, st.omega, &mut work);
        if ev || rs != 0 {
            st.g = kern::fixed_point_norm(&d.a, &mut st.v, st.eta, st.omega, &mut work);
            if rs != 0 {
                st.g0 = st.g;
            }
        }
        kern::halpern(&mut st.v, st.k, &mut work);
        st.it += 1;
        st.it_pdhg += 1;
        st.k += 1;
    };
    st.eval(&d, true, &mut work);
    let x = (0..n).map(|j| st.v.xhat[j] * d.s[j]).collect();
    PdhgResult { status: code, obj: st.pobj * md.sense as f64, iters: st.it, x, work }
}
