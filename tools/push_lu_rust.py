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
GENERATE = False


def use_generate():
    """21g 생성 모드: 재분해 factor를 주입하지 않고 Rust가 만든다(별도 입력·trace)."""
    global D, TRACE, SCHEMA, GENERATE
    D = ROOT / 'bench/oracle/m3_push_refactor_rust'
    TRACE = ROOT / 'target/m3_push_refactor_rust/trace.txt'
    SCHEMA = 'M3-04c-21g-v1'
    GENERATE = True


BLOCK = False


def use_block():
    """21h: 생성 모드 + `1da0b2` 2×2 블록 단계. 첫 push 재분해의 S+0x2c0/0x4e0/0x4e4만 원본(K 연산)에서 넣고 이후는 Rust 연쇄."""
    global D, TRACE, SCHEMA, BLOCK
    use_generate()
    D = ROOT / 'bench/oracle/m3_push_block_rust'
    TRACE = ROOT / 'target/m3_push_block_rust/trace.txt'
    SCHEMA = 'M3-04c-21h-v1'
    BLOCK = True


def block_rows(name):
    """21h 블록 상태 캡처(정식 *-1)의 push 재분해 표: lu id → (진입, 반환). 검사기 controls·acceptance를 먼저 통과해야 한다."""
    import push_block_check as pbc
    global BLOCK_CHECKED
    if not BLOCK_CHECKED:
        caps, base = pbc.load_all()
        try:
            pbc.controls(caps, base)
            pbc.acceptance(caps)
        except pbc.Fail as e:
            raise Fail(f'블록 상태 캡처 검사 실패: {e}')
        BLOCK_CHECKED = True
    rows, outside = pbc.table(pbc.load(pbc.path_of(pbc.D, name, 1)), name)
    ids = [r['block'] for r in rows]
    require(not [o for o in outside if ids[0] < o['block'] < ids[-1]], f'{name}: push 재분해 사이에 push 밖 036ade')
    return {r['lu']: r for r in rows}


BLOCK_CHECKED = False
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


def matrix(e):
    """push 진입 CSC(S+0x64/0x68/0x70…0x88): 21g 생성 모드의 분해 입력."""
    M = e['model']; st = e['state']; m, n = st['m'], st['n']
    starts = [int.from_bytes(bytes.fromhex(x), 'little', signed=True) for x in M['starts']]
    nnz = max((s + l for s, l in zip(starts, M['lengths'])), default=0)
    return [str(m), str(n)] + ints(starts) + ints(M['lengths']) + ints(M['rows'][:nnz]) + hexes(M['values'][:nnz])


