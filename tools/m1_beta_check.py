"""M1-02a check (docs/m1/m1_02a_beta.md section 3): the DSE beta of the original vs rsolver on kb2.
Reads the three captures of tools/m1_beta_path.js, the stored spx_oracle file and a fresh RSOLVER_SNAP run of the
rsolver on PYTHONPATH (build it first with tools/build_local.sh). Exit 1 on any failure.
usage: PYTHONPATH=$PWD/python ../.venv/bin/python tools/m1_beta_check.py [--self-test]"""
import copy, gzip, hashlib, json, os, struct, subprocess, sys, tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DIR = ROOT / "bench/oracle/m1_02a_beta"
MODEL = ROOT.parent / "re/bench/data/netlib/kb2.mps.gz"
ORACLE = ROOT / "bench/oracle/iter0-30/kb2.jsonl.gz"
BIN = "8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c"
MODEL_SHA = "fd4e89375a650a3a5eb25c1d57c293e1cf9b1791e0d6fa61a0d586b76ba5d516"
GZ_SHA = "caad5976f259e668d473e9e992b4c1b69e44254cbce7b63eda778c7b479e48b1"  # each of kb2_{1,2,3}.json.gz (section 6.2)
TOOL = ROOT / "tools/m1_beta_path.js"
PARAMS = {"Threads": 1, "Method": 1, "Presolve": 0, "OutputFlag": 0}
RUST_PIVOTS = 22  # pivots 0..21: before the pivot path splits (pathcmp: first diff at pivot 21 is the 22nd entering choice)


def u2f(h):
    """capture fields: raw u64 bits as 16 hex digits"""
    return struct.unpack("<d", int(h, 16).to_bytes(8, "little"))[0]


fh = float.fromhex  # spx_oracle and RSOLVER_SNAP fields: float.hex strings


def flip_fh(h):
    return struct.unpack("<d", (struct.unpack("<Q", struct.pack("<d", fh(h)))[0] ^ 1).to_bytes(8, "little"))[0].hex()


def lanes(v, peel=0, chain=False):
    """8b473 SIMD path: `peel` leading elements into lane 0, 8 lanes, combine, then the tail in order."""
    a = [0.0] * 8
    for x in v[:peel]:
        a[0] += x * x
    end = len(v) - ((len(v) - peel) & 7)
    for k in range(peel, end, 8):
        for q in range(8):
            a[q] += v[k + q] * v[k + q]
    if end == peel:
        w = a[0]
    elif chain:
        w = (((a[0] + a[2]) + a[4]) + a[6]) + (((a[1] + a[3]) + a[5]) + a[7])
    else:
        w = ((a[0] + a[2]) + (a[4] + a[6])) + ((a[1] + a[3]) + (a[5] + a[7]))
    for x in v[end:]:
        w += x * x
    return w


def seq(v):
    w = 0.0
    for x in v:
        w += x * x
    return w


def rust_sum_squares(v):
    """crates/lp/src/lu.rs sum_squares: 8 lanes from index 0, ((0+2)+(4+6))+((1+3)+(5+7)), tail in order."""
    return lanes(v)


def original_sum(e):
    v = [u2f(x) for x in e["val"]]
    if e["path"] == "scalar":
        return seq(v)
    return lanes(v, peel=1 if e["mod16"] else 0)


def load_runs():
    runs = []
    for r in (1, 2, 3):
        p = DIR / f"kb2_{r}.json.gz"
        d = json.load(gzip.open(p))
        d["_gz_sha256"] = hashlib.sha256(p.read_bytes()).hexdigest()
        runs.append((p, d))
    return runs


def rust_snapshot():
    fd, snap = tempfile.mkstemp(suffix=".jsonl")
    os.close(fd)
    code = (f"import rsolver; m = rsolver.read({str(MODEL)!r}); m.Params['Method'] = 1; m.Params['Presolve'] = 0; "
            "m.Params['OutputFlag'] = 0; m.optimize()")
    subprocess.run([sys.executable, "-c", code], env=dict(os.environ, RSOLVER_SNAP=snap), check=True)
    mine = {}
    for line in open(snap):
        s = json.loads(line)
        mine.setdefault(s["iter"], s)
    os.unlink(snap)
    return mine


