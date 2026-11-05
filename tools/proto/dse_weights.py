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

res = {"dense8": 0, "sparse_step8": 0}; tot = 0
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
        def sum8(v):
            w = 0.0; k = len(v)
            if k <= 6:
                for t in v: w = w + t * t
                return w
            nb = k - (k % 8)
            acc = [w, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]
            for b in range(0, nb, 8):
                for q in range(8): acc[q] = acc[q] + v[b + q] * v[b + q]
            if nb:
                w = ((acc[0] + acc[2]) + acc[4]) + acc[6]
                w = w + (((acc[1] + acc[3]) + acc[5]) + acc[7])
            for t in v[nb:]: w = w + t * t
            return w
        dense = sum8(y)
        sparse = sum8([y[r] for r, pos, d in steps if y[r] != 0.0])
        w_row.append(dense); w_step.append(sparse)
    tot += 1
    res["dense8"] += w_row == g; res["sparse_step8"] += w_step == g
    if w_row != g and w_step != g:
        print(name, "dense8", sum(a == b for a, b in zip(w_row, g)), "sparse8", sum(a == b for a, b in zip(w_step, g)), "of", m)
print(res, "of", tot)
