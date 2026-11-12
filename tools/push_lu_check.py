"""M3-04c-21e: 지연 LU 연쇄 오라클의 출처/독립3회/선행21c 비간섭, 원본 C 규칙 판정(연산 사이 factor 보존·FT 갱신·재분해 재번호), 변조 거부."""
import argparse
import copy
import gzip
import hashlib
import json
from pathlib import Path
import sys

from push_supplier_check import Fail, equal, require
from push_ftran_check import check_factor, flip
from push_btran_check import check_lrow

ROOT=Path(__file__).resolve().parents[1]
D=Path('bench/oracle/m3_push_lu_chain')
BT=Path('bench/oracle/m3_push_btran')
BINARY_SHA='8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c'
SCHEMA='M3-04c-21e-v1'
LOCAL=('arms','retry','interior','control')
NETLIB=('adlittle','scsd1','capri')
NAMES=LOCAL+NETLIB
CASES={
 'arms':('bench/oracle/m3_push_arms/arms.lp','bench/oracle/m3_push_arms/arms.extra.json'),
 'retry':('bench/oracle/m3_push_retry/retry.lp','bench/oracle/m3_push_retry/retry.extra.json'),
 'interior':('bench/oracle/m3_primal_push_step/interior.lp',None),
 'control':('bench/oracle/m3_push_arms/arms.lp','bench/oracle/m3_push_arms/arms.extra.json'),
 'adlittle':('../re/bench/data/netlib/adlittle.mps.gz',None),
 'scsd1':('../re/bench/data/netlib/scsd1.mps.gz',None),
 'capri':('../re/bench/data/netlib/capri.mps.gz',None)}
# push 안 05b257 호출 수. 탐색 history/explore에서 고정(선행 집계 arms4·retry3·interior9와 일치).
EXPECTED={'arms':4,'retry':3,'interior':9,'control':0,'adlittle':14,'scsd1':2,'capri':23}
TOOLS=('tools/push_lu_agent.js','tools/push_lu_capture.js','tools/push_lu_runs.py','tools/push_lu_check.py',
       'tools/push_btran_agent.js','tools/push_btran_capture.js','tools/push_btran_check.py',
       'tools/push_ftran_check.py','tools/push_supplier_check.py','tools/frida_resolve.cjs','tools/gurun.sh')
SOURCES=('../re/decomp/c/000f15c0_PRIVATE000000000005b257.c','../re/decomp/c/000d4190_PRIVATE0000000000036ade.c',
         '../re/decomp/c/00131760_PRIVATE000000000009e47c.c')
ETA_OLDEST_RESET=2000000000


def sha(p):return hashlib.sha256((ROOT/p).read_bytes()).hexdigest()
def capture(d,n,i):return d/(f'{n}-{i}.json.gz' if n in NETLIB else f'{n}-{i}.json')
def load(p):
    b=(ROOT/p).read_bytes()
    return json.loads(gzip.decompress(b) if str(p).endswith('.gz') else b)
def baseline_path(n):return capture(BT,n,1)
def records(d=D,names=NAMES,runs=(1,2,3)):return {n:[load(capture(d,n,i)) for i in runs] for n in names}


def file_sets():
    return dict(tools=set(TOOLS),inputs={p for model,extra in CASES.values() for p in (model,extra) if p},
        sources=set(SOURCES),assembly={str(D/'source.asm')},
        captures={str(capture(D,n,i)) for n in NAMES for i in (1,2,3)},
        baselines={str(baseline_path(n)) for n in NAMES}|{str(BT/'manifest.json')},
        history={str(p.relative_to(ROOT)) for p in (ROOT/D/'history').rglob('*') if p.is_file()})


def manifest():
    return dict(task='M3-04c-21e',binary_sha256=BINARY_SHA,**{k:{p:sha(p) for p in sorted(v)} for k,v in file_sets().items()})


