"""M2-03d-8: Python reconstruction of the DynamicRows dive row selection of 5fb8ec (C:1581-1856), docs/m2/mip_dive_rows.md section 2.

Inputs come from a tools/mip_dive_rows.js capture (d:model / d:sel events). Floating point follows the asm operation order
(no FMA); comparisons follow the comisd branch (jb / jae / jbe) including the unordered case.
`variant` switches one clause off or changes it; it is used only by the discrimination check (V) of tools/mip_dive_rows_check.py.
"""
import math, struct

INF100 = 1e100
CUTOFF_NONE = '54b249ad2594c37d'   # 5e7204 return without an incumbent object (= 1e100)


def f64(h):
    return struct.unpack('<d', struct.pack('<Q', int(h, 16)))[0]


def bits(x):
    return format(struct.unpack('<Q', struct.pack('<d', x))[0], '016x')


def split16(s):
    return [s[k:k + 16] for k in range(0, len(s), 16)]


def jb(x, y):            # comisd y, x ; jb  -> x < y or unordered
    return not (x >= y)


def a7cec0(s, ind, val, lb, ub):
    """sum over the row of bound * (s * a): ub where s*a <= 0, else lb; -1e100 at an infinite bound (+-1e30)."""
    acc = 0.0
    for j, v in zip(ind, val):
        p = v * s
        if p <= 0.0:
            b = ub[j]
            if 1e30 <= b:
                return -1e100
        else:
            b = lb[j]
            if b <= -1e30:
                return -1e100
        acc = acc + b * p
    return acc


def f66661c(rhs, gap, ind, val, vtype, lb, ub, rc, variant=None):
    """asm 0x554a30: 1 when raising the row activity above rhs (rounded up for integral rows) costs more than gap."""
    loc = 0.0
    integral = True
    E, C, W = [None] * 6, [None] * 6, [None] * 6     # 1-based slots 1..5 (|rc|, |a|, ub-lb), sorted by |rc/a|
    k = 0
    for j, a in zip(ind, val):
        l, u = lb[j], ub[j]
        d = u - l
        if 1e-10 >= d:
            loc = loc + (a * 0.5) * (l + u)
            continue
        if integral and vtype[j] != 'C':
            integral = (a - math.floor(1e-10 + a)) <= 1e-10
        else:
            integral = False
        if 1e-6 >= abs(a):
            return 0
        r = rc[j]
        if a > 0.0 and 1e-10 >= r:
            loc = loc + u * a
            continue
        if a > 0.0 and -1e30 >= l:
            return 0
        if 0.0 > a and not jb(r, -1e-10):
            loc = loc + l * a
            continue
        if 0.0 > a and u >= 1e30:
            return 0
        q = abs(r / a)
        loc = loc + (l * a if a > 0.0 else u * a)
        p = k
        while p > 0:
            if q >= E[p] / C[p]:
                break
            if p < 5:
                E[p + 1], C[p + 1], W[p + 1] = E[p], C[p], W[p]
            p -= 1
        if p < 5:
            E[p + 1], C[p + 1], W[p + 1] = abs(r), abs(a), d
            if k < 5:
                k += 1
    if integral and variant != 'f666_noround':
        rhs = math.floor(rhs + 9.999999999999999e-05) + 1.0
    lim = rhs - 1e-6
    if not (lim > loc):
        return 0
    cost = 0.0
    for c in range(k):
        step = (rhs - loc) / C[c + 1]
        if c < k - 1 and variant != 'f666_nocap':
            step = step if step < W[c + 1] else W[c + 1]      # minsd
        loc = loc + C[c + 1] * step
        cost = cost + E[c + 1] * step
        if loc >= lim:
            break
    return 1 if gap < cost else 0


class Model:
    def __init__(self, ev):
        self.nrows, self.ncols = ev['nrows'], ev['ncols']
        val = [f64(h) for h in split16(ev['val'])]
        need = sum(ev['row_len'])
        if len(ev['ind']) != need or len(val) != need:
            raise ValueError(f'model {ev["model"]}: row data length {len(ev["ind"])}/{len(val)} != {need}')
        self.rows, o = [], 0
        for n in ev['row_len']:
            self.rows.append((ev['ind'][o:o + n], val[o:o + n]))
            o += n
        self.rhs = [f64(h) for h in split16(ev['rhs'])]
        self.sense, self.vtype = ev['sense'], ev['vtype']
        if len(self.rhs) != self.nrows or len(self.sense) != self.nrows or len(self.vtype) != self.ncols:
            raise ValueError(f'model {ev["model"]}: rhs/sense/vtype length')


