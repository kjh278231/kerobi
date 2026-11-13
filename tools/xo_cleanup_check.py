#!/usr/bin/env python3
"""M3-04f-2: crossover cleanup 진입 오라클 검사기.

  python3 tools/xo_cleanup_check.py --controls      # 사례별 독립 3회 사건열 동일
  python3 tools/xo_cleanup_check.py --acceptance    # 13f716 선택 규칙·0cda5b 인자·0325b4 설치를 C 규칙으로 재계산
  python3 tools/xo_cleanup_check.py --self-test     # 자료 변조·규칙 변이가 거부되는지
  python3 tools/xo_cleanup_check.py --write-manifest

규칙 근거: 13f716 C 1363–1690·asm 0x1bf2ee–0x1bfc5c, 0cda5b C 178–191, 0325b4 C 전체(docs/m3/crossover_cleanup_entry.md §2).
"""
import argparse, collections, copy, gzip, hashlib, json, struct, sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
D = ROOT / 'bench/oracle/m3_xo_cleanup'
BIN = '8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c'
CROSSOVER = (-1, 1, 2, 3, 4)
SITES = {0x1bfa54: 'dual', 0x1bf7be: 'primal_nobasis', 0x1bf786: 'primal_basis'}
VOLATILE = ('pid', 'run_id')


def f(h):
    return struct.unpack('<d', bytes.fromhex(h))[0]


def h(v):
    return struct.pack('<d', v).hex()


class Fail(Exception):
    pass


def need(cond, ctx, msg):
    if not cond:
        raise Fail(f'{ctx}: {msg}')


def load(p):
    return json.loads(gzip.decompress(p.read_bytes()))


def runs():
    out = collections.defaultdict(dict)
    for c in CROSSOVER:
        for p in sorted((D / f'x{c}').glob('*.json.gz')):
            name, r = p.name[:-8].rsplit('-', 1)
            out[(c, name)][int(r)] = p
    for p in sorted((D / 'cases').glob('*.json.gz')):
        name, r = p.name[:-8].rsplit('-', 1)
        out[('case', name)][int(r)] = p
    return out


# ---- C 규칙 ------------------------------------------------------------------------------------------------

def dual_check(d, vstat, tol, V):
    """13f716 asm 0x1bfaf5–0x1bfbda: 기저 |d| 최댓값 + 비기저 쌍대 위반 최댓값(< tol이면 건너뜀) < tol."""
    bmax = nmax = 0.0
    for dj, s in zip(d, vstat):
        if s >= 0:
            a = abs(dj)
            bmax = a if a > bmax else bmax
        elif s == -3:
            continue
        else:
            if s < -3:
                v = 0.0
            else:
                v = 0.0 if 0.0 > -dj else -dj  # maxsd: dst(0) > src(-d)면 0
                if V == 'dual_upper_sign' and s == -2:
                    v = 0.0 if 0.0 > dj else dj
            if V == 'dual_no_tol_skip' or not (tol > v):
                nmax = v if v > nmax else nmax
    tot = nmax + bmax
    return tot <= tol if V == 'dual_le' else tot < tol


def decide(enter, reset, dual, V=None):
    """13f716 C 1523–1662: 반환 (bVar6, 쌍대 검사 경유 여부)."""
    c, retry, rem = enter['crossover'], reset['retry'], reset['remaining']
    if c == -1:
        if retry != 0:
            return False, False
        if rem > 0 and V != 'auto_pos_primal':
            return True, False
        s2d8, s120 = f(enter['s2d8']), f(enter['s120'])
        gate = s2d8 == 0.0 and s120 > 0.0
        if V == 'auto_no_s120':
            gate = s2d8 == 0.0
        if not gate:
            return False, False
        if enter['precision'] != 0:
            return False, False
        if dual is None:
            raise Fail('쌍대 검사 경로인데 sel:dual 없음')
        return dual_check([f(x) for x in dual['d']], dual['vstat'], f(enter['opttol_slot']), V), True
    if V == 'cross_mask':
        return not (c == 1 or rem < 0), False
    if V == 'neg_ignored':
        return (c & ~2) != 1, False
    return not ((c & ~2) == 1 or rem < 0), False


