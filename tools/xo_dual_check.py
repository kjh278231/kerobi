#!/usr/bin/env python3
"""M3-04f-4a: 251b4f·BTRAN 원본 캡처의 식별·재현성 검사.

  python3 tools/xo_dual_check.py --manifest      # bench/oracle/m3_xo_dual/manifest.json 작성(캡처 뒤 한 번)
  python3 tools/xo_dual_check.py --controls      # 해시·스키마·성공·사례별 3회 사건열(seq 제외) 동일, 분기 집계
  python3 tools/xo_dual_check.py --self-test     # 변조(한 run의 값·사건 삭제·해시)가 거부되는지
"""
import argparse, collections, copy, gzip, hashlib, json, sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
D = ROOT / 'bench/oracle/m3_xo_dual'
SCHEMA = 'M3-04f-4a-v1'
BINARY = '8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c'
TOOLS = ('tools/xo_dual_agent.js', 'tools/xo_dual_capture.js', 'tools/xo_dual_runs.py')
SITES = {0x1c6b49: 'dpush-a', 0x1c8533: 'dpush-b', 0x1c87a9: 'dpush-c', 0x1bf09e: 'post', 0x1bfae2: 'dualcheck'}


class Fail(Exception):
    pass


def require(c, msg):
    if not c:
        raise Fail(msg)


def sha(p):
    return hashlib.sha256(Path(p).read_bytes()).hexdigest()


def files():
    out = collections.defaultdict(dict)
    for p in sorted(D.glob('*/*.json.gz')):
        stem = p.name[:-len('.json.gz')]
        name, rep = stem.rsplit('-', 1)
        out[f'{p.parent.name}/{name}'][int(rep)] = p
    return dict(out)


def write_manifest():
    fs = files()
    failed = sorted(str(p.relative_to(D)) for p in D.glob('*/*.failed'))
    m = dict(schema=SCHEMA, binary_sha256=BINARY, tools={t: sha(ROOT / t) for t in TOOLS},
             failed=failed, files={str(p.relative_to(D)): sha(p) for c in fs.values() for p in c.values()})
    (D / 'manifest.json').write_text(json.dumps(m, indent=1, sort_keys=True) + '\n')
    print('manifest', len(m['files']), '파일, 실패', len(failed))


def load_manifest():
    m = json.loads((D / 'manifest.json').read_text())
    require(m['schema'] == SCHEMA, 'manifest schema')
    require(m['binary_sha256'] == BINARY, 'manifest binary')
    require(not m['failed'], f'실패 캡처 {m["failed"]}')
    return m


def check_files(m):
    fs = files()
    seen = {str(p.relative_to(D)) for c in fs.values() for p in c.values()}
    require(seen == set(m['files']), f'파일 목록 불일치 {sorted(seen ^ set(m["files"]))[:5]}')
    for rel, h in m['files'].items():
        require(sha(D / rel) == h, f'파일 해시 불일치 {rel}')


def load(p):
    return json.loads(gzip.decompress(Path(p).read_bytes()))


def records(m):
    out = {}
    for case, reps in files().items():
        rs = []
        for k in sorted(reps):
            r = load(reps[k])
            require(r['schema'] == SCHEMA, f'{case}-{k}: schema')
            require(r['binary_sha256'] == BINARY, f'{case}-{k}: binary')
            require(r['agent_sha256'] == m['tools']['tools/xo_dual_agent.js'], f'{case}-{k}: agent 해시')
            require(r['reason'] == 'process-terminated' and not r['errors'] and r['result'] is not None, f'{case}-{k}: 실패 실행')
            rs.append(r)
        out[case] = rs
    return out


def strip(r):
    return [{k: v for k, v in e.items() if k != 'seq'} for e in r['events']]


def controls(data):
    for case, rs in data.items():
        require(len(rs) == 3, f'{case}: 반복 {len(rs)}')
        base = strip(rs[0])
        require(any(e['event'] == 'dv:enter' for e in base), f'{case}: 251b4f 사건 0회')
        for k, r in enumerate(rs[1:], 2):
            other = strip(r)
            require(len(other) == len(base), f'{case}: run {k} 사건 수 {len(other)} != {len(base)}')
            for i, (a, b) in enumerate(zip(base, other)):
                if a != b:
                    key = next(x for x in sorted(set(a) | set(b)) if a.get(x) != b.get(x))
                    raise Fail(f'{case}: run {k} 사건 {i} {a["event"]} 필드 {key} 다름')
            require(r['result'] == rs[0]['result'], f'{case}: run {k} 결과 다름')


