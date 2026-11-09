"""Symmetry oracle (docs/m1/m1_presolve_symmetry.md): patch `390989`'s call to `5577ba` (pass-0 symmetry detection)
and `5577ba`'s call to `551cc0` so Python thunks see what goes in and out, then run Model.presolve() at defaults.
usage: python tools/symmetry_oracle.py <out.json> [--param Name=Value ...] [--relax DIR] [--presolved DIR]
       [model.mps(.gz) | instance name ...]
       (Gurobi: one process at a time; wrap in tools/gurun.sh)
Per call: state sizes and removal counts, work (*state+0x458) before/after each call, 5577ba's return and local_f4,
551cc0's return and its out parameters (param_24, param_27, param_30), and how often each callee site inside
`551cc0` ran (which stages a model reaches). Default instances: the 64 Netlib LPs."""
import argparse, ctypes, json, sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

I = lambda a: ctypes.c_int.from_address(a).value
L = lambda a: ctypes.c_long.from_address(a).value
D = lambda a: ctypes.c_double.from_address(a).value
libc = ctypes.CDLL(None)
libc.mmap.restype = ctypes.c_void_p
libc.mmap.argtypes = [ctypes.c_void_p, ctypes.c_size_t, ctypes.c_int, ctypes.c_int, ctypes.c_int, ctypes.c_long]
libc.mprotect.argtypes = [ctypes.c_void_p, ctypes.c_size_t, ctypes.c_int]
import gurobipy as gp  # noqa: E402  (loads libgurobi130.so)

base = next(int(l.split("-")[0], 16) for l in open("/proc/self/maps") if "libgurobi130.so" in l and l.split()[2] == "00000000")
cave = None  # a page within +-2GB of the library for the jump stubs (as in uncrush_oracle.py)
for k in range(1, 64):
    p = libc.mmap(base - k * 0x100000, 4096, 7, 0x22 | 0x100000, -1, 0)  # RWX, PRIVATE|ANON|FIXED_NOREPLACE
    if p and p != ctypes.c_void_p(-1).value and abs(p - base) < 0x7000_0000:
        cave = p
        break
assert cave, "no stub page near libgurobi"
keep, used = [], [0]


def patch_call(site_off, addr):
    """Redirect the `call rel32` at file offset site_off to absolute address addr (movabs/jmp stub; stack untouched)."""
    site = base + site_off
    assert ctypes.c_ubyte.from_address(site).value == 0xE8
    stub = cave + used[0]
    used[0] += 16
    code = b"\x48\xb8" + addr.to_bytes(8, "little") + b"\xff\xe0"
    ctypes.memmove(stub, code, len(code))
    assert libc.mprotect(site & ~0xFFF, 0x2000, 7) == 0
    ctypes.c_int32.from_address(site + 1).value = stub - (site + 5)


def fn_addr(f):
    keep.append(f)
    return ctypes.cast(f, ctypes.c_void_p).value

# counted call sites inside 551cc0 / 5577ba (file offsets from objdump), label = callee
SITES = {0x4A148A: "549df5_rows", 0x4A19EE: "549df5_cols", 0x4A17AA: "ab6a5f_type", 0x4A1BB6: "ab9789_q",
         0x4A1FE9: "ab9789_lb", 0x4A241A: "ab9789_ub", 0x4A3431: "790e3", 0x4A37ED: "ab6a5f_2", 0x4A3CED: "54ad3b_1",
         0x4A456F: "54ad3b_2", 0x4A4441: "a7b4d6_1", 0x4A4451: "a7b4d6_2", 0x4A4F35: "54e0c4_1", 0x4A5133: "54e5eb_1",
         0x4A52B5: "54e0c4_2", 0x4A54A9: "54e5eb_2", 0x4A5745: "5507cb", 0x4A59E6: "54d420_1", 0x4A7879: "54d420_2",
         0x4A5C4A: "a9d3ef_1", 0x4A5ECB: "a9d3ef_2", 0x4A69A9: "ab6a5f_3", 0x4A6F73: "ab6a5f_4"}
TARGET = {"549df5": 0x4AB270, "ab6a5f": 0x8C4D20, "ab9789": 0x8C8B70, "790e3": 0x10A2A0, "54ad3b": 0x4AACA0,
          "a7b4d6": 0x88A870, "54e0c4": 0x4A8F20, "54e5eb": 0x4A7CA0, "5507cb": 0x4A91C0, "54d420": 0x4A82E0,
          "a9d3ef": 0x8B1730}
counters = (ctypes.c_long * len(SITES))()


