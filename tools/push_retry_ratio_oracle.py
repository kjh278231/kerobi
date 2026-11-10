"""M3-04c-7: 위반 basis 재시도 ratio 056973(0xf0120)의 합성 경계 직접 호출 캡처와 저장 결과 검사.

push_infeasible_ratio_oracle.py(M3-04c-2, 054eef)와 같은 방식이다. 그 파일은 04c-2 기준 자료가 자기 해시로 고정하므로
고치지 않고 새로 둔다. 입력은 합성이며 자연 crossover에서 캡처한 상태와 구분한다. 기대 행(design)은 사례가 노린 분기에
들어갔는지만 확인하고, 값의 기준은 원본 출력 자체다. 분기 근거는 docs/m3/primal_push_retry.md §2.
"""

import argparse
import copy
import ctypes as c
import hashlib
import json
import math
from pathlib import Path
import platform
import struct


ROOT = Path(__file__).resolve().parents[1]
BINARY_SHA = "8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c"
SOURCE = "../re/decomp/c/000f0120_PRIVATE0000000000056973.c"
H, PIV, INF = 1e-6, 3e-8, 1e30  # 자연 캡처의 S+0x380·S+0x398·S+0x370
SENTINEL = -123.0  # 호출 전 theta·pivot 슬롯. 원본이 쓰지 않는 경로를 구분한다.
LEAVE0 = 7  # 호출 전 S+0x2f8
# 차선 합 판별: 첫 분할 pivot(10)이 전체를 왼쪽에 두어 11원소 벡터 합이 돈다. 원본 차선 순서는 t=7, 순차 합은 t=8
LANE_A = [-0.7, -0.1, -0.01, -1.3, -0.6, -0.7, -0.03, -0.01, -0.1, -1.1]
LANE_R = [10, 4, 8, 9, 6, 5, 3, 7, 10, 1]
QS_A = [-0.5, -0.25, -1.0, -0.75, -0.5]
QS_R = [3, 5, 1, 4, 2]


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def bits(value):
    return struct.pack("<d", value).hex()


