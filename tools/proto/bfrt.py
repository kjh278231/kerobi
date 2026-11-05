"""BFRT (1c35c5) prototype checked against bench/oracle/bfrt_io (Gurobi inputs/outputs per call).
usage: python tools/proto/bfrt.py [-v] [name ...]   reports calls whose flip list or entering column differ."""
import gzip, json, sys
from pathlib import Path
D = Path(__file__).resolve().parents[2] / "bench/oracle/bfrt_io"
OPT, PIV = 1e-6, 3e-8


def quickselect(cand, run):
    """Hoare partitions (pivot = first, ratio descending, contribution ascending) narrowing to the crossing."""
    lo, hi = 0, len(cand) - 1
    while lo < hi:
        pr, pc = cand[lo][2], cand[lo][1]
        i, j = hi + 1, lo - 1
        while True:
            i -= 1
            while cand[i][2] < pr or (cand[i][2] == pr and cand[i][1] > pc):
                i -= 1
            j += 1
            while cand[j][2] > pr or (cand[j][2] == pr and cand[j][1] < pc):
                j += 1
            if i <= j:
                break
            if cand[j][2] < cand[i][2] or cand[i][1] < cand[j][1]:
                cand[i], cand[j] = cand[j], cand[i]
        if run >= 0:
            run2 = run - sum(c[1] for c in cand[i + 1:hi + 1])
            if run2 >= 0:
                hi, run = i, run2
            else:
                lo, run = i + 1, run2
        else:
            run2 = run + sum(c[1] for c in cand[lo:i + 1])
            if run2 >= 0:
                hi, run = i, run2
            else:
                lo, run = i + 1, run2
    return lo, run


def bfrt(e, lens=None):
    p2, p3, (r, jl, x, lr, ur) = e[2], e[3], e[4]
    cnt, idx, val = e[5]
    d, vs, l, u, big = e[6], e[7], e[8], e[9], e[10]
    feas = 1e-6 * 1e30 * 1e-5 if big else 1e-6
    delta = (ur - x) if p3 else (x - lr)
    bound, cand, best = -1e100, [], -1
    order = list(range(p2, cnt)) + list(range(0, p2))
    amin = 0.0
    for t in order:
        a = val[t]
        if a >= -1e-10:
            continue
        if t >= p2 and vs[t] not in (-1, -2):
            continue
        amin = min(amin, a)
        ratio = d[t] / a
        if ratio < bound:
            continue
        if t >= p2:
            c = delta * -2.0
            upd = True
        else:
            c = (l[t] - u[t]) * a
            upd = feas - delta < c
        nb = bound
        if upd and d[t] >= 0.0:
            nb = max(bound, OPT / a + ratio)
        if best < 0 or ratio > cand[best][2] + 1e-10 or (abs(ratio - cand[best][2]) <= 1e-10 and c < cand[best][1]):
            best = len(cand)
        cand.append((t, c, ratio))
        bound = nb
    if not cand:
        return None, []
    if delta + cand[best][1] < 0.0:
        lo, run = quickselect(cand, delta)
    else:
        cand[0], cand[best] = cand[best], cand[0]
        lo = 0
    if lo > 0 and not OPT < d[cand[lo][0]]:
        lo = 0
    flips = [c[0] for c in cand[:lo]]
    thr = max(-amin * 1e-13, PIV)
    theta, hmax, rec = -1e100, -1e100, []
    for t, c, ratio in cand[lo:]:
        a = val[t]
        if -a >= thr and d[t] < theta * a:
            rec.append(t)
            h = (OPT + d[t]) / a
            hmax = max(h, hmax)
            if d[t] >= 0.0:
                theta = max(theta, h)
    # pick the largest |α|/w: w = column count (structural) or 0.5 (slack), first on ties
    def w(t):
        j = idx[t]
        return float(lens[j]) * (1.0 * 0.3 + 0.7) if lens is not None and j < len(lens) else 0.5 * (1.0 * 0.3 + 0.7)
    q, best_s = (rec[0] if rec else None), 0.0
    for t in rec:
        wt = w(t)
        if d[t] <= hmax * val[t] and val[t] < best_s * wt:
            best_s, q = val[t] / wt, t
    return (idx[q] if q is not None else None), flips


def main():
    verbose = "-v" in sys.argv
    names = [a for a in sys.argv[1:] if not a.startswith("-")] or sorted(p.name[:-9] for p in D.glob("*.jsonl.gz"))
    tot = okq = okf = 0
    for name in names:
        rows = [json.loads(x) for x in gzip.open(D / f"{name}.jsonl.gz", "rt")]
        lens = json.loads(gzip.open(D.parent / "iter0-5p" / f"{name}.jsonl.gz", "rt").readline())["len"]
        rows = [x for x in rows if x[0] == "bfrt"]
        shown = 0
        for k in range(0, len(rows) - 1, 2):
            e, lv = rows[k], rows[k + 1]
            q, flips = bfrt(e, lens)
            gq, gflips = lv[3], lv[4][1]
            tot += 1
            okq += q == gq
            okf += flips == gflips
            if (q != gq or flips != gflips) and verbose and shown < 2:
                shown += 1
                print(f"{name} call {k // 2}: q g {gq} mine {q}; flips g {len(gflips)} {gflips[:8]} mine {len(flips)} {flips[:8]}")
    print(f"calls {tot}: q equal {okq}, flip list equal {okf}")


if __name__ == "__main__":
    main()