def program(name, r, generate=False):
    """push 안 연산 열: (op, ev 번호, 입력 토큰, 원본 기대값). generate면 재분해 factor를 주입하지 않고 Rust가 만든다(21g)."""
    ev = r['events']; out = []; inside = False; markowitz = None; first = True
    brows = block_rows(name) if BLOCK else None; bfirst = True
    for i, e in enumerate(ev):
        k = e['event']
        if k == 'push:enter':
            inside = True
            if generate:
                out.append(('M', i, ['M', name, str(i), str(e['state']['m'])] + matrix(e), None))
        elif k == 'push:leave':
            inside = False
        if k == 'factor:enter':
            markowitz = e['factor_ctl']['state']['markowitz']
        if not inside or k not in ('ftran:enter', 'btran:enter', 'lu:enter'):
            continue
        was_first = first; first = False
        m = len(e['factor']['diag']) if 'factor' in e and e['factor'] else len(e['header'])
        if k == 'lu:enter':
            blk = lu_block(ev, i); lv = blk[-1]
            equal(0, lv['ret'], f'{name}#{i}.ret'); equal(0, e['x290'], f'{name}#{i}.x290')
            require(all(e['pricing_null']), f'{name}#{i}.pricing')
            if e['request'] != 0:
                ref = [x for x in blk if x['event'] == 'lu:refactor' and x['phase'] == 'leave']
                require(ref, f'{name}#{i}: 036ade 반환 없음')
                st = ref[-1]['state']; equal(0, ref[-1]['ret'], f'{name}#{i}.036ade'); equal([0, 0], ref[-1]['out'], f'{name}#{i}.036ade out')
                want = dict(header=lv['header'], leave=lv['leave'], vstat=lv['vstat'], spike_count=lv['spike_count'],
                            buffers={k: lv['buffers'][k] for k in ('spike_idx_is_old_header', 'header_is_old_spike_idx')},
                            factor=lv['factor'])
                if generate:
                    require(markowitz is not None, f'{name}#{i}: Markowitz 없음')
                    equal(ref[0]['state']['header'], e['header'], f'{name}#{i}.036ade header')
                    if was_first:
                        # push 첫 factor 호출: 할당 상태 4필드(풀 크기·정렬·갱신 한도·work 배율)만 원본에서 읽는다. 첫 lu:enter
                        # factor는 캡처되지 않아(21e) 첫 재분해 lu:leave에서 읽는다: 036ade는 풀 부족 재시도(C:2592–2620,
                        # 036ade 2회 이상)에서만 F+0x158을 바꾸므로 036ade 1회인 호출에서는 호출 전 값과 같다.
                        equal(1, sum(1 for x in blk if x['event'] == 'lu:refactor' and x['phase'] == 'leave'), f'{name}#{i}: 036ade 재시도')
                        s0 = (e['factor'] or lv['factor'])['scalars']
                        alloc = ['1', str(s0['x158']), str(s0['pool_mod16'][0]), str(s0['pool_mod16'][1]), str(s0['x188']), s0['x1f8']]
                    else:
                        alloc = ['0']
                    toks = ['G', name, str(i), str(m), (e['factor'] or lv['factor'])['zero'] if was_first else '-', markowitz] + alloc + ints(e['header']) + [str(e['leave'])] + ints(e['vstat'])
                    want['perm'] = {k: st[k] for k in ('x20', 'x28', 'x30', 'x38')}
                    if BLOCK:
                        row = brows.get(ref[-1]['id'])
                        require(row is not None, f'{name}#{i}: 블록 상태 캡처에 lu {ref[-1]["id"]} 없음')
                        want['block'] = {'enter': row['enter'], 'leave': row['leave']}
                        if bfirst:
                            # push 첫 재분해의 036ade 진입 상태만 원본에서 넣는다(이후 0x2c0 +1, 0x4e0/0x4e4는 Rust 반환값)
                            b = row['enter']
                            out.append(('K', i, ['K', name, str(i), str(m), str(b['x2c0']), str(b['x4e0']), str(b['x4e4'])], None))
                            bfirst = False
                    out.append(('G', i, toks, want))
                else:
                    toks = ['R', name, str(i), str(m)] + inject(lv['factor'], lv['buffers']) + ints(e['header']) + [str(e['leave'])] + ints(e['vstat']) + ints(st['x28']) + ints(st['x38'])
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
        out += [(name,) + x for x in program(name, data[name][0], GENERATE)]
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


def unknown_bucket(want, got, key):
    """재분해 뒤 F+0x1a8은 036ade scratch 잔류값: Rust가 미지값(null)으로 둔 칸은 세고 비교에서 뺀다(읽히면 출력이 달라져 드러난다)."""
    if not isinstance(want, dict) or not isinstance(got, dict):
        return want
    wb = (want.get('Lrow') or {}).get('bucket'); gb = (got.get('Lrow') or {}).get('bucket')
    if wb is None or gb is None:
        return want
    want = copy.deepcopy(want)
    for k, (x, y) in enumerate(zip(wb, gb)):
        if y is None:
            want['Lrow']['bucket'][k] = None
            UNKNOWN[key] += 1
    return want


def pairs(op, want, got):
    """(이름, 원본, Rust) 비교 쌍."""
    H = ('header',)
    if op in ('M', 'K'):
        return []
    if GENERATE and op != 'G':
        want = dict(want)
        for k in ('enter', 'leave'):
            if k in want and k in got:
                want[k] = unknown_bucket(want[k], got[k], f'{op} bucket')
    if op == 'G':
        return ([('block', want['block'], got.get('block'))] if BLOCK else []) + [('perm', want['perm'], got['perm'])] + pairs('R', dict(want, factor=unknown_bucket(want['factor'], got['factor'], 'G bucket')), got)
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


# 21g 계약 §1에 사전 명시한 예상 실패: capri 23회째 재분해(ev 813)는 `1da0b2` 2×2 블록 단계(M1 미구현)로 피벗한다.
# 그 재분해와 같은 push의 이후 연산은 실패로 세고 통과 수에 넣지 않는다(후속 M3-04c-21h).
EXPECTED = {'capri': 813}


def fail_key(f):
    name, rest = f.split('#', 1)
    return name, int(rest.split(' ', 1)[0])


def expected_set():
    """21h(블록 단계 이식)에서는 예상 실패가 없다."""
    return {} if BLOCK else EXPECTED


