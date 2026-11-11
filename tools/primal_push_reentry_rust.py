"""M3-04c-5: 원본 재진입 합성 캡처를 Rust feasible_ratio 입력으로 투영하고, Rust trace를 원본 출력과 비트 비교한다.

입력 줄 형식은 primal_push_ratio_rust.py(M3-04c)에 pivot 초기값과 12dd49 반환값을 더한 것이고, Rust 테스트
push_ratio::reentry_trace가 읽는다. 12dd49 반환값은 캡처의 factor 설정에서 C·asm(0x1bb6f0)대로 정한다:
S+0x470이 있으면 +0x18c, 없으면 S+0x498의 +0x1a4, 둘 다 없으면 -1.

  --write                      원본 캡처 → bench/oracle/m3_reentry/rust_input.txt
  --compare                    Rust trace 비교(cargo test -p rsolver-lp --test push_ratio 뒤)
  --self-test                  비교기 변조 거부
  --mutate-rust <새 경로>.json  Rust 소스 변이(04c 재생 비교와 이 비교 둘 다 실행)
"""
import argparse
import copy
import hashlib
import json
from pathlib import Path
import struct
import subprocess
import sys

sys.path.insert(0, str(Path(__file__).resolve().parent))
import primal_push_reentry_oracle as oracle  # noqa: E402
import primal_push_ratio_rust as r04c  # noqa: E402  04c 재생 비교(변이 때 함께 실행)

ROOT = Path(__file__).resolve().parents[1]
CAPS = [ROOT / f"bench/oracle/m3_reentry/reentry-{i}.json" for i in (1, 2, 3)]
INPUT = ROOT / "bench/oracle/m3_reentry/rust_input.txt"
TRACE = ROOT / "target/m3_reentry/trace.txt"
FIELDS = ("pos", "row", "theta", "pivot", "work", "refactor")
# 2차 패스가 후보 없이 또는 상한 I로 끝나 12dd49를 부르는 사례(asm 0xeff6d로 파생, 원본에서 호출 자체는 관측 안 함)
ASKS = ("reentry-inf-bound", "reentry-none", "pivot-below-0.1", "thr-below", "scale-below", "nan-mid")


def bits(v):
    return struct.pack("<d", v).hex()


def updates(factor):
    u470, u498 = factor
    return u470 if u470 is not None else (u498 if u498 is not None else -1)


def collect():
    records = oracle.load(CAPS)  # 메타·3회 일치
    oracle.controls(records)  # 대조군
    oracle.acceptance(records)  # 사전 기대
    cases = []
    for c, e in zip(oracle.cases(), records[0]["events"], strict=True):
        assert c["name"] == e["name"]
        dense = c["dense"]
        asks = int(c["kind"] == "target" and c["name"].startswith(ASKS))
        cases.append(dict(
            name=c["name"], dense=dense, m=len(c["x"]),
            scal=[bits(c["h"]), bits(c["tol"]), bits(oracle.INF), bits(c["cap"]), bits(0.25), bits(0.0),
                  bits(oracle.SENTINEL), bits(oracle.SENTINEL)],
            updates=updates(c["factor"]), idx=None if dense else c["order"],
            val=[bits(v) for v in (c["alpha"] if dense else [c["alpha"][i] for i in c["order"]])],
            x=[bits(v) for v in c["x"]], width=[bits(v) for v in c["width"]],
            marks=[ch.encode().hex() for ch in c["marks"]],
            expect=dict(pos=e["chosen"], row=e["row"], theta=e["theta"], pivot=e["pivot"], work=e["work"],
                        refactor=int(e["refactor"] == 2)),
            asks=asks))
        assert (e["refactor"], e["leaving"]) in ((2, -1), (0, c["leave0"])), (c["name"], "S 칸")
    return cases


def line(c):
    parts = [c["name"], "1" if c["dense"] else "0", str(c["m"]), *c["scal"], str(c["updates"])]
    if not c["dense"]:
        parts += [str(len(c["idx"])), *map(str, c["idx"])]
    return " ".join(parts + c["val"] + c["x"] + c["width"] + c["marks"])


def render(cases):
    body = "".join(line(c) + "\n" for c in cases)
    return f"input {hashlib.sha256(body.encode()).hexdigest()}\n" + body


