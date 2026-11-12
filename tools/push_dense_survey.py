#!/usr/bin/env python3
"""M3-04c-21i 조사: Netlib 자연 push(Crossover=3, 21e·04f-1과 같은 설정)에서 036ade 조밀 전환 경로가 언제 도는지 센다.

  python3 tools/push_dense_survey.py OUT [--models a,b] [--timeout 600]   # 모델당 1회, OUT/<모델>.json
  python3 tools/push_dense_survey.py OUT --summary                        # 요약(push 재분해·push 밖 나눔)

탐색 도구다. 정식 근거는 같은 경로를 전체 상태로 캡처한 21i 오라클(push_dense_*)이 맡는다.
"""
import argparse, json, subprocess, sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
NETLIB = (ROOT / '../re/bench/data/netlib').resolve()


def run(out, models, timeout):
    out.mkdir(parents=True, exist_ok=True)
    for name in models:
        dst = out / f'{name}.json'
        if dst.exists() or Path(str(dst) + '.failed').exists():
            continue
        try:
            r = subprocess.run(['tools/gurun.sh', 'node', 'tools/push_dense_survey_capture.js', 'primal',
                                str(NETLIB / f'{name}.mps.gz'), str(dst), '{}'], cwd=ROOT, timeout=timeout,
                               capture_output=True, text=True)
            print(name, r.returncode, r.stdout.strip()[:80], flush=True)
        except subprocess.TimeoutExpired:
            print(name, '시간 초과', flush=True)
            subprocess.run(['pkill', '-f', 'push_dense_survey_capture.js'])


def summary(out):
    for p in sorted(out.glob('*.json')):
        d = json.loads(p.read_text())
        ev = [e for e in d['events'] if e['event'] == 'lu:dense']
        if not ev:
            continue
        for e in ev:
            tag = 'push' if e['push'] is not None and e['ret'] == 0xf34f2 else 'other'
            print(p.stem, tag, 'push#', e['push'], 'id', e['id'], 'ret', hex(e['ret']), 'm', e['m'], 'rv', e['rv'], e['marks'])


if __name__ == '__main__':
    ap = argparse.ArgumentParser()
    ap.add_argument('out', type=Path)
    ap.add_argument('--models')
    ap.add_argument('--timeout', type=int, default=600)
    ap.add_argument('--summary', action='store_true')
    a = ap.parse_args()
    if a.summary:
        summary(a.out)
    else:
        models = a.models.split(',') if a.models else sorted(p.name[:-7] for p in NETLIB.glob('*.mps.gz'))
        run(a.out, models, a.timeout)
