"""M3-04b-1 자연 캡처의 필수 상태·사건·재현성과 오류 검출 검사."""
import argparse
import copy
import hashlib
import json
import math
from pathlib import Path
import struct
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[1]
BASE = ROOT / 'bench/oracle/m3_primal_push_step'
BINARY_SHA = '8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c'


def sha(p):
    return hashlib.sha256(p.read_bytes()).hexdigest()


def digest(value):
    return hashlib.sha256(json.dumps(value, sort_keys=True, separators=(',', ':')).encode()).hexdigest()


def number(bits):
    assert isinstance(bits, str) and len(bits) == 16
    value = struct.unpack('<d', bytes.fromhex(bits))[0]
    assert math.isfinite(value), 'NaN 또는 IEEE 무한대'
    return value


def array(values, count, kind):
    assert isinstance(values, list) and len(values) == count, '배열 누락·길이'
    for value in values:
        if kind == 'int':
            assert type(value) is int
        elif kind == 'f64':
            number(value)
        else:
            assert isinstance(value, str) and len(bytes.fromhex(value)) == kind


def state(s, detail):
    keys = {'m', 'n', 'status', 'precision', 'vstat', 'header', 'x', 'slack', 'factor', 'counters', 'tolerances'}
    if detail:
        keys |= {'basic', 'width', 'marks', 'rhs', 'pinf'}
    assert s.keys() == keys, '필수 상태 누락'
    m, n = s['m'], s['n']
    assert type(m) is int and type(n) is int and 0 < m <= 100000 and 0 < n <= 100000
    assert s['precision'] == 0 and s['status'] == 1
    for k, count, kind in [('vstat',m+n,'int'),('header',m,'int'),('x',n,'f64'),('slack',m,'f64')]:
        array(s[k], count, kind)
    assert len(set(s['header'])) == m
    for row, col in enumerate(s['header']):
        assert 0 <= col < n+m and s['vstat'][col] == row, 'basis header·vstat 불일치'
    assert s['factor'].keys() == {'updates','limit','nnzL','nnzU'}
    assert s['counters'].keys() == {'numeric','failure','request','iterations'}
    number(s['counters']['iterations'])
    assert s['tolerances'].keys() == {'370','380','390','398','3b0'}
    for value in s['tolerances'].values():
        number(value)
    if detail:
        for k, kind in [('basic','f64'),('width','f64'),('marks',1),('rhs','f64')]:
            array(s[k], m, kind)
        number(s['pinf'])


def sparse(s, m):
    assert s.keys() == {'count','indices','values'}
    n = m if s['count'] < 0 else s['count']
    assert 0 <= n <= m
    array(s['values'],n,'f64')
    if s['count'] < 0:
        assert s['indices'] is None
    else:
        array(s['indices'],n,'int')
        assert all(0 <= i < m for i in s['indices'])


