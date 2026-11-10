"""M3-04c-2: 위반 basis(local_1b0>0)의 primal push 단계를 밟도록 만든 블록 대각 LP와 시작 벡터를 쓴다.

push_arm_lp.py(M3-04b-3)와 같은 형식이다. 그 파일은 04b-3 manifest가 해시로 고정하므로 고치지 않고 묶음만 새로 둔다.
시작값이 행을 위반하면 crash basis의 basic slack이 B·A로 분류돼 위반 경로로 간다. 묶음별 의도는
docs/m3/primal_push_violated.md §4.
"""
import json
import sys
from pathlib import Path

# (이름, lb, ub, 목적 계수, 시작값), (행, {변수: 계수}, sense, rhs 또는 None, gap). gap<0이면 시작값이 행을 위반한다.
GADGETS = {
    # 후보 없는 영구 위반 행(B). 이 행이 남는 동안 다른 묶음 단계도 위반 경로로 간다(σ=0).
    'W': ([('xw', 0, 1, 0, 1), ('yw', 0, 1, 0, 1)], [('cw', {'xw': 1, 'yw': 1}, '<=', None, -0.3)]),
    # |σ|<e 안의 bound 절: W 행(B)에 5e-9 계수를 걸어 σ=∓5e-9, dL·|σ| 또는 dU·|σ|가 e(1e-8) 이상이 되게 한다
    'S5': ([('x5', 0, 100, -1e-9, 5), ('y5', 0, 10, 0, 0), ('w5', 0, 1, 0, 1)],
           [('c5', {'w5': 1, 'x5': -5e-9}, '<=', None, -0.1), ('r5', {'x5': -1, 'y5': 1}, '<=', None, 1.0)]),
    'S6': ([('x6', 0, 100, 1e-9, 95), ('y6', 0, 10, 0, 0), ('w6', 0, 1, 0, 1)],
           [('c6', {'w6': 1, 'x6': 5e-9}, '<=', None, -0.1), ('r6', {'x6': 1, 'y6': 1}, '<=', None, 1.0)]),
    # 판별 묶음: 절 하나를 끄면 다음 절이 반대 방향을 내도록 자기 B 행에 σ를 건다(D1 σ=-1, D2 σ=+1, D3·D4 σ=∓2e-8)
    'D1': ([('xD1', 0, 10, 1, 5e-5), ('yD1', 0, 10, 0, 0), ('wD1', 0, 1, 0, 1)],
           [('bD1', {'wD1': 1, 'xD1': -1}, '<=', None, -0.05), ('rD1', {'xD1': -1, 'yD1': 1}, '<=', None, 1.0)]),
    'D2': ([('xD2', 0, 10, -1, 10 - 5e-5), ('yD2', 0, 10, 0, 0), ('wD2', 0, 1, 0, 1)],
           [('bD2', {'wD2': 1, 'xD2': 1}, '<=', None, -0.05), ('rD2', {'xD2': 1, 'yD2': 1}, '<=', None, 1.0)]),
    'D3': ([('xD3', 0, 10, 1e-5, 5e-5), ('yD3', 0, 10, 0, 0), ('wD3', 0, 1, 0, 1)],
           [('bD3', {'wD3': 1, 'xD3': -2e-8}, '<=', None, -0.05), ('rD3', {'xD3': -1, 'yD3': 1}, '<=', None, 1.0)]),
    'D4': ([('xD4', 0, 10, -1e-5, 10 - 5e-5), ('yD4', 0, 10, 0, 0), ('wD4', 0, 1, 0, 1)],
           [('bD4', {'wD4': 1, 'xD4': 2e-8}, '<=', None, -0.05), ('rD4', {'xD4': 1, 'yD4': 1}, '<=', None, 1.0)]),
    # dU/dL=9.5, r=0: 계수 10을 9로 바꾸면 σ 절(1), r>=0으로 바꾸면 r 절(1)이 원래 0을 뒤집는다
    'D7': ([('xD7', 0, 10.5, 0, 1), ('yD7', 0, 10, 0, 0)], [('rD7', {'xD7': 1, 'yD7': 1}, '<=', None, 1.0)]),
    'S4b': ([('x4b', 0, 10, -1e-5, 10 - 5e-5), ('y4b', 0, 10, 0, 0)], [('r4b', {'x4b': 1, 'y4b': 1}, '<=', None, 1.0)]),
    'S1': ([('x1', 0, 10, 1, 5e-5), ('y1', 0, 10, 0, 0)], [('r1', {'x1': -1, 'y1': 1}, '<=', None, 1.0)]),
    'S1r': ([('x1r', 0, 10, 1, 5e-5), ('y1r', 0, 10, 0, 0)], [('r1r', {'x1r': -1, 'y1r': 1}, '<=', None, 1e-5)]),
    'S2': ([('x2', 0, 10, -1, 10 - 5e-5), ('y2', 0, 10, 0, 0)], [('r2', {'x2': 1, 'y2': 1}, '<=', None, 1.0)]),
    'S3': ([('x3', 0, 10, 0, 5e-5), ('y3', 0, 10, 0, 0)], [('r3', {'x3': -1, 'y3': 1}, '<=', None, 1.0)]),
    'S4': ([('x4', 0, 10, 0, 10 - 5e-5), ('y4', 0, 10, 0, 0)], [('r4', {'x4': 1, 'y4': 1}, '<=', None, 1.0)]),
    'S7': ([('x7', 0, 10, 1, 5), ('y7', 0, 10, 0, 0)], [('r7', {'x7': -1, 'y7': 1}, '<=', None, 1.0)]),
    'S7n': ([('x7n', 0, 10, -1, 5), ('y7n', 0, 10, 0, 0)], [('r7n', {'x7n': 1, 'y7n': 1}, '<=', None, 1.0)]),
    'VB': ([('xv', 0, 10, 0, 5), ('yv', 0, 10, 0, 0)], [('cv', {'xv': 1, 'yv': 1}, '<=', None, -0.2)]),
    'S8n': ([('x8', 0, 10, 0, 5), ('z8', 0, 10, 0, 10)], [('r8', {'x8': -1, 'z8': 1}, '<=', None, -0.2)]),
    # 등식 행의 내부 구조 변수 중 crash가 고르는 쪽이 basic(3.2>ub 3, A). 둘 다 상한 3이라 어느 쪽이 골라져도 A다
    'SA': ([('pa', 0, 3, 0, 1), ('qa', 0, 10, 0, 0), ('ra', 0, 3, 0, 1)], [('ea', {'pa': 1, 'qa': 1, 'ra': 1}, '=', 4.2, 0.0)]),
    # 재시도 발동용(대상 LP에는 넣지 않는다): 차단 행이 없어 054eef가 -1이고 056973을 부른다
    'SR': ([('xr', 0, 10, 1, 5)], [('rr', {'xr': 1}, '<=', None, 100.0)]),
}
TARGET = ['W', 'S5', 'S6', 'D1', 'D2', 'D3', 'D4', 'D7', 'S4b', 'S1', 'S1r', 'S2', 'S3', 'S4', 'S7', 'S7n', 'VB', 'S8n', 'SA']
DUALS = {}
PARAMS = {'ScaleFlag': 0}