def reset(en, xe, V=None):
    """13f716 C 1363–1521(retry 잔존, env+0x2340 = 0): 행은 slack basis(위치 i), 열은 D bound로 −1/−2/−3/−4, remaining = 0."""
    n, m = en['n'], en['m']
    fx, feas, inf = f(xe['fixtol']), f(xe['feastol']), f(en['tol']['370'])
    fx = (feas if feas <= fx else fx) if 0.0 <= fx else 0.0
    out = []
    for j in range(n):
        lb, ub, x = f(en['d_lb'][j]), f(en['d_ub'][j]), f(en['x'][j])
        if not (fx < ub - lb):
            out.append(-4)
        elif -inf < lb or ub < inf:
            out.append(-2 if (lb + ub <= x + x) != (V == 'reset_mid') else -1)
        else:
            out.append(-3)
    return out + list(range(m))


def install(e, V=None):
    """0325b4(S+0x4d8 ≠ 0, 스케일 없음): 상태 복사 → sense flag → 열·행 비기저 재분류와 superbasic 목록."""
    n, m = e['n'], e['m']
    vstat = list(e['vb']) + list(e['cb'])
    tau, fix, opt = f(e['tol']['380']), f(e['tol']['3b0']), f(e['tol']['388'])
    lb, ub = [f(x) for x in e['lb']], [f(x) for x in e['ub']]
    x = None if e['x'] is None else [f(v) for v in e['x']]
    flags = e['d_flags']
    if e['d_flag3c'] == 1:
        for j in range(n):
            if flags[j] == '01' and vstat[j] == -2:
                vstat[j] = -1
        if x is not None:
            x = [-v if flags[j] == '01' else v for j, v in enumerate(x)]
    lst = []
    if x is not None:
        for j in range(n):
            if e['vb'][j] >= 0:
                continue
            if not (fix < ub[j] - lb[j]) and V != 'no_fix':
                vstat[j] = -4
            elif not (tau <= abs(x[j] - lb[j])):
                vstat[j] = -1
            elif not (tau <= abs(x[j] - ub[j])):
                vstat[j] = -2
            elif e['collen'][j] != 0 or abs(f(e['cost'][j])) > opt or V == 'no_cost_gate':
                vstat[j] = -3
                if 1e-13 < abs(x[j]) or V == 'no_tiny':
                    lst.append((j, x[j]))
    if e['slack'] is not None:
        for i in range(m):
            j = n + i
            if e['cb'][i] >= 0:
                continue
            s = f(e['slack'][i])
            if not (fix < ub[j] - lb[j]):
                vstat[j] = -4
            elif tau <= abs(s - lb[j]) and not (V == 'row_ub' and not (tau <= abs(s - ub[j]))):
                vstat[j] = -3
                lst.append((j, s))
            else:
                vstat[j] = -1
    return vstat, lst


# ---- 사례 검사 ----------------------------------------------------------------------------------------------

