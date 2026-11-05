"""Prototype of Gurobi's supernodal LDL^T of A Theta A^T (docs/m3_chol.md), checked bit for bit against the factor
oracle (tools/bar_oracle.py --chol).

Order of operations (2d8ee5 for one block c of width w, rows R = F+0xb0[c], len = |R|):
  1. assembly (32d72f if w < 3 or w*len > 30000, else 32e116) into a dense w x w row-major lower buffer and the
     (len-w) x w row-major rectangle; F+0xf0 = the buffer diagonal after assembly
  2. updates 32c49d from each descendant block (F+0x88/0x90/0x98 list, then earlier blocks of the same supernode):
     A = -(L o d) of the rows inside c, DGEMM('T','N') with the stored L rows, direct or through a scratch + scatter
  3. dense left-looking LDL^T of the buffer (8-lane dot products), pivot rule |d| <= 1e-16 |F+0xf0| -> d = 0, 1/d = 0
  4. DTRSM('L','U','T','U') of the rectangle with the unit L11, then rectangle *= 1/d (1 where d was dropped)

usage: python tools/proto/bar_chol.py <oracle.jsonl.gz> [iteration ...]   (MKL is called through libgurobi130.so)
"""
import ctypes, gzip, json, math, sys

LIB = "/home/sds/workspace/gurobi-sandbox/.venv/lib/python3.13/site-packages/gurobipy/.libs/libgurobi130.so"
_lib = ctypes.CDLL(LIB)
_BASE = next(int(ln.split("-")[0], 16) for ln in open("/proc/self/maps")
             if "libgurobi130.so" in ln and ln.split()[2] == "00000000")
VP = ctypes.c_void_p
_DGEMM = ctypes.CFUNCTYPE(None, *[VP] * 13)(_BASE + 0x9ffc30)
_DTRSM = ctypes.CFUNCTYPE(None, *[VP] * 11)(_BASE + 0x9ff5e0)
_p = lambda x: ctypes.cast(ctypes.pointer(x), VP)
DA = ctypes.c_double


def buf(n):
    """64-byte aligned double buffer (Gurobi's scratch is at least 16-byte aligned)."""
    raw = (DA * (n + 8))()
    off = (-ctypes.addressof(raw) % 64) // 8
    a = (DA * n).from_address(ctypes.addressof(raw) + off * 8)
    a._keep = raw
    return a


def at(a, k):
    return ctypes.cast(ctypes.addressof(a) + 8 * k, VP)


def gemm_tn(m, n, k, A, a0, lda, B, b0, ldb, C, c0, ldc):
    _DGEMM(_p(ctypes.c_char(b"T")), _p(ctypes.c_char(b"N")), _p(ctypes.c_int(m)), _p(ctypes.c_int(n)),
           _p(ctypes.c_int(k)), _p(DA(1.0)), at(A, a0), _p(ctypes.c_int(lda)), at(B, b0), _p(ctypes.c_int(ldb)),
           _p(DA(1.0)), at(C, c0), _p(ctypes.c_int(ldc)))


def trsm_lutu(m, n, A, a0, lda, B, b0, ldb):
    _DTRSM(_p(ctypes.c_char(b"L")), _p(ctypes.c_char(b"U")), _p(ctypes.c_char(b"T")), _p(ctypes.c_char(b"U")),
           _p(ctypes.c_int(m)), _p(ctypes.c_int(n)), _p(DA(1.0)), at(A, a0), _p(ctypes.c_int(lda)), at(B, b0),
           _p(ctypes.c_int(ldb)))


PEEL = 0


