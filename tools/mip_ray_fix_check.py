#!/usr/bin/env python3
"""M2-06f-5: 6684cd (in-place fix of the infeasible-node ray) replayed from the inputs captured by tools/mip_ray_fix.js.
Contract: docs/m2/mip_infeas_ray_fix.md section 3.

(default)          controls + acceptance on bench/oracle/m2_06f5
--write-manifest   record the SHA-256 of every capture (once, after the capture)
--self-test        also T (capture tampers) and V (rule variants; expected outcome fixed from the branch counts)
--sweep FILE...    replay single exploratory captures, print branch counts (not acceptance)
Exit code 0 only when every requested check passes.
"""
import collections, copy, gzip, hashlib, json, math, os, struct, sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
DATA = os.path.join(ROOT, 'bench/oracle/m2_06f5')
OLD = os.path.join(ROOT, 'bench/oracle/m2_06f4')  # same runs (NodeLimit 2000): pp events, output line
TOOL = os.path.join(ROOT, 'tools/mip_ray_fix.js')
BINARY = '8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c'
RUNS = (1, 2, 3)
CASES = ('markshare_4_0', 'pk1', 'neos-860300', 'tr12-30', 'fhnw-binpack4-4', 'neos5')
PARAMS = {'Threads': 1, 'Seed': 0, 'LogToConsole': 0, 'NodeLimit': 2000}
GATE = struct.unpack('>d', bytes.fromhex('3f1a36e2eb1c432c'))[0]  # 1e-4 as compared in 5fb8ec C:2212
LT, GT, EQ, CONT = ord('<'), ord('>'), ord('='), ord('C')
# rule variants: (name, branch-count key; > 0 means the variant changes a decision or a value, so it must be rejected)
VARIANTS = {
    'fifo': 'cand_multi',        # candidates in push order instead of last-pushed first
    'mark_over': 'mark_kept',    # lists 1/2 overwrite an existing mark (the records do, the lists do not)
    'tol_a': 'small_a',          # no |a| > 0.001 filter
    'age_lt': 'age_eq',          # cut age < L+0x5c instead of <=
    'no_clamp': 'clamp_hit',     # no slack clamp of a negative step
    'no_ratio': 'ratio_hit',     # no ratio test on the other row entries
    'one_flag': 'flag_other',    # one used flag per row instead of one per direction
    'lim_g': 'lim_diff',         # acceptance threshold g instead of min(g, 1e-4)
    'seq_sum': 'seq_diff',       # delta summed left to right (no two-lane split)
    'c_order': 'corder_diff',    # (scalar + lane0) + lane1, the order the C text shows (asm: scalar + (lane0 + lane1))
    'no_zero': 'zeroed',         # no |x| <= 1e-13 -> 0 in the ray update
    'rho_order': 'rho_diff',     # rho += rhs * (s * lambda) instead of (rhs * s) * lambda
    'stop_first': 'acc_cont',    # stop a candidate after its first accepted row
    'w8_next': 'cand_zero',      # candidate end after a zeroed entry adds 8 * (k + 1) instead of 8 * k
}


class Fail(Exception):
    pass


def need(cond, msg):
    if not cond:
        raise Fail(msg)


def sha(p):
    with open(p, 'rb') as fh:
        return hashlib.sha256(fh.read()).hexdigest()


def load(path):
    return json.loads(gzip.decompress(open(path, 'rb').read()))


def f(h):
    return struct.unpack('>d', bytes.fromhex(h))[0]


def h(x):
    return struct.pack('>d', x).hex()


def calls(d):
    """Group the events of one capture into 6684cd calls: fx, the inner events, fx_."""
    out, cur = [], None
    for i, e in enumerate(d['events']):
        if e['e'] == 'fx':
            need(cur is None, f'event {i}: fx inside fx')
            cur = {'fx': e, 'i': i, 'q': [], 'ce': None, 'ce_': None}
        elif e['e'] == 'fx_':
            need(cur is not None, f'event {i}: fx_ without fx')
            cur['ret'] = e
            out.append(cur)
            cur = None
        elif e['e'] == 'q':
            need(cur is not None, f'event {i}: q outside fx')
            cur['q'].append(e)
        elif e['e'] in ('ce', 'ce_'):
            need(cur is not None and cur[e['e']] is None, f'event {i}: {e["e"]} outside fx or repeated')
            cur[e['e']] = e
    need(cur is None, 'capture ends inside fx')
    return out