def build(names):
    vars_ = [v for k in names for v in GADGETS[k][0]]
    rows = [r for k in names for r in GADGETS[k][1]]
    start = {v[0]: v[4] for v in vars_}
    # 목적에 모든 변수를 적어 LP 읽기의 열 순서를 묶음 순서로 고정한다
    lines = ['Minimize', ' obj: ' + ' '.join(f'{c:+.17g} {n}' for n, _, _, c, _ in vars_), 'Subject To']
    for name, coefs, sense, rhs, gap in rows:
        if rhs is None:
            act = sum(a * start[v] for v, a in coefs.items())
            rhs = act + gap if sense == '<=' else act - gap
        lines.append(f' {name}: ' + ' '.join(f'{a:+.17g} {v}' for v, a in coefs.items()) + f' {sense} {rhs:.17g}')
    lines += ['Bounds', *(f' {lb:.17g} <= {n} <= {ub:.17g}' for n, lb, ub, _, _ in vars_), 'End']
    starts = {'PStart': [v[4] for v in vars_], 'DStart': [float(DUALS.get(r[0], 0.0)) for r in rows]}
    return '\n'.join(lines) + '\n', {'params': PARAMS, 'starts': starts}


if __name__ == '__main__':
    out = Path(sys.argv[1])  # 예: bench/oracle/m3_push_violated/violated
    names = sys.argv[2].split(',') if len(sys.argv) > 2 else TARGET
    lp, extra = build(names)
    for path, text in ((out.with_suffix('.lp'), lp), (out.with_suffix('.extra.json'), json.dumps(extra) + '\n')):
        assert not path.exists() or path.read_text() == text, f'기존 입력과 다름: {path}'
        path.write_text(text)
    print(out.with_suffix('.lp'), out.with_suffix('.extra.json'))
