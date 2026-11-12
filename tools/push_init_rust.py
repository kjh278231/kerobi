#!/usr/bin/env python3
"""M3-04c-1d: Rust push_init을 M3-04c-1b 오라클 15사례·17 push 호출과 비교한다.

  python3 tools/push_init_rust.py --write          # 1b controls·acceptance 뒤 입력 작성(bench/oracle/m3_push_init_ext_rust/input.txt)
  cargo test -p rsolver-lp --test push_init_ext    # Rust 재생 → target/m3_push_init_ext_rust/trace.txt
  python3 tools/push_init_rust.py --compare        # 1b 사건과 줄·필드 비교(첫 차이 출력, 실패 시 종료 1)
  python3 tools/push_init_rust.py --self-test      # 기대 줄 변조가 모두 거부되는지
  python3 tools/push_init_rust.py --mutate-rust OUT  # Rust 변이(임시 복사본)마다 재생·비교, 결과 JSON

입력은 push:enter 상태(합성 enter 편집 반영)와 외부 값(τ_in·S+0x120·S+0x3c8·환경 flag·F+0x188·조건수 tape,
first=0의 FTRAN 출력)뿐이다. 초기 재분해 입력은 같은 기저 모델의 1a 입력(21e·21h 투영)을 쓴다. 계약:
docs/m3/primal_push_init_ext_rust.md.
"""
import argparse, copy, hashlib, json, os, shutil, subprocess, sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import push_init_check as pic  # noqa: E402
import push_init_survey as sv  # noqa: E402

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
D = os.path.join(ROOT, 'bench/oracle/m3_push_init_ext_rust')
INPUT = os.path.join(D, 'input.txt')
TRACE = os.path.join(ROOT, 'target/m3_push_init_ext_rust/trace.txt')
INPUT_1A = os.path.join(ROOT, 'bench/oracle/m3_push_init_rust/input.txt')
SCHEMA = 'M3-04c-1d-v1'
TEST = 'push_init_ext'


def require(c, msg):
    if not c:
        raise SystemExit(f'입력 전제 실패: {msg}')


def j(xs):
    return ' '.join(map(str, xs))


def load_1a():
    lines = open(INPUT_1A).read().splitlines()
    out = {}
    for k in range(1, len(lines), 5):
        toks = [ln.split() for ln in lines[k:k + 5]]
        require([t[0] for t in toks] == ['case', 'model', 'cfg', 'enter', 'lu'], '1a 입력 형식')
        out[toks[0][1]] = dict(model=toks[1], cfg=toks[2], enter=toks[3], lu=toks[4])
    return out


def base_of(case):
    spec = pic.synthetic().get(case)
    return spec['base'] if spec else ('adlittle' if case == 'adlittle_x1' else case)


def ev1(evs, kind):
    xs = [e for e in evs if e['event'] == kind]
    require(len(xs) == 1, f'{kind} 1회')
    return xs[0]


def model_tokens(pe):
    md, st = pe['model'], pe['state']
    return ['model', str(st['n']), str(st['m']), str(len(md['rows'])), *(str(sv.i64(s)) for s in md['starts']),
            *map(str, md['lengths']), *map(str, md['rows']), *md['values'], *md['cost'], *md['lb'], *md['ub']]