def expected(f):
    name, ev = fail_key(f)
    exp = expected_set()
    return GENERATE and name in exp and ev >= exp[name]


def compare(trace, cs, head, verbose=True):
    got = read_trace(trace, head)
    equal(len(cs), len(got), 'trace 연산 수')
    fields = 0; branches = Counter(); report = Counter(); fails = []; ops = Counter(); passed = Counter(); gen = Counter()
    for c, (name, ev, g) in zip(cs, got):
        cname, op, i, toks, want = c
        equal((cname, i), (name, ev), 'trace 순서')
        ops[op] += 1
        if g.get('error') is not None:
            fails.append(f'{name}#{ev} {op}: Rust 오류 {g["error"]}' + (' (앞선 오류로 건너뜀)' if g.get('skipped') else ''))
            continue
        equal(op, g['op'], f'{name}#{ev}.op')
        ok = True
        for label, a, b in pairs(op, want, g):
            d = first_diff(a, b, f'{name}#{ev} {op}.{label}')
            if d:
                fails.append(d); ok = False
                break
            fields += count_leaves(a)
        if not ok:
            continue
        passed[op] += 1
        if op == 'G':
            sc = want['factor']['scalars']; m = want['factor']['m']
            gen['kernel' if sc['xe4'] < m else 'triangular'] += 1
            gen['L levels' if sc['x128'] >= 0 else 'no levels'] += 1
        if op in ('F', 'B'):
            for k, v in g['branches'].items():
                branches[f'{op}.{k}={v}'] += 1
        if op == 'U':
            for k, v in g['report'].items():
                report[f'{k}={v}'] += 1
    if verbose:
        print(f'비교 연산 {sum(ops.values())} {dict(ops)}, 통과 {dict(passed)}, 필드 {fields}, 실패 {len(fails)}')
        if GENERATE:
            print('생성 재분해 통과 분류', dict(gen))
            print('예상 실패(2×2 블록, 계약 §1)', [f for f in fails if expected(f)])
        print('FT 분기', dict(sorted(report.items())))
        print('FTRAN/BTRAN 분기', dict(sorted(branches.items())))
        print('미지 칸(비교 제외)', dict(UNKNOWN))
        for f in [f for f in fails if not expected(f)][:40]:
            print('  ', f)
    return fields, fails, report, branches


def judge(fails):
    """통과 판정: 예상 실패 밖의 실패가 없고, 생성 모드면 예상 실패가 실제로 그 재분해에서 시작해야 한다."""
    bad = [f for f in fails if not expected(f)]
    if bad:
        raise Fail(f'{len(bad)}개 연산 불일치: 첫 {bad[0]}')
    if GENERATE:
        for name, ev in expected_set().items():
            first = [f for f in fails if fail_key(f)[0] == name]
            require(first and fail_key(first[0])[1] == ev and ' G.' in first[0], f'예상 실패가 {name}#{ev} 재분해에서 시작하지 않음: {first[:1]}')


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


def kernel_g(g):
    """커널(Markowitz)이 있고 L·U가 비어 있지 않은 생성 재분해."""
    f = g['factor']; return f['scalars']['xe4'] < f['m'] and f['L']['rows'] and any(len(r['cols']) > 1 for r in f['U']['row'])


def level_g(g):
    return g['factor']['scalars']['x128'] >= 0


def first_multi(rows, key):
    return next(r for r in rows if len(r[key]) > 1)


def gf(g):
    return g['factor']


def swap_in_column(beg, idx):
    """한 열(beg 구간) 안의 서로 다른 첫 두 원소를 맞바꾼다."""
    for k in range(len(beg) - 1):
        a, b = beg[k] - beg[0], beg[k + 1] - beg[0]
        if b - a >= 2 and idx[a] != idx[a + 1]:
            idx[a], idx[a + 1] = idx[a + 1], idx[a]
            return
    raise Fail('맞바꿀 열 없음')


def bump(d, k, by=1):
    d[k] = d[k] + by


