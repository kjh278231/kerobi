"""M3-04c-21f: Rust 직렬 numeric backend(`crossover_lu`)를 21e 원본 push factor 연쇄와 비트 비교한다.

입력 평문(`bench/oracle/m3_push_lu_rust/input.txt`)에는 재분해 호출의 `lu:leave` factor(주입점: header·spike·cache·F1d8
제외)와 push 상태 입력(FTRAN/BTRAN 입력 벡터·인자, FT 이탈 위치·header, 재분해 진입 header·이탈 위치·vstat·036ade 반환
순열)만 넣는다. 재분해 사이의 factor·spike·cache·F1d8·bucket·scratch는 Rust가 스스로 들고 간다. 원본의 FTRAN/BTRAN/FT
진입·반환 factor와 출력은 이 비교기만 읽는다.
"""
import argparse
from collections import Counter
import copy
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import sys

import push_lu_check as oracle
from push_ftran_rust import first_diff, count_leaves
from push_supplier_check import Fail, equal, require

ROOT = Path(__file__).resolve().parents[1]
D = ROOT / 'bench/oracle/m3_push_lu_rust'
TRACE = ROOT / 'target/m3_push_lu_rust/trace.txt'
SOURCE = Path('crates/lp/src/crossover_lu.rs')
SCHEMA = 'M3-04c-21f-v1'
MODELS = ('arms', 'retry', 'interior', 'adlittle', 'scsd1', 'capri')


def sha(p):
    return hashlib.sha256(p.read_bytes()).hexdigest()


def ints(xs):
    return [str(len(xs))] + [str(int(x)) for x in xs]


def hexes(xs):
    return [str(len(xs))] + list(xs)


def opt(xs):
    return ['0'] if xs is None else ['1'] + ints(xs)


def inject(f, buffers):
    """재분해 lu:leave factor(header·spike·cache·F1d8 제외)와 F40/F48 → F28/F38 별칭(C:2572–2573)."""
    m = f['m']; s = f['scalars']; p = f['perm']; L = f['L']; R = f['Lrow']; U = f['U']
    out = [f['zero']]
    for k in ('x20', 'x28', 'x30', 'x38', 'x40', 'x48'):
        out += ints(p[k])
    out += [str(int(buffers['x40_is_x28'])), str(int(buffers['x48_is_x38']))]
    out += hexes(f['diag'])
    out += ints(L['beg']) + ints(L['rows']) + hexes(L['vals']) + [str(L['levels'])] + opt(L['lev']) + opt(L['lst'])
    equal(s['x140'], R['levels'], 'Lrow.levels')
    out += ints(R['beg']) + ints(R['cols']) + hexes(R['vals']) + [str(R['levels'])] + opt(R['lev']) + opt(R['lst']) + opt(R['bucket'])
    for k in ('col_start', 'col_end', 'col_bound', 'row_start', 'row_end'):
        out += ints(U[k])
    for j in range(m):
        out += ints(U['col'][j]['rows']) + hexes(U['col'][j]['vals'])
    for i in range(m):
        out += ints(U['row'][i]['cols']) + hexes(U['row'][i]['vals'])
    for k in ('xd8', 'xdc', 'xe4', 'xe8', 'x100', 'x104'):
        out.append(str(s[k]))
    out += [s['x108'], s['x110']]
    for k in ('x184', 'x188', 'x18c', 'x158', 'x160', 'x168'):
        out.append(str(s[k]))
    out += [s['x1f8'], str(s['pool_mod16'][0]), str(s['pool_mod16'][1]), str(s['x1e0'])]
    equal(s['xec'], len(f['prefix']['list']), 'xec')
    out += ints(f['prefix']['list']) + ints(f['prefix']['flag']) + ints(f['eta_oldest'])
    equal(s['x180'], len(f['etas']), 'x180')
    out.append(str(len(f['etas'])))
    for t in f['etas']:
        equal(t['idx_off'], t['val_off'], 'eta offsets')
        out += [str(t['r']), str(t['idx_off'])] + ints(t['idx']) + hexes(t['val'])
    return out


def vec(v):
    return [str(v['count'])] + ints(v['idx'] or []) + hexes(v['val'])


def lu_block(ev, i):
    """lu:enter 위치 i에서 lu:leave까지."""
    j = i
    while ev[j]['event'] != 'lu:leave':
        j += 1
    return ev[i:j + 1]


