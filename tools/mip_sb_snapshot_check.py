#!/usr/bin/env python3
"""M2-06h-1: 저장 factor와 bound 큐 적용 검사."""
import argparse
import collections
import copy
import gzip
import json
import struct
from pathlib import Path
from mip_sb_restore_check import BINARY, sha, same, rules

ROOT = Path(__file__).resolve().parents[1]
DATA = ROOT / 'bench/oracle/m2_06h1/scaled'
OLD = ROOT / 'bench/oracle/m2_06g2/final/pk1_iters10_1.json.gz'
NEW = {'factorSave', 'boundwrite', 'flush'}


def load(p):
    with gzip.open(p) as f:
        return json.load(f)


def projection(d):
    d = copy.deepcopy(d)
    for e in d['events']:
        if 'steps' in e:
            e['steps'] = [s for s in e['steps'] if s['kind'] not in NEW]
            for s in e['steps']:
                s.pop('factor', None)
                s.pop('factorDst', None)
    return d


def factor(f, at):
    assert isinstance(f, dict), at + '.missing'
    m, u, l, eta = (f[k] for k in ('m', 'u', 'l', 'eta'))
    assert m > 0 and min(u, l, eta) >= 0, at
    lengths = [m*8, (m+1)*8, u*4, u*8, (m+1)*8, m*8, l*4, l*8, eta*4, m*4]
    same(lengths, [len(bytes.fromhex(a)) for a in f['arrays']], at + '.lengths')
    same(u, int.from_bytes(bytes.fromhex(f['arrays'][1])[-8:], 'little', signed=True), at+'.Uend')
    same(l, int.from_bytes(bytes.fromhex(f['arrays'][4])[-8:], 'little', signed=True), at+'.Lend')


def acceptance(d):
    totals = collections.Counter()
    for ei, e in enumerate(d['events']):
        if e['e'] != 'double':
            continue
        baseline = None
        pending = {}
        bounds = copy.deepcopy(e['bounds0'])
        for si, s in enumerate(e['steps']):
            at = f'double[{ei}].steps[{si}]'
            k = s['kind']
            totals[k] += 1
            if k == 'factorSave':
                assert baseline is None, at+'.duplicate save'
                factor(s['src'], at+'.src')
                factor(s['dst'], at+'.dst')
                same(s['src'], s['dst'], at+'.save')
                baseline = s['dst']
            elif k == 'restore':
                assert baseline is not None, at+'.missing save'
                factor(s['factor'], at+'.factor')
                factor(s['factorDst'], at+'.factorDst')
                same(baseline, s['factor'], at+'.saved unchanged')
                same(baseline, s['factorDst'], at+'.restored')
                totals['up' if s['branch'][1] == 62 else 'down'] += 1
            elif k == 'boundwrite':
                same(0, s['rc'], at+'.rc')
                same(bounds, s['before'], at+'.before')
                same(bounds, s['after'], at+'.deferred')
                same(len(s['j']), len(s['val']), at+'.value length')
                same(len(s['j']), len(bytes.fromhex(s['dir'])), at+'.dir length')
                same([False]*len(s['j']), s['flip'], at+'.uncovered flip')
                for j, v, direction, scale in zip(s['j'], s['val'], bytes.fromhex(s['dir']), s['scales']):
                    assert 0 <= j < bounds[0] and direction in (60, 62), at+'.request'
                    x=struct.unpack('>d',bytes.fromhex(v))[0]
                    scale=struct.unpack('>d',bytes.fromhex(scale))[0]
                    pending[(j, 1 if direction == 62 else 2)] = struct.pack('<d',x/scale if abs(x)<1e30 else x).hex()
            elif k == 'flush':
                same(0, s['rc'], at+'.rc')
                for (j, side), val in pending.items():
                    row = bounds[side]
                    bounds[side] = row[:j*16] + val + row[(j+1)*16:]
                same(bounds, s['bounds'], at+'.applied')
                pending.clear()
        assert not pending, f'double[{ei}].pending'
    for k in ('factorSave', 'restore', 'up', 'down', 'flush', 'boundwrite'):
        assert totals[k] > 0, 'zero target: '+k
    return dict(totals)


