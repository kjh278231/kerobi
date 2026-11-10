#!/usr/bin/env python3
"""M2-06a (docs/m2/mip_node_select_spec.md): replay the original node containers from tools/mip_node_select.js traces.

A1 three independent runs per case are identical (events, counts and output line).
A2 the global heaps (one per tree) replayed with the C rules equal every heap snapshot (node id order, key and bound
   bits) at 6e8eb3 calls that pop and at 6e460c returns: 6e5802 insert (sift-up), 6e8eb3 pop (sift-down), 6e460c prune
   (stable removal of the nodes it frees, then sift-down from the last index to 0). Popped nodes are the deferred adds of that call followed by the selected node.
A3 tie variants V1-V4 replayed the same way, reported per case as rejected (first difference) or surviving, at the pop
   level (which node is taken) and at the array level (heap layout).
A4 plunge list after a selection: the chosen entry is swap-removed with the last one, the kept entries keep that order,
   and the removed ones that are re-inserted reach 6e5802 in that order.
A5 tie counts: pops with another heap entry of the same key bits.

  python3 tools/mip_node_select_check.py                  # controls + acceptance on bench/oracle/m2_06a
  python3 tools/mip_node_select_check.py --sweep FILE...  # exploration only (no manifest, no 3-run check)
  python3 tools/mip_node_select_check.py --self-test      # tampers must be rejected
  python3 tools/mip_node_select_check.py --write-manifest
"""
import argparse, copy, gzip, hashlib, json, struct, sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
DATA = ROOT / 'bench/oracle/m2_06a'
TOOL = ROOT / 'tools/mip_node_select.js'
BINARY = '8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c'
CASES = {  # name -> (model file, NodeLimit); fixed in docs/m2/mip_node_select_spec.md §4 before the official capture
    'markshare_4_0': ('markshare_4_0.mps.gz', 2000),
    'pk1': ('pk1.mps.gz', 2000),
    'neos5': ('neos5.mps.gz', 2000),
    'tr12-30': ('tr12-30.mps.gz', 2000),
}
RUNS = (1, 2, 3)
RULES = ('C', 'V1', 'V2', 'V3', 'V4')


class Fail(Exception):
    pass


def need(cond, msg):
    if not cond:
        raise Fail(msg)


def f(h):
    return struct.unpack('>d', bytes.fromhex(h))[0]


def sha(p):
    return hashlib.sha256(Path(p).read_bytes()).hexdigest()


def load(p):
    return json.loads(gzip.decompress(Path(p).read_bytes()))


# ---- heap rules; entries are [id, key hex, bound hex] ------------------------------------------------------------
def k(e):
    return f(e[1])


def sift_down(h, i, left=False, down_eq=False):
    """6e8eb3 C:241-266 and 6e460c C:73-101: two children: m = min with ties to the right child, stop when key <= m;
    one child: move only when strictly smaller. left (V1): ties to the left child. down_eq (V3): also move on key == m."""
    n, x = len(h), h[i]
    xk = k(x)
    while True:
        c1, c2 = 2 * i + 1, 2 * i + 2
        if n <= c1:
            break
        if n <= c2:
            if (k(h[c1]) <= xk) if down_eq else (k(h[c1]) < xk):
                h[i] = h[c1]
                i = c1
            break
        k1, k2 = k(h[c1]), k(h[c2])
        j = (c2 if k2 < k1 else c1) if left else (c2 if k2 <= k1 else c1)
        if (xk < min(k1, k2)) if down_eq else (xk <= min(k1, k2)):
            break
        h[i] = h[j]
        i = j
    h[i] = x