GEN_TRACE_MUTATIONS = [
    ('trace: 연산 누락', lambda L: L[:5] + L[6:]),
    ('trace: 머리 해시', lambda L: ['M3-04c-21g-v1 ' + '0' * 64] + L[1:]),
    ('trace: G 오류 주입', lambda L: edit(L, 'G', lambda g: g.update(error='주입'))),
    ('G: 036ade 피벗 행 x20', lambda L: edit(L, 'G', lambda g: g['perm']['x20'].reverse(), kernel_g)),
    ('G: 036ade 옛 위치 o', lambda L: edit(L, 'G', lambda g: g['perm']['x28'].reverse(), kernel_g)),
    ('G: 대각 비트', lambda L: edit(L, 'G', lambda g: gf(g)['diag'].__setitem__(-1, flipbit(gf(g)['diag'][-1])), kernel_g)),
    ('G: L 열 안 순서', lambda L: edit(L, 'G', lambda g: swap_in_column(gf(g)['L']['beg'], gf(g)['L']['rows']), kernel_g)),
    ('G: L 값 비트', lambda L: edit(L, 'G', lambda g: gf(g)['L']['vals'].__setitem__(0, flipbit(gf(g)['L']['vals'][0])), kernel_g)),
    ('G: L 열 시작', lambda L: edit(L, 'G', lambda g: gf(g)['L']['beg'].__setitem__(1, gf(g)['L']['beg'][1] + 1), kernel_g)),
    ('G: L 레벨', lambda L: edit(L, 'G', lambda g: gf(g)['L']['lev'].__setitem__(0, gf(g)['L']['lev'][0] + 1), level_g)),
    ('G: L lst 마지막 칸', lambda L: edit(L, 'G', lambda g: gf(g)['L']['lst'].__setitem__(-1, gf(g)['L']['lst'][-1] + 1), level_g)),
    ('G: Lᵀ 행 안 순서', lambda L: edit(L, 'G', lambda g: swap_in_column(gf(g)['Lrow']['beg'], gf(g)['Lrow']['cols']), lambda g: kernel_g(g) and any(b - a >= 2 for a, b in zip(gf(g)['Lrow']['beg'], gf(g)['Lrow']['beg'][1:])))),
    ('G: Lᵀ 레벨', lambda L: edit(L, 'G', lambda g: gf(g)['Lrow']['lev'].__setitem__(0, gf(g)['Lrow']['lev'][0] + 1), level_g)),
    ('G: Lᵀ lst', lambda L: edit(L, 'G', lambda g: gf(g)['Lrow']['lst'].__setitem__(-1, gf(g)['Lrow']['lst'][-1] - 1), level_g)),
    ('G: U 열 행 순서', lambda L: edit(L, 'G', lambda g: first_multi(gf(g)['U']['col'], 'rows')['rows'].reverse(), kernel_g)),
    ('G: U 열 경계', lambda L: edit(L, 'G', lambda g: gf(g)['U']['col_bound'].__setitem__(-1, gf(g)['U']['col_bound'][-1] + 1), kernel_g)),
    ('G: U 행 순서', lambda L: edit(L, 'G', lambda g: first_multi(gf(g)['U']['row'], 'cols')['cols'].reverse(), kernel_g)),
    ('G: U 행 시작', lambda L: edit(L, 'G', lambda g: gf(g)['U']['row_start'].__setitem__(-1, gf(g)['U']['row_start'][-1] + 1), kernel_g)),
    ('G: xd8', lambda L: edit(L, 'G', lambda g: bump(gf(g)['scalars'], 'xd8'), kernel_g)),
    ('G: xdc', lambda L: edit(L, 'G', lambda g: bump(gf(g)['scalars'], 'xdc'), kernel_g)),
    ('G: xe4', lambda L: edit(L, 'G', lambda g: bump(gf(g)['scalars'], 'xe4'), kernel_g)),
    ('G: xe8', lambda L: edit(L, 'G', lambda g: bump(gf(g)['scalars'], 'xe8'), kernel_g)),
    ('G: x100 t', lambda L: edit(L, 'G', lambda g: bump(gf(g)['scalars'], 'x100'), kernel_g)),
    ('G: x104', lambda L: edit(L, 'G', lambda g: bump(gf(g)['scalars'], 'x104'), kernel_g)),
    ('G: x108 비트', lambda L: edit(L, 'G', lambda g: gf(g)['scalars'].update(x108=flipbit(gf(g)['scalars']['x108'])), kernel_g)),
    ('G: x110 비트', lambda L: edit(L, 'G', lambda g: gf(g)['scalars'].update(x110=flipbit(gf(g)['scalars']['x110'])), kernel_g)),
    ('G: x128', lambda L: edit(L, 'G', lambda g: bump(gf(g)['scalars'], 'x128'), level_g)),
    ('G: x140', lambda L: edit(L, 'G', lambda g: bump(gf(g)['scalars'], 'x140'), level_g)),
    ('G: x160 풀 끝', lambda L: edit(L, 'G', lambda g: bump(gf(g)['scalars'], 'x160', 2), kernel_g)),
    ('G: x168', lambda L: edit(L, 'G', lambda g: bump(gf(g)['scalars'], 'x168'), kernel_g)),
    ('G: x1e0', lambda L: edit(L, 'G', lambda g: bump(gf(g)['scalars'], 'x1e0'), kernel_g)),
    ('G: x188', lambda L: edit(L, 'G', lambda g: bump(gf(g)['scalars'], 'x188'), kernel_g)),
    ('G: 접두 목록', lambda L: edit(L, 'G', lambda g: gf(g)['prefix']['list'].append(0), kernel_g)),
    ('G: 접두 자리', lambda L: edit(L, 'G', lambda g: gf(g)['prefix']['flag'].__setitem__(-1, -1), kernel_g)),
    ('G: F1a0', lambda L: edit(L, 'G', lambda g: gf(g)['eta_oldest'].__setitem__(0, 0), kernel_g)),
    ('G: header 재번호', lambda L: edit(L, 'G', lambda g: g['header'].reverse(), kernel_g)),
    ('G: 이탈 위치', lambda L: edit(L, 'G', lambda g: g.update(leave=g['leave'] + 1), lambda g: g['leave'] >= 0)),
    ('G: vstat', lambda L: edit(L, 'G', lambda g: g['vstat'].__setitem__(g['header'][0], g['vstat'][g['header'][0]] + 1))),
    ('G: x40 별칭 identity', lambda L: edit(L, 'G', lambda g: gf(g)['perm']['x40'].reverse(), kernel_g)),
    ('F: 출력 값 비트', lambda L: edit(L, 'F', lambda g: g['output']['val'].__setitem__(0, flipbit(g['output']['val'][0])), lambda g: g['output']['val'] and g['output']['val'][0])),
    ('U: 반환 대각 비트', lambda L: edit(L, 'U', lambda g: g['leave']['diag'].__setitem__(0, flipbit(g['leave']['diag'][0])))),
    ('B: cache 값', lambda L: edit(L, 'B', lambda g: g['leave']['cache']['val'].__setitem__(0, flipbit(g['leave']['cache']['val'][0])), lambda g: g['leave']['cache']['count'] > 0)),
]


