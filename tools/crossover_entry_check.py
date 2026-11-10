"""M3-04a-1 캡처의 목표 사건과 필수 상태 검사. 미완성 캡처는 실패한다."""
import argparse
import hashlib
import json
import re
import sys
import struct
from pathlib import Path

HASH = re.compile(r"[0-9a-f]{64}\Z")
EXPECTED = "8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c"


def require(ok, field):
    if not ok:
        raise ValueError(field)


def transformed(result, scale):
    def number(x):
        return struct.unpack("<d", bytes.fromhex(x))[0]
    def bits(x):
        return struct.pack("<d", x).hex()
    def negate(x):
        b = bytearray.fromhex(x)
        b[7] ^= 128
        return b.hex()
    x, slack, reduced, y = (list(result[k]) for k in ("x", "slack", "reduced", "y"))
    if scale["c"] is not None:
        o = number(scale["o"][0])
        for j, c in enumerate(scale["c"]):
            c = number(c)
            x[j] = bits(number(x[j]) * c)
            reduced[j] = bits(number(reduced[j]) / (c * o))
        for i, r in enumerate(scale["r"]):
            r = number(r)
            slack[i] = bits(number(slack[i]) / r)
            y[i] = bits((r / o) * number(y[i]))
    y = [negate(v) for v in y]
    if scale["flip_enabled"] == 1:
        for j, flag in enumerate(scale["flip"]):
            if flag == "01":
                x[j], reduced[j] = negate(x[j]), negate(reduced[j])
    for i, sense in enumerate(scale["sense"]):
        if sense == "3e":
            slack[i], y[i] = negate(slack[i]), negate(y[i])
    return x + slack, reduced + y


def check(data):
    require(data.get("origin") == "natural", "실제 실행 출처")
    kind = data.get("kind")
    require(kind in ("pdhg", "barrier", "warm"), "실제 목표 사례 kind")
    for key in ("model_sha256", "binary_sha256", "tool_sha256"):
        v = data.get(key, "")
        require(isinstance(v, str) and HASH.fullmatch(v) and v != "0" * 64, key)
    require(data["binary_sha256"] == EXPECTED, "binary_sha256 기준 불일치")
    model = Path(__file__).resolve().parent.parent / "bench/oracle/m3_crossover_entry/small.lp"
    require(data["model_sha256"] == hashlib.sha256(model.read_bytes()).hexdigest(), "입력 파일 해시 불일치")
    tool = Path(__file__).resolve().parent / "crossover_entry_capture.js"
    require(data["tool_sha256"] == hashlib.sha256(tool.read_bytes()).hexdigest(), "도구 해시 불일치")
    params = {"Threads": 1, "Seed": 0, "OutputFlag": 0, "Presolve": 0, "Method": {"pdhg": 6, "barrier": 2, "warm": 1}[kind], "Crossover": 1, "LPWarmStart": 2}
    require(data.get("params") == params, "파라미터")
    require(data.get("starts") == ({"PStart": [2, 1, 1], "DStart": [1, 0, 0]} if kind == "warm" else None) and "starts" in data, "시작 벡터 메타데이터")
    require((data.get("result") or {}).get("status") == 2 and not data.get("errors"), "정상 완료 상태")
    events = data.get("events")
    require(isinstance(events, list) and events, "events")
    mode = 1 if kind == "warm" else 0
    target = [e for e in events if e.get("event") == "body:enter" and e.get("mode") == mode]
    require(target, "목표 body 사건 0회")
    stack = []
    for e in events:
        name = e.get("event", "")
        if name.endswith(":enter"):
            if name == "body:enter" and e in target:
                need = {"pdhg": ["pdhg", "helper", "dispatch"], "barrier": ["barrier", "dispatch"], "warm": ["warm"]}[kind]
                names = [x[0] for x in stack]
                cursor = 0
                for parent in need:
                    require(parent in names[cursor:], "부모 호출 순서: " + parent)
                    cursor = names.index(parent, cursor) + 1
                for parent, call in stack:
                    if parent in need:
                        entry = next(v for v in events if v.get("event") == parent + ":enter" and v.get("call") == call)
                        require(entry.get("state", {}).get("model") == e.get("state", {}).get("model"), "부모 모델 불일치")
            stack.append((name[:-6], e.get("call")))
        elif name.endswith(":leave"):
            require(bool(stack) and stack[-1] == (name[:-6], e.get("call")), "호출 중첩·순서")
            stack.pop()
    require(not stack, "미반환 호출")
    for event in target:
        s = event.get("state") or {}
        n, m = s.get("n"), s.get("m")
        require(isinstance(n, int) and n > 0 and isinstance(m, int) and m > 0, "차원")
        env, scale = s.get("env") or {}, s.get("scale") or {}
        require(set(scale) == {"c", "r", "o", "flip", "flip_enabled", "sense"}, "scale 필드 누락")
        def raw(value, count, width, name, nullable=False):
            if nullable and value is None:
                return
            require(isinstance(value, list) and len(value) == count and all(isinstance(v, str) and re.fullmatch("[0-9a-f]{%d}" % (2 * width), v) for v in value), name)
        raw(env.get("field_2478"), 1, 8, "env.field_2478")
        raw(env.get("field_24a0"), 1, 8, "env.field_24a0")
        require(type(env.get("field_26fc")) is int, "env.field_26fc")
        raw(scale.get("c"), n, 8, "scale.c", True)
        raw(scale.get("r"), m, 8, "scale.r", scale.get("c") is None)
        raw(scale.get("o"), 1, 8, "scale.o")
        raw(scale.get("flip"), n, 1, "scale.flip", scale.get("flip_enabled") != 1)
        raw(scale.get("sense"), m, 1, "scale.sense")
        require(type(scale.get("flip_enabled")) is int, "scale.flip_enabled")
        if mode == 0:
            r = s.get("result") or {}
            for k, length in (("x", n), ("slack", m), ("reduced", n), ("y", m)):
                raw(r.get(k), length, 8, "result." + k)
        else:
            start = s.get("start") or {}
            require(start.get("status") == 2 and isinstance(start.get("values"), list) and len(start["values"]) == n + m, "start.values")
        start_index = events.index(event)
        if mode == 0:
            dispatch = next((e for e in reversed(events[:start_index]) if e.get("event") == "dispatch:enter" and e.get("state", {}).get("model") == s.get("model")), None)
            require(dispatch is not None, "dispatch 문맥")
            ds = dispatch["state"]
            copy = ds.get("copy") or {}
            primal, dual = transformed(ds["result"], ds["scale"])
            for key, expected in (("primal", primal), ("dual", dual)):
                diff = difference(expected, copy.get(key), field="S." + key, case=kind)
                require(not diff, diff)
        prepared = next((e for e in events[start_index + 1:] if e.get("event") == "prepared"), None)
        require(prepared is not None and prepared.get("model") == s.get("model"), "준비 사건")
        for key, size in (("x", n), ("slack", m), ("reduced", n), ("y", m)):
            raw(prepared.get(key), size, 8, "prepared." + key)
        leave = next((e for e in events if e.get("event") == "body:leave" and e.get("call") == event.get("call")), None)
        require(leave is not None and leave.get("ret") == 0 and events.index(prepared) < events.index(leave), "body 반환·준비 순서")
        require(type(leave.get("state", {}).get("status")) is int, "반환 모델 상태")
        ws = (leave.get("state") or {}).get("workspace") or {}
        require(type(ws.get("rows")) is int and type(ws.get("cols")) is int and ws["rows"] >= 0 and ws["cols"] >= 0, "workspace 차원")
        raw(ws.get("vstat"), ws["rows"] + ws["cols"], 4, "출력 workspace.vstat")
        require((leave["state"].get("env")) == s["env"], "환경 복원")
    required = {"pdhg": ["pdhg:enter", "helper:enter", "dispatch:enter"], "barrier": ["barrier:enter", "dispatch:enter"], "warm": ["warm:enter"]}[kind]
    for name in required:
        require(any(e.get("event") == name and e.get("state", {}).get("model") == target[0]["state"]["model"] for e in events), name + " 모델 문맥")
    return len(target)