def cases():
    # (이름, alpha, x, width, 표시, cap, sparse 순서, 기대 행(dense 기준), factor 갱신 수)
    lane_x = [-r * abs(a) for r, a in zip(LANE_R, LANE_A)] + [5.0]
    specs = [
        ("posN-fallback", [1.0], [2.0], [10.0], "N", INF, [0], 0, 0),
        ("posN-t0", [1.0], [2e-7], [10.0], "N", INF, [0], 0, 0),
        # 행 1은 잔차*2가 h를 넘고 잔차만으로는 h 안이다: *2가 없으면 큰 |a|인 행 1을 고른다
        ("posN-x2", [1.0, 2.0], [1e-7, 8e-7], [10.0, 10.0], "NN", INF, [0, 1], 0, 0),
        # 행 0(L)은 *2 없이 h 안이다: *2를 붙이면 행 1을 고른다
        ("posL-nox2", [1.0, 0.5], [8e-7, 1e-7], [10.0, 10.0], "LN", INF, [0, 1], 0, 0),
        ("posU-t0", [1.0], [1e-7], [10.0], "U", INF, [0], 0, 0),
        ("posX", [1.0], [0.0], [0.0], "X", INF, [0], 0, 0),
        ("posA-first", [1.0], [3.0], [1.0], "A", INF, [0], 0, 0),
        ("posA-second", [1.0, 1.0], [3.0, 10.0], [1.0, 1.0], "AA", INF, [0, 1], 0, 0),
        ("posB-skip", [1.0], [-1.0], [10.0], "B", INF, [0], -1, 0),
        ("negB-one", [-1.0], [-2.0], [INF], "B", INF, [0], 0, 0),
        ("negB-second", [-1.0, -1.0], [-1.0, -5.0], [1.0, INF], "BB", INF, [0, 1], 0, 0),
        ("negN-w", [-1.0], [3.0], [10.0], "N", INF, [0], 0, 0),
        ("negN-winf-skip", [-1.0], [3.0], [INF], "N", INF, [0], -1, 0),
        ("negL-t0", [-1.0], [10.0], [10.0], "L", INF, [0], 0, 0),
        ("negU-at-bound", [-1.0], [10.0], [10.0], "U", INF, [0], 0, 0),
        ("negU-nox2", [-1.0, 0.5], [10.0 + 8e-7, 1e-7], [10.0, 10.0], "UN", INF, [0, 1], 0, 0),
        ("negX", [-1.0], [0.0], [0.0], "X", INF, [0], 0, 0),
        ("negA-skip", [-1.0], [12.0], [10.0], "A", INF, [0], -1, 0),
        ("F-skip", [1.0, -1.0], [2.0, 2.0], [INF, INF], "FF", INF, [0, 1], -1, 0),
        ("at-tol-skip", [PIV, -PIV], [1e-9, -1e-9], [10.0, 10.0], "NB", INF, [0, 1], -1, 0),
        ("tie-t0", [1.0, 1.0], [1e-7, 1e-7], [10.0, 10.0], "NN", INF, [0, 1], 0, 0),
        ("tie-t0-reversed", [1.0, 1.0], [1e-7, 1e-7], [10.0, 10.0], "NN", INF, [1, 0], 1, 0),
        ("larger-alpha-t0", [1.0, 2.0], [1e-7, 2e-7], [10.0, 10.0], "NN", INF, [0, 1], 1, 0),
        ("fallback-tie", [1.0, 2.0], [2.0, 4.0], [10.0, 10.0], "NN", INF, [0, 1], 0, 0),
        ("fallback-tie-reversed", [1.0, 2.0], [2.0, 4.0], [10.0, 10.0], "NN", INF, [1, 0], 1, 0),
        # sparse N/U(a>0)는 cap을 거르지 않아 대체 최소 ratio가 cap 너머(2)를 고르고, dense는 걸러 -1이다
        ("cap-sparse-N", [1.0], [2.0], [10.0], "N", 1.0, [0], -1, 0),
        ("cap-no-cand", [-1.0], [-2.0], [INF], "B", 1.0, [0], -1, 0),
        # sparse N 행만 cap(1)과 같은 breakpoint 1을 넣어 t=cap: cap 종료(theta=cap, -1). dense는 후보 없음 종료(theta=cap)
        ("cap-exit", [-1.0, 1.0], [-5.0, 1.0], [INF, 10.0], "BN", 1.0, [0, 1], -1, 0),
        ("slope-pos-with-cand", [1.0, 5.0], [3.0, 0.0], [1.0, 10.0], "AL", INF, [0, 1], 1, 0),
        # 5개 B 행과 L 행(+1): 기울기 -2가 ratio 3(행 0)에서 0이 된다
        ("quickselect", QS_A + [1.0], [-r * abs(a) for r, a in zip(QS_R, QS_A)] + [5.0], [INF] * 6, "BBBBBL", INF,
         [5, 3, 0, 4, 1, 2], 0, 0),
        ("lanes", LANE_A + [2.11], lane_x, [INF] * 11, "B" * 10 + "L", INF, list(range(11)), 7, 0),
        ("small-pivot-ok", [0.05], [1.0], [10.0], "N", INF, [0], 0, 0),
        # 작은 pivot 재진입: 1차 행 0(a=1e-7) 뒤 허용오차를 3e-8*(1e14*1e-6)=3으로 올려 다시 돈다. Rust는 거부한다
        ("reentry-pick", [1e-7, 1e14], [1e-8, 1e17], [10.0, INF], "NN", INF, [0, 1], 1, 0),
        # 재진입에서 후보 없이 기울기<0: factor 갱신 수(12dd49)가 16 이하면 1차 선택, 넘으면 S+0x294=2·-1
        ("reentry-keep", [1e-7, 1e14], [1e-8, 3e14], [10.0, 1e14], "NA", 1.0, [0, 1], 0, 3),
        ("reentry-refactor", [1e-7, 1e14], [1e-8, 3e14], [10.0, 1e14], "NA", 1.0, [0, 1], -1, 20),
        ("empty", [], [], [], "", INF, [], -1, 0),
        ("zero-alpha", [0.0], [2.0], [10.0], "N", INF, [0], -1, 0),
    ]
    result = []
    for name, alpha, x, width, marks, cap, order, row, updates in specs:
        for dense in (False, True):
            if dense and name.endswith("-reversed"):
                continue
            want = 0 if (name == "cap-sparse-N" and not dense) else row
            result.append(dict(name=name + ("-dense" if dense else "-sparse"), alpha=alpha, x=x, width=width,
                               marks=marks, cap=cap, order=order, dense=dense, row=want, updates=updates))
    return result


