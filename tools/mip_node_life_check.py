#!/usr/bin/env python3
"""M2-06f-2: node lifetime (5ff358 / 5f9d4e / 5fa560) and decay inputs against captures of tools/mip_node_life.js.
Contract: docs/m2/mip_node_life.md section 3.

(default)          L1-L7 on bench/oracle/m2_06f2
--write-manifest   record the SHA-256 of every capture (once, after the capture)
--self-test        also T (capture tampers) and V (model variants W1-W6, outcomes fixed in section 3)
--sweep FILE...    replay single exploratory captures, print branch counts and the variant outcomes (not acceptance)
Exit code 0 only when every requested check passes.
"""
import copy, gzip, hashlib, json, os, sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
DATA = os.path.join(ROOT, 'bench/oracle/m2_06f2')
OLD = os.path.join(ROOT, 'bench/oracle/m2_06e')  # same runs (NodeLimit 2000), L4
TOOL = os.path.join(ROOT, 'tools/mip_node_life.js')
RUNS = (1, 2, 3)
CASES = ('markshare_4_0', 'pk1', 'neos5', 'tr12-30', 'fhnw-binpack4-4', 'neos-860300')
PARAMS = {'Threads': 1, 'Seed': 0, 'LogToConsole': 0, 'NodeLimit': 2000}
RA_CUT, RA_OPT, RA_FK = 0x4f42d8, 0x4f6fa1, 0x781170  # 12ef7f after 0x4f42d3 / 0x4f6f9c, 95da98 marker
VARIANTS = {'W1': 'old5', 'W2': 'pstage3', 'W3': 'cascade0', 'W4': 'stage4', 'W5': 'free1', 'W6': 'root'}
DGATE = ('nest', 'id', 'parent', 'ik', 'st', 'j', 'lp', 'p')
DEC = ('nest', 'id', 'noI', 'noL', 'l38', 'l58', 'l54', 'l18', 'l28', 'l70', 'anc')


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


class Node:
    __slots__ = ('par', 'nchild', 'pend', 'stage', 'I', 'L', 'root', 'rel6')

    def __init__(self, par, root):
        self.par, self.nchild, self.pend, self.stage, self.I, self.L, self.root, self.rel6 = par, 0, 0, 5, True, False, root, -1