def project_call(case, c, evs, a1):
    """(입력 6줄, 기대 줄 목록)."""
    pe = evs[0]
    require(pe['event'] == 'push:enter', f'{case} call{c} push:enter')
    st, n, m = pe['state'], pe['state']['n'], pe['state']['m']
    ie = ev1(evs, 'init:enter')
    ex = ie['extra']
    require(all(v == pic.ZERO for v in ie['residual_scratch']), f'{case} S+0x1f8 진입 0')
    require(st['precision'] == 0 and pe['alias_contiguous'], f'{case} double·별칭')
    first = pe['first']
    edits = [e for e in evs if e['event'] == 'edit']
    vstat, x, slack = list(st['vstat']), list(st['x']), list(st['slack'])
    s390, s3c8 = st['tolerances']['390'], ex['s3c8']
    for e in edits:
        if e['at'] == 'enter':
            tgt = dict(vstat=vstat, x=x, slack=slack)[e['field']]
            require(tgt[e['index']] == e['before'], f'{case} edit before != push:enter')
            tgt[e['index']] = e['after']
        else:
            require(e['at'] == 'ftran', 'edit 시점')
            if e['field'] == 's390':
                require(e['before'] == s390, f'{case} s390 before')
                s390 = e['after']
            elif e['field'] == 's3c8':
                require(e['before'] == s3c8, f'{case} s3c8 before')
                s3c8 = e['after']
            else:
                require(False, f'{case} 편집 필드 {e["field"]}')
    mt = model_tokens(pe)
    if c == 1 and first:
        require(mt == a1['model'], f'{case} 모델 != 1a {base_of(case)}')
    tol, cnt = st['tolerances'], st['counters']
    cfg = ['cfg', tol['380'], tol['398'], tol['370'], tol['3b0'], a1['cfg'][5], str(pe['model_status']),
           str(pe['attempt']), str(first)]
    require(cnt['request'] == ex['request'] and cnt['numeric'] == ex['numeric'] and st['status'] == ex['status'],
            f'{case} 카운터')
    enter = ['enter', *map(str, vstat), *map(str, st['header']), *x, *slack, *pe['model']['rhs'], s390,
             cnt['iterations'], str(ex['request']), str(ex['numeric']), str(cnt['failure']), str(ex['status'])]
    e1 = a1['enter']
    if c == 1 and first:
        # 편집 전 push:enter가 1a(21e·21h) enter 줄과 같아야 초기 재분해 입력을 빌릴 수 있다
        raw = ['enter', *map(str, st['vstat']), *map(str, st['header']), *st['x'], *st['slack'],
               *pe['model']['rhs'], st['tolerances']['390'], cnt['iterations'], str(ex['request']), str(ex['numeric']),
               str(cnt['failure']), str(ex['status'])]
        require(raw == e1[:-2], f'{case} push:enter != 1a {base_of(case)} enter')
    enter += e1[-2:] if first else ['0', '0']  # S+0x2fc·S+0x2f8: 1a(21e lu:enter), first=0은 미사용
    conds = [e['condition'] for e in evs if e['event'] == 'init:condition:leave']
    tau = ev1(evs, 'init:tau')['tau_in']
    ext = ['ext', tau, ex['s120'], str(s3c8), str(ex['env_2c54']), str(ex['f188']), ie['condition_in'],
           str(len(conds)), *conds]
    if not first:
        tape = [e['values'] for e in evs if e['event'] == 'init:ftran:leave']
        lu = ['lu', 'tape', str(len(tape))] + sum(tape, [])
    elif pe['attempt'] == 0 and c == 1:
        lu = ['lu', 'new', *a1['lu'][1:]]
        require(a1['lu'][2] == st['tolerances']['390'], f'{case} 초기 재분해 Markowitz != push:enter S+0x390')
    else:
        lu = ['lu', 'carry', st['tolerances']['390']]
    toks = [['case', case, str(c)], mt, cfg, enter, ext, lu]

    want = []
    for e in evs:
        k = e['event']
        if k == 'init:lu':
            require(e['request_in'] == 10, 'init:lu 요청 10')
            want.append(('lu', dict(ret=str(e['ret']), hdr=j(e['header']), vstat=j(e['vstat']))))
        elif k == 'init:ftran:enter':
            require(e['count'] == -1, 'FTRAN count -1')
            want.append(('ftin', dict(v=j(e['values']))))
        elif k == 'init:ftran:leave':
            want.append(('ftout', dict(v=j(e['values']))))
        elif k == 'init:residual:leave':
            require(e['caller'] == '1c1287', 'residual caller')
            want.append(('res', dict(v=j(e['out']))))
        elif k == 'init:strengthen:enter':
            want.append(('mk', dict(new=e['markowitz_new'], resid=e['residual'])))
        elif k == 'init:condition:enter':
            x = e['extra']
            want.append(('cond', dict(pinf=e['pinf'], viol=str(e['violations']), before=e['condition_before'],
                                      s390=x['s390'], s3c8=str(x['s3c8']), f188=str(x['f188']),
                                      numeric=str(x['numeric']))))
        elif k == 'init:adjust-gate':
            w = dict(adj=str(e['adjusted']), cand=str(e['candidates']), c01=str(e['count01']),
                     c001=str(e['count001']), c0001=str(e['count0001']), tau=e['tau_in'])
            if e['pinf_slot'] is not None:
                w['pinf'] = e['pinf_slot']
            want.append(('gate', w))
        elif k == 'supply:enter':
            s, cx = e['state'], e['supplier']['context']
            zero = all(v == 0 for v in e['support_marks'] or [])
            w = dict(cand=j(e['candidates']), sup=str(e['support']),
                     smarks='none' if zero else j(e['support_marks']), pos=str(e['position']),
                     dfr=str(cx['deferred']), fail=j(e['failures']), csum=cx['count_sum'],
                     succ=str(cx['successes']), vstat=j(s['vstat']), header=j(s['header']),
                     xs=j(s['x'] + s['slack']), rhs=j(s['rhs']), basic=j(s['basic']), width=j(s['width']),
                     marks=j(s['marks']), pinf=s['pinf'], iter=s['counters']['iterations'],
                     numeric=str(s['counters']['numeric']), failure=str(s['counters']['failure']),
                     request=str(s['counters']['request']), status=str(s['status']), s390=s['tolerances']['390'])
            require(e['batch'] == 0 and s['precision'] == 0, 'batch·x87')
            if first:
                f = s['factor']
                w['factor'] = f"{f['updates']} {f['limit']} {sv.i64(f['nnzL'])} {sv.i64(f['nnzU'])}"
            want.append(('start', w))
            break
        elif k == 'push:leave':
            require(e['retry'] == 1, f'{case} call{c}: supply 없이 push:leave(retry 아님)')
            x = e['extra']
            want.append(('retry', dict(numeric=str(x['numeric']), s390=x['s390'], s3c8=str(x['s3c8']),
                                       f188=str(x['f188']), s120=x['s120'], cond=e['condition'])))
    if not first:
        want = [w for w in want if w[0] != 'ftout']  # 주입한 출력은 비교하지 않는다
    return toks, want


