"""M1-02f C1 (docs/m1/m1_02f_bfrt_rust.md): replay every 1c35c5 call of the M1-02e BFRT corpus through Rust
bfrt::bfrt (crates/lp/tests/bfrt_corpus.rs) and compare with the original outputs and tools/proto/bfrt.py.
usage: python3 tools/m1_bfrt_rust.py [--self-test]
Pass: the input files match bench/oracle/m1_02e_bfrt/SHA256SUMS, Rust equals the prototype on every call, and the only
calls where Rust differs from the original are tuff 31, 33, 36, 37 (the RNG cost shift, M1-02e §6).
--self-test: NONDEGENFLIP forced off must lose kb2 call 21 and match the prototype's BFRT_RESCAN=0 rule; a tampered
expected flip list, a truncated input and a wrong corpus hash must each be rejected."""
import gzip, hashlib, importlib.util, json, os, struct, subprocess, sys, tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
D = ROOT / "bench/oracle/m1_02e_bfrt"
LENS = ROOT / "bench/oracle/iter0-5p"
EXPECT_DIFF = {("tuff", 31), ("tuff", 33), ("tuff", 36), ("tuff", 37)}
spec = importlib.util.spec_from_file_location("proto_bfrt", ROOT / "tools/proto/bfrt.py")
proto = importlib.util.module_from_spec(spec)
spec.loader.exec_module(proto)


def hx(x):
    return struct.pack(">d", x).hex()


def need(c, msg):
    if not c:
        raise SystemExit(f"FAIL: {msg}")


