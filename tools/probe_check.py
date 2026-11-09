"""Bit-for-bit check of the probing port (crates/mip probe.rs, probe_engine.rs, conflict.rs) against Gurobi dumps of
tools/probe_trace.js (docs/m2_red_probe.md §4). Every pass of every 4d3a1d call is replayed from its start state:
conflict-graph merge (5b9aad), probe matrix and context (4d3a1d §2.3-2.7, 396a2d, 7886b), engine (4e48f7), and the
implication export (5b88ce) plus the final merge, each compared with the dump including the work counter.
usage: PYTHONPATH=python python tools/probe_check.py <dump_dir> [name ...]   (dumps: <dump_dir>/<name>.json)
make dumps: tools/gurun.sh node tools/probe_trace.js <name> <dump_dir>/<name>.json
"""
import json, struct, sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from parity import instances  # noqa: E402
import rsolver  # noqa: E402


def f(h):
    return struct.unpack('>d', bytes.fromhex(h.rjust(16, '0')))[0]


def fl(a):
    return [] if a is None else [f(x) for x in a]


def bits(x):
    return struct.pack('<d', x)


def same(a, b):
    """lists or scalars, doubles by bits (so -0.0 != 0.0)"""
    if isinstance(a, float) or isinstance(b, float):
        return bits(float(a)) == bits(float(b))
    if isinstance(a, (list, tuple)) and isinstance(b, (list, tuple)):
        return len(a) == len(b) and all(same(x, y) for x, y in zip(a, b))
    return a == b


def chars(s):
    return [] if s is None else [ord(c) for c in s]


def graph(d, g, pre):
    d[pre + 'n0'], d[pre + 'rc0'] = g['n0'], g['rc0']
    d[pre + 'ptr'] = g['ptr'] or []
    d[pre + 'adj'] = g['adj'] or []
    d[pre + 'pa'], d[pre + 'pb'] = g['pa'] or [], g['pb'] or []
    d[pre + 'flag'] = g['flag'] or []


def passes(ev):
    """group events by (call, pass)"""
    out, cur, call = [], None, None
    for e in ev:
        if e['ev'] == 'call':
            call = e
        elif e['ev'] == 'pass':
            cur = {'call': call, 'pass': e}
            out.append(cur)
        elif e['ev'] != 'ret' and cur is not None:
            cur[e['ev']] = e
    return out


