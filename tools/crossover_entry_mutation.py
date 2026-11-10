"""실제 저장 캡처 복사본의 변조가 CLI에서 거부되는지 확인한다."""
import copy
import json
import subprocess
import sys
import tempfile
from pathlib import Path

root = Path(__file__).resolve().parent.parent
base = root / 'bench/oracle/m3_crossover_entry'
records = [json.loads((base / f'barrier-{i}-final.json').read_text()) for i in range(1, 4)]


def no_events(d):
    d['events'] = []


def no_target(d):
    for e in d['events']:
        if e.get('event') == 'body:enter':
            e['mode'] = 1


def missing_x(d):
    next(e for e in d['events'] if e.get('event') == 'body:enter')['state']['result']['x'] = []


def wrong_prepared(d):
    e = next(e for e in d['events'] if e.get('event') == 'prepared')
    e['x'][0] = '0000000000000000' if e['x'][0] != '0000000000000000' else '0100000000000000'


cases = [
    ('binary0', lambda d: d.update(binary_sha256='0' * 64), 'binary_sha256'),
    ('input0', lambda d: d.update(model_sha256='0' * 64), 'model_sha256'),
    ('events0', no_events, 'events'),
    ('target0', no_target, '목표 body 사건 0회'),
    ('x누락', missing_x, 'result.x'),
    ('수치변조', wrong_prepared, 'prepared'),
    ('합성출처', lambda d: d.update(origin='synthetic'), '실제 실행 출처'),
]
with tempfile.TemporaryDirectory(prefix='mutation-', dir=base) as directory:
    paths = [Path(directory) / f'{i}.json' for i in range(3)]
    for name, mutate, expected in cases:
        data = copy.deepcopy(records)
        mutate(data[1])
        for path, record in zip(paths, data):
            path.write_text(json.dumps(record))
        result = subprocess.run([sys.executable, str(root / 'tools/crossover_entry_check.py'), *map(str, paths)],
                                cwd=root, text=True, capture_output=True, check=False)
        if result.returncode == 0 or expected not in result.stderr:
            raise AssertionError(f'{name}: rc={result.returncode}, stderr={result.stderr}')
        print(f'{name}: 종료 {result.returncode}, {result.stderr.strip()}')
