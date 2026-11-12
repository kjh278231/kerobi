"""M3-04c-13: 원본 supplier 제어 직접 호출. FTRAN만 합성 backend로 대체한다.

격리 프로세스에서 09d008 진입을 ctypes callback으로 바꿨다가 복원한다.
실제 LU/FTRAN 수치 실행 또는 자연 crossover 사례로 해석하면 안 된다.
"""
import argparse
import ctypes as c
import hashlib
import json
import math
from pathlib import Path
import platform
import struct
import uuid

ROOT = Path(__file__).resolve().parents[1]
BINARY_SHA = '8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c'
SCHEMA = 'M3-04c-13-synthetic-v1'
SUBSTITUTION = '09d008@0x120680 replaced by explicit synthetic FTRAN backend; supplier@0x1c5310 is original'


def bits(v):
    return struct.pack('<d', v).hex()


def sha(p):
    return hashlib.sha256(p.read_bytes()).hexdigest()


def specs():
    # m=9 (odd), n=3; 컬럼0의 CSC는 행5,1,8 순서다.
    sparse = dict(count=3, indices=[8, 0, 3], values=[bits(2.), bits(-0.), bits(-3.)])
    tol = 0.25
    dense = dict(count=-1, indices=None, values=list(map(bits, [tol, -tol, math.nextafter(tol, 0),
        math.nextafter(tol, math.inf), -math.nextafter(tol, math.inf), 0., -0., 2., -3.])))
    base = dict(candidates=[0], position=0, deferred=0, support=0, failures=[0]*12,
                support_marks=None, count_sum=bits(7.25), successes=4, batch=[0, 0, 0])
    result = []
    def add(name, raw=sparse, **kw):
        result.append(dict(name=name, before=dict(base, **kw), raw=raw, drop=bits(tol)))
    add('structural-sparse')
    add('slack-sparse', candidates=[6])
    add('negative-support', support=-1)
    add('positive-support', support=1, support_marks=[0]*9)
    add('structural-conflict', candidates=[0, 1], support=1, support_marks=[0,0,0,0,0,1,0,0,0])
    add('slack-conflict', candidates=[8, 6], support=1, support_marks=[0,0,0,0,0,1,0,0,0])
    add('failure-two', failures=[2]+[0]*11)
    add('failure-three', candidates=[0, 1], failures=[3]+[0]*11)
    add('all-failure', raw=None, candidates=[0, 1], failures=[3, 4]+[0]*10)
    add('all-conflict', raw=None, candidates=[0, 8], support=1, support_marks=[0,0,0,0,0,1,0,0,0])
    add('already-exhausted', raw=None, position=1)
    add('empty-list', raw=None, candidates=[])
    add('empty-column', candidates=[2], raw=dict(count=0, indices=[], values=[]))
    add('dense-boundaries', raw=dense)
    add('dense-even', raw=dict(dense, values=dense['values'][:8]), m=8)
    add('dense-all-drop', raw=dict(count=-1, indices=None, values=[bits(-0.)]*9))
    add('dense-negative-count', raw=dict(dense, count=-7))
    add('prefix-preserved', candidates=[2, 0, 1], position=1, deferred=1, failures=[3]+[0]*11)
    return result


class Vector(c.Structure):
    _fields_ = [('count', c.c_int), ('indices', c.c_void_p), ('values', c.c_void_p)]