def lanes_sum(D, ent, ray, lb0, ub0, sd, lam, al, mode, c):
    """Delta of contract section 2.3: D starts at -lam*rhs. ent = [(col, coef)]. mode selects the true order or a variant."""
    cnt = len(ent)

    def scal(D, cc, wv):
        r = ray[cc] * sd
        r2 = lam * wv + r
        if r <= 1e-13:
            if r < -1e-13:
                D = D - ub0[cc] * r
        else:
            D = D - lb0[cc] * r
        if r2 <= 1e-13:
            if r2 < -1e-13:
                D = D + ub0[cc] * r2
        else:
            D = D + lb0[cc] * r2
        return D

    if mode == 'seq':
        for cc, wv in ent:
            D = scal(D, cc, wv)
        return D
    peel = None
    if cnt >= 2:
        peel = 0 if al == 0 else (1 if al & 7 == 0 else None)
        if peel is not None and cnt < peel + 2:
            peel = None
    if peel is None:
        c['lane_scalar'] += 1
        start, end = 0, 0
    else:
        c[f'lane_peel{peel}'] += 1
        start, end = peel, cnt - ((cnt - peel) & 1)
    for t in range(start):
        D = scal(D, *ent[t])
    L = [0.0, 0.0]
    for t in range(start, end, 2):
        p = [ent[t], ent[t + 1]]
        r = [ray[cc] * sd for cc, _ in p]
        r2 = [lam * wv + r[q] for q, (_, wv) in enumerate(p)]
        A = [r[q] > 1e-13 for q in (0, 1)]
        if all(A):
            L = [L[q] - lb0[p[q][0]] * r[q] for q in (0, 1)]
        else:
            L = [L[q] - ((lb0[p[q][0]] * r[q]) if A[q] else 0.0) for q in (0, 1)]
        B = [not A[q] and r[q] < -1e-13 for q in (0, 1)]
        if any(B):
            L = [L[q] - ((ub0[p[q][0]] * r[q]) if B[q] else 0.0) for q in (0, 1)]
        Cm = [r2[q] > 1e-13 for q in (0, 1)]
        if all(Cm):
            L = [L[q] + lb0[p[q][0]] * r2[q] for q in (0, 1)]
        else:
            L = [L[q] + ((lb0[p[q][0]] * r2[q]) if Cm[q] else 0.0) for q in (0, 1)]
        Dm = [not Cm[q] and r2[q] < -1e-13 for q in (0, 1)]
        if any(Dm):
            L = [L[q] + ((ub0[p[q][0]] * r2[q]) if Dm[q] else 0.0) for q in (0, 1)]
    for t in range(end if peel is not None else 0, cnt):
        D = scal(D, *ent[t])
    if mode == 'c_order':
        return (D + L[0]) + L[1]
    return D + (L[0] + L[1])


