#!/usr/bin/env python3
"""대조군 재현과 자연 목표 acceptance를 별도로 판정한다."""
import argparse
import copy
import hashlib
import json
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parent.parent
OLD = ROOT / 'bench/oracle/mip_linear_child_trace'
EXP = ROOT / 'bench/oracle/mip_linear_child_experiments'
BINARY = '8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c'
OLD_MANIFEST_SHA = 'b072ff49f9222d12e9f971ac23d0b1d88c86ef6de986c12a34f4bb7bcc213887'
PARAMS = {'Threads': 1, 'Presolve': 0, 'Cuts': 0, 'Heuristics': 0, 'Seed': 0, 'NodeLimit': 100, 'LogToConsole': 0}


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def require(condition, message):
    if not condition:
        raise ValueError(message)


def validate(records, mode, expected_model):
    require(len(records) == 3, f'{mode}: 독립 실행 수가 3이 아님')
    for i, j in enumerate(records, 1):
        case = f'{mode} run {i}'
        require(j.get('binary_sha256') == BINARY, f'{case}: 원본 바이너리 해시 불일치')
        require(j.get('model_sha256') == expected_model, f'{case}: 모델 해시 불일치')
        require(j.get('params') == PARAMS, f'{case}: 파라미터 불일치')
        events = j.get('events')
        require(isinstance(events, list) and len(events) > 0, f'{case}: 필수 사건 없음')
        require(all(isinstance(e, dict) and isinstance(e.get('event'), str) for e in events), f'{case}: 사건 형식 오류')
        names = [e['event'] for e in events]
        require('solve:enter' in names and 'normalize_before' in names and 'normalize_after' in names, f'{case}: 정규화 필수 사건 누락')
        if mode == 'root_unbounded':
            require(j.get('output') == 'DONE 5 1', f'{case}: 루트 무계 출력 불일치')
            require(any(e['event'] == 'normalize_before' and e.get('lp_status') == 5 and e.get('node', {}).get('root') is True for e in events), f'{case}: 루트 LP 상태 5 누락')
        elif mode in ('fractional_branch', 'natural', 'inject_reclass'):
            require(j.get('output') == 'DONE 2 3', f'{case}: 유한 모델 출력 불일치')
            require(any(e['event'] == 'solve:enter' and e.get('node', {}).get('depth') == 1 and e['node'].get('parent_is_root') is True and e['node'].get('root') is False for e in events), f'{case}: 실제 자식 없음')
        if mode in ('fractional_branch', 'natural', 'inject_reclass'):
            count = 3 if mode == 'fractional_branch' else 4
            nodes = [e['node'] for e in events if e['event'] == 'solve:enter']
            for n in nodes:
                for k in ('domain_lower', 'domain_upper'):
                    require(isinstance(n.get(k), list) and len(n[k]) == count, f'{case}: {k} 열 개수 오류')
            root = next(n for n in nodes if n['root'])
            require(any(n['parent_is_root'] and (n['domain_lower'], n['domain_upper']) != (root['domain_lower'], root['domain_upper']) for n in nodes), f'{case}: 자식 bound 변화 없음')
        if mode.startswith('inject_') or mode == 'natural':
            require(j.get('schema') == 2 and j.get('mode') == mode, f'{case}: 실험 식별 오류')
        if mode == 'inject_reclass':
            inject = next((k for k, e in enumerate(events) if e['event'] == 'inject_reclass'), -1)
            require(inject >= 0 and events[inject]['before'] == 2 and events[inject]['after'] == 5, f'{case}: 주입 누락')
            require(events[inject+1]['event'] == 'normalize_after' and events[inject+1]['lp_status'] == 12, f'{case}: 재분류 누락')
            require(events[inject+2]['event'] == 'retry' and events[inject+3]['event'] == 'simplex:enter' and events[inject+4]['event'] == 'simplex:leave', f'{case}: 재풀이 순서 오류')
            require(events[inject+4]['ret'] == 0, f'{case}: 재풀이 실패')
            enter = events[inject+2]['lp']; call = events[inject+2]['solve_call']
            out = next((e for e in events[inject+5:] if e['event'] == 'solve:leave' and e['call'] == call), None)
            require(out is not None, f'{case}: 같은 solve 반환 누락')
            for key in ('presolve', 'method', 'concurrent_method', 'threads'):
                require(enter[key] == out['lp'][key], f'{case}: {key} 복원 실패')
        if mode == 'inject_branch':
            require(j.get('ok') is False and names.count('inject_branch') == 1 and names[-1] in ('inject_branch', 'create:enter'), f'{case}: 주입 실패 경계 불일치')
    for i in (1, 2):
        if mode == 'inject_branch':
            def prefix(r):
                e=r['events']; k=next(i for i, x in enumerate(e) if x['event']=='inject_branch'); return e[:k+1]
            require(prefix(records[0]) == prefix(records[i]), f'{mode}: 주입 이전의 사건·필드 불일치')
        else:
            require(records[0] == records[i], f'{mode}: run 1과 {i+1}의 사건·필드 불일치')


def read3(folder, name):
    return [json.loads((folder / f'{name}_{i}.json').read_text()) for i in range(1, 4)]


