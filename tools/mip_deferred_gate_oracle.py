#!/usr/bin/env python3
"""Check audited call-free node setter/requeue exits in Gurobi 13.0.3."""
import argparse
import copy
import ctypes as c
import json
from pathlib import Path
import struct
import subprocess
import sys

from mip_unbounded_entry_oracle import BINARY_SHA256, digest


STATUSES = (0, 1, 2, 3, 4, 5, 6, 9, 11)
ENTRIES = {'setter': 0x5180f0, 'gate': 0x517690, 'branchable': 0x519b10}
PREFIXES = {'setter': '4154488b47188970', 'gate': '4154415541564157',
            'branchable': '415753564989ff49'}


def worker():
    import gurobipy as gp

    if gp.gurobi.version() != (13, 0, 3):
        raise RuntimeError('Gurobi 13.0.3 required')
    with open('/proc/self/maps') as maps:
        mapping = next(line.split() for line in maps
                       if 'libgurobi130.so' in line and line.split()[2] == '00000000')
    if digest(mapping[-1]) != BINARY_SHA256:
        raise RuntimeError('binary hash mismatch; refusing internal address call')
    base = int(mapping[0].split('-')[0], 16)
    for name, offset in ENTRIES.items():
        if c.string_at(base + offset, 8).hex() != PREFIXES[name]:
            raise RuntimeError(f'{name}: entry bytes mismatch')
    setter = c.CFUNCTYPE(None, c.c_void_p, c.c_uint)(base + ENTRIES['setter'])
    gate = c.CFUNCTYPE(c.c_int, c.c_void_p, c.c_void_p, c.c_void_p)(base + ENTRIES['gate'])
    branchable = c.CFUNCTYPE(c.c_bool, c.c_void_p)(base + ENTRIES['branchable'])
    rows = []
    for is_root in (False, True):
        for status in STATUSES:
            # No queue, allocator, lock or actual LP state is reached in these exits.
            buffers = {key: c.create_string_buffer(size) for key, size in
                       [('node', 0x40), ('other_node', 0x40), ('info', 0xc0),
                        ('mip', 0x798), ('model', 0x108), ('env', 8),
                        ('top', 0xd8), ('holder', 8)]}

            def put(key, offset, fmt, value):
                struct.pack_into(fmt, buffers[key], offset, value)

            def link(key, offset, target):
                put(key, offset, '<Q', c.addressof(buffers[target]))

            link('holder', 0, 'node')
            link('node', 0x18, 'info')
            link('info', 0, 'mip')
            link('mip', 8, 'model')
            link('model', 0x100, 'env')
            link('mip', 0x620, 'top')
            link('mip', 0x628, 'mip')
            link('mip', 0x790, 'node' if is_root else 'other_node')
            put('info', 0x28, '<i', status)
            put('mip', 0, '<i', 1)
            put('mip', 0xa8, '<i', 2)
            put('top', 0xd4, '<i', 17)
            before = {key: bytes(value) for key, value in buffers.items()}
            setter(c.addressof(buffers['node']), 3)
            setter_status = struct.unpack_from('<i', buffers['info'], 0x28)[0]
            expected = {key: bytearray(value) for key, value in before.items()}
            struct.pack_into('<i', expected['info'], 0x28, 3)
            setter_preserved = all(bytes(value) == expected[key]
                                   for key, value in buffers.items())
            put('info', 0x28, '<i', status)
            # Status 1 reaches predicates requiring a real model; leave that case static.
            eligible = None if status == 1 else branchable(c.addressof(buffers['node']))
            rc = gate(c.addressof(buffers['holder']), c.addressof(buffers['mip']), None)
            rows.append(dict(status=status, is_root=is_root, setter_status=setter_status,
                             setter_preserved=setter_preserved, branchable=eligible, gate_rc=rc,
                             gate_preserved=all(bytes(value) == before[key]
                                                for key, value in buffers.items())))
    return dict(schema=1, kind='synthetic-call-free-deferred-gates',
                gurobi_version=[13, 0, 3], binary_sha256=BINARY_SHA256,
                tool_sha256=digest(__file__),
                dependency_sha256=digest(Path(__file__).with_name('mip_unbounded_entry_oracle.py')),
                entries_elf={name: hex(offset) for name, offset in ENTRIES.items()},
                entry_bytes=PREFIXES, cases=rows)


def check_document(doc):
    assert doc['schema'] == 1
    assert len(doc['runs']) == 3
    for run_index, run in enumerate(doc['runs']):
        assert run['schema'] == 1
        assert run['kind'] == 'synthetic-call-free-deferred-gates'
        assert run['gurobi_version'] == [13, 0, 3]
        assert run['binary_sha256'] == BINARY_SHA256
        assert run['entries_elf'] == {name: hex(offset) for name, offset in ENTRIES.items()}
        assert run['entry_bytes'] == PREFIXES
        assert len(run['cases']) == 18
        for index, (is_root, status) in enumerate((root, status)
                                                for root in (False, True) for status in STATUSES):
            expected = dict(status=status, is_root=is_root, setter_status=3,
                            setter_preserved=True, branchable=None if status == 1 else status in (2, 3),
                            gate_rc=0, gate_preserved=True)
            actual = run['cases'][index]
            for field in expected:
                assert actual[field] == expected[field], (
                    f'run={run_index} case={index} root={is_root} status={status} '
                    f'field={field} expected={expected[field]!r} actual={actual[field]!r} rule=exact')
            assert actual.keys() == expected.keys(), (run_index, index, 'case fields')
    assert doc['runs'][0] == doc['runs'][1] == doc['runs'][2], 'independent runs differ'


def self_test(doc):
    check_document(doc)
    for field, value in [('setter_status', 11), ('branchable', False),
                         ('gate_rc', 1), ('gate_preserved', False)]:
        bad = copy.deepcopy(doc)
        bad['runs'][0]['cases'][3][field] = value
        try:
            check_document(bad)
        except AssertionError as error:
            assert f'field={field}' in str(error), str(error)
            print(f'REJECT: {error}')
        else:
            raise AssertionError(f'failed to detect mutation: {field}')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    group = parser.add_mutually_exclusive_group(required=True)
    group.add_argument('--worker', action='store_true', help=argparse.SUPPRESS)
    group.add_argument('--output', type=Path)
    group.add_argument('--check', type=Path)
    group.add_argument('--self-test', type=Path)
    args = parser.parse_args()
    if args.worker:
        print(json.dumps(worker(), sort_keys=True))
        return
    if args.check or args.self_test:
        doc = json.loads((args.check or args.self_test).read_text())
        if args.self_test:
            self_test(doc)
        else:
            check_document(doc)
        print('PASS: 3 runs x (18 setters, 18 requeue exits, 16 branch predicates)')
        return
    runs = []
    for _ in range(3):
        result = subprocess.run([sys.executable, str(Path(__file__).resolve()), '--worker'],
                                capture_output=True, text=True, timeout=30)
        if result.returncode:
            raise RuntimeError(f'isolated worker exit {result.returncode}: {result.stderr}')
        runs.append(json.loads(result.stdout))
    doc = dict(schema=1, runs=runs)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(doc, sort_keys=True, indent=2) + '\n')
    check_document(doc)
    print(f'PASS: 3 independent runs x 18 synthetic cases; {args.output}')


if __name__ == '__main__':
    main()
