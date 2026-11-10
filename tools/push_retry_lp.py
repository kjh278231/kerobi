"""M3-04c-7: 위반 basis에서 054eef가 작은 pivot(|pivot|<0.001)을 돌려 push가 056973 재시도를 부르게 만든 블록 대각 LP.

형식과 build는 push_violated_lp.py(M3-04c-2)를 그대로 쓴다. 그 파일은 04c-2 manifest가 해시로 고정하므로 고치지 않고
묶음만 이 파일에 둔다. 묶음별 의도는 docs/m3/primal_push_retry.md §4.
"""
import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import push_violated_lp as base  # noqa: E402

# (이름, lb, ub, 목적 계수, 시작값), (행, {변수: 계수}, sense, rhs 또는 None, gap). crash는 내부에서 시작한 z를 basic으로 고르고,
# '='의 gap>0이면 z가 하한 아래(B), gap<0이면 상한 위(A)로 다시 계산된다. v는 LP를 feasible로 두는 완충 열이다. 계수는 모두 1e-3보다 작아 054eef pivot이 재시도 게이트에 걸린다.
GADGETS = {
    # 기울기 0(N 행만): t=0 선택이 없어 최소 ratio로 간다. 054eef는 Harris 안의 큰 |a|(행 b)를, 056973은 최소 ratio(행 a)를 고른다
    'TF': ([('xf', 0, 10, 0, 5), ('yfa', 0, 10, 0, 0), ('yfb', 0, 10, 0, 0)],
           [('tfa', {'xf': 1e-4, 'yfa': 1}, '<=', None, 5e-5), ('tfb', {'xf': 5e-4, 'yfb': 1}, '<=', None, 2.5e-4 + 5e-10)]),
    # B 두 행: 기울기 -9e-4. 첫 B 행의 상한 breakpoint(0.6)에서 기울기가 0 이상이 되어 B(a<0) → -2
    'TB2': ([('xb2', 0, 10, 0, 5), ('z1', 0, 2e-4, 0, 1e-4), ('z2', 0, 10, 0, 5), ('v2', 0, 10, 0, 0)],
            [('e1', {'z1': 1, 'xb2': 5e-4}, '=', None, 2e-4), ('e2', {'z2': 1, 'xb2': 4e-4, 'v2': -1}, '=', None, 5 + 4e-4)]),
    # A 두 행: 같은 구조의 거울. 첫 A 행의 하한 breakpoint(0.6)에서 A(a>=0) → -1
    'TA2': ([('xa2', 0, 10, 0, 5), ('z3', 0, 2e-4, 0, 1e-4), ('z4', 0, 10, 0, 5), ('v4', 0, 10, 0, 0)],
            [('e3', {'z3': 1, 'xa2': 5e-4}, '=', None, -2e-4), ('e4', {'z4': 1, 'xa2': 4e-4, 'v4': 1}, '=', None, -(5 + 4e-4))]),
    # basic slack이 하한(0)에 붙은 행: 기울기 +a라 t=0, 2단계 L(a>0) 잔차 |x|<=h로 고른다
    'TZ': ([('xz', 0, 10, 0, 5), ('yz', 0, 10, 0, 0)], [('tz', {'xz': 5e-4, 'yz': 1}, '<=', None, 0.0)]),
    # 재시도 결과 pivot이 1e-6보다 작아 보류된다(위반 경로 보류는 056973 뒤에만 온다)
    'TD': ([('xd', 0, 10, 0, 5), ('yd', 0, 10, 0, 0)], [('td', {'xd': 5e-7, 'yd': 1}, '<=', None, 2.5e-7)]),
}
base.GADGETS.update(GADGETS)
# SR(04c-2): 차단 행이 없어 054eef·056973 모두 -1, theta=I, 방향 1이라 재호출 없이 거리 flip
TARGET = ['W', 'TF', 'TB2', 'TA2', 'TZ', 'TD', 'SR']

if __name__ == '__main__':
    out = Path(sys.argv[1])
    names = sys.argv[2].split(',') if len(sys.argv) > 2 else TARGET
    lp, extra = base.build(names)
    for path, text in ((out.with_suffix('.lp'), lp), (out.with_suffix('.extra.json'), json.dumps(extra) + '\n')):
        assert not path.exists() or path.read_text() == text, f'기존 입력과 다름: {path}'
        path.write_text(text)
    print(out.with_suffix('.lp'), out.with_suffix('.extra.json'))
