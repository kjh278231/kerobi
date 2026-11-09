# M2 clique 묶음 해독 (`45885d`, `451bed`, `52be10`, `46a62d`)

Gurobi 13.0.3 MIP presolve(`3cd406`) 바깥 패스의 clique 묶음. 포팅 전 해독 노트(2026-11-06). 상태 규약은 [m1_presolve.md](../m1/m1_presolve.md) §5, 연결 리스트 노드는 [m1_presolve_agg.md](../m1/m1_presolve_agg.md) §1.1(노드 0x20바이트 {f64 값, i32 행 @+8, i32 열 @+0xc(−1 = 죽음), 행 안 다음 @+0x10, 열 안 다음 @+0x18}, 행 머리 +0x178, 열 머리 +0x180)를 따른다.

표기: `W`는 작업량 `*(state+0x458)`, `μ`는 `state+0x448`. `W += x·c·μ`는 `W = ((x as f64 · c) · μ) + W`이다(덧셈은 항목마다 따로, 적힌 순서대로). `ft` = FeasibilityTol(env+0x2460, 1e-6), `INF` = env+0x2458(1e30), `itol` = IntFeasTol(env+0x2468, 1e-5), `ZERO` = GURO_PAR_ZERO(env+0x2498, 1e-13). `T` = 100·ft. `act_max[r]` = +0x90(max activity − rhs, 유한 부분), `inf_max[r]` = +0x70, `act_min[r]` = +0x98, `inf_min[r]` = +0x78. `rc` = 행 개수 +0x60, `cc` = 열 개수 +0x68, `vt` = +0x150. 리터럴 `l ≥ 0`은 x_l, `l < 0`은 x_{−1−l}의 여(complement). `var(l)` = l ≥ 0이면 l, 아니면 −1−l.

## 0. 요약

| 함수 | 하는 일 | 기본값 38개에서 상태를 바꾸는 모델 |
|---|---|---|
| `451bed` | 행에서 clique 표를 만든다(행 clique + 이진 열을 거친 2-clique). probing `4d3a1d`도 쓴다 | neos-860300(호출 1: 거의 고정된 열 152개를 `3de18e`로 고정) |
| `45885d` | clique 표로 행을 clique 분할해 activity를 조이고, bound 강화·중복 행 삭제·계수 강화 | csched007, csched008, gmu-35-40, gmu-35-50, neos-3754480-nidda, neos-4954672-berkel, neos-860300, neos859080, sp150x300d |
| `52be10` | 이진 ±1 집합 포장 행의 리터럴 쌍 그래프. 다른 행에 덮이는 2-행 삭제, 삼각형 클리크로 2-행 확장(`=`이면 고정) | gmu-35-40(호출 1, 행 17개 삭제). 나머지는 훑기 작업량만 |
| `46a62d` | 같은 크기 계수의 이진 행을 ±1로 정규화하고 행 리스트를 다시 엮은 뒤(B 노드 \|a\| 내림차순), 다른 clique 행에 든 행 삭제, 충돌 리터럴로 clique 행 확장 | 행 삭제·확장은 gmu-35-40, gmu-35-50. 정규화·행 재배열·열 압축(+0xd8)은 13개 모델 더(리스트 순서가 바뀐다) |

포트(2026-11-06, `crates/mip/src/`): `451bed` = `clique.rs`의 `State::clique_table(p, list, bin_only) -> Result<Cliques, i32>`(probing도 이것을 쓴다), `45885d` = `State::clique`(clique.rs), `52be10` = `State::set_packing`(setpack.rs, `5279b5`·`528f31`·`469c12`·`a9dcac` 정렬 포함), `46a62d` = `State::clique_rows(p, mode)`(cliquerows.rs, `4694fe` 포함). 열 고정·활동 재계산·GC는 공용 helper(`fix_col`, `fix_near`, `activity`, `gc`)를 쓴다. shortcut: reduction timer를 받지 않아 `adf2be` 정지와 46a62d 예산 B를 보지 않는다(38개 모델에서 걸리지 않음), 노드 풀 증설 `3b85ea`는 assert. 확인: `tools/mip_dump_check.py` 45885d·52be10·46a62d 각 78/78, `crates/mip/tests/clique.rs`(451bed 5개 fixture), sweep_check의 첫 불일치가 셋 중 어디에도 없다. 덤프는 이제 +0xf0(`nbin`, 경계 반올림 때 센 값으로 그 뒤 고치지 않는다)을 담고 `State::from_parts`가 쓴다(mas74 등에서 'B' 개수와 다르다).

검증: 네 함수 모두 Python 참조 모델을 진입 상태에 돌려 복귀 상태(리스트 순서·값, lb/ub, rhs, 개수, activity, 카운터, 레코드)와 작업량을 비트 단위로 비교했다. `45885d`(+`451bed`) 78회, `46a62d` 78회는 38개 모델 전부, `52be10`은 동작하는 gmu-35-40과 훑기만 하는 모델, 합성 모델에서 같다(§7).

## 1. 호출 지점 (`3cd406` L3610–3700)

```
if env.PRECLIQUE (env+0x254c, 기본 1) != 0 {
    for (f, timer) in [(45885d, local_598), (52be10, local_658), (46a62d, local_618)] {   // 각자 timer
        if adf236(timer, W) < 0.8 {          // 이 reduction의 누적 작업량 / 예산
            ade8ca(timer, W);                 // 시작: t[0] = (t[0] − W) + 1.0, 실행 중 표시
            ret = f(state, env, timer);       // rdx = timer(디컴파일은 빠뜨림). 46a62d만 넷째 인자 mode(§6.1)
            ade770(timer, W);                 // 끝: t[0] += W
            if ret != 0 { goto LAB_004bbf43 } // 상태 코드로 끝
            count += 1; if state.prelimit(+0x1dc) <= count { goto 마무리(LAB_004b2415) }
            ret = 3ba5ac(env, state, 0) ?: 3ba9de(env, state); if ret != 0 { goto LAB_004bbf70 }
        }
    }
}
3ba9de(env, state)
```
- PRELIMIT 카운트는 호출마다 +1이다(아무것도 안 바꿔도). 예산 판정(`adf236` ≥ 0.8)으로 건너뛰면 세지 않는다.
- timer 구조: `t[0]` 누적 작업량, `t[2]` 예산, `*(int*)(t+0x28)` 실행 중. `adf236` = t[2] < 0이면 1, t[2] == 0이면 0, 아니면 t[0]/t[2](실행 중이면 t[0]+W). `adf2be(t, W)` = 같은 값 ≥ 1. 예산은 `3cd406`이 정한다(L1560–1585, 식은 spine 노트 몫). 관측값은 45885d·46a62d가 1250·nnz + 5e6 꼴(glass4 7,268,750, gmu-35-40 11,053,750, neos-860300 485,411,250), 52be10은 그 두 배(glass4 14,537,500, gmu-35-40 22,107,500). 기본값 38개에서는 예산에 걸린 적이 없다.
- 상태 작업 배열: `+0x390`(i32, 길이 ≥ m. 451bed 출력 행 목록·45885d 출력 행 목록·52be10 클리크 멤버·46a62d 행 목록. 진입 값 무관), `+0x3a8`(i32, 45885d는 열 표시(n), 46a62d는 행 카운터(m)로 쓰므로 길이 ≥ max(m, n)), `+0x3b0`(f64, 길이 n, 45885d 계수 흩기), `+0x3b8`(46a62d 고정 열 목록). `+0x3a8`·`+0x3b0`은 진입 때 0이어야 하고 나갈 때 0으로 돌려놓는다.

## 2. `451bed`: clique 표 만들기 (공유 인터페이스)

### 2.1 인자와 출력

`451bed(env, state, nlist, list, bin_only, out_nrec, out_maxlen, rec_rows, rec_flag, out_ncl, out_sense, out_flag, out_start, out_lits, out_colstart, out_cidx) -> i32`

| 인자 | `45885d`(0x405273) | probing `4d3a1d`(0x461719) | 뜻 |
|---|---|---|---|
| nlist, list | `53a487` 목록(개수, 행 번호) | m, NULL | 볼 행. NULL이면 0..nlist |
| bin_only | 0 | 1 | 1이면 'B' 열만 리터럴이 된다. 0일 때만 §2.3이 돈다 |
| rec_rows | state+0x390 | state+0x390 | 출력: "clique로 다 덮이지 않은 행" 목록 |
| rec_flag | malloc(m) u8 | NULL | 출력: 그 행이 clique를 냈는지(0/1) |
| out_flag | &int* | NULL | clique마다 1이면 정확(비이진 리터럴 ≤ 1개), 0이면 아님 |

출력(호출자가 해제): `ncl`, `sense[ncl]`(u8 '=' 분할 / '<' 묶음), `flag[ncl]`(int), `start[ncl+1]`(int, 리터럴 구간), `lits[start[ncl]]`(int 리터럴), `colstart[n+1]`(int, 실제 할당 n+2), `cidx[start[ncl]]`(열 j가 든 clique: 양의 리터럴이면 c, 여이면 −1−c. colstart[j]..colstart[j+1]). **ncl == 0이면 out_nrec·out_maxlen은 0으로 남고 배열은 NULL이다**(초기화만 하고 끝에서 쓰지 않음). 반환 0, 메모리 부족 10001(0x2711), 실행 불가 3(state+0x470 = 열).

clique 뜻: 리터럴 "켜짐" = 변수가 기준 bound에서 벗어남(양의 리터럴은 lb에서, 여는 ub에서). 한 clique에서 켜질 수 있는 리터럴은 하나뿐이다. sense '='이면 정확히 하나가 켜진다.

### 2.2 행 clique (모든 호출)

```
nrec = 0; maxlen = 0; nlit = 0; ncl = 0
for i in 0..nlist {
    r = list ? list[i] : i
    if rc[r] <= 0 { continue }                   // 기록도 안 한다
    so = sense[r]; s = 0; nb = 0; produced = 0; mark = nlit
    if inf_min[r] == 0 && inf_max[r] == 0 {
        ent = 0
        for node in row r {                       // +0x10 순서, 죽은 노드 포함
            ent += 1; j = node.col; if j < 0 { continue }
            w = ub[j] − lb[j]
            if w <= 1e-10 {                       // (45885d 첫 훑기는 < 1e-10)
                if ub[j] + ft < lb[j] { state.0x470 = j; return 3 }
                mx = 0.0; if w > 1e-13 { mx = max |a| over live nodes of col j (maxsd, mx <= |a|면 교체) }
                if w·mx < ft {
                    v = if ub[j] <= 1e-10 && lb[j] >= −1e-10 { 0.0 } else if obj[j] > 0 { lb[j] } else { ub[j] }
                    ret = 3de18e(v, INF, env, state, j, rec=1); if ret != 0 { return ret }
                    continue                      // 이 노드는 리터럴 처리 없음
                }
            }
            if bin_only != 0 && vt[j] != 'B' { so = '<'; continue }
            a = node.val
            if |a| < 1e-4 { so = '<'; continue }
            if s < 0 || { d = w·|a| + act_min[r]; d < −1e-10 } {        // s < 0이면 d를 구하지 않는다
                if s > 0 || sense[r] != '=' || act_max[r] − w·|a| > 1e-10 { so = '<'; continue }
                a = −a; s −= 1; if vt[j] != 'B' { nb += 1 }            // max 쪽(여) 모드
            } else {
                s += 1; if vt[j] != 'B' { nb += 1 }
                if so == '=' && d > 1e-10 { so = '<' }
            }
            push lit (a >= 0 ? j : −1−j)          // 버퍼 0x400에서 두 배씩, 2e9 넘으면 행 루프를 끝낸다
        }
        W += ent·8.0·μ
    }
    if |s| >= 2 { start[ncl] = mark; sense_c[ncl] = (s == rc[r]) ? so : '<'; flag[ncl] = (nb < 2); ncl += 1; produced = 1 }
    else { nlit = mark }
    if s != rc[r] && −s != rc[r] {                // rc는 3de18e 고정 뒤 값
        rec_rows[nrec] = r; rec_flag?[nrec] = produced; nrec += 1; maxlen = max(maxlen, rc[r])
    }
}
W += 8.0·nlist·μ
```
- 판정은 "한 변수만 끝까지 움직여도 행의 여유(rhs − min activity)를 다 쓴다"이다. s > 0 모드는 min activity, s < 0 모드는 `=` 행의 max activity를 쓴다. 첫 리터럴이 정한 모드와 다른 항목은 '<'로만 만든다.
- 행이 clique 하나로 다 덮이면(|s| == rc) rec_rows에 넣지 않는다.

