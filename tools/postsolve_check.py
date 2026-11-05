"""Compare rsolver's uncrush (postsolve.rs) with an uncrush oracle capture (tools/uncrush_oracle.py), bit for bit.

usage: PYTHONPATH=python python tools/postsolve_check.py <oracle_dir> [--param Name=Value ...] [--oracle-align] [-v] [name ...]
The input of the uncrush is the oracle's `pre_after` (what 363daa reads), so simplex plays no part. Models for which
the oracle never reached 363daa (nothing presolved) are skipped. --oracle-align takes the value-array parities from
the oracle's real record addresses instead of the arena simulation. Needs no Gurobi.
"""
import argparse, json, struct, sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from parity import instances  # noqa: E402
import rsolver  # noqa: E402

H = lambda v: [float.fromhex(s) for s in v]
# byte offsets (in the record's data block) of the value-array pointers per record type
VAL_PTRS = {1: (0x28,), 2: (0x30, 0x48), 3: (0x30,), 21: (0x68,)}


def oracle_aligns(recs):
    out = []
    for r in reversed(recs):  # oracle lists newest first
        raw, a = bytes.fromhex(r["raw"]), [0, 0]
        for k, off in enumerate(VAL_PTRS.get(r["type"], ())):
            a[k] = struct.unpack_from("<Q", raw, off)[0] & 0xF
        out += a
    return out


def _n2(r):  # does the type-2 record have a column part?
    return struct.unpack_from('<i', bytes.fromhex(r['raw']), 0x38)[0] != 0 and struct.unpack_from('<Q', bytes.fromhex(r['raw']), 0x48)[0] > 0x1000


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("oracle")
    ap.add_argument("--param", action="append", default=[])
    ap.add_argument("--oracle-align", action="store_true")
    ap.add_argument("--align", action="store_true", help="first parity mismatch per model")
    ap.add_argument("--arena", action="store_true", help="list unexplained arena gaps instead of comparing")
    ap.add_argument("-v", action="store_true")
    ap.add_argument("names", nargs="*")
    a = ap.parse_args()
    params = [(k, float(v)) for k, v in (p.split("=", 1) for p in a.param)]
    insts = instances()
    names = a.names or sorted(p.stem for p in Path(a.oracle).glob("*.json"))
    ok = skipped = 0
    for name in names:
        cap = json.load(open(Path(a.oracle) / f"{name}.json"))
        if not cap["daa"]:
            skipped += 1
            continue
        e = cap["daa"][-1]
        core = rsolver.read(str(insts[name]))._core
        pr = core.post_records(params)
        kinds = [k for k, _, _, _ in pr]
        want_kinds = [r["type"] for r in reversed(e["records"])]
        if kinds != want_kinds:
            print(f"{name:12s} records differ: got {len(kinds)} want {len(want_kinds)}"
                  f" first diff at {next((i for i, (x, y) in enumerate(zip(kinds, want_kinds)) if x != y), min(len(kinds), len(want_kinds)))}")
            continue
        if a.align:  # first record whose simulated array parity differs from the real one (arrays used by a dot product)
            real = oracle_aligns(e["records"])
            for i in range(len(pr)):
                if kinds[i] not in VAL_PTRS:
                    continue
                r = e["records"][len(pr) - 1 - i]
                raw = bytes.fromhex(r["raw"])
                bad = []
                for q, off in enumerate(VAL_PTRS[kinds[i]]):
                    ptr = struct.unpack_from("<Q", raw, off)[0]
                    if ptr > 0x10000 and ptr % 8 == 0 and pr[i][1 + q] != ptr & 8:  # garbage pointers (unused array) are skipped
                        bad.append(q)
                if bad:
                    print(f"{name:12s} first parity mismatch at record {i} kind {kinds[i]} arrays {bad} sim {pr[i][1:3]}; context kinds {kinds[max(0,i-3):i+2]}")
                    break
            continue
        if a.arena:  # gaps between consecutive records that the push sizes do not explain
            rs = list(reversed(e["records"]))
            for i in range(len(rs) - 1):
                g = rs[i + 1]["rec"] - rs[i]["rec"] - pr[i][3]
                if g != 0 and 0 < rs[i + 1]["rec"] - rs[i]["rec"] < 65536:
                    print(f"{name:12s} rec {i} type {kinds[i]} gap {g} before type {kinds[i + 1]}")
            continue
        pa = e["pre_after"]
        try:
            basis, x, dual = core.uncrush(params, H(pa["x"]), H(pa["dj"]), pa["st"],
                                          oracle_aligns(e["records"]) if a.oracle_align else None)
        except BaseException as ex:
            print(f"{name:12s} PANIC {ex}")
            continue
        o = e["out"]
        bad = []
        if basis != o["basis"]:
            bad.append("basis@" + str([i for i, (u, v) in enumerate(zip(basis, o["basis"])) if u != v][:6]))
        for tag, got, want in (("x", x, o["x"]), ("dual", dual, o["dual"])):
            idx = [i for i, (u, v) in enumerate(zip(got, want)) if u.hex() != v]
            if idx:
                bad.append(f"{tag}@{idx[:6]}")
        if bad:
            print(f"{name:12s} " + " ".join(bad))
        else:
            ok += 1
    print(f"match {ok}/{len(names) - skipped}" + (f" ({skipped} without uncrush)" if skipped else ""))


if __name__ == "__main__":
    main()
