"""M1-02a-1 check (docs/m1/m1_02a1_tau.md section 3), retargeted by M1-02d (docs/m1/m1_02d_tau.md section 3): the DSE
tau = B^-1 rho of the original 09d112 vs rsolver on kb2, and rsolver's row-wise U order (snapshot urow/t) vs the capture.
Reads the three captures of tools/m1_tau_path.js, the M1-02a beta capture, the stored spx_oracle file, the pinned
Rust LU dump (bench/oracle/m1_02a1_tau/rust_lu_kb2.jsonl.gz, made with rust_lu_dump.patch) and a fresh RSOLVER_SNAP
run of the rsolver on PYTHONPATH (build it first with tools/build_local.sh). Exit 1 on any failure.
usage: PYTHONPATH=$PWD/python ../.venv/bin/python tools/m1_tau_check.py [--self-test]"""
import copy, gzip, hashlib, json, struct, sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from m1_beta_check import BIN, MODEL_SHA, PARAMS, fh, load_oracle, rust_snapshot, u2f  # noqa: E402

ROOT = Path(__file__).resolve().parents[1]
DIR = ROOT / "bench/oracle/m1_02a1_tau"
BETA = ROOT / "bench/oracle/m1_02a_beta/kb2_1.json.gz"  # pinned by tools/m1_beta_check.py
TOOL = ROOT / "tools/m1_tau_path.js"
GZ_SHA = "64cc9dea89f809b601f7cb2e11636cc632f382318ceeaec0273a599687bdf393"  # each of kb2_{1,2,3}.json.gz
DUMP_SHA = "e7ba87485cacadd05574a8280398bb3f221531e7dc801d9ce32a02123e7cda30"  # rust_lu_kb2.jsonl.gz
PATCH_SHA = "c698cc8d0ccf247da8da40417f76ed59df8d6aafb37a241837e69d2b79680978"  # rust_lu_dump.patch
ZERO = 1e-13  # crates/lp/src/lu.rs ZERO = LU+0x10 drop tolerance
NP = 22  # pivots 0..21: same input in both (the pivot path splits at 21)
FIRST = (11, [20, 22])  # first pivot where the pre-M1-02d Rust rule (upper_rust) differs from the original tau
NO = 30  # last snapshot of the stored oracle (iter0-30)

bits = lambda x: struct.pack("<d", x)
same = lambda a, b: bits(a) == bits(b)


def dot8(v):
    """crates/lp/src/lu.rs dot8: in sequence below 8 terms, else 8 lanes ((0+2)+(4+6))+((1+3)+(5+7)), tail in order."""
    if len(v) < 8:
        a = 0.0
        for t in v:
            a += t
        return a
    n8 = len(v) // 8 * 8
    a = [0.0] * 8
    for c in range(0, n8, 8):
        for q in range(8):
            a[q] += v[c + q]
    w = ((a[0] + a[2]) + (a[4] + a[6])) + ((a[1] + a[3]) + (a[5] + a[7]))
    for t in v[n8:]:
        w += t
    return w


def lower(lu, x):
    """Rust ftran_lower: L columns in factor order (skip |x_r| <= ZERO), then row etas x_r = eta.x (dot8, drop)."""
    x = list(x)
    for k, (r, _, _) in enumerate(lu["steps"]):
        xr = x[r]
        if abs(xr) > ZERO:
            for i, l in lu["lcol"][k]:
                x[i] -= l * xr
    for r, eta in lu["retas"]:
        v = dot8([x[i] * e for i, e in eta])
        x[r] = 0.0 if abs(v) <= ZERO else v
    return x


def upper_rust(lu, x):
    """Rust ftran_upper: U columns in descending U order, scatter into every row; by position."""
    x = list(x)
    for k in reversed(lu["order"]):
        r, _, d = lu["steps"][k]
        if abs(x[r]) <= ZERO or abs(x[r] / d) <= ZERO:
            x[r] = 0.0
            continue
        x[r] /= d
        for i, u in lu["ucol"][k]:
            x[i] -= u * x[r]
    out = [0.0] * len(x)
    for r, p, _ in lu["steps"]:
        out[p] = x[r]
    return out


