"""M3-04c-11: push 연쇄 생존 경계 캡처의 재현성·출처·목표 태그 검사와 manifest.

  --manifest   bench/oracle/m3_push_chain/manifest.json을 쓴다(검사 통과 뒤, 기존 내용과 다르면 거부)
  --check      manifest·캡처를 검사한다. 재현성·출처·목표 태그 중 하나라도 어긋나면 종료 1
  --self-test  변조(바이너리 해시 0, 사건 비움, 목표 0회 입력, 실행 간 차이, 도구 해시)가 모두 거부되는지 확인
"""
import argparse
import copy
import hashlib
import json
from pathlib import Path
import struct
import sys

ROOT = Path(__file__).resolve().parents[1]
D = 'bench/oracle/m3_push_chain'
BINARY_SHA = '8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c'
TOOL, AGENT = 'tools/primal_push_recall_capture.js', 'tools/primal_push_recall_agent.js'
LPS = {'shift': ('lb-enter', 'lb-leave-shift', 'free-flip', 'f-row-loop'), 'resolve': ('v-then-f',)}
KINDS = ('primal', 'dual')
TAGS = ('lb-enter', 'lb-leave-shift', 'v-then-f', 'free-flip', 'f-row-loop')


def sha(p):
    return hashlib.sha256((ROOT / p).read_bytes()).hexdigest()


def f64(h):
    return struct.unpack('<d', bytes.fromhex(h))[0]


def tags(rec):
    """원본 사건에서 목표 태그를 센다(계약 §1 표)."""
    ev, got = rec['events'], dict.fromkeys(TAGS, 0)
    model, prev = None, None
    for i, e in enumerate(ev):
        if e['event'] == 'push:enter':
            model, prev = e['model'], None
        elif e['event'] == 'factor:leave' and e['request'] == 10:
            prev = None  # 재분해 뒤는 새 구간
        elif e['event'] in ('step:enter', 'step:defer'):
            k = i - 1
            while ev[k]['event'] != 'supply:leave':
                k -= 1
            sl, j, n = ev[k], ev[k]['candidate'], e['state']['n']
            path = next(x for x in ev[k:i] if x['event'] == 'ratio:enter')['name']
            if prev == 'infeasible' and path == 'feasible':
                got['v-then-f'] += 1
            prev = path
            if e['event'] == 'step:defer':
                continue
            lb = [f64(x) for x in model['lb']]
            after, marks, idx = ev[i + 1]['state'], e['state']['marks'], sl['alpha']['indices'] or []
            assert ev[i + 1]['event'] == 'step:applied', '적용 사건 없음'
            if e['kind'] == 'pivot':
                r = ev[i + 1]['selected']['leave']
                lv = e['state']['header'][r]
                got['lb-enter'] += j < n and -1e30 < lb[j] and lb[j] != 0
                got['lb-leave-shift'] += lv < n and after['vstat'][lv] != -1 and lb[lv] != 0
                got['f-row-loop'] += any(marks[q] == '46' and q != r for q in idx)
            else:
                x = f64(e['state']['x'][j]) if j < n else 0.0
                got['free-flip'] += after['vstat'][j] == -3 and j < n and sl['alpha']['count'] > 0 and x != 0
                got['f-row-loop'] += any(marks[q] == '46' for q in idx)
    return got


def load(lp, kind):
    return [json.loads((ROOT / f'{D}/{lp}-{kind}-{i}.json').read_text()) for i in (1, 2, 3)]


def check(recs_by_case, manifest):
    """recs_by_case: {(lp, kind): [rec×3]}. 통과하면 태그 집계를 돌려준다."""
    out = {}
    for (lp, kind), recs in recs_by_case.items():
        name = f'{lp}-{kind}'
        case = manifest['cases'][name]
        for r in recs:
            assert r['binary_sha256'] == BINARY_SHA, (name, 'binary_sha256')
            assert r['tool_sha256'] == manifest['tools'][TOOL] == sha(TOOL), (name, 'tool_sha256')
            assert r['agent_sha256'] == manifest['tools'][AGENT] == sha(AGENT), (name, 'agent_sha256')
            assert r['model_sha256'] == case['model_sha256'] == sha(f'{D}/{lp}.lp'), (name, 'model_sha256')
            assert r['kind'] == kind and r['origin'] == 'natural' and not r['errors'], (name, '출처·오류')
            assert r['result'] and r['result']['status'] == 2 and r['reason'] == 'process-terminated', (name, '결과')
            assert r['events'], (name, '사건 없음')
            assert all(e['event'] != 'unsupported:x87' for e in r['events']), (name, 'x87 경로')
        for field in ('params', 'starts', 'events', 'result', 'output'):
            assert all(r[field] == recs[0][field] for r in recs[1:]), (name, '독립 3회 차이', field)
        extra = json.loads((ROOT / f'{D}/{lp}.extra.json').read_text())
        assert recs[0]['starts'] == extra['starts'], (name, '시작 벡터')
        t = tags(recs[0])
        out[name] = t
    for lp, need in LPS.items():
        for tag in need:
            total = sum(out[f'{lp}-{k}'][tag] for k in KINDS if f'{lp}-{k}' in out)
            assert total > 0, (lp, '목표 사건 0회', tag)
    return out


