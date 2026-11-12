"""M3-04c-19: 출처/독립3회/선행17 비간섭, round 제어 재생, 변조 거부."""
import argparse
from collections import Counter
import copy
import hashlib
import json
import math
from pathlib import Path
import sys

from push_supplier_check import Fail, equal, require, f64, vector
from push_recompute_check import natural as check_recompute
from push_round_runs import CASES
from push_round_synthetic import BINARY_SHA, SCHEMA, SUBSTITUTION, bits, specs, sha_specs

ROOT=Path(__file__).resolve().parents[1]
D=Path('bench/oracle/m3_push_round')
TOOLS=('tools/push_round_agent.js','tools/push_round_capture.js','tools/push_round_synthetic.py',
       'tools/push_round_runs.py','tools/push_round_check.py','tools/push_factor_synthetic.py',
       'tools/push_supplier_check.py','tools/push_recompute_check.py','tools/push_recompute_runs.py',
       'tools/push_recompute_synthetic.py','tools/frida_resolve.cjs','tools/gurun.sh')
SOURCES=('../re/decomp/c/001c0070_PRIVATE0000000000134a87.c',
         '../re/decomp/c/001c5310_PRIVATE000000000025615c.c',
         '../re/decomp/c/00131710_PRIVATE000000000009e376.c')
CTX=('m','total','candidates','position','deferred','failures','support','support_marks','count_sum','successes','batch')


def sha(p):return hashlib.sha256((ROOT/p).read_bytes()).hexdigest()
def load(p):return json.loads((ROOT/p).read_text())
def records():return {n:[load(D/f'{n}-{i}.json') for i in (1,2,3)] for n in (*CASES,'synthetic')}


def file_sets():
    return dict(tools=set(TOOLS),inputs={p for model,extra in CASES.values() for p in (model,extra) if p}|{str(D/'synthetic-inputs.json')},
        sources=set(SOURCES),assembly={str(D/'source.asm')},
        captures={str(D/f'{n}-{i}.json') for n in (*CASES,'synthetic') for i in (1,2,3)},
        baselines={f'bench/oracle/m3_push_recompute/{n}-1.json' for n in CASES}|{'bench/oracle/m3_push_recompute/manifest.json'},
        history={str(p.relative_to(ROOT)) for p in (ROOT/D/'history').rglob('*') if p.is_file()})


def manifest():
    return dict(task='M3-04c-19',binary_sha256=BINARY_SHA,specs_sha256=sha_specs(),
        **{k:{p:sha(p) for p in sorted(v)} for k,v in file_sets().items()})


def check_files(m):
    equal('M3-04c-19',m['task'],'manifest.task');equal(BINARY_SHA,m['binary_sha256'],'manifest.binary')
    equal(sha_specs(),m['specs_sha256'],'manifest.specs')
    for k,paths in file_sets().items():
        equal(paths,set(m[k]),'manifest.'+k+'.keys')
        for p in paths:
            if k=='sources' and not (ROOT/p).exists():continue
            equal(m[k][p],sha(p),'manifest.'+p)
    equal(specs(),load(D/'synthetic-inputs.json'),'synthetic-inputs')


def project(r):return dict(r,events=[e for e in r['events'] if not e['event'].startswith('roundctl:')])