def tau_original(lu, x, ft, need):
    """09d112 dense two-RHS tau (C:2086-2402): tail U columns (U position >= t) divide and scatter into tail rows only;
    each alpha_q position in the prefix with a U row (LU+0xf8 >= 0) gathers (x_i - sum u*tau_j)/d_i over the U row in
    its stored order (LU+0xb0/0xb8/0xd0/0xc8), LU+0xf8 = -1 divides, -2 (tail) keeps the column-solved value."""
    steps, order, t = lu["steps"], lu["order"], ft["t"]
    m = len(steps)
    upos = [0] * m
    for p, k in enumerate(order):
        upos[k] = p
    step_of_row = {r: k for k, (r, _, _) in enumerate(steps)}
    x = list(x)
    for p in range(m - 1, t - 1, -1):
        k = order[p]
        r, _, d = steps[k]
        if abs(x[r]) <= ZERO or abs(x[r] / d) <= ZERO:
            x[r] = 0.0
            continue
        x[r] /= d
        for i, u in lu["ucol"][k]:
            if upos[step_of_row[i]] >= t:
                x[i] -= u * x[r]
    out, long_rows = {}, 0
    for p in need:
        r, _, d = steps[p]
        f = ft["f8"][p]
        if f == -2:
            out[p] = x[r]
        elif f == -1:
            out[p] = x[r] / d
        else:
            row = ft["urow"][p]
            terms = [u2f(v) * x[steps[c][0]] for c, v in zip(row["idx"], row["val"])]
            if len(terms) >= 8:
                long_rows += 1  # 8-lane gather (C:2343-2387) is not in the capture: C only
            v = x[r]
            for tt in terms:
                v -= tt
            out[p] = v / d
    return out, long_rows


def final_list(e):
    """8b473 C:466-482 on a sparse alpha_q output: the pivot position's element is overwritten by the last (tau too)."""
    ft = e["ft"]
    idx, aq = list(ft["aq_out"]["idx"]), list(ft["aq_out"]["val"])
    tau = list(ft["tau_out"]["val"][:len(idx)])
    if e["r"] in idx:
        j = idx.index(e["r"])
        idx[j], aq[j], tau[j] = idx[-1], aq[-1], tau[-1]
        idx.pop(), aq.pop(), tau.pop()
    return idx, aq, tau


def load_dump():
    out = []
    for line in gzip.open(DIR / "rust_lu_kb2.jsonl.gz", "rt"):
        d = json.loads(line)
        lu = d["lu"]
        lu["steps"] = [(r, p, u2f(v)) for r, p, v in lu["steps"]]
        lu["lcol"] = [[(i, u2f(v)) for i, v in c] for c in lu["lcol"]]
        lu["ucol"] = [[(i, u2f(v)) for i, v in c] for c in lu["ucol"]]
        lu["retas"] = [(r, [(i, u2f(v)) for i, v in e]) for r, e in lu["retas"]]
        d["rho"] = [u2f(v) for v in d["rho"]]
        out.append(d)
    return out


def load_runs():
    runs = []
    for r in (1, 2, 3):
        p = DIR / f"kb2_{r}.json.gz"
        d = json.load(gzip.open(p))
        d["_gz_sha256"] = hashlib.sha256(p.read_bytes()).hexdigest()
        runs.append((p, d))
    return runs


