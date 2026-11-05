"""Prototype of the barrier ordering (docs/m3_ordering.md): AA' pattern (303e2a), stage labels (309dd0),
approximate minimum degree (3047bb) and symbolic counts (302921), checked against bench/oracle/bar_order/*.json.gz
(tools/proto/bar_order_oracle.py).

usage: python3 -I tools/proto/bar_order.py <oracle_dir> [name ...]
"""
import gzip, json, math, os, sys


def aat(m, beg, ln, ind):
    """303e2a: pattern of A·Aᵀ without the diagonal, both triangles, rows ascending in each column."""
    adj = [set() for _ in range(m)]
    for j in range(len(ln)):
        rows = ind[beg[j]:beg[j] + ln[j]]
        for a in rows:
            for b in rows:
                if a != b:
                    adj[a].add(b)
    cp, ri = [0], []
    for i in range(m):
        ri += sorted(adj[i])
        cp.append(len(ri))
    return cp, ri


def dense_thresh(n, nnz):
    """309dd0 / 3047bb: max(100, min((int)(50·nnz/n), 10·sqrt(n)))."""
    d = float(int(nnz * 50.0 / n))
    s = math.sqrt(n) * 10.0
    if s <= d:
        d = s
    return max(100, int(d))


def stage_labels(n, m, cp):
    """309dd0 before any bisection: rows with degree > thresh get stage 2 (-2), dense columns stage 3 (-3)."""
    t = dense_thresh(n, cp[n])
    lab = [-2 if cp[i + 1] - cp[i] > t else -1 for i in range(min(m, n))]
    return lab + [-3] * (n - len(lab))


def amd(n, cp, ri, label):
    """3047bb with Seed = 0 (no random shuffle). label: -k = stage k. Returns positions (inverse permutation)."""
    label = list(label)
    adj = [ri[cp[i]:cp[i + 1]] for i in range(n)]   # list = elements (first elen) then variables
    ln = [len(a) for a in adj]
    elen = [0] * n
    deg = list(ln)                                     # degree before dense removal (sic)
    nv = [1] * n
    parent = list(range(n))
    mark = [-1] * n
    hsh = [0.0] * n
    maxstage = max([0] + [-x for x in label])
    t = dense_thresh(n, cp[n])
    ndense = 0
    for i in range(n):
        if ln[i] > t:
            ndense += 1
            label[i] = n - label[i]
            ln[i] = -1
    if ndense:
        for i in range(n):
            adj[i] = [j for j in adj[i][:max(ln[i], 0)] if label[j] < 0]
            ln[i] = len(adj[i])
    pos = 0
    for i in range(n):
        if label[i] == -1 and ln[i] == 0:
            label[i] = pos
            pos += 1
            ln[i] = -1
    while pos < n:
        stage = maxstage
        for i in range(n):
            if nv[i] > 0:
                if label[i] < 0 and -label[i] < stage:
                    stage = -label[i]
                elif label[i] > n and label[i] - n < stage:
                    stage = label[i] - n
        head = [-1] * n
        nxt = [-1] * n
        prv = [-1] * n
        mindeg = n

        def insert(i):
            nonlocal mindeg
            d = deg[i]
            h = head[d]
            if h != -1:
                prv[h] = i
            nxt[i] = h
            prv[i] = -1
            head[d] = i
            if d < mindeg:
                mindeg = d

        def remove(i):
            a, b = prv[i], nxt[i]
            if a == -1:
                head[deg[i]] = b
            else:
                nxt[a] = b
            if b != -1:
                prv[b] = a

        for i in range(n - 1, -1, -1):
            if label[i] == -stage and nv[i] > 0:
                insert(i)
        while mindeg < n:
            d = mindeg
            while d < n and head[d] == -1:
                d += 1
            if d >= n:
                break
            me = head[d]
            mindeg = d
            remove(me)
            label[me] = pos
            mark[me] = me
            Lme, degme = [], 0

            def take(i):
                nonlocal degme
                if mark[i] != me and nv[i] > 0 and label[i] < 0:
                    Lme.append(i)
                    mark[i] = me
                    degme += nv[i]

            for e in adj[me][:elen[me]]:
                for i in adj[e][:max(ln[e], 0)]:
                    take(i)
                elen[e] = -1
                ln[e] = -1
            for i in adj[me][elen[me]:ln[me]]:
                take(i)
            w = {}
            for i in Lme:
                for e in adj[i][:elen[i]]:
                    w[e] = nv[e]
            for i in Lme:
                for e in adj[i][:elen[i]]:
                    w[e] -= nv[i]
                    if w[e] == 0:
                        ln[e] = 0
            for i in Lme:
                if label[i] == -stage:
                    remove(i)
            for i in Lme:
                h = float(label[i] * n)
                dsum = 0
                keep = []
                for e in adj[i][:elen[i]]:
                    if ln[e] > 0:
                        dsum += w[e]
                        h += e
                        keep.append(e)
                keep.append(me)
                ne = len(keep)
                for j in adj[i][elen[i]:ln[i]]:
                    if mark[j] != me and nv[j] > 0:
                        dsum += nv[j]
                        keep.append(j)
                        h += j
                adj[i] = keep
                elen[i] = ne
                ln[i] = len(keep)
                deg[i] = min(degme + deg[i] - nv[i], dsum)
                hsh[i] = h
                if 5 * n - 5 * pos < dsum + nv[i]:
                    label[i] = n - label[i]
                    ln[i] = -1
            L2 = []
            for i in Lme:
                if label[i] == -stage and ln[i] == 1:
                    nv[me] += nv[i]
                    degme -= nv[i]
                    nv[i] = 0
                    parent[i] = me
                    ln[i] = -1
                elif label[i] < n:
                    L2.append(i)
            pos += nv[me]
            nv[me] = degme
            for i in L2:
                deg[i] = min(degme - nv[i] + deg[i], n - 1)
            Lme = sort_by_hash(L2, hsh)
            # supervariable detection on equal-hash runs (exact list equality)
            k = 0
            while k < len(Lme):
                e2 = k + 1
                while e2 < len(Lme) and hsh[Lme[e2]] == hsh[Lme[k]]:
                    e2 += 1
                for a in range(k, e2):
                    for b in range(a + 1, e2):
                        i, j = Lme[a], Lme[b]
                        if i < 0 or j < 0:
                            continue
                        if label[i] == label[j] and ln[i] == ln[j] and elen[i] == elen[j] and adj[i][:ln[i]] == adj[j][:ln[j]]:
                            nv[i] += nv[j]
                            deg[i] -= nv[j]
                            nv[j] = 0
                            parent[j] = i
                            ln[j] = -1
                            Lme[b] = -1
                k = e2
            Lme = [i for i in Lme if i >= 0]
            adj[me] = Lme
            elen[me] = 0
            ln[me] = len(Lme)
            for i in Lme:
                if label[i] == -stage:
                    insert(i)
        for i in range(n):
            if label[i] == n + stage:
                label[i] = pos
                ln[i] = -1
                pos += nv[i]
                nv[i] = 0
    for i in range(n):
        if label[i] < 0:
            r = parent[i]
            while parent[r] != r:
                r = parent[r]
            label[i] = label[r]
            label[r] += 1
    return label