def self_test(cs, head):
    if GENERATE:
        return self_test_generate(cs, head)
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


def self_test_generate(cs, head):
    """21g: 기준 trace의 실패는 예상 실패뿐이다. 변조는 판정(judge)을 실패시켜야 거부다."""
    base = TRACE.read_text().splitlines()
    _, fails, _, _ = compare(TRACE, cs, head, verbose=False)
    judge(fails)
    tmp = TRACE.parent / 'mutated.txt'
    rejected = []
    for name, fn in GEN_TRACE_MUTATIONS + (BLOCK_TRACE_MUTATIONS if BLOCK else []):
        tmp.write_text('\n'.join(fn(base)) + '\n')
        try:
            _, f2, _, _ = compare(tmp, cs, head, verbose=False)
            judge(f2)
        except Fail as e:
            rejected.append((name, str(e)))
            continue
        raise Fail(f'변조 통과: {name}')
    for name, why in rejected:
        print(f'  거부 {name}: {why[:150]}')
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
GEN_SOURCE = Path('crates/lp/src/crossover_refactor.rs')
GEN_MUTATIONS = [
    # 계약 §3: 재분해를 건너뛰고 직전 factor 유지(반드시 거부)
    ('refactor-skipped(직전 factor 유지)', '    let (m, n) = (a.m, a.n);\n',
     '    let (m, n) = (a.m, a.n);\n    if let Some(p) = prev {\n        let id: Vec<i32> = (0..m as i32).collect();\n        return Ok(Refactor { state: p.clone(), o: id.clone(), n: id });\n    }\n'),
    ('markowitz-default', 'Lu::factor_block(&spx, &head, tol, block)', 'Lu::factor_block(&spx, &head, crate::lu::MARKOWITZ_TOL, block)'),
    ('L-order-reversed', 'for &(i, l) in &p.lcol[k] {', 'for &(i, l) in p.lcol[k].iter().rev() {'),
    ('L-rows-not-steps', 'lrows.push(step(i));', 'lrows.push(i as i32);'),
    ('Lt-order-reversed', 'for &(j, l) in &p.lrow[k] {', 'for &(j, l) in p.lrow[k].iter().rev() {'),
    ('Urow-order-reversed', 'for &(j, u) in &p.urow[k] {', 'for &(j, u) in p.urow[k].iter().rev() {'),
    ('Ucol-order-reversed', 'for &(i, u) in &p.ucol[k] {', 'for &(i, u) in p.ucol[k].iter().rev() {'),
    ('pool-fill-U-only', 'let x160 = 2 * (lnnz + unnz) as i64;', 'let x160 = 2 * unnz as i64;'),
    ('column-base-after-rows', 'let colbase = x160 / 2;', 'let colbase = unnz as i64;'),
    ('t-M1-prefix', 'let (t, dc) = (p.s3, p.dc);', 'let (t, dc) = (p.prefix, p.dc);'),
    ('tail-bound-strict', 'p.prow[e.0] >= t as i64', 'p.prow[e.0] > t as i64'),
    ('xe8-nonempty', 'while e8 > t && ce[e8] != cs[e8] {', 'while e8 > t && ce[e8] == cs[e8] {'),
    ('f8-dc-inclusive', 'if k < dc { -1 }', 'if k <= dc { -1 }'),
    ('x108-x110-swapped', 'x108: mean(t, m),\n        x110: mean(dc, t),', 'x108: mean(dc, t),\n        x110: mean(t, m),'),
    ('mean-empty-1', '} else { 1.5 };', '} else { 1.0 };'),
    ('x1e0-no-diag', 'x1e0: (m + lnnz) as i64,', 'x1e0: lnnz as i64,'),
    ('xd8-slack-end', 'xd8: p.s1 as i32,', 'xd8: p.dc as i32,'),
    ('xe4-phase3', 'xe4: p.s4 as i32,', 'xe4: p.s3 as i32,'),
    ('lst-last-summed', 'for l in 1..=levels {', 'for l in 1..=levels + 1 {'),
    ('Lt-lst-kept', '            lst.pop();\n', ''),
    ('F1a0-zero', 'oldest: vec![2_000_000_000; m],', 'oldest: vec![0; m],'),
    ('spike-unknown', '                    val.set(i, 0.0);\n', ''),
    ('o-inverse', 'let o: Vec<i32> = p.steps.iter().map(|s| s.1 as i32).collect();', 'let o: Vec<i32> = p.steps.iter().map(|s| s.0 as i32).collect();'),
]


