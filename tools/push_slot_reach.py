"""M3-04b-2: 134a87 스택 슬롯의 쓰기→읽기 도달을 objdump CFG로 계산한다.

한계: 직접 jmp·fall-through만 따른다(간접 jmp가 있으면 실패). call은 fall-through로 본다.
rsp가 인자 push로 이동한 구간의 쓰기는 별도로 frame 기준 주소로 환산해 출력한다.
"""
import hashlib
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
LIB = ROOT.parent / '.venv/lib/python3.13/site-packages/gurobipy/.libs/libgurobi130.so'
BINARY_SHA = '8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c'
SLOTS = ['0x1ec8', '0x1eb0', '0x1e88']
# (시작, 끝, rsp 이동량): 인자 push·add $-0x30 구간
WINDOWS = [(0x1c24f1, 0x1c258a, 0x30), (0x1c25b4, 0x1c2651, 0x30), (0x1c2856, 0x1c28cf, 0x40),
           (0x1c4b06, 0x1c4ba0, 0x30), (0x1c4bca, 0x1c4c69, 0x30), (0x1c4c72, 0x1c4cff, 0x40)]


def disasm():
    assert hashlib.sha256(LIB.read_bytes()).hexdigest() == BINARY_SHA, '바이너리 해시'
    out = subprocess.check_output(['objdump', '-d', '--no-show-raw-insn', '--start-address=0x1c0070',
                                   '--stop-address=0x1c5310', str(LIB)], text=True)
    ins = []
    for line in out.splitlines():
        m = re.match(r'\s+([0-9a-f]+):\s+(\S+)\s*(.*)', line)
        if m:
            ins.append((int(m[1], 16), m[2], m[3]))
    return ins


def reach(ins, slot):
    idx = {a: i for i, (a, _, _) in enumerate(ins)}
    assert not [a for a, o, r in ins if o.startswith('j') and '*' in r], '간접 jmp'

    def succ(i):
        a, o, r = ins[i]
        out = []
        if o.startswith('j'):
            m = re.match(r'([0-9a-f]+) <', r)
            if m and int(m[1], 16) in idx:
                out.append(idx[int(m[1], 16)])
            if o != 'jmp':
                out.append(i + 1)
        elif not o.startswith('ret') and i + 1 < len(ins):
            out.append(i + 1)
        return out
    target = slot + '(%rsp)'
    kills = {i for i, (a, o, r) in enumerate(ins) if r.endswith(',' + target)}
    reads = [i for i, (a, o, r) in enumerate(ins) if target in r and i not in kills]

    def dfs(start):
        seen, stack = set(), list(start)
        while stack:
            i = stack.pop()
            if i in seen or i in kills:
                continue
            seen.add(i)
            stack += succ(i)
        return seen
    result = {'writes': {}, 'entry_unwritten': [hex(ins[i][0]) for i in reads if i in dfs([0])]}
    for k in sorted(kills):
        seen = dfs(succ(k))
        result['writes'][hex(ins[k][0])] = [hex(ins[i][0]) for i in reads if i in seen]
    return result


def shifted_writes(ins):
    out = []
    for a, b, shift in WINDOWS:
        for x, o, r in ins:
            m = re.search(r',(?:0x([0-9a-f]+))?\(%rsp\)$', r)
            if a <= x <= b and m and not o.startswith(('cmp', 'comi', 'ucomi', 'test')):
                out.append((hex(x), o, r, hex(int(m[1] or '0', 16) - shift)))
    return out


if __name__ == '__main__':
    ins = disasm()
    # 루프 준비 쓰기만 루프 읽기(§2.1 표)에 닿아야 한다.
    expect = {'0x1ec8': ('0x1c1fa5', ['0x1c227c', '0x1c2287', '0x1c22a9', '0x1c2955', '0x1c2964', '0x1c4961']),
              '0x1eb0': ('0x1c1fae', ['0x1c229a', '0x1c494e']),
              '0x1e88': ('0x1c1f8a', ['0x1c2c45', '0x1c2ca6', '0x1c2d27', '0x1c2d5f'])}
    ok = True
    for slot in SLOTS:
        res = reach(ins, slot)
        print(slot, res)
        setup, reads = expect[slot]
        others = {r for w, rs in res['writes'].items() if w != setup for r in rs}
        ok &= not res['entry_unwritten'] and res['writes'][setup] == reads and not others & set(reads)
    lea = [(hex(a), r) for a, o, r in ins if o == 'lea' and '(%rsp)' in r]
    print('lea', lea)
    sw = shifted_writes(ins)
    print('shifted', sw)
    frames = {f for *_, f in sw}
    ok &= not frames & {'0x1ec8', '0x1eb0', '0x1e88'}
    sys.exit(0 if ok else 1)
