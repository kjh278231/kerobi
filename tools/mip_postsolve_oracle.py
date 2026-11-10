"""Exercise Gurobi 13.0.3's 363164 on synthetic records."""
import ctypes as c
import random
import struct
import sys
from pathlib import Path
import gurobipy  # load the library

base = next(int(line.split('-')[0], 16) for line in open('/proc/self/maps')
            if 'libgurobi130.so' in line and line.split()[2] == '00000000')
uncrush = c.CFUNCTYPE(None, c.c_void_p, c.c_void_p, c.c_void_p, c.c_void_p,
                      c.c_void_p, c.c_void_p)(base + 0x36c2d0)


class Node(c.Structure):
    pass


Node._fields_ = [('kind', c.c_int), ('data', c.c_void_p), ('next', c.POINTER(Node))]


class Scale(c.Structure):
    _fields_ = [('index', c.c_int), ('shift', c.c_double), ('scale', c.c_double)]


class Fix(c.Structure):
    _fields_ = [('index', c.c_int), ('value', c.c_double)]


class Affine(c.Structure):
    _fields_ = [('j', c.c_int), ('k', c.c_int), ('d1', c.c_double),
                ('d2', c.c_double), ('d3', c.c_double)]


class Slack(c.Structure):
    _fields_ = [('row', c.c_int), ('column', c.c_int), ('a', c.c_double)]


class Parallel(c.Structure):
    _fields_ = [('x', c.c_int), ('y', c.c_int), ('lx', c.c_double), ('ly', c.c_double),
                ('ux', c.c_double), ('uy', c.c_double), ('ratio', c.c_double), ('flag', c.c_int)]


def generate_parallel_fixture():
    rng = random.Random(1303)
    mapping = (c.c_int * 3)(-1, -1, 0)
    context = c.create_string_buffer(0x130)
    c.c_int.from_buffer(context, 0x0c).value = 3
    c.c_void_p.from_buffer(context, 0x90).value = c.addressof(mapping)
    c.c_double.from_buffer(context, 0x128).value = 1e30
    rows = []
    for flag in (1, 2):
        for _ in range(500):
            lx = rng.randint(-5, 1)
            ly = rng.randint(-5, 1)
            ux = lx + rng.randint(1, 8)
            uy = ly + rng.randint(1, 8)
            ratio = rng.choice([-4, -3, -2, -1, 1, 2, 3, 4])
            value = rng.randint(-20, 20) + rng.choice([0., 0.00001, -0.00001, 0.00008, -0.00008, 0.25, 0.5])
            par = Parallel(2, 1, lx, ly, ux, uy, ratio, flag)
            node = Node(4, c.addressof(par), None)
            c.c_void_p.from_buffer(context, 0x80).value = c.addressof(node)
            source, result = (c.c_double * 1)(value), (c.c_double * 3)()
            uncrush(None, c.addressof(context), source, result, None, None)
            rows.append(' '.join(map(repr, (flag, lx, ly, ux, uy, ratio, value, result[2], result[1]))))
    path = Path(__file__).resolve().parents[1] / 'bench/oracle/mip_postsolve_parallel.txt'
    path.write_text('\n'.join(rows) + '\n')
    print(f'363164 type 4 flag 1/2: {len(rows)} cases saved to {path}')


