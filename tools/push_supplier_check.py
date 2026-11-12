"""M3-04c-13: 공급자 출처·비간섭·제어 재생·오류 주입 검사.

--controls는 출처/독립3회/이전 agent 투영을 검사한다.
--acceptance는 자연 목표0회 및 필수 누락을 거부하고 공급자 제어를 재생한다.
"""
import argparse
from collections import Counter
import copy
import hashlib
import json
import math
from pathlib import Path
import struct
import sys

from push_supplier_runs import CASES
from push_supplier_synthetic import BINARY_SHA, SCHEMA, SUBSTITUTION, bits, sha_specs, specs

ROOT=Path(__file__).resolve().parents[1]
D='bench/oracle/m3_push_supplier'
TOOLS=('tools/push_supplier_capture.js','tools/push_supplier_agent.js','tools/push_supplier_synthetic.py',
       'tools/push_supplier_runs.py','tools/push_supplier_check.py','tools/frida_resolve.cjs','tools/gurun.sh')
SOURCES=('../re/decomp/c/001c5310_PRIVATE000000000025615c.c',
         '../re/decomp/c/00120680_PRIVATE000000000009d008.c')
BASELINES={
 'arms':('bench/oracle/m3_push_arms/arms-primal-1.json','bench/oracle/m3_push_arms/manifest.json'),
 'retry':('bench/oracle/m3_push_retry/retry-primal-1.json','bench/oracle/m3_push_retry/manifest.json'),
 'interior':('bench/oracle/m3_primal_push_step/primal-1.json','bench/oracle/m3_primal_push_step/manifest.json'),
 'control':('bench/oracle/m3_push_arms/arms-control-1.json','bench/oracle/m3_push_arms/manifest.json')}


class Fail(Exception):
    pass


def require(ok, label):
    if not ok:raise Fail(label)


def equal(want, got, label):
    """첫 차이의 경로와 기준/실제값을 출력한다. float는 hex string으로 비교한다."""
    if type(want) is not type(got):raise Fail(f'{label}: 타입 기준={type(want).__name__}, 실제={type(got).__name__}')
    if isinstance(want,dict):
        require(want.keys()==got.keys(), f'{label}: 필드 기준={list(want)}, 실제={list(got)}')
        for k in want:equal(want[k],got[k],f'{label}.{k}')
    elif isinstance(want,list):
        require(len(want)==len(got),f'{label}: 길이 기준={len(want)}, 실제={len(got)}')
        for i,(a,b) in enumerate(zip(want,got)):equal(a,b,f'{label}[{i}]')
    elif want!=got:raise Fail(f'{label}: 기준={want!r}, 실제={got!r}')


def f64(h):
    require(isinstance(h,str) and len(h)==16, f'f64 LE hex 누락/길이: {h!r}')
    try:v=struct.unpack('<d',bytes.fromhex(h))[0]
    except (ValueError,struct.error) as e:raise Fail(f'f64 인코딩: {h!r}') from e
    require(math.isfinite(v),f'유한값 전제: {h}')
    return v


def sha(p):
    return hashlib.sha256((ROOT/p).read_bytes()).hexdigest()


def load(p):
    return json.loads((ROOT/p).read_text())


def records():
    return {name:[load(f'{D}/{name}-{i}.json') for i in (1,2,3)] for name in (*CASES,'synthetic')}


def projection(rec,name):
    out=[]
    for source in rec['events']:
        if source['event'].startswith('supplier:'):continue
        e=copy.deepcopy(source);e.pop('supplier',None)
        if name=='interior':e.pop('frame',None)
        elif 'frame' in e:e['frame'].pop('ninf',None)
        out.append(e)
    return out


def check_files(manifest):
    equal('M3-04c-13',manifest['task'],'manifest.task')
    equal(BINARY_SHA,manifest['binary_sha256'],'manifest.binary_sha256')
    for field in ('tools','inputs','baselines','captures','history'):
        for p,h in manifest[field].items():equal(h,sha(p),f'manifest.{field}.{p}')
    # 원본 C 없이도 저장 재생은 가능하다. 존재하는 C는 해시까지 대조한다.
    for p,h in manifest['sources'].items():
        if (ROOT/p).exists():equal(h,sha(p),f'manifest.sources.{p}')


