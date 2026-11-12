"""M3-04c-21h: `036ade` 블록 상태(S+0x2c0/0x4e0/0x4e4) 캡처 검사기.

controls   : manifest 해시, 7사례 × 3회 `lu:block` 사건 정확 일치, `lu:block`을 뺀 사건·결과·출력 = 21e 정식 `*-1`(비간섭).
acceptance : push 재분해(`lu:refactor` enter)마다 바로 앞에 push gate가 켜진 `lu:block` enter가 하나 있고 id가 맞는다.
             반환값 0, S+0x4e0은 그대로이거나 +1, S+0x4e4는 그대로·+1·−1·2e9 중 하나(C:1494–1510).
             push 재분해 표(진입 0x2c0/0x4e0/0x4e4, 반환 0x4e0/0x4e4)를 state.json으로 쓴다(--write).
--self-test: 메모리 변조가 각 검사에서 거부되는지 확인한다.
"""
import copy
import gzip
import hashlib
import json
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[1]
D = ROOT / 'bench/oracle/m3_push_block'
BASE = ROOT / 'bench/oracle/m3_push_lu_chain'
CASES = ('arms', 'retry', 'interior', 'control', 'adlittle', 'scsd1', 'capri')
BINARY = '8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c'
TOOLS = ('tools/push_block_agent.js', 'tools/push_block_capture.js', 'tools/push_block_runs.py', 'tools/push_lu_runs.py',
         'tools/frida_resolve.cjs', 'tools/gurun.sh', 'tools/push_block_check.py')


class Fail(Exception):
    pass


def sha(p):
    return hashlib.sha256(Path(p).read_bytes()).hexdigest()


def path_of(d, name, i):
    p = d / f'{name}-{i}.json'
    return p if p.exists() else d / f'{name}-{i}.json.gz'


def load(p):
    b = p.read_bytes()
    return json.loads(gzip.decompress(b) if p.suffix == '.gz' else b)


def load_all():
    caps = {(c, i): load(path_of(D, c, i)) for c in CASES for i in (1, 2, 3)}
    base = {c: load(path_of(BASE, c, 1)) for c in CASES}
    return caps, base


def blocks(cap):
    return [e for e in cap['events'] if e['event'] == 'lu:block']


def controls(caps, base, manifest=True):
    if manifest:
        m = json.loads((D / 'manifest.json').read_text())
        for k, v in {**m['tools'], **m['captures']}.items():
            if sha(ROOT / k) != v:
                raise Fail(f'manifest 해시 불일치 {k}')
    for c in CASES:
        ref = None
        for i in (1, 2, 3):
            cap = caps[(c, i)]
            if cap['binary_sha256'] != BINARY or cap['schema'] != 'M3-04c-21h-v1' or cap['errors'] or cap['reason'] != 'process-terminated':
                raise Fail(f'{c}-{i}: 바이너리·스키마·오류·종료 사유')
            bl = blocks(cap)
            if c != 'control' and not bl:
                raise Fail(f'{c}-{i}: lu:block 사건 없음')
            if ref is None:
                ref = bl
            elif bl != ref:
                k = next((j for j, (x, y) in enumerate(zip(bl, ref)) if x != y), min(len(bl), len(ref)))
                raise Fail(f'{c}-{i}: 회차 차이 lu:block #{k}: {bl[k] if k < len(bl) else None} != {ref[k] if k < len(ref) else None}')
            proj = [e for e in cap['events'] if e['event'] != 'lu:block']
            b = base[c]
            if proj != b['events']:
                k = next((j for j, (x, y) in enumerate(zip(proj, b['events'])) if x != y), min(len(proj), len(b['events'])))
                raise Fail(f'{c}-{i}: 21e 투영 차이 사건 #{k}')
            for f in ('result', 'output', 'params', 'starts', 'model_sha256', 'kind'):
                if cap[f] != b[f]:
                    raise Fail(f'{c}-{i}: 21e {f} 차이')
    return True


def table(cap, name):
    """push 재분해마다 (lu id, 진입 상태, 반환 상태)와 push 밖 호출 목록."""
    ev = cap['events']
    rows, outside = [], []
    pend = {}
    for j, e in enumerate(ev):
        if e['event'] == 'lu:block' and e['phase'] == 'enter':
            pend[e['id']] = e
            if e['push']:
                nxt = ev[j + 1] if j + 1 < len(ev) else None
                if not (nxt and nxt['event'] == 'lu:refactor' and nxt['phase'] == 'enter' and nxt['id'] == e['lu']):
                    raise Fail(f'{name}: push lu:block #{e["id"]} 뒤에 같은 lu의 lu:refactor enter가 없음')
                if not e['s_active'] or e['ret'] != 0xf34f2:
                    raise Fail(f'{name}: push lu:block #{e["id"]}의 S·반환 주소')
        elif e['event'] == 'lu:block' and e['phase'] == 'leave':
            st = pend.pop(e['id'], None)
            if st is None:
                raise Fail(f'{name}: 짝 없는 lu:block leave #{e["id"]}')
            if e['ret'] != 0:
                raise Fail(f'{name}: 036ade 반환 {e["ret"]}')
            a, b = st['state'], e['state']
            if b['x4e0'] not in (a['x4e0'], a['x4e0'] + 1):
                raise Fail(f'{name}: S+0x4e0 {a["x4e0"]} → {b["x4e0"]}')
            if b['x4e4'] not in (a['x4e4'], a['x4e4'] + 1, -1, 2000000000):
                raise Fail(f'{name}: S+0x4e4 {a["x4e4"]} → {b["x4e4"]}')
            r = {'block': st['id'], 'lu': st['lu'], 'enter': a, 'leave': b, 'm': st['m']}
            (rows if st['push'] else outside).append(r)
        elif e['event'] == 'lu:refactor' and e['phase'] == 'enter':
            prev = ev[j - 1] if j else None
            if not (prev and prev['event'] == 'lu:block' and prev['phase'] == 'enter' and prev['push'] and prev['lu'] == e['id']):
                raise Fail(f'{name}: lu:refactor #{e["id"]} 앞에 push lu:block 없음')
    if pend:
        raise Fail(f'{name}: 반환 없는 lu:block {sorted(pend)}')
    return rows, outside


