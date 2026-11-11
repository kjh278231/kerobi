"""M3-04c-5: 055f68(0xef960) 작은 pivot 재진입 패스와 12dd49 요청 2의 원본 합성 직접 호출 캡처와 검사.

primal_push_ratio_oracle.py(M3-04b)와 같은 호출 방식이다. 그 파일은 04b·04c 기준 자료가 자기 해시로 고정하므로 고치지
않고, 그 cases()를 대조군(control-*)으로 같은 S·factor 설정에서 다시 부른다. 새 사례의 입력은 합성이며 자연 crossover
상태와 구분한다. 사전 기대값(위치·theta·pivot·요청·패스 수·work 단위)은 캡처 전에 asm(docs/m3/primal_push_reentry.md
§2.2)으로 정했다. 원본과 다르면 검사가 실패하며 기대를 결과에 맞춰 고치지 않는다.

  --capture <새 경로>        원본 1회 캡처(gurun.sh로 실행)
  --controls <3개 이상>      대조군 재현: 메타·3회 일치·04c 29사례 저장 결과와 동일
  --acceptance <3개 이상>    목표 합격: 사전 기대·재진입·요청 2·보존
  --self-test <3개 이상>     변조가 위 두 검사에서 거부되는지
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
import primal_push_ratio_oracle as base  # noqa: E402  M3-04b 합성 29사례(대조군)

ROOT = Path(__file__).resolve().parents[1]
BINARY_SHA = base.BINARY_SHA
SOURCES = ("../re/decomp/c/000ef960_PRIVATE0000000000055f68.c", "../re/decomp/c/001bb6f0_PRIVATE000000000012dd49.c")
SCHEMA = "M3-04c-5-reentry-v1"
H, TOL, INF = 1e-6, 1e-9, 1e30
SENTINEL = -123.0  # 호출 전 theta·pivot 슬롯
LEAVE0 = 7  # 새 사례의 호출 전 S+0x2f8. 요청 2의 -1 쓰기를 구분한다
CONTROL_STORED = ROOT / "bench/oracle/m3_primal_push"
NAN = float("nan")
THR = (TOL * 1e3) * 0.01  # max=1e3의 재진입 임계 (tol·max)·coef = 1.0000000000000002e-08
BOUND1 = (1.0 + H) / 1.0  # second-bound-tie 첫 행의 Harris 상한


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def bits(value):
    return struct.pack("<d", value).hex()


def value(hexbits):
    return struct.unpack("<d", bytes.fromhex(hexbits))[0]


def below(v):
    return math.nextafter(v, 0.0)


def scale_cases():
    # 강화 계수 경계 B: max=B는 다음 구간 계수(c), max 직전 값은 10c. pivot=5·(tol·B)·c는 앞쪽만 재진입한다.
    out = []
    for B, coef in ((1e4, 1e-3), (1e7, 1e-4), (1e10, 1e-5), (1e13, 1e-6)):
        p = 5 * (TOL * B) * coef
        tag = f"{B:.0e}".replace("+", "")
        out.append((f"scale-at-{tag}", [p, B], [p, 0.0], [10.0, 10.0], "NF", INF, (17, None),
                    (0, 1.0, p, False, 1, 11, 7)))
        out.append((f"scale-below-{tag}", [p, below(B)], [p, 0.0], [10.0, 10.0], "NF", INF, (17, None),
                    (0, 1.0, p, True, 2, 19, 9)))
    return out


def new_specs():
    # (이름, alpha, x, 폭, 표시, cap, (S+0x470 갱신 수, S+0x498 갱신 수; None=포인터 0),
    #  기대 (위치, theta, pivot, 요청 2, 패스 수, sparse work 단위, dense work 단위)). work = 0.25 * 단위.
    none = [5e-9, 1e3], [5e-9, 0.0], [10.0, 10.0], "NF", INF
    none_expect = lambda req: (0, 1.0, 5e-9, req, 2, 19, 9)
    specs = [
        # 1차 행 0(2e-9) 뒤 tol=1e-8로 재진입해 행 1(2e-8)을 고른다. 2e-8은 0.1과 2차 임계 1e-7보다 작지만 3차 패스는 없다
        ("reentry-pick", [2e-9, 2e-8, 1e3], [2e-9, 2e-5, 0.0], [10.0, 10.0, 10.0], "NNF", INF, (20, None),
         (1, 2e-5 / 2e-8, 2e-8, False, 2, 30, 18)),
        # max가 홀수 개 열의 마지막 음수 alpha(폭 I라 후보 아님). 꼬리를 빼면 max=0.5라 재진입하지 않는다
        ("reentry-negmax-pick", [5e-9, 0.5, -1e3], [5e-9, 1e3, 0.0], [10.0, 10.0, INF], "NNN", INF, (17, None),
         (1, 1e3 / 0.5, 0.5, False, 2, 30, 18)),
        # 2차 상한(=cap, 행 1 ratio 10은 후보 아님) >= 유한 cap 5: -1·theta=cap
        ("reentry-flip", [5e-7, 1e6], [5e-7, 1e7], [10.0, 10.0], "NN", 5.0, (20, None),
         (-1, 5.0, 0.0, False, 2, 19, 13)),
        # 2차에 행 1이 후보(ratio 1e30)지만 상한이 I: 위치=2차 마지막 후보, theta·pivot은 1차 값
        ("reentry-inf-bound", [5e-9, 1024.0], [5e-9, 1e30 * 1024], [10.0, 10.0], "NN", INF, (17, None),
         (1, 1.0, 5e-9, True, 2, 22, 16)),
        ("reentry-inf-bound-keep", [5e-9, 1024.0], [5e-9, 1e30 * 1024], [10.0, 10.0], "NN", INF, (3, None),
         (1, 1.0, 5e-9, False, 2, 22, 16)),
        # 2차 후보 0: 12dd49 변형. S+0x470 우선, 없으면 S+0x498, 둘 다 없으면 -1. signed > 16
        ("reentry-none-470-17", *none, (17, None), none_expect(True)),
        ("reentry-none-470-16", *none, (16, None), none_expect(False)),
        ("reentry-none-470-0", *none, (0, None), none_expect(False)),
        ("reentry-none-470-neg", *none, (-5, None), none_expect(False)),
        ("reentry-none-470-max", *none, (2147483647, None), none_expect(True)),
        ("reentry-none-498-17", *none, (None, 17), none_expect(True)),
        ("reentry-none-498-16", *none, (None, 16), none_expect(False)),
        ("reentry-none-null", *none, (None, None), none_expect(False)),
        ("reentry-none-470-0-498-20", *none, (0, 20), none_expect(False)),
        # 0.1 경계(임계≈1): 0.1은 반환, 직전 값은 재진입
        ("pivot-at-0.1", [0.1, 1e15], [0.1, 0.0], [10.0, 10.0], "NF", INF, (17, None),
         (0, 1.0, 0.1, False, 1, 11, 7)),
        ("pivot-below-0.1", [below(0.1), 1e15], [below(0.1), 0.0], [10.0, 10.0], "NF", INF, (17, None),
         (0, 1.0, below(0.1), True, 2, 19, 9)),
        # 임계 동률: (tol·max)·coef == |pivot|이면 반환
        ("thr-equal", [THR, 1e3], [THR, 0.0], [10.0, 10.0], "NF", INF, (17, None),
         (0, 1.0, THR, False, 1, 11, 7)),
        ("thr-below", [below(THR), 1e3], [below(THR), 0.0], [10.0, 10.0], "NF", INF, (17, None),
         (0, 1.0, below(THR), True, 2, 19, 9)),
        *scale_cases(),
        # 새 tol=tol·(max·coef)=1e-8 < THR: THR 크기 행 1이 2차 후보. 왼쪽부터 곱하면 새 tol=THR이라 후보 0
        ("mul-order", [2e-9, THR, 1e3], [2e-9, 1000 * THR, 0.0], [10.0, 10.0, 10.0], "NNF", INF, (17, None),
         (1, (1000 * THR) / THR, THR, False, 2, 30, 18)),
        # 마지막 원소 NaN이면 maxsd 결과 NaN → 계수 1e-6·임계 NaN → 재진입 안 함
        ("nan-max-last", [5e-9, 1e3, NAN], [5e-9, 0.0, 0.0], [10.0, 10.0, 10.0], "NFN", INF, (17, None),
         (0, 1.0, 5e-9, False, 1, 15, 8)),
        # 가운데 NaN은 다음 원소가 덮는다. NaN alpha는 후보·dense 시험 수에서 빠진다
        ("nan-mid", [5e-9, NAN, 1e3], [5e-9, 0.0, 0.0], [10.0, 10.0, 10.0], "NNF", INF, (17, None),
         (0, 1.0, 5e-9, True, 2, 27, 11)),
        # 1차 패스 2차 선택 ratio == 상한(2·BOUND1/2): 큰 pivot 행 1
        ("second-bound-tie", [1.0, 2.0], [1.0, 2 * BOUND1], [10.0, 10.0], "NN", INF, (17, None),
         (1, BOUND1, 2.0, False, 1, 14, 12)),
        # 1차 패스 후보가 있는데 상한 I: -1·theta=I
        ("candidate-inf-bound", [1024.0], [1e30 * 1024], [10.0], "N", INF, (17, None),
         (-1, INF, 0.0, False, 1, 7, 6)),
    ]
    return specs


def cases():
    result = []
    for case in base.cases():
        # 04b와 같은 설정: S 0, S+0x470 factor 갱신 수 0, S+0x498 0
        result.append(dict(case, name="control-" + case["name"], kind="control", h=H, tol=TOL, factor=[0, None],
                           leave0=0))
    for name, alpha, x, width, marks, cap, factor, expect in new_specs():
        pos, theta, pivot, req, passes, units_sparse, units_dense = expect
        for dense in (False, True):
            result.append(dict(name=name + ("-dense" if dense else "-sparse"), kind="target", alpha=alpha, x=x,
                               width=width, marks=marks, cap=cap, order=list(range(len(x))), dense=dense, h=H, tol=TOL,
                               factor=list(factor), leave0=LEAVE0, row=pos, theta=bits(theta), pivot=bits(pivot),
                               request=req, passes=passes, work=bits(0.25 * (units_dense if dense else units_sparse))))
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
    ratio = c.CFUNCTYPE(c.c_int, *([c.c_double] * 4), pointer, c.c_uint,
                        c.c_int, *([pointer] * 11))(base_addr + 0xef960)

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
        header, lengths = integers(range(n)), integers([1] * n)
        candidates, ratios = integers([0] * n), doubles([0.0] * n)
        workspace = c.create_string_buffer(0x500)
        factors = [c.create_string_buffer(0x200), c.create_string_buffer(0x200)]
        for slot, field, factor, updates in zip((0x470, 0x498), (0x18c, 0x1a4), factors, case["factor"]):
            if updates is not None:
                c.c_void_p.from_buffer(workspace, slot).value = c.addressof(factor)
                c.c_int.from_buffer(factor, field).value = updates
        c.c_double.from_buffer(workspace, 0x148).value = 0.25
        c.c_int.from_buffer(workspace, 0x2f8).value = case["leave0"]
        inputs = (indices, alpha, x, width, marks, header, lengths)
        before = [bytes(v) for v in inputs]
        ws_before, factor_before = bytes(workspace), [bytes(f) for f in factors]
        theta, pivot, work = c.c_double(SENTINEL), c.c_double(SENTINEL), c.c_double(0.0)
        chosen = ratio(case["h"], case["tol"], INF, case["cap"], workspace, n, 0,
                       lengths, header, c.byref(vector), x, width, marks,
                       candidates, ratios, c.byref(theta), c.byref(pivot), c.byref(work))
        row = chosen if chosen < 0 or case["dense"] else case["order"][chosen]
        ws_after = bytearray(bytes(workspace))
        refactor = c.c_int.from_buffer(workspace, 0x294).value
        leaving = c.c_int.from_buffer(workspace, 0x2f8).value
        ws_masked = bytearray(ws_before)
        ws_masked[0x294:0x298] = ws_after[0x294:0x298]
        ws_masked[0x2f8:0x2fc] = ws_after[0x2f8:0x2fc]
        events.append(dict(name=case["name"], chosen=chosen, row=row, theta=bits(theta.value),
                           pivot=bits(pivot.value), work=bits(work.value),
                           inputs_preserved=before == [bytes(v) for v in inputs],
                           workspace_preserved=bytes(ws_masked) == bytes(ws_after),
                           factor_preserved=factor_before == [bytes(f) for f in factors],
                           refactor=refactor, leaving=leaving))
    return dict(schema=SCHEMA, provenance="original-synthetic", binary_sha256=BINARY_SHA,
                source_sha256=[sha(ROOT / s) for s in SOURCES], tool_sha256=sha(Path(__file__)),
                base_tool_sha256=sha(Path(base.__file__)), inputs_sha256=inputs_sha(),
                environment=dict(machine=platform.machine(), system=platform.system(), libc=platform.libc_ver()),
                events=events)


KEYS = {"name", "chosen", "row", "theta", "pivot", "work", "inputs_preserved", "workspace_preserved",
        "factor_preserved", "refactor", "leaving"}


def validate_meta(record):
    expected = dict(schema=SCHEMA, provenance="original-synthetic", binary_sha256=BINARY_SHA,
                    source_sha256=[sha(ROOT / s) for s in SOURCES], tool_sha256=sha(Path(__file__)),
                    base_tool_sha256=sha(Path(base.__file__)), inputs_sha256=inputs_sha())
    for field, want in expected.items():
        assert record[field] == want, ("메타", field, want, record.get(field))
    assert isinstance(record["events"], list) and len(record["events"]) == len(cases()), "필수 사건 누락"
    for case, event in zip(cases(), record["events"], strict=True):
        assert event.keys() == KEYS, (case["name"], "필수 필드 불일치", sorted(KEYS ^ set(event)))
        assert event["name"] == case["name"], (case["name"], "사례 순서", event["name"])


def load(paths):
    assert len(paths) >= 3, "독립 캡처 3회 필요"
    records = [json.loads(Path(p).read_text()) for p in paths]
    for record in records:
        validate_meta(record)
    for path, record in zip(paths[1:], records[1:], strict=True):
        for first, event in zip(records[0]["events"], record["events"], strict=True):
            for field in KEYS:
                assert first[field] == event[field], ("3회 불일치", str(path), first["name"], field,
                                                      first[field], event[field])
    return records


def controls(records):
    stored = base.check([CONTROL_STORED / f"ratio-{i}.json" for i in (1, 2, 3)])  # 04b 기준 자료 자체 검사
    old = {"control-" + e["name"]: e for e in stored["events"]}
    count = 0
    for record in records:
        for event in record["events"]:
            if event["name"] not in old:
                continue
            want = old[event["name"]]
            for field in want:
                got = event[field] if field != "name" else event["name"][len("control-"):]
                assert got == want[field], ("대조군 변화", event["name"], field, want[field], got)
            assert event["workspace_preserved"] and event["factor_preserved"], (event["name"], "대조군 S·factor 변경")
            count += 1
    assert count == 29 * len(records), ("대조군 수", count)
    return count // len(records)


def acceptance(records):
    want_cases = [k for k in cases() if k["kind"] == "target"]
    by_name = {k["name"]: k for k in want_cases}
    reentries = requests = 0
    for event in records[0]["events"]:
        case = by_name.get(event["name"])
        if case is None:
            continue
        chosen = case["row"]  # 새 사례는 sparse 순서가 항등이라 위치=행
        for field, want in dict(chosen=chosen, row=case["row"], theta=case["theta"], pivot=case["pivot"],
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
        reentries += case["passes"] == 2
        requests += case["request"]
    # 목표 사건이 없으면 합격이 아니다: 재진입(2패스 work 일치) 사례와 요청 2 사례가 모두 있어야 한다.
    assert reentries > 0, "재진입 사례 0"
    assert requests > 0, "요청 2 사례 0"
    required = {"reentry-pick", "reentry-flip", "reentry-inf-bound", "reentry-none-470-17", "reentry-none-498-17",
                "reentry-none-null", "second-bound-tie", "candidate-inf-bound", "mul-order", "nan-max-last"}
    seen = {e["name"].rsplit("-", 1)[0] for e in records[0]["events"] if e["name"] in by_name}
    assert required <= seen, ("필수 사례 누락", sorted(required - seen))
    return dict(target=len(want_cases), reentry=reentries, request=requests)


def full(records):
    """메타·3회 일치·대조군·목표 합격을 차례로 검사한다(load와 같은 순서, 메모리 기록용)."""
    assert len(records) >= 3, "독립 캡처 3회 필요"
    for r in records:
        validate_meta(r)
    for r in records[1:]:
        for a, b in zip(records[0]["events"], r["events"], strict=True):
            for field in KEYS:
                assert a[field] == b[field], ("3회 불일치", a["name"], field)
    controls(records)
    return acceptance(records)


def self_test(paths):
    global cases
    records = load(paths)
    full(records)
    events = records[0]["events"]
    first_target = next(i for i, e in enumerate(events) if not e["name"].startswith("control-"))
    request_target = next(i for i, e in enumerate(events) if e["refactor"] == 2)

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
            del r["events"][first_target]["refactor"]

    mutations = {
        "binary-hash-0": meta("binary_sha256", "0" * 64),
        "inputs-hash-0": meta("inputs_sha256", "0" * 64),
        "tool-hash-0": meta("tool_sha256", "0" * 64),
        "source-hash-0": meta("source_sha256", ["0" * 64, "0" * 64]),
        "events-empty": meta("events", []),
        "provenance-natural": meta("provenance", "natural"),
        "missing-field": drop_field,
        "run-differs": edit(first_target, "work", bits(0.0), all_runs=False),
        "two-runs": lambda rs: rs.pop(),
        "control-theta": edit(0, "theta", bits(3.0)),
        "target-theta": edit(first_target, "theta", bits(3.0)),
        "target-pos": edit(first_target, "chosen", 0),
        "target-work-1pass": edit(first_target, "work", bits(0.25 * 11)),
        "target-pivot-nan": edit(first_target, "pivot", bits(NAN)),
        "request-cleared": edit(request_target, "refactor", 0),
        "leaving-unwritten": edit(request_target, "leaving", LEAVE0),
        "spurious-request": edit(first_target, "refactor", 2),
        "workspace-changed": edit(first_target, "workspace_preserved", False),
        "factor-changed": edit(first_target, "factor_preserved", False),
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
    # 목표 사건 0회: 사례 정의에서 재진입(또는 요청 2) 사례를 빼고 기록도 같이 빼면 메타 검사 없이 목표 검사만 남는다.
    original = cases
    for name, drop in (("reentry-0", lambda k: k.get("passes") == 2), ("request-0", lambda k: k.get("request"))):
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
            n = controls(load(args.controls))
            print(json.dumps(dict(controls=n, runs=len(args.controls)), ensure_ascii=False))
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
