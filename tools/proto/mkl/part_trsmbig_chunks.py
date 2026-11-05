"""print the (rows, cols, k) of every dtrsm_kernel_ll_0 call for given shapes"""
import sys
import numpy as np
import mk, part_trsmbig_tr as T
rng = np.random.default_rng(1)
for spec in sys.argv[1:]:
    m, n = map(int, spec.split(','))
    U = rng.random(m * m); B = rng.random(m * n)
    calls = [(r[14], r[15], r[8]) for nm, r in T.run(lambda: mk.trsm_lutu(m, n, U, m, B, m), {0x2443000: ('ll_0', 7)})]
    print(spec, calls if len(calls) < 40 else (len(calls), sorted(set(c[1] for c in calls)), calls[-3:]))
