#!/usr/bin/env python3
"""M1-02w: the two terms of 036ade's first pool fill local_f8 = max(cap/5, min(m/2+2, d^2)*2m) (m < 500; d = nnz(B)/m,
lessons/original.md) at every stair refactor, B from the original basis heads of bench/oracle/m1_01e/explore/stair.json.gz
(tools/m1_block_state.js) and column counts from the MPS file (objective row excluded), cap = F+0x158 from
bench/oracle/m1_02w/explore/stair_pool.json.gz. Run from the worktree root."""
import gzip
import json

heads = json.load(gzip.open('bench/oracle/m1_01e/explore/stair.json.gz'))['events']
cap = json.load(gzip.open('bench/oracle/m1_02w/explore/stair_pool.json.gz'))['events'][0]['enter'][0]
rows, cols, order, obj, sec = {}, {}, [], None, None
for line in gzip.open('../re/bench/data/netlib/stair.mps.gz', 'rt'):
    if line[0] not in ' \t':
        sec = line.split()[0]
        continue
    f = line.split()
    if sec == 'ROWS':
        obj = obj or (f[1] if f[0] == 'N' else None)
        rows[f[1]] = f[0]
    elif sec == 'COLUMNS':
        if f[0] not in cols:
            cols[f[0]] = 0
            order.append(f[0])
        cols[f[0]] += sum(1 for i in range(1, len(f), 2) if rows.get(f[i]) != 'N')
n, m = len(order), sum(1 for t in rows.values() if t != 'N')
assert (m, n) == (heads[0]['m'], heads[0]['n']), (m, n)
print(f'stair m {m} n {n} cap {cap} cap/5 {cap // 5}')
print('k nnzB y=min(m/2+2,d^2)*2m local_f8 parity winner')
for e in heads:
    nnz = sum(cols[order[j]] if j < n else 1 for j in e['head'])
    d = nnz / m
    y = min(m // 2 + 2, d * d) * 2 * m
    f8 = int(max(y, cap // 5))
    print(e['k'], nnz, f'{y:.3f}', f8, f8 % 2, 'nnz(B)' if y > cap // 5 else 'cap/5')
