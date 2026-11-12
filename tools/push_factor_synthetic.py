"""M3-04c-15: 원본 선택/직렬 wrapper 명령 구간의 격리 실행.

원본 내부 명령을 재작성하지 않는다. 입출구만 반환 stub으로 바꾸고
wrapper의 factor·품질·문맥 helper를 명시적 backend로 대체한다.
"""
import argparse
import ctypes as c
import hashlib
import json
import mmap
from pathlib import Path
import platform
import struct
import uuid

ROOT = Path(__file__).resolve().parents[1]
BINARY_SHA = '8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c'
SCHEMA = 'M3-04c-15-synthetic-v1'
SUBSTITUTION = 'original in-place selector/wrapper; exit stubs; wrapper factor, quality, context helpers replaced; no numeric LU'

def bits(x):
    return struct.pack('<d', x).hex()

def sha(p):
    return hashlib.sha256(p.read_bytes()).hexdigest()

def specs():
    sel = dict(m=8, request=0, numeric=0, k=2, limit=10, q=4, C=0, E=0, L=20, U=20, support=0)
    cases = []
    def add(name, **kw):
        cases.append(dict(name=name, kind='select', input=dict(sel, **kw)))
    add('update')
    for req in (2, 6, 10):add(f'preserve-{req}', request=req, k=10, support=1)
    for k in (9, 10, 11):add(f'limit-{k}', k=k, q=12)
    for k in (3, 4, 5):
        for num in (0, 1):add(f'q-{k}-numeric-{num}', k=k, numeric=num)
    for support in (1, -1):add(f'skip-{support}', support=support)
    for cost in (-1, 0, 1):
        # m=2, L+U=5 -> a²=4; (C+E)*2m=4 when C+E=1.
        add(f'cost-{cost}', m=2, k=4, C=1+cost, L=2, U=3, q=4)
    for C in (19,20,21):add(f'min-q-term-{C}', m=2, k=4, C=C, L=50, U=50, q=4)
    add('min-q-zero', m=2, k=0, C=1, L=2, U=3, q=0)
    # Signed machine-input boundary, not reachable nonnegative nnz counters:
    # integer sum is 6; separately converted L/U sum is 5.
    add('sum-convert', m=2, k=4, C=2, E=0, L=2**53+1, U=-2**53+5, q=4)
    add('negative-cost-support', m=2, k=4, C=0, L=2, U=3, support=1)
    base = dict(request=0, numeric=3, failure=0, status=1, k=1, period=50, attempt=0, numeric_limit=13, enter=7)
    def wrap(name, replies=None, qualities=None, **kw):
        cases.append(dict(name=name, kind='wrapper', input=dict(base, **kw),
                          replies=replies or [dict(ret=0)], qualities=qualities or []))
    wrap('normal')
    wrap('repeat-one', [dict(ret=1), dict(ret=0)])
    wrap('repeat-two', [dict(ret=1), dict(ret=1), dict(ret=0)])
    wrap('repeat-request', [dict(ret=1,request=6),dict(ret=0)])
    wrap('numeric-crossed-by-backend', [dict(ret=0,numeric=13)])
    wrap('return12', [dict(ret=12)])
    wrap('error7', [dict(ret=7)])
    wrap('error-precedes-status', [dict(ret=7, status=-1000)])
    for status in (-999, -1000):wrap(f'status-{status}', status=status)
    for attempt in (0, 1):
        wrap(f'failure-attempt-{attempt}', attempt=attempt, failure=1)
        for num in (12, 13, 14):wrap(f'numeric-{num}-attempt-{attempt}', numeric=num, attempt=attempt)
    wrap('request-recompute', request=10)
    wrap('request-preserved-on-retry', request=6, replies=[dict(ret=12)])
    wrap('zero-updates-period100', k=0, period=100)
    wrap('nonzero-clamp100', k=1, period=100)
    wrap('period20-kept', k=1, period=20)
    for inc in (5, 10, 20, 50):
        wrap(f'quality-increment-{inc}', k=50, qualities=[dict(increment=inc, request=0)])
    wrap('quality-above-period', k=51, qualities=[dict(increment=10, request=0)])
    wrap('quality-request6', replies=[dict(ret=0),dict(ret=0, k=0)], k=50,
         qualities=[dict(increment=5, request=6)])
    wrap('quality-request6-repeat', replies=[dict(ret=0),dict(ret=1),dict(ret=0,k=0)], k=50,
         qualities=[dict(increment=20, request=6)])
    wrap('quality-status12', k=50, qualities=[dict(increment=50, request=0, status=12)])
    return cases