def cases():
    """[(사례, 호출, 사건열)] — 1b run 1."""
    out = []
    for case in pic.cases():
        doc = sv.load(pic.path(case, 1))
        for c, evs in sorted(pic.calls_of(doc).items()):
            out.append((case, c, evs))
    return out


def build():
    a1 = load_1a()
    toks, wants = [], []
    for case, c, evs in cases():
        t, w = project_call(case, c, evs, a1[base_of(case)])
        toks.append(t)
        wants.append(((case, c), w))
    return toks, wants


def head():
    h = hashlib.sha256()
    for p in [pic.MANIFEST, INPUT_1A]:
        h.update(open(p, 'rb').read())
    return f'{SCHEMA} {h.hexdigest()}'


def write():
    docs = pic.load_all()
    errs = pic.controls(docs, json.load(open(pic.MANIFEST)))
    fails, _ = pic.acceptance(docs)
    require(not errs and not fails, f'1b 검사 실패: {(errs + fails)[:3]}')
    toks, wants = build()
    os.makedirs(D, exist_ok=True)
    text = head() + '\n' + '\n'.join(' '.join(t) for blk in toks for t in blk) + '\n'
    if os.path.exists(INPUT) and open(INPUT).read() == text:
        print(f'입력 불변: {os.path.relpath(INPUT, ROOT)}')
    else:
        open(INPUT, 'w').write(text)
        print(f'작성: {os.path.relpath(INPUT, ROOT)} ({len(toks)} 호출)')