def replay(call, arrays, V=frozenset(), c=None):
    """Contract section 2.3. Returns the outputs and the 95ce25 entry state; c collects branch counts."""
    c = c if c is not None else collections.Counter()
    fx = call['fx']
    A = lambda k: arrays[fx[k]] if fx.get(k) is not None else None
    n, m, n0, m0, fm = fx['n'], fx['m'], fx['n0'], fx['m0'], fx['fm']
    need(fx['v70'] == 1 and n + m > 0, 'no ray')
    need(fm != 1, 'flip mode 1 (f7dc1/7d3ce, out of scope)')
    need(n0 == n, f'n0 {n0} != n {n} (out of scope)')
    need(fx['w'] is not None, 'work pointer null (out of scope)')
    need(fx['I0M'] == 1, 'I[0] != M')
    c['x32e8_null' if fx['x32e8'] == 0 else 'x32e8_set'] += 1
    C = [f(x) for x in A('C')] if fx.get('C') else None
    need(C is not None, 'no column scale (out of scope)')
    R = [f(x) for x in A('R')]
    lb0, ub0 = [f(x) for x in A('lb0')], [f(x) for x in A('ub0')]
    slb, sub = [f(x) for x in A('slb')], [f(x) for x in A('sub')]
    vt, sense, flip = A('vt'), A('sense'), A('flip')
    chain, cols, rows, age = A('chain'), A('cols'), A('rows'), A('age')
    for v in lb0 + ub0 + slb + sub + C + R:
        need(not math.isnan(v), 'NaN input (out of scope)')
    inf = f(fx['inf'])
    N = n + m
    ray = [0.0] * N
    for k, v in fx['ray'].items():
        ray[int(k)] = f(v)
    backup = list(ray)
    s = fx['l54']
    sd, nsd = float(s), float(-s)
    g, rho, l5c_in = f(fx['l68']), f(fx['l60']), fx['l5c']
    w = f(fx['w'])
    U, Lo, inl = [0] * n, [0] * n, [0] * n
    used = [0] * m
    qs = call['q']
    qi = 0
    depth, levels, changes = fx['depth'], 0, 0
    c['calls'] += 1
    if not chain:
        c['root'] += 1
    for node in chain:
        lst = []

        def push(v):
            lst.append(v)
            inl[v] = 1
        for v in node['l1']:
            r = sd * ray[v]
            if U[v] != 0 and r < -1e-10 and not inl[v]:
                c['mark_kept'] += 1
            if U[v] == 0 or 'mark_over' in V:
                if r < -1e-10 and not inl[v]:
                    push(v)
                U[v] = depth
        w = float(len(node['l1'])) * 3.0 + w
        for v in node['l2']:
            r = sd * ray[v]
            if Lo[v] != 0 and r > 1e-10 and not inl[v]:
                c['mark_kept'] += 1
            if Lo[v] == 0 or 'mark_over' in V:
                if r > 1e-10 and not inl[v]:
                    push(v)
                Lo[v] = depth
        w = w + float(len(node['l2'])) * 3.0
        for sg, v in node['rec']:
            r = sd * ray[v]
            if sg == LT:
                if r < -1e-10 and not inl[v]:
                    push(v)
                U[v] = depth
            else:
                if r > 1e-10 and not inl[v]:
                    push(v)
                Lo[v] = depth
                if sg != GT:
                    c['rec_eq'] += 1
                    if r < -1e-10 and not inl[v]:
                        push(v)
                    U[v] = depth
        w = w + float(len(node['rec'])) * 4.0
        c['cand'] += len(lst)
        on = 0
        order = lst if 'fifo' in V else lst[::-1]
        for v in order:
            inl[v] = 0
            t = ray[v] * sd
            lim = g if 'lim_g' in V else (1e-4 if 1e-4 <= g else g)
            if not ((t > 1e-10 and Lo[v] > 0) or (t < -1e-10 and U[v] > 0)):
                c['cand_off'] += 1
                continue
            c['cand_on'] += 1
            on += 1
            if on == 2:
                c['cand_multi'] += 1
            col = cols.get(str(v))
            need(col is not None, f'column {v} not captured')
            cidx, cval = col[0], [f(x) for x in (col[1] or [])]
            k, cnt, ended = 0, len(cidx), False
            while k < cnt:
                i = cidx[k]
                a = cval[k] / (C[v] * R[i])
                if not 0.001 < abs(a):
                    if a != 0.0:
                        c['small_a'] += 1
                    if not ('tol_a' in V and a != 0.0):
                        k += 1
                        continue
                need(qi < len(qs) and qs[qi]['row'] == i, f'788a84 query {qi} for row {i}: captured {qs[qi]["row"] if qi < len(qs) else None}')
                ret = qs[qi]['ret']
                qi += 1
                if ret != 0:
                    c['q_skip'] += 1
                    k += 1
                    continue
                if i >= m0:
                    rr = rows.get(str(i))
                    need(rr is not None, f'row {i} not captured')
                    ag = age[i - m0]
                    if ag == l5c_in:
                        c['age_eq'] += 1
                    if (ag >= l5c_in) if 'age_lt' in V else (ag > l5c_in):
                        c['age_skip'] += 1
                        k += 1
                        continue
                lam = (ray[v] * nsd) / a
                sig = 1.0 if lam > 0.0 else -1.0
                if sig < 0.0 and sense[i] != EQ:
                    tt = ray[n0 + i] * sd
                    if tt <= 0.0:
                        tt = 0.0
                    if lam <= -tt:
                        if lam != -tt:
                            c['clamp_hit'] += 1
                        if 'no_clamp' not in V:
                            lam = -tt
                if sig * lam <= 1e-10:
                    c['step_small'] += 1
                    k += 1
                    continue
                bit = 2 if sig <= 0.0 else 1
                if used[i] & (3 if 'one_flag' in V else bit):
                    c['flag_skip'] += 1
                    k += 1
                    continue
                if used[i] & (3 - bit):
                    c['flag_other'] += 1
                used[i] |= bit
                rr = rows.get(str(i))
                need(rr is not None, f'row {i} not captured')
                ent = list(zip(rr['idx'], [f(x) for x in (rr['val'] or [])]))
                rhs, al = f(rr['rhs']), rr['al']
                if i >= m0:
                    c['cut_row'] += 1
                c['sig_pos' if sig > 0 else 'sig_neg'] += 1
                abort = None
                for t2, (cc, wv) in enumerate(ent):
                    if cc == v:
                        continue
                    for side in (1, -1):
                        if side == 1:
                            ok = sig * wv > 1e-13 and sd * ray[cc] <= 1e-10 and (lb0[cc] <= -1e30 or Lo[cc] > 0 or vt[cc] == CONT)
                        else:
                            ok = sig * wv < -1e-13 and -1e-10 <= sd * ray[cc] and (1e30 <= ub0[cc] or U[cc] > 0 or vt[cc] == CONT)
                        if ok:
                            q = (ray[cc] * nsd) / wv
                            new = (q if lam <= q else lam) if sig <= 0.0 else (q if q <= lam else lam)
                            if new != lam:
                                c['ratio_hit'] += 1
                                if 'no_ratio' not in V:
                                    lam = new
                    x = sig * lam
                    if x <= 1e-10 or 1e8 < x:
                        abort = t2
                        break
                if abort is not None:
                    c['ratio_abort'] += 1
                    w = float(abort) * 7.0 + w
                    k += 1
                    continue
                w = float(len(ent)) * 7.0 + w
                D0 = -(lam * rhs)
                mode = 'seq' if 'seq_sum' in V else ('c_order' if 'c_order' in V else 'asm')
                D = lanes_sum(D0, ent, ray, lb0, ub0, sd, lam, al, mode, c)
                cnt_r = len(ent)
                w = float(cnt_r) * 5.0 + w
                acc = not (D + g < lim)  # comisd + jbe: accepted when lim <= D + g or unordered
                if mode == 'asm':
                    ds = lanes_sum(D0, ent, ray, lb0, ub0, sd, lam, al, 'seq', collections.Counter())
                    dc = lanes_sum(D0, ent, ray, lb0, ub0, sd, lam, al, 'c_order', collections.Counter())
                    if (acc or not (ds + g < lim)) and h(ds) != h(D):
                        c['seq_diff'] += 1
                    if (acc or not (dc + g < lim)) and h(dc) != h(D):
                        c['corder_diff'] += 1
                    lg = (not (D + g < g))
                    if lg != acc:
                        c['lim_diff'] += 1
                margin = D + g - lim
                c['_margin'] = min(c.get('_margin', math.inf), abs(margin))
                if not acc:
                    c['reject'] += 1
                    k += 1
                    continue
                c['accept'] += 1
                ls = lam * sd
                for cc, wv in ent:
                    x = wv * ls + ray[cc]
                    if abs(x) <= 1e-13 and x != 0.0:
                        c['zeroed'] += 1
                    ray[cc] = 0.0 if abs(x) <= 1e-13 and 'no_zero' not in V else x
                w = w + float(cnt_r) * 3.0
                x = ls + ray[n0 + i]
                if abs(x) <= 1e-13 and x != 0.0:
                    c['zeroed'] += 1
                ray[n0 + i] = 0.0 if abs(x) <= 1e-13 and 'no_zero' not in V else x
                g = D + g
                changes += 1
                r1 = (rhs * sd) * lam + rho
                r2 = rhs * (sd * lam) + rho
                if h(r1) != h(r2):
                    c['rho_diff'] += 1
                rho = r2 if 'rho_order' in V else r1
                if 1e-10 < abs(ray[v]):
                    c['acc_cont'] += 1
                    if 'stop_first' not in V:
                        k += 1
                        continue
                else:
                    c['cand_zero'] += 1
                w = w + float(k + (1 if 'w8_next' in V else 0)) * 8.0
                ended = True
                break
            if not ended:
                w = w + float(cnt) * 8.0
        w = w + float(len(lst))
        levels += 1
        depth -= 1
    need(qi == len(qs), f'{len(qs) - qi} unused 788a84 calls')
    w = w + (float(levels) + float(levels))
    c['levels'] += levels
    o = {'rc': 0, 'l5c': fx['l5c'], 'l60': f(fx['l60']), 'l68': f(fx['l68']), 'ce': None}
    if changes > 0:
        c['changed'] += 1
        o['ce'] = {'ray': list(ray), 'rho': rho, 'g': g, 'w': w, 's': s}
        nu2 = 1e-200 if fm == 2 else 0.0
        act = 0.0
        for k in range(n):
            r = ray[k]
            fk = fm == 1 and flip[k] == 1
            if sd * r <= 0.0:
                if fk:
                    b = -slb[k] * C[k]
                else:
                    b = sub[k]
                    b = b * C[k] if b < inf else 1e100
            else:
                b = -1e100 if fk or slb[k] <= -inf else slb[k] * C[k]
            act = act + b * r
            nu2 = nu2 + r * r
        w = float(n) * 4.0 + w
        if nu2 < 1e-4:
            c['fail5'] += 1
            o['ce_'] = {'fail': 5, 'rho': rho, 'l5c': fx['l5c'], 'g': g, 'w': w}
            ray = backup
        else:
            nu = math.sqrt(nu2)
            g = (sd * (act - rho)) / nu
            ray = [x / nu for x in ray]
            w = float(N) + w
            rho = rho / nu
            mx = 0
            for t in range(m - m0):
                if 1e-10 < abs(ray[n + m0 + t]):
                    need(age[t] is not None, f'cut {t} unreadable')
                    if age[t] > mx:
                        mx = age[t]
            cntc = m if m0 < m else m0
            w = float(cntc + cntc) + w
            o['ce_'] = {'fail': 0, 'rho': rho, 'l5c': mx, 'g': g, 'w': w}
            o.update(l5c=mx, l60=rho, l68=g)
    else:
        c['unchanged'] += 1
    o['ray'] = ray
    o['w'] = w
    return o