def all_cases():
    return {(lp, k): load(lp, k) for lp in LPS for k in KINDS}


def build_manifest(recs):
    cases = {}
    for (lp, kind), rs in recs.items():
        caps = [f'{D}/{lp}-{kind}-{i}.json' for i in (1, 2, 3)]
        cases[f'{lp}-{kind}'] = dict(role='target', kind=kind, model=f'{D}/{lp}.lp', model_sha256=sha(f'{D}/{lp}.lp'),
                                     extra=f'{D}/{lp}.extra.json', tool=TOOL, agent=AGENT, schema=rs[0]['schema'],
                                     captures=caps, capture_sha256=[sha(c) for c in caps],
                                     events_sha256=hashlib.sha256(json.dumps(rs[0]['events'], sort_keys=True).encode()).hexdigest())
    return dict(task='M3-04c-11', schema_version=1, original_version='13.0.3', binary_sha256=BINARY_SHA,
                float_encoding='IEEE-754 binary64 little-endian hex; exact bits including signed zero',
                environment=dict(frida_node='17.26.2', preload='tools/frida_resolve.cjs via tools/gurun.sh'),
                tools={p: sha(p) for p in (TOOL, AGENT, 'tools/push_chain_lp.py', 'tools/push_chain_oracle.py',
                                           'tools/frida_resolve.cjs')},
                inputs={p: sha(p) for lp in LPS for p in (f'{D}/{lp}.lp', f'{D}/{lp}.extra.json')},
                history={str(p.relative_to(ROOT)): sha(p.relative_to(ROOT)) for p in sorted((ROOT / D / 'history').iterdir())},
                cases=cases)


def self_test(recs, manifest):
    def bad(label, fn):
        r2 = copy.deepcopy(recs)
        fn(r2)
        try:
            check(r2, manifest)
        except AssertionError:
            return label
        raise AssertionError(('변조 미검출', label))

    def zero_lb(r):  # 목표 0회 입력: 모든 lb를 0으로 바꾸면 lb 태그가 0이 된다
        for k in KINDS:
            for rec in r[('shift', k)]:
                for e in rec['events']:
                    if e['event'] == 'push:enter':
                        e['model']['lb'] = ['0000000000000000'] * len(e['model']['lb'])

    def drop_steps(r):  # resolve 목표 0회: feasible 경로 ratio를 위반 경로로 바꿈
        for k in KINDS:
            for rec in r[('resolve', k)]:
                for e in rec['events']:
                    if e['event'] == 'ratio:enter':
                        e['name'] = 'infeasible'

    out = [bad('binary-zero', lambda r: r[('shift', 'primal')][0].__setitem__('binary_sha256', '0' * 64)),
           bad('events-empty', lambda r: r[('resolve', 'dual')][2].__setitem__('events', [])),
           bad('lb-zero', zero_lb),
           bad('v-then-f-zero', drop_steps),
           bad('run-differs', lambda r: r[('shift', 'dual')][1]['events'].pop()),
           bad('tool-sha', lambda r: r[('resolve', 'primal')][0].__setitem__('tool_sha256', '1' * 64)),
           bad('model-sha', lambda r: r[('shift', 'primal')][2].__setitem__('model_sha256', '2' * 64)),
           bad('x87', lambda r: r[('resolve', 'primal')][0]['events'].append({'event': 'unsupported:x87'}))]
    return out


def main():
    p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    g = p.add_mutually_exclusive_group(required=True)
    g.add_argument('--manifest', action='store_true')
    g.add_argument('--check', action='store_true')
    g.add_argument('--self-test', action='store_true')
    a = p.parse_args()
    try:
        recs = all_cases()
        path = ROOT / D / 'manifest.json'
        if a.manifest:
            m = build_manifest(recs)
            print(json.dumps(check(recs, m)))
            text = json.dumps(m, ensure_ascii=False, indent=1) + '\n'
            assert not path.exists() or path.read_text() == text, '기존 manifest와 다름'
            path.write_text(text)
            return
        m = json.loads(path.read_text())
        for name, c in m['cases'].items():
            assert [sha(x) for x in c['captures']] == c['capture_sha256'], (name, '캡처 파일 해시')
        if a.check:
            print(json.dumps(check(recs, m)))
        else:
            print(json.dumps({'rejected': self_test(recs, m)}))
    except (AssertionError, KeyError, StopIteration) as e:
        print('불합격: ' + str(e), file=sys.stderr)
        sys.exit(1)


if __name__ == '__main__':
    main()
