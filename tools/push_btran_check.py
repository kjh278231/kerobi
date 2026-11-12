"""M3-04c-21c: 지원 BTRAN 본체 오라클의 출처/독립3회/선행21a 비간섭, 원본 C 규칙 판정, 변조 거부."""
import argparse
import copy
import gzip
import hashlib
import json
import math
from pathlib import Path
import sys

from push_supplier_check import Fail, equal, require
from push_ftran_check import check_factor, check_vec, f64, flip

ROOT=Path(__file__).resolve().parents[1]
D=Path('bench/oracle/m3_push_btran')
FT=Path('bench/oracle/m3_push_ftran')
BINARY_SHA='8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c'
SCHEMA='M3-04c-21c-v1'
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
# 지원 BTRAN 호출 수. arms/retry/interior는 선행19 저장 자료(M3-04c-21 survey §4.1), Netlib은 이번 탐색 history/explore2에서 고정.
EXPECTED={'arms':17,'retry':4,'interior':1,'control':0,'adlittle':2,'scsd1':1,'capri':7}
TOOLS=('tools/push_btran_agent.js','tools/push_btran_capture.js','tools/push_btran_runs.py','tools/push_btran_check.py',
       'tools/push_btran_survey.js','tools/push_btran_survey_agent.js',
       'tools/push_ftran_agent.js','tools/push_ftran_capture.js','tools/push_ftran_check.py',
       'tools/push_supplier_check.py','tools/frida_resolve.cjs','tools/gurun.sh')
SOURCES=('../re/decomp/c/00131710_PRIVATE000000000009e376.c','../re/decomp/c/0012de60_PRIVATE0000000000221173.c',
         '../re/decomp/c/00130b70_PRIVATE000000000021a941.c')
ZERO_BITS='0000000000000000'
# 진입과 반환 사이에 바뀔 수 있는 factor 필드(C: cache 경로 C:1197–1227, level bucket C:620–624)
VOLATILE=('cache','x1d8')


def sha(p):return hashlib.sha256((ROOT/p).read_bytes()).hexdigest()
def capture(d,n,i):return d/(f'{n}-{i}.json.gz' if n in NETLIB else f'{n}-{i}.json')
def load(p):
    b=(ROOT/p).read_bytes()
    return json.loads(gzip.decompress(b) if str(p).endswith('.gz') else b)
def baseline_path(n):return D/'baseline'/f'{n}-1.json.gz' if n in NETLIB else FT/f'{n}-1.json'
def records(d=D,names=NAMES,runs=(1,2,3)):return {n:[load(capture(d,n,i)) for i in runs] for n in names}


def file_sets():
    return dict(tools=set(TOOLS),inputs={p for model,extra in CASES.values() for p in (model,extra) if p},
        sources=set(SOURCES),assembly={str(D/'source.asm')},
        captures={str(capture(D,n,i)) for n in NAMES for i in (1,2,3)},
        baselines={str(baseline_path(n)) for n in NAMES}|{str(FT/'manifest.json')},
        history={str(p.relative_to(ROOT)) for p in (ROOT/D/'history').rglob('*') if p.is_file()})


def manifest():
    return dict(task='M3-04c-21c',binary_sha256=BINARY_SHA,**{k:{p:sha(p) for p in sorted(v)} for k,v in file_sets().items()})


def check_files(m):
    equal('M3-04c-21c',m['task'],'manifest.task');equal(BINARY_SHA,m['binary_sha256'],'manifest.binary')
    for k,paths in file_sets().items():
        equal(paths,set(m[k]),'manifest.'+k+'.keys')
        for p in paths:
            if k in ('sources','inputs') and p.startswith('../') and not (ROOT/p).exists():continue
            equal(m[k][p],sha(p),'manifest.'+p)


