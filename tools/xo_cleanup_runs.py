#!/usr/bin/env python3
"""M3-04f-2: crossover cleanup 진입 캡처 실행기. Netlib 소형 45(push_init_runs.SURVEY45) × Crossover {-1,1,2,3,4}.

  python3 tools/xo_cleanup_runs.py --reps 3                    # 정식: bench/oracle/m3_xo_cleanup/x<C>/<모델>-<r>.json.gz
  python3 tools/xo_cleanup_runs.py --out DIR --reps 1 [--models a,b] [--crossover 3]   # 탐색(기준 자료 아님)
  python3 tools/xo_cleanup_runs.py --cases --reps 3   # 자연 특수(arms·retry·interior × Crossover 5) + 합성(synthetic.json) → cases/

기존 출력은 건너뛴다(덮어쓰지 않음). Gurobi는 tools/gurun.sh로 한 프로세스씩 순차 실행한다.
"""
import argparse, gzip, json, os, subprocess, sys
from pathlib import Path

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import push_init_runs as pir  # noqa: E402

ROOT = pir.ROOT
D = ROOT / 'bench/oracle/m3_xo_cleanup'
CROSSOVER = (-1, 1, 2, 3, 4)


# 1b 자연 사례(push_init_runs.NATURAL 중 Netlib 45 밖): remaining > 0 자연 경로(arms·retry, 04f-1 자료).
SPECIAL = {k: pir.NATURAL[k] for k in ('arms', 'retry', 'interior')}


def cases():
    out = {}
    for name, (model, extra) in SPECIAL.items():
        for c in CROSSOVER:
            out[f'{name}_x{c}'] = (ROOT / model, (ROOT / extra).read_text() if extra else '{}', c, {})
    for name, spec in json.loads((D / 'synthetic.json').read_text()).items():
        if not name.startswith('_'):
            out[name] = (pir.NETLIB / f"{spec['model']}.mps.gz", '{}', spec['crossover'], {'edits': spec['edits']})
    return out


def capture(cross, model, out, extra='{}', config=None):
    subprocess.run(['tools/gurun.sh', 'node', 'tools/xo_cleanup_capture.js', f'x{cross}', str(model), str(out), extra, json.dumps(config or {})],
                   cwd=ROOT, check=True)
    Path(str(out) + '.gz').write_bytes(gzip.compress(out.read_bytes(), mtime=0))
    out.unlink()


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--out')
    ap.add_argument('--models')
    ap.add_argument('--crossover')
    ap.add_argument('--reps', type=int, default=3)
    ap.add_argument('--cases', action='store_true')
    a = ap.parse_args()
    if a.cases:
        outdir = (Path(a.out) if a.out else D) / 'cases'
        outdir.mkdir(parents=True, exist_ok=True)
        for r in range(1, a.reps + 1):
            for name, (model, extra, c, config) in cases().items():
                out = outdir / f'{name}-{r}.json'
                if any(Path(str(out) + s).exists() for s in ('', '.gz', '.failed')):
                    continue
                try:
                    capture(c, model, out, extra, config)
                except subprocess.CalledProcessError as e:
                    print('실패', name, r, e.returncode, flush=True)
        return
    base = Path(a.out) if a.out else D
    crosses = [int(c) for c in a.crossover.split(',')] if a.crossover else CROSSOVER
    models = a.models.split(',') if a.models else pir.SURVEY45
    for r in range(1, a.reps + 1):
        for c in crosses:
            outdir = base / f'x{c}'
            outdir.mkdir(parents=True, exist_ok=True)
            for name in models:
                out = outdir / f'{name}-{r}.json'
                if any(Path(str(out) + s).exists() for s in ('', '.gz', '.failed')):
                    continue
                try:
                    capture(c, pir.NETLIB / f'{name}.mps.gz', out)
                except subprocess.CalledProcessError as e:
                    print('실패', c, name, r, e.returncode, flush=True)


if __name__ == '__main__':
    main()
