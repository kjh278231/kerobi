"""M3-04c-17: 원본 두 재계산 명령 구간. FTRAN/로그는 명시적 대체 backend."""
import argparse
import ctypes as c
import hashlib
import json
import math
import mmap
import os
from pathlib import Path
import platform
import struct
import uuid

from push_factor_synthetic import BINARY_SHA, Patches, bits, sha

ROOT = Path(__file__).resolve().parents[1]
SCHEMA = 'M3-04c-17-synthetic-v1'
SUBSTITUTION = 'original in-place recompute; jump exits; FTRAN supplied raw output; log no-op; no numeric LU'


def specs():
    cases = []
    I, h = 1e30, 0.125
    for site in ('pivot', 'round'):
        def add(name, triples, **kw):
            # Reverse header makes a wrong coordinate or sorted traversal observable.
            m = len(triples)
            x = dict(m=m, header=list(reversed(range(m))),
                     lb=[bits(t[0]) for t in triples], ub=[bits(t[1]) for t in triples],
                     raw=[bits(t[2]) for t in reversed(triples)],
                     rhs=[bits(17.25+i) for i in range(m)], I=bits(I), minus_I=bits(-I),
                     h=bits(h), minus_h=bits(-h), attempt=0, model_status=2, request=10,
                     aliases=dict(rhs_input=False, raw_basic=False))
            x.update(kw)
            cases.append(dict(name=site+'-'+name, site=site, input=x))
        for lower, upper, label in ((0., 4., 'bounded'), (3., 7., 'shifted'), (0., 0., 'fixed')):
            width = upper-lower
            for label2, v in (('minus-h', -h), ('h', h), ('width-minus-h', width-h), ('width-plus-h', width+h)):
                raw = lower+v
                for side, value in (('below', math.nextafter(raw, -math.inf)), ('exact', raw), ('above', math.nextafter(raw, math.inf))):
                    add(label+'-'+label2+'-'+side, [(lower, upper, value)])
        add('free', [(-I, I, -9.)])
        add('lower-infinite', [(-I, 4., -1.), (-I, 4., 4.25)])
        add('upper-infinite', [(3., I, 2.75), (3., I, 3.5)])
        add('outside-sentinel', [(-2*I, 2*I, 1.), (-2*I, 3., -0.25), (2., 2*I, 2.5)])
        add('signed-zero', [(0., 4., -0.), (-0., 4., 0.), (-I, I, -0.)])
        add('mixed', [(3., 7., 2.75), (-I, I, 0.), (0., 0., 0.), (0., 4., 4.25), (0., 4., h), (0., 4., 4.)])
        # Different magnitudes make PInf accumulation in header order observable.
        add('accumulation', [(0., I, -1.), (0., I, -1.), (0., I, -(2**54))], model_status=1)
        for rhs_alias in (False, True):
            for basic_alias in (False, True):
                add(f'alias-{int(rhs_alias)}-{int(basic_alias)}', [(3., 7., 6.25), (-I, 4., 4.5)],
                    aliases=dict(rhs_input=rhs_alias, raw_basic=basic_alias))
        for amount in (math.nextafter(1e10, 0.), 1e10, math.nextafter(1e10, math.inf)):
            for status in (1, 2):
                for attempt in (0, 1):
                    add(f'retry-{amount.hex()}-{status}-{attempt}', [(0., I, -amount)], model_status=status, attempt=attempt)
        add('empty', [])
    return cases


def sha_specs():
    return hashlib.sha256(json.dumps(specs(), sort_keys=True).encode()).hexdigest()


