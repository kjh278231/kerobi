"""M3-04c-13: 자연/대조군/합성을 Gurobi 단일 프로세스로 순차 실행."""
import json
from pathlib import Path
import subprocess
import sys

ROOT=Path(__file__).resolve().parents[1]
D=Path('bench/oracle/m3_push_supplier')
CASES={
 'arms':('bench/oracle/m3_push_arms/arms.lp','bench/oracle/m3_push_arms/arms.extra.json'),
 'retry':('bench/oracle/m3_push_retry/retry.lp','bench/oracle/m3_push_retry/retry.extra.json'),
 'interior':('bench/oracle/m3_primal_push_step/interior.lp',None),
 'control':('bench/oracle/m3_push_arms/arms.lp','bench/oracle/m3_push_arms/arms.extra.json')}


def run(outdir):
 outdir.mkdir(parents=True,exist_ok=True)
 for name,(model,extra) in CASES.items():
  for i in (1,2,3):
   out=outdir/f'{name}-{i}.json'
   if out.exists():raise FileExistsError(out)
   subprocess.run(['tools/gurun.sh','node','tools/push_supplier_capture.js',
       'control' if name=='control' else 'primal',model,str(out),
       (ROOT/extra).read_text() if extra else '{}'],cwd=ROOT,check=True)
 for i in (1,2,3):
  subprocess.run(['tools/gurun.sh','../.venv/bin/python','tools/push_supplier_synthetic.py',
      '--capture',str(outdir/f'synthetic-{i}.json')],cwd=ROOT,check=True)


if __name__=='__main__':
 run(Path(sys.argv[1]) if len(sys.argv)>1 else D)