def controls(data,manifest):
    counts={}
    for name,(model,extra) in CASES.items():
        rs=data[name];base_path,_=BASELINES[name];old=load(base_path)
        require(len(rs)==3,f'{name}: 독립3회 필요')
        require(len({r['run_id'] for r in rs})==3,f'{name}: 독립 run_id')
        require(len({r['pid'] for r in rs})==3,f'{name}: 독립 pid')
        expected_extra=load(extra) if extra else {}
        params=dict(Threads=1,Seed=0,OutputFlag=0,Presolve=0,Method=2,
                    Crossover=0 if name=='control' else 3,LPWarmStart=2,**expected_extra.get('params',{}))
        for i,r in enumerate(rs,1):
            label=f'{name}-{i}'
            equal(BINARY_SHA,r['binary_sha256'],label+'.binary_sha256')
            equal(sha(model),r['model_sha256'],label+'.model_sha256')
            equal(sha(TOOLS[0]),r['tool_sha256'],label+'.tool_sha256')
            equal(sha(TOOLS[1]),r['agent_sha256'],label+'.agent_sha256')
            equal('M3-04c-13-v1',r['schema'],label+'.schema')
            equal('natural',r['origin'],label+'.origin')
            equal('control' if name=='control' else 'primal',r['kind'],label+'.kind')
            equal(params,r['params'],label+'.params')
            expected_starts=expected_extra.get('starts')
            if expected_starts is not None:
                expected_starts={k:[int(v) if isinstance(v,float) and v.is_integer() else v for v in vs] for k,vs in expected_starts.items()}
            equal(expected_starts,r['starts'],label+'.starts')
            equal([],r['errors'],label+'.errors')
            equal('process-terminated',r['reason'],label+'.reason')
            equal(2,r['result']['status'],label+'.result.status')
            equal('x64',r['environment']['arch'],label+'.arch')
            for field in ('params','starts','events','output','result','environment'):
                equal(rs[0][field],r[field],label+'.repeat.'+field)
            equal(old['events'],projection(r,name),label+'.noninterference.events')
            for field in ('output','result','params','starts','binary_sha256','model_sha256'):
                # 이전 interior 캡처에는 starts 키가 없다.
                equal(old.get(field),r.get(field),label+'.noninterference.'+field)
        counts[name]=len(rs[0]['events'])
    rs=data['synthetic']
    require(len(rs)==3 and len({r['run_id'] for r in rs})==3,'synthetic: 독립3회')
    expected=specs()
    for i,r in enumerate(rs,1):
        label=f'synthetic-{i}'
        for field,value in [('binary_sha256',BINARY_SHA),('schema',SCHEMA),('origin','synthetic-backend'),
                            ('substitution',SUBSTITUTION),('tool_sha256',sha(TOOLS[2])),('input_sha256',sha_specs())]:
            equal(value,r[field],label+'.'+field)
        equal(rs[0]['events'],r['events'],label+'.repeat.events')
        equal(len(expected),len(r['events']),label+'.cases')
        for case,e in zip(expected,r['events']):
            equal(case['name'],e['name'],label+'.case')
            m=case.get('m',9)
            before=dict(case['before'],m=m,total=len(case['before']['candidates']),failures=case['before']['failures'][:m+3])
            equal(before,e['before'],label+'.'+e['name']+'.input')
            equal(case['drop'],e['drop'],label+'.'+e['name']+'.drop')
            equal([] if case['raw'] is None else [case['raw']], [f['raw'] for f in e['ftran']],label+'.'+e['name']+'.backend')
    return counts


def units(rec):
    """자연 사건을 공급자 단위로 연결한다. 중첩·짝 누락·순서 오류는 거부한다."""
    model=current=pending=None;out=[];seq=0
    for i,e in enumerate(rec['events']):
        event=e['event'];label=f'event[{i}].{event}'
        if event=='push:enter':
            require(current is None,label+': 공급 중 push 진입');model=e['model']
        elif event=='supply:enter':
            require(current is None and model is not None,label+': 잘못된 진입')
            seq+=1
            equal(seq,e['supplier']['id'],label+'.id')
            current=dict(name=label,m=e['state']['m'],n=e['state']['n'],model=model,drop=e['supplier']['drop'],
                         before=e['supplier']['context'],ftran=[],call=e['call'],step=e['step'])
        elif event=='supplier:ftran:enter':
            require(current is not None and pending is None,label+': 잘못된 FTRAN 진입')
            equal(seq,e['supplier_id'],label+'.id')
            equal(current['call'],e['call'],label+'.call');equal(current['step'],e['step'],label+'.step')
            pending=e
        elif event=='supplier:ftran:leave':
            require(current is not None and pending is not None,label+': FTRAN 진입 없음')
            equal(seq,e['supplier_id'],label+'.id')
            equal(current['call'],e['call'],label+'.call');equal(current['step'],e['step'],label+'.step')
            current['ftran'].append(dict(input=pending['input'],aliases=pending['aliases'],raw=e['raw'],
                                        spike_before=pending['spike'],spike_after=e['spike']))
            pending=None
        elif event=='supply:leave':
            require(current is not None and pending is None,label+': 공급 진입/FTRAN 종료 없음')
            equal(seq,e['supplier']['id'],label+'.id')
            equal(current['call'],e['call'],label+'.call')
            equal(current['step']+int(e['candidate']>=0),e['step'],label+'.step')
            current.update(after=e['supplier']['context'],ret=e['candidate'],alpha=e['alpha'],output_is_s210=e['supplier']['output_is_s210'])
            out.append(current);current=None
        elif current is not None:
            raise Fail(label+': 공급자 경계에 예상 밖 사건')
    require(current is None and pending is None,'공급자/FTRAN 끝 누락')
    return out


