"""M3-04c-15: 출처/3회/비간섭 검사와 원본 factor caller 재생을 분리한다."""
import argparse
from collections import Counter
import copy
import hashlib
import json
from pathlib import Path
import sys

from push_supplier_check import Fail, equal, require, f64
from push_factor_runs import CASES
from push_factor_synthetic import BINARY_SHA, SCHEMA, SUBSTITUTION, bits, specs, sha_specs

ROOT=Path(__file__).resolve().parents[1]
D=Path('bench/oracle/m3_push_factor')
TOOLS=('tools/push_factor_agent.js','tools/push_factor_capture.js','tools/push_factor_synthetic.py',
       'tools/push_factor_runs.py','tools/push_factor_check.py','tools/push_supplier_check.py',
       'tools/push_supplier_synthetic.py','tools/push_supplier_runs.py','tools/frida_resolve.cjs','tools/gurun.sh')
SOURCES=('../re/decomp/c/001c0070_PRIVATE0000000000134a87.c',
         '../re/decomp/c/00115ed0_PRIVATE0000000000089f61.c')

def sha(p):return hashlib.sha256((ROOT/p).read_bytes()).hexdigest()
def load(p):return json.loads((ROOT/p).read_text())
def records():return {n:[load(D/f'{n}-{i}.json') for i in (1,2,3)] for n in (*CASES,'synthetic')}
def project(r):
    events=[]
    for e in r['events']:
        if e['event'].startswith('factorctl:'):continue
        e=copy.deepcopy(e);e.pop('factor_ctl',None);events.append(e)
    return events

def manifest():
    inputs={p for model,extra in CASES.values() for p in (model,extra) if p}
    baselines={f'bench/oracle/m3_push_supplier/{n}-1.json' for n in CASES}
    baselines.add('bench/oracle/m3_push_supplier/manifest.json')
    return dict(task='M3-04c-15',binary_sha256=BINARY_SHA,specs_sha256=sha_specs(),
        tools={p:sha(p) for p in TOOLS},inputs={p:sha(p) for p in sorted(inputs)},
        baselines={p:sha(p) for p in sorted(baselines)},sources={p:sha(p) for p in SOURCES},
        history={str(p.relative_to(ROOT)):sha(p) for p in sorted((ROOT/D/'history').glob('*')) if p.is_file()},
        assembly={str(D/'source.asm'):sha(D/'source.asm')},
        captures={str(D/f'{n}-{i}.json'):sha(D/f'{n}-{i}.json') for n in (*CASES,'synthetic') for i in (1,2,3)})

def check_files(m):
    equal('M3-04c-15',m['task'],'manifest.task');equal(BINARY_SHA,m['binary_sha256'],'manifest.binary')
    equal(sha_specs(),m['specs_sha256'],'manifest.specs')
    # Required membership prevents a manifest with empty hash maps from passing.
    equal(set(TOOLS),set(m['tools']),'manifest.tools.keys')
    equal({str(D/f'{n}-{i}.json') for n in (*CASES,'synthetic') for i in (1,2,3)},set(m['captures']),'manifest.captures.keys')
    required_inputs={p for model,extra in CASES.values() for p in (model,extra) if p}
    equal(required_inputs,set(m['inputs']),'manifest.inputs.keys')
    equal({f'bench/oracle/m3_push_supplier/{n}-1.json' for n in CASES}|{'bench/oracle/m3_push_supplier/manifest.json'},set(m['baselines']),'manifest.baselines.keys')
    equal(set(SOURCES),set(m['sources']),'manifest.sources.keys')
    for field in ('tools','inputs','baselines','captures','sources','history','assembly'):
        for p,h in m[field].items():
            if field=='sources' and not (ROOT/p).exists():continue
            equal(h,sha(p),f'manifest.{field}.{p}')

