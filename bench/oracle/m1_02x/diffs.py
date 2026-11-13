"""M1-02x: every e_r output difference (form, idx, value) of the current build (PYTHONPATH) inside the M1-02k same-state
range, with the value differences classified by ulp distance (same sign, both nonzero) or zero/sign. Run on the final
build and on the build without the LU+0x140 test to show which differences the test does not cause.
usage (worktree root): PYTHONPATH=$PWD/python ../.venv/bin/python bench/oracle/m1_02x/diffs.py"""
import struct, tempfile
src = open('tools/m1_lt_check.py').read().replace('\nmain()\n', '\n')
g = {'__file__': 'tools/m1_lt_check.py', '__name__': 'diffs'}
exec(compile(src, 'm1_lt_check', 'exec'), g)
data, tmp = g['load'](), tempfile.mkdtemp()
f = lambda h: struct.unpack('>d', bytes.fromhex(h))[0]
for n in g['MODELS']:
    nat = g['rust_run'](n, tmp)
    er = [e for e in data[n]['caps'][0]['events'] if tuple(e[0][:2]) == g['ER']]
    matched = next((k for k, (e, r) in enumerate(zip(er, nat)) if (e[8], e[5]) != (r['r'], r['e'])), min(len(er), len(nat)))
    matched = g['KNOWN_D'].get(n, matched)
    form, idx, val = [], [], []
    for k in range(matched):
        e, r = er[k], nat[k]
        if (e[10] < 0) != r['od']:
            form.append(k)
            continue
        if e[10] >= 0 and e[13] != r['idx']:
            idx.append(k)
        if e[14] != r['val']:
            ulp, zs = 0, 0
            for a, b in zip(e[14], r['val']):
                if a == b:
                    continue
                x, y = f(a), f(b)
                if x == 0 or y == 0 or (x < 0) != (y < 0):
                    zs += 1
                else:
                    ulp = max(ulp, abs(int(a, 16) - int(b, 16)))
            val.append((k, ulp, zs))
    print(n, 'range', matched, 'form', form)
    print(n, 'idx', idx)
    print(n, 'val', [k for k, _, _ in val])
    print(n, 'val max_ulp', max((u for _, u, _ in val), default=0), 'calls with zero/sign diffs',
          [(k, z) for k, _, z in val if z])
