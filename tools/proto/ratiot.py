"""Dual ratio-test candidates of the pivot t -> t+1 from bench/oracle/iter0-5p (exact Gurobi state at t, matrix from
snapshot 0): usage python tools/proto/ratiot.py <name> <t>. Prints Gurobi's leaving position and entering column
(fields r/q of snapshot t+1) and the candidates sorted by |d/alpha|."""
import gzip, json, sys
H = float.fromhex
sys.path.insert(0, 'tools/proto')
exec(open('tools/proto/dse_weights.py').read().split('if __name__')[0].replace('exec(open(os.path.join(os.path.dirname(os.path.abspath(__file__)), "lu_singleton.py"))', 'exec(open("tools/proto/lu_singleton.py")'))
name, t = sys.argv[1], int(sys.argv[2])
snaps = [json.loads(x) for x in gzip.open(f'bench/oracle/iter0-5p/{name}.jsonl.gz', 'rt')]
o0 = snaps[0]
o = next(s for s in snaps if s['iter'] == t)
o1 = next(s for s in snaps if s['iter'] == t + 1)
n, m = o0['n'], o0['m']; beg, ind, val = o0['beg'], o0['ind'], [H(v) for v in o0['val']]
vs = o['vstat']; d = [H(x) for x in o['d']]; xb = [H(x) for x in o['xB']]; l = [H(x) for x in o['l']]; u = [H(x) for x in o['u']]
head = [None]*m
for j, s in enumerate(vs):
    if s >= 0: head[s] = j
views = factor_views(m, n, beg, ind, val, head)
r, q = o1['r'], o1['q']
rho = [0.0]*m
for rr, v in btran_unit(r, m, *views): rho[rr] = v
x = xb[r]; jl = head[r]
delta = x - l[jl] if x < l[jl] else x - u[jl]
flips = [j for j in range(n + m) if vs[j] < 0 and o1['vstat'][j] < 0 and vs[j] != o1['vstat'][j]]
print(f'pivot {t}->{t+1}: leaving pos {r} var {jl} x {x} l {l[jl]} u {u[jl]} delta {delta}; gurobi enters {q}, flips {flips}')
col = lambda j: [(ind[k], val[k]) for k in range(beg[j], beg[j+1])] if j < n else [(j-n, 1.0)]
rows = []
for j in range(n+m):
    if vs[j] >= 0: continue
    a = sum(rho[i]*v for i, v in col(j))
    if abs(a) < 1e-12: continue
    rows.append((j, vs[j], a, d[j], l[j], u[j]))
for c in sorted(rows, key=lambda c: abs(c[3]/c[2])):
    print('  j %4d vstat %2d alpha % .6g d % .6g ratio %.6g range %.3g %s' % (c[0], c[1], c[2], c[3], abs(c[3]/c[2]), c[5]-c[4], '<- q' if c[0] == q else ''))