def replay(d, V=frozenset(), name=''):
    """Run the section 2.2 model over the events; raise Fail on the first difference. Returns branch counts."""
    ev = d['events']
    st = {}
    gone = set()
    c = dict.fromkeys(('nn', 'rel0', 'rel3', 'rel4', 'rel6', 'relx', 'cascade0', 'free1', 'stop_old5', 'stop_pstage3',
                       'root_fr', 'dec_noI_end', 'dec_nop_end', 'dec_cap', 'dec_selfnoI', 'dec_noL', 'src_cut', 'src_fk',
                       'src_opt', 'novec', 'rc_checked', 'rc_basic', 'v_old5', 'v_pstage3', 'v_cascade0', 'v_stage4',
                       'v_free1', 'v_root', 'chain_gt_depth', 'mv_moved'), 0)

    def node(k, i, what):
        need(k >= 0, f'{name}: event {i} {what}: node serial {k} (not created by 5ff358)')
        need(k in st, f'{name}: event {i} {what}: node {k} already freed' if k in gone else f'{name}: event {i} {what}: node {k} unknown')
        return st[k]

    def fr_model(X, s):
        """5fa560(X, s): returns the expected (fr, fr_) fields and applies the frees."""
        n = st[X]
        pre = {'n': X, 's': s, 'I': int(n.I), 'L': (int(n.L) if n.I else None), 'root': (int(n.root) if n.I else None)}
        if n.root and n.I and 1 <= s <= 5:
            c['root_fr'] += 1
            if s <= 2 or n.L:
                c['v_root'] += 1
        root_exc = n.root and 'root' not in V
        if s == 0:
            del st[X]
            gone.add(X)
            return pre, {'n': X, 'gone': 1}
        if n.I and not root_exc and s <= 5:
            n.L = False
            if s <= 2:
                n.I = False
        return pre, {'n': X, 'I': int(n.I), 'L': (int(n.L) if n.I else None)}

    def release(k, s):
        out, X, first_old = [], k, None
        while True:
            n = st[X]
            old = n.stage if n.I else 1
            if first_old is None:
                first_old = n.stage if n.I else None
            if n.I:
                n.stage = s
            P = n.par
            out.append(fr_model(X, s))
            if P < 0:
                break
            p = st.get(P)
            need(p is not None, f'{name}: release of {X}: parent {P} freed before its child')
            if s == 0:
                p.nchild -= 1
            if p.nchild == 0:
                c['cascade0'] += 1
                c['v_cascade0'] += 1
                if 'cascade0' in V:
                    break
                X, s = P, 0
                continue
            if not p.I:
                break
            if s > 4 or ('stage4' in V and s > 3):
                break
            if old < 5 and 'old5' not in V:
                c['stop_old5'] += 1
                if p.pend == 1 and p.stage >= 3:
                    c['v_old5'] += 1
                break
            p.pend -= 1
            if s == 4 and p.pend == 0 and p.stage >= 3:
                c['v_stage4'] += 1
            if p.pend != 0:
                break
            if p.stage < 3 and 'pstage3' not in V:
                c['stop_pstage3'] += 1
                c['v_pstage3'] += 1
                break
            c['free1'] += 1
            c['v_free1'] += 1
            X, s = P, (0 if 'free1' in V else 1)
        return first_old, out

    i = 0
    while i < len(ev):
        e = ev[i]
        t = e['e']
        if t == 'nn':
            need(e['rc'] == 0, f'{name}: event {i} nn: 5ff358 failed rc {e["rc"]}')
            k = e['s']
            need(k not in st and k not in gone, f'{name}: event {i} nn: serial {k} reused')
            if e['par'] >= 0:
                p = node(e['par'], i, 'nn parent')
                need(p.I, f'{name}: event {i} nn: parent {e["par"]} without I')
                p.nchild += 1
                p.pend += 1
            else:
                need(e['par'] == -1, f'{name}: event {i} nn: parent pointer not a node ({e["par"]})')
            st[k] = Node(e['par'], bool(e['root']))
            c['nn'] += 1
        elif t == 'rel':
            n = node(e['n'], i, 'rel')
            s = 0 if e['s'] == -1 else e['s']
            if s == 6:
                need(n.I, f'{name}: event {i} rel 6 on node {e["n"]} without I')
                need(e['L'] == int(n.L), f'{name}: event {i} rel 6 node {e["n"]}: L before {e["L"]}, model {int(n.L)}')
                need(ev[i + 1]['e'] == 'rel6' and ev[i + 1]['n'] == e['n'] and ev[i + 1]['rc'] == 0 and ev[i + 1]['L'] == 1,
                     f'{name}: event {i + 1}: expected rel6 of node {e["n"]} with L, got {ev[i + 1]}')
                n.L = True
                n.rel6 = i
                c['rel6'] += 1
                i += 2
                continue
            c['rel%d' % s if s in (0, 3, 4) else 'relx'] += 1
            old, want = release(e['n'], s)
            need(e['old'] == old, f'{name}: event {i} rel node {e["n"]} stage {s}: old {e["old"]}, model {old}')
            j = i + 1
            for a, b in want:
                need(j + 1 < len(ev), f'{name}: event {j}: capture ends inside a release')
                got_a = {k: ev[j].get(k) for k in a}
                need(ev[j]['e'] == 'fr' and got_a == a, f'{name}: event {j} (release at {i}): fr {ev[j]}, model {a}')
                got_b = {k: ev[j + 1].get(k) for k in b}
                need(ev[j + 1]['e'] == 'fr_' and got_b == b, f'{name}: event {j + 1} (release at {i}): fr_ {ev[j + 1]}, model {b}')
                j += 2
            need(j >= len(ev) or ev[j]['e'] != 'fr', f'{name}: event {j}: extra fr {ev[j] if j < len(ev) else None} after release at {i}')
            i = j
            continue
        elif t == 'mv':
            node(e['n'], i, 'mv')
            c['mv_moved'] += e['moved']
        elif t in ('fr', 'fr_', 'rel6'):
            raise Fail(f'{name}: event {i}: {t} outside a release')
        elif t == 'sol':
            if 'basnz' in e:
                c['rc_checked'] += 1
                c['rc_basic'] += e['bas']
                need(e['basnz'] == 0, f'{name}: event {i} sol ({e["from"]}, {e["path"]}): {e["basnz"]} basic columns with a nonzero first output')
        elif t == 'fk':
            need(e['r9'] == 1, f'{name}: event {i} fk: r9 != r8 - 4')
        elif t == 'dec':
            n = node(e['n'], i, 'dec')
            if not n.I:
                need(e.get('noI') == 1, f'{name}: event {i} dec node {e["n"]}: model has no I, capture {e}')
                c['dec_selfnoI'] += 1
            else:
                need(not e.get('noI'), f'{name}: event {i} dec node {e["n"]}: capture has no I, model has I')
                if not n.L:
                    need(e.get('noL') == 1, f'{name}: event {i} dec node {e["n"]}: model has no L, capture has L')
                    c['dec_noL'] += 1
                else:
                    need(not e.get('noL'), f'{name}: event {i} dec node {e["n"]}: capture has no L, model has L')
                    chain, A = [], n.par
                    while A >= 0 and len(chain) < 40:
                        chain.append(A)
                        a = node(A, i, 'dec ancestor')
                        if not a.I:
                            break
                        A = a.par
                    end = 'nop' if A < 0 else ('cap' if len(chain) >= 40 and st[A].I else 'noI')
                    need(chain == e['as'] and end == e['end'],
                         f'{name}: event {i} dec node {e["n"]}: ancestors {e["as"]} end {e["end"]}, model {chain} end {end}')
                    c['dec_%s_end' % end if end != 'cap' else 'dec_cap'] += 1
                    if len(e['anc']) > e['depth']:
                        c['chain_gt_depth'] += 1
                    # L5: provenance of the vector the decay reads (06e section 2.4 selection)
                    if e['l38'] == 6 or e['l58'] >= 0:
                        pv, site, key = e['p70'], (RA_CUT if e['l38'] == 6 else RA_FK), ('src_cut' if e['l38'] == 6 else 'src_fk')
                    elif e['l18']:
                        pv, site, key = e['p28'], RA_OPT, 'src_opt'
                    else:
                        pv, site, key = None, None, None
                    if pv is None:
                        c['novec'] += 1
                    else:
                        need(pv[1] == site and pv[0] > n.rel6,
                             f'{name}: event {i} dec node {e["n"]}: vector last written by {pv} (node LP at {n.rel6}), want site {site:#x}')
                        w = ev[pv[0]]
                        need((w['e'] == 'sol' and w['ra'] == site) or (w['e'] == 'fk_' and site == RA_FK and w['rc'] == 0 and w['l58'] == e['l58']
                                                                      and w['l54'] == e['l54']),
                             f'{name}: event {i} dec: writer event {pv[0]} is {w}')
                        c[key] += 1
        i += 1
    return c


