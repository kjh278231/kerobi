#!/usr/bin/env python3
"""M1-02a-2: storage order of the original row-wise U (LU+0xb0/0xb8/0xd0/0xc8), contract docs/m1/m1_02a2_urow.md section 3.

(default)    M (pinned inputs), G (05b257 update rule on the 34 captured updates), V (update variants rejected),
             L (Rust factor of the pre-factor basis = original renumbering, U and diagonal at pivots 15/26),
             F (initial order rule = captured rows at pivots 15/26), T (targets)
--self-test  also every in-memory tamper must be rejected
--write-dump regenerate the Rust factor dump (needs a build with rust_factor_dump.patch; see section 5)
Exit code 0 only when every requested check passes.
"""
import copy, gzip, hashlib, json, os, sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
CAPS = [f'bench/oracle/m1_02a1_tau/kb2_{r}.json.gz' for r in (1, 2, 3)]
ORACLE = 'bench/oracle/iter0-30/kb2.jsonl.gz'
DUMP = 'bench/oracle/m1_02a2_urow/rust_factor_kb2.jsonl.gz'
PATCH = 'bench/oracle/m1_02a2_urow/rust_factor_dump.patch'
PIN = {CAPS[0]: '64cc9dea89f809b601f7cb2e11636cc632f382318ceeaec0273a599687bdf393',
       ORACLE: '007d5a557fa84fcb73cdd7021f61e9fc4b5e97625615d9803cab6e7c96bc3bda',
       DUMP: '7294c10c01b7f5da66bc159975057d3c7ed1c799bc4cca18a94fb3239c38d839',
       PATCH: 'fa5096866a9088e5983d8fa6ee7abe398e8091a7b6a3936358ca1edf775a2986'}
PIN[CAPS[1]] = PIN[CAPS[2]] = PIN[CAPS[0]]
REFACTORS = (15, 26)       # refactors with U rows (pivots 0 and 6 have none)


class Fail(Exception):
    pass


def need(cond, msg):
    if not cond:
        raise Fail(msg)


def sha(p):
    with open(os.path.join(ROOT, p), 'rb') as f:
        return hashlib.sha256(f.read()).hexdigest()


def load():
    caps = [json.load(gzip.open(os.path.join(ROOT, p)))['events'] for p in CAPS]
    snaps = {int(d['iter']): d for d in map(json.loads, gzip.open(os.path.join(ROOT, ORACLE), 'rt'))}
    dump = [json.loads(ln) for ln in gzip.open(os.path.join(ROOT, DUMP), 'rt')]
    return {'caps': caps, 'snaps': snaps, 'dump': dump, 'sha': {p: sha(p) for p in PIN}}


def rows(e):
    """Captured row-wise U by position: [(column step, value bits)] in storage order."""
    return [list(zip(r['idx'], r['val'])) for r in e['ft']['urow']]


def update(before, r, eta, after, v=None):
    """05b257 on the rows of pivot k: column r out (C:556-567: last entry into the hole), row r emptied on a row eta
    (C:970-1037, LU+0x180 + 1), new column r entries appended (C:1302-1304). Membership and values of the new column
    come from the next capture (contract section 3 G)."""
    s = [list(x) for x in before]
    for row in s:
        h = next((i for i, (c, _) in enumerate(row) if c == r), None)
        if h is None:
            continue
        if v == 'stable':
            row.pop(h)
        else:
            row[h] = row[-1]
            row.pop()
    if eta and v != 'keep_r':
        s[r] = []
    for i, row in enumerate(after):
        new = [x for x in row if x[0] == r]
        if new and not any(c == r for c, _ in s[i]):
            s[i] = new + s[i] if v == 'prepend' else s[i] + new
    return s


def check_updates(ev, v=None, stats=None):
    n = 0
    for k in range(len(ev) - 1):
        a, b = ev[k], ev[k + 1]
        if b['lu_upd'] == 0:
            continue
        need(b['lu_upd'] == a['lu_upd'] + 1, f'G pivot {k}: LU+0x18c {a["lu_upd"]} -> {b["lu_upd"]}')
        de = b['ft']['lu']['x180'] - a['ft']['lu']['x180']
        need(de in (0, 1), f'G pivot {k}: LU+0x180 grows by {de}')
        before, after = rows(a), rows(b)
        got = update(before, a['r'], de == 1, after, v)
        bad = [i for i in range(len(got)) if got[i] != after[i]]
        need(not bad, f'G pivot {k}->{k + 1} (r {a["r"]}, eta {de}): rows {bad[:4]} differ, first rule '
                      f'{[c for c, _ in got[bad[0]]] if bad else ""} vs capture {[c for c, _ in after[bad[0]]] if bad else ""}')
        if stats is not None:
            stats['updates'] += 1
            stats['eta'] += de
            stats['not_stable'] += update(before, a['r'], de == 1, after, 'stable') != after
        n += 1
    return n