def capture():
    import gurobipy as gp
    assert gp.gurobi.version() == (13, 0, 3) and platform.machine() == 'x86_64'
    maps = [x.split() for x in Path('/proc/self/maps').read_text().splitlines()
            if 'libgurobi130.so' in x and x.split()[2] == '00000000']
    assert len(maps) == 1
    library, base = Path(maps[0][-1]), int(maps[0][0].split('-')[0], 16)
    assert sha(library) == BINARY_SHA
    # SysV: save callee-saved, use aligned synthetic stack, r14=S, r15=work=0,
    # r12=r13=0 (round initial counters). Exits JMP to cleanup: round writes [rsp].
    pro = bytes.fromhex('55 53 41 54 41 55 41 56 41 57 49 89 e3 48 89 f4 4c 89 9c 24 00 21 00 00 49 89 fe 45 31 ff 45 31 e4 45 31 ed ff e2')
    cleanup = bytes.fromhex('48 8b a4 24 00 21 00 00 41 5f 41 5e 41 5d 41 5c 5b 5d c3')
    code = mmap.mmap(-1, 4096, prot=mmap.PROT_READ | mmap.PROT_WRITE | mmap.PROT_EXEC)
    code.write(pro+cleanup)
    address = c.addressof(c.c_char.from_buffer(code))
    native = c.CFUNCTYPE(c.c_int, c.c_void_p, c.c_void_p, c.c_void_p)(address)
    patch, events = Patches(base), []
    def jump(off, target):
        patch.put(off, b'\x48\xb8'+struct.pack('<Q', target)+b'\xff\xe0')
    def stop(off, result):
        patch.put(off, b'\xb8'+struct.pack('<i', result)+b'\x49\xbb'+struct.pack('<Q', address+len(pro))+b'\x41\xff\xe3')
    try:
        for case in specs():
            keep = []
            def buf(size):
                obj = c.create_string_buffer(max(size, 8)); keep.append(obj)
                return c.addressof(obj)
            def put(at, off, typ, value):
                typ.from_address(at+off).value = value
            def integer(at, off=0):
                return c.c_int.from_address(at+off).value
            def floats(values):
                at = buf(len(values)*8)
                if values:
                    raw = bytes.fromhex(''.join(values)); c.memmove(at, raw, len(raw))
                return at
            def array(at, m):
                return [c.string_at(at+i*8, 8).hex() for i in range(m)]
            x = case['input']; m = x['m']
            s, f, memory = buf(0x500), buf(0x300), buf(0x20000)
            frame = (memory+0x10000+15) & -16
            rhs = floats(x['rhs']); basic = floats([bits(123.5)]*m)
            ivals = rhs if x['aliases']['rhs_input'] else floats([bits(-71.25)]*m)
            ovals = basic if x['aliases']['raw_basic'] else floats([bits(91.75)]*m)
            iv, ov = buf(24), buf(24)
            put(iv, 0, c.c_int, 99); put(iv, 16, c.c_void_p, ivals)
            put(ov, 0, c.c_int, 88); put(ov, 16, c.c_void_p, ovals)
            header, marks, width, retry = buf(m*4), buf(m), floats([bits(-88.)]*m), buf(4)
            for i, j in enumerate(x['header']):put(header, i*4, c.c_int, j)
            lb, ub = floats(x['lb']), floats(x['ub'])
            for off, at in ((0x2048, f), (0x1fd0, rhs), (0x2000, iv), (0x2008, ov),
                            (0x2010, basic), (0x2018, marks), (0x1ff0, width),
                            (0x1f90, lb), (0x1f88, ub), (0x20e8, retry)):
                put(frame, off, c.c_void_p, at)
            put(f, 0x18, c.c_void_p, header)
            for off, at in ((0x470, f), (0x200, rhs), (0x228, iv), (0x220, ov), (0x1e8, basic), (0x280, marks)):
                put(s, off, c.c_void_p, at)
            for off, value in ((0x2068, m), (0x1fa0, x['attempt']), (0x1f98, x['model_status'])):
                put(frame, off, c.c_int, value)
            put(frame, 0x1f50, c.c_int64, m)
            for off, key in ((0x1fc8, 'I'), (0x1f00, 'minus_I'), (0x2070, 'h'), (0x1f08, 'minus_h')):
                c.memmove(frame+off, bytes.fromhex(x[key]), 8)
            put(s, 0x294, c.c_int, x['request'])
            calls, errors = [], []
            @c.CFUNCTYPE(None, c.c_void_p, c.c_void_p, c.c_void_p, c.c_int, c.c_void_p)
            def ftran(ff, inp, out, flag, work):
                try:
                    calls.append(dict(input=dict(count=integer(inp), values=array(ivals, m)),
                                      args=dict(factor=ff==f, input=inp==iv, output=out==ov, flag=flag, work=not work)))
                    if m:c.memmove(ovals, bytes.fromhex(''.join(x['raw'])), m*8)
                except BaseException as exc:errors.append(repr(exc))
            jump(0x1205d0, c.cast(ftran, c.c_void_p).value)
            patch.put(0x164c60, b'\xc3')  # logging only; pinf/counters stored by caller
            stop(0x1c3aed, 100); stop(0x1c400a, 100); stop(0x1c08d5, 101)
            exit_code = native(s, frame, base+(0x1c3872 if case['site']=='pivot' else 0x1c3db4))
            assert not errors, errors
            assert exit_code in (100, 101), exit_code
            result = dict(exit='normal' if exit_code==100 else 'retry', basic=array(basic, m), width=array(width, m),
                          marks=[c.string_at(marks+i, 1).hex() for i in range(m)], pinf=c.string_at(s+0x120, 8).hex(),
                          violations=integer(frame, 0x1f28), retry=integer(retry), request=integer(s, 0x294),
                          calls=calls, preserved=dict(header=[integer(header, i*4) for i in range(m)],
                                                    lb=array(lb, m), ub=array(ub, m), rhs=array(rhs, m)),
                          aliases=dict(rhs_input=rhs==c.c_void_p.from_address(iv+16).value,
                                       raw_basic=basic==c.c_void_p.from_address(ov+16).value))
            patch.restore()
            events.append(dict(case, result=result))
    finally:
        patch.restore()
    return dict(schema=SCHEMA, origin='synthetic-original-slice', substitution=SUBSTITUTION,
                binary_sha256=BINARY_SHA, tool_sha256=sha(Path(__file__)), input_sha256=sha_specs(),
                run_id=str(uuid.uuid4()), pid=os.getpid(),
                environment=dict(machine=platform.machine(), system=platform.system(), python=platform.python_version()), events=events)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--capture', type=Path, required=True)
    args = parser.parse_args()
    assert not args.capture.exists(), '기존 캡처 덮어쓰기 거부'
    record = capture()
    args.capture.write_text(json.dumps(record, ensure_ascii=False, indent=1)+'\n')
    print('원본 재계산 slice 합성', len(record['events']), '사례')
