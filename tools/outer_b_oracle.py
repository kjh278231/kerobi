"""Reproduce outer-B's uncovered branches through direct native calls on synthetic MIP states.

PYTHONPATH=$PWD/python ../.venv/bin/python tools/outer_b_oracle.py --capture
uses gurun.sh serially; without --capture this checks the committed, compressed state pairs offline.
The direct hook runs after the initial activity calculation. Protected column flags are injected explicitly
because SOS/general constraint setup belongs to another reduction; both implementations receive the same state.
"""
import argparse
import gzip
import json
from pathlib import Path
import subprocess
import tempfile

from mip_dump_check import compare, to_parts
from rsolver import _core

ROOT = Path(__file__).resolve().parents[1]
FIX = ROOT / 'bench/oracle/mip_outer_b'


def cases():
    result = []

    def add(name, fn, lp, removed, params=(), flags=()):
        result.append(dict(name=name, fn=fn, lp=lp, removed=removed, params=list(params), flags=list(flags)))

    for a in (-1, 1):
        for ak in (-2, 2):
            lp = f'''Minimize
 obj: {-a} x + y
Subject To
 trigger: {a} x + {ak} k <= {2*a + ak/2}
 extra: {-a} x + y <= 10
Bounds
 1 <= x <= 3
 0 <= y <= 2
Binary
 k
End
'''
            add(f'dual_a{a}_k{ak}', '418542', lp, 1)
    add('dual_growth_reject', '418542', lp, 0, [('GURO_PAR_AGGMULTLIMIT', 1)])
    add('dual_two_candidates', '418542', '''Minimize
 obj: - x - y + z
Subject To
 trigger: x + y - k <= 0
 extra: - x - y + z <= 1
Bounds
 0 <= x <= 0.5
 0 <= y <= 0.5
 0 <= z <= 2
Binary
 k
End
''', 2)

    for sign in (-1, 1):
        for flag in (1, 4, 0):
            add(f'protected_sign{sign}_flag{flag}', '37f484', f'''Minimize
 obj: j + k
Subject To
 pair: j + {sign} k = 0
Bounds
 -10 <= j <= 10
 -3.4 <= k <= 5.6
General
 j
End
''', 1 if flag else 0, flags=[(1, flag)])

    # Columns are ordered by their objective appearance. The larger row prevents stages A/B from eliminating
    # any column but permits pairwise shifts, so a change here must go through stage C.
    configs = [
        ('continuous', 2, '', [8, 7, 6, 5], 2),
        ('continuous_swap', 3, '', [8, 2, 3, 5], 2),
        ('integers', 1, 'General\n p1 p2 n1 n2\n', [8, 7, 6, 5], 2),
        ('mixed_floor', 1, 'General\n p2 n2\n', [3.7, 7, 6, 5], 2),
        ('mixed_binary', 1, 'Binary\n p1\nGeneral\n n1\n', [1, 7, 6, 5], 2),
        ('integer_ratio_reject', 2, 'General\n p1 n1\n', [8, 7, 6, 5], 0),
        ('mixed_pair_reject', 1, 'General\n n2\n', [3.7, 7, 6, 5], 0),
        ('all_binary_reject', 1, 'Binary\n p1 p2 n1 n2\n', [1, 1, 1, 1], 0),
    ]
    for name, ratio, types, ub, removed in configs:
        lp = f'''Minimize
 obj: p1 + p2 + n1 + n2 + z
Subject To
 pair: p1 + {ratio} p2 - n1 - {ratio} n2 = 0
 cap: p1 + p2 + n1 + n2 + z <= 40
Bounds
''' + ''.join(f' 0 <= {v} <= {u}\n' for v, u in zip(('p1', 'p2', 'n1', 'n2'), ub)) + types + 'Binary\n z\nEnd\n'
        add('pair_' + name, '37f484', lp, removed)
    continuous = next(c['lp'] for c in result if c['name'] == 'pair_continuous')
    add('pair_growth_reject', '37f484', continuous, 0, [('GURO_PAR_AGGMULTLIMIT', 1)])
    add('pair_fill', '37f484', continuous.replace('Bounds', ' solo: p1 + p2 + z <= 50\nBounds'), 2)
    add('pair_cancel', '37f484', continuous.replace('Bounds', ' cancel: p1 + 2 p2 + n1 - 2 n2 <= 50\nBounds'), 2)
    dual = next(c['lp'] for c in result if c['name'] == 'dual_a1_k-2')
    dual = dual.replace('obj: -1 x + y', 'obj: -1 x + y' + ''.join(f' + d{i}' for i in range(100)))
    add('dual_test_only', '418542', dual, 0)
    add('dominated', '4030c9', '''Minimize
 obj: - 2 x - y + z
Subject To
 cover: - x - y <= -2
 bound: x + y <= 5
 other: z + x + y <= 12
Bounds
 0 <= x <= 10
 0 <= y <= 4
Binary
 z
End
''', 1)
    xs = [f'x{i}' for i in range(59)]
    row = ' + '.join(xs + ['z'])
    add('dominated_large', '4030c9', 'Minimize\n obj: - 2 x0 - ' + ' - '.join(xs[1:]) + ' + z\nSubject To\n cover: - ' + ' - '.join(xs + ['z']) + ' <= -2\n bound: ' + row + ' <= 5\nBounds\n' + ''.join(f' 0 <= {x} <= 10\n' for x in xs) + 'Binary\n z\nEnd\n', 59)
    return result


def check(case, directory=FIX):
    with gzip.open(directory / (case['name'] + '.json.gz'), 'rt') as f:
        enter, leave = json.load(f)
    before, want = enter['s'], leave['s']
    assert want['counters'][4] - before['counters'][4] == case['removed'], case['name']
    got = _core.mip_dump_run(case['fn'], to_parts(before), case['params'])
    diffs, count = compare(got, want, before['recs'])
    assert not count, f"{case['name']}: {count} differences: {diffs}"
    assert got['ret'] == leave['ret'], case['name']
    return got


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument('--capture', action='store_true')
    ap.add_argument('--directory', type=Path, default=FIX)
    ap.add_argument('--only', nargs='+')
    args = ap.parse_args()
    args.directory.mkdir(parents=True, exist_ok=True)
    for c in cases():
        if args.only and c['name'] not in args.only:
            continue
        if args.capture:
            src = args.directory / (c['name'] + '.lp')
            src.write_text(c['lp'])
            with tempfile.TemporaryDirectory() as d:
                out = Path(d) / 'state.json'
                cmd = [str(ROOT / 'tools/gurun.sh'), 'node', str(ROOT / 'tools/mip_state_dump.js'),
                       str(src), c['fn'], '1', str(out), '--direct', '--col-flags=' + json.dumps(c['flags'])]
                cmd.extend(f'{k}={v}' for k, v in c['params'])
                subprocess.run(cmd, check=True, capture_output=True, text=True)
                data = json.loads(out.read_text())
            (args.directory / (c['name'] + '.json.gz')).write_bytes(gzip.compress(json.dumps(data).encode(), mtime=0))
        check(c, args.directory)
        print(c['name'] + ': match', flush=True)


if __name__ == '__main__':
    main()
