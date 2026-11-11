"""M1-02k: synthetic BTRAN right-hand sides for tools/m1_btran_path.js SPEC and the Rust RSOLVER_BTRAN_PROBE.
For each chosen e_r BTRAN index k (9e376 from 28c93, counted in a plain capture of the same model) with m and
D = (int)(LU+0x108 + LU+0x110) read from that capture: nnz in {1, m/D, m/D+1, m/3, m/3+1, m/2} (floors, deduplicated),
positions spread ascending, and the first nnz with m/D < nnz <= m/3 (old 3·nnz test sparse, the original's dense)
once more in reverse order. Values are exact binary fractions with alternating sign.
usage: python3 tools/m1_btran_spec.py PLAIN_CAPTURE.json.gz K[,K...] > SPEC"""
import gzip, json, struct, sys


def er_calls(cap):
    for e in cap['events']:
        if e[0][0] == 376 and e[0][1] == 0xc51dd:
            yield e


def main():
    cap = json.load(gzip.open(sys.argv[1]))
    ks = [int(k) for k in sys.argv[2].split(',')]
    calls = list(er_calls(cap))
    f = lambda h: struct.unpack('>d', bytes.fromhex(h))[0]
    for k in ks:
        _, m, _, x108, x110, *_ = calls[k]
        d = int(f(x108) + f(x110))
        sizes = sorted({n for n in (1, m // d, m // d + 1, m // 3, m // 3 + 1, m // 2) if 1 <= n <= m})
        band = [n for n in sizes if d * n > m >= 3 * n]
        cases = [(n, False) for n in sizes] + [(n, True) for n in band[:1]]
        for n, rev in cases:
            pos = [(2 * i + 1) * m // (2 * n) for i in range(n)]
            assert len(set(pos)) == n
            val = [(-1) ** i * (1 + (i % 7) / 8) for i in range(n)]
            xs = list(zip(pos, val))[::-1] if rev else list(zip(pos, val))
            print(f"{k}\t" + ';'.join(f"{p},{struct.unpack('>Q', struct.pack('>d', v))[0]:016x}" for p, v in xs))


main()
