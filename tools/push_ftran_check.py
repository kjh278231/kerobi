"""M3-04c-21a: 한 RHS FTRAN 본체 오라클의 출처/독립3회/선행19 비간섭, 원본 C 규칙 판정, 변조 거부."""
import argparse
import copy
import hashlib
import json
import math
from pathlib import Path
import struct
import sys

from push_supplier_check import Fail, equal, require

ROOT=Path(__file__).resolve().parents[1]
D=Path('bench/oracle/m3_push_ftran')
BASE=Path('bench/oracle/m3_push_round')
BINARY_SHA='8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c'
SCHEMA='M3-04c-21a-v1'
ZERO_LE='82764968c2253c3d'  # 1e-13, little-endian bytes as the agent writes them
NAMES=('arms','retry','interior','control')
CASES={
 'arms':('bench/oracle/m3_push_arms/arms.lp','bench/oracle/m3_push_arms/arms.extra.json'),
 'retry':('bench/oracle/m3_push_retry/retry.lp','bench/oracle/m3_push_retry/retry.extra.json'),
 'interior':('bench/oracle/m3_primal_push_step/interior.lp',None),
 'control':('bench/oracle/m3_push_arms/arms.lp','bench/oracle/m3_push_arms/arms.extra.json')}
# 선행 저장 자료(M3-04c-21 survey §4.1)에서 고정한 호출 수: (공급 FTRAN, 재계산 FTRAN)
EXPECTED={'arms':(25,3),'retry':(8,2),'interior':(9,3),'control':(0,0)}
TOOLS=('tools/push_ftran_agent.js','tools/push_ftran_capture.js','tools/push_ftran_runs.py','tools/push_ftran_check.py',
       'tools/push_supplier_check.py','tools/frida_resolve.cjs','tools/gurun.sh')
SOURCES=('../re/decomp/c/00120740_PRIVATE00000000002207de.c','../re/decomp/c/001205d0_PRIVATE000000000009cf9b.c',
         '../re/decomp/c/00120680_PRIVATE000000000009d008.c','../re/decomp/c/0012a990_PRIVATE000000000021b9de.c',
         '../re/decomp/c/0012bd30_PRIVATE0000000000214344.c','../re/decomp/c/001223e0_PRIVATE000000000021686f.c',
         '../re/decomp/c/00121ef0_PRIVATE00000000002126cd.c')
ZERO_BITS='0000000000000000'


def sha(p):return hashlib.sha256((ROOT/p).read_bytes()).hexdigest()
def load(p):return json.loads((ROOT/p).read_text())
def records(d=D):return {n:[load(d/f'{n}-{i}.json') for i in (1,2,3)] for n in NAMES}


def file_sets():
    return dict(tools=set(TOOLS),inputs={p for model,extra in CASES.values() for p in (model,extra) if p},
        sources=set(SOURCES),assembly={str(D/'source.asm')},
        captures={str(D/f'{n}-{i}.json') for n in NAMES for i in (1,2,3)},
        baselines={str(BASE/f'{n}-1.json') for n in NAMES}|{str(BASE/'manifest.json')},
        history={str(p.relative_to(ROOT)) for p in (ROOT/D/'history').rglob('*') if p.is_file()})


def manifest():
    return dict(task='M3-04c-21a',binary_sha256=BINARY_SHA,**{k:{p:sha(p) for p in sorted(v)} for k,v in file_sets().items()})


def check_files(m):
    equal('M3-04c-21a',m['task'],'manifest.task');equal(BINARY_SHA,m['binary_sha256'],'manifest.binary')
    for k,paths in file_sets().items():
        equal(paths,set(m[k]),'manifest.'+k+'.keys')
        for p in paths:
            if k=='sources' and not (ROOT/p).exists():continue
            equal(m[k][p],sha(p),'manifest.'+p)


def is_new(e):return e['event'].startswith('ftran:')
def project(r):return [e for e in r['events'] if not is_new(e)]


