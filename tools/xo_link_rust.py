#!/usr/bin/env python3
"""M3-04f-4b: Rust push 연쇄(04f-1 연결)가 남긴 factor·vstat·header를 `crossover_dual`(251b4f)로 이어 원본과 비교한다.

  python3 tools/xo_link_rust.py --write            # 04f-1 입력·04f-4a 원본 자료 → bench/oracle/m3_xo_link_rust/link.txt
  cargo test --release -p rsolver-lp --test push_init_ext dual_link_trace   # → target/m3_xo_link_rust/trace.txt
  python3 tools/xo_link_rust.py --compare          # 연결(vstat·header·factor·x1d8) + 251b4f 출력, 원본 3회 각각
  python3 tools/xo_link_rust.py --self-test        # trace 변조 거부
  python3 tools/xo_link_rust.py --mutate-rust OUT  # Rust 변이(임시 사본) 거부/생존

link.txt에는 원본이 push에서 만들지 않는 상태만 넣는다(dual push 감산 목록, S+0x3a8, work, 출력 머리 count, S+0x380).
factor·vstat·header·비용·행 방향 A는 Rust가 push 연쇄와 1c 모델 기록으로 만든다.
L1 = dual push 첫 호출 0x1c6b49(dpush-a), L2 = dual push가 factor·header·vstat을 바꾸지 않은 사례의 0x1bf09e(post).
계약: docs/m3/crossover_dual_link.md.
"""
import argparse, collections, copy, hashlib, json, os, shutil, subprocess, sys
from pathlib import Path

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import xo_dual_check as oracle  # noqa: E402
import xo_dual_rust as xdr  # noqa: E402
from xo_dual_check import Fail, require  # noqa: E402

ROOT = Path(__file__).resolve().parents[1]
CHAIN = ROOT / 'bench/oracle/m3_push_install_rust/input.txt'
OUT = ROOT / 'bench/oracle/m3_xo_link_rust'
LINK = OUT / 'link.txt'
TRACE = ROOT / 'target/m3_xo_link_rust/trace.txt'
SCHEMA = 'M3-04f-4b-v1'
SPECIAL = ('arms', 'retry', 'interior')


def oracle_case(name):
    """1c 사례 이름 → 04f-4a 원본 자료의 Crossover 3 사례(없으면 None)."""
    if name.startswith('netlib/'):
        return 'x3/' + name[len('netlib/'):]
    if name in SPECIAL:
        return f'cases/{name}_x3'
    return None


def chain_cases():
    ls = CHAIN.read_text().split('\n')
    out, cur = collections.OrderedDict(), None
    for l in ls[1:]:
        if l.startswith('case '):
            _, name, call = l.split()
            cur = (name, int(call))
            require(cur not in out, f'1c 사례 중복 {cur}')
            out[cur] = []
        if l:
            out[cur].append(l)
    return ls[0], out


def model(recs):
    t = recs[1].split()
    require(t[0] == 'model', 'model 기록 아님')
    n, m, nnz = map(int, t[1:4])
    t = t[4:]
    st = list(map(int, t[:n])); ln = list(map(int, t[n:2 * n]))
    rw = list(map(int, t[2 * n:2 * n + nnz])); vl = t[2 * n + nnz:2 * n + 2 * nnz]
    cost = t[2 * n + 2 * nnz:2 * n + 2 * nnz + n + m]
    rows = [[] for _ in range(m)]
    for j in range(n):
        for k in range(st[j], st[j] + ln[j]):
            rows[rw[k]].append((j, vl[k]))
    return n, m, cost, rows


def check_rowA(name, A, rows):
    for i, want in enumerate(rows):
        lo, L = A['start'][i] - A['base'], A['len'][i]
        got = list(zip(A['cols'][lo:lo + L], A['vals'][lo:lo + L])) if L else []
        require(got == want, f'{name}: 원본 행 방향 A 행 {i}가 1c 모델 전치와 다름')


def site_tokens(k, kind):
    e, be = k['enter'], k['bt_enter']
    a = be['args']
    require(a['factor_s470'] and a['input_s220'] and a['output_s228'] and a['cache_null'], 'btran 인자')
    require(be['input']['count'] == -1 and be['x200'] == xdr.ZERO64, 'btran 진입')
    require(not be['factor']['scratch']['x1b0'] and not be['factor']['scratch']['x1b8'], 'scratch 진입')
    require(e['f1f8'] == e['s148'], 'F1f8 진입')
    toks = [kind, str(be['output_count']), str(len(e['sub']))]
    for _, j, v in e['sub']:
        toks += [str(j), v]
    toks.append(e['tol3a8'])
    toks += xdr.opt(e['work'])
    toks += xdr.opt(k['sel']['tol380'] if kind == 'post' else None)
    return toks