def calls(r):
    """251b4f 호출별 (enter, btran enter, btran leave, leave, 뒤따르는 sel:enter, sel:decide)."""
    out, cur = [], None
    ev = r['events']
    for i, e in enumerate(ev):
        t = e['event']
        if t == 'dv:enter':
            require(cur is None, '251b4f 중첩')
            cur = dict(enter=e, bt_enter=None, bt_leave=None, leave=None, sel=None, decide=None, index=i)
        elif t == 'dv:btran:enter':
            require(cur and cur['bt_enter'] is None and e['id'] == cur['enter']['id'], 'btran enter 순서')
            cur['bt_enter'] = e
        elif t == 'dv:btran:leave':
            require(cur and cur['bt_leave'] is None and e['id'] == cur['enter']['id'], 'btran leave 순서')
            cur['bt_leave'] = e
        elif t == 'dv:leave':
            require(cur and e['id'] == cur['enter']['id'], 'leave 순서')
            cur['leave'] = e
            site = SITES.get(cur['enter']['ret_site'])
            require(site is not None, f'알 수 없는 호출 지점 {cur["enter"]["ret_site"]:#x}')
            cur['site'] = site
            rest = ev[i + 1:]
            if site == 'post':
                nxt = next((x for x in rest if x['event'] in ('sel:enter', 'dv:enter', 'xo:leave')), None)
                require(nxt and nxt['event'] == 'sel:enter', '뒤처리 호출 뒤 sel:enter 없음')
                cur['sel'] = nxt
            if site == 'dualcheck':
                sel = [x for x in ev[:i] if x['event'] == 'sel:enter']
                require(sel, '쌍대 검사 전 sel:enter 없음')
                cur['sel'] = sel[-1]
                nxt = next((x for x in rest if x['event'] in ('sel:decide', 'dv:enter', 'xo:leave')), None)
                require(nxt and nxt['event'] == 'sel:decide', '쌍대 검사 뒤 sel:decide 없음')
                cur['decide'] = nxt
            for k in ('bt_enter', 'bt_leave'):
                require(cur[k] is not None, f'{k} 누락')
            out.append(cur)
            cur = None
    require(cur is None, '반환 없는 251b4f')
    return out


def coverage(data):
    c = collections.Counter()
    for case, rs in data.items():
        for k in calls(rs[0]):
            c['call'] += 1
            c['site:' + k['site']] += 1
            e = k['enter']
            if e['sub']:
                c['corrected'] += 1
                c['corrections'] += len(e['sub'])
            c['bt_out_count:' + ('dense' if k['bt_enter']['output_count'] < 0 else 'sparse')] += 1
            c['etas>0'] += k['bt_enter']['factor']['scalars']['x180'] > 0
            c['levels>=0'] += k['bt_enter']['factor']['Lrow']['levels'] >= 0
            c['work'] += e['work'] is not None
            if k['decide']:
                c[f'r14={k["decide"]["r14"]}'] += 1
    return c


def self_test(m, data):
    case = next(c for c, rs in data.items() if any(x['site'] == 'dualcheck' for x in calls(rs[0])))
    bad = 0

    def expect_fail(name, f):
        nonlocal bad
        d = copy.deepcopy({case: data[case]})
        f(d[case])
        try:
            controls(d)
        except Fail as e:
            print('거부', name, '-', e)
            return
        bad += 1
        print('생존', name)

    def flip_d(rs):
        e = next(x for x in rs[1]['events'] if x['event'] == 'dv:leave')
        e['d'][0] = format(int(e['d'][0], 16) ^ 1, '016x')

    def drop_btran(rs):
        rs[2]['events'] = [x for x in rs[2]['events'] if x['event'] != 'dv:btran:leave']

    def r14(rs):
        e = next(x for x in rs[1]['events'] if x['event'] == 'sel:decide')
        e['r14'] ^= 1

    def no_dv(rs):
        for r in rs:
            r['events'] = [x for x in r['events'] if not x['event'].startswith('dv')]

    def two_runs(rs):
        rs.pop()

    for name, f in (('d 비트', flip_d), ('btran leave 삭제', drop_btran), ('r14', r14), ('251b4f 0회', no_dv), ('반복 2회', two_runs)):
        expect_fail(name, f)
    mm = copy.deepcopy(m)
    k0 = next(iter(mm['files']))
    mm['files'][k0] = '0' * 64
    try:
        check_files(mm)
        bad += 1
        print('생존 파일 해시')
    except Fail as e:
        print('거부 파일 해시 -', e)
    require(not bad, f'변조 생존 {bad}')


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--manifest', action='store_true')
    ap.add_argument('--controls', action='store_true')
    ap.add_argument('--self-test', action='store_true')
    a = ap.parse_args()
    if a.manifest:
        write_manifest()
    m = load_manifest()
    check_files(m)
    data = records(m)
    if a.controls:
        controls(data)
        cov = coverage(data)
        print('controls 통과', len(data), '사례 ×3', dict(sorted(cov.items())))
    if a.self_test:
        self_test(m, data)
        print('self-test 통과')
    return 0


if __name__ == '__main__':
    try:
        sys.exit(main())
    except Fail as e:
        print('실패:', e)
        sys.exit(1)
