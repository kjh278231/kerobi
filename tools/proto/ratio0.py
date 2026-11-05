"""Ratio-test candidates at iteration 0 from bench/oracle/iter0 (exact state): usage python tools/proto/ratio0.py <name> <leaving var>"""
import gzip, json, sys
H = float.fromhex
sys.path.insert(0, 'tools/proto')
exec(open('tools/proto/dse_weights.py').read().split('if __name__')[0].replace('exec(open(os.path.join(os.path.dirname(os.path.abspath(__file__)), "lu_singleton.py"))', 'exec(open("tools/proto/lu_singleton.py")'))
name, leave_var = sys.argv[1], int(sys.argv[2])
o = json.loads(gzip.open(f'bench/oracle/iter0/{name}.jsonl.gz', 'rt').readline())
n, m = o['n'], o['m']; beg, ind, val = o['beg'], o['ind'], [H(v) for v in o['val']]
vs = o['vstat']; d = [H(x) for x in o['d']]; xb = [H(x) for x in o['xB']]; l = [H(x) for x in o['l']]; u = [H(x) for x in o['u']]
head = [None]*m
for j, s in enumerate(vs):
    if s >= 0: head[s] = j
views = factor_views(m, n, beg, ind, val, head)
r = vs[leave_var]
rho = [0.0]*m
for rr, v in btran_unit(r, m, *views): rho[rr] = v
x = xb[r]; jl = head[r]
delta = x - l[jl] if x < l[jl] else x - u[jl]
print('leaving pos', r, 'x', x, 'l', l[jl], 'u', u[jl], 'delta', delta)
col = lambda j: [(ind[k], val[k]) for k in range(beg[j], beg[j+1])] if j < n else [(j-n, 1.0)]
rows = []
for j in range(n+m):
    if vs[j] >= 0: continue
    a = sum(rho[i]*v for i, v in col(j))
    if abs(a) < 1e-12: continue
    rows.append((j, vs[j], a, d[j], l[j], u[j]))
for t in sorted(rows, key=lambda t: abs(t[3]/t[2])): print('  j %4d vstat %2d alpha % .6g d % .6g  ratio %.6g range %.3g' % (t[0], t[1], t[2], t[3], abs(t[3]/t[2]), t[5]-t[4]))
