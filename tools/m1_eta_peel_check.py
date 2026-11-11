"""M1-02l check (docs/m1/m1_02l_eta_peel.md section 3): the one-element peel of the row-eta dot products.
09d112's R phase sums x[idx]*val of each row eta in 8 lanes; when the eta value array is 8- but not 16-byte aligned it
first peels one element, and with n < peel + 8 the whole sum is sequential (C:1149-1260, asm 0x1272d7). The value
array lives in the LU pool LU+0x150 at offset LU+0x160 of its update (05b257 C:906-918), so its alignment is the pool
offset parity. The new FT diagonal (05b257 C:321-372) reads the LU+0x1d0 buffer instead.
Inputs: tools/m1_eta_pool.js captures (bench/oracle/m1_02l_pool, 3 runs), the 02a-1 capture (non-interference) and the
02i Rust BTRAN dump (L, nnz(L)).
With --fresh (after the Rust change) also the DSE beta of every kb2 pivot from the rsolver on PYTHONPATH.
usage: python3 tools/m1_eta_peel_check.py [--self-test]
       PYTHONPATH=$PWD/python ../.venv/bin/python tools/m1_eta_peel_check.py --fresh"""
import copy, gzip, hashlib, json, struct, sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from m1_beta_check import BIN, MODEL_SHA, PARAMS, fh, rust_snapshot, u2f  # noqa: E402
from m1_eta_check import Z, load as load_02i, orig_eta  # noqa: E402
from m1_tau_check import DUMP_SHA as LU_DUMP_SHA  # noqa: E402

ROOT = Path(__file__).resolve().parents[1]
DIR = ROOT / "bench/oracle/m1_02l_pool"
GZ_SHA = "cda2949cc0840bdb89a599a7c9e4c957ee2903462e975b8ab20355c5a73863a0"  # each of kb2_{1,2,3}.json.gz
SPIKE = [10, 11, 12, 13, 18, 19, 20, 21, 22, 23, 24, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36]  # updates with etas
DIAG = [9, 10, 11, 12, 13, 17, 18, 19, 20, 21, 22, 23, 24, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36]  # new etas
# fixed before the capture from the stored 02a-1 data (section 3 V); None = must pass everywhere
EXPECT = {"orig": ([], []), "peel0": ([30, 32, 35], []), "peel1": ([], [33, 35]), "invert": ([30, 32, 35], [33, 35]),
          "seq": ([], [33])}
bits = lambda x: struct.pack("<d", x)


def load():
    base = load_02i()
    runs = []
    for r in (1, 2, 3):
        p = DIR / f"kb2_{r}.json.gz"
        d = json.load(gzip.open(p))
        d["_sha"] = hashlib.sha256(p.read_bytes()).hexdigest()
        runs.append(d)
    tool = hashlib.sha256((ROOT / "tools/m1_eta_pool.js").read_bytes()).hexdigest()
    # nnz(L) of the factors at pivots 0 and 6 (slack-only U; the 02i dump starts at pivot 1): the pinned 02a-1 Rust LU
    # dump, whose code matched the capture up to pivot 11
    p = ROOT / "bench/oracle/m1_02a1_tau/rust_lu_kb2.jsonl.gz"
    lnnz = {}
    for ln in gzip.open(p, "rt"):
        d = json.loads(ln)
        if d["iter"] in (0, 6) and d["iter"] not in lnnz:
            lnnz[d["iter"]] = sum(len(c) for c in d["lu"]["lcol"])
    return {"runs": runs, "tau1": base["tau"][0], "bt": base["bt"], "tool": tool, "beta": base["beta"][0]["events"],
            "lnnz_old": lnnz, "lu_dump_sha": hashlib.sha256(p.read_bytes()).hexdigest()}


def dot(t, peel):
    """09d112 R-phase sum of the products t (C:1149-1260, asm 0x1272d7-0x12742f)."""
    n = len(t)
    if n < 8 or n < peel + 8:
        a = 0.0
        for x in t:
            a += x
        return a
    a = [0.0] * 8
    for x in t[:peel]:
        a[0] += x
    nb = (n - peel) - ((n - peel) & 7)
    for i in range(nb):
        a[i & 7] += t[peel + i]
    w = ((a[0] + a[2]) + (a[4] + a[6])) + ((a[1] + a[3]) + (a[5] + a[7]))
    for x in t[peel + nb:]:
        w += x
    return w