def controls(data,m):
    totals={}
    for name in (*CASES,'synthetic'):
        rs=data[name];equal(3,len(rs),name+'.runs')
        equal(3,len({r['pid'] for r in rs}),name+'.pid');equal(3,len({r['run_id'] for r in rs}),name+'.run_id')
        for i,r in enumerate(rs):
            label=f'{name}/{i+1}'
            equal(BINARY_SHA,r['binary_sha256'],label+'.binary')
            tool='tools/push_round_synthetic.py' if name=='synthetic' else 'tools/push_round_capture.js'
            equal(m['tools'][tool],r['tool_sha256'],label+'.tool')
            equal(rs[0]['events'],r['events'],label+'.events');equal(rs[0]['environment'],r['environment'],label+'.environment')
            if name=='synthetic':
                equal(SCHEMA,r['schema'],label+'.schema');equal(SUBSTITUTION,r['substitution'],label+'.substitution')
                equal('synthetic-original-slice',r['origin'],label+'.origin');equal(sha_specs(),r['input_sha256'],label+'.inputs')
            else:
                model,extra=CASES[name];base=load(f'bench/oracle/m3_push_recompute/{name}-1.json');ex=load(extra) if extra else {}
                equal('M3-04c-19-v1',r['schema'],label+'.schema');equal('natural',r['origin'],label+'.origin')
                equal('control' if name=='control' else 'primal',r['kind'],label+'.kind')
                equal(m['tools']['tools/push_round_agent.js'],r['agent_sha256'],label+'.agent')
                equal(sha(model),r['model_sha256'],label+'.model')
                params=dict(Threads=1,Seed=0,OutputFlag=0,Presolve=0,Method=2,Crossover=0 if name=='control' else 3,LPWarmStart=2)
                params.update(ex.get('params',{}));equal(params,r['params'],label+'.params');require(ex.get('starts')==r['starts'],label+'.starts')
                equal([],r['errors'],label+'.errors');equal('process-terminated',r['reason'],label+'.reason')
                equal(base['events'],project(r)['events'],label+'.projection');equal(base['result'],r['result'],label+'.result');equal(base['output'],r['output'],label+'.output')
        totals[name]=len(rs[0]['events'])
    return totals


def compact(st):
    y=copy.deepcopy(st)
    require(y['batch']==[0,0,0],'batch=0 contract')
    y['candidates']=y['candidates'][:y['deferred']]+y['candidates'][y['position']:]
    y['total']=len(y['candidates']);y['position']=y['deferred']=0
    return y


def marks(st,values):
    st['marks']=values
    if 'support_marks' in st:st['support_marks']=values if st['support']>0 else None


def support(st):
    y=copy.deepcopy(st)
    if math.sqrt(y['m'])>y['approved']:y['support']=0
    elif y['support']!=0:marks(y,[0]*y['m'])
    if 'support_marks' in y:y['support_marks']=y['marks'] if y['support']>0 else None
    return y


def exhausted(st):
    y=copy.deepcopy(st)
    if y['deferred']==y['total']:
        if y['deferred']==0 or y['support']==0:return 'end',y
        y['support']=0
        if 'support_marks' in y:y['support_marks']=None
    return 'restart',y


def reference(case):
    x=case['input'];kind=case['kind'];y={k:copy.deepcopy(x[k]) for k in ('m','candidates','position','deferred','failures','marks','support','approved','request','status','numeric','failure','count_sum','successes')}
    y.update(total=len(y['candidates']),retry=0,return_code=0,batch=[0,0,0],calls=[])
    if kind=='defer':
        y['candidates'][y['deferred']]=x['enter'];y['deferred']+=1;y['failures'][x['enter']]+=1;out=101
    elif kind=='pass':
        if y['support']!=0:marks(y,[0]*y['m'])
        y=compact(y);out=101
    elif kind=='exhausted':
        action,y=exhausted(y)
        if action=='end':out=103
        else:y=compact(support(y));y['request']=10;out=102
    elif kind=='btran':
        out=104
        if x['support']!=0:
            y['calls']=[dict(event='btran',input=dict(count=1,indices=[x['leave']],values=[bits(1.)]))]
            raw=x['raw'];vector(raw,x['m'],case['name']+'.raw')
            if raw['count']>=0:
                for j in raw['indices']:y['marks'][j]=1
            else:
                y['support']=-1;marks(y,[0]*y['m']);y=compact(y)
                if y['total']==0:out=103
                else:y=compact(support(y));y['request']=10;out=102
    else:
        require(kind=='wrapper','synthetic.kind')
        for reply in x['replies']:
            y['calls'].append(dict(event='factor',request=y['request'],reply=reply))
            for k in ('request','status','numeric','failure'):
                if k in reply:y[k]=reply[k]
            if reply['ret']!=1:break
        ret=reply['ret'];out=105
        if ret==12:y['retry']=1;out=106
        elif ret>1:y['return_code']=ret;out=106
        elif y['status']<-999:y['status']=1;y['retry']=1;out=106
        elif x['attempt']==0 and (y['failure']!=0 or y['numeric']>=x['numeric_limit']):y['retry']=1;out=106
    return dict(exit=out,**y)


def ctx(st):return {k:st[k] for k in CTX}


