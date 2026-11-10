#!/usr/bin/env python3
"""Compare saved read-only captures; this is not a solver parity oracle."""
import copy
import json
import sys
from pathlib import Path

BASE = Path('bench/oracle/mip_linear_child_trace')


def fail(case, field, expected, actual):
    raise ValueError(f'{case}: {field}: expected {expected!r}, actual {actual!r}; rule=exact')


def compare(case, a, b, path='$'):
    if type(a) is not type(b):
        fail(case, path, a, b)
    if isinstance(a, dict):
        if a.keys() != b.keys():
            fail(case, path + '.keys', sorted(a), sorted(b))
        for k in a:
            compare(case, a[k], b[k], path + '.' + k)
    elif isinstance(a, list):
        if len(a) != len(b):
            fail(case, path + '.length', len(a), len(b))
        for i, (x, y) in enumerate(zip(a, b)):
            compare(case, x, y, f'{path}[{i}]')
    elif a != b:
        fail(case, path, a, b)


def validate_child(j):
    ev = j['events']
    sol = [x for x in ev if x['event'] == 'solve:enter']
    if not sol:
        fail('fractional_branch', 'solve count', '>0', len(sol))
    for e in sol:
        for k in ('domain_lower', 'domain_upper'):
            v = e['node'][k]
            if v is None or len(v) != 3:
                fail('fractional_branch', f"call {e['call']} {k}.length", 3, None if v is None else len(v))
    root = next(e['node'] for e in sol if e['node']['root'])
    children = [e['node'] for e in sol if e['node']['parent'] == root['id'] and e['node']['depth'] == 1 and not e['node']['root'] and e['node']['parent_is_root']]
    if not children:
        fail('fractional_branch', 'real child', 'parent/root/depth matched', children)
    if not any((n['domain_lower'], n['domain_upper']) != (root['domain_lower'], root['domain_upper']) for n in children):
        fail('fractional_branch', 'child domain', 'changed bound', children)
    pairs = {}
    for e in ev:
        if ':enter' in e['event']:
            pairs[e['call']] = e['event'].split(':')[0]
        elif ':leave' in e['event']:
            expected = pairs.pop(e['call'], None)
            actual = e['event'].split(':')[0]
            if expected != actual:
                fail('fractional_branch', f"call {e['call']} pairing", expected, actual)
    if pairs:
        fail('fractional_branch', 'unmatched calls', {}, pairs)
    return len(sol), len(children)


def main():
    records = {}
    for case in ('fractional_branch', 'root_unbounded'):
        runs = [json.loads((BASE / f'{case}_{i}.json').read_text()) for i in range(1, 4)]
        for i in (1, 2):
            compare(f'{case} run {i + 1}', runs[0], runs[i])
        records[case] = runs[0]
        print(f'{case}: 3 captures match, {len(runs[0]["events"])} events/run')
    solves, children = validate_child(records['fractional_branch'])
    print(f'fractional_branch: {solves} solves, {children} root children with complete 3-column domains and changed bound')
    if '--self-test' in sys.argv:
        for field in ('length', 'bound', 'parent', 'order'):
            altered = copy.deepcopy(records['fractional_branch'])
            events = altered['events']
            child = next(e for e in events if e['event'] == 'solve:enter' and e['node']['parent_is_root'])
            if field == 'length':
                child['node']['domain_lower'].pop()
            elif field == 'bound':
                child['node']['domain_lower'][0] = 'ffffffffffffffff'
            elif field == 'parent':
                child['node']['parent'] = 999
            else:
                events[0], events[1] = events[1], events[0]
            try:
                compare('mutation ' + field, records['fractional_branch'], altered)
            except ValueError as e:
                print('rejected:', e)
            else:
                raise AssertionError('mutation accepted: ' + field)
    return 0


if __name__ == '__main__':
    try:
        sys.exit(main())
    except (ValueError, KeyError, StopIteration) as exc:
        print(exc, file=sys.stderr)
        sys.exit(1)
