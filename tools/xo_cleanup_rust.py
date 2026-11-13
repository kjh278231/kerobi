#!/usr/bin/env python3
"""M3-04f-3: Rust cleanup 진입(crossover_cleanup)을 04f-2 원본 캡처 768회와 비교한다.

  python3 tools/xo_cleanup_rust.py --write           # 검사기(--controls --acceptance) 통과 뒤 입력 작성
  cargo test --release -p rsolver-lp --test xo_cleanup   # Rust 재생 → target/m3_xo_cleanup_rust/trace.txt
  python3 tools/xo_cleanup_rust.py --compare         # 사례마다 reset·select·install 줄 비교
  python3 tools/xo_cleanup_rust.py --self-test       # 기대·출력 줄 변조가 모두 거부되는지
  python3 tools/xo_cleanup_rust.py --mutate-rust OUT # Rust 규칙 변이(임시 복사본)마다 재생·비교
  python3 tools/xo_cleanup_rust.py --coverage        # 생존 변이 분기의 자료 출현 수

입력은 sel:enter(push 반환·Crossover·S+0x2d8·S+0x120·S+0x28·OptTol·local_98·S+0x178·S+0x184·vstat·x),
xo:enter(FIXTOL·FeasTol·PerturbValue)와 D bound, sel:dual d(주입; 251b4f 미이식), i325b4:enter의 S 배열(lb·ub·cost·
열 길이·τ·FIXTOL·OptTol·sense flag·slack). 설치 입력 상태와 x는 Rust 자신의 재설정·선택 결과와 sel:enter x다.
계약: docs/m3/crossover_cleanup_rust.md.
"""
import argparse, hashlib, json, os, shutil, subprocess, sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import xo_cleanup_check as xc  # noqa: E402

ROOT = str(xc.ROOT)
D = os.path.join(ROOT, 'bench/oracle/m3_xo_cleanup_rust')
INPUT = os.path.join(D, 'input.txt')
TRACE = os.path.join(ROOT, 'target/m3_xo_cleanup_rust/trace.txt')
SCHEMA = 'M3-04f-3-v1'
SITE = {0x1bfa54: 'dual', 0x1bf7be: 'primal_nobasis', 0x1bf786: 'primal_basis'}


def require(c, msg):
    if not c:
        raise SystemExit('실패: ' + msg)


def head():
    return f'{SCHEMA} {hashlib.sha256((xc.D / "manifest.json").read_bytes()).hexdigest()}'


def j(v):
    return ' '.join(map(str, v))


def ordered():
    rs = xc.runs()
    out = []
    for key, by in sorted(rs.items(), key=lambda t: (str(t[0][0]), t[0][1])):
        for r, p in sorted(by.items()):
            out.append((f'{key[0]}/{key[1]}/{r}', p))
    return out


def project(doc):
    """(입력 줄 목록, 기대 줄 목록)."""
    ev = doc['events']
    by = {}
    for e in ev:
        by.setdefault(e['event'], e)
    xe, en, rs, dc, ce = by['xo:enter'], by['sel:enter'], by['sel:reset'], by['sel:decide'], by['cda5b:enter']
    n, m = en['n'], en['m']
    require(len(en['vstat']) == n + m and len(en['x']) == n, 'sel:enter 크기')
    toks = [['sel', n, m, en['crossover'], en['retry'], en['remaining'], en['s2d8'], en['s120'], en['precision'],
             en['opttol_slot'], en['local_98'], dc['s178'], en['s184'], *en['vstat'], *en['x']],
            ['reset', xe['fixtol'], xe['feastol'], en['tol']['370'], xe['perturb'], *en['d_lb'], *en['d_ub']]]
    dual = by.get('sel:dual')
    toks.append(['dual', 0] if dual is None else ['dual', len(dual['d']), *dual['d']])
    ie, il = by.get('i325b4:enter'), by.get('i325b4:leave')
    if ie is None:
        toks.append(['inst', 0])
    else:
        require(ie['n'] == n and ie['m'] == m and ie['s4d8'] != 0 and ie['s28'] == 0, '설치 범위(스케일 없음·double)')
        flag = 1 if ie['d_flag3c'] == 1 else 0
        flags = [int(x, 16) for x in ie['d_flags']] if flag else []
        toks.append(['inst', 1, ie['tol']['380'], ie['tol']['3b0'], ie['tol']['388'], flag, *flags, *ie['lb'], *ie['ub'],
                     *ie['cost'], *ie['collen'], *ie['slack']])
    want = []
    if en['retry'] != 0:
        want.append(f"reset;rem {rs['remaining']};s184 {rs['s184']};perturb {rs['env_24a0']};vstat {j(rs['vstat'])}")
    else:
        want.append(f"reset;rem {rs['remaining']};s184 {rs['s184']};vstat {j(rs['vstat'])}")
    call = SITE[ce['ret_site']]
    require(ce['alg'] == (call == 'dual'), 'alg·호출 지점')
    want.append(f"select;r14 {int(bool(dc['r14']))};checked {int(dual is not None)};call {call};s180 {ce['s180']};"
                f"s178 {ce['s178']}")
    if ie is not None:
        want.append(f"install;vstat {j(il['vstat'])};s450 {il['s450']};list {j(il['list'] or [])};vals {j(il['vals'] or [])};"
                    f"x {j(il['x'])};slack {j(il['slack'])}")
    return toks, want