def parse_trace(text):
    lines = text.splitlines()
    got, cur = {}, None
    for ln in lines[1:]:
        kind, *rest = ln.split(';')
        if kind == 'case':
            name, c = rest[0].split()
            cur = (name, int(c))
            got[cur] = []
            continue
        fields = {}
        for f in rest:
            k, _, v = f.partition(' ')
            fields[k] = v
        got[cur].append((kind, fields))
    return lines[0] if lines else '', got


def compare_lines(wants, got, verbose=True):
    """첫 차이 목록과 집계."""
    errs, nlines, nfields = [], 0, 0
    for key, want in wants:
        g = got.get(key)
        if g is None:
            errs.append(f'{key}: Rust 출력 없음')
            continue
        gl = [x for x in g if not (x[0] == 'ftout' and not any(w[0] == 'ftout' for w in want))]
        kinds_w = [k for k, _ in want]
        kinds_g = [k for k, _ in gl]
        if kinds_w != kinds_g:
            i = next((i for i, (a, b) in enumerate(zip(kinds_w, kinds_g)) if a != b), min(len(kinds_w), len(kinds_g)))
            errs.append(f'{key}: 사건 순서 {i}: 원본 {kinds_w[i:i + 3]} Rust {kinds_g[i:i + 3]}')
            continue
        for i, ((k, w), (_, r)) in enumerate(zip(want, gl)):
            nlines += 1
            for f, v in w.items():
                nfields += 1
                if f not in r:
                    errs.append(f'{key} 줄{i} {k}.{f}: Rust 필드 없음')
                    break
                if r[f] != v:
                    a, b = v.split(), r[f].split()
                    idx = next((x for x, (p, q) in enumerate(zip(a, b)) if p != q), min(len(a), len(b)))
                    errs.append(f'{key} 줄{i} {k}.{f}[{idx}]: 원본 {a[idx] if idx < len(a) else "끝"} '
                                f'Rust {b[idx] if idx < len(b) else "끝"} (길이 {len(a)}/{len(b)})')
                    break
            else:
                continue
            break
    if verbose:
        for e in errs[:10]:
            print('불일치', e)
    return errs, nlines, nfields


def compare(trace_text=None, wants=None, verbose=True):
    if wants is None:
        _, wants = build()
    text = trace_text if trace_text is not None else open(TRACE).read()
    h, got = parse_trace(text)
    errs = []
    if h != head():
        errs.append('trace 머리(입력 해시) 불일치: --write 뒤 cargo test를 다시 실행')
    errs += [f'{k}: Rust 오류 {f.get("msg")}' for k, g in got.items() for kind, f in g if kind == 'error']
    e2, nl, nf = compare_lines(wants, got, verbose=False)
    errs += e2
    extra = set(got) - {k for k, _ in wants}
    if extra:
        errs.append(f'예상하지 않은 호출 {sorted(extra)}')
    if verbose:
        for e in errs[:10]:
            print('불일치', e)
        print(f'호출 {len(wants)}·줄 {nl}·필드 {nf} 비교, 불일치 {len(errs)}')
    return errs, nl, nf


