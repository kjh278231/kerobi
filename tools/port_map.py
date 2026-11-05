"""Assign every decompiled Gurobi core function to an rsolver design module and report port coverage.

Ownership (rules in docs/10_porting.md §2): functions named in re/notes/1x go to that note's module (most mentions wins;
a note's own table row counts 10); module entry points and a few shared helpers are pinned in OVERRIDE; an undocumented
named helper called from 3+ modules becomes `common`. Callees inherit the owner when only one module reaches them,
else `common`. The rest are `01_model_api` (reached from a kept GRB* API), `excluded` (license/WLS/batch/tune/server),
`extern` (MKL, OpenSSL), or `unreached`.
Coverage: a function counts as ported when some .rs file under crates/ carries `port: <short>@0x<off>`.
usage: python3 -I tools/port_map.py <re_dir> <rsolver_dir> [libgurobi130.so]   -> writes docs/port_map.tsv, prints the summary
       (with the .so, function pointers in data tables also count as edges; needs objdump and readelf)
"""
import csv, glob, os, re, sys
from collections import Counter, defaultdict, deque

re_dir, rs_dir = sys.argv[1], sys.argv[2]
NOTE_MODULE = {"10": "02_driver", "11": "02_driver", "12": "03_presolve", "13": "04_simplex",
               "14": "05_barrier_pdhg_concurrent", "15": "06_bnb", "16": "07_cuts_heur", "17": "08_genconstr_nl"}
EXCL_API = re.compile(r"batch|tune|license|token|wls|pingserver|hostid|Container|SystemID|gpu|replay|record|IIS|"
                      r"loadenv|emptyenv|startenv|getusername|gethostname|getcommstats|getdistro", re.I)
EXCL_STR = re.compile(r"licen[cs]e|\bWLS\b|token server|Compute Server|Cluster Manager|Instant Cloud|Tuning", re.I)
rd = lambda p: csv.DictReader(open(p), delimiter="\t", quoting=csv.QUOTE_NONE)

fn, by_short = {}, defaultdict(list)
for r in rd(f"{re_dir}/decomp/index.tsv"):
    off = int(r["off"], 16)
    m = re.fullmatch(r"PRIVATE0*([0-9a-f]+)", r["name"])
    short = m.group(1)[-6:].lstrip("0") if m else r["name"]
    fn[off] = {"name": r["name"], "short": short, "size": int(r["size"]), "status": r["status"]}
    by_short[short].append(off)
callees = defaultdict(set)
for r in rd(f"{re_dir}/decomp/callgraph.tsv"):
    a, b = int(r["caller_off"], 16), int(r["callee_off"], 16)
    if a in fn and b in fn and a != b:
        callees[a].add(b)
by_name = defaultdict(list)
for off, f in fn.items():
    by_name[f["name"]].append(off)
for f in glob.glob(f"{re_dir}/decomp/c/*.c"):
    off = int(os.path.basename(f)[:8], 16)
    if off in fn:
        src = open(f, errors="replace").read()
        fn[off]["lines"] = src.count("\n")
        # address-taken functions (callbacks, thread entries, dispatch tables) are edges the call graph misses;
        # shortcut: a duplicated name links to every function sharing it, fine for module ownership only.
        for name in set(re.findall(r"\b(PRIVATE[0-9a-f]{16}|GRB\w+)\b(?!\s*\()", src)) - {fn[off]["name"]}:
            callees[off].update(by_name.get(name, ()))
