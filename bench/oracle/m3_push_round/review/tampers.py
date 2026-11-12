"""독립 검토: 원본 사건의 caller 연결과 출력 변조 검사."""
import copy
import json
from pathlib import Path
import sys
sys.path.insert(0,str(Path(__file__).resolve().parents[4]/'tools'))
import push_round_check as q

D=Path(__file__).resolve().parent
source=q.records();manifest=q.load(q.D/'manifest.json');results=[]
def test(name,mutate,check):
 data=copy.deepcopy(source);m=copy.deepcopy(manifest);mutate(data,m)
 try:check(data,m)
 except Exception as exc:
  results.append(dict(name=name,rejected=True,error=f'{type(exc).__name__}: {exc}'))
 else:results.append(dict(name=name,rejected=False))

def mutate_all(data,fn):
 for rec in data['retry']:fn(rec['events'])

def bt_wrong_leave(events):
 prep=next(e for e in events if e['event']=='roundctl:btran:prepare')
 enter=next(e for e in events if e['event']=='roundctl:btran:enter')
 prep['selected']['leave']=(prep['selected']['leave']+1)%10
 enter['selected']['leave']=prep['selected']['leave']
 enter['input']['indices']=[prep['selected']['leave']]

def bt_wrong_request(events):
 for tag in ('roundctl:btran:prepare','roundctl:btran:applied'):
  next(e for e in events if e['event']==tag)['state']['request']=6

def bt_reset_existing_marks(events):
 prep=[e for e in events if e['event']=='roundctl:btran:prepare'][1]
 leave=[e for e in events if e['event']=='roundctl:btran:leave'][1]
 applied=[e for e in events if e['event']=='roundctl:btran:applied'][1]
 prep['state']['marks']=[0]*10;prep['state']['support_marks']=[0]*10
 values=[0]*10
 for j in leave['raw']['indices']:values[j]=1
 applied['state']['marks']=values;applied['state']['support_marks']=values

def accept_and_controls(d,m):q.controls(d,m);q.acceptance(d)

test('binary-zero',lambda d,m:d['arms'][0].update(binary_sha256='0'*64),q.controls)
test('empty-events',lambda d,m:d['arms'][0].update(events=[]),lambda d,m:q.acceptance(d))
test('target-zero',lambda d,m:[r.update(events=[e for e in r['events'] if not e['event'].startswith('roundctl:')]) for n in ('arms','retry','interior') for r in d[n]],lambda d,m:q.acceptance(d))
for name,fn in [('caller-leave-and-unit-consistently-wrong',bt_wrong_leave),('caller-request-consistently-wrong',bt_wrong_request),('caller-existing-marks-lost',bt_reset_existing_marks)]:
 test(name,lambda d,m,fn=fn:mutate_all(d,fn),accept_and_controls)
for name,tag,key,val in [('factor-request','roundctl:factor','request',0),('approval-count','roundctl:accepted','approved',100),('failed-defer-increment','roundctl:defer:leave','failures',[0]*28)]:
 test(name,lambda d,m,t=tag,k=key,v=val:next(e for e in d['retry'][0]['events'] if e['event']==t)['state'].update({k:v}),lambda d,m:q.acceptance(d))
(D/'tampers-final.json').write_text(json.dumps(results,ensure_ascii=False,indent=2)+'\n')
print(json.dumps(results,ensure_ascii=False,indent=2))