def check_run(d, ctx, V=None, stats=None):
    need(d['binary_sha256'] == BIN, ctx, '바이너리 해시')
    need(d['schema'] == 'M3-04f-2-v1' and d['reason'] == 'process-terminated' and not d['errors'], ctx, '캡처 상태')
    ev = d['events']
    need(ev and [e['seq'] for e in ev] == list(range(len(ev))), ctx, '사건 순번')
    names = [e['event'] for e in ev]
    need(names[0] == 'xo:enter' and names[-1] == 'xo:leave', ctx, '13f716 진입·반환 경계')
    need(names.count('xo:enter') == 1, ctx, '13f716 1회')
    xe = ev[0]
    need(xe['mode'] in (0, 1) and xe['crossover'] == d['params']['Crossover'], ctx, 'mode·Crossover')
    st = stats if stats is not None else collections.Counter()
    st['runs'] += 1
    st[f'mode {xe["mode"]}'] += 1
    if 'sel:enter' not in names:
        st['no_selection'] += 1
        need('cda5b:enter' not in names, ctx, '선택 없이 cleanup')
        return st
    i = names.index('sel:enter')
    need(names[i:i + 2] == ['sel:enter', 'sel:reset'], ctx, 'sel:reset 순서')
    en, rs = ev[i], ev[i + 1]
    j = i + 2
    dual = None
    if names[j] == 'sel:dual':
        dual = ev[j]
        j += 1
    need(names[j] == 'sel:decide', ctx, 'sel:decide 순서')
    dc = ev[j]
    # push 반환 소비: sel:enter의 retry·remaining = 마지막 push 출력
    pushes = [e for e in ev[:i] if e['event'] == 'push:leave']
    need(pushes, ctx, 'push 반환 없음')
    edits = {e['field']: e for e in ev[:i] if e['event'] == 'edit'}
    need(names[:i].count('edit') == len(edits) and (d['origin'] == 'synthetic-edit') == bool(edits), ctx, '편집 사건·origin')
    prim = [p for p in pushes if p['kind'] == 'primal']
    got_retry = edits['retry']['before'] if 'retry' in edits else en['retry']
    got_rem = edits['remaining']['before'] if 'remaining' in edits else en['remaining']
    need(got_retry == pushes[-1]['retry'], ctx, 'retry ≠ 마지막 push')
    need(got_rem == (prim[-1]['remaining'] if prim else -1), ctx, 'remaining ≠ 마지막 primal push(없으면 −1)')
    for fld, e in edits.items():
        need(e['after'] == en[fld], ctx, f'편집 {fld} 적용값')
    if edits:
        st['synthetic'] += 1
    need(en['status'] == 1, ctx, 'S+0xf0 ≠ 1')
    need(en['opttol_slot'] == xe['opttol'], ctx, 'OptimalityTol slot')
    # retry 잔존 재설정(C 1363–1521)
    if en['retry'] != 0:
        st['retry_reset'] += 1
        need(en['env_2340'] == 0, ctx, 'env+0x2340 ≠ 0 경로는 미캡처 분기')
        need(rs['remaining'] == 0 and rs['s184'] == 0, ctx, 'retry 재설정: remaining·S+0x184')
        exp = reset(en, xe, V)
        if rs['vstat'] != exp:
            t = next(t for t in range(len(exp)) if exp[t] != rs['vstat'][t])
            raise Fail(f'{ctx}: retry 재설정 vstat[{t}] 기대 {exp[t]} 실제 {rs["vstat"][t]}')
        need(rs['env_24a0'] == xe['perturb'], ctx, 'retry 재설정: env+0x24a0 = 진입 PerturbValue')
    else:
        need(rs['vstat'] == en['vstat'] and rs['remaining'] == en['remaining'] and rs['s184'] == en['s184'], ctx, '재설정 없음인데 상태 변화')
    want, via = decide(en, rs, dual, V)
    need(via == (dual is not None), ctx, f'쌍대 검사 경유 기대 {via}')
    need(bool(dc['r14']) == want, ctx, f'선택 기대 {want} 실제 r14={dc["r14"]}')
    rem = rs['remaining']
    key = ('auto' if en['crossover'] == -1 else f'x{en["crossover"]}', 'retry' if en['retry'] else ('rem+' if rem > 0 else 'rem0' if rem == 0 else 'rem-'),
           'dualchk' if via else '-', 'dual' if want else 'primal')
    st['branch ' + '/'.join(key)] += 1
    # 0cda5b 진입
    need(names[j + 1] == 'cda5b:enter', ctx, 'cleanup 호출 없음')
    ce = ev[j + 1]
    site = SITES.get(ce['ret_site'])
    exp_site = 'dual' if want else ('primal_nobasis' if rem == 0 else 'primal_basis')
    need(site == exp_site, ctx, f'호출 지점 {site} 기대 {exp_site}')
    need(ce['alg'] == (1 if want else 0), ctx, 'alg')
    basis = exp_site == 'primal_basis'
    need(ce['vb_null'] == ce['cb_null'] == ce['x_null'] == ce['sl_null'] == (not basis) and ce['p7_null'] and ce['p8_null'], ctx, '인자 null 형태')
    if basis:
        need(ce['vb_is_vstat'] and ce['cb_is_vstat_n'] and ce['sl_is_x_n'], ctx, '인자 별칭(S+0x168·+4n·x+8n)')
    need(ce['s180'] == 1, ctx, 'S+0x180 = 1')
    need(ce['s178'] == dc['s178'] + (10 if dc['local_98'] == 2 else 0), ctx, 'S+0x178 (+10 iff local_98 = 2)')
    k = j + 2
    if names[k] == 'i325b4:enter':
        need(basis, ctx, 'basis 없이 0325b4')
        ie, il = ev[k], ev[k + 1]
        need(names[k + 1] == 'i325b4:leave', ctx, '0325b4 반환')
        need(ie['vb_is_vstat'] and ie['cb_is_vstat_n'], ctx, '0325b4 별칭')
        need(ie['vstat'] == rs['vstat'] and ie['vb'] == rs['vstat'][:ie['n']] and ie['cb'] == rs['vstat'][ie['n']:], ctx, '설치 입력 상태 = 선택 시점 상태')
        if en['retry'] == 0:
            need(ie['x'] == en['x'], ctx, '설치 입력 x = 선택 시점 x(rbx)')
        need(ie['s4d8'] != 0 and ie['s28'] == 0, ctx, 'S+0x4d8 = 0 또는 x87 경로(미캡처 분기)')
        vs, lst = install(ie, V)
        need(il['ret'] == 0, ctx, '0325b4 반환')
        if il['vstat'] != vs:
            idx = next(t for t in range(len(vs)) if vs[t] != il['vstat'][t])
            raise Fail(f'{ctx}: 0325b4 vstat[{idx}] 기대 {vs[idx]} 실제 {il["vstat"][idx]}')
        need(il['s450'] == len(lst), ctx, f'S+0x450 기대 {len(lst)} 실제 {il["s450"]}')
        if lst:
            need(il['list'] == [t for t, _ in lst] and il['vals'] == [h(v) for _, v in lst], ctx, 'superbasic 목록·값')
        need(il['x'] == (None if ie['x'] is None else ie['x']) and il['slack'] == ie['slack'], ctx, 'x·slack 보존')
        st['install'] += 1
        st['install superbasic'] += len(lst)
        st['install changed'] += sum(a != b for a, b in zip(ie['vstat'], il['vstat']))
        st['install flag3c'] += ie['d_flag3c'] == 1
        k += 2
    else:
        need(not basis, ctx, 'basis 인자인데 0325b4 없음')
    need(names[k] == 'cda5b:leave', ctx, 'cda5b 반환 순서')
    st['cleanup ret=%d status=%d' % (ev[k]['ret'], ev[k]['model_status'])] += 1
    return st


