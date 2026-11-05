"""036ade LU prototype: 1da0b2 singleton phases 1-4 + the sparse Markowitz kernel of 036ade, dense FTRAN.

Extends rsolver/tools/proto/lu_singleton.py. Spec: spec_markowitz.md (same folder).
shortcut: no 2x2 block phase (1da0b2 phase 5, kernel >= 50), no dense switch (036ade L1542/L1762), no param_4 path.

usage: python3 lu_kernel.py <snapshots.jsonl> [...]   (oracle records; matrix taken from the first record)
For every record whose LU update count is 0 and whose previous iteration is present, the pre-refactor head is
the previous head with the leaving position given the entering variable. The factor of that head must reproduce
(a) the post-refactor head (Gurobi renumbers positions = factor steps) and (b) the oracle x_B bits.
"""
import gzip, json, sys

H = float.fromhex
ZERO = 1e-13
TOL = 0.0078125  # MarkowitzTol 2^-7 (spx+0x390)


ABL = set()  # ablation switches for the self-check (empty = original rules)


class Kernel(Exception):
    pass


def factor(m, n, beg, ln, ind, val, head, tol=TOL, trace=None):
    col = lambda j: [(ind[k], val[k]) for k in range(beg[j], beg[j] + ln[j])] if j < n else [(j - n, 1.0)]
    # ---- 1da0b2 phase 1: row singletons (stack seeded ascending, LIFO) ----
    rc = [0] * m
    for pos in range(m):
        j = head[pos]
        if j < n:
            for i, _ in col(j): rc[i] += 1
        else:
            rc[j - n] += 1 + m
    rows = [[] for _ in range(m)]
    for pos in range(m):
        j = head[pos]
        if j < n:
            for i, a in col(j):
                if rc[i] <= m: rows[i].append((pos, a))
    prow, ppos, steps = [-1] * m, [-1] * m, []  # steps: (row, pos, diag)
    def pivot(r, pos, d):
        prow[r] = len(steps); ppos[pos] = len(steps); steps.append((r, pos, d))
    stack = [i for i in range(m) if rc[i] == 1]
    while stack:
        r = stack.pop()
        if rc[r] <= 0: continue
        rc[r] = 0
        pos, d = next(x for x in rows[r] if ppos[x[0]] == -1)
        pivot(r, pos, d)
        for i, _ in col(head[pos]):
            if prow[i] == -1:
                rc[i] -= 1
                if rc[i] == 1: stack.append(i)
    s1 = len(steps)
    # ---- phase 2: slacks of rows without structural entries, position order ----
    for pos in range(m):
        j = head[pos]
        if j >= n and rc[j - n] == m + 1:
            rc[j - n] = 1; pivot(j - n, pos, 1.0)
    s2 = len(steps)
    # ---- phase 3: structural length-1 columns (row active) and every other slack, position order ----
    for pos in range(m):
        if ppos[pos] >= 0: continue
        j = head[pos]
        if j < n:
            if ln[j] == 1:
                i, a = col(j)[0]
                if rc[i] <= m and prow[i] < 0: pivot(i, pos, a)
        else:
            rc[j - n] -= m; pivot(j - n, pos, 1.0)
    s3 = len(steps)
    # ---- phase 4: column singletons of the active part (1da0b2 L530-690); skipped when phases 2-3 pivoted
    # nothing (1da0b2 L461 `if (uVar17 == uVar3) goto LAB_001e0920`) ----
    cc = [0] * m
    rcopy = [[] for _ in range(m)]  # active row copy, ascending position
    for c in range(m):
        if ppos[c] == -1:
            for i, a in col(head[c]):
                if prow[i] < 0:
                    cc[c] += 1; rcopy[i].append((c, a))
    stack = [c for c in range(m) if cc[c] == 1] if s3 != s1 and 'nophase4' not in ABL else []
    while stack:
        c = stack.pop()
        if cc[c] <= 0: continue
        r, d = next((i, a) for i, a in col(head[c]) if prow[i] == -1)
        pivot(r, c, d)
        for c2, _ in rcopy[r]:
            if ppos[c2] == -1:
                cc[c2] -= 1
                if cc[c2] == 1: stack.append(c2)
    s4 = len(steps)
    # L, U of the singleton steps straight from the columns (values untouched by the singleton phases)
    lcol = {}   # step -> [(row, l)]
    kernel_u = []  # (step of U row, pos, u) from the kernel
    # ---- 036ade kernel (L622-760 build, L799-1150 singletons, L1526-2570 Markowitz) ----
    if s4 < m:
        # elements [val, row, col]; col = -1 marks dead. Lists head-first.
        rl = [[] for _ in range(m)]
        cl = [[] for _ in range(m)]
        rcs = [-1] * m  # stored count - 1
        ccs = [-1] * m
        for pos in range(m):
            if ppos[pos] >= 0: continue
            for i, a in col(head[pos]):
                if prow[i] < 0 and ZERO <= abs(a):
                    e = [a, i, pos]
                    rl[i].insert(0, e); cl[pos].insert(0, e); rcs[i] += 1; ccs[pos] += 1
        # count buckets: circular lists, insert after head then become head; walk via prv (LIFO)
        class B:
            def __init__(s):
                s.head = [-1] * (m + 1); s.nxt = [0] * m; s.prv = [0] * m
            def ins(s, i, k):
                h = s.head[k]
                if h < 0: s.nxt[i] = s.prv[i] = i
                else:
                    s.nxt[i] = s.nxt[h]; s.nxt[h] = i; s.prv[i] = h; s.prv[s.nxt[i]] = i
                s.head[k] = i
            def rem(s, i, k):
                if s.nxt[i] == i: s.head[k] = -1; return
                if s.head[k] == i: s.head[k] = s.prv[i]
                s.prv[s.nxt[i]] = s.prv[i]; s.nxt[s.prv[i]] = s.nxt[i]
        RB, CB = B(), B()
        erows, ecols = [], []  # empty rows / columns in discovery order (tail steps m-1, m-2, ...)
        for i in range(m):  # rows then column for each index (L700-756)
            if prow[i] < 0:
                if rcs[i] == -1: erows.append(i)
                else: RB.ins(i, rcs[i])
            if ppos[i] < 0:
                if ccs[i] == -1: ecols.append(i)
                else: CB.ins(i, ccs[i])
        alive = lambda lst: [e for e in lst if e[2] != -1]
        nnzB = sum(ln[j] if j < n else 1 for j in head)
        est = float(nnzB)  # local_128
        def put_row(i):
            if rcs[i] < 0: erows.append(i)
            else: RB.ins(i, rcs[i])
        def put_col(j):
            if ccs[j] < 0: ecols.append(j)
            else: CB.ins(j, ccs[j])
        step = s4
        while True:
            if CB.head[0] >= 0:  # column singleton (L1003-1140)
                c = CB.head[0]; CB.rem(c, 0)
                e = alive(cl[c])[0]; r = e[1]
                RB.rem(r, rcs[r])
                d = e[0]; e[2] = -1
                pivot(r, c, d); kind = 'c'
                for f in alive(rl[r]):
                    j = f[2]; kernel_u.append((len(steps) - 1, j, f[0])); f[2] = -1
                    CB.rem(j, ccs[j]); ccs[j] -= 1; put_col(j)
                lcol[len(steps) - 1] = []
                rcs[r] = ccs[c] = -1
            elif RB.head[0] >= 0:  # row singleton (L839-1000)
                r = RB.head[0]; RB.rem(r, 0)
                e = alive(rl[r])[0]; c = e[2]
                CB.rem(c, ccs[c])
                d = e[0]; e[2] = -1
                pivot(r, c, d); kind = 'r'
                L = []
                for f in alive(cl[c]):
                    i = f[1]; L.append((i, f[0] / d)); f[2] = -1
                    RB.rem(i, rcs[i]); rcs[i] -= 1; put_row(i)
                lcol[len(steps) - 1] = L
                rcs[r] = ccs[c] = -1
            else:
                nact = (m - len(steps)) - len(ecols)
                if nact == 0: break
                N2 = float(nact) * float(nact)
                if 0.36 * N2 < est and 4.0 * m < N2 and nact > 50 and m + N2 < 2e9:
                    est = float(sum(ccs[j] + 1 for j in range(m) if ccs[j] >= 0))
                    if 0.325 * N2 < est: raise Kernel('dense switch (fill)')
                cidx = next((k for k in range(1, m) if CB.head[k] >= 0), m)
                ridx = next((k for k in range(1, m) if RB.head[k] >= 0), m)
                k0 = min(cidx, ridx)
                limit = min(max(4, int(k0 * 0.5)), nact) if 'nolimit' not in ABL else 10**9
                if not ((nact - 201) & 0xffffffff > 9798 or float(cidx) * float(ridx) <= nact * 0.1 or 0.25 * N2 <= est):
                    raise Kernel('dense switch (markowitz)')
                if k0 >= m: raise Kernel('0x2732 no candidate')
                best, bcost, bratio, ns = None, float(m) * float(m), 0.0, 0
                bound = float(cidx) * float(ridx)
                k = k0
                done = False
                while k < m and not done:
                    fk = float(k)
                    bound = max(fk * fk, bound)
                    h = CB.head[k]
                    if h >= 0:
                        c = h
                        while True:
                            cm = max([abs(f[0]) for f in alive(cl[c])] + [0.0])
                            for f in alive(cl[c]):
                                cost = float(rcs[f[1]]) * fk; a = abs(f[0])
                                if 'cost_full' in ABL: cost = float(rcs[f[1]] + 1) * (fk + 1)
                                tie = (bratio * cm <= a) if 'tie_ge' in ABL else (bratio * cm < a)
                                if 'notie' in ABL: tie = False
                                if (cost < bcost and tol * cm < a) or (cost == bcost and tie):
                                    best, bcost, bratio = f, cost, a / cm; src = 'm'
                            ns += 1
                            if ns < limit and bound < bcost:
                                c = CB.nxt[c] if 'fifo' in ABL else CB.prv[c]
                                if c == h: break
                                continue
                            CB.head[k] = c if (best[2] == c or 'norot' in ABL) else CB.prv[c]
                            done = True; break
                    if done or bcost <= bound: break
                    h = RB.head[k]
                    if h >= 0:
                        r = h
                        while True:
                            rm = max(abs(f[0]) for f in alive(rl[r]))
                            for f in alive(rl[r]):
                                cost = float(ccs[f[2]]) * fk; a = abs(f[0])
                                rmx = max(abs(x[0]) for x in alive(cl[f[2]])) if 'rowtol_colmax' in ABL else rm
                                if 'cost_full' in ABL: cost = float(ccs[f[2]] + 1) * (fk + 1)
                                tie = (rmx * bratio <= a) if 'tie_ge' in ABL else (rmx * bratio < a)
                                if 'notie' in ABL: tie = False
                                if (cost < bcost and tol * rmx < a) or (cost == bcost and tie):
                                    best, bcost, bratio = f, cost, a / rmx; src = 'R'
                            ns += 1
                            if ns < limit and bound < bcost:
                                r = RB.nxt[r] if 'fifo' in ABL else RB.prv[r]
                                if r == h: break
                                continue
                            RB.head[k] = r if (best[1] == r or 'norot' in ABL) else RB.prv[r]
                            done = True; break
                    if done or bcost <= bound: break
                    k += 1
                if best is None: raise Kernel('0x2732')
                r, c, v = best[1], best[2], best[0]
                est = est + ((float(rcs[r] - 1) * float(ccs[c] - 1)) - 2.0)
                CB.rem(c, ccs[c]); RB.rem(r, rcs[r])
                best[2] = -1
                pivot(r, c, v); kind = src + ('t' if ns >= limit else '')
                sk = len(steps) - 1
                U = []
                for f in alive(rl[r]):  # U row (row list order)
                    j = f[2]; U.append((j, f[0])); f[2] = -1
                    CB.rem(j, ccs[j]); ccs[j] -= 1
                ujs = {j: u for j, u in U}
                L = []
                for f in alive(cl[c]):  # L column (column list order), multiplier a/v
                    i = f[1]; l = f[0] * (1.0 / v) if 'recip' in ABL else f[0] / v; L.append((i, l)); f[2] = -1
                    RB.rem(i, rcs[i]); rcs[i] -= 1
                    seen = set()
                    for g in alive(rl[i]):  # update in place: a - l*u, drop below ZERO
                        j = g[2]
                        if j in ujs:
                            seen.add(j)
                            g[0] = g[0] - l * ujs[j]
                            if not (ZERO <= abs(g[0])):
                                g[2] = -1; rcs[i] -= 1; ccs[j] -= 1
                    for j, u in U:  # fill-in in U order, prepended to row i and column j
                        if j not in seen:
                            p = -(l * u)
                            if ZERO <= abs(p):
                                g = [p, i, j]
                                if 'append' in ABL: rl[i].append(g); cl[j].append(g)
                                else: rl[i].insert(0, g); cl[j].insert(0, g); rcs[i] += 1; ccs[j] += 1
                    put_row(i)
                for j, _ in U: put_col(j)
                rcs[r] = ccs[c] = -1
                lcol[sk] = L
                for j, u in U: kernel_u.append((sk, j, u))
            if trace is not None: trace.append((kind, steps[-1][0], steps[-1][1]))
    else:
        erows, ecols = [], []
    return dict(steps=steps, prow=prow, ppos=ppos, s=(s1, s2, s3, s4), erows=erows, ecols=ecols,
                lcol=lcol, kernel_u=kernel_u, col=col)