def self_test():
    """기대 줄 변조(사례마다 줄 종류별 1비트·값, 순서 교환, 누락)가 모두 거부되는지."""
    _, wants = build()
    text = open(TRACE).read()
    errs, _, _ = compare(text, wants, verbose=False)
    require(not errs, f'기준 비교가 먼저 통과해야 함: {errs[:2]}')
    tried, survived = 0, []

    def flip(v):
        a = v.split()
        t = a[0]
        if len(t) == 16 and all(ch in '0123456789abcdef' for ch in t):
            a[0] = format(int(t, 16) ^ 1, '016x')
        elif t.lstrip('-').isdigit():
            a[0] = str(int(t) + 1)
        else:
            a[0] = t + 'x'
        return ' '.join(a)

    for wi, (key, want) in enumerate(wants):
        seen = set()
        for li, (k, w) in enumerate(want):
            for f in w:
                if (k, f) in seen:
                    continue
                seen.add((k, f))
                mut = copy.deepcopy(wants)
                mut[wi][1][li][1][f] = flip(w[f])
                tried += 1
                if not compare_lines(mut, parse_trace(text)[1], verbose=False)[0]:
                    survived.append(f'{key} {k}.{f}')
        if len(want) >= 2:
            mut = copy.deepcopy(wants)
            mut[wi][1][0], mut[wi][1][1] = mut[wi][1][1], mut[wi][1][0]
            tried += 1
            if not compare_lines(mut, parse_trace(text)[1], verbose=False)[0]:
                survived.append(f'{key} 순서')
        mut = copy.deepcopy(wants)
        mut[wi][1].pop()
        tried += 1
        if not compare_lines(mut, parse_trace(text)[1], verbose=False)[0]:
            survived.append(f'{key} 누락')
    print(f'변조 {tried}종, 생존 {len(survived)}')
    for s in survived:
        print('생존', s)
    return 1 if survived else 0


I = 'crates/lp/src/crossover_init.rs'
MUTATIONS = [
    # (이름, 파일, 원문, 변이) — 1b 15사례가 구별해야 하는 외부 값 경로
    ('adjust-tau-always-01', I, '            * if c[0] < total {\n                0.001', '            * if false {\n                0.001'),
    ('adjust-tau-no-scale', I, '        tau_in = Some(tau);', '        tau_in = Some(tau_in.unwrap());\n        let tau = tau_in.unwrap();'),
    ('rebuild-subtract-m45', I, '            if !(s < 0 && s > -4) {\n                continue;\n            }\n            let x = st.xs[j];',
     '            if s >= 0 {\n                continue;\n            }\n            if s <= -4 {\n                let v = if s == -4 { lp.lb[j] } else { lp.ub[j] };\n                if v != 0.0 {\n                    subtract_column(lp, j, v, &mut r);\n                }\n                continue;\n            }\n            let x = st.xs[j];'),
    ('promote-m1-strict', I, '-1 => tau + lower <= x,', '-1 => tau + lower < x,'),
    ('promote-m1-never', I, '-1 => tau + lower <= x,', '-1 => false,'),
    ('rebuild-slack-m2-kept', I, 'let v = if s != -1 || tau <= slack {', 'let v = if s == -3 || (s == -1 && tau <= slack) {'),
    ('adjust-never', I, '        let adjust = !out.adjusted\n            && pinf > 1.0', '        let adjust = false\n            && pinf > 1.0'),
    ('adjust-twice', I, '        let adjust = !out.adjusted\n', '        let adjust = true\n'),
    ('adjust-prev-pinf-ignored', I, '            && h > e.prev_pinf.expect("tolerance adjustment test without S+0x120")\n', ''),
    ('adjust-count-ignored', I, '            && counts.expect("tolerance adjustment without param_1")[2] < cand.len() as i32;', ';'),
    ('slack-m2-zero', I, '            // -2: asm 0x1c4f3d jumps to the subtraction without loading a value (the previous xmm0)\n            _ => xmm0,', '            _ => 0.0,'),
    ('col-m5-lb', I, '            -5 => lp.ub[j],', '            -5 => lp.lb[j],'),
    ('col-m5-zero', I, '            -5 => lp.ub[j],', '            -5 => 0.0,'),
    ('mk-05-threshold', I, '|| 1e-8 < residual {', '|| 1e-3 < residual {'),
    ('mk-no-tighten', I, '        if markowitz <= candidate && candidate != markowitz {', '        if false {'),
    ('mk-tighten-not-stored', I, '            markowitz = candidate;\n            // S+0x390', '            // S+0x390'),
    ('limit-never', I, '    (limit < t.update_limit).then_some(limit)', '    None::<i32>.filter(|_| limit < t.update_limit)'),
    ('limit-env-ignored', I, '    if t.env_2c54 >= 1 {', '    if t.env_2c54 >= 0 {'),
    ('numeric-threshold', I, '        if 1e-6 < residual {\n            ctl.numeric += 1;', '        if 1e-3 < residual {\n            ctl.numeric += 1;'),
    ('gate-p-always-1', I, 'let p = if e.model_status == 2 { pinf } else { 1.0 };', 'let p = 1.0;'),
    ('gate-p-always-pinf', I, 'let p = if e.model_status == 2 { pinf } else { 1.0 };', 'let p = pinf;'),
    ('gate-pinf-no-count', I, '        if 1e15 < pinf {\n            ctl.numeric += 1;', '        if 1e15 < pinf {'),
    ('gate-pinf-threshold', I, '        if 1e15 < pinf {', '        if 1e18 < pinf {'),
    ('gate-attempt-ignored', I, '        if e.attempt == 0 && (cond_above(1e14)', '        if (cond_above(1e14)'),
    ('cond-not-called', I, '            condition = backend.condition(condition);\n', ''),
    ('cond-called-first0', I, '        if e.first {\n            backend.observe(InitEvent::Condition', '        if true {\n            backend.observe(InitEvent::Condition'),
    ('cond-product-ignored', I, ' || condition.is_some_and(|c| 1e16 < p * c)', ''),
    ('first0-refactor', I, '    if e.first {\n        ctl.request = 10;', '    if e.first || e.attempt == 0 {\n        ctl.request = 10;'),
    ('retry-markowitz-entry', I, '            out.markowitz = candidate;\n', ''),
]