def check_manifests(old_dir):
    oldmanifest = ROOT / 'bench/oracle/mip_linear_child_manifest.json'
    require(sha(oldmanifest) == OLD_MANIFEST_SHA, '기존 manifest 해시 불일치')
    old = json.loads(oldmanifest.read_text())
    require(old['binary_sha256'] == BINARY, '기존 manifest 바이너리 불일치')
    for path, digest in old['files'].items():
        file = old_dir / Path(path).name if path.startswith('bench/oracle/mip_linear_child_trace/') else ROOT / ('bench/oracle/mip_linear_child_check_3e9fc00.py.txt' if path == 'tools/mip_linear_child_check.py' else path)
        require(sha(file) == digest, f'기존 manifest 파일 해시 불일치: {path}')
    for path, digest in old['external_sources'].items():
        require(sha(ROOT.parent / path) == digest, f'외부 원본 해시 불일치: {path}')
    manifest = json.loads((ROOT / 'bench/oracle/mip_linear_child_experiments_manifest.json').read_text())
    require(manifest['binary_sha256'] == BINARY and manifest['params'] == PARAMS, '실험 manifest 식별 불일치')
    for path, digest in manifest['files'].items():
        require(sha(ROOT / path) == digest, f'실험 manifest 해시 불일치: {path}')
    binary = ROOT.parent / '.venv/lib/python3.13/site-packages/gurobipy/.libs/libgurobi130.so'
    require(sha(binary) == BINARY, '설치된 원본 바이너리 해시 불일치')
    return old, manifest


def selftest(old):
    model = old['files']['bench/mip_linear_child/root_unbounded.lp']
    base = read3(OLD, 'root_unbounded')
    corruptions = ('all_zero_hash_and_empty_root', 'zero_hash', 'empty_root', 'status', 'sequence', 'missing_field')
    for label in corruptions:
        data = copy.deepcopy(base)
        if label in ('all_zero_hash_and_empty_root', 'zero_hash'):
            for j in data:
                j['binary_sha256'] = '0' * 64
        if label in ('all_zero_hash_and_empty_root', 'empty_root'):
            for j in data:
                j['events'] = []
        if label == 'status':
            for j in data:
                next(e for e in j['events'] if e['event'] == 'normalize_before')['lp_status'] = 2
        if label == 'sequence':
            data[1]['events'].reverse()
        if label == 'missing_field':
            for j in data:
                del j['params']
        try:
            validate(data, 'root_unbounded', model)
        except (ValueError, KeyError, StopIteration, IndexError) as e:
            print(f'변조 거부 {label}: {e}')
        else:
            raise ValueError(f'변조를 허용함: {label}')
    # 독립 검토자의 전체 6개 파일 변조를 파일 입력 경로에서도 검사한다.
    import tempfile
    with tempfile.TemporaryDirectory() as t:
        folder = Path(t)
        for name in ('fractional_branch', 'root_unbounded'):
            records = read3(OLD, name)
            for i, j in enumerate(records, 1):
                j['binary_sha256'] = '0' * 64
                if name == 'root_unbounded':
                    j['events'] = []
                (folder / f'{name}_{i}.json').write_text(json.dumps(j))
        try:
            check_manifests(folder)
        except ValueError as e:
            print(f'변조 거부 6파일 실제 경로: {e}')
        else:
            raise ValueError('6파일 변조를 허용함')


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('action', choices=('controls', 'acceptance'))
    parser.add_argument('--self-test', action='store_true')
    parser.add_argument('--controls-dir', type=Path, default=OLD)
    args = parser.parse_args()
    old, manifest = check_manifests(args.controls_dir)
    for name in ('fractional_branch', 'root_unbounded'):
        validate(read3(args.controls_dir, name), name, old['files'][f'bench/mip_linear_child/{name}.lp'])
    model = manifest['files']['bench/mip_linear_child/continuous_candidate.lp']
    experiments = {}
    for mode in ('natural', 'inject_reclass', 'inject_branch'):
        experiments[mode] = read3(EXP, mode)
        validate(experiments[mode], mode, model)
    print('대조군: 기존 2종 및 신규 자연 1종 각 3회 일치; 합성 재분류 3회 일치; 합성 분기 3회 비정상 종료 (주입까지 일치, 이후 사건 수 불일치)')
    if args.self_test:
        selftest(old)
    natural = experiments['natural'][0]['events']
    child5 = sum(e['event'] == 'normalize_before' and e.get('lp_status') == 5 and not e['node']['root'] for e in natural)
    status3 = sum(e['event'] == 'branch:enter' and e['node']['status'] == 3 for e in natural)
    child10 = sum(e['event'] == 'set:leave' and e.get('value') == 10 for e in natural)
    print(f'목표 사건: 자연 자식 상태5={child5}, 자연 상태3 분기={status3}, 자연 새 자식 상태10={child10}; 합성 재분류=1/실행, 합성 분기=실패; 필수 상태 미검증=1 이상')
    if args.action == 'acceptance':
        missing = []
        if not child5: missing.append('자연 자식 LP 상태 5')
        if not status3: missing.append('자연 상태 3 분기')
        if not child10: missing.append('자연 새 자식 상태 10')
        # shortcut: 현재 스키마에는 factor/norm 및 분기 연결을 검증할 수 있는 자료가 없다.
        missing.append('factor/norm 전체와 동일 노드의 재분류·재풀이·자식·gate 연결')
        print('목표 계약 실패: ' + ', '.join(missing), file=sys.stderr)
        return 1
    return 0


if __name__ == '__main__':
    try:
        sys.exit(main())
    except (ValueError, OSError, KeyError, TypeError, IndexError, StopIteration, json.JSONDecodeError) as exc:
        print(f'검사 실패: {exc}', file=sys.stderr)
        sys.exit(1)
