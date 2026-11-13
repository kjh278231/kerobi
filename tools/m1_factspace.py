#!/usr/bin/env python3
"""M1-02w-2: the LU element pool size F+0x158 that `045a76` (0xe3130, asm 0xe3697–0xe3bf0) sets before the first
factor. Contract and C/asm references: docs/m1/m1_02w2_factspace.md.

usage (PYTHONPATH=<main worktree>/python ../.venv/bin/python):
  tools/m1_factspace.py [POOL_DIR] [--self-test]   M1-02w pool captures: first 036ade entry F+0x158 vs the formula,
                                                   nnz from rsolver NumNZs, k 5, c 0 (bench/oracle/m1_02w/pool)
  tools/m1_factspace.py --direct DIR               m1_factspace.js captures: every 045a76 call's F+0x158 vs the formula
                                                   on the m·n·nnz·k·c read at its entry; repeats *_1/_2/_3 identical
Exit 0 when everything matches.
"""
import glob, gzip, json, math, os, re, sys

I64 = 9.223372036854776e18
NETLIB = os.environ.get('NETLIB', '../re/bench/data/netlib')


def factspace(m, n, nnz, k=5, c=0, mut=None, br=None):
    """F+0x158 when the first allocation try succeeds (loop at 0xe3a7c ends on its first pass). `br` collects the
    branches taken; `mut` names a deliberate rule change for the self-test."""
    br = br if br is not None else set()
    fm, fn, fz = float(m), float(n), float(nnz)
    if m < n:  # 0xe36a1
        d = fz / fn
        if m < 500:
            x = d * d; br.add('m<n,m<500')
        elif m < 3000:
            x = d * math.sqrt(d); br.add('m<n,m<3000')
        elif m < 10000 and fm > d * 4.0:
            # shortcut: asm 0xe3707–0xe37cb read, no case in the captures; check it before relying on m ≥ 3000.
            q = d * math.sqrt(math.sqrt(d))
            x = q if fm * 0.25 > q else fm * 0.25; br.add('m<n,m<10000')
        else:
            x = d; br.add('m<n,other')
        half = float(int(m / 2) + 2)
        if mut == 'half':
            half = float(int(m / 2))
        d0 = (half if half < x else x) * fm
        if half < x:
            br.add('m<n,half')
    elif m == 0:  # 0xe3835
        d0 = 100.0; br.add('m=0')
    else:  # 0xe385d
        d = fz / fm
        if n < 500:
            x = d * d; br.add('m>=n,n<500')
        elif n < 3000:
            x = d * math.sqrt(d); br.add('m>=n,n<3000')
        elif n < 10000:
            x = d * math.sqrt(math.sqrt(d)); br.add('m>=n,n<10000')  # shortcut: as above, no case
        else:
            x = d; br.add('m>=n,other')
        half = float(int(n / 2))
        d0 = fz * 0.5 + (half if half < x else x) * fn
        if half < x:
            br.add('m>=n,half')
    r8 = int(d0) + 1  # cvttsd2si, inc
    if fz > fn * (11.0 if mut == '10n' else 10.0):  # 0xe3950
        dd = float(r8) * 6.0 + fm * 30.0 * float(k); br.add('nnz>10n')
    elif m < n:
        dd = float(r8) * 6.0 + fm * 3.0 * fz * float(k) / fn
    else:
        dd = float(r8) * 6.0 + fm * 3.0 * float(m + nnz) * float(k) / float(m + 1 + n)
    assert c < 8, 'shortcut: c >= 8 takes the paired mulpd path (0xe3a0b), not modelled'
    for _ in range(c):
        dd *= 1.5
    if c:
        br.add('c>0')
    dd *= 0.8 if mut == '0.75' else 0.75  # 0xe3a7c, first pass
    assert dd < I64
    return int(dd) + 1


def load(f):
    return json.load(gzip.open(f))


def pool_cases(pool):
    out = []
    for f in sorted(glob.glob(os.path.join(pool, '*.json.gz'))):
        d = load(f)
        assert d['ok'] and d['binary_sha256'].startswith('8e87fca2') and d['kind'] == 'm1_pool_size', f
        e0 = d['events'][0]
        out.append((d['model'][:-7], e0['m'], e0['n'], e0['enter'][0], d['model']))
    return out