def selections(events):
    """Yield (d:sel event, Model, lb, ub) with the node bounds rebuilt from the per-model diffs."""
    models, bounds = {}, {}
    for e in events:
        if e['event'] == 'd:model':
            if e['model'] in models:
                raise ValueError(f'model id {e["model"]} sent twice')
            models[e['model']] = Model(e)
        elif e['event'] == 'd:sel':
            m = models[e['model']]
            lb, ub = bounds.get(e['model'], ([None] * m.ncols, [None] * m.ncols))
            lb, ub = list(lb), list(ub)
            for j, h in e['lb_diff']:
                lb[j] = h
            for j, h in e['ub_diff']:
                ub[j] = h
            if None in lb or None in ub:
                raise ValueError(f'sel call {e["call"]}: bounds not fully known')
            bounds[e['model']] = (lb, ub)
            yield e, m, [f64(h) for h in lb], [f64(h) for h in ub]


def cutoff(e):
    raw = f64(e['cutoff_raw'] or CUTOFF_NONE)
    return raw - f64(e['i10']) if INF100 > raw else raw


def select(e, m, lb, ub, variant=None, stats=None):
    """Return (selected rows in order, [(row, 66661c result)]) for one d:sel event."""
    st = stats if stats is not None else {}
    inc = lambda k: st.__setitem__(k, st.get(k, 0) + 1)
    flags = {i: v for i, v in e['flags']}
    b7 = e['b7889a5']
    b7rows = {i for i, v in b7['nonneg'] if i < b7['count']} if b7 else set()
    t = 0.1 * f64(e['feastol'])
    if variant == 'tol0':
        t = 0.0
    co = cutoff(e)
    rc = [f64(h) for h in split16(e['rc'])] if e['rc'] is not None else None
    nrows, sel, calls = e['nrows'], [], []
    for i in range(nrows):
        if flags.get(i, 0) != 0:
            inc('model_flag')
            if variant != 'no_flag_model':
                continue
        if i in b7rows:
            inc('b7889a5')
            if variant != 'no_b7':
                continue
        ind, val = m.rows[i]
        rhs, eq = m.rhs[i], m.sense[i] == '='
        maxact = -a7cec0(-1.0, ind, val, lb, ub)
        x0 = (rhs - t) if variant == 'tol_sign' else (t + rhs)
        if variant == 'no_maxact':
            x0 = INF100
        if jb(x0, maxact):
            path = True
        else:
            if (t + rhs) >= maxact > rhs:
                inc('maxact_in_tol')
            if eq:
                inc('eq_redundant_side')
                x0 = INF100 if variant == 'no_eq_min' else a7cec0(1.0, ind, val, lb, ub)
            else:
                x0 = INF100
            path = jb(x0, rhs - t)
        if path:
            inc('cutoff_path')
            if e['dual'] == 0 and variant != 'no_dual':
                continue
            if not (INF100 > co) and variant != 'cutoff_inf':
                continue
            if eq and variant != 'eq_cut':
                inc('cutoff_path_eq')
                continue
            if rc is None:
                raise ValueError(f'sel call {e["call"]}: 66661c needs rc')
            r = f66661c(rhs, co, ind, val, m.vtype, lb, ub, rc, variant)
            calls.append([i, r])
            inc('66661c')
            if r:
                inc('66661c_1')
            if variant == 'f666_0':
                r = 0
            if variant == 'f666_1':
                r = 1
            if r == 0:
                continue
        sel.append(i)
    cuts = []
    slack = [f64(h) for h in split16(e['cut_slack'])]
    c40 = set(e['cut_40'])
    thr = 0.0 if variant == 'slack0' else 1e-5
    for j in range(e['ncuts']):
        if flags.get(nrows + j, 0) != 0:
            inc('cut_flag')
            if variant != 'no_flag_cut':
                continue
        if e['cut_type'][j] == 0x14:
            inc('cut_type_14')
            if variant != 'no_t14':
                continue
        if j in c40:
            inc('cut_40')
            if variant != 'no_40':
                continue
        if 0.0 < slack[j] <= 1e-5:
            inc('slack_le_1e-5')
        if not (slack[j] > thr):
            continue
        cuts.append(nrows + j)
    out = cuts + sel if variant == 'cuts_first' else sel + cuts
    return out, calls
