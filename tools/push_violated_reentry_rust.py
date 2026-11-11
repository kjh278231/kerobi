"""M3-04c-9: 054eef·056973 재진입 원본 합성 캡처를 Rust infeasible_ratio·retry_ratio 입력으로 투영하고 trace를 비트 비교한다.

입력 줄은 함수 이름(054eef·056973) 뒤에 primal_push_reentry_rust.py(M3-04c-5)와 같은 형식이고, Rust 테스트
push_ratio::violated_reentry_trace가 읽는다. 12dd49 반환값은 캡처의 factor 설정에서 C·asm(0x1bb6f0)대로 정한다.
대조군(04c-2 57·04c-7 74호출)도 같은 형식으로 전부 비교한다(04c-7 reentry-keep·refactor 포함).

  --write                      원본 캡처 → bench/oracle/m3_violated_reentry/rust_input.txt
  --compare                    Rust trace 비교(cargo test -p rsolver-lp --test push_ratio 뒤)
  --self-test                  비교기 변조 거부
  --mutate-rust <새 경로>.json  Rust 소스 변이(04c·04c-2·04c-5·04c-7 재생과 이 재생을 함께 실행)
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
import push_violated_reentry_oracle as oracle  # noqa: E402

ROOT = Path(__file__).resolve().parents[1]
CAPS = [ROOT / f"bench/oracle/m3_violated_reentry/reentry-{i}.json" for i in (1, 2, 3)]
INPUT = ROOT / "bench/oracle/m3_violated_reentry/rust_input.txt"
TRACE = ROOT / "target/m3_violated_reentry/trace.txt"
RUST = ROOT / "crates/lp/src/crossover.rs"
FIELDS = ("pos", "row", "theta", "pivot", "work", "refactor")
# 요청 2가 없는데 12dd49를 부르는 사례(054eef 0xef671, 056973 0xf0a32로 파생. 원본에서 호출 자체는 관측 안 함)
ASK_NO_REQUEST = {"inf-reentry-inf-bound-keep", "inf-reentry-none-470-16", "inf-reentry-none-null",
                  "ret-reentry-none-470-16", "ret-reentry-none-null", "control-ret-reentry-keep"}


def bits(v):
    return struct.pack("<d", v).hex()


def updates(factor):
    u470, u498 = factor
    return u470 if u470 is not None else (u498 if u498 is not None else -1)


def collect():
    records = oracle.load(CAPS)  # 메타·3회 일치
    oracle.controls(records)
    oracle.acceptance(records)
    cases = []
    for c, e in zip(oracle.cases(), records[0]["events"], strict=True):
        assert c["name"] == e["name"]
        assert (e["refactor"], e["leaving"]) in ((2, -1), (0, c["leave0"])), (c["name"], "S 칸")
        dense = c["dense"]
        base = c["name"].rsplit("-", 1)[0]
        cases.append(dict(
            fn=c["fn"], name=c["name"], dense=dense, m=len(c["x"]),
            scal=[bits(c["h"]), bits(c["tol"]), bits(oracle.INF), bits(c["cap"]), bits(0.25), bits(0.0),
                  bits(oracle.SENTINEL), bits(oracle.SENTINEL)],
            updates=updates(c["factor"]), idx=None if dense else c["order"],
            val=[bits(v) for v in (c["alpha"] if dense else [c["alpha"][i] for i in c["order"]])],
            x=[bits(v) for v in c["x"]], width=[bits(v) for v in c["width"]],
            marks=[ch.encode().hex() for ch in c["marks"]],
            expect=dict(pos=e["chosen"], row=e["row"], theta=e["theta"], pivot=e["pivot"], work=e["work"],
                        refactor=int(e["refactor"] == 2)),
            asks=int(e["refactor"] == 2 or base in ASK_NO_REQUEST)))
    return cases


def line(c):
    parts = [c["fn"], c["name"], "1" if c["dense"] else "0", str(c["m"]), *c["scal"], str(c["updates"])]
    if not c["dense"]:
        parts += [str(len(c["idx"])), *map(str, c["idx"])]
    return " ".join(parts + c["val"] + c["x"] + c["width"] + c["marks"])


def render(cases):
    body = "".join(line(c) + "\n" for c in cases)
    return f"input {hashlib.sha256(body.encode()).hexdigest()}\n" + body


REENTRY = {c["name"] for c in oracle.cases() if c.get("passes") == 2}


def compare(cases, text, trace):
    assert text == render(cases), "입력 텍스트가 원본 캡처 투영과 다름(--write 필요)"
    rows = trace.splitlines()
    assert rows and rows[0] == text.splitlines()[0], "trace의 입력 해시 불일치(오래된 trace)"
    assert len(rows) == len(cases) + 1, ("사례 수", len(cases), len(rows) - 1)
    reentry = {"054eef": 0, "056973": 0}
    request = {"054eef": 0, "056973": 0}
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
        reentry[c["fn"]] += c["name"] in REENTRY
        request[c["fn"]] += c["expect"]["refactor"]
    for fn in reentry:
        assert reentry[fn] > 0 and request[fn] > 0, ("목표 사건 0", fn, reentry[fn], request[fn])
    return dict(cases=len(cases), control=sum(c["name"].startswith("control-") for c in cases),
                reentry=reentry, request=request, fields=len(FIELDS) + 1)


def self_test(cases, text, trace):
    compare(cases, text, trace)
    rows = trace.splitlines()
    names = [r.split()[0] for r in rows[1:]]
    at = {n: i + 1 for i, n in enumerate(names)}
    inf_t, ret_t = at["inf-reentry-pick-sparse"], at["ret-reentry-qs-dense"]
    req_ret, keep_ret = at["ret-reentry-none-470-17-sparse"], at["ret-reentry-none-470-16-sparse"]
    asks = at["inf-reentry-none-470-16-dense"]

    def edit(i, k, v):
        t = rows[i].split()
        t[k] = v
        return "\n".join(rows[:i] + [" ".join(t)] + rows[i + 1:]) + "\n"
    bad = {
        "inf-pos": edit(inf_t, 1, "0"), "inf-theta": edit(inf_t, 2, bits(3.0)), "ret-pivot": edit(ret_t, 3, bits(0.0)),
        "ret-work-1pass": edit(ret_t, 4, bits(0.25 * 32)), "ret-request-missing": edit(req_ret, 5, "0"),
        "ret-request-returns-prev": edit(req_ret, 1, "0"), "ret-keep-returns-minus1": edit(keep_ret, 1, "-1"),
        "spurious-request": edit(inf_t, 5, "1"), "asks": edit(asks, 6, "0"),
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
    for name, fn in (("input-drift", lambda cs: cs[inf_t - 1]["x"].__setitem__(0, bits(1.0))),
                     ("updates-drift", lambda cs: cs[req_ret - 1].__setitem__("updates", 16))):
        changed = copy.deepcopy(cases)
        fn(changed)
        try:
            compare(changed, text, trace)
        except AssertionError:
            rejected.append(name)
        else:
            raise AssertionError(("변조 허용", name))
    return rejected


# 재진입 분기별 Rust 소스 변이(대상 문자열은 정확히 한 번 나와야 한다). 각 변이는 어느 재생 비교에서든 실패해야 한다.
MUTANTS = {
    # 공유 판정 tightened(): 세 ratio 모두에 영향
    "no-reentry": ("(tol * big * scale > pivot).then(|| tol * (big * scale))", "None"),
    "pivot-threshold-le": ("if !(0.1 > pivot) {", "if !(0.1 >= pivot) {"),
    "thr-tie": ("(tol * big * scale > pivot)", "(tol * big * scale >= pivot)"),
    "mul-order": (".then(|| tol * (big * scale))", ".then(|| tol * big * scale)"),
    "max-fmax": ("fold(0.0f64, |m, v| if m > v.abs() { m } else { v.abs() })", "fold(0.0f64, |m, v| m.max(v.abs()))"),
    "max-no-tail": ("let big = val.iter().fold", "let big = val[..val.len() & !1].iter().fold"),
    "scale-1e4": ("b if 1e4 > b => 0.01,", "b if 1e4 >= b => 0.01,"),
    "scale-1e13": ("b if 1e13 > b => 1e-5,", "b if 1e13 >= b => 1e-5,"),
    # 054eef
    "inf-third-pass": ("match first.then(|| tightened(tol, &val[..count], *pivot)).flatten() {\n            Some(t) => tol = t,\n            None => return chosen,\n        }\n        first = false;\n    }\n}\n\n/// The push's",
                       "match tightened(tol, &val[..count], *pivot) {\n            Some(t) => tol = t,\n            None => return chosen,\n        }\n        first = false;\n    }\n}\n\n/// The push's"),
    "inf-old-tol": ("let (up, down) = (a > tol, a < -tol);", "let (up, down) = (a > piv_tol, a < -piv_tol);"),
    "inf-request-ge16": ("            if factor_updates() > 16 {\n                *refactor = true;\n            }\n            return chosen; // the second",
                         "            if factor_updates() >= 16 {\n                *refactor = true;\n            }\n            return chosen; // the second"),
    "inf-return-minus1": ("return chosen; // the second pass's last recorded row", "return -1; // the second pass's last recorded row"),
    "inf-theta-reset": ("            if factor_updates() > 16 {\n                *refactor = true;\n            }\n            return chosen; // the second",
                        "            *theta = inf;\n            if factor_updates() > 16 {\n                *refactor = true;\n            }\n            return chosen; // the second"),
    "inf-cand-inf-bound": ("c + c + n * 4.0 };\n        *work += units * work_coef;\n        if cap < inf && cap <= bound {\n            *pivot = 0.0;\n            *theta = cap;\n            return -1;\n        }\n        if bound == inf || cands.is_empty() {",
                           "c + c + n * 4.0 };\n        *work += units * work_coef;\n        if cap < inf && cap <= bound {\n            *pivot = 0.0;\n            *theta = cap;\n            return -1;\n        }\n        if cands.is_empty() {"),
    "inf-work-once": ("c + c + n * 4.0 };\n        *work += units * work_coef;", "c + c + n * 4.0 };\n        if first { *work += units * work_coef; }"),
    # 056973
    "ret-third-pass": ("match first.then(|| tightened(piv_tol, &val[..count], *pivot)).flatten() {",
                       "match tightened(piv_tol, &val[..count], *pivot) {"),
    "ret-request-returns-prev": ("                    *refactor = true;\n                    return -1;", "                    *refactor = true;\n                    return prev;"),
    "ret-keep-returns-minus1": ("                return prev;\n            }", "                return -1;\n            }"),
    "ret-request-ge16": ("                if factor_updates() > 16 {\n                    *refactor = true;\n                    return -1;",
                         "                if factor_updates() >= 16 {\n                    *refactor = true;\n                    return -1;"),
    "ret-reentry-like-first": ("            if !first {\n                if factor_updates", "            if false {\n                if factor_updates"),
    "ret-prev-stale": ("        prev = chosen;\n", ""),
}


def mutate_rust(cases, text):
    import primal_push_ratio_rust as r04c  # 04c 재생
    import primal_push_reentry_rust as r04c5  # 04c-5 재생
    import push_violated_rust as r04c2  # 04c-2 재생
    import push_retry_rust as r04c7  # 04c-7 재생
    others = [("04c", r04c, r04c.natural() + r04c.synthetic()), ("04c-5", r04c5, r04c5.collect()),
              ("04c-2", r04c2, r04c2.cases()), ("04c-7", r04c7, r04c7.cases())]
    others = [(label, mod, cs, mod.INPUT.read_text()) for label, mod, cs in others]
    original, result = RUST.read_text(), {}
    try:
        for name, (old, new) in MUTANTS.items():
            assert original.count(old) == 1, ("변이 대상이 하나가 아님", name, original.count(old))
            RUST.write_text(original.replace(old, new))
            for mod in (None, *(o[1] for o in others)):
                (TRACE if mod is None else mod.TRACE).unlink(missing_ok=True)
            run = subprocess.run(["cargo", "test", "-q", "-p", "rsolver-lp", "--test", "push_ratio"], cwd=ROOT,
                                 capture_output=True, text=True)
            verdicts = []
            for label, mod, cs, txt in [*others, ("04c-9", None, cases, text)]:
                try:
                    if mod is None:
                        compare(cs, txt, TRACE.read_text())
                    else:
                        mod.compare(cs, txt, mod.TRACE.read_text())
                    verdicts.append(f"{label} 통과")
                except (AssertionError, FileNotFoundError) as error:
                    verdicts.append(f"{label} 거부: {str(error)[:140]}")
            rejected = any("거부" in v for v in verdicts)
            result[name] = dict(verdict="거부" if rejected else "생존", cargo_rc=run.returncode, detail=verdicts)
    finally:
        RUST.write_text(original)
        TRACE.unlink(missing_ok=True)
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