def controls(data,m):
    out={}
    for name in NAMES:
        rs=data[name];equal(3,len(rs),name+'.runs')
        equal(3,len({r['pid'] for r in rs}),name+'.pid');equal(3,len({r['run_id'] for r in rs}),name+'.run_id')
        model,extra=CASES[name];ex=load(extra) if extra else {};base=load(BASE/f'{name}-1.json')
        for i,r in enumerate(rs):
            label=f'{name}/{i+1}'
            equal(BINARY_SHA,r['binary_sha256'],label+'.binary');equal(SCHEMA,r['schema'],label+'.schema')
            equal('natural',r['origin'],label+'.origin');equal('control' if name=='control' else 'primal',r['kind'],label+'.kind')
            equal(m['tools']['tools/push_ftran_capture.js'],r['tool_sha256'],label+'.tool')
            equal(m['tools']['tools/push_ftran_agent.js'],r['agent_sha256'],label+'.agent')
            equal(sha(model),r['model_sha256'],label+'.model')
            params=dict(Threads=1,Seed=0,OutputFlag=0,Presolve=0,Method=2,Crossover=0 if name=='control' else 3,LPWarmStart=2)
            params.update(ex.get('params',{}));equal(params,r['params'],label+'.params');require(ex.get('starts')==r['starts'],label+'.starts')
            equal([],r['errors'],label+'.errors');equal('process-terminated',r['reason'],label+'.reason')
            equal(rs[0]['events'],r['events'],label+'.events');equal(rs[0]['environment'],r['environment'],label+'.environment')
            equal(rs[0]['result'],r['result'],label+'.result')
            equal(base['events'],project(r),label+'.projection');equal(base['result'],r['result'],label+'.baseline_result')
            equal(base['output'],r['output'],label+'.baseline_output')
        new=[e for e in rs[0]['events'] if is_new(e)]
        if name=='control':equal(0,len(rs[0]['events']),'control.events')
        else:require(new,name+': 새 FTRAN 사건0')
        out[name]=dict(events=len(rs[0]['events']),ftran=len(new))
    return out


# ---------- 원본 C 규칙 ----------
def f64(h):
    require(isinstance(h,str) and len(h)==16,f'f64 LE hex: {h!r}')
    v=struct.unpack('<d',bytes.fromhex(h))[0]
    require(math.isfinite(v),f'유한값 전제: {h}')
    return v


def check_vec(v,m,label):
    require(isinstance(v,dict) and set(v)=={'count','idx','val'},label+': 벡터 필드')
    c=v['count']
    if c<0:
        equal(-1,c,label+'.dense count');require(v['idx'] is None,label+'.dense idx');equal(m,len(v['val']),label+'.dense len')
    else:
        require(isinstance(v['idx'],list) and len(v['idx'])==c and len(v['val'])==c,label+'.sparse len')
        require(all(0<=i<m for i in v['idx']),label+'.idx 범위');require(len(set(v['idx']))==c,label+'.idx 중복')
    for h in v['val']:f64(h)


def inverse(a,b,label):
    m=len(a);require(sorted(a)==list(range(m)),label+': 순열 아님')
    equal(list(range(m)),[b[a[i]] for i in range(m)],label+': 역순열')


def check_factor(f,m,label):
    equal(m,f['m'],label+'.m');equal(ZERO_LE,f['zero'],label+'.zero')
    p=f['perm']
    inverse(p['x20'],p['x30'],label+'.F20/F30');inverse(p['x28'],p['x38'],label+'.F28/F38');inverse(p['x40'],p['x48'],label+'.F40/F48')
    beg=f['L']['beg'];equal(m+1,len(beg),label+'.L.beg len')
    require(all(beg[i]<=beg[i+1] for i in range(m)),label+'.L.beg 단조')
    equal(beg[m]-beg[0],len(f['L']['rows']),label+'.L.rows len');equal(len(f['L']['rows']),len(f['L']['vals']),label+'.L.vals len')
    require(all(0<=r<m for r in f['L']['rows']),label+'.L.rows 범위')
    U=f['U']
    for j in range(m):
        require(U['col_start'][j]<=U['col_bound'][j]<=U['col_end'][j],f'{label}.U.col[{j}] 경계')
        equal(U['col_end'][j]-U['col_start'][j],len(U['col'][j]['rows']),f'{label}.U.col[{j}] len')
        require(U['row_start'][j]<=U['row_end'][j],f'{label}.U.row[{j}] 경계')
        equal(U['row_end'][j]-U['row_start'][j],len(U['row'][j]['cols']),f'{label}.U.row[{j}] len')
    for h in f['diag']+f['L']['vals']:f64(h)
    s=f['scalars'];etas=f['etas']
    equal(s['x180'],len(etas),label+'.eta 수');equal(s['x184'],sum(e['len'] for e in etas),label+'.eta 길이합')
    for k,e in enumerate(etas):
        require(0<=e['r']<m and e['len']==len(e['idx'])==len(e['val']),f'{label}.eta[{k}]')
        require(all(0<=i<m for i in e['idx']),f'{label}.eta[{k}].idx')
    equal(s['xec'],len(f['prefix']['list']),label+'.prefix 수')
    equal({'x1b0':[],'x1b8':[]},f['scratch'],label+'.scratch')