def capture():
    import gurobipy as gp  # 기존 gurobi_guard가 단일 프로세스 잠금을 얻는다.

    assert gp.gurobi.version() == (13, 0, 3)
    assert platform.machine() == "x86_64" and c.sizeof(c.c_void_p) == 8
    mappings = [line.split() for line in Path("/proc/self/maps").read_text().splitlines()
                if "libgurobi130.so" in line and line.split()[2] == "00000000"]
    assert len(mappings) == 1, mappings
    library = Path(mappings[0][-1])
    assert sha(library) == BINARY_SHA, "기준 ELF 불일치"
    base = int(mappings[0][0].split("-")[0], 16)
    pointer = c.c_void_p
    # xmm0..3 = h, pivot 허용오차, I, cap. rdi=S, esi=m, rdx=alpha, rcx=basic, r8=width, r9=표시,
    # 스택 = theta*, pivot*, 후보 scratch, ratio scratch, work* (push 호출점 0x1c25b4-0x1c263a; 054eef와 순서가 다르다)
    ratio = c.CFUNCTYPE(c.c_int, *([c.c_double] * 4), pointer, c.c_uint, *([pointer] * 9))(base + 0xf0120)

    class Vector(c.Structure):
        _fields_ = [("count", c.c_int), ("indices", pointer), ("values", pointer)]

    events = []
    for case in cases():
        n = len(case["x"])
        integers = lambda values: (c.c_int * max(1, len(values)))(*values)
        doubles = lambda values: (c.c_double * max(1, len(values)))(*values)
        indices = integers(case["order"])
        alpha = doubles(case["alpha"] if case["dense"] else [case["alpha"][i] for i in case["order"]])
        vector = Vector(-1 if case["dense"] else n, c.addressof(indices), c.addressof(alpha))
        x, width = doubles(case["x"]), doubles(case["width"])
        marks = c.create_string_buffer(case["marks"].encode())
        # A(a>0)·B(a<0)는 행마다 breakpoint 둘을 넣는다
        candidates, ratios = integers([0] * (2 * n + 2)), doubles([0.0] * (2 * n + 2))
        workspace = c.create_string_buffer(0x500)
        factor = c.create_string_buffer(0x200)
        c.c_void_p.from_buffer(workspace, 0x470).value = c.addressof(factor)
        c.c_int.from_buffer(factor, 0x18c).value = case["updates"]
        c.c_double.from_buffer(workspace, 0x148).value = 0.25
        c.c_int.from_buffer(workspace, 0x2f8).value = LEAVE0
        inputs = (indices, alpha, x, width, marks)
        before = [bytes(value) for value in inputs]
        theta, pivot, work = c.c_double(SENTINEL), c.c_double(SENTINEL), c.c_double(0.0)
        chosen = ratio(H, PIV, INF, case["cap"], workspace, n, c.byref(vector), x, width, marks,
                       c.byref(theta), c.byref(pivot), candidates, ratios, c.byref(work))
        row = chosen if chosen < 0 or case["dense"] else case["order"][chosen]
        events.append(dict(name=case["name"], chosen=chosen, row=row, theta=bits(theta.value),
                           pivot=bits(pivot.value), work=bits(work.value),
                           inputs_preserved=before == [bytes(v) for v in inputs],
                           refactor=c.c_int.from_buffer(workspace, 0x294).value,
                           leaving=c.c_int.from_buffer(workspace, 0x2f8).value))
    return dict(schema="M3-04c-7-retry-ratio-v1", provenance="original-synthetic", binary_sha256=BINARY_SHA,
                source_sha256=sha(ROOT / SOURCE), tool_sha256=sha(Path(__file__)),
                inputs_sha256=hashlib.sha256(json.dumps(cases(), sort_keys=True).encode()).hexdigest(),
                environment=dict(machine=platform.machine(), system=platform.system(), libc=platform.libc_ver()),
                events=events)


