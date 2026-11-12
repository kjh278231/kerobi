"""M3-04c-11: M3-04c-10 연쇄 재생에서 판별하지 못한 경계를 밟는 LP와 시작 벡터(PStart·DStart)를 쓴다.

push_arm_lp.py(M3-04b-3)·push_violated_lp.py(M3-04c-2)의 묶음을 가져와 쓴다. 두 파일은 manifest가 해시로 고정하므로
고치지 않는다. 사례와 의도는 docs/m3/primal_push_chain_oracle.md §4.

- shift: arm 묶음 전체의 모든 변수를 SHIFT만큼 옮긴다(lb·ub·시작값, 명시 rhs는 Σa·SHIFT를 더함). 진입·이탈 열 lb≠0.
  자유 묶음 FZ(자유 basic 행만 걸리는 자유 열, x≠0)를 더한다.
- resolve: 위반 묶음 중 해소되는 SA(A)와 feasible 묶음만 둔다(영구 위반 행 W·후보 없는 B 행 제외). 위반 행을 고치는 pivot 뒤 위반 수가 0이 되어 같은 구간 안에서 F 경로로 간다.
"""
import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import push_arm_lp as arm  # noqa: E402
import push_violated_lp as vio  # noqa: E402

SHIFT = 0.3
INF = 1e30
# 자유 열 f(시작 0.5)와 자유 z(시작 0.7)가 등식 행 하나만 공유한다. crash가 하나를 basic(F)으로 두면 남은 자유 열의
# ratio는 F 행만 보게 되어 양방향 -1 → theta=I 재호출 → -3 flip(이동량 ±x, 비어 있지 않은 열)으로 간다.
FREE = {'FZ': ([('fz', -INF, INF, 0, 0.5), ('zz', -INF, INF, 0, 0.7)], [('ez', {'fz': 1, 'zz': 1}, '=', None, 0.0)])}


RESOLVE = ['SA', 'S1', 'S2', 'S3', 'S4', 'S7', 'S7n', 'S4b', 'D7']


def shifted(gadgets, s):
    out = {}
    for k, (vars_, rows) in gadgets.items():
        nv = [(n, lb + s, ub + s, c, x + s) for n, lb, ub, c, x in vars_]
        nr = [(r, co, sense, None if rhs is None else rhs + sum(a * s for a in co.values()), gap)
              for r, co, sense, rhs, gap in rows]
        out[k] = (nv, nr)
    return out


def build(gadgets, names, duals, params):
    vars_ = [v for k in names for v in gadgets[k][0]]
    rows = [r for k in names for r in gadgets[k][1]]
    start = {v[0]: v[4] for v in vars_}
    lines = ['Minimize', ' obj: ' + ' '.join(f'{c:+.17g} {n}' for n, _, _, c, _ in vars_), 'Subject To']
    for name, coefs, sense, rhs, gap in rows:
        if rhs is None:
            act = sum(a * start[v] for v, a in coefs.items())
            rhs = act + gap if sense == '<=' else act - gap
        lines.append(f' {name}: ' + ' '.join(f'{a:+.17g} {v}' for v, a in coefs.items()) + f' {sense} {rhs:.17g}')
    bound = lambda n, lb, ub: f' {n} free' if lb <= -INF and ub >= INF else f' {lb:.17g} <= {n} <= {ub:.17g}'  # noqa: E731
    lines += ['Bounds', *(bound(n, lb, ub) for n, lb, ub, _, _ in vars_), 'End']
    starts = {'PStart': [v[4] for v in vars_], 'DStart': [float(duals.get(r[0], 0.0)) for r in rows]}
    return '\n'.join(lines) + '\n', {'params': params, 'starts': starts}


def model(kind):
    if kind == 'shift':
        g = {**shifted(arm.GADGETS, SHIFT), **FREE}
        return build(g, [*arm.GADGETS, *FREE], arm.DUALS, arm.PARAMS)
    if kind == 'resolve':
        # 위반 묶음 전체에서 W만 빼면 후보 없는 B 행 7개가 남아 위반이 풀리지 않았다(history/resolve-v1). 해소되는 SA(A)와
        # feasible 묶음만 둔다. SA 행이 고쳐진 뒤 위반 수가 0이 되어 F 경로로 간다.
        return build(vio.GADGETS, RESOLVE, vio.DUALS, vio.PARAMS)
    raise SystemExit(f'알 수 없는 사례: {kind}')


if __name__ == '__main__':
    kind, out = sys.argv[1], Path(sys.argv[2])  # 예: shift bench/oracle/m3_push_chain/shift
    lp, extra = model(kind)
    for path, text in ((out.with_suffix('.lp'), lp), (out.with_suffix('.extra.json'), json.dumps(extra) + '\n')):
        assert not path.exists() or path.read_text() == text, f'기존 입력과 다름: {path}'
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text)
    print(out.with_suffix('.lp'), out.with_suffix('.extra.json'))