def check_case(c, d, old, V=frozenset()):
    need(d['ok'] and not d['errors'], f'{c}: capture not ok')
    need(old['output'] == d['output'], f'L4 {c}: output {d["output"]!r} != 06e {old["output"]!r}')
    a = [tuple(e.get(k) for k in DGATE) if e['e'] == 'dgate' else tuple(e.get(k) for k in DEC)
         for e in d['events'] if e['e'] in ('dgate', 'dec')]
    b = [tuple(e.get(k) for k in DGATE) if e['e'] == 'dgate' else tuple(e.get(k) for k in DEC)
         for e in old['events'] if e['e'] in ('dgate', 'dec')]
    for i, (x, y) in enumerate(zip(a, b)):
        need(x == y, f'L4 {c}: decay event {i}: {x} != 06e {y}')
    need(len(a) == len(b), f'L4 {c}: {len(a)} decay events != 06e {len(b)}')
    return replay(d, V, c)


def rejected(d, old, v, c):
    try:
        check_case(c, d, old, frozenset([VARIANTS[v]]))
        return False
    except Fail:
        return True


def a1(manifest):
    tool = sha(TOOL)
    out = {}
    for c in CASES:
        hs = []
        for r in RUNS:
            p = os.path.join(DATA, f'{c}_{r}.json.gz')
            need(os.path.exists(p), f'L1: missing {p}')
            hs.append(sha(p))
            need(manifest['files'].get(os.path.basename(p)) == hs[-1], f'L1: {p} sha256 != manifest')
        need(len(set(hs)) == 1, f'L1: {c}: runs differ {hs}')
        d = load(os.path.join(DATA, f'{c}_1.json.gz'))
        need(d['tool_sha256'] == tool, f'L1: {c}: capture tool sha256 {d["tool_sha256"]} != current tool {tool}')
        need(d['params'] == PARAMS, f'L1: {c}: params {d["params"]}')
        need(d['binary_sha256'] == '8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c', f'L1: {c}: binary')
        out[c] = d
    return out


REQUIRED = ('rel0', 'rel3', 'rel4', 'rel6', 'cascade0', 'free1', 'stop_old5', 'stop_pstage3', 'root_fr', 'dec_noI_end',
            'dec_nop_end', 'src_cut', 'src_fk', 'src_opt', 'rc_checked')
# V expectation: variant -> branch counter that must be > 0 in a case for the variant to be rejected there
VKEY = {'W1': 'v_old5', 'W2': 'v_pstage3', 'W3': 'v_cascade0', 'W4': 'v_stage4', 'W5': 'v_free1', 'W6': 'v_root'}


def acceptance(docs, olds):
    stats = {}
    for c in CASES:
        stats[c] = check_case(c, docs[c], olds[c])
        print(c, 'PASS', json.dumps({k: v for k, v in stats[c].items() if v}))
    tot = {k: sum(st[k] for st in stats.values()) for k in REQUIRED}
    print('L7 branch totals', json.dumps(tot))
    need(all(tot.values()), f'L7: a required branch never ran: {[k for k, v in tot.items() if not v]}')
    return stats


