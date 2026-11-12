"""M3-04c-19: 원본 보류/패스/고갈/BTRAN/round factor 명령 구간 실행.

내부 명령은 보존한다. 출구 stub, 수치 BTRAN/factor와 문맥 helper만 대체한다.
batch 용량/미소비=0. 자연 도달성을 뜻하지 않는 기계 상태 경계다.
"""
import argparse
import ctypes as c
import hashlib
import json
import mmap
import os
from pathlib import Path
import platform
import struct
import uuid

from push_factor_synthetic import BINARY_SHA, Patches, bits

ROOT = Path(__file__).resolve().parents[1]
SCHEMA = 'M3-04c-19-synthetic-v1'
SUBSTITUTION = 'original in-place defer/pass/exhaustion/BTRAN-consumer/round-wrapper; exit stubs; numeric BTRAN/factor and context helpers replaced; batch=0'


def specs():
    base = dict(m=9, candidates=[5,2,7,2,8,1], position=4, deferred=2,
                failures=[0,1,2,3,4,0,1,2,3,0,0,0], marks=[1,0,1,0,1,0,1,0,1],
                support=1, approved=3, enter=4, leave=2, request=0, status=1,
                numeric=3, failure=0, attempt=0, numeric_limit=13,
                count_sum=bits(-3.), successes=7)
    cases=[]
    def add(kind,name,**kw):
        x=dict(base,**kw)
        if x['m']!=9:x['marks']=[i%2 for i in range(x['m'])]
        cases.append(dict(name=name,kind=kind,input=x))
    for failure in (0,1,2,3):
        f=base['failures'].copy();f[4]=failure
        add('defer',f'defer-{failure}',failures=f)
    for support in (0,1,-1):
        add('pass',f'pass-{support}',support=support)
        for m in (0,1,2,4,9,10):
            for approved in sorted({0,max(0,int(m**.5)-1),int(m**.5),int(m**.5)+1}):
                add('exhausted',f'sqrt-m{m}-a{approved}-s{support}',m=m,approved=approved,support=support)
        add('exhausted',f'all-deferred-{support}',deferred=6,position=6,support=support,approved=0)
        add('exhausted',f'empty-{support}',candidates=[],deferred=0,position=0,support=support,approved=0)
    add('exhausted','no-prefix',deferred=0,position=3)
    add('exhausted','prefix-only',deferred=2,position=6)
    add('exhausted','duplicates-order',candidates=[8,2,7,2,8,1],position=3,deferred=2)
    for indices in ([],[4],[8,1],[8,0,4],[7,2,5,0]):
        add('btran','sparse-'+str(len(indices)),raw=dict(count=len(indices),indices=indices,values=[bits(0. if k%2 else -2.) for k in range(len(indices))]))
    for approved in (0,3,4):
        add('btran',f'dense-a{approved}',approved=approved,raw=dict(count=-1,indices=None,values=[bits(float(i)) for i in range(9)]))
    add('btran','dense-empty',candidates=[],deferred=0,position=0,raw=dict(count=-1,indices=None,values=[bits(0.)]*9))
    for support in (0,1,-1):
        add('btran',f'support-{support}',support=support,raw=dict(count=1,indices=[4],values=[bits(1.)]))
    reply_cases={
        'normal':[dict(ret=0)],'repeat1':[dict(ret=1),dict(ret=0)],
        'repeat2':[dict(ret=1),dict(ret=1),dict(ret=0)],
        'return12':[dict(ret=12)],'error7':[dict(ret=7)],
        'error-before-status':[dict(ret=7,status=-1000)],
        'backend-numeric':[dict(ret=0,numeric=13)],
        'backend-request':[dict(ret=1,request=6),dict(ret=0)]}
    for name,replies in reply_cases.items():add('wrapper',name,replies=replies,request=10)
    for status in (-999,-1000):add('wrapper',f'status{status}',status=status,request=10,replies=[dict(ret=0)])
    for attempt in (0,1):
        add('wrapper',f'failure-attempt{attempt}',failure=1,attempt=attempt,request=10,replies=[dict(ret=0)])
        for numeric in (12,13,14):add('wrapper',f'numeric{numeric}-attempt{attempt}',numeric=numeric,attempt=attempt,request=10,replies=[dict(ret=0)])
    return cases


