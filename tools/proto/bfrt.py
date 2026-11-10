"""BFRT (1c35c5) prototype checked against bench/oracle/bfrt_io (Gurobi inputs/outputs per call).
usage: python tools/proto/bfrt.py [-v] [--strict] [--dir DIR [--same-as OLD]] [name ...]   reports calls whose flip list
or entering column differ (--strict: exit 1 then). --same-as: every row of DIR cut to the length of the OLD row equals it.
BFRT_RESCAN=0 turns the re-scan off (the pre-M1-02e rule). --shift-replay: rerun mismatches with the returned d
of candidates whose d was in (-OptTol, -1e-13) (the original shifts those costs with its RNG).
--dir bench/oracle/m1_02e_bfrt: corpus whose entry also has spx+0x438/0x420/0x430/0x3fc/0x178/0x380 (M1-02e);
there the flip re-scan (spx+0x3fc != 0) runs."""
import gzip, json, os, sys
from pathlib import Path
D = Path(__file__).resolve().parents[2] / "bench/oracle/bfrt_io"
OPT, PIV = 1e-6, 3e-8
# re-scan margin dVar103·OptTol: dVar103 holds 1000.0 there (C:1219/1278, the /1000 of the C:1335 shift); env for mutants
OFF = float(os.environ.get("BFRT_OFF", 1000.0 * OPT))


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
        if len(e) > 17 and e[17] != 0 and os.environ.get("BFRT_RESCAN", "1") != "0":
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


def trigger(tdir):
    """48d5f C:2106-2111 sets spx+0x3fc = -1 (NONDEGENFLIP 0) only on its full-recompute path (LU+0x18c == 0, right
    after a factorization) when spx+0x338 > 2 or m < 3·spx+0x2a0 or spx+0x178 != 0. Rows of
    tools/trace_specs/nondegenflip_m1_02e.json: enter [LU+0x18c, spx+0x2a0, m], leave [spx+0x3fc, spx+0x338, spx+0x178]."""
    bad = 0
    for p in sorted(tdir.glob("*.jsonl.gz")):
        rows = [x for x in map(json.loads, gzip.open(p, "rt")) if x[0] == "dual"]
        on, first = 0, None
        for k in range(0, len(rows) - 1, 2):
            (upd, it, m), (f, c338, c178) = rows[k][2:5], rows[k + 1][2:5]
            want = on or (upd == 0 and (c338 > 2 or m < 3.0 * it or c178 != 0))
            if bool(f) != bool(want):
                bad += 1
                print(f"{p.name}: call {k // 2} iter {it} LU+0x18c {upd}: 0x3fc {f}, rule {-1 if want else 0}")
                break
            if f and not on:
                first = it
            on = on or f
        print(f"{p.name}: switch at iter {first}")
    return 1 if bad else 0


def main():
    global D
    args = sys.argv[1:]
    if "--dir" in args:
        k = args.index("--dir")
        D = Path(args[k + 1]).resolve()
        del args[k:k + 2]
    if "--trigger" in args:
        sys.exit(trigger(Path(args[args.index("--trigger") + 1])))
    old = None
    if "--same-as" in args:
        k = args.index("--same-as")
        old = Path(args[k + 1]).resolve()
        del args[k:k + 2]
    verbose, strict = ("-vv" in args) + ("-v" in args or "-vv" in args), "--strict" in args
    names = [a for a in args if not a.startswith("-")] or sorted(p.name[:-9] for p in D.glob("*.jsonl.gz"))
    if old is not None:
        bad = 0
        for name in names:
            a = [json.loads(x) for x in gzip.open(old / f"{name}.jsonl.gz", "rt")]
            b = [json.loads(x) for x in gzip.open(D / f"{name}.jsonl.gz", "rt")]
            k = next((i for i, (x, y) in enumerate(zip(a, b)) if y[:len(x)] != x), None)
            if k is not None or len(a) != len(b):
                bad += 1
                print(f"{name}: rows {len(a)} vs {len(b)}, first differing row {k}")
        print(f"projection: {len(names) - bad}/{len(names)} models equal")
        sys.exit(1 if bad else 0)
    shift = "--shift-replay" in args
    tot = okq = okf = nshift = 0
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
            if (q != gq or flips != gflips) and shift:
                # candidates with -OptTol < d < -1e-13 get a random cost shift in the original (C:1690-1740);
                # replay with their d as returned (AX:0x1e0 at leave, matched by column)
                ld = dict(zip(lv[6][1], lv[5]))
                e2 = json.loads(json.dumps(e))
                for t, j in enumerate(e2[5][1]):
                    if -1e-6 < e2[6][t] < -1e-13 and j in ld:
                        e2[6][t] = ld[j]
                fixed = bfrt(e2, lens) == (gq, gflips)
                nshift += fixed
                print(f"{name} call {k // 2}: replay with returned d of small negative d: {'equal' if fixed else 'differs'}")
            if (q != gq or flips != gflips) and verbose and shown < (99 if verbose > 1 else 2):
                shown += 1
                print(f"{name} call {k // 2}: q g {gq} mine {q}; flips g {len(gflips)} {gflips[:8]} mine {len(flips)} {flips[:8]}")
    print(f"calls {tot}: q equal {okq}, flip list equal {okf}" + (f", mismatches equal after shift replay {nshift}" if shift else ""))
    if strict and (okq != tot or okf != tot or not tot):
        sys.exit(1)


if __name__ == "__main__":
    main()
