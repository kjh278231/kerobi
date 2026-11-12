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


# ---------- 판별력 ----------
def flipbit(h):
    b = bytearray(bytes.fromhex(h)); b[0] ^= 1
    return b.hex()


def find(lines, op, pred=lambda g: True):
    for k, ln in enumerate(lines[1:], 1):
        name, ev, js = ln.split(' ', 2); g = json.loads(js)
        if g.get('op') == op and g.get('error') is None and pred(g):
            return k, name, ev, g
    raise Fail(f'변조 대상 없음 {op}')


def edit(lines, op, fn, pred=lambda g: True):
    k, name, ev, g = find(lines, op, pred)
    fn(g)
    out = list(lines); out[k] = f'{name} {ev} {json.dumps(g)}'
    return out


def last_eta(f):
    return f['etas'][-1]


TRACE_MUTATIONS = [
    ('trace: 연산 누락', lambda L: L[:5] + L[6:]),
    ('trace: 순서 교환', lambda L: L[:3] + [L[4], L[3]] + L[5:]),
    ('trace: 머리 해시', lambda L: ['M3-04c-21f-v1 ' + '0' * 64] + L[1:]),
    ('trace: 오류 주입', lambda L: edit(L, 'U', lambda g: g.update(error='주입'))),
    ('R: header', lambda L: edit(L, 'R', lambda g: g['header'].__setitem__(0, g['header'][0] + 1))),
    ('R: 이탈 위치', lambda L: edit(L, 'R', lambda g: g.update(leave=g['leave'] + 1), lambda g: g['leave'] >= 0)),
    ('R: vstat', lambda L: edit(L, 'R', lambda g: g['vstat'].__setitem__(g['header'][0], g['vstat'][g['header'][0]] + 1))),
    ('R: spike count', lambda L: edit(L, 'R', lambda g: g.update(spike_count=1))),
    ('R: 버퍼 교환', lambda L: edit(L, 'R', lambda g: g['buffers'].update(spike_idx_is_old_header=False))),
    ('R: cache count', lambda L: edit(L, 'R', lambda g: g['factor']['cache'].update(count=1))),
    ('R: F1d8', lambda L: edit(L, 'R', lambda g: g['factor'].update(x1d8=0))),
    ('U: 반환 대각 비트', lambda L: edit(L, 'U', lambda g: g['leave']['diag'].__setitem__(0, flipbit(g['leave']['diag'][0])))),
    ('U: x48 교환', lambda L: edit(L, 'U', lambda g: g['leave']['perm']['x48'].reverse())),
    ('U: x28(별칭)', lambda L: edit(L, 'U', lambda g: g['leave']['perm']['x28'].reverse())),
    ('U: U 열 시작', lambda L: edit(L, 'U', lambda g: g['leave']['U']['col_start'].__setitem__(-1, g['leave']['U']['col_start'][-1] + 1))),
    ('U: U 열 경계', lambda L: edit(L, 'U', lambda g: g['leave']['U']['col_bound'].__setitem__(-1, g['leave']['U']['col_bound'][-1] + 1))),
    ('U: U 행 시작', lambda L: edit(L, 'U', lambda g: g['leave']['U']['row_start'].__setitem__(0, g['leave']['U']['row_start'][0] + 1))),
    ('U: U 열 값 비트', lambda L: edit(L, 'U', lambda g: next(c for c in g['leave']['U']['col'] if c['vals'])['vals'].__setitem__(0, flipbit(next(c for c in g['leave']['U']['col'] if c['vals'])['vals'][0])))),
    ('U: U 행 순서', lambda L: edit(L, 'U', lambda g: next(r for r in g['leave']['U']['row'] if len(r['cols']) > 1)['cols'].reverse(),
                                    lambda g: any(len(r['cols']) > 1 for r in g['leave']['U']['row']))),
    ('U: eta offset', lambda L: edit(L, 'U', lambda g: last_eta(g['leave']).update(idx_off=last_eta(g['leave'])['idx_off'] + 1), lambda g: g['leave']['etas'])),
    ('U: eta 값 비트', lambda L: edit(L, 'U', lambda g: last_eta(g['leave'])['val'].__setitem__(-1, flipbit(last_eta(g['leave'])['val'][-1])), lambda g: g['leave']['etas'])),
    ('U: eta 정렬', lambda L: edit(L, 'U', lambda g: last_eta(g['leave']).update(val_mod16=8 - last_eta(g['leave'])['val_mod16']), lambda g: g['leave']['etas'])),
    ('U: F1a0', lambda L: edit(L, 'U', lambda g: g['leave']['eta_oldest'].__setitem__(0, 7))),
    ('U: x160 풀 끝', lambda L: edit(L, 'U', lambda g: g['leave']['scalars'].update(x160=g['leave']['scalars']['x160'] + 1))),
    ('U: x100 t', lambda L: edit(L, 'U', lambda g: g['leave']['scalars'].update(x100=g['leave']['scalars']['x100'] + 1))),
    ('U: xe8', lambda L: edit(L, 'U', lambda g: g['leave']['scalars'].update(xe8=g['leave']['scalars']['xe8'] + 1))),
    ('U: x18c', lambda L: edit(L, 'U', lambda g: g['leave']['scalars'].update(x18c=g['leave']['scalars']['x18c'] + 1))),
    ('U: x184', lambda L: edit(L, 'U', lambda g: g['leave']['scalars'].update(x184=g['leave']['scalars']['x184'] + 1))),
    ('U: F+0xf0 목록', lambda L: edit(L, 'U', lambda g: g['leave']['prefix']['list'].append(0))),
    ('U: F+0xf8', lambda L: edit(L, 'U', lambda g: g['leave']['prefix']['flag'].__setitem__(0, -3))),
    ('U: spike 잔류', lambda L: edit(L, 'U', lambda g: g['leave']['spike'].update(count=1, indices=[0], values=['0' * 16]))),
    ('U: 행 eta 누락', lambda L: edit(L, 'U', lambda g: g.update(roweta=None, roweta_cache=None), lambda g: g['roweta'] is not None)),
    ('U: 행 eta 추가', lambda L: edit(L, 'U', lambda g: g.update(roweta=0), lambda g: g['roweta'] is None)),
    ('U: 행 eta cache 값', lambda L: edit(L, 'U', lambda g: g['roweta_cache']['val'].__setitem__(0, flipbit(g['roweta_cache']['val'][0])),
                                         lambda g: g['roweta_cache'] and g['roweta_cache']['count'] > 0)),
    ('U: 진입 spike 값', lambda L: edit(L, 'U', lambda g: g['enter']['spike']['values'].__setitem__(0, flipbit(g['enter']['spike']['values'][0])))),
    ('F: 출력 값 비트', lambda L: edit(L, 'F', lambda g: g['output']['val'].__setitem__(0, flipbit(g['output']['val'][0])), lambda g: g['output']['val'] and g['output']['val'][0])),
    ('F: 별칭', lambda L: edit(L, 'F', lambda g: g['alias']['in'].update(idx='out' if g['alias']['in']['idx'] == 'in' else 'in'))),
    ('F: 반환 spike 값', lambda L: edit(L, 'F', lambda g: g['leave']['spike']['values'].__setitem__(0, flipbit(g['leave']['spike']['values'][0])), lambda g: g['leave']['spike']['count'] > 0)),
    ('F: wrapper clear 전 spike', lambda L: edit(L, 'F', lambda g: g['enter']['spike'].update(count=1, indices=[0], values=['0' * 16]))),
    ('F: scratch 잔류', lambda L: edit(L, 'F', lambda g: g['leave']['scratch']['x1b0'].append([0, 1]))),
    ('F: 재계산 입력 후상태', lambda L: edit(L, 'F', lambda g: g['input_after']['val'].__setitem__(0, flipbit(g['input_after']['val'][0])), lambda g: g['input_after']['val'] and g['input_after']['val'][0] and g['enter']['spike']['count'] == 0 and g['input_after']['count'] < 0)),
    ('B: cache 값', lambda L: edit(L, 'B', lambda g: g['leave']['cache']['val'].__setitem__(0, flipbit(g['leave']['cache']['val'][0])), lambda g: g['leave']['cache']['count'] > 0)),
    ('B: F1d8', lambda L: edit(L, 'B', lambda g: g['leave'].update(x1d8=g['leave']['x1d8'] + 1))),
    ('B: level bucket', lambda L: edit(L, 'B', lambda g: g['leave']['Lrow']['bucket'].__setitem__(0, g['leave']['Lrow']['bucket'][0] + 1), lambda g: g['leave']['Lrow']['bucket'])),
    ('B: 출력 목록', lambda L: edit(L, 'B', lambda g: g['output']['idx'].__setitem__(0, g['output']['idx'][0] + 1))),
]