def actual_affine_records():
    sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'python'))
    from rsolver import _core

    data = Path(__file__).resolve().parents[2] / 're/bench/data/miplib'
    counts = {}
    for name, kind in [('neos-860300', 10), ('graphdraw-domain', 11)]:
        model = _core.read_model(str(data / f'{name}.mps.gz'))
        records = [record[1] for record in model.mip_post_records([('Threads', 1.0)]) if record[0] == kind]
        for raw in records:
            if kind == 10:
                j, k, d1, d2, d3 = struct.unpack('<iiddd', raw)
                size = max(j, k) + 1
                mapping = (c.c_int * size)(*([-1] * size))
                mapping[j], mapping[k] = 0, 1
                source = (c.c_double * 2)(2.25, -3.5)
                expected = d2 * source[1] + d3 + source[0] * d1
            else:
                row, j, _ = struct.unpack('<iid', raw)
                size = j + 1
                mapping = (c.c_int * size)(*([-1] * size))
                mapping[j] = 0
                source = (c.c_double * 1)(2.25)
                expected = 2.25
            context = c.create_string_buffer(0x130)
            c.c_int.from_buffer(context, 0x0c).value = size
            c.c_void_p.from_buffer(context, 0x90).value = c.addressof(mapping)
            payload = c.create_string_buffer(raw, len(raw))
            node = Node(kind, c.addressof(payload), None)
            c.c_void_p.from_buffer(context, 0x80).value = c.addressof(node)
            result = (c.c_double * size)()
            uncrush(None, c.addressof(context), source, result, None, None)
            assert result[j] == expected, (name, j, result[j], expected)
        counts[name] = len(records)
    assert counts == {'neos-860300': 136, 'graphdraw-domain': 20}, counts
    print('363164 actual model records with synthetic inputs:', counts)


def main():
    if sys.argv[1:] == ['--generate-parallel']:
        generate_parallel_fixture()
        return
    if sys.argv[1:] == ['--actual-records']:
        actual_affine_records()
        return
    # Reduced column 0 maps to original column 2. Linked list is newest first.
    mapping = (c.c_int * 3)(-1, -1, 0)
    context = c.create_string_buffer(0x130)
    c.c_int.from_buffer(context, 0x0c).value = 3
    c.c_void_p.from_buffer(context, 0x90).value = c.addressof(mapping)
    old = Scale(2, 3.0, 4.0)
    fix = Fix(1, -7.0)
    new = Scale(2, -2.0, 5.0)
    tail = Node(9, c.addressof(old), None)
    middle = Node(1, c.addressof(fix), c.pointer(tail))
    head = Node(9, c.addressof(new), c.pointer(middle))
    c.c_void_p.from_buffer(context, 0x80).value = c.addressof(head)
    source = (c.c_double * 1)(2.0)
    result = (c.c_double * 3)()
    uncrush(None, c.addressof(context), source, result, None, None)
    assert list(result) == [0.0, -7.0, 35.0], list(result)
    print('363164 synthetic mixed records:', list(result))
    # Type 10 consumes a fixed column; type 11 does not change primal values.
    affine = Affine(2, 1, 2.0, 3.0, 4.0)
    slack = Slack(0, 2, 7.0)
    end = Node(10, c.addressof(affine), None)
    second = Node(11, c.addressof(slack), c.pointer(end))
    start = Node(1, c.addressof(fix), c.pointer(second))
    c.c_void_p.from_buffer(context, 0x80).value = c.addressof(start)
    result = (c.c_double * 3)()
    uncrush(None, c.addressof(context), source, result, None, None)
    assert list(result) == [0.0, -7.0, -13.0], list(result)
    print('363164 synthetic type 10/11:', list(result))
    c.c_double.from_buffer(context, 0x128).value = 1e30
    for lx, ly, ux, uy, ratio, value, expected in [
        (0., 0., 3., 5., 2., 7., (0., 3.5)),
        (-3., -2., 4., 5., -2., -7., (3., 5.)),
        (-1e30, -1e30, 4., 3., 2., 3., (-3., 3.)),
        (1., 2., 6., 5., 2., 20., (6., 7.)),
    ]:
        par = Parallel(2, 1, lx, ly, ux, uy, ratio, 0)
        node = Node(4, c.addressof(par), None)
        c.c_void_p.from_buffer(context, 0x80).value = c.addressof(node)
        source[0] = value
        result = (c.c_double * 3)()
        uncrush(None, c.addressof(context), source, result, None, None)
        assert (result[2], result[1]) == expected, (list(result), expected)
    print('363164 synthetic type 4 flag 0: 4 cases')


if __name__ == '__main__':
    main()
