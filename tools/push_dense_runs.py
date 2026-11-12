#!/usr/bin/env python3
"""M3-04c-21i: 조밀 전환 경계 캡처(push_dense_agent.js)를 Gurobi 단일 프로세스로 순차 실행한다.

  python3 tools/push_dense_runs.py                 # 조밀 5모델 × 독립 3회 → bench/oracle/m3_push_dense/<모델>-<회>.json.gz
  python3 tools/push_dense_runs.py --controls      # 21e/21h 7사례 × 1회 → .../controls/<사례>-1.json.gz (비간섭 대조군)
  python3 tools/push_dense_runs.py --explore OUT [--models a,b]   # 탐색(기준 자료 아님)

모든 출력은 결정적 gzip(mtime=0)이며 내용은 push_dense_capture.js 출력 그대로다. 기존 출력은 덮어쓰지 않는다.
"""
import argparse, gzip, subprocess, sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import push_lu_runs  # noqa: E402

ROOT = Path(__file__).resolve().parents[1]
D = ROOT / 'bench/oracle/m3_push_dense'
NETLIB = Path('../re/bench/data/netlib')
MODELS = ('grow7', 'grow22', 'perold', 'pilot4', 'stair')


def capture(kind, model, out, extra):
    gz = Path(str(out) + '.gz')
    if out.exists() or gz.exists() or Path(str(out) + '.failed').exists():
        raise FileExistsError(out)
    subprocess.run(['tools/gurun.sh', 'node', 'tools/push_dense_capture.js', kind, str(model), str(out), extra],
                   cwd=ROOT, check=True)
    gz.write_bytes(gzip.compress(out.read_bytes(), mtime=0))
    out.unlink()


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--controls', action='store_true')
    ap.add_argument('--explore', type=Path)
    ap.add_argument('--models')
    a = ap.parse_args()
    if a.controls:
        out = D / 'controls'
        out.mkdir(parents=True, exist_ok=True)
        for name, (model, extra) in push_lu_runs.CASES.items():
            capture('control' if name == 'control' else 'primal', model, out / f'{name}-1.json',
                    (ROOT / extra).read_text() if extra else '{}')
        return
    out = a.explore or D
    out.mkdir(parents=True, exist_ok=True)
    reps = (1,) if a.explore else (1, 2, 3)
    for name in (a.models.split(',') if a.models else MODELS):
        for r in reps:
            capture('primal', NETLIB / f'{name}.mps.gz', out / f'{name}-{r}.json', '{}')


if __name__ == '__main__':
    main()