def expected(k, kind):
    e, f = k['enter'], k['bt_enter']['factor']
    w = xdr.expected(k)
    w.pop('r14')
    if kind != 'post':
        w['s2d8'] = None
    w.update(vstat=e['vstat'], header=e['header'], x1d8=f['x1d8'], scratch0=True,
             factor=' '.join(xdr.factor_fields(f, 'factor')))
    return w


def build():
    head, chain = chain_cases()
    m = oracle.load_manifest()
    oracle.check_files(m)
    data = oracle.records(m)
    oracle.controls(data)
    lines, wants, info = [], {}, collections.OrderedDict()
    for (name, call), recs in chain.items():
        oc = oracle_case(name)
        if oc is None:
            continue
        require(oc in data, f'{name}: 04f-4a 원본 자료 {oc} 없음')
        require(call == 1 and len(recs) == 8 and recs[5].split()[1] == 'new', f'{name} {call}: 연결 사례 형식')
        n, mm, cost, rows = model(recs)
        per_run = [oracle.calls(r) for r in data[oc]]
        ks = per_run[0]
        require(ks[0]['site'] == 'dpush-a', f'{name}: 첫 251b4f가 dual push 첫 호출이 아님')
        posts = [i for i, k in enumerate(ks) if k['site'] == 'post']
        require(len(posts) == 1, f'{name}: 0x1bf09e 호출 수 {len(posts)}')
        a, p = ks[0], ks[posts[0]]
        require((a['enter']['n'], a['enter']['m']) == (n, mm) and a['enter']['cost'] == cost, f'{name}: n·m·비용')
        check_rowA(name, a['enter']['rowA'], rows)
        kept = (a['bt_enter']['factor'] == p['bt_enter']['factor'] and a['enter']['header'] == p['enter']['header']
                and a['enter']['vstat'] == p['enter']['vstat'])
        sites = [('dpush-a', 0)] + ([('post', posts[0])] if kept else [])
        toks = [name, str(call), 'link', str(len(sites))]
        for kind, idx in sites:
            toks += site_tokens(ks[idx], kind)
            wants[(name, kind)] = [expected(calls[idx], kind) for calls in per_run]
        lines.append(' '.join(toks))
        info[name] = dict(oracle=oc, kept=kept, sub=len(a['enter']['sub']), dual_calls=sum(k['site'].startswith('dpush') for k in ks))
    body = '\n'.join(lines) + '\n'
    h = hashlib.sha256(head.encode() + b'\n' + body.encode()).hexdigest()
    return f'{SCHEMA} {h}\n' + body, wants, info


def parse_trace(text):
    ls = text.rstrip('\n').split('\n')
    got, cur, leave, errors = collections.defaultdict(list), None, {}, []
    for l in ls[1:]:
        if l.startswith('case;'):
            cur = l[5:].split()[0]
        elif l.startswith('dlink;'):
            got[cur].append(json.loads(l[6:]))
        elif l.startswith('leave;'):
            leave[cur] = l
        elif l.startswith('error;'):
            errors.append(f'{cur}: {l}')
    return ls[0], got, leave, errors


def compare(text, wants, verbose=True):
    head, got, leave, errors = parse_trace(text)
    require(head == LINK.read_text().split('\n', 1)[0], 'trace 머리줄이 link.txt와 다름')
    errs = list(errors)
    names = sorted({n for n, _ in wants})
    nfields = runs = 0
    branches = collections.Counter()
    for name in names:
        kinds = [k for n, k in wants if n == name]
        g = got.get(name, [])
        if len(g) != len(kinds):
            errs.append(f'{name}: dlink {len(g)}줄, 기대 {len(kinds)}')
            continue
        if 'retry 0;' not in leave.get(name, ''):
            errs.append(f'{name}: push 반환 retry 0 아님')
        for kind, gg in zip(kinds, g):
            gg = dict(gg)
            if gg.pop('site') != kind:
                errs.append(f'{name}: 호출 지점 순서')
                continue
            br = gg.pop('branches', None)
            if br:
                branches.update(f'{k}={v}' for k, v in br.items())
            for r, w in enumerate(wants[(name, kind)], 1):
                runs += 1
                d = xdr.first_diff(w, {k: gg.get(k) for k in w})
                if d:
                    errs.append(f'{name} [{kind}] run {r}{d}')
                nfields += xdr.count_fields(w) + len(w['factor'].split()) - 1
    if verbose:
        for e in errs[:20]:
            print('차이', e[:300])
        c = collections.Counter(k for _, k in wants)
        print(f'사례 {len(names)}, 호출 {dict(c)}, 비교 실행 {runs}, 필드 {nfields}, 차이 {len(errs)}')
        print('BTRAN 분기', dict(sorted(branches.items())))
    return errs, runs, nfields