class Heap:
    """rule 'C' (original), heap variants 'V1'-'V3', or 'V4' (stable sort by key, then insertion order)."""
    def __init__(self, rule):
        self.rule, self.h, self.seq = rule, [], 0
        self.left, self.down_eq, self.tie_up = rule == 'V1', rule == 'V3', rule == 'V2'

    def push(self, e):
        if self.rule == 'V4':
            self.h.append((k(e), self.seq, e))
            self.seq += 1
            self.h.sort(key=lambda t: (t[0], t[1]))
            return
        h = self.h  # 6e5802: append, move up while the parent key is greater (V2: greater or equal)
        h.append(e)
        i, x = len(h) - 1, k(e)
        while i:
            p = (i - 1) >> 1
            if (k(h[p]) < x) if self.tie_up else (k(h[p]) <= x):
                break
            h[i] = h[p]
            i = p
        h[i] = e

    def pop(self):
        if self.rule == 'V4':
            return self.h.pop(0)[2]
        h = self.h  # 6e8eb3 C:233-266: root out, last entry to the root, sift down
        top = h[0]
        h[0] = h[-1]
        h.pop()
        if h:
            sift_down(h, 0, self.left, self.down_eq)
        return top

    def prune(self, drop):
        """6e460c C:49-104: keep the others in array order, then sift down from the last index to 0."""
        if self.rule == 'V4':
            self.h = [t for t in self.h if t[2][0] not in drop]
            return
        self.h = [e for e in self.h if e[0] not in drop]
        for i in range(len(self.h) - 1, -1, -1):
            sift_down(self.h, i, self.left, self.down_eq)

    def array(self):
        return [t[2] for t in self.h] if self.rule == 'V4' else self.h

    def reset(self, arr):
        self.h = [(k(e), j, e) for j, e in enumerate(arr)] if self.rule == 'V4' else [list(e) for e in arr]
        self.seq = len(arr)

    def __len__(self):
        return len(self.h)


def ids(lst):
    return [e[0] for e in lst]


def first_diff(a, b):
    return next((i for i, (x, y) in enumerate(zip(a, b)) if x != y), min(len(a), len(b)))


def replay(trace, rule='C', arrays=True, stats=None):
    """Replay one trace; raise Fail at the first difference. Returns counts of the checked items."""
    ev = trace['events']
    for e in ev:  # pair each enter with its leave (the pop count M+0xd0 is read at leave)
        if e['e'] == 'enter':
            op = e
        elif e['e'] == 'leave':
            op['_leave'] = e
    heaps, dirty, h10 = {}, set(), {}
    n = dict.fromkeys(('enter', 'snap', 'pops', 'ins', 'ins10', 'defer', 'prune', 'pruned', 'a4', 'dropped', 'restart',
                       'resync', 'tfree'), 0)
    cur, pr = None, None

    def heap(t, x):
        need(t in heaps, f'event {x}: tree {t} unknown')
        return heaps[t]

    def close_prune(x):
        nonlocal pr
        if pr is None:
            return
        hp, drop = heap(pr['tree'], x), set(pr['drop'])
        if pr['tree'] not in dirty:
            have = {e[0]: e for e in hp.array()}
            main = [i for i in pr['drop'] if i in have]
            need(len(main) + sum(1 for i in pr['drop'] if i in h10.get(pr['tree'], set())) == len(pr['drop']),
                 f'event {x}: prune drops a node not in the heaps')
            kept = [f(e[2]) for e in hp.array() if e[0] not in drop]
            if main and kept:
                need(max(kept) < min(f(have[i][2]) for i in main), f'event {x}: prune keeps a bound >= a dropped bound')
            hp.prune(drop)
            n['pruned'] += len(main)
        h10.get(pr['tree'], set()).difference_update(drop)
        n['prune'] += 1
        pr = None

    def snap(st, x, where):
        hp = heap(st['tree'], x)
        if st['tree'] in dirty:
            if 'heap' in st:
                hp.reset(st['heap'])
                dirty.discard(st['tree'])
                n['resync'] += 1
            return
        need(st['heap_n'] == len(hp), f'event {x} {where}: heap size {st["heap_n"]} vs replay {len(hp)}')
        if 'heap' in st:
            arr = hp.array()
            if arrays and rule != 'V4':
                need(st['heap'] == arr, f'event {x} {where}: heap array differs at index {first_diff(st["heap"], arr)}')
            n['snap'] += 1

    def do_pops(x):
        cur['popdone'] = True
        en, lv = cur['enter'], cur['enter']['_leave']
        npop = lv['st']['pops'] - en['st']['pops']
        need(npop == 0 or not en['st']['plunge'], f'event {x} sel {en["sel"]}: pops with a non-empty plunge list')
        hp = heap(en['st']['tree'], x)
        for _ in range(npop):
            need(len(hp) > 0, f'event {x} sel {en["sel"]}: pop from empty heap')
            if stats is not None:
                arr = hp.array()
                stats['pops'] += 1
                stats['tie_pops'] += sum(1 for e in arr if e[1] == arr[0][1]) > 1
            cur['popped'].append(hp.pop()[0])
            n['pops'] += 1

    for x, e in enumerate(ev):
        t = e['e']
        if t != 'acct':
            close_prune(x)
        if t == 'tnew':
            need(e['tree'] not in heaps, f'event {x}: tree {e["tree"]} created twice')
            heaps[e['tree']] = Heap(rule)
        elif t == 'tfree':
            if e['tree'] not in dirty:
                need(e['heap_n'] == len(heap(e['tree'], x)), f'event {x}: freed tree {e["tree"]} size differs')
            heaps.pop(e['tree'], None)
            dirty.discard(e['tree'])
            n['tfree'] += 1
        elif t == 'enter':
            need(cur is None, f'event {x}: enter inside a selection')
            snap(e['st'], x, f'sel {e["sel"]} enter')
            cur = {'enter': e, 'popped': [], 'defer': [], 'ins': [], 'popdone': False}
            n['enter'] += 1
        elif t == 'defer':
            n['defer'] += 1
            if cur is not None:
                cur['defer'].append(e['node'][0])
        elif t == 'ins':
            n['ins'] += 1
            if e['node'][6] == 10:
                h10.setdefault(e['tree'], set()).add(e['node'][0])
                n['ins10'] += 1
                continue
            if cur is not None and e['in_sel'] and not cur['popdone']:
                do_pops(x)
            if cur is not None and e['in_sel']:
                cur['ins'].append(e['node'][0])
            hp = heap(e['tree'], x)
            if e['tree'] in dirty:
                continue
            need(e['heap_n'] == len(hp), f'event {x}: insert of {e["node"][0]} sees heap size {e["heap_n"]} vs replay {len(hp)}')
            nd = e['node']
            hp.push([nd[0], nd[2], nd[1]])
        elif t == 'prune':
            if e['tree'] not in dirty:
                need(e['heap_n'] == len(heap(e['tree'], x)), f'event {x}: prune sees heap size {e["heap_n"]}')
            pr = {'tree': e['tree'], 'drop': []}
        elif t == 'acct':
            if e['from'] == '6e460c':
                need(pr is not None and pr['tree'] == e['tree'], f'event {x}: 6e460c removal outside a prune')
                pr['drop'].append(e['node'])
            elif e['from'] == '6a6563':  # restart (M2-06j): tree contents no longer modeled until a snapshot
                dirty.add(e['tree'])
                n['restart'] += 1
            else:
                raise Fail(f'event {x}: unmodeled node removal by {e["from"]}')
        elif t == 'leave':
            need(cur is not None and cur['enter']['sel'] == e['sel'], f'event {x}: leave without enter')
            if not cur['popdone']:
                do_pops(x)
            cur['leave'] = e
            check_leave(cur, x, n)
            snap(e['st'], x, f'sel {e["sel"]} leave')
            cur = None
        elif t == 'pruned':  # 6e460c return: heap after the stable removal and the heapify
            if e['tree'] not in dirty:
                arr = heap(e['tree'], x).array()
                if arrays and rule != 'V4':
                    need(e['heap'] == arr, f'event {x}: heap after prune differs at index {first_diff(e["heap"], arr)}')
                else:
                    need(len(e['heap']) == len(arr), f'event {x}: heap size after prune differs')
                n['snap'] += 1
        elif t == 'flush':
            pass
        else:
            raise Fail(f'event {x}: unknown event {t}')
    close_prune(len(ev))
    need(cur is None, 'trace ends inside a selection')
    return n


