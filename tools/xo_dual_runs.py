#!/usr/bin/env python3
"""M3-04f-4a: crossover 쌍대값 251b4f·BTRAN 캡처 실행기(xo_cleanup_runs.py와 같은 사례 정의).

  python3 tools/xo_dual_runs.py --reps 3                    # 정식 Netlib: bench/oracle/m3_xo_dual/x<C>/<모델>-<r>.json.gz
  python3 tools/xo_dual_runs.py --cases --reps 3            # 자연 특수(arms·retry·interior × Crossover 5) + 04f-2 합성(synthetic.json) → cases/
  python3 tools/xo_dual_runs.py --out DIR --reps 1 --survey [--models a,b] [--crossover 3]   # factor 생략 호출 수 조사(기준 자료 아님)

기존 출력은 건너뛴다(덮어쓰지 않음). Gurobi는 tools/gurun.sh로 한 프로세스씩 순차 실행한다.
"""
import argparse, gzip, json, os, subprocess, sys
from pathlib import Path

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import push_init_runs as pir  # noqa: E402
import xo_cleanup_runs as xcr  # noqa: E402

ROOT = pir.ROOT
D = ROOT / 'bench/oracle/m3_xo_dual'
CROSSOVER = xcr.CROSSOVER


def capture(cross, model, out, extra='{}', config=None):
    r = subprocess.run(['tools/gurun.sh', 'node', 'tools/xo_dual_capture.js', f'x{cross}', str(model), str(out), extra, json.dumps(config or {})],
                       cwd=ROOT, capture_output=True, text=True)
    print(r.stdout.strip()[:200], flush=True)
    if r.returncode:
        raise subprocess.CalledProcessError(r.returncode, r.args, r.stdout, r.stderr)
    Path(str(out) + '.gz').write_bytes(gzip.compress(out.read_bytes(), mtime=0))
    out.unlink()


def done(out):
    return any(Path(str(out) + s).exists() for s in ('', '.gz', '.failed'))


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--out')
    ap.add_argument('--models')
    ap.add_argument('--crossover')
    ap.add_argument('--reps', type=int, default=3)
    ap.add_argument('--cases', action='store_true')
    ap.add_argument('--survey', action='store_true')
    a = ap.parse_args()
    extra_cfg = {'factor': False} if a.survey else {}
    fails = 0
    if a.cases:
        outdir = (Path(a.out) if a.out else D) / 'cases'
        outdir.mkdir(parents=True, exist_ok=True)
        for r in range(1, a.reps + 1):
            for name, (model, extra, c, config) in xcr.cases().items():
                out = outdir / f'{name}-{r}.json'
                if done(out):
                    continue
                try:
                    capture(c, model, out, extra, {**config, **extra_cfg})
                except subprocess.CalledProcessError as e:
                    fails += 1
                    print('실패', name, r, e.returncode, flush=True)
        return 1 if fails else 0
    base = Path(a.out) if a.out else D
    crosses = [int(c) for c in a.crossover.split(',')] if a.crossover else CROSSOVER
    models = a.models.split(',') if a.models else pir.SURVEY45
    for r in range(1, a.reps + 1):
        for c in crosses:
            outdir = base / f'x{c}'
            outdir.mkdir(parents=True, exist_ok=True)
            for name in models:
                out = outdir / f'{name}-{r}.json'
                if done(out):
                    continue
                try:
                    capture(c, pir.NETLIB / f'{name}.mps.gz', out, '{}', extra_cfg)
                except subprocess.CalledProcessError as e:
                    fails += 1
                    print('실패', c, name, r, e.returncode, flush=True)
    return 1 if fails else 0


if __name__ == '__main__':
    sys.exit(main())