def count_site(site_off, k, target_off):
    """`call` at site_off goes through a register-preserving stub that bumps counters[k], then jumps to the callee."""
    stub = libc.mmap(None, 64, 7, 0x22, -1, 0)
    code = (b"\x50\x48\xb8" + (ctypes.addressof(counters) + 8 * k).to_bytes(8, "little")  # push rax; movabs rax, &c
            + b"\x48\xff\x00\x58"  # inc qword [rax]; pop rax
            + b"\xff\x25\x00\x00\x00\x00" + (base + target_off).to_bytes(8, "little"))  # jmp [rip]; dq target
    ctypes.memmove(stub, code, len(code))
    patch_call(site_off, stub)


cur = {}
T551 = ctypes.CFUNCTYPE(ctypes.c_int, ctypes.c_double, ctypes.c_double, *[ctypes.c_void_p] * 29)
orig_551 = T551(base + 0x4A0900)


def h551(*a):
    st, work = a[3], a[30]
    w0 = D(work) if work else None
    r = orig_551(*a)
    o = lambda p: I(p) if p else None
    cur["c551"] = {"ret": r, "p24": o(a[23]), "p27": o(a[26]), "p30": o(a[29]), "work": (D(work) - w0) if work else None,
                   "args": [a[0], a[1], a[4] & 0xFFFFFFFF, a[5] & 0xFFFFFFFF, a[21] & 0xFFFFFFFF]}
    return r


T577 = ctypes.CFUNCTYPE(ctypes.c_int, ctypes.c_double, ctypes.c_double, ctypes.c_void_p, ctypes.c_void_p, ctypes.c_int,
                        ctypes.c_int, ctypes.c_void_p)
orig_577 = T577(base + 0x49FDA0)
calls = []


def h577(f1, lim, env, st, nodes, p6, out):
    ctypes.memset(counters, 0, ctypes.sizeof(counters))
    cur.clear()
    wp = L(st + 0x458)
    e = {"n": I(st + 0xC), "m": I(st + 0x10), "cols_removed": I(st + 0x408), "rows_removed": I(st + 0x404),
         "work_limit": lim, "nodes": nodes, "budget": D(st + 0x1F0), "work0": D(wp) if wp else None}
    r = orig_577(f1, lim, env, st, nodes, p6, out)
    e.update(ret=r, f4=I(out), work1=D(wp) if wp else None, c551=cur.get("c551"),
             work_hex=[e["work0"].hex(), D(wp).hex()] if wp else None,
             sites={lab: counters[k] for k, lab in enumerate(SITES.values()) if counters[k]})
    calls.append(e)
    return r


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("out")
    ap.add_argument("--param", action="append", default=[])
    ap.add_argument("--relax", metavar="DIR", help="presolve the LP relaxation instead, written to DIR/<name>.mps")
    ap.add_argument("--presolved", metavar="DIR", help="also write Gurobi's presolved model to DIR/<name>.mps")
    ap.add_argument("models", nargs="*")
    a = ap.parse_args()
    from parity import instances
    for k, (site, lab) in enumerate(SITES.items()):
        count_site(site, k, TARGET[lab.split("_")[0]])
    patch_call(0x4A0555, fn_addr(T551(h551)))
    patch_call(0x38F1F0, fn_addr(T577(h577)))
    params = {k: float(v) if "." in v or "e" in v else int(v) for k, v in (p.split("=", 1) for p in a.param)}
    insts = instances()
    models = a.models or [n for n, p in insts.items() if p.parent.name == "netlib"]
    res = {}
    for name in models:
        path = insts[name] if name in insts else Path(name)
        env = gp.Env(params={"OutputFlag": 0, "Threads": 1, "WorkLimit": 60, **params})
        m = gp.read(str(path), env)
        if a.relax:
            r = m.relax()
            m.dispose()
            m = r
            Path(a.relax).mkdir(parents=True, exist_ok=True)
            m.write(str(Path(a.relax) / (path.name.split(".")[0] + ".mps")))
        calls.clear()
        q = m.presolve()
        if a.presolved:
            Path(a.presolved).mkdir(parents=True, exist_ok=True)
            q.write(str(Path(a.presolved) / (path.name.split(".")[0] + ".mps")))
        res[path.name.split(".")[0]] = {"calls": list(calls), "presolved": [q.NumConstrs, q.NumVars, q.NumNZs]}
        q.dispose(); m.dispose(); env.dispose()
        c = res[path.name.split(".")[0]]["calls"]
        print(path.name.split(".")[0], [(x["ret"], x["f4"], x["n"] - x["cols_removed"], x["m"] - x["rows_removed"],
                                         x["c551"] and x["c551"]["work"], x["sites"]) for x in c], flush=True)
    json.dump(res, open(a.out, "w"), indent=1)


if __name__ == "__main__":
    main()