def program(name, r):
    """push 안 연산 열: (op, ev 번호, 입력 토큰, 원본 기대값)."""
    ev = r['events']; out = []; inside = False
    for i, e in enumerate(ev):
        k = e['event']
        if k == 'push:enter':
            inside = True
        elif k == 'push:leave':
            inside = False
        if not inside or k not in ('ftran:enter', 'btran:enter', 'lu:enter'):
            continue
        m = len(e['factor']['diag']) if 'factor' in e and e['factor'] else len(e['header'])
        if k == 'lu:enter':
            blk = lu_block(ev, i); lv = blk[-1]
            equal(0, lv['ret'], f'{name}#{i}.ret'); equal(0, e['x290'], f'{name}#{i}.x290')
            require(all(e['pricing_null']), f'{name}#{i}.pricing')
            if e['request'] != 0:
                ref = [x for x in blk if x['event'] == 'lu:refactor' and x['phase'] == 'leave']
                require(ref, f'{name}#{i}: 036ade 반환 없음')
                st = ref[-1]['state']; equal(0, ref[-1]['ret'], f'{name}#{i}.036ade'); equal([0, 0], ref[-1]['out'], f'{name}#{i}.036ade out')
                toks = ['R', name, str(i), str(m)] + inject(lv['factor'], lv['buffers']) + ints(e['header']) + [str(e['leave'])] + ints(e['vstat']) + ints(st['x28']) + ints(st['x38'])
                want = dict(header=lv['header'], leave=lv['leave'], vstat=lv['vstat'], spike_count=lv['spike_count'],
                            buffers={k: lv['buffers'][k] for k in ('spike_idx_is_old_header', 'header_is_old_spike_idx')},
                            factor=lv['factor'])
                out.append(('R', i, toks, want))
            else:
                rw = [x for x in blk if x['event'] == 'lu:roweta']
                require(len(rw) in (0, 2), f'{name}#{i}.roweta')
                toks = ['U', name, str(i), str(m), str(e['leave'])] + ints(e['header'])
                want = dict(enter=e['factor'], roweta=rw[0]['args']['row'] if rw else None,
                            roweta_cache=rw[1]['cache'] if rw else None, leave=lv['factor'])
                out.append(('U', i, toks, want))
        elif k == 'ftran:enter':
            a = e['args']
            equal(0, a['k5'], 'k5'); require(a['p6_null'] and a['vals_null'] and a['mark_null'], f'{name}#{i}.args')
            require(a['spike_null'] != a['spike_f1c8'], f'{name}#{i}.spike arg')
            j = i
            while ev[j]['event'] != 'ftran:leave':
                j += 1
            lv = ev[j]
            toks = ['F', name, str(i), str(m)] + vec(e['input']) + [str(a['save']), str(int(a['spike_f1c8']))]
            want = dict(enter=e['factor'], leave=lv['factor'], output=lv['output'], input_after=lv['input_after'], alias=lv['alias'],
                        supplier=e['site'] == 'supplier')
            out.append(('F', i, toks, want))
        else:
            a = e['args']
            require(a['factor_s470'], f'{name}#{i}.factor')
            j = i
            while ev[j]['event'] != 'btran:leave':
                j += 1
            lv = ev[j]
            cache = (not a['cache_null']) and a['cache_f1d0']
            require(cache or a['cache_null'], f'{name}#{i}.cache arg')
            toks = ['B', name, str(i), str(m)] + vec(e['input']) + [str(e['output_count']), str(int(cache)), e['x200']]
            want = dict(enter=e['factor'], leave=lv['factor'], output=lv['output'], input_after=lv['input_after'], alias=lv['alias'])
            out.append(('B', i, toks, want))
    return out


def cases():
    m = oracle.load(oracle.D / 'manifest.json')
    oracle.check_files(m)
    data = oracle.records()
    oracle.controls(data, m)
    out = []
    for name in MODELS:
        out += [(name,) + x for x in program(name, data[name][0])]
    return out, m


def write():
    cs, m = cases()
    D.mkdir(parents=True, exist_ok=True)
    head = f'{SCHEMA} {hashlib.sha256((ROOT / oracle.D / "manifest.json").read_bytes()).hexdigest()}'
    lines = [head] + [' '.join(c[3]) for c in cs]
    (D / 'input.txt').write_text('\n'.join(lines) + '\n')
    print(f'input: {len(cs)} 연산', dict(Counter(c[1] for c in cs)))


# ---------- 비교 ----------
def like(cap, rust, drop=()):
    """원본 스냅숏에 있는 키만 Rust에서 고른다(scalars 안쪽 포함)."""
    out = {}
    for k in cap:
        if k in drop:
            continue
        require(k in rust, f'Rust 스냅숏에 {k} 없음')
        if k == 'scalars':
            out[k] = {x: rust[k][x] for x in cap[k] if x in rust[k]}
            for x in cap[k]:
                require(x in rust[k], f'Rust scalars에 {x} 없음')
        else:
            out[k] = rust[k]
    return out


