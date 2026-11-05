"""Singleton-phase LU (1da0b2 phases 1-5) + column-oriented FTRAN; compare xB at iteration 0 with the oracle."""
import gzip, json, sys, glob, os
H = float.fromhex
ZERO = 1e-13

def singleton_order(m, n, beg, ind, val, head):
    rc = [0] * m
    for pos in range(m):
        j = head[pos]
        if j < n:
            for k in range(beg[j], beg[j + 1]): rc[ind[k]] += 1
        else:
            rc[j - n] += 1 + m
    rows = [[] for _ in range(m)]  # row-wise copy for rows with rc <= m: (pos, value), pos ascending
    for pos in range(m):
        j = head[pos]
        if j < n:
            for k in range(beg[j], beg[j + 1]):
                if rc[ind[k]] <= m: rows[ind[k]].append((pos, val[k]))
    prow, ppos, steps = [-1] * m, [-1] * m, []
    def pivot(r, pos, d):
        prow[r] = len(steps); ppos[pos] = len(steps); steps.append((r, pos, d))
    stack = [i for i in range(m) if rc[i] == 1]
    while stack:
        r = stack.pop()
        if rc[r] <= 0: continue
        rc[r] = 0
        e = next(x for x in rows[r] if ppos[x[0]] == -1)
        pos, d = e
        pivot(r, pos, d)
        j = head[pos]
        for k in range(beg[j], beg[j + 1]):
            i = ind[k]
            if prow[i] == -1:
                rc[i] -= 1
                if rc[i] == 1: stack.append(i)
    for pos in range(m):  # slack rows without structural entries
        j = head[pos]
        if j >= n and rc[j - n] == m + 1:
            rc[j - n] = 1; pivot(j - n, pos, 1.0)
    for pos in range(m):  # remaining column singletons and slacks
        if ppos[pos] >= 0: continue
        j = head[pos]
        if j < n:
            if beg[j + 1] - beg[j] == 1:
                i = ind[beg[j]]
                if rc[i] <= m and prow[i] < 0: pivot(i, pos, val[beg[j]])
        else:
            i = j - n; rc[i] -= m; pivot(i, pos, 1.0)
    return steps, prow

def solve(m, n, beg, ind, val, head, steps, prow, rhs):
    x = list(rhs)
    col = lambda j: [(ind[k], val[k]) for k in range(beg[j], beg[j + 1])] if j < n else [(j - n, 1.0)]
    for k, (r, pos, d) in enumerate(steps):  # L, forward
        if abs(x[r]) > ZERO:
            xr = x[r]
            for i, a in col(head[pos]):
                if prow[i] > k: x[i] = x[i] - (a / d) * xr
    for k in range(len(steps) - 1, -1, -1):  # U, backward
        r, pos, d = steps[k]
        if abs(x[r]) <= ZERO or abs(x[r] / d) <= ZERO:
            x[r] = 0.0; continue
        x[r] = x[r] / d
        xr = x[r]
        for i, a in col(head[pos]):
            if prow[i] < k: x[i] = x[i] - a * xr
    xb = [0.0] * m
    for r, pos, d in steps: xb[pos] = x[r]
    return xb

ok = full = tot = 0
for f in sorted(glob.glob(os.path.join(os.path.dirname(os.path.abspath(__file__)), "../../bench/oracle/iter0/*.jsonl.gz"))):
    name = os.path.basename(f)[:-9]
    o = json.loads(gzip.open(f, "rt").readline()); n, m = o["n"], o["m"]
    beg, ind, val = o["beg"], o["ind"], [H(v) for v in o["val"]]
    b = [H(v) for v in o["b"]]; l = [H(v) for v in o["l"]]; u = [H(v) for v in o["u"]]; vs = o["vstat"]
    head = [None] * m
    for j, s in enumerate(vs):
        if s >= 0: head[s] = j
    steps, prow = singleton_order(m, n, beg, ind, val, head)
    tot += 1
    if len(steps) < m: continue
    full += 1
    rhs = list(b)
    for j in range(n + m):
        s = vs[j]
        if s >= 0: continue
        xj = u[j] if s == -2 else l[j] if s in (-1, -4) else 0.0
        if xj == 0.0: continue
        for i, a in ([(ind[k], val[k]) for k in range(beg[j], beg[j + 1])] if j < n else [(j - n, 1.0)]):
            rhs[i] = rhs[i] - a * xj
    xb = solve(m, n, beg, ind, val, head, steps, prow, rhs)
    g = [H(v) for v in o["xB"]]
    same = sum(a == c for a, c in zip(xb, g))
    ok += same == m
    if same != m: print(f"{name}: triangular, xB bits {same}/{m}")
print(f"triangular after singleton phase: {full}/{tot}; xB bit-exact: {ok}/{full}")
