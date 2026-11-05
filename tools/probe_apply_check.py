"""Check the whole probing call (`State::probe_run`, write-back included) against Gurobi: start from the state
tools/mip_state_dump.js took at a `4d3a1d` entry, with the conflict graph and timer tools/probe_trace.js recorded
for the same call, and compare the state at its return (bounds, rows, counts, activities, lists, counters, work,
new postsolve records).
usage: PYTHONPATH=python python tools/probe_apply_check.py <state_dir> <trace_dir> [name.call ...]
  state_dir: <name>.<call>.json from `tools/gurun.sh node tools/mip_state_dump.js <name> 4d3a1d <call> ...`
  trace_dir: <name>.json from `tools/gurun.sh node tools/probe_trace.js <name> ...`
"""
import json, struct, sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from parity import instances  # noqa: E402
import rsolver  # noqa: E402


def f(h):
    return struct.unpack('>d', bytes.fromhex(h.rjust(16, '0')))[0]


def queue(t):
    lst = [x or [] for x in t['list']]
    return {'flag': t['flag'], 'a': t['a'], 'b': t['b'], 'list': lst, 'cache': t['cache'], 'cache2': t['cache2']}


def state_dict(s):
    d = {k: s[k] for k in ('n', 'm', 'lb', 'ub', 'obj', 'rhs', 'row_cnt', 'col_cnt', 'inf_max', 'inf_min', 'act_max',
                           'act_min', 'mult_lo', 'mult_hi', 'objcon', 'work', 'work_mult', 'prelimit')}
    d['col_flag'] = s['col_flag'] or [0] * s['n']
    d['sense'], d['vtype'] = [ord(c) for c in s['sense']], [ord(c) for c in s['vtype']]
    d['nnz'] = int(s['nnz'])
    d['counters'] = s['counters'][:13]
    d['nfree'], d['npend'] = int(s['free'][0]), int(s['free'][1])
    d['rows'] = [[tuple(x) for x in l] for l in s['rows']]
    d['cols'] = [[tuple(x) for x in l] for l in s['cols']]
    t = s['tracker']
    d['level'], d['active'] = t['level'], [bool(x) for x in t['active']]
    d['tr_r'], d['tr_c'] = queue(t['r']), queue(t['c'])
    return d


def trace_call(trace, call):
    """the `call`-th 4d3a1d call of a probe_trace dump: its timer, arguments and the conflict graph at its start"""
    k = 0
    for i, e in enumerate(trace):
        if e['ev'] == 'call':
            k += 1
            if k == call:
                g = next(x for x in trace[i:] if x['ev'] == 'pass')['g']
                return e, g
    raise KeyError(call)


def check(core, enter, leave, tcall, g):
    d = state_dict(enter)
    d['params'] = []
    d['args'] = tcall['args']
    d['t_acc'], d['t_running'], d['t_limit'] = f(tcall['timer']['acc']), tcall['timer']['running'], f(tcall['timer']['limit'])
    g = g or {}
    d['g_n0'], d['g_rc0'] = g.get('n0', 0), g.get('rc0', 0)
    for k in ('ptr', 'adj', 'pa', 'pb', 'flag'):
        d['g_' + k] = g.get(k) or []
    r = core.mip_probe_full(d)
    bad = []
    for k in ('lb', 'ub', 'obj', 'rhs', 'act_max', 'act_min', 'mult_lo', 'mult_hi', 'row_cnt', 'col_cnt', 'inf_max',
              'inf_min', 'objcon', 'work'):
        if r[k] != leave[k]:
            bad.append(k)
    if r['counters'] != leave['counters'][:13]:
        bad.append('counters %s/%s' % (r['counters'][:5], leave['counters'][:5]))
    if list(r['sense']) != [ord(c) for c in leave['sense']] or list(r['vtype']) != [ord(c) for c in leave['vtype']]:
        bad.append('sense/vtype')
    if r['nnz'] != int(leave['nnz']) or list(r['free']) != [int(x) for x in leave['free']]:
        bad.append('nnz/free %s %s' % ((r['nnz'], r['free']), (leave['nnz'], leave['free'])))
    for k in ('rows', 'cols'):
        if [[list(x) for x in l] for l in r[k]] != leave[k]:
            bad.append(k)
    for k in ('r', 'c'):
        flag, a, b, lst = r['tr_' + k]
        t = leave['tracker'][k]
        if list(flag) != t['flag'] or list(a) != t['a'] or list(b) != t['b'] or \
                [list(x)[:b[i]] for i, x in enumerate(lst)] != [(x or [])[:b[i]] for i, x in enumerate(t['list'])]:
            bad.append('tracker ' + k)
    want = leave['recs'][len(enter['recs']):]
    if [x[0] for x in r['recs']] != [x[0] for x in want]:
        bad.append('record types')
    else:
        for (t, b, pads, ptrs), w in zip(r['recs'], want):
            wb = bytes.fromhex(w[1])
            skip = set()
            for a, e in pads:
                skip.update(range(a, e))
            for o in ptrs:
                skip.update(range(o, o + 8))
            if any(b[i] != wb[i] for i in range(len(b)) if i not in skip):
                bad.append('record bytes')
                break
    if r['rc'] != leave_ret(leave):
        bad.append('status %d' % r['rc'])
    return bad, len(want)


def leave_ret(leave):
    return leave.get('_ret', 0)


def main():
    sdir, tdir = Path(sys.argv[1]), Path(sys.argv[2])
    names = sys.argv[3:] or sorted(p.stem for p in sdir.glob('*.*.json'))
    insts = instances()
    ok = 0
    for nc in names:
        name, call = nc.rsplit('.', 1)
        ev = json.load(open(sdir / f'{nc}.json'))
        enter = next(x['s'] for x in ev if 'ret' not in x)
        lv = next(x for x in ev if 'ret' in x)
        leave = dict(lv['s'], _ret=lv['ret'])
        tcall, g = trace_call(json.load(open(tdir / f'{name}.json')), int(call))
        core = rsolver.read(str(insts[name]))._core
        bad, nrec = check(core, enter, leave, tcall, g)
        print(f'{nc:28s}', 'DIFF ' + ' '.join(bad) if bad else f'ok ({nrec} records)')
        ok += not bad
    print(f'{ok}/{len(names)} calls match')


if __name__ == '__main__':
    main()