def ftran(m, n, head, F, rhs):
    steps, prow, ppos, col = F['steps'], F['prow'], F['ppos'], F['col']
    K = F['s'][3]
    L = [[] for _ in steps]
    Uc = [[] for _ in steps]
    for k, (r, pos, d) in enumerate(steps):
        if k < K:
            for i, a in col(head[pos]):
                ki = prow[i]
                if ki > k: L[k].append((i, a / d))
                elif 0 <= ki < k: Uc[k].append((i, a))
        else:
            L[k] = F['lcol'][k]
            for i, a in col(head[pos]):  # U entries in rows pivoted by the singleton phases (original values)
                if 0 <= prow[i] < K: Uc[k].append((i, a))
    for s, p, u in F['kernel_u']:
        Uc[ppos[p]].append((steps[s][0], u))
    x = list(rhs)
    for k, (r, pos, d) in enumerate(steps):
        xr = x[r]
        if abs(xr) > ZERO:
            for i, l in L[k]: x[i] = x[i] - l * xr
    for k in range(len(steps) - 1, -1, -1):
        r, pos, d = steps[k]
        if abs(x[r]) <= ZERO or abs(x[r] / d) <= ZERO:
            x[r] = 0.0; continue
        x[r] = x[r] / d
        xr = x[r]
        for i, u in Uc[k]: x[i] = x[i] - u * xr
    xb = [0.0] * m
    for r, pos, d in steps: xb[pos] = x[r]
    return xb