def acceptance(caps):
    out = {}
    for c in CASES:
        if c == 'control':
            if blocks(caps[(c, 1)]):
                raise Fail('control: push 밖 대조군에 lu:block 사건이 있음(crossover 없음 기대)')
            continue
        rows, outside = table(caps[(c, 1)], c)
        if not rows:
            raise Fail(f'{c}: push 재분해 없음')
        # push 재분해 사이에 push 밖 036ade가 끼었는지(같은 S에서): 블록 id가 연속인지로 본다
        ids = [r['block'] for r in rows]
        between = [o['block'] for o in outside if ids[0] < o['block'] < ids[-1]]
        out[c] = {'push': rows, 'outside_between': between, 'outside': len(outside)}
    return out


def tampers(caps, base):
    yield '바이너리 해시', lambda c, b: c[('arms', 2)].__setitem__('binary_sha256', '0' * 64), 'c'
    yield '빈 사건', lambda c, b: c[('arms', 1)].__setitem__('events', [e for e in c[('arms', 1)]['events'] if e['event'] != 'lu:block']), 'c'
    yield '회차 차이 0x4e0', lambda c, b: blocks(c[('capri', 3)])[4]['state'].__setitem__('x4e0', 7), 'c'
    yield '회차 차이 0x4e4', lambda c, b: blocks(c[('retry', 2)])[0]['state'].__setitem__('x4e4', 2000000000), 'c'
    yield '회차 차이 0x2c0', lambda c, b: blocks(c[('interior', 3)])[0]['state'].__setitem__('x2c0', 9), 'c'
    yield '투영 차이', lambda c, b: b['scsd1']['events'][3].__setitem__('step', -5), 'c'
    yield '결과 차이', lambda c, b: b['adlittle'].__setitem__('result', None), 'c'
    yield '오류 기록', lambda c, b: c[('capri', 2)]['errors'].append('x'), 'c'

    def all3(c, f):
        for i in (1, 2, 3):
            f(c[('capri', i)])
    yield '0x4e0 +2', lambda c, b: all3(c, lambda x: blocks(x)[1]['state'].__setitem__('x4e0', blocks(x)[0]['state']['x4e0'] + 2)), 'a'
    yield '0x4e4 임의값', lambda c, b: all3(c, lambda x: blocks(x)[1]['state'].__setitem__('x4e4', 5)), 'a'
    yield '반환≠0', lambda c, b: all3(c, lambda x: blocks(x)[1].__setitem__('ret', -1)), 'a'
    yield 'push gate 끔', lambda c, b: all3(c, lambda x: blocks(x)[0].__setitem__('push', False)), 'a'
    yield 'lu id 어긋남', lambda c, b: all3(c, lambda x: blocks(x)[0].__setitem__('lu', 99)), 'a'
    yield 'leave 누락', lambda c, b: all3(c, lambda x: x['events'].remove(blocks(x)[1])), 'a'


def self_test():
    caps0, base0 = load_all()
    bad = 0
    for name, f, kind in tampers(caps0, base0):
        caps, base = copy.deepcopy(caps0), copy.deepcopy(base0)
        f(caps, base)
        try:
            controls(caps, base, manifest=False)
            acceptance(caps)
            print('NOT REJECTED', name)
            bad += 1
        except Fail as e:
            print('rejected', name, '|', e)
    print('self-test', '실패' if bad else '통과', bad)
    return bad == 0


def write_manifest():
    caps = sorted(p for p in D.iterdir() if p.name.endswith(('.json', '.json.gz')) and p.name not in ('manifest.json', 'state.json'))
    m = {'task': 'M3-04c-21h', 'binary_sha256': BINARY,
         'tools': {t: sha(ROOT / t) for t in TOOLS},
         'captures': {str(p.relative_to(ROOT)): sha(p) for p in caps},
         'baseline': 'bench/oracle/m3_push_lu_chain (21e manifest)'}
    (D / 'manifest.json').write_text(json.dumps(m, indent=1, ensure_ascii=False) + '\n')


def main():
    a = sys.argv[1:]
    if '--write-manifest' in a:
        write_manifest()
    ok = True
    if '--controls' in a or '--acceptance' in a or '--write' in a:
        caps, base = load_all()
        try:
            controls(caps, base)
            print('controls 통과: 7사례 × 3회 lu:block 일치, 21e 투영 일치')
            acc = acceptance(caps)
            for c, v in acc.items():
                s = ' '.join(f"{r['lu']}:{r['enter']['x2c0']}/{r['enter']['x4e0']}/{r['enter']['x4e4']}→{r['leave']['x4e0']}/{r['leave']['x4e4']}" for r in v['push'])
                print(f'  {c}: push {len(v["push"])} [{s}] push 밖 {v["outside"]} (사이 {v["outside_between"]})')
            print('acceptance 통과')
            if '--write' in a:
                (D / 'state.json').write_text(json.dumps(acc, indent=1) + '\n')
        except Fail as e:
            print('실패:', e)
            ok = False
    if '--self-test' in a:
        ok = self_test() and ok
    sys.exit(0 if ok else 1)


if __name__ == '__main__':
    main()