def build():
    """입력은 사례당 1회(run 1), 기대값은 768실행 각각. run 2·3의 입력 투영은 run 1과 같아야 한다."""
    rows, wants, first = [], [], {}
    for name, p in ordered():
        toks, want = project(xc.load(p))
        case = name.rsplit('/', 1)[0]
        if case not in first:
            first[case] = toks
            rows.append((case, toks))
        else:
            require(toks == first[case], f'{name}: 입력 투영이 run 1과 다름')
        wants.append((name, case, want))
    return rows, wants


def write():
    rs = xc.runs()
    print('controls: 사례', xc.controls(rs), '× 3회 동일')
    st = xc.acceptance(rs, verbose=False)
    print('acceptance: 실행', st['runs'], '통과')
    rows, _ = build()
    os.makedirs(D, exist_ok=True)
    with open(INPUT, 'w') as f:
        f.write(head() + '\n')
        for name, toks in rows:
            f.write(f'case {name}\n')
            for t in toks:
                f.write(' '.join(map(str, t)) + '\n')
    print('input', INPUT, len(rows), '사례(run 1 입력, run 2·3 투영 동일)')


def parse_trace(text):
    lines = text.rstrip('\n').split('\n')
    hd, out, cur = lines[0], [], None
    for l in lines[1:]:
        if l.startswith('case;'):
            cur = (l[5:], [])
            out.append(cur)
        else:
            cur[1].append(l)
    return hd, out


def fields(line):
    tag, *rest = line.split(';')
    return tag, dict(p.split(' ', 1) if ' ' in p else (p, '') for p in rest)


def compare(text=None, wants=None, verbose=True):
    if wants is None:
        _, wants = build()
    if text is None:
        require(os.path.exists(TRACE), f'{TRACE} 없음(cargo test 먼저)')
        text = open(TRACE).read()
    hd, got = parse_trace(text)
    errs = []
    if hd != head():
        errs.append(f'머리줄 {hd!r} ≠ {head()!r}')
    if [n for n, _ in got] != list(dict.fromkeys(c for _, c, _ in wants)):
        errs.append(f'사례 목록 다름: Rust {len(got)} 기대 {len({c for _, c, _ in wants})}')
    nl = nf = 0
    gd = dict(got)
    for name, case, want in wants:
        g = gd.get(case)
        if g is None:
            errs.append(f'{name}: Rust 출력 없음')
            continue
        if len(g) != len(want):
            errs.append(f'{name}: 줄 수 Rust {len(g)} 기대 {len(want)} ({[fields(x)[0] for x in g]})')
            continue
        for a, b in zip(g, want):
            ta, fa = fields(a)
            tb, fb = fields(b)
            if ta != tb or fa.keys() != fb.keys():
                errs.append(f'{name} {tb}: 필드 구성 Rust {ta} {list(fa)} 기대 {list(fb)}')
                continue
            for k in fb:
                nf += 1
                if fa[k] != fb[k]:
                    va, vb = fa[k].split(' '), fb[k].split(' ')
                    i = next((t for t in range(min(len(va), len(vb))) if va[t] != vb[t]), min(len(va), len(vb)))
                    errs.append(f'{name} {tb}.{k}[{i}]: Rust {va[i] if i < len(va) else "-"} 기대 {vb[i] if i < len(vb) else "-"}')
            nl += 1
    if verbose:
        if errs:
            print('불일치', len(errs))
            for e in errs[:20]:
                print(' ', e)
        else:
            print(f'일치: 사례 {len(gd)}, 실행 {len(wants)}, 줄 {nl}, 필드 {nf}')
    return errs, nl, nf


