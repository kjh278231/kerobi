#!/usr/bin/env python3
"""M2-06h-4: count the 695bd7 branches that M2-06h-3 never saw, in tools/mip_sb_loop.js captures.

Usage: mip_sb_census.py CAPTURE.json.gz... [--self-test]
Exit 0 always for a census; --require BRANCH... exits 1 when a named branch has 0 hits.
"""
import argparse
import collections
import gzip
import json
import struct

BIG = 1e100
TARGETS = ['up contradiction', 'down contradiction', 'timer expired', 'down >= 1e100 only',
           'both >= 1e100', 'up prefilled (down only)', 'status7 dual < parent', 'continuous candidate',
           'no propagation', 'stride > 1', 'other status', 'restore strict', 'duplicate column']


def f(h):
    return struct.unpack('>d', bytes.fromhex(h))[0]


def census(d):
    c = collections.Counter()
    for e in d['events']:
        if e.get('e') != 'double':
            continue
        c['batches'] += 1
        c['candidates'] += e['n']
        c['stride > 1'] += e['stride'] > 1
        types = bytes.fromhex(e['types'])
        c['continuous candidate'] += sum(types[j] == ord('C') for j in e['j'])
        steps = e['steps']
        parent = next((f(s['value']) for s in steps if s['kind'] == 'attr' and s['attr'] == 'ObjVal'), None)
        dirs = collections.defaultdict(set)
        last = None
        orig = {s['kind']: s['v'] for s in steps if s['kind'] in ('origLower', 'origUpper')}
        for i, s in enumerate(steps):
            k = s['kind']
            if k == 'propEnter':
                last = s
                # By call site, not by dir: the continuous-column down path also calls with 0x3e (C:761-783).
                dirs[s['col']].add(62 if s['ra'] == '572375' else 60)
            elif k == 'prop':
                if s['out'] != 0:
                    c['up contradiction' if last['ra'] == '572375' else 'down contradiction'] += 1
                c['duplicate column'] += len(s['list']) != len(set(s['list']))
                for col, fl, lo, up in zip(s['list'], s['flags'], s['lower'], s['upper']):
                    # C:592-647: a set flag requests a restore only when the bound moved strictly.
                    c['restore strict'] += (fl & 1 and not f(orig['origLower'][col]) < f(lo)) + \
                        (fl & 2 and not f(up) < f(orig['origUpper'][col]))
            elif k == 'timer' and s['r'] != 0:
                c['timer expired'] += 1
            elif k == 'solve' and i != len(steps) - 1:
                c[f'status {s["status"]}'] += 1
                if s['status'] not in (2, 6, 7):
                    c['other status'] += 1
                if s['status'] in (7, 11, -1000) and parent is not None and f(s['dual']) < parent:
                    c['status7 dual < parent'] += 1
        if not dirs and any(s['kind'] == 'solve' for s in steps[:-1]):
            c['no propagation'] += 1
        for col, ds in dirs.items():
            c['up prefilled (down only)'] += ds == {60}
        for a, z in zip(e['a'], e['z']):
            a, z = f(a), f(z)
            c['down >= 1e100 only'] += a >= BIG > z
            c['both >= 1e100'] += a >= BIG and z >= BIG
    return c


def self_test():
    h = lambda x: struct.pack('>d', x).hex()
    e = {'e': 'double', 'n': 2, 'stride': 1, 'j': [0, 1], 'types': '4243',
         'a': [h(BIG), h(1.0)], 'z': [h(2.0), h(3.0)],
         'steps': [{'kind': 'attr', 'attr': 'ObjVal', 'value': h(5.0)},
                   {'kind': 'origLower', 'v': [h(0.0)] * 2}, {'kind': 'origUpper', 'v': [h(1.0)] * 2},
                   {'kind': 'propEnter', 'col': 0, 'dir': 62, 'ra': '572375'},
                   {'kind': 'prop', 'out': 1, 'list': [0], 'flags': [1], 'lower': [h(1.0)], 'upper': [h(1.0)]},
                   {'kind': 'propEnter', 'col': 1, 'dir': 60, 'ra': '572ca6'},
                   {'kind': 'prop', 'out': 0, 'list': [1, 1], 'flags': [2, 3], 'lower': [h(0.0)] * 2, 'upper': [h(0.0), h(1.0)]},
                   {'kind': 'solve', 'status': 7, 'dual': h(4.0)},
                   {'kind': 'timer', 'r': 1}, {'kind': 'solve', 'status': 2, 'dual': h(0.0)}]}
    c = census({'events': [e]})
    want = {'up contradiction': 1, 'timer expired': 1, 'down >= 1e100 only': 1, 'continuous candidate': 1,
            'up prefilled (down only)': 1, 'status7 dual < parent': 1, 'status 7': 1,
            'restore strict': 2, 'duplicate column': 1}
    for k, v in want.items():
        assert c[k] == v, (k, c[k], v)
    assert c['down contradiction'] == c['both >= 1e100'] == c['no propagation'] == 0
    print('self-test PASS')


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('files', nargs='*')
    ap.add_argument('--self-test', action='store_true')
    ap.add_argument('--require', nargs='*', default=[])
    a = ap.parse_args()
    if a.self_test:
        self_test()
    total = collections.Counter()
    for p in a.files:
        with gzip.open(p) as fh:
            d = json.load(fh)
        c = census(d)
        total += c
        hits = ' '.join(f'{k}={c[k]}' for k in TARGETS if c[k])
        st = ' '.join(f'{k}={v}' for k, v in sorted(c.items()) if k.startswith('status '))
        print(f'{p}: ok={d.get("ok")} batches={c["batches"]} cand={c["candidates"]} {st} | {hits or "no target"}')
    if a.files:
        print('TOTAL', ' '.join(f'{k}={total[k]}' for k in TARGETS))
    missing = [k for k in a.require if total[k] == 0]
    if missing:
        print('MISSING', missing)
        raise SystemExit(1)


if __name__ == '__main__':
    main()