def check(runs, oracle, mine):
    """Returns (failures, report lines)."""
    bad, rep = [], []
    # M: inputs
    for p, d in runs:
        if d.get("binary_sha256") != BIN:
            bad.append(f"M {p.name}: binary sha")
        if d.get("model_sha256") != MODEL_SHA:
            bad.append(f"M {p.name}: model sha")
        if d.get("params") != PARAMS:
            bad.append(f"M {p.name}: params {d.get('params')}")
        if not d.get("ok") or d.get("output") != "DONE 2 38" or d.get("errors"):
            bad.append(f"M {p.name}: run not ok ({d.get('output')!r})")
        if not d.get("events"):
            bad.append(f"M {p.name}: no events")
        if d.get("_gz_sha256") != GZ_SHA:
            bad.append(f"M {p.name}: file sha {d.get('_gz_sha256')} != pinned")
    tools = {d.get("tool_sha256") for _, d in runs}
    if tools != {hashlib.sha256(TOOL.read_bytes()).hexdigest()}:
        bad.append(f"M tool sha {tools} != current {TOOL.name}")
    rep.append(f"M: 3 runs, binary/model/params/output, pinned file sha, tool = current file {next(iter(tools))[:16]}")
    # B: three runs equal
    ev = runs[0][1]["events"]
    for p, d in runs[1:]:
        if d["events"] != ev:
            k = next((i for i, (a, b) in enumerate(zip(ev, d["events"])) if a != b), min(len(ev), len(d["events"])))
            bad.append(f"B {p.name}: event {k} differs from run 1")
    rep.append(f"B: {len(ev)} pivots, 3 runs equal")
    # C: compaction 28c93 C:149-163
    nd = 0
    for e in ev:
        b, thr = e["bt"], u2f(e["thr"])
        if not b or not b["same_vector"] or not e["v_is_218"]:
            bad.append(f"C pivot {e['k']}: BTRAN output not the summed vector")
            continue
        if b["nnz"] < 0:
            nd += 1
            want = [(i, h) for i, h in enumerate(b["val"]) if abs(u2f(h)) > thr]
            got = list(zip(e["idx"] or [], e["val"]))
            if e["nnz"] != len(want) or got != want:
                bad.append(f"C pivot {e['k']}: dense compaction: want {len(want)} ({want[:3]}...), got {e['nnz']} ({got[:3]}...)")
        elif (b["nnz"], b["idx"], b["val"]) != (e["nnz"], e["idx"], e["val"]):
            bad.append(f"C pivot {e['k']}: sparse output changed before 8b473")
    rep.append(f"C: dense outputs compacted in index order with |x| > spx+0x3a8 ({nd} dense, {len(ev) - nd} sparse)")
    # O: original sum
    paths = {}
    for e in ev:
        paths[(e["path"], e["mod16"], e["alias"])] = paths.get((e["path"], e["mod16"], e["alias"]), 0) + 1
        if e["path"] == "simd" and (e["n"] <= 6 or e["alias"]):
            bad.append(f"O pivot {e['k']}: simd with n {e['n']} alias {e['alias']}")
        if e["path"] == "scalar" and e["n"] > 6 and not e["alias"]:
            bad.append(f"O pivot {e['k']}: scalar with n {e['n']} and no alias")
        got = original_sum(e)
        if struct.pack("<d", got) != struct.pack("<d", u2f(e["beta"])):
            bad.append(f"O pivot {e['k']}: sum {got.hex()} != stored beta {u2f(e['beta']).hex()}")
    chain_same = sum(lanes([u2f(x) for x in e["val"]], chain=True) == u2f(e["beta"]) for e in ev if e["path"] == "simd")
    nsimd = sum(e["path"] == "simd" for e in ev)
    rep.append(f"O: stored beta = sum in captured order for {len(ev)} pivots; paths {sorted(paths.items())}; "
               f"chain combine also equal on {chain_same}/{nsimd} simd pivots")
    # R: Rust operation on the same input, pivots 0..21 (snapshot j holds pivot j-1)
    diff = []
    for e in ev[:RUST_PIVOTS]:
        s = mine.get(e["k"] + 1)
        if s is None:
            bad.append(f"R pivot {e['k']}: no Rust snapshot {e['k'] + 1}")
            continue
        b = e["bt"]
        raw = [u2f(x) for x in b["val"]]
        rb = fh(s["beta"])
        if rb != rust_sum_squares(raw):
            bad.append(f"R pivot {e['k']}: Rust beta {rb.hex()} != sum_squares(BTRAN output, {'dense' if b['nnz'] < 0 else 'sparse'}) "
                       f"{rust_sum_squares(raw).hex()}")
        if rb != u2f(e["beta"]):
            diff.append(e["k"])
            if b["nnz"] >= 0:
                bad.append(f"R pivot {e['k']}: Rust beta differs on a sparse output")
    if not diff:
        bad.append("R: no pivot with a Rust beta difference")
    rep.append(f"R: Rust beta = sum_squares(uncompacted BTRAN output) on pivots 0-{RUST_PIVOTS - 1}; differs from the original on "
               f"pivots {diff} (all dense outputs, compacted sum equals the original there)")
    # F: first divergence in the stored oracle vs the Rust snapshot
    first = None
    for it in sorted(oracle):
        a, s = oracle[it], mine.get(it)
        if s is None:
            break
        same = a["vstat"] == s["vstat"] and all([fh(x) for x in a[k]] == [fh(x) for x in s[k]] for k in ("xB", "d"))
        wa = [fh(x) for x in a["w"]]
        wb = [fh(x) for x in s["w"]]
        same_w = all(wa[a["vstat"][j]] == wb[s["vstat"][j]] for j in range(len(a["vstat"])) if a["vstat"][j] >= 0 and s["vstat"][j] >= 0)
        if not (same and same_w):
            first = (it, "state" if not same else "w")
            break
    if first != (12, "w"):
        bad.append(f"F: first differing snapshot {first}, want (12, 'w')")
    for it in range(1, 13):
        a, s = oracle[it], mine[it]
        if (fh(a["beta"]) == fh(s["beta"])) != (it < 12):
            bad.append(f"F: beta at snapshot {it}: oracle {a['beta']} Rust {s['beta']}")
        if fh(a["ar"]) != fh(s["ar"]):
            bad.append(f"F: alpha_r at snapshot {it}: oracle {a['ar']} Rust {s['ar']}")
    rep.append(f"F: snapshots 0-11 equal (vstat, x_B, d, w), first w difference at 12; beta equal at 1-11, differs at 12; alpha_r equal 1-12")
    return bad, rep


