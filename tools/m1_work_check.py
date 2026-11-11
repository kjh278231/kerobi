#!/usr/bin/env python3
"""M1-02p: every write to the dual simplex work double during the 06268b run (degen2, kb2) and the refactor-time fields
spx+0x150 / 0x158 / 0x3d0. Contract: docs/m1/m1_02p_work.md section 3. Exit 0 only when every check passes.
  --self-test   also the tampers and model variants (T)
  --write-sites rewrite bench/oracle/m1_02p/sites.tsv from the run-1 captures (the inventory S compares against)
"""
import collections, copy, gzip, hashlib, json, os, struct, sys

ROOT = os.path.join(os.path.dirname(os.path.abspath(__file__)), '..')
D = os.path.join(ROOT, 'bench/oracle/m1_02p')
BIN = '8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c'
TOOL = os.path.join(ROOT, 'tools/m1_work.js')
CAP = {'degen2': 'c476371aeb7317374cde3a43fc5bccb1299d0b788fe10f7cba67bd716f4d4401',
       'kb2': 'f4dfd4ba946a43b13e90cf73cd235b33d70932fe877c7caae2d567f3480e168b'}  # gz SHA-256 of runs 1-3
MODEL = {n: os.path.join(ROOT, '../re/bench/data/netlib', n + '.mps.gz') for n in CAP}
UP = {n: os.path.join(ROOT, 'bench/oracle/m1_02n', n + '_up_1.json.gz') for n in CAP}  # M1-02n captures (N)
UP_SHA = {'degen2': '10646348f90e674f7522f1f3c23716c36299e3398db652c2246af509de46c205',
          'kb2': '613052312e6327da6d15c05284bcd17924e852ba0108598c773214e1504e3ca6'}
F36ADE = 0xd4190  # LU factorization 036ade
NAMES = {0x11f620: '8b473', 0x120740: '2207de', 0x125f50: '9d112', 0x12de10: '9e288', 0x131710: '9e376', 0x132460: '9e60b',
         0x134d90: 'a4109', 0xbdc10: '15d2d', 0xbe6b0: '16b0a', 0xbf5b0: '1ca57', 0xc0050: '1bae9c', 0xc3f80: '1b7106',
         0xc4fe0: '28c93', 0xc74a0: '1c35c5', 0xccbc0: '2896a3', 0xcd720: '1c8477', 0xd4190: '36ade', 0xe45c0: '48d5f',
         0xe7c50: '534f7', 0xf15c0: '5b257', 0xcc6a0: '1c090a'}
F = lambda h: struct.unpack('<d', int(h, 16).to_bytes(8, 'little'))[0]
H = lambda x: struct.pack('<d', x)[::-1].hex()
sha = lambda p: hashlib.sha256(open(p, 'rb').read()).hexdigest()
load = lambda p: json.loads(gzip.decompress(open(p, 'rb').read()))


def dims(path):
    """(rows, columns, A nonzeros) of an MPS: constraint rows, COLUMNS entries outside the objective row (nonzero)."""
    sec, obj, n, rows, cols = None, None, 0, 0, set()
    for line in gzip.open(path, 'rt'):
        if not line.strip() or line.startswith('*'):
            continue
        if not line[0].isspace():
            sec = line.split()[0]
            continue
        f = line.split()
        if sec == 'ROWS' and f[0] == 'N' and obj is None:
            obj = f[1]
        elif sec == 'ROWS':
            rows += f[0] != 'N'
        elif sec == 'COLUMNS' and 'MARKER' not in line:
            cols.add(f[0])
            n += sum(1 for r, v in zip(f[1::2], f[2::2]) if r != obj and float(v) != 0.0)
    return rows, len(cols), n