def validate_state(st,push,label):
    require(type(st) is dict,label+'.state required')
    required=set(CTX)|{'marks','mf','request','status','numeric','failure','attempt','numeric_limit','retry'}
    require(set(st) in (required,required|{'approved'}),label+'.state keys')
    m=push['state']['m'];nm=m+push['state']['n'];equal(m,st['m'],label+'.m');equal(bits(float(m)),st['mf'],label+'.mf')
    require(0<=st['deferred']<=st['position']<=st['total'],label+'.cursor range')
    equal(st['total'],len(st['candidates']),label+'.total');equal(nm,len(st['failures']),label+'.failures.len')
    require(all(type(j) is int and 0<=j<nm for j in st['candidates']),label+'.candidate range')
    require(all(type(v) is int and v>=0 for v in st['failures']),label+'.failure range')
    equal([0,0,0],st['batch'],label+'.batch');equal(push['attempt'],st['attempt'],label+'.attempt')
    equal(push['state']['counters']['numeric']+10,st['numeric_limit'],label+'.numeric limit');f64(st['count_sum'])
    if st['marks'] is not None:
        equal(m,len(st['marks']),label+'.marks.len');require(all(v in (0,1) for v in st['marks']),label+'.marks values')
    if st['support']>0 and st['marks'] is not None:equal(st['marks'],st['support_marks'],label+'.support_marks')
    else:equal(None,st['support_marks'],label+'.unused support_marks')