def controls(data,m):
    counts={}
    for name in (*CASES,'synthetic'):
        rs=data[name];require(len(rs)==3,f'{name}: 독립3회')
        require(len({r['run_id'] for r in rs})==3,f'{name}: run_id')
        require(len({r['pid'] for r in rs})==3,f'{name}: pid')
        for i,r in enumerate(rs):
            label=f'{name}/{i+1}'
            equal(BINARY_SHA,r['binary_sha256'],label+'.binary')
            equal(m['tools']['tools/push_factor_synthetic.py' if name=='synthetic' else 'tools/push_factor_capture.js'],r['tool_sha256'],label+'.tool')
            equal(rs[0]['events'],r['events'],label+'.events')
            equal(rs[0]['environment'],r['environment'],label+'.environment')
            if name=='synthetic':
                equal(SCHEMA,r['schema'],label+'.schema');equal(SUBSTITUTION,r['substitution'],label+'.substitution')
                equal('synthetic-original-slice',r['origin'],label+'.origin');equal(sha_specs(),r['input_sha256'],label+'.input')
            else:
                model,extra=CASES[name];baseline=load(f'bench/oracle/m3_push_supplier/{name}-1.json')
                equal('M3-04c-15-v1',r['schema'],label+'.schema');equal('natural',r['origin'],label+'.origin')
                equal(m['tools']['tools/push_factor_agent.js'],r['agent_sha256'],label+'.agent')
                equal(sha(model),r['model_sha256'],label+'.model')
                ex=load(extra) if extra else {}
                params=dict(Threads=1,Seed=0,OutputFlag=0,Presolve=0,Method=2,Crossover=0 if name=='control' else 3,LPWarmStart=2)
                params.update(ex.get('params',{}));equal(params,r['params'],label+'.params');require(ex.get('starts')==r['starts'],label+'.input-starts');equal(baseline['starts'],r['starts'],label+'.starts')
                equal([],r['errors'],label+'.errors');equal('process-terminated',r['reason'],label+'.reason')
                require(r['output'].count('CROSSOVER_RESULT ')==1,label+'.result marker')
                equal(baseline['output'],r['output'],label+'.output');equal(baseline['result'],r['result'],label+'.result')
                equal(baseline['events'],project(r),label+'.projection')
        counts[name]=len(rs[0]['events'])
    return counts

def select(x):
    req=x['request']
    if not req:
        if x['k']==x['limit']:req=2
        else:
            m=float(x['m']);a=float(int(x['L'])+int(x['U']))-m*1.5
            cost=0.
            if x['k']>=x['q']:
                cost=((float(x['C'])+float(x['E']))*2.)*m-min(float(x['q'])*((m*5.)*m),a*a)
            if cost>0. or (x['numeric']!=0 and x['k']>x['q']):req=10
    return dict(request=req,call=req!=0 or x['support']==0,preserved=True)

def wrapper(case):
    x=copy.deepcopy(case['input']);calls=[];fi=qi=0
    def finish(exit,retry=0):
        return dict(exit=exit,request=x['request'],status=x['status'],numeric=x['numeric'],failure=x['failure'],retry=retry,
                    period=x['period'],k=x['k'],alpha_restored=True,spike_restored=True,contents_preserved=True,calls=calls)
    for _ in range(20):
        while True:
            require(fi<len(case['replies']),case['name']+': factor input exhausted')
            reply=case['replies'][fi];fi+=1
            calls.append(dict(event='factor',request=x['request'],alpha_current=True,spike_original=True,reply=reply))
            for key in ('status','numeric','failure','request','k'):
                if key in reply:x[key]=reply[key]
            ret=reply['ret']
            if ret!=1:break
        if ret==12:out=finish(202,1);break
        if ret>1:out=finish(ret);break
        if x['status']<-999:x['status']=1;out=finish(202,1);break
        if x['attempt']==0 and (x['failure']!=0 or x['numeric']>=x['numeric_limit']):out=finish(202,1);break
        if x['request']!=0:out=finish(201);break
        if x['k']>=x['period']:
            require(qi<len(case['qualities']),case['name']+': quality input exhausted')
            reply=case['qualities'][qi];qi+=1
            calls.append(dict(event='quality',enter=x['enter'],period=x['period'],request=x['request'],reply=reply))
            x['request']=reply['request'];x['status']=reply.get('status',x['status'])
            if x['status']==12:x['status']=1;out=finish(202,1);break
            if x['request']!=0:x['period']=20;continue
            x['period']+=reply['increment']
        elif x['k']!=0 and x['period']>50:x['period']=50
        x['request']=0;out=finish(200);break
    else:raise Fail(case['name']+': backend loop')
    equal(len(case['replies']),fi,case['name']+'.factor-consumed');equal(len(case['qualities']),qi,case['name']+'.quality-consumed')
    return out