# Q: sites whose amount reads only m, n, A nonzeros and the multiplier (docs/m1/m1_02p_work.md section 6.2), in the
# asm operation order. (w, m, n, nz, x148, LU+0x1f8) -> new value
QSITES = {0xe84b9: ('534f7 A nnz', lambda w, m, n, z, x, u, f: (float(z) * 2.0) * x + w),
          0xf48ae: ('05b257 3m (LU+0x1f8)', lambda w, m, n, z, x, u, f: (float(m) * 3.0) * u + w),
          0xf4f9b: ('05b257 9m', lambda w, m, n, z, x, u, f: (float(m) * 9.0) * x + w),
          0x1329c1: ('9e60b 3m (LU+0x1f8)', lambda w, m, n, z, x, u, f: w + (u * 3.0) * float(m)),
          0xbf981: ('1ca57 (m+n)(2 + [spx+0x338])', lambda w, m, n, z, x, u, f: (float(m + n) * (float(f != 0) + 2.0)) * x + w),
          0xc391f: ('1bae9c m', lambda w, m, n, z, x, u, f: float(m) * x + w)}


def sites():
    """bench/oracle/m1_02p/sites.tsv: (trap pc, innermost hooked function) -> {model: count}."""
    t = {}
    with open(os.path.join(D, 'sites.tsv')) as fh:
        hdr = fh.readline().rstrip('\n').split('\t')
        for line in fh:
            r = dict(zip(hdr, line.rstrip('\n').split('\t')))
            t[(int(r['pc'], 16), int(r['fn_off'], 16))] = {n: int(r[n]) for n in CAP}
    return t