def self_test():
    _, wants = build()
    require(os.path.exists(TRACE), f'{TRACE} 없음')
    text = open(TRACE).read()
    errs, _, _ = compare(text, wants, verbose=False)
    require(not errs, '기준 비교가 먼저 일치해야 함')
    hd, got = parse_trace(text)

    def emit(g, h=hd):
        return h + '\n' + '\n'.join(f'case;{n}\n' + '\n'.join(ls) for n, ls in g) + '\n'

    muts = []
    seen = set()
    for idx, (name, ls) in enumerate(got):
        for li, line in enumerate(ls):
            tag, fs = fields(line)
            for k, v in fs.items():
                key = (tag, k)
                if key in seen or not v:
                    continue
                seen.add(key)
                toks = v.split(' ')
                t0 = toks[0]
                if len(t0) == 16 and all(c in '0123456789abcdef' for c in t0):
                    nv = format(int(t0, 16) ^ 1, '016x')
                elif t0.lstrip('-').isdigit():
                    nv = str(int(t0) + 1)
                else:
                    nv = t0 + 'x'
                new = line.replace(f'{k} {v}', f'{k} {" ".join([nv] + toks[1:])}', 1)
                g2 = [(n, list(l)) for n, l in got]
                g2[idx][1][li] = new
                muts.append((f'{tag}.{k} 첫 값', emit(g2)))
    first_inst = next(i for i, (_, ls) in enumerate(got) if any(l.startswith('install;') for l in ls))
    g2 = [(n, [l for l in ls if not (i == first_inst and l.startswith('install;'))]) for i, (n, ls) in enumerate(got)]
    muts.append(('install 줄 삭제', emit(g2)))
    muts.append(('사례 하나 삭제', emit(got[1:])))
    muts.append(('머리줄', emit(got, hd + 'x')))
    rej = 0
    for name, t in muts:
        e, _, _ = compare(t, wants, verbose=False)
        if e:
            rej += 1
        else:
            print('  생존(변조):', name)
    print(f'변조 {len(muts)}종 중 거부 {rej}')
    return 0 if rej == len(muts) else 1