def nzmap(vec):
    return {str(i): h(v) for i, v in enumerate(vec) if h(v) != '0' * 16}


def cmp_ray(got, exp, where):
    if got != exp:
        diff = sorted(set(got) ^ set(exp) | {i for i in set(got) & set(exp) if got[i] != exp[i]}, key=int)
        i = diff[0]
        raise Fail(f'{where}[{i}] {got.get(i, "0" * 16)} != {exp.get(i, "0" * 16)} ({len(diff)} entries differ)')


def compare(call, o, where):
    r = call['ret']
    need(r['rc'] == o['rc'], f'{where}: rc {r["rc"]} != {o["rc"]}')
    need(r['same'] == 1, f'{where}: ray pointer moved')
    need(r['l5c'] == o['l5c'], f'{where}: L+0x5c {r["l5c"]} != {o["l5c"]}')
    for k in ('l60', 'l68'):
        need(r[k] == h(o[k]), f'{where}: L+0x{k[1:]} {r[k]} ({f(r[k])!r}) != {h(o[k])} ({o[k]!r})')
    need(r['w'] is not None and r['w'] == h(o['w']), f'{where}: work {r["w"] and f(r["w"])!r} != {o["w"]!r}')
    if o['ce'] is None:
        need(call['ce'] is None and call['ce_'] is None, f'{where}: 95ce25 called, replay made no change')
    else:
        ce, ce_ = call['ce'], call['ce_']
        need(ce is not None and ce_ is not None, f'{where}: 95ce25 not called, replay made {o["ce"]} changes')
        need(ce['same'] == 1 and ce['s'] == o['ce']['s'], f'{where}: 95ce25 ray pointer or s')
        cmp_ray(ce['ray'], nzmap(o['ce']['ray']), f'{where}: 95ce25 entry ray')
        for k in ('rho', 'g', 'w'):
            need(ce[k] == h(o['ce'][k]), f'{where}: 95ce25 entry {k} {ce[k]} ({f(ce[k])!r}) != {o["ce"][k]!r}')
        need(ce_['fail'] == o['ce_']['fail'], f'{where}: 95ce25 fail {ce_["fail"]} != {o["ce_"]["fail"]}')
        need(ce_['l5c'] == o['ce_']['l5c'] or ce_['fail'] != 0, f'{where}: 95ce25 L5c {ce_["l5c"]} != {o["ce_"]["l5c"]}')
        for k in ('rho', 'g', 'w'):
            need(ce_[k] == h(o['ce_'][k]), f'{where}: 95ce25 leave {k} {ce_[k]} ({f(ce_[k])!r}) != {o["ce_"][k]!r}')
    cmp_ray(r['ray'], nzmap(o['ray']), f'{where}: ray')
    d = sum(1 for k in set(r['ray']) | set(call['fx']['ray']) if r['ray'].get(k) != call['fx']['ray'].get(k))
    need(r['d'] == d, f'{where}: changed entry count {r["d"]} != {d}')


