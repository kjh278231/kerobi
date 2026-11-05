"""Compare MIP presolve postsolve records with Gurobi's, byte for byte (padding skipped, pointers as offsets from the
record's data): Gurobi's list comes from tools/mip_state_dump.js at the first `3b0917` call (model generation),
the port's from `CoreModel.mip_post_records`. Runs Gurobi (one process: wrapped in tools/gurun.sh).
usage: PYTHONPATH=python python tools/mip_rec_check.py <model> [Param=Value ...]
Prints the record counts, the type counts and the first differing record.
"""
import collections, json, os, subprocess, sys, tempfile
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from mip_dump_check import rec_diffs  # noqa: E402
from parity import instances  # noqa: E402
import rsolver  # noqa: E402

TOOLS = Path(__file__).resolve().parent


def gurobi_records(model, kv):
    with tempfile.TemporaryDirectory() as d:
        out = os.path.join(d, "dump.json")
        subprocess.run([str(TOOLS / "gurun.sh"), "node", str(TOOLS / "mip_state_dump.js"), model, "3b0917", "1", out, *kv],
                       check=True, capture_output=True)
        dumps = json.load(open(out))
    return dumps[0]["s"]["recs"] if dumps else []


def main():
    model, kv = sys.argv[1], sys.argv[2:]
    params = [(k, float(v)) for k, v in (p.split("=") for p in kv)]
    path = model if model.endswith((".mps", ".lp", ".gz")) else str(instances()[model])
    mine = rsolver.read(path)._core.mip_post_records(params)
    theirs = gurobi_records(model, kv)
    tm = collections.Counter(r[0] for r in mine)
    tg = collections.Counter(r[0] for r in theirs)
    bad = next(((i, e) for i, (m, g) in enumerate(zip(mine, theirs)) if (e := rec_diffs(m, g))), None)
    same = bad is None and len(mine) == len(theirs)
    print(f"{model} {' '.join(kv)}: port {len(mine)} records {dict(tm)}, gurobi {len(theirs)} {dict(tg)}: "
          + ("match" if same else f"first difference at {bad[0]}: {bad[1]}" if bad else "prefix matches"))
    sys.exit(0 if same else 1)


if __name__ == "__main__":
    main()