S = 'crates/lp/src/crossover_cleanup.rs'
MUTATIONS = [
    ('reset-fix-raw', S, 'let fix = if 0.0 <= r.fixtol { if r.feastol <= r.fixtol { r.feastol } else { r.fixtol } } else { 0.0 };',
     'let fix = r.fixtol;'),
    ('reset-mid-lt', S, 'if lb + ub <= r.x[j] + r.x[j] {', 'if lb + ub < r.x[j] + r.x[j] {'),
    ('reset-swap', S, '                -2\n            } else {\n                -1\n            }', '                -1\n            } else {\n                -2\n            }'),
    ('reset-free-as-bound', S, '        } else {\n            -3\n        });', '        } else {\n            -1\n        });'),
    ('reset-rows-zero', S, '    vstat.extend(0..r.m as i32);', '    vstat.extend((0..r.m).map(|_| 0));'),
    ('dual-le', S, '    nmax + bmax < tol\n', '    nmax + bmax <= tol\n'),
    ('dual-no-skip', S, '        if tol > v {\n            continue;\n        }\n', ''),
    ('dual-sign', S, 'let v = if s < -3 || 0.0 > -dj { 0.0 } else { -dj };', 'let v = if s < -3 || 0.0 > dj { 0.0 } else { dj };'),
    ('dual-no-basic', S, '            bmax = if a > bmax { a } else { bmax };\n', ''),
    ('auto-retry-dual', S, '        if s.retry != 0 {\n            false', '        if s.retry != 0 {\n            true'),
    ('auto-pos-primal', S, '        } else if s.remaining > 0 {\n            true', '        } else if s.remaining > 0 {\n            false'),
    ('auto-no-pinf', S, 's.s2d8 == 0.0 && 0.0 < s.pinf && s.precision == 0', 's.s2d8 == 0.0 && s.precision == 0'),
    ('auto-no-s2d8', S, 's.s2d8 == 0.0 && 0.0 < s.pinf && s.precision == 0', '0.0 < s.pinf && s.precision == 0'),
    ('auto-no-check', S, '            dual_check(&reduced_costs(), vstat, s.opt_tol)', '            { let _ = reduced_costs(); true }'),
    ('cross-mask', S, '!((s.crossover & !2) == 1 || s.remaining < 0)', '!(s.crossover == 1 || s.remaining < 0)'),
    ('cross-neg-ignored', S, '!((s.crossover & !2) == 1 || s.remaining < 0)', '(s.crossover & !2) != 1'),
    ('basis-on-zero', S, '    } else if s.remaining == 0 {\n        Cleanup::Primal', '    } else if s.remaining < 0 {\n        Cleanup::Primal'),
    ('s178-always', S, 'if s.restarts == 2 { 10 } else { 0 }', '10'),
    ('inst-no-fix', S, '            if !(e.fixtol < ub - lb) {\n                vstat[j] = -4;\n            } else if !(e.tau',
     '            if false {\n                vstat[j] = -4;\n            } else if !(e.tau'),
    ('inst-lb-lt', S, '} else if !(e.tau <= (x[j] - lb).abs()) {', '} else if !(e.tau < (x[j] - lb).abs()) {'),
    ('inst-swap', S, '                vstat[j] = -1;\n            } else if !(e.tau <= (x[j] - ub).abs()) {\n                vstat[j] = -2;',
     '                vstat[j] = -2;\n            } else if !(e.tau <= (x[j] - ub).abs()) {\n                vstat[j] = -1;'),
    ('inst-no-cost-gate', S, 'e.lengths[j] != 0 || (e.opt_tol <= e.cost[j].abs() && e.cost[j].abs() != e.opt_tol)', 'true'),
    ('inst-no-tiny', S, '                if 1e-13 < x[j].abs() {', '                if true {'),
    ('inst-row-ub', S, '            } else if e.tau <= (s[i] - lb).abs() {', '            } else if e.tau <= (s[i] - lb).abs() && e.tau <= (s[i] - e.ub[j]).abs() {'),
    ('inst-row-swap', S, '                list.push((j as i32, s[i]));\n            } else {\n                vstat[j] = -1;',
     '                list.push((j as i32, s[i]));\n            } else {\n                vstat[j] = -2;'),
    ('inst-list-reversed', S, '    Some(Installed { vstat: vstat.to_vec(), list, factor_reset: true })',
     '    list.reverse();\n    Some(Installed { vstat: vstat.to_vec(), list, factor_reset: true })'),
    ('inst-basic-too', S, '            if vb[j] >= 0 {\n                continue;\n            }\n', ''),
    ('inst-list-value-lb', S, '                    list.push((j as i32, x[j]));', '                    list.push((j as i32, lb));'),
]


def mutate_rust(out_path):
    require(not os.path.exists(out_path), f'{out_path} 이미 있음')
    _, wants = build()
    work = os.path.join(ROOT, 'target/m3_xo_cleanup_mutations')
    if os.path.exists(work):
        shutil.rmtree(work)
    src = os.path.join(work, 'src')
    shutil.copytree(ROOT, src, ignore=shutil.ignore_patterns('target', '.git', 'bench', 'python', 'docs'))
    os.symlink(os.path.join(ROOT, 'bench'), os.path.join(src, 'bench'))
    env = dict(os.environ, CARGO_TARGET_DIR=os.path.join(work, 'target'))
    results = []
    for name, f, a, b in MUTATIONS:
        p = os.path.join(src, f)
        orig = open(p).read()
        if orig.count(a) != 1:
            results.append(dict(name=name, result='원문 없음'))
            print(name, '원문 없음')
            continue
        open(p, 'w').write(orig.replace(a, b))
        tr = os.path.join(src, 'target/m3_xo_cleanup_rust/trace.txt')
        try:
            r = subprocess.run(['cargo', 'test', '-q', '--release', '-p', 'rsolver-lp', '--test', 'xo_cleanup'],
                               cwd=src, env=env, capture_output=True, text=True, timeout=900)
            if 'error[' in r.stderr and not os.path.exists(tr):
                res = dict(result='컴파일 실패', detail=r.stderr[-300:])
            elif not os.path.exists(tr):
                res = dict(result='trace 없음')
            else:
                e, _, _ = compare(open(tr).read(), wants, verbose=False)
                res = dict(result='거부' if e else '생존', first=e[0] if e else None, count=len(e))
                os.remove(tr)
        except subprocess.TimeoutExpired:
            res = dict(result='시간 초과(거부)')
        finally:
            open(p, 'w').write(orig)
        res['name'] = name
        results.append(res)
        print(name, res['result'], (res.get('first') or '')[:150])
    os.makedirs(os.path.dirname(os.path.abspath(out_path)), exist_ok=True)
    json.dump(results, open(out_path, 'w'), ensure_ascii=False, indent=1)
    bad = [r for r in results if r['result'] in ('원문 없음', '컴파일 실패', 'trace 없음')]
    print(f'변이 {len(results)}: 거부 {sum(r["result"].startswith(("거부", "시간")) for r in results)}, '
          f'생존 {sum(r["result"] == "생존" for r in results)}, 무효 {len(bad)}')
    return 1 if bad else 0


