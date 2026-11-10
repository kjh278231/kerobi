#!/usr/bin/env python3
"""Look up a decompiled Gurobi function without dumping whole files into context.

  tools/dc.py 3a484d                      # by name or name suffix: header, callers/callees, note refs
  tools/dc.py 0x3959f0 / --dec 3759600    # by address (any address inside the function)
  tools/dc.py 3a484d --grep 'lp\\+0x1c'   # matching lines with line numbers
  tools/dc.py 3a484d --range 120,260      # line slice (capped by --max)
  tools/dc.py 3a484d --asm [--range 0x3959f0,0x395a40]   # objdump of the function or an address window
"""
import argparse
import bisect
import re
import subprocess
import sys
from pathlib import Path

GS = Path(__file__).resolve().parents[2]  # gurobi-sandbox: works from every rsolver worktree
DEC = GS / 're/decomp'
SO = GS / '.venv/lib/python3.13/site-packages/gurobipy/.libs/libgurobi130.so'
NOTE_DIRS = [GS / 're/notes', Path(__file__).resolve().parents[1] / 'docs']


def index():
    rows = []
    with open(DEC / 'index.tsv') as f:
        next(f)
        for line in f:
            off, name, size, callers, callees = line.split('\t')[:5]
            rows.append((int(off, 16), name, int(size), int(callers), int(callees)))
    rows.sort()
    return rows


def find(rows, key, dec):
    if dec or key.lower().startswith('0x'):
        a = int(key, 10 if dec else 16)
        i = bisect.bisect_right([r[0] for r in rows], a) - 1
        if i >= 0 and rows[i][0] <= a < rows[i][0] + rows[i][2]:
            return [rows[i]], a
        near = rows[i][1] if i >= 0 else '-'
        sys.exit(f'{hex(a)}: not inside an indexed function (nearest preceding: {near})')
    hits = [r for r in rows if r[1] == key] or [r for r in rows if r[1].endswith(key)] or [r for r in rows if key in r[1]]
    if not hits:
        sys.exit(f'{key}: no function in index.tsv')
    return hits, None


def cfile(r):
    return DEC / 'c' / f'{r[0]:08x}_{r[1]}.c'


def short(name):
    return name[len('PRIVATE'):].lstrip('0') if name.startswith('PRIVATE') else name


def edges(off):
    up, down = [], []
    with open(DEC / 'callgraph.tsv') as f:
        next(f)
        for line in f:
            co, cn, eo, en = line.rstrip('\n').split('\t')
            if int(co, 16) == off:
                down.append(f'{eo} {short(en)}')
            if int(eo, 16) == off:
                up.append(f'{co} {short(cn)}')
    return up, down


def notes(r, limit):
    keys = {short(r[1]), f'{r[0]:x}'}
    pat = re.compile('|'.join(re.escape(k) for k in keys))
    out = []
    for d in NOTE_DIRS:
        for p in sorted(d.rglob('*.md')):
            for n, line in enumerate(p.open(errors='replace'), 1):
                if pat.search(line):
                    out.append(f'{p.relative_to(GS)}:{n}: {line.strip()[:160]}')
    for line in out[:limit]:
        print('  ' + line)
    if len(out) > limit:
        print(f'  ... {len(out) - limit} more (--notes N)')


def span(text, sep=','):
    a, b = text.split(sep)
    return a, b


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument('key', help='name, name suffix (e.g. 3a484d) or 0x address')
    ap.add_argument('--dec', action='store_true', help='key is a decimal address (e.g. a return address from a capture)')
    ap.add_argument('--grep', help='regex; print matching C lines')
    ap.add_argument('--range', help='C line range a,b, or with --asm an address range 0xa,0xb')
    ap.add_argument('--asm', action='store_true', help='objdump instead of C')
    ap.add_argument('--max', type=int, default=200, help='max printed lines (default 200)')
    ap.add_argument('--notes', type=int, default=8, help='max note/doc references (default 8)')
    a = ap.parse_args()

    rows = index()
    hits, addr = find(rows, a.key, a.dec)
    if len(hits) > 1:
        print(f'{len(hits)} matches; refine the key:')
        for r in hits[:30]:
            print(f'  {r[0]:#x} {r[1]} size={r[2]}')
        return 1
    r = hits[0]
    path = cfile(r)
    body = path.read_text(errors='replace').splitlines() if path.exists() else []
    printed = []

    if a.asm:
        lo, hi = (int(x, 16) for x in span(a.range)) if a.range else (r[0], r[0] + r[2])
        cmd = ['objdump', '-d', '--no-show-raw-insn', f'--start-address={lo:#x}', f'--stop-address={hi:#x}', str(SO)]
        lines = subprocess.run(cmd, capture_output=True, text=True, check=True).stdout.splitlines()[6:]
        printed = lines
    elif a.grep:
        pat = re.compile(a.grep)
        printed = [f'{n}: {line}' for n, line in enumerate(body, 1) if pat.search(line)]
    elif a.range:
        lo, hi = (int(x) for x in span(a.range))
        printed = [f'{n}: {body[n - 1]}' for n in range(max(lo, 1), min(hi, len(body)) + 1)]
    else:
        where = f' (contains {hex(addr)}, +{addr - r[0]:#x})' if addr is not None else ''
        print(f'{r[0]:#x} {r[1]} size={r[2]} C={path.relative_to(GS)} lines={len(body)}{where}')
        up, down = edges(r[0])
        print(f'callers({r[3]}): ' + ', '.join(up[:20]))
        print(f'callees({r[4]}): ' + ', '.join(down[:40]))
        print('refs:')
        notes(r, a.notes)
        return 0

    for line in printed[:a.max]:
        print(line)
    if len(printed) > a.max:
        print(f'... {len(printed) - a.max} more lines (narrow --range/--grep or raise --max)')
    return 0


if __name__ == '__main__':
    sys.exit(main())
