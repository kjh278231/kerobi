"""Uncrush oracle: patch Gurobi's call to 363daa (LP simplex-path postsolve, in 0c3d0d) so a Python thunk sees its
inputs and outputs, then compare the uncrushed arrays with the final model attributes.
usage: python uncrush_oracle.py <model.mps.gz> <out.json> [Param=Value ...]   (Gurobi: one process at a time)
Captured: post record list (type, data ptr, val-array alignment), presolved spx arrays (x/dj/status, n_pre+m_pre)
before and after the call, the 363daa outputs (basis int[n+m], x[n+m], dual[n+m]) and final X/Slack/RC/Pi/VBasis/CBasis.
Also counts hits of the 36578f-path calls (365159 x2, 363164) to show which path ran."""
import ctypes, json, sys
import gurobipy as gp

I = lambda a: ctypes.c_int.from_address(a).value
L = lambda a: ctypes.c_long.from_address(a).value
D = lambda a: ctypes.c_double.from_address(a).value
dbl = lambda p, k: [x.hex() for x in (ctypes.c_double * k).from_address(p)] if p and k > 0 else []
ints = lambda p, k: list((ctypes.c_int * k).from_address(p)) if p and k > 0 else []

libc = ctypes.CDLL(None, use_errno=True)
libc.mmap.restype = ctypes.c_void_p
libc.mmap.argtypes = [ctypes.c_void_p, ctypes.c_size_t, ctypes.c_int, ctypes.c_int, ctypes.c_int, ctypes.c_long]
libc.mprotect.argtypes = [ctypes.c_void_p, ctypes.c_size_t, ctypes.c_int]
base = next(int(l.split("-")[0], 16) for l in open("/proc/self/maps") if "libgurobi130.so" in l and l.split()[2] == "00000000")

# a page within +-2GB of the library for the jump stubs
cave = None
for k in range(1, 64):
    want = base - k * 0x100000
    p = libc.mmap(want, 4096, 7, 0x22 | 0x100000, -1, 0)  # RWX, PRIVATE|ANON|FIXED_NOREPLACE
    if p and p != ctypes.c_void_p(-1).value and abs(p - base) < 0x7000_0000:
        cave = p
        break
assert cave, "no stub page near libgurobi"
keep, used = [], [0]


def patch_call(site_off, thunk):
    """Redirect the `call rel32` at file offset site_off to thunk (via a movabs/jmp stub; the stack is untouched)."""
    site = base + site_off
    assert ctypes.c_ubyte.from_address(site).value == 0xE8
    stub = cave + used[0]
    used[0] += 16
    code = b"\x48\xb8" + ctypes.c_uint64(ctypes.cast(thunk, ctypes.c_void_p).value).value.to_bytes(8, "little") + b"\xff\xe0"
    ctypes.memmove(stub, code, len(code))
    pg = site & ~0xFFF
    assert libc.mprotect(pg, 0x2000, 7) == 0
    ctypes.c_int32.from_address(site + 1).value = stub - (site + 5)
    keep.append(thunk)


cap = {"hits": {"365159@372458": 0, "365159@372e6e": 0, "363164@3729d4": 0}, "daa": []}

T363daa = ctypes.CFUNCTYPE(None, *[ctypes.c_void_p] * 3, ctypes.c_int, ctypes.c_int, *[ctypes.c_void_p] * 5)
orig_daa = T363daa(base + 0x36fb00)


def records(post):
    out, r = [], L(post + 0x80)
    while r:
        t, d = I(r), L(r + 8)
        e = {"type": t, "data": d & 0xF}
        if t == 1:  # j, v, obj, status @+0x18, n @+0x1c, ind @+0x20, val @+0x28
            e.update(j=I(d), v=D(d + 8).hex(), obj=D(d + 16).hex(), status=I(d + 0x18), n=I(d + 0x1c), val_al=L(d + 0x28) & 0xF)
        elif t == 2:  # j, r, c0, a, obj, n @+0x20, ind @+0x28, val @+0x30, mode/ncol @+0x38
            e.update(j=I(d), r=I(d + 4), n=I(d + 0x20), mode=I(d + 0x38), val_al=L(d + 0x30) & 0xF)
        out.append(e)
        r = L(r + 0x10)
    return out


def daa(model, odata, spx, m, n, post, basis, x, dual, pdata):
    e = {"m": m, "n": n, "records": records(post), "spx": bool(spx)}
    if spx:
        npre, mpre = I(pdata + 0xC), I(pdata + 8)
        e["pre_n_m"] = [npre, mpre]
        e["spx28"] = I(spx + 0x28)
        e["flip"] = I(pdata + 0x3C) == 1 and bool(L(pdata + 0x408)) and list(
            (ctypes.c_ubyte * npre).from_address(L(pdata + 0x408)))
        xo, do = (0x250, 0x248) if I(spx + 0x28) else (0x200, 0x1F8)
        e["pre_before"] = {"x": dbl(L(spx + xo), npre + mpre), "dj": dbl(L(spx + do), npre + mpre),
                           "st": ints(L(spx + 0x168), npre + mpre)}
    orig_daa(model, odata, spx, m, n, post, basis, x, dual, pdata)
    if spx:
        e["pre_after"] = {"x": dbl(L(spx + xo), npre + mpre), "dj": dbl(L(spx + do), npre + mpre),
                          "st": ints(L(spx + 0x168), npre + mpre)}
    e["out"] = {"basis": ints(basis, n + m), "x": dbl(x, n + m), "dual": dbl(dual, n + m)}
    cap["daa"].append(e)