def is_new(e):return e['event'].startswith('btran:')
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
            equal(m['tools']['tools/push_btran_capture.js'],r['tool_sha256'],label+'.tool')
            equal(m['tools']['tools/push_btran_agent.js'],r['agent_sha256'],label+'.agent')
            equal(m['inputs'][model],r['model_sha256'],label+'.model')
            params=dict(Threads=1,Seed=0,OutputFlag=0,Presolve=0,Method=2,Crossover=0 if name=='control' else 3,LPWarmStart=2)
            params.update(ex.get('params',{}));equal(params,r['params'],label+'.params');require(ex.get('starts')==r['starts'],label+'.starts')
            equal([],r['errors'],label+'.errors');equal('process-terminated',r['reason'],label+'.reason')
            equal(rs[0]['events'],r['events'],label+'.events');equal(rs[0]['environment'],r['environment'],label+'.environment')
            equal(rs[0]['result'],r['result'],label+'.result')
            # 비간섭: 새 사건을 빼면 선행21a agent 캡처와 사건·결과·출력 전체가 같다.
            equal(base['events'],project(r),label+'.projection');equal(base['result'],r['result'],label+'.baseline_result')
            equal(base['output'],r['output'],label+'.baseline_output')
        if name in NETLIB:
            b=base;equal(m['tools']['tools/push_ftran_agent.js'],b['agent_sha256'],name+'.baseline.agent')
            equal(m['inputs'][model],b['model_sha256'],name+'.baseline.model');equal([],b['errors'],name+'.baseline.errors')
        new=[e for e in rs[0]['events'] if is_new(e)]
        if name=='control':equal(0,len(rs[0]['events']),'control.events')
        else:require(new,name+': 새 BTRAN 사건0')
        out[name]=dict(events=len(rs[0]['events']),btran=len(new))
    return out


# ---------- 원본 C 규칙 ----------
def check_lrow(f,m,label):
    """L 행 방향(F70/78/80)이 L 열 방향(F58/60/68)과 같은 비대각 원소·값의 전치인지 본다."""
    R=f['Lrow'];beg=R['beg'];equal(m+1,len(beg),label+'.Lrow.beg len')
    require(all(beg[i]<=beg[i+1] for i in range(m)),label+'.Lrow.beg 단조')
    equal(beg[m]-beg[0],len(R['cols']),label+'.Lrow.cols len');equal(len(R['cols']),len(R['vals']),label+'.Lrow.vals len')
    require(all(0<=c<m for c in R['cols']),label+'.Lrow.cols 범위')
    L=f['L'];col=[];lb=L['beg']
    for j in range(m):
        for k in range(lb[j]-lb[0],lb[j+1]-lb[0]):col.append((L['rows'][k],j,L['vals'][k]))
    row=[]
    for i in range(m):
        for k in range(beg[i]-beg[0],beg[i+1]-beg[0]):row.append((i,R['cols'][k],R['vals'][k]))
    equal(sorted(col),sorted(row),label+'.Lrow 전치')
    for h in R['vals']:f64(h)
    s=f['scalars'];equal(s['x140'],R['levels'],label+'.x140')
    if s['x140']>=0:
        require(R['lev'] is not None and len(R['lev'])==m and all(0<=v<=s['x140'] for v in R['lev']),label+'.level 범위')
    else:require(R['lev'] is None and R['bucket'] is None,label+'.level 없음')


def strip(f):
    y=copy.deepcopy(f)
    for k in VOLATILE:y.pop(k)
    y['Lrow'].pop('bucket')
    return y


def calls(r):
    """선행 wrapper 사건(roundctl:btran) 안에 본체 사건열을 묶는다."""
    ev=r['events'];out=[];i=0
    while i<len(ev):
        e=ev[i]
        if e['event']=='roundctl:btran:enter':
            j=i+1;body=[]
            while ev[j]['event']!='roundctl:btran:leave':
                require(is_new(ev[j]),f'wrapper 안 다른 사건 {ev[j]["event"]}');body.append(ev[j]);j+=1
            out.append((e,body,ev[j]));i=j+1
        else:
            require(not is_new(e),f'wrapper 밖 BTRAN 사건 {e["event"]}');i+=1
    return out


