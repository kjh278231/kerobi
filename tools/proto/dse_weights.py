"""Iteration-0 DSE weights w_p = ||e_p^T B^-1||^2 (EXACTNORM, 1bae9c L1971-2110), bit-exact against the oracle.

BTRAN of e_p (221173) on the singleton LU: sparse U^T (21a941, ascending step), then sparse L^T in step space:
heap path (descending step) when LU+0x140 = -1, else level path (036ade L5850-5912 levels, buckets in insertion
order, L^T rows in descending step). Output is sparse, by row, in processing order; 1bae9c sums squares in that
order with 8 accumulators combined ((0+2)+(4+6)) + ((1+3)+(5+7)) (asm 0xc3a26).
usage: python tools/proto/dse_weights.py  (reads bench/oracle/iter0, from tools/spx_oracle.py)"""
import gzip, glob, heapq, json, os, sys
H = float.fromhex
exec(open(os.path.join(os.path.dirname(os.path.abspath(__file__)), "lu_singleton.py")).read().split("ok = full = tot = 0")[0])


def factor_views(m, n, beg, ind, val, head):
    steps, prow = singleton_order(m, n, beg, ind, val, head)
    col = lambda j: [(ind[k], val[k]) for k in range(beg[j], beg[j + 1])] if j < n else [(j - n, 1.0)]
    kpos = [0] * m
    for k, (r, pos, d) in enumerate(steps): kpos[pos] = k
    urow = [[] for _ in range(m)]  # U row of step k: (later step, a)
    lrow = [[] for _ in range(m)]  # L^T row of step k: (earlier step j, a/d_j), descending j
    for k in range(m - 1, -1, -1):
        r, pos, d = steps[k]
        for i, a in col(head[pos]):
            if prow[i] > k: lrow[prow[i]].append((k, a / d))
    for k, (r, pos, d) in enumerate(steps):
        for i, a in col(head[pos]):
            if prow[i] < k: urow[prow[i]].append((k, a))
    lnnz = sum(map(len, lrow))
    t = (lnnz / m + 1.0) * (lnnz / m + 1.0) * 5.0
    sparse_mult = 2 if lnnz <= m else m if m <= t else int(t)  # LU+0x104 (036ade L5709)
    lev = None
    if m >= 250 and sparse_mult < m:  # LU+0x140 >= 0 (036ade L5757)
        lev = [0] * m
        for k in range(m - 1, -1, -1):
            for j, _ in lrow[k]: lev[j] = max(lev[j], lev[k] + 1)
    return steps, kpos, urow, lrow, sparse_mult, lev


def btran_unit(p, m, steps, kpos, urow, lrow, sparse_mult, lev):
    """[(row, value)] of e_p^T B^-1 in Gurobi's output order."""
    w = {kpos[p]: 1.0}; hp = [kpos[p]]; out_u = []
    while hp:  # U^T, 21a941 path for LU+0x108 <= 5
        k = heapq.heappop(hp)
        t = w.pop(k) / steps[k][2]
        if not ZERO < abs(t): continue
        out_u.append((k, t))
        for k2, a in urow[k]:
            if k2 not in w: w[k2] = 0.0; heapq.heappush(hp, k2)
            w[k2] = -(a * t) + w[k2]
    # shortcut: dense L^T (221173 L1576, output nnz = -1) and the linear-scan path (L956) are not emulated; at iteration 0
    # U is diagonal so U^T gives one entry. Port them when a refactored basis has a Markowitz part.
    assert sparse_mult * len(out_u) <= m
    x = dict(out_u); out = []
    if lev is None:
        hp = [-k for k, _ in out_u]; heapq.heapify(hp)
        while hp:
            k = -heapq.heappop(hp); v = x.pop(k)
            if not abs(v) >= ZERO: continue
            out.append((steps[k][0], v))
            for j, l in lrow[k]:
                if j not in x: x[j] = 0.0; heapq.heappush(hp, -j)
                x[j] = x[j] - l * v
    else:
        buckets = {}
        for k, _ in out_u: buckets.setdefault(lev[k], []).append(k)
        L = min(buckets, default=0)
        while buckets:
            for k in buckets.pop(L, []):  # fills land on higher levels only
                v = x.pop(k)
                if not abs(v) >= ZERO: continue
                out.append((steps[k][0], v))
                for j, l in lrow[k]:
                    if j not in x: x[j] = 0.0; buckets.setdefault(lev[j], []).append(j)
                    x[j] = -(l * v) + x[j]
            L += 1
    return out


def sum_squares(v):
    """1bae9c: <= 7 values in sequence, else 8 lanes over the 8-aligned head then the tail (buffer 16-byte aligned)."""
    if len(v) < 8:
        w = 0.0
        for t in v: w = w + t * t
        return w
    nb = len(v) - len(v) % 8
    a = [0.0] * 8
    for b in range(0, nb, 8):
        for q in range(8): a[q] = a[q] + v[b + q] * v[b + q]
    w = ((a[0] + a[2]) + (a[4] + a[6])) + ((a[1] + a[3]) + (a[5] + a[7]))
    for t in v[nb:]: w = w + t * t
    return w


if __name__ == "__main__":
    ok = tot = 0
    for f in sorted(glob.glob(os.path.join(os.path.dirname(os.path.abspath(__file__)), "../../bench/oracle/iter0/*.jsonl.gz"))):
        name = os.path.basename(f)[:-9]
        if name in ("capri", "perold", "pilot4", "tuff"): continue  # iteration-0 snapshot taken before x_B exists
        o = json.loads(gzip.open(f, "rt").readline()); n, m = o["n"], o["m"]
        beg, ind, val = o["beg"], o["ind"], [H(v) for v in o["val"]]
        head = [None] * m
        for j, s in enumerate(o["vstat"]):
            if s >= 0: head[s] = j
        views = factor_views(m, n, beg, ind, val, head)
        w = [sum_squares([v for _, v in btran_unit(p, m, *views)]) for p in range(m)]
        g = [H(x) for x in o["w"]]
        same = sum(a == b for a, b in zip(w, g))
        tot += 1; ok += same == m
        if same != m: print(f"{name}: w bits {same}/{m}")
    print(f"iteration-0 DSE weights bit-exact: {ok}/{tot}")
