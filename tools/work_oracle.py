"""Presolve work oracle: Gurobi's deterministic work counter (*state+0x458) when LP presolve (390989) reaches model
generation (3b0917), with the budget fields the work-limit check 3ba9de reads (base +0x450, budget +0x1f0) and the
work multiplier +0x448. Patches the `call 3b0917` at file offset 0x38ed1d (same stub technique as uncrush_oracle.py).
usage: python tools/work_oracle.py <out.json> [--param Name=Value ...] [name ...]   (Gurobi: one process at a time)
"""
import argparse, ctypes, json, sys, tempfile
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from parity import instances  # noqa: E402

L = lambda a: ctypes.c_long.from_address(a).value
D = lambda a: ctypes.c_double.from_address(a).value
I = lambda a: ctypes.c_int.from_address(a).value


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("out")
    ap.add_argument("--param", action="append", default=[])
    ap.add_argument("names", nargs="*")
    a = ap.parse_args()
    import gurobipy as gp
    libc = ctypes.CDLL(None, use_errno=True)
    libc.mmap.restype = ctypes.c_void_p
    libc.mmap.argtypes = [ctypes.c_void_p, ctypes.c_size_t, ctypes.c_int, ctypes.c_int, ctypes.c_int, ctypes.c_long]
    libc.mprotect.argtypes = [ctypes.c_void_p, ctypes.c_size_t, ctypes.c_int]
    base = next(int(l.split("-")[0], 16) for l in open("/proc/self/maps") if "libgurobi130.so" in l and l.split()[2] == "00000000")
    cave = None
    for k in range(1, 64):
        p = libc.mmap(base - k * 0x100000, 4096, 7, 0x22 | 0x100000, -1, 0)
        if p and p != ctypes.c_void_p(-1).value and abs(p - base) < 0x7000_0000:
            cave = p
            break
    assert cave, "no stub page near libgurobi"
    T = ctypes.CFUNCTYPE(ctypes.c_int, ctypes.c_void_p, ctypes.c_void_p, ctypes.c_int)
    orig = T(base + 0x3A1440)
    seen = []

    def hook(model, st, flag):
        w = L(st + 0x458)
        seen.append({"work": D(w).hex() if w else None, "base": D(st + 0x450).hex(), "budget": D(st + 0x1F0).hex(),
                     "mult": D(st + 0x448).hex(), "nnz": L(st + 0xD8), "prelimit": I(st + 0x1DC)})
        return orig(model, st, flag)
    thunk = T(hook)
    site = base + 0x38ED1D
    assert ctypes.c_ubyte.from_address(site).value == 0xE8
    code = b"\x48\xb8" + ctypes.c_uint64(ctypes.cast(thunk, ctypes.c_void_p).value).value.to_bytes(8, "little") + b"\xff\xe0"
    ctypes.memmove(cave, code, len(code))
    assert libc.mprotect(site & ~0xFFF, 0x2000, 7) == 0
    ctypes.c_int32.from_address(site + 1).value = cave - (site + 5)

    params = {k: float(v) if "." in v or "e" in v else int(v) for k, v in (p.split("=", 1) for p in a.param)}
    insts = instances()
    names = a.names or [n for n, p in insts.items() if p.parent.name == "netlib"]
    out = {}
    with tempfile.TemporaryDirectory() as tmp:
        for name in names:
            env = gp.Env(params={"LogToConsole": 0, "LogFile": str(Path(tmp) / "g.log"), "Threads": 1, "WorkLimit": 60, **params})
            m = gp.read(str(insts[name]), env)
            seen.clear()
            q = m.presolve()
            out[name] = seen[-1] if seen else None
            q.dispose(); m.dispose(); env.dispose()
    json.dump(out, open(a.out, "w"), indent=0)
    print(f"{len(names)} models -> {a.out}")


if __name__ == "__main__":
    main()