def self_test(cs, head):
    base = TRACE.read_text().splitlines()
    tmp = ROOT / 'target/m3_push_lu_rust/mutated.txt'
    rejected = []
    for name, fn in TRACE_MUTATIONS:
        tmp.write_text('\n'.join(fn(base)) + '\n')
        try:
            _, fails, _, _ = compare(tmp, cs, head, verbose=False)
            require(fails, f'변조 통과: {name}')
            rejected.append((name, fails[0]))
        except Fail as e:
            if str(e).startswith('변조 통과'):
                raise
            rejected.append((name, str(e)))
    for name, why in rejected:
        print(f'  거부 {name}: {why[:140]}')
    print(f'trace 변조 {len(rejected)}종 모두 거부')
    return rejected


MUTATIONS = [
    # 계약이 반드시 거부를 요구하는 두 변이
    ('ft-skipped(매번 재분해 대체)', 'let mut rep = FtReport::default();\n        let ri = r as i32;',
     'let mut rep = FtReport::default();\n        if r < usize::MAX {\n            self.spike.count = 0;\n            self.exit();\n            return Ok(rep);\n        }\n        let ri = r as i32;'),
    ('roweta-always(무조건 행 eta)', '        if i35 != i52 {\n            if ri != self.x1d8 {', '        if true {\n            if ri != self.x1d8 {'),
    ('roweta-never', '        if i35 != i52 {\n            if ri != self.x1d8 {', '        if false {\n            if ri != self.x1d8 {'),
    ('local40-first', 'if q < i52 || i52 == tm1 { tm1 - (np != 0) as i32 } else { tm1 }', 'if q < i52 || i52 == tm1 { tm1 } else { tm1 }'),
    ('local40-last', 'if i52 < i35 && i52 == tm1 { t - 2 + (np == 0) as i32 } else { tm1 }', 'if i52 < i35 && i52 == tm1 { t - 2 } else { tm1 }'),
    ('promote-never', 'if found && i52 < i17 {', 'if false {'),
    ('local80-3-0', '                if tm1 < q {\n                    0\n                } else {\n                    3\n                }',
     '                if tm1 < q {\n                    3\n                } else {\n                    0\n                }'),
    ('local80-1-2', '(tm1 < q) as i32 + 1', '(tm1 >= q) as i32 + 1'),
    ('cycle-strict', 'if (ll - base) as i64 <= len {', 'if ((ll - base) as i64) < len {'),
    ('rotate-none', '            self.x48[i35 as usize] = k0;\n            self.x40[k0 as usize] = i35;', '            let _ = k0;'),
    ('pool-extra', 'let extra = (((len as i32) / 2).max(4)).min(room) as i64;', 'let extra = (((len as i32) / 2).max(2)).min(room) as i64;'),
    ('row-cap', 'self.rc[iu] = ns + len + extra;', 'self.rc[iu] = ns + len;'),
    ('column-base', 'self.cs[r] = self.sc.x160 - self.colbase;', 'self.cs[r] = self.sc.x160;'),
    ('tail-prefix', 'if local40 < self.upos(i) {\n                self.set_col(head, i, v);', 'if local40 <= self.upos(i) {\n                self.set_col(head, i, v);'),
    ('oldest-overwrite', 'self.oldest[i] = self.oldest[i].min(ne);', 'self.oldest[i] = ne;'),
    ('dot-sequential', 'let d = eta_dot(&terms, false);', 'let d = terms.iter().fold(0.0, |a, &t| a + t);'),
    ('dot-peel', 'let d = eta_dot(&terms, false);', 'let d = eta_dot(&terms, true);'),
    ('unlist-mark', 'self.unlist(ri, -2);', 'self.unlist(ri, -1);'),
    ('xe8-never', '                    self.sc.xe8 = self.sc.xe8.max(self.upos(col as i32));', '                    let _ = col;'),
    ('renumber-n', 'let new: Vec<i32> = (0..m).map(|p| header[o[p] as usize]).collect();', 'let new: Vec<i32> = (0..m).map(|p| header[n[p] as usize]).collect();'),
    ('leave-o', 'let leave = if leave >= 0 { n[leave as usize] } else { leave };', 'let leave = if leave >= 0 { o[leave as usize] } else { leave };'),
    ('vstat-none', '            vstat[new[p] as usize] = p as i32;', '            let _ = p;'),
    ('wrapper-clear-none', '    pub fn wrapper_clear(&mut self) {\n        for k', '    pub fn wrapper_clear(&mut self) {\n        #[allow(unreachable_code)]\n        return;\n        for k'),
    ('bucket-not-shared', '        s.bucket = self.bucket.clone();\n', '        s.bucket = self.bucket.clone().map(|b| vec![0; b.len()]);\n'),
]
MUT_SOURCE = {'bucket-not-shared': SOURCE}