def load_oracle():
    return {s["iter"]: s for s in map(json.loads, gzip.open(ORACLE, "rt"))}


def self_test(runs, oracle, mine):
    def flip(h):
        return format(int(h, 16) ^ 1, "016x")

    tampers = {}
    t = copy.deepcopy(runs); t[0][1]["binary_sha256"] = "0" * 64; tampers["binary sha"] = t
    t = copy.deepcopy(runs); t[2][1]["events"][5]["n"] += 1; t[2][1]["_gz_sha256"] = GZ_SHA; tampers["run 3 differs"] = t
    t = copy.deepcopy(runs); t[1][1]["_gz_sha256"] = "0" * 64; tampers["file sha"] = t
    t = copy.deepcopy(runs)
    for _, d in t:
        d["tool_sha256"] = "1" * 64
    tampers["tool sha (all runs)"] = t
    t = copy.deepcopy(runs)
    for _, d in t:
        d["events"] = []
    tampers["empty events"] = t
    t = copy.deepcopy(runs)
    for _, d in t:
        e = d["events"][11]; e["val"][3] = flip(e["val"][3])
    tampers["pivot 11 rho value bit"] = t
    t = copy.deepcopy(runs)
    for _, d in t:
        e = d["events"][11]; e["beta"] = flip(e["beta"])
    tampers["pivot 11 beta bit"] = t
    t = copy.deepcopy(runs)
    for _, d in t:
        e = d["events"][12]; e["idx"] = e["idx"][1:]; e["val"] = e["val"][1:]; e["nnz"] -= 1
    tampers["dense compaction drops one element"] = t
    out = []
    for name, rr in tampers.items():
        bad, _ = check(rr, oracle, mine)
        out.append((name, bool(bad), bad[:1]))
    m2 = copy.deepcopy(mine)
    m2[12]["beta"] = u2f(runs[0][1]["events"][11]["beta"]).hex()
    bad, _ = check(runs, oracle, m2)
    out.append(("Rust beta replaced by original beta", bool(bad), bad[:1]))
    o2 = copy.deepcopy(oracle)
    p0 = next(p for p in o2[5]["vstat"] if p >= 0)  # position of the first basic variable
    o2[5]["w"][p0] = flip_fh(o2[5]["w"][p0])
    bad, _ = check(runs, o2, mine)
    out.append(("stored oracle w bit at iteration 5", bool(bad), bad[:1]))
    ok = True
    for name, rejected, why in out:
        print(f"  tamper {name}: {'rejected' if rejected else 'NOT REJECTED'} {why[0] if why else ''}")
        ok &= rejected
    return ok


def main():
    runs = load_runs()
    oracle = load_oracle()
    mine = rust_snapshot()
    bad, rep = check(runs, oracle, mine)
    print("\n".join(rep))
    for b in bad:
        print("FAIL", b)
    ok = not bad
    if "--self-test" in sys.argv:
        print("self-test:")
        ok &= self_test(runs, oracle, mine)
    print("PASS" if ok else "FAIL")
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