def check_capture(d, name, V=frozenset(), c=None):
    """Replay every 6684cd call; gate fields at entry; classify the successful 95da98 calls 6684cd did not follow."""
    c = c if c is not None else collections.Counter()
    arrays = d['arrays']
    ev = d['events']
    cs = calls(d)
    for n, call in enumerate(cs):
        fx = call['fx']
        where = f'{name} call {n} (event {call["i"]})'
        need(fx['l38'] == 3 and fx['l58'] >= 0 and f(fx['l68']) >= GATE and fx['v70'] == 1 and fx['g2a48'] != 0,
             f'{where}: gate fields L+0x38 {fx["l38"]} L+0x58 {fx["l58"]} L+0x68 {f(fx["l68"])!r} ray {fx["v70"]} env+0x2a48 {fx["g2a48"]}')
        need(fx['fk'] >= 0 and ev[fx['fk']]['e'] == 'fk', f'{where}: no 95da98 event filled this L')
        fk = ev[fx['fk']]
        need(fk['rc'] == 0 and fk['f'] == 0 and fk['l58'] == fx['l58'], f'{where}: the 95da98 that filled L did not succeed')
        o = replay(call, arrays, V, c)
        compare(call, o, where)
        if call['ret']['d']:
            c['d_pos'] += 1
        else:
            c['d_zero'] += 1
    gate(ev, used={call['fx']['fk'] for call in cs}, name=name, c=c)
    return c


SITE_5FB8EC = 0x510BC0  # return address of the 5ce67b call in 5fb8ec (C:1106)
OTHER = {0x736C17: '8faaa4', 0x7229A3: '8e4682'}  # LP diving, sub-MIP construction: they never call 6684cd


def gate_true(e):
    return e['l38'] == 3 and e['l58'] >= 0 and f(e['l68']) >= GATE and e['v70'] == 1


def gate(ev, used, name, c):
    """Contract Q2. Every successful 95da98 with L+0x68 >= 1e-4 is either read by a 6684cd call, or comes from a 5ce67b
    caller other than 5fb8ec, or its 5fb8ec call returned at C:802-804 (5eda00 != 1) before the gate probe.
    Every gate probe (0x5124b4) with the gate fields true is followed by 6684cd on the same L, and only then."""
    for i, e in enumerate(ev):
        if e['e'] == 'gt':
            nxt = next((x for x in ev[i + 1:] if x['e'] in ('fx', 'gt', 'nb_')), None)
            if gate_true(e):
                need(nxt is not None and nxt['e'] == 'fx' and e['k'] >= 0 and nxt['fk'] == e['k'], f'{name} event {i}: gate true, no 6684cd on that L')
                c['gt_true'] += 1
            else:
                need(nxt is None or nxt['e'] != 'fx', f'{name} event {i}: gate false, 6684cd called')
                c['gt_false'] += 1
        elif e['e'] == 'fx':
            prv = next((x for x in reversed(ev[:i]) if x['e'] in ('gt', 'fx', 'nb_')), None)
            need(prv is not None and prv['e'] == 'gt' and prv['k'] == e['fk'], f'{name} event {i}: 6684cd without the gate probe')
        if not (e['e'] == 'fk' and e['rc'] == 0 and e['f'] == 0 and f(e['l68']) >= GATE):
            continue
        c['fk_gate'] += 1
        if i in used:
            c['fk_called'] += 1
            continue
        if e['ra5'] != SITE_5FB8EC:
            need(e['ra5'] in OTHER, f'{name} event {i}: uncalled 95da98 from 5ce67b caller {e["ra5"]:#x}')
            c[f'nc_{OTHER[e["ra5"]]}'] += 1
            continue
        end = next((j for j in range(i + 1, len(ev)) if ev[j]['e'] == 'nb_'), None)
        need(end is not None, f'{name} event {i}: no 5fb8ec leave after the fk')
        win = ev[i + 1:end]
        stop = [x for x in win if x['e'] == 'ck' and x['ret'] != 1]
        need(stop and stop[0]['ra'] == 0x511AA9 or (stop and stop[0]['ra'] == 0x510C53),
             f'{name} event {i}: uncalled in 5fb8ec without a 5eda00 stop')
        need(not any(x['e'] in ('gt', 'fx') for x in win), f'{name} event {i}: gate probe or 6684cd after the 5eda00 stop')
        need(all(x['ret'] == 1 for x in win if x['e'] == 'ck' and x is not stop[0] and win.index(x) < win.index(stop[0])),
             f'{name} event {i}: 5eda00 stop is not the first non-1 return')
        need(ev[end]['rc'] == 0 and any(gate_true(x) for x in ev[end]['ls'] if x['k'] == i),
             f'{name} event {i}: 5fb8ec leave rc or L state')
        c[f'nc_5eda00_{stop[0]["ret"]}_d8_{e["d8"]}'] += 1


