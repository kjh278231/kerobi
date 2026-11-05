import json, sys, glob, os
H = float.fromhex
exec(open("/home/sds/workspace/gurobi-sandbox/rsolver/tools/proto/lu_singleton.py").read().split("ok = full = tot = 0")[0])

def btran_vec(m, n, beg, ind, val, head, steps, prow, urow, w):
    col = lambda j: [(ind[k], val[k]) for k in range(beg[j], beg[j + 1])] if j < n else [(j - n, 1.0)]
    w = list(w); y = [0.0] * m
    for k, (r, pos, d) in enumerate(steps):
        t = w[pos] / d
        if abs(t) <= ZERO: t = 0.0
        y[r] = t
        if t != 0.0:
            for p2, a in urow[r]: w[p2] = -(a * t) + w[p2]
    for k in range(len(steps) - 1, -1, -1):
        r, pos, d = steps[k]
        s = y[r]
        for i, a in col(head[pos]):
            if prow[i] > k: s = s - (a / d) * y[i]
        y[r] = s
    return y

res = {"row_order": 0, "two_lane": 0}; tot = 0
for f in sorted(glob.glob(sys.argv[1] + "/or0/*.jsonl")):
    name = os.path.basename(f)[:-6]
    o = json.loads(open(f).readline()); n, m = o["n"], o["m"]
    if name in ("capri", "perold", "pilot4", "tuff") or m > 900: continue
    beg, ind, val = o["beg"], o["ind"], [H(v) for v in o["val"]]; vs = o["vstat"]
    head = [None] * m
    for j, s in enumerate(vs):
        if s >= 0: head[s] = j
    steps, prow = singleton_order(m, n, beg, ind, val, head)
    col = lambda j: [(ind[k], val[k]) for k in range(beg[j], beg[j + 1])] if j < n else [(j - n, 1.0)]
    urow = [[] for _ in range(m)]
    for k, (r, pos, d) in enumerate(steps):
        for i, a in col(head[pos]):
            if prow[i] < k: urow[i].append((pos, a))
    g = [H(x) for x in o["w"]]
    w_row = []; w_step = []
    for p in range(m):
        e = [0.0] * m; e[p] = 1.0
        y = btran_vec(m, n, beg, ind, val, head, steps, prow, urow, e)
        s1 = 0.0
        for v in y: s1 += v * v
        a0 = a1 = 0.0
        for i in range(0, m - 1, 2): a0 += y[i] * y[i]; a1 += y[i + 1] * y[i + 1]
        s2 = a0 + a1
        if m % 2: s2 += y[m - 1] * y[m - 1]
        w_row.append(s1); w_step.append(s2)
    tot += 1
    res["row_order"] += w_row == g; res["two_lane"] += w_step == g
    if w_row != g and w_step != g:
        print(name, "row-order bits", sum(a == b for a, b in zip(w_row, g)), "two-lane", sum(a == b for a, b in zip(w_step, g)), "of", m)
print(res, "of", tot)