def ups_of(ev, k):
    start = max(j for j in range(k + 1) if ev[j]["lu_upd"] == 0)
    return start, [j for j in range(start, k) if ev[j + 1]["ft"]["lu"]["x180"] > ev[j]["ft"]["lu"]["x180"]]


def l_forward(d, x, m):
    """FTRAN through the Rust L of the dump (lrow = L^T rows by step), x by row."""
    row = [s[0] for s in d["steps"]]
    cols = [[] for _ in range(m)]
    for k in range(m):
        for j, h in d["lrow"][k]:
            cols[j].append((row[k], u2f(h)))
    for j in range(m):
        v = x[row[j]]
        if abs(v) > Z:
            for i, l in cols[j]:
                x[i] -= l * v
    return row


def pool_growth(ev, k, cap, m):
    """LU+0x160 growth of the update at pivot k (05b257): the new eta (C:906-918), the new U column at the pool end
    when it is longer than the old column (C:529 local_150, C:1224-1227), and each full U row that takes a new entry
    moved to the end with len + min(max(4, len/2), LU+0x188 - LU+0x18c) places (C:1264-1282). Row capacities right
    after a factor = lengths."""
    a, b = ev[k]["ft"], ev[k + 1]["ft"]
    col = a["r"]
    de = b["pool"]["x184"] - a["pool"]["x184"]
    lens = [len([j for j in a["urow"][i]["idx"] if j != col]) for i in range(m)]
    if de > 0:
        lens[col] = 0
    lim = a["pool"]["x188"] - a["pool"]["x18c"]
    grow, bound = 0, False
    newc = [i for i in range(m) if col in b["urow"][i]["idx"] and i != col]
    for i in newc:
        if lens[i] == cap[i]:
            extra = max(4, lens[i] // 2)
            bound |= lim < extra
            extra = min(extra, lim)
            grow += lens[i] + extra
            cap[i] = lens[i] + extra
        lens[i] += 1
    if sum(1 for r in a["urow"] if col in r["idx"]) < len(newc):
        grow += len(newc)
    return de + grow, bound


def check(data, rule="orig"):
    bad, rep = [], []
    runs, bt = data["runs"], data["bt"]
    # M: pinned inputs, three identical runs
    for i, d in enumerate(runs):
        if GZ_SHA is not None and d["_sha"] != GZ_SHA:
            bad.append(f"M run {i + 1}: gz sha {d['_sha'][:12]}")
        if (d.get("binary_sha256") != BIN or d.get("model_sha256") != MODEL_SHA or d.get("params") != PARAMS
                or d.get("tool_sha256") != data["tool"] or not d.get("ok") or d.get("errors")):
            bad.append(f"M run {i + 1}: binary/model/params/tool/ok")
    ev = runs[0]["events"]
    if any(d["events"] != ev for d in runs) or [e.get("k") for e in ev] != list(range(38)):
        bad.append("M: the three runs differ or pivots != 0..37")
        return bad, rep
    if data["lu_dump_sha"] != LU_DUMP_SHA or sorted(data["lnnz_old"]) != [0, 6]:
        bad.append("M: 02a-1 Rust LU dump sha / pivots 0, 6")
    m = ev[0]["ft"]["m"]
    # N: without ft.pool the capture is the 02a-1 capture
    proj = copy.deepcopy(ev)
    for e in proj:
        e["ft"].pop("pool", None)
    if proj != data["tau1"]["events"]:
        bad.append("N: the capture without ft.pool != 02a-1 kb2_1 events")
    # P: eta records, offsets and alignment
    n_rec = 0
    for k in range(38):
        p = ev[k]["ft"]["pool"]
        start, ups = ups_of(ev, k)
        if len(p["etas"]) != ev[k]["ft"]["lu"]["x180"] or len(p["etas"]) != len(ups):
            bad.append(f"P pivot {k}: {len(p['etas'])} records, LU+0x180 {ev[k]['ft']['lu']['x180']}, {len(ups)} updates")
            continue
        if p["a1d0"] not in (0, 8) or p["b150"] not in (0, 8):
            bad.append(f"P pivot {k}: LU+0x1d0 value array or LU+0x150 not 8-aligned")
        for q, j in zip(p["etas"], ups):
            n_rec += 1
            want = (q["r"] == ev[j]["r"] and q["len"] == len(orig_eta(ev[j], m)) and q["io"] == q["vo"]
                    and q["vo"] == ev[j]["ft"]["pool"]["x160"] and q["va"] == (p["b150"] + 8 * q["vo"]) % 16)
            if not want:
                bad.append(f"P pivot {k}: eta of update {j} {q} != (r, len, offset = LU+0x160 at {j}, va)")
    rep.append(f"P: {n_rec} eta records: r, len = U^T^-1 e_r of the capture, idx/val offset = LU+0x160 at the update, "
               f"val mod 16 = 8*offset mod 16; LU+0x150 mod 16 {sorted({e['ft']['pool']['b150'] for e in ev})}, "
               f"LU+0x1d0 values mod 16 {sorted({e['ft']['pool']['a1d0'] for e in ev})}")
    # G, X: LU+0x160 after a factor and its growth per update
    gok, gn, binds, xs = 0, 0, 0, []
    for k in range(38):
        if ev[k]["lu_upd"] == 0:
            u = [len(r["idx"]) for r in ev[k]["ft"]["urow"]]
            cap = u[:]
            lnnz = data["lnnz_old"].get(k) if k in (0, 6) else sum(len(r) for r in bt[k]["lrow"])
            xs.append((k, ev[k]["ft"]["pool"]["x160"], 2 * (lnnz + sum(u))))
        if k == 37 or ev[k + 1]["lu_upd"] == 0:
            continue
        g, bound = pool_growth(ev, k, cap, m)
        gn += 1
        binds += bound
        if g == ev[k + 1]["ft"]["pool"]["x160"] - ev[k]["ft"]["pool"]["x160"]:
            gok += 1
        else:
            bad.append(f"G pivot {k}: LU+0x160 growth {ev[k + 1]['ft']['pool']['x160'] - ev[k]['ft']['pool']['x160']}, model {g}")
    if any(a != b for _, a, b in xs):
        bad.append(f"X: LU+0x160 after a factor != 2*(nnz L + nnz U): {xs}")
    rep.append(f"G: LU+0x160 growth = model on {gok}/{gn} updates (row cap LU+0x188-0x18c bound {binds} times); "
               f"X: LU+0x160 after the factors at {[k for k, _, _ in xs]} = 2*(nnz L + nnz U) {[a for _, a, _ in xs]}")
    # S, D: spike and new diagonal under the rule
    def peel(va, kind):
        if rule == "peel0":
            return 0
        if rule == "peel1":
            return 1
        p = 1 if va == 8 else 0
        return 1 - p if rule == "invert" else p

    def sm(t, p):
        if rule == "seq":
            a = 0.0
            for x in t:
                a += x
            return a
        return dot(t, p)

    sf, df = [], []
    for k in sorted(set(SPIKE) | set(DIAG)):
        e, nxt = ev[k]["ft"], ev[k + 1]["ft"]
        _, ups = ups_of(ev, k)
        x = [0.0] * m
        for i, h in zip(e["aq_in"]["idx"], e["aq_in"]["val"]):
            x[i] = u2f(h)
        row = l_forward(bt[k], x, m)
        for q, j in zip(e["pool"]["etas"], ups):
            v = sm([x[row[i]] * z for i, z in orig_eta(ev[j], m)], peel(q["va"], "R"))
            x[row[q["r"]]] = 0.0 if abs(v) <= Z else v
        col = e["r"]
        if k in SPIKE:
            ent = [(i, u2f(h)) for i in range(m) for j, h in zip(nxt["urow"][i]["idx"], nxt["urow"][i]["val"]) if j == col]
            if not ent or any(bits(x[row[i]]) != bits(v) for i, v in ent):
                sf.append(k)
        if k in DIAG:
            t = [x[row[i]] * z for i, z in orig_eta(ev[k], m)]
            if bits(sm(t, peel(e["pool"]["a1d0"], "D"))) != bits(u2f(nxt["diag"][col])):
                df.append(k)
    rep.append(f"S/D ({rule}): spike = capture on {len(SPIKE) - len(sf)}/{len(SPIKE)} updates with etas, new diagonal on "
               f"{len(DIAG) - len(df)}/{len(DIAG)} eta updates; fail {sf} / {df}")
    if (sf, df) != EXPECT[rule]:
        bad.append(f"S/D ({rule}): fails {sf} / {df}, expected {EXPECT[rule]}")
    return bad, rep


def self_test(data):
    out = []

    def tamper(name, fn):
        d = copy.deepcopy(data)
        fn(d)
        out.append((name, bool(check(d)[0])))

    def each(d, f):
        for r in d["runs"]:
            f(r["events"])

    tamper("binary sha", lambda d: d["runs"][0].update(binary_sha256="0" * 64))
    tamper("empty events", lambda d: each(d, lambda ev: ev.clear()))
    tamper("run 2 va flipped", lambda d: d["runs"][1]["events"][30]["ft"]["pool"]["etas"][1].update(va=0))
    tamper("va of the length-8 eta flipped (all runs)",
           lambda d: each(d, lambda ev: [ev[k]["ft"]["pool"]["etas"][1].update(va=0) for k in range(28, 38)]))
    tamper("vo shifted", lambda d: each(d, lambda ev: ev[33]["ft"]["pool"]["etas"][2].update(vo=574, io=574)))
    tamper("02a-1 field", lambda d: d["tau1"]["events"][5]["ft"]["lu"].update(x104=0))

    def flip_u(ev):
        r = ev[31]["ft"]["urow"][17]
        i = r["idx"].index(16)
        r["val"][i] = f"{int(r['val'][i], 16) ^ 1:016x}"
    tamper("new U column bit pivot 31 (all runs, 02a-1 too)",
           lambda d: (each(d, flip_u), flip_u(d["tau1"]["events"])))
    tamper("eta len", lambda d: each(d, lambda ev: ev[20]["ft"]["pool"]["etas"][0].update(len=5)))
    tamper("LU+0x160 after factor", lambda d: each(d, lambda ev: ev[15]["ft"]["pool"].update(x160=198)))
    for rule in ("peel0", "peel1", "invert", "seq"):
        out.append((f"rule {rule} (expected fails)", not check(data, rule)[0]))
    return out


def main():
    data = load()
    bad, rep = check(data)
    if "--fresh" in sys.argv:
        mine, bev = rust_snapshot(), data["beta"]
        diff = [k for k in range(len(bev)) if k + 1 not in mine or bits(fh(mine[k + 1]["beta"])) != bits(u2f(bev[k]["beta"]))]
        ev = data["runs"][0]["events"]
        pdiff = [k for k in range(len(ev)) if k not in mine or mine[k].get("pool") != ev[k]["ft"]["pool"]["x160"]]
        rep.append(f"F: fresh Rust beta = capture on {len(bev) - len(diff)}/{len(bev)} pivots; differ {diff}; Rust pool "
                   f"(LU+0x160, snapshot k) = capture x160 at pivot k on {len(ev) - len(pdiff)}/{len(ev)}; differ {pdiff}")
        if diff or pdiff:
            bad.append(f"F: Rust beta != capture on pivots {diff}, pool on {pdiff}")
    for ln in rep:
        print(ln)
    if "--self-test" in sys.argv:
        for name, ok in self_test(data):
            print(f"T {name}: {'ok' if ok else 'NOT REJECTED / unexpected'}")
            if not ok:
                bad.append(f"T {name}")
    for b in bad:
        print("FAIL", b)
    print("ACCEPTANCE", "FAIL" if bad else "PASS" + (" (+ self-test)" if "--self-test" in sys.argv else ""))
    sys.exit(1 if bad else 0)


if __name__ == "__main__":
    main()