def check(caps, ups, nz, table, var=None):
    bad, rep = [], []
    for name, cap in caps.items():
        if cap['binary_sha256'] != BIN or not cap['ok'] or not cap['events'] or cap['errors']:
            bad.append(f'M {name}: binary {cap["binary_sha256"][:12]} ok {cap["ok"]} events {len(cap["events"])}')
            continue
        ev, n = cap['events'], collections.Counter()
        # C: E then W are all the changes; every D / U / L sees the last value; same pointer
        cur, nE = None, 0
        for e in ev:
            t = e['type']
            if t == 'E':
                cur, nE = e['v'], nE + 1
            elif t == 'W':
                if e['x'] not in ('breakpoint', 'single-step') or e['f'] < 0:
                    bad.append(f'C {name} pivot {e["k"]}: write at pc {e["pc"]:#x} exception {e["x"]} function {e["f"]}')
                cur = e['v']
                n['C writes'] += 1
            elif t in ('D', 'U') and (not e['wp'] or e['work'] != cur):
                bad.append(f'C {name} {t} pivot {e["k"]}: work {e["work"]} last written {cur} same pointer {e["wp"]}')
            elif t == 'L' and e['v'] != cur:
                bad.append(f'C {name} return: work {e["v"]} last written {cur}')
        if nE != 1 or ev[-1]['type'] != 'L':
            bad.append(f'C {name}: {nE} 06268b entries, last event {ev[-1]["type"]}')
        # S: write sites and counts = the inventory
        cnt = collections.Counter((e['pc'], e['f']) for e in ev if e['type'] == 'W')
        for key in sorted(set(cnt) | {k for k, v in table.items() if v[name]}):
            if cnt[key] != table.get(key, {}).get(name, 0):
                bad.append(f'S {name}: pc {key[0]:#x} in {NAMES.get(key[1], hex(key[1]))} runs {cnt[key]}, '
                           f'inventory {table.get(key, {}).get(name)}')
        n['S sites'] = len(cnt)
        # F: 0x150 / 0x158 around each 05b257 call from 06268b, unchanged elsewhere; 0x3d0 = A nonzeros
        last = None
        for j, e in enumerate(ev):
            if e['type'] == 'D':
                if last is not None and (e['w150'], e['w158']) != last:
                    bad.append(f'F {name} decision pivot {e["k"]}: 0x150/0x158 {e["w150"]}/{e["w158"]} last 05b257 return {last}')
                if e['c3d0'] != nz[name][2]:
                    bad.append(f'F {name} decision pivot {e["k"]}: spx+0x3d0 {e["c3d0"]}, A nonzeros {nz[name][2]}')
                n['F 0x3d0'] += 1
            if e['type'] != 'U':
                continue
            if last is not None and (e['w150'], e['w158']) != last:
                bad.append(f'F {name} 05b257 entry pivot {e["k"]}: 0x150/0x158 changed since the last return')
            v = next(i for i in range(j + 1, len(ev)) if ev[i]['type'] == 'V')
            ws = [i for i in range(j + 1, v) if ev[i]['type'] == 'W']
            fa = [i for i in ws if ev[i]['f'] == F36ADE]
            out, old = ev[v], F(e['w158'])
            if not fa:
                n['F calls without 036ade'] += 1
                if (out['w150'], out['w158']) != (e['w150'], e['w158']):
                    bad.append(f'F {name} 05b257 pivot {e["k"]} (reason {e["reason"]}): no factorization, 0x150/0x158 changed')
            else:
                n['F factorizations (reason %d)' % e['reason']] += 1
                before = ev[max([i for i in ws if i < fa[0]], default=j)]
                before = before['v'] if before['type'] == 'W' else before['work']
                n['F 0x150 = work at 05b257 entry'] += before == e['work']
                if out['w150'] != before:
                    bad.append(f'F {name} 05b257 pivot {e["k"]}: 0x150 {out["w150"]}, work before the first 036ade write {before}')
                w150 = F(out['w150'])
                d = F(ev[ws[-1]]['v'] if var == 'ret' else ev[fa[-1]]['v'])
                if e['env2454'] != 0 and e['env2728'] < 2:
                    dl = d - w150
                    new = dl if old <= 0.0 else (dl + old) * (1.0 if var == 'nohalf' else 0.5)
                else:
                    new = old
                    n['F 0x158 not updated (env)'] += 1
                if H(new) != out['w158']:
                    bad.append(f'F {name} 05b257 pivot {e["k"]}: 0x158 {out["w158"]}, model {H(new)} (old {e["w158"]}, '
                               f'last 036ade write {H(d)}, 0x150 {out["w150"]})')
            last = (out['w150'], out['w158'])
        # R: the work predicate (06268b C:1295-1303) on this capture; true -> refactor reason 10 at the next 05b257
        mg = []
        for j, e in enumerate(ev):
            if e['type'] != 'D' or e['upd'] % 10 != 9 or e['upd'] >= e['per'] or F(e['w158']) <= 0.0:
                continue
            lhs = F(e['w158']) * 14.0 + float(e['c3d0'] * 100)
            rhs = (F(e['work']) if e['work'] else 0.0) - F(e['w150'])
            mg.append(rhs / lhs)
            nxt = next((x for x in ev[j + 1:] if x['type'] == 'U'), None)
            if lhs < rhs:
                n['R work true'] += 1
                if nxt is not None and nxt['reason'] != 10:
                    bad.append(f'R {name} pivot {e["k"]}: work predicate true, next 05b257 reason {nxt["reason"]}')
        n['R k%10=9 decisions'] = len(mg)
        # Q: model-constant amounts, bit for bit from the previous value (multipliers from the first decision)
        d0 = next(e for e in ev if e['type'] == 'D')
        x, u, (m, nc, z), prev = F(d0['x148']), F(d0['x1f8']), nz[name], None
        for e in ev:
            if e['type'] == 'E':
                prev = e['v']
            elif e['type'] == 'W':
                if e['pc'] in QSITES:
                    want = H(QSITES[e['pc']][1](F(prev), m, nc, z, x, u, 0 if var == 'q338' else e.get('x338')))
                    n['Q ' + QSITES[e['pc']][0]] += 1
                    if want != e['v']:
                        bad.append(f'Q {name} pivot {e["k"]} pc {e["pc"]:#x} ({QSITES[e["pc"]][0]}): {e["v"]}, model {want}')
                prev = e['v']
        n['Q x148 = LU+0x1f8'] = d0['x148'] == d0['x1f8']
        # N: the decision rows equal the M1-02n capture
        da = [(e['k'], e['upd'], e['reason'], e['work'], e['w150'], e['w158'], e['c3d0']) for e in ev if e['type'] == 'D']
        db = [(e['k'], e['lu']['upd'], e['reason'], e['work'], e['w150'], e['w158'], e['c3d0']) for e in ups[name]['events']
              if e['type'] == 'D']
        if da != db or cap['output'] != ups[name]['output']:
            i = next((i for i, (a, b) in enumerate(zip(da, db)) if a != b), min(len(da), len(db)))
            bad.append(f'N {name}: decision {i} of {len(da)}/{len(db)}: {da[i] if i < len(da) else None} vs 02n '
                       f'{db[i] if i < len(db) else None}; output {cap["output"]!r} vs {ups[name]["output"]!r}')
        n['N decisions'] = len(da)
        rep.append(f'{name}: {dict(sorted(n.items()))}' + (f' margin rhs/lhs min {min(mg):.4f} max {max(mg):.4f}' if mg else ''))
    return bad, rep


