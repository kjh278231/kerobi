"""M3-04c-2: bench/oracle/m3_push_violated/manifest.json을 캡처·도구·입력·기록 파일에서 다시 쓴다(캡처 뒤, 검사 전에 실행)."""
import hashlib
import json
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / 'tools'))
import primal_push_step_check as step  # noqa: E402

D = 'bench/oracle/m3_push_violated'
N = f'{D}/noninterference'
sha = lambda p: hashlib.sha256((ROOT / p).read_bytes()).hexdigest()  # noqa: E731
TOOL, AGENT = 'tools/primal_push_violated_capture.js', 'tools/primal_push_violated_agent.js'
OLD_TOOL, OLD_AGENT = 'tools/primal_push_arm_capture.js', 'tools/primal_push_arm_agent.js'
extra = json.loads((ROOT / D / 'violated.extra.json').read_text())
base = {'Threads': 1, 'Seed': 0, 'OutputFlag': 0, 'Presolve': 0, 'Method': 2, 'LPWarmStart': 2, **extra['params']}
cases = {}
for kind, xo, role in (('primal', 3, 'target'), ('dual', 1, 'target'), ('control', 0, 'control')):
    caps = [f'{D}/violated-{kind}-{i}.json' for i in (1, 2, 3)]
    rec = json.loads((ROOT / caps[0]).read_text())
    assert rec['params'] == dict(base, Crossover=xo), rec['params']
    cases[f'violated-{kind}'] = dict(role=role, kind=kind, model=f'{D}/violated.lp', model_sha256=sha(f'{D}/violated.lp'),
                                     params=rec['params'], starts=extra['starts'], tool=TOOL, agent=AGENT,
                                     schema='M3-04c-2-v1', captures=caps, events_sha256=step.digest(rec['events']),
                                     capture_sha256=[sha(c) for c in caps])
for kind in ('primal', 'dual'):
    cases[f'violated-{kind}-armagent'] = dict(role='noninterference', firing=False, kind=kind,
                                              model_sha256=sha(f'{D}/violated.lp'), new=f'{D}/violated-{kind}-1.json',
                                              old=f'{N}/violated-{kind}-armagent.json',
                                              capture_sha256=[sha(f'{N}/violated-{kind}-armagent.json')])
cases['retry-primal'] = dict(role='noninterference', firing=True, kind='primal', model_sha256=sha(f'{D}/retry.lp'),
                             new=f'{N}/retry-primal-new.json', old=f'{N}/retry-primal-armagent.json',
                             capture_sha256=[sha(f'{N}/retry-primal-new.json'), sha(f'{N}/retry-primal-armagent.json')])
hist = sorted(p.relative_to(ROOT).as_posix() for p in (ROOT / D / 'history').iterdir())
records = [p for p in (f'{D}/model-mutation.json', f'{D}/rust-mutation.json', f'{D}/ratio-rust-mutation.json',
                       f'{D}/ratio04c-rust-mutation-rerun.json',
                       f'{D}/ratio_input.txt', 'bench/oracle/m3_push_decision_rust/input.txt') if (ROOT / p).exists()]
m = dict(task='M3-04c-2', schema_version=1,
         float_encoding='IEEE-754 binary64 little-endian hex; exact bits including signed zero',
         original_version='13.0.3', binary_sha256=step.BINARY_SHA,
         tools={p: sha(p) for p in (TOOL, AGENT, OLD_TOOL, OLD_AGENT, 'tools/push_violated_lp.py',
                                    'tools/push_infeasible_ratio_oracle.py', 'tools/push_violated_rust.py',
                                    'tools/primal_push_decision_check.py', 'tools/primal_push_decision_rust.py',
                                    'tools/push_violated_runs.sh', 'tools/push_violated_manifest.py')},
         records={p: sha(p) for p in records},
         sources={p: sha(p) for p in ('../re/decomp/c/001c0070_PRIVATE0000000000134a87.c',
                                      '../re/decomp/c/000eefa0_PRIVATE0000000000054eef.c')},
         inputs={p: sha(p) for p in (f'{D}/violated.lp', f'{D}/violated.extra.json', f'{D}/retry.lp',
                                     f'{D}/retry.extra.json')},
         ratio_captures={f'{D}/ratio-{i}.json': sha(f'{D}/ratio-{i}.json') for i in (1, 2, 3)},
         history={p: sha(p) for p in hist}, cases=cases)
(ROOT / D / 'manifest.json').write_text(json.dumps(m, ensure_ascii=False, indent=1) + '\n')
print('ok', len(cases), len(hist))
