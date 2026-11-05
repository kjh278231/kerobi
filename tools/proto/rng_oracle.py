"""Call libgurobi's own RNG (a7b4d6 @ file+0x88a870, a7b54f @ +0x88a8d0) on a u64 state of 12345 and print 4 ints and
one double (bits) for the common.rs test. Imports gurobipy first so the single-process guard applies."""
import ctypes
import gurobipy  # noqa: F401  (loads libgurobi, takes the guard lock)
base = next(int(l.split("-")[0], 16) for l in open("/proc/self/maps") if "libgurobi130.so" in l and l.split()[2] == "00000000")
ri = ctypes.CFUNCTYPE(ctypes.c_int, ctypes.c_void_p)(base + 0x88a870)
rd = ctypes.CFUNCTYPE(ctypes.c_double, ctypes.c_void_p)(base + 0x88a8d0)
st = ctypes.c_uint64(12345)
print([ri(ctypes.addressof(st)) for _ in range(4)], hex(ctypes.c_uint64.from_buffer(ctypes.c_double(rd(ctypes.addressof(st)))).value))