def validate(r, manifest, control=False):
    assert r['schema'] == 'M3-04b-1-v1' and r['origin'] == 'natural'
    assert r['binary_sha256'] == BINARY_SHA
    for field, path in [('tool_sha256','tools/primal_push_step_capture.js'),('agent_sha256','tools/primal_push_step_agent.js')]:
        assert r[field] == sha(ROOT/path) == manifest['tools'][path], field
    case = manifest['cases'][r['kind']]
    assert r['model_sha256'] == case['model_sha256'] == sha(ROOT/case['model'])
    expected_params = dict(Threads=1,Seed=0,OutputFlag=0,Presolve=0,Method=2,Crossover={'primal':3,'dual':1,'control':0}[r['kind']],LPWarmStart=2)
    assert r['params'] == expected_params and r['starts'] is None
    assert r['errors'] == [] and r['reason'] == 'process-terminated' and r['result']['status'] == 2
    assert type(r['pid']) is int and r['pid'] > 0 and len(r['run_id']) == 36
    events=r['events']
    if control:
        assert r['kind'] == 'control' and events == []
        assert r['result']['vbasis'] is None and r['result']['cbasis'] is None
        return {'pivot':0,'flip':0,'push':0}
    assert r['kind'] in ('primal','dual') and events, '목표 0회'
    counts=dict(pivot=0,flip=0,push=0)
    active=False; pending=None; applied=None; supply=False; ratio=None; factor=False; step=0; candidate=None; chosen=None
    for e in events:
        name=e['event']
        if name == 'push:enter':
            assert not active and not pending and not supply and not ratio and not factor
            active=True;step=0;counts['push']+=1
            assert e['call'] == counts['push'] and e['first'] == (r['kind']=='primal') and e['alias_contiguous'] is True
            assert type(e['attempt']) is int and type(e['model_status']) is int
            state(e['state'],False)
            m,n=e['state']['m'],e['state']['n'];model=e['model']
            assert model.keys() == {'starts','lengths','rows','values','cost','rhs','lb','ub'}
            array(model['starts'],n,8);array(model['lengths'],n,'int')
            nnz=max(int.from_bytes(bytes.fromhex(x),'little')+length for x,length in zip(model['starts'],model['lengths'],strict=True))
            array(model['rows'],nnz,'int');array(model['values'],nnz,'f64')
            for k,size in [('cost',n+m),('rhs',m),('lb',n+m),('ub',n+m)]:array(model[k],size,'f64')
        else:
            assert active and e['call'] == counts['push'], '사건의 push 연결'
            if 'state' in e:state(e['state'],True)
            if name == 'supply:enter':
                assert not pending and not supply and not ratio and not factor
                supply=True
                array(e['candidates'],len(e['candidates']),'int')
                assert all(0 <= i < n+m for i in e['candidates'])
                assert 0 <= e['position'] <= len(e['candidates'])
                array(e['failures'],n+m,'int')
                if e['support'] > 0:array(e['support_marks'],m,'int')
                else:assert e['support_marks'] is None
                assert e['batch'] == 0, 'batch는 후속 범위'
            elif name == 'supply:leave':
                assert supply;supply=False;candidate=e['candidate'];chosen=None
                if candidate >= 0:
                    step+=1;assert candidate < n+m;sparse(e['alpha'],m)
                else:assert e['alpha'] is None
            elif name == 'ratio:enter':
                assert candidate is not None and candidate >= 0 and ratio is None and not pending
                ratio=e['name'];assert ratio in ('feasible','infeasible')
                assert e['selected']['enter'] == candidate and e['selected']['direction'] in (0,1)
                sparse(e['alpha'],m)
                for k,kind in [('basic','f64'),('width','f64'),('marks',1)]:array(e[k],m,kind)
                ratio_input=e
            elif name == 'ratio:leave':
                assert e['name'] == ratio;ratio=None;chosen=e
                number(e['theta']);number(e['pivot']);number(e['work'])
                idx=e['chosen'];a=ratio_input['alpha']
                assert idx >= -1
                if idx >= 0:
                    assert idx < (m if a['count'] < 0 else a['count'])
                    assert e['row'] == (idx if a['count'] < 0 else a['indices'][idx])
                else:assert e['row'] == -1
            elif name == 'step:enter':
                assert chosen and not ratio and not pending and candidate >= 0
                assert e['selected']['enter'] == candidate and e['selected']['direction'] in (0,1)
                pending=e;assert e['kind'] in ('pivot','flip');number(e['theta'])
                if e['kind']=='pivot':
                    assert e['selected']['leave'] == chosen['chosen'] >= 0
                    assert number(e['pivot']) >= 1e-6
                else:assert e['selected']['leave'] == -1 and e['pivot'] is None
            elif name == 'step:applied':
                assert not applied and pending and e['kind'] == pending['kind'] and e['selected']['enter'] == candidate
                before,after=pending['state'],e['state'];enter=candidate
                assert number(after['counters']['iterations']) == number(before['counters']['iterations'])+1
                if e['kind']=='pivot':
                    row=chosen['row'];leave=before['header'][row]
                    expected=before['header'].copy();expected[row]=enter
                    assert after['header'] == expected and after['vstat'][enter] == row and after['vstat'][leave] in (-1,-2,-4)
                else:
                    assert before['header'] == after['header'] and after['vstat'][enter] in (-1,-2,-3)
                applied=e
            elif name == 'step:leave':
                assert pending and applied and e['kind']==pending['kind']
                assert e['selected']['enter']==candidate
                assert set(e['state']['header'])==set(applied['state']['header']), 'factor 후 basis 집합'
                assert e['state']['counters']['iterations']==applied['state']['counters']['iterations']
                counts[e['kind']]+=1;pending=None;applied=None
            elif name == 'factor:enter':
                assert not factor;factor=True
                array(e['header'],m,'int')
            elif name == 'factor:leave':
                assert factor;factor=False
                assert e['ret'] in (0,1);array(e['header'],m,'int')
            elif name == 'push:leave':
                assert not pending and not ratio and not supply and not factor
                assert e['ret'] == 0 and e['retry'] == 0 and e['remaining'] == 0
                number(e['condition']);active=False
                array(r['result']['vbasis'],n,'int');array(r['result']['cbasis'],m,'int')
            else:
                raise AssertionError('알 수 없는 사건: '+name)
        assert e['step'] == step
    assert not active and not pending and not ratio and not supply and not factor
    assert counts['push'] > 0 and counts['pivot'] > 0 and counts['flip'] > 0, ('목표 부족',counts)
    assert digest(events) == case['events_sha256'], '기준 사건·상태 변조'
    return counts