def check_leave(cur, x, n):
    en, lv = cur['enter'], cur['leave']
    sel, popped = en['sel'], cur['popped']
    lst = ids(en['st']['plunge'])
    if popped:
        need(cur['defer'] == popped[:-1] or (lv['node'] is None and cur['defer'] == popped),
             f'event {x} sel {sel}: rejected pops {popped[:-1]} vs deferred adds {cur["defer"]}')
        if lv['node'] is None:
            need(not lv['st']['plunge'], f'event {x} sel {sel}: null selection with a plunge list')
            return
        lst = lst + [popped[-1]]
    else:
        need(not cur['defer'], f'event {x} sel {sel}: deferred adds without pops')
    if lv['node'] is None:
        need(not lst, f'event {x} sel {sel}: null selection with candidates')
        return
    sid = lv['node'][0]
    need(sid in lst, f'event {x} sel {sel}: selected {sid} not in the plunge list {lst}')
    sw = lst[:]
    sw[lst.index(sid)] = sw[-1]
    sw.pop()
    kept = ids(lv['st']['plunge'])
    it = iter(sw)
    need(all(any(a == b for b in it) for a in kept), f'event {x} sel {sel}: kept {kept} is not a subsequence of {sw}')
    removed = [a for a in sw if a not in kept]
    it = iter(removed)
    need(all(any(a == b for b in it) for a in cur['ins']), f'event {x} sel {sel}: re-inserted {cur["ins"]} not in order of {removed}')
    need(len(kept) + len(removed) == len(sw), f'event {x} sel {sel}: kept entries not from the list')
    n['a4'] += 1
    n['dropped'] += len(removed) - len(cur['ins'])


