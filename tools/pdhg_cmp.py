"""Compare the Rust PDHG state with Gurobi's, callback by callback (docs/m3/m3_pdhg.md §3).

usage: python tools/pdhg_cmp.py <oracle.jsonl[.gz]> <dump.jsonl> [--all]
  oracle: tools/pdhg_oracle.py <model> <oracle.jsonl> --vec          (Gurobi, needs the license)
  dump:   PDHGDUMP=<dump.jsonl> with the rsolver Method=6 solve       (no Gurobi)
Both are keyed by the callback's iteration count P[0]; dump records past the oracle's last one are ignored. Prints, per
callback, the P fields and vectors whose bits differ (first differing index for vectors), stopping at the first callback
with a difference unless --all. Exit code 1 on a difference or an oracle callback missing in the dump. No gurobipy.
"""
import argparse, gzip, json, struct, sys

NAMES = {0x38: "pobj", 0x40: "dobj", 0x48: "compl", 0x50: "rp_inf", 0x58: "rd_inf", 0x60: "rp2", 0x68: "rd2",
         0x70: "rp_rel", 0x78: "rd_rel", 0x80: "gap", 0xe0: "eta", 0xe8: "omega", 0x100: "integ", 0x108: "err",
         0x110: "g", 0x118: "g_prev", 0x120: "g0", 0x170: "bnorm", 0x178: "cnorm", 0x188: "eps", 0x190: "tol",
         0x28: "x", 0x30: "y", 0xb8: "xhat", 0xc0: "yhat", 0x128: "x0", 0x130: "y0"}
bits = lambda h: struct.unpack("<Q", struct.pack("<d", float.fromhex(h)))[0]
val = lambda b: struct.unpack("<d", struct.pack("<Q", b))[0]


def load(path):
    with (gzip.open(path, "rt") if path.endswith(".gz") else open(path)) as f:
        return {r["it"]: r for r in map(json.loads, f)}


def diff(o, r):
    out = []
    cnt = [bits(o["P"][k]) for k in range(4)]  # P[0..3] are int64; the oracle stored their bits as doubles
    if cnt != r["cnt"]:
        out.append(f"counters P[0..3] {cnt} vs {r['cnt']}")
    for off, b in r["d"].items():
        g = bits(o["P"][int(off, 16) // 8])
        if g != b:
            out.append(f"{NAMES[int(off, 16)]} (P+{off}) {val(g)!r} vs {val(b)!r}")
    for off, v in r["vec"].items():
        if off not in o.get("vec", {}):
            continue
        g = [bits(h) for h in o["vec"][off][1:1 + len(v)]]  # oracle vectors carry allocator slack after the length
        k = next((i for i, (a, b) in enumerate(zip(g, v)) if a != b), None)
        if k is not None:
            out.append(f"{NAMES[int(off, 16)]}[{k}] (P+{off}) {val(g[k])!r} vs {val(v[k])!r}, "
                       f"{sum(a != b for a, b in zip(g, v))}/{len(v)} differ")
    return out


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("oracle")
    ap.add_argument("dump")
    ap.add_argument("--all", action="store_true")
    a = ap.parse_args()
    o, r = load(a.oracle), load(a.dump)
    common = sorted(set(o) & set(r))
    bad = 0
    for it in common:
        d = diff(o[it], r[it])
        if d:
            bad += 1
            print(f"callback it={it}:\n  " + "\n  ".join(d))
            if not a.all:
                break
    only = sorted(set(o) - set(r))  # the oracle is often cut short (--iters), so extra dump records are fine
    print(f"{len(common)} common callbacks, {bad} with differences" + (f"; missing in dump: {only[:10]}" if only else ""))
    sys.exit(1 if bad or only else 0)


if __name__ == "__main__":
    main()