def column(model,j,n,m):
    if j>=n:return dict(count=1,indices=[j-n],values=[bits(1.)])
    start=model['starts'][j]
    if isinstance(start,str):start=struct.unpack('<q',bytes.fromhex(start))[0]
    count=model['lengths'][j]
    require(isinstance(start,int) and isinstance(count,int) and start>=0 and count>=0,'CSC offset/length')
    rows=model['rows'][start:start+count];values=model['values'][start:start+count]
    equal(count,len(rows),'CSC rows length');equal(count,len(values),'CSC values length')
    require(all(type(r) is int and 0<=r<m for r in rows),'CSC row 범위')
    for h in values:f64(h)
    return dict(count=count,indices=rows,values=values)


def vector(v,m,label):
    require(isinstance(v,dict),label+': 필수 벡터 누락')
    require(type(v['count']) is int,label+'.count 정수')
    count=v['count'];n=m if count<0 else count
    require(0<=n<=m,label+': count 범위')
    equal(n,len(v['values']),label+'.values 길이')
    if count<0:equal(None,v['indices'],label+'.dense indices')
    else:
        equal(n,len(v['indices']),label+'.indices 길이')
        require(all(type(x) is int and 0<=x<m for x in v['indices']),label+': index 범위')
    for h in v['values']:f64(h)


def replay(u,label):
    b,a,m,n=u['before'],u['after'],u['m'],u['n'];t=Counter(calls=1)
    require(isinstance(b,dict) and isinstance(a,dict),label+': 필수 문맥 누락')
    equal(m,b['m'],label+'.m');equal(len(b['candidates']),b['total'],label+'.total')
    equal(n+m,len(b['failures']),label+'.failures length')
    equal([0,0,0],b['batch'],label+'.batch')
    require(0<=b['deferred']<=b['position']<=b['total'],label+': cursor/deferred 전제')
    require(all(type(j) is int and 0<=j<n+m for j in b['candidates']),label+': 후보 범위')
    expected=copy.deepcopy(b);expected['support']=max(0,b['support'])
    if expected['support']>0:
        equal(m,len(b['support_marks']),label+'.support_marks 길이')
        t['support-positive']+=1
    else:
        equal(None,b['support_marks'],label+'.unused support_marks')
        t['support-zero']+=1
    t['support-negative']+=int(b['support']<0)
    ret=-1;col=None
    while expected['position']<expected['total']:
        j=expected['candidates'][expected['position']];expected['position']+=1
        fail=b['failures'][j]>=3
        col=column(u['model'],j,n,m)
        conflict=not fail and expected['support']>0 and any(b['support_marks'][r]!=0 for r in col['indices'])
        t['failure-two']+=int(b['failures'][j]==2)
        if fail or conflict:
            t['skip-failure' if fail else 'skip-conflict']+=1
            t['skip-structural' if j<n else 'skip-slack']+=1
            expected['candidates'][expected['deferred']]=j;expected['deferred']+=1
        else:
            ret=j;break
    equal(ret,u['ret'],label+'.ret')
    equal(True,u['output_is_s210'],label+'.output_is_s210')
    equal(int(ret>=0),len(u['ftran']),label+'.FTRAN calls')
    if ret<0:
        t['exhausted']+=1;equal(None,u['alpha'],label+'.exhausted alpha')
    else:
        t['success']+=1;t['structural' if ret<n else 'slack']+=1
        f=u['ftran'][0]
        equal(col,f['input'],label+'.FTRAN original column')
        aliases=dict(factor=True,input_s220=True,output_s210=True,spike_null=expected['support']>0,
                     spike_f1c8=expected['support']==0,work=True,zeros=[True,True,True])
        equal(aliases,f['aliases'],label+'.FTRAN aliases')
        if 'spike_before' in f:
            if expected['support']>0:
                equal(None,f['spike_before'],label+'.spike_before');equal(None,f['spike_after'],label+'.spike_after')
            else:
                vector(f['spike_before'],m,label+'.spike_before');vector(f['spike_after'],m,label+'.spike_after')
        raw=f['raw'];vector(raw,m,label+'.raw');vector(u['alpha'],m,label+'.alpha')
        expected['successes']+=1
        expected['count_sum']=bits(float(raw['count'])+f64(b['count_sum']))
        drop=f64(u['drop']);require(drop>=0,label+': drop>=0')
        alpha=copy.deepcopy(raw)
        if raw['count']<0:
            t['dense']+=1
            indices=[r for r,v in enumerate(raw['values']) if drop<abs(f64(v))]
            alpha=dict(count=len(indices),indices=indices,values=[raw['values'][r] for r in indices])
        else:t['sparse']+=1
        equal(alpha,u['alpha'],label+'.alpha compression/order')
    equal(expected,a,label+'.after')
    return t