def strip(d):
    return {k: v for k, v in d.items() if k not in VOLATILE}


def controls(rs):
    n = 0
    for key, by in sorted(rs.items()):
        need(sorted(by) == [1, 2, 3], key, f'회차 {sorted(by)}')
        base = load(by[1])
        for r in (2, 3):
            o = load(by[r])
            need(o['events'] == base['events'], f'{key} run{r}', '사건열 run1과 다름')
            need(o['result'] == base['result'], f'{key} run{r}', '최종 basis run1과 다름')
        n += 1
    return n


def acceptance(rs, V=None, verbose=True):
    st = collections.Counter()
    for key, by in sorted(rs.items()):
        for r, p in sorted(by.items()):
            check_run(load(p), f'{key} run{r}', V, st)
    if verbose:
        for k in sorted(st):
            print(f'  {k}: {st[k]}')
    return st


VARIANTS = ['reset_mid', 'auto_pos_primal', 'auto_no_s120', 'cross_mask', 'neg_ignored', 'dual_le', 'dual_no_tol_skip', 'dual_upper_sign',
            'no_fix', 'no_cost_gate', 'no_tiny', 'row_ub']


def self_test(rs):
    one = {}
    for key, by in sorted(rs.items()):
        d = load(by[1])
        names = [e['event'] for e in d['events']]
        for tag in ('i325b4:enter', 'sel:dual', 'cda5b:enter'):
            if tag in names and tag not in one:
                one[tag] = (key, d)
    rejected = survived = 0
    muts = []

    def mut(name, key_tag, fn):
        if key_tag not in one:
            print('  변조 생략(사례 없음):', name)
            return
        key, d = one[key_tag]
        x = copy.deepcopy(d)
        fn(x)
        muts.append((name, key, x))

    def ev(x, tag):
        return next(e for e in x['events'] if e['event'] == tag)
    mut('바이너리 해시 0', 'cda5b:enter', lambda x: x.__setitem__('binary_sha256', '0' * 64))
    mut('사건 비우기', 'cda5b:enter', lambda x: x.__setitem__('events', []))
    mut('선택 r14 반전', 'cda5b:enter', lambda x: ev(x, 'sel:decide').__setitem__('r14', 1 - ev(x, 'sel:decide')['r14']))
    mut('alg 반전', 'cda5b:enter', lambda x: ev(x, 'cda5b:enter').__setitem__('alg', 1 - ev(x, 'cda5b:enter')['alg']))
    mut('remaining 부호', 'cda5b:enter', lambda x: [ev(x, t).__setitem__('remaining', -ev(x, t)['remaining'] - 1) for t in ('sel:enter', 'sel:reset')])
    mut('push retry 변경', 'cda5b:enter', lambda x: ev(x, 'push:leave').__setitem__('retry', 1))
    mut('S+0x178 +1', 'cda5b:enter', lambda x: ev(x, 'cda5b:enter').__setitem__('s178', ev(x, 'cda5b:enter')['s178'] + 1))
    mut('호출 지점', 'cda5b:enter', lambda x: ev(x, 'cda5b:enter').__setitem__('ret_site', 0x1bfa54 if ev(x, 'cda5b:enter')['ret_site'] != 0x1bfa54 else 0x1bf7be))
    mut('0325b4 출력 vstat', 'i325b4:enter', lambda x: ev(x, 'i325b4:leave')['vstat'].__setitem__(0, -9))
    mut('0325b4 S+0x450', 'i325b4:enter', lambda x: ev(x, 'i325b4:leave').__setitem__('s450', ev(x, 'i325b4:leave')['s450'] + 1))
    mut('0325b4 입력 x 1비트', 'i325b4:enter', lambda x: ev(x, 'i325b4:enter')['x'].__setitem__(0, h(f(ev(x, 'i325b4:enter')['x'][0]) + 1.0)))
    mut('0325b4 사건 삭제', 'i325b4:enter', lambda x: x.__setitem__('events', [e for e in x['events'] if not e['event'].startswith('i325b4')]))
    mut('쌍대 d 크게', 'sel:dual', lambda x: ev(x, 'sel:dual')['d'].__setitem__(0, h(1.0)))
    for name, key, x in muts:
        try:
            check_run(x, f'{key} [{name}]')
            print('  생존(변조):', name)
            survived += 1
        except (Fail, KeyError, StopIteration, IndexError, TypeError):
            rejected += 1
    print(f'  자료 변조 {len(muts)}종 중 거부 {rejected}, 생존 {survived}')
    vrej = []
    for V in VARIANTS:
        try:
            acceptance(rs, V, verbose=False)
            print('  생존(규칙 변이):', V)
        except Fail as e:
            vrej.append(V)
            print('  거부(규칙 변이):', V, '—', str(e)[:120])
    print(f'  규칙 변이 {len(VARIANTS)}종 중 거부 {len(vrej)}')
    return survived == 0