def check(paths, control=False):
    assert len(paths)>=3 and len(set(p.resolve() for p in paths))==len(paths), '서로 다른 독립 캡처 3회 필요'
    manifest=json.loads((BASE/'manifest.json').read_text())
    for path,expected in manifest['sources'].items():assert sha(ROOT/path)==expected, path
    records=[json.loads(p.read_text()) for p in paths]
    assert len({r['run_id'] for r in records})==len(records), '독립 실행 중복'
    assert len({r['pid'] for r in records})==len(records), 'PID 중복'
    counts=[validate(r,manifest,control) for r in records]
    fields=['kind','model_sha256','binary_sha256','tool_sha256','agent_sha256','params','starts','environment','result','reason','errors','events']
    first={k:records[0][k] for k in fields}
    for r in records[1:]:assert {k:r[k] for k in fields}==first, '독립 실행 불일치'
    return counts


def mutation(paths):
    r=json.loads(paths[0].read_text());cases={}
    def put(name,edit):
        x=copy.deepcopy(r);edit(x);cases[name]=x
    for key in ['binary_sha256','model_sha256','tool_sha256','agent_sha256']:
        put(key,lambda x,k=key:x.__setitem__(k,'0'*64))
    put('events-empty',lambda x:x.__setitem__('events',[]))
    put('synthetic',lambda x:x.__setitem__('origin','original-synthetic'))
    put('state-missing',lambda x:x['events'][0]['state'].pop('header'))
    put('pivot-zero',lambda x:x.__setitem__('events',[e for e in x['events'] if e.get('kind')!='pivot']))
    put('flip-zero',lambda x:x.__setitem__('events',[e for e in x['events'] if e.get('kind')!='flip']))
    put('ratio-theta',lambda x:next(e for e in x['events'] if e['event']=='ratio:leave').__setitem__('theta','0'*16))
    put('basis',lambda x:next(e for e in x['events'] if e['event']=='step:leave')['state']['header'].__setitem__(0,-1))
    put('remaining',lambda x:next(e for e in x['events'] if e['event']=='push:leave').__setitem__('remaining',-1))
    put('source-kind',lambda x:x.__setitem__('kind','control'))
    results={}
    with tempfile.TemporaryDirectory() as tmp:
        for name,x in cases.items():
            p=Path(tmp)/f'{name}.json';p.write_text(json.dumps(x))
            proc=subprocess.run([sys.executable,__file__,str(p),*[str(p) for p in paths[1:]]],capture_output=True,text=True)
            assert proc.returncode!=0, ('변조 허용',name)
            results[name]={'returncode':proc.returncode,'error':proc.stderr.strip()}
        proc=subprocess.run([sys.executable,__file__,str(paths[0]),str(paths[0]),str(paths[0])],capture_output=True,text=True)
        assert proc.returncode!=0
        results['duplicate-run']={'returncode':proc.returncode,'error':proc.stderr.strip()}
    return results


if __name__=='__main__':
    parser=argparse.ArgumentParser();parser.add_argument('paths',nargs='+',type=Path);parser.add_argument('--control',action='store_true');parser.add_argument('--mutation',type=Path)
    args=parser.parse_args()
    try:
        counts=check(args.paths,args.control)
        if args.mutation:
            assert not args.mutation.exists(), '검증 기록 덮어쓰기 거부'
            args.mutation.write_text(json.dumps(mutation(args.paths),ensure_ascii=False,indent=2)+'\n')
        print(json.dumps({'captures':len(counts),'counts':counts},ensure_ascii=False))
    except (AssertionError,KeyError,TypeError,ValueError,IndexError) as error:
        print('불합격: '+str(error),file=sys.stderr);sys.exit(1)
