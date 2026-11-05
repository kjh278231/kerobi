"""Compare the optimize log of rsolver with Gurobi's log.txt of a tools/parity.py capture, line by line, from
"Optimize a model" to the end of the solve, with the run-dependent fields masked (seconds, work units, the time
column of iteration rows). --no-iters also masks simplex iteration rows and counts (simplex internals are G1's);
--summary keeps only the presolve result and summary lines (status, objective, "… solved model …", warnings).
usage: PYTHONPATH=$PWD/python python tools/log_check.py <capture_dir> --param Name=Value ... [--no-iters] [name ...]
"""
import argparse, re, sys, tempfile
from pathlib import Path

import rsolver

DATA = Path(__file__).resolve().parents[2] / "re/bench/data/netlib"


def body(text, no_iters):
    t = re.split(r"\nUser-callback calls|\nPresolve a model", text[text.index("Optimize a model"):])[0].rstrip("\n")
    # PDHG rows between the first and the last are printed when the DisplayInterval bucket of the wall clock changes
    # (`355831`): run-dependent, so only the first and last row of that table are compared
    lines, keep, pdhg, rows = t.splitlines(), [], False, []
    for l in lines + [""]:
        if l.startswith("     Iter       Primal"):
            pdhg = True
            keep.append(l)
        elif pdhg and re.match(r"^ {0,8}\d+  ", l):
            rows.append(l)
        else:
            if pdhg:
                keep += rows[:1] + rows[-1:] if len(rows) > 1 else rows
                pdhg, rows = False, []
            keep.append(l)
    out = []
    for l in keep[:-1]:
        l = re.sub(r"[0-9.]+ seconds \([0-9.]+ work units\)", "T", re.sub(r"time: [0-9.]+s", "time: T", l))
        l = re.sub(r" +\d+s$", " Ts", l)  # time column of iteration rows
        if no_iters:
            if re.match(r"^ +\d+ +[-+0-9.e]+ +[-+0-9.e]+ +[-+0-9.e]+ +Ts$", l):
                continue  # simplex iteration row
            l = re.sub(r"^Solved in \d+", "Solved in N", re.sub(r"^Extra simplex iterations after uncrush: \d+", "Extra N", l))
            if l.startswith("Extra N"):
                continue
        out.append(l)
    return out


ap = argparse.ArgumentParser()
ap.add_argument("ref")
ap.add_argument("--param", action="append", default=[])
ap.add_argument("--no-iters", action="store_true")
ap.add_argument("--summary", action="store_true", help="only the presolve result and summary lines")
ap.add_argument("names", nargs="*")
a = ap.parse_args()
names = a.names or sorted(p.name for p in Path(a.ref).iterdir() if (p / "log.txt").exists())
ok = 0
for n in names:
    m = rsolver.read(DATA / f"{n}.mps.gz")
    f = tempfile.NamedTemporaryFile(suffix=".log", delete=False).name
    for k, v in {"OutputFlag": 1, "LogToConsole": 0, "LogFile": f, "Threads": 1, "WorkLimit": 60}.items():
        m.setParam(k, v)
    for kv in a.param:
        k, v = kv.split("=")
        m.setParam(k, float(v))
    m.optimize()
    got, want = body(Path(f).read_text(), a.no_iters), body((Path(a.ref) / n / "log.txt").read_text(), a.no_iters)
    if a.summary:
        summ = re.compile(r"^(Presolve|Presolved|Solved in|Barrier solved|Barrier performed|PDHG solved|PDHG performed|"
                          r"Optimal objective|Extra simplex|Warning|Infeasible|Unbounded|Model |Work limit|Iteration limit)")
        got, want = [l for l in got if summ.match(l)], [l for l in want if summ.match(l)]
    if got == want:
        ok += 1
    else:
        i = next((i for i, (u, v) in enumerate(zip(got, want)) if u != v), min(len(got), len(want)))
        print(f"{n:12s} line {i}: got {got[i] if i < len(got) else '<end>'!r} want {want[i] if i < len(want) else '<end>'!r}")
print(f"match {ok}/{len(names)}")
