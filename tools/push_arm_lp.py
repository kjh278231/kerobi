"""M3-04b-3: primal push 판단 arm을 하나씩 밟도록 만든 블록 대각 LP와 시작 벡터(PStart·DStart)를 쓴다.

각 묶음은 서로 다른 변수·행만 쓰므로 alpha와 r이 묶음 밖으로 번지지 않는다. rhs가 None인 행은 시작값에서
활동값+gap으로 둔다(slack=gap). 계약과 묶음별 의도는 docs/m3/primal_push_arms_oracle.md §4.
"""
import json
import sys
from pathlib import Path

# (이름, lb, ub, 목적 계수, 시작값), (행, {변수: 계수}, sense, rhs 또는 None, gap)
GADGETS = {
    'A': ([('xa', 0, 10, -1, 10 - 5e-5), ('ya', 0, 10, 0, 0)], [('ca', {'xa': 1, 'ya': 1}, '<=', None, 4.96e-5)]),
    'A2': ([('xA2', 0, 10, -1, 10 - 5e-3), ('yA2', 0, 10, 0, 0)], [('rA2', {'xA2': 1, 'yA2': 1}, '<=', None, 4.9e-3)]),
    'B': ([('xb', 0, 10, 1, 5e-5)], [('cb', {'xb': -1}, '<=', None, 4.96e-5)]),
    'C': ([('xc', 0, 10, 0, 5e-5), ('yc', 0, 10, 0, 0)], [('cc', {'xc': -1, 'yc': 1}, '<=', None, 1.0)]),
    'E8': ([('xE8', 0, 10, -1e-3, 5e-5), ('yE8', 0, 10, 0, 0)], [('rE8', {'xE8': 1, 'yE8': 1}, '<=', None, 1.0)]),
    'H': ([('xh', 0, 10, 0, 5), ('yh', 0, 10, 0, 0), ('zh', 0, 10, 0, 0)],
          [('ch1', {'xh': 1e-7, 'yh': 1}, '<=', None, 0.0), ('ch2', {'xh': 1, 'zh': 1}, '<=', 100, 0.0)]),
    'U': ([('xu', 0, 10, 0, 5), ('wu', 0, 100, 1, 3), ('zu', 0, 10, 0, 0)],
          [('ru', {'xu': 1, 'wu': 1}, '<=', None, 5 - 5e-7), ('ru2', {'wu': 1, 'zu': 1}, '<=', 1000, 0.0)]),
    'P': ([('xp', 0, 10, 0, 5), ('wp', 0, 100, -1, 3), ('zp', 0, 10, 0, 0)],
          [('rp', {'xp': 1, 'wp': 1}, '<=', None, 5 - 5e-7), ('rp2', {'wp': 1, 'zp': 1}, '<=', 1000, 0.0)]),
    'L': ([('xl', 0, 10, 1, 5), ('wl', 0, 100, -2, 3), ('zl', 0, 10, 0, 0)],
          [('rl', {'xl': -1, 'wl': 1}, '<=', None, 5 - 5e-7), ('rl2', {'wl': 1, 'zl': 1}, '<=', 1000, 0.0)]),
    'K': ([('xK', 0, 2.4e-6, 0, 1.2e-6), ('wK', 0, 100, -1, 3), ('zK', 0, 10, 0, 0)],
          [('rK', {'xK': 1, 'wK': 1}, '<=', None, 3e-7), ('rK2', {'wK': 1, 'zK': 1}, '<=', 1000, 0.0)]),
    'K2': ([('xK2', 0, 2.4e-6, 5e-5, 1.2e-6), ('wK2', 0, 100, -1, 3), ('zK2', 0, 10, 0, 0)],
           [('rK3', {'xK2': -1, 'wK2': 1}, '<=', None, 6e-7), ('rK4', {'wK2': 1, 'zK2': 1}, '<=', 1000, 0.0)]),
    'X': ([('xm', 2, 2, 0, 2), ('pm', 0, 10, 0, 5), ('zm', 0, 10, 0, 0)],
          [('em', {'xm': 1, 'pm': 1e-4}, '=', None, 0.0), ('gm', {'pm': 1, 'zm': 1}, '<=', 100, 0.0)]),
    'SD': ([('xt', 0, 10, 0, 3), ('wt', 0, 10, 0, 0)], [('rt', {'xt': 1, 'wt': 1}, '<=', None, 0.5)]),
    'T': ([('xT', 0, 10.5, 0, 1), ('wT', 0, 10, 0, 0)], [('rT', {'xT': 1, 'wT': 1}, '<=', None, 1.0)]),
    'T2': ([('xT2', 0, 11, 0, 1), ('wT2', 0, 10, 0, 0)], [('rT2', {'xT2': 1, 'wT2': 1}, '<=', None, 1.0)]),
    'T3': ([('xT3', 0, 21, -1, 1), ('wT3', 0, 10, 0, 0)], [('rT3', {'xT3': 1, 'wT3': 1}, '<=', None, 1.0)]),
    'T4': ([('xT4', 0, 10, 0, 5), ('wT4', 0, 10, 0, 0)], [('rT4', {'xT4': 1, 'wT4': 1}, '<=', 10 - 5e-11, 0.0)]),
    'T5': ([('xT5', 0, 10, 0, 5), ('wT5', 0, 10, 0, 0)], [('rT5', {'xT5': 1, 'wT5': 1}, '<=', 9.9999999999, 0.0)]),
}
DUALS = {'rt': -1.0}  # SD: 단독 묶음 dual-first에서 rt의 slack이 비기저로 남았다(DStart 0이면 basic, history/tSD0)
PARAMS = {'ScaleFlag': 0}


def build(names):
    vars_ = [v for k in names for v in GADGETS[k][0]]
    rows = [r for k in names for r in GADGETS[k][1]]
    start = {v[0]: v[4] for v in vars_}
    # 목적에 모든 변수를 적어 LP 읽기의 열 순서를 묶음 순서로 고정한다
    lines = ['Minimize', ' obj: ' + ' '.join(f'{c:+.17g} {n}' for n, _, _, c, _ in vars_), 'Subject To']
    for name, coefs, sense, rhs, gap in rows:
        if rhs is None:
            rhs = sum(a * start[v] for v, a in coefs.items()) + gap
        lines.append(f' {name}: ' + ' '.join(f'{a:+.17g} {v}' for v, a in coefs.items()) + f' {sense} {rhs:.17g}')
    lines += ['Bounds', *(f' {lb:.17g} <= {n} <= {ub:.17g}' for n, lb, ub, _, _ in vars_), 'End']
    starts = {'PStart': [v[4] for v in vars_], 'DStart': [float(DUALS.get(r[0], 0.0)) for r in rows]}
    return '\n'.join(lines) + '\n', {'params': PARAMS, 'starts': starts}


if __name__ == '__main__':
    out = Path(sys.argv[1])  # 예: bench/oracle/m3_push_arms/arms
    lp, extra = build(list(GADGETS))
    for path, text in ((out.with_suffix('.lp'), lp), (out.with_suffix('.extra.json'), json.dumps(extra) + '\n')):
        assert not path.exists() or path.read_text() == text, f'기존 입력과 다름: {path}'
        path.write_text(text)
    print(out.with_suffix('.lp'), out.with_suffix('.extra.json'))