### 2.3 이진 열을 거친 2-clique (bin_only == 0이고 1 ≤ state+0xf0 ≤ n−2일 때)

state+0xf0 = 이진 열 수(`State::nbin`, `3cd406` 경계 반올림이 센다). SOS(+0x1f8 > 0)가 있으면 먼저 `a85f71`로 열→SOS 표를 만들고 SOS1·2변수 `=` 행 짝을 쓰는 분기가 있는데 기본값 38개에는 SOS가 없어 해독하지 않았다(§8).
```
visited = calloc(m); inl = calloc(n)              // visited는 호출 내내 지우지 않는다
for b in 0..n { if vt[b] != 'B' { continue }
    L0 = []; L1 = []; ce = 0
    for node in col b {                           // +0x18 순서
        if |L0| >= 16 && |L1| >= 16 { break }     // 노드마다 맨 앞에서 본다
        ce += 1
        if node.col < 0 || visited[node.row] { continue }
        r = node.row; ab = node.val
        if inf_min[r] == 0 && |ab| + act_min[r] >= −1e-10 { dr = 1.0 }
        else if sense[r] == '=' && inf_max[r] == 0 && |ab| + act_max[r] <= 1e-10 { dr = −1.0 }   // asm 그대로(act_max − |ab|가 아니다)
        else { continue }
        visited[r] = 1; L = (ab·dr > 0) ? L0 : L1; it = 0
        for e in row r { if |L| >= 16 { break }; it += 1
            v = e.col; if v >= 0 && vt[v] != 'B' && !inl[v] { inl[v] = 1; L.push(dr·e.val > 0 ? v : −1−v) } }
        W += it·3.0·μ
    }
    W += ce·7.0·μ
    for l in L0 { inl[var(l)] = 0 }; W += (|L0| as f64 + |L0| as f64)·μ
    for l in L1 { inl[var(l)] = 0 }; W += (|L1| as f64 + |L1| as f64)·μ
    if L0 비었음 || L1 비었음 { continue }
    lo = |L0|; hi = |L1|; if lo·hi > 16 { if hi < lo { lo = 16 / hi } else { hi = 16 / lo } }   // 정수 나눗셈
    for i in 0..lo { for k in 0..hi { clique {L0[i], L1[k]}, sense '<', flag 1 }; W += hi·6.0·μ }
    W += lo as f64·μ
}
W += n as f64·μ
```
뜻: b = 1이면 L0 행이 꽉 차서 L0 변수가 기준에 묶이고, b = 0이면 L1 변수가 묶인다. 그래서 L0 리터럴과 L1 리터럴은 함께 켜질 수 없다. 기본값에서 쌍이 생기는 모델은 neos-4954672-berkel(252개)과 neos859080(40개)뿐이다.

### 2.4 마무리 (ncl > 0일 때)
```
start[ncl] = nlit; 배열을 실제 크기로 줄인다(realloc)
W += nlit as f64·μ
colstart = calloc(n+2); for l in lits { colstart[var(l)+2] += 1 }
if n >= 1 { 누적합 colstart[j+2] += colstart[j+1] (j = 0..n); W += n as f64·μ }
for c in 0..ncl { for k in start[c]..start[c+1] { v = var(lits[k]); cidx[colstart[v+1]] = lits[k] >= 0 ? c : −1−c; colstart[v+1] += 1 }
                  W += ((start[c+1]−start[c]) as f64·μ)·3.0 }        // 순서 주의: μ를 먼저 곱한다
W += ncl as f64·μ
out_nrec = nrec; out_maxlen = maxlen; 나머지 출력 포인터
```
열 j 안의 cidx 순서는 clique 번호 오름차순이다. colstart는 결과적으로 길이 n+1의 시작 배열이다.

## 3. `45885d`: clique 분할로 행 강화

`45885d(state, env, timer) -> i32`. 반환 0, 3(실행 불가: +0x470 = 열 또는 +0x478 = 행), 10001, `3de18e`/`3df916`/콜백의 코드.

### 3.1 준비
```
W0 = W                                            // 1e7 콜백 기준
(nlist, list) = 53a487(tracker, state)            // LP 포트 adjacent_list
rec_flag = malloc(m)
ret = 451bed(env, state, nlist, list, 0, &nrec, &maxlen, state.0x390, rec_flag, &ncl, &csense, &cflag, &cstart, &lits, &colstart, &cidx)
if ret != 0 || ncl == 0 { free; return ret }
cnt = calloc(max(ncl, m)) int; lst = int[max(ncl, m)]
newlb = f64[n]; newub = f64[n]; chg = int[n] (nchg = 0); chgf = state.0x3a8; sc = state.0x3b0
part 배열(길이 maxlen): psense u8, pflag int, pstart int, plit int, pco f64, pmin f64, pmax f64
ncoef = 0; nrem = 0; rowsout = state.0x390 (nout = 0)   // 451bed 출력 목록 위에 덮어 쓴다(읽은 자리 뒤로만 쓴다)
```

### 3.2 행 루프
```
idx = 0
while idx < nrec {
    if adf2be(timer, W) { break }
    if W − W0 > 1e7 {
        ret = 3ba5ac(env, state, 0, 0, −1); if ret { goto 정리 }
        ret = 3ba9de(env, state, &stop); if ret || state.prelimit < 0 { goto 정리 }
        if stop { break }; W0 = W
    }   // 정리: chg의 chgf를 0으로, W += (k+k)·μ (k = 지운 수), 카운터 갱신 없이 ret 반환
    r = rows[idx]; prod = rec_flag[idx]
    if inf_max[r] != 0 && (inf_min[r] != 0 || sense[r] != '=') { idx += 1; continue }
    // (a) 첫 훑기: 거의 고정된 열 고정. 451bed와 같고 판정만 w < 1e-10(엄격)
    k = 0; for node in row r { k += 1; if node.col >= 0 && ub−lb < 1e-10 { …451bed와 같은 실행 불가/3de18e… } }
    W += k·4.0·μ
    if rc[r] < 2 { idx += 1; continue }
    // (b) 계수 흩기와 clique 세기
    nd = 0; nn = 0
    for node in row r { nn += 1; j = node.col; if j < 0 { continue }
        sc[j] = node.val
        for t in colstart[j]..colstart[j+1] { c = var(cidx[t]); if cnt[c] == 0 { lst[nd] = c; nd += 1 }; cnt[c] += 1 }
        W += (colstart[j+1]−colstart[j])·3.0·μ }
    W += nn·5.0·μ
    if nd == 0 { 행의 살아 있는 열 sc = 0 (노드 수 c2); W += c2·3.0·μ; idx += 1; continue }
    // (c) 탐욕 clique 분할
    target = rc[r]; p = 0; nlit = 0; cur = 0; acc = 0.0
    loop {
        pstart[p] = nlit
        if p >= nd || acc >= 1e8 { break }
        if timer && adf2be(timer, W) { break }
        best = 1; bestpos = −1; st = cur; scanned = 0
        loop {                                            // lst[p..nd)를 cur부터 돌아가며
            c = lst[cur]
            if cnt[c] == target { pick = cur; stop = cur; break }
            if best < cnt[c] { best = cnt[c]; bestpos = cur }
            cur += 1; if cur == nd { cur = p }
            if cur == st { pick = bestpos; stop = cur; break }
            scanned += 1
        }
        W += (scanned as f64 + scanned as f64)·μ
        if pick == −1 { break }
        acc += scanned; swap(lst[pick], lst[p])
        if best < target && stop != pick { target = best }
        cur = (stop == p) ? stop + 1 : stop
        c = lst[p]; s0 = nlit
        for t in cstart[c]..cstart[c+1] {
            l = lits[t]; v = var(l); a = sc[v]; if a == 0.0 { continue }
            plit[nlit] = l; pco[nlit] = a; nlit += 1; sc[v] = 0.0
            for u in colstart[v]..colstart[v+1] { c3 = var(cidx[u]); if cnt[c3] != 0 { cnt[c3] −= 1 } }
            W += (colstart[v+1]−colstart[v])·2.0·μ
        }
        W += (cstart[c+1]−cstart[c])·7.0·μ
        psense[p] = (nlit − s0 == cstart[c+1]−cstart[c]) ? csense[c] : '<'; pflag[p] = cflag[c]; p += 1
    }
    np = p; W += np·4.0·μ
    for i in 0..nd { cnt[lst[i]] = 0 }; W += (nd as f64 + nd as f64)·μ
    if np == 0 { 행의 살아 있는 열 sc = 0; W += (노드 수)·3.0·μ; idx += 1; continue }
    // (d) 남은 열(단독)과 clique별 범위
    nchg0 = nchg; minS = maxS = minM = maxM = 0.0; c2 = 0
    for node in row r { c2 += 1; j = node.col
        if j >= 0 && sc[j] != 0.0 { a = sc[j]; sc[j] = 0.0; pco[nlit] = a; plit[nlit] = j; nlit += 1
            if a <= 0 { if ub[j] < INF { minS += a·ub[j]; minM −= |ub[j]|·a }; if −INF < lb[j] { maxS += lb[j]·a; maxM −= |lb[j]|·a } }
            else      { if ub[j] < INF { maxS += a·ub[j]; maxM += |ub[j]|·a }; if −INF < lb[j] { minS += lb[j]·a; minM += |lb[j]|·a } } } }
    W += c2·7.0·μ
    for q in 0..np {
        dmin = 0.0; dmax = 0.0; bs = 0.0
        for t in pstart[q]..pstart[q+1] { l = plit[t]; a = pco[t]
            (base, d) = l < 0 ? (ub[v], lb[v] − ub[v]) : (lb[v], ub[v] − lb[v])
            bs = bs + a·base; d = d·a; dmin = (dmin <= d) ? dmin : d; dmax = (d <= dmax) ? dmax : d }
        W += (pstart[q+1]−pstart[q])·4.0·μ
        pmin[q] = dmin; pmax[q] = dmax
        minS = (minS + dmin) + bs; maxS = (maxS + bs) + dmax; minM = (minM + |dmin|) + |bs|; maxM = (maxM + |dmax|) + |bs|
    }
    W += np·3.0·μ
    okmin = minM < 1e7; okmax = maxM < 1e7
    cmax = maxS − rhs[r]; cmin = minS − rhs[r]        // clique를 반영한 activity − rhs (가드 없는 덧셈)
    if inf_min[r] == 0 && T < cmin { state.0x478 = r; return 3 }
    if sense[r] == '<' {
        if !(okmin || okmax) { idx += 1; continue }
        if !(inf_max[r] != 0 || T <= cmax) {
            if prod == 0 && cmax < 1e-10 {            // 중복 행 삭제
                nrem += 1; for node in row r { if node.col >= 0 { 5379f1(tracker, node.col); cc[node.col] −= 1; node.col = −1 } }
                W += (노드 수)·3.0·μ; rc[r] = −2
            }
            idx += 1; continue
        }
    } else {
        if inf_max[r] == 0 && cmax < −T { state.0x478 = r; return 3 }
        if !(okmin || okmax) { idx += 1; continue }
    }
    → (e), (f), (g)
    idx += 1
}
```

