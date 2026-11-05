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

    def __init__(self, A, b, c, lb, ub, T, eq, objscale=1.0, objcon=0.0, b0=None, ncomp=None, cshift=0.0):
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
              cshift=float.fromhex(rec["B"][0x618 // 8]))


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


def solve_aug(lp, th, rx, ry):
    """2ee6c5: -Theta^{-1} dx + A^T dy = rx, A dx = ry  =>  (A Theta A^T) dy = ry + A Theta rx, dx = Theta (A^T dy - rx).
    shortcut: dense Cholesky/LU without Gurobi's pivot regularization (32af6a), fine for Netlib sizes."""
    n = lp.n
    Ath = lp.A * th[:n]
    M = Ath @ lp.A.T
    M[np.diag_indices_from(M)] += np.where(lp.eq, 0.0, th[n:])
    rhs = ry + lp.Ax(th * rx)
    try:
        dy = np.linalg.solve(M, rhs)
        if not np.all(np.isfinite(dy)):
            raise np.linalg.LinAlgError
    except np.linalg.LinAlgError:  # dependent rows: any solution gives the same dx; take the least-norm dy
        dy = np.linalg.lstsq(M, rhs, rcond=1e-14)[0]
    dx = th * (lp.ATy(dy) - rx)
    return dx, dy


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
    st.x, _ = solve_aug(lp, th, rx, lp.b)
    st.w = np.where(lp.up[:n], lp.ub - st.x[:n], 0.0)
    dual_from_c(lp, st)
    da = shifts(lp, st)
    if barstart != 1:
        v, yb = solve_aug(lp, th, np.concatenate([lp.c, np.zeros(m)]), np.zeros(m))
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


if __name__ == "__main__":
    sys.exit("prototype driver not written yet")