def shell_sort(a, key, lo, hi):
    """ab9789 tail: shell sort with gaps len/2, then gap/2.2 (2 -> 1); equal keys do not move."""
    gap = (hi - lo) // 2
    while gap > 0:
        while True:
            for i in range(lo + gap, hi):
                v, x, j = key[i], a[i], i
                while j >= lo + gap and not (key[j - gap] <= v):
                    key[j], a[j] = key[j - gap], a[j - gap]
                    j -= gap
                key[j], a[j] = v, x
            if gap == 2 and lo + 1 < hi:
                gap = 1
                continue
            break
        gap = int(gap / 2.2)


def sort_by_hash(lst, hsh):
    """ab9789(n, keys, idx, tmp): sort indices by double key."""
    a = list(lst)
    key = [hsh[i] for i in a]
    # shortcut: the quicksort phase for n >= 100 (ab9789_quick) is not decoded; shell sort covers n < 100
    assert len(a) < 100, len(a)
    shell_sort(a, key, 0, len(a))
    return a


def symbolic(n, cp, ri, pos):
    """Column counts of L (strict lower) for P·AAᵀ·Pᵀ via the elimination tree."""
    perm = [0] * n
    for i, p in enumerate(pos):
        perm[p] = i
    parent = [-1] * n
    anc = [-1] * n
    for k in range(n):
        i = perm[k]
        for j0 in ri[cp[i]:cp[i + 1]]:
            j = pos[j0]
            if j >= k:
                continue
            while anc[j] != -1 and anc[j] != k:
                t = anc[j]
                anc[j] = k
                j = t
            if anc[j] == -1:
                anc[j] = k
                parent[j] = k
    # row structure counts by traversing
    cnt = [0] * n
    flag = [-1] * n
    for k in range(n):
        i = perm[k]
        flag[k] = k
        for j0 in ri[cp[i]:cp[i + 1]]:
            j = pos[j0]
            while j < k and flag[j] != k:
                cnt[j] += 1
                flag[j] = k
                j = parent[j]
    return cnt, parent


def main():
    d = sys.argv[1]
    names = sys.argv[2:] or sorted(f.split(".")[0] for f in os.listdir(d) if f.endswith(".json.gz"))
    ok = okc = 0
    for nm in names:
        r = json.load(gzip.open(os.path.join(d, nm + ".json.gz")))
        A, o = r["A"], r["ord"]
        m = A["m"]
        cp, ri = aat(m, A["beg"], A["len"], A["ind"])
        assert cp == o["colptr"] and ri == o["rowind"], nm
        n = o["n"]
        pos = amd(n, cp, ri, stage_labels(n, m, cp))
        cnt, _ = symbolic(n, cp, ri, pos)
        nnzl, ops = sum(c + 1 for c in cnt), float(sum((c + 1) ** 2 for c in cnt))  # 302921
        same = pos == o["calls"][0]["perm"]
        cnt_ok = nnzl == o["nnzL"] and ops == float.fromhex(o["ops"])
        ok += same
        okc += cnt_ok
        print(f"{nm:10s} perm={'ok' if same else 'DIFF'} counts={'ok' if cnt_ok else 'DIFF'} nnzL={nnzl}/{o['nnzL']} ops={ops:.0f}/{float.fromhex(o['ops']):.0f}")
    print("perm match", ok, "/", len(names), " nnzL+ops (302921) match", okc, "/", len(names))


if __name__ == "__main__":
    main()
