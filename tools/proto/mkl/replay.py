"""replay recorded Gurobi MKL calls through direct ctypes calls (same ld, same alignment) and compare bitwise"""
import sys, gzip, json, collections
import numpy as np
from mk import gemm_tn, trsm_lutu
fh = float.fromhex
st = collections.Counter()
for fn in sys.argv[1:]:
    for r in json.load(gzip.open(fn, "rt"))["calls"]:
        if r["f"] == "DGEMM":
            assert r["ta"] == "T" and r["tb"] == "N"
            out = gemm_tn(r["m"], r["n"], r["k"], [fh(x) for x in r["A"]], r["lda"], [fh(x) for x in r["B"]], r["ldb"],
                          [fh(x) for x in r["C0"]], r["ldc"], (r["Aal"], r["Bal"], r["Cal"]))
            ok = all(float(o).hex() == fh(x).hex() for o, x in zip(out, r["C1"]))
        else:
            out = trsm_lutu(r["m"], r["n"], [fh(x) for x in r["A"]], r["lda"], [fh(x) for x in r["B0"]], r["ldb"], (r["Aal"], r["Bal"]))
            ok = all(float(o).hex() == fh(x).hex() for o, x in zip(out, r["B1"]))
        st[(r["f"], ok)] += 1
print(st)