def check_pass(core, P):
    s, call = P['pass']['s'], P['call']
    d = {'params': [], 'pass': P['pass']['pass'], 'args': call['args'],
         't_acc': f(call['timer']['acc']), 't_running': call['timer']['running'], 't_limit': f(call['timer']['limit'])}
    for k in ('lb', 'ub', 'obj', 'rhs', 'act_max', 'act_min', 'mult_lo', 'mult_hi', 'rval'):
        d[k] = fl(s[k])
    for k in ('row_cnt', 'col_cnt', 'inf_max', 'inf_min', 'rcol', 'rbeg', 'nnz', 'prelimit', 'eq_rows', 'cols_removed', 'rows_removed'):
        d[k] = s[k]
    d['vtype'], d['sense'] = chars(s['vtype']), chars(s['sense'])
    for k in ('objcon', 'cutoff', 'mu', 'work_base', 'budget'):
        d[k] = f(s[k])
    d['work'] = f(s['work'])
    graph(d, P['pass']['g'] or {'n0': 0, 'rc0': 0, 'ptr': None, 'adj': None, 'pa': None, 'pb': None, 'flag': None}, 'g_')
    if 'imp' in P:
        im = P['imp']
        d['imp_lb'], d['imp_ub'], d['imp_col_cnt'] = fl(im['lb']), fl(im['ub']), im['col_cnt']
        d['imp_nnz'], d['imp_cols_removed'], d['imp_rows_removed'] = im['nnz'], im['cols_removed'], im['rows_removed']
        d['imp_work'] = f(im['work'])
        graph(d, im['g'], 'imp_g_')
    r = core.mip_probe_pass(d)
    bad = []

    def cmp(what, a, b):
        if not same(a, b):
            bad.append(what)
    mg = P['merged']
    cmp('merge.work', r['merged_work'], f(mg['work']))
    g = mg['g']
    cmp('merge.graph', list(r['merged_g']), [g['n0'], g['rc0'], g['ptr'] or [], g['adj'] or []])
    if 'ctx_in' not in P:
        if 'build_stop' not in r:
            bad.append('build: port probes, Gurobi does not')
        return bad, 'no engine (%s)' % r.get('build_stop')
    if 'build_stop' in r:
        return bad + ['build stopped: ' + r['build_stop']], ''
    ci, co = P['ctx_in'], P['ctx_out']
    c, x = r['ctx_in'], ci['c']
    cmp('build.cand', r['cand_in'], ci['cand'])
    cmp('build.work', r['in_work'], f(ci['work']))
    for k in ('rbeg', 'rcol', 'cbeg', 'crow'):
        cmp('build.' + k, c[k], x[k])
    cmp('build.rnb', c['rnb'], x['rnb'])
    for k in ('rval', 'cval', 'rhs', 'range', 'lb', 'ub', 'amin', 'amax'):
        cmp('build.' + k, c[k], fl(x[k]))
    cmp('build.budget', c['budget'], f(x['budget']))
    cmp('build.sense', list(c['sense']), chars(x['sense']))
    c, x = r['ctx_out'], co['c']
    cmp('engine.end', r['engine_end'], 'Done')
    cmp('engine.cand', r['cand_out'], co['cand'])
    cmp('engine.work', r['out_work'], f(co['work']))
    for k in ('lb', 'ub', 'amin', 'amax'):
        cmp('engine.' + k, c[k], fl(x[k]))
    # aggmul doubles as 396a2d's scratch and aggoff is uninitialised: only aggregated columns carry values
    agg = [j for j, t in enumerate(x['aggto']) if t >= 0]
    for k in ('aggmul', 'aggoff'):
        cmp('engine.' + k, [c[k][j] for j in agg], [f(x[k][j]) for j in agg])
    for k in ('aggto', 'cnt0', 'cnt1'):
        cmp('engine.' + k, c[k], x[k])
    cmp('engine.fac', c['fac'], f(x['fac']))
    for k in ('stop', 'nchg', 'nbin', 'neq', 'nagg', 'imp'):
        cmp('engine.' + k, c[k], x[k])
    cmp('engine.sense', list(c['sense']), chars(x['sense']))
    if x.get('gbeg') is not None:
        cmp('engine.groups', c['gbeg'], x['gbeg'])
        cmp('engine.imps', c['imps'], x['imps'] or [])
    if x['rec']:
        cmp('engine.rhead', c['rhead'], x['rhead'])
        cmp('engine.rnext', c['rnext'], x['rnext'] or [])
        cmp('engine.rlit', c['rlit'], x['rlit'] or [])
        cmp('engine.rslack', c['rslack'], fl(x['rslack']))
    if 'imp' in P:
        if r.get('export') is None:
            bad.append('export: port found the graph full')
        elif 'end' in P:
            e = P['end']
            cmp('end.out', r['end_out'], e['out'])
            cmp('end.work', r['end_work'], f(e['work']))
            g = e['g']
            cmp('end.graph', list(r['end_g']), [g['n0'], g['rc0'], g['ptr'] or [], g['adj'] or []])
    elif r.get('export') is not None and 'end' in P:
        bad.append('export: Gurobi skipped it')
    return bad, ''


def main():
    dump = Path(sys.argv[1])
    names = sys.argv[2:] or sorted(p.stem for p in dump.glob('*.json'))
    insts = instances()
    ok = tot = 0
    for name in names:
        core = rsolver.read(str(insts[name]))._core
        for P in passes(json.load(open(dump / f'{name}.json'))):
            tot += 1
            bad, note = check_pass(core, P)
            tag = f"{name:22s} call {P['call']['k'] + 1:4d} pass {P['pass']['pass']}"
            if bad:
                print(tag, 'DIFF', ' '.join(bad))
            else:
                ok += 1
                print(tag, 'ok', note)
    print(f'{ok}/{tot} passes match')


if __name__ == '__main__':
    main()
