"""Synthetic symmetric LPs for the pass-0 symmetry detection (docs/m1_presolve_symmetry.md).

A random block of p columns and q rows (the first qe rows `=`, the last of them the sum of the first two, so
`47c445` would drop it; the other rows `<` with slack at a point x0) is copied k times, side by side. The copies are
an equitable partition: `551cc0` finds it and `5577ba` returns local_f4 > 0, which turns automatic PreDepRow off,
so the dependent rows stay. nnz > 2.5 n keeps PreDepRow's automatic value at 3 otherwise. Deterministic (Python's
`random` with a fixed seed), no Gurobi needed.
usage: python tools/symmetry_models.py <out_dir>     writes <out_dir>/<name>.mps for every model in MODELS
"""
import random
import sys
from pathlib import Path

# name: (copies k, columns p, rows q, `=` rows qe, density, seed)
MODELS = {
    "sym_k6": (6, 100, 20, 10, 0.2, 1),
    "sym_k5": (5, 110, 24, 12, 0.2, 2),
    "sym_k8": (8, 70, 16, 8, 0.25, 3),
    "sym_k3": (3, 200, 40, 20, 0.1, 4),
    "sym_k12": (12, 50, 10, 5, 0.3, 5),
}


def block_lp(name, k, p, q, qe, density, seed):
    """MPS text of k side-by-side copies of one random block (see the module doc)."""
    rnd = random.Random(seed)
    val = lambda: round(rnd.choice((-1, 1)) * rnd.uniform(0.5, 5.0), 2)
    a = [[val() if rnd.random() < density else 0.0 for _ in range(p)] for _ in range(q)]
    a[qe - 1] = [round(x + y, 2) for x, y in zip(a[0], a[1])]  # dependent `=` row
    x0 = [round(rnd.uniform(0.0, 10.0), 2) for _ in range(p)]
    cost = [val() for _ in range(p)]
    act = [sum(r[j] * x0[j] for j in range(p)) for r in a]
    rhs = [round(act[i], 6) if i < qe else round(act[i] + rnd.uniform(0.5, 3.0), 6) for i in range(q)]
    rhs[qe - 1] = round(rhs[0] + rhs[1], 6)
    out = [f"NAME {name}", "ROWS", " N  OBJ"]
    out += [f" {'E' if i < qe else 'L'}  R{c}_{i}" for c in range(k) for i in range(q)]
    out.append("COLUMNS")
    for c in range(k):
        for j in range(p):
            out.append(f"    X{c}_{j}  OBJ  {cost[j]!r}")
            out += [f"    X{c}_{j}  R{c}_{i}  {a[i][j]!r}" for i in range(q) if a[i][j] != 0.0]
    out.append("RHS")
    out += [f"    RHS  R{c}_{i}  {rhs[i]!r}" for c in range(k) for i in range(q) if rhs[i] != 0.0]
    out.append("BOUNDS")
    out += [f" UP BND  X{c}_{j}  10" for c in range(k) for j in range(p)]
    out.append("ENDATA")
    return "\n".join(out) + "\n"


def main():
    out = Path(sys.argv[1])
    out.mkdir(parents=True, exist_ok=True)
    for name, spec in MODELS.items():
        (out / f"{name}.mps").write_text(block_lp(name, *spec))
    print(f"{len(MODELS)} models -> {out}")


if __name__ == "__main__":
    main()
