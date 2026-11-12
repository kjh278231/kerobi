"""M3-04c-21h: 21e와 같은 7사례를 블록 상태 관측 agent로 Gurobi 단일 프로세스 순차 실행(합성 없음)."""
from pathlib import Path
import sys

sys.path.insert(0, str(Path(__file__).resolve().parent))
import push_lu_runs  # noqa: E402

D = Path('bench/oracle/m3_push_block')

if __name__ == '__main__':
    # 탐색: python3 tools/push_block_runs.py OUT --explore [--only=capri]
    explore = '--explore' in sys.argv
    names = [a[7:] for a in sys.argv[1:] if a.startswith('--only=')]
    args = [a for a in sys.argv[1:] if not a.startswith('--')]
    push_lu_runs.run(Path(args[0]) if args else D, names=names[0].split(',') if names else None,
                     reps=(1,) if explore else (1, 2, 3), tool='tools/push_block_capture.js')