def groups(cap):
    """G (information): per factorization interval, the increments by writer function."""
    out, cur, seg, acc = [], None, None, collections.Counter()
    for e in cap['events']:
        if e['type'] == 'E':
            cur = F(e['v'])
        elif e['type'] == 'U' and e['reason'] != 0:
            if seg is not None:
                out.append((seg, acc))
            seg, acc = (e['k'], e['reason']), collections.Counter()
        elif e['type'] == 'W':
            v = F(e['v'])
            acc[NAMES.get(e['f'], hex(e['f']))] += v - cur
            cur = v
    out.append((seg, acc))
    return out


def write_sites(caps):
    c, calls = collections.defaultdict(collections.Counter), collections.defaultdict(set)
    for name, cap in caps.items():
        for e in cap['events']:
            if e['type'] == 'W':
                c[(e['pc'], e['f'])][name] += 1
                calls[(e['pc'], e['f'])].add(e['c'])
    with open(os.path.join(D, 'sites.tsv'), 'w') as o:
        o.write('pc\tfn_off\tfn\tdegen2\tkb2\tcall_sites\n')
        for k in sorted(c, key=lambda k: (NAMES[k[1]], k[0])):
            o.write('%#x\t%#x\t%s\t%d\t%d\t%s\n' % (k[0], k[1], NAMES[k[1]], c[k]['degen2'], c[k]['kb2'],
                                                  ','.join('%#x' % x for x in sorted(calls[k]))))


def units(cap):
    """G (information): per write site, increments that are not a multiple of 0.5 * x148 (the LU solve folds)."""
    n, tot, prev = collections.Counter(), collections.Counter(), None
    x = F(next(e for e in cap['events'] if e['type'] == 'D')['x148'])
    for e in cap['events']:
        if e['type'] == 'E':
            prev = F(e['v'])
        elif e['type'] == 'W':
            v = F(e['v'])
            a = 2.0 * (v - prev) / x
            key = NAMES.get(e['f'], hex(e['f']))
            tot[key] += 1
            n[key] += abs(a - round(a)) > 1e-6 * max(1.0, abs(a))
            prev = v
    return {k: (v, tot[k]) for k, v in n.items() if v}


