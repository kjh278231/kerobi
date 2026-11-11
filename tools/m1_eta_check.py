"""M1-02i check (docs/m1/m1_02i_eta.md section 3): the row-eta phase of the original BTRAN 221173 after FT updates.
Replays rho_r = e_r^T B^-1 of every kb2 dual pivot with row etas from the pinned Rust BTRAN dump (U^T result, etas, L;
bench/oracle/m1_02i_eta/rust_bt_kb2.jsonl.gz, made with rust_bt_dump.patch on main 63ec7ab) and compares it bit for
bit with the captured rho of tools/m1_beta_path.js. The original rule (C:1231-1567): after the U^T solve, dense etas
when the U^T output is dense or m < LU+0x104 * nnz, and then |v| <= ZERO clears w_r and skips the eta; sparse etas
skip only v == 0.0 and keep |w| >= ZERO. With --fresh also the beta and statecmp of the rsolver on PYTHONPATH.
usage: PYTHONPATH=$PWD/python ../.venv/bin/python tools/m1_eta_check.py [--self-test] [--fresh]"""
import copy, gzip, hashlib, json, math, struct, subprocess, sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from m1_beta_check import BIN, MODEL_SHA, PARAMS, fh, rust_snapshot, u2f  # noqa: E402
from m1_beta_check import GZ_SHA as BETA_SHA  # noqa: E402
from m1_tau_check import GZ_SHA as TAU_SHA  # noqa: E402

ROOT = Path(__file__).resolve().parents[1]
DIR = ROOT / "bench/oracle/m1_02i_eta"
DUMP_SHA = "2b44481de2fca2dcf1fd79c065d7bd9a276004c9860d964f8244c07b120c684d"  # rust_bt_kb2.jsonl.gz
PATCH_SHA = "f401053481a9071f45ee6461d8134ca235a43a9da4903a438289332d7bf4fae2"  # rust_bt_dump.patch
Z = 1e-13  # LU+0x10 (dVar2, C:80)
NP = 31  # pivots 0..30: Rust U = capture; the U column of the update at pivot 30 differs (section 7, M1-02l)
RUST_FAILS = [22, 25]
PIVOTS = [k for k in range(38) if k not in (0, 6)]  # dumped: btran_updated calls (0 and 6 follow a slack-only factor)  # pivots where the pre-M1-02i Rust eta rule (v != 0.0 always) misses the captured rho
bits = lambda x: struct.pack("<d", x)


def load():
    def runs(sub, pin):
        out = []
        for r in (1, 2, 3):
            p = ROOT / f"bench/oracle/{sub}/kb2_{r}.json.gz"
            d = json.load(gzip.open(p))
            d["_sha"], d["_pin"] = hashlib.sha256(p.read_bytes()).hexdigest(), pin
            out.append(d)
        return out
    dump = [json.loads(ln) for ln in gzip.open(DIR / "rust_bt_kb2.jsonl.gz", "rt")]
    bt = {}
    for i, d in enumerate(dump):  # the RHODUMP line of pivot k follows the BTDUMP line of its btran_updated call
        if "rho" in d and i and "y0" in dump[i - 1]:
            bt[d["iter"]] = dict(dump[i - 1], r=d["r"], rho_m=d["rho"], dense_m=d["dense"])
    shas = {p: hashlib.sha256((DIR / p).read_bytes()).hexdigest() for p in ("rust_bt_kb2.jsonl.gz", "rust_bt_dump.patch")}
    return {"beta": runs("m1_02a_beta", BETA_SHA), "tau": runs("m1_02a1_tau", TAU_SHA), "bt": bt, "shas": shas}


def orig_eta(c, m):
    """Row eta stored by 05b257 at the update of capture pivot c: U^T^-1 e_r on that pivot's U (21a941 order: r first,
    then U position ascending; entries with ZERO < |z|)."""
    ft, r = c["ft"], c["r"]
    pos = {i: p for p, i in enumerate(ft["ord"])}
    w, out = {r: 1.0}, []
    for p in range(pos[r], m):
        i = ft["ord"][p]
        x = w.get(i, 0.0)
        if x == 0.0:
            continue
        z = x / u2f(ft["diag"][i])
        if abs(z) <= Z:
            continue
        out.append((i, z))
        for j, v in zip(ft["urow"][i]["idx"], ft["urow"][i]["val"]):
            w[j] = w.get(j, 0.0) - u2f(v) * z
    return out