def self_test():
    _, wants, _ = build()
    text = TRACE.read_text()
    errs, _, _ = compare(text, wants, verbose=False)
    require(not errs, '기준 비교가 먼저 일치해야 함')
    ls = text.rstrip('\n').split('\n')

    def flip(h):
        return format(int(h, 16) ^ 1, '016x')

    def at(kind, pred=lambda g: True):
        for i, l in enumerate(ls):
            if l.startswith('dlink;'):
                g = json.loads(l[6:])
                if g['site'] == kind and pred(g):
                    return i
        raise Fail(f'{kind} 줄 없음')

    def edit(i, f):
        out = list(ls)
        g = json.loads(out[i][6:])
        f(g)
        out[i] = 'dlink;' + json.dumps(g)
        return '\n'.join(out) + '\n'

    def factor_tok(g, k, new):
        t = g['factor'].split()
        t[k] = new
        g['factor'] = ' '.join(t)

    a, post = at('dpush-a', lambda g: g['rhs'] != []), at('post')
    muts = [
        ('vstat', a, lambda g: g['vstat'].__setitem__(0, -9)),
        ('header', a, lambda g: g['header'].reverse()),
        ('x1d8', a, lambda g: g.__setitem__('x1d8', 3)),
        ('scratch0', a, lambda g: g.__setitem__('scratch0', False)),
        ('factor perm', a, lambda g: factor_tok(g, 1, str(int(g['factor'].split()[1]) + 1))),
        ('factor 끝', a, lambda g: factor_tok(g, -1, '9')),
        ('rhs', a, lambda g: g['rhs'].__setitem__(0, flip(g['rhs'][0]))),
        ('btran output', a, lambda g: g['btran']['output']['val'].__setitem__(0, flip(g['btran']['output']['val'][0]))),
        ('btran x200', a, lambda g: g['btran'].__setitem__('x200', flip(g['btran']['x200']))),
        ('y', a, lambda g: g['y'].__setitem__(-1, flip(g['y'][-1]))),
        ('d', a, lambda g: g['d'].__setitem__(0, flip(g['d'][0]))),
        ('work', a, lambda g: g.__setitem__('work', flip(g['work']))),
        ('post d', post, lambda g: g['d'].__setitem__(-1, flip(g['d'][-1]))),
        ('post s2d8', post, lambda g: g.__setitem__('s2d8', flip(g['s2d8']))),
        ('지점 이름', post, lambda g: g.__setitem__('site', 'dpush-a')),
    ]
    bad = 0
    for label, i, f in muts:
        e2, _, _ = compare(edit(i, f), wants, verbose=False)
        if e2:
            print('거부', label, '-', e2[0][:140])
        else:
            bad += 1
            print('생존', label)
    extra = [
        ('post 줄 삭제', '\n'.join(l for k, l in enumerate(ls) if k != post) + '\n'),
        ('Rust 오류 줄', text + 'error;msg panic\n'),
        ('retry 1', text.replace('leave;retry 0;', 'leave;retry 1;', 1)),
    ]
    for label, t in extra:
        e2, _, _ = compare(t, wants, verbose=False)
        if e2:
            print('거부', label, '-', e2[0][:140])
        else:
            bad += 1
            print('생존', label)
    require(not bad, f'변조 생존 {bad}')
    print('self-test 통과', len(muts) + len(extra))