def analyse(trace):
    """C replay with tie counts, then every variant at pop level (arrays=False) and array level."""
    stats = {'pops': 0, 'tie_pops': 0}
    n = replay(copy.deepcopy(trace), 'C', True, stats)
    var = {}
    for r in RULES[1:]:
        for arrays in (False, True):
            try:
                replay(copy.deepcopy(trace), r, arrays)
                var[r + ('/array' if arrays else '/pop')] = 'survives'
            except Fail as ex:
                var[r + ('/array' if arrays else '/pop')] = 'rejected: ' + str(ex)
    return n, stats, var


def strip(trace):
    return {'output': trace['output'], 'events': trace['events'], 'counts': trace['counts']}


def path(case, run):
    return DATA / f'{case}_{run}.json.gz'


def controls(traces):
    """A1 and the manifest: files, model and binary identity, current tool hash, ok runs, 3 identical runs."""
    man = json.loads((DATA / 'manifest.json').read_text())
    need(man['tool_sha256'] == sha(TOOL), 'capture tool changed since the manifest')
    need(sorted(man['files']) == sorted(f'{c}_{r}.json.gz' for c in CASES for r in RUNS), 'manifest file list')
    for c, (model, limit) in CASES.items():
        for r in RUNS:
            tr = traces[c][r]
            need(man['files'][f'{c}_{r}.json.gz'] == sha(path(c, r)), f'{c} run {r}: file hash differs from manifest')
            need(tr['ok'] and not tr['errors'], f'{c} run {r}: capture not ok')
            need(tr['binary_sha256'] == BINARY and tr['tool_sha256'] == man['tool_sha256'], f'{c} run {r}: binary/tool hash')
            need(tr['model'] == model and tr['params'].get('NodeLimit') == limit and tr['params'].get('Threads') == 1,
                 f'{c} run {r}: model/params')
            need(tr['model_sha256'] == man['models'][model], f'{c} run {r}: model hash')
        base = strip(traces[c][1])
        for r in RUNS[1:]:
            other = strip(traces[c][r])
            if other != base:
                i = first_diff(base['events'], other['events'])
                raise Fail(f'A1 {c}: run {r} differs from run 1 (output {base["output"] == other["output"]}, first event {i})')
    return man


def acceptance(traces, verbose=True):
    """A2/A4 on run 1 of each case (A1 makes the others equal), A3/A5 reported. Every case must pop and snapshot."""
    out = {}
    for c in CASES:
        n, stats, var = analyse(traces[c][1])
        need(n['snap'] > 0 and n['pops'] > 0 and n['a4'] > 0, f'{c}: nothing checked ({n})')
        out[c] = (n, stats, var)
        if verbose:
            print(f'{c}: {traces[c][1]["output"]} C PASS {json.dumps(n)} {json.dumps(stats)}')
            for r, v in var.items():
                print(f'   {r}: {v[:160]}')
    return out


def load_all():
    return {c: {r: load(path(c, r)) for r in RUNS} for c in CASES}


