"""M3-04b: 055f68의 합성 경계 캡처와 저장 결과 검사."""

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
SOURCE = "../re/decomp/c/000ef960_PRIVATE0000000000055f68.c"


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def bits(value):
    return struct.pack("<d", value).hex()


def cases():
    # 합성 입력이며 자연 crossover에서 캡처한 상태와 구분한다.
    specs = [
        ("lower", [1.0], [2.0], [10.0], "N", 1e30, [0], 0, 2.0, 1.0),
        ("upper", [-2.0], [2.0], [10.0], "N", 1e30, [0], 0, 4.0, 2.0),
        ("tie", [1.0, 1.0], [2.0, 2.0], [10.0, 10.0], "NN", 1e30, [0, 1], 0, 2.0, 1.0),
        ("tie-reversed", [1.0, 1.0], [2.0, 2.0], [10.0, 10.0], "NN", 1e30, [1, 0], 1, 2.0, 1.0),
        ("larger-pivot", [1.0, 2.0], [2.0, 4.0], [10.0, 10.0], "NN", 1e30, [0, 1], 1, 2.0, 2.0),
        ("free", [1.0], [2.0], [1e30], "F", 1e30, [0], -1, 1e30, 0.0),
        ("upper-infinite", [-1.0], [2.0], [1e30], "N", 1e30, [0], -1, 1e30, 0.0),
        ("pivot-at-tol", [1e-9], [2.0], [10.0], "N", 1e30, [0], -1, 1e30, 0.0),
        ("pivot-above-tol", [math.nextafter(1e-9, math.inf)], [0.0], [10.0], "N", 1e30, [0], 0, 0.0, math.nextafter(1e-9, math.inf)),
        ("negative-zero", [1.0], [-0.0], [10.0], "N", 1e30, [0], 0, -0.0, 1.0),
        ("empty", [], [], [], "", 1e30, [], -1, 1e30, 0.0),
        ("finite-flip", [1.0], [2.0], [10.0], "N", 1.0, [0], -1, 1.0, 0.0),
        ("flip-tie", [1.0], [2.0], [10.0], "N", 2.0, [0], -1, 2.0, 0.0),
        ("flip-relaxed-tie", [1.0], [2.0], [10.0], "N", 2.0 + 0.5e-6, [0], -1, 2.0 + 0.5e-6, 0.0),
        ("zero-alpha", [0.0], [2.0], [10.0], "N", 1e30, [0], -1, 1e30, 0.0),
    ]
    result = []
    for name, alpha, x, width, marks, cap, order, row, theta, pivot in specs:
        for dense in (False, True):
            if dense and name == "tie-reversed":
                continue
            result.append(dict(name=name + ("-dense" if dense else "-sparse"),
                               alpha=alpha, x=x, width=width, marks=marks,
                               cap=cap, order=order, dense=dense,
                               row=row, theta=bits(theta), pivot=bits(pivot)))
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
    ratio = c.CFUNCTYPE(c.c_int, *([c.c_double] * 4), pointer, c.c_uint,
                       c.c_int, *([pointer] * 11))(base + 0xef960)

    class Vector(c.Structure):
        _fields_ = [("count", c.c_int), ("indices", pointer), ("values", pointer)]

    events = []
    for case in cases():
        n = len(case["x"])
        integers = lambda values: (c.c_int * max(1, len(values)))(*values)
        doubles = lambda values: (c.c_double * max(1, len(values)))(*values)
        indices = integers(case["order"])
        alpha = doubles(case["alpha"] if case["dense"] else
                        [case["alpha"][i] for i in case["order"]])
        vector = Vector(-1 if case["dense"] else n, c.addressof(indices), c.addressof(alpha))
        x, width = doubles(case["x"]), doubles(case["width"])
        marks = c.create_string_buffer(case["marks"].encode())
        header, lengths = integers(range(n)), integers([1] * n)
        candidates, ratios = integers([0] * n), doubles([0.0] * n)
        workspace = c.create_string_buffer(0x500)
        factor = c.create_string_buffer(0x200)
        c.c_void_p.from_buffer(workspace, 0x470).value = c.addressof(factor)
        c.c_double.from_buffer(workspace, 0x148).value = 0.25
        inputs = (indices, alpha, x, width, marks, header, lengths)
        before = [bytes(value) for value in inputs]
        theta, pivot, work = c.c_double(-123.0), c.c_double(-123.0), c.c_double(0.0)
        chosen = ratio(1e-6, 1e-9, 1e30, case["cap"], workspace, n, 0,
                       lengths, header, c.byref(vector), x, width, marks,
                       candidates, ratios, c.byref(theta), c.byref(pivot), c.byref(work))
        row = chosen if chosen < 0 or case["dense"] else case["order"][chosen]
        events.append(dict(name=case["name"], chosen=chosen, row=row,
                           theta=bits(theta.value), pivot=bits(pivot.value),
                           work=bits(work.value), inputs_preserved=before == [bytes(v) for v in inputs],
                           refactor=c.c_int.from_buffer(workspace, 0x294).value,
                           leaving=c.c_int.from_buffer(workspace, 0x2f8).value))
    return dict(schema="M3-04b-ratio-v1", provenance="original-synthetic",
                binary_sha256=BINARY_SHA, source_sha256=sha(ROOT / SOURCE),
                tool_sha256=sha(Path(__file__)), inputs_sha256=hashlib.sha256(
                    json.dumps(cases(), sort_keys=True).encode()).hexdigest(),
                environment=dict(machine=platform.machine(), system=platform.system(),
                                 libc=platform.libc_ver()), events=events)