### 3.3 (e) bound 강화
```
R1 = T − cmin; R2 = T + cmax; ncoef0 = ncoef
for s in [−1.0, 1.0] {                               // s = −1은 '=' 행만
    if !(s >= 0 || sense[r] == '=') { continue }
    if !((s > 0 && inf_min[r] == 0 && okmin) || (s < 0 && inf_max[r] == 0 && okmax)) { continue }
    for q in 0..np {
        res = (s <= 0) ? R2 − pmax[q] : pmin[q] + R1
        for t in pstart[q]..pstart[q+1] {
            sa = s·pco[t]; l = plit[t]; v = var(l)
            if !(|sa| >= 1e-6) { continue }
            if pflag[q] == 0 && (vt[v] == 'C' || ub[v] − lb[v] > 1.5) { continue }
            u = ub[v]; lo = lb[v]
            if l < 0 {
                if sa >= 0 { d = (u < INF) ? −(u − lo) : lo; if res < sa·d { mark(v,1); newlb[v] = u } }
                else { x = res/sa + u; if vt[v] != 'C' { x = ceil(x − itol) }; x = (x < u) ? x : u      // minsd
                       if lo + 1e-6 < x { mark(v,1); newlb[v] = x } }
            } else {
                if sa <= 0 { d = (lo > −INF) ? u − lo : u; if res < sa·d { mark(v,2); newub[v] = lo } }
                else { x = res/sa + lo; if vt[v] != 'C' { x = floor(x + itol) }; x = (x > lo) ? x : lo    // maxsd
                       if x < u − 1e-6 { mark(v,2); newub[v] = x } }
            }
        }
        W += (pstart[q+1]−pstart[q])·7.0·μ
    }
    W += (np as f64 + np as f64)·μ
    dd = (s <= 0) ? cmax : −cmin
    t = pstart[np]
    while t < rc[r] {                                   // 단독 열(plit는 열 번호)
        sa = s·pco[t]; j = plit[t]
        if sa > 1e-6 { x = dd/sa + lb[j]
            if chgf[j] & 2 == 0 { if x < ub[j] − T { mark(j,2); newub[j] = x } } else if x < newub[j] − T { newub[j] = x } }
        else if sa < −1e-6 { x = dd/sa + ub[j]
            if chgf[j] & 1 == 0 { if lb[j] + T < x { mark(j,1); newlb[j] = x } } else if newlb[j] + T < x { newlb[j] = x } }
        t += 1
    }
    W += (t − pstart[np])·5.0·μ
}
mark(v, bit) = { if chgf[v] == 0 { chg[nchg] = v; nchg += 1 }; chgf[v] |= bit }
```
- clique 리터럴의 새 bound는 이전 newlb/newub와 비교하지 않고 덮어 쓴다. 단독 열은 비교한다.
- 비교는 모두 현재 lb/ub 기준이다(bound는 §3.6에서야 바뀐다).

### 3.4 (f) 계수 강화 (`<` 행, inf_max == 0, 이 행에서 새 bound 표시가 없고(nchg == nchg0), okmax)
```
for q in 0..np {
    dd = cmax − pmax[q]
    if !(pmin[q] + dd < −T && |dd| < 1e8) { continue }
    de = −dd; if de <= T { de = 0.0 }; dd = dd + de
    t = pstart[q]; while t < pstart[q+1] && vt[var(plit[t])] != 'C' && ub−lb == 1.0 { t += 1 }
    W += (t − pstart[q])·2.0·μ
    if t < pstart[q+1] { continue }                    // 모두 0/1 폭일 때만
    rhs[r] = rhs[r] − de
    for t in pstart[q]..pstart[q+1] { a = pco[t]; l = plit[t]; v = var(l)
        if (l >= 0) == (a > 0) {
            if de > 0 { if l < 0 { pco[t] = a + de; rhs[r] = ub[v]·de + rhs[r] } else { pco[t] = a − de; rhs[r] = rhs[r] − lb[v]·de }; ncoef += 1 }
        } else if T < |a| − dd {
            if l < 0 { if ub[v] == 0 || a < 1e8 { ncoef += 1; x = rhs[r] − |a|·ub[v]; rhs[r] = x + ub[v]·dd; pco[t] = dd } }
            else     { if lb[v] == 0 || −a < 1e8 { ncoef += 1; x = |a|·lb[v] + rhs[r]; rhs[r] = x − lb[v]·dd; pco[t] = −dd } }
        } }
    W += (pstart[q+1]−pstart[q])·2.0·μ
}
W += (np as f64 + np as f64)·μ
end = rc[r]; t = pstart[np]; last = t
while t < end { j = plit[t]; last = t + 1
    if vt[j] != 'C' && lb[j] == 0.0 { a = pco[t]
        if a > R2 && ub[j] == 1.0 { ncoef += 1; d = a − cmax; pco[t] = a − d; rhs[r] = rhs[r] − d }
        else if R2 < −a { ncoef += 1; pco[t] = a + (−(cmax + a)) } }
    t += 1 }
W += (last − pstart[np])·5.0·μ
```
R2 = T + cmax(asm: xmm8, 0x4064e7). 이 블록은 (e)를 지난 행에서만 돈다.

### 3.5 (g) 계수 되쓰기 (ncoef != ncoef0일 때)
```
rowsout[nout] = r; nout += 1; 53763f(tracker, r)
for i in 0..rc[r] { sc[var(plit[i])] = pco[i] }; W += rc[r]·3.0·μ
c2 = 0
for node in row r { c2 += 1; j = node.col; if j < 0 { continue }
    x = sc[j]
    if |x| <= ZERO { 5379f1(tracker, j); node.col = −1; cc[j] −= 1; rc[r] −= 1 }
    else { if 1e-10 < |node.val − x| { 5379f1(tracker, j) }; node.val = x }
    sc[j] = 0.0 }
W += c2·4.0·μ
```
행 안 순서는 그대로다(값만 바뀌거나 col = −1). activity는 §3.6의 `370047`이 다시 구한다.

### 3.6 마무리
```
W += idx·8.0·μ                                       // idx = 루프를 나온 위치
rowflag = cnt(다시 0인 상태); for r in rowsout[..nout] { rowflag[r] = 1 }; W += (nout as f64 + nout as f64)·μ
for j in chg[..nchg] (표시 순서) {
    5379f1(tracker, j)
    for node in col j { if node.col >= 0 && rowflag[node.row] == 0 { rowflag[node.row] = 1; rowsout[nout] = node.row; nout += 1 } }
    if chgf[j] & 1 { if vt[j] != 'C' { newlb[j] = ceil(newlb[j] − itol) }; lb[j] = newlb[j] }
    if chgf[j] & 2 { if vt[j] != 'C' { newub[j] = floor(newub[j] + itol) }; ub[j] = newub[j] }
    if ub[j] − 1e-10 < lb[j] { ret = 3df916(ft, env, state, j); if ret { goto 해제(카운터 갱신 없음) } }
    chgf[j] = 0
}
W += nchg·8.0·μ
state.0x3fc += ncoef; state.0x3f8 += nchg; state.0x404 += nrem
if nout != 0 { 370047(state, env, nout, rowsout) }
```
- bound는 바로 대입한다(activity 증분 갱신 없음). `3df916`의 고정도 activity를 건드리지 않는다. 바뀐 열의 행과 계수가 바뀐 행 전부를 `370047`이 다시 더한다.
- 레코드는 `3de18e`·`3df916`(→ `39ab33`)의 type 1뿐이다. bound·계수·행 삭제는 레코드가 없다.

## 4. 보조 함수 (기본값에서 도는 것)

| 함수 | 기본값 호출 | 내용 |
|---|---|---|
| `53a487(tracker, state, &n, &list)` | 45885d 진입 | LP 포트 `adjacent_list`(m1_presolve.md §7 stage 3)와 같은 규칙. 다만 MIP 상태(+0xe0 == 0)는 이웃을 CSR 대신 행·열 리스트(+0x178/+0x180)로 훑는다(L295, L523). 이 목록 순서가 451bed의 행 순서다 |
| `3de18e(v, INF, env, state, j, rec)` | 451bed(neos-860300 152회) | MIP 연결 리스트판 열 고정. 아래 |
| `3df916(tol, env, state, j)` | 45885d 끝(gmu-35-40 15회, gmu-35-50 23회) | 거의 고정된 열 고정(activity 갱신 없음). 아래 |
| `39ab33(v, env, state, j)` | 두 고정 함수 | MIP(state+0xfc ≠ 0)이면 +0x408 += 1, 레코드 type 1: 헤더 0x18바이트 {i32 type=1, 패딩, data*, next*}(+0x370 앞에 넣는다), data 16바이트 {i32 j, 패딩 4, f64 v}. 작업량 없음 |
| `53802c(tracker, state, j)` | 두 고정 함수 | 열 j의 살아 있는 행 표시. 연결 리스트(+0xe0 == 0)면 W += (μ·5.0)·(열 노드 수, 죽은 것 포함) |
| `5379f1`/`53763f` | 열/행 표시 | 추적기 표시(작업량 없음) |
| `370047(state, env, k, list)` | 45885d 끝 | LP 포트 `agg_activity`(crates/lp/src/presolve/agg.rs)와 같다. 행마다 W += μ·7.0·(방문 노드), 끝에 W += μ·8.0·k |
| `3ba5ac(env, state, 0, 0, −1)`, `3ba9de(env, state, &stop)` | 45885d(neos-860300 2회) | 콜백·예산. 기본값에서 작업량을 더하지 않는다(neos-860300에서 확인) |

`3de18e(v, INF, env, state, j, rec)`:
```
if flags(+0x170)[j] & 0xffffff77 == 0 { if rec { (LP 상태면 3a0f5b); ret = 39ab33(v, env, state, j); if ret { return 10001 } } } else { rec = 0 }
objcon(+0xc8) = obj[j]·v + objcon; obj[j] = 0; (Q 대각 +0x250, Q 비대각 +0x260, PWL 0x80 분기는 MIP 기본값에서 안 돎)
flags[j] &= ~8
if cc[j] >= 1 {
    53802c(tracker, state, j)
    for node in col j { if node.col < 0 { continue }; a = node.val; r = node.row; t = v·a
        node.col = −1; rc[r] −= 1; rhs[r] = rhs[r] − t
        if a <= 0 { if ub0 < INF { act_min[r] = gdn(act_min[r], −(ub0·a)) } else { inf_min[r] −= 1 }
                    if −INF < lb0 { act_max[r] = gup(act_max[r], −(a·lb0)) } else { inf_max[r] −= 1 } }
        else      { if ub0 < INF { act_max[r] = gup(act_max[r], −(ub0·a)) } else { inf_max[r] −= 1 }
                    if −INF < lb0 { act_min[r] = gdn(act_min[r], −(a·lb0)) } else { inf_min[r] −= 1 } }
        act_max[r] = gup(act_max[r], t); act_min[r] = gdn(act_min[r], t) }
    W += cc[j]·7.0·μ
    if rec { cc[j] = −2 } else { cc[j] = 0; 5379f1(tracker, j) }
} else if rec { cc[j] = −2 }
lb[j] = ub[j] = v; return 0
```
gup/gdn은 LP `guarded_add(old, t, up)`이다(m1_presolve.md §5).

`3df916(tol, env, state, j)`:
```
w = ub[j] − lb[j]; if tol + tol < −w { state.0x470 = j; return 3 }
if 1e-13 < w { mx = max |a| over live nodes of col j; W += (열 노드 수)·3.0·μ; if tol < mx·w { return 0 } }
c = obj[j]
v = if tol·0.01 <= |w·c| { c <= 0 ? ub[j] : lb[j] } else if lb[j] > 0 || ub[j] < 0 { (ub[j] + lb[j])·0.5 } else { 0.0 }
mask = 0x88 (+0x248 ≠ 0이면 lb < 0.5일 때 0x800 추가)
if flags[j] & !mask == 0 { (PWL 0x80 분기); ret = 39ab33(v, env, state, j); if ret { return 10001 }; flags[j] = 0 }
objcon = c·v + objcon; obj[j] = 0; (Q 분기)
53802c(tracker, state, j)
for node in col j { if node.col >= 0 { rhs[r] = rhs[r] − v·a; rc[r] −= 1; node.col = −1 } }; W += (열 노드 수)·6.0·μ
cc[j] = (flags[j] == 0) ? −2 : 0          // lb/ub와 activity는 그대로 둔다
```