def acceptance(data):
    natural=Counter();synthetic=Counter();by_case={}
    for name in CASES:
        for run,r in enumerate(data[name],1):
            require(r['origin']=='natural',f'{name}: 자연 실행 필수')
            us=units(r)
            if name=='control':equal([],us,f'control-{run}: 공급자0회')
            else:require(len(us)>0,f'{name}: 목표 사건0회')
            count=Counter()
            for i,u in enumerate(us):count.update(replay(u,f'{name}-{run}.supply[{i}]'))
            if run==1:by_case[name]=dict(count);natural.update(count)
    for tag in ('success','exhausted','structural','slack','support-positive','support-zero','skip-conflict','skip-failure','failure-two'):
        require(natural[tag]>0,f'자연 목표0회: {tag}')
    for run,r in enumerate(data['synthetic'],1):
        equal('synthetic-backend',r['origin'],f'synthetic-{run}.origin')
        require(len(r['events'])==18,'합성18사례 필요')
        count=Counter()
        for u in r['events']:count.update(replay(u,f'synthetic-{run}.{u["name"]}'))
        if run==1:synthetic.update(count)
    for tag in ('support-negative','dense','sparse','exhausted','skip-failure','skip-conflict','failure-two'):
        require(synthetic[tag]>0,f'합성 목표0회: {tag}')
    return dict(natural=dict(natural),synthetic=dict(synthetic),cases=by_case)


def manifest():
    caps={f'{D}/{name}-{i}.json':sha(f'{D}/{name}-{i}.json') for name in (*CASES,'synthetic') for i in (1,2,3)}
    inputs={p:sha(p) for paths in CASES.values() for p in paths if p}
    baselines={p:sha(p) for paths in BASELINES.values() for p in paths}
    return dict(task='M3-04c-13',schema_version=1,original_version='13.0.3',binary_sha256=BINARY_SHA,
        float_encoding='IEEE-754 binary64 LE hex, exact bits incl signed zero',
        tools={p:sha(p) for p in TOOLS},sources={p:sha(p) for p in SOURCES},inputs=inputs,baselines=baselines,captures=caps,
        history={str(p.relative_to(ROOT)):sha(str(p.relative_to(ROOT))) for p in sorted((ROOT/D/'history').iterdir())})