def dot8(acc, x, y, n, peel=None):
    """acc + sum_{p<n} x[p]*y[p] as 2d8ee5 does it (SSE2, no fma): `peel` leading terms sequentially (alignment of the
    scratch vector), sequential when fewer than peel + 8 terms, else 8 lanes (4 xmm pairs) with lane 0 seeded by acc,
    reduced ((l0+l2)+(l4+l6)) + ((l1+l3)+(l5+l7)) (asm 0x2f2290), then the tail sequentially."""
    peel = PEEL if peel is None else peel
    if n < 7 or n < peel + 8:
        for p in range(n):
            acc = acc + x[p] * y[p]
        return acc
    for p in range(peel):
        acc = acc + x[p] * y[p]
    nv = n - ((n - peel) & 7)
    lane = [acc] + [0.0] * 7
    for p in range(peel, nv):
        lane[(p - peel) & 7] = lane[(p - peel) & 7] + x[p] * y[p]
    s = ((lane[0] + lane[2]) + (lane[4] + lane[6])) + ((lane[1] + lane[3]) + (lane[5] + lane[7]))
    for p in range(nv, n):
        s = s + x[p] * y[p]
    return s


class Chol:
    def __init__(self, st, A):
        """st: oracle static structure (snode, blocks, col_len, rows, f88, f90, f98, F80); A: dict with m, n, beg, ind,
        val (columns, rows ascending), sense."""
        self.m = m = A["m"]
        self.A = A
        self.blocks, self.clen, self.snode = st["blocks"], st["col_len"], st["snode"]
        self.rows = {int(c): r for c, r in st["rows"].items()}
        self.f88, self.f90, self.f98, self.F80 = st["f88"], st["f90"], st["f98"], st["F80"]
        self.starts = [c for c in range(m) if self.blocks[c] > 0]
        n = A["n"]
        # row-wise A (columns ascending) and, per row-wise entry, its position in the column arrays (B+0x120)
        self.rbeg = [[] for _ in range(m)]
        for j in range(n):
            for p in range(A["beg"][j], A["beg"][j + 1]):
                self.rbeg[A["ind"][p]].append((j, p))

    def factor(self, theta):
        m, A, bl = self.m, self.A, self.blocks
        n = A["n"]
        self.d, self.e, self.diag0 = [0.0] * m, [0.0] * m, [0.0] * m
        self.L = {}
        self.min_ratio = 1e100
        for c in self.starts:
            self.block(c, theta, n)
        return self.min_ratio

    def assemble(self, c, theta, n):
        A, w, R = self.A, self.blocks[c], self.rows[c]
        ln = len(R)
        mp = {r: q for q, r in enumerate(R)}
        D = [0.0] * (w * w)
        rect = [0.0] * ((ln - w) * w)
        beg, ind, val = A["beg"], A["ind"], A["val"]

        def add(i, k, v):  # M[i, k] += v, i >= k (rows of the block)
            qi, qk = mp[i], mp[k]
            if qi < w:
                D[qi * w + qk] = D[qi * w + qk] + v
            else:
                rect[(qi - w) * w + qk] = rect[(qi - w) * w + qk] + v

        if w < 3 or w * ln > 30000:
            # 32d72f: column k of the block, columns j of row k in order, all rows i >= k of column j
            for k in range(c, c + w):
                for j, p in self.rbeg[k]:
                    t = theta[j] * val[p]
                    for q in range(p, beg[j + 1]):
                        if ind[q] in mp:
                            add(ind[q], k, val[q] * t)
                if A["sense"][k] != "=":
                    add(k, k, theta[n + k])
        else:
            # 32e116: rows k of the block in order; a column is processed at its first row >= c
            for k in range(c, c + w):
                for j, p in self.rbeg[k]:
                    if p == beg[j] or ind[p - 1] < c:
                        e = p
                        while e < beg[j + 1] and mp.get(ind[e], w) < w:
                            e += 1
                        for a in range(p, e):
                            t = val[a] * theta[j]
                            for q in range(a, beg[j + 1]):
                                if ind[q] in mp:
                                    add(ind[q], ind[a], val[q] * t)
                if A["sense"][k] != "=":
                    add(k, k, theta[n + k])
        return D, rect, mp

    def update(self, s, p0, t, D, rect, mp):
        """32c49d: block s (factored) into target t (buffer D, rectangle rect, row map mp)."""
        ws, Rs, Ls = self.blocks[s], self.rows[s], self.L[s]
        lens, wt = len(Rs), self.blocks[t]
        tri = ws * (ws + 1) // 2
        Lrow = lambda q: tri + (q - ws) * ws  # offset of rectangle row (position q >= ws) in Ls
        p1 = p0 + 1
        while p1 < lens and Rs[p1] < t + wt:
            p1 += 1
        nr = p1 - p0
        ds = self.d[s:s + ws]
        if ws == 1:
            # scalar path: target w < 5 multiplies L[row] by -(L[col] d), w >= 5 multiplies L[col] by -(L[row] d)
            for r in range(p0, p1):
                col = mp[Rs[r]]
                for q in range(r, lens):
                    g = mp[Rs[q]]
                    v = Ls[q] * (-Ls[r] * ds[0]) if wt < 5 else Ls[r] * (-Ls[q] * ds[0])
                    if g < wt:
                        D[g * wt + col] = v + D[g * wt + col]
                    else:
                        rect[(g - wt) * wt + col] = v + rect[(g - wt) * wt + col]
            return
        tmpA = buf(nr * ws)
        for i in range(nr):
            o = Lrow(p0 + i)
            for q in range(ws):
                tmpA[i * ws + q] = -Ls[o + q] * ds[q]
        Lb = self.Lbuf[s]
        j0 = mp[Rs[p0]]
        contig = mp[Rs[p1 - 1]] == j0 + nr - 1
        if contig:
            Db = buf(wt * wt)
            Db[:] = D
            for a in range(0, nr, 24):
                k = min(24, nr - a)
                gemm_tn(a + k, k, ws, tmpA, 0, ws, Lb, Lrow(p0 + a), ws, Db, j0 * wt + j0 + a * wt, wt)
            D[:] = Db[:]
        else:
            C = buf(nr * nr)
            for a in range(0, nr, 24):
                k = min(24, nr - a)
                gemm_tn(a + k, k, ws, tmpA, 0, ws, Lb, Lrow(p0 + a), ws, C, a * nr, nr)
            for r in range(nr):
                g = mp[Rs[p0 + r]]
                for q in range(r + 1):
                    D[g * wt + mp[Rs[p0 + q]]] = D[g * wt + mp[Rs[p0 + q]]] + C[r * nr + q]
        a = p1
        while a < lens:
            b = min(a + min(self.F80 // nr, 192), lens)
            g0 = mp[Rs[a]]
            if contig and mp[Rs[b - 1]] == g0 + (b - a) - 1:
                while b < lens and mp[Rs[b]] == g0 + (b - a):
                    b += 1
                Rb = buf(len(rect))
                Rb[:] = rect
                gemm_tn(nr, b - a, ws, tmpA, 0, ws, Lb, Lrow(a), ws, Rb, (g0 - wt) * wt + j0, wt)
                rect[:] = Rb[:]
            else:
                C = buf(nr * (b - a))
                gemm_tn(nr, b - a, ws, tmpA, 0, ws, Lb, Lrow(a), ws, C, 0, nr)
                for r in range(a, b):
                    g = mp[Rs[r]]
                    for q in range(p0, p1):
                        o = (g - wt) * wt + mp[Rs[q]]
                        rect[o] = rect[o] + C[(r - a) * nr + (q - p0)]
            a = b

    def block(self, c, theta, n):
        w, R = self.blocks[c], self.rows[c]
        ln = len(R)
        D, rect, mp = self.assemble(c, theta, n)
        for q in range(w):
            self.diag0[c + q] = D[q * w + q]
        for u in range(self.f88[c], self.f88[c + 1]):
            s0 = self.f90[u]
            s = s0
            while s < s0 + self.snode[s0]:
                self.update(s, self.f98[u] + s0 - s, c, D, rect, mp)
                s += self.blocks[s]
        s = c + min(self.snode[c], 0)
        while s != c:
            self.update(s, c - s, c, D, rect, mp)
            s += self.blocks[s]
        dense = any(D[i * w + k] != 0.0 for i in range(w) for k in range(i))
        d0 = self.diag0
        for j in range(w):
            if dense:
                wv = [-D[j * w + p] * self.d[c + p] for p in range(j)]
                for i in range(j, w):
                    D[i * w + j] = dot8(D[i * w + j], D[i * w:i * w + j], wv, j)
                    if i > j:
                        D[i * w + j] = D[i * w + j] * self.e[c + j]
                    else:
                        self.pivot(c + j, D[j * w + j])
                        D[j * w + j] = self.d[c + j]
            else:
                self.pivot(c + j, D[j * w + j])
                D[j * w + j] = self.d[c + j]
        if dense and ln > w:
            Db, Rb = buf(w * w), buf(len(rect))
            Db[:], Rb[:] = D, rect
            trsm_lutu(w, ln - w, Db, 0, w, Rb, 0, w)
            rect = list(Rb)
        for r in range(ln - w):
            for q in range(w):
                ev = self.e[c + q]
                rect[r * w + q] = rect[r * w + q] * (ev if ev != 0.0 else 1.0)
        L = []
        for i in range(w):
            L.extend(D[i * w:i * w + i + 1])
        L.extend(rect)
        self.L[c] = L
        self.Lbuf = getattr(self, "Lbuf", {})
        b = buf(len(L))
        b[:] = L
        self.Lbuf[c] = b

    def pivot(self, k, dk):
        d0 = self.diag0[k]
        if math.isnan(dk):
            self.min_ratio = 0.0
        elif d0 != 0.0:
            self.min_ratio = min(self.min_ratio, abs(dk / d0))
        if abs(d0) * 1e-16 < abs(dk):
            self.d[k], self.e[k] = dk, 1.0 / dk
        else:
            self.d[k], self.e[k] = 0.0, 0.0


def load(path):
    recs = [json.loads(ln) for ln in gzip.open(path, "rt")]
    r0 = recs[0]
    Dm = r0["D"]
    h = lambda v: [float.fromhex(x) for x in v]
    m, n = Dm["m"], Dm["n"]
    beg = [b for b in Dm["beg"]] + [Dm["beg"][-1] + Dm["len"][-1]]
    A = {"m": m, "n": n, "beg": beg, "ind": Dm["ind"], "val": h(Dm["val"]), "sense": Dm["sense"]}
    import struct
    F = r0["F"]
    st = dict(r0["chol"])
    st["F80"] = struct.unpack("<q", struct.pack("<d", float.fromhex(F[0x80 // 8])))[0]
    return recs, A, st


def theta0(r0):
    """2d0663: free 1e-6, boxed 0.5, others 1; slacks 1, '=' rows 0 (type array B+0x640)."""
    n, m, T = r0["n"], r0["m"], r0["type"]
    th = [1e-6 if T[j] == 0 else 0.5 if T[j] == 3 else 1.0 for j in range(n)]
    th += [0.0 if T[n + i] == 0 else 1.0 for i in range(m)]
    return th


def compare(ch, rec):
    c = rec["chol"]
    h = lambda v: [float.fromhex(x) for x in v]
    res = {}
    res["diag0"] = sum(a == b for a, b in zip(ch.diag0, h(c["diag0"]))), ch.m
    res["d"] = sum(a == b for a, b in zip(ch.d, h(c["d"]))), ch.m
    res["e"] = sum(a == b for a, b in zip(ch.e, h(c["e"]))), ch.m
    tot = same = 0
    first = None
    for k, v in c["vals"].items():
        g = h(v)
        mine = ch.L[int(k)]
        for i, (a, b) in enumerate(zip(mine, g)):
            tot += 1
            if a == b:
                same += 1
            elif first is None:
                first = (int(k), i, a, b)
    res["L"] = same, tot
    res["F100"] = ch.min_ratio == float.fromhex(c["F100"])
    res["first_L_diff"] = first
    return res


if __name__ == "__main__":
    recs, A, st = load(sys.argv[1])
    ch = Chol(st, A)
    its = [int(x) for x in sys.argv[2:]] or [0]
    byit = {r["iter"]: r for r in recs}
    for it in its:
        th = theta0(recs[0]) if it == 0 else [float.fromhex(x) for x in byit[it - 1]["theta"]]
        ch.factor(th)
        print(it, compare(ch, byit[it]))