def pp_check(d, o, name, c):
    """Independent check against M2-06f-4 run 1 (same run): its pp events are the 6684cd calls on an L a successful
    95da98 filled, with the entries that differ from the vector before and L+0x5c/60/68 after."""
    mine = calls(d)
    theirs = [e for e in o['events'] if e['e'] == 'pp']
    need(len(mine) == len(theirs), f'{name}: {len(mine)} 6684cd calls here, {len(theirs)} pp in M2-06f-4 run 1')
    for t, (a, b) in enumerate(zip(mine, theirs)):
        need('moved' not in b, f'{name} pp {t}: moved')
        ch = {k: v for k, v in a['ret']['ray'].items() if a['fx']['ray'].get(k) != v}
        ch.update({k: '0' * 16 for k in a['fx']['ray'] if k not in a['ret']['ray']})
        need(a['ret']['d'] == b['d'] and ch == b['ch'], f'{name} pp {t}: changed entries differ from M2-06f-4')
        need((a['ret']['l5c'], a['ret']['l60'], a['ret']['l68']) == (b['l5c'], b['l60'], b['l68']),
             f'{name} pp {t}: L+0x5c/60/68 differ from M2-06f-4')
        c['pp_equal'] += 1


def controls(caps, manifest, old):
    need(manifest['binary_sha256'] == BINARY, 'manifest binary hash')
    need(manifest['tool_sha256'] == sha(TOOL), 'manifest tool hash differs from tools/mip_ray_fix.js')
    for case in CASES:
        runs = [caps[(case, r)] for r in RUNS]
        for r, d in zip(RUNS, runs):
            name = f'{case}_{r}'
            need(d['binary_sha256'] == BINARY, f'{name}: binary hash')
            need(d['tool_sha256'] == manifest['tool_sha256'], f'{name}: tool hash')
            need(d['params'] == PARAMS, f'{name}: params')
            need(d['ok'] and not d['errors'], f'{name}: run not ok')
            need(len(d['events']) > 0 or case == 'neos5', f'{name}: no events')
            need(d['model_sha256'] == manifest['models'][case], f'{name}: model hash')
        for r, d in zip(RUNS[1:], runs[1:]):
            need(d['events'] == runs[0]['events'] and d['arrays'] == runs[0]['arrays'] and d['output'] == runs[0]['output'],
                 f'{case}: run {r} differs from run 1')
        need(runs[0]['output'] == old[case]['output'], f'{case}: output line {runs[0]["output"]!r} != M2-06f-4 {old[case]["output"]!r}')


def load_all():
    caps = {(c, r): load(os.path.join(DATA, f'{c}_{r}.json.gz')) for c in CASES for r in RUNS}
    old = {c: load(os.path.join(OLD, f'{c}_1.json.gz')) for c in CASES}
    manifest = json.load(open(os.path.join(DATA, 'manifest.json')))
    for c in CASES:
        for r in RUNS:
            fn = f'{c}_{r}.json.gz'
            need(manifest['files'][fn] == sha(os.path.join(DATA, fn)), f'{fn}: file hash differs from the manifest')
    return caps, old, manifest


def show(name, c):
    mg = c.pop('_margin', None)
    print(f'  {name}: ' + ' '.join(f'{k}={v}' for k, v in sorted(c.items())) + (f' margin_min={mg:.3g}' if mg is not None else ''))


def acceptance(caps, old):
    tot = collections.Counter()
    for case in CASES:
        c = check_capture(caps[(case, 1)], case)
        pp_check(caps[(case, 1)], old[case], case, c)
        show(case, c)
        tot.update(c)
    need(tot['calls'] > 0 and tot['changed'] > 0 and tot['accept'] > 0, 'no 6684cd call changed the ray')
    need(tot['fk_gate'] > tot['fk_called'], 'no uncalled 95da98 success (gate question has no event)')
    need(any(k.startswith('nc_5eda00') for k in tot) and tot['nc_8faaa4'] + tot['nc_8e4682'] > 0, 'an uncalled class has no event')
    return tot