def self_test(data,m):
    out=[]
    def bad(label,edit,which='acceptance'):
        d=copy.deepcopy(data);edit(d)
        try:
            (lambda: controls(d,m))() if which=='controls' else acceptance(d)
        except Fail as e:
            out.append(dict(name=label,first_difference=str(e)));return
        raise Fail('변조 미검출: '+label)
    def every(name,fn):
        return lambda d:[fn(r) for r in d[name]]
    def supply(r,event='supply:leave'):
        return next(e for e in r['events'] if e['event']==event)
    def synth(r,name):return next(e for e in r['events'] if e['name']==name)
    bad('binary-zero',every('arms',lambda r:r.__setitem__('binary_sha256','0'*64)),'controls')
    bad('model-zero',every('arms',lambda r:r.__setitem__('model_sha256','0'*64)),'controls')
    bad('tool-zero',every('arms',lambda r:r.__setitem__('tool_sha256','0'*64)),'controls')
    bad('agent-zero',every('arms',lambda r:r.__setitem__('agent_sha256','0'*64)),'controls')
    bad('events-empty',every('arms',lambda r:r.__setitem__('events',[])))
    bad('target-zero',lambda d:[r.__setitem__('events',[]) for name in CASES if name!='control' for r in d[name]])
    bad('run-difference',lambda d:d['arms'][1]['events'].pop(),'controls')
    bad('projection-difference',every('arms',lambda r:supply(r,'push:leave').__setitem__('ret',9)),'controls')
    bad('null-context',every('arms',lambda r:supply(r)['supplier'].__setitem__('context',None)))
    bad('null-alpha',every('arms',lambda r:supply(r).__setitem__('alpha',None)))
    bad('candidate-ret',every('arms',lambda r:supply(r).__setitem__('candidate',3)))
    bad('cursor',every('arms',lambda r:supply(r)['supplier']['context'].__setitem__('position',0)))
    bad('deferred',every('arms',lambda r:supply(r)['supplier']['context'].__setitem__('deferred',1)))
    bad('failures',every('arms',lambda r:supply(r)['supplier']['context']['failures'].__setitem__(0,7)))
    bad('marks',every('arms',lambda r:supply(r)['supplier']['context']['support_marks'].__setitem__(0,1)))
    bad('success-counter',every('arms',lambda r:supply(r)['supplier']['context'].__setitem__('successes',0)))
    bad('count-sum',every('arms',lambda r:supply(r)['supplier']['context'].__setitem__('count_sum',bits(99.))))
    bad('output-alias',every('arms',lambda r:supply(r)['supplier'].__setitem__('output_is_s210',False)))
    bad('ftran-spike-alias',every('arms',lambda r:supply(r,'supplier:ftran:enter')['aliases'].__setitem__('spike_null',False)))
    bad('ftran-column',every('arms',lambda r:supply(r,'supplier:ftran:enter')['input']['values'].__setitem__(0,bits(4.))))
    bad('raw-null',every('arms',lambda r:supply(r,'supplier:ftran:leave').__setitem__('raw',None)))
    bad('raw-value',every('arms',lambda r:supply(r,'supplier:ftran:leave')['raw']['values'].__setitem__(0,bits(4.))))
    bad('dense-keeps-equal-tol',every('synthetic',lambda r:synth(r,'dense-boundaries')['alpha']['indices'].__setitem__(0,0)))
    bad('dense-order',every('synthetic',lambda r:synth(r,'dense-boundaries')['alpha']['indices'].reverse()))
    bad('dense-count-stat',every('synthetic',lambda r:synth(r,'dense-boundaries')['after'].__setitem__('count_sum',bits(12.25))))
    bad('sparse-order',every('synthetic',lambda r:synth(r,'structural-sparse')['alpha']['indices'].reverse()))
    bad('negative-support-not-normalized',every('synthetic',lambda r:synth(r,'negative-support')['after'].__setitem__('support',-1)))
    bad('skip-failure-increments',every('synthetic',lambda r:synth(r,'failure-three')['after']['failures'].__setitem__(0,4)))
    bad('deferred-prefix-order',every('synthetic',lambda r:synth(r,'prefix-preserved')['after']['candidates'].__setitem__(0,0)))
    bad('exhausted-null-null',every('synthetic',lambda r:synth(r,'all-failure').__setitem__('after',None)))
    bad('synthetic-only',lambda d:[r.__setitem__('origin','synthetic-backend') for r in d['arms']])
    return out


def main():
    p=argparse.ArgumentParser(description=__doc__)
    g=p.add_mutually_exclusive_group(required=True)
    for flag in ('controls','acceptance','self-test','write-manifest'):g.add_argument('--'+flag,action='store_true')
    p.add_argument('--output',type=Path);a=p.parse_args()
    try:
        d=records();mp=ROOT/D/'manifest.json'
        if a.write_manifest:
            m=manifest();controls(d,m);acceptance(d)
            txt=json.dumps(m,ensure_ascii=False,indent=1)+'\n'
            require(not mp.exists() or mp.read_text()==txt,'기존 manifest와 다름')
            mp.write_text(txt);result=dict(manifest=str(mp.relative_to(ROOT)))
        else:
            m=load(f'{D}/manifest.json');check_files(m)
            if a.controls:result=controls(d,m)
            elif a.acceptance:controls(d,m);result=acceptance(d)
            else:controls(d,m);acceptance(d);result=dict(rejected=self_test(d,m))
        txt=json.dumps(result,ensure_ascii=False,indent=1)+'\n'
        if a.output:
            require(not a.output.exists(),'기존 결과 덮어쓰기 거부');a.output.write_text(txt)
        print(txt,end='')
    except (Fail,KeyError,TypeError,ValueError,IndexError,OSError) as e:
        print('불합격: '+str(e),file=sys.stderr);sys.exit(1)


if __name__=='__main__':main()
