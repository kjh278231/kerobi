"""Run one ported MIP presolve reduction on Gurobi's state at the entry of a call (tools/mip_state_dump.js output) and
compare the result with Gurobi's state at its return: bounds, rows, activities, counts, counters, work, list orders,
free/pending counts, tracker and new postsolve records (docs/m2_red_inner.md §9). Needs no Gurobi.
usage: PYTHONPATH=python python tools/mip_dump_check.py <fn> <dump.json ...> [--after DUMP] [--param K=V ...]
  <fn>: 3e39a6 36ddf2 37168e 36c5cf 56f793 56c80e 36e6b6 (mode 0 or 1, whichever matches) 370047 implied_bounds
  45885d 52be10 46a62d 5a9282 5ab429 59c193 55a779 374521 5431ee 3e5363 57627e 3ff744:<mode>:<flag>
  3e40d1 36ee92 418542 37f484 4030c9 414c03 414c53 5577ba 838543 423e35 5aaaf7 579e67 5a1293 5a04fc (with an OUT_I32=4 dump, f4 too)
  5a4d8b 5a5eff 419ffc 52cba4 521497.
  --after: the "after" state is the entry state of the same call number in DUMP (for an inline block: dump the
  function before it and the one after it).
Doubles compare with == (the JSON dump loses the sign of -0.0); records compare byte for byte except padding.
"""
import argparse, json, struct, sys

import rsolver  # noqa: F401
from rsolver import _core

FIELDS = ["lb", "ub", "obj", "rhs", "row_cnt", "col_cnt", "inf_max", "inf_min", "act_max", "act_min", "mult_lo",
          "mult_hi", "objcon", "nnz", "work"]
# void functions (the dumped return value is a stale register)
NO_RET = {"56f793", "3e40d1", "57999f"}


def to_parts(s):
    d = {k: s[k] for k in ["n", "m", "lb", "ub", "obj", "rhs", "row_cnt", "col_cnt", "inf_max", "inf_min", "act_max",
                           "act_min", "mult_lo", "mult_hi", "objcon", "work", "work_mult"]}
    d["sense"] = list(s["sense"].encode())
    d["vtype"] = list(s["vtype"].encode())
    d["col_flag"] = s["col_flag"]
    d["nnz"] = int(s["nnz"])
    d["counters"] = s["counters"][:14]  # +0x3f8 .. +0x42c
    d["nfree"], d["npend"] = int(s["free"][0]), int(s["free"][1])
    d["nbin"] = s.get("nbin", -1)  # +0xf0 (counted at bound rounding, not kept up to date)
    d["nint"] = s.get("nint", -1)  # +0xf4 (likewise)
    d["node_cap"], d["chunk"] = int(s.get("node_cap", 0)), int(s.get("chunk", 0))  # 0: not in older dumps
    d["rows"] = [[tuple(x) for x in r] for r in s["rows"]]
    d["cols"] = [[tuple(x) for x in c] for c in s["cols"]]
    t = s["tracker"]
    d["level"] = t["level"]
    d["active"] = [x != 0 for x in t["active"]]
    for k in "rc":
        q = t[k]
        d[f"{k}_flag"], d[f"{k}_a"], d[f"{k}_b"] = q["flag"], q["a"], q["b"]
        d[f"{k}_list"] = [x or [] for x in q["list"]]
        d[f"{k}_cache"], d[f"{k}_cache2"] = q["cache"], q["cache2"]
    return d


def rec_diffs(mine, theirs, loose_zero=False):
    """mine: (type, bytes, pads, ptrs); theirs: dump record [type, hex, rec addr, data addr]. The dump caps a record
    at 512 bytes when it cannot see where the next one starts: then only that prefix is compared. `loose_zero`:
    an 8-byte field that is ±0 on both sides matches (states loaded from JSON lost the sign of -0.0)."""
    t, b, pads, ptrs = mine
    if t != theirs[0]:
        return f"type {t} vs {theirs[0]}"
    g = bytes.fromhex(theirs[1])
    if len(g) < len(b) and len(g) != 512:
        return f"length {len(b)} vs {len(g)}"
    data = int(theirs[3], 16)
    skip = set()
    for a, e in pads:
        skip.update(range(a, e))
    for o in ptrs:
        rel = struct.unpack_from("<Q", g, o)[0] - data
        if rel != struct.unpack_from("<Q", b, o)[0]:
            return f"pointer at +{o:#x}"
        skip.update(range(o, o + 8))
    bad = [i for i in range(min(len(b), len(g))) if i not in skip and b[i] != g[i]]
    if loose_zero:
        zero = lambda x, o: o + 8 <= len(x) and struct.unpack_from("<d", x, o)[0] == 0.0
        bad = [i for i in bad if not (zero(b, i & ~7) and zero(g, i & ~7))]
    return f"bytes differ at {bad[:8]}" if bad else None