def heads(o):
    m = o['m']; head = [None] * m
    for j, s in enumerate(o['vstat']):
        if s >= 0: head[s] = j
    return head


def main(paths):
    for path in paths:
        op = gzip.open if path.endswith('.gz') else open
        recs = [json.loads(l) for l in op(path, 'rt')]
        o0 = recs[0]; n, m = o0['n'], o0['m']
        beg, ln, ind = o0['beg'], o0['len'], o0['ind']; val = [H(v) for v in o0['val']]
        byit = {r['iter']: r for r in recs}
        for r in recs:
            lu = r.get('lu') or {}
            if lu.get('k') != 0 or r['iter'] - 1 not in byit: continue
            prev, post = heads(byit[r['iter'] - 1]), heads(r)
            sp, sq = set(prev), set(post)
            enter, leave = sq - sp, sp - sq
            if len(enter) != 1:
                print(f"{path} it {r['iter']}: basis change {len(enter)} vars, skipped"); continue
            pre = list(prev); pre[prev.index(leave.pop())] = enter.pop()
            tr = []
            try:
                F = factor(m, n, beg, ln, ind, val, pre, trace=tr)
            except Kernel as ex:
                print(f"{path} it {r['iter']}: {ex}"); continue
            st = F['steps']
            order = [pre[p] for _, p, _ in st]
            same = sum(a == b for a, b in zip(order, post))
            s1, s2, s3, s4 = F['s']
            nk = len(st) - s4
            first_diff = next((k for k in range(len(st)) if order[k] != post[k]), None)
            msg = f"{path.split('/')[-1]} it {r['iter']}: m={m} s=({s1},{s2},{s3},{s4}) kernel={nk} singular={len(F['ecols'])} " \
                  f"positions==steps {same}/{len(st)}"
            if first_diff is not None: msg += f" first diff at step {first_diff}"
            if len(st) == m:
                b = [H(v) for v in r['b']] if 'b' in r else [H(v) for v in o0['b']]
                l = [H(v) for v in r['l']]; u = [H(v) for v in r['u']]; vs = r['vstat']
                rhs = list(b)
                for j in range(n + m):
                    s = vs[j]
                    if s >= 0: continue
                    xj = u[j] if s == -2 else l[j] if s in (-1, -4) else 0.0
                    if xj == 0.0: continue
                    for i, a in F['col'](j): rhs[i] = rhs[i] - a * xj
                xb = ftran(m, n, pre, F, rhs)
                # oracle xB is by post-refactor position; ours by pre position -> map through variables
                g = [H(v) for v in r['xB']]
                gv = {post[p]: g[p] for p in range(m)}
                ok = sum(xb[p] == gv[pre[p]] for p in range(m))
                okk = sum(xb[p] == gv[pre[p]] for _, p, _ in st[s4:])
                msg += f"; xB bits {ok}/{m} (kernel positions {okk}/{nk})"
            print(msg)
            if '-v' in sys.argv:
                print('  ours  kernel', order[s4:]); print('  gurobi kernel', post[s4:])


if __name__ == '__main__':
    main([a for a in sys.argv[1:] if not a.startswith('-')])