## 5. `52be10` — 2-행 집합 포장 행의 중복 제거와 삼각형 클리크 병합

한 줄 요약: 계수 ±1인 이진 집합 포장(set packing) 행에서 리터럴 쌍(간선)을 모두 뽑아 리터럴 그래프를 만든다. 리터럴이 2개인 행("2-행")의 간선이 다른 행에도 있으면 그 2-행을 지우고(중복·지배), 2-행 간선으로 이어진 삼각형을 찾으면 탐욕적으로 클리크를 키워 씨앗 2-행을 그 클리크 행으로 늘리고(`<`) 흡수된 2-행을 지운다. 씨앗이 `=` 행이면 늘리지 않고 새 리터럴을 0으로 고정한다. 2-행이 10개(쌍 10개) 미만이면 첫 훑기와 작업량만 남는다(38개 중 37개 모델이 이 경우).

### 5.1 호출, 인자, 가드, 반환

- 호출: `3cd406` L3655, `52be10(state, env, timer)`. 세 번째 인자는 asm 0x3b73f5 `lea 0x640(%rsp),%rdx` = timer `local_658`(디컴파일에서 빠짐). 바깥 가드(PRECLIQUE env+0x254c ≠ 0, `adf236(local_658) < 0.8`)와 PRELIMIT 카운트 +1은 `3cd406` 쪽(§1)에 있다.
- timer: `adf2be(timer, work)` = (timer[0] + *work)/timer[2] ≥ 1(timer[5] ≠ 0이면 *work를 더함, timer[2] ≤ 0이면 각각 참/거짓). 관찰값 timer = [−(진입 work − 1), 0, 예산, 0, 1], 예산은 gmu-35-40 22,107,500, glass4 14,537,500, neos5 15,040,000(값을 정하는 곳은 `3cd406`). 이 함수에서는 528f31 루프에서만 본다.
- 반환: 0, 3(실행 불가, +0x470 = 열), 0x2711(할당 실패, `469c12` 실패 포함), `3ba5ac`/`3ba9de`/`3de18e`의 0이 아닌 반환값. 3을 내는 곳은 `=` 씨앗 행 처리뿐이다.
- 가드: 1.0 + n + n > 2e9이면(가드 덧셈 순서: `1.0 + n`, 그다음 `+ n`) 작업량 없이 0.
- 리터럴 번호: 0..n−1은 x_j(행 계수 +1), n..2n−1은 x̄_j(행 계수 −1, j = lit − n). 계수가 ≤ 0.0이면 x̄.

### 5.2 알고리즘

```rust
// μ = state+0x448, W = *(state+0x458) (null이면 모든 W 갱신 생략), ft = env+0x2460 FeasibilityTol, INF = env+0x2458
fn f52be10(st, env, timer) -> i32 {
    let w_ref = W;                                  // 진입 작업량(1e7 판정 기준)
    let n = st.n /*+0xc*/;
    if 1.0 + n as f64 + n as f64 > 2e9 { return 0 }
    let L: &[i32] = tracker_538f30(st);            // 행 목록, 이 순서로 모든 단계가 돈다
    let limit: i64 = st.nnz /*+0xd8, i64*/ * 2 + 10000;
    let mut hist = [0i64; 201];
    // A. 후보 행 훑기(쌍 개수 히스토그램)
    for &r in L {
        let rc = rowcnt[r] /*+0x60*/;
        if rc < 2 { continue }
        let d = act_min[r];                         // +0x98 = min activity − rhs(유한 부분)
        if !(-ft > d) || inf_min[r] /*+0x78*/ != 0 || inf_max[r] /*+0x70*/ != 0 { continue }
        let (k, nfix, ent) = scan(r, d, 200, None);
        W = (μ * 3.0) * ent as f64 + W;             // 거르기를 통과한 행마다(성공 여부와 무관)
        if nfix + k == rc && k >= 2 { hist[k] += k * (k - 1) / 2 }
    }
    W = L.len() as f64 * μ + W;
    if hist[2] < 10 { return 0 }                    // 2-행이 10개 미만: 여기서 끝(작업량만)
    // 최대 리터럴 수 maxk: 크기 2..maxk 행의 쌍 합이 limit 미만인 가장 큰 크기
    let (mut maxk, mut tot) = (1usize, 0i64);
    loop { let t = tot + hist[maxk + 1]; if limit <= t { break } maxk += 1; tot = t; if maxk >= 200 { break } }
    let mut edges: Vec<(i32, i32)> = with_capacity(tot); let mut tags: Vec<i32> = with_capacity(tot);
    // B. 같은 거르기로 다시 훑어 간선을 만든다(행 순서, 행 안에서는 (0,1),(0,2),…,(0,k−1),(1,2),…)
    for &r in L {
        /* A와 같은 거르기 */
        let mut lits = vec![];
        let (k, nfix, ent) = scan(r, d, maxk, Some(&mut lits));
        W = (μ * 3.0) * ent as f64 + W;
        if nfix + k == rc && k >= 2 {
            let tag = if k == 2 { r } else { -1_999_999_999 };
            for a in 0..k - 1 { for b in a + 1..k {
                edges.push((min(lits[a], lits[b]), max(lits[a], lits[b]))); tags.push(tag);
            } }
        }
    }
    W = (edges.len() as f64 * 10.0 + L.len() as f64 * 5.0) * μ + W;
    if edges.len() <= 2 { return 0 }
    let G = f5279b5(μ, env, st.m, n, &edges, &tags, /*weights*/ None, W)?;   // edges/tags는 여기서 해제
    // C. 리터럴마다 삼각형 클리크
    if 2 < 2 * n {
        for k in 0..2 * n - 2 {
            if adf2be(timer, W) { break }
            if 1e7 < W - w_ref {
                ret = 3ba5ac(env, st, 0, 0, -1); if ret != 0 { return ret }
                let mut stop = 0; ret = 3ba9de(env, st, &mut stop);
                if ret != 0 || st.prelimit /*+0x1dc*/ < 0 { return ret }   // ret이 0일 수 있음(삭제·확장 없이 반환)
                if stop != 0 { break }
                w_ref = W;
            }
            f528f31(μ, env, &mut G, st.buf390 /*state+0x390, i32*/, k, W)?;
        }
    }
    // D. 지배된 2-행 삭제
    let old404 = st.c404; let mut ndel = 0;
    for &r in L {
        if G.rowflag[r] == -1 && sense[r] == '<' {
            ndel += 1; let mut c = 0;
            for node in row_list(r) {                 // 죽은 항목 포함 모두 센다
                c += 1;
                if node.col >= 0 { tracker_5379f1(node.col); colcnt[node.col] -= 1; node.col = -1; }
            }
            W = (c as f64 + c as f64) * μ + W;
            rowcnt[r] = -2;                           // activity·rhs·nnz(+0xd8)는 그대로
        }
    }
    st.c404 = old404 + ndel;
    if (st.m - old404) / 5 < ndel { gc_36e6b6(st, 1) }  // C 정수 나눗셈
    // E. 씨앗 행: 클리크의 셋째 리터럴부터 처리(앞 둘은 씨앗 행 자신의 리터럴)
    for &r in L {
        let id = G.rowflag[r]; if id <= 0 { continue }
        let (a, b) = (G.cstart[id - 1], G.cstart[id]);
        let (mut dpos, mut dneg, mut changed) = (0.0f64, 0.0f64, false);
        let mut p = a + 2;
        while p < b {
            let lit = G.cbuf[p];
            if sense[r] == '=' {                      // x+y=1과 둘 다 충돌하는 리터럴은 0
                if lit < n { let j = lit;
                    if lb[j] > 0.5 { st.c470 = j; return 3 }
                    if !(0.5 > ub[j]) { fix_3de18e(0.0, INF, env, st, j, 1)? }
                } else { let j = lit - n;
                    if 0.5 > ub[j] { st.c470 = j; return 3 }
                    if lb[j] <= 0.5 { fix_3de18e(1.0, INF, env, st, j, 1)? }
                }
            } else {
                let (j, coef) = if lit < n { (lit, 1.0) } else { dneg = dneg - 1.0; dpos = dpos + 1.0; (lit - n, -1.0) };
                if 0.5 > ub[j] {                      // 0으로 고정된 열: 아무것도 안 함
                } else if lb[j] > 0.5 {               // 1로 고정된 열: 항목 없이 rhs로
                    dneg = dneg - coef; dpos = dpos + coef;
                } else {
                    add_entry_469c12(coef, env, st, j, r)?;   // 행·열 목록 머리에 삽입
                    if coef > 0.0 { dpos = 1.0 + dpos }
                    tracker_5379f1(j); changed = true;
                }
            }
            p += 1;
        }
        W = ((p - a) - 2) as f64 * μ + W;             // p = max(b, a+2)
        if sense[r] == '<' {
            act_max[r] = dpos + act_max[r];           // +0x90 (max activity − rhs)
            rhs[r] = dneg + rhs[r];                   // +0xb8
            if changed || dneg.abs() > 1e-10 { tracker_53763f(r) }
            // act_min(+0x98)은 고치지 않는다(1 고정 리터럴이면 min−rhs가 어긋남, Gurobi 그대로)
        }
    }
    W = (L.len() as f64 + L.len() as f64) * μ + W;
    0
}

// 행 하나 훑기. 반환 (자유 이진 수 k, 고정 이진 수 nfix, 지나간 노드 수 ent)
fn scan(r, d, kmax, out) -> (usize, i32, i32) {
    for node in row_list(r) {                         // 연결 리스트 순서, 중간 break
        ent += 1;
        let j = node.col; if j < 0 { continue }
        if vtype[j] != 'B' { break }
        if lb[j] > 0.5 || ub[j] < 0.5 { nfix += 1; continue }
        if k >= kmax { break }                        // asm: k ≥ 200 판정이 계수 판정보다 먼저
        let a = node.val;
        if (a.abs() - 1.0).abs() > 1e-10 || a.abs() + d < -1e-10 { break }
        if let Some(v) = out { v.push(if a <= 0.0 { j + n } else { j }) }
        k += 1;
    }
}
```

break가 일어나면 그 노드와 뒤 노드가 세지지 않으므로 `nfix + k == rc`가 거짓이 되어 행이 빠진다. `|a| + d ≥ −1e-10`(d = min act − rhs)은 "리터럴 하나가 켜지면 행이 꽉 찬다"는 뜻이라 두 리터럴은 동시에 켜질 수 없다.

#### `5279b5(μ, env, &G, m, n, ne, edges, tags, weights=NULL, W)` — 리터럴 그래프

G는 `ac64e6(1, 0x60)`로 만든 슬롯 12개(i64/포인터):