def ut_dense(ft, r, m):
    """Dense U^T of e_r on the captured U (pos 0..m-1, |x| <= Z or |x/d| <= Z gives 0), by LU index."""
    w, y = [0.0] * m, [0.0] * m
    w[r] = 1.0
    for i in ft["ord"]:
        x, d = w[i], u2f(ft["diag"][i])
        if abs(x) <= Z or abs(x / d) <= Z:
            continue
        y[i] = t = x / d
        for j, v in zip(ft["urow"][i]["idx"], ft["urow"][i]["val"]):
            w[j] -= u2f(v) * t
    return y


def replay(d, mult, rule):
    """221173 eta phase on the dump's U^T result (by row) and the dense L^T (C:1675-1735), by row.
    rule 'orig': dense when m < mult * nnz (then |v| <= Z clears w_r), else sparse; 'rust': always sparse;
    'dense'/'sparse': forced (mutants); 'fma': orig with fused eta products (mutant)."""
    m = len(d["steps"])
    row = [s[0] for s in d["steps"]]
    y = [u2f(h) for h in d["y0"]]
    nnz = sum(1 for v in y if v != 0.0)
    dense = rule == "dense" or (rule in ("orig", "fma") and m < mult * nnz)
    zeroed = []
    for r, ev in reversed(d["retas"]):
        v = y[r]
        if dense and abs(v) <= Z:
            if v != 0.0:
                zeroed.append((r, v))
            y[r] = 0.0
            continue
        if v == 0.0:
            continue
        for i, h in ev:
            e = u2f(h)
            y[i] = e * v if i == r else (math.fma(e, v, y[i]) if rule == "fma" else e * v + y[i])
    if not dense:
        y = [v if abs(v) >= Z else 0.0 for v in y]
    dn = [y[row[k]] for k in range(m)]
    for k in range(m - 1, -1, -1):
        v = dn[k]
        if Z < abs(v):
            for j, h in d["lrow"][k]:
                dn[j] -= u2f(h) * v
    out = [0.0] * m
    for k in range(m):
        out[row[k]] = dn[k]
    return out, dense, nnz, zeroed


def captured_rho(e, m):
    bt = e["bt"]
    if bt["nnz"] < 0:
        return [u2f(h) for h in bt["val"]]
    out = [0.0] * m
    for i, h in zip(e["idx"], e["val"]):
        out[i] = u2f(h)
    return out