def check_files(m):
    equal('M3-04c-21e',m['task'],'manifest.task');equal(BINARY_SHA,m['binary_sha256'],'manifest.binary')
    for k,paths in file_sets().items():
        equal(paths,set(m[k]),'manifest.'+k+'.keys')
        for p in paths:
            if k in ('sources','inputs') and p.startswith('../') and not (ROOT/p).exists():continue
            equal(m[k][p],sha(p),'manifest.'+p)


def is_new(e):return e['event'].startswith('lu:')
def project(r):return [e for e in r['events'] if not is_new(e)]


def controls(data,m):
    out={}
    for name in data:
        rs=data[name];equal(3,len(rs),name+'.runs')
        equal(3,len({r['pid'] for r in rs}),name+'.pid');equal(3,len({r['run_id'] for r in rs}),name+'.run_id')
        model,extra=CASES[name];ex=load(extra) if extra else {};base=load(baseline_path(name))
        for i,r in enumerate(rs):
            label=f'{name}/{i+1}'
            equal(BINARY_SHA,r['binary_sha256'],label+'.binary');equal(SCHEMA,r['schema'],label+'.schema')
            equal('natural',r['origin'],label+'.origin');equal('control' if name=='control' else 'primal',r['kind'],label+'.kind')
            equal(m['tools']['tools/push_lu_capture.js'],r['tool_sha256'],label+'.tool')
            equal(m['tools']['tools/push_lu_agent.js'],r['agent_sha256'],label+'.agent')
            equal(m['inputs'][model],r['model_sha256'],label+'.model')
            params=dict(Threads=1,Seed=0,OutputFlag=0,Presolve=0,Method=2,Crossover=0 if name=='control' else 3,LPWarmStart=2)
            params.update(ex.get('params',{}));equal(params,r['params'],label+'.params');require(ex.get('starts')==r['starts'],label+'.starts')
            equal([],r['errors'],label+'.errors');equal('process-terminated',r['reason'],label+'.reason')
            equal(rs[0]['events'],r['events'],label+'.events');equal(rs[0]['environment'],r['environment'],label+'.environment')
            equal(rs[0]['result'],r['result'],label+'.result')
            # 비간섭: 새 사건을 빼면 선행21c 정식 캡처와 사건·결과·출력 전체가 같다.
            equal(base['events'],project(r),label+'.projection');equal(base['result'],r['result'],label+'.baseline_result')
            equal(base['output'],r['output'],label+'.baseline_output')
        equal(m['tools']['tools/push_btran_agent.js'],base['agent_sha256'],name+'.baseline.agent')
        new=[e for e in rs[0]['events'] if is_new(e)]
        if name=='control':equal(0,len(rs[0]['events']),'control.events')
        else:require(new,name+': 새 LU 사건0')
        out[name]=dict(events=len(rs[0]['events']),lu=len(new))
    return out


# ---------- 원본 C 규칙 ----------
OPS={'ftran':('ftran:enter','ftran:leave'),'btran':('btran:enter','btran:leave'),'lu':('lu:enter','lu:leave')}
ENTER={v[0]:k for k,v in OPS.items()};LEAVE={v[1]:k for k,v in OPS.items()}


def lu_calls(r):
    """push 안 factor:enter/lu:enter … lu:leave/factor:leave 묶음(규칙1)."""
    ev=r['events'];out=[];i=0
    while i<len(ev):
        e=ev[i]
        if e['event']=='factor:enter':
            require(ev[i+1]['event']=='lu:enter',f'#{i} factor:enter 뒤 lu:enter 없음')
            j=i+2;body=[]
            while ev[j]['event']!='factor:leave':
                body.append(ev[j]);j+=1
            # agent onLeave 순서: factor:leave(선행15) 뒤 lu:leave.
            require(j+1<len(ev) and ev[j+1]['event']=='lu:leave',f'#{j} factor:leave 뒤 lu:leave 없음')
            out.append(dict(fen=e,en=ev[i+1],body=body,lv=ev[j+1],flv=ev[j]));i=j+2
        else:
            require(e['event'] not in ('lu:enter','lu:leave','lu:roweta','lu:refactor'),f'#{i} factor 밖 LU 사건 {e["event"]}');i+=1
    return out


