"""Uncrush oracle: patch Gurobi's call to 363daa (LP simplex-path postsolve, in 0c3d0d) so a Python thunk sees its
inputs and outputs, then compare the uncrushed arrays with the final model attributes.
usage: python uncrush_oracle.py <model.mps.gz> <out.json> [Param=Value ...] [--uncrush-delta COL=DELTA]
       (Gurobi: one process at a time)
Captured: post record list (type, data ptr, val-array alignment), presolved spx arrays (x/dj/status, n_pre+m_pre)
before and after the call, the 363daa outputs (basis int[n+m], x[n+m], dual[n+m]) and final X/Slack/RC/Pi/VBasis/CBasis.
Also counts hits of the 36578f-path calls (365159 x2, 363164) to show which path ran; on that path (Method 2/6 with
Crossover 0) the 365159 inputs and outputs are captured under "c159". --uncrush-delta deliberately perturbs one
original-model X after 365159, before Gurobi checks the uncrush violation; this is a fault-injection probe of
the status-13 rule, not an ordinary solver result. The unmodified X and perturbation are recorded explicitly."""
import argparse, ctypes, json
import gurobipy as gp

ap = argparse.ArgumentParser(description=__doc__)
ap.add_argument("model")
ap.add_argument("out")
ap.add_argument("params", nargs="*")
ap.add_argument("--uncrush-delta", metavar="COL=DELTA")
args = ap.parse_intermixed_args()
injection = None
if args.uncrush_delta is not None:
    col, delta = args.uncrush_delta.split("=", 1)
    injection = int(col), float(delta)

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
        # rec = list node (type, data ptr, next ptr, ?); raw = first 0x90 bytes of the data block (layouts, array pointers)
        e = {"type": t, "data": d & 0xF, "rec": r, "ptr": d, "raw": bytes((ctypes.c_ubyte * 0x90).from_address(d)).hex(),
             "hdr": bytes((ctypes.c_ubyte * 0x20).from_address(r)).hex(),
             "pre": bytes((ctypes.c_ubyte * 0x60).from_address(r - 0x60)).hex()}  # memory before the record: arena gaps
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
cap["c159"] = []


def c159(key):
    """365159(model, x, slack, d, y of the presolved model, then x, slack, d, y of the original): the nonbasic LP
    uncrush of 36578f (barrier/PDHG without crossover). Records inputs, outputs and the record list."""
    f = T9(base + 0x3734F0)

    def h(model, xi, si, di, yi, xo, so, do, yo):
        cap["hits"][key] += 1
        od, pd = L(model + 0xE8), L(L(model + 0x1D8) + 0xE8)
        n, m, npre, mpre = I(od + 0xC), I(od + 8), I(pd + 0xC), I(pd + 8)
        cs, rs = L(pd + 0x3E0), L(pd + 0x3D8)  # presolved model's scale arrays (36578f's 1/min scale term)
        e = {"site": key, "n_m": [n, m], "pre_n_m": [npre, mpre], "records": records(L(model + 0x1D0)),
             "pre_scale": [dbl(cs, npre), dbl(rs, mpre)] if cs else None,
             "pre_viol": D(L(L(model + 0x1D8) + 0x1B0) + 0x58),
             "in": {"x": dbl(xi, npre), "slack": dbl(si, mpre), "d": dbl(di, npre), "y": dbl(yi, mpre)}}
        f(model, xi, si, di, yi, xo, so, do, yo)
        if injection is not None:
            col, delta = injection
            e["unmodified_x"] = dbl(xo, n)
            value = ctypes.c_double.from_address(xo + 8 * col)
            value.value += delta
            e["fault_injection"] = {"column": col, "delta": delta.hex()}
        e["out"] = {"x": dbl(xo, n), "slack": dbl(so, m), "d": dbl(do, n), "y": dbl(yo, m)}
        cap["c159"].append(e)
    return T9(h)


patch_call(0x14B2F5, T363daa(daa))
patch_call(0x372458, c159("365159@372458"))
patch_call(0x372E6E, c159("365159@372e6e"))
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

params = {"OutputFlag": 1, "LogToConsole": 0, "LogFile": args.out + ".log", "Threads": 1}
for kv in args.params:
    k, v = kv.split("=")
    params[k] = float(v) if "." in v or "e" in v else int(v)
env = gp.Env(params=params)
mod = gp.read(args.model, env)
if injection is not None and not 0 <= injection[0] < mod.NumVars:
    ap.error("--uncrush-delta column must be a valid original-model column index")
api = ctypes.pythonapi
api.PyCapsule_GetName.restype = ctypes.c_char_p
api.PyCapsule_GetName.argtypes = [ctypes.py_object]
api.PyCapsule_GetPointer.restype = ctypes.c_void_p
api.PyCapsule_GetPointer.argtypes = [ctypes.py_object, ctypes.c_char_p]
cap["user_model"] = hex(api.PyCapsule_GetPointer(mod._cmodel, api.PyCapsule_GetName(mod._cmodel)))
mod.optimize()
fin = {a: [float(v).hex() for v in mod.getAttr(a, mod.getVars() if a in ("X", "RC") else mod.getConstrs())]
       for a in ("X", "RC", "Slack", "Pi")}
try:
    fin["VBasis"] = mod.getAttr("VBasis", mod.getVars())
    fin["CBasis"] = mod.getAttr("CBasis", mod.getConstrs())
except gp.GurobiError:  # no basis without crossover
    fin["VBasis"] = fin["CBasis"] = None
fin["sense"] = "".join(mod.getAttr("Sense", mod.getConstrs()))
cap["final"] = fin
cap["status"], cap["iters"], cap["obj"] = mod.Status, mod.IterCount, mod.ObjVal.hex()
cap["max_violation"] = mod.MaxVio.hex()
json.dump(cap, open(args.out, "w"))

# bitwise comparison of the 363daa output with the final attributes
if cap["c159"]:
    o, n = cap["c159"][-1]["out"], len(fin["X"])
    print("365159", cap["hits"], "status", cap["status"], "X == out.x", fin["X"] == o["x"], "RC == out.d", fin["RC"] == o["d"],
          "Pi == -out.y?", [float.fromhex(v) for v in fin["Pi"]][:3], [float.fromhex(v) for v in o["y"]][:3])
elif cap["daa"]:
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