| 슬롯 | 내용 |
|---|---|
| [0] | `start: i64[2n+1]` 리터럴별 인접 구간(위쪽 삼각형: 리터럴 k에는 k보다 큰 이웃만) |
| [1] | `adj: i32[ne]` 이웃 리터럴 |
| [2] | `tag: i32[ne]` 간선 꼬리표. r ≥ 0: 2-행 r의 간선, −1999999999: 3개 이상 행의 간선, −1−r: 씨앗으로 쓴 간선, −2000000000: 클리크에 흡수된 간선 |
| [3] | 가중치 f64[ne] (weights가 있을 때만, 여기서는 NULL) |
| [4] | `mark: i64[2n]`(calloc) 528f31의 표시 |
| [5] | `cnt: i32[2n]` 528f31의 후보 점수 |
| [6] | `rowflag: i64[m]`(calloc) 0 = 그대로, −1 = 지배됨(삭제 대상), c ≥ 1 = 클리크 c−1의 씨앗 행 |
| [7] | `cstart: i64[m+1]`, [7][0] = 0 |
| [8] | 클리크 가중치(weights가 있을 때만) |
| [9] | `cbuf: i32` 클리크 리터럴(처음 malloc 0x1000) |
| [10] | 클리크 수 ncl(처음 0) |
| [11] | [9]의 용량(처음 0x400) |

```rust
fn f5279b5(μ, m, n, edges, tags, W) -> G {
    let N2 = 2 * n; let ne = tags.len();
    start[..] = 0;
    for e in 0..ne { start[edges[e].0 + 2] += 1 }
    W = (ne as f64 + ne as f64) * μ + W;
    if N2 > 2 { for t in 3..=N2 { start[t] += start[t - 1] }  W = W + μ * 2.0 * (N2 - 2) as f64; }
    for e in 0..ne { let p = start[edges[e].0 + 1]; adj[p] = edges[e].1; tag[p] = tags[e]; start[edges[e].0 + 1] = p + 1; }
    W = W + μ * 7.0 * ne as f64;
    for k in 0..N2 { sort_a9dcac(&mut adj[start[k]..start[k+1]], &mut tag[start[k]..start[k+1]]) }
    W = W + (N2 as f64 + start[N2] as f64 * 2.0) * μ;
    // 같은 이웃의 중복 간선 제거: 2-행 간선이 다른 간선과 겹치면 그 2-행을 지배됨으로
    let mut w = 0;
    for k in 0..N2 {
        let (a, b) = (start[k], start[k + 1]); start[k] = w;
        for t in a..b {
            if t == a || adj[w - 1] < adj[t] { adj[w] = adj[t]; tag[w] = tag[t]; w += 1; }
            else {
                let pt = tag[w - 1];
                if pt < 0 { if tag[t] >= 0 { rowflag[tag[t]] = -1 } }
                else { rowflag[pt] = -1; tag[w - 1] = tag[t]; }    // 남는 꼬리표는 뒤의 것
            }
        }
    }
    W = W + μ * (start[N2] as f64 * 2.0 + N2 as f64);       // start[N2]는 아직 ne
    start[N2] = w;
}
```

`a9dcac`는 `aaedd9`(LP 포트 `crates/lp/src/presolve/sort.rs`)와 같은 알고리즘의 i32 키·i32 동반값 판이다(길이 ≥ 100이면 median-of-3 3-way quicksort, 깊이 50, 그 밑은 Shell 정렬 n/2, ÷2.2, 2 다음 1). 같은 이웃이 여러 번 나올 때 어느 2-행이 −1이 되는지가 이 동률 순서로 정해진다. gmu-35-40(인접 구간 100 이상 포함)에서 정렬 뒤 결과가 Gurobi와 같았다.

#### `528f31(μ, env, &G, buf = state+0x390, k, W)` — 리터럴 k에서 삼각형 찾고 키우기

```rust
fn f528f31(μ, G, buf: &mut [i32], k, W) -> i32 {
    let (s0, s1) = (start[k], start[k + 1]);
    for e in s0..s1 { mark[adj[e]] = e + 1 }                // k의 이웃: 위치+1
    W = ((s1 - s0) as f64 * μ) * 2.0 + W;
    let d2 = μ * 2.0;
    for e in s0..s1 {
        if tag[e] < 0 { continue }                          // 아직 안 쓴 2-행 간선만 씨앗
        let r = tag[e]; let i = adj[e];
        // i의 이웃 l 중 k의 이웃이기도 하고 (k,l) 또는 (i,l)이 2-행 간선인 첫 l
        let mut f = start[i]; let mut p = -1;
        while f < start[i + 1] { p = mark[adj[f]] - 1; if p >= 0 && (tag[p] >= 0 || tag[f] >= 0) { break } f += 1; }
        W = (f - start[i]) as f64 * d2 + W;                 // 찾은 위치는 세지 않는다
        if f == start[i + 1] { continue }
        let l = adj[f];
        G.rowflag[r] = G.ncl + 1; tag[e] = -1 - r;
        if tag[p] >= 0 { G.rowflag[tag[p]] = -1; tag[p] = -2_000_000_000 }
        if tag[f] >= 0 { G.rowflag[tag[f]] = -1; tag[f] = -2_000_000_000 }
        buf[0..3] = [k, i, l]; let mut size = 3; let mut cand = false;
        for f2 in f + 1..start[i + 1] {                     // k, i 둘 다의 이웃 = 후보
            let p2 = mark[adj[f2]] - 1;
            if p2 >= 0 { mark[adj[f2]] = -2; cnt[adj[f2]] = if tag[p2] < 0 { 0 } else { 1 }; cand = true; }
        }
        W = W + ((start[i + 1] - f) as f64 * μ) * 3.0;
        if cand {
            loop {                                          // 마지막 멤버의 이웃 중 모든 멤버와 인접한 후보
                let v = buf[size - 1]; let (mut best, mut bc) = (-1, 0);
                for g in start[v]..start[v + 1] {
                    let u = adj[g];
                    if mark[u] == 1 - size as i64 {
                        mark[u] = -(size as i64);
                        if tag[g] >= 0 { cnt[u] += 1 }
                        if bc < cnt[u] { best = u; bc = cnt[u] }   // 2-행 간선 점수 최대, 첫 것
                    }
                }
                W = (start[v + 1] - start[v]) as f64 * d2 + W;
                if best < 0 { break }
                buf[size] = best; size += 1;
            }
            if size > 3 {                                   // 새 멤버 사이·새 멤버로 가는 2-행 간선 흡수
                for q in &buf[3..size] { mark[*q] = -2_000_000_000 }
                W = (size - 3) as f64 * d2 + W;
                for q in &buf[0..size] {
                    for g in start[q]..start[q + 1] {
                        if mark[adj[g]] == -2_000_000_000 && tag[g] >= 0 { G.rowflag[tag[g]] = -1; tag[g] = -2_000_000_000 }
                    }
                    W = (start[q + 1] - start[q]) as f64 * d2 + W;
                }
                W = W + (size as f64 + size as f64) * μ;
            }
        }
        // 저장: 용량이 모자라면 max(필요, 2·용량)으로 realloc(실패 0x2711)
        cbuf[cstart[ncl]..][..size] = buf[..size]; cstart[ncl + 1] = cstart[ncl] + size; ncl += 1;
        if cand {                                           // 지워진 k 이웃 표시 복구
            for e2 in e + 1..s1 { mark[adj[e2]] = e2 + 1 }
            W = ((-1 - e) + s1) as f64 * d2 + W;
        }
    }
    W = W + μ * 4.0 * (s1 - s0) as f64;
    for e in s0..s1 { mark[adj[e]] = 0 }
    W = W + μ * 2.0 * (s1 - s0) as f64;
    0
}
```

멤버는 k < i < l < … 오름차순이다(인접이 위쪽 삼각형이라). `cnt`는 지우지 않는다(후보로 표시할 때 다시 채운다). `state+0x390`은 클리크 멤버를 담는 i32 작업 버퍼로만 쓴다(크기 ≤ 클리크 크기).

#### 보조

- `469c12(coef=xmm0, env, state, j, r, NULL, NULL, &err)`: 노드 풀 free 목록(+0x1a0 머리, +0x198 개수)에서 노드를 꺼낸다. 비었으면 `3b85ea(env, state, max(3000, m))`로 늘리고, 실패하면 err = 1(→ 0x2711). nnz(+0xd8) += 1, 노드 = {값 coef, 행 r(+8), 열 j(+0xc)}, 열 j 목록(+0x180)과 행 r 목록(+0x178)의 **머리**에 넣고 colcnt[j]++, rowcnt[r]++. activity·작업량은 건드리지 않는다. 기본값 MIPLIB 38개에서는 불리지 않는다(합성 모델에서 확인).
- `36e6b6(state, 1)`(GC, 내부 루프 노트 담당): 대기 목록(+0x190, 개수 +0x188)을 free 목록에 잇고(작업량 (길이−1)·5·μ), 모든 열 목록에서 죽은 노드(열 < 0)를 떼어 free 목록에 넣고(+0x198++, nnz−−), 모든 행 목록에서도 죽은 노드를 뗀다. 작업량은 비지 않은 목록마다 `(노드 수·μ)·8.0`, 열 다 돈 뒤 `(n·3.0)·μ`, 행 다 돈 뒤 `(m·3.0)·μ`. 끝에 nnz = 살아 있는 노드 수. 52be10은 이번에 지운 행 수가 (m − 진입 +0x404)/5보다 많을 때만 부른다(합성 모델에서 확인. MIPLIB 기본값에서는 안 불림).
- `3de18e(값, INF, env, state, j, 1)`: §4의 열 고정(type 1 레코드 {j, 값}, +0x408++). `=` 씨앗 행에서만 부른다.

### 5.3 기본값에서 하는 일

- MIPLIB 38개 중 히스토그램 경로(hist[2] ≥ 10)를 타는 것은 gmu-35-40의 첫 호출뿐이다(`5279b5` 1회, `528f31` 2·1205−2 = 2408회). 나머지는 A 훑기 작업량만 더한다.
- gmu-35-40 #1: 2-행 21개, 간선 15976개(maxk까지), 중복 제거 뒤 8659개. 삼각형은 하나도 없다(ncl = 0). 지운 `<` 2-행 17개(153, 176, 177, 188, 192, 204, 218, 219, 220, 225, 237, 284, 285, 304, 305, 319, 334)는 모두 두 리터럴이 더 큰 집합 포장/분할 행(크기 3–18)에도 함께 있는 행이다(5279b5 중복 제거). +0x404 += 17, 레코드·고정 없음, GC 안 불림.

### 5.4 오라클 확인

Python 모델(`$S/clq/f52/m52.py`)을 `mip_state_dump.js`의 52be10 진입 상태와 Frida 탐침(`$S/clq/f52/ag_52.js`: 538f30 목록, 5279b5 입출력, 마지막 528f31 뒤 G, timer)에 맞췄다. 작업량은 진입값부터 순서대로 더해 비트 단위로 비교했다.

| 모델(호출) | 경로 | 일치한 것 |
|---|---|---|
| glass4 #1, neos5 #1 | A 훑기만 | 작업량 비트 일치 |
| gmu-35-40 #1 | A, B, 5279b5, 528f31×2408, D | 간선·꼬리표, 5279b5 출력(start/adj/tag/rowflag)과 작업량, 528f31 뒤 G(ncl 0)와 작업량, 삭제 행 17개, 복귀 상태(행·열 리스트 순서, rowcnt, colcnt, rhs, activity, nnz), 최종 작업량 |
| 합성 `syn.lp`(K4 6묶음 = 2-행 36개) | 삼각형 + 확장(크기 4), 469c12 12회, 삭제 30, GC | cbuf/cstart/rowflag, 머리 삽입 순서까지 리스트, act_max·rhs, nnz, 작업량 전부 |
| 합성 `syn2.lp` 기본값 | x̄ 리터럴 클리크(x − z ≤ 0 꼴) | rhs −1, act_max, 확장, 표시, 작업량 |
| 합성 `syn2.lp` Aggregate=0 | `=` 씨앗(x + y = 1) | z 3개를 `3de18e(0.0)`로 고정(type 1 레코드 {27, 0.0}, {30, 0.0}, {33, 0.0}, +0x408 += 3), objcon, 나머지 상태, 작업량 |

합성 모델은 `$S/clq/f52/syn.lp`, `syn2.lp`에 있고, 경로를 받는 도구 판은 `$S/clq/f52/probe_path.js`, `dump_path.js`다(worktree 도구는 그대로 둠).

