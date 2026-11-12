"""M3-04c-17 독립 검토: 재캡처 비교와 별도 오류 주입. 저장소 루트에서 실행."""
import copy
import hashlib
import json
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[4]
sys.path.insert(0, str(ROOT/'tools'))
import push_recompute_check as check

D = ROOT/'bench/oracle/m3_push_recompute'
R = D/'review'
data = check.records()
m = check.load(check.D/'manifest.json')
check.check_files(m)
comparisons = {}
for name in ('interior', 'synthetic'):
    base = json.loads((D/f'{name}-1.json').read_text())
    fresh = json.loads((R/f'{name}-review-1.json').read_text())
    for field in ('events', 'binary_sha256', 'tool_sha256', 'environment', 'schema', 'origin'):
        check.equal(base[field], fresh[field], name+'.recapture.'+field)
    for field in ('result', 'output', 'params', 'starts', 'agent_sha256', 'model_sha256', 'input_sha256', 'substitution'):
        if field in base:check.equal(base[field], fresh[field], name+'.recapture.'+field)
    check.require(base['pid']!=fresh['pid'] and base['run_id']!=fresh['run_id'], name+'.independent process')
    comparisons[name] = dict(events=len(fresh['events']), independent_pid=fresh['pid'], sha256=hashlib.sha256((R/f'{name}-review-1.json').read_bytes()).hexdigest())

rejected=[]
def test(name, mutation, fn):
    d, mm = copy.deepcopy(data), copy.deepcopy(m)
    mutation(d, mm)
    try:fn(d, mm)
    except (check.Fail, KeyError, ValueError, TypeError, IndexError) as exc:
        rejected.append(dict(name=name, first_difference=str(exc)))
        return
    raise AssertionError('변조 생존: '+name)

def target(d, tag):
    return next(e for e in d['interior'][0]['events'] if e['event']==tag)

test('binary-zero', lambda d,m:d['interior'][0].update(binary_sha256='0'*64), check.controls)
test('input-model-zero', lambda d,m:d['interior'][0].update(model_sha256='0'*64), check.controls)
test('manifest-model-zero', lambda d,m:m['inputs'].update({'bench/oracle/m3_primal_push_step/interior.lp':'0'*64}), lambda d,m:check.check_files(m))
test('empty-events', lambda d,m:d['interior'][0].update(events=[]), lambda d,m:check.acceptance(d))
def zero_targets(d,m):
    for name in ('arms', 'retry', 'interior'):
        for r in d[name]:r['events']=[e for e in r['events'] if not e['event'].startswith('recompute:')]
test('natural-target-zero', zero_targets, lambda d,m:check.acceptance(d))
test('required-rhs-missing', lambda d,m:target(d,'recompute:enter')['input'].pop('rhs'), lambda d,m:check.acceptance(d))
def nulls(d,m):
    target(d,'recompute:enter')['input']['rhs']=None
    target(d,'recompute:ftran:enter')['input']['values']=None
test('null-null-rhs', nulls, lambda d,m:check.acceptance(d))
test('shift-basic-output', lambda d,m:target(d,'recompute:leave')['basic'].__setitem__(0,check.bits(0.)), lambda d,m:check.acceptance(d))
test('pinf-sign-zero', lambda d,m:target(d,'recompute:leave').update(pinf=check.bits(-0.)), lambda d,m:check.acceptance(d))
test('ftran-dense-count', lambda d,m:target(d,'recompute:ftran:enter')['input'].update(count=0), lambda d,m:check.acceptance(d))
test('raw-basic-alias', lambda d,m:target(d,'recompute:leave')['aliases'].update(raw_basic=True), lambda d,m:check.acceptance(d))
def swap_order(d,m):
    events=d['interior'][0]['events']; i=next(i for i,e in enumerate(events) if e['event']=='recompute:ftran:enter');events[i],events[i+1]=events[i+1],events[i]
test('ftran-event-order', swap_order, lambda d,m:check.acceptance(d))
def reorder_header(d,m):
    e=target(d,'recompute:enter')['input']['header'];e[0],e[1]=e[1],e[0]
test('new-header-order', reorder_header, lambda d,m:check.acceptance(d))
def boundary(d,m):
    event=next(e for e in d['synthetic'][0]['events'] if e['name']=='round-bounded-h-exact');event['result']['marks'][0]='4c'
test('h-exact-strictness', boundary, lambda d,m:check.acceptance(d))
def threshold(d,m):
    event=next(e for e in d['synthetic'][0]['events'] if e['name']=='pivot-retry-0x1.2a05f20000000p+33-2-0');event['result']['retry']=1
test('pinf-equal-threshold', threshold, lambda d,m:check.acceptance(d))

asm = subprocess.check_output(['python3','tools/dc.py','0x1c0070','--asm','--max','10000'], cwd=ROOT, text=True)
asm='\n'.join(line.rstrip() for line in asm.splitlines())+'\n'
check.equal(asm, (D/'source.asm').read_text(), 'fresh-asm')
result=dict(task='M3-04c-17', reviewer='/root/m3_04c17_review', reviewed_commit=subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),
            recaptures=comparisons, additional_mutations=dict(rejected=len(rejected), details=rejected),
            original_assembly=dict(lines=len(asm.splitlines()), sha256=hashlib.sha256(asm.encode()).hexdigest()),
            verdict='통과', blocking_findings=[], original_execution_count=2)
(R/'review.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n')
print(json.dumps(dict(recaptures=comparisons,rejected=len(rejected),assembly_lines=len(asm.splitlines()),verdict='통과'),ensure_ascii=False))
