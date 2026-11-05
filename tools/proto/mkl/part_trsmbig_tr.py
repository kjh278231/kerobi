"""log args of selected MKL functions during one DTRSM call: python part_trsmbig_tr.py m n [lda ldb]"""
import ctypes, sys
import numpy as np
import mk
D = __file__.rsplit('/', 1)[0]
tr = ctypes.CDLL(D + '/part_trsmbig_tracer.so')
F = {0x2443000: ('ll_0', 7), 0x1cabbe0: ('copy_ll', 0), 0x1b97c00: ('dcopy_r12', 0), 0x1cac3f0: ('scale_lu', 0),
     0x1cb40a0: ('kern_ll', 0), 0x1ca7720: ('fullacopy', 0)}
def run(f, funcs=F):
    ad = sorted(funcs)
    tr.tr_set((ctypes.c_uint64 * len(ad))(*[mk.BASE + a for a in ad]), (ctypes.c_int * len(ad))(*[funcs[a][1] for a in ad]), len(ad))
    f(); tr.tr_off()
    buf = (ctypes.c_uint64 * (18 * 100000))()
    n = tr.tr_get(buf, 100000)
    return [(funcs[buf[i * 18] - mk.BASE][0], [buf[i * 18 + j] for j in range(1, 18)]) for i in range(n)]
if __name__ == "__main__":
    m, n = int(sys.argv[1]), int(sys.argv[2])
    lda, ldb = (int(sys.argv[3]), int(sys.argv[4])) if len(sys.argv) > 4 else (m, m)
    rng = np.random.default_rng(1)
    U = rng.random(lda * m); B = rng.random(ldb * n)
    for nm, r in run(lambda: mk.trsm_lutu(m, n, U, lda, B, ldb)):
        if nm == 'll_0':
            print(f"ll_0 m={r[14]} n={r[15]} mtot={r[16]} Apk={r[4]:x} Bpk={r[5]:x} B={r[7]:x} ldb={r[8]} k={r[9]}")
        else:
            print(nm, ' '.join(f'{x:x}' for x in r[:12]))
