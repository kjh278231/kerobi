"""M3-04b-3: bench/oracle/m3_push_arms/manifest.json을 캡처·도구·입력·기록 파일에서 다시 쓴다(캡처 뒤, 검사 전에 실행)."""
import json, sys, hashlib
from pathlib import Path
ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / 'tools'))
import primal_push_step_check as step
D = 'bench/oracle/m3_push_arms'
sha = lambda p: hashlib.sha256((ROOT / p).read_bytes()).hexdigest()
TOOL, AGENT = 'tools/primal_push_arm_capture.js', 'tools/primal_push_arm_agent.js'
extra = json.loads((ROOT / D / 'arms.extra.json').read_text())
base = {'Threads': 1, 'Seed': 0, 'OutputFlag': 0, 'Presolve': 0, 'Method': 2, 'LPWarmStart': 2, **extra['params']}
cases = {}
for kind, xo, role in (('primal', 3, 'target'), ('dual', 1, 'target'), ('control', 0, 'control')):
    caps = [f'{D}/arms-{kind}-{i}.json' for i in (1, 2, 3)]
    rec = json.loads((ROOT / caps[0]).read_text())
    cases[f'arms-{kind}'] = dict(role=role, kind=kind, model=f'{D}/arms.lp', model_sha256=sha(f'{D}/arms.lp'),
        params={k: v for k, v in rec['params'].items()}, starts=extra['starts'], tool=TOOL, agent=AGENT, schema='M3-04b-3-v1',
        captures=caps, events_sha256=step.digest(rec['events']), capture_sha256=[sha(c) for c in caps])
    assert rec['params'] == dict(base, Crossover=xo) | {'Crossover': xo}, rec['params']
cases['face5-noninterference'] = dict(role='noninterference', kind='primal', model='bench/oracle/m3_push_decision/face5.lp',
    model_sha256=sha('bench/oracle/m3_push_decision/face5.lp'), tool=TOOL, agent=AGENT,
    captures=[f'{D}/face5-primal-noninterference.json'], compare='bench/oracle/m3_push_decision/face5-primal-1.json',
    capture_sha256=[sha(f'{D}/face5-primal-noninterference.json')])
for kind in ('primal', 'dual'):
    cases[f'arms-{kind}-old-agent'] = dict(role='noninterference-old-agent', kind=kind, model_sha256=sha(f'{D}/arms.lp'), agent=AGENT,
        captures=[f'{D}/history/arms-probe-{kind}.json'], compare=f'{D}/arms-{kind}-1.json')
hist = sorted(p.relative_to(ROOT).as_posix() for p in (ROOT / D / 'history').iterdir())
m = dict(task='M3-04b-3', schema_version=1, float_encoding='IEEE-754 binary64 little-endian hex; exact bits including signed zero',
    original_version='13.0.3', binary_sha256=step.BINARY_SHA,
    tools={p: sha(p) for p in (TOOL, AGENT, 'tools/push_arm_lp.py', 'tools/primal_push_decision_check.py', 'tools/primal_push_decision_rust.py', 'tools/push_arm_manifest.py')},
    records={p: sha(p) for p in (f'{D}/model-mutation.json', f'{D}/rust-mutation.json', 'bench/oracle/m3_push_decision_rust/input.txt')},
    sources={'../re/decomp/c/001c0070_PRIVATE0000000000134a87.c': sha('../re/decomp/c/001c0070_PRIVATE0000000000134a87.c')},
    inputs={p: sha(p) for p in (f'{D}/arms.lp', f'{D}/arms.extra.json')},
    history={p: sha(p) for p in hist}, cases=cases)
(ROOT / D / 'manifest.json').write_text(json.dumps(m, ensure_ascii=False, indent=1) + '\n')
print('ok', len(hist))