def acceptance(data):
    totals=Counter()
    for name in CASES:
        events=data[name][0]['events'];pending=None;wrap=None;factor=None;last_gate=None;last_factor=None;push=None;await_clear=False
        if name=='control':equal([],events,'control.events');continue
        counts=Counter(e['event'] for e in events)
        require(counts['factorctl:select']>0,name+': 목표 판단0회')
        require(counts['factor:enter']>0,name+': factor0회')
        for idx,e in enumerate(events):
            label=f'{name}/event[{idx}]/{e["event"]}'
            event=e['event']
            if event=='push:enter':push=e
            if event.startswith('factorctl:') and 'frame' in e:
                require(push is not None,label+': push context missing')
                equal(push['attempt'],e['frame']['attempt'],label+'.attempt')
                equal(push['state']['counters']['numeric']+10,e['frame']['numeric_limit'],label+'.numeric_limit')
                equal([0,0,0],e['frame']['batch'],label+'.batch')
            if await_clear and event=='supply:enter':
                equal(0,e['state']['counters']['request'],label+'.request-cleared');await_clear=False
            elif await_clear and event=='factor:enter':await_clear=False # round writes request10
            if event=='factorctl:select':
                require(pending is None,label+': 겹친 판단')
                st=e['state'];fr=e['frame'];equal([0,0,0],fr['batch'],label+'.batch')
                equal([bits(st['m']*1.5),bits((st['m']*5.)*st['m']),bits(float(st['m']))],fr['constants'],label+'.constants')
                require(fr['period']>0,label+'.period');f64(st['quality_tol']);f64(st['markowitz'])
                pending=(select(dict(st,support=fr['support'])),e['call'],e['step'],e)
                totals['select']+=1
            elif event in ('factorctl:call','factorctl:clear') and e['decision']:
                require(pending is not None,label+': 판단 누락')
                want,call,step,selection=pending
                equal(dict(selection['state'],request=want['request']),e['state'],label+'.selected-state')
                equal(selection['frame'],e['frame'],label+'.selected-frame')
                equal((call,step),(e['call'],e['step']),label+'.context')
                equal(want['request'],e['state']['request'],label+'.request')
                equal(want['call'],event=='factorctl:call',label+'.call')
                totals['selected_call' if want['call'] else 'skip']+=1;pending=None
            if event=='factorctl:call':
                require(wrap is None,label+': wrapper overlap');wrap=e;totals['wrapper']+=1
            elif event=='factor:enter':
                require(factor is None,label+': factor overlap');factor=e
                fc=e['factor_ctl'];equal(True,fc['workspace'],label+'.workspace')
                equal(e['request'],fc['state']['request'],label+'.request')
                if wrap:
                    equal(wrap['state'],fc['state'],label+'.call-state')
                    require(fc['caller'] in ('0x1c3538','0x1c36ea'),label+'.caller '+fc['caller'])
                    equal(True,fc['alpha_is_current'],label+'.alpha')
            elif event=='factor:leave':
                require(factor is not None,label+': factor enter missing')
                equal(True,e['factor_ctl']['spike_preserved'],label+'.spike')
                equal(e['request'],e['factor_ctl']['state']['request'],label+'.request')
                last_factor=e;factor=None;totals['factor']+=1
            elif event=='factorctl:restored':
                require(wrap is not None,label+': wrapper missing')
                equal(True,e['alpha_restored'],label+'.alpha');equal(True,e['spike_restored'],label+'.spike')
                equal(0,e['ret'],label+'.natural-ret')
                require(last_factor is not None,label+': factor leave missing')
                equal(last_factor['factor_ctl']['state'],e['state'],label+'.restored-state')
                equal(last_factor['ret'],e['ret'],label+'.ret')
                equal(wrap['frame'],e['frame'],label+'.restored-frame')
                require(e['state']['status']>=-999,label+': natural status retry unexpectedly')
                require(e['frame']['attempt']!=0 or (e['state']['failure']==0 and e['state']['numeric']<e['frame']['numeric_limit']),label+': natural retry gate unexpectedly')
                wrap=None;totals['restored']+=1
            elif event=='factorctl:quality_gate':
                require(last_factor is not None,label+': preceding factor missing')
                equal(last_factor['factor_ctl']['state'],e['state'],label+'.quality-gate-state');last_gate=e
            elif event=='factorctl:clear' and not e['decision']:
                require(last_gate is not None,label+': gate missing')
                k=last_gate['state']['k'];period=last_gate['frame']['period']
                require(k<period,label+': natural quality unexpected (extend caller check)')
                period=min(period,50) if k else period
                equal(last_gate['state'],e['state'],label+'.clear-state')
                equal(dict(last_gate['frame'],period=period),e['frame'],label+'.clear-frame')
                last_gate=None
            if event=='factorctl:clear':await_clear=True
        require(pending is None and wrap is None and factor is None,name+': 끝나지 않은 구간')
    require(totals['skip']>0 and totals['selected_call']>0,'자연 skip/call 양쪽 필요')
    events=data['synthetic'][0]['events'];equal(len(specs()),len(events),'synthetic.count')
    for case,e in zip(specs(),events):
        equal(case,{k:v for k,v in e.items() if k!='result'},case['name']+'.input')
        expected=select(case['input']) if case['kind']=='select' else wrapper(case)
        equal(expected,e['result'],'synthetic/'+case['name']+'.result')
        totals['synthetic_'+case['kind']]+=1
    return dict(totals)