if len(sys.argv) > 3:  # function pointers stored in data tables (attribute/param/reader tables): edge from the code using the table
    import bisect, subprocess
    lib, lo = sys.argv[3], min(fn)
    hi = max(o + f["size"] for o, f in fn.items())
    asm = subprocess.run(["objdump", "-d", "--no-show-raw-insn", f"--start-address={lo:#x}", f"--stop-address={hi:#x}", lib],
                         capture_output=True, text=True, check=True).stdout
    refs, cur = set(), None
    for line in asm.splitlines():
        if line.endswith(">:"):
            cur = int(line.split()[0], 16)
        elif (m := re.search(r"# ([0-9a-f]+) <", line)) and cur in fn and int(m.group(1), 16) > hi:
            refs.add((int(m.group(1), 16), cur))
    refs = sorted(refs)
    rel = subprocess.run(["readelf", "-rW", lib], capture_output=True, text=True, check=True).stdout
    for slot, tgt in re.findall(r"^([0-9a-f]+)\s+\S+\s+R_X86_64_RELATIVE\s+([0-9a-f]+)", rel, re.M):
        slot, tgt = int(slot, 16), int(tgt, 16)
        i = bisect.bisect_right(refs, (slot, float("inf"))) - 1
        # shortcut: nearest preceding referenced address within 12KB is taken as the table base (99% of slots lie within 10.4KB).
        if tgt in fn and i >= 0 and slot - refs[i][0] <= 0x3000:
            callees[refs[i][1]].add(tgt)

# 1. seeds from note mentions: `abc123` or `abc123@0xoff`
votes, documented = defaultdict(Counter), set()
for path in glob.glob(f"{re_dir}/notes/1[0-7]_*.md"):
    mod = NOTE_MODULE[os.path.basename(path)[:2]]
    for line in open(path):
        weight = 10 if re.match(r"\|\s*`[0-9a-f]{5,6}", line) else 1  # the note whose table row documents the function
        for m in re.finditer(r"(?<![0-9a-fA-Fx_@+])([0-9a-f]{5,6})(?:@0x([0-9a-f]+))?(?![0-9a-fA-F])", line):
            if not re.search(r"[a-f]", m.group(1)) and len(m.group(1)) < 6 and not m.group(2):
                continue  # same rule as re/scripts/verify_notes.py: short decimal-looking tokens are not names
            for off in ([int(m.group(2), 16)] if m.group(2) else by_short.get(m.group(1).lstrip("0"), [])):
                if off in fn:
                    votes[off][mod] += weight
                    if weight > 1 and m.start() < 12:
                        documented.add(off)
owner = {off: c.most_common(1)[0][0] for off, c in votes.items()}
# module entry points (docs/00_overview.md §4) and cross-cutting helpers: RNG, limits check, param registry,
# node heuristic dispatcher, product branching score
OVERRIDE = {"9828a1": "02_driver", "c23877": "02_driver",
            "390989": "03_presolve", "3cd406": "03_presolve", "363164": "03_presolve", "365159": "03_presolve",
            "cda5b": "04_simplex", "6268b": "04_simplex", "478d3": "04_simplex", "36ade": "04_simplex", "5b257": "04_simplex",
            "2bb770": "05_barrier_pdhg_concurrent", "2bf1ea": "05_barrier_pdhg_concurrent", "13f716": "05_barrier_pdhg_concurrent",
            "34c004": "05_barrier_pdhg_concurrent", "172cc6": "05_barrier_pdhg_concurrent",
            "5bbf7f": "06_bnb", "5c239f": "06_bnb", "6e8eb3": "06_bnb", "5fb8ec": "06_bnb", "62a05d": "06_bnb",
            "64da9f": "07_cuts_heur", "7b77cd": "07_cuts_heur", "6d8986": "07_cuts_heur", "6de7ca": "07_cuts_heur",
            "9ee0cf": "08_genconstr_nl", "5b1161": "08_genconstr_nl", "897f79": "08_genconstr_nl", "7aef0b": "08_genconstr_nl",
            "a7b4d6": "common", "a83b2f": "common", "ac81d7": "01_model_api", "8ad3a3": "07_cuts_heur", "646052": "06_bnb"}
for short, mod in OVERRIDE.items():
    assert by_short[short], short
    for off in by_short[short]:
        owner[off] = mod
callers = defaultdict(set)
for a, bs in callees.items():
    for b in bs:
        callers[b].add(a)