def validate(record):
    expected_meta = dict(schema="M3-04c-7-retry-ratio-v1", provenance="original-synthetic",
                         binary_sha256=BINARY_SHA, source_sha256=sha(ROOT / SOURCE), tool_sha256=sha(Path(__file__)),
                         inputs_sha256=hashlib.sha256(json.dumps(cases(), sort_keys=True).encode()).hexdigest())
    for field, expected in expected_meta.items():
        assert record[field] == expected, (field, expected, record[field])
    keys = {"name", "chosen", "row", "theta", "pivot", "work", "inputs_preserved", "refactor", "leaving"}
    assert len(record["events"]) == len(cases()), "필수 사건 누락"
    for case, event in zip(cases(), record["events"], strict=True):
        assert event.keys() == keys, (case["name"], "필수 필드 불일치")
        assert event["name"] == case["name"], (case["name"], "순서")
        assert event["row"] == case["row"], (case["name"], "설계 행", case["row"], event["row"])
        assert event["inputs_preserved"] is True, (case["name"], "입력 변경")
        for field in ("theta", "pivot", "work"):
            value = struct.unpack("<d", bytes.fromhex(event[field]))[0]
            assert len(event[field]) == 16 and not math.isnan(value), (case["name"], field)
        # 재진입 요청 2는 갱신 수가 16을 넘는 사례에서만 나온다(12dd49 C·asm f0a3c)
        refactor = case["name"].startswith("reentry-refactor")
        assert (event["refactor"], event["leaving"]) == ((2, -1) if refactor else (0, LEAVE0)), (case["name"], "S 변경")


def check(paths):
    assert len(paths) >= 3, "독립 캡처 3회 필요"
    records = [json.loads(path.read_text()) for path in paths]
    for record in records:
        validate(record)
    for path, record in zip(paths[1:], records[1:], strict=True):
        for first, event in zip(records[0]["events"], record["events"], strict=True):
            for field in first:
                assert first[field] == event[field], (str(path), first["name"], field, first[field], event[field])
    print(f"원본 합성 재시도 ratio: {len(cases())}사례 × {len(paths)}회, 사건 필드 9개 일치")
    return records[0]


def selftest(record):
    mutations = [("binary_sha256", "0" * 64), ("inputs_sha256", "0" * 64), ("tool_sha256", "0" * 64),
                 ("source_sha256", "0" * 64), ("events", []), ("provenance", "natural")]
    for field, value in mutations:
        changed = copy.deepcopy(record)
        changed[field] = value
        rejected(changed)
    for field, value in [("row", 5), ("inputs_preserved", False), ("theta", bits(float("nan"))), ("refactor", 2),
                         ("leaving", -1)]:
        changed = copy.deepcopy(record)
        changed["events"][0][field] = value
        rejected(changed)
    changed = copy.deepcopy(record)
    next(e for e in changed["events"] if e["name"].startswith("reentry-refactor"))["refactor"] = 0
    rejected(changed)
    changed = copy.deepcopy(record)
    del changed["events"][0]["pivot"]
    rejected(changed)
    print("변조 13개 거부")


def rejected(record):
    try:
        validate(record)
    except (AssertionError, KeyError):
        return
    raise AssertionError("변조를 거부하지 않음")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    group = parser.add_mutually_exclusive_group(required=True)
    group.add_argument("--capture", type=Path)
    group.add_argument("--check", nargs="+", type=Path)
    parser.add_argument("--selftest", action="store_true")
    args = parser.parse_args()
    if args.capture:
        record = capture()
        args.capture.parent.mkdir(parents=True, exist_ok=True)
        with args.capture.open("x") as output:
            json.dump(record, output, indent=2)
            output.write("\n")
        validate(record)
        print(f"원본 합성 {len(record['events'])}사례 저장: {args.capture}")
    else:
        record = check(args.check)
        if args.selftest:
            selftest(record)


if __name__ == "__main__":
    main()