def check(runs, beta, oracle, dump, mine, dump_sha=None):
    bad, rep = [], []
    # M
    for p, d in runs:
        if d.get("binary_sha256") != BIN:
            bad.append(f"M {p.name}: binary sha")
        if d.get("model_sha256") != MODEL_SHA:
            bad.append(f"M {p.name}: model sha")
        if d.get("params") != PARAMS:
            bad.append(f"M {p.name}: params {d.get('params')}")
        if not d.get("ok") or d.get("output") != "DONE 2 38" or d.get("errors"):
            bad.append(f"M {p.name}: run not ok ({d.get('output')!r})")
        if not d.get("events"):
            bad.append(f"M {p.name}: no events")
        if d.get("_gz_sha256") != GZ_SHA:
            bad.append(f"M {p.name}: file sha {d.get('_gz_sha256')} != pinned")
    tools = {d.get("tool_sha256") for _, d in runs}
    if tools != {hashlib.sha256(TOOL.read_bytes()).hexdigest()}:
        bad.append(f"M tool sha {tools} != current {TOOL.name}")
    dsha = dump_sha or hashlib.sha256((DIR / "rust_lu_kb2.jsonl.gz").read_bytes()).hexdigest()
    if dsha != DUMP_SHA:
        bad.append(f"M Rust LU dump sha {dsha} != pinned")
    if hashlib.sha256((DIR / "rust_lu_dump.patch").read_bytes()).hexdigest() != PATCH_SHA:
        bad.append("M Rust LU dump patch sha != pinned")
    rep.append(f"M: 3 runs, binary/model/params/output, pinned file sha, tool = current file {str(next(iter(tools)))[:16]}, Rust LU dump and patch pinned")
    ev = runs[0][1]["events"]
    if not ev:
        return bad + ["no events"], rep
    # B
    for p, d in runs[1:]:
        if d["events"] != ev:
            k = next((i for i, (a, b) in enumerate(zip(ev, d["events"])) if a != b), min(len(ev), len(d["events"])))
            bad.append(f"B {p.name}: event {k} differs from run 1")
    rep.append(f"B: {len(ev)} pivots, 3 runs equal")
    # X
    bev = beta["events"]
    if len(bev) != len(ev):
        bad.append(f"X: {len(ev)} pivots, M1-02a capture has {len(bev)}")
    for e, b in zip(ev, bev):
        k, ft = e["k"], e.get("ft")
        if e.get("n9d") != 1 or not ft:
            bad.append(f"X pivot {k}: {e.get('n9d')} 09d112 calls")
            continue
        ri = ft["rho_in"]
        if (ri["nnz"], ri["idx"], ri["val"]) != (b["nnz"], b["idx"], b["val"]):
            bad.append(f"X pivot {k}: 09d112 rho input != M1-02a compacted rho (nnz {ri['nnz']} vs {b['nnz']})")
        if e["beta"] != b["beta"]:
            bad.append(f"X pivot {k}: beta {e['beta']} != M1-02a {b['beta']}")
        s = oracle.get(k + 1)
        if s is not None and (not same(fh(s["ar"]), u2f(e["ar"])) or not same(fh(s["beta"]), u2f(e["beta"]))):
            bad.append(f"X pivot {k}: alpha_r/beta != stored oracle snapshot {k + 1}")
    rep.append("X: one 09d112 call per pivot; its rho input = M1-02a compacted rho, beta = M1-02a, alpha_r/beta = stored oracle")
    # A
    dense_aq = 0
    finals = {}
    for e in ev:
        if not e.get("ft"):
            continue
        if e["ft"]["aq_out"]["nnz"] < 0:
            dense_aq += 1
            bad.append(f"A pivot {e['k']}: dense alpha_q output (compaction branch not in this contract)")
            continue
        idx, aq, tau = final_list(e)
        finals[e["k"]] = (idx, aq, tau)
        if (e["aq"]["nnz"], e["aq"]["idx"], e["aq"]["val"]) != (len(idx), idx, aq):
            bad.append(f"A pivot {e['k']}: 8b473 alpha_q list != 09d112 output with the pivot swapped out")
        if e["tau"] != tau:
            bad.append(f"A pivot {e['k']}: 8b473 tau values != 09d112 tau output with the same swap")
    rep.append(f"A: alpha_q list and aligned tau at the 8b473 return = 09d112 outputs with the pivot swapped out ({len(finals)} pivots)")
    # W: 1bae9c C:420-459 between pivot k and k+1 on the update path
    nw = nrf = 0
    for e, n in zip(ev, ev[1:]):
        if n["lu_upd"] == 0:
            nrf += 1
            continue
        if e["k"] not in finals:
            continue
        w = [u2f(h) for h in e["w"]]
        beta, ar0, r = u2f(e["beta"]), u2f(e["ar"]), e["r"]
        ar = ar0 if (ar0 < 0.0 or 1e-08 <= ar0) else 1e-08
        if ar0 < 0.0 and ar0 > -1e-08:
            ar = -1e-08
        w[r] = 1e-08 if beta == 0.0 else (beta / ar) / ar
        idx, aq, tau = (e["aq"]["idx"], e["aq"]["val"], e["tau"])
        for p, a, tv in zip(idx, aq, tau):
            t = u2f(a) / ar
            v = (beta * t * t - (t + t) * u2f(tv)) + w[p]
            w[p] = v if 1e-08 <= v else 1e-08
        got = [u2f(h) for h in n["w"]]
        diff = [p for p in range(len(w)) if not same(w[p], got[p])]
        nw += 1
        if diff:
            bad.append(f"W pivot {e['k']}: weights at the next 8b473 differ at positions {diff[:5]} (want {w[diff[0]].hex()} got {got[diff[0]].hex()})")
    if nw == 0:
        bad.append("W: no update-path pivot")
    rep.append(f"W: 1bae9c DSE update from the captured alpha_q, tau, beta, alpha_r = next weights on {nw} pivots ({nrf} refactor boundaries not checked)")
    # U, I, R, T, G on pivots 0..21
    nu = nrows = 0
    t_diff, z_diff, n_tau, n_gather2, long_rows = [], [], 0, 0, 0  # value differences / signed-zero-only differences
    old_rule = []  # value differences of the pre-M1-02d Rust rule (all-row scatter) from the original rule
    for k in range(NP):
        e, d = ev[k], dump[k]
        lu, ft = d["lu"], e.get("ft")
        if not ft or d["iter"] != k or d["r"] != e["r"]:
            bad.append(f"U pivot {k}: dump/capture mismatch (iter {d['iter']}, r {d['r']} vs {e['r']})")
            continue
        m = len(lu["steps"])
        if any(p != i for i, (_, p, _) in enumerate(lu["steps"])):
            bad.append(f"U pivot {k}: dump steps are not by position")
        if lu["order"] != ft["ord"] or ft["t"] != oracle[k]["lu"]["t"]:
            bad.append(f"U pivot {k}: U order or prefix t != original")
        upos = {s: p for p, s in enumerate(lu["order"])}
        for pos in range(m):
            r, _, dg = lu["steps"][pos]
            mine_row = sorted((c, bits(u)) for c in range(m) for i, u in lu["ucol"][c] if i == r)
            row = ft["urow"][pos]
            orig_row = sorted((c, bits(u2f(v))) for c, v in zip(row["idx"], row["val"]))
            nrows += 1
            if mine_row != orig_row:
                bad.append(f"U pivot {k} position {pos}: U row {mine_row[:3]} != original {orig_row[:3]}")
            if not same(dg, u2f(ft["diag"][pos])):
                bad.append(f"U pivot {k} position {pos}: diagonal != original")
            tail = upos[pos] >= ft["t"]
            f = ft["f8"][pos]
            if (f == -2) != tail or (f == -1) != (not tail and not row["idx"]) or (f >= 0) != (not tail and bool(row["idx"])):
                bad.append(f"U pivot {k} position {pos}: LU+0xf8 {f} vs tail {tail} row {len(row['idx'])}")
        nu += 1
        # I: rho and alpha_q
        ri = ft["rho_in"]
        want = [0.0] * m
        if ri["nnz"] < 0:
            want = [u2f(v) for v in ri["val"]]
        else:
            for i, v in zip(ri["idx"], ri["val"]):
                want[i] = u2f(v)
        if [bits(v) for v in want] != [bits(v) for v in d["rho"]]:
            bad.append(f"I pivot {k}: Rust rho != original 09d112 rho input")
        ai = ft["aq_in"]
        a = [0.0] * m
        for i, v in zip(ai["idx"] or [], ai["val"]):
            a[i] = u2f(v)
        col = upper_rust(lu, lower(lu, a))
        ao = dict(zip(ft["aq_out"]["idx"], ft["aq_out"]["val"]))
        if any(not same(col[p], u2f(ao[p])) if p in ao else col[p] != 0.0 for p in range(m)):
            bad.append(f"I pivot {k}: Rust FTRAN of the original a_q != original alpha_q output")
        # R: the dump with the original rule (captured U row order) reproduces the current Rust tau
        idx, _, tau = finals[k]
        x = lower(lu, d["rho"])
        rt = upper_rust(lu, x)
        og, lr = tau_original(lu, x, ft, idx)
        s = mine.get(k + 1)
        if s is None or any(not same(og[p], fh(s["tau"][p])) for p in idx):
            bad.append(f"R pivot {k}: original rule on the dump != Rust snapshot {k + 1} tau")
            continue
        # T: Rust tau vs original tau
        diff = [p for p, tv in zip(idx, tau) if not same(fh(s["tau"][p]), u2f(tv))]
        val = [p for p, tv in zip(idx, tau) if fh(s['tau'][p]) != u2f(tv)]
        if val:
            t_diff.append((k, val))
        if len(val) < len(diff):
            z_diff.append((k, [p for p in diff if p not in val]))
        # G: the original rule on the same dump
        long_rows += lr
        n_tau += len(idx)
        n_gather2 += sum(1 for p in idx if ft["f8"][p] >= 0 and len(ft["urow"][p]["idx"]) >= 2)
        gd = [p for p, tv in zip(idx, tau) if not same(og[p], u2f(tv))]
        if gd:
            bad.append(f"G pivot {k}: original rule != original tau at positions {gd} (first {og[gd[0]].hex()} vs {u2f(tau[idx.index(gd[0])]).hex()})")
        rd = [p for p in idx if rt[p] != og[p]]
        if rd:
            old_rule.append((k, rd))
    rep.append(f"U: Rust LU dump = original U rows (sets, values), diagonal, U order and prefix t on {nu} pivots ({nrows} rows); LU+0xf8 = -2 tail, -1 empty prefix row, >=0 prefix row")
    rep.append(f"I: Rust rho = original 09d112 rho input; Rust FTRAN of the original a_q = original alpha_q output (pivots 0-{NP - 1})")
    rep.append(f"R: original rule on the dump (captured U row order) = current Rust snapshot tau (pivots 0-{NP - 1}, final alpha_q positions)")
    if t_diff or z_diff:
        bad.append(f"T: Rust tau != original tau: values {t_diff[:3]}, signed zero only {z_diff[:3]}")
    if not old_rule or old_rule[0] != FIRST:
        bad.append(f"T: the pre-M1-02d rule's first difference {old_rule[:1]}, want {FIRST}")
    rep.append(f"T: Rust tau = original tau on all {n_tau} alpha_q positions of pivots 0-{NP - 1} (bits, signed zeros included); "
               f"the pre-M1-02d all-row scatter would differ at {sum(len(v) for _, v in old_rule)} positions, first {old_rule[:1]}")
    if n_gather2 == 0:
        bad.append("G: no gathered prefix row with 2+ terms")
    rep.append(f"G: original rule (tail scatter to tail rows, prefix gather over the stored U row order) = original tau on all {n_tau} positions; "
               f"{n_gather2} gathered rows with 2+ terms, {long_rows} with 8+ (C only)")
    # F: the weights entering the same-input pivots (snapshot NP is the output of pivot NP - 1, whose entering choice
    # differs: tools/proto/pathcmp.py kb2)
    wbad = []
    for j in range(NP):
        wd = [p for p in range(len(oracle[j]["w"])) if not same(fh(oracle[j]["w"][p]), fh(mine[j]["w"][p]))]
        if wd:
            wbad.append((j, wd[:5]))
    if wbad:
        bad.append(f"F: Rust w != stored oracle at snapshots {wbad[:3]}")
    rep.append(f"F: Rust snapshots 0-{NP - 1} w = stored oracle")
    # O: Rust row-wise U order and prefix t at the tau point = captured LU (pivots on the original's path)
    no = 0
    for e in ev:
        k, ft = e["k"], e.get("ft")
        # pivot k's LU at tau is the one after pivot k - 1: the same input while snapshots 0..k equal
        if k + 1 > NO or k + 1 not in mine or any(mine.get(j, {}).get("vstat") != oracle[j]["vstat"] for j in range(k + 1)):
            break
        s = mine[k + 1]
        if s.get("t") != ft["t"]:
            bad.append(f"O pivot {k}: Rust prefix t {s.get('t')} != original {ft['t']}")
        for pos, row in enumerate(ft["urow"]):
            got = s["urow"][pos]
            if [c for c, _ in got] != row["idx"] or any(not same(fh(h), u2f(v)) for (_, h), v in zip(got, row["val"])):
                bad.append(f"O pivot {k} position {pos}: Rust U row {[c for c, _ in got]} != original {row['idx']}")
                break
        no += 1
    if no < NP:
        bad.append(f"O: Rust path equals the original on {no} pivots, want at least {NP}")
    rep.append(f"O: Rust row-wise U (column order, value bits) and prefix t = capture on pivots 0-{no - 1} (same basis as the stored oracle through snapshot {no - 1})")
    # P (report only)
    rows = []
    for e in ev[:NP]:
        ft = e["ft"]
        rows.append(f"{e['k']}:{'T' if any(k == e['k'] for k, _ in t_diff) else '.'} rho{ft['rho_in']['nnz']} tau{ft['tau_out']['nnz']} "
                    f"eta{ft['lu']['x180']} {'+'.join(c.split('@')[0] for c in ft['calls']) or 'inline'}")
    rep.append("P: " + "; ".join(rows))
    return bad, rep


