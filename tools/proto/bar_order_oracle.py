"""Ordering oracle (docs/m3_ordering.md): at the MESSAGE callback for "Ordering time" (printed by the ordering
driver 300f28 before 304736 frees the AA' pattern), dump the AA' graph B+0x110/B+0x118, the chosen permutation
F+0x30, nnz(L) F+0x38, ops F+0x60, and the original A (model data D = model+0xe8, before 2e9086 permutes its rows).
Then call Gurobi's ordering kernel 309dd0 directly for the baseline (mode 3) and the ND trials (mode 1, seeds 11..15)
so each candidate can be compared. At the first BARRIER callback dump F again (after symbolic 2e4f14) and stop.

usage: python tools/proto/bar_order_oracle.py <model.mps.gz> <out.json.gz> [--no-calls] [--bisect]
--bisect also runs 30b0f2 directly on each component 309dd0 mode 3 would split (bench/oracle/bar_order/bisect/).
Gurobi runs one process at a time; wrap with a retry loop on 'already running'.
"""
import ctypes, gzip, json, os, sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import bar_oracle as bo  # noqa: E402
import gurobipy as gp  # noqa: E402

I, L, D, arr, api = bo.I, bo.L, bo.D, bo.arr, bo.api
base = next(int(l.split("-")[0], 16) for l in open("/proc/self/maps") if "libgurobi130.so" in l and l.split()[2] == "00000000")
# 309dd0 @ file 0x324330: (xmm0 ops limit, model, B, F, mode, seed*, perm*, nnzL*, ops*, work*, thread, round, timer, aux*)
ORDER = ctypes.CFUNCTYPE(ctypes.c_int, ctypes.c_double, ctypes.c_void_p, ctypes.c_void_p, ctypes.c_void_p, ctypes.c_int,
                         ctypes.c_void_p, ctypes.c_void_p, ctypes.c_void_p, ctypes.c_void_p, ctypes.c_void_p, ctypes.c_int,
                         ctypes.c_int, ctypes.c_void_p, ctypes.c_void_p)(base + 0x324330)


# 30b0f2 @ file 0x325850: (env, n, colptr long*, rowind int*, part int*, round, seed*, work*) — multilevel bisection
BISECT = ctypes.CFUNCTYPE(ctypes.c_int, ctypes.c_void_p, ctypes.c_int, ctypes.c_void_p, ctypes.c_void_p, ctypes.c_void_p,
                          ctypes.c_int, ctypes.c_void_p, ctypes.c_void_p)(base + 0x325850)


def call_bisect(env, xa, ad, round_, seed):
    """Run Gurobi's bisection on one component subgraph; seed is a one-element list updated in place."""
    n = len(xa) - 1
    cx, ca, part = (ctypes.c_long * len(xa))(*xa), (ctypes.c_int * max(len(ad), 1))(*ad), (ctypes.c_int * n)()
    sd, work = ctypes.c_long(seed[0]), (ctypes.c_double * 4)()
    rc = BISECT(env, n, ctypes.addressof(cx), ctypes.addressof(ca), ctypes.addressof(part), round_, ctypes.addressof(sd),
                ctypes.addressof(work))
    seed[0] = sd.value
    return rc, list(part)


