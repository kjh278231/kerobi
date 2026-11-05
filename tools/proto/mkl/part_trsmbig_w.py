import numpy as np, collections
import mk, part_trsmbig_tr as T
rng = np.random.default_rng(1)
W = collections.defaultdict(list)
for m in list(range(13, 100)) + list(range(100, 400, 7)):
    if m % 4 == 0 and m <= 88: continue
    n = 1000
    U = rng.random(m * m); B = rng.random(m * n)
    calls = [(r[14], r[15], r[8]) for nm, r in T.run(lambda: mk.trsm_lutu(m, n, U, m, B, m), {0x2443000: ('ll_0', 7)})]
    ws = calls[0][1]; ks = [c[2] for c in calls]
    reset = any(ks[i + 1] < ks[i] and calls[i + 1][0] != m for i in range(len(ks) - 1))
    W[(ws, reset)].append(m)
for k, v in sorted(W.items()): print(k, v[0], '..', v[-1], len(v))
