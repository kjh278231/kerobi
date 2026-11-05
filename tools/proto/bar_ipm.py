"""Barrier (Method=2, LP) prototype: Gurobi 13.0.3 interior-point rules (docs/m3_ipm.md) in numpy.

Internal form (barrier workspace B, docs/m3_ipm.md §1): n columns then m slacks, A x + s = b, slack of a non-'=' row
has coefficient +1 and lower bound 0, '=' rows have no slack (type 0). Column type T: bit0 lower bound (x >= 0),
bit1 finite upper bound u (boxed columns already use u = ub - lb). Linear algebra is dense numpy (no supernodal
LDL^T), so values match Gurobi to rounding, not bit for bit.

usage: python tools/proto/bar_ipm.py <model.mps.gz>    (runs the prototype from the MPS file)
"""
import json
import sys

import numpy as np

BIG = 1e30


class LP:
    """Internal LP: dense A (m x n), b, c, lb, ub (u for boxed), T (types, m+n), eq (row is '=')."""

    def __init__(self, A, b, c, lb, ub, T, eq, objscale=1.0, objcon=0.0, b0=None, ncomp=None, cshift=0.0,
                 nnz=None, tol=1e-8):
        self.A, self.b, self.c, self.lb, self.ub = A, b, c, lb, ub
        self.T, self.eq, self.objscale, self.objcon = np.asarray(T, dtype=np.int8), np.asarray(eq, bool), objscale, objcon
        self.cshift = cshift  # B+0x618 = c . lb from the 312a1e bound shift
        self.m, self.n = A.shape
        self.lo = (self.T & 1) != 0  # has lower bound (m+n)
        self.up = np.zeros(self.m + self.n, bool)
        self.up[:self.n] = (self.T[:self.n] & 2) != 0
        # ncomp (B+0x210) and the b norms (B+0x160/0x168) are taken in 2ca086, before 30f5b9 adds/drops bounds and
        # before 312a1e shifts b by A lb: callers pass the original b0 and ncomp.
        self.ncomp = int(self.lo.sum() + self.up.sum()) if ncomp is None else ncomp
        b0 = b if b0 is None else b0
        self.bnorm2, self.bnorminf = float(np.sqrt(b0 @ b0)), float(np.abs(b0).max(initial=0.0))
        self.cnorm2, self.cnorminf = float(np.sqrt(c @ c)), float(np.abs(c).max(initial=0.0))
        self.nnz = int(np.count_nonzero(A)) if nnz is None else nnz  # B+0x5a8 = 2 nnz drives the work counter
        self.ptol = (np.sqrt(self.m + 1.0) + self.bnorm2) * tol  # B+0x260
        self.dtol = (np.sqrt(self.n + 1.0) + self.cnorm2) * tol  # B+0x268

    def Ax(self, x):  # 2d4252 / 2d3b3e: A x[:n] + slack
        r = self.A @ x[:self.n]
        r[~self.eq] += x[self.n:][~self.eq]
        return r

    def ATy(self, y):  # 2d4252 / 2d3d91: [A^T y ; y_i for non-'=' rows, 0 for '=']
        return np.concatenate([self.A.T @ y, np.where(self.eq, 0.0, y)])


def load_oracle(path):
    """Records of tools/bar_oracle.py (jsonl or jsonl.gz); a bare instance name means bench/oracle/bar/<name>.jsonl.gz."""
    import gzip
    import os
    if not os.path.exists(path):
        path = os.path.join(os.path.dirname(__file__), "..", "..", "bench", "oracle", "bar", path + ".jsonl.gz")
    with (gzip.open(path, "rt") if path.endswith(".gz") else open(path)) as f:
        return [json.loads(line) for line in f]


def hexf(v):
    return np.array([float.fromhex(t) for t in v]) if v is not None else None