def tampers(caps, old, manifest):
    base = caps[('pk1', 1)]
    cs = calls(base)
    ci = next(c['i'] for c in cs if c['ce'] is not None)
    call = next(c for c in cs if c['i'] == ci)
    ri = base['events'].index(call['ret'])
    cei = base['events'].index(call['ce'])
    ce_i = base['events'].index(call['ce_'])
    T = []

    def ev(fn):
        def t():
            d = copy.deepcopy(base)
            fn(d)
            check_capture(d, 'pk1')
        return t

    def bit(x):
        return x[:-1] + format(int(x[-1], 16) ^ 1, 'x')

    def setf(i, k, fn):
        return ev(lambda d: d['events'][i].__setitem__(k, fn(d['events'][i][k])))

    T.append(('fx_ l5c', setf(ri, 'l5c', lambda x: x + 1)))
    T.append(('fx_ l60 bit', setf(ri, 'l60', bit)))
    T.append(('fx_ l68 bit', setf(ri, 'l68', bit)))
    T.append(('fx_ work bit', setf(ri, 'w', bit)))
    T.append(('fx_ rc', setf(ri, 'rc', lambda x: 1)))
    T.append(('fx_ ray bit', ev(lambda d: (lambda r: r.__setitem__(next(iter(r)), bit(r[next(iter(r))])))(d['events'][ri]['ray']))))
    T.append(('fx_ changed count', setf(ri, 'd', lambda x: x + 1)))
    T.append(('ce ray bit', ev(lambda d: (lambda r: r.__setitem__(next(iter(r)), bit(r[next(iter(r))])))(d['events'][cei]['ray']))))
    T.append(('ce g bit', setf(cei, 'g', bit)))
    T.append(('ce rho bit', setf(cei, 'rho', bit)))
    T.append(('ce_ fail', setf(ce_i, 'fail', lambda x: 5)))
    T.append(('ce dropped', ev(lambda d: d['events'].pop(cei))))
    T.append(('q ret flip', ev(lambda d: next(e for e in d['events'] if e['e'] == 'q').__setitem__('ret', 1))))
    T.append(('q dropped', ev(lambda d: d['events'].pop(next(i for i, e in enumerate(d['events']) if e['e'] == 'q')))))
    T.append(('gate l68 below', setf(ci, 'l68', lambda x: h(5e-5))))
    T.append(('fx without fk', setf(ci, 'fk', lambda x: -1)))
    T.append(('n0 != n', setf(ci, 'n0', lambda x: x + 1)))
    T.append(('work null', setf(ci, 'w', lambda x: None)))

    def arr(key, fn):
        def g(d):
            hk = d['events'][ci][key]
            new = copy.deepcopy(d['arrays'][hk])
            fn(new)
            d['arrays']['tampered'] = new
            d['events'][ci][key] = 'tampered'
        return ev(g)

    def chain_var(x):
        nd = next(nd for nd in x if nd['rec'])
        nd['rec'][-1][0] = GT if nd['rec'][-1][0] == LT else LT
    T.append(('chain record sense', arr('chain', chain_var)))
    T.append(('chain dropped', arr('chain', lambda x: x.pop())))

    def col_val(x):
        v = next(k for k, cv in x.items() if cv and cv[1])
        x[v][1][0] = bit(x[v][1][0])
    T.append(('column value bit', arr('cols', col_val)))

    def row_val(x):
        i = next(k for k, r in x.items() if r and r['val'])
        x[i]['val'][0] = h(f(x[i]['val'][0]) * 1.5)
    T.append(('row value', arr('rows', row_val)))
    T.append(('row alignment', arr('rows', lambda x: [r.__setitem__('al', 8 - r['al']) for r in x.values() if r])))
    T.append(('lb0 entry', arr('lb0', lambda x: x.__setitem__(0, h(-1e31) if f(x[0]) > -1e30 else h(0.0)))))

    def mani(fn):
        def t():
            m2 = copy.deepcopy(manifest)
            fn(m2)
            controls(caps, m2, old)
        return t
    T.append(('binary hash', mani(lambda m: m.__setitem__('binary_sha256', '0' * 64))))
    T.append(('tool hash', mani(lambda m: m.__setitem__('tool_sha256', '0' * 64))))

    def capt(fn):
        def t():
            c2 = dict(caps)
            c2[('pk1', 2)] = copy.deepcopy(caps[('pk1', 2)])
            fn(c2[('pk1', 2)])
            controls(c2, manifest, old)
        return t
    T.append(('run 2 event', capt(lambda d: d['events'][ri].__setitem__('l5c', 99))))
    T.append(('run 2 empty', capt(lambda d: d.__setitem__('events', []))))
    T.append(('model hash', capt(lambda d: d.__setitem__('model_sha256', '0' * 64))))
    T.append(('output line', capt(lambda d: d.__setitem__('output', 'DONE 8 1999'))))

    def pp_t(fn):
        def t():
            o2 = copy.deepcopy(old['pk1'])
            fn(o2)
            pp_check(base, o2, 'pk1', collections.Counter())
        return t
    T.append(('06f-4 pp value', pp_t(lambda o2: (lambda e: e['ch'].__setitem__(next(iter(e['ch'])), bit(e['ch'][next(iter(e['ch']))])))(
        next(e for e in o2['events'] if e['e'] == 'pp' and e.get('d'))))))
    T.append(('06f-4 pp dropped', pp_t(lambda o2: o2['events'].remove(next(e for e in o2['events'] if e['e'] == 'pp')))))

    def no_call():
        d = copy.deepcopy(base)
        d['events'] = [e for e in d['events'] if e['e'] not in ('fx', 'fx_', 'q', 'ce', 'ce_')]
        tot = check_capture(d, 'pk1')
        need(tot['calls'] > 0 and tot['changed'] > 0 and tot['accept'] > 0, 'no 6684cd call changed the ray')
    T.append(('no 6684cd calls', no_call))

    def gate_t(case, fn):
        def t():
            d = copy.deepcopy(caps[(case, 1)])
            fn(d['events'])
            check_capture(d, case)
        return t

    def uncalled(evs, site):
        used = {e['fk'] for e in evs if e['e'] == 'fx'}
        return next(e for i, e in enumerate(evs) if e['e'] == 'fk' and e['rc'] == 0 and e['f'] == 0 and i not in used and e['ra5'] == site)
    T.append(('gate: stop ret 1', gate_t('fhnw-binpack4-4', lambda evs: [x.__setitem__('ret', 1) for x in evs if x['e'] == 'ck'])))
    T.append(('gate: uncalled caller unknown', gate_t('tr12-30', lambda evs: uncalled(evs, 0x7229A3).__setitem__('ra5', 0x4F0000))))
    T.append(('gate: probe true dropped fx', gate_t('pk1', lambda evs: evs.__setitem__(slice(None), [x for x in evs if x['e'] != 'fx' or x['fk'] != next(e['fk'] for e in evs if e['e'] == 'fx')]))))
    T.append(('gate: probe dropped', gate_t('pk1', lambda evs: evs.remove(next(x for x in evs if x['e'] == 'gt')))))
    T.append(('gate: stop then probe', gate_t('markshare_4_0', lambda evs: (lambda e: evs.insert(evs.index(e) + 1, {'e': 'gt', 'k': evs.index(e), 'l38': 3, 'l58': e['l58'], 'l68': e['l68'], 'v70': 1}))(uncalled(evs, SITE_5FB8EC)))))
    bad = []
    for name, t in T:
        try:
            t()
            bad.append(name)
            print(f'  T {name}: NOT REJECTED')
        except (Fail, KeyError, IndexError, StopIteration) as e:
            print(f'  T {name}: rejected ({type(e).__name__}: {str(e)[:100]})')
    return bad