def validate(record):
    expected_meta = dict(schema="M3-04b-ratio-v1", provenance="original-synthetic",
                         binary_sha256=BINARY_SHA, source_sha256=sha(ROOT / SOURCE),
                         tool_sha256=sha(Path(__file__)), inputs_sha256=hashlib.sha256(
                             json.dumps(cases(), sort_keys=True).encode()).hexdigest())
    for field, expected in expected_meta.items():
        assert record[field] == expected, (field, expected, record[field])
    expected_keys = {"name", "chosen", "row", "theta", "pivot", "work",
                     "inputs_preserved", "refactor", "leaving"}
    assert len(record["events"]) == len(cases()), "필수 사건 누락"
    for case, event in zip(cases(), record["events"], strict=True):
        assert event.keys() == expected_keys, (case["name"], "필수 필드 불일치")
        chosen = case["row"]
        if chosen >= 0 and not case["dense"]:
            chosen = case["order"].index(chosen)
        expected = dict(name=case["name"], row=case["row"], chosen=chosen,
                        theta=case["theta"], pivot=case["pivot"], inputs_preserved=True,
                        refactor=0, leaving=0)
        for field, value in expected.items():
            assert event[field] == value, (case["name"], field, value, event[field])
        assert len(event["work"]) == 16 and math.isfinite(
            struct.unpack("<d", bytes.fromhex(event["work"]))[0]), (case["name"], "work")


def check(paths):
    assert len(paths) >= 3, "독립 캡처 3회 필요"
    records = [json.loads(path.read_text()) for path in paths]
    for record in records:
        validate(record)
    for path, record in zip(paths[1:], records[1:], strict=True):
        for first, event in zip(records[0]["events"], record["events"], strict=True):
            for field in first:
                assert first[field] == event[field], (str(path), first["name"], field,
                                                     first[field], event[field])
    print(f"원본 합성 ratio: {len(cases())}사례 × {len(paths)}회, 사건 필드 9개 일치")
    return records[0]


def selftest(record):
    mutations = [("binary_sha256", "0" * 64), ("inputs_sha256", "0" * 64),
                 ("tool_sha256", "0" * 64), ("source_sha256", "0" * 64),
                 ("events", []), ("provenance", "natural")]
    for field, value in mutations:
        changed = copy.deepcopy(record)
        changed[field] = value
        rejected(changed)
    for field, value in [("theta", bits(3.0)), ("row", 1), ("chosen", 1)]:
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
        assert not args.selftest
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