def capture():
    import gurobipy as gp
    assert gp.gurobi.version() == (13, 0, 3)
    assert platform.machine() == 'x86_64' and c.sizeof(c.c_void_p) == 8
    maps = [x.split() for x in Path('/proc/self/maps').read_text().splitlines()
            if 'libgurobi130.so' in x and x.split()[2] == '00000000']
    assert len(maps) == 1
    library, base = Path(maps[0][-1]), int(maps[0][0].split('-')[0], 16)
    assert sha(library) == BINARY_SHA
    ptr = c.c_void_p
    supplier = c.CFUNCTYPE(c.c_int, *([ptr]*5))(base + 0x1c5310)
    libc = c.CDLL(None, use_errno=True)
    libc.mprotect.argtypes = [ptr, c.c_size_t, c.c_int]
    saved = c.string_at(base + 0x120680, 16)
    current, events, callback_errors = {}, [], []

    def backend(f, iv, ov, spike, a4, a5, a6, work):
        try:
            raw, m = current['raw'], current['m']
            assert raw is not None, '예상 밖 FTRAN'
            current['calls'].append(dict(input=read_vec(iv, m), aliases=dict(factor=f==current['f'],
                input_s220=iv==current['iv'], output_s210=ov==current['ov'], spike_null=not spike,
                spike_f1c8=spike==current['spike'], work=work==current['work'], zeros=[not a4,not a5,not a6]), raw=raw))
            v = Vector.from_address(ov)
            v.count = raw['count']
            if raw['indices'] is not None:
                for i, row in enumerate(raw['indices']):
                    c.c_int.from_address(v.indices+i*4).value = row
            for i, h in enumerate(raw['values']):
                c.memmove(v.values+i*8, bytes.fromhex(h), 8)
        except BaseException as exc:
            callback_errors.append(str(exc))

    callback = c.CFUNCTYPE(None, *([ptr]*8))(backend)
    target = base + 0x120680
    page, pagesize = target & -4096, 4096
    assert libc.mprotect(page, pagesize, 7) == 0
    c.memmove(target, b'\x48\xb8'+struct.pack('<Q', c.cast(callback, ptr).value)+b'\xff\xe0', 12)
    assert libc.mprotect(page, pagesize, 5) == 0
    try:
        for case in specs():
            before, m, n = case['before'], case.get('m', 9), 3
            # 모든 버퍼는 호출 종료까지 keep이 소유한다.
            keep = []
            def buf(size):
                a = c.create_string_buffer(size);keep.append(a);return c.addressof(a)
            def ints(v):
                a=(c.c_int*max(1,len(v)))(*v);keep.append(a);return c.addressof(a)
            def doubles(v):
                a=(c.c_double*max(1,len(v)))(*v);keep.append(a);return c.addressof(a)
            def put(p, off, typ, v):
                typ.from_address(p+off).value=v
            def vector():
                v=Vector(0,ints([0]*m),doubles([0.]*m));keep.append(v);return c.addressof(v)
            p,s,f = buf(0x1870),buf(0x480),buf(0x200)
            iv,ov,spike = vector(),vector(),vector()
            starts=(c.c_int64*3)(0,3,5);keep.append(starts)
            model=dict(starts=[0,3,5],lengths=[3,2,0],rows=[5,1,m-1,2,7],
                       values=list(map(bits,[2.,-1.,3.,0.5,-2.])))
            for off, val in [(0x70,c.addressof(starts)),(0x78,ints(model['lengths'])),(0x80,ints(model['rows'])),
                             (0x88,doubles([2.,-1.,3.,0.5,-2.])),(0x210,ov),(0x220,iv),(0x470,f)]:
                put(s,off,ptr,val)
            put(s,0x64,c.c_int,m);put(s,0x68,c.c_int,n);put(s,0x3a8,c.c_double,0.25)
            put(f,0x1c8,ptr,spike)
            put(p,0,c.c_int,m);put(p,4,c.c_int,len(before['candidates']))
            for off,val in [(8,ints(before['candidates'])),(0x10,ints(before['failures'][:n+m])),
                            (0x18,ints(before['support_marks'] or [0]*m))]:
                put(p,off,ptr,val)
            for off,key in [(0x20,'position'),(0x24,'deferred'),(0x38,'successes')]:
                put(p,off,c.c_int,before[key])
            put(p,0x30,c.c_double,7.25)
            support,out,work = c.c_int(before['support']),ptr(0),c.c_double(0.)
            current.update(raw=case['raw'],m=m,f=f,iv=iv,ov=ov,spike=spike,work=c.addressof(work),calls=[])
            ret=supplier(p,s,c.byref(out),c.byref(support),c.byref(work))
            assert not callback_errors, callback_errors
            def pints(off, count):
                v=ptr.from_address(p+off).value
                return [c.c_int.from_address(v+i*4).value for i in range(count)]
            after=dict(m=m,total=len(before['candidates']),candidates=pints(8,len(before['candidates'])),
                position=c.c_int.from_address(p+0x20).value,deferred=c.c_int.from_address(p+0x24).value,
                failures=pints(0x10,n+m),support=support.value,support_marks=pints(0x18,m) if support.value>0 else None,
                count_sum=c.string_at(p+0x30,8).hex(),successes=c.c_int.from_address(p+0x38).value,batch=[0,0,0])
            before=dict(before,m=m,total=len(before['candidates']),failures=before['failures'][:n+m])
            events.append(dict(name=case['name'],m=m,n=n,model=model,drop=case['drop'],before=before,
                after=after,ret=ret,output_is_s210=out.value==ov,alpha=read_vec(ov,m) if ret>=0 else None,
                ftran=current['calls']))
    finally:
        assert libc.mprotect(page, pagesize, 7) == 0
        c.memmove(target,saved,len(saved))
        assert libc.mprotect(page, pagesize, 5) == 0
    return dict(schema=SCHEMA,origin='synthetic-backend',substitution=SUBSTITUTION,run_id=str(uuid.uuid4()),
        binary_sha256=BINARY_SHA,tool_sha256=sha(Path(__file__)),input_sha256=sha_specs(),
        environment=dict(machine=platform.machine(),system=platform.system(),python=platform.python_version()),events=events)


def read_vec(p, m):
    v=Vector.from_address(p)
    count=v.count;n=m if count<0 else count
    assert 0<=n<=m
    return dict(count=count,indices=None if count<0 else [c.c_int.from_address(v.indices+i*4).value for i in range(n)],
                values=[c.string_at(v.values+i*8,8).hex() for i in range(n)])


def sha_specs():
    return hashlib.sha256(json.dumps(specs(),sort_keys=True).encode()).hexdigest()


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--capture',required=True,type=Path);a=p.parse_args()
    assert not a.capture.exists(), '기존 캡처 덮어쓰기 거부'
    result=capture()
    a.capture.write_text(json.dumps(result,ensure_ascii=False,indent=1)+'\n')
    print('합성 backend',len(result['events']),'사례')
