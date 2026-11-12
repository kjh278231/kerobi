#!/usr/bin/env python3
"""M3-04c-1c: 조건수 116fd7 내부 경계 캡처 실행기. 사례·설정은 M3-04c-1b(push_init_runs.cases)와 같다.

  python3 tools/push_cond_runs.py                     # 15사례 × 3회(기존 출력 덮어쓰기 거부)
  python3 tools/push_cond_runs.py --only=arms,adjust  # 일부 사례
  python3 tools/push_cond_runs.py --explore OUT       # 각 사례 1회(탐색, 기준 자료 아님)
  python3 tools/push_cond_runs.py --netlib [--models a,b]   # Netlib 소형 45 자연(Crossover=3) × 3회 → D/netlib(gzip)
  python3 tools/push_cond_runs.py --netlib --out OUT --reps 1  # 탐색

Gurobi는 tools/gurun.sh로 한 프로세스씩 순차 실행한다.
"""
import argparse, gzip, json, os, subprocess, sys
from pathlib import Path

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import push_init_runs as pir  # noqa: E402

ROOT = pir.ROOT
D = ROOT / 'bench/oracle/m3_push_condition'


def capture(kind, model, out, extra, config, gz=False):
    if out.exists() or Path(str(out) + '.gz').exists():
        raise FileExistsError(out)
    subprocess.run(['tools/gurun.sh', 'node', 'tools/push_cond_capture.js', kind, str(model), str(out),
                    (ROOT / extra).read_text() if extra else '{}', json.dumps(config or {})], cwd=ROOT, check=True)
    if gz or out.stat().st_size > 1 << 20:
        Path(str(out) + '.gz').write_bytes(gzip.compress(out.read_bytes(), mtime=0)); out.unlink()


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--only')
    ap.add_argument('--explore')
    ap.add_argument('--netlib', action='store_true')
    ap.add_argument('--out')
    ap.add_argument('--models')
    ap.add_argument('--reps', type=int, default=3)
    a = ap.parse_args()
    if a.netlib:
        outdir = Path(a.out) if a.out else D / 'netlib'
        outdir.mkdir(parents=True, exist_ok=True)
        for name in (a.models.split(',') if a.models else pir.SURVEY45):
            for r in range(1, a.reps + 1):
                out = outdir / f'{name}-{r}.json'
                if out.exists() or Path(str(out) + '.gz').exists() or Path(str(out) + '.failed').exists():
                    continue
                try:
                    capture('primal', pir.NETLIB / f'{name}.mps.gz', out, None, {}, gz=True)
                except subprocess.CalledProcessError as e:
                    print('실패', name, e.returncode, flush=True)
        return
    outdir = Path(a.explore) if a.explore else D
    outdir.mkdir(parents=True, exist_ok=True)
    reps = (1,) if a.explore else (1, 2, 3)
    names = a.only.split(',') if a.only else None
    for name, (model, extra, config, kind) in pir.cases().items():
        if names and name not in names:
            continue
        for r in reps:
            capture(kind, ROOT / model, outdir / f'{name}-{r}.json', extra, config)


if __name__ == '__main__':
    main()
