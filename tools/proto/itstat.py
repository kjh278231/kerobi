"""IterCount closeness from bench/lp_check.csv: equal, within ±5%, geometric mean of mine/gurobi."""
import csv, math, sys
rows = [r for r in csv.DictReader(open(sys.argv[1] if len(sys.argv) > 1 else "bench/lp_check.csv")) if r["ref_iters"] and r["iters"]]
rat = [float(r["iters"]) / float(r["ref_iters"]) for r in rows]
print(f"n {len(rows)} equal {sum(r == 1 for r in rat)} within5% {sum(abs(r - 1) <= 0.05 for r in rat)} "
      f"geomean {math.exp(sum(map(math.log, rat)) / len(rat)):.3f} mean|log| {sum(abs(math.log(r)) for r in rat) / len(rat):.3f}")