def write_manifest(rs):
    files = {}
    for key, by in sorted(rs.items()):
        for r, p in sorted(by.items()):
            files[str(p.relative_to(D))] = hashlib.sha256(p.read_bytes()).hexdigest()
    man = {'task': 'M3-04f-2', 'schema': 'M3-04f-2-v1', 'binary_sha256': BIN,
           'tools': {t: hashlib.sha256((ROOT / 'tools' / t).read_bytes()).hexdigest() for t in ('xo_cleanup_agent.js', 'xo_cleanup_capture.js', 'xo_cleanup_runs.py')},
           'params': {'Threads': 1, 'Seed': 0, 'OutputFlag': 0, 'Presolve': 0, 'Method': 2, 'LPWarmStart': 2, 'Crossover': list(CROSSOVER)},
           'models': 'tools/push_init_runs.py SURVEY45 (../re/bench/data/netlib)', 'files': files}
    (D / 'manifest.json').write_text(json.dumps(man, indent=1) + '\n')
    print('manifest', len(files))


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--controls', action='store_true')
    ap.add_argument('--acceptance', action='store_true')
    ap.add_argument('--self-test', action='store_true')
    ap.add_argument('--write-manifest', action='store_true')
    a = ap.parse_args()
    rs = runs()
    need(rs, 'D', '캡처 없음')
    ok = True
    try:
        if a.controls:
            print('controls: 사례', controls(rs), '× 3회 동일')
        if a.acceptance:
            st = acceptance(rs)
            print('acceptance: 실행', st['runs'], '통과')
        if a.self_test:
            ok = self_test(rs)
        if a.write_manifest:
            write_manifest(rs)
    except Fail as e:
        print('실패', e)
        return 1
    return 0 if ok else 1


if __name__ == '__main__':
    sys.exit(main())