def natural(r,name):
    check_recompute(project(r),name)
    totals=Counter();push=None;approved=0;pass_in=None;ready=None;defer=None;bt=None;raw=None
    supply_out=None;supply_in=None;action=None;expected=None;factor=None;factor_replies=[];recomp=None;step_kind=None;accepted_event=None
    flow_ctx=None;last_applied=None;bt_result=None
    for e in r['events']:
        tag=e['event'];label=name+'.'+tag+f'.step{e.get("step")}'
        if tag=='push:enter':push=e;approved=0
        elif tag=='supply:enter':
            if flow_ctx is not None:equal(flow_ctx,e['supplier']['context'],label+'.P continuity')
            if ready is not None:equal(ctx(ready),e['supplier']['context'],label+'.ready handoff');ready=None
            supply_in=e
        elif tag=='supply:leave':supply_out=e;flow_ctx=e['supplier']['context']
        elif tag=='step:enter':step_kind=e['kind']
        elif tag=='step:applied':
            require(accepted_event is not None,label+'.accepted required');equal(step_kind,accepted_event['kind'],label+'.kind')
            equal(e['selected'],accepted_event['selected'],label+'.selected');last_applied=accepted_event;accepted_event=None
        elif tag=='step:defer':
            require(defer is not None,label+'.defer required');equal(e['selected'],defer['selected'],label+'.selected')
            equal(e['deferred'],defer['state']['deferred'],label+'.deferred')
        elif tag=='factorctl:select' and bt_result is not None:
            for k in ('m','request','status','numeric','failure'):equal(bt_result[k],e['state'][k],label+'.BTRAN handoff.'+k)
            for k in ('support','attempt','numeric_limit'):equal(bt_result[k],e['frame'][k],label+'.BTRAN handoff.'+k)
            bt_result=None
        elif tag=='factor:enter' and factor is not None:
            equal(factor['request'],e['request'],label+'.request');factor_replies.append(None)
        elif tag=='factor:leave' and factor is not None:
            require(factor_replies and factor_replies[-1] is None,label+'.factor pairing');factor_replies[-1]=e
        elif tag=='recompute:leave' and e['site']=='round':recomp=e
        if not tag.startswith('roundctl:'):continue
        require(push is not None,label+'.push required');equal(push['call'],e['call'],label+'.call')
        t=tag[9:];totals[t]+=1
        st=e.get('state')
        if st is not None:validate_state(st,push,label)
        if t=='pass:enter':
            if expected is not None:
                y=copy.deepcopy(expected);y.pop('approved',None);equal(y,st,label+'.after recompute')
            approved=0;pass_in=st;expected=None
        elif t=='pass:ready':
            require(pass_in is not None,label+'.pass required');y=copy.deepcopy(pass_in)
            if y['support']!=0:marks(y,[0]*y['m'])
            y=compact(y);equal(y,st,label+'.pass transition');ready=st;pass_in=None
            if action=='clear':expected=st
        elif t=='accepted':
            require(supply_out is not None and supply_out['candidate']>=0,label+'.supplier required')
            equal(ctx(st),supply_out['supplier']['context'],label+'.supplier state')
            equal(supply_out['candidate'],e['selected']['enter'],label+'.enter')
            approved+=1;equal(approved,st['approved'],label+'.approved');equal(step_kind,e['kind'],label+'.kind')
            accepted_event=e
        elif t=='defer:enter':
            require(defer is None,label+'.nested defer');defer=e
            equal(approved,st['approved'],label+'.approved');equal(supply_out['candidate'],e['selected']['enter'],label+'.enter')
            equal(ctx(st),supply_out['supplier']['context'],label+'.supplier state')
        elif t=='defer:leave':
            require(defer is not None,label+'.defer required');y=copy.deepcopy(defer['state']);j=defer['selected']['enter']
            y['candidates'][y['deferred']]=j;y['deferred']+=1;y['failures'][j]+=1
            equal(y,st,label+'.defer transition');ready=st;defer=None
        elif t=='btran:prepare':
            require(bt is None,label+'.nested btran');require(st['support']!=0,label+'.support required')
            require(last_applied is not None,label+'.preceding applied step required')
            equal(last_applied['step'],e['step'],label+'.applied step')
            equal(last_applied['selected'],e['selected'],label+'.applied selected')
            equal(last_applied['state'],st,label+'.applied state')
            equal(approved,st['approved'],label+'.approved');bt=e;raw=None
        elif t=='btran:enter':
            require(bt is not None and raw is None,label+'.prepare required')
            equal(dict(count=1,indices=[bt['selected']['leave']],values=[bits(1.)]),e['input'],label+'.unit input')
            equal(bt['selected'],e['selected'],label+'.selected');equal(dict(factor=True,input=True,output=True,work=True),e['args'],label+'.args')
            raw='entered'
        elif t=='btran:leave':
            equal('entered',raw,label+'.enter required');raw=e['raw'];vector(raw,push['state']['m'],label+'.raw')
        elif t=='btran:applied':
            require(bt is not None and isinstance(raw,dict) and raw['count']>=0,label+'.sparse raw required')
            y=copy.deepcopy(bt['state']);require(y['marks'] is not None,label+'.marks required')
            for j in raw['indices']:y['marks'][j]=1
            y['support_marks']=y['marks'] if y['support']>0 else None
            equal(y,st,label+'.mark transition');bt_result=st;bt=None;raw=None
        elif t=='dense':
            require(bt is not None and isinstance(raw,dict) and raw['count']<0,label+'.dense required')
            equal(bt['state'],st,label+'.dense before')
        elif t=='dense:compacted':
            require(bt is not None and raw['count']<0,label+'.dense pairing');y=copy.deepcopy(bt['state'])
            y['support']=-1;marks(y,[0]*y['m']);y=compact(y);equal(y,st,label+'.dense transition')
            action='restart' if y['total'] else 'end';expected=y;bt=None;raw=None
        elif t=='exhausted':
            require(supply_out is not None and supply_out['candidate']==-1,label+'.exhausted supplier required')
            equal(supply_out['supplier']['context'],ctx(st),label+'.supplier state')
            equal(approved,e['approved_live'],label+'.approved live');equal(approved,st['approved'],label+'.approved')
            action,expected=exhausted(st)
        elif t=='restart':
            equal('restart',action,label+'.action');equal(expected,st,label+'.restart');expected=support(st);action='support'
        elif t=='support':
            equal('support',action,label+'.action');equal(expected,st,label+'.support transition')
            expected=compact(st);expected['request']=10;action='factor'
        elif t=='factor':
            equal('factor',action,label+'.action');equal(expected,st,label+'.compact request10');factor=st;factor_replies=[];action='return'
        elif t=='factor:return':
            equal('return',action,label+'.action');require(len(factor_replies)>0,label+'.factor calls required')
            equal([1]*(len(factor_replies)-1)+[e['ret']],[v['ret'] for v in factor_replies],label+'.repeats')
            last=factor_replies[-1];equal(last['request'],st['request'],label+'.request')
            for k in ('numeric','failure','status'):equal(last['factor_ctl']['state'][k],st[k],label+'.'+k)
            expected=copy.deepcopy(factor)
            for k in ('request','numeric','failure','status'):expected[k]=st[k]
            equal(expected,st,label+'.factor state');factor=None;action='recomputed'
        elif t=='recomputed':
            equal('recomputed',action,label+'.action');require(recomp is not None,label+'.recompute required')
            equal(0,st['retry'],label+'.retry');equal(expected,st,label+'.preserved P');equal(recomp['request'],st['request'],label+'.request')
            expected=copy.deepcopy(st);expected['request']=0;action='clear';recomp=None
        elif t=='clear':
            equal('clear',action,label+'.action');require(expected is not None,label+'.prepared state required');y=copy.deepcopy(expected);y.pop('approved',None)
            equal(y,st,label+'.request cleared');ready=st;action=None
        elif t=='end':
            require(action in ('end','clear'),label+'.end action')
            equal(expected,st,label+'.end state');equal(st['total'],e['remaining'],label+'.remaining');action=None;expected=None
        elif t=='error':raise Fail(label+'.unexpected natural failure')
        else:raise Fail(label+'.unknown round event')
        if st is not None:flow_ctx=ctx(st)
    require(bt_result is None and bt is None and defer is None and accepted_event is None and ready is None and action is None,label+'.unfinished round')
    require(totals['pass:ready']>0 and totals['exhausted']>0 and totals['end']>0,name+'.target zero')
    equal(totals['pass:enter'],totals['pass:ready'],name+'.pass pairing');equal(totals['factor'],totals['factor:return'],name+'.factor pairing')
    equal(totals['defer:enter'],totals['defer:leave'],name+'.defer pairing')
    equal(totals['btran:prepare'],totals['btran:enter'],name+'.btran pairing');equal(totals['btran:enter'],totals['btran:leave'],name+'.btran pairing')
    return dict(totals)


