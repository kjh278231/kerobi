"""M3-04c-8: theta=I 뒤 반대 방향 재호출(push C 1458-1590)을 밟도록 만든 블록 대각 LP와 시작 벡터.

형식과 build는 push_violated_lp.py(M3-04c-2)를 그대로 쓴다. 그 파일은 04c-2 manifest가 해시로 고정하므로 고치지 않고
묶음만 이 파일에 둔다. 묶음별 의도는 docs/m3/primal_push_recall.md §4.
"""
import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import push_violated_lp as base  # noqa: E402

I = 1e30  # LP 파일에서 |bound|>=1e30은 무한이다
# (이름, lb, ub, 목적 계수, 시작값), (행, {변수: 계수}, sense, rhs 또는 None, gap). '<=' 행의 slack은 gap에서 시작해 basic이다.
# 계수 -1인 N 행은 진입 열이 늘 때 slack이 늘어 막지 않고, 줄 때 gap에서 막는다. +1이면 반대다.
GADGETS = {}
for p in ('F', 'V'):
    GADGETS.update({
        # 자유 열, r=0 → 방향 0. 위로는 막는 행이 없어 -1, 자유라 재호출 → 방향 1에서 N 행이 막아 pivot
        f'{p}R': ([(f'x{p}R', -I, I, 0, 2), (f'y{p}R', 0, 10, 0, 0)], [(f'r{p}R', {f'x{p}R': -1, f'y{p}R': 1}, '<=', None, 1.0)]),
        # 자유 열, 빈 열: 두 방향 모두 -1. 재호출 뒤에도 theta=I·자유라 leave -1, flip은 자유 arm(-3)
        f'{p}N': ([(f'x{p}N', -I, I, 0, 3)], []),
        # lb 유한·ub=I, r=0 → 방향 1. 아래로 막는 행이 없어 -1이지만 방향 1·자유 아님이라 재호출 없이 거리 flip(-1)
        f'{p}U': ([(f'x{p}U', 0, I, 0, 3), (f'y{p}U', 0, 10, 0, 0)], [(f'r{p}U', {f'x{p}U': 1, f'y{p}U': 1}, '<=', None, 1.0)]),
    })
GADGETS.update({
    # feasible: lb 유한·ub=I, r=-1 → 방향 0. 위쪽은 |a|=1e-8(허용오차 미만) 행만 있어 -1 → 재호출(방향 0·ub=I) → 방향 1 pivot.
    # 1e-8 행이 x를 1e8 근처에서 막으므로 LP는 유계다
    'FD': ([('xFD', 0, I, -1, 3), ('yFD', 0, 10, 0, 0), ('zFD', 0, 10, 0, 0)],
           [('rFD', {'xFD': -1, 'yFD': 1}, '<=', None, 1.0), ('tFD', {'xFD': 1e-8, 'zFD': 1}, '<=', None, 1.0)]),
    # 위반: 자기 B 행 계수 -2e-8(σ=-2e-8, |a|<허용오차) → 방향 0(σ 부호 절). 위쪽 -1 → 재호출(방향 0·ub=I, 자유 아님) → pivot
    'VD': ([('xVD', 0, I, 0, 3), ('yVD', 0, 10, 0, 0), ('wVD', 0, 1, 0, 1)],
           [('bVD', {'wVD': 1, 'xVD': -2e-8}, '<=', None, -0.05), ('rVD', {'xVD': -1, 'yVD': 1}, '<=', None, 1.0)]),
    # VD와 같되 dL=0.5 < 재호출 theta 1: 재호출 뒤 새 방향 1의 거리 검사로 flip(-1)
    'VDd': ([('xVDd', 0, I, 0, 0.5), ('yVDd', 0, 10, 0, 0), ('wVDd', 0, 1, 0, 1)],
            [('bVDd', {'wVDd': 1, 'xVDd': -2e-8}, '<=', None, -0.05), ('rVDd', {'xVDd': -1, 'yVDd': 1}, '<=', None, 1.0)]),
    # 위반: 자유 열, 자기 B 행 계수 +2e-8(σ=+2e-8) → 방향 1. 아래쪽 -1 → 자유라 재호출 → 방향 0 pivot(자유 절 단독 판별)
    'VS': ([('xVS', -I, I, 0, 2), ('yVS', 0, 10, 0, 0), ('wVS', 0, 1, 0, 1)],
           [('bVS', {'wVS': 1, 'xVS': 2e-8}, '<=', None, -0.05), ('rVS', {'xVS': 1, 'yVS': 1}, '<=', None, 1.0)]),
    # 위반: VR과 같되 계수 -5e-4 → 재호출의 054eef pivot이 0.001 미만이라 재호출 안에서 056973 게이트
    'VRg': ([('xVRg', -I, I, 0, 2), ('yVRg', 0, 10, 0, 0)], [('rVRg', {'xVRg': -5e-4, 'yVRg': 1}, '<=', None, 1e-4)]),
})
base.GADGETS.update(GADGETS)
TARGETS = {'feasible': ['FR', 'FN', 'FU', 'FD'], 'violated': ['W', 'VR', 'VN', 'VU', 'VD', 'VDd', 'VS', 'VRg']}

if __name__ == '__main__':
    out = Path(sys.argv[1])
    names = TARGETS[sys.argv[2]] if sys.argv[2] in TARGETS else sys.argv[2].split(',')
    lp, extra = base.build(names)
    for path, text in ((out.with_suffix('.lp'), lp), (out.with_suffix('.extra.json'), json.dumps(extra) + '\n')):
        assert not path.exists() or path.read_text() == text, f'기존 입력과 다름: {path}'
        path.write_text(text)
    print(out.with_suffix('.lp'), out.with_suffix('.extra.json'))