def mutate_rust(path, cs, head):
    scratch = ROOT / 'target/m3_lu_mutations'
    if scratch.exists():
        shutil.rmtree(scratch)
    scratch.mkdir(parents=True)
    for file in ('Cargo.toml', 'Cargo.lock'):
        shutil.copy2(ROOT / file, scratch / file)
    shutil.copytree(ROOT / 'crates', scratch / 'crates')
    (scratch / 'bench/oracle/m3_push_lu_rust').mkdir(parents=True)
    shutil.copy2(D / 'input.txt', scratch / 'bench/oracle/m3_push_lu_rust/input.txt')
    original = (ROOT / SOURCE).read_text(); results = []
    trace = scratch / 'target/m3_push_lu_rust/trace.txt'
    for name, old, new in MUTATIONS:
        equal(1, original.count(old), 'unique mutation ' + name)
        (scratch / SOURCE).write_text(original.replace(old, new, 1)); trace.unlink(missing_ok=True)
        try:
            proc = subprocess.run(['cargo', 'test', '-p', 'rsolver-lp', '--test', 'push_lu'], cwd=scratch,
                                  text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, timeout=240)
        except subprocess.TimeoutExpired:
            # 재생이 끝나지 않음(예: 순환 탈출 조건 변이의 무한 루프): trace가 없어 비교기가 받지 않는다
            subprocess.run(['pkill', '-f', 'm3_lu_mutations/target/debug/deps/push_lu-'])
            results.append(dict(name=name, result='replay-timeout'))
            print(f'  {name}: replay-timeout', flush=True); continue
        if proc.returncode or not trace.exists():
            results.append(dict(name=name, result='build-or-test-failed', log=proc.stdout[-1500:])); continue
        _, fails, _, _ = compare(trace, cs, head, verbose=False)
        results.append(dict(name=name, result='rejected' if fails else 'survived', first=fails[0] if fails else None))
        print(f'  {name}: {results[-1]["result"]} {(fails[0] if fails else "")[:120]}', flush=True)
    (scratch / SOURCE).write_text(original)
    path.write_text(json.dumps(results, ensure_ascii=False, indent=2) + '\n')
    return dict(Counter(r['result'] for r in results))