pinned = {o for short in OVERRIDE for o in by_short[short]}
for off in [o for o in owner if o not in pinned | documented]:  # undocumented helper used by 3+ modules (limits check) is shared
    if len({owner[c] for c in callers[off] if c in owner}) >= 3:
        owner[off] = "common"
seed_of = {off: "note" for off in owner}

# the env-free warning "free is deferred (Continue to use WLS)" sits in multiobj/concurrent env code, not license code
excl_str = {int(r["func_off"], 16) for r in rd(f"{re_dir}/decomp/strings_xref.tsv")
            if EXCL_STR.search(r["string"]) and "free is deferred" not in r["string"]}
for off, f in fn.items():
    if off not in owner and (off in excl_str or (f["name"].startswith("GRB") and EXCL_API.search(f["name"]))):
        owner[off], seed_of[off] = "excluded", "string/api"

# 2. inherit down the call graph without crossing another owned function
reached = defaultdict(set)
for off, mod in owner.items():
    if mod == "excluded":
        continue
    q = deque([off])
    while q:
        u = q.popleft()
        for v in callees[u]:
            if v not in owner and mod not in reached[v]:
                reached[v].add(mod)
                q.append(v)
for off, mods in reached.items():
    owner[off], seed_of[off] = (next(iter(mods)), "callee") if len(mods) == 1 else ("common", "callee:" + "+".join(sorted(mods)))

# 3. the rest: model/API surface, third-party, unreachable
def closure(starts):
    seen, q = set(starts), deque(starts)
    while q:
        for v in callees[q.popleft()]:
            if v not in seen:
                seen.add(v); q.append(v)
    return seen
kept_api = closure([o for o, f in fn.items() if f["name"].startswith("GRB") and owner.get(o) != "excluded"])
opt = closure([o for o, f in fn.items() if f["name"] == "GRBoptimize"])
for off, f in fn.items():
    if off not in owner:
        third = not f["name"].startswith(("PRIVATE", "GRB"))
        owner[off] = "extern" if third else "01_model_api" if off in kept_api else "unreached"
        seed_of[off] = "rest"

assert len(owner) == len(fn)
for short, mod in {"c23877": "02_driver", "3cd406": "03_presolve", "cda5b": "04_simplex", "34c004": "05_barrier_pdhg_concurrent",
                   "6e8eb3": "06_bnb", "7b77cd": "07_cuts_heur", "9ee0cf": "08_genconstr_nl"}.items():
    assert {owner[o] for o in by_short[short]} == {mod}, (short, mod)  # self-check: known entry points keep their module

ported = set()
for p in glob.glob(f"{rs_dir}/crates/**/*.rs", recursive=True):
    ported |= {int(o, 16) for o in re.findall(r"port: [0-9a-zA-Z_]+@0x([0-9a-f]+)", open(p).read())}

with open(f"{rs_dir}/docs/port_map.tsv", "w") as out:
    out.write("off\tshort\tsize\tlines\tmodule\tbasis\ton_optimize_path\tported\n")
    for off in sorted(fn):
        f = fn[off]
        out.write(f"{off:#x}\t{f['short']}\t{f['size']}\t{f.get('lines', 0)}\t{owner[off]}\t{seed_of[off]}\t"
                  f"{int(off in opt)}\t{int(off in ported)}\n")

rows = defaultdict(lambda: [0, 0, 0, 0, 0, 0])
for off, f in fn.items():
    r = rows[owner[off]]
    r[0] += 1; r[1] += f["size"]; r[2] += f.get("lines", 0); r[3] += seed_of[off] == "note"; r[4] += off in opt; r[5] += off in ported
print("| module | functions | named in notes | on optimize path | code KB | pseudocode lines | ported |")
print("|---|---:|---:|---:|---:|---:|---:|")
for mod in sorted(rows):
    n, sz, ln, seeds, onopt, done = rows[mod]
    print(f"| {mod} | {n:,} | {seeds:,} | {onopt:,} | {sz // 1024:,} | {ln:,} | {done:,} |")