def nnz_of(model_file):
    import rsolver
    return rsolver.read(os.path.join(NETLIB, model_file)).NumNZs


def check_pool(pool, self_test):
    cases = pool_cases(pool)
    if not cases:
        print('no captures in', pool); return 1
    rows, bad, br = [], 0, set()
    for name, m, n, got, mf in cases:
        nnz = nnz_of(mf)
        want = factspace(m, n, nnz, br=br)
        rows.append((m, n, nnz, got))
        bad += want != got
        print(f"{name:10} m={m:5} n={n:5} nnz={nnz:6} orig={got:7} formula={want:7} {'ok' if want == got else 'DIFF'}")
    print(f'{len(cases) - bad}/{len(cases)} match (k=5, c=0); branches seen: {sorted(br)}')
    if self_test:
        assert factspace(27, 32, 83) == 1607  # afiro by hand: (181 + 1)·6 + 27·3·83·5/32 = 2142.47, ·0.75 → 1607
        for name, kw in {'k=4': dict(k=4), 'k=6': dict(k=6), 'c=1': dict(c=1), 'half': dict(mut='half'),
                         '10n': dict(mut='10n'), '0.75': dict(mut='0.75')}.items():
            moved = sum(factspace(m, n, z, **kw) != got for m, n, z, got in rows)
            print(f'mutant {name}: {moved} captures rejected')
            assert moved > 0, name
        print('SELF-TEST PASS')
    return 1 if bad else 0


def check_direct(d):
    files = sorted(glob.glob(os.path.join(d, '*_[123].json.gz')))
    if not files:
        print('no captures in', d); return 1
    bad, groups, br = 0, {}, set()
    for f in files:
        x = load(f)
        name = os.path.basename(f)[:-len('_1.json.gz')]
        assert x['binary_sha256'].startswith('8e87fca2') and x['kind'] == 'm1_factspace', f
        if not x['ok']:
            print(f, 'capture not ok'); bad += 1; continue
        ev = x['events']
        allocs = [e for e in ev if e['ev'] == 'alloc']
        if len(allocs) < 1:
            print(f, 'no 045a76 call'); bad += 1; continue
        nnz = nnz_of(x['model'])
        for i, e in enumerate(ev):
            if e['ev'] != 'alloc':
                continue
            want = factspace(e['m'], e['n'], e['nnz'], e['k'], e['c'], br=br)
            nxt = ev[i + 1] if i + 1 < len(ev) else None
            errs = []
            if e['quad'] != 0 or e['rc'] != 0: errs.append('quad/rc')
            if e['x158'] != want: errs.append(f"x158 {e['x158']} != formula {want}")
            if e['x168'] != e['x158'] - 1 or e['x160'] != 0: errs.append('x160/x168')
            if e['nnz'] != nnz: errs.append(f"spx nnz {e['nnz']} != rsolver {nnz}")
            if not nxt or nxt['ev'] != 'factor' or nxt['x158'] != e['x158']: errs.append('first 036ade F+0x158')
            bad += bool(errs)
            print(f"{os.path.basename(f):24} call {i}: ret {e['ret']:#x} m={e['m']} n={e['n']} nnz={e['nnz']} "
                  f"k={e['k']} c={e['c']} x158={e['x158']} {'ok' if not errs else 'FAIL ' + '; '.join(errs)}")
        groups.setdefault(name, []).append(json.dumps(ev, sort_keys=True))
    for name, evs in groups.items():
        same = len(evs) == 3 and len(set(evs)) == 1
        bad += not same
        print(f'{name}: {len(evs)} runs, events identical: {same}')
    print(f'branches seen: {sorted(br)}')
    print('DIRECT PASS' if not bad else f'DIRECT FAIL ({bad})')
    return 1 if bad else 0


def main(argv):
    if '--direct' in argv:
        return check_direct(argv[argv.index('--direct') + 1])
    pos = [a for a in argv if not a.startswith('--')]
    return check_pool(pos[0] if pos else 'bench/oracle/m1_02w/pool', '--self-test' in argv)


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
