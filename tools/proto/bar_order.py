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


M64 = (1 << 64) - 1


def rand31(seed):
    """a7b4d6: PCG-style step of seed[0] (signed 64-bit state), tempered 31-bit output."""
    st = (seed[0] * 0x5851f42d4c957f2d + 0x14057b7ef767814f) & M64
    seed[0] = st
    u = st >> 32
    u ^= u >> 11
    u ^= (u & 0x13a58ad) << 7
    u &= 0xffffffff
    u ^= (u & 0x1df8c) << 15
    u &= 0xffffffff
    return (u ^ (u >> 18)) & 0x7fffffff


def rand01(seed):
    """a7b54f: rand31 * 4.656612873077393e-10."""
    return float(rand31(seed)) * 4.656612873077393e-10


class Heap:
    """a786fc..a7936f: binary max-heap on int keys with a locator; ties never move."""

    def __init__(self, n):
        self.node, self.key, self.loc = [], [], [-1] * n

    def reset(self):
        for v in self.node:
            self.loc[v] = -1
        self.node, self.key = [], []

    def _swap(self, i, j):
        nd, k = self.node, self.key
        nd[i], nd[j] = nd[j], nd[i]
        k[i], k[j] = k[j], k[i]
        self.loc[nd[i]] = i
        self.loc[nd[j]] = j

    def _up(self, i):
        while i > 0:
            p = (i - 1) // 2
            if self.key[i] <= self.key[p]:
                return
            self._swap(i, p)
            i = p

    def _down(self, i):
        n = len(self.node)
        while 2 * i + 1 < n:
            c = 2 * i + 2
            if n <= c or self.key[c] <= self.key[2 * i + 1]:
                c = 2 * i + 1
            if self.key[c] <= self.key[i]:
                return
            self._swap(i, c)
            i = c

    def insert(self, v, k):
        self.node.append(v)
        self.key.append(k)
        self.loc[v] = len(self.node) - 1
        self._up(len(self.node) - 1)

    def peek(self):
        return self.node[0] if self.node else -1

    def contains(self, v):
        return self.loc[v] >= 0

    def delete(self, v):
        i = self.loc[v]
        if i < 0:
            return
        self.loc[v] = -1
        last = len(self.node) - 1
        if i == last:
            self.node.pop()
            self.key.pop()
            return
        self.node[i], self.key[i] = self.node[last], self.key[last]
        self.loc[self.node[i]] = i
        self.node.pop()
        self.key.pop()
        if i == 0:
            self._down(0)
        elif self.key[(i - 1) // 2] < self.key[i]:
            self._up(i)
        else:
            self._down(i)

    def pop(self):
        v = self.node[0]
        self.delete(v)
        return v

    def update(self, v, k):
        i = self.loc[v]
        self.key[i] = k
        if i == 0:
            self._down(0)
        elif self.key[(i - 1) // 2] < k:
            self._up(i)
        else:
            self._down(i)


def sep_cost(p0, p1, p2):
    return float(p0 + p1 + p2) * 100.0 if p0 == 0 or p1 == 0 else float(p2) / (float(p0) * float(p1))


def fm_refine(ratio, seed, n, xa, ad, vw, where):
    """30d189: two-sided FM refinement of a vertex separator (part 2), up to 10 passes, rollback to the best cut."""
    pw = [0, 0, 0]
    edeg = [[0] * n, [0] * n]
    nsep = 0
    for v in range(n):
        pw[where[v]] += vw[v]
        if where[v] == 2:
            nsep += 1
            edeg[0][v] = edeg[1][v] = 0
            for u in ad[xa[v]:xa[v + 1]]:
                if where[u] != 2:
                    edeg[where[u]][v] += vw[u]
    limit = min(500, 5 * nsep)
    q = [Heap(n), Heap(n)]
    for _pass in range(10):
        q[0].reset()
        q[1].reset()
        perm = [v for v in range(n) if where[v] == 2]
        for k in range(len(perm) - 1, -1, -1):
            r = rand31(seed) % (k + 1)
            perm[r], perm[k] = perm[k], perm[r]
        for v in perm:
            q[0].insert(v, vw[v] - edeg[1][v])
            q[1].insert(v, vw[v] - edeg[0][v])
        moved = [-1] * n
        mincut = sep_cost(pw[0], pw[1], pw[2])
        mincutorder = -1
        swaps, hist, mlist = [], [], []
        nswaps = 0
        while nswaps < n:
            if limit < nswaps - mincutorder:
                break
            t = [q[0].peek(), q[1].peek()]
            if t[0] < 0:
                to = 1
            elif t[1] >= 0 and float(pw[1]) <= ratio * float(pw[0]):
                if float(pw[1]) * ratio < float(pw[0]):
                    to = 1
                else:
                    w0, e1 = vw[t[0]], edeg[1][t[0]]
                    a, b = pw[0] + w0, pw[1] - e1
                    c0 = float(a + pw[1] + (pw[2] - w0)) * 100.0 if a == 0 or b == 0 else float(pw[2] - w0 + e1) / (float(a) * float(b))
                    e0, w1 = edeg[0][t[1]], vw[t[1]]
                    a, b = pw[0] - e0, pw[1] + w1
                    c1 = float(pw[0] + pw[1] + pw[2]) * 100.0 if a == 0 or b == 0 else float(pw[2] - w1 + e0) / (float(b) * float(a))
                    to = 1 if c1 <= c0 else 0
            else:
                to = 0
            if t[to] < 0:
                break
            other = 1 - to
            v = q[to].pop()
            q[other].delete(t[to])
            where[v] = to
            moved[v] = nswaps
            swaps.append(v)
            hist.append(len(mlist))
            pw[2] -= vw[v]
            pw[to] += vw[v]
            for u in ad[xa[v]:xa[v + 1]]:
                if where[u] == 2:
                    edeg[to][u] += vw[v]
                    # asm 0x3281eb tests contains(q[other], v), not u: v was just deleted, so u's key never moves
                    if q[other].contains(v):
                        q[other].update(u, vw[u] - edeg[to][u])
                elif where[u] == other:
                    where[u] = 2
                    pw[other] -= vw[u]
                    mlist.append(u)
                    edeg[0][u] = edeg[1][u] = 0
                    pw[2] += vw[u]
                    for x in ad[xa[u]:xa[u + 1]]:
                        if where[x] == 2:
                            edeg[other][x] -= vw[u]
                            if q[to].contains(x):
                                q[to].update(x, vw[x] - edeg[other][x])
                        else:
                            edeg[where[x]][u] += vw[x]
                    if moved[u] == -1:
                        q[to].insert(u, vw[u] - edeg[other][u])
                        moved[u] = -2
            cost = sep_cost(pw[0], pw[1], pw[2])
            if cost < mincut:
                mincutorder = nswaps
                mincut = cost
            nswaps += 1
        hist.append(len(mlist))
        for i in range(nswaps - 1, mincutorder, -1):
            v = swaps[i]
            to = where[v]
            other = 1 - to
            where[v] = 2
            pw[to] -= vw[v]
            edeg[0][v] = edeg[1][v] = 0
            pw[2] += vw[v]
            for u in ad[xa[v]:xa[v + 1]]:
                if where[u] == 2:
                    edeg[to][u] -= vw[v]
                else:
                    edeg[where[u]][v] += vw[u]
            for k in range(hist[i], hist[i + 1]):
                u = mlist[k]
                pw[2] -= vw[u]
                pw[other] += vw[u]
                where[u] = other
                for x in ad[xa[u]:xa[u + 1]]:
                    if where[x] == 2:
                        edeg[other][x] += vw[u]
        if mincutorder == -1:
            break
    return where


def bisect(n, xa, ad, round_, seed):
    """30b0f2: multilevel vertex bisection. Returns part per node (0, 1, 2 = separator)."""
    levels = [(n, xa, ad, [1] * n)]
    maps = []
    while True:
        nl, xl, al, vl = levels[-1]
        deg = [xl[i + 1] - xl[i] for i in range(nl)]
        order = sorted(range(nl), key=lambda i: deg[i])  # counting sort, stable
        i = 0
        while i < nl:
            j = i + 1
            while j < nl and deg[order[j]] == deg[order[i]]:
                j += 1
            for k in range(j - 1, i, -1):
                r = rand31(seed) % (k - i + 1)
                order[k], order[i + r] = order[i + r], order[k]
            i = j
        match = [-1] * nl
        cptr, clist = [], []
        for v in order:
            if match[v] >= 0:
                continue
            best, bd = -1, nl + 1
            for u in al[xl[v]:xl[v + 1]]:
                if round_ == 3:
                    if match[u] < 0:
                        best = u
                        break
                elif match[u] < 0 and deg[u] < bd:
                    bd, best = deg[u], u
            c = len(cptr)
            match[v] = c
            cptr.append(len(clist))
            clist.append(v)
            if best >= 0:
                match[best] = c
                clist.append(best)
        nc = len(cptr)
        cptr.append(len(clist))
        mark = [-1] * nc
        cx, ca, cw = [], [], []
        for c in range(nc):
            cx.append(len(ca))
            mark[c] = c
            wsum = 0
            for f in clist[cptr[c]:cptr[c + 1]]:
                wsum += vl[f]
                for w in al[xl[f]:xl[f + 1]]:
                    cc = match[w]
                    if mark[cc] != c:
                        mark[cc] = c
                        ca.append(cc)
            cw.append(wsum)
        cx.append(len(ca))
        levels.append((nc, cx, ca, cw))
        maps.append((cptr, clist))
        if not (nc > 99 and nc != nl and len(levels) - 2 + 1 < 19):
            break
    lev = len(levels) - 1
    where = [2] * levels[lev][0]
    ratio = rand01(seed) * 5.0 + 5.0
    while True:
        nl, xl, al, vl = levels[lev]
        fm_refine(ratio, seed, nl, xl, al, vl, where)
        if lev == 0:
            return where
        cptr, clist = maps[lev - 1]
        fine = [0] * levels[lev - 1][0]
        for c in range(nl):
            for f in clist[cptr[c]:cptr[c + 1]]:
                fine[f] = where[c]
        where = fine
        lev -= 1


def nd_labels(n, m, cp, ri, mode, seed, round_=-1):
    """309dd0 modes 1-3: dense rows -2, then bisect every stage-1 component larger than max(600, n/100)."""
    lab = stage_labels(n, m, cp)
    leaf = max(600, n // 100)
    while True:
        comp = [-1 if lab[i] == -1 else 2000000000 for i in range(n)]
        nc = 0
        for s in range(n):
            if comp[s] != -1:
                continue
            comp[s] = nc
            stack = [s]
            while stack:
                v = stack.pop()
                for u in ri[cp[v]:cp[v + 1]]:
                    if comp[u] == -1:
                        comp[u] = nc
                        stack.append(u)
            nc += 1
        size = [0] * nc
        for i in range(n):
            if 0 <= comp[i] < nc:
                size[comp[i]] += 1
        nsplit = 0
        for c in range(nc):
            if size[c] <= leaf:
                continue
            loc = [-1] * n
            nodes = [i for i in range(n) if comp[i] == c]
            for k, i in enumerate(nodes):
                loc[i] = k
            xa, ad = [0], []
            for i in nodes:
                ad += [loc[j] for j in ri[cp[i]:cp[i + 1]] if loc[j] >= 0]
                xa.append(len(ad))
            part = bisect(len(nodes), xa, ad, round_, seed)
            for k, i in enumerate(nodes):
                if part[k] == 2:
                    lab[i] = -2
            nsplit += 1
        if nsplit == 0 or mode > 1:
            return lab


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


def col_structs(n, cp, ri, pos):
    """Row structure of each column of L (incl. diagonal, ascending) for the order pos (2e4f14 row merge)."""
    perm = [0] * n
    for i, p in enumerate(pos):
        perm[p] = i
    kids = [[] for _ in range(n + 1)]
    st = [None] * n
    for k in range(n):
        s = {k}
        for j0 in ri[cp[perm[k]]:cp[perm[k] + 1]]:
            if pos[j0] > k:
                s.add(pos[j0])
        for c in kids[k]:
            s.update(st[c][1:])
        st[k] = sorted(s)
        kids[st[k][1] if len(st[k]) > 1 else n].append(k)
    return st


def supernodes(n, st, m=None):
    """2e4f14: maximal supernodes, relaxed amalgamation, sibling merge of single columns, postorder.
    Returns (supernode list in final order, each = column list in final order; per-supernode row count)."""
    m = n if m is None else m
    ln = [len(s) for s in st]
    width, sn_of, first = [], [0] * n, []
    c = 0
    while c < n:
        w = 1
        while w < n - c and ln[c + w] == ln[c] - w and st[c][w] == c + w and (m <= c) == (m <= c + w):
            w += 1
        for k in range(w):
            sn_of[c + k] = len(width)
        first.append(c)
        width.append(w)
        c += w
    ns = len(width)
    ncol = list(width)
    nrow = [ln[f] for f in first]
    par = [ns] * ns
    child = [-1] * (ns + 1)
    sib = [-1] * ns
    for s in range(ns):
        if ncol[s] != nrow[s]:
            par[s] = sn_of[st[first[s]][ncol[s]]]
        sib[s] = child[par[s]]
        child[par[s]] = s
    head = [-1] * ns
    nxt = [-1] * n
    for col in range(n):
        nxt[col] = head[sn_of[col]]
        head[sn_of[col]] = col
    S = lambda k: k * (k + 1.0) * (2 * k + 1.0) / 6.0
    for s in range(ns):
        p = par[s]
        if p >= ns or (m <= head[s]) != (m <= head[p]):
            continue
        c_, r_, rp = ncol[s], nrow[s], nrow[p]
        b = r_ - c_
        lim = (3.0 if c_ < 192 else 0.1) * b
        # shortcut: float expression order follows the decompile; check the asm if a borderline case differs
        if r_ > 1 and (S(rp + c_) - S(rp)) - (S(r_) - S(b)) < lim * b + 1000.0:
            col = head[s]
            while col != -1:
                t = nxt[col]
                sn_of[col] = p
                nxt[col] = head[p]
                head[p] = col
                col = t
            if child[p] == s:
                child[p] = sib[s]
            else:
                q = child[p]
                while sib[q] != s:
                    q = sib[q]
                sib[q] = sib[s]
            ch = child[s]
            while ch != -1:
                t = sib[ch]
                par[ch] = p
                sib[ch] = child[p]
                child[p] = ch
                ch = t
            nrow[p] += ncol[s]
            ncol[p] += ncol[s]
            ncol[s] = -1
    for p in range(ns):
        if ncol[p] <= 0:
            continue
        c = child[p]
        while c != -1:
            if ncol[c] == 1 and nrow[c] > 24 and sib[c] != -1:
                c0 = head[c]
                prev = c
                while True:
                    if ncol[c] > 23:
                        break
                    s2 = sib[prev]
                    if ncol[s2] == 1:
                        k2 = head[s2]
                        if ln[k2] == ln[c0] and (m <= k2) == (m <= c0) and st[k2][1:] == st[c0][1:]:
                            nrow[c] += 1
                            ncol[c] += 1
                            nxt[k2] = head[c]
                            head[c] = k2
                            ch = child[s2]
                            while ch != -1:
                                t = sib[ch]
                                par[ch] = c
                                sib[ch] = child[c]
                                child[c] = ch
                                ch = t
                            ncol[s2] = -1
                            sib[prev] = sib[sib[prev]]
                            nx = sib[prev]
                        else:
                            prev = s2
                            nx = sib[s2]
                    else:
                        prev = s2
                        nx = sib[s2]
                    if nx == -1:
                        break
                c = sib[c]
            else:
                c = sib[c]
    stack, V = [], []
    r = child[ns]
    while r != -1:
        stack.append(r)
        r = sib[r]
    while stack:
        x = stack.pop()
        V.append(x)
        c = child[x]
        while c != -1:
            stack.append(c)
            c = sib[c]
    order = V[::-1]
    order = [s for s in order if head[s] < m] + [s for s in order if head[s] >= m]
    out = []
    for s in order:
        cols = []
        col = head[s]
        while col != -1:
            cols.append(col)
            col = nxt[col]
        out.append(cols)
    rows = [len(c) + ln[max(c)] - 1 for c in out]
    return out, rows


def split_blocks(widths, collen, small):
    """2e4f14 (Threads=1): F+0x78 = supernodes cut into blocks of at most 24 (when ops < 2·F+0x48) and at most
    192 columns (240 when the block's first column has >= 1920 rows). Entry = width at block start, -k inside."""
    lim = 24 if small else 2000000000
    out, j = [], 0
    for w in widths:
        end = j + w
        while j < end:
            b = min(lim, end - j)
            b = min(b, 192 if collen[j] < 1920 else 240)
            out += [b] + [-i for i in range(1, b)]
            j += b
    return out


def factor_stats(out, rows):
    """Factor NZ and Ops as printed by 2e8590 (F+0x38, F+0x60): dense trapezoid per supernode."""
    nz, ops = 0, 0.0
    for cols, r in zip(out, rows):
        for i in range(len(cols)):
            nz += r - i
            ops += float((r - i) ** 2)
    return nz, ops


def analyze(m, beg, ln, ind):
    """Full pipeline for a model with m rows: returns dict like the Rust ordering::analyze."""
    cp, ri = aat(m, beg, ln, ind)
    n = m
    pos = amd(n, cp, ri, nd_labels(n, m, cp, ri, 3, [0]))
    cnt, _ = symbolic(n, cp, ri, pos)
    st = col_structs(n, cp, ri, pos)
    out, rows = supernodes(n, st, m)
    newpos = [0] * n
    k = 0
    for cols in out:
        for c in cols:
            newpos[c] = k
            k += 1
    nz, ops = factor_stats(out, rows)
    collen = [r - i for cols, r in zip(out, rows) for i in range(len(cols))]
    ops2 = float(sum(x * x for x in ln))  # F+0x48: cost of forming A·D·Aᵀ (all columns; no dense ones here)
    snw = split_blocks([len(c) for c in out], collen, ops < 2.0 * ops2)
    # F+0xc8: per supernode, its columns then the rows below its last column, ascending in the new order
    rowidx = [x for cols in out for x in sorted([newpos[c] for c in cols] + [newpos[i] for i in st[max(cols)][1:]])]
    return dict(ops2=ops2, rowidx=rowidx, cp=cp, ri=ri, pos=pos, aat_nnz=cp[n], nnzL_ord=sum(c + 1 for c in cnt),
                ops_ord=float(sum((c + 1) ** 2 for c in cnt)), perm=[newpos[p] for p in pos],
                nnzL=nz, ops=ops, collen=collen, snw=snw)


def main():
    d = sys.argv[1]
    names = sys.argv[2:] or sorted(f.split(".")[0] for f in os.listdir(d) if f.endswith(".json.gz"))
    tally = dict(aat=0, md=0, ord=0, perm=0, sn=0, fnz=0)
    for nm in names:
        r = json.load(gzip.open(os.path.join(d, nm + ".json.gz")))
        A, o, sy = r["A"], r["ord"], r["sym"]
        x = analyze(A["m"], A["beg"], A["len"], A["ind"])
        res = dict(aat=x["cp"] == o["colptr"] and x["ri"] == o["rowind"],
                   md=x["pos"] == o["perm"],
                   ord=x["nnzL_ord"] == o["nnzL"] and x["ops_ord"] == float.fromhex(o["ops"]),
                   perm=x["perm"] == sy["perm"],
                   sn=x["snw"] == sy["snw"] and x["collen"] == sy["collen"] and x["rowidx"] == sy["rowidx"],
                   fnz=x["nnzL"] == sy["nnzL"] and x["ops"] == float.fromhex(sy["ops"]))
        for k, v in res.items():
            tally[k] += v
        bad = [k for k, v in res.items() if not v]
        print(f"{nm:10s} {'ok' if not bad else 'DIFF ' + ','.join(bad)}  FactorNZ {x['nnzL']}/{sy['nnzL']} Ops {x['ops']:.0f}/{float.fromhex(sy['ops']):.0f}")
    print("match:", ", ".join(f"{k} {v}/{len(names)}" for k, v in tally.items()))


if __name__ == "__main__":
    main()