def spike_from(v,m):
    """C:703–778, save=1: dense면 F10<|v|인 위치 오름차순, 아니면 목록 그대로 복사(값은 v*1.0)."""
    if v['count']<1:
        idx=[i for i in range(m) if 1e-13<abs(f64(v['val'][i]))]
        return {'count':len(idx),'indices':idx,'values':[v['val'][i] for i in idx]}
    return {'count':v['count'],'indices':list(v['idx']),'values':list(v['val'])}


def l_kernel(f,count,m):
    s=f['scalars']
    return '21b9de' if count>0 and count*s['x104']<=m and s['x128']<0.1*m else '214344'


def u_kernel(f,count,m):
    s=f['scalars'];dd=f64(s['x108'])+f64(s['x110'])
    if count>=0 and ((m-s['x100'])+s['xec']+count<m or dd*dd*count<m+m):return '21686f'
    return '2126cd'


def strip_spike(f):y=copy.deepcopy(f);y.pop('spike');return y


def calls(r):
    """선행 wrapper 사건 안에 본체 사건열을 묶는다."""
    ev=r['events'];out=[];i=0
    while i<len(ev):
        e=ev[i]
        if e['event'] in ('supplier:ftran:enter','recompute:ftran:enter'):
            leave=e['event'].replace('enter','leave');j=i+1;body=[]
            while ev[j]['event']!=leave:
                require(is_new(ev[j]),f'wrapper 안 다른 사건 {ev[j]["event"]}');body.append(ev[j]);j+=1
            out.append((e,body,ev[j]));i=j+1
        else:
            require(not is_new(e),f'wrapper 밖 FTRAN 사건 {e["event"]}');i+=1
    return out