def self_test(runs, beta, oracle, dump, mine):
    def flip(h):
        return format(int(h, 16) ^ 1, "016x")

    def each(fn):
        t = copy.deepcopy(runs)
        for _, d in t:
            fn(d)
        return t

    tampers = {}
    t = copy.deepcopy(runs); t[0][1]["binary_sha256"] = "0" * 64; tampers["binary sha"] = (t, dump, mine, None)
    t = copy.deepcopy(runs); t[2][1]["events"][5]["r"] += 1; tampers["run 3 differs"] = (t, dump, mine, None)
    t = copy.deepcopy(runs); t[1][1]["_gz_sha256"] = "0" * 64; tampers["file sha"] = (t, dump, mine, None)
    tampers["tool sha (all runs)"] = (each(lambda d: d.update(tool_sha256="1" * 64)), dump, mine, None)
    tampers["dump sha"] = (runs, dump, mine, "0" * 64)
    tampers["empty events"] = (each(lambda d: d.update(events=[])), dump, mine, None)

    def tau_bit(d):
        e = d["events"][11]; j = e["aq"]["idx"].index(20); jo = e["ft"]["aq_out"]["idx"].index(20)
        e["tau"][j] = flip(e["tau"][j])  # both copies, so the A consistency check alone does not catch it
        e["ft"]["tau_out"]["val"][jo] = flip(e["ft"]["tau_out"]["val"][jo])
    tampers["pivot 11 original tau bit"] = (each(tau_bit), dump, mine, None)

    def aq_drop(d):
        e = d["events"][11]
        for v in (e["aq"], e["ft"]["aq_out"]):
            v["idx"] = v["idx"][1:]; v["val"] = v["val"][1:]; v["nnz"] -= 1
        e["tau"] = e["tau"][1:]; e["ft"]["tau_out"]["val"] = e["ft"]["tau_out"]["val"][1:]
    tampers["pivot 11 alpha_q drops one element"] = (each(aq_drop), dump, mine, None)

    def rho_bit(d):
        v = d["events"][11]["ft"]["rho_in"]["val"]; v[0] = flip(v[0])
    tampers["pivot 11 09d112 rho bit"] = (each(rho_bit), dump, mine, None)

    def w_bit(d):
        v = d["events"][12]["w"]; v[20] = flip(v[20])
    tampers["pivot 12 w bit"] = (each(w_bit), dump, mine, None)

    def urow_swap(d):
        row = d["events"][11]["ft"]["urow"][20]
        row["idx"].reverse(); row["val"].reverse()
    tampers["pivot 11 U row order swapped"] = (each(urow_swap), dump, mine, None)
    old = upper_rust(dump[11]["lu"], lower(dump[11]["lu"], dump[11]["rho"]))  # pre-M1-02d rule at pivot 11
    m2 = copy.deepcopy(mine)
    for p in (20, 22):
        m2[12]["tau"][p] = old[p].hex()
    tampers["Rust tau at pivot 11 replaced by the pre-M1-02d rule"] = (runs, dump, m2, None)

    def orig_is_old(d):
        e = d["events"][11]
        for p in (20, 22):
            v = format(struct.unpack("<Q", struct.pack("<d", old[p]))[0], "016x")
            e["tau"][e["aq"]["idx"].index(p)] = v
            e["ft"]["tau_out"]["val"][e["ft"]["aq_out"]["idx"].index(p)] = v
    tampers["original tau at pivot 11 replaced by the pre-M1-02d rule"] = (each(orig_is_old), dump, mine, None)
    m3 = copy.deepcopy(mine)
    p13 = final_list(runs[0][1]["events"][13])[0][0]
    m3[14]["tau"][p13] = (fh(m3[14]["tau"][p13]) * (1 + 2 ** -52)).hex()
    tampers["Rust tau value at pivot 13"] = (runs, dump, m3, None)
    m4 = copy.deepcopy(mine)
    row = next(r for r in m4[16]["urow"] if len(r) >= 3 and r[0][0] != r[1][0])
    row[0], row[1] = row[1], row[0]
    tampers["Rust U row order at pivot 15 (refactor)"] = (runs, dump, m4, None)
    m5 = copy.deepcopy(mine)
    m5[15]["t"] += 1
    tampers["Rust prefix t at pivot 14"] = (runs, dump, m5, None)
    # the Rust LU dump bridge (U, I, R): one bit of a dump L value, of a captured U value, of the original a_q input
    d2 = copy.deepcopy(dump)
    lc = d2[11]["lu"]["lcol"]
    k = next(k for k, c in enumerate(lc) if c)
    i0, v0 = lc[k][0]
    lc[k][0] = (i0, struct.unpack("<d", (struct.unpack("<Q", bits(v0))[0] ^ 1).to_bytes(8, "little"))[0])
    tampers["dump L value bit at pivot 11"] = (runs, d2, mine, DUMP_SHA)

    def urow_bit(d):
        row = next(r for r in d["events"][11]["ft"]["urow"] if r["val"])
        row["val"][0] = flip(row["val"][0])
    tampers["captured U value bit at pivot 11"] = (each(urow_bit), dump, mine, None)

    def aq_in_bit(d):
        v = d["events"][11]["ft"]["aq_in"]["val"]
        v[0] = flip(v[0])
    tampers["original a_q input bit at pivot 11"] = (each(aq_in_bit), dump, mine, None)
    out = []
    for name, (rr, dd, mm, ds) in tampers.items():
        bad, _ = check(rr, beta, oracle, dd, mm, ds)
        out.append((name, bool(bad), bad[:1]))
    ok = True
    for name, rejected, why in out:
        print(f"  tamper {name}: {'rejected' if rejected else 'NOT REJECTED'} {why[0] if why else ''}")
        ok &= rejected
    return ok


def main():
    runs = load_runs()
    beta = json.load(gzip.open(BETA))
    oracle = load_oracle()
    dump = load_dump()
    mine = rust_snapshot()
    bad, rep = check(runs, beta, oracle, dump, mine)
    print("\n".join(rep))
    for b in bad:
        print("FAIL", b)
    ok = not bad
    if "--self-test" in sys.argv:
        print("self-test:")
        ok &= self_test(runs, beta, oracle, dump, mine)
    print("PASS" if ok else "FAIL")
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