def sparse_kernel(f,c,m):
    """C:86–90 일반 경로: count≥0이고 (int)(F108+F110)·count ≤ m이면 희소 Uᵀ(asm 0x12deee cvttsd2si)."""
    s=f['scalars'];t=math.trunc(f64(s['x108'])+f64(s['x110']))
    return c>=0 and t*c<=m


def check_call(wen,body,wlv,label,stats):
    require(len(body)>=2 and body[0]['event']=='btran:enter' and body[-1]['event']=='btran:leave',label+': 본체 enter/leave 1회')
    en,lv=body[0],body[-1];ks=body[1:-1]
    require(all(k['event']=='btran:kernel' and k['id']==en['id'] for k in ks) and lv['id']==en['id'],label+': 사건 id')
    equal(['enter','leave'][:len(ks)],[k['phase'] for k in ks],label+'.kernel phases')
    f=en['factor'];m=f['m']
    # 규칙1: 인자와 wrapper 연결
    equal(dict(factor_s470=True,factor_frame=True,input_frame=True,output_frame=True,cache_null=False,cache_f1d0=True),en['args'],label+'.args')
    w=wen['input'];equal({'count':w['count'],'idx':w['indices'],'val':w['values']},en['input'],label+'.wrapper input')
    # 규칙2: factor
    for tag,g in (('enter',f),('leave',lv['factor'])):
        check_factor(g,m,f'{label}.{tag}.factor');check_lrow(g,m,f'{label}.{tag}.factor')
    equal(strip(f),strip(lv['factor']),label+'.factor 불변')
    for k in ks:
        equal({'x1b0':[],'x1b8':[]},k['scratch'],f'{label}.kernel.{k["phase"]}.scratch')
    inp=en['input'];check_vec(inp,m,label+'.input');c=inp['count']
    require(c>=1,label+': 입력 count≥1 전제(지원 caller는 단위 벡터, C:1999–2023)')
    stats['input_count%d'%min(c,2)]+=1
    s=f['scalars'];r=inp['idx'][0];urow=f['U']['row_end'][r]-f['U']['row_start'][r]
    leave_cache=lv['factor']['cache']
    # 규칙3·4: cache 경로와 Uᵀ 커널
    if urow==0:
        stats['urow_empty']+=1
        expect_kernel=sparse_kernel(f,c,m)
        equal(expect_kernel,bool(ks),f'{label}.일반 경로 Uᵀ 커널 호출(기준={expect_kernel})')
        equal(f['x1d8'],lv['factor']['x1d8'],label+'.일반 경로 F1d8 불변')
        equal(0,leave_cache['count'],label+'.일반 경로 cache.count')
    else:
        stats['urow_nonempty']+=1
        require(len(ks)==2,label+': U 행 비영이면 커널 강제')
        equal(r,ks[0]['x1d8'],label+'.커널 진입 F1d8=r');equal(0,ks[0]['cache']['count'],label+'.커널 진입 cache.count')
        equal(r,lv['factor']['x1d8'],label+'.반환 F1d8')
    if ks:
        ke,kl=ks
        equal(dict(F=True,m=m,x20=True,**{'in':True,'out':True},x1b0=True,x1b8=True),ke['args'],label+'.kernel args')
        equal(inp,ke['input'],label+'.커널 입력=본체 입력')
        kout=kl['output'];check_vec(kout,m,label+'.Uᵀ out');k=kout['count']
        stats['kernel']+=1;stats['kout%s'%(k if k in (-1,0,1) else '2+')]+=1
        if urow:
            if k>0:equal(kout,leave_cache,label+'.cache=Uᵀ 출력');stats['cache_written']+=1
            else:equal(0,leave_cache['count'],label+'.cache.count')
    else:
        k=-1;stats['dense_UT']+=1
    # 규칙5: eta·Lᵀ 경로와 출력 표현
    out=lv['output'];check_vec(out,m,label+'.output')
    if ks and k==0:
        equal(0,out['count'],label+'.Uᵀ 0 조기 종료');stats['early0']+=1
    else:
        eta=s['x180']
        if eta==0:
            lt_sparse=k>=0 and s['x104']*k<=m and s['x140']<0.1*m
            stats['eta0']+=1;stats['LT_'+('sparse' if lt_sparse else 'dense')]+=1
            equal(lt_sparse,out['count']>=0,f'{label}.Lᵀ 표현(희소 기준={lt_sparse}, 실제 count={out["count"]})')
        elif k<0 or m<s['x104']*k:
            stats['eta_dense']+=1;equal(-1,out['count'],label+'.조밀 eta 뒤 조밀 Lᵀ')
        else:
            stats['eta_sparse_unjudged']+=1
    stats['out_'+('dense' if out['count']<0 else 'sparse')]+=1
    stats['level']+=s['x140']>=0
    if s['x140']>=0:stats['bucket_changed']+=f['Lrow']['bucket']!=lv['factor']['Lrow']['bucket']
    stats['perm_nonid']+=f['perm']['x20']!=list(range(m))
    # 규칙6: 출력과 wrapper raw, 별칭
    wr=wlv['raw'];equal({'count':wr['count'],'idx':wr['indices'],'val':wr['values']},out,label+'.wrapper raw')
    a=lv['alias'];stats['alias_'+'%s%s'%(a['out']['idx'],a['out']['val'])]+=1