def coverage():
    """생존 변이가 노리는 분기의 자료 출현 수(0이면 자료로 판별할 수 없음)."""
    import collections
    f, c = xc.f, collections.Counter()
    for _, p in ordered():
        ev = {}
        for e in xc.load(p)['events']:
            ev.setdefault(e['event'], e)
        en = ev['sel:enter']
        if en['retry']:
            inf = f(en['tol']['370'])
            for k in range(en['n']):
                lb, ub, x = f(en['d_lb'][k]), f(en['d_ub'][k]), f(en['x'][k])
                c['reset-fix-raw: 재설정 고정 열(ub − lb ≤ 0)'] += not (0.0 < ub - lb)
                c['reset-mid-lt: 재설정 lb + ub = 2x'] += (lb + ub == x + x) and 0 < ub - lb
                c['reset-free-as-bound: 재설정 자유 열'] += not (-inf < lb or ub < inf)
        if en['crossover'] == -1 and en['retry'] == 0 and en['remaining'] <= 0 and f(en['s2d8']) != 0:
            c['auto-no-s2d8: auto S+0x2d8 ≠ 0이고 PInf > 0'] += f(en['s120']) > 0
        du = ev.get('sel:dual')
        if du:
            tol = f(en['opttol_slot'])
            dd = [f(x) for x in du['d']]
            b = max([abs(v) for v, s in zip(dd, du['vstat']) if s >= 0] + [0])
            viol = [max(0, -v) for v, s in zip(dd, du['vstat']) if s in (-1, -2)]
            nm = max([v for v in viol if v >= tol] + [0])
            c['dual-le: 쌍대 합 = OptTol'] += b + nm == tol
            c['dual-no-basic: 기저 |d|가 결과를 바꿈'] += (nm < tol) != (b + nm < tol)
            c['dual-no-skip: OptTol 미만 위반이 결과를 바꿈'] += (max(viol + [0]) + b < tol) != (nm + b < tol)
        ie = ev.get('i325b4:enter')
        if ie:
            tau, opt = f(ie['tol']['380']), f(ie['tol']['388'])
            for k in range(ie['n']):
                if ie['vb'][k] >= 0:
                    continue
                x, lb, ub = f(ie['x'][k]), f(ie['lb'][k]), f(ie['ub'][k])
                away = tau <= abs(x - lb) and tau <= abs(x - ub)
                c['inst-lb-lt: |x − lb| = τ'] += abs(x - lb) == tau
                c['inst-no-cost-gate: bound에서 먼 빈 열·|c| ≤ OptTol'] += away and ie['collen'][k] == 0 and not abs(f(ie['cost'][k])) > opt
                c['inst-no-tiny: bound에서 먼 |x| ≤ 1e-13'] += away and abs(x) <= 1e-13
            for i in range(ie['m']):
                if ie['cb'][i] >= 0:
                    continue
                k = ie['n'] + i
                s, lb, ub = f(ie['slack'][i]), f(ie['lb'][k]), f(ie['ub'][k])
                c['inst-row-ub: lb에서 τ 이상, ub에서 τ 미만인 행'] += tau <= abs(s - lb) and not tau <= abs(s - ub)
    for k in sorted(c):
        print(f'  {k}: {c[k]}')
    return c


def main():
    p = argparse.ArgumentParser()
    p.add_argument('--write', action='store_true')
    p.add_argument('--compare', action='store_true')
    p.add_argument('--self-test', action='store_true')
    p.add_argument('--mutate-rust')
    p.add_argument('--coverage', action='store_true')
    a = p.parse_args()
    rc = 0
    if a.write:
        write()
    if a.compare:
        e, _, _ = compare()
        rc |= 1 if e else 0
    if a.self_test:
        rc |= self_test()
    if a.mutate_rust:
        rc |= mutate_rust(a.mutate_rust)
    if a.coverage:
        coverage()
    return rc


if __name__ == '__main__':
    sys.exit(main())