def manifest():
    files = [D / 'input.txt', ROOT / SOURCE, ROOT / 'crates/lp/tests/push_lu.rs', ROOT / 'crates/lp/src/crossover_ftran.rs',
             ROOT / 'crates/lp/src/crossover_btran.rs', Path(__file__).resolve(), ROOT / oracle.D / 'manifest.json']
    return dict(task='M3-04c-21f', schema=SCHEMA, binary_sha256=oracle.BINARY_SHA,
                files={str(p.relative_to(ROOT)): sha(p) for p in files})


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--write', action='store_true')
    ap.add_argument('--compare', action='store_true')
    ap.add_argument('--trace', default=str(TRACE))
    ap.add_argument('--self-test', action='store_true')
    ap.add_argument('--mutate-rust', metavar='결과경로')
    ap.add_argument('--manifest', action='store_true')
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
        if a.self_test or a.mutate_rust:
            cs, m = cases()
            head = (D / 'input.txt').read_text().splitlines()[0]
            if a.self_test:
                self_test(cs, head)
            if a.mutate_rust:
                print('Rust 변이', mutate_rust(Path(a.mutate_rust), cs, head))
        if a.manifest:
            (D / 'manifest.json').write_text(json.dumps(manifest(), ensure_ascii=False, indent=1) + '\n')
            print('manifest 갱신')
    except Fail as e:
        print('FAIL', e)
        sys.exit(1)


if __name__ == '__main__':
    main()
