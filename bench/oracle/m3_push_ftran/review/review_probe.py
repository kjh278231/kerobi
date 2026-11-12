"""독립 검토: 자료 분포 확인과 추가 변조(검사기 약점 탐색). 추적 파일은 바꾸지 않는다."""
import copy, struct, sys, json
from pathlib import Path
ROOT=Path(__file__).resolve().parents[4]
sys.path.insert(0,str(ROOT/'tools'))
import push_ftran_check as C
data=C.records()

def f(h):return struct.unpack('<d',bytes.fromhex(h))[0]
# 1) 분포
info=dict(levels=set(),near_zero_dense_spike=0,filtered_exact_zero=0,filtered_nonzero=0,eta0_U_eq_L=0,eta0_calls=0,sparse_U_cond_false=0,
          lev_ge0_21b9de=0,x104=set())
for name in ('arms','retry','interior'):
    for w,body,lv in C.calls(data[name][0]):
        en=body[0];ks=body[1:-1];fa=en['factor'];m=fa['m'];s=fa['scalars']
        info['levels'].add(s['x128']);info['x104'].add(s['x104'])
        if ks and ks[0]['kernel']=='21b9de' and s['x128']>=0:info['lev_ge0_21b9de']+=1
        if len(ks)==4 and s['x180']==0:
            info['eta0_calls']+=1
            info['eta0_U_eq_L']+=ks[2]['input']==ks[1]['output']
        if len(ks)==4:
            u=ks[2]['input']
            if u['count']<1 and en['args']['save']:
                for v in u['val']:
                    a=abs(f(v))
                    if a<=1e-13:
                        if a==0:info['filtered_exact_zero']+=1
                        else:info['filtered_nonzero']+=1
            if u['count']>=0:
                dd=f(s['x108'])+f(s['x110'])
                if not ((m-s['x100'])+s['xec']+u['count']<m or dd*dd*u['count']<2*m):info['sparse_U_cond_false']+=1
info={k:sorted(v) if isinstance(v,set) else v for k,v in info.items()}
print(json.dumps(info))

# 2) 추가 변조
m=C.load(C.D/'manifest.json')
def run(name,mut):
    d=copy.deepcopy(data);mut(d)
    try:C.acceptance(d)
    except Exception as e:return name,'거부',str(e)[:120]
    return name,'통과(미거부)',''
def ev(d,model,event,k):return C.ev(d,model,event,k)
def kern(d,model,pred):return [e for e in d[model][0]['events'] if e['event']=='ftran:kernel' and pred(e)]
res=[]
# T1 eta0 호출의 U 진입 입력 값(=L 출력 swap 결과) 비트 변조, save0 sparse(arms)
def t1(d):
    e=kern(d,'arms',lambda e:e['kernel']=='21686f' and e['phase']=='enter')[0];e['input']['val'][0]=C.flip(e['input']['val'][0])
res.append(run('T1 eta0 U진입 입력 비트(arms, save0)',t1))
# T2 L 커널 반환 out 값 변조
def t2(d):
    e=kern(d,'arms',lambda e:e['kernel']=='21b9de' and e['phase']=='leave')[0];e['output']['val'][0]=C.flip(e['output']['val'][0])
res.append(run('T2 L 반환 출력 비트(arms)',t2))
# T3 dense gather 뒤 out 구조체(원 입력 val 보유) 변조
def t3(d):
    e=kern(d,'interior',lambda e:e['kernel']=='214344' and e['phase']=='enter')[0]
    e['alias']['out']={'idx':'out','val':'out'}
res.append(run('T3 dense 교환 뒤 out 별칭(interior)',t3))
# T4 본체 반환 별칭 변조
def t4(d):
    e=ev(d,'arms','ftran:leave',0);e['alias']['out']={'idx':'other','val':'other'}
res.append(run('T4 본체 반환 별칭(arms)',t4))
# T5 공급 호출 입력 후상태 변조
def t5(d):
    e=ev(d,'arms','ftran:leave',0);e['input_after']['val'][0]=C.flip(e['input_after']['val'][0])
res.append(run('T5 공급 입력 후상태 비트(arms)',t5))
# T6/T7 dense spike 문턱 엄격성: U 입력의 첫 spike 위치 값을 x로 바꾸고 그 위치를 spike에서 제거한다.
def thr(x):
    def mut(d):
        evs=d['interior'][0]['events']
        for k,e in enumerate(evs):
            if e['event']=='ftran:kernel' and e['kernel']=='2126cd' and e['phase']=='enter' and e['spike']['count']>0:
                i0=e['spike']['indices'][0];e['input']['val'][i0]=x
                lv=next(z for z in evs[k:] if z['event']=='ftran:leave');wl=next(z for z in evs[k:] if z['event']=='supplier:ftran:leave')
                for sp in (e['spike'],lv['factor']['spike'],wl['spike']):
                    j=sp['indices'].index(i0);sp['indices'].pop(j);sp['values'].pop(j);sp['count']-=1
                return
    return mut
b=bytearray.fromhex(C.ZERO_LE);b[0]+=1
res.append(run('T6 |v|=ZERO 정확값을 spike에서 제외(통과가 정답)',thr(C.ZERO_LE)))
res.append(run('T7 |v|=ZERO+ulp를 spike에서 제외(거부가 정답)',thr(b.hex())))
# T8 recompute 커널 반환 input(U 반환 시 in) 변조
def t8(d):
    e=kern(d,'arms',lambda e:e['kernel']=='2126cd' and e['phase']=='leave')[0];e['input']['val'][0]=C.flip(e['input']['val'][0])
res.append(run('T8 U 반환 시 in 벡터 비트(arms 재계산)',t8))
for r in res:print(' | '.join(r))
