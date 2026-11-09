"""Gurobi 13.0 native 919394 row-boundary fixture; run through tools/gurun.sh."""
import ctypes as c
import hashlib
from pathlib import Path

import gurobipy as gp


def main():
    m = gp.Model()
    m.Params.OutputFlag = 0
    x = m.addVars(3, vtype=gp.GRB.BINARY)
    m.addConstr(x[0] + x[1] >= 2)
    m.addConstr(x[2] >= 1)
    m.update()
    api = c.pythonapi
    api.PyCapsule_GetName.argtypes = [c.py_object]
    api.PyCapsule_GetName.restype = c.c_char_p
    api.PyCapsule_GetPointer.argtypes = [c.py_object, c.c_char_p]
    api.PyCapsule_GetPointer.restype = c.c_void_p
    model = api.PyCapsule_GetPointer(m._cmodel, api.PyCapsule_GetName(m._cmodel))
    mapping = next(line for line in open('/proc/self/maps')
                   if 'libgurobi130.so' in line and line.split()[2] == '00000000')
    base = int(mapping.split('-')[0], 16)
    digest = hashlib.sha256(Path(mapping.split()[-1]).read_bytes()).hexdigest()
    if gp.gurobi.version() != (13, 0, 3) or digest != '8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c':
        raise SystemExit('unsupported Gurobi library: this fixture requires the verified 13.0.3 build')
    func = c.CFUNCTYPE(c.c_int, c.c_double, c.c_double, *([c.c_void_p] * 9), c.c_int, c.c_void_p)(base + 0x74a5e0)
    array = c.c_double * 3
    for initial_work in (0, 1000):
        candidate, reference, lb, ub = array(0, 0, 0), array(0, 0, 0), array(0, 0, 0), array(1, 1, 1)
        vtype, rng = c.create_string_buffer(b'BBB'), c.c_uint64(20)
        remaining, work = c.c_int(-1), c.c_double(initial_work)
        result = func(1e100, 20, model, c.byref(rng), candidate, reference, vtype, lb, ub,
                      None, c.byref(remaining), 200, c.byref(work))
        assert (result, list(candidate), remaining.value, work.value) == (0, [1, 1, 0], 1, initial_work + 117)
        print(f'initial work {initial_work}: x={list(candidate)}, remaining={remaining.value}, work={work.value}')


if __name__ == "__main__":
    main()