### 5.5 남은 질문

- 확인하지 못한 경로: timer 예산 정지(adf2be)와 1e7 콜백 정지, 2e9 가드, 할당 실패, `=` 씨앗의 실행 불가(3), `<` 확장 중 1로 고정된 리터럴(dneg −= coef, dpos += coef), `469c12`의 노드 풀 확장(`3b85ea`), maxk < 200으로 잘리는 경우, 36e6b6의 대기 목록 잇기.
- `<` 확장 뒤 act_min(+0x98)을 고치지 않는 것은 Gurobi 동작 그대로다(1 고정 리터럴이 있으면 min−rhs가 실제와 1 어긋남). 뒤 단계가 그 행 activity를 다시 계산하는지는 확인하지 않았다.
- `state+0x390` 버퍼 용량(클리크 최대 크기 ≤ n이면 충분)과 timer 예산을 정하는 식은 `3cd406`/`3a484d` 쪽 해독에 맡긴다.

## 6. `46a62d` — clique 행 정규화·병합·확장 (20,983 B)

요약: 계수 크기가 모두 같은 이진 행(set packing/partitioning 꼴, 보수 리터럴 허용)을 ±1 꼴로 정규화하고, 그 둘레 행을 분류한 뒤 짧은 clique 행마다 (a) 다른 clique 행에 통째로 들어 있으면 지우고, (b) 아니면 행의 모든 리터럴과 충돌하는 리터럴을 찾아 행에 붙이고(확장), (c) 확장된 행에 통째로 들어가는 clique 행을 지운다. `=` 행이면 (b)의 리터럴을 0으로 고정한다(기본값 38개에서는 안 돎).

### 6.1 호출, 인자, 가드, 반환

- `3cd406` L3689(asm 0x3b74e0–0x3b7510): `46a62d(state, env, timer = &local_618, mode)`. Ghidra는 3·4번째 인자를 빠뜨린다. `mode`(ecx) = 2 if (`3cd406`의 param_3(mipctx) ≠ 0 || Presolve(env+0x2424) > 1 || GURO_PAR_PRECLIQUE(env+0x254c) > 1) else 1. Model.presolve 기본값에서는 **1**(오라클 25회 전부 mode 1).
- 호출 지점 가드: PRECLIQUE ≠ 0 블록 안(45885d → 52be10 → 46a62d 순), `adf236(local_618) < 0.8`, `ade8ca`/`ade770`로 타이머를 감싼다. 호출 뒤 PRELIMIT 카운트 +1, `3ba5ac`/`3ba9de`.
- 반환: 0(정상), 0x2711(할당 실패), 3(고정 경로에서 한 열에 x=0과 x=1이 다 요구됨, +0x470 = 열), `3de18e`/`3df916`/`3b85ea`/`3ba5ac`/`3ba9de`의 0 아닌 값. `2.0e9 < (double)n + (double)n`이면 아무것도 안 하고 0.
- **예산 B**(본 루프 직전에 한 번): `B = (t[2] − elapsed)·0.01 + 100000.0`, elapsed = `adf00e(t, W)` = t[0] + W(t+0x28 실행 플래그가 켜져 있으면). `0.0 ≥ B`이면 본 루프와 마무리를 건너뛰고 0을 돌려준다(+0x3fc도 안 더함).
  - t는 `local_618`(46a62d 전용 타이머). `ade8ca`: t[0] = (t[0] − W) + 1.0, 플래그 1. `ade770`: t[0] = t[0] + W. 그래서 elapsed = (이전 46a62d 호출들의 ΔW 합 + 호출 횟수) + 이번 호출의 지금까지 ΔW.
  - t[2]는 `3cd406` L1585(`adf1a6(local_c98, local_618)`, c98 = (c90 + 1e7)·local_930)가 정한다(spine 몫). 측정값은 t[2] = 1250·nnz + 5e6꼴(glass4 7,268,750, gmu-35-40 11,053,750, gmu-35-50 15,803,750, neos-860300 485,411,250). 오라클의 B는 172k–255k.
- `adf2be(t, W)`(본 루프의 행마다): t[2] > 0이고 (t[0]+W)/t[2] ≥ 1이면 참(t[2] < 0이면 늘 참) → 본 루프를 끝내고 마무리로. 기본값에서는 거짓.
- 1e7 규칙(본 루프의 행마다): `W − W_last > 1e7`이면 `3ba5ac(env, st, 0, 0, −1)`, `3ba9de(env, st, &stop)`. 0 아닌 반환은 그대로 반환, stop ≠ 0이면 마무리로, `st+0x1dc < 0`이면 0 반환(마무리 없음). 그다음 W_last = W. W_last 초기값은 함수 진입 때 W. 상태는 안 바뀐다.

### 6.2 자료

| 이름(Ghidra) | 크기 | 내용 |
|---|---|---|
| `cls`(local_50) | m, calloc | 0 미분류, 1 제외, 2 `<` clique, 3 `=` clique. 비트 4는 "이번 행 동안 제외"로 쓰려던 것 같지만 **켜는 곳이 `|= 1`이다**(아래 확장). 끝에 `&= ~4`. 그대로 옮긴다 |
| `f0`(local_f0) | m, memset −1 | 정규화한 clique 행 0. 확장 행 고르기의 카운터. 지운/제외한 행 −1 |
| `bit`(local_120) | 2n, calloc | 단계 2에서 앞 n칸은 열 표시. 본 루프에서는 리터럴 → 현재 행 r 안의 비트. 리터럴: 계수 > 0인 열 j는 j, 아니면 n + j(보수) |
| `mask`(local_128) | 2n, calloc | 리터럴이 충돌하는 r 리터럴 비트의 OR |
| `Lc`(r12) | 2n int | 후보 리터럴 목록, 길이 lc(0xbc) |
| `S`(local_130) | m int | r을 품은(superset) clique 행 후보 |
| `rowList` = state+0x390, `cnt90` = state+0x3a8 | m | 행 목록·행 카운터. 진입·퇴장 때 0이어야 한다 |
| `fixflag`(local_e8), `fixlist` = state+0x3b8 | n | `=` 행 고정(1: x=0, 2: x=1) |
| `wacc`(local_108) | m double, calloc | 행 i2를 훑는 데 쓴 작업량 누적(예산 B와 비교) |
| `f8`(local_f8), `best`(local_100) | m int | 확장 행 고르기 목록, 이번 행에서 고른 행 목록 |
| state+0x1b0 | 포인터 배열 | 단계 3 정렬 scratch(쓰고 0으로 지움) |

리스트 연산은 [m1_presolve_agg.md](../m1/m1_presolve_agg.md) §2.3 어휘를 쓴다: "행 압축" = 죽은 노드를 행 리스트에서 떼기(개수 변화 없음), "열 압축" = 열 리스트의 죽은 노드를 pending으로(+0x188++, +0xd8−−), "새 노드" = +0x1a0 free 목록에서(+0x198−−, +0xd8++). 이 함수의 새 노드는 **열 리스트 맨 앞, 행 리스트 맨 끝**에 붙는다(행 r의 마지막으로 훑은 노드 뒤). free가 0이면 `3b85ea(env, st, max(3000, m))`(기본값에서는 안 돎).

### 6.3 알고리즘

작업량 표기: `W += (k·c)·μ`는 Gurobi 순서 그대로 `((double)k·c)·μ + W`. c가 2의 거듭제곱(0.5, 2, 4, 8)이면 순서가 결과에 영향이 없고, 3·5·6·11은 이 순서를 지켜야 한다. `2k`는 `(double)k + (double)k`. μ = st+0x448, ft = FeasibilityTol(env+0x2460), ft100 = ft·100.0.

```
fn clique_merge(st, env, t, mode) -> i32 {
  if 2e9 < n as f64 + n as f64 { return 0 }
  (cap, L) = if mode == 2 { (3200, m) }
             else { x = min(m, max(n, 1_000_000)); (32, max(32, 4 * (pow(x as f64, 0.4) as i32))) }
  list = 538f30(tracker)                       // 현재 레벨 행 목록(m1_presolve §5)
  // 단계 1: 후보 행
  for i in list {
    if !(row_cnt[i] >= 2 && !(act_min[i] > -ft)) { continue }     // act_min = min activity − rhs(유한부)
    k = 0; ok = true; p = 행 압축한 머리
    if p == None { k = 1 }                                         // (일어나지 않음)
    else loop {
      j = p.col
      if ub[j] == lb[j] { 3de18e(env, st, j, 1, x=lb[j], INF); if row_cnt[i] < 2 { ok = false; break } }
      else if L < col_cnt[j] || vtype[j] != 'B' || 1e-10 < |(|p.val| + act_min[i])| { ok = false; break }
      q = 다음 살아 있는 노드(행 압축); k += 1; if q == None { break }; p = q
    }
    W += (μ·8)·k
    if ok { kept_all.push(i) }
  }
  W += (len(list)·5)·μ
  if kept_all 비었음 { return 0 }
  for i in kept_all { cls[i] = if sense[i]=='<' {2} else {3}; f0[i] = 0; normalize(i)
                      if row_cnt[i] <= cap { kept.push(i) } }
  W += (len(kept_all)·6)·μ
```
`normalize(i)`: 살아 있는 노드를 행 압축하며 val ≤ 0이면 −1.0(neg += 1), 아니면 1.0. W += (2k)·μ(k = 노드 수). rhs[i] = 1.0 − neg, act_min[i] = −1.0, act_max[i] = (double)row_cnt[i] − 1.0. inf 개수는 안 건드린다. 계수를 바꾸지만 +0x3fc는 안 센다.

```
  // 단계 2: kept 행의 2-hop 이웃 분류 (colList, rowList는 진입 때 비어 있음)
  for r in kept {
    p = 행 압축한 머리; t = 0
    if p == None { W += (t·4)·μ; continue }
    loop {
      if len(colList) >= n || len(rowList) >= m { break }          // 이때는 t에 +1 없음
      j = p.col
      if bit[j] == 0 {
        bit[j] = 1; colList.push(j); q = 열 압축한 머리; c = 0
        while q != None {
          if m <= len(rowList) { break }
          i = q.row
          if cnt90[i] == 0 {                                       // state+0x3a8를 표시로 씀
            cnt90[i] = 1; rowList.push(i)
            if row_cnt[i] > 0 && cls[i] == 0 {
              ok = row_cnt[i] >= 2 && !(1e-9 >= |act_min[i]|)
              for x in 살아 있는 노드(행 압축) {
                if ub==lb { 3de18e(...) }                          // ok는 그대로
                else if !(ok && vtype=='B' && |(|x.val| + act_min[i])| < 1e-10) { ok = false }
              }
              W += (노드수·8)·μ
              if row_cnt[i] > 1 && ok { cls[i] = 2/3; f0[i] = 0; normalize(i) }
            }
          }
          q = 다음(열 압축); c += 1
        }
        W += (c·6)·μ
      }
      q = 다음(행 압축); t += 1; if q == None { break }; p = q
    }
    W += (t·4)·μ
  }
  W += (2·len(kept))·μ
  // 단계 3: 표시 지우기, 분류, 행 재배열
  if len(colList) > n/4 (C 정수 나눗셈) { bit[0..n] = 0; W += (0.5·(double)n)·μ }
  else { colList의 bit = 0; W += (2·len(colList))·μ }
  for i in rowList {
    cnt90[i] = 0
    if row_cnt[i] < 2 || (inf_min[i] != 0 && (inf_max[i] != 0 || sense[i]=='<')) { cls[i] = 1; continue }
    // 행 리스트를 다시 엮는다: 죽은 노드는 빠진다
    e = 0; arr = []; nonB = []
    for p in 행 리스트 전부(죽은 것 포함) { e += 1
      if p.col >= 0 { if vtype[p.col]=='B' { arr.push(p); if 0 > p.val { p.col = −p.col−1; p.val = −p.val } }
                      else { nonB.push(p) } } }
    W += (e·5)·μ
    rowhead[i] = nonB를 원래 순서로 이은 것(없으면 None)
    if arr != [] {
      shell_sort_desc(arr, key = val)                              // 아래
      arr를 이 순서로 잇고 보수 복원(col<0이면 col = −col−1, val = −val); W += (len·3)·μ
      arr.last.next = rowhead[i]; rowhead[i] = arr[0]; scratch 0; W += (len·0.5)·μ
    }
    p = rowhead[i]; q = p.next
    if vtype[q.col] != 'B' { cls[i] = 1 }
    else { s = |p.val| + |q.val|
           if ft100 > act_min[i] + s && (sense[i]=='<' || ft100 > s − act_max[i]) { cls[i] = 1 } }
    // 통과하면 cls는 그대로(0 또는 2/3)
  }
  W += (len(rowList)·11)·μ
```
`shell_sort_desc`: h = len/2(정수). h > 0인 동안: j = h..len−1마다 key = arr[j], t = j, sh = 0; t ≥ h이고 key.val > arr[t−h].val이면 밀고 t −= h, sh += 1; arr[t] = key; W += sh·(4·μ). 한 바퀴 끝에 W += (len − h)·(2·μ)(h ≥ len이면 0). h = (h == 2) ? 1 : (int)((double)h/2.2). 결과: **B 노드가 |a| 내림차순(같은 값의 순서는 이 Shell 정렬이 정함), 그 뒤 비-B 노드가 원래 순서**. 이 순서는 그대로 남아 출력 행 순서를 바꾼다.