def sha_specs():
    return hashlib.sha256(json.dumps(specs(),sort_keys=True).encode()).hexdigest()


def sha(p):
    return hashlib.sha256(p.read_bytes()).hexdigest()


def capture():
    import gurobipy as gp
    assert gp.gurobi.version()==(13,0,3) and platform.machine()=='x86_64'
    maps=[x.split() for x in Path('/proc/self/maps').read_text().splitlines() if 'libgurobi130.so' in x and x.split()[2]=='00000000']
    assert len(maps)==1
    library,base=Path(maps[0][-1]),int(maps[0][0].split('-')[0],16)
    assert sha(library)==BINARY_SHA
    # Save actual SysV stack/callee saves. ecx seeds ebp (approved or leave).
    # r12/r13 are zero at caller exhaustion; r14=S, r15=work(NULL), rbx=context(NULL).
    pro=bytes.fromhex('55 53 41 54 41 55 41 56 41 57 49 89 e3 48 89 f4 4c 89 9c 24 00 21 00 00 49 89 fe 45 31 ff 45 31 e4 45 31 ed 31 db 89 cd 49 89 d3 8b 54 24 0c 41 ff e3')
    cleanup=bytes.fromhex('48 8b a4 24 f8 20 00 00 41 5f 41 5e 41 5d 41 5c 5b 5d c3')
    code=mmap.mmap(-1,4096,prot=mmap.PROT_READ|mmap.PROT_WRITE|mmap.PROT_EXEC);code.write(pro+cleanup)
    ca=c.addressof(c.c_char.from_buffer(code))
    native=c.CFUNCTYPE(c.c_int,c.c_void_p,c.c_void_p,c.c_void_p,c.c_int)(ca)
    patch=Patches(base);events=[]
    def stop(off,val):patch.put(off,b'\xb8'+struct.pack('<i',val)+b'\xc3')
    def jump(off,target):patch.put(off,b'\x48\xb8'+struct.pack('<Q',target)+b'\xff\xe0')
    try:
        for case in specs():
            keep=[]
            def buf(n):
                b=c.create_string_buffer(max(1,n));keep.append(b);return c.addressof(b)
            def put(p,o,t,v):t.from_address(p+o).value=v
            def read(p,o,t=c.c_int):return t.from_address(p+o).value
            def ints(values):
                a=buf(max(32,len(values)*4));
                for i,v in enumerate(values):put(a,i*4,c.c_int,v)
                return a
            x=case['input'];m=x['m'];s=buf(0x500);f=buf(0x300)
            memory=buf(0x20000);frame=(memory+0x10000+15)&-16;p=frame+8
            retry=buf(4);candidates=ints(x['candidates']);failures=ints(x['failures']);marks=ints(x['marks'])
            put(frame,0,c.c_void_p,ca+len(pro));put(s,0x470,c.c_void_p,f)
            for off,val in ((0,m),(4,len(x['candidates'])),(0x20,x['position']),(0x24,x['deferred']),(0x38,x['successes'])):put(p,off,c.c_int,val)
            for off,val in ((8,candidates),(0x10,failures),(0x18,marks)):put(p,off,c.c_void_p,val)
            c.memmove(p+0x30,bytes.fromhex(x['count_sum']),8)
            for off,key in ((0x209c,'support'),(0x1ea0,'approved'),(0x1fa0,'attempt'),(0x1fe0,'numeric_limit')):put(frame,off,c.c_int,x[key])
            put(frame,0x1e98,c.c_double,float(m));put(frame,0x20e8,c.c_void_p,retry);put(frame,0x2048,c.c_void_p,f)
            for off,key in ((0x294,'request'),(0xf0,'status'),(0x178,'numeric'),(0x184,'failure'),(0x2fc,'enter'),(0x2f8,'leave')):put(s,off,c.c_int,x[key])
            def result(exit,calls):
                total=read(p,4)
                return dict(exit=exit,m=read(p,0),total=total,candidates=[read(candidates,i*4) for i in range(total)],
                    position=read(p,0x20),deferred=read(p,0x24),failures=[read(failures,i*4) for i in range(len(x['failures']))],
                    marks=[read(marks,i*4) for i in range(m)],support=read(frame,0x209c),approved=read(frame,0x1ea0),
                    return_code=read(frame,0x1f70),request=read(s,0x294),status=read(s,0xf0),numeric=read(s,0x178),failure=read(s,0x184),retry=read(retry,0),
                    count_sum=c.string_at(p+0x30,8).hex(),successes=read(p,0x38),batch=[read(p,o) for o in (0x1840,0x1844,0x1848)],calls=calls)
            calls=[];errors=[];kind=case['kind'];seed=x['approved']
            if kind=='defer':
                stop(0x1c3b9f,101);entry=0x1c3b7b
            elif kind=='pass':
                # rax holds destination count after compression; stop before passing P to supplier.
                patch.put(0x1c20be,bytes.fromhex('89 44 24 0c 44 89 64 24 28 44 89 64 24 2c b8 65 00 00 00 c3'))
                entry=0x1c1ff8
            elif kind=='exhausted':
                stop(0x1c3d26,102);stop(0x1c4021,103);entry=0x1c3bdf
                for off in (0x8eacf0,0x8ead90):patch.put(off,b'\xc3')
            elif kind=='btran':
                seed=x['leave'];vin=buf(24);vout=buf(24);idx=ints([0]*max(1,m));values=buf(max(8,m*8))
                put(vin,8,c.c_void_p,idx);put(vin,16,c.c_void_p,values)
                put(frame,0x1ff8,c.c_void_p,vin);put(frame,0x2000,c.c_void_p,vout)
                @c.CFUNCTYPE(None,c.c_void_p,c.c_void_p,c.c_void_p,c.c_void_p)
                def btran(factor,inp,out,work):
                    try:
                        assert factor==f and inp==vin and out==vout and not work
                        calls.append(dict(event='btran',input=dict(count=read(vin,0),indices=[read(idx,0)],values=[c.string_at(values,8).hex()])))
                        raw=x['raw'];ri=ints(raw['indices'] or []);rv=buf(max(8,len(raw['values'])*8))
                        for i,v in enumerate(raw['values']):c.memmove(rv+i*8,bytes.fromhex(v),8)
                        put(vout,0,c.c_int,raw['count']);put(vout,8,c.c_void_p,ri);put(vout,16,c.c_void_p,rv)
                    except BaseException as exc:errors.append(repr(exc))
                jump(0x131710,c.cast(btran,c.c_void_p).value)
                stop(0x1c336d,104);stop(0x1c3d26,102);stop(0x1c4021,103)
                for off in (0x8eacf0,0x8ead90):patch.put(off,b'\xc3')
                entry=0x1c32b5
            else:
                assert kind=='wrapper';fi=0
                @c.CFUNCTYPE(c.c_int,c.c_void_p,c.c_void_p)
                def factor(ws,work):
                    nonlocal fi
                    try:
                        assert ws==s and not work and fi<len(x['replies'])
                        reply=x['replies'][fi];fi+=1
                        calls.append(dict(event='factor',request=read(s,0x294),reply=reply))
                        for key,off in (('status',0xf0),('numeric',0x178),('failure',0x184),('request',0x294)):
                            if key in reply:put(s,off,c.c_int,reply[key])
                        return reply['ret']
                    except BaseException as exc:errors.append(repr(exc));return 99
                jump(0xf15c0,c.cast(factor,c.c_void_p).value)
                for off in (0x8eacf0,0x8ead90):patch.put(off,b'\xc3')
                stop(0x1c3db4,105);stop(0x1c08d5,106);entry=0x1c3d26
            ret=native(s,frame,base+entry,seed)
            assert not errors,(case['name'],errors)
            if kind=='wrapper':assert fi==len(x['replies'])
            events.append(dict(case,result=result(ret,calls)))
            patch.restore()
    finally:patch.restore()
    return dict(schema=SCHEMA,origin='synthetic-original-slice',substitution=SUBSTITUTION,binary_sha256=BINARY_SHA,
        tool_sha256=sha(Path(__file__)),input_sha256=sha_specs(),run_id=str(uuid.uuid4()),pid=os.getpid(),
        environment=dict(machine=platform.machine(),system=platform.system(),python=platform.python_version()),events=events)


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--capture',type=Path,required=True);args=parser.parse_args()
    assert not args.capture.exists(),'기존 캡처 덮어쓰기 거부'
    args.capture.write_text(json.dumps(capture(),ensure_ascii=False,indent=1)+'\n')
    print('원본 round slice 합성',len(specs()),'사례')
