"""M3-04c-7: bench/oracle/m3_push_retry/manifest.json을 캡처·도구·입력·기록 파일에서 다시 쓴다(캡처 뒤, 검사 전에 실행)."""
import hashlib
import json
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / 'tools'))
import primal_push_step_check as step  # noqa: E402

D = 'bench/oracle/m3_push_retry'
V = 'bench/oracle/m3_push_violated'
N = f'{D}/noninterference'
sha = lambda p: hashlib.sha256((ROOT / p).read_bytes()).hexdigest()  # noqa: E731
TOOL, AGENT = 'tools/primal_push_retry_capture.js', 'tools/primal_push_retry_agent.js'
OLD_TOOL, OLD_AGENT = 'tools/primal_push_violated_capture.js', 'tools/primal_push_violated_agent.js'
extra = json.loads((ROOT / D / 'retry.extra.json').read_text())
base = {'Threads': 1, 'Seed': 0, 'OutputFlag': 0, 'Presolve': 0, 'Method': 2, 'LPWarmStart': 2, **extra['params']}
cases = {}
for kind, xo, role in (('primal', 3, 'target'), ('dual', 1, 'target'), ('control', 0, 'control')):
    caps = [f'{D}/retry-{kind}-{i}.json' for i in (1, 2, 3)]
    rec = json.loads((ROOT / caps[0]).read_text())
    assert rec['params'] == dict(base, Crossover=xo), rec['params']
    cases[f'retry-{kind}'] = dict(role=role, kind=kind, model=f'{D}/retry.lp', model_sha256=sha(f'{D}/retry.lp'),
                                  params=rec['params'], starts=extra['starts'], tool=TOOL, agent=AGENT,
                                  schema='M3-04c-7-v1', captures=caps, events_sha256=step.digest(rec['events']),
                                  capture_sha256=[sha(c) for c in caps])
for kind in ('primal', 'dual'):
    old = f'{N}/retry-{kind}-violatedagent.json'
    cases[f'retry-{kind}-violatedagent'] = dict(role='noninterference', firing=True, kind=kind,
                                                model_sha256=sha(f'{D}/retry.lp'), new=f'{D}/retry-{kind}-1.json',
                                                old=old, capture_sha256=[sha(old)])
new = f'{N}/violated-primal-retryagent.json'
cases['violated-primal-retryagent'] = dict(role='noninterference', firing=False, kind='primal',
                                           model_sha256=sha(f'{V}/violated.lp'), new=new,
                                           old=f'{V}/violated-primal-1.json', capture_sha256=[sha(new)])
hist = sorted(p.relative_to(ROOT).as_posix() for p in (ROOT / D / 'history').iterdir())
records = [p for p in (f'{D}/model-mutation.json', f'{D}/rust-mutation.json', f'{D}/ratio-rust-mutation.json',
                       f'{D}/ratio_input.txt', 'bench/oracle/m3_push_decision_rust/input.txt') if (ROOT / p).exists()]
m = dict(task='M3-04c-7', schema_version=1,
         float_encoding='IEEE-754 binary64 little-endian hex; exact bits including signed zero',
         original_version='13.0.3', binary_sha256=step.BINARY_SHA,
         tools={p: sha(p) for p in (TOOL, AGENT, OLD_TOOL, OLD_AGENT, 'tools/push_retry_lp.py', 'tools/push_violated_lp.py',
                                    'tools/push_retry_ratio_oracle.py', 'tools/push_retry_rust.py',
                                    'tools/primal_push_decision_check.py', 'tools/primal_push_decision_rust.py',
                                    'tools/push_retry_runs.sh', 'tools/push_retry_manifest.py') if (ROOT / p).exists()},
         records={p: sha(p) for p in records},
         sources={p: sha(p) for p in ('../re/decomp/c/001c0070_PRIVATE0000000000134a87.c',
                                      '../re/decomp/c/000f0120_PRIVATE0000000000056973.c',
                                      '../re/decomp/c/001bb6f0_PRIVATE000000000012dd49.c')},
         inputs={p: sha(p) for p in (f'{D}/retry.lp', f'{D}/retry.extra.json')},
         ratio_captures={f'{D}/ratio-{i}.json': sha(f'{D}/ratio-{i}.json') for i in (1, 2, 3)},
         history={p: sha(p) for p in hist}, cases=cases)
(ROOT / D / 'manifest.json').write_text(json.dumps(m, ensure_ascii=False, indent=1) + '\n')
print('ok', len(cases), len(hist))