def check_control(data):
    require(data.get("origin") == "natural" and data.get("kind") == "control", "대조군 실제 출처")
    root = Path(__file__).resolve().parent.parent
    require(data.get("model_sha256") == hashlib.sha256((root / "bench/oracle/m3_crossover_entry/small.lp").read_bytes()).hexdigest(), "대조군 입력 해시")
    require(data.get("tool_sha256") == hashlib.sha256((root / "tools/crossover_entry_capture.js").read_bytes()).hexdigest(), "대조군 도구 해시")
    require(data.get("binary_sha256") == EXPECTED, "대조군 바이너리 해시")
    require(data.get("params") == {"Threads": 1, "Seed": 0, "OutputFlag": 0, "Presolve": 0, "Method": 2, "Crossover": 0, "LPWarmStart": 2}, "대조군 설정")
    require(data.get("starts", "missing") is None and (data.get("result") or {}).get("status") == 2 and not data.get("errors"), "대조군 정상 종료")
    events = data.get("events") or []
    require(any(e.get("event") == "barrier:enter" for e in events), "대조군 barrier 사건 없음")
    require(not any(e.get("event") in ("dispatch:enter", "body:enter") for e in events), "대조군 목표 사건 발생")
    return 0


def difference(a, b, field="$", case=""):
    if type(a) is not type(b):
        return f"{case} {field}: 기준={a!r}, 실제={b!r}"
    if isinstance(a, dict):
        for key in sorted(set(a) | set(b)):
            if key not in a or key not in b:
                return f"{case} {field}.{key}: 기준={a.get(key, '<없음>')!r}, 실제={b.get(key, '<없음>')!r}"
            result = difference(a[key], b[key], f"{field}.{key}", case)
            if result:
                return result
    elif isinstance(a, list):
        if len(a) != len(b):
            return f"{case} {field}.length: 기준={len(a)}, 실제={len(b)}"
        for i, (x, y) in enumerate(zip(a, b)):
            result = difference(x, y, f"{field}[{i}]", case)
            if result:
                return result
    elif a != b:
        return f"{case} {field}: 기준={a!r}, 실제={b!r}"
    return None


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--control", action="store_true")
    ap.add_argument("files", nargs="+")
    args = ap.parse_args()
    try:
        records = [json.loads(Path(p).read_text()) for p in args.files]
        for p, data in zip(args.files, records):
            print(p, "대조군 목표 사건" if args.control else "목표 사건", check_control(data) if args.control else check(data))
        for p, data in zip(args.files[1:], records[1:]):
            baseline = {k: v for k, v in records[0].items() if k != "output"}
            current = {k: v for k, v in data.items() if k != "output"}
            diff = difference(baseline, current, case=p)
            require(not diff, diff)
        if not args.control:
            raise ValueError("검사기 미완성: 실제 원본으로 필드와 경로별 검증 필요")
    except (ValueError, OSError, TypeError, KeyError) as exc:
        print("실패:", exc, file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