# 21h: `1da0b2` 2×2 블록 단계(lu.rs block_phase) 소스 변이(임시 Cargo 사본). 계약 §3: 블록 단계 끔은 반드시 거부.
BLOCK_SOURCE = Path('crates/lp/src/lu.rs')
BLOCK_MUTATIONS = [
    # 반드시 거부: 블록 단계를 건너뜀(S+0x4e0 증가는 그대로라 거부는 factor에서 나야 한다)
    ('block-phase-skipped', '        st.x4e0 += 1;\n', '        st.x4e0 += 1;\n        return;\n'),
    ('x4e0-not-counted', '        st.x4e0 += 1;\n', ''),
    ('run-threshold-50-65', 'st.x4e4 < 2_000_000_000 && active < 50', 'st.x4e4 < 2_000_000_000 && active < 65'),
    ('run-threshold-50-49', 'st.x4e4 < 2_000_000_000 && active < 50', 'st.x4e4 < 2_000_000_000 && active < 49'),
    ('table-size-2active', 'let prime = next_prime((active * 2) as i32) as i64;', 'let prime = active * 2;'),
    ('A-skipped', '        while let Some((i0, i1)) = h.pop() {\n            let (r6, r10)', '        h.pairs.clear();\n        while let Some((i0, i1)) = h.pop() {\n            let (r6, r10)'),
    ('A-pivot-row-reversed', 'if v39.abs() < v42.abs() {', 'if v39.abs() > v42.abs() {'),
    ('A-accept-0', '            let s = v43 - (v42 * v38) / v40;\n            if s.abs() < 1e-10 {', '            let s = v43 - (v42 * v38) / v40;\n            if s.abs() < 0.0 {'),
    ('A-value-swap', 'rval[w] = rval[j];', 'rval.swap(w, j);'),
    ('B-skipped', '        while let Some((i0, i1)) = h.pop() {\n            let (ca, cb, key)', '        h.pairs.clear();\n        while let Some((i0, i1)) = h.pop() {\n            let (ca, cb, key)'),
    ('B-pivot-col-reversed', 'if va.abs() < vb.abs() {', 'if va.abs() > vb.abs() {'),
    ('B-key-order-reversed', '{ (second, first) } else { (first, second) }', '{ (first, second) } else { (second, first) }'),
    ('B-accept-0', '            let s = bo - (vo * bp) / vp;\n            if s.abs() < 1e-10 {', '            let s = bo - (vo * bp) / vp;\n            if s.abs() < 0.0 {'),
    ('B-accept-huge', '            let s = bo - (vo * bp) / vp;\n            if s.abs() < 1e-10 {', '            let s = bo - (vo * bp) / vp;\n            if s.abs() < 1e10 {'),
    ('B-U-merge-reversed', 'for (c, x) in uk1 {', 'for (c, x) in uk1.into_iter().rev() {'),
    ('B-U-no-drop', 'if ZERO < (x + w[c]).abs() {', 'if true {'),
    ('B-other-col-last-dropped', '            uk.push((oth, vo));\n', '            uk.push((oth, 0.0));\n'),
    ('B-col-singletons-skipped', '            // column singletons (C:1384–1493)\n            while let Some(c) = stack.pop() {', '            // column singletons (C:1384–1493)\n            stack.clear();\n            while let Some(c) = stack.pop() {'),
    ('pairs-fifo', 'let b = self.pairs.pop()?;\n                let a = self.pairs.pop()?;', 'if self.pairs.is_empty() { return None; }\n                let a = self.pairs.remove(0);\n                let b = self.pairs.remove(0);'),
    ('update-threshold-20-19', 'if self.steps.len() as i64 - s4 as i64 > 19 {', 'if self.steps.len() as i64 - s4 as i64 > 18 {'),
    ('settle-reversed', 'st.x4e4 = if st.x4e4 * 2 < st.x4e0 { -1 } else { 2_000_000_000 };', 'st.x4e4 = if st.x4e4 * 2 < st.x4e0 { 2_000_000_000 } else { -1 };'),
    ('xe4-after-blocks', '        lu.n_singleton = ns;\n        // 036ade C:543', '        lu.n_singleton = ns;\n        lu.s4 = ns;\n        // 036ade C:543'),
    ('block-L-generic', '            lu.lcol[s4 + b] = l.clone();\n', ''),
]