def self_test():
    traces = load_all()
    controls(traces)
    acceptance(traces, verbose=False)
    c0 = next(iter(CASES))
    base = traces[c0][1]
    ev = base['events']
    enters = [i for i, e in enumerate(ev) if e['e'] == 'enter' and len(e['st'].get('heap', [])) >= 2
              and len({h[1] for h in e['st']['heap']}) >= 2]
    ins = [i for i, e in enumerate(ev) if e['e'] == 'ins' and e['in_sel']]
    leaves = [i for i, e in enumerate(ev) if e['e'] == 'leave' and len(e['st']['plunge']) >= 2]
    prunes = [i for i, e in enumerate(ev) if e['e'] == 'acct' and e['from'] == '6e460c']
    tampers = []

    def t_swap_heap(tr):
        st = tr['events'][enters[-1]]['st']['heap']
        j = next(j for j in range(1, len(st)) if st[j][1] != st[0][1])
        st[0], st[j] = st[j], st[0]
    tampers.append(('heap order', t_swap_heap))

    def t_key(tr):
        tr['events'][ins[len(ins) // 2]]['node'][2] = struct.pack('>d', -1e30).hex()
    tampers.append(('insert key', t_key))

    def t_drop_ins(tr):
        del tr['events'][ins[len(ins) // 2]]
    tampers.append(('insert removed', t_drop_ins))

    def t_bound(tr):
        st = tr['events'][enters[-1]]['st']['heap']
        st[-1][2] = struct.pack('>d', 1e30).hex()
    tampers.append(('heap bound bits', t_bound))

    def t_list(tr):
        tr['events'][leaves[0]]['st']['plunge'].reverse()
    if leaves:
        tampers.append(('plunge order', t_list))

    def t_selected(tr):
        lv = tr['events'][leaves[0]]
        lv['node'] = list(lv['st']['plunge'][0])
    if leaves:
        tampers.append(('selected node', t_selected))

    def t_prune(tr):
        del tr['events'][prunes[0]]
    if prunes:
        tampers.append(('prune removal dropped', t_prune))

    def t_unmodeled(tr):
        tr['events'].insert(ins[0], {'e': 'acct', 'in_sel': 0, 'from': '5fe9cd', 'tree': tr['events'][ins[0]]['tree'], 'node': -1})
    tampers.append(('unmodeled removal', t_unmodeled))
    bad = 0
    for name, fn in tampers:
        tr = copy.deepcopy(base)
        fn(tr)
        try:
            replay(tr, 'C')
            print(f'tamper {name}: NOT rejected')
            bad += 1
        except Fail as ex:
            print(f'tamper {name}: rejected ({ex})')
    # A1: one changed event in run 2
    t2 = copy.deepcopy(traces)
    t2[c0][2]['events'][ins[0]]['node'][2] = '0000000000000000'
    try:
        controls_mem(t2)
        print('tamper run 2 event: NOT rejected')
        bad += 1
    except Fail as ex:
        print(f'tamper run 2 event: rejected ({ex})')
    # an empty trace must not pass acceptance
    t3 = copy.deepcopy(traces)
    t3[c0][1]['events'] = [e for e in t3[c0][1]['events'] if e['e'] in ('tnew',)]
    try:
        acceptance(t3, verbose=False)
        print('tamper empty trace: NOT rejected')
        bad += 1
    except Fail as ex:
        print(f'tamper empty trace: rejected ({ex})')
    print('SELF-TEST', 'FAIL' if bad else 'PASS', f'({len(tampers) + 2} tampers)')
    return 1 if bad else 0


def controls_mem(traces):
    """A1 only, on in-memory traces (self-test)."""
    for c in CASES:
        base = strip(traces[c][1])
        for r in RUNS[1:]:
            if strip(traces[c][r]) != base:
                raise Fail(f'A1 {c}: run {r} differs from run 1')


def write_manifest():
    models = {}
    for c, (model, _) in CASES.items():
        models[model] = load(path(c, 1))['model_sha256']
    man = {'task': 'M2-06a', 'schema': 1, 'binary_sha256': BINARY, 'tool_sha256': sha(TOOL),
           'cases': {c: {'model': m, 'NodeLimit': l, 'Threads': 1, 'Seed': 0} for c, (m, l) in CASES.items()},
           'models': models, 'files': {f'{c}_{r}.json.gz': sha(path(c, r)) for c in CASES for r in RUNS}}
    (DATA / 'manifest.json').write_text(json.dumps(man, indent=1, sort_keys=True) + '\n')
    print('wrote', DATA / 'manifest.json')


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--sweep', nargs='+')
    ap.add_argument('--self-test', action='store_true')
    ap.add_argument('--write-manifest', action='store_true')
    a = ap.parse_args()
    try:
        if a.sweep:
            rc = 0
            for p in a.sweep:
                tr = load(p)
                try:
                    n, stats, var = analyse(tr)
                    print(Path(p).name, tr['output'], 'C PASS', json.dumps(n), json.dumps(stats))
                    for r, v in var.items():
                        print('  ', r, v[:160])
                except Fail as ex:
                    print(Path(p).name, tr['output'], 'C FAIL', ex)
                    rc = 1
            return rc
        need(CASES, 'no cases fixed yet')
        if a.write_manifest:
            write_manifest()
            return 0
        if a.self_test:
            return self_test()
        traces = load_all()
        controls(traces)
        print('A1 PASS', ', '.join(f'{c} x{len(RUNS)}' for c in CASES))
        acceptance(traces)
        print('ACCEPTANCE PASS')
        return 0
    except Fail as ex:
        print('FAIL', ex)
        return 1


if __name__ == '__main__':
    sys.exit(main())