def bisect_calls(env, n, cp, ri, lab):
    """309dd0 mode 3 by hand: components of stage-1 nodes larger than max(600, n/100), bisected with seed 0 onwards."""
    comp = [-1 if lab[i] == -1 else 2000000000 for i in range(n)]
    nc = 0
    for s in range(n):
        if comp[s] != -1:
            continue
        comp[s], stack = nc, [s]
        while stack:
            v = stack.pop()
            for u in ri[cp[v]:cp[v + 1]]:
                if comp[u] == -1:
                    comp[u] = nc
                    stack.append(u)
        nc += 1
    seed, out = [0], []
    for c in range(nc):
        nodes = [i for i in range(n) if comp[i] == c]
        if len(nodes) <= max(600, n // 100):
            continue
        loc = {i: k for k, i in enumerate(nodes)}
        xa, ad = [0], []
        for i in nodes:
            ad += [loc[j] for j in ri[cp[i]:cp[i + 1]] if j in loc]
            xa.append(len(ad))
        s0 = seed[0]
        rc, part = call_bisect(env, xa, ad, -1, seed)
        out.append(dict(nodes=nodes, seed=s0, seed_after=seed[0], rc=rc, part=part))
    return out


def find_BF(m):
    """2bf1ea keeps local_98 = B+0x5c8, local_90 = B+0x5b8, local_88 = F next to each other; B is unknown in a
    MESSAGE callback, so match the pair by its 0x10 difference and check B+0x18 = B+0x1c = m, F+4 = m."""
    lo, hi, _ = next(r for r in bo.readable() if r[2] == "[stack]")
    w = (ctypes.c_long * ((hi - lo) // 8)).from_address(lo)
    for k in range(len(w) - 2):
        a, b, F = w[k], w[k + 1], w[k + 2]
        if a - b == 0x10 and a > 0x10000 and F > 0x10000:
            B = a - 0x5c8
            try:
                if I(B + 0x18) == m and I(B + 0x1c) == m and I(F + 4) == m:
                    return B, F
            except (ValueError, OSError):
                pass
    return 0, 0


def ints(p, k, t=ctypes.c_int):
    return list((t * k).from_address(p)) if p and k > 0 else []


def snapD(Dm):
    n, m, nnz = I(Dm + 0xc), I(Dm + 8), L(Dm + 0x10)
    dl = lambda o, k: [x.hex() for x in (ctypes.c_double * k).from_address(L(Dm + o))]
    return dict(m=m, n=n, nnz=nnz, beg=ints(L(Dm + 0xf8), n + 1, ctypes.c_long), len=ints(L(Dm + 0x100), n),
                ind=ints(L(Dm + 0x108), nnz), lb=dl(0x390, n), ub=dl(0x398, n),
                sense=bytes((ctypes.c_char * m).from_address(L(Dm + 0x388))).decode())


def call_order(mp, B, F, mode, seed, n):
    perm = (ctypes.c_int * max(n, 1))()
    nnzL, ops, aux = ctypes.c_long(0), ctypes.c_double(0), ctypes.c_long(0)
    work = (ctypes.c_double * 4)()
    sd = ctypes.c_long(seed)
    rc = ORDER(1e100, mp, B, F, mode, ctypes.addressof(sd), ctypes.addressof(perm), ctypes.addressof(nnzL),
               ctypes.addressof(ops), ctypes.addressof(work), 1, 0, B + 0x568, ctypes.addressof(aux))
    return dict(mode=mode, seed=seed, rc=rc, perm=list(perm)[:n], nnzL=nnzL.value, ops=ops.value.hex(),
                seed_after=sd.value, work=work[0].hex())


def main():
    path, out = sys.argv[1], sys.argv[2]
    calls = "--no-calls" not in sys.argv
    env = gp.Env(params={"OutputFlag": 1, "LogToConsole": 0, "Threads": 1, "Method": 2, "Presolve": 0, "Crossover": 0})
    model = gp.read(path, env)
    mp = api.PyCapsule_GetPointer(model._cmodel, api.PyCapsule_GetName(model._cmodel))
    res = {"name": os.path.basename(path).split(".")[0], "A": snapD(L(mp + 0xe8)), "seed_b0_pre": L(mp + 0xb0)}
    m = res["A"]["m"]

    def cb(mdl, where):
        if where == gp.GRB.Callback.MESSAGE and "ord" not in res:
            msg = mdl.cbGet(gp.GRB.Callback.MSG_STRING)
            if not msg.startswith("Ordering time"):
                return
            B, F = find_BF(m)
            if not B:
                res["ord"] = {"error": "B/F not found"}
                return
            n = I(B + 0x1c) + I(B + 0x218)
            nz = L(B + 0x100)
            o = dict(n=n, ndense=I(B + 0x218), nfree=I(B + 0x208), b24=I(B + 0x24), bf0=I(B + 0xf0), nnz=nz,
                     colptr=ints(L(B + 0x110), n + 1, ctypes.c_long), rowind=ints(L(B + 0x118), nz),
                     perm=ints(L(F + 0x30), n), nnzL=L(F + 0x38), ops=D(F + 0x60).hex(), ops2=D(F + 0x48).hex(),
                     F18=I(F + 0x18), seed_b0=L(mp + 0xb0), F=arr(F, 0x200 // 8))
            if "--bisect" in sys.argv:
                t = max(100, int(min(float(int(nz * 50.0 / n)), n ** 0.5 * 10.0)))
                lab = [-2 if o["colptr"][i + 1] - o["colptr"][i] > t else -1 for i in range(n)]
                o["bisect"] = bisect_calls(L(mp + 0x100), n, o["colptr"], o["rowind"], lab)
            if calls:
                o["calls"] = [call_order(mp, B, F, 3, res["seed_b0_pre"], n)]
                o["calls"] += [call_order(mp, B, F, 1, 11 + k, n) for k in range(5)]
            res["ord"] = o
        elif where == gp.GRB.Callback.BARRIER and "sym" not in res:
            c = mdl._callback._cbdata
            B = L(api.PyCapsule_GetPointer(c, api.PyCapsule_GetName(c)) + 0x68)
            F = bo.find_F(B, m)
            n = I(F + 0) if F else 0
            res["sym"] = {"F_addr": F, "F": arr(F, 0x200 // 8) if F else None, "B": arr(B, 0xa00 // 8)}
            if F:  # after symbolic 2e4f14: final positions, column lengths, supernode widths, row indices
                nr = L(F + 0x40)
                res["sym"].update(n=n, perm=ints(L(F + 0x30), n), collen=ints(L(F + 0xa8), n), snw=ints(L(F + 0x78), n),
                                  nrowidx=nr, rowidx=ints(L(F + 0xc8), nr), nnzL=L(F + 0x38), ops=D(F + 0x60).hex(),
                                  ops2=D(F + 0x48).hex())
            res["A_cb"] = snapD(L(mp + 0xe8))
            mdl.terminate()

    model.optimize(cb)
    with gzip.open(out, "wt") as f:
        json.dump(res, f)
    o = res.get("ord", {})
    print(res["name"], "n", o.get("n"), "nnz", o.get("nnz"), "nnzL", o.get("nnzL"), "ops", o.get("ops") and float.fromhex(o["ops"]),
          "calls", [(c["mode"], c["seed"], c["nnzL"], float.fromhex(c["ops"])) for c in o.get("calls", [])])


if __name__ == "__main__":
    main()