def blk_g(g):
    """블록 단계가 피벗한 재분해(capri 23회째: F+0xe4 = 블록 전 단계 수 140)."""
    return g.get('block') and g['block']['leave']['x4e0'] > g['block']['enter']['x4e0'] and gf(g)['scalars']['xe4'] == 140


def ran_g(g):
    return g.get('block') and g['block']['leave']['x4e0'] > g['block']['enter']['x4e0']


BLOCK_TRACE_MUTATIONS = [
    ('K: 블록 상태 연산 누락', lambda L: [x for x in L if '"op": "K"' not in x and '"op":"K"' not in x]),
    ('G: 블록 진입 x2c0', lambda L: edit(L, 'G', lambda g: g['block']['enter'].update(x2c0=g['block']['enter']['x2c0'] + 1), ran_g)),
    ('G: 블록 진입 x4e0', lambda L: edit(L, 'G', lambda g: g['block']['enter'].update(x4e0=g['block']['enter']['x4e0'] + 1), ran_g)),
    ('G: 블록 진입 x4e4', lambda L: edit(L, 'G', lambda g: g['block']['enter'].update(x4e4=-1), ran_g)),
    ('G: 블록 반환 x4e0', lambda L: edit(L, 'G', lambda g: g['block']['leave'].update(x4e0=g['block']['enter']['x4e0']), ran_g)),
    ('G: 블록 반환 x4e4', lambda L: edit(L, 'G', lambda g: g['block']['leave'].update(x4e4=1), ran_g)),
    ('G: 블록 없음', lambda L: edit(L, 'G', lambda g: g.pop('block'), ran_g)),
    ('G(블록): F+0xe4', lambda L: edit(L, 'G', lambda g: bump(gf(g)['scalars'], 'xe4', 3), blk_g)),
    ('G(블록): 피벗 행 x20', lambda L: edit(L, 'G', lambda g: g['perm']['x20'].__setitem__(140, 217), blk_g)),
    ('G(블록): Schur 대각 비트', lambda L: edit(L, 'G', lambda g: gf(g)['diag'].__setitem__(141, flipbit(gf(g)['diag'][141])), blk_g)),
    ('G(블록): 두 번째 U 행 순서', lambda L: edit(L, 'G', lambda g: gf(g)['U']['row'][141]['cols'].reverse(), blk_g)),
    ('G(블록): 첫 L 열 값', lambda L: edit(L, 'G', lambda g: gf(g)['L']['vals'].__setitem__(gf(g)['L']['beg'][140], flipbit(gf(g)['L']['vals'][gf(g)['L']['beg'][140]])), blk_g)),
]