def tampers(docs, olds):
    def idx(d, pred):
        return next(i for i, e in enumerate(d['events']) if pred(e))

    def t_fr_stage(d):
        d['events'][idx(d, lambda e: e['e'] == 'fr' and e['s'] == 3)]['s'] = 2

    def t_fr_drop(d):
        i = idx(d, lambda e: e['e'] == 'fr' and e['s'] == 1)
        del d['events'][i:i + 2]

    def t_dec_anc(d):
        e = d['events'][idx(d, lambda e: e['e'] == 'dec' and len(e.get('as') or []) > 1)]
        e['as'][0] += 1

    def t_dec_end(d):
        e = d['events'][idx(d, lambda e: e['e'] == 'dec' and e.get('end') == 'noI')]
        e['end'] = 'nop'

    def t_src(d):
        e = d['events'][idx(d, lambda e: e['e'] == 'dec' and e.get('p28') and e.get('l38') == 2 and e.get('l58', 0) < 0)]
        e['p28'][1] = RA_CUT

    def t_basic(d):
        d['events'][idx(d, lambda e: e['e'] == 'sol' and 'basnz' in e)]['basnz'] = 1

    plan = [('fr stage', 'pk1', t_fr_stage), ('fr drop', 'tr12-30', t_fr_drop), ('dec ancestor', 'neos5', t_dec_anc),
            ('dec end', 'pk1', t_dec_end), ('vector source', 'markshare_4_0', t_src), ('basic nonzero', 'neos-860300', t_basic)]
    for nm, c, fn in plan:
        d = copy.deepcopy(docs[c])
        fn(d)
        try:
            check_case(c, d, olds[c])
            raise Fail(f'T {nm} ({c}): tampered capture passed')
        except Fail as ex:
            if str(ex).startswith('T '):
                raise
            print(f'T {nm} ({c}): rejected: {str(ex)[:160]}')


def variants(docs, olds, stats):
    for v, key in VKEY.items():
        for c in CASES:
            want = stats[c][key] > 0
            got = rejected(docs[c], olds[c], v, c)
            need(got == want, f'V {v} ({c}): rejected {got}, expected {want} ({key} = {stats[c][key]})')
        print(f'V {v}: rejected in {[c for c in CASES if stats[c][key] > 0]}')


def main():
    args = sys.argv[1:]
    if args[:1] == ['--sweep']:
        bad = 0
        for p in args[1:]:
            d = load(p)
            c = d['model'].replace('.mps.gz', '')
            op = os.path.join(OLD, f'{c}_1.json.gz')
            old = load(op) if os.path.exists(op) and d['params'] == PARAMS else None
            try:
                st = check_case(c, d, old) if old else replay(d, name=c)
                rej = {v: (rejected(d, old, v, c) if old else _rej_replay(d, v, c)) for v in VKEY}
                rule = {v: st[k] > 0 for v, k in VKEY.items()}
                print(os.path.basename(p), 'PASS', 'L4' if old else '(no 06e)', json.dumps({k: v for k, v in st.items() if v}))
                print('   variants rejected', json.dumps(rej), 'rule', 'same' if rej == rule else 'DIFF ' + json.dumps(rule))
                if rej != rule:
                    bad += 1
            except Fail as ex:
                bad += 1
                print(os.path.basename(p), 'FAIL', ex)
        return 1 if bad else 0
    mpath = os.path.join(DATA, 'manifest.json')
    try:
        if args[:1] == ['--write-manifest']:
            files = {}
            for c in CASES:
                for r in RUNS:
                    p = os.path.join(DATA, f'{c}_{r}.json.gz')
                    files[os.path.basename(p)] = sha(p)
            with open(mpath, 'w') as fh:
                json.dump({'task': 'M2-06f-2', 'schema': 1, 'tool': 'tools/mip_node_life.js', 'tool_sha256': sha(TOOL),
                           'binary_sha256': '8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c',
                           'params': PARAMS, 'models': '../re/bench/data/miplib/<case>.mps.gz', 'files': files}, fh, indent=1)
            print('manifest written', mpath)
            return 0
        need(os.path.exists(mpath), 'L1: no manifest (run --write-manifest once after the capture)')
        docs = a1(json.load(open(mpath)))
        olds = {c: load(os.path.join(OLD, f'{c}_1.json.gz')) for c in CASES}
        stats = acceptance(docs, olds)
        if '--self-test' in args:
            tampers(docs, olds)
            variants(docs, olds, stats)
        print('ACCEPTANCE PASS' + (' (+ self-test)' if '--self-test' in args else ''))
        return 0
    except Fail as ex:
        print('FAIL', ex)
        return 1


def _rej_replay(d, v, c):
    try:
        replay(d, frozenset([VARIANTS[v]]), c)
        return False
    except Fail:
        return True


if __name__ == '__main__':
    sys.exit(main())