def lp_from_oracle(rec):
    """LP exactly as the oracle's first record shows it (D: scaled, rows permuted; B: bounds after 30f5b9)."""
    Dd = rec["D"]
    m, n = rec["m"], rec["n"]
    A = np.zeros((m, n))
    beg, ln, ind, val = Dd["beg"], Dd["len"], Dd["ind"], hexf(Dd["val"])
    for j in range(n):
        for k in range(beg[j], beg[j] + ln[j]):
            A[ind[k], j] = val[k]
    eq = np.array([ch == "=" for ch in Dd["sense"]])
    return LP(A, hexf(rec["rhs"]), hexf(rec["obj"]), hexf(rec["lb"]), hexf(rec["ub"]), rec["type"], eq,
              float.fromhex(Dd["objscale"]), float.fromhex(Dd["objcon"]), b0=hexf(Dd["rhs"]), ncomp=rec["ncomp"],
              cshift=float.fromhex(rec["B"][0x618 // 8]), nnz=Dd["nnz"])


class State:
    def __init__(self, lp):
        m, n = lp.m, lp.n
        self.x, self.z = np.zeros(m + n), np.zeros(m + n)
        self.y = np.zeros(m)
        self.w, self.s = np.zeros(n), np.zeros(n)
        self.tau, self.kappa = 1.0, 0.0


def residuals(lp, st, it=0, freepert_prev=None):
    """2d4977: residual vectors, norms, objectives, compl/mu, and the scaling theta for the next factorization."""
    n, tau = lp.n, st.tau
    R = {}
    rp = lp.b * tau - lp.Ax(st.x)
    rd = np.concatenate([lp.c * tau, np.zeros(lp.m)]) - lp.ATy(st.y) - st.z
    rd[:n] += st.s
    ru = np.where(lp.up[:n], lp.ub * tau - st.x[:n] - st.w, 0.0)
    R["rp"], R["rd"], R["ru"] = rp, rd, ru
    R["rp2"], R["rd2"], R["ru2"] = np.sqrt(rp @ rp) / tau, np.sqrt(rd @ rd) / tau, np.sqrt(ru @ ru) / tau
    R["rp_rel"] = max(np.abs(rp).max(initial=0.0), np.abs(ru).max(initial=0.0)) / tau
    dc = rd[:n]
    neg = lp.lo[:n] & (dc < 0)
    dinf = np.where(neg, np.maximum(-(dc + st.z[:n]), 0.0), np.abs(dc)).max(initial=0.0)
    dinf = max(dinf, st.y[~lp.eq].max(initial=0.0))
    R["rd_rel"] = dinf / tau
    xz = np.sum(st.x[lp.lo] * st.z[lp.lo]) + np.sum(st.w[lp.up[:n]] * st.s[lp.up[:n]])
    R["compl"] = (tau * st.kappa + xz) / lp.ncomp
    R["mu"] = R["compl"] / tau / tau
    ctx = lp.c @ st.x[:n] + lp.cshift * tau
    bty = lp.b @ st.y - lp.ub[lp.up[:n]] @ st.s[lp.up[:n]] + lp.cshift * tau
    R["ctx"], R["bty"] = ctx, bty
    R["pobj"] = ctx / lp.objscale / tau + lp.objcon
    R["dobj"] = bty / lp.objscale / tau + lp.objcon
    return R


def theta_inv(lp, st):
    """B+0x800: z/x (T&1) + s/w (T&2), 0 for free columns and '=' slacks (used by the 2f9699 refinement)."""
    n = lp.n
    d = np.zeros(lp.m + n)
    d[lp.lo] = st.z[lp.lo] / st.x[lp.lo]
    d[:n][lp.up[:n]] += st.s[lp.up[:n]] / st.w[lp.up[:n]]
    return d


def theta(lp, st, R):
    """2d4977 tail: theta = 1/(z/x + s/w) capped at 1e12; free columns get the BARFREEPERT perturbation."""
    n = lp.n
    x, z = st.x, st.z
    d = np.zeros(lp.m + n)
    d[lp.lo] = z[lp.lo] / x[lp.lo]
    d[:n][lp.up[:n]] += st.s[lp.up[:n]] / st.w[lp.up[:n]]
    th = np.zeros(lp.m + n)
    pos = d > 0
    th[pos] = np.minimum(1.0 / d[pos], 1e12)
    free = np.zeros(lp.m + n, bool)
    free[:n] = ~pos[:n]
    if free.any():
        g = (abs(R["pobj"]) + 1.0) * 1e-8 / np.maximum(np.abs(x[free]), 1e-10) ** 2
        th[free] = 1.0 / np.clip(g, 1e-12, 1e-6)
    th[n:][lp.eq] = 0.0
    return th


NB = 64


def ldlt(M):
    """Dense LDL^T in natural (ordering-permuted) row order with Gurobi's small-pivot rule (32af6a L264-280):
    a pivot |d| <= t = clamp(1e-10 |diag(M)|, 1e-12, 1e-6) becomes sign(d) t. Returns L (unit lower), d, number of
    replaced pivots, min |d/diag| (F+0x100). shortcut: no supernodes, so rounding differs from Gurobi."""
    A = M.copy()
    m = A.shape[0]
    diag0 = np.diag(M).copy()
    d = np.zeros(m)
    nsmall, minratio = 0, 1e100
    for k0 in range(0, m, NB):
        k1 = min(k0 + NB, m)
        P = A[k0:, k0:k1]
        for j in range(k1 - k0):
            g = k0 + j
            piv = P[j, j]
            if np.isnan(piv):
                minratio = 0.0
            elif diag0[g] != 0.0:
                minratio = min(minratio, abs(piv / diag0[g]))
            t = min(max(abs(diag0[g]) * 1e-10, 1e-12), 1e-6)
            if not t < abs(piv):
                piv = -t if piv < 0 else t
                nsmall += 1
            d[g] = piv
            col = P[j + 1:, j].copy()
            P[j + 1:, j + 1:] -= np.outer(col, col[:k1 - k0 - j - 1]) / piv
            P[j + 1:, j] = col / piv
        if k1 < m:
            Lr = A[k1:, k0:k1]
            A[k1:, k1:] -= (Lr * d[k0:k1]) @ Lr.T
    return np.tril(A, -1) + np.eye(m), d, nsmall, minratio


def trisolve(L, b, lower=True):
    """Blocked substitution with unit triangular L (lower) or L^T (upper)."""
    m = len(b)
    x = b.astype(float).copy()
    blocks = list(range(0, m, NB))
    if lower:
        for k0 in blocks:
            k1 = min(k0 + NB, m)
            x[k0:k1] = np.linalg.solve(L[k0:k1, k0:k1], x[k0:k1])
            x[k1:] -= L[k1:, k0:k1] @ x[k0:k1]
    else:
        for k0 in reversed(blocks):
            k1 = min(k0 + NB, m)
            x[k0:k1] = np.linalg.solve(L[k0:k1, k0:k1].T, x[k0:k1])
            x[:k0] -= L[k0:k1, :k0].T @ x[k0:k1]
    return x


class Normal:
    """Factor of A Theta A^T (2d8e22 -> 31c54b), reused by every solve of the iteration (2ee6c5).
    mode1 (F+0x178 = 1): free columns remain, or min pivot ratio F+0x100 < 1e-10 -> 2f59f1 refines (2f9699)."""

    def __init__(self, lp, th):
        self.lp, self.th = lp, th
        n = lp.n
        M = (lp.A * th[:n]) @ lp.A.T
        M[np.diag_indices_from(M)] += np.where(lp.eq, 0.0, th[n:])
        self.L, self.d, self.nsmall, self.minratio = ldlt(M)
        self.mode1 = bool((lp.T[:n] == 0).any()) or self.minratio < 1e-10
        self.nsolve = 0
        self.reflog = []  # refinement solves per 2f59f1 call (work counter)

    def solve(self, rx, ry):
        """-Theta^{-1} dx + A^T dy = rx, A dx = ry => (A Theta A^T) dy = ry + A Theta rx, dx = Theta (A^T dy - rx)."""
        lp, th = self.lp, self.th
        self.nsolve += 1
        rhs = ry + lp.Ax(th * rx)
        dy = trisolve(self.L, trisolve(self.L, rhs) / self.d, lower=False)
        return th * (lp.ATy(dy) - rx), dy

    def refine(self, D, rx, ry, dx, dy, reg):
        """2f9699: iterative refinement on the unperturbed system -D dx + A^T dy = rx, A dx = ry while
        |r|^2 >= reg^2; a correction is kept only if it lowers |r|^2 below 0.99x, at most 5 kept."""
        lp = self.lp

        def res(dx, dy):
            return ry - lp.Ax(dx), rx - lp.ATy(dy) + D * dx
        r_y, r_x = res(dx, dy)
        nrm = r_y @ r_y + r_x @ r_x
        kept = 0
        steps = 0
        while nrm >= reg * reg:
            cx, cy = self.solve(r_x, r_y)
            steps += 1
            ndx, ndy = dx + cx, dy + cy
            n_y, n_x = res(ndx, ndy)
            new = n_y @ n_y + n_x @ n_x
            if nrm * 0.99 <= new:
                break
            dx, dy, r_y, r_x, nrm = ndx, ndy, n_y, n_x, new
            kept += 1
            if kept > 4:
                break
        return dx, dy, steps


def solve_aug(lp, th, rx, ry):
    return Normal(lp, th).solve(rx, ry)


def shifts(lp, st):
    """2d252f: Mehrotra-type shifts (dx, dz) for the starting point."""
    n = lp.n
    lo, up = lp.lo, lp.up[:n]
    mx = min(st.x[lo].min(initial=1e100), st.w[up].min(initial=1e100))
    mz = min(st.z[lo].min(initial=1e100), st.s[up].min(initial=1e100))
    dx, dz = max(mx * -1.5, 1e-4), max(mz * -1.5, 1e-4)
    xs = np.concatenate([st.x[lo] + dx, st.w[up] + dx])
    zs = np.concatenate([st.z[lo] + dz, st.s[up] + dz])
    sx, sz, sxz = xs.sum(), zs.sum(), xs @ zs
    ex = (sxz * 0.5) / sz + dx if sz > 0 else 100.0
    ez = dz + (sxz * 0.5) / sx if sx > 0 else 100.0
    if np.sqrt(lp.bnorminf * 1.0001 * ex) >= ex:
        ex = np.sqrt(ex * lp.bnorminf * 1.0001)
    if np.sqrt(lp.cnorminf * 1.0001 * ez) >= ez:
        ez = np.sqrt(ez * lp.cnorminf * 1.0001)
    floor = 0.1  # B+0x30 (BARSAFE) = 0, no safe mode, LP
    return max(ex, floor), max(ez, floor)


def dual_from_c(lp, st):
    """2d0663 candidate 1: y = 0, c split between z (lower) and s (upper) by column type."""
    n, T, c = lp.n, lp.T[:lp.n], lp.c
    st.y[:] = 0.0
    st.z[:] = 0.0
    st.s[:] = 0.0
    zc = np.where(T == 3, c * 0.5, np.where(T == 1, c, 0.0))
    st.z[:n] = zc
    st.s[:] = np.where(T == 3, -(c * 0.5), np.where(T == 2, -c, 0.0))


def initial_point(lp, barstart=0):
    """2d0663 (BARSTART 0): least-squares x, two dual candidates, shifts, then the 2d8838 recentring of z, s."""
    n, m = lp.n, lp.m
    st = State(lp)
    T = lp.T
    th = np.where(T[:n] == 0, 1e-6, np.where(T[:n] == 3, 0.5, 1.0))
    th = np.concatenate([th, np.where(lp.eq, 0.0, 1.0)])
    rx = np.zeros(m + n)
    rx[:n][lp.up[:n]] = -lp.ub[lp.up[:n]]
    fac = Normal(lp, th)
    st.x, _ = fac.solve(rx, lp.b)
    st.w = np.where(lp.up[:n], lp.ub - st.x[:n], 0.0)
    dual_from_c(lp, st)
    da = shifts(lp, st)
    if barstart != 1:
        v, yb = fac.solve(np.concatenate([lp.c, np.zeros(m)]), np.zeros(m))
        zb, sb = np.where(lp.lo, -v, 0.0), np.where(lp.up[:n], v[:n], 0.0)
        y0, z0, s0 = st.y, st.z, st.s
        st.y, st.z, st.s = yb, zb, sb
        db = shifts(lp, st)
        if barstart == 2 or db[1] < da[1]:
            da = db
        else:
            st.y, st.z, st.s = y0, z0, s0
    ex, ez = da
    lo, up = lp.lo, lp.up[:n]
    st.x = np.where(lo, np.minimum(st.x + ex, 1e10), np.clip(st.x, -1e10, 1e10))
    st.z = np.where(lo, np.minimum(st.z + ez, 1e10), st.z)
    st.w = np.where(up, np.minimum(st.w + ex, 1e10), st.w)
    st.s = np.where(up, np.minimum(st.s + ez, 1e10), st.s)
    st.y = np.clip(st.y, -1e10, 1e10)
    st.shift = min(ex, ez)
    R = residuals(lp, st)
    recenter(lp, st, R["rd"], R["compl"])
    return st


def recenter(lp, st, rd, mu):
    """2d8838 (and 2d4977 when mu stalls): move z toward mu/x and s toward mu/w, by at most what r_d allows."""
    n = lp.n

    def step(t, cap):
        return np.where(t > 0, np.minimum(t, np.maximum(cap, 1e-20)), np.maximum(t, np.minimum(cap, -1e-20)))
    for j in range(n):
        if lp.lo[j]:
            t = step(mu / st.x[j] - st.z[j], rd[j])
            st.z[j] += t
            rd[j] -= t
        if lp.up[j]:
            t = step(mu / st.w[j] - st.s[j], -rd[j])
            st.s[j] += t
            rd[j] += t
    for i in range(n, n + lp.m):
        if lp.T[i] == 1:
            t = step(mu / st.x[i] - st.z[i], rd[i])
            st.z[i] += t
            rd[i] -= t


def predictor_rhs(lp, st, R):
    """2d4977 tail: rxz = -x z (T&1), rws = -w s (T&2), rhs_x = r_d - rxz/x + (rws - r_u s)/w."""
    n, up = lp.n, lp.up[:lp.n]
    rxz = np.where(lp.lo, -st.x * st.z, 0.0)
    rws = np.where(up, -st.w * st.s, 0.0)
    rhs = R["rd"].copy()
    rhs[lp.lo] -= rxz[lp.lo] / st.x[lp.lo]
    rhs[:n][up] += (rws[up] - R["ru"][up] * st.s[up]) / st.w[up]
    return rxz, rws, rhs


def direction(lp, st, fac, rhs_x, rhs_y, ru, rxz, rws):
    """2f59f1: solve (+ 2f9699 refinement in mode 1), then dw = r_u - dx, ds = (rws - s dw)/w (T&2),
    dz = (rxz - z dx)/x (T&1)."""
    n, up = lp.n, lp.up[:lp.n]
    dx, dy = fac.solve(rhs_x, rhs_y)
    if fac.mode1:
        dx, dy, steps = fac.refine(fac.Dtrue, rhs_x, rhs_y, dx, dy, fac.reg)
        fac.reflog.append(steps)
    dw = np.where(up, ru - dx[:n], 0.0)
    ds = np.zeros(n)
    ds[up] = (rws[up] - st.s[up] * dw[up]) / st.w[up]
    dz = np.zeros(lp.m + n)
    dz[lp.lo] = (rxz[lp.lo] - st.z[lp.lo] * dx[lp.lo]) / st.x[lp.lo]
    return {"dx": dx, "dy": dy, "dz": dz, "dw": dw, "ds": ds}


def ratio(v, d, a):
    """2f15ba RT rule, sequential: a shrinks only when the current a crosses the bound of that entry."""
    with np.errstate(divide="ignore", invalid="ignore"):
        hit = a * d + v < 0
        while hit.any():  # same result as the index-order loop: each firing lowers a, later entries re-test
            k = np.argmax(hit)
            a = -(v[k] + -1e-30) / d[k] - 1e-13
            v, d = v[k + 1:], d[k + 1:]
            hit = a * d + v < 0
    return a


def step_lengths(lp, st, D, om=0.0, D1=None):
    """2f15ba / 2f4fe9 inner test: max step (cap 1.001) along D (+ om*D1) for x, w (primal) and z, s (dual)."""
    n, up = lp.n, lp.up[:lp.n]
    g = (lambda k: D[k] + om * D1[k]) if D1 is not None else (lambda k: D[k])
    ap = ratio(st.x[lp.lo], g("dx")[lp.lo], 1.001)
    ap = max(ratio(st.w[up], g("dw")[up], ap), 0.0)
    ad = ratio(st.z[lp.lo], g("dz")[lp.lo], 1.001)
    ad = max(ratio(st.s[up], g("ds")[up], ad), 0.0)
    return ap, ad


def mu_aff(lp, st, D, ap, ad):
    """2f28c2: complementarity at the trial point / ncomp (floor 1e-30)."""
    up = lp.up[:lp.n]
    S = np.sum((D["dx"][lp.lo] * ap + st.x[lp.lo]) * (D["dz"][lp.lo] * ad + st.z[lp.lo]))
    S += np.sum((D["dw"][up] * ap + st.w[up]) * (D["ds"][up] * ad + st.s[up]))
    return max((st.tau * st.kappa + S) / lp.ncomp, 1e-30)


def update(lp, st, D, alpha_p, alpha_d, it, B):
    """2f2e95: damped step (1-eta) with eta from the worst trial product, then x,z,w,s floored at 1e-30."""
    up, lo = lp.up[:lp.n], lp.lo
    eta0, gam = 1e-6, 0.01
    ap, ad = min(alpha_p * (1 - eta0), 1.0), min((1 - eta0) * alpha_d, 1.0)
    if ap * ad > 5e-5:
        B["f4"] = it + 1
    p = np.concatenate([(D["dx"][lo] * ap + st.x[lo]) * (D["dz"][lo] * ad + st.z[lo]),
                        (D["dw"][up] * ap + st.w[up]) * (D["ds"][up] * ad + st.s[up])])
    mn = p.min(initial=1e100)
    avg = (p.sum() + st.kappa * st.tau) / lp.ncomp
    eta = max(eta0, (gam * eta0 * avg) / mn) if mn < gam * avg else eta0
    eta = min(eta, 0.5)
    ap, ad = min(alpha_p * (1 - eta), 1.0), min((1 - eta) * alpha_d, 1.0)
    x = st.x + ap * D["dx"]
    z = st.z + ad * D["dz"]
    st.x = np.where(lo, np.maximum(x, 1e-30), x)
    st.z = np.where(lo, np.maximum(z, 1e-30), z)
    st.w = np.where(up, np.maximum(st.w + ap * D["dw"], 1e-30), st.w)
    st.s = np.where(up, np.maximum(st.s + ad * D["ds"], 1e-30), st.s)
    st.y = st.y + ad * D["dy"]
    return ap, ad, eta


def corrector_rhs(lp, st, D0, mu_t, k, alpha_p, alpha_d):
    """2d64b4: k=1 Mehrotra (mu_t - dx dz of the affine direction), k>=2 Gondzio box [mu_t/10, 10 mu_t] at the
    trial point with alphas 1.5 a + 0.3 (cap 1.001). Correctors carry no r_p, r_d, r_u terms."""
    n, up, lo = lp.n, lp.up[:lp.n], lp.lo
    rxz, rws = np.zeros(lp.m + n), np.zeros(n)
    if k == 1:
        rxz[lo] = mu_t - D0["dx"][lo] * D0["dz"][lo]
        rws[up] = mu_t - D0["dw"][up] * D0["ds"][up]
    else:
        ap, ad = min(1.5 * alpha_p + 0.3, 1.001), min(1.5 * alpha_d + 0.3, 1.001)

        def box(v):
            return np.where(v < mu_t / 10, mu_t / 10 - v, np.where(v > 10 * mu_t, np.maximum(10 * mu_t - v, -10 * mu_t), 0.0))
        rxz[lo] = box((st.x[lo] + ap * D0["dx"][lo]) * (st.z[lo] + ad * D0["dz"][lo]))
        rws[up] = box((st.w[up] + ap * D0["dw"][up]) * (st.s[up] + ad * D0["ds"][up]))
    rhs = np.zeros(lp.m + n)
    rhs[lo] -= rxz[lo] / st.x[lo]
    rhs[:n][up] = rws[up] / st.w[up] + rhs[:n][up]
    return rxz, rws, rhs


def line_search(lp, st, D0, D1, k, N, alpha_p, alpha_d):
    """2f4fe9: grid over omega from 1 down to lo (lo = clamp(ap ad, 0.01, 1) for k=1, 0 for k>=2); primal and dual
    pick their own omega (first strict 1.0001x improvement); set 0 += omega * set 1."""
    lo_ = max(min(alpha_p * alpha_d, 1.0), 0.01) if k < 2 else 0.0
    bp = bd = -1e100
    wp = wd = 0.0
    for i in range(N):
        om = (float(N - 1 - i) * (1 - lo_)) / float(N - 1) + lo_
        if om != 0:
            ap, ad = step_lengths(lp, st, D0, om, D1)
        else:
            ap, ad = alpha_p, alpha_d
        if bp * 1.0001 < ap:
            bp, wp = ap, om
        if bd * 1.0001 < ad:
            bd, wd = ad, om
    for key in ("dx", "dw"):
        D0[key] = D0[key] + wp * D1[key]
    for key in ("dz", "ds", "dy"):
        D0[key] = D0[key] + wd * D1[key]
    return bp, bd, (wp == 0 and wd == 0)


def adjust_K(K, ratio, avg, base):
    """2bf1ea L780-835: corrector count from the work ratio t(extra correctors)/t(factor+pred+first corr+update)."""
    if K < 20:
        if ratio < 0.25:
            return K + 1
        if K < base:
            if ratio < 0.5 and avg < 0.1:
                return K + 1
        elif base < K and ratio > 0.25:
            return K - 1
        if K > 1 and ratio > 0.25 and avg > 0.1:
            return K - 1
        return K
    if K < base:
        if ratio < 0.5 and avg < 0.1:
            return K + 1
        if ratio > 0.25 and avg > 0.1:
            return K - 1
        return K
    if K <= base or ratio <= 0.25:
        if ratio <= 0.25:
            return K
        return K - 1 if avg > 0.1 else K
    return K - 1


def iteration(lp, st, R, ctx):
    """One pass of the 2bf1ea loop body after the termination test: factor (theta), predictor, correctors, update.
    ctx carries B state: it, K (B+0x298), N (B+0x29c), f4 (B+0xf4), ab (B+0x5e0/0x5e8/0x5f0), BarCorrectors."""
    it = ctx["it"]
    th = theta(lp, st, R)
    rxz, rws, rhs = predictor_rhs(lp, st, R)
    fac = Normal(lp, th)
    fac.Dtrue = theta_inv(lp, st)
    fac.reg = max(np.sqrt(R["rp2"] ** 2 + R["rd2"] ** 2) / 10.0, 1e-8)  # B+0x280 (no safe mode)
    D0 = direction(lp, st, fac, rhs, R["rp"], R["ru"], rxz, rws)
    ap, ad = step_lengths(lp, st, D0)
    ma = mu_aff(lp, st, D0, ap, ad)
    compl = R["compl"]
    r = ma / max(compl, 1e-30)
    sigma = r * r * min(r, 0.1)
    mu_t = compl * 0.9 if ctx["f4"] < it else compl * sigma
    K = ctx["K"] if ctx.get("barcorrectors", -1) < 0 else max(1, ctx["barcorrectors"])
    if ctx.get("barcorrectors", -1) >= 0:
        ctx["K"] = K
    ncorr = 0
    Daff = {k: v.copy() for k, v in D0.items()}
    for k in range(1, K + 1):
        cxz, cws, crhs = corrector_rhs(lp, st, Daff if k == 1 else D0, mu_t, k, ap, ad)
        D1 = direction(lp, st, fac, crhs, np.zeros(lp.m), np.zeros(lp.n), cxz, cws)
        nap, nad, none = line_search(lp, st, D0, D1, k, ctx["N"], ap, ad)
        ncorr += 1
        stop = none or (nap >= 1.001 and nad >= 1.001) or (k > 1 and nad * nap < 1e-100)
        if k > max(1, ctx["K"] // 10) and nap * 0.99 * nad < ap * ad:
            stop = True
        ap, ad = nap, nad
        if stop:
            break
    ctx["ncorr_done"] = ncorr
    ctx["reflog"] = fac.reflog if fac.mode1 else []
    ab = ctx["ab"]
    ctx["ab"] = [ap * ad, ab[0], ab[1]]
    fap, fad, eta = update(lp, st, D0, ap, ad, it, ctx)
    ctx["alpha"] = (ap, ad, fap, fad, eta)
    return D0


def corrector_start(lp, F160, F168):
    """2e7f38: initial K (B+0x298) and line-search grid N (B+0x29c) from solve vs factor work."""
    r = (2 * lp.nnz * 2 + F160) / F168
    K = int(min(max(0.1 / r, 1.0), 20.0))
    return K, (3 if r > 0.2 else 5)


def terminate(lp, R, it, H, tol=1e-8, kappa=0.0, tau=1.0):
    """b2aecf for LP with Crossover=0 (B+0x2a0 = 0): returns 2 (optimal), 13 (stalled) or 1 (continue)."""
    N = lp.ncomp
    pobj, dobj, mu, compl = R["pobj"], R["dobj"], R["mu"], R["compl"]
    gap = abs(pobj - dobj)
    grel = max(gap / (abs(pobj - lp.objcon) + 1), gap / (100 * abs(pobj) + 1))
    cg = compl * N / (abs(R["ctx"]) + 1)
    mg = N * mu / (abs(pobj) + 1)
    h78 = H["mu"]
    fast = it > 0 and mu > tol and h78[it] < 0.5 * h78[it - 1]
    pfeas = np.sqrt(R["rp2"] ** 2 + R["ru2"] ** 2) < lp.ptol or R["rp_rel"] < 10 * tol
    dfeas = R["rd2"] < lp.dtol or R["rd_rel"] < 10 * tol
    kc = kappa < 100 * tol * tau
    if np.isnan(mu) or mu > 1e100 or (it > 5 and compl > 1e20 and compl > 0.99 * H["compl"][it - 1]):
        return 13
    if ((grel < tol and mg < 100 * tol and pfeas and dfeas and kc and not fast)
            or (cg < tol and mg < 100 * tol and grel < 0.1 and pfeas and dfeas and not fast)
            or (grel < tol and cg < tol and pfeas and dfeas and it > 1 and h78[it] > 0.5 * h78[it - 1])
            or (mu < 1e-16 and pfeas and dfeas)):
        return 2
    if it > 10 and kc and grel < 10 * tol and pfeas and dfeas and compl > 0.8 * H["compl"][it - 3]:
        return 2
    if it >= 1000:
        return 7
    return 1  # shortcut: stall rules (status 13, b2aecf L180-285) not ported yet


def run(lp, F160, F168, barcorrectors=-1, maxit=1000, log=None):
    """Whole barrier loop (2bf1ea L455-1030) without homogeneous mode. Returns (status, iterations, trace)."""
    m, n = lp.m, lp.n
    nub = int(np.flatnonzero(lp.ub < 1e30).max() + 1 - np.flatnonzero(lp.ub < 1e30).min()) if (lp.ub < 1e30).any() else 0
    w_solve = (2 * lp.nnz * 2 + F160 + 11 * m + 4 * n) * 0.75  # 2ee6c5 incl. 2d3b3e, 2d3d91
    w_dir = w_solve + (5 * (m + n) + 9 * nub) * 0.75             # 2f59f1
    w_ratio = (7 * (m + n) + 6 * nub) * 0.75                    # 2f15ba, one 2f4fe9 grid point
    w_crhs = (3 * (m + nub) + 13 * n) * 0.75                     # 2d64b4
    T = {"f": 1 + 0.75 * F168, "p": 0.0, "c1": 0.0, "cx": 0.0, "u": 0.0}  # timers: work + 1 per start
    K, N = corrector_start(lp, F160, F168)
    st = initial_point(lp)
    ctx = dict(K=K, N=N, f4=0, ab=[1e100] * 3, barcorrectors=barcorrectors)
    H = {k: [] for k in ("mu", "compl", "rp2", "rd2")}
    trace = []
    base = max(1, int(np.floor(np.log10(max(n, 1)))) - 2)
    status = 1
    for it in range(maxit + 1):
        ctx["it"] = it
        R = residuals(lp, st)
        if it >= 11 and not H["mu"][it - 1] <= H["mu"][it - 2] * 0.99:
            recenter(lp, st, R["rd"], R["compl"])
        trace.append((it, R["pobj"], R["dobj"], R["rp2"], R["rd2"], R["compl"], ctx["K"], dict(T)))
        if log:
            log(trace[-1])
        H["mu"].append(R["mu"])
        H["compl"].append(R["compl"])
        H["rp2"].append(R["rp2"])
        H["rd2"].append(R["rd2"])
        status = terminate(lp, R, it, H)
        if status != 1:
            break
        T["f"] += 1 + 0.75 * F168
        Kcur, ab = ctx["K"], ctx["ab"]
        Ngrid = ctx["N"]
        iteration(lp, st, R, ctx)
        nc = ctx["ncorr_done"]
        rl = ctx["reflog"]
        w_ref = [1.5 * 2 * lp.nnz + k * (w_solve + 1.5 * 2 * lp.nnz) for k in rl] or [0.0] * (nc + 1)  # 2f9699
        T["p"] += 1 + w_dir + w_ratio + w_ref[0]
        wc = 1 + w_crhs + w_dir
        T["c1"] += (wc + w_ratio * Ngrid + w_ref[1]) if nc >= 1 else 0
        T["cx"] += sum(wc + w_ratio * (Ngrid - 1) + w_ref[k] for k in range(2, nc + 1))  # k>=2: omega=0 skips RT
        if barcorrectors < 0:
            avg = 1.0 if it < 3 else (ab[0] + ab[1] + ab[2]) * 0.3333333333333333
            ratio = T["cx"] / (T["f"] + T["p"] + T["c1"] + T["u"])
            ctx["K"] = adjust_K(Kcur, ratio, avg, base)
        ctx["N"] = max(3, min(ctx["K"] + 1, 10))
        T["u"] += 1
        if ctx["f4"] < it and barcorrectors == -1:
            ctx["K"] = max(ctx["K"], 10) if ctx["K"] >= 11 else 10
            ctx["N"] = 10
    return status, it, trace


if __name__ == "__main__":
    sys.exit("prototype driver not written yet")