def check(data, rule="orig", fresh=None):
    bad, rep = [], []
    beta, tau, bt = data["beta"], data["tau"], data["bt"]
    # M: pinned inputs
    for d in beta + tau:
        if d["_sha"] != d["_pin"]:
            bad.append(f"M {d['kind']}: gz sha {d['_sha'][:12]}")
        if d.get("binary_sha256") != BIN or d.get("model_sha256") != MODEL_SHA or d.get("params") != PARAMS:
            bad.append(f"M {d['kind']}: binary/model/params")
    if data["shas"] != {"rust_bt_kb2.jsonl.gz": DUMP_SHA, "rust_bt_dump.patch": PATCH_SHA}:
        bad.append(f"M dump/patch sha {data['shas']}")
    if any(b["events"] != beta[0]["events"] for b in beta) or any(t["events"] != tau[0]["events"] for t in tau):
        bad.append("M: the three runs differ")
    bev, tev = beta[0]["events"], tau[0]["events"]
    if [e["k"] for e in bev] != list(range(38)) or [e["k"] for e in tev] != list(range(38)) or sorted(bt) != PIVOTS:
        bad.append("M: pivots 0..37 in the captures, the dump without 0 and 6")
        return bad, rep
    m = tev[0]["ft"]["m"]
    # U, E, Y: same input state in pivots 0..NP-1
    first_u = None
    for k in PIVOTS:
        d, ft = bt[k], tev[k]["ft"]
        steps = d["steps"]
        u_same = (all(bits(u2f(steps[i][2])) == bits(u2f(ft["diag"][i])) for i in range(m)) and d["order"] == ft["ord"]
                  and all([tuple(x) for x in d["urow"][i]] == list(zip(ft["urow"][i]["idx"], ft["urow"][i]["val"]))
                          for i in range(m)))
        if not u_same:
            first_u = k if first_u is None else first_u
            if k < NP:
                bad.append(f"U pivot {k}: Rust U (diag/ord/row order) != capture")
    rep.append(f"U: Rust U = capture (diag, ord, U rows in storage order) on pivots 0..{NP - 1}; first differing pivot {first_u}")
    with_eta, n_eta = [], 0
    for k in [k for k in PIVOTS if k < NP]:
        d = bt[k]
        prow = {s[0]: i for i, s in enumerate(d["steps"])}
        start = max(j for j in range(k + 1) if tev[j]["lu_upd"] == 0)
        ups = [j for j in range(start, k) if tev[j + 1]["ft"]["lu"]["x180"] > tev[j]["ft"]["lu"]["x180"]]
        if len(ups) != len(d["retas"]) or len(ups) != tev[k]["ft"]["lu"]["x180"]:
            bad.append(f"E pivot {k}: {len(d['retas'])} Rust etas, {len(ups)} created, LU+0x180 {tev[k]['ft']['lu']['x180']}")
            continue
        for (r, ev), j in zip(d["retas"], ups):
            mine = [(prow[i], bits(u2f(h))) for i, h in ev]
            if prow[r] != tev[j]["r"] or mine != [(i, bits(z)) for i, z in orig_eta(tev[j], m)]:
                bad.append(f"E pivot {k}: eta of the update at pivot {j} != U^T^-1 e_r of the capture")
        n_eta += len(ups)
        y0 = ut_dense(tev[k]["ft"], tev[k]["r"], m)
        if any(bits(y0[i]) != bits(u2f(d["y0"][s[0]])) for i, s in enumerate(d["steps"])):
            bad.append(f"Y pivot {k}: Rust U^T result != dense U^T on the captured U")
        if ups:
            with_eta.append(k)
    rep.append(f"E/Y: Rust etas = U^T^-1 e_r of the captured U ({n_eta} eta checks) and Rust U^T result = dense U^T on "
               f"the captured U, pivots 0..{NP - 1}")
    # R: replay of the eta phase + dense L^T = captured rho
    fails, paths, zeroed = [], {"dense": 0, "sparse": 0}, []
    for k in with_eta:
        got, dense, nnz, z = replay(bt[k], tev[k]["ft"]["lu"]["x104"], rule)
        want = captured_rho(bev[k], m)
        paths["dense" if dense else "sparse"] += 1
        zeroed += [(k, r, v) for r, v in z]
        diff = [i for i in range(m) if bits(got[i]) != bits(want[i])]
        if diff:
            fails.append(k)
            i = diff[0]
            bad.append(f"R pivot {k}: {len(diff)} rows differ, first row {i}: capture {want[i]!r} replay {got[i]!r}")
    rep.append(f"R ({rule}): rho replay = capture on {len(with_eta) - len(fails)}/{len(with_eta)} pivots with etas "
               f"{with_eta}; paths {paths}; |v| <= ZERO cleared {[(k, r, f'{v:.3g}') for k, r, v in zeroed]}")
    if rule == "orig":
        rust = [k for k in with_eta if any(bits(a) != bits(b) for a, b in zip(replay(bt[k], 0, "rust")[0], captured_rho(bev[k], m)))]
        if rust != RUST_FAILS:
            bad.append(f"R: the pre-M1-02i Rust rule fails on {rust}, expected {RUST_FAILS}")
        rep.append(f"R: the pre-M1-02i Rust rule (sparse etas always) fails on {rust} (discriminating)")
    # F: a fresh rsolver run (after the fix): beta of pivots 0..NP-1 = capture, statecmp 0..30
    if fresh is not None:
        mine = fresh
        bdiff = [k for k in range(NP) if bits(fh(mine[k + 1]["beta"])) != bits(u2f(bev[k]["beta"]))]
        if bdiff:
            bad.append(f"F: Rust beta != capture on pivots {bdiff}")
        after = [k for k in range(NP, 38) if k + 1 in mine and bits(fh(mine[k + 1]["beta"])) != bits(u2f(bev[k]["beta"]))]
        rep.append(f"F: fresh Rust beta = capture on pivots 0..{NP - 1} ({NP - len(bdiff)}/{NP}); differing after: {after}")
    return bad, rep


