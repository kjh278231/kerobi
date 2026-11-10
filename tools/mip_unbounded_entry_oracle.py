#!/usr/bin/env python3
"""Call only the audited, call-free 600d25 paths on synthetic state (13.0.3)."""
import argparse
import ctypes as c
import hashlib
import json
from pathlib import Path
import struct
import subprocess
import sys


BINARY_SHA256 = '8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c'
ENTRY = 0x518170
# name, LP status, root relaxation status, fractional count, node status, counter delta
CASES = [
    ('linear_unbounded_root_unbounded', 5, 5, 0, 3, 1),
    ('linear_unbounded_root_optimal', 5, 2, 0, 3, 0),
    ('linear_unbounded_root_unknown', 5, -1, 0, 3, 1),
    ('linear_unbounded_root_other', 5, 4, 0, 3, 1),
    ('optimal_integral', 2, 2, 0, 1, 0),
    ('optimal_fractional', 2, 2, 1, 2, 0),
    ('infeasible', 3, 2, 0, 4, 0),
    ('inf_or_unbounded', 4, 5, 0, 6, 0),
    ('cutoff', 6, 2, 0, 5, 0),
    ('iteration_limit', 7, 2, 0, 3, 1),
    ('time_limit', 9, 2, 0, 7, 0),
    ('numeric_root_optimal', 12, 2, 0, 3, 1),
    ('suboptimal_root_optimal', 13, 2, 0, 3, 1),
]


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def worker():
    import gurobipy as gp

    if gp.gurobi.version() != (13, 0, 3):
        raise RuntimeError('Gurobi 13.0.3 required')
    with open('/proc/self/maps') as maps:
        mapping = next(line.split() for line in maps
                       if 'libgurobi130.so' in line and line.split()[2] == '00000000')
    binary = Path(mapping[-1])
    if digest(binary) != BINARY_SHA256:
        raise RuntimeError('binary hash mismatch; refusing internal address call')
    base = int(mapping[0].split('-')[0], 16)
    # ELF executable LOAD has p_vaddr == p_offset. Ghidra adds 0x100000.
    if c.string_at(base + ENTRY, 8).hex() != '4154415541564157':
        raise RuntimeError('entry bytes mismatch')
    table = c.string_at(base + 0x2a6c440, 16)
    if table.hex() != '594238270d6363006300636363630000':
        raise RuntimeError('status jump table mismatch')
    fn = c.CFUNCTYPE(c.c_int, c.c_void_p, c.c_void_p, c.c_void_p, c.c_void_p)(base + ENTRY)
    rows = []
    for name, lp_status, root_status, fractional, _, _ in CASES:
        # These branches read no model arrays, timers, allocator or solver state.
        buffers = {key: c.create_string_buffer(size) for key, size in
                   [('node', 0x20), ('info', 0x90), ('mip', 0x798),
                    ('model', 0xf0), ('data', 0x260), ('top', 0xd8), ('lp', 0xc0)]}

        def put(key, offset, fmt, value):
            struct.pack_into(fmt, buffers[key], offset, value)

        def link(key, offset, target):
            put(key, offset, '<Q', c.addressof(buffers[target]))

        link('node', 0x18, 'info')
        link('info', 0, 'mip')
        link('mip', 8, 'model')
        link('mip', 0x620, 'top')
        link('model', 0xe8, 'data')
        # mip+0x628 stays null, so infeasible/cutoff skip root-only callbacks.
        put('info', 0x28, '<i', 99)
        put('mip', 0xa8, '<i', root_status)
        put('lp', 0x38, '<i', lp_status)
        put('lp', 0x80, '<i', fractional)
        put('top', 0xd4, '<i', 17)
        work = c.c_double(123.25)
        before = {key: bytes(value) for key, value in buffers.items()}
        rc = fn(c.addressof(buffers['node']), c.addressof(buffers['lp']), None, c.byref(work))
        status = struct.unpack_from('<i', buffers['info'], 0x28)[0]
        counter = struct.unpack_from('<i', buffers['top'], 0xd4)[0]
        expected_buffers = {key: bytearray(value) for key, value in before.items()}
        struct.pack_into('<i', expected_buffers['info'], 0x28, status)
        struct.pack_into('<i', expected_buffers['top'], 0xd4, counter)
        rows.append(dict(name=name, lp_status=lp_status, root_status=root_status,
                         fractional=fractional, node_status=status, counter=counter,
                         rc=rc, work_hex=struct.pack('<d', work.value).hex(),
                         preserved=all(bytes(value) == expected_buffers[key]
                                       for key, value in buffers.items())))
    return dict(schema=1, kind='synthetic-call-free-state', gurobi_version=[13, 0, 3],
                binary_sha256=BINARY_SHA256, tool_sha256=digest(__file__),
                entry_elf=hex(ENTRY), jump_table_hex=table.hex(), cases=rows)


def check_capture(capture):
    assert capture['schema'] == 1
    assert capture['kind'] == 'synthetic-call-free-state'
    assert capture['gurobi_version'] == [13, 0, 3]
    assert capture['binary_sha256'] == BINARY_SHA256
    assert capture['entry_elf'] == hex(ENTRY)
    assert capture['jump_table_hex'] == '594238270d6363006300636363630000'
    assert len(capture['cases']) == len(CASES)
    for row, (name, lp, root, fractional, status, delta) in zip(capture['cases'], CASES):
        expected = dict(name=name, lp_status=lp, root_status=root, fractional=fractional,
                        node_status=status, counter=17 + delta, rc=0,
                        work_hex=struct.pack('<d', 123.25).hex(), preserved=True)
        assert row == expected, (name, expected, row)


def check_document(document):
    assert document['schema'] == 1
    assert len(document['runs']) == 3
    for capture in document['runs']:
        check_capture(capture)
    assert document['runs'][0] == document['runs'][1] == document['runs'][2]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    group = parser.add_mutually_exclusive_group(required=True)
    group.add_argument('--output', type=Path)
    group.add_argument('--check', type=Path)
    group.add_argument('--worker', action='store_true', help=argparse.SUPPRESS)
    args = parser.parse_args()
    if args.worker:
        print(json.dumps(worker(), sort_keys=True))
        return
    if args.check:
        check_document(json.loads(args.check.read_text()))
        print('PASS: 3 independent runs x 13 synthetic states')
        return
    runs = []
    for _ in range(3):
        result = subprocess.run([sys.executable, str(Path(__file__).resolve()), '--worker'],
                                capture_output=True, text=True, timeout=30)
        if result.returncode:
            raise RuntimeError(f'isolated worker exit {result.returncode}: {result.stderr}')
        runs.append(json.loads(result.stdout))
    document = dict(schema=1, runs=runs)
    # Keep failed observations for diagnosis as well as successful captures.
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(document, indent=2, sort_keys=True) + '\n')
    check_document(document)
    print(f'PASS: 3 independent runs x 13 synthetic states; {args.output}')


if __name__ == '__main__':
    main()
