"""M2-10a summary of tools/mip_bnb_profile.sh output (docs/m2/mip_bnb_node_cost.md).

python3 tools/mip_bnb_profile.py <out dir> [--self-test]
Fails when an instrumented run's RESULT (status, nodes, iterations, bound and objective bits, path) differs from the
uninstrumented run or a model lacks exactly one PROFILE line. The patch itself asserts the LP replays per node.
"""
import shutil
import statistics
import sys
import tempfile
from pathlib import Path

def parse(path):
    out, prof = {}, []
    for line in path.read_text().splitlines():
        kind, rest = line.split(' ', 1)
        if kind == 'PROFILE':
            prof.append({k: float(v) for k, v in (f.split('=') for f in rest.split())})
        elif kind == 'RESULT':
            name, *fields = rest.split()
            res = dict(f.split('=') for f in fields)
            out[name] = (res, prof)
            prof = []
    return out


def load(d):
    base = parse(d / 'base.txt')
    models = sum('.mps.gz' in line for line in (d / 'env.txt').read_text().splitlines())
    reps = [parse(p) for p in sorted(d.glob('prof*.txt'))]
    if len(base) != models or not reps:
        raise SystemExit(f'FAIL base has {len(base)} models, {len(reps)} instrumented runs')
    for i, rep in enumerate(reps, 1):
        for name, (res, _) in base.items():
            if name not in rep:
                raise SystemExit(f'FAIL prof{i} {name}: no RESULT')
            got, prof = rep[name]
            for k in ('status', 'nodes', 'iters', 'bound', 'obj', 'path'):
                if got[k] != res[k]:
                    raise SystemExit(f'FAIL prof{i} {name} {k}: base {res[k]} instrumented {got[k]}')
            if len(prof) != 1:
                raise SystemExit(f'FAIL prof{i} {name}: {len(prof)} PROFILE lines')
    return base, reps


def med(reps, name, key):
    return statistics.median(r[name][1][0][key] for r in reps)


def report(base, reps):
    print(f'무영향 PASS: {len(reps)}회 계측 실행의 상태·노드·반복·경계·목적·경로가 계측 없는 실행과 같음 ({len(base)}모델)')
    print('\n| 모델 | 노드 | 계측 없는 벽시각 s | presolve % | 루트 % | 노드 LP % | 선택 % | 분기·복사 % | incumbent % | 기타 % | LP 최소–최대 s |')
    print('|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---|')
    for name, (res, _) in base.items():
        g = lambda k: med(reps, name, k)
        cold = g('spx') + g('eng') + g('solve') + g('unscaled')
        warm = g('warm_all') + g('wspx') + g('weng') + g('wsolve') + g('wunscaled')
        net = g('loop') - cold - warm
        total = g('presolve') + g('root') + net
        parts = [g('presolve'), g('root'), g('lp'), g('select'), g('branch'), g('incumbent')]
        other = net - sum(parts[2:])
        lps = [r[name][1][0]['lp'] for r in reps]
        print(f"| {name} | {res['nodes']} | {float(res['wall']):.3f} | "
              + ' | '.join(f'{100 * v / total:.1f}' for v in parts + [other]) + f' | {min(lps):.3f}–{max(lps):.3f} |')
    print('\n| 모델 | 반복/노드 | LP ms/노드 | 스케일 작업 공간 % | 엔진 생성 % | dual 풀이 % | 무스케일 재확인 % | 재생/실제 | 복사 nnz/노드 | 최대 열린 노드 |')
    print('|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|')
    for name, (res, _) in base.items():
        g = lambda k: med(reps, name, k)
        n = float(res['nodes'])
        cold = g('spx') + g('eng') + g('solve') + g('unscaled')
        print(f"| {name} | {g('lp_iter') / n:.1f} | {1e3 * g('lp') / n:.2f} | "
              + ' | '.join(f'{100 * g(k) / cold:.1f}' for k in ('spx', 'eng', 'solve', 'unscaled'))
              + f" | {cold / g('lp'):.2f} | {g('clone_nnz') / max(g('clones'), 1):.0f} | {g('max_open'):.0f} |")
    print('\n| 모델 | 비교 노드 | 불일치 | cold 반복 | warm 반복 | 반복 비 | cold LP s | warm LP s | 시간 비 | warm 고정 ms/노드 | warm 풀이 % |')
    print('|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|')
    for name, (res, _) in base.items():
        g = lambda k: med(reps, name, k)
        wsum = g('wspx') + g('weng') + g('wsolve') + g('wunscaled')
        fixed = (g('wspx') + g('weng') + g('wunscaled')) / max(g('warm_nodes'), 1)
        print(f"| {name} | {g('warm_nodes'):.0f} | {g('warm_mismatch'):.0f} | {g('warm_cold_iter'):.0f} | {g('warm_iter'):.0f} | "
              f"{g('warm_cold_iter') / max(g('warm_iter'), 1):.1f} | {g('warm_cold_lp'):.3f} | {g('warm_lp'):.3f} | "
              f"{g('warm_cold_lp') / g('warm_lp'):.1f} | {1e3 * fixed:.3f} | {100 * g('wsolve') / wsum:.1f} |")


def self_test(d):
    tamper = [('prof2.txt', 'nodes=500 iters=', 'nodes=500 iters=1'), ('prof1.txt', 'PROFILE presolve=', 'XPROFILE presolve=')]
    for f, a, b in tamper:
        with tempfile.TemporaryDirectory() as t:
            t = Path(t)
            for p in d.glob('*.txt'):
                shutil.copy(p, t)
            text = (t / f).read_text()
            assert a in text, a
            (t / f).write_text(text.replace(a, b, 1))
            try:
                load(t)
            except (SystemExit, ValueError, KeyError) as e:
                print(f'self-test: {f} {b!r} rejected ({e})')
                continue
            raise SystemExit(f'FAIL self-test: tampered {f} accepted')


def main():
    d = Path(sys.argv[1])
    base, reps = load(d)
    if '--self-test' in sys.argv:
        self_test(d)
    report(base, reps)


if __name__ == '__main__':
    main()