def check_call(wenter,body,wleave,label,stats):
    require(len(body)>=2 and body[0]['event']=='ftran:enter' and body[-1]['event']=='ftran:leave',label+': 본체 enter/leave 1회')
    en,lv=body[0],body[-1];ks=body[1:-1]
    require(all(k['event']=='ftran:kernel' and k['id']==en['id'] for k in ks) and lv['id']==en['id'],label+': 사건 id')
    site='supplier' if wenter['event'].startswith('supplier') else 'recompute'
    equal(site,en['site'],label+'.site');equal(site,lv['site'],label+'.leave.site')
    f=en['factor'];m=f['m'];check_factor(f,m,label+'.enter.factor');check_factor(lv['factor'],m,label+'.leave.factor')
    a=en['args'];equal(dict(k5=0,p6_null=True,vals_null=True,mark_null=True,p10_null=True,work_null=False),
        {k:a[k] for k in ('k5','p6_null','vals_null','mark_null','p10_null','work_null')},label+'.args')
    if site=='supplier':
        require(all(en['ctx'][k] for k in ('factor_s470','input_s220','output_s210')),label+'.ctx alias')
        equal(wenter['supplier_id'],en['ctx']['supplier_id'],label+'.supplier_id')
        save=0 if wenter['spike'] is None else 1
        equal(save,a['save'],label+'.save');equal(save==0,a['spike_null'],label+'.spike_null');equal(save==1,a['spike_f1c8'],label+'.spike_f1c8')
    else:
        require(all(en['ctx'][k] for k in ('factor_frame','input_frame','output_frame')),label+'.ctx alias')
        save=0;equal(0,a['save'],label+'.save');require(a['spike_null'],label+'.spike_null')
    stats['save'+str(save)]+=1
    # 규칙5 앞부분: wrapper clear
    if save:
        equal(0,f['spike']['count'],label+'.clear count');require(en['clear'] is not None,label+'.clear 누락')
        equal(wenter['spike'],en['clear']['before'],label+'.clear before')
        equal([ZERO_BITS]*len(en['clear']['before']['indices']),en['clear']['values_now'],label+'.clear values')
        stats['clear_nonempty']+=en['clear']['before']['count']>0
    else:require(en['clear'] is None,label+'.clear 비대상')
    # 규칙2: factor는 spike 외 불변
    equal(strip_spike(f),strip_spike(lv['factor']),label+'.factor 불변')
    inp=en['input'];check_vec(inp,m,label+'.input');c=inp['count']
    stats['input_'+('dense' if c<0 else 'zero' if c==0 else 'sparse')]+=1
    stats['eta_pos']+=f['scalars']['x180']>0
    stats['perm_nonid']+=f['perm']['x20']!=list(range(m))
    names=[k['kernel'] for k in ks if k['phase']=='enter']
    equal([k['phase'] for k in ks],['enter','leave']*len(names),label+'.kernel phases')
    equal(names,[k['kernel'] for k in ks if k['phase']=='leave'],label+'.kernel pair')
    if c==0:
        equal([],names,label+'.count0 kernels');equal(0,lv['output']['count'],label+'.count0 output')
        equal(f['spike'],lv['factor']['spike'],label+'.count0 spike');stats['count0']+=1
        return
    lk=l_kernel(f,c,m);require(names and names[0]==lk,f'{label}.L 경로 기준={lk}, 실제={names[:1]}')
    stats['L_'+lk]+=1
    k0=ks[0]
    # 규칙3: 입력 변환
    if c>0:
        equal({'idx':'in','val':'in'},k0['alias']['in'],label+'.sparse alias')
        equal({'count':c,'idx':[f['perm']['x30'][i] for i in inp['idx']],'val':inp['val']},k0['input'],label+'.sparse 행 치환')
    else:
        equal({'idx':'out','val':'out'},k0['alias']['in'],label+'.dense 교환')
        equal({'count':-1,'idx':None,'val':[inp['val'][f['perm']['x20'][i]] for i in range(m)]},k0['input'],label+'.dense gather')
    lout=ks[1]['output'];check_vec(lout,m,label+'.L out')
    if lout['count']==0:
        equal([lk],names,label+'.L0 종료');stats['L_out0']+=1;return
    equal(2,len(names),label+'.kernel 수')
    uin=ks[2]['input'];check_vec(uin,m,label+'.U in')
    uk=u_kernel(f,uin['count'],m);equal(uk,names[1],label+'.U 경로');stats['U_'+uk]+=1
    # 규칙5: spike는 U 진입 전에 저장
    if save:
        sp=spike_from(uin,m);equal(sp,ks[2]['spike'],label+'.spike 저장')
        equal(sp,lv['factor']['spike'],label+'.spike 반환');stats['spike_'+('dense' if uin['count']<1 else 'sparse')]+=1
    else:
        equal(f['spike'],ks[2]['spike'],label+'.save0 spike');equal(f['spike'],lv['factor']['spike'],label+'.save0 반환 spike')
    # 규칙6: 출력
    uout=ks[3]['output'];check_vec(uout,m,label+'.U out')
    equal(uout,lv['output'],label+'.본체 출력=U 출력')
    if site=='supplier':
        w=wleave['raw'];equal({'count':w['count'],'idx':w['indices'],'val':w['values']},lv['output'],label+'.wrapper raw')
        equal(lv['factor']['spike'] if save else None,wleave['spike'],label+'.wrapper spike')
    else:
        require(lv['output']['count']<0,label+'.재계산 dense 출력');equal(wleave['raw'],lv['output']['val'],label+'.wrapper raw')
        require(lv['input_after']['count']<0,label+'.입력 후상태 dense');equal(wleave['input_after'],lv['input_after']['val'],label+'.입력 후상태')
    stats['out_'+('dense' if lv['output']['count']<0 else 'sparse')]+=1