STAT_KEYS=('input_count1','input_count2','urow_empty','urow_nonempty','kernel','dense_UT','kout-1','kout0','kout1','kout2+','cache_written',
    'early0','eta0','eta_dense','eta_sparse_unjudged','LT_sparse','LT_dense','out_sparse','out_dense','level','bucket_changed','perm_nonid',
    'alias_inin','alias_outout','alias_inout','alias_outin')


def acceptance(data):
    out={}
    for name in data:
        r=data[name][0];cs=calls(r);stats={k:0 for k in STAT_KEYS}
        equal(EXPECTED[name],len(cs),name+'.호출 수')
        for k,(w,body,lv) in enumerate(cs):check_call(w,body,lv,f'{name}#{k+1}',stats)
        require(len([e for e in r['events'] if is_new(e)])==sum(len(b) for _,b,_ in cs),name+'.사건 수')
        out[name]=dict(calls=len(cs),**stats)
    tot={k:sum(v[k] for v in out.values()) for k in ('calls',)+STAT_KEYS}
    require(tot['calls']==sum(EXPECTED[n] for n in data),'전체 호출 수')
    return dict(models=out,total=tot)


# ---------- 변조 ----------
def ev(d,model,event,k,run=0,phase=None):
    return [e for e in d[model][run]['events'] if e['event']==event and (phase is None or e.get('phase')==phase)][k]


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
    test('manifest-hash',lambda d,mm:mm['tools'].update({'tools/push_btran_agent.js':'0'*64}),lambda d,mm:check_files(mm))
    test('empty-events',lambda d,mm:[r.update(events=[]) for r in d['retry']],ctl)
    test('run-diff',lambda d,mm:ev(d,'capri','btran:leave',0,run=2)['output']['val'].__setitem__(0,'0000000000000080'),ctl)
    test('projection-diff',lambda d,mm:[next(e for e in r['events'] if e['event']=='roundctl:btran:leave')['raw'].update(count=9) for r in d['arms']],ctl)
    test('netlib-projection-diff',lambda d,mm:[next(e for e in r['events'] if e['event']=='supply:leave').update(candidate=-7) for r in d['adlittle']],ctl)
    test('baseline-result',lambda d,mm:[r['result'].update(status=3) for r in d['scsd1']],ctl)
    test('no-new-events',lambda d,mm:[r.update(events=project(r)) for r in d['arms']],ctl)
    test('model-hash',lambda d,mm:[r.update(model_sha256='0'*64) for r in d['capri']],ctl)
    # 사건 구조
    test('null-null',lambda d,mm:[e.update(output=None) for e in d['retry'][0]['events'] if e['event']=='btran:leave'],acc)
    test('missing-leave',lambda d,mm:d['retry'][0].update(events=[e for e in d['retry'][0]['events'] if e['event']!='btran:leave']),acc)
    test('missing-kernel',lambda d,mm:d['capri'][0].update(events=[e for e in d['capri'][0]['events'] if e['event']!='btran:kernel']),acc)
    test('extra-call',lambda d,mm:d['retry'][0]['events'].append(dict(ev(d,'retry','btran:enter',0))),acc)
    def mut(name,model,event,k,fn,phase=None):test(name,lambda d,mm:fn(ev(d,model,event,k,phase=phase)),acc)
    # 규칙1 인자·wrapper
    mut('R1-cache-arg','arms','btran:enter',0,lambda e:e['args'].update(cache_f1d0=False))
    mut('R1-wrapper-input','retry','roundctl:btran:enter',0,lambda e:e['input'].update(indices=[(e['input']['indices'][0]+1)%10]))
    # 규칙2 factor
    mut('R2-zero','arms','btran:enter',0,lambda e:e['factor'].update(zero=ZERO_BITS))
    mut('R2-perm','capri','btran:enter',0,lambda e:e['factor']['perm']['x30'].reverse())
    mut('R2-Lrow-value','adlittle','btran:enter',0,lambda e:e['factor']['Lrow']['vals'].__setitem__(0,flip(e['factor']['Lrow']['vals'][0])))
    mut('R2-Lrow-col','capri','btran:enter',1,lambda e:e['factor']['Lrow']['cols'].__setitem__(0,(e['factor']['Lrow']['cols'][0]+1)%e['factor']['m']))
    mut('R2-level','capri','btran:enter',0,lambda e:e['factor']['Lrow']['lev'].__setitem__(0,10**6))
    mut('R2-scratch','retry','btran:leave',0,lambda e:e['factor']['scratch']['x1b0'].append([0,1]))
    mut('R2-kernel-scratch','capri','btran:kernel',0,lambda e:e['scratch']['x1b8'].append([0,'000000000000f03f']),phase='leave')
    mut('R2-factor-changed','scsd1','btran:leave',0,lambda e:e['factor']['diag'].__setitem__(0,flip(e['factor']['diag'][0])))
    # 규칙3 cache
    def first_nonempty(d,model,event,phase=None):
        for k,e in enumerate([e for e in d[model][0]['events'] if e['event']=='btran:enter']):
            f=e['factor'];r=e['input']['idx'][0]
            if f['U']['row_end'][r]>f['U']['row_start'][r]:return k
        raise Fail('U 행 비영 사례 없음')
    def cache_val(d,mm):
        k=first_nonempty(d,'capri','btran:enter');c=ev(d,'capri','btran:leave',k)['factor']['cache'];c['val'][0]=flip(c['val'][0])
    test('R3-cache-value',cache_val,acc)
    def cache_order(d,mm):
        k=first_nonempty(d,'scsd1','btran:enter');c=ev(d,'scsd1','btran:leave',k)['factor']['cache'];c['idx'].reverse();c['val'].reverse()
    test('R3-cache-order',cache_order,acc)
    def cache_drop(d,mm):
        k=first_nonempty(d,'adlittle','btran:enter');c=ev(d,'adlittle','btran:leave',k)['factor']['cache'];c['idx'].pop();c['val'].pop();c['count']-=1
    test('R3-cache-drop',cache_drop,acc)
    def x1d8(d,mm):
        k=first_nonempty(d,'capri','btran:enter');ev(d,'capri','btran:leave',k)['factor']['x1d8']=-1
    test('R3-x1d8',x1d8,acc)
    def kx1d8(d,mm):
        k=first_nonempty(d,'capri','btran:enter');ev(d,'capri','btran:kernel',k,phase='enter')['x1d8']+=1
    test('R3-kernel-x1d8',kx1d8,acc)
    mut('R3-empty-row-cache','arms','btran:leave',0,lambda e:e['factor'].update(cache={'count':1,'idx':[0],'val':[ZERO_BITS]}))
    mut('R3-empty-row-x1d8','arms','btran:leave',0,lambda e:e['factor'].update(x1d8=3))
    def urow_fake(d,mm):
        # U 행 r을 비영으로 위조하면(경계만 협조 변조) 커널 진입 F1d8·반환 F1d8 규칙에 걸린다.
        for e in (ev(d,'arms','btran:enter',0),ev(d,'arms','btran:leave',0)):
            f=e['factor'];r=ev(d,'arms','btran:enter',0)['input']['idx'][0];row=f['U']['row'][r]
            f['U']['row_end'][r]+=1;row['cols'].append(0);row['vals'].append(ZERO_BITS)
    test('R3-urow-cooperative',urow_fake,acc)
    # 규칙4 Uᵀ 커널
    mut('R4-kernel-input','retry','btran:kernel',0,lambda e:e['input']['val'].__setitem__(0,flip(e['input']['val'][0])),phase='enter')
    mut('R4-kernel-args','arms','btran:kernel',0,lambda e:e['args'].update(x20=False),phase='enter')
    def general_pred(d,mm):
        # 일반 경로 판정 입력(F108)을 크게 바꾸면 희소 Uᵀ가 아니어야 한다.
        for e in (ev(d,'arms','btran:enter',0),ev(d,'arms','btran:leave',0)):e['factor']['scalars']['x108']='000000000000e07f'
    test('R4-general-predicate',general_pred,acc)
    # 규칙5 Lᵀ 표현
    def lt_pred(d,mm):
        for e in (ev(d,'retry','btran:enter',0),ev(d,'retry','btran:leave',0)):e['factor']['scalars']['x104']=1000
    test('R5-x104',lt_pred,acc)
    def lt_level(d,mm):
        e0=ev(d,'capri','btran:enter',1);n=e0['factor']['m']
        for e in (e0,ev(d,'capri','btran:leave',1)):
            e['factor']['scalars']['x140']=n;e['factor']['Lrow']['levels']=n
    test('R5-x140',lt_level,acc)
    def eta_dense(d,mm):
        e0=ev(d,'arms','btran:enter',0)
        for e in (e0,ev(d,'arms','btran:leave',0)):
            f=e['factor'];f['scalars'].update(x180=1,x184=1,x104=100);f['etas']=[dict(r=0,len=1,idx_off=0,val_off=0,idx_mod16=0,val_mod16=0,idx=[0],val=[ZERO_BITS])]
    test('R5-eta-dense',eta_dense,acc)
    # 규칙6 출력
    mut('R6-output-bit','arms','btran:leave',0,lambda e:e['output']['val'].__setitem__(0,flip(e['output']['val'][0])))
    mut('R6-output-dup','capri','btran:leave',0,lambda e:e['output']['idx'].__setitem__(1,e['output']['idx'][0]))
    mut('R6-wrapper-raw','scsd1','roundctl:btran:leave',0,lambda e:e['raw']['values'].__setitem__(0,flip(e['raw']['values'][0])))
    def dense_out(d,mm):
        ks=[k for k,e in enumerate([e for e in d['adlittle'][0]['events'] if e['event']=='btran:leave']) if e['output']['count']<0]
        for e in (ev(d,'adlittle','btran:leave',ks[0]),ev(d,'adlittle','roundctl:btran:leave',ks[0])):
            o=e.get('output') or e['raw']
            if 'idx' in o:o.update(count=1,idx=[0],val=[o['val'][0]])
            else:o.update(count=1,indices=[0],values=[o['values'][0]])
    test('R5-dense-as-sparse',dense_out,acc)
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