def acceptance(data):
    totals={n:natural(data[n][0],n) for n in ('arms','retry','interior')}
    equal([],data['control'][0]['events'],'control.events')
    rs=data['synthetic'][0]['events'];equal(len(specs()),len(rs),'synthetic.count')
    counts=Counter()
    for case,e in zip(specs(),rs):
        equal(case,{k:e[k] for k in ('name','kind','input')},'synthetic.'+case['name']+'.input')
        equal(reference(case),e['result'],'synthetic.'+case['name']+'.result');counts[case['kind']]+=1
    require(sum(t.get('defer:leave',0) for t in totals.values())>0,'natural defer target zero')
    require(sum(t.get('btran:applied',0) for t in totals.values())>0,'natural BTRAN target zero')
    return dict(natural=totals,synthetic=dict(counts),cases=len(rs))


def self_test(data,m):
    rejected=[]
    def test(name,mutate,fn):
        d,mm=copy.deepcopy(data),copy.deepcopy(m);mutate(d,mm)
        try:fn(d,mm)
        except (Fail,KeyError,TypeError,IndexError,ValueError) as exc:
            rejected.append(dict(name=name,error=str(exc)));return
        raise Fail('NOT REJECTED '+name)
    for k in ('binary_sha256','model_sha256','agent_sha256','tool_sha256'):
        test(k,lambda d,m,k=k:d['arms'][0].update({k:'0'*64}),controls)
    for k in ('tools','inputs','sources','assembly','captures','baselines','history'):
        test('manifest-empty-'+k,lambda d,m,k=k:m.update({k:{}}),lambda d,m:check_files(m))
    test('manifest-binary',lambda d,m:m.update(binary_sha256='0'*64),lambda d,m:check_files(m))
    test('events-empty',lambda d,m:d['arms'][0].update(events=[]),lambda d,m:acceptance(d))
    test('repeat-difference',lambda d,m:d['arms'][1]['events'].pop(),controls)
    test('projection',lambda d,m:d['arms'][0]['events'][0].update(first=99),controls)
    test('null-null',lambda d,m:d['synthetic'][0]['events'][0].update(result=None),lambda d,m:acceptance(d))
    def zero(d,m):
        for n in ('arms','retry','interior'):
            for r in d[n]:r['events']=[e for e in r['events'] if not e['event'].startswith('roundctl:')]
    test('all-target-zero',zero,lambda d,m:acceptance(d))
    def change(d,tag,key,value):
        e=next(e for e in d['retry'][0]['events'] if e['event']=='roundctl:'+tag)
        if key.startswith('state.'):e['state'][key[6:]]=value
        else:e[key]=value
    for tag,key,value in (
        ('defer:enter','state.failures',None),('defer:leave','state.failures',[0]*28),
        ('defer:leave','state.deferred',0),('defer:leave','state.candidates',[]),
        ('accepted','state.approved',99),('accepted','kind','flip'),
        ('exhausted','approved_live',99),('exhausted','state.support',0),
        ('restart','state.support',-1),('support','state.marks',[1]*10),
        ('factor','state.request',0),('factor','state.position',1),
        ('factor:return','ret',1),('recomputed','state.request',0),
        ('clear','state.request',10),('end','remaining',0),
        ('btran:enter','input',dict(count=1,indices=[0],values=[bits(1.)])),
        ('btran:enter','args',dict(factor=True,input=True,output=True,work=False)),
        ('btran:leave','raw',dict(count=0,indices=[],values=[])),
        ('btran:applied','state.marks',[0]*10),('pass:ready','state.total',99)):
        test('natural-'+tag+'.'+key,lambda d,m,t=tag,k=key,v=value:change(d,t,k,v),lambda d,m:acceptance(d))
    for tag in ('accepted','defer:enter','exhausted','btran:enter','btran:leave','btran:applied','factor:return','recomputed','pass:ready','end'):
        def remove(d,m,t=tag):
            r=d['retry'][0];r['events']=[e for e in r['events'] if e['event']!='roundctl:'+t]
        test('missing-'+tag,remove,lambda d,m:acceptance(d))
    def coordinated(d,m,kind):
        for r in d['retry']:
            prep=[e for e in r['events'] if e['event']=='roundctl:btran:prepare']
            enters=[e for e in r['events'] if e['event']=='roundctl:btran:enter']
            raws=[e for e in r['events'] if e['event']=='roundctl:btran:leave']
            applied=[e for e in r['events'] if e['event']=='roundctl:btran:applied']
            if kind=='leave':
                j=(prep[0]['selected']['leave']+1)%10
                prep[0]['selected']['leave']=j;enters[0]['selected']['leave']=j;enters[0]['input']['indices']=[j]
            elif kind=='request':
                prep[0]['state']['request']=6;applied[0]['state']['request']=6
            else:
                prep[1]['state']['marks']=[0]*10;prep[1]['state']['support_marks']=[0]*10
                values=[0]*10
                for j in raws[1]['raw']['indices']:values[j]=1
                applied[1]['state']['marks']=values;applied[1]['state']['support_marks']=values
    for kind in ('leave','request','marks'):
        test('coordinated-BTRAN-'+kind,lambda d,m,k=kind:coordinated(d,m,k),lambda d,m:acceptance(d))
    for i,case in enumerate(specs()):
        expected=reference(case)
        # One behavior change per case; choose an output specific to its branch.
        k='retry' if case['kind']=='wrapper' else 'deferred' if case['kind']=='defer' else 'support' if case['kind']=='btran' else 'candidates'
        val=(1-expected[k]) if type(expected[k]) is int else list(reversed(expected[k]))+[11]
        def mutate(d,m,i=i,k=k,v=val):d['synthetic'][0]['events'][i]['result'][k]=v
        test(case['name']+'.'+k,mutate,lambda d,m:acceptance(d))
    return dict(rejected=len(rejected),details=rejected)


def main():
    p=argparse.ArgumentParser(description=__doc__)
    for flag in ('controls','acceptance','self-test','write-manifest'):p.add_argument('--'+flag,action='store_true')
    p.add_argument('--report',type=Path);a=p.parse_args()
    if a.write_manifest:(ROOT/D/'manifest.json').write_text(json.dumps(manifest(),ensure_ascii=False,indent=2)+'\n')
    m=load(D/'manifest.json');check_files(m);data=records();out={}
    if a.controls:out['controls']=controls(data,m)
    if a.acceptance:out['acceptance']=acceptance(data)
    if a.self_test:out['self_test']=self_test(data,m)
    if a.report:a.report.write_text(json.dumps(out,ensure_ascii=False,indent=2)+'\n')
    print(json.dumps({k:dict(rejected=v['rejected']) if k=='self_test' else v for k,v in out.items()},ensure_ascii=False))


if __name__=='__main__':
    try:main()
    except (Fail,KeyError,TypeError,IndexError,ValueError) as exc:
        print('FAIL',exc,file=sys.stderr);sys.exit(1)
