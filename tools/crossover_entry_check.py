"""M3-04a-1 캡처의 목표 사건과 필수 상태 검사. 미완성 캡처는 실패한다."""
import argparse
import hashlib
import json
import re
import sys
from pathlib import Path

HASH = re.compile(r"[0-9a-f]{64}\Z")
EXPECTED = "8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c"


def require(ok, field):
    if not ok:
        raise ValueError(field)


def check(data):
    require(data.get("origin") == "natural", "실제 실행 출처")
    kind = data.get("kind")
    require(kind in ("pdhg", "barrier", "warm"), "실제 목표 사례 kind")
    for key in ("model_sha256", "binary_sha256", "tool_sha256"):
        v = data.get(key, "")
        require(isinstance(v, str) and HASH.fullmatch(v) and v != "0" * 64, key)
    require(data["binary_sha256"] == EXPECTED, "binary_sha256 기준 불일치")
    model = Path("bench/oracle/m3_crossover_entry/small.lp")
    require(data["model_sha256"] == hashlib.sha256(model.read_bytes()).hexdigest(), "입력 파일 해시 불일치")
    require(data.get("result", {}).get("status") == 2 and not data.get("errors"), "정상 완료 상태")
    events = data.get("events")
    require(isinstance(events, list) and events, "events")
    mode = 1 if kind == "warm" else 0
    target = [e for e in events if e.get("event") == "body:enter" and e.get("mode") == mode]
    require(target, "목표 body 사건 0회")
    stack = []
    for e in events:
        name = e.get("event", "")
        if name.endswith(":enter"):
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
                require(isinstance(r.get(k), list) and len(r[k]) == length, "result." + k)
        else:
            start = s.get("start") or {}
            require(start.get("status") == 2 and isinstance(start.get("values"), list) and len(start["values"]) == n + m, "start.values")
        start_index = events.index(event)
        prepared = next((e for e in events[start_index + 1:] if e.get("event") == "prepared"), None)
        require(prepared is not None and prepared.get("model") == s.get("model"), "준비 사건")
        for key, size in (("x", n), ("slack", m), ("reduced", n), ("y", m)):
            raw(prepared.get(key), size, 8, "prepared." + key)
        leave = next((e for e in events if e.get("event") == "body:leave" and e.get("call") == event.get("call")), None)
        require(leave is not None and leave.get("ret") == 0, "body 반환")
        ws = (leave.get("state") or {}).get("workspace") or {}
        require(isinstance(ws.get("vstat"), list) and len(ws["vstat"]) == ws.get("rows", -1) + ws.get("cols", -1), "출력 workspace.vstat")
        require((leave["state"].get("env")) == s["env"], "환경 복원")
    required = {"pdhg": ["pdhg:enter", "helper:enter", "dispatch:enter"], "barrier": ["barrier:enter", "dispatch:enter"], "warm": ["warm:enter"]}[kind]
    for name in required:
        require(any(e.get("event") == name and e.get("state", {}).get("model") == target[0]["state"]["model"] for e in events), name + " 모델 문맥")
    return len(target)


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
    ap.add_argument("files", nargs="+")
    args = ap.parse_args()
    try:
        records = [json.loads(Path(p).read_text()) for p in args.files]
        for p, data in zip(args.files, records):
            print(p, "목표 사건", check(data))
        for p, data in zip(args.files[1:], records[1:]):
            diff = difference(records[0], data, case=p)
            require(not diff, diff)
        raise ValueError("검사기 미완성: 복사·변환 식과 경로별 중첩 검증 필요")
    except (ValueError, OSError, TypeError, KeyError) as exc:
        print("실패:", exc, file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
