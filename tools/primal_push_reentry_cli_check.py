"""M3-04c-5: 변조한 캡처 파일로 primal_push_reentry_oracle.py CLI(--controls·--acceptance)가 0이 아닌 값으로
끝나는지 실제 종료 코드로 확인하고 기록한다. 변조 파일은 임시 디렉터리에만 만든다.

  python3 tools/primal_push_reentry_cli_check.py <새 결과 경로>.json
"""
import copy
import json
from pathlib import Path
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[1]
TOOL = ROOT / "tools/primal_push_reentry_oracle.py"
CAPS = [ROOT / f"bench/oracle/m3_reentry/reentry-{i}.json" for i in (1, 2, 3)]


def run(mode, paths):
    return subprocess.run([sys.executable, str(TOOL), mode, *map(str, paths)], cwd=ROOT, capture_output=True,
                          text=True).returncode


def main():
    out = Path(sys.argv[1])
    assert not out.exists(), "검증 기록 덮어쓰기 거부"
    records = [json.loads(p.read_text()) for p in CAPS]
    target = next(i for i, e in enumerate(records[0]["events"]) if not e["name"].startswith("control-"))
    request = next(i for i, e in enumerate(records[0]["events"]) if e["refactor"] == 2)
    reentry_names = {e["name"] for e in records[0]["events"]
                     if e["name"].startswith(("reentry-", "pivot-below", "thr-below", "scale-below", "mul-order",
                                              "nan-mid"))}

    def meta(field, v):
        return lambda r: r.__setitem__(field, v)

    def ev(i, field, v):
        return lambda r: r["events"][i].__setitem__(field, v)

    mutations = {
        "binary-hash-0": meta("binary_sha256", "0" * 64),
        "inputs-hash-0": meta("inputs_sha256", "0" * 64),
        "tool-hash-0": meta("tool_sha256", "0" * 64),
        "events-empty": meta("events", []),
        "missing-field": lambda r: r["events"][target].pop("refactor"),
        "reentry-events-removed": lambda r: r.__setitem__(
            "events", [e for e in r["events"] if e["name"] not in reentry_names]),
        "request-cleared": ev(request, "refactor", 0),
        "target-work-1pass": ev(target, "work", "0000000000000640"),
        "control-theta": ev(0, "theta", "0000000000000840"),
    }
    result = dict(baseline={m: run(m, CAPS) for m in ("--controls", "--acceptance")}, mutations={})
    assert result["baseline"] == {"--controls": 0, "--acceptance": 0}, result
    with tempfile.TemporaryDirectory() as tmp:
        for name, fn in mutations.items():
            paths = []
            for i, record in enumerate(records):
                changed = copy.deepcopy(record)
                fn(changed)
                path = Path(tmp) / f"{name}-{i}.json"
                path.write_text(json.dumps(changed))
                paths.append(path)
            codes = {m: run(m, paths) for m in ("--controls", "--acceptance")}
            result["mutations"][name] = codes
            # control-theta는 대조군 검사, 나머지는 목표 합격 검사가 거부해야 한다.
            key = "--controls" if name == "control-theta" else "--acceptance"
            assert codes[key] != 0, ("CLI가 변조를 통과시킴", name, codes)
    out.write_text(json.dumps(result, ensure_ascii=False, indent=2) + "\n")
    print(json.dumps(result, ensure_ascii=False))


if __name__ == "__main__":
    main()
