"""M3-04c-9: 054eef(0xeefa0)·056973(0xf0120) 작은 pivot 재진입의 원본 합성 직접 호출 캡처와 검사.

push_infeasible_ratio_oracle.py(M3-04c-2)·push_retry_ratio_oracle.py(M3-04c-7)와 같은 호출 방식이다. 두 파일은 기준 자료가
자기 해시로 고정하므로 고치지 않고 그 cases()를 대조군(control-inf-*, control-ret-*)으로 같은 S·factor 설정에서 다시 부른다.
새 사례의 입력은 합성이며 자연 crossover 상태와 구분한다. 사전 기대값(위치·theta·pivot·요청·반복 수·work 단위)은 캡처
전에 asm(docs/m3/primal_push_violated_reentry.md §2)으로 정했다. 원본과 다르면 검사가 실패하며 기대를 고치지 않는다.

  --capture <새 경로>        원본 1회 캡처(gurun.sh로 실행)
  --controls <3개 이상>      대조군 재현: 메타·3회 일치·04c-2 57·04c-7 74사례 저장 결과와 동일
  --acceptance <3개 이상>    목표 합격: 사전 기대·함수별 재진입·요청 2·보존
  --self-test <3개 이상>     변조가 위 검사에서 거부되는지
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
import sys

sys.path.insert(0, str(Path(__file__).resolve().parent))
import push_infeasible_ratio_oracle as inf_base  # noqa: E402  M3-04c-2 054eef 합성 57호출(대조군)
import push_retry_ratio_oracle as ret_base  # noqa: E402  M3-04c-7 056973 합성 74호출(대조군)

ROOT = Path(__file__).resolve().parents[1]
BINARY_SHA = inf_base.BINARY_SHA
SOURCES = ("../re/decomp/c/000eefa0_PRIVATE0000000000054eef.c", "../re/decomp/c/000f0120_PRIVATE0000000000056973.c",
           "../re/decomp/c/001bb6f0_PRIVATE000000000012dd49.c")
SCHEMA = "M3-04c-9-violated-reentry-v1"
ENTRY = {"054eef": 0xeefa0, "056973": 0xf0120}
H, TOL, INF = 1e-6, 1e-9, 1e30
SENTINEL = -123.0
LEAVE0 = 7  # 새 사례의 호출 전 S+0x2f8
STORED = {"054eef": ROOT / "bench/oracle/m3_push_violated", "056973": ROOT / "bench/oracle/m3_push_retry"}
NAN = float("nan")
THR = (TOL * 1e3) * 0.01  # max=1e3의 재진입 임계 (tol·max)·coef = 1.0000000000000002e-08


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def bits(v):
    return struct.pack("<d", v).hex()


def value(hexbits):
    return struct.unpack("<d", bytes.fromhex(hexbits))[0]


def below(v):
    return math.nextafter(v, 0.0)


def inf_specs():
    # (이름, alpha, x, 폭, 표시, cap, (S+0x470 갱신 수, S+0x498 갱신 수; None=포인터 0),
    #  기대 (위치, theta, pivot, 요청 2, 반복 수, sparse work 단위, dense work 단위)). work = 0.25 * 단위.
    # 054eef work: sparse 2c+4n, dense 2c+2t+n (c 후보, t |a|>tol 수, n 열 길이)
    none = [5e-9, 1e3], [5e-9, 0.0], [10.0, 10.0], "NF", INF
    none_expect = lambda req: (0, 1.0, 5e-9, req, 2, 18, 12)
    out = [
        # 1차 행 0(2e-9) 뒤 tol 1e-8로 다시 돌아 행 1(2e-8)을 고른다. 2e-8은 2차 임계보다 작지만 3차 패스는 없다
        ("inf-reentry-pick", [2e-9, 2e-8, 1e3], [2e-9, 2e-5, 0.0], [10.0] * 3, "NNF", INF, (20, None),
         (1, 2e-5 / 2e-8, 2e-8, False, 2, 28, 20)),
        # 같은 설계를 B 표시 음수 alpha로(num=x, Harris는 빼기)
        ("inf-reentry-negB", [-2e-9, -2e-8, 1e3], [-2e-9, -2e-5, 0.0], [10.0] * 3, "BBF", INF, (20, None),
         (1, -2e-5 / -2e-8, 2e-8, False, 2, 28, 20)),
        # max가 홀수 개 열의 마지막 음수 alpha(폭 I인 N이라 후보 아님). 꼬리를 빼면 max=0.5라 재진입하지 않는다
        ("inf-reentry-negmax", [5e-9, 0.5, -1e3], [5e-9, 1e3, 0.0], [10.0, 10.0, INF], "NNN", INF, (17, None),
         (1, 1e3 / 0.5, 0.5, False, 2, 28, 20)),
        # 2차 후보 없음, 상한 = 유한 cap 5: -1·theta=cap
        ("inf-reentry-flip", [5e-7, 1e6], [5e-7, 1e7], [10.0, 10.0], "NN", 5.0, (20, None),
         (-1, 5.0, 0.0, False, 2, 18, 12)),
        # 2차에 행 1이 후보(ratio 1e30)지만 상한 I: 위치=2차 마지막 후보, theta·pivot은 1차 값
        ("inf-reentry-inf-bound", [5e-9, 1024.0], [5e-9, 1e30 * 1024], [10.0, 10.0], "NN", INF, (17, None),
         (1, 1.0, 5e-9, True, 2, 20, 14)),
        ("inf-reentry-inf-bound-keep", [5e-9, 1024.0], [5e-9, 1e30 * 1024], [10.0, 10.0], "NN", INF, (3, None),
         (1, 1.0, 5e-9, False, 2, 20, 14)),
        ("inf-reentry-none-470-17", *none, (17, None), none_expect(True)),
        ("inf-reentry-none-470-16", *none, (16, None), none_expect(False)),
        ("inf-reentry-none-498-17", *none, (None, 17), none_expect(True)),
        ("inf-reentry-none-null", *none, (None, None), none_expect(False)),
        ("inf-pivot-at-0.1", [0.1, 1e15], [0.1, 0.0], [10.0, 10.0], "NF", INF, (17, None),
         (0, 1.0, 0.1, False, 1, 10, 8)),
        ("inf-pivot-below-0.1", [below(0.1), 1e15], [below(0.1), 0.0], [10.0, 10.0], "NF", INF, (17, None),
         (0, 1.0, below(0.1), True, 2, 18, 12)),
        ("inf-thr-equal", [THR, 1e3], [THR, 0.0], [10.0, 10.0], "NF", INF, (17, None),
         (0, 1.0, THR, False, 1, 10, 8)),
        ("inf-thr-below", [below(THR), 1e3], [below(THR), 0.0], [10.0, 10.0], "NF", INF, (17, None),
         (0, 1.0, below(THR), True, 2, 18, 12)),
    ]
    for B, coef in ((1e4, 1e-3), (1e7, 1e-4), (1e10, 1e-5), (1e13, 1e-6)):
        p = 5 * (TOL * B) * coef
        tag = f"{B:.0e}".replace("+", "")
        out.append((f"inf-scale-at-{tag}", [p, B], [p, 0.0], [10.0, 10.0], "NF", INF, (17, None),
                    (0, 1.0, p, False, 1, 10, 8)))
        out.append((f"inf-scale-below-{tag}", [p, below(B)], [p, 0.0], [10.0, 10.0], "NF", INF, (17, None),
                    (0, 1.0, p, True, 2, 18, 12)))
    out += [
        # 새 tol=tol·(max·coef)=1e-8 < THR: THR 행 1이 2차 후보. 왼쪽부터 곱하면 새 tol=THR이라 후보 0
        ("inf-mul-order", [2e-9, THR, 1e3], [2e-9, 1000 * THR, 0.0], [10.0] * 3, "NNF", INF, (17, None),
         (1, (1000 * THR) / THR, THR, False, 2, 28, 20)),
        # 마지막 원소 NaN이면 maxsd 결과 NaN → 계수 1e-6·임계 NaN → 재진입 안 함
        ("inf-nan-max-last", [5e-9, 1e3, NAN], [5e-9, 0.0, 0.0], [10.0] * 3, "NFN", INF, (17, None),
         (0, 1.0, 5e-9, False, 1, 14, 9)),
        # 가운데 NaN은 짝 비교에서 max를 NaN으로 만들고 다음 원소가 덮는다. NaN alpha는 dense 시험 수에서 빠진다
        ("inf-nan-mid", [5e-9, NAN, 1e3], [5e-9, 0.0, 0.0], [10.0] * 3, "NNF", INF, (17, None),
         (0, 1.0, 5e-9, True, 2, 26, 14)),
        # 1차 패스 2차 선택 ratio == 상한: 큰 pivot 행 1
        ("inf-second-bound-tie", [1.0, 2.0], [1.0, 2 * (1.0 + H)], [10.0, 10.0], "NN", INF, (17, None),
         (1, (2 * (1.0 + H)) / 2.0, 2.0, False, 1, 12, 10)),
        # 1차 패스 후보가 있는데 상한 I: -1·theta=I
        ("inf-candidate-inf-bound", [1024.0], [1e30 * 1024], [10.0], "N", INF, (17, None),
         (-1, INF, 0.0, False, 1, 6, 5)),
    ]
    return out


def ret_specs():
    # 056973 work: 1단계 sparse 4c+2n, dense 4c+nnz+m; 2단계 sparse 4n, dense 3m (후보 없음+기울기<0 종료는 2단계 없음)
    none = ([2e-9, -1e3], [1e-9, -1e4], [10.0, INF], "NB", 1.0)  # 2회째 B 행(r=10)만 기울기 -1e3, cap 1로 후보 없음
    none_expect = lambda req: (-1 if req else 0, 0.5, 2e-9, req, 2, 20, 17)
    fallback = (-1, INF, 0.0, False, 2, 28, 23)  # 2회째 후보 0·기울기 0 → 대체: theta=I, pivot 0, -1
    out = [
        # 1회째 t=0에서 행 0(2e-9) → tol 1e-8 → 2회째 행 1만 후보, t=0 근처 없음 → 대체 최소 ratio
        ("ret-reentry-pick", [2e-9, 2e-8, 1e3], [2e-9, 2e-5, 0.0], [10.0] * 3, "NNF", INF, (20, None),
         (1, 2e-5 / 2e-8, 2e-8, False, 2, 48, 41)),
        # B 행(cap 150 너머 breakpoint 200)이 기울기만 준다: 1회째 t=100(행 3), 2회째 후보 행 2 하나 → t=5에서 행 2
        ("ret-reentry-qs", [2e-9, -1e3, 2e-8, 3e-9], [1e-9, -2e5, 1e-7, 3e-7], [10.0, INF, 10.0, 10.0], "NBNN", 150.0,
         (20, None), (2, 1e-7 / 2e-8, 2e-8, False, 2, 64, 54)),
        ("ret-reentry-empty", [2e-9, 1e3], [2e-9, 0.0], [10.0, 10.0], "NF", INF, (20, None), fallback),
        ("ret-reentry-none-470-17", *none, (17, None), none_expect(True)),
        ("ret-reentry-none-470-16", *none, (16, None), none_expect(False)),
        ("ret-reentry-none-498-17", *none, (None, 17), none_expect(True)),
        ("ret-reentry-none-null", *none, (None, None), none_expect(False)),
        ("ret-reentry-negmax", [5e-9, 0.5, -1e3], [5e-9, 1e3, 0.0], [10.0, 10.0, INF], "NNN", INF, (17, None),
         (1, 1e3 / 0.5, 0.5, False, 2, 48, 41)),
        ("ret-pivot-at-0.1", [0.1, 1e15], [1e-7, 0.0], [10.0, 10.0], "NF", INF, (17, None),
         (0, 1e-7 / 0.1, 0.1, False, 1, 16, 14)),
        ("ret-pivot-below-0.1", [below(0.1), 1e15], [1e-7, 0.0], [10.0, 10.0], "NF", INF, (17, None), fallback),
        ("ret-thr-equal", [THR, 1e3], [THR, 0.0], [10.0, 10.0], "NF", INF, (17, None), (0, 1.0, THR, False, 1, 16, 14)),
        ("ret-thr-below", [below(THR), 1e3], [below(THR), 0.0], [10.0, 10.0], "NF", INF, (17, None), fallback),
    ]
    for B, coef in ((1e4, 1e-3), (1e7, 1e-4), (1e10, 1e-5), (1e13, 1e-6)):
        p = 5 * (TOL * B) * coef
        tag = f"{B:.0e}".replace("+", "")
        out.append((f"ret-scale-at-{tag}", [p, B], [p * 1e-6, 0.0], [10.0, 10.0], "NF", INF, (17, None),
                    (0, (p * 1e-6) / p, p, False, 1, 16, 14)))
        out.append((f"ret-scale-below-{tag}", [p, below(B)], [p * 1e-6, 0.0], [10.0, 10.0], "NF", INF, (17, None),
                    fallback))
    out += [
        ("ret-mul-order", [2e-9, THR, 1e3], [2e-9, 1000 * THR, 0.0], [10.0] * 3, "NNF", INF, (17, None),
         (1, (1000 * THR) / THR, THR, False, 2, 48, 41)),
        ("ret-nan-max-last", [5e-9, 1e3, NAN], [5e-9, 0.0, 0.0], [10.0] * 3, "NFN", INF, (17, None),
         (0, 1.0, 5e-9, False, 1, 22, 18)),
        ("ret-nan-mid", [5e-9, NAN, 1e3], [5e-9, 0.0, 0.0], [10.0] * 3, "NNF", INF, (17, None),
         (-1, INF, 0.0, False, 2, 40, 31)),
    ]
    return out


def cases():
    result = []
    for case in inf_base.cases():  # 04c-2 설정: tol 1e-9, S+0x470 factor 갱신 수 0, S+0x2f8 0
        result.append(dict(case, name="control-inf-" + case["name"], fn="054eef", kind="control", h=inf_base.H,
                           tol=inf_base.PIV, factor=[0, None], leave0=0))
    for case in ret_base.cases():  # 04c-7 설정: tol 3e-8, S+0x470 갱신 수는 사례별, S+0x2f8 7
        case = dict(case)
        updates = case.pop("updates")
        result.append(dict(case, name="control-ret-" + case["name"], fn="056973", kind="control", h=ret_base.H,
                           tol=ret_base.PIV, factor=[updates, None], leave0=ret_base.LEAVE0))
    for fn, specs in (("054eef", inf_specs()), ("056973", ret_specs())):
        for name, alpha, x, width, marks, cap, factor, expect in specs:
            pos, theta, pivot, req, passes, units_sparse, units_dense = expect
            for dense in (False, True):
                result.append(dict(name=name + ("-dense" if dense else "-sparse"), fn=fn, kind="target", alpha=alpha,
                                   x=x, width=width, marks=marks, cap=cap, order=list(range(len(x))), dense=dense,
                                   h=H, tol=TOL, factor=list(factor), leave0=LEAVE0, row=pos, theta=bits(theta),
                                   pivot=bits(pivot), request=req, passes=passes,
                                   work=bits(0.25 * (units_dense if dense else units_sparse))))
    return result


def inputs_sha():
    return hashlib.sha256(json.dumps(cases(), sort_keys=True).encode()).hexdigest()


def capture():
    import gurobipy as gp  # 기존 gurobi_guard가 단일 프로세스 잠금을 얻는다.

    assert gp.gurobi.version() == (13, 0, 3)
    assert platform.machine() == "x86_64" and c.sizeof(c.c_void_p) == 8
    mappings = [line.split() for line in Path("/proc/self/maps").read_text().splitlines()
                if "libgurobi130.so" in line and line.split()[2] == "00000000"]
    assert len(mappings) == 1, mappings
    library = Path(mappings[0][-1])
    assert sha(library) == BINARY_SHA, "기준 ELF 불일치"
    base_addr = int(mappings[0][0].split("-")[0], 16)
    pointer = c.c_void_p
    proto = c.CFUNCTYPE(c.c_int, *([c.c_double] * 4), pointer, c.c_uint, *([pointer] * 9))
    funcs = {fn: proto(base_addr + off) for fn, off in ENTRY.items()}

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
        candidates, ratios = integers([0] * (2 * n + 2)), doubles([0.0] * (2 * n + 2))
        workspace = c.create_string_buffer(0x500)
        factors = [c.create_string_buffer(0x200), c.create_string_buffer(0x200)]
        for slot, field, factor, updates in zip((0x470, 0x498), (0x18c, 0x1a4), factors, case["factor"]):
            if updates is not None:
                c.c_void_p.from_buffer(workspace, slot).value = c.addressof(factor)
                c.c_int.from_buffer(factor, field).value = updates
        c.c_double.from_buffer(workspace, 0x148).value = 0.25
        c.c_int.from_buffer(workspace, 0x2f8).value = case["leave0"]
        inputs = (indices, alpha, x, width, marks)
        before = [bytes(v) for v in inputs]
        ws_before, factor_before = bytes(workspace), [bytes(f) for f in factors]
        theta, pivot, work = c.c_double(SENTINEL), c.c_double(SENTINEL), c.c_double(0.0)
        if case["fn"] == "054eef":
            tail = (candidates, ratios, c.byref(theta), c.byref(pivot), c.byref(work))
        else:  # 056973은 theta*·pivot*이 scratch 앞이다(push 호출점 0x1c25b4-0x1c263a)
            tail = (c.byref(theta), c.byref(pivot), candidates, ratios, c.byref(work))
        chosen = funcs[case["fn"]](case["h"], case["tol"], INF, case["cap"], workspace, n, c.byref(vector), x, width,
                                   marks, *tail)
        row = chosen if chosen < 0 or case["dense"] else case["order"][chosen]
        ws_after = bytes(workspace)
        ws_masked = bytearray(ws_before)
        ws_masked[0x294:0x298] = ws_after[0x294:0x298]
        ws_masked[0x2f8:0x2fc] = ws_after[0x2f8:0x2fc]
        events.append(dict(name=case["name"], chosen=chosen, row=row, theta=bits(theta.value),
                           pivot=bits(pivot.value), work=bits(work.value),
                           inputs_preserved=before == [bytes(v) for v in inputs],
                           workspace_preserved=bytes(ws_masked) == ws_after,
                           factor_preserved=factor_before == [bytes(f) for f in factors],
                           refactor=c.c_int.from_buffer(workspace, 0x294).value,
                           leaving=c.c_int.from_buffer(workspace, 0x2f8).value))
    return dict(schema=SCHEMA, provenance="original-synthetic", binary_sha256=BINARY_SHA,
                source_sha256=[sha(ROOT / s) for s in SOURCES], tool_sha256=sha(Path(__file__)),
                base_tool_sha256=[sha(Path(inf_base.__file__)), sha(Path(ret_base.__file__))],
                inputs_sha256=inputs_sha(),
                environment=dict(machine=platform.machine(), system=platform.system(), libc=platform.libc_ver()),
                events=events)


KEYS = {"name", "chosen", "row", "theta", "pivot", "work", "inputs_preserved", "workspace_preserved",
        "factor_preserved", "refactor", "leaving"}
STORED_KEYS = ("chosen", "row", "theta", "pivot", "work", "inputs_preserved", "refactor", "leaving")


def validate_meta(record):
    expected = dict(schema=SCHEMA, provenance="original-synthetic", binary_sha256=BINARY_SHA,
                    source_sha256=[sha(ROOT / s) for s in SOURCES], tool_sha256=sha(Path(__file__)),
                    base_tool_sha256=[sha(Path(inf_base.__file__)), sha(Path(ret_base.__file__))],
                    inputs_sha256=inputs_sha())
    for field, want in expected.items():
        assert record[field] == want, ("메타", field, want, record.get(field))
    assert isinstance(record["events"], list) and len(record["events"]) == len(cases()), "필수 사건 누락"
    for case, event in zip(cases(), record["events"], strict=True):
        assert event.keys() == KEYS, (case["name"], "필수 필드 불일치", sorted(KEYS ^ set(event)))
        assert event["name"] == case["name"], (case["name"], "사례 순서", event["name"])


def same_runs(records):
    assert len(records) >= 3, "독립 캡처 3회 필요"
    for record in records:
        validate_meta(record)
    for record in records[1:]:
        for first, event in zip(records[0]["events"], record["events"], strict=True):
            for field in KEYS:
                assert first[field] == event[field], ("3회 불일치", first["name"], field, first[field], event[field])


def load(paths):
    records = [json.loads(Path(p).read_text()) for p in paths]
    same_runs(records)
    return records


def controls(records):
    stored = {"054eef": inf_base.check([STORED["054eef"] / f"ratio-{i}.json" for i in (1, 2, 3)]),
              "056973": ret_base.check([STORED["056973"] / f"ratio-{i}.json" for i in (1, 2, 3)])}
    old = {}
    for fn, prefix in (("054eef", "control-inf-"), ("056973", "control-ret-")):
        old.update({prefix + e["name"]: e for e in stored[fn]["events"]})
    counts = {"054eef": 0, "056973": 0}
    fns = {k["name"]: k["fn"] for k in cases()}
    for record in records:
        for event in record["events"]:
            want = old.get(event["name"])
            if want is None:
                continue
            for field in STORED_KEYS:
                assert event[field] == want[field], ("대조군 변화", event["name"], field, want[field], event[field])
            assert event["workspace_preserved"] and event["factor_preserved"], (event["name"], "대조군 S·factor 변경")
            counts[fns[event["name"]]] += 1
    n = len(records)
    assert counts == {"054eef": 57 * n, "056973": 74 * n}, ("대조군 수", counts)
    return {fn: k // n for fn, k in counts.items()}


def acceptance(records):
    by_name = {k["name"]: k for k in cases() if k["kind"] == "target"}
    reentry = {"054eef": 0, "056973": 0}
    request = {"054eef": 0, "056973": 0}
    for event in records[0]["events"]:
        case = by_name.get(event["name"])
        if case is None:
            continue
        for field, want in dict(chosen=case["row"], row=case["row"], theta=case["theta"], pivot=case["pivot"],
                                work=case["work"]).items():
            assert event[field] == want, ("사전 기대 불일치", case["name"], field, want, event[field])
        for field in ("theta", "pivot", "work"):
            assert not math.isnan(value(event[field])), (case["name"], field, "NaN 출력")
        assert event["inputs_preserved"] is True, (case["name"], "입력 변경")
        assert event["workspace_preserved"] is True, (case["name"], "S 변경(요청 칸 밖)")
        assert event["factor_preserved"] is True, (case["name"], "factor 변경")
        req = (2, -1) if case["request"] else (0, LEAVE0)
        assert (event["refactor"], event["leaving"]) == req, (case["name"], "S+0x294/0x2f8", req,
                                                              (event["refactor"], event["leaving"]))
        reentry[case["fn"]] += case["passes"] == 2
        request[case["fn"]] += case["request"]
    # 목표 사건이 없으면 합격이 아니다: 두 함수 각각 재진입(2반복 work 일치) 사례와 요청 2 사례가 있어야 한다.
    for fn in reentry:
        assert reentry[fn] > 0, (fn, "재진입 사례 0")
        assert request[fn] > 0, (fn, "요청 2 사례 0")
    required = {"inf-reentry-pick", "inf-reentry-flip", "inf-reentry-inf-bound", "inf-reentry-none-470-17",
                "inf-mul-order", "inf-nan-max-last", "ret-reentry-pick", "ret-reentry-qs", "ret-reentry-empty",
                "ret-reentry-none-470-17", "ret-reentry-none-470-16", "ret-mul-order", "ret-nan-mid"}
    seen = {e["name"].rsplit("-", 1)[0] for e in records[0]["events"] if e["name"] in by_name}
    assert required <= seen, ("필수 사례 누락", sorted(required - seen))
    return dict(target=len(by_name), reentry=reentry, request=request)


def full(records):
    same_runs(records)
    controls(records)
    return acceptance(records)


def self_test(paths):
    global cases
    records = load(paths)
    full(records)
    events = records[0]["events"]
    idx = {e["name"]: i for i, e in enumerate(events)}
    inf_t, ret_t = idx["inf-reentry-pick-sparse"], idx["ret-reentry-qs-dense"]
    req_inf, req_ret = idx["inf-reentry-none-470-17-sparse"], idx["ret-reentry-none-470-17-dense"]
    keep_ret = idx["ret-reentry-none-470-16-sparse"]

    def edit(i, field, v, all_runs=True):
        def fn(rs):
            for r in (rs if all_runs else rs[:1]):
                r["events"][i][field] = v
        return fn

    def meta(field, v):
        def fn(rs):
            for r in rs:
                r[field] = v
        return fn

    def drop_field(rs):
        for r in rs:
            del r["events"][inf_t]["refactor"]

    mutations = {
        "binary-hash-0": meta("binary_sha256", "0" * 64),
        "inputs-hash-0": meta("inputs_sha256", "0" * 64),
        "tool-hash-0": meta("tool_sha256", "0" * 64),
        "source-hash-0": meta("source_sha256", ["0" * 64] * 3),
        "base-tool-hash-0": meta("base_tool_sha256", ["0" * 64] * 2),
        "events-empty": meta("events", []),
        "provenance-natural": meta("provenance", "natural"),
        "missing-field": drop_field,
        "run-differs": edit(ret_t, "work", bits(0.0), all_runs=False),
        "two-runs": lambda rs: rs.pop(),
        "control-inf-theta": edit(0, "theta", bits(3.0)),
        "control-ret-refactor": edit(idx["control-ret-reentry-refactor-sparse"], "refactor", 0),
        "inf-theta": edit(inf_t, "theta", bits(3.0)),
        "inf-pos": edit(inf_t, "chosen", 0),
        "inf-work-1pass": edit(inf_t, "work", bits(0.25 * 14)),
        "ret-pos": edit(ret_t, "chosen", 3),
        "ret-work-1pass": edit(ret_t, "work", bits(0.25 * 32)),
        "ret-pivot-nan": edit(ret_t, "pivot", bits(NAN)),
        "inf-request-cleared": edit(req_inf, "refactor", 0),
        "ret-request-cleared": edit(req_ret, "refactor", 0),
        "ret-request-returns-prev": edit(req_ret, "chosen", 0),
        "ret-keep-returns-minus1": edit(keep_ret, "chosen", -1),
        "leaving-unwritten": edit(req_inf, "leaving", LEAVE0),
        "spurious-request": edit(inf_t, "refactor", 2),
        "workspace-changed": edit(ret_t, "workspace_preserved", False),
        "factor-changed": edit(inf_t, "factor_preserved", False),
    }
    result = {}
    for name, fn in mutations.items():
        changed = copy.deepcopy(records)
        fn(changed)
        try:
            full(changed)
        except (AssertionError, KeyError) as error:
            result[name] = "거부: " + str(error)[:140]
            continue
        raise AssertionError(("변조를 거부하지 않음", name))
    # 함수별 목표 사건 0회: 사례 정의와 기록에서 함께 빼 메타 검사 없이 목표 검사만 남긴다.
    original = cases
    drops = {f"{fn}-reentry-0": (lambda k, fn=fn: k["fn"] == fn and k.get("passes") == 2) for fn in ENTRY}
    drops.update({f"{fn}-request-0": (lambda k, fn=fn: k["fn"] == fn and k.get("request")) for fn in ENTRY})
    for name, drop in drops.items():
        kept = [k for k in original() if not drop(k)]
        names = {k["name"] for k in kept}
        changed = copy.deepcopy(records)
        for r in changed:
            r["events"] = [e for e in r["events"] if e["name"] in names]
        cases = lambda kept=kept: kept
        try:
            acceptance(changed)
        except AssertionError as error:
            result[name] = "거부: " + str(error)[:140]
        else:
            raise AssertionError(("변조를 거부하지 않음", name))
        finally:
            cases = original
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    group = parser.add_mutually_exclusive_group(required=True)
    group.add_argument("--capture", type=Path)
    group.add_argument("--controls", nargs="+", type=Path)
    group.add_argument("--acceptance", nargs="+", type=Path)
    group.add_argument("--self-test", nargs="+", type=Path)
    args = parser.parse_args()
    try:
        if args.capture:
            record = capture()
            args.capture.parent.mkdir(parents=True, exist_ok=True)
            with args.capture.open("x") as output:
                json.dump(record, output, indent=2)
                output.write("\n")
            validate_meta(record)
            print(f"원본 합성 {len(record['events'])}사례 저장: {args.capture}")
        elif args.controls:
            print(json.dumps(dict(controls=controls(load(args.controls)), runs=len(args.controls))))
        elif args.acceptance:
            print(json.dumps(dict(acceptance(load(args.acceptance)), runs=len(args.acceptance)), ensure_ascii=False))
        else:
            result = self_test(args.self_test)
            print(json.dumps(dict(rejected=len(result), detail=result), ensure_ascii=False, indent=1))
    except (AssertionError, KeyError) as error:
        print("불합격: " + str(error), file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