def corpus(sums=None):
    """[(name, call, enter row, leave row, lens)]; checks SHA256SUMS first."""
    sums = sums or (D / "SHA256SUMS").read_text()
    want = dict(reversed(l.split()) for l in sums.strip().splitlines())
    calls = []
    for p in sorted(D.glob("*.jsonl.gz")):
        need(want.get(p.name) == hashlib.sha256(p.read_bytes()).hexdigest(), f"{p.name} sha256 != SHA256SUMS")
        rows = [r for r in map(json.loads, gzip.open(p, "rt")) if r[0] == "bfrt"]
        lens = json.loads(gzip.open(LENS / p.name, "rt").readline())["len"]
        name = p.name[:-9]
        for k in range(0, len(rows) - 1, 2):
            calls.append((name, k // 2, rows[k], rows[k + 1], lens))
    need(len(calls) == 3112, f"{len(calls)} calls, expected 3112")
    return calls


def encode(calls, nondeg_off=False):
    """The candidate list the prototype builds (tools/proto/bfrt.py bfrt()), one call block per corpus call."""
    out = []
    fac = 1.0 * 0.3 + 0.7  # Netlib: no integer columns
    for name, k, e, lv, lens in calls:
        p2, p3, (r, jl, x, lr, ur) = e[2], e[3], e[4]
        cnt, idx, val = e[5]
        d, vs, l, u, big = e[6], e[7], e[8], e[9], e[10]
        need(e[18] < 10, f"{name} call {k}: spx+0x178 = {e[18]} (pow-less branch, not ported)")
        feas = 1e-6 * 1e30 * 1e-5 if big else 1e-6
        delta = (ur - x) if p3 else (x - lr)
        rows = []
        for t in range(cnt):
            if t >= p2 and vs[t] not in (-1, -2):
                continue
            j = idx[t]
            c = float(lens[j]) if j < len(lens) else 0.5
            rows.append(f"{j} {hx(val[t])} {hx(d[t])} {hx(u[t] - l[t])} {int(t >= p2)} {hx(c)} {hx(fac)}")
        on = int(e[17] != 0 and not nondeg_off)
        out.append(f"call {on} {hx(delta)} {hx(feas)} {len(rows)}")
        out += rows
    out.append("end")
    return "\n".join(out) + "\n"


def run_rust(text):
    with tempfile.TemporaryDirectory() as tmp:
        fin, fout = Path(tmp, "in"), Path(tmp, "out")
        fin.write_text(text)
        r = subprocess.run(["cargo", "test", "--release", "-q", "-p", "rsolver-lp", "--test", "bfrt_corpus", "--",
                            "--ignored"], cwd=ROOT, env=dict(os.environ, BFRT_IN=str(fin), BFRT_OUT=str(fout)),
                           capture_output=True, text=True)
        if r.returncode:
            return None
        res = []
        for line in fout.read_text().splitlines():
            t = [int(x) for x in line.split()]
            res.append((None if t[0] < 0 else t[0], t[1:]))
        return res


def positions(calls, res):
    """Rust returns flipped column numbers; the corpus and the prototype list α-vector positions."""
    return [(q, [{j: t for t, j in enumerate(e[5][1])}[j] for j in flips]) for (_, _, e, _, _), (q, flips) in zip(calls, res)]


def check(calls, res, rescan=True):
    """(rust vs original differences, rust vs prototype differences)"""
    need(res is not None and len(res) == len(calls), "Rust replay failed or returned a different call count")
    res = positions(calls, res)
    old = os.environ.get("BFRT_RESCAN")
    os.environ["BFRT_RESCAN"] = "1" if rescan else "0"
    try:
        dg, dp = [], []
        for (name, k, e, lv, lens), (q, flips) in zip(calls, res):
            if (q, flips) != (lv[3], lv[4][1]):
                dg.append((name, k))
            if (q, flips) != proto.bfrt(e, lens):
                dp.append((name, k))
    finally:
        if old is None:
            os.environ.pop("BFRT_RESCAN")
        else:
            os.environ["BFRT_RESCAN"] = old
    return dg, dp


def acceptance(calls):
    res = run_rust(encode(calls))
    dg, dp = check(calls, res)
    res = positions(calls, res)
    nq = sum(q == lv[3] for (_, _, _, lv, _), (q, _) in zip(calls, res))
    nf = sum(f == lv[4][1] for (_, _, _, lv, _), (_, f) in zip(calls, res))
    print(f"calls {len(calls)}: entering equal {nq}, flip list equal {nf}; vs prototype {len(calls) - len(dp)} equal; "
          f"differs from the original at {dg}")
    need(not dp, f"Rust differs from the prototype at {dp[:5]}")
    need(set(dg) == EXPECT_DIFF, f"Rust vs original differences {dg}, expected {sorted(EXPECT_DIFF)}")
    return res


def self_test(calls):
    bad = []
    # M1: NONDEGENFLIP forced off
    res = run_rust(encode(calls, nondeg_off=True))
    dg, dp = check(calls, res, rescan=False)
    print(f"M1 nondeg off: differs from the original at {len(dg)} calls, from the BFRT_RESCAN=0 prototype at {len(dp)}")
    if ("kb2", 21) not in dg or dp:
        bad.append("M1 nondeg off")
    # T1: one expected flip list changed
    t = [list(c) for c in calls]
    i = next(i for i, c in enumerate(t) if c[3][4][1])
    t[i][3] = json.loads(json.dumps(t[i][3]))
    t[i][3][4][1] = t[i][3][4][1][1:]
    try:
        acceptance([tuple(c) for c in t])
        bad.append("T1 expected flip list")
    except SystemExit as ex:
        print(f"T1 rejected: {ex}")
    # T2: truncated input
    text = encode(calls)
    if run_rust(text[: text.rindex("end")]) is not None:
        bad.append("T2 truncated input")
    else:
        print("T2 rejected: truncated input makes the Rust replay fail")
    # T3: corpus hash
    sums = (D / "SHA256SUMS").read_text()
    try:
        corpus(sums.replace(sums.split()[0], "0" * 64, 1))
        bad.append("T3 corpus hash")
    except SystemExit as ex:
        print(f"T3 rejected: {ex}")
    need(not bad, f"NOT REJECTED: {bad}")
    print("SELF-TEST PASS")


def main():
    calls = corpus()
    acceptance(calls)
    if "--self-test" in sys.argv[1:]:
        self_test(calls)
    print("ACCEPTANCE PASS")


if __name__ == "__main__":
    main()