def self_test(data,m):
    done=[]
    def test(name,mutate,fn):
        d=copy.deepcopy(data);mm=copy.deepcopy(m);mutate(d,mm)
        try:fn(d,mm)
        except (Fail,KeyError,TypeError,IndexError,ValueError):done.append(name);return
        raise Fail('NOT REJECTED '+name)
    test('binary-zero',lambda d,m:d['arms'][0].update(binary_sha256='0'*64),controls)
    test('model-zero',lambda d,m:d['arms'][0].update(model_sha256='0'*64),controls)
    test('tool-zero',lambda d,m:d['arms'][0].update(tool_sha256='0'*64),controls)
    test('agent-zero',lambda d,m:d['arms'][0].update(agent_sha256='0'*64),controls)
    test('manifest-binary-zero',lambda d,m:m.update(binary_sha256='0'*64),lambda d,m:check_files(m))
    test('manifest-empty-inputs',lambda d,m:m.update(inputs={}),lambda d,m:check_files(m))
    test('empty-events',lambda d,m:d['arms'][0].update(events=[]),lambda d,m:acceptance(d))
    test('target-zero',lambda d,m:d['arms'][0].update(events=[e for e in d['arms'][0]['events'] if not e['event'].startswith('factorctl:')]),lambda d,m:acceptance(d))
    test('independent-difference',lambda d,m:d['arms'][1]['events'].pop(),controls)
    test('null-null',lambda d,m:d['synthetic'][0]['events'][0].update(result=None),lambda d,m:acceptance(d))
    for name,key,value in [('preserve-6','request',2),('limit-11','request',2),('limit-10','request',0),
                           ('cost-0','request',10),('cost-1','request',0),('skip-1','call',True),('sum-convert','preserved',False),
                           ('repeat-two','alpha_restored',False),('return12','retry',0),('error7','exit',200),
                           ('error-precedes-status','status',1),('status--999','retry',1),('status--1000','status',-1000),
                           ('failure-attempt-1','retry',1),('numeric-13-attempt-0','retry',0),('request-recompute','exit',200),
                           ('nonzero-clamp100','period',100),('zero-updates-period100','period',50),
                           ('quality-increment-5','period',100),('quality-request6','period',50),('quality-status12','status',12),
                           ('normal','spike_restored',False),('normal','contents_preserved',False),('min-q-term-19','request',10),('min-q-term-20','request',10)]:
        def mutate(d,m,n=name,k=key,v=value):
            next(e for e in d['synthetic'][0]['events'] if e['name']==n)['result'][k]=v
        test(name+'.'+key,mutate,lambda d,m:acceptance(d))
    def call_change(d,m):
        next(e for e in d['synthetic'][0]['events'] if e['name']=='repeat-two')['result']['calls'].pop(0)
    test('repeat-call-order',call_change,lambda d,m:acceptance(d))
    def alias(d,m):
        next(e for e in d['interior'][0]['events'] if e['event']=='factorctl:restored')['alpha_restored']=False
    test('natural-alpha-restore',alias,lambda d,m:acceptance(d))
    for tag,key,value in [('factorctl:restored','status',-1000),('factorctl:call','attempt',99),('factorctl:call','numeric_limit',999)]:
        def mutate(d,m,t=tag,k=key,v=value):
            e=next(e for e in d['interior'][0]['events'] if e['event']==t)
            e['state' if k=='status' else 'frame'][k]=v
        test('natural-'+tag+'.'+key,mutate,lambda d,m:acceptance(d))
    def enter(d,m):
        next(e for e in d['synthetic'][0]['events'] if e['name']=='quality-increment-5')['result']['calls'][1]['enter']=8
    test('quality-enter',enter,lambda d,m:acceptance(d))
    return dict(rejected=len(done),names=done)

def main():
    p=argparse.ArgumentParser(description=__doc__)
    for flag in ('controls','acceptance','self-test','write-manifest'):p.add_argument('--'+flag,action='store_true')
    a=p.parse_args()
    if a.write_manifest:(ROOT/D/'manifest.json').write_text(json.dumps(manifest(),ensure_ascii=False,indent=2)+'\n')
    m=load(D/'manifest.json');check_files(m);data=records()
    if a.controls:print(json.dumps(dict(controls=controls(data,m)),ensure_ascii=False))
    if a.acceptance:print(json.dumps(dict(acceptance=acceptance(data)),ensure_ascii=False))
    if a.self_test:print(json.dumps(dict(self_test=self_test(data,m)),ensure_ascii=False))

if __name__=='__main__':
    try:main()
    except Fail as exc:print('FAIL',exc,file=sys.stderr);sys.exit(1)