def acceptance(data):
    out={}
    for name in NAMES:
        r=data[name][0];cs=calls(r)
        stats={k:0 for k in ('save0','save1','clear_nonempty','input_dense','input_sparse','input_zero','count0','L_21b9de','L_214344',
            'L_out0','U_21686f','U_2126cd','spike_dense','spike_sparse','out_dense','out_sparse','eta_pos','perm_nonid')}
        sup=sum(1 for w,_,_ in cs if w['event'].startswith('supplier'))
        equal(EXPECTED[name],(sup,len(cs)-sup),name+'.호출 수')
        for k,(w,body,lv) in enumerate(cs):check_call(w,body,lv,f'{name}#{k+1}',stats)
        require(len([e for e in r['events'] if is_new(e)])==sum(len(b) for _,b,_ in cs),name+'.사건 수')
        out[name]=dict(calls=len(cs),**stats)
    tot={k:sum(v[k] for v in out.values()) for k in out['arms']}
    require(tot['calls']==50,'전체 FTRAN 50회')
    return dict(models=out,total=tot)


# ---------- 변조 ----------
def self_test(data,m):
    rejected=[]
    def test(name,mutate,check,all_runs=True):
        d=copy.deepcopy(data);mm=copy.deepcopy(m);mutate(d,mm)
        try:check(d,mm)
        except (Fail,KeyError,TypeError,IndexError,ValueError,StopIteration) as exc:rejected.append(dict(name=name,error=str(exc)[:200]));return
        raise Fail('NOT REJECTED: '+name)
    ctl=lambda d,mm:controls(d,mm);acc=lambda d,mm:acceptance(d)
    test('binary-hash',lambda d,mm:d['arms'][1].update(binary_sha256='0'*64),ctl)
    test('manifest-hash',lambda d,mm:mm['tools'].update({'tools/push_ftran_agent.js':'0'*64}),lambda d,mm:check_files(mm))
    test('empty-events',lambda d,mm:[r.update(events=[]) for r in d['retry']],ctl)
    test('run-diff',lambda d,mm:ev(d,'interior','ftran:leave',0,run=1)['output']['val'].__setitem__(0,'0000000000000080'),ctl)
    test('projection-diff',lambda d,mm:[next(e for e in r['events'] if e['event']=='supplier:ftran:leave')['raw'].update(count=9) for r in d['arms']],ctl)
    test('baseline-result',lambda d,mm:[r['result'].update(status=3) for r in d['retry']],ctl)
    test('no-new-events',lambda d,mm:[r.update(events=project(r)) for r in d['arms']],ctl)
    test('null-null',lambda d,mm:[e.update(output=None) for e in d['retry'][0]['events'] if e['event']=='ftran:leave'],acc)
    test('missing-leave',lambda d,mm:d['retry'][0].update(events=[e for e in d['retry'][0]['events'] if e['event']!='ftran:leave']),acc)
    test('extra-call',lambda d,mm:d['retry'][0]['events'].append(dict(ev(d,'retry','ftran:enter',0))),acc)
    def mut(name,model,event,k,fn):test(name,lambda d,mm:fn(ev(d,model,event,k)),acc)
    # 규칙2 factor
    mut('R2-zero','arms','ftran:enter',0,lambda e:e['factor'].update(zero='0000000000000000'))
    mut('R2-perm','arms','ftran:enter',0,lambda e:e['factor']['perm']['x30'].reverse())
    mut('R2-eta-len','interior','ftran:enter',4,lambda e:e['factor']['etas'][0].update(len=e['factor']['etas'][0]['len']+1))
    mut('R2-scratch','retry','ftran:leave',0,lambda e:e['factor']['scratch']['x1b0'].append([0,1]))
    mut('R2-factor-changed','interior','ftran:leave',2,lambda e:e['factor']['diag'].__setitem__(0,'000000000000f03f'))
    mut('R2-Ubound','arms','ftran:enter',1,lambda e:e['factor']['U']['col_bound'].__setitem__(0,-5))
    # 규칙3 입력 변환
    mut('R3-sparse-perm','arms','ftran:kernel',0,lambda e:e['input'].update(idx=[(i+1)%25 for i in e['input']['idx']]))
    mut('R3-dense-gather','interior','ftran:kernel',4,lambda e:e['input']['val'].reverse())
    mut('R3-swap-alias','interior','ftran:kernel',4,lambda e:e['alias']['in'].update(val='in'))
    # 규칙4 경로
    def lkern(d,mm):
        for e in [ev(d,'arms','ftran:kernel',0),ev(d,'arms','ftran:kernel',1)]:e['kernel']='214344'
    test('R4-L',lkern,acc)
    def ukern(d,mm):
        for e in d['retry'][0]['events']:
            if e['event']=='ftran:kernel' and e['kernel']=='21686f':e['kernel']='2126cd'
    test('R4-U',ukern,acc)
    def x104(d,mm):
        for e in [ev(d,'arms','ftran:enter',0),ev(d,'arms','ftran:leave',0)]:e['factor']['scalars']['x104']=1000
    test('R4-x104',x104,acc)
    def ucond(d,mm):
        for e in [ev(d,'retry','ftran:enter',0),ev(d,'retry','ftran:leave',0)]:e['factor']['scalars'].update(x100=0,x108='000000000000e07f')
    test('R4-U-condition',ucond,acc)
    # 규칙5 spike
    def spike_tamper(d,mm,kind):
        sav=[i for i,e in enumerate(d['interior'][0]['events']) if e['event']=='ftran:leave' and e['factor']['spike'] and e['factor']['spike']['count']>1]
        e=d['interior'][0]['events'][sav[0]]['factor']['spike']
        if kind=='order':e['indices'].reverse();e['values'].reverse()
        elif kind=='value':e['values'][0]='000000000000f03f'
        else:e['indices'].pop();e['values'].pop();e['count']-=1
    for kind in ('order','value','drop'):test('R5-spike-'+kind,lambda d,mm,k=kind:spike_tamper(d,mm,k),acc)
    mut('R5-clear','interior','ftran:enter',5,lambda e:e['clear']['values_now'].__setitem__(0,'000000000000f03f'))
    mut('R5-save0-spike','arms','ftran:leave',0,lambda e:e['factor'].update(spike={'count':1,'indices':[0],'values':[ZERO_BITS]}))
    mut('R5-save-flag','retry','ftran:enter',9,lambda e:e['args'].update(save=0))
    # 규칙6 출력
    mut('R6-output-bit','arms','ftran:leave',0,lambda e:e['output']['val'].__setitem__(0,flip(e['output']['val'][0])))
    mut('R6-output-order','retry','ftran:leave',3,lambda e:(e['output']['idx'].reverse(),e['output']['val'].reverse()))
    mut('R6-recompute-input','arms','ftran:leave',18,lambda e:e['input_after']['val'].__setitem__(0,flip(e['input_after']['val'][0])))
    mut('R6-alias','arms','ftran:enter',0,lambda e:e['ctx'].update(output_s210=False))
    def kspike(d,mm):
        e=[e for e in d['interior'][0]['events'] if e['event']=='ftran:kernel' and e['kernel']=='2126cd' and e['phase']=='enter' and e['spike']['count']>1][0]
        e['spike']['indices'].reverse();e['spike']['values'].reverse()
    test('R5-spike-at-U-enter',kspike,acc)
    def wraw(d,mm):
        lv=ev(d,'arms','ftran:leave',0);k=ev(d,'arms','ftran:kernel',3)
        for v in (lv['output'],k['output']):v['val'][0]=flip(v['val'][0])
    test('R6-wrapper-raw',wraw,acc)
    return dict(rejected=len(rejected),details=rejected)


def flip(h):b=bytearray.fromhex(h);b[0]^=1;return b.hex()


def ev(d,model,event,k,run=0):return [e for e in d[model][run]['events'] if e['event']==event][k]


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
