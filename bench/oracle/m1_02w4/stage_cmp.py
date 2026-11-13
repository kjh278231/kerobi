"""M1-02w-4 exploration: compare the original e_r stage capture (tools/m1_er_dense.js) with the Rust probe
(RSOLVER_BTRAN_PROBE=<spec> RSOLVER_STAGE_PROBE=k0,k1) call by call: r and row eta count, row etas (count, rows, value
bits), U^T output (rows, order, value bits), the dense vector after the etas (0x12e912 hit required when there are
etas) and the final output (length and bits). Original indices are basis positions (the capture header's "by row" is
wrong); mapped with the probe pos_row. Exit 1 on any difference or when no call is compared.
usage: python3 bench/oracle/m1_02w4/stage_cmp.py ORIGINAL.json.gz RUST_SPEC.out[.gz]"""
import gzip, json, sys

o = json.load(gzip.open(sys.argv[1]))
ev = {e[0]: e for e in o['events']}
rs = [json.loads(l) for l in (gzip.open(sys.argv[2], 'rt') if sys.argv[2].endswith('.gz') else open(sys.argv[2]))]
nat = {x['k']: x for x in rs if 'r' in x and 'k' in x and 'i' not in x}
st = {x['stage']: x for x in rs if 'stage' in x}


def ulp(a, b):
    a, b = int(a, 16), int(b, 16)
    return abs(a - b) if (a >> 63) == (b >> 63) else 'sign'


total = 0
for k in sorted(st):
    e, s, n = ev[k], st[k], nat[k]
    d, pr = e[4], s['pos_row']
    bad = []
    if (e[1], e[2]) != (n['r'], n['e']):
        bad.append(f'r/etas original {e[1]}/{e[2]} Rust {n["r"]}/{n["e"]}')
    oe, re_ = d['etas'], s['etas']
    if len(oe) != len(re_):
        bad.append(f'eta count {len(oe)} vs {len(re_)}')
    for i, (a, b) in enumerate(zip(oe, re_)):
        ai = [pr[x] for x in a[1]]
        if (pr[a[0]], ai, a[2]) != (b[0], b[1][0], b[1][1]):
            bad.append(f'eta {i}: r {pr[a[0]]}/{b[0]}, rows same {ai == b[1][0]}, '
                       f'first value diff {next(((ai[j], x, y, ulp(x, y)) for j, (x, y) in enumerate(zip(a[2], b[1][1])) if x != y), None)}')
    if d['ut'] is None:
        bad.append('original U^T output missing (21a941 not called)')
    else:
        oi, ov = [pr[x] for x in d['ut'][1]], d['ut'][2]
        ri, rv = s['ut']
        if oi != ri:
            bad.append(f'U^T rows/order differ ({len(oi)} vs {len(ri)})')
        od, rd = dict(zip(oi, ov)), dict(zip(ri, rv))
        bad += [f'U^T row {i}: {od.get(i)} vs {rd.get(i)}' for i in sorted(set(od) | set(rd)) if od.get(i) != rd.get(i)]
    if oe:
        if d['hit912'] != 1 or d['eta_out'] is None:
            bad.append(f'0x12e912 hit {d["hit912"]}: no dense eta vector')
        else:
            eor = [None] * len(pr)
            for p, v in enumerate(d['eta_out']):
                eor[pr[p]] = v
            bad += [f'eta_out row {i}: {eor[i]} vs {s["eta_out"][i]} ({ulp(eor[i], s["eta_out"][i])})'
                    for i in range(len(pr)) if eor[i] != s['eta_out'][i]]
    if d['out_val'] != n['val'] or (e[3] >= 0 and d['out_idx'] != n['idx']):
        bad.append(f'output differs ({len(d["out_val"])} vs {len(n["val"])} values)')
    total += len(bad)
    print(f'k {k} r {e[1]} etas {e[2]} hit912 {d["hit912"]} diffs {len(bad)}')
    for b in bad[:8]:
        print('  ', b)
print('CALLS', len(st), 'TOTAL DIFFS', total)
sys.exit(1 if total or not st else 0)