def compare(cases, text, trace):
    assert text == render(cases), "입력 텍스트가 원본 캡처 투영과 다름(--write 필요)"
    rows = trace.splitlines()
    assert rows and rows[0] == text.splitlines()[0], "trace의 입력 해시 불일치(오래된 trace)"
    assert len(rows) == len(cases) + 1, ("사례 수", len(cases), len(rows) - 1)
    reentry = request = 0
    for c, row in zip(cases, rows[1:], strict=True):
        t = row.split()
        assert t[0] == c["name"], ("사례 순서", c["name"], t[0])
        assert len(t) == 7, (c["name"], "필드 수", t)
        pos = int(t[1])
        got = dict(pos=pos, row=pos if pos < 0 or c["dense"] else c["idx"][pos], theta=t[2], pivot=t[3], work=t[4],
                   refactor=int(t[5]))
        for f in FIELDS:
            assert got[f] == c["expect"][f], (c["name"], f, "기준", c["expect"][f], "실제", got[f], "정확·비트 비교")
        assert int(t[6]) == c["asks"], (c["name"], "12dd49 조회 수(asm 파생)", c["asks"], t[6])
        reentry += c["name"] in REENTRY
        request += c["expect"]["refactor"]
    assert reentry > 0 and request > 0, ("목표 사건 0", reentry, request)
    return dict(cases=len(cases), control=sum(c["name"].startswith("control-") for c in cases),
                reentry=reentry, request=request, fields=len(FIELDS) + 1)


REENTRY = {c["name"] for c in oracle.cases() if c.get("passes") == 2}


def self_test(cases, text, trace):
    compare(cases, text, trace)
    rows = trace.splitlines()
    names = [r.split()[0] for r in rows[1:]]
    target = 1 + next(i for i, n in enumerate(names) if n.startswith("reentry-pick"))
    req = 1 + next(i for i, c in enumerate(cases) if c["expect"]["refactor"])
    asks = 1 + next(i for i, c in enumerate(cases) if c["asks"] and not c["expect"]["refactor"])

    def edit(i, k, v):
        t = rows[i].split()
        t[k] = v
        return "\n".join(rows[:i] + [" ".join(t)] + rows[i + 1:]) + "\n"
    bad = {
        "pos": edit(target, 1, "0"), "theta": edit(target, 2, bits(3.0)), "pivot": edit(target, 3, bits(0.0)),
        "work-1pass": edit(target, 4, bits(0.25 * 11)), "request-missing": edit(req, 5, "0"),
        "spurious-request": edit(target, 5, "1"), "asks": edit(asks, 6, "0"),
        "missing-case": "\n".join(rows[:-1]) + "\n", "stale-hash": "\n".join(["input " + "0" * 64] + rows[1:]) + "\n",
        "empty": rows[0] + "\n",
    }
    rejected = []
    for name, t in bad.items():
        try:
            compare(cases, text, t)
        except AssertionError:
            rejected.append(name)
            continue
        raise AssertionError(("변조 허용", name))
    for name, fn in (("input-drift", lambda cs: cs[target - 1]["x"].__setitem__(0, bits(1.0))),
                     ("updates-drift", lambda cs: cs[req - 1].__setitem__("updates", 16))):
        changed = copy.deepcopy(cases)
        fn(changed)
        try:
            compare(changed, text, trace)
        except AssertionError:
            rejected.append(name)
        else:
            raise AssertionError(("변조 허용", name))
    return rejected