def mutate_rust(out_path):
    require(not os.path.exists(out_path), f'{out_path} 이미 있음')
    _, wants = build()
    work = os.path.join(ROOT, 'target/m3_push_init_ext_mutations')
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
        try:
            r = subprocess.run(['cargo', 'test', '-q', '-p', 'rsolver-lp', '--test', TEST], cwd=src, env=env,
                               capture_output=True, text=True, timeout=900)
            tr = os.path.join(src, 'target/m3_push_init_ext_rust/trace.txt')
            if 'error[' in r.stderr and not os.path.exists(tr):
                res = dict(result='컴파일 실패', detail=r.stderr[-300:])
            elif not os.path.exists(tr):
                res = dict(result='trace 없음')
            else:
                errs, _, _ = compare(open(tr).read(), wants, verbose=False)
                res = dict(result='거부' if errs else '생존', first=errs[0] if errs else None)
                os.remove(tr)
        except subprocess.TimeoutExpired:
            res = dict(result='시간 초과(거부)')
        finally:
            open(p, 'w').write(orig)
        res['name'] = name
        results.append(res)
        print(name, res['result'], res.get('first') or '')
    json.dump(results, open(out_path, 'w'), ensure_ascii=False, indent=1)
    bad = [r for r in results if r['result'] in ('원문 없음', '컴파일 실패', 'trace 없음')]
    print(f'변이 {len(results)}: 거부 {sum(r["result"].startswith(("거부", "시간")) for r in results)}, '
          f'생존 {sum(r["result"] == "생존" for r in results)}, 무효 {len(bad)}')
    return 1 if bad else 0


def main():
    p = argparse.ArgumentParser()
    p.add_argument('--write', action='store_true')
    p.add_argument('--compare', action='store_true')
    p.add_argument('--self-test', action='store_true')
    p.add_argument('--mutate-rust')
    a = p.parse_args()
    rc = 0
    if a.write:
        write()
    if a.compare:
        errs, _, _ = compare()
        rc |= 1 if errs else 0
    if a.self_test:
        rc |= self_test()
    if a.mutate_rust:
        rc |= mutate_rust(a.mutate_rust)
    return rc


if __name__ == '__main__':
    sys.exit(main())
