"""Compare rsolver's MIP presolve with a PRELIMIT sweep (tools/prelimit_sweep.py) at every saved checkpoint k and at
the end, and name the reduction behind the first difference (docs/m2/m2_presolve.md §2a). Needs no Gurobi.
usage: PYTHONPATH=python python tools/sweep_check.py <sweep_dir> [-v] [name ...]
One line per model: ok, or the first k whose output differs, what count k is in the port's trace and the first
differing line; then the number of models matching the default result.
"""
import argparse, sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from parity import instances  # noqa: E402
import rsolver  # noqa: E402


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("sweep")
    ap.add_argument("-v", action="store_true", help="also list the trace around the first difference")
    ap.add_argument("names", nargs="*")
    a = ap.parse_args()
    insts = instances()
    names = a.names or sorted(p.name for p in Path(a.sweep).iterdir() if (p / "sweep.tsv").exists())
    final_ok = 0
    for name in names:
        d = Path(a.sweep) / name
        core = rsolver.read(str(insts[name]))._core
        try:
            trace = core.mip_presolve_trace([])
        except BaseException as e:  # a Rust panic surfaces as PanicException (BaseException)
            print(f"{name:22s} PANIC {e}")
            continue
        ks = sorted(int(p.stem) for p in d.glob("*.mps"))
        last = max(ks)
        bad = None
        for k in ks[1:] + [None]:  # every change point, then the default result (== the last saved file)
            want = (d / f"{last if k is None else k}.mps").read_text().splitlines()
            try:
                got = core.presolve([] if k is None else [("GURO_PAR_PRELIMIT", k)]).to_mps().splitlines()
            except BaseException as e:
                got = [f"PANIC {e}"]
            if got != want:
                i = next((j for j, (x, y) in enumerate(zip(got, want)) if x != y), min(len(got), len(want)))
                bad = (k, i, got[i] if i < len(got) else "<eof>", want[i] if i < len(want) else "<eof>")
                break
        if bad is None:
            final_ok += 1
            print(f"{name:22s} ok ({len(trace)} counts, {len(ks) - 1} change points)")
            continue
        k, i, g, w = bad
        what = "default" if k is None else (trace[k - 1] if 0 < k <= len(trace) else f"beyond trace ({len(trace)})")
        print(f"{name:22s} k={k} [{what}] line {i + 1}: got {g!r} want {w!r}")
        if a.v and k is not None:
            print("    trace:", " ".join(f"{j + 1}:{t}" for j, t in enumerate(trace[max(0, k - 6):k + 3], max(0, k - 6))))
    print(f"default match {final_ok}/{len(names)}")


if __name__ == "__main__":
    main()
