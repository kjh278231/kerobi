import gzip, json, sys, glob, os
sys.path.insert(0, "/home/sds/workspace/gurobi-sandbox/rsolver/tools/proto")
H = float.fromhex
src = open("/home/sds/workspace/gurobi-sandbox/rsolver/tools/proto/lu_singleton.py").read().split("ok = full = tot = 0")[0]
exec(src)

def btran(m, n, beg, ind, val, head, steps, prow, cb):
    """y (by row) with B^T y = cb (by basis position)."""
    col = lambda j: [(ind[k], val[k]) for k in range(beg[j], beg[j + 1])] if j < n else [(j - n, 1.0)]
    v = [0.0] * m  # indexed by row of the step
    w = list(cb)   # rhs by basis position
    for k, (r, pos, d) in enumerate(steps):            # U^T forward
        t = w[pos] / d
        if abs(t) <= ZERO: t = 0.0
        v[r] = t
        if t != 0.0:
            for i, a in col(head[pos]):                 # u entries of later columns in row r: scatter via columns
                pass
    # U^T via rows: for step k, U row r_k holds entries of columns pivoted later; rebuild per row
    return None

ok = tot = 0
for f in sorted(glob.glob(os.path.join(os.path.dirname(os.path.abspath(__file__)), "../../bench/oracle/iter0/*.jsonl.gz"))):
    name = os.path.basename(f)[:-9]
    if name in ("capri", "perold", "pilot4", "tuff"): continue
    o = json.loads(gzip.open(f, "rt").readline()); n, m = o["n"], o["m"]
    beg, ind, val = o["beg"], o["ind"], [H(v) for v in o["val"]]
    c = [H(v) for v in o["c"]]; vs = o["vstat"]
    head = [None] * m
    for j, s in enumerate(vs):
        if s >= 0: head[s] = j
    steps, prow = singleton_order(m, n, beg, ind, val, head)
    col = lambda j: [(ind[k], val[k]) for k in range(beg[j], beg[j + 1])] if j < n else [(j - n, 1.0)]
    step_of_pos = {pos: k for k, (r, pos, d) in enumerate(steps)}
    # U^T solve: process steps forward; z_k = (c_B[pos_k] - sum_{earlier steps i in column} ...) -- scatter form:
    w = [c[head[p]] for p in range(m)]        # by position
    y = [0.0] * m                              # by row
    # U row of row r_k = entries a_{r_k, j} of columns j pivoted at later steps (prow[r_k] < step(j))
    urow = [[] for _ in range(m)]
    for k, (r, pos, d) in enumerate(steps):
        for i, a in col(head[pos]):
            if prow[i] < k: urow[i].append((pos, a))
    for k, (r, pos, d) in enumerate(steps):
        t = w[pos] / d
        if abs(t) <= ZERO: t = 0.0
        y[r] = t
        if t != 0.0:
            for p2, a in urow[r]: w[p2] = -(a * t) + w[p2]
    # L^T backward: y[r_k] -= sum over L column k entries (i > k): l_ik * y[i]  (scatter-free dot product, entry order)
    for k in range(len(steps) - 1, -1, -1):
        r, pos, d = steps[k]
        s = y[r]
        for i, a in col(head[pos]):
            if prow[i] > k: s = s - (a / d) * y[i]
        y[r] = s
    dd = []
    for j in range(n + m):
        if vs[j] >= 0: dd.append(0.0); continue
        if vs[j] == -4 and j < n: dd.append(c[j]); continue  # fixed structurals are not in the row-wise nonbasic copy
        t = c[j]
        for i, a in col(j): t = t - a * y[i]
        dd.append(-t if vs[j] == -2 else t)
    g = [H(v) for v in o["d"]]
    same = sum(a == b for a, b in zip(dd, g))
    tot += 1; ok += same == len(g)
    if same != len(g): print(f"{name}: d bits {same}/{len(g)}")
print(f"d bit-exact {ok}/{tot}")
