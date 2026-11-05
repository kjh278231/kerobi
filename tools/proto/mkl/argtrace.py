"""record args at entry of given MKL functions: argtrace.run(fn_names, callable) -> list of (name, [rdi..r9, stack...])"""
import ctypes, os
import mk
D = os.path.dirname(os.path.abspath(__file__))
SYM = {}
for ln in open(D + '/syms.txt'):
    p = ln.split()
    if len(p) >= 3 and p[2] in 'tT': SYM.setdefault(p[-1], int(p[0], 16))
at = ctypes.CDLL(D + '/libargtrace.so')
def run(names, f):
    addrs = [SYM[n] for n in names]
    arr = (ctypes.c_uint64 * len(addrs))(*[mk.BASE + a for a in addrs])
    at.at_set(arr, len(addrs)); f(); at.at_off()
    buf = (ctypes.c_uint64 * (16 * 100000))()
    n = at.at_get(buf, 100000)
    inv = {mk.BASE + a: nm for nm, a in zip(names, addrs)}
    return [(inv[buf[16 * i]], [buf[16 * i + j] for j in range(1, 16)]) for i in range(n)]