RUST = ROOT / "crates/lp/src/crossover.rs"
# feasible_ratio 재진입 분기별 Rust 소스 변이. 각 변이는 원본 대조(04c 재생 또는 이 재생)에서 실패해야 한다.
MUTANTS = {
    "no-reentry": ("let reenter = tol * big * scale > *pivot;", "let reenter = false;"),
    "pivot-threshold-le": ("let small = first && 0.1 > *pivot;", "let small = first && 0.1 >= *pivot;"),
    "third-pass": ("let small = first && 0.1 > *pivot;", "let small = 0.1 > *pivot;"),
    "thr-tie": ("let reenter = tol * big * scale > *pivot;", "let reenter = tol * big * scale >= *pivot;"),
    "mul-order": ("tol *= big * scale;", "tol = tol * big * scale;"),
    "scale-10": ("b if 10.0 > b => 0.1,", "b if 10.0 >= b => 0.1,"),
    "scale-1e4": ("b if 1e4 > b => 0.01,", "b if 1e4 >= b => 0.01,"),
    "scale-1e7": ("b if 1e7 > b => 0.001,", "b if 1e7 >= b => 0.001,"),
    "scale-1e10": ("b if 1e10 > b => 1e-4,", "b if 1e10 >= b => 1e-4,"),
    "scale-1e13": ("b if 1e13 > b => 1e-5,", "b if 1e13 >= b => 1e-5,"),
    "max-fmax": ("fold(0.0f64, |m, v| if m > v.abs() { m } else { v.abs() })", "fold(0.0f64, |m, v| m.max(v.abs()))"),
    "max-no-tail": ("let big = val[..count].iter()", "let big = val[..count & !1].iter()"),
    "nan-up": ("let up = a > tol && marks[r] != b'F';", "let up = !(a <= tol) && marks[r] != b'F';"),
    "keep-first-pick": ("chosen = k as i32;\n                let relaxed = if up",
                        "if first { chosen = k as i32; }\n                let relaxed = if up"),
    "request-ge16": ("if factor_updates() > 16 {", "if factor_updates() >= 16 {"),
    "request-never": ("*refactor = true;", "*refactor = false;"),
    "theta-reset": ("            if factor_updates() > 16 {", "            *theta = inf;\n            if factor_updates() > 16 {"),
    "ask-always": ("            if factor_updates() > 16 {\n                *refactor = true;\n            }\n            return chosen;",
                   "            if factor_updates() > 16 {\n                *refactor = true;\n            }\n            factor_updates();\n            return chosen;"),
    "second-bound-strict": ("for (&ratio, &k) in ratios.iter().zip(&cands) {\n            if ratio <= bound",
                            "for (&ratio, &k) in ratios.iter().zip(&cands) {\n            if ratio < bound"),
    "cand-inf-bound": ("if bound == inf || cands.is_empty() {\n            if first",
                       "if cands.is_empty() {\n            if first"),
    "work-once": ("*work += units * work_coef;\n        if cap < inf", "if first { *work += units * work_coef; }\n        if cap < inf"),
    "flip-first-pass-only": ("        if cap < inf && cap <= bound {\n            *pivot = 0.0;\n            *theta = cap;\n            return -1;\n        }\n        if bound == inf",
                             "        if first && cap < inf && cap <= bound {\n            *pivot = 0.0;\n            *theta = cap;\n            return -1;\n        }\n        if bound == inf"),
}


def mutate_rust(cases, text):
    original, result = RUST.read_text(), {}
    old_cases, old_text = r04c.natural() + r04c.synthetic(), r04c.INPUT.read_text()
    try:
        for name, (old, new) in MUTANTS.items():
            assert original.count(old) == 1, ("변이 대상이 하나가 아님", name, original.count(old))
            RUST.write_text(original.replace(old, new))
            TRACE.unlink(missing_ok=True)
            r04c.TRACE.unlink(missing_ok=True)
            run = subprocess.run(["cargo", "test", "-q", "-p", "rsolver-lp", "--test", "push_ratio"], cwd=ROOT,
                                 capture_output=True, text=True)
            verdicts = []
            for label, fn in (("04c", lambda: r04c.compare(old_cases, old_text, r04c.TRACE.read_text())),
                              ("04c-5", lambda: compare(cases, text, TRACE.read_text()))):
                try:
                    fn()
                    verdicts.append(f"{label} 통과")
                except (AssertionError, FileNotFoundError) as error:
                    verdicts.append(f"{label} 거부: {str(error)[:140]}")
            rejected = any("거부" in v for v in verdicts)
            result[name] = dict(verdict="거부" if rejected else "생존", cargo_rc=run.returncode, detail=verdicts)
    finally:
        RUST.write_text(original)
        TRACE.unlink(missing_ok=True)
        r04c.TRACE.unlink(missing_ok=True)
    return result


def main():
    p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    g = p.add_mutually_exclusive_group(required=True)
    g.add_argument("--write", action="store_true")
    g.add_argument("--compare", action="store_true")
    g.add_argument("--self-test", action="store_true")
    g.add_argument("--mutate-rust", type=Path, help="변이 결과 JSON 경로(덮어쓰기 거부)")
    args = p.parse_args()
    try:
        cases = collect()
        if args.write:
            INPUT.write_text(render(cases))
            print(f"{len(cases)}사례 저장: {INPUT.relative_to(ROOT)}")
            return
        text = INPUT.read_text()
        if args.mutate_rust:
            assert not args.mutate_rust.exists(), "검증 기록 덮어쓰기 거부"
            result = mutate_rust(cases, text)
            args.mutate_rust.write_text(json.dumps(result, ensure_ascii=False, indent=2) + "\n")
            print(json.dumps({k: v["verdict"] for k, v in result.items()}, ensure_ascii=False))
        elif args.compare:
            print(json.dumps(compare(cases, text, TRACE.read_text()), ensure_ascii=False))
        else:
            print(json.dumps({"rejected": self_test(cases, text, TRACE.read_text())}, ensure_ascii=False))
    except (AssertionError, KeyError, ValueError, IndexError, FileNotFoundError) as error:
        print("불합격: " + str(error), file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
