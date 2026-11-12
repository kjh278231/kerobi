"""M3-04c-21e: 21c와 같은 자연/대조군 7사례를 Gurobi 단일 프로세스로 순차 실행(합성 없음)."""
import gzip
import json
from pathlib import Path
import subprocess
import sys

ROOT=Path(__file__).resolve().parents[1]
D=Path('bench/oracle/m3_push_lu_chain')
CASES={
 'arms':('bench/oracle/m3_push_arms/arms.lp','bench/oracle/m3_push_arms/arms.extra.json'),
 'retry':('bench/oracle/m3_push_retry/retry.lp','bench/oracle/m3_push_retry/retry.extra.json'),
 'interior':('bench/oracle/m3_primal_push_step/interior.lp',None),
 'control':('bench/oracle/m3_push_arms/arms.lp','bench/oracle/m3_push_arms/arms.extra.json'),
 # 저장소 밖 Netlib(해시는 manifest). 조사 history/survey-netlib45.json에서 지원 BTRAN 경로를 고른 모델.
 'adlittle':('../re/bench/data/netlib/adlittle.mps.gz',None),
 'scsd1':('../re/bench/data/netlib/scsd1.mps.gz',None),
 'capri':('../re/bench/data/netlib/capri.mps.gz',None)}
NETLIB=('adlittle','scsd1','capri')


def run(outdir,names=None,reps=(1,2,3),tool='tools/push_lu_capture.js'):
 global TOOL;TOOL=tool
 outdir.mkdir(parents=True,exist_ok=True)
 for name,(model,extra) in CASES.items():
  if names and name not in names:continue
  for i in reps:
   out=outdir/f'{name}-{i}.json';gz=outdir/f'{name}-{i}.json.gz'
   if out.exists() or gz.exists():raise FileExistsError(out)
   subprocess.run(['tools/gurun.sh','node',TOOL,
       'control' if name=='control' else 'primal',model,str(out),
       (ROOT/extra).read_text() if extra else '{}'],cwd=ROOT,check=True)
   if name in NETLIB:
    # Netlib 캡처는 크므로 결정적 gzip(mtime=0)으로 저장한다. 내용은 capture.js 출력 그대로다.
    (ROOT/gz).write_bytes(gzip.compress((ROOT/out).read_bytes(),mtime=0));(ROOT/out).unlink()

if __name__=='__main__':
 # 탐색: python3 tools/push_lu_runs.py OUT --explore (각 사례 1회). 비간섭 기준선은 21c 정식 캡처를 쓴다.
 explore='--explore' in sys.argv
 names=[a[7:] for a in sys.argv[1:] if a.startswith('--only=')]
 args=[a for a in sys.argv[1:] if not a.startswith('--')]
 run(Path(args[0]) if args else D,names=names[0].split(',') if names else None,reps=(1,) if explore else (1,2,3))
