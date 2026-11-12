#!/usr/bin/env python3
"""M3-04c-1b: push 초기화 중간 경계 캡처 실행기. Gurobi는 한 프로세스씩 순차 실행한다.

  python3 tools/push_init_runs.py                       # 정식 사례 전체 3회(기존 출력 덮어쓰기 거부)
  python3 tools/push_init_runs.py --only=arms,adjust    # 일부 사례
  python3 tools/push_init_runs.py --explore OUT         # 각 사례 1회(탐색, 기준 자료 아님)
  python3 tools/push_init_runs.py --survey OUT --crossover 3   # Netlib 소형 45 자연 탐색(요약 agent)
"""
import argparse, gzip, json, subprocess, sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
D = ROOT / 'bench/oracle/m3_push_init'
NETLIB = ROOT / '../re/bench/data/netlib'
# 21h(push_lu_runs.CASES)와 같은 자연 6사례.
NATURAL = {
    'arms': ('bench/oracle/m3_push_arms/arms.lp', 'bench/oracle/m3_push_arms/arms.extra.json'),
    'retry': ('bench/oracle/m3_push_retry/retry.lp', 'bench/oracle/m3_push_retry/retry.extra.json'),
    'interior': ('bench/oracle/m3_primal_push_step/interior.lp', None),
    'adlittle': ('../re/bench/data/netlib/adlittle.mps.gz', None),
    'scsd1': ('../re/bench/data/netlib/scsd1.mps.gz', None),
    'capri': ('../re/bench/data/netlib/capri.mps.gz', None),
}
# 자연 탐색에서 찾은 first=0(Crossover=1, dual 뒤 primal push) 사례.
DUAL = {'adlittle_x1': ('../re/bench/data/netlib/adlittle.mps.gz', None)}
# 합성 사례: (기반 사례, 편집 목록). 편집 값은 탐색(§6)으로 고정한다.
SYNTHETIC = json.loads((ROOT / 'bench/oracle/m3_push_init/synthetic.json').read_text()) \
    if (ROOT / 'bench/oracle/m3_push_init/synthetic.json').exists() else {}
SURVEY45 = ['afiro', 'sc50b', 'sc50a', 'sc105', 'kb2', 'stocfor1', 'adlittle', 'blend', 'scagr7', 'sc205', 'share2b',
            'recipe', 'lotfi', 'vtpbase', 'share1b', 'boeing2', 'scorpion', 'bore3d', 'sctap1', 'scagr25', 'israel',
            'brandy', 'scsd1', 'standata', 'capri', 'standgub', 'grow7', 'etamacro', 'scfxm1', 'e226', 'beaconfd',
            'standmps', 'finnis', 'agg', 'bandm', 'forplan', 'gfrd-pnc', 'degen2', 'scsd6', 'boeing1', 'modszk1',
            'scrs8', 'tuff', 'shell', 'ship04s']


def cases():
    out = {k: (m, e, None, 'primal') for k, (m, e) in NATURAL.items()}
    out.update({k: (m, e, None, 'dual') for k, (m, e) in DUAL.items()})
    for name, spec in SYNTHETIC.items():
        if name.startswith('_'):
            continue
        m, e = NATURAL[spec['base']] if spec['base'] in NATURAL else (spec['model'], spec.get('extra'))
        out[name] = (m, e, {'edits': spec['edits']}, 'primal')
    return out


def capture(kind, model, out, extra, config):
    if out.exists() or Path(str(out) + '.gz').exists():
        raise FileExistsError(out)
    subprocess.run(['tools/gurun.sh', 'node', 'tools/push_init_capture.js', kind, str(model), str(out),
                    (ROOT / extra).read_text() if extra else '{}', json.dumps(config or {})], cwd=ROOT, check=True)
    if out.stat().st_size > 1 << 20:
        # 큰 캡처는 결정적 gzip(mtime=0). 내용은 capture.js 출력 그대로다.
        Path(str(out) + '.gz').write_bytes(gzip.compress(out.read_bytes(), mtime=0)); out.unlink()


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--only')
    ap.add_argument('--explore')
    ap.add_argument('--survey')
    ap.add_argument('--crossover', type=int, default=3)
    ap.add_argument('--models')
    a = ap.parse_args()
    if a.survey:
        outdir = Path(a.survey); outdir.mkdir(parents=True, exist_ok=True)
        kind = {3: 'primal', 1: 'dual'}[a.crossover]
        for name in (a.models.split(',') if a.models else SURVEY45):
            out = outdir / f'{name}-x{a.crossover}.json'
            if out.exists() or (outdir / f'{name}-x{a.crossover}.json.failed').exists():
                continue
            try:
                capture(kind, NETLIB / f'{name}.mps.gz', out, None, {'survey': True})
            except subprocess.CalledProcessError as e:
                print('실패', name, e.returncode, flush=True)
        return
    outdir = Path(a.explore) if a.explore else D
    outdir.mkdir(parents=True, exist_ok=True)
    reps = (1,) if a.explore else (1, 2, 3)
    names = a.only.split(',') if a.only else None
    for name, (model, extra, config, kind) in cases().items():
        if names and name not in names:
            continue
        for r in reps:
            capture(kind, ROOT / model, outdir / f'{name}-{r}.json', extra, config)


if __name__ == '__main__':
    main()