def prefactor(snaps, ev, k):
    """vstat before the refactor at pivot k: snapshot k-1 with pivot k-1's swap (position r gets the entering q)."""
    v = list(snaps[k - 1]['vstat'])
    r, q = ev[k - 1]['r'], snaps[k]['q']
    need(v.count(r) == 1 and v[q] < 0, f'L pivot {k}: swap r {r} q {q} does not fit snapshot {k - 1}')
    v[v.index(r)] = -1
    v[q] = r
    return v


def initial(d, v=None):
    """036ade/1da0b2 initial order by step. Singleton steps (step < ns): entries by ascending pre-factor position of
    the column (1da0b2 C:530-560 walks the still unpivoted columns in position order and appends to the pivoted row's
    U row; kb2 has t = ns, so singleton rows with entries in singleton columns are not exercised). Kernel steps: the
    active row list head first (036ade C:600-690 prepends while walking positions upward, fill is prepended at
    C:2258-2265, C:2366-2393 copies the list), i.e. Rust kernel_u."""
    steps, ppos = d['steps'], d['ppos']
    rstep = {r: s for s, (r, _, _) in enumerate(steps)}
    u = [[] for _ in steps]
    for c, col in enumerate(d['ucol']):
        for i, val in col:
            u[rstep[i]].append((c, val))
    out = []
    for s, row in enumerate(u):
        if v == 'rust_now':                       # current Rust finish(): ascending column step
            out.append(sorted(row))
        elif s < d['ns'] or v == 'all_singleton':
            out.append(sorted(row, key=lambda x: steps[x[0]][1]))
        elif v == 'kernel_desc':
            out.append(sorted(row, key=lambda x: -steps[x[0]][1]))
        else:
            out.append([(ppos[p], val) for p, val in d['ku'][s]])
    return out, u


def check_refactors(data, v=None, stats=None):
    ev, snaps = data['caps'][0], data['snaps']
    need(len(data['dump']) == len(REFACTORS), f'L: {len(data["dump"])} dump lines')
    for k, d in zip(REFACTORS, data['dump']):
        e = ev[k]
        need(e['lu_upd'] == 0 and snaps[k]['lu']['k'] in (0, '0'), f'L pivot {k}: not a refactor')
        pre, cur, head = prefactor(snaps, ev, k), snaps[k]['vstat'], d['head']
        need(all(pre[head[p]] == p for p in range(len(head))), f'L pivot {k}: dump head is not the pre-factor basis')
        need(all(cur[head[p]] == d['ppos'][p] for p in range(len(head))), f'L pivot {k}: Rust step order != snapshot {k} positions')
        ft = e['ft']
        need(d['ns'] == ft['t'], f'L pivot {k}: Rust singleton steps {d["ns"]} != prefix t {ft["t"]}')
        got, u = initial(d, v)
        cap = rows(e)
        need(all(sorted(u[s]) == sorted(cap[s]) for s in range(len(cap))), f'L pivot {k}: U sets/values != capture')
        need([x[2] for x in d['steps']] == ft['diag'], f'L pivot {k}: diagonal != capture')
        bad = [s for s in range(len(cap)) if got[s] != cap[s]]
        need(not bad, f'F pivot {k}: steps {bad[:4]} differ, rule {[c for c, _ in got[bad[0]]] if bad else ""} vs capture '
                      f'{[c for c, _ in cap[bad[0]]] if bad else ""}')
        if stats is not None:
            for s, row in enumerate(cap):
                if len(row) < 2:
                    continue
                kind = 'kernel' if s >= d['ns'] else 'singleton'
                stats[kind] += 1
                stats['not_ascending'] += [c for c, _ in row] != sorted(c for c, _ in row)
                if kind == 'kernel':
                    stats['kernel_fill_order'] += got[s] != sorted(u[s], key=lambda x: -d['steps'][x[0]][1])


