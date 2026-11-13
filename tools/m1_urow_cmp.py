#!/usr/bin/env python3
"""M1-02z: compare the row-wise U and diagonal of the original (tools/m1_urow_dump.js) with the Rust probe
(RSOLVER_BTRAN_PROBE=<spec> RSOLVER_UROW_PROBE=<k>, line with "urow" in <spec>.out, may be gzipped) at the same e_r
call. M1-02w: also takes tools/m1_lu108_dump.js captures (same diag/rows fields).
Prints the positions whose diagonal or row differs, the first rows, and for each differing original diagonal the Rust
positions holding the same bits. Exit 0 when identical, 1 otherwise.
usage: python3 tools/m1_urow_cmp.py ORIGINAL.json.gz RUST_SPEC.out"""
import gzip
import json
import sys


def main(orig, rust):
    d = json.load(gzip.open(orig))
    assert d['kind'] in ('m1_urow_dump', 'm1_lu108_dump') and d['ok'], 'original capture'
    o = d['events'][0]
    r = next(json.loads(l) for l in (gzip.open(rust, 'rt') if rust.endswith('.gz') else open(rust)) if '"urow"' in l)
    assert (o['call'], o['m']) == (r['urow'], r['m']), f'call/m {o["call"]},{o["m"]} vs {r["urow"]},{r["m"]}'
    dd = [p for p in range(o['m']) if o['diag'][p] != r['diag'][p]]
    rd = [p for p in range(o['m']) if o['rows'][p] != r['rows'][p]]
    print(f'{d["model"]} e_r {o["call"]} m {o["m"]}: diag diff {len(dd)} {dd}')
    print(f'row diff {len(rd)} {rd}')
    for p in rd[:4]:
        print(f'  row {p}: original {o["rows"][p][0]} rust {r["rows"][p][0]}')
    for p in dd:
        print(f'  diag {p} {o["diag"][p]} at rust {[q for q in range(o["m"]) if r["diag"][q] == o["diag"][p]][:6]}')
    return 1 if dd or rd else 0


if __name__ == '__main__':
    sys.exit(main(*sys.argv[1:3]))