def counter(key, T, off):
    f = T(base + off)

    def h(*a):
        cap["hits"][key] += 1
        f(*a)
    return h


T9 = ctypes.CFUNCTYPE(None, *[ctypes.c_void_p] * 9)
T6 = ctypes.CFUNCTYPE(None, *[ctypes.c_void_p] * 6)
patch_call(0x14B2F5, T363daa(daa))
patch_call(0x372458, T9(counter("365159@372458", T9, 0x3734F0)))
patch_call(0x372E6E, T9(counter("365159@372e6e", T9, 0x3734F0)))
patch_call(0x3729D4, T6(counter("363164@3729d4", T6, 0x36C2D0)))
# warm-start simplex on the original model after 363daa: which start vectors does it get (nonzero = passed)?
Tcda = ctypes.CFUNCTYPE(ctypes.c_int, ctypes.c_void_p, ctypes.c_int, *[ctypes.c_void_p] * 7)
orig_cda = Tcda(base + 0x153CC0)
cap["cda5b"] = []


def cda_hook(site):
    def h(model, meth, vb, cb, x, xr, d, y, w):
        cap["cda5b"].append({"site": site, "model": hex(model or 0), "args_nonnull": [bool(v) for v in (vb, cb, x, xr, d, y)]})
        r = orig_cda(model, meth, vb, cb, x, xr, d, y, w)
        cap["cda5b"][-1]["extra_iters"] = D(L(model + 0xE0) + 0x2A0) if L(model + 0xE0) else None
        return r
    return Tcda(h)


for s_ in (0x14C18D, 0x14C61B, 0x14C68E, 0x14CBB9, 0x14CCEF, 0x14EF65, 0x14F4F9, 0x14FE68, 0x1502D1):
    patch_call(s_, cda_hook(hex(s_)))

params = {"OutputFlag": 1, "LogToConsole": 0, "LogFile": sys.argv[2] + ".log", "Threads": 1}
for kv in sys.argv[3:]:
    k, v = kv.split("=")
    params[k] = float(v) if "." in v or "e" in v else int(v)
env = gp.Env(params=params)
mod = gp.read(sys.argv[1], env)
api = ctypes.pythonapi
api.PyCapsule_GetName.restype = ctypes.c_char_p
api.PyCapsule_GetName.argtypes = [ctypes.py_object]
api.PyCapsule_GetPointer.restype = ctypes.c_void_p
api.PyCapsule_GetPointer.argtypes = [ctypes.py_object, ctypes.c_char_p]
cap["user_model"] = hex(api.PyCapsule_GetPointer(mod._cmodel, api.PyCapsule_GetName(mod._cmodel)))
mod.optimize()
fin = {a: [float(v).hex() for v in mod.getAttr(a, mod.getVars() if a in ("X", "RC") else mod.getConstrs())]
       for a in ("X", "RC", "Slack", "Pi")}
fin["VBasis"] = mod.getAttr("VBasis", mod.getVars())
fin["CBasis"] = mod.getAttr("CBasis", mod.getConstrs())
fin["sense"] = "".join(mod.getAttr("Sense", mod.getConstrs()))
cap["final"] = fin
cap["status"], cap["iters"], cap["obj"] = mod.Status, mod.IterCount, mod.ObjVal.hex()
json.dump(cap, open(sys.argv[2], "w"))

# bitwise comparison of the 363daa output with the final attributes
if cap["daa"]:
    o, n, m = cap["daa"][-1]["out"], len(fin["X"]), len(fin["Pi"])
    neq = lambda a, b: [i for i, (u, v) in enumerate(zip(a, b)) if u != v]
    print("hits", cap["hits"], "status", cap["status"], "iters", cap["iters"], "cda5b", cap["cda5b"], "user", cap["user_model"])
    print("types", [r["type"] for r in cap["daa"][-1]["records"]])
    print("X  != out.x[:n]      at", neq(fin["X"], o["x"][:n]))
    print("RC != out.dual[:n]   at", neq(fin["RC"], o["dual"][:n]))
    print("Pi != out.dual[n:]   at", neq(fin["Pi"], o["dual"][n:]))
    print("Slack != out.x[n:]   at", neq(fin["Slack"], o["x"][n:]))
    print("basis cols", o["basis"][:n])
    print("VBasis    ", fin["VBasis"])
    print("basis rows", o["basis"][n:])
    print("CBasis    ", fin["CBasis"])
else:
    print("363daa not reached; hits", cap["hits"])