def sha_specs():
    return hashlib.sha256(json.dumps(specs(), sort_keys=True).encode()).hexdigest()

class Patches:
    def __init__(self, base):
        self.base, self.saved = base, []
        self.libc = c.CDLL(None)
        self.libc.mprotect.argtypes = [c.c_void_p, c.c_size_t, c.c_int]
    def put(self, off, data):
        at = self.base+off
        page=at & -mmap.PAGESIZE
        size=((at+len(data)-page+mmap.PAGESIZE-1)//mmap.PAGESIZE)*mmap.PAGESIZE
        self.saved.append((at, c.string_at(at,len(data)),page,size))
        assert self.libc.mprotect(page,size,7)==0
        c.memmove(at,data,len(data))
        assert self.libc.mprotect(page,size,5)==0
    def restore(self):
        for at,data,page,size in reversed(self.saved):
            assert self.libc.mprotect(page,size,7)==0
            c.memmove(at,data,len(data))
            assert self.libc.mprotect(page,size,5)==0
        self.saved=[]

def capture():
    import gurobipy as gp
    assert gp.gurobi.version()==(13,0,3) and platform.machine()=='x86_64'
    maps=[x.split() for x in Path('/proc/self/maps').read_text().splitlines()
          if 'libgurobi130.so' in x and x.split()[2]=='00000000']
    assert len(maps)==1
    library,base=Path(maps[0][-1]),int(maps[0][0].split('-')[0],16)
    assert sha(library)==BINARY_SHA
    # Native SysV trampoline: save rbp/rbx/r12-r15, switch to synthetic frame,
    # r14=S, r15=0(work), jump original entry. Exit ret returns to cleanup;
    # cleanup restores the real stack and all callee-saved registers.
    pro=bytes.fromhex('55 53 41 54 41 55 41 56 41 57 49 89 e3 48 89 f4 4c 89 9c 24 00 21 00 00 49 89 fe 45 31 ff ff e2')
    cleanup=bytes.fromhex('48 8b a4 24 f8 20 00 00 41 5f 41 5e 41 5d 41 5c 5b 5d c3')
    code=mmap.mmap(-1,4096,prot=mmap.PROT_READ|mmap.PROT_WRITE|mmap.PROT_EXEC)
    code.write(pro+cleanup)
    ca=c.addressof(c.c_char.from_buffer(code))
    native=c.CFUNCTYPE(c.c_int,c.c_void_p,c.c_void_p,c.c_void_p)(ca)
    patch=Patches(base)
    events=[]
    def stop(off, val):patch.put(off,b'\xb8'+struct.pack('<i',val)+b'\xc3')
    def jump(off, target):patch.put(off,b'\x48\xb8'+struct.pack('<Q',target)+b'\xff\xe0')
    try:
        for case in specs():
            keep=[]
            def buf(n):
                b=c.create_string_buffer(n);keep.append(b);return c.addressof(b)
            def put(p,o,t,v):t.from_address(p+o).value=v
            def read(p,o,t=c.c_int):return t.from_address(p+o).value
            s,f=buf(0x500),buf(0x300)
            memory=buf(0x20000);frame=(memory+0x10000+15)&-16
            retry=buf(4);oldalpha=buf(24);alpha=buf(24);spike=buf(24)
            put(frame,0,c.c_void_p,ca+len(pro))
            put(s,0x470,c.c_void_p,f);put(s,0x210,c.c_void_p,oldalpha);put(f,0x1c8,c.c_void_p,spike)
            sentinels=[(oldalpha,bytes(range(24))),(alpha,bytes(range(24,48))),(spike,bytes(range(48,72)))]
            for at,content in sentinels:c.memmove(at,content,len(content))
            put(frame,0x2048,c.c_void_p,f);put(frame,0x2090,c.c_void_p,alpha);put(frame,0x20e8,c.c_void_p,retry)
            x=case['input']
            for off,key in ((0x294,'request'),(0x178,'numeric'),(0x184,'failure'),(0xf0,'status'),(0x46c,'q')):
                put(s,off,c.c_int,x.get(key,0))
            for off,key in ((0x18c,'k'),(0x188,'limit'),(0x184,'E')):put(f,off,c.c_int,x.get(key,0))
            for off,key in ((0x190,'C'),(0x1e0,'L'),(0x1e8,'U')):put(f,off,c.c_int64,x.get(key,0))
            put(frame,0x209c,c.c_int,x.get('support',0))
            put(s,0x2fc,c.c_int,x.get('enter',7))
            put(frame,0x1fa0,c.c_int,x.get('attempt',0));put(frame,0x1fe0,c.c_int,x.get('numeric_limit',10))
            put(frame,0x1fd8,c.c_int,x.get('period',50))
            m=float(x.get('m',8))
            for off,val in ((0x1e78,m*1.5),(0x1e80,(m*5.)*m),(0x1e98,m)):put(frame,off,c.c_double,val)
            calls=[];errors=[]
            if case['kind']=='select':
                stop(0x1c3465,101);stop(0x1c3b6e,100)
                before_s=c.string_at(s,0x500);before_f=c.string_at(f,0x300)
                ret=native(s,frame,base+0x1c336d)
                expected=bytearray(before_s);expected[0x294:0x298]=c.string_at(s+0x294,4)
                assert c.string_at(s,0x500)==bytes(expected) and c.string_at(f,0x300)==before_f
                result=dict(request=read(s,0x294),call=ret==101,preserved=True)
                assert ret in (100,101)
            else:
                fi=qi=0
                @c.CFUNCTYPE(c.c_int,c.c_void_p,c.c_void_p)
                def factor(ws, work):
                    nonlocal fi
                    try:
                        assert ws==s and not work and fi<len(case['replies'])
                        reply=case['replies'][fi];fi+=1
                        calls.append(dict(event='factor',request=read(s,0x294),alpha_current=read(s,0x210,c.c_void_p)==alpha,
                                          spike_original=read(f,0x1c8,c.c_void_p)==spike,reply=reply))
                        for key,off in (('status',0xf0),('numeric',0x178),('failure',0x184),('request',0x294)):
                            if key in reply:put(s,off,c.c_int,reply[key])
                        if 'k' in reply:put(f,0x18c,c.c_int,reply['k'])
                        return reply['ret']
                    except BaseException as exc:errors.append(repr(exc));return 99
                @c.CFUNCTYPE(None,c.c_void_p,c.c_int,c.c_void_p,c.c_void_p)
                def quality(ws, enter, inc, work):
                    nonlocal qi
                    try:
                        assert ws==s and not work and qi<len(case['qualities']) and enter==x['enter']
                        reply=case['qualities'][qi];qi+=1
                        calls.append(dict(event='quality',enter=enter,period=read(frame,0x1fd8),request=read(s,0x294),reply=reply))
                        put(inc,0,c.c_int,reply['increment']);put(s,0x294,c.c_int,reply['request'])
                        if 'status' in reply:put(s,0xf0,c.c_int,reply['status'])
                    except BaseException as exc:errors.append(repr(exc));put(s,0xf0,c.c_int,12)
                for off in (0x8eacf0,0x8ead90):patch.put(off,b'\xc3')
                jump(0xf15c0,c.cast(factor,c.c_void_p).value);jump(0x115ed0,c.cast(quality,c.c_void_p).value)
                stop(0x1c3b79,200);stop(0x1c3872,201)
                stop(0x1c08d5,202);patch.put(0x1c4591,bytes.fromhex('89 d8 c3'))
                ret=native(s,frame,base+0x1c3465)
                assert not errors,errors
                assert fi==len(case['replies']) and qi==len(case['qualities']),(case['name'],fi,qi)
                result=dict(exit=ret,request=read(s,0x294),status=read(s,0xf0),numeric=read(s,0x178),failure=read(s,0x184),
                            retry=read(retry,0),period=read(frame,0x1fd8),k=read(f,0x18c),
                            alpha_restored=read(s,0x210,c.c_void_p)==oldalpha,spike_restored=read(f,0x1c8,c.c_void_p)==spike,contents_preserved=all(c.string_at(at,24)==content for at,content in sentinels),calls=calls)
            patch.restore()
            events.append(dict(case,result=result))
    finally:patch.restore()
    return dict(schema=SCHEMA,origin='synthetic-original-slice',substitution=SUBSTITUTION,
                binary_sha256=BINARY_SHA,tool_sha256=sha(Path(__file__)),input_sha256=sha_specs(),run_id=str(uuid.uuid4()),
                pid=__import__('os').getpid(),environment=dict(machine=platform.machine(),system=platform.system(),python=platform.python_version()),events=events)

if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--capture',type=Path,required=True);args=parser.parse_args()
    assert not args.capture.exists(),'기존 캡처 덮어쓰기 거부'
    result=capture();args.capture.write_text(json.dumps(result,ensure_ascii=False,indent=1)+'\n')
    print('원본 slice 합성',len(result['events']),'사례')