def mutate_rust(path, cs, head):
    scratch = ROOT / 'target/m3_lu_mutations'
    if scratch.exists():
        shutil.rmtree(scratch)
    scratch.mkdir(parents=True)
    for file in ('Cargo.toml', 'Cargo.lock'):
        shutil.copy2(ROOT / file, scratch / file)
    shutil.copytree(ROOT / 'crates', scratch / 'crates')
    for sub in ('m3_push_lu_rust', 'm3_push_refactor_rust', 'm3_push_block_rust'):
        if (ROOT / 'bench/oracle' / sub / 'input.txt').exists():
            (scratch / 'bench/oracle' / sub).mkdir(parents=True)
            shutil.copy2(ROOT / 'bench/oracle' / sub / 'input.txt', scratch / 'bench/oracle' / sub / 'input.txt')
    source, mutations = (BLOCK_SOURCE, BLOCK_MUTATIONS) if BLOCK else (GEN_SOURCE, GEN_MUTATIONS) if GENERATE else (SOURCE, MUTATIONS)
    test = 'block_chain_trace' if BLOCK else 'refactor_chain_trace' if GENERATE else 'lu_chain_trace'
    original = (ROOT / source).read_text(); results = []
    trace = scratch / TRACE.relative_to(ROOT)
    for name, old, new in mutations:
        equal(1, original.count(old), 'unique mutation ' + name)
        (scratch / source).write_text(original.replace(old, new, 1)); trace.unlink(missing_ok=True)
        try:
            proc = subprocess.run(['cargo', 'test', '-p', 'rsolver-lp', '--test', 'push_lu', test], cwd=scratch,
                                  text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, timeout=240)
        except subprocess.TimeoutExpired:
            # 재생이 끝나지 않음(예: 순환 탈출 조건 변이의 무한 루프): trace가 없어 비교기가 받지 않는다
            subprocess.run(['pkill', '-f', 'm3_lu_mutations/target/debug/deps/push_lu-'])
            results.append(dict(name=name, result='replay-timeout'))
            print(f'  {name}: replay-timeout', flush=True); continue
        if proc.returncode or not trace.exists():
            results.append(dict(name=name, result='build-or-test-failed', log=proc.stdout[-1500:]))
            print(f'  {name}: build-or-test-failed', flush=True); continue
        _, fails, _, _ = compare(trace, cs, head, verbose=False)
        try:
            judge(fails); first = None
        except Fail as e:
            first = str(e)
        results.append(dict(name=name, result='rejected' if first else 'survived', first=first))
        print(f'  {name}: {results[-1]["result"]} {(first or "")[:140]}', flush=True)
    (scratch / source).write_text(original)
    path.write_text(json.dumps(results, ensure_ascii=False, indent=2) + '\n')
    return dict(Counter(r['result'] for r in results))


def manifest():
    files = [D / 'input.txt', ROOT / SOURCE, ROOT / 'crates/lp/tests/push_lu.rs', ROOT / 'crates/lp/src/crossover_ftran.rs',
             ROOT / 'crates/lp/src/crossover_btran.rs', Path(__file__).resolve(), ROOT / oracle.D / 'manifest.json']
    if GENERATE:
        files += [ROOT / GEN_SOURCE, ROOT / 'crates/lp/src/lu.rs']
    if BLOCK:
        files += [ROOT / 'bench/oracle/m3_push_block/manifest.json', ROOT / 'tools/push_block_check.py']
    return dict(task='M3-04c-21h' if BLOCK else 'M3-04c-21g' if GENERATE else 'M3-04c-21f', schema=SCHEMA, binary_sha256=oracle.BINARY_SHA,
                files={str(p.relative_to(ROOT)): sha(p) for p in files})


def main():
    if '--block' in sys.argv:
        use_block()
    elif '--generate' in sys.argv:
        use_generate()
    ap = argparse.ArgumentParser()
    ap.add_argument('--generate', action='store_true', help='21g: 재분해 factor를 Rust가 생성(주입 없음)')
    ap.add_argument('--block', action='store_true', help='21h: 생성 모드 + 2×2 블록 단계(블록 상태 연쇄)')
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
            judge(fails)
            print('OK' + (f' (예상 실패 {len(fails)}연산 제외)' if fails else ''))
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
