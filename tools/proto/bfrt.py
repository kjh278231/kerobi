"""BFRT (1c35c5) prototype checked against bench/oracle/bfrt_io (Gurobi inputs/outputs per call).
usage: python tools/proto/bfrt.py [-v] [--dir DIR] [name ...]   reports calls whose flip list or entering column differ.
--dir bench/oracle/m1_02e_bfrt: corpus whose entry also has spx+0x438/0x420/0x430/0x3fc/0x178/0x380 (M1-02e);
there the flip re-scan (spx+0x3fc != 0) runs."""
import gzip, json, os, sys
from pathlib import Path
D = Path(__file__).resolve().parents[2] / "bench/oracle/bfrt_io"
OPT, PIV = 1e-6, 3e-8
OFF = float(os.environ.get("BFRT_OFF", "0"))  # M1-02e: unknown dVar103*OptTol in the re-scan test (C:1515)


def lane_sum(run, xs, start, sign):
    """run ± Σxs as the original sums (asm 0xcba7f): one element peeled when the start index is odd (16-byte
    aligned buffer), fewer than 8 then in sequence, else 8 lanes with run in lane 0 combined
    ((L0+L2)+(L4+L6)) + ((L1+L3)+(L5+L7)), then the tail in sequence."""
    xs = [sign * x for x in xs]
    k = 1 if start % 2 else 0
    if len(xs) < k + 8:
        for x in xs:
            run += x
        return run
    for x in xs[:k]:
        run += x
    nv = k + ((len(xs) - k) // 8) * 8
    L = [run] + [0.0] * 7
    for b in range(k, nv, 8):
        for t in range(8):
            L[t] += xs[b + t]
    run = ((L[0] + L[2]) + (L[4] + L[6])) + ((L[1] + L[3]) + (L[5] + L[7]))
    for x in xs[nv:]:
        run += x
    return run


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
            run = lane_sum(run, [c[1] for c in cand[i + 1:hi + 1]], i + 1, -1.0)
        else:
            run = lane_sum(run, [c[1] for c in cand[lo:i + 1]], lo, 1.0)
        if run >= 0:
            hi = i
        else:
            lo = i + 1
    return lo, run


def bfrt(e, lens=None):
    p2, p3, (r, jl, x, lr, ur) = e[2], e[3], e[4]
    cnt, idx, val = e[5]
    d, vs, l, u, big = e[6], e[7], e[8], e[9], e[10]
    feas = 1e-6 * 1e30 * 1e-5 if big else 1e-6
    delta = (ur - x) if p3 else (x - lr)
    bound, cand, best, cbest = -1e100, [], -1, 0.0
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
            c = min(c, delta * -2.0)
        nb = bound
        if upd and d[t] >= 0.0:
            nb = max(bound, OPT / a + ratio)
        if best < 0 or ratio > cand[best][2] + 1e-10:
            best, cbest = len(cand), c
        elif ratio >= cand[best][2] - 1e-10:
            cbest = min(cbest, c)
            if ratio > cand[best][2]:
                best = len(cand)
        cand.append((t, c, ratio))
        bound = nb
    if not cand:
        return None, []
    if delta + cbest < 0.0:
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
    if q is not None:
        # flips: passed breakpoints within the dual step θ = d_q/α_q, q excluded (L1575-1610)
        th = hmax
        flips = [c[0] for c in cand[:lo] if c[0] != q and d[c[0]] <= val[c[0]] * th]
        if len(e) > 17 and e[17] != 0:
            q, flips = rescan(q, best_s, cand[:lo], flips, d, val, idx, e[18] < 10, w)
    return (idx[q] if q is not None else None), flips


def rescan(q, score, passed, flips, d, val, idx, use_pow, w):
    """spx+0x3fc != 0 (1c35c5 C:1500-1610): a passed breakpoint t with 0 <= d_t <= θ_q·α_t − OFF and a better
    |α|/w' than q enters instead, w' = (column count (slack 0.5) + ((θ_q·α_t − d_t)/OptTol)^0.25 when spx+0x178 < 10)
    · (0.3·f + 0.7); the flips are then the other passed breakpoints with d ≤ α·θ_new, in order."""
    thq, new = d[q] / val[q], None
    for c in passed:
        t = c[0]
        if d[t] < 0.0:
            continue
        dv = thq * val[t]
        if d[t] <= dv - OFF:
            base = w(t) / (1.0 * 0.3 + 0.7)
            ww = (base + (((dv - d[t]) / OPT) ** 0.25 if use_pow else 0.0)) * (1.0 * 0.3 + 0.7)
            if val[t] < score * ww:
                new, score = t, val[t] / ww
    if new is None:
        return q, flips
    th = d[new] / val[new]
    return new, [c[0] for c in passed if idx[c[0]] != idx[new] and d[c[0]] <= val[c[0]] * th]


def main():
    global D
    args = sys.argv[1:]
    if "--dir" in args:
        k = args.index("--dir")
        D = Path(args[k + 1]).resolve()
        del args[k:k + 2]
    verbose = "-v" in args
    names = [a for a in args if not a.startswith("-")] or sorted(p.name[:-9] for p in D.glob("*.jsonl.gz"))
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
