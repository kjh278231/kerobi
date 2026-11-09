"""Verify/regenerate native 5e2931/6f45ed fixtures via tools/gurun.sh.

Default verifies stored bytes; --write regenerates the two TSV files.
Only the SHA-256-verified Gurobi 13.0.3 library is supported.
"""
import ctypes as c
import hashlib
import itertools
import sys
from pathlib import Path

import gurobipy as gp

ROOT = Path(__file__).resolve().parents[1]
HASH = '8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c'


def main():
    mapping = next(line for line in open('/proc/self/maps')
                   if 'libgurobi130.so' in line and line.split()[2] == '00000000')
    if gp.gurobi.version() != (13, 0, 3) or hashlib.sha256(Path(mapping.split()[-1]).read_bytes()).hexdigest() != HASH:
        raise SystemExit('unsupported library: requires the verified Gurobi 13.0.3 build')
    base = int(mapping.split('-')[0], 16)
    cutoff = c.CFUNCTYPE(c.c_double, c.c_double, c.c_void_p)(base + 0x5b9090)
    step = c.CFUNCTYPE(c.c_int, c.c_void_p)(base + 0x508120)
    api = c.pythonapi
    api.PyCapsule_GetName.argtypes, api.PyCapsule_GetName.restype = [c.py_object], c.c_char_p
    api.PyCapsule_GetPointer.argtypes, api.PyCapsule_GetPointer.restype = [c.py_object, c.c_char_p], c.c_void_p
    model = gp.Model()
    model.Params.OutputFlag = 0
    ptr = api.PyCapsule_GetPointer(model._cmodel, api.PyCapsule_GetName(model._cmodel))
    context = c.create_string_buffer(0x660)
    address = c.addressof(context)
    c.c_void_p.from_address(address + 8).value = ptr
    results = ['incumbent\tuser\tstep\toffset\tintfeastol\topttol\tcutoff\n']
    cases = itertools.product((0.0, 1.0, -5.0, 6.964, 1e8, 1e100),
                              (1e100, -10.0, 0.0, 1.0, 20.0),
                              (0.0, -1e100, 1.0, 0.25, 3.0), (0.0, 7.0, -2.25))
    for incumbent, user, lattice, offset in cases:
        model.ObjCon = offset
        model.Params.Cutoff = user
        model.update()
        c.c_double.from_address(address + 0x40).value = lattice
        for intfeastol, opttol in ((1e-5, 1e-6), (1e-9, 1e-2)):
            model.Params.IntFeasTol, model.Params.OptimalityTol = intfeastol, opttol
            value = cutoff(incumbent, address)
            results.append('\t'.join(map(repr, (incumbent, user, lattice, offset, intfeastol, opttol, value))) + '\n')
    tables = {'cutoff.tsv': ''.join(results)}
    model.dispose()
    results = ['coefficients\ttypes\tintfeastol\tobjgcd\tstep\n']
    for coefficients, types in [([2.0, 3.0], 'II'), ([0.25, 0.5], 'BI'),
                                ([0.3, 0.5], 'II'), ([2.0, 3.0], 'IC'),
                                ([2.0, 0.0], 'IC'), ([0.0, 0.0], 'BC'),
                                ([1e-11, 1.0], 'CI'), ([1e-9, 1.0], 'CI'),
                                ([1e-6, 1.0], 'II'), ([2e9, 3.0], 'II'),
                                ([1.4142135623730951, 2.0], 'II')]:
        model = gp.Model()
        model.Params.OutputFlag = 0
        for coefficient, kind in zip(coefficients, types):
            model.addVar(obj=coefficient, vtype=kind)
        model.update()
        ptr = api.PyCapsule_GetPointer(model._cmodel, api.PyCapsule_GetName(model._cmodel))
        c.c_void_p.from_address(address + 8).value = ptr
        kinds = c.create_string_buffer(types.encode())
        c.c_void_p.from_address(address + 0x650).value = c.addressof(kinds)
        for intfeastol, objgcd in itertools.product((1e-5, 1e-9), (0, 1)):
            model.Params.IntFeasTol = intfeastol
            model.setParam('GURO_PAR_OBJGCD', objgcd)
            assert step(address) == 0
            value = c.c_double.from_address(address + 0x40).value
            results.append(f'{",".join(map(repr, coefficients))}\t{types}\t{intfeastol!r}\t{objgcd}\t{value!r}\n')
        model.dispose()
    tables['step.tsv'] = ''.join(results)
    destination = ROOT / 'bench/oracle/mip_cutoff'
    for name, contents in tables.items():
        if '--write' in sys.argv:
            destination.mkdir(parents=True, exist_ok=True)
            (destination / name).write_text(contents)
        else:
            assert (destination / name).read_text() == contents, name
        print(f'{name}: {len(contents.splitlines()) - 1} native cases verified')


if __name__ == '__main__':
    main()