def main():
    st = '--self-test' in sys.argv
    caps, ups, nz, bad = {}, {}, {}, []
    th = sha(TOOL)
    for name, h in CAP.items():
        hs = [sha(os.path.join(D, f'{name}_w_{r}.json.gz')) for r in (1, 2, 3)]
        if len(set(hs)) != 1 or (h is not None and hs[0] != h):
            bad.append(f'M {name}: run hashes {[x[:12] for x in hs]}, pinned {h and h[:12]}')
        caps[name] = load(os.path.join(D, f'{name}_w_1.json.gz'))
        if caps[name]['model_sha256'] != sha(MODEL[name]) or caps[name]['tool_sha256'] != th:
            bad.append(f'M {name}: model or tool sha differs from the capture')
        if sha(UP[name]) != UP_SHA[name]:
            bad.append(f'M {os.path.basename(UP[name])}: sha {sha(UP[name])[:12]}')
        ups[name], nz[name] = load(UP[name]), dims(MODEL[name])
    if '--write-sites' in sys.argv:
        write_sites(caps)
    table = sites()
    b, rep = check(caps, ups, nz, table)
    bad += b
    print('\n'.join(rep))
    for name in caps:
        for seg, acc in groups(caps[name]):
            top = ', '.join(f'{k} {v:.0f}' for k, v in acc.most_common(6))
            print(f'G {name} from pivot {seg[0]} (reason {seg[1]}): total {sum(acc.values()):.0f}: {top}')
        print(f'G {name} increments not a multiple of 0.5*x148 (count, writes): {units(caps[name])}')
    if st:
        def tamper(desc, f, var=None):
            c2 = copy.deepcopy(caps)
            f(c2)
            b2, _ = check(c2, ups, nz, table, var)
            print(f'T {desc}: {"rejected" if b2 else "NOT rejected"}' + (f' ({b2[0][:110]})' if b2 else ''))
            if not b2:
                bad.append(f'T {desc} not rejected')

        def ev(c2, name, f, last=False):
            x = [e for e in c2[name]['events'] if f(e)]
            return x[-1] if last else x[0]

        flip = lambda h: format(int(h, 16) ^ 1, '016x')

        def drop_w(c2):
            e = ev(c2, 'degen2', lambda e: e['type'] == 'W' and e['k'] == 200)
            c2['degen2']['events'].remove(e)

        def wfn(c2):
            e = ev(c2, 'degen2', lambda e: e['type'] == 'W' and e['k'] == 200 and e['f'] != 0x125f50)
            e['f'] = 0x125f50

        def pc(c2):
            ev(c2, 'kb2', lambda e: e['type'] == 'W')['pc'] += 1

        def v158(c2):
            e = ev(c2, 'degen2', lambda e: e['type'] == 'V' and e['k'] == 356)
            e['w158'] = flip(e['w158'])

        def v150(c2):
            e = ev(c2, 'degen2', lambda e: e['type'] == 'V' and e['k'] == 316)
            e['w150'] = flip(e['w150'])

        def c3d0(c2):
            ev(c2, 'kb2', lambda e: e['type'] == 'D')['c3d0'] += 1

        def dwork(c2):
            e = ev(c2, 'degen2', lambda e: e['type'] == 'D' and e['k'] == 356)
            e['work'] = flip(e['work'])

        tamper('binary hash 0', lambda c2: c2['kb2'].__setitem__('binary_sha256', '0' * 64))
        tamper('no events', lambda c2: c2['degen2'].__setitem__('events', []))
        tamper('one write deleted (degen2 pivot 200)', drop_w)
        tamper('one write attributed to another function (degen2 pivot 200)', wfn)
        tamper('write pc not in the inventory (kb2)', pc)
        tamper('0x158 return bit (degen2 refactor before pivot 356)', v158)
        tamper('0x150 return bit (degen2 refactor before pivot 316)', v150)
        tamper('0x3d0 + 1 (kb2)', c3d0)
        tamper('decision work bit (degen2 before pivot 356)', dwork)
        for var, desc in (('nohalf', '0x158 average without 0.5'), ('ret', '0x158 from the work at the 05b257 return'),
                          ('q338', '1ca57 amount without the spx+0x338 term')):
            b2, _ = check(caps, ups, nz, table, var)
            print(f'T model: {desc}: {"rejected" if b2 else "survives"}' + (f' ({b2[0][:110]})' if b2 else ''))
    for x in bad:
        print('FAIL', x)
    print('PASS' if not bad else f'FAIL ({len(bad)})')
    sys.exit(1 if bad else 0)


if __name__ == '__main__':
    main()