T = 'crates/lp/tests/push_init_ext.rs'
MUTATIONS = [
    # 연결: factor·상태 전달
    ('l2-fresh-btran-state', T, '    lu.x1d8 = c.btran.x1d8;\n    lu.bucket = c.btran.bucket.clone();\n', ''),
    ('vstat-before-install', T, '                        backend.line(dual_link(n, m, &cost, &rows, &s.st.vstat, &s.st.header, ls, sb));',
     '                        let mut v = s.st.vstat.clone();\n                        for x in v.iter_mut() { if *x == -2 { *x = -1; } }\n                        backend.line(dual_link(n, m, &cost, &rows, &v, &s.st.header, ls, sb));'),
    ('header-rotate', T, '                        backend.line(dual_link(n, m, &cost, &rows, &s.st.vstat, &s.st.header, ls, sb));',
     '                        let mut h = s.st.header.clone();\n                        h.rotate_left(1);\n                        backend.line(dual_link(n, m, &cost, &rows, &s.st.vstat, &h, ls, sb));'),
    ('x1d8-zero', T, '    let x1d8 = lu.x1d8;\n', '    let x1d8 = 0;\n'),
    ('bucket-none', T, '    let c = dual_values(&e, &f, ls.out_count, x1d8, lu.bucket.clone(), ls.work);',
     '    let c = dual_values(&e, &f, ls.out_count, x1d8, None, ls.work);'),
    ('cost-model-order', T, '    let e = DualInput { n, m, cost, vstat,', '    let rc: Vec<f64> = cost.iter().rev().copied().collect();\n    let e = DualInput { n, m, cost: &rc, vstat,'),
    # 전치: 행 안 순서·값
    ('transpose-col-reverse', T, '    for j in 0..n {\n        for k in starts[j] as usize..starts[j] as usize + lengths[j] as usize {\n            let i = rows[k] as usize;',
     '    for j in (0..n).rev() {\n        for k in starts[j] as usize..starts[j] as usize + lengths[j] as usize {\n            let i = rows[k] as usize;'),
    ('transpose-drop-last', T, '            fill[i] += 1;\n', '            if fill[i] + 1 < start[i] + len[i] { fill[i] += 1; } else { vals[fill[i]] = 0.0; fill[i] += 1; }\n'),
]


def mutate_rust(out_path):
    out_path = Path(out_path)
    require(not out_path.exists(), f'{out_path} 이미 있음')
    _, wants, _ = build()
    work = ROOT / 'target/m3_xo_link_mutations'
    if work.exists():
        shutil.rmtree(work)
    src = work / 'src'
    shutil.copytree(ROOT, src, ignore=shutil.ignore_patterns('target', '.git', 'bench', 'python', 'docs'))
    os.symlink(ROOT / 'bench', src / 'bench')
    env = dict(os.environ, CARGO_TARGET_DIR=str(work / 'target'))
    results = []
    for name, f, a, b in MUTATIONS:
        p = src / f
        orig = p.read_text()
        if orig.count(a) != 1:
            results.append(dict(name=name, result='원문 없음'))
            print(name, '원문 없음')
            continue
        p.write_text(orig.replace(a, b))
        tr = src / 'target/m3_xo_link_rust/trace.txt'
        if tr.exists():
            tr.unlink()
        try:
            r = subprocess.run(['cargo', 'test', '-q', '--release', '-p', 'rsolver-lp', '--test', 'push_init_ext',
                                'dual_link_trace'], cwd=src, env=env, capture_output=True, text=True, timeout=1800)
            if not tr.exists():
                res = dict(result='컴파일·실행 실패', detail=r.stderr[-300:])
            else:
                errs, _, _ = compare(tr.read_text(), wants, verbose=False)
                res = dict(result='거부' if errs else '생존', first=errs[0][:200] if errs else None, count=len(errs))
        finally:
            p.write_text(orig)
        results.append(dict(name=name, **res))
        print(name, res['result'], res.get('first') or res.get('detail') or '')
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(json.dumps(results, ensure_ascii=False, indent=1) + '\n')
    c = collections.Counter(r['result'] for r in results)
    print('변이', len(results), dict(c))


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--write', action='store_true')
    ap.add_argument('--compare', action='store_true')
    ap.add_argument('--self-test', action='store_true')
    ap.add_argument('--mutate-rust')
    a = ap.parse_args()
    if a.write:
        text, wants, info = build()
        OUT.mkdir(parents=True, exist_ok=True)
        LINK.write_text(text)
        kept = [n for n, v in info.items() if v['kept']]
        print('연결 사례', len(info), '호출', len(wants), 'L2(basis 불변)', len(kept), kept)
        print('감산 목록이 있는 L1', sum(v['sub'] > 0 for v in info.values()))
    if a.compare:
        _, wants, _ = build()
        require(TRACE.exists(), f'{TRACE} 없음')
        errs, _, _ = compare(TRACE.read_text(), wants)
        require(not errs, f'차이 {len(errs)}')
    if a.self_test:
        self_test()
    if a.mutate_rust:
        mutate_rust(a.mutate_rust)


if __name__ == '__main__':
    try:
        main()
    except Fail as e:
        print('실패:', e)
        sys.exit(1)