def spike_span(f,r):
    """C:263–309: i52=max(F40[r],F100−1), i35=max(i52, max_{s∈spike, s≠r} F40[s])."""
    x40=f['perm']['x40'];i52=max(x40[r],f['scalars']['x100']-1);i35=i52
    for s in f['spike']['indices']:
        if s!=r:i35=max(i35,x40[s])
    return i35,i52


def check_call(c,label,stats):
    fen,en,body,lv,flv=c['fen'],c['en'],c['body'],c['lv'],c['flv']
    # 규칙1: 식별
    equal(en['id'],lv['id'],label+'.id');equal(fen['request'],en['request'],label+'.request');equal(flv['ret'],lv['ret'],label+'.ret')
    for e in body:
        require(e['event'] in ('lu:roweta','lu:refactor','ftran:enter','ftran:leave','ftran:kernel'),f'{label}: factor 안 다른 사건 {e["event"]}')
        require(e['event'] not in ('ftran:enter','ftran:leave','ftran:kernel'),f'{label}: factor 안 FTRAN')
        equal(en['id'],e['id'],label+'.body id')
    rw=[e for e in body if e['event']=='lu:roweta'];rf=[e for e in body if e['event']=='lu:refactor']
    for evs,tag in ((rw,'roweta'),(rf,'refactor')):
        equal(['enter','leave']*(len(evs)//2),[e['phase'] for e in evs],f'{label}.{tag} phases');equal(0,len(evs)%2,f'{label}.{tag} 쌍')
    if en['request']==0:equal([],rf,label+': 요청0에 재분해')
    else:equal([],rw,label+': 요청≠0에 행 eta')
    equal(0,lv['ret'],label+': 반환0 전제(반환≠0은 계약 밖)')
    m=len(en['header'])
    f1=lv['factor'];check_factor(f1,m,label+'.leave.factor');check_lrow(f1,m,label+'.leave.factor')
    # 규칙6: 공통 종료
    equal(0,f1['cache']['count'],label+'.반환 cache.count');equal(-1,f1['x1d8'],label+'.반환 F1d8')
    equal(f1['header'],lv['header'],label+'.leave header 필드')
    b=lv['buffers'];equal(True,b['factor_same'],label+'.F 동일')
    if en['request']==0 and en['x290']==0:
        # 규칙4: FT 갱신
        stats['ft']+=1
        f0=en['factor'];check_factor(f0,m,label+'.enter.factor');check_lrow(f0,m,label+'.enter.factor')
        r=en['leave'];sp=f0['spike'];require(sp is not None and sp['count']>=1,label+': spike count≥1')
        i35,i52=spike_span(f0,r);need=i35!=i52 and r!=f0['x1d8']
        equal(need,bool(rw),f'{label}.행 eta 호출(기준={need}, i35={i35}, i52={i52}, r={r}, F1d8={f0["x1d8"]})')
        if rw:equal(r,rw[0]['args']['row'],label+'.행 eta 인자');equal(dict(F=True,s220=True,row=r,work=True),rw[0]['args'],label+'.행 eta args');stats['roweta']+=1
        elif i35!=i52:stats['cache_reuse']+=1
        else:stats['span_trivial']+=1
        equal(f0['scalars']['x18c']+1,f1['scalars']['x18c'],label+'.F18c+1')
        equal(0,f1['spike']['count'],label+'.spike count0');equal(en['header'],lv['header'],label+'.header 불변')
        equal(True,b['header_same'] and b['spike_idx_same'],label+'.버퍼 불변')
        d=f1['scalars']['x180']-f0['scalars']['x180'];require(d in (0,1),f'{label}.F180 증가 {d}')
        if d==1:require(i35!=i52,label+': 행 eta 생성인데 i35=i52');stats['eta_added']+=1
    else:
        require(en['request']!=0,label+': S+0x290≠0 경로는 계약 밖')
        require(rf,label+': 요청≠0인데 036ade 없음')
        stats['refactor']+=1;stats['refactor_calls']+=len(rf)//2
        last=rf[-1];equal(0,last['ret'],label+'.036ade 반환');equal([0,0],last['out'],label+'.036ade 오류 칸')
        equal(dict(S=True,m=m),rf[0]['args'],label+'.036ade args')
        require(all(en['pricing_null'][:2]),label+': pricing 객체 경로는 계약 밖')
        o=last['state']['x28'];n=last['state']['x38']
        # 규칙5
        equal([en['header'][o[p]] for p in range(m)],lv['header'],label+'.header 재번호')
        if en['leave']>=0:equal(n[en['leave']],lv['leave'],label+'.이탈 위치 재번호');stats['leave_renumbered']+=1
        for p,h in enumerate(lv['header']):equal(p,lv['vstat'][h],f'{label}.vstat[header[{p}]]')
        ident=list(range(m))
        equal(ident,f1['perm']['x28'],label+'.F28 identity');equal(ident,f1['perm']['x38'],label+'.F38 identity')
        equal(True,b['x40_is_x28'] and b['x48_is_x38'],label+'.F40/F48 버퍼')
        equal([ETA_OLDEST_RESET]*m,f1['eta_oldest'],label+'.F1a0')
        equal(0,f1['spike']['count'],label+'.spike count0')
        equal(True,b['spike_idx_is_old_header'] and b['header_is_old_spike_idx'],label+'.spike idx/header 버퍼 교환')
        equal((0,0,0),(f1['scalars']['x180'],f1['scalars']['x184'],f1['scalars']['x18c']),label+'.F180/184/18c')
        stats['header_reordered']+=lv['header']!=en['header']
        stats['old_perm_nonid']+=o!=ident
        stats['init' if en['caller']=='0x1c07c6' else 'refactor_push']+=1


def chain(r,label,stats):
    """규칙3: 같은 push 호출 안 연산 사이 factor 보존(header 제외, spike는 다음 공급 FTRAN wrapper clear 제외)."""
    cur=None;call=None;prev=None;hdr=None
    for i,e in enumerate(r['events']):
        if e['event'] in ('push:enter','push:leave'):cur=None;prev=None;call=e['call'];hdr=None;continue
        k=e['event']
        if k in ENTER:
            f=e.get('factor')
            if f is not None and cur is not None:
                skip={'header'}
                if k=='ftran:enter' and e['site']=='supplier' and not e['args']['spike_null']:skip.add('spike')
                for key,v in f.items():
                    if key in skip or key not in cur:continue
                    if key=='scalars':
                        # FTRAN 스냅숏(factorState)에는 x140·x1e0이 없다. 공통 스칼라만 비교한다.
                        for sk,sv in v.items():
                            if sk in cur[key]:equal(cur[key][sk],sv,f'{label}#{i} {k}.factor.scalars.{sk} 연산 사이 보존(직전 {prev})')
                    elif key=='Lrow':
                        # bucket은 F+0x1a8 scratch(FTRAN 레벨 L·BTRAN 레벨 Lᵀ가 같이 씀, 21a §2·21c C:620–624)라 factor 보존 대상이 아니다.
                        a={x:y for x,y in cur[key].items() if x!='bucket'};c={x:y for x,y in v.items() if x!='bucket'}
                        equal(a,c,f'{label}#{i} {k}.factor.Lrow 연산 사이 보존(직전 {prev})')
                        stats['links_bucket_changed']+=cur[key]['bucket']!=v['bucket']
                    else:equal(cur[key],v,f'{label}#{i} {k}.factor.{key} 연산 사이 보존(직전 {prev})')
                stats['links']+=1
                if 'header' in cur and cur['header']!=f['header']:stats['links_header_changed']+=1
                if prev=='lu:leave':stats['links_after_lu']+=1
                if ENTER[k]=='lu':stats['links_into_lu']+=1
        if k in LEAVE:
            f=e.get('factor')
            if f is not None:
                sc=dict((cur or {}).get('scalars',{}));sc.update(f['scalars'])
                cur=dict(cur or {});cur.update(f);cur['scalars']=sc;prev=k


STAT_KEYS=('ft','roweta','cache_reuse','span_trivial','eta_added','refactor','refactor_calls','init','refactor_push',
    'leave_renumbered','header_reordered','old_perm_nonid','links','links_header_changed','links_bucket_changed','links_after_lu','links_into_lu')


def acceptance(data):
    out={}
    for name in data:
        r=data[name][0];cs=lu_calls(r);stats={k:0 for k in STAT_KEYS}
        equal(EXPECTED[name],len(cs),name+'.factor 호출 수')
        for k,c in enumerate(cs):check_call(c,f'{name}#{k+1}',stats)
        chain(r,name,stats)
        require(len([e for e in r['events'] if is_new(e)])==sum(2+len(c['body']) for c in cs),name+'.사건 수')
        out[name]=dict(calls=len(cs),**stats)
    tot={k:sum(v[k] for v in out.values()) for k in ('calls',)+STAT_KEYS}
    require(tot['calls']==sum(EXPECTED[n] for n in data),'전체 호출 수')
    return dict(models=out,total=tot)


# ---------- 변조 ----------
def evs(d,model,event,run=0,phase=None,pred=None):
    return [e for e in d[model][run]['events'] if e['event']==event and (phase is None or e.get('phase')==phase) and (pred is None or pred(e))]


def ft_call(d,model,with_roweta=True,k=0):
    """요청0 FT 갱신 lu:enter 중 행 eta 호출 유무로 고른 k번째의 (enter, leave, body)."""
    ev=d[model][0]['events'];out=[]
    for i,e in enumerate(ev):
        if e['event']=='lu:enter' and e['request']==0:
            j=i+1;body=[]
            while ev[j]['event']!='factor:leave':body.append(ev[j]);j+=1
            if bool([b for b in body if b['event']=='lu:roweta'])==with_roweta:out.append((e,ev[j+1],body,i))
    return out[k]


def rf_call(d,model,k=0,push=True):
    ev=d[model][0]['events'];out=[]
    for i,e in enumerate(ev):
        if e['event']=='lu:enter' and e['request']!=0 and (e['caller']!='0x1c07c6')==push:
            j=i+1;body=[]
            while ev[j]['event']!='factor:leave':body.append(ev[j]);j+=1
            out.append((e,ev[j+1],body,i))
    return out[k]


def link_header_changed(d,model):
    """support 생략 pivot 뒤(header만 바뀐) 다음 연산 진입 사건."""
    cur=None
    for e in d[model][0]['events']:
        if e['event'] in ('push:enter','push:leave'):cur=None;continue
        if e['event'] in ENTER and e.get('factor') is not None and cur is not None and cur!=e['factor']['header']:return e
        if e['event'] in LEAVE and e.get('factor') is not None:cur=e['factor']['header']
    raise Fail('header 변경 연결 없음')


def self_test(data,m):
    rejected=[]
    def test(name,mutate,check):
        d=copy.deepcopy(data);mm=copy.deepcopy(m);mutate(d,mm)
        try:check(d,mm)
        except (Fail,KeyError,TypeError,IndexError,ValueError,StopIteration) as exc:rejected.append(dict(name=name,error=str(exc)[:200]));return
        raise Fail('NOT REJECTED: '+name)
    ctl=lambda d,mm:controls(d,mm);acc=lambda d,mm:acceptance(d)
    # 출처·재현·비간섭
    test('binary-hash',lambda d,mm:d['arms'][1].update(binary_sha256='0'*64),ctl)
    test('manifest-hash',lambda d,mm:mm['tools'].update({'tools/push_lu_agent.js':'0'*64}),lambda d,mm:check_files(mm))
    test('empty-events',lambda d,mm:[r.update(events=[]) for r in d['retry']],ctl)
    test('run-diff',lambda d,mm:evs(d,'capri','lu:leave',run=2)[3]['factor']['diag'].__setitem__(0,flip(evs(d,'capri','lu:leave',run=2)[3]['factor']['diag'][0])),ctl)
    test('projection-diff',lambda d,mm:[next(e for e in r['events'] if e['event']=='factor:leave').update(ret=1) for r in d['arms']],ctl)
    test('netlib-projection-diff',lambda d,mm:[next(e for e in r['events'] if e['event']=='btran:leave')['output'].update(count=9) for r in d['adlittle']],ctl)
    test('baseline-result',lambda d,mm:[r['result'].update(status=3) for r in d['scsd1']],ctl)
    test('no-new-events',lambda d,mm:[r.update(events=project(r)) for r in d['interior']],ctl)
    test('model-hash',lambda d,mm:[r.update(model_sha256='0'*64) for r in d['capri']],ctl)
    # 규칙1 구조
    test('R1-missing-lu-leave',lambda d,mm:d['retry'][0].update(events=[e for e in d['retry'][0]['events'] if e['event']!='lu:leave']),acc)
    test('R1-missing-lu-enter',lambda d,mm:d['arms'][0].update(events=[e for e in d['arms'][0]['events'] if e['event']!='lu:enter']),acc)
    test('R1-null-null',lambda d,mm:[e.update(factor=None) for e in d['scsd1'][0]['events'] if e['event']=='lu:leave'],acc)
    test('R1-ret',lambda d,mm:evs(d,'interior','lu:leave')[2].update(ret=1),acc)
    test('R1-request',lambda d,mm:evs(d,'adlittle','lu:enter')[4].update(request=7),acc)
    test('R1-lu-outside',lambda d,mm:d['retry'][0]['events'].insert(1,dict(evs(d,'retry','lu:roweta' if evs(d,'retry','lu:roweta') else 'lu:refactor')[0])),acc)
    def roweta_in_refactor(d,mm):
        en,lv,body,i=rf_call(d,'capri');ev=d['capri'][0]['events'];rw=evs(d,'capri','lu:roweta')[:2]
        for x in reversed(rw):y=dict(x);y['id']=en['id'];ev.insert(i+1,y)
    test('R1-roweta-in-refactor',roweta_in_refactor,acc)
    # 규칙2 factor 무결성
    test('R2-zero',lambda d,mm:evs(d,'arms','lu:leave')[1]['factor'].update(zero='0000000000000000'),acc)
    test('R2-perm',lambda d,mm:evs(d,'capri','lu:leave')[5]['factor']['perm']['x30'].reverse(),acc)
    test('R2-Lrow',lambda d,mm:evs(d,'adlittle','lu:leave')[6]['factor']['Lrow']['vals'].__setitem__(0,flip(evs(d,'adlittle','lu:leave')[6]['factor']['Lrow']['vals'][0])),acc)
    test('R2-scratch',lambda d,mm:evs(d,'interior','lu:leave')[4]['factor']['scratch']['x1b8'].append([0,'000000000000f03f']),acc)
    # 규칙3 연산 사이 보존
    def skip_factor(d,mm):
        e=link_header_changed(d,'arms');e['factor']['perm']['x40'].reverse()
    test('R3-support-skip-factor-changed',skip_factor,acc)
    def skip_refactor(d,mm):
        # "pivot마다 재분해" 변이: header 변경 연결 뒤 factor 순열을 새 header 기준 identity로 바꾼다.
        e=link_header_changed(d,'retry');mm_=len(e['factor']['header']);e['factor']['perm']['x28']=list(range(mm_));e['factor']['perm']['x38']=list(range(mm_))
        e['factor']['perm']['x20'],e['factor']['perm']['x30']=e['factor']['perm']['x30'],e['factor']['perm']['x20']
    test('R3-refactor-every-pivot',skip_refactor,acc)
    def always_update(d,mm):
        # "pivot마다 FT 갱신" 변이: support 생략 연결의 갱신 수를 1 올린다.
        e=link_header_changed(d,'capri');e['factor']['scalars']['x18c']+=1
    test('R3-update-every-pivot',always_update,acc)
    def after_lu(d,mm):
        en,lv,body,i=ft_call(d,'adlittle');ev=d['adlittle'][0]['events']
        nxt=next(e for e in ev[i+1:] if e['event'] in ENTER and e.get('factor') is not None)
        nxt['factor']['diag'][0]=flip(nxt['factor']['diag'][0])
    test('R3-after-ft',after_lu,acc)
    def after_rf(d,mm):
        en,lv,body,i=rf_call(d,'interior');ev=d['interior'][0]['events']
        nxt=next(e for e in ev[i+1:] if e['event'] in ENTER and e.get('factor') is not None)
        nxt['factor']['etas'].append(dict(r=0,len=0,idx_off=0,val_off=0,idx_mod16=0,val_mod16=0,idx=[],val=[]))
    test('R3-after-refactor-eta',after_rf,acc)
    def spike_into(d,mm):
        en,lv,body,i=ft_call(d,'capri');sp=en['factor']['spike'];sp['values'][0]=flip(sp['values'][0])
    test('R3-spike-into-ft',spike_into,acc)
    def cache_link(d,mm):
        e=evs(d,'capri','btran:enter')[2];e['factor']['x1d8']=5
    test('R3-x1d8-between',cache_link,acc)
    # 규칙4 FT 갱신
    def drop_roweta(d,mm):
        en,lv,body,i=ft_call(d,'capri');ids={id(b) for b in body if b['event']=='lu:roweta'}
        d['capri'][0]['events']=[e for e in d['capri'][0]['events'] if id(e) not in ids]
    test('R4-roweta-missing',drop_roweta,acc)
    def add_roweta(d,mm):
        en,lv,body,i=ft_call(d,'adlittle',with_roweta=False);rw=evs(d,'adlittle','lu:roweta')[:2];ev=d['adlittle'][0]['events']
        for x in reversed(rw):
            y=copy.deepcopy(x);y['id']=en['id']
            if 'args' in y:y['args']['row']=en['leave']
            ev.insert(i+1,y)
    test('R4-roweta-extra',add_roweta,acc)
    def roweta_row(d,mm):
        en,lv,body,i=ft_call(d,'interior');body[0]['args']['row']+=1
    test('R4-roweta-row',roweta_row,acc)
    def cache_claim(d,mm):
        # F1d8=r이면 cache를 다시 쓰므로 행 eta를 부르면 안 된다(C:311–320).
        en,lv,body,i=ft_call(d,'capri');en['factor']['x1d8']=en['leave']
    test('R4-cache-reuse-predicate',cache_claim,acc)
    def span_pred(d,mm):
        en,lv,body,i=ft_call(d,'adlittle');f=en['factor'];r=en['leave'];f['perm']['x40']=[0]*len(f['perm']['x40']);f['scalars']['x100']=1
    test('R4-span-predicate',span_pred,acc)
    test('R4-x18c',lambda d,mm:ft_call(d,'interior')[1]['factor']['scalars'].update(x18c=0),acc)
    test('R4-spike-left',lambda d,mm:ft_call(d,'capri',k=3)[1]['factor']['spike'].update(count=1,indices=[0],values=['000000000000f03f']),acc)
    def ft_header(d,mm):
        lv=ft_call(d,'adlittle',k=2)[1];lv['header'][0],lv['header'][1]=lv['header'][1],lv['header'][0];lv['factor']['header']=list(lv['header'])
    test('R4-header-changed',ft_header,acc)
    test('R4-eta-delta',lambda d,mm:ft_call(d,'capri',k=1)[1]['factor']['scalars'].update(x180=ft_call(d,'capri',k=1)[0]['factor']['scalars']['x180']+2),acc)
    def eta_trivial(d,mm):
        en,lv,body,i=ft_call(d,'capri',with_roweta=False);lv['factor']['scalars']['x180']+=1
    test('R4-eta-without-span',eta_trivial,acc)
    test('R4-buffer',lambda d,mm:ft_call(d,'interior')[1]['buffers'].update(spike_idx_same=False),acc)
    # 규칙5 재분해·재번호
    def rf_header(d,mm):
        lv=rf_call(d,'arms')[1];h=lv['header'];h[0],h[1]=h[1],h[0];lv['factor']['header']=list(h)
        v=lv['vstat'];v[h[0]],v[h[1]]=0,1
    test('R5-header-renumber',rf_header,acc)
    def rf_old(d,mm):
        body=rf_call(d,'capri')[2];o=body[-1]['state']['x28'];o[0],o[1]=o[1],o[0]
    test('R5-old-of-new',rf_old,acc)
    test('R5-leave',lambda d,mm:rf_call(d,'retry')[1].update(leave=rf_call(d,'retry')[1]['leave']+1),acc)
    def rf_newold(d,mm):
        en,lv,body,i=rf_call(d,'interior');n=body[-1]['state']['x38'];r=en['leave'];n[r]=(n[r]+1)%len(n)
    test('R5-new-of-old',rf_newold,acc)
    def rf_vstat(d,mm):
        lv=rf_call(d,'scsd1',push=False)[1];h=lv['header'];lv['vstat'][h[0]]+=1
    test('R5-vstat',rf_vstat,acc)
    def rf_ident(d,mm):
        f=rf_call(d,'adlittle')[1]['factor'];f['perm']['x28'][0],f['perm']['x28'][1]=1,0;f['perm']['x38'][0],f['perm']['x38'][1]=1,0
        f['perm']['x40'],f['perm']['x48']=list(f['perm']['x28']),list(f['perm']['x38'])
    test('R5-identity',rf_ident,acc)
    test('R5-buffers-alias',lambda d,mm:rf_call(d,'capri')[1]['buffers'].update(x40_is_x28=False),acc)
    test('R5-buffer-swap',lambda d,mm:rf_call(d,'arms',k=1)[1]['buffers'].update(spike_idx_is_old_header=False),acc)
    test('R5-eta-oldest',lambda d,mm:rf_call(d,'interior',k=1)[1]['factor']['eta_oldest'].__setitem__(0,0),acc)
    test('R5-reset',lambda d,mm:rf_call(d,'capri',k=1)[1]['factor']['scalars'].update(x18c=1),acc)
    test('R5-036ade-ret',lambda d,mm:rf_call(d,'retry')[2][-1].update(ret=1),acc)
    test('R5-036ade-err',lambda d,mm:rf_call(d,'arms')[2][-1].update(out=[0,3]),acc)
    test('R5-pricing',lambda d,mm:rf_call(d,'scsd1')[0].update(pricing_null=[True,False,True]),acc)
    test('R5-spike-left',lambda d,mm:rf_call(d,'adlittle',k=1)[1]['factor']['spike'].update(count=1,indices=[0],values=['000000000000f03f']),acc)
    # 규칙6 공통 종료
    test('R6-cache',lambda d,mm:ft_call(d,'adlittle')[1]['factor'].update(cache={'count':1,'idx':[0],'val':['000000000000f03f']}),acc)
    test('R6-x1d8',lambda d,mm:rf_call(d,'capri')[1]['factor'].update(x1d8=0),acc)
    return dict(rejected=len(rejected),details=rejected)


def main():
    p=argparse.ArgumentParser(description=__doc__)
    for flag in ('controls','acceptance','self-test','write-manifest'):p.add_argument('--'+flag,action='store_true')
    p.add_argument('--report',type=Path)
    p.add_argument('--explore',type=Path,help='탐색 자료 1회분에 acceptance만 적용(판정 자료 아님)')
    a=p.parse_args()
    if a.explore:
        data={}
        for n in NAMES:
            for suf in ('.json','.json.gz'):
                if (ROOT/a.explore/f'{n}-1{suf}').exists():data[n]=[load(a.explore/f'{n}-1{suf}')]
        print(json.dumps(acceptance(data),ensure_ascii=False));return
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