def run(data):
    lines = []
    for p, h in PIN.items():
        need(data['sha'][p] == h, f'M: {p} sha256 {data["sha"][p]} != {h}')
    need(data['caps'][0] == data['caps'][1] == data['caps'][2], 'M: capture runs differ')
    lines.append('M: 3 captures (pinned gz sha256, equal events), stored oracle, Rust factor dump and its patch pinned')
    st = {'updates': 0, 'eta': 0, 'not_stable': 0}
    check_updates(data['caps'][0], None, st)
    need(st['updates'] == 34, f'G: {st["updates"]} updates, expected 34')
    lines.append(f'G: 05b257 rule = next captured rows on all {st["updates"]} updates (row etas {st["eta"]}, '
                 f'stable removal would differ on {st["not_stable"]})')
    for v in ('stable', 'prepend', 'keep_r'):
        try:
            check_updates(data['caps'][0], v)
        except Fail as x:
            lines.append(f'V: {v}: rejected ({str(x)[:90]})')
            continue
        raise Fail(f'V: update variant {v} is not rejected')
    fs = {'kernel': 0, 'singleton': 0, 'not_ascending': 0, 'kernel_fill_order': 0}
    check_refactors(data, None, fs)
    lines.append(f'L: pivots {REFACTORS}: Rust factor of the pre-factor basis renumbers to the snapshot positions; U sets, '
                 f'value bits and diagonal = capture; singleton steps = prefix t')
    lines.append(f'F: initial order rule = captured rows at pivots {REFACTORS} (multi-entry rows: singleton {fs["singleton"]}, '
                 f'kernel {fs["kernel"]}; not ascending in step {fs["not_ascending"]})')
    for v, must in (('rust_now', True), ('all_singleton', True), ('kernel_desc', False)):
        try:
            check_refactors(data, v)
            res = 'NOT DISCRIMINATED'
        except Fail as x:
            res = 'rejected (' + str(x)[:90] + ')'
        need(not must or res != 'NOT DISCRIMINATED', f'F: initial variant {v} is not rejected')
        lines.append(f'F variant {v}: {res}')
    need(st['not_stable'] >= 1 and st['eta'] >= 1, f'T: update targets {st}')
    need(fs['not_ascending'] >= 1 and fs['kernel'] >= 1 and fs['singleton'] >= 1, f'T: refactor targets {fs}')
    lines.append(f'T: targets met; kernel rows whose list order differs from descending position (fill): {fs["kernel_fill_order"]}')
    return lines


def tampers(data):
    def swap_row(k):
        def f(d):
            for ev in d['caps']:
                r = next(x for x in ev[k]['ft']['urow'] if len(x['idx']) >= 2)
                r['idx'][:2] = r['idx'][1::-1]
                r['val'][:2] = r['val'][1::-1]
        return f

    def eta_flag(d):
        for ev in d['caps']:
            k = next(k for k in range(1, len(ev)) if ev[k]['lu_upd'] and ev[k]['ft']['lu']['x180'] > ev[k - 1]['ft']['lu']['x180'])
            for e in ev[k:]:
                if e['lu_upd'] == 0:
                    break
                e['ft']['lu']['x180'] -= 1

    def dump_val(d):
        c = next(c for c in d['dump'][0]['ucol'] if c)
        c[0][1] = format(int(c[0][1], 16) ^ 1, '016x')

    def dump_ppos(d):
        p = d['dump'][1]['ppos']
        p[0], p[1] = p[1], p[0]

    def carried_val(d):
        # a value bit of an entry that pivot 13 already had (not the new column) at pivot 14
        for ev in d['caps']:
            r = ev[13]['r']
            row = next(x for x in ev[14]['ft']['urow'] if any(c != r for c in x['idx']))
            i = next(i for i, c in enumerate(row['idx']) if c != r)
            row['val'][i] = format(int(row['val'][i], 16) ^ 1, '016x')

    def ku_order(d):
        # only F reads the kernel list order: reverse one multi-entry kernel row of pivot 15
        ku = next(x for x in d['dump'][0]['ku'] if len(x) >= 2)
        ku.reverse()

    def cap_sha(d):
        d['sha'][CAPS[1]] = '0' * 64

    yield 'capture sha256', cap_sha
    yield 'update: two entries of a pivot 12 row exchanged', swap_row(12)
    yield 'update: row eta flag dropped', eta_flag
    yield 'update: value bit of a carried entry at pivot 14', carried_val
    yield 'dump: one U value bit', dump_val
    yield 'dump: two step positions exchanged', dump_ppos
    yield 'refactor: two entries of a pivot 26 row exchanged (G 26->27 sees it first)', swap_row(26)
    yield 'dump: kernel list order of a pivot 15 row reversed (F only)', ku_order


def write_dump():
    """Regenerate the dump with a build that has rust_factor_dump.patch applied (section 5)."""
    import rsolver
    data = load()
    out = os.path.join(ROOT, DUMP[:-3])
    if os.path.exists(out):
        os.remove(out)
    os.environ['RSOLVER_UROWDUMP'] = out
    m = rsolver.read(os.path.join(ROOT, '../re/bench/data/netlib/kb2.mps.gz'))
    for k in REFACTORS:
        m._core.row_costs(prefactor(data['snaps'], data['caps'][0], k))
    need(os.path.exists(out), 'write-dump: no dump written (is the patch applied?)')
    print('wrote', out, '(gzip -n it to', DUMP + ')')


def main():
    args = sys.argv[1:]
    if '--write-dump' in args:
        write_dump()
        return 0
    try:
        data = load()
        for ln in run(data):
            print(ln)
        if '--self-test' in args:
            for name, f in tampers(data):
                d = copy.deepcopy(data)
                f(d)
                try:
                    run(d)
                except Fail as x:
                    print(f'rejected: {name}: {str(x)[:110]}')
                    continue
                raise Fail(f'tamper not rejected: {name}')
    except Fail as x:
        print('FAIL', x)
        return 1
    print('PASS')
    return 0


if __name__ == '__main__':
    sys.exit(main())