def strip(cap, drop=()):
    return {k: v for k, v in cap.items() if k not in drop}


UNKNOWN = Counter()


def mask(a, b, key):
    """공급 FTRAN input_after(21a 계약: 기록만): 원본 push 버퍼의 잔류값이라 Rust가 미지값으로 둔 칸은 세고 비교에서 뺀다."""
    a = copy.deepcopy(a)
    for f in ('idx', 'val'):
        if a.get(f) is None or b.get(f) is None:
            continue
        for k, (x, y) in enumerate(zip(a[f], b[f])):
            if y is None and x is not None:
                a[f][k] = None
                UNKNOWN[key] += 1
    return a


def pairs(op, want, got):
    """(이름, 원본, Rust) 비교 쌍."""
    H = ('header',)
    if op == 'R':
        return [('header', want['header'], got['header']), ('leave', want['leave'], got['leave']),
                ('vstat', want['vstat'], got['vstat']), ('spike_count', want['spike_count'], got['spike_count']),
                ('buffers', want['buffers'], got['buffers']),
                ('factor', strip(want['factor']), like(want['factor'], got['factor']))]
    if op in ('F', 'B'):
        return [('enter', strip(want['enter'], H), like(want['enter'], got['enter'], H)),
                ('leave', strip(want['leave'], H), like(want['leave'], got['leave'], H)),
                ('output', want['output'], got['output']),
                ('input_after', mask(want['input_after'], got['input_after'], 'supplier input_after') if want.get('supplier') else want['input_after'],
                 got['input_after']),
                ('alias', want['alias'], got['alias'])]
    return [('enter', strip(want['enter']), like(want['enter'], got['enter'])),
            ('roweta', want['roweta'], got['roweta']), ('roweta_cache', want['roweta_cache'], got['roweta_cache']),
            ('leave', strip(want['leave']), like(want['leave'], got['leave']))]


def read_trace(path, head):
    lines = path.read_text().splitlines()
    equal(head, lines[0], 'trace.header')
    out = []
    for ln in lines[1:]:
        name, ev, js = ln.split(' ', 2)
        out.append((name, int(ev), json.loads(js)))
    return out


def compare(trace, cs, head, verbose=True):
    got = read_trace(trace, head)
    equal(len(cs), len(got), 'trace 연산 수')
    fields = 0; branches = Counter(); report = Counter(); first = None; fails = []
    for c, (name, ev, g) in zip(cs, got):
        cname, op, i, toks, want = c
        equal((cname, i), (name, ev), 'trace 순서')
        if g.get('error') is not None:
            fails.append(f'{name}#{ev} {op}: Rust 오류 {g["error"]}' + (' (앞선 오류로 건너뜀)' if g.get('skipped') else ''))
            continue
        equal(op, g['op'], f'{name}#{ev}.op')
        for label, a, b in pairs(op, want, g):
            d = first_diff(a, b, f'{name}#{ev} {op}.{label}')
            if d:
                fails.append(d)
                break
            fields += count_leaves(a)
        if op in ('F', 'B'):
            for k, v in g['branches'].items():
                branches[f'{op}.{k}={v}'] += 1
        if op == 'U':
            for k, v in g['report'].items():
                report[f'{k}={v}'] += 1
    if verbose:
        print(f'비교 연산 {len(cs)}, 필드 {fields}, 실패 {len(fails)}')
        print('FT 분기', dict(sorted(report.items())))
        print('FTRAN/BTRAN 분기', dict(sorted(branches.items())))
        print('미지 칸(비교 제외)', dict(UNKNOWN))
        for f in fails[:40]:
            print('  ', f)
    return fields, fails, report, branches


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--write', action='store_true')
    ap.add_argument('--compare', action='store_true')
    ap.add_argument('--trace', default=str(TRACE))
    a = ap.parse_args()
    try:
        if a.write:
            write()
        if a.compare:
            cs, m = cases()
            head = (D / 'input.txt').read_text().splitlines()[0]
            fields, fails, _, _ = compare(Path(a.trace), cs, head)
            if fails:
                raise Fail(f'{len(fails)}개 연산 불일치: 첫 {fails[0]}')
            print('OK')
    except Fail as e:
        print('FAIL', e)
        sys.exit(1)


if __name__ == '__main__':
    main()
