"""At the last iteration-0 SIMPLEX callback, call Gurobi's own BTRAN (9e288 @ file+0x12de10) on every e_p and dump
the output vector (nnz < 0 = dense; else idx/val in output order) with the DSE weights w, then terminate.
usage: python tools/proto/btran_oracle.py <model.mps.gz> <out.json>  (Gurobi: one process at a time)"""
import ctypes, json, sys
import gurobipy as gp

api = ctypes.pythonapi
api.PyCapsule_GetName.restype = ctypes.c_char_p
api.PyCapsule_GetName.argtypes = [ctypes.py_object]
api.PyCapsule_GetPointer.restype = ctypes.c_void_p
api.PyCapsule_GetPointer.argtypes = [ctypes.py_object, ctypes.c_char_p]
I = lambda a: ctypes.c_int.from_address(a).value
L = lambda a: ctypes.c_long.from_address(a).value
D = lambda a: ctypes.c_double.from_address(a).value

base = next(int(l.split("-")[0], 16) for l in open("/proc/self/maps") if "libgurobi130.so" in l and l.split()[2] == "00000000")
BTRAN = ctypes.CFUNCTYPE(None, ctypes.c_void_p, ctypes.c_void_p, ctypes.c_void_p, ctypes.c_void_p)(base + 0x12de10)


class Vec(ctypes.Structure):
    _fields_ = [("nnz", ctypes.c_int), ("idx", ctypes.c_void_p), ("val", ctypes.c_void_p)]


def run(spx):
    m = I(spx + 0x64)
    LU, P = L(spx + 0x470), L(spx + 0x478)
    w = [x.hex() for x in (ctypes.c_double * m).from_address(L(P + 0x08))]
    bufs = [((ctypes.c_int * (m + 8))(), (ctypes.c_double * (m + 8))()) for _ in range(2)]
    a, b = Vec(0, ctypes.addressof(bufs[0][0]), ctypes.addressof(bufs[0][1])), Vec(0, ctypes.addressof(bufs[1][0]), ctypes.addressof(bufs[1][1]))
    out = []
    for r in range(m):
        ctypes.c_int.from_address(a.idx).value = r
        ctypes.c_double.from_address(a.val).value = 1.0
        a.nnz = 1
        BTRAN(LU, ctypes.addressof(a), ctypes.addressof(b), None)
        k = b.nnz
        if k < 0:
            vals = (ctypes.c_double * m).from_address(b.val)
            out.append({"nnz": k, "val": [x.hex() for x in vals]})
        else:
            out.append({"nnz": k, "idx": list((ctypes.c_int * k).from_address(b.idx)), "val": [x.hex() for x in (ctypes.c_double * k).from_address(b.val)]})
        for v in (a, b):  # restore clean scratch for the next call (BTRAN may swap buffers between in and out)
            ctypes.memset(v.val, 0, 8 * (m + 8))
    lu = {"i104": I(LU + 0x104), "d108": D(LU + 0x108), "d110": D(LU + 0x110), "i140": I(LU + 0x140), "i180": I(LU + 0x180),
          "i100": I(LU + 0x100), "i0xdc": I(LU + 0xdc), "i0xf8": I(LU + 0xf8), "zero": D(LU + 0x10)}
    vec_align = [L(L(spx + o) + 0x10) & 0xf for o in (0x220, 0x228)]
    return {"m": m, "w": w, "btran": out, "lu": lu, "vec_val_align": vec_align, "mode": I(spx + 0x3ec), "w_mode": I(spx + 0x298)}


def main():
    env = gp.Env(params={"OutputFlag": 0, "Threads": 1, "Method": 1, "Presolve": 0})
    model = gp.read(sys.argv[1], env)
    ptr = api.PyCapsule_GetPointer(model._cmodel, api.PyCapsule_GetName(model._cmodel))
    snap = {}

    def cb(mdl, where):
        if where != gp.GRB.Callback.SIMPLEX:
            return
        it = int(mdl.cbGet(gp.GRB.Callback.SPX_ITRCNT))
        spx = L(ptr + 0xe0)
        if it == 0 and spx and L(spx + 0x478) and L(L(spx + 0x478) + 0x08) and L(spx + 0x470):
            snap["d"] = run(spx)  # keep the last iteration-0 callback; later ones see an updated LU
        elif it > 0:
            mdl.terminate()

    model.optimize(cb)
    json.dump(snap.get("d"), open(sys.argv[2], "w"))


main()