def statecmp():
    import tempfile
    with tempfile.NamedTemporaryFile(suffix=".jsonl") as f:
        f.write(gzip.open(ROOT / "bench/oracle/iter0-30/kb2.jsonl.gz").read())
        f.flush()
        out = subprocess.run([sys.executable, str(ROOT / "tools/proto/statecmp.py"), "kb2", f.name], capture_output=True,
                             text=True, check=True).stdout
    lines = [ln for ln in out.splitlines() if ln.startswith("iter ")]
    return [ln for ln in lines if not ln.endswith(": OK")], len(lines)


def self_test(data):
    """Each tamper or mutant must be rejected (bad non-empty)."""
    out = []

    def tamper(name, fn):
        d = copy.deepcopy(data)
        fn(d)
        out.append((name, bool(check(d)[0])))

    def flip_rho(d):
        v = d["beta"][0]["events"][22]["bt"]["val"]
        v[5] = f"{int(v[5], 16) ^ 1:016x}"
        for b in d["beta"][1:]:
            b["events"] = d["beta"][0]["events"]

    def swap_urow(d):
        rw = d["bt"][20]["urow"]
        i = next(i for i, x in enumerate(rw) if len(x) >= 2)
        rw[i][0], rw[i][1] = rw[i][1], rw[i][0]

    def drop_eta_entry(d):
        d["bt"][22]["retas"][-1][1].pop()

    def shift_ut(d):
        d["bt"][21]["y0"] = [f"{int(h, 16) ^ 1:016x}" if int(h, 16) else h for h in d["bt"][21]["y0"]]

    tamper("captured rho pivot 22 row 5 last bit", flip_rho)
    tamper("Rust U row order pivot 20", swap_urow)
    tamper("Rust eta entry dropped pivot 22", drop_eta_entry)
    tamper("Rust U^T result last bits pivot 21", shift_ut)
    for rule in ("rust", "sparse", "dense", "fma"):
        bad = check(data, rule)[0]
        out.append((f"rule {rule}", bool(bad)))
    return out


def main():
    data = load()
    fresh = rust_snapshot() if "--fresh" in sys.argv else None
    bad, rep = check(data, fresh=fresh)
    for ln in rep:
        print(ln)
    if fresh is not None:
        diffs, n = statecmp()
        print(f"S: statecmp kb2 {n - len(diffs)}/{n} snapshots OK" + (f"; first diff: {diffs[0]}" if diffs else ""))
        if diffs:
            bad.append(f"S: statecmp {diffs[0]}")
    if "--self-test" in sys.argv:
        for name, rejected in self_test(data):
            print(f"T {name}: {'rejected' if rejected else 'SURVIVED'}")
            if not rejected and name not in SURVIVE:
                bad.append(f"T {name} survived")
            if rejected and name in SURVIVE:
                bad.append(f"T {name} rejected, expected to survive")
    for b in bad:
        print("FAIL", b)
    print("ACCEPTANCE", "FAIL" if bad else "PASS" + (" (+ self-test)" if "--self-test" in sys.argv else ""))
    sys.exit(1 if bad else 0)


SURVIVE = {"rule dense"}  # kb2 never takes the sparse eta path (m < LU+0x104 * nnz on all 17 pivots with etas)

if __name__ == "__main__":
    main()
