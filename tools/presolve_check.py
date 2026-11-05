"""Compare rsolver's presolved model with a presolve oracle capture (tools/presolve_oracle.py), byte for byte.

usage: PYTHONPATH=python python tools/presolve_check.py <oracle_dir> [--param Name=Value ...] [--src DIR] [-v] [name ...]
Prints one line per mismatch (first differing line) and the match count. Needs no Gurobi.
"""
import argparse, sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from parity import instances  # noqa: E402
import rsolver  # noqa: E402


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("oracle")
    ap.add_argument("--param", action="append", default=[])
    ap.add_argument("-v", action="store_true", help="also print the size of each mismatching model")
    ap.add_argument("--src", help="read the models from DIR/<name>.mps instead of the instance list")
    ap.add_argument("names", nargs="*")
    a = ap.parse_args()
    params = [(k, float(v)) for k, v in (p.split("=", 1) for p in a.param)]
    insts = instances()
    names = a.names or sorted(p.stem for p in Path(a.oracle).glob("*.mps"))
    ok = 0
    for name in names:
        want = (Path(a.oracle) / f"{name}.mps").read_text().splitlines()
        core = rsolver.read(str(Path(a.src) / f"{name}.mps" if a.src else insts[name]))._core
        try:
            got = core.presolve(params).to_mps().splitlines()
        except BaseException as e:  # a Rust panic surfaces as PanicException (BaseException)
            print(f"{name:12s} PANIC {e}")
            continue
        if got == want:
            ok += 1
            continue
        i = next((k for k, (x, y) in enumerate(zip(got, want)) if x != y), min(len(got), len(want)))
        print(f"{name:12s} line {i + 1}: got {got[i] if i < len(got) else '<eof>'!r} want {want[i] if i < len(want) else '<eof>'!r}"
              + (f"  (lines {len(got)} vs {len(want)})" if a.v else ""))
    print(f"match {ok}/{len(names)}")


if __name__ == "__main__":
    main()