```
  // 예산
  B = (t[2] − (t[0] + W))·0.01 + 100000.0
  if 0.0 >= B { return 0 }
  // 단계 4: 본 루프
  it = 0; added_total = 0
  for r in kept {
    (adf2be, 1e7 규칙: 6.1)
    if (cls[r] & !1) == 2 { added_total += process_row(r) }
    it += 1
  }
  W += (it·3)·μ
  // 단계 5: = 행 고정(기본값 38개에서는 fixlist가 늘 비어 있다)
  rows = []
  for j in fixlist { 열 압축하며 살아 있는 노드의 행을 rows에 모음(cnt90 표시); W += (노드수·3)·μ
                     match fixflag[j] { 3 => { +0x470 = j; return 3 }, 1 => ub[j] = 0.0, _ => lb[j] = 1.0 }
                     3df916(ft, env, st, j)?; fixflag[j] = 0 }
  W += (len(fixlist)·4)·μ; rows의 cnt90 = 0; W += (2·len(rows))·μ
  if rows != [] { 370047(st, env, len(rows), rows) }
  st+0x3fc += added_total
  return 0
}
```

`process_row(r)` (asm 0x41557e–0x4166ad). 변수: mask = 0, bit = 1, cf = 0(청크 번호가 0이면 0, 이후 1), esi = 0, S = [], Lc/lc, added = 0, best = [], p = rowhead[r](압축 없음), start = p, tail.
```
  loop {                                                    // 청크 하나
    edi = 0; rowList = []                                   // rowList 길이(스택 rsp+0x250) = 0
    loop { v = p.val; c = p.col; esi += 1
           lit = if v > 0 { c } else { n + c }
           mask |= bit; edi += 1; bit[lit] = bit; bit <<= 1 (u32)
           q = 열 압축한 머리(c); k = 0
           while q != None { i2 = q.row
             if i2 != r && cls[i2] != 1 && cls[i2] & 4 == 0 && (0 < q.val) == (0 < v) {
               if cnt90[i2] == 0 { rowList.push(i2) }; cnt90[i2] += 1 }
             q = 다음(열 압축); k += 1 }
           W += (k·5)·μ
           tail = p; p = p.next                             // 죽은 노드 검사 없음
           if p == None || esi >= 32 { break } }
    W += ((esi − cf)·4)·μ
    if cf == 0 { for i2 in rowList { if cnt90[i2] == edi && (cls[i2] & !1) == 2 { S.push(i2) }; cnt90[i2] = 0 }
                 W += (2·len(rowList))·μ }
    else { k = len(S); if k > 0 { S.retain(|i2| cnt90[i2] == edi) }; W += (2k)·μ
           rowList의 cnt90 = 0; W += (2·len(rowList))·μ }
    if p == None && S != [] {                               // r ⊆ 다른 clique 행: r을 없앤다
      r의 노드 전부(머리부터, 검사 없음): bit[c] = bit[n+c] = 0; W += (노드수·4)·μ
      if sense[r] == '<' { 노드마다 5379f1(c), col_cnt[c] −= 1, col = −1; W += (노드수·3)·μ
                           row_cnt[r] = −2; st+0x404 += 1 }          // 행 리스트·머리는 그대로
      cls[r] = 1; f0[r] = −1
      if cf != 0 { Lc[..lc]의 mask = 0; W += (2·lc)·μ }
      break
    }
    // 이 청크와 충돌하는 리터럴
    first = cf == 0; if first { Lc = []; lc = 0 }
    for i2 in rowList {
      d = wacc[i2]; if d > B { continue }
      wacc[i2] = d − W
      if (cls[i2] & !1) == 2 {                             // clique 행: i2의 리터럴은 i2 안의 r 리터럴과 모두 충돌
        bs = OR(살아 있는 노드(행 압축): v > 0 ? bit[c] : bit[n+c]); W += (노드수·4)·μ
        for x in 행 i2(머리부터) { l = 리터럴(x); u = mask[l]
          if first { if u == 0 { Lc.push(l); lc += 1 }; mask[l] = u | bs } else if u != 0 { mask[l] = u | bs } }
        W += (노드수·5)·μ
      } else {
        if inf_min[i2] == 0 { 4694fe(st, i2, bit, mask, Lc, first, s = 1.0, d = act_min[i2] − ft100) }
        if sense[i2] == '=' && inf_max[i2] == 0 { 4694fe(..., s = −1.0, d = (−act_max[i2]) − ft100) }
      }
      wacc[i2] = W + wacc[i2]
    }
    W += (len(rowList)·3)·μ
    k = lc
    if k > 0 { 제자리 압축: c = l mod n; bit[c] != 0 || bit[n+c] != 0 || mask[l] != mask이면 mask[l] = 0(버림), 아니면 남김; lc = 남은 수 }
    W += (k·5)·μ
    if p == None {
      if end_of_row() == Continue(p') { p = p' } else {   // 끝
        start부터 행 끝까지 bit[c] = bit[n+c] = 0; W += ((esi − cf)·4)·μ; break }
    }
    // 다음 청크(행이 32개보다 길 때, 또는 확장으로 붙인 노드를 이어서 볼 때)
    Lc[..lc]의 mask = 1; W += (2·lc)·μ
    mask = 1; bit = 2; cf = 1
    start..p(제외)의 bit[c] = bit[n+c] = 0; W += ((esi − 1)·4)·μ     // 첫 청크 32개면 31·4
    if lc == 0 { break }
    esi = 1; start = p
  }
  for b in best { cls[b] &= !4 }                           // 효과 없음(위 |= 1 참고)
  return added
```
`end_of_row()` (asm 0x4166d2–0x417f53):
```
  if sense[r] == '=' {                                      // Lc의 리터럴은 모두 0이어야 한다
    for l in Lc[..lc] { mask[l] = 0; c = l mod n; if fixflag[c] == 0 { fixlist.push(c) }
                        fixflag[c] |= if l >= n {2} else {1} }
    W += (lc·4)·μ; return Done }
  loop {
    if lc <= 0 { break }
    saved = added; t = min(lc, 10); f8 = []
    for l in Lc[..t] { c = l mod n; q = 열 압축한 머리(c); e = 0
      while q != None { i2 = q.row
        if f0[i2] >= 0 && ((q.val > 0 && l < n) || (!(q.val > 0) && (!(0 > q.val) || l >= n))) {
          if f0[i2] == 0 { f8.push(i2) }; f0[i2] += 1 }
        q = 다음(열 압축); e += 1 }
      W += (e·5)·μ }
    W += (2t)·μ
    bestrow = −1; bc = 0
    for i2 in f8 { x = f0[i2]; f0[i2] = 0; if x > bc { bc = x; bestrow = i2 } }   // 처음 나온 최대
    W += (2·len(f8))·μ
    newfirst = None
    if bestrow >= 0 {                                       // 기본값에서 확장은 전부 이 경로
      rd = 0.0; md = 0.0; e = 0
      for x in 살아 있는 노드(bestrow, 행 압축) { e += 1
        if x.val > 0 && mask[x.col] == mask { add(x.col, +1.0) }
        else if 0 > x.val && mask[n + x.col] == mask { add(x.col, −1.0); rd = rd − 1.0 } else { continue }
        row_cnt[r] += 1; added += 1; 5379f1(x.col); newfirst.get_or_insert(새 노드); md = md + 1.0 }
      W += (e·3)·μ
      act_max[r] = md + act_max[r]; rhs[r] = rd + rhs[r]; 53763f(r)
      cls[bestrow] |= 1; best.push(bestrow)
    } else {                                                // 후보 전부를 붙인다(기본값에서 안 돎)
      for l in Lc[..lc] { (c, v) = if l < n {(l, 1.0)} else {(l − n, −1.0)}
        add(c, v)(할당 실패면 건너뜀); row_cnt[r] += 1; added += 1; 53763f(r); 5379f1(c)
        act_max[r] = act_max[r] + 1.0; if 0 > v { rhs[r] = rhs[r] − 1.0 } }
      W += (2·lc)·μ
    }
    if added − saved < lc {
      if newfirst != None { return Continue(newfirst) }    // 붙인 노드를 다음 청크로 본다
      break                                                 // (mask에 lc개가 남는다. Gurobi 그대로)
    }
    Lc[..lc]의 mask = 0; W += (2·lc)·μ; break
  }
  if added > 0 {                                            // 확장된 r에 들어가는 행
    rowList = []; er = 0
    for x in 행 r(머리부터, 붙인 노드 포함) { q = 열 압축한 머리(x.col); e = 0
      while q != None { if q.row != r && (0 < q.val) == (0 < x.val) {
                          if cnt90[q.row] == 0 { rowList.push(q.row) }; cnt90[q.row] += 1 }
                        q = 다음(열 압축); e += 1 }
      W += (e·4)·μ; er += 1 }
    W += (er·3)·μ
    for i2 in rowList {
      if cnt90[i2] == row_cnt[i2] {
        if cls[i2] == 2 { for y in 살아 있는 노드(i2, 행 압축) { 5379f1(y.col); col_cnt[y.col] −= 1; y.col = −1 }
                          W += (2·노드수)·μ; row_cnt[i2] = −2; st+0x404 += 1 }
        cls[i2] = 1; f0[i2] = −1 }                          // cls 0·3인 행은 제외만
      cnt90[i2] = 0 }
    W += (len(rowList)·4)·μ
  }
  return Done
```
`add(c, v)`: 새 노드 {val v, row r, col c}를 열 c 머리에 넣고 tail 뒤에 붙인다(next = None, tail = 새 노드). col_cnt[c] += 1, +0xd8 += 1, +0x198 −= 1.

- `5379f1(tr, j)` / `53763f(tr, i)`는 추적기 열·행 표시(m1_presolve §5). 순서: 확장 노드마다 5379f1, 확장 한 번 끝에 53763f(r). 전부 붙이기 경로는 노드마다 53763f(r) 다음 5379f1(c).
- 레코드는 하나도 쌓지 않는다(행 삭제·계수 정규화·확장 모두 MIP에서 postsolve가 필요 없는 변형). 카운터: +0x404(지운 행), +0x3fc(붙인 비영 수). bound는 고정 경로에서만 바뀐다.

