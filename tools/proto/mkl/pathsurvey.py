import json, sys, collections
import numpy as np
import mk, trace
rng = np.random.default_rng(1)
def path(f):
    seq = trace.run(f)
    names = [s for s, c in seq if not s.startswith(('mkl_serv', '__', 'mkl_set_xerbla', 'mkl_blas_errchk', 'check_strategy')) and s not in ('mkl_blas_dgemm', 'mkl_blas_xdgemm', 'mkl_blas_dtrsm', 'mkl_blas_xdtrsm')]
    return tuple(dict.fromkeys(names))
which = sys.argv[1]
res = collections.defaultdict(list)
if which == 'G':
    for m, n, k in json.load(open('../gshapes.json')):
        p = path(lambda: mk.gemm_tn(m, n, k, rng.random(k * m), k, rng.random(k * n), k, rng.random(m * n), m))
        res[p].append((m, n, k))
else:
    for m, n in json.load(open('../tshapes.json')):
        p = path(lambda: mk.trsm_lutu(m, n, rng.random(m * m), m, rng.random(m * n), m))
        res[p].append((m, n))
for p, v in sorted(res.items(), key=lambda kv: -len(kv[1])):
    print(len(v), p)
    print('    ', v[:12])
json.dump({' > '.join(p): v for p, v in res.items()}, open(f'paths_{which}.json', 'w'))
