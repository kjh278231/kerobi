"""Compare the port's pass-0 symmetry detection with a symmetry oracle capture (tools/symmetry_oracle.py).

usage: PYTHONPATH=python python tools/symmetry_check.py <oracle.json> [--src DIR] [--param Name=Value ...] [name ...]
Per model: whether `5577ba` is called (count), local_f4, and the work it adds to state+0x458. The port's work before
the call differs from Gurobi's (work counted elsewhere), so RSOLVER_SYM_BASE hands it the oracle's work before the
call and the port reruns the call on a copy from there; that work after must equal the oracle's bit for bit. The
port reports through RSOLVER_SYM_TRACE (stderr, captured here). Models come from the instance list, or DIR/<name>.mps
with --src.
Needs no Gurobi.
"""
import argparse, json, os, struct, sys, tempfile
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
os.environ["RSOLVER_SYM_TRACE"] = "1"
from parity import instances  # noqa: E402
import rsolver  # noqa: E402


def port_calls(path, params, base):
    """(n, m, work before, work after, local_f4, work after from `base`) per 5577ba call of the port's presolve."""
    os.environ.pop("RSOLVER_SYM_BASE", None)
    if base is not None:
        os.environ["RSOLVER_SYM_BASE"] = struct.pack(">d", base).hex()
    saved, tmp = os.dup(2), tempfile.TemporaryFile()
    os.dup2(tmp.fileno(), 2)
    try:
        rsolver.read(str(path))._core.presolve(params)
    finally:
        os.dup2(saved, 2)
        os.close(saved)
    tmp.seek(0)
    bits = lambda h: struct.unpack(">d", bytes.fromhex(h))[0]
    return [(int(n), int(m), bits(b), bits(a), int(f), bits(r)) for t, n, m, b, a, f, r in
            (line.split() for line in tmp.read().decode().splitlines() if line.startswith("5577ba "))]


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("oracle")
    ap.add_argument("--src")
    ap.add_argument("--param", action="append", default=[])
    ap.add_argument("names", nargs="*")
    a = ap.parse_args()
    params = [(k, float(v)) for k, v in (p.split("=", 1) for p in a.param)]
    cap = json.load(open(a.oracle))
    insts = instances()
    ok = {"calls": 0, "f4": 0, "work": 0}
    called = 0
    for name in a.names or sorted(cap):
        want = cap[name]["calls"]
        got = port_calls(Path(a.src) / f"{name}.mps" if a.src else insts[name], params, want[0]["work0"] if want else None)
        called += bool(want)
        if len(got) != len(want):
            print(f"{name:20s} calls {len(got)} want {len(want)}")
            continue
        ok["calls"] += 1
        if not want:
            continue
        line = []
        f4 = all(g[4] == w["f4"] for g, w in zip(got, want))
        work = all(g[5].hex() == w["work1"].hex() for g, w in zip(got, want))
        ok["f4"] += f4
        ok["work"] += work
        for g, w in zip(got, want):
            line.append(f"f4 {g[4]}{'' if g[4] == w['f4'] else ' want ' + str(w['f4'])}, work {w['work0']!r} -> {g[5]!r}"
                        + ("" if g[5].hex() == w["work1"].hex() else f" want {w['work1']!r}") + f" (+{w['work1'] - w['work0']!r})")
        print(f"{name:20s} {'; '.join(line)}")
    print(f"calls match {ok['calls']}/{len(a.names or cap)}; of {called} called: local_f4 {ok['f4']}/{called}, "
          f"work {ok['work']}/{called}")


if __name__ == "__main__":
    main()