### 6.4 `4694fe` — 일반 행에서 함의되는 충돌 (750 B, 기본값에서 4개 모델)

`4694fe(st, i2, bit, mask, list, cntp, s(xmm0), d(xmm1))`. cntp = NULL이면 "이후 청크" 모드(새 후보 없이 0 아닌 mask에만 OR). 행 i2는 단계 3에서 B 노드가 |a| 내림차순으로 앞에 와 있다.
```
  acc = 0; masks = []; thr = []; cnt1 = 0
  p = 행 압축한 머리
  while p != None && vtype[p.col] == 'B' {
    v = p.val·s; c = p.col
    if v > 0 { u = bit[c];   if u != 0 { acc |= u; masks.push(acc); thr.push(v + d) } }
    else     { u = bit[n+c]; if u != 0 { acc |= u; masks.push(acc); thr.push(d − v) } }
    p = 다음(행 압축); cnt1 += 1 }
  k = len(masks); W += ((double)k + (double)k + (double)cnt1·5.0)·μ
  pp = k − 1; cnt2 = 0
  for x in 행 i2(머리부터) {
    if vtype[x.col] != 'B' { break }
    v = x.val·s; while pp >= 0 && !(0.0 <= thr[pp] + |v|) { pp −= 1 }
    if pp < 0 { break }
    l = if v > 0 { x.col } else { n + x.col }; u = mask[l]
    if cntp != NULL { if u == 0 { list[*cntp] = l; *cntp += 1 }; mask[l] = u | masks[pp] }
    else if u != 0 { mask[l] = u | masks[pp] }
    cnt2 += 1 }
  W += ((double)k + (double)k + (double)cnt2·6.0)·μ
```
뜻: thr[q] = (min activity − rhs − 100·ft) + |a_q|는 r 리터럴 q가 나쁜 쪽일 때의 여유. 리터럴 x가 나쁜 쪽이고 thr[pp] + |a_x| ≥ 0이면 x는 q = 0..pp(계수가 더 큰 r 리터럴)와 함께 행을 어긴다. s = −1이면 `=` 행의 반대쪽(max activity)이다. 리터럴이 32개를 넘으면 스택 배열(int 32, double 32)을 넘치지만 bit ≠ 0인 것만 쌓으므로 청크 크기(≤ 32)로 막힌다.

### 6.5 기본값에서 닿는 보조 함수

coverage(`cov2.json`, 함수별 caller 표시): 46a62d가 부르는 것은 `538f30`(늘), `4694fe`(glass4 13, gmu-35-40 175, gmu-35-50 231, fhnw-binpack4-4 24회), `5379f1`/`53763f`(gmu 두 모델)뿐이다. **`3de18e`, `3df916`, `370047`, `3b85ea`는 46a62d에서 한 번도 불리지 않는다**(38개 모델에서 `46a62d>3de18e` 등 0회. 같은 모델의 `3de18e`/`3df916` 호출은 다른 reduction·45885d·probing에서 온다). 그래서 단계 1·2의 `ub == lb` 고정, `=` 행 고정(단계 5), 노드 풀 증설은 기본값 비교 대상이 아니다. 포트에서는 오류(미구현)로 막아 두고 오라클이 생기면 채운다.

### 6.6 오라클 확인

도구: `/tmp/claude-1000/-home-sds-workspace-gurobi-sandbox-rsolver-barrier/6a50ff5f-7bc0-40cb-ba03-b7467a59149b/scratchpad/clq/d46.js`(`node d46.js <model> <out.json> [Param=Value]`, `tools/gurun.sh`로 감쌈)(46a62d 호출마다 진입·퇴장 상태를 노드 주소까지 덤프, 538f30 목록, 타이머 t[0]/t[2]/플래그, mode, 5379f1·53763f 순서, 4694fe 호출)와 `/tmp/claude-1000/-home-sds-workspace-gurobi-sandbox-rsolver-barrier/6a50ff5f-7bc0-40cb-ba03-b7467a59149b/scratchpad/clq/sim46.py`(`python3 -I sim46.py <dump.json>`. 위 의사코드를 그대로 옮긴 Python 참조 모델. 진입 상태를 넣고 돌려 퇴장 상태와 비교).
- **기본값 38개 모델의 46a62d 호출 78회 전부(cov2 합계와 같음) 비트 단위로 같다**: 행·열 리스트 순서와 값, lb/ub/rhs/row_cnt/col_cnt/act, 카운터 14개, +0xd8, +0x198/+0x188, 작업량 W, 추적기 표시 순서. 추가로 `GURO_PAR_PRECLIQUE=2`(mode 2, cap 3200, L = m)에서 assign1-5-8·csched007·exp-1-500-5-5·glass4·gmu-35-40 13회도 같다(32개 넘는 원래 행의 다중 청크: csched007 16번).
- 실제로 지난 경로(시뮬레이터 계수, 기본값): r ⊆ 다른 clique 행 삭제(`<`) 113회, best 행 확장 211회(+붙인 노드를 이어 보는 청크 14회), 확장 행에 들어가는 행 삭제 20회, `4694fe(s=+1)` 633회.
- 모델을 실제로 줄이는 것은 **gmu-35-40, gmu-35-50 둘뿐**이다(gmu-35-40 첫 호출: +0x404 27→63, +0x3fc 5→122, row/col_cnt, act_max, nnz). 다른 모델에서 바뀌는 것은 단계 1–3의 정규화 값, 행 재배열(B 노드 |a| 내림차순, 죽은 노드 제거; assign1-5-8 130행, csched007 210행, exp-1-500-5-5 250행, graphdraw-domain 214행, neos-860300 152행, glass4·fhnw·neos-911970 등), 열 압축(csched007·csched008 둘째 호출의 +0xd8·+0x188)이다. 리스트 순서와 +0xd8이 바뀌므로 포트는 이 단계들을 빼면 안 된다. 나머지 25개 모델은 W만 바뀐다.
- 오라클로 못 본 경로: 전부 붙이기(best < 0), `=` 행 고정과 단계 5(`3df916`, `370047`), `4694fe(s=−1)`, `wacc > B`로 건너뛰기, `=` clique 행의 r ⊆ 다른 행(제외만), cls 0·3 부분 행의 제외, 할당 실패(`3b85ea`), 단계 1·2의 `3de18e`, 1e7 규칙의 stop. 시뮬레이터는 이 중 고정 경로·`3de18e`·`3b85ea`에 닿으면 "not modelled"로 멈추는데, 91회 중 한 번도 닿지 않았다.

### 6.7 열린 질문

1. `cls[best] |= 1` / `&= ~4`: 비트 4를 쓰려던 것으로 보이지만 asm(0x416ab9 `orl $0x1`)이 1이다. best 행이 `<` clique(2)이면 3이 되어 이후 "확장 행에 들어가는 행 삭제"에서 지워지지 않고 제외만 된다. 그대로 옮긴다.
2. `added − saved < lc`이고 이어 볼 새 노드가 없으면(할당 실패 때만) mask에 lc개가 남은 채 다음 행으로 간다. 다음 행에서 그 리터럴은 목록에 안 들어가지만 값이 OR된다. 기본값에서는 안 일어난다.
3. t[2]의 식(`3cd406` L1560–1585, local_930)은 spine 노트에서 확정해야 한다. 기본값 B가 17만–25만이라 wacc 건너뛰기는 관측되지 않았지만 큰 모델에서는 일어날 수 있다.
4. 단계 5의 `3df916(ft, env, st, j)`는 §4와 같다(45885d가 gmu에서 부른다).
5. NaN: `thr + |v| ≥ 0` 비교는 asm에서 NaN이면 참(jbe), 의사코드 `!(0 <= x)`는 거짓이다. NaN이 들어올 일은 없다고 보고 무시했다.

## 7. 오라클 확인

- 도구: `tools/mip_state_dump.js`(진입·복귀 상태), 세션 스크래치의 Frida 탐침(`451bed` 인자와 출력 배열: clique 표, colstart, cidx, rec_rows/flag, 45885d의 timer). Python 모델 `m451.py`(451bed + 3de18e), `m458.py`(45885d + 3df916 + 53802c + 370047)를 진입 상태에 돌려 복귀 상태와 비교했다. 작업량은 진입 W에서 항목을 순서대로 더해 비트 비교했다.
- `451bed` 호출 1(45885d에서): 38개 모델 전부에서 ncl, start, lits, sense, flag, rec_rows, rec_flag, maxlen, colstart, cidx, 작업량이 같다. neos-860300은 3de18e 고정 152개(레코드·activity·objcon 포함)까지 같다. 2-clique(§2.3)는 berkel·neos859080에서 확인. probing 호출(bin_only 1, list NULL)은 glass4(ncl 36), graphdraw-domain(45), tr12-30·ic97_potential·berkel(ncl 0, 작업량만)에서 같다.
- `45885d`: 38개 모델의 호출 78개 전부 OK(위 상태 + 카운터 +0x3f8..+0x408 + 레코드 + 작업량). 동작한 호출: csched007 #1(bound 1) #2(행 3 삭제), csched008 #1(bound 1) #2(계수 48, 행 42 삭제, rhs) #3(계수 28), gmu-35-40 #1(bound 16, 계수 3, 3df916 고정 15) #2 #4(계수), gmu-35-50 #1(bound 24, 고정 23) #2 #4(계수), nidda #1(ub 2), berkel #1(lb 2, 계수 21), neos-860300 #1(451bed 고정 152, 1e7 콜백 2회) #2(계수 8), neos859080 #1(계수 20), sp150x300d #1(ub 22, 계수 2).
- 확인하지 못한 경로: 45885d 첫 훑기의 3de18e(기본값 0회), 예산 정지(adf2be), 3ba9de stop, 실행 불가 반환, SOS 분기, 버퍼 2e9 한계.
- `52be10`: §5.4(gmu-35-40 #1 전 단계, glass4·neos5 훑기, 합성 모델로 삼각형 확장·`469c12`·GC·`=` 씨앗 고정). `46a62d`: §6.6(78회 전부, PRECLIQUE=2 13회).
- 참조 모델(세션 스크래치 `clq/`): `m451.py`, `m458.py`, `f52/m52.py`, `sim46.py`. 포트의 시험 오라클로 쓸 수 있다(진입 덤프 → 복귀 덤프).

## 8. 미해결

- `53a487`의 연결 리스트 분기 출력은 따로 확인하지 않았다(검증은 Frida로 잡은 목록을 입력으로 썼다). 뼈대·inner 쪽에서 포팅할 때 이 목록을 오라클로 비교해야 한다. `538f30`(52be10·46a62d)도 같다.
- `3cd406`이 timer 예산(t[2])을 정하는 식(L1560–1585, local_930). spine 노트에서 확인한다. 예산에 걸리면 호출 자체를 건너뛰고 PRELIMIT도 세지 않는다. 46a62d는 예산으로 행 훑기도 건너뛴다(§6.3의 B, wacc).
- 기본값에서 안 도는 경로(오라클 없음): 45885d 첫 훑기 고정, 52be10 `=` 씨앗 실행 불가·1 고정 리터럴 확장, 46a62d 전부 붙이기·`=` 행 고정(단계 5)·`4694fe(s=−1)`. 포트에서는 미구현 오류로 막아 두고 오라클이 생기면 채운다.
- 52be10의 `<` 확장은 act_min(+0x98)을 고치지 않는다(§5.5). 46a62d의 `cls[best] |= 1`(§6.7)도 그대로 옮긴다.
- `451bed` SOS 분기(+0x1f8 > 0: `a85f71`, +0x208 type 1 SOS의 리터럴 clique, SOS 짝 표 local_118)는 해독하지 않았다. 기본값 38개에는 SOS가 없다.
- §2.3의 `=` 방향 판정 `|a| + act_max ≤ 1e-10`은 asm 그대로이며 뜻으로는 `act_max − |a|`가 맞아 보인다(버그 호환으로 둔다).