def identity(d, manifest):
    same(BINARY, d['binary_sha256'], 'binary')
    same(manifest['model'], d['model_sha256'], 'model')
    same(sha(ROOT/'tools/mip_sb_snapshot.js'), d['tool_sha256'], 'tool')
    same('mip_sb_snapshot', d['kind'], 'kind')
    same(True, d['ok'], 'ok')
    same([], d['errors'], 'errors')
    same(dict(Threads=1, Seed=0, LogToConsole=0, NodeLimit=300, GURO_PAR_SBMAXITERS=10), d['params'], 'params')


def check(d, manifest):
    identity(d, manifest)
    totals = acceptance(d)
    p = projection(d)
    rules(p, 'pk1_iters10')
    old = load(OLD)
    for k in ('events', 'output', 'counts', 'params', 'model_sha256', 'binary_sha256'):
        same(old[k], p[k], 'projection.'+k)
    return totals


def self_test(original, manifest):
    def step(d, k):
        return next(s for e in d['events'] if e['e']=='double' for s in e['steps'] if s['kind']==k)
    def field(k, f, v):
        return lambda d: step(d, k).__setitem__(f, v)
    tests = [
        ('binary', lambda d:d.__setitem__('binary_sha256', '0'*64)),
        ('model', lambda d:d.__setitem__('model_sha256', '0'*64)),
        ('empty', lambda d:d.__setitem__('events', [])),
        ('no double', lambda d:d.__setitem__('events', [e for e in d['events'] if e['e']!='double'])),
        ('null factor', field('restore','factor',None)),
        ('null dst', field('restore','factorDst',None)),
        ('bound', lambda d:step(d,'flush')['bounds'].__setitem__(1,'00')),
        ('request', lambda d:step(d,'boundwrite')['val'].__setitem__(0,'0000000000000000')),
    ]
    for i in range(10):
        tests.append((f'array {i}', lambda d,i=i:step(d,'restore')['factorDst']['arrays'].__setitem__(i,'00')))
    for name, mutate in tests:
        d=copy.deepcopy(original); mutate(d)
        try:
            check(d, manifest)
        except AssertionError:
            print('REJECT', name)
        else:
            raise AssertionError('NOT REJECTED '+name)


def main():
    ap=argparse.ArgumentParser();ap.add_argument('--self-test',action='store_true');ap.add_argument('--write-manifest',action='store_true');a=ap.parse_args()
    files=[DATA/f'{r}.json.gz' for r in (1,2,3)]+[DATA/'control.json.gz']
    if a.write_manifest:
        sources=[ROOT/'tools/mip_sb_snapshot.js',ROOT/'tools/mip_sb_snapshot_check.py',ROOT/'tools/mip_sb_snapshot_runs.sh', OLD]
        sources += [p for prefix in ('00576220','005754e0','00576a30','001690d0','00184e60') for p in (ROOT.parent/'re/decomp/c').glob(prefix+'*')]
        m={'model':load(files[0])['model_sha256'],'files':{str(p.relative_to(ROOT)) if p.is_relative_to(ROOT) else str(p):sha(p) for p in files+sources}}
        (DATA/'manifest.json').write_text(json.dumps(m,indent=2)+'\n')
    m=json.loads((DATA/'manifest.json').read_text())
    for p,h in m['files'].items():same(h,sha(ROOT/p),'manifest.'+p)
    ds=[load(p) for p in files]
    for d in ds:identity(d,m)
    same([],ds[3]['events'],'control.events')
    same(ds[0]['output'],ds[3]['output'],'control.output')
    for d in ds[1:3]:same(ds[0],d,'repeat')
    print(check(ds[0],m))
    if a.self_test:self_test(ds[0],m)
    print('ACCEPTANCE PASS')


if __name__=='__main__':main()
