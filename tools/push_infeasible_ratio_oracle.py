"""M3-04c-2: 위반 basis ratio 054eef(0xeefa0)의 합성 경계 직접 호출 캡처와 저장 결과 검사.

primal_push_ratio_oracle.py(M3-04b, 055f68)와 같은 방식이다. 입력은 합성이며 자연 crossover에서 캡처한 상태와
구분한다. 기대 행(design)은 사례가 노린 분기에 들어갔는지만 확인하고, 값의 기준은 원본 출력 자체다.
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
SOURCE = "../re/decomp/c/000eefa0_PRIVATE0000000000054eef.c"
H, PIV, INF = 1e-6, 1e-9, 1e30
B = 1.0 + 1e-6  # 첫 후보(N, a=1, x=1)가 남기는 Harris 상한 (1+h)/1


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def bits(value):
    return struct.pack("<d", value).hex()


def cases():
    # (이름, alpha, x, width, 표시, cap, sparse 순서, 기대 행(dense 기준), dense 생략)
    specs = [
        ("pos-N", [1.0], [2.0], [10.0], "N", INF, [0], 0),
        ("pos-U", [1.0], [9.5], [10.0], "U", INF, [0], 0),
        ("pos-A", [2.0], [12.0], [10.0], "A", INF, [0], 0),
        ("pos-L", [1.0], [0.0], [10.0], "L", INF, [0], 0),
        ("pos-X", [1.0], [0.0], [0.0], "X", INF, [0], 0),
        ("pos-B-skip", [1.0], [-1.0], [10.0], "B", INF, [0], -1),
        ("pos-F-skip", [1.0], [2.0], [INF], "F", INF, [0], -1),
        ("neg-B", [-1.0], [-2.0], [10.0], "B", INF, [0], 0),
        ("neg-L", [-1.0], [1.0], [10.0], "L", INF, [0], 0),
        ("neg-N", [-1.0], [3.0], [10.0], "N", INF, [0], 0),
        ("neg-N-inf-skip", [-1.0], [3.0], [INF], "N", INF, [0], -1),
        ("neg-U", [-1.0], [9.5], [10.0], "U", INF, [0], 0),
        ("neg-X", [-1.0], [0.0], [0.0], "X", INF, [0], 0),
        ("neg-A-skip", [-1.0], [12.0], [10.0], "A", INF, [0], -1),
        ("neg-F-skip", [-1.0], [2.0], [INF], "F", INF, [0], -1),
        ("at-tol-skip", [PIV, -PIV], [2.0, -2.0], [10.0, 10.0], "NB", INF, [0, 1], -1),
        ("tie", [1.0, -1.0], [2.0, -2.0], [10.0, 10.0], "NB", INF, [0, 1], 0),
        ("tie-reversed", [1.0, -1.0], [2.0, -2.0], [10.0, 10.0], "NB", INF, [1, 0], 1),
        ("larger-pivot", [1.0, -2.0], [2.0, -4.0], [10.0, 10.0], "NB", INF, [0, 1], 1),
        ("harris", [1.0, 2.0], [1.0, 2.0 + 1e-6], [10.0, 10.0], "NN", INF, [0, 1], 1),
        # 두 번째 행 ratio가 첫 후보의 Harris 상한과 같다: sparse는 <=로 받고 dense는 <로 거른다(asm ef6c1·ef826)
        ("negU-at-bound", [1.0, -2.0], [1.0, 0.0], [10.0, 2 * B], "NU", INF, [0, 1], 0),
        ("negX-at-bound", [1.0, -2.0], [1.0, 0.0], [10.0, 2 * B], "NX", INF, [0, 1], 0),
        ("negB-at-bound", [1.0, -2.0], [1.0, -2 * B], [10.0, 10.0], "NB", INF, [0, 1], 1),
        ("finite-cap", [1.0], [2.0], [10.0], "N", 1.0, [0], -1),
        ("empty", [], [], [], "", INF, [], -1),
        ("zero-alpha", [0.0], [2.0], [10.0], "N", INF, [0], -1),
        ("negative-zero", [1.0], [-0.0], [10.0], "L", INF, [0], 0),
        ("small-pivot-ok", [0.05], [1.0], [10.0], "N", INF, [0], 0),
        # 작은 pivot 재진입: 강화 임계값(1e-9*1e14*1e-6=0.1)으로 첫 패스를 다시 돈다. Rust는 거부한다.
        ("reentry", [1e-8, 1e14], [1e-9, 1e17], [10.0, 1e30], "NN", INF, [0, 1], 1),
    ]
    result = []
    for name, alpha, x, width, marks, cap, order, row in specs:
        for dense in (False, True):
            if dense and name == "tie-reversed":
                continue
            # sparse 기대 행: at-bound 사례는 sparse가 둘째 행(|a|=2)을 고른다
            want = row
            if not dense and name in ("negU-at-bound", "negX-at-bound"):
                want = 1
            result.append(dict(name=name + ("-dense" if dense else "-sparse"), alpha=alpha, x=x, width=width,
                               marks=marks, cap=cap, order=order, dense=dense, row=want))
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
    # xmm0..3 = Harris h, pivot 허용오차, I, cap. rdi=S, esi=m, rdx=alpha, rcx=basic, r8=width, r9=표시,
    # 스택 = 후보 scratch, ratio scratch, theta*, pivot*, work* (push 호출점 0x1c24f1-0x1c2573)
    ratio = c.CFUNCTYPE(c.c_int, *([c.c_double] * 4), pointer, c.c_uint, *([pointer] * 9))(base + 0xeefa0)

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
        candidates, ratios = integers([0] * n), doubles([0.0] * n)
        workspace = c.create_string_buffer(0x500)
        factor = c.create_string_buffer(0x200)
        c.c_void_p.from_buffer(workspace, 0x470).value = c.addressof(factor)
        c.c_double.from_buffer(workspace, 0x148).value = 0.25
        inputs = (indices, alpha, x, width, marks)
        before = [bytes(value) for value in inputs]
        theta, pivot, work = c.c_double(-123.0), c.c_double(-123.0), c.c_double(0.0)
        chosen = ratio(H, PIV, INF, case["cap"], workspace, n, c.byref(vector), x, width, marks,
                       candidates, ratios, c.byref(theta), c.byref(pivot), c.byref(work))
        row = chosen if chosen < 0 or case["dense"] else case["order"][chosen]
        events.append(dict(name=case["name"], chosen=chosen, row=row, theta=bits(theta.value),
                           pivot=bits(pivot.value), work=bits(work.value),
                           inputs_preserved=before == [bytes(v) for v in inputs],
                           refactor=c.c_int.from_buffer(workspace, 0x294).value,
                           leaving=c.c_int.from_buffer(workspace, 0x2f8).value))
    return dict(schema="M3-04c-2-infeasible-ratio-v1", provenance="original-synthetic", binary_sha256=BINARY_SHA,
                source_sha256=sha(ROOT / SOURCE), tool_sha256=sha(Path(__file__)),
                inputs_sha256=hashlib.sha256(json.dumps(cases(), sort_keys=True).encode()).hexdigest(),
                environment=dict(machine=platform.machine(), system=platform.system(), libc=platform.libc_ver()),
                events=events)


def validate(record):
    expected_meta = dict(schema="M3-04c-2-infeasible-ratio-v1", provenance="original-synthetic",
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


def check(paths):
    assert len(paths) >= 3, "독립 캡처 3회 필요"
    records = [json.loads(path.read_text()) for path in paths]
    for record in records:
        validate(record)
    for path, record in zip(paths[1:], records[1:], strict=True):
        for first, event in zip(records[0]["events"], record["events"], strict=True):
            for field in first:
                assert first[field] == event[field], (str(path), first["name"], field, first[field], event[field])
    print(f"원본 합성 infeasible ratio: {len(cases())}사례 × {len(paths)}회, 사건 필드 9개 일치")
    return records[0]


def selftest(record):
    mutations = [("binary_sha256", "0" * 64), ("inputs_sha256", "0" * 64), ("tool_sha256", "0" * 64),
                 ("source_sha256", "0" * 64), ("events", []), ("provenance", "natural")]
    for field, value in mutations:
        changed = copy.deepcopy(record)
        changed[field] = value
        rejected(changed)
    for field, value in [("row", 5), ("inputs_preserved", False), ("theta", bits(float("nan")))]:
        changed = copy.deepcopy(record)
        changed["events"][0][field] = value
        rejected(changed)
    changed = copy.deepcopy(record)
    del changed["events"][0]["pivot"]
    rejected(changed)
    print("변조 10개 거부")


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