def variants(caps, tot):
    bad = []
    for v, key in VARIANTS.items():
        expect = tot[key] > 0
        caught = None
        for case in CASES:
            try:
                check_capture(caps[(case, 1)], case, frozenset([v]))
            except Fail as e:
                caught = f'{case}: {str(e)[:90]}'
                break
        ok = (caught is not None) == expect
        print(f'  V {v} ({key}={tot[key]}): {"rejected " + caught if caught else "survives"}' + ('' if ok else '  <-- UNEXPECTED'))
        if not ok:
            bad.append(v)
    return bad


def write_manifest():
    files, models = {}, {}
    for c in CASES:
        for r in RUNS:
            fn = f'{c}_{r}.json.gz'
            files[fn] = sha(os.path.join(DATA, fn))
            models[c] = load(os.path.join(DATA, fn))['model_sha256']
    m = {'task': 'M2-06f-5', 'schema': 1, 'tool': 'tools/mip_ray_fix.js', 'tool_sha256': sha(TOOL), 'binary_sha256': BINARY,
         'params': PARAMS, 'models': models, 'model_paths': '../re/bench/data/miplib/<case>.mps.gz', 'files': files}
    p = os.path.join(DATA, 'manifest.json')
    if os.path.exists(p):
        raise SystemExit('manifest exists')
    json.dump(m, open(p, 'w'), indent=1)
    print('wrote', p)


def main(argv):
    if argv[:1] == ['--write-manifest']:
        write_manifest()
        return 0
    if argv[:1] == ['--sweep']:
        bad = 0
        for p in argv[1:]:
            d = load(p)
            try:
                c = check_capture(d, os.path.basename(p))
                print(os.path.basename(p), d['output'], end=' ')
                show('', c)
            except Fail as e:
                bad += 1
                print(os.path.basename(p), 'FAIL', e)
        return 1 if bad else 0
    try:
        caps, old, manifest = load_all()
        controls(caps, manifest, old)
        print('controls: PASS (manifest, 3 runs equal, output line equal to M2-06f-4 run 1)')
        tot = acceptance(caps, old)
    except Fail as e:
        print('FAIL', e)
        return 1
    print('ACCEPTANCE PASS')
    if '--self-test' in argv:
        bt = tampers(caps, old, manifest)
        bv = variants(caps, tot)
        if bt or bv:
            print('SELF-TEST FAIL', bt, bv)
            return 1
        print('ACCEPTANCE PASS (+ self-test)')
    return 0


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
