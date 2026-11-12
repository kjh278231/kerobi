"""M3-04c-21a: 자연/대조군을 Gurobi 단일 프로세스로 순차 실행(합성 없음)."""
import json
from pathlib import Path
import subprocess
import sys

ROOT=Path(__file__).resolve().parents[1]
D=Path('bench/oracle/m3_push_ftran')
CASES={
 'arms':('bench/oracle/m3_push_arms/arms.lp','bench/oracle/m3_push_arms/arms.extra.json'),
 'retry':('bench/oracle/m3_push_retry/retry.lp','bench/oracle/m3_push_retry/retry.extra.json'),
 'interior':('bench/oracle/m3_primal_push_step/interior.lp',None),
 'control':('bench/oracle/m3_push_arms/arms.lp','bench/oracle/m3_push_arms/arms.extra.json')}


def run(outdir,names=None,reps=(1,2,3)):
 outdir.mkdir(parents=True,exist_ok=True)
 for name,(model,extra) in CASES.items():
  if names and name not in names:continue
  for i in reps:
   out=outdir/f'{name}-{i}.json'
   if out.exists():raise FileExistsError(out)
   subprocess.run(['tools/gurun.sh','node','tools/push_ftran_capture.js',
       'control' if name=='control' else 'primal',model,str(out),
       (ROOT/extra).read_text() if extra else '{}'],cwd=ROOT,check=True)

if __name__=='__main__':
 # 탐색: python3 tools/push_ftran_runs.py OUT --explore (각 사례 1회)
 explore='--explore' in sys.argv
 args=[a for a in sys.argv[1:] if a!='--explore']
 run(Path(args[0]) if args else D,reps=(1,) if explore else (1,2,3))