def compare(got, want, enter_recs, show=12):
    out = []

    def ck(name, a, b):
        if a != b and not (isinstance(a, float) and isinstance(b, float) and a != a and b != b):
            out.append(f"{name}: port {a!r} gurobi {b!r}")

    for k in FIELDS:
        a, b = got[k], want[k]
        if k == "nnz":
            b = int(b)
        if isinstance(b, list):
            for i, (x, y) in enumerate(zip(a, b)):
                # dead rows' activities are stale memory: with --after it comes from another process
                if k.startswith("act_") and got["row_cnt"][i] < 0 and want["row_cnt"][i] < 0:
                    continue
                ck(f"{k}[{i}]", x, y)
        else:
            ck(k, a, b)
    ck("sense", bytes(got["sense"]).decode(), want["sense"])
    ck("vtype", bytes(got["vtype"]).decode(), want["vtype"])
    ck("counters", list(got["counters"][:14]), want["counters"][:14])
    ck("free/pending", [got["nfree"], got["npend"]], [int(x) for x in want["free"]])
    for kind in ("rows", "cols"):
        for i, (x, y) in enumerate(zip(got[kind], want[kind])):
            ck(f"{kind}[{i}]", [list(e) for e in x], y)
    t = want["tracker"]
    for k in "rc":
        q = t[k]
        ck(f"tracker {k} flag", list(got[f"{k}_flag"]), q["flag"])
        ck(f"tracker {k} a/b", (list(got[f"{k}_a"]), list(got[f"{k}_b"])), (q["a"], q["b"]))
        for lv in range(3):
            ck(f"tracker {k} list{lv}", list(got[f"{k}_list"][lv][:q["b"][lv]]), (q["list"][lv] or [])[:q["b"][lv]])
    new = want["recs"][len(enter_recs):]
    ck("records", len(got["recs"]), len(new))
    for i, (m, g) in enumerate(zip(got["recs"], new)):
        e = rec_diffs(m, g, loose_zero=True)
        if e:
            out.append(f"record {i}: {e}")
    return out[:show], len(out)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("fn")
    ap.add_argument("dumps", nargs="+")
    ap.add_argument("--after")
    ap.add_argument("--param", action="append", default=[])
    ap.add_argument("-v", action="store_true")
    a = ap.parse_args()
    params = [(k, float(v)) for k, v in (p.split("=") for p in a.param)]
    after = {}
    if a.after:
        for x in json.load(open(a.after)):
            if "ret" not in x:
                after[x["call"]] = x["s"]
    tot = ok = 0
    for path in a.dumps:
        by = {}
        for x in json.load(open(path)):
            by.setdefault(x["call"], []).append(x)
        for c, xs in sorted(by.items()):
            if len(xs) < 2:
                continue
            enter, leave = xs[0]["s"], xs[1]["s"]
            if a.after:
                if c not in after:
                    continue
                enter, leave = leave, after[c]
            names = ["36e6b6:0", "36e6b6:1"] if a.fn == "36e6b6" else [a.fn]
            if a.fn == "3ff744":  # mode and flag are the 4th and 5th arguments
                names = [f"3ff744:{xs[0]['args'][3]}:{xs[0]['args'][4]}"]
            if a.fn in ("838543", "423e35"):  # k / probe mode is the 4th argument
                names = [f"{a.fn}:{xs[0]['args'][3]}"]
            best = None
            for name in names:
                got = _core.mip_dump_run(name, to_parts(enter), params)
                diffs, nd = compare(got, leave, enter["recs"] if not a.after else enter["recs"])
                if not a.after and a.fn != "36e6b6" and got["ret"] != xs[1]["ret"] and a.fn not in NO_RET:
                    diffs.append(f"ret port {got['ret']} gurobi {xs[1]['ret']}")
                    nd += 1
                if xs[1].get("out") is not None and got["extra"] != xs[1]["out"]:  # OUT_I32 dumps (5577ba f4)
                    diffs.append(f"out port {got['extra']} gurobi {xs[1]['out']}")
                    nd += 1
                if best is None or nd < best[1]:
                    best = (diffs, nd, name)
            tot += 1
            if best[1] == 0:
                ok += 1
                if a.v:
                    print(f"{path} call {c}: ok ({best[2]})")
            else:
                print(f"{path} call {c}: {best[1]} differences")
                for line in best[0]:
                    print("   ", line)
    print(f"{a.fn}: {ok}/{tot} calls match")
    sys.exit(0 if ok == tot else 1)


if __name__ == "__main__":
    main()
