# M2 MIP presolve 바깥 패스 B 해독 (`4030c9`, `40d2bc`, `36ee92`, `3e40d1`, `37f484` MIP, `418542`)

`3cd406` 바깥 패스 가운데(m2_spine_b.md §2, 카운트 20–28)의 reduction 해독과 포팅 노트. 표기·규칙은 [m2_red_inner.md](m2_red_inner.md) §1과 같다(`W += x·μ`, 괄호가 곱 순서, `gup`/`gdn`, 노드·리스트·추적기, 보조 함수 §1.4–1.9와 §11 API). 근거는 디컴파일(`re/decomp/c/`)과 `objdump`(주소는 파일 오프셋), 런타임 근거는 `tools/mip_state_dump.js`로 뜬 기본값 38개 모델의 모든 호출(진입·복귀 상태) 및 §8의 직접 호출 합성 상태다. 바깥 패스는 추적기 레벨 1이다(`538f30`·`5399fc`·`53af0f`는 레벨 1 목록과 캐시).

## 0. 요약

| 함수 | 크기 | 가드(3cd406) | 하는 일 | 38개 호출 | 덤프 비교 |
|---|---|---|---|---|---|
| `3e40d1(st, env)` | 2,622 | DualReductions | 쌍대 논리로 `<` 행을 `=`로 | 78(바꾼 것 5) | 78/78 |
| `418542(st, env)` | 2,645 | Aggregate && DualReductions | 쌍대 집계: 행의 이진 열과 다른 열 묶기(`3bb6fe`) | 78(`3bb6fe` 0회: 작업량만) | 78/78 |
| `414c03`/`414c53` → `40d2bc(st, env, mode, M, T, W)` | 19 + 22 + 17,951 | PRECANCEL / PRECANCELCOL && T15/T16 < 0.8 | 비영 소거: `=` 행(mode 0)·암시 자유 열(mode 1)의 배수를 더해 겹치는 항을 지움 | 78 + 78(바꾼 것 행 15, 열 1) | 78/78, 78/78 |
| `37f484(st, env)` MIP 분기 | 34,685 | Aggregate && T1 < 0.8 (앞에 PREBOUND면 `37168e`) | 집계: 단계 A(`=` 행으로 열 대입), B(`<` 행 → 슬랙), C(쌍 병합, MIP 레코드·타입·bound) | 78(바꾼 것 16) | 78/78 |
| `4030c9(st, env, T17)` | 21,516 | PREDOMINATE && DualReductions | 지배 열: 두 단계 bound 고정, 큰 행 부분집합 | 78 | 78/78 |
| `36ee92(env, st)` | 357 | 없음(카운트 없음) | 노드 배열 압축(`510953` CSR 복사 → `514031` 다시 깔기) | 78(압축 1: neos-860300 첫 호출) | 78/78 |

## 2. `3e40d1` 부등식 → 등식 (`004c3920`)

### 2.1 서명·가드·반환
`void 3e40d1(st, env)`. 3cd406은 DualReductions ≠ 0일 때 T4 타이머 안에서 부르고 반환값을 보지 않고 센다(카운트 23). 자체 가드 없음. 할당 없음.

### 2.2 알고리즘
```
INF = env+0x2458
(cnt, cols) = 5399fc(tr, st)                    // 레벨 1 열 목록(캐시)
W += ((nnz · 5 · cnt) / (n + 1)) · μ             // nnz = +0xd8(i64), n + 1은 int 덧셈. 곱·나눗셈 순서 그대로
for j in cols:
  if !(0 < col_cnt[j]) || colflag[j] != 0: continue
  l = lb[j]; u = ub[j]
  if l > u − 1e-10: continue                      // 고정(에 가까운) 열
  c = obj[j]
  up = (1e-10 < c) ? −2 : −1                      // −1: 아직 없음, −2: 안 됨, ≥ 0: 그 행
  dn = (c < −1e-10) ? −2 : −1
  for node in col j list, 살아 있는 것만:
    r = node.row
    if sense[r] == '=': 다음 열로                  // 열 전체를 건너뜀
    if node.val > 0:  (sense[r] == '<' && up == −1) ? (up = r, a_up = val) : up = −2
    else:             (sense[r] == '<' && dn == −1) ? (dn = r, a_dn = val) : dn = −2
  if dn < 0 && up < 0: continue
  if dn ≥ 0:                                      // 내리는 쪽: 행 dn 하나만 x_j를 아래에서 막는다
    ok = true
    if −INF < l: (lo, _) = 3dcc78(INF, a_dn, st, j, mode 1, single, dn); ok = !(lb[j] − 1e-9 > lo)
    if ok && vtype[j] != 'C': ok = INTROW(dn, a_dn)
    if !ok: dn = −2
  pick = dn
  if up ≥ 0:                                      // 올리는 쪽
    ok = true
    if INF > ub[j]: (_, hi) = 3dcc78(INF, a_up, st, j, 1, single, up); ok = !(hi > ub[j] + 1e-9)
    if ok && vtype[j] != 'C': ok = INTROW(up, a_up)
    if ok && !(dn ≥ 0 && row_cnt[up] > row_cnt[dn]): pick = up      // 둘 다 되면 짧은 행, 같으면 up
  if pick < 0: continue
  [+0xfc == 0 && +0x100 ≠ 0이면 반대쪽 bound를 무한으로 풀고 3dd6f1: LP 상태 전용, MIP에서는 안 돎]
  sense[pick] = '='; +0x400 += 1; 53763f(tr, pick)

INTROW(r, a):                                     // x_j를 정수 걸음으로 움직여 행을 꽉 채울 수 있는가
  q = rhs[r] / a
  if !(1e-10 >= q − floor(1e-10 + q)): return row_head[r] == NULL     // 빈 행이면 통과(원본 그대로)
  for node in row r list, 살아 있는 것만:
    if vtype[node.col] == 'C' || !(1e-10 >= v − floor(1e-10 + v)) (v = node.val / a): return false
  return true
```
- 논리: 비용이 x_j를 내리는(올리는) 쪽을 막지 않고(c ≥ −1e-10, 올림은 c ≤ 1e-10), x_j를 그쪽으로 막는 것이 `<` 행 하나뿐이며 그 행만으로 이미 bound보다 강한 함의 bound가 나오면, 최적해 하나에서 그 행은 꽉 찬다.
- 3dcc78 단일 행 모드(m2_red_inner §1.8)는 정수 열이면 lo/hi를 IFT로 안쪽 반올림하고 `W += μ·3`. 둘째 double 인자(xmm1)는 그 행의 계수(asm 0x3c3bb1, 0x3c3d16).
- 행 하나를 `=`로 바꾸면 같은 호출의 뒤 열들은 그 행을 `=`로 보고 건너뛴다(순서 의존).
- 비교는 모두 asm 그대로: `a > 0`(NaN이면 a ≤ 0 쪽), `l > u − 1e-10`, `lb − 1e-9 > lo`, `hi > ub + 1e-9`, 정수성 `1e-10 >= x − floor(1e-10 + x)`(NaN은 정수 아님).
- 작업량: `5399fc`(목록), 위 한 줄, `3dcc78` 호출마다 `μ·3`. 열·행을 걷는 것 자체는 세지 않는다.

### 2.3 오라클 확인
`mip_state_dump.js <model> 3e40d1 all`(38개, 78호출). 바꾼 호출 5개(gmu-35-40·gmu-35-50·neos-2657525-crna·neos-4954672-berkel·neos859080 각 첫 호출, 행 하나씩). `tools/mip_dump_check.py 3e40d1`: **78/78 일치**(sense, +0x400, 추적기, 작업량 포함).

## 3. `418542` 쌍대 집계 (`004e12c0`)

### 3.1 서명·가드·반환
`int 418542(st, env)`. 3cd406은 Aggregate && DualReductions일 때 T4 안에서 부르고 센다(카운트 28). 반환 0, 할당 실패 0x2711, `3bb6fe`가 0이 아닌 값을 주면 그 값.

### 3.2 알고리즘
```
ft = minsd(FeasibilityTol, 1e-6)
neg[n], pos[n] = 0 (calloc); mark[m] = 0 (calloc); buf = malloc(8m)(3bb6fe 작업 공간)
for j in 0..n:                                         // 열마다 `<` 행의 계수 부호 세기
  walked = 0
  for node in col j list:
    if node.col ≥ 0:
      if sense[node.row] == '=': pos[node.col] = neg[node.col] = 2e9; break    // 이 노드는 walked에 안 셈
      node.val > 0 ? pos[node.col]++ : neg[node.col]++
    walked++
  W += walked · (3·μ)
W += μ · n
pass = 0
loop:
  succ = 0
  for i in 0..m:
    if mark[i]: 370047(st, env, 1, &i); mark[i] = 0              // 3bb6fe가 표시한 행만
    if sense[i] == '=' || row_cnt[i] ≤ 0 || inf_min[i] ≠ 0 || inf_max[i] ≠ 0: continue
    // 행을 혼자 느슨하게 만들 수 있는 첫 이진 열 k
    before = 0
    for node in row i list:
      if node.col ≥ 0 && vtype ≠ 'C' && lb == 0.0 && ub == 1.0
         && act_min[i] + |a| >= −1e-10 && ft >= act_max[i] − |a|: k = node.col, a_k = a; break
      before++
    W += before · (4·μ)
    if k 없음: continue
    walked = 0
    for node in row i list (처음부터):
      c = node.col
      if c ≥ 0 && c ≠ k:
        d = ub[c] − lb[c]
        if d > 1e-6 && |a|·d > 1e-6 && colflag[c] == 0:
          if a > 1e-4 && 1e-10 >= obj[c] && pos[c] == 1: s = +1
          elif −1e-4 > a && obj[c] >= −1e-10 && neg[c] == 1: s = −1
          else: s 없음
          if s: (x, b) = (0 > s·a_k) ? (d, lb[c]) : (−d, ub[c])
                rc = 3bb6fe(x, b, env, st, c, k, buf, mark, 0, 0, pass == 0, &ok)
                if rc ≠ 0: return rc
                if ok: succ++; if pass > 0: pos[k] = neg[k] = 2e9
      walked++                                        // 모든 노드
    W += (4·μ) · walked
  W += (5·μ) · m
  if pass == 1 && succ > 0: 표시된 행마다 370047(…, 1, &i), 표시 지움; W += m · μ
  if succ < (n − +0x408) / 50: break                  // i32 나눗셈(0 쪽 버림)
  pass++; if pass ≥ 2: break                          // succ는 다음 pass에서 0부터
```
- 열 c는 비용이 그 방향을 막지 않고(a > 0이면 obj ≤ 1e-10) 그 부호로는 이 `<` 행 하나에만 있다. 그러면 최적해에서 x_c를 이진 x_k에 묶을 수 있다는 논리로 `3bb6fe`(2,389 B, x_c = b + x·x_k 꼴 집계, `5431ee`도 부른다)를 부른다.
- 기본값 38개에서는 `3bb6fe`가 한 번도 불리지 않는다(coverage `cov2.json` 0회, 훅 있음). 그래서 mark·succ는 늘 0이고, (n − 지운 열) < 50이면 둘째 pass가 돈다(작업량만 다시 더함).
- 곱 순서는 asm 그대로: `walked·(3·μ)`, `μ·n`, `before·(4·μ)`, `(4·μ)·walked`, `(5·μ)·m`.

### 3.3 `3bb6fe` 실제 집계 (2026-11-06 보완)
`dualagg.rs::dual_agg_subst`에 대입 `x_c = b + x·x_k`를 옮겼다. `|x|·mult_lo[k] < 1/L` 또는 `L < |x|·mult_hi[k]`이면 실패. **pass 0은 검사만 하고 성공을 센다.** 성공 수가 `(n − 지운 열)/50` 미만이면 실제 집계 없이 끝난다. pass 1에서만 다음을 적용한다.

1. c의 행 scatter를 비우고 k의 살아 있는 노드를 표시한다. c에만 있는 행에는 값 0의 k 노드를 먼저 만들어 행·열 머리에 넣는다. 작업량은 차례로 `2·walk_c·μ`, `2·walk_k·μ`, `(6·added + 2·walk_c)·μ`.
2. c의 노드마다 `a_k = a_c·x + a_k`, `rhs -= a_c·b`. `|a_k| < 1e-10`이면 k 노드도 죽인다. c 노드는 모두 죽이고 행을 mark 및 추적기에 표시한다. 작업량 `walk_c·6·μ`. 삭제는 lazy: nnz는 GC까지 그대로다.
3. `obj[k] = obj[c]·x + obj[k]`, `objcon += obj[c]·b`, `obj[c] = 0`, `col_cnt[c] = −3`, 열 제거 카운터 +1. type 2 `{c, b, [k], [x]}`를 만든다. x ≠ 1이면 k의 multiplier bound에 c의 `|x|` 배율을 반영한다. k를 추적기에 표시한다.
4. 표시된 행은 다음 방문 직전에 `370047`로 재계산한다. pass 1에서 성공했으면 끝에서도 남은 mark를 재계산하고 `m·μ`를 더한다. k의 pos/neg는 2e9로 막는다.

`5431ee`에서 부르는 경로 및 그 호출자의 이식은 outer A 범위다. 여기서는 `418542`가 사용하는 지연 activity 갱신 경로를 구현했다.

### 3.4 오라클 확인
기존 38개 모델은 실제 대입이 0회라 분기 검증이 되지 않는다. 현재 코드로 기존 덤프 **78/78을 재비교**했고, 별도의 직접 호출 합성 상태 7개(`bench/oracle/mip_outer_b/dual_*.json.gz`)도 **7/7 일치**했다. 계수 부호 × 이진 계수 부호 4조합(폭 2, 0이 아닌 절편), 두 후보 연속 집계(폭 0.5), 배율 한도 거절, n ≥ 100에서 첫 검사 패스만 하고 끝나는 경로를 포함한다. 실제 열 제거·채움·상쇄·목적 상수·레코드 원시 바이트·행 activity·추적기·작업량을 비교했다.

재현: `PYTHONPATH=$PWD/python ../.venv/bin/python tools/mip_dump_check.py 418542 bench/oracle/mip_outer_b/dual_a*.json.gz bench/oracle/mip_outer_b/dual_two_candidates.json.gz bench/oracle/mip_outer_b/dual_test_only.json.gz`. 배율 거절 픽스처는 `dual_growth_reject.json.gz --param GURO_PAR_AGGMULTLIMIT=1`로 따로 비교한다. 새 오라클의 `--direct`는 첫 activity 계산 직후 원본 함수를 호출하므로, 앞선 reduction에 의해 합성 패턴이 사라지지 않는다.

## 4. `36ee92` 노드 배열 압축 (`00478040`)

### 4.1 서명·가드·반환
`int 36ee92(env, st)`. 3cd406은 `36e6b6(st, 1)`(전체 GC) 바로 뒤에 T27 안에서 부르고 세지 않는다. 반환은 0, 할당 실패면 0x2711.

### 4.2 알고리즘
```
nnz = +0xd8; F = +0x198(free 수); cap = +0x158(배열 크기); ch = +0x1a8(다음 청크 크기)
if nnz != 0 && (2·cap < nnz + F || (nnz < F && 100000 < F && 2·ch < F)):
  +0x1a8 = F
  510953(env, st)                     // CSR 복사
  514031(st, env)                     // 리스트 다시 깔기
+0x20..+0x58 CSR 버퍼 해제, +0xd0 = 0
```
`510953`(1,664 B): 작업량은 μ 없이 원본 순서대로.
```
W = (n + n) + W
for i in 0..m: k = 행 i 리스트의 살아 있는 노드 수(열 ≥ 0); W = k·3 + W      // 지운 행도 걷는다
W = m·4 + W
for j in 0..n: 열 j 리스트의 살아 있는 노드를 순서대로 (행, 값) 복사, k = 그 수; W = k·3 + W
W = n·4 + W
```
`514031`(1,950 B):
```
start[i] = Σ_{i' < i} max(row_cnt[i'], 0); total = start[m]; W = m·3 + W
if +0x1b8 != 0 || cap < total + F:          // +0x1b8 = 3b85ea가 더 잡은 청크 사슬
   3b8c59(청크 해제); 배열을 total + F 크기로 realloc; cap = total + F
for j in 0..n:
  if col_cnt[j] ≥ 0:
    for (r, v) in 열 j의 복사본: if r ≥ 0 && row_cnt[r] ≥ 0: 노드를 행 r의 다음 칸에 놓고 열 j 리스트 끝에 잇는다
    W = k_j·7 + W                            // k_j = 복사본 길이
  열 j 끝(또는 머리) = NULL                   // 지운 열은 머리 NULL
W = n·4 + W
for i in 0..m: 칸 범위가 비면 머리 NULL, 아니면 칸 순서로 잇고 W = (len − 1)·4 + W
W = m·6 + W
F = cap − total; 칸 total..cap−1을 순서대로 free 리스트(열 −1)로; pending = NULL, +0x188 = 0
```
- 결과: 행 리스트는 열 번호 순서, 열 리스트는 순서 유지(죽은 노드 제거), 지운 행·열의 노드는 사라진다. +0xd8(nnz)은 그대로다.
- `3b85ea`·`3bea24`는 +0x158을 바꾸지 않는다(청크는 +0x1b8 사슬로만 간다). 그래서 첫 조건(`2·cap < nnz + F`)은 청크가 많이 붙었을 때만 참이다. 포트는 `State::node_cap`을 +0x158 그대로 두고(`new_node`가 더 이상 늘리지 않는다), 배열이 node_cap보다 크면 "청크가 있다"로 본다.

### 4.3 오라클 확인
덤프에 +0x158(`node_cap`)과 +0x1a8(`chunk`)을 더했다(`mip_state_dump.js`, 없으면 포트는 배열 크기·10000을 쓴다). 38개 78호출 중 압축은 neos-860300 첫 호출 하나(nnz 201,792 < F 220,969, ch·2 < F). 그 호출에서 작업량 +3,453,024(μ = 1.2588인데 μ를 곱하지 않음)가 비트 단위로 맞는다. 행 리스트는 이미 열 순서였다. `mip_dump_check.py 36ee92`: **78/78**. 청크가 붙은 경로(realloc)는 지나지 않았다.

## 5. `40d2bc` 비영 소거 (`004da220`, 래퍼 `414c03` mode 0 / `414c53` mode 1)

### 5.1 서명·가드·반환
`int 40d2bc(st, env, mode, M, T, W)`. `414c03(st, env, M, T)`/`414c53`는 mode 0/1과 `W = *(st+0x458)`을 붙여 꼬리 호출한다. M은 3cd406이 `PRECANCEL == −1 ? 2 + (local_a8 > 0) : PRECANCEL − 1`로 준다(기본 2). T = T15/T16(한도 0.5B). 반환 0, 할당 실패 0x2711, `3ba5ac`/`3ba9de`의 값.

이름: mode 0에서 피벗과 소거 대상은 행(major), 그 항목은 열(minor). mode 1은 바꿔서 major = 열, minor = 행이다.

| | mode 0 | mode 1 |
|---|---|---|
| major 리스트·개수 | 행 리스트(+0x10), row_cnt | 열 리스트(+0x18), col_cnt |
| minor 리스트·개수 | 열 리스트, col_cnt | 행 리스트, row_cnt |
| 대상 목록 | `53a487`(레벨 1 행) | `53af0f`(레벨 1 열) |
| 피벗 조건 | sense `=` | colflag 0, col_cnt ≤ 500, `379173` && `3790d9`(위·아래 bound가 행으로 암시됨) |
| rhs' | rhs | obj |

### 5.2 알고리즘
```
F = NumericFocus 0/1/2/≥3 → 0.1/10/100/1000; tol = F·FeasibilityTol; poison = −NMIN − 1
CNT[NMAJ], MISS[NMAJ] = 0 (calloc), STAMP[NMIN] = 0, PC[NMIN](피벗 계수), CP/CN[NMAJ], MULT(st+0x3a0)/MN[NMAJ],
SGN[m] = 0 (행 부호 캐시), RMARK(st+0x3a8)/RLIST(st+0x3b8) 비어 있음, ncancel = 0, W0 = Wchk = W
L = 53a487 / 53af0f
for idx in 0..|L|: p = L[idx]
  if maj_cnt[p] < 2 || !피벗 조건: continue                     // 379173·3790d9는 W += (μ·8)·(멈출 때까지 걸은 노드)
  two = M ≠ 0 && maj_cnt[p] ≥ 3; maxc = 0; marked = false
  if adf2be(T, W): break                                          // 타이머 소진
  if W − Wchk > 1e7: 3ba5ac, 3ba9de(env, st, &flag) (≠0이면 그 값으로 바로 반환), PRELIMIT < 0이면 0으로 바로 반환
                      (아래 끝처리 없음), flag면 break, Wchk = W, W − W0 > budget/5면 break
  // 피벗의 minor 모으기
  for node in maj p 리스트: 살아 있으면 q = minor; PC[q] = val; LST += q; KEY += −min_cnt[q]; tot += min_cnt[q]
  W += (μ·6)·걸은 노드
  aabbe3(k, KEY, LST)                                             // KEY 내림차순 = minor 길이 오름차순
  if 200 < −KEY[0]: continue
  s = (−KEY[1] ≤ 200 && two) ? 1 : 0; need = k − s               // seed = LST[0..=s]; 많아야 s개 빠져도 됨
  sum = 0; for t in 0..=s: LST[t]의 minor 리스트에서 살아 있는 노드마다 sum += maj_cnt[major];
           W = 걸은·μ + W; (t == s면) W = W + μ·(2(s+1))
  local230 = 0
  if tot/4 < sum:                                                 // A: minor 순서대로 세기 (i64 나눗셈)
    full = need − 1
    for t = 0, 1, …:
      for node in LST[t] minor 리스트(살아 있는 것), i = major:
        c = CNT[i]
        if t ≤ s:
          if !(maj_cnt[i] ≥ need && i ≠ p): 다음
          if t == 1 && c == 0: (M < 2 && k < maj_cnt[i])면 다음; MISS[i] = LST[0] + 1
        else:
          if !(t − s ≤ c): 다음
          if c == t − 1 && MISS[i] == 0: (M < 2 && k < maj_cnt[i])면 CNT[i] = 0, 다음; MISS[i] = LST[t−1] + 1
        CNT[i] = c + 1; maxc = max(maxc, CNT[i]); CNT[i] == full이면 local230 += maj_cnt[i]
      W = (t ≤ s ? (μ·5)·걸은 : (μ·4)·걸은) + W; t == s면 + (2(s+1))·μ
      if t ≥ s && (k ≤ t+1 || maxc < t+1−s): W += (μ·2)·(t−s); break
  else:                                                           // B: 피벗 minor 표시 후 major마다 겹침 세기
    STAMP[LST[*]] = 1; W = (k + k)·μ + W; marked = true; stamp = 1
    for t in 0..=s:
      for node in LST[t] minor 리스트(살아 있는 것), i = major:
        if i == p || maj_cnt[i] < need || (mode 1 && maj_cnt[i] > 500) || CNT[i] ≠ 0: 다음
        if t == 1: (M < 2 && k < maj_cnt[i])면 다음; MISS[i] = LST[0] + 1
        stamp++; maj i 리스트의 살아 있는 노드마다 STAMP[q] ≠ 0이면 STAMP[q] = stamp, CNT[i]++; W = (걸은 + 걸은)·μ + W
        if s ≠ 0 && t == 0 && CNT[i] == k − 1:
          (M < 2 && k < maj_cnt[i])면 CNT[i] = 0, 다음
          u = 1..k에서 STAMP[LST[u]] ≠ stamp인 첫 u: MISS[i] = LST[u] + 1; W = u·μ + W (못 찾으면 u = k)
        maxc = max(maxc, CNT[i]); local230 += maj_cnt[i]
      W = μ·4·걸은 + W; t == s면 + μ·(2(s+1))
  if need ≤ maxc:
    if tot < local230:                                            // C1: minor 순서대로 투표
      for t = 0, 1, …, k−1:
        for node in LST[t] minor 리스트(살아 있는 것), i = major, r = −val / PC[LST[t]]:
          c = CNT[i]
          if c < need: c < 0이면 VOTE(i, r); 아니면 (t ≤ s일 때만) CNT[i] = MISS[i] = 0
          elif t ≤ s:                                             // 후보 시작
            r ≤ 0 ? (MULT[i] = 1e100, CP[i] = 0, MN[i] = r) : (MN[i] = 1e100, CN[i] = 0, MULT[i] = r)
            c == k면 MISS[i] = −1, 아니고 MISS[i] == 0이면 ((M < 2 && k < maj_cnt[i])면 CNT[i] = 0, 다음) MISS[i] = LST[k−1] + 1
            CNT[i] = −1; (r ≤ 0 ? CN : CP)[i] = −1
        W = μ·5·걸은 + W; t == s면 + (2(s+1))·μ; t ≥ s && k ≤ t+1면 W += (μ·2)·(t−s), 끝
    else:                                                         // C2: 후보마다 자기 리스트로 투표
      marked가 아니면 STAMP 표시, W = (k+k)·μ + W, marked = true
      for t in 0..=s: for node in LST[t] minor 리스트(살아 있는 것), i = major, c = CNT[i]:
        c < 0: 다음; c < need: CNT[i] = MISS[i] = 0, 다음
        MISS 규칙(C1과 같음); CNT[i] = −1; CP = CN = 0; MULT = MN = 1e100
        maj i 리스트의 살아 있는 노드 중 STAMP[q] ≠ 0이면 VOTE(i, −val / PC[q]) (r > 0이면 MULT/CP 쪽); W = μ·5·걸은 + W
        (seed마다) W = μ·5·걸은 + W; t == s면 + μ·(2(s+1))
    // 고르기
    nsel = 0; score = 0
    for t in 0..=s: for node in LST[t] minor 리스트(살아 있는 것), i = major, c = CNT[i]:
      c ≥ 1: 다음 (이미 뽑힘)
      c == 0 || !PICK(i): MISS[i] = CNT[i] = 0
      else: CNT[i] = −CNT[i]; SEL += i; score += CNT[i] − (MISS[i] > 0)
      W = μ·4·걸은 + W; t == s면 + μ·(2(s+1))
    if nsel > 0:
      mode 0: 537da8(tr, st, p) / mode 1: score < 5면 SEL의 CNT·MISS = 0, W = μ·3·nsel + W, 적용 없이 끝; 아니면 53802c(tr, st, p)
      APPLY; ncancel += nsel
  else: for t in 0..=s: LST[t] minor 리스트의 살아 있는 노드의 major마다 CNT = MISS = 0; W = μ·4·걸은 + W; t == s면 + μ·(2(s+1))
  if marked: STAMP[LST[*]] = 0; W = (k + k)·μ + W
W = W + (3·idx)·μ                                                 // 정상 끝은 idx = |L|, break면 그때 idx
+0x3fc += ncancel; RMARK 지우기; W = (|RLIST| + |RLIST|)·μ + W; 370047(st, env, |RLIST|, RLIST)

VOTE(i, r) (`40c0b2` 4가지 사본 = F별 상수, C2는 인라인):
  (slot, cnt) = r ≤ 0 ? (MN[i], CN[i]) : (MULT[i], CP[i])      // C2 인라인은 0 < r ? MULT : MN (같음)
  d = |r − slot|
  d ≤ 1e-13: cnt −= 1 / tol < d: |r| < |slot|이면 cnt = −1, slot = r / 그 밖(가까운데 다름): cnt = poison, |r| < |slot|이면 slot = r
PICK(i):
  MN == 1e100 || CN ≤ poison → 양(CNT = CP); 아니면 MULT == 1e100 || CP ≤ poison → 음(CNT = CN, MULT = MN);
  아니면 PRECANCELSELECT(env+0x25a4, 기본 −1):
    −1: −MN < 10·MULT면 (MULT < −10·MN면 개수 비교, 아니면 음), 아니면 양
    ≠0: 개수 비교 / 0: −MN < MULT면 음 아니면 양
    개수 비교: CP < CN → 양, CN < CP → 음, 같으면 −MN < MULT ? 음 : 양
  c = CNT[i]; !(c < 0 && −NMIN ≤ c) → 거절
  MISS[i] > 0 (채움 하나): M == 1: maj_cnt[i] + c > 1 → 거절; M == 2: c + maj_cnt[i] > 1이고 (k < 21 || k + c > 1) → 거절;
                          c == −1 → 거절
  x = MULT[i]; !(|x| ≤ 1000 && 0.001 ≤ |x|) → 거절
  q = MISS[i] − 1
  mode 0: row_cnt[i] + c + (q ≥ 0) > 2 && row_cnt[p] + c > 1 && q ≥ 0이면: vtype[q] == 'B'이고 SIGN(i) 규칙(아래 mode 0형)일 때만 받음
  mode 1 (tp = vtype[p]):
    tp ≠ 'C': i가 'C'이거나 x − floor(x + 1e-10) > 1e-10면 거절; M < 3 → 짧은 채움 규칙; q < 0 → 받음; tp ≠ 'B' → SIGN(q); 아니면 이진 규칙
    tp == 'C': M < 3 → 짧은 채움 규칙; q ≥ 0 → SIGN(q); 아니면 받음
    짧은 채움 규칙: q < 0 → 받음; row_cnt[q] < 3 → 거절; tp == 'B' → 이진 규칙; 아니면 SIGN(q)
    이진 규칙: |PC[q]| + act_min[q] < −1e-10 → SIGN(q), 아니면 거절
    받으면 tp ≠ 'C'일 때 MULT[i] = floor(MULT[i] + 0.5)
  SIGN(r): SGN[r] == 0이면 행 r의 살아 있는 계수가 모두 > 0이면 +2, 모두 < 0이면 +4, 1을 더해 저장; W += (μ·4)·걸은
    y = PC[q]·x; mode 0: g&2 ? 0 < y : g&4 ? y < 0 : 받음 / mode 1: g&2 ? !(y ≤ 0) : !(g&4 && 0 ≤ y)
APPLY:
  sumlen = 0
  for i in SEL: x = MULT[i]; sumlen += maj_cnt[i]; rhs'[i] = rhs'[p]·x + rhs'[i]
    mode 0: RMARK/RLIST에 i, 53763f(tr, i)
    mode 1: tp == 'B'면 5b91ad(env, st, st+0x3f0, p)(충돌 그래프 +0x40 플래그 길이를 n으로 맞추고 [p] = 1)
            (l, u) = bounds of p; x > 0: nl = l − ub[i]·x, nu = u − lb[i]·x / x ≤ 0: nl = l − lb[i]·x, nu = u − ub[i]·x
            (±1e30 넘는 bound가 끼면 ∓1e30); lb[p] = nl, ub[p] = nu
            col p 리스트의 살아 있는 노드마다 bound가 바뀌었으면 행 activity에서 옛 bound 기여 빼고 새 bound 기여 더함(gup/gdn),
            그 행을 RMARK/RLIST에; W = μ·6·걸은 + W
            tp ≠ 'C'면 vtype[p] = (lb, ub) == (0, 1) ? 'B' : 'I'
            3a07b5(1.0, x, 0.0, env, st, p, i): type 10 레코드 {i32 p, i32 i, f64 1, f64 x, f64 0}(x_p = y + x·x_i); 5379f1(tr, i)
  W = μ·5·nsel + W
  if sumlen·6 ≤ k·3 + tot·6:                                      // major 쪽으로 갱신
    marked가 아니면 STAMP 표시, W = (k+k)·μ + W, marked = true
    for i in SEL: maj i 리스트의 살아 있는 노드 중 STAMP[q] ≠ 0이면 40c618(MULT[i]·PC[q], node); W = μ·6·걸은 + W
                  CNT[i] = 0; MISS[i] > 0이면 40cc92(MULT[i]·PC[q], i, q = MISS−1); MISS[i] = 0
    W = μ·3·nsel + W
  else:                                                           // minor 쪽으로 갱신
    for q in LST: minor q 리스트의 살아 있는 노드 중 CNT[major] ≠ 0이면 40c618(MULT[major]·PC[q], node); W = μ·6·걸은 + W
    W = W + k·(μ·3); SEL마다 CNT = 0, 채움(위와 같음), MISS = 0; W = W + (μ·3)·nsel
40c618(d, node): v = d + val; mode 1이면 행 activity에서 옛 값 기여 빼고 새 값 기여 더함; val = v; |v| < 1e-10이면 노드 죽이고 row_cnt·col_cnt −1
40cc92(v, i, q): mode 1이면 행 q activity에 열 i bound로 v 기여를 더한 뒤, 3bea24로 새 노드(행·열 맨 앞, 개수·nnz +1). |v| ≤ 1e-13이어도 만든다
```
- 끝의 `370047`이 RLIST 행의 activity를 다시 계산하므로 mode 0은 activity를 따로 고치지 않는다. mode 1은 따로 고친 뒤 다시 계산한다(결과는 다시 계산한 값).
- `379173(st, env, j, W)`: ub가 무한이면 참. 아니면 열 노드 중 |a| ≥ ft인 것: a ≤ 0이고 `=` 행이며 inf_max == [lb 무한]이면 v = act_max/−a, a > 0이고 inf_min == [lb 무한]이면 v = act_min/−a; `−ub − ft ≤ d − v`(d = lb 유한이면 −lb, 아니면 0)면 참. `3790d9`는 대칭(lb 쪽, a ≥ 0이면 `=`·act_max, a < 0이면 act_min, `lb − ft ≤ d − v/a`, d = ub 유한이면 ub). 둘 다 W += (μ·8)·(찾은 노드 앞까지 또는 전부).
- 레코드 type 10(`3a07b5`, 0x38 B, data 0x20 B): {i32 j, i32 k, f64 d1, f64 d2, f64 d3}, 패딩 없음.

### 5.3 오라클 확인
`mip_dump_check.py 414c03` **78/78**, `414c53` **78/78**. 바꾼 호출: mode 0은 assign1-5-8, csched007(2), csched008, gmu-35-40, gmu-35-50(2), mad, neos-3627168-kasai, neos-4954672-berkel, neos-860300(2), neos859080, ran14x18-disj-8(경로 A·B, C1·C2, 채움, 양·음 선택). mode 1은 neos-860300 첫 호출(136 열, obj·bound·vtype 'B'→'I', type 10 레코드 136개 바이트 일치). sweep에서 `cancel_rows` 첫 불일치 5개가 모두 사라졌다(default 9 → 11/38).
- 지나지 않은 경로: 타이머·1e7 예산 멈춤, 3ba9de flag, PRECANCELSELECT ≠ −1, NumericFocus ≠ 0, M ≠ 2, mode 1의 정수 피벗 규칙 일부.

## 6. `37f484` 집계, MIP 분기 (`00381f20`)

LP 포트(`crates/lp/src/presolve/agg.rs`, [m1_presolve_agg.md](../m1/m1_presolve_agg.md))가 CSR을 연결 리스트로 바꿔 부르는 것과 같은 함수다. MIP는 상태 리스트에서 바로 부른다(+0xfc = 1, +0xe0 = 0). 포트는 LP 코드를 `crates/mip/src/aggregate.rs`로 복사해 아래 차이만 더했다(LP 하위 모듈은 private이고 `crates/lp`는 고치지 않는다). 리스트·free/pending은 호출 동안 상태에서 꺼내 LP의 `Mat`으로 쓰고 끝에 되돌린다.

### 6.1 LP와 다른 점 (MIP, 기본값)
| | LP (`519a52` 안) | MIP (`3cd406` 바깥 패스) |
|---|---|---|
| prescan(m1 §3.1) | 첫 호출 | 안 함. +0x488 = −1, tolA = 1e-6, tolI = FeasibilityTol |
| F(AggFill) | 10/20 | `3db19d`가 −1을 0으로 둔다 → 0 |
| 후보 목록 | 전체(`3707ac` 전파로 분류) 또는 `53c9ee` | 늘 `53af0f`(레벨 1 열, 캐시) → 유한성으로만 분류(lb 유한: ub 유한 0 / 무한 2, lb 무한: ub 유한 1 / 3). W = (|목록|·6)·μ + W |
| 행 캐시 | `local_170`, `local_168` NULL | `local_170`(정수 기준값) 있음, `local_168` NULL |
| 단계 A | | 아래 6.2 |
| 단계 B | cls 0 건너뜀 | 연속 열만, cls 0은 `378c30__0` && `__1`, 아래 6.3 |
| 단계 C | PREPAIRAGG 1, 3 | PREPAIRAGG 2, 3(기본 3). 비용 흔들기(bC) 없음, 정수 검사(6.4) |
| 단계 D | 함 | 안 함(+0xfc == 0 조건) |
| 레코드 | type 2 LP 꼴, 11, 21, 6 | type 2는 `39b9e0` MIP 분기(`helpers::push_subst_row`와 같음, 리스트를 걷고 W += (5·μ)·걸은), type 11 = {i32 r, i32 j, f64 |a|}(16 B) |
| 끝 | `519a52`가 activity 다시 계산 | +0x408/+0x404가 늘었으면 `53d3d9`. activity는 3cd406이 바로 뒤 `370047`로 다시 계산 |

### 6.2 단계 A의 정수 열
- `3773aa`(행 캐시)는 `local_170[r]`도 채운다: 살아 있는 열 중 'C'가 있으면 −1, 아니면 min|a|; 그다음 rhs/min|a|와 모든 a/min|a|가 정수(`x − floor(x + 1e-10) ≤ 1e-10`)가 아니면 −1. 둘째 걷기는 W += (μ·3)·걸은. 배율 한도로 멈추면 −1.
- `377cb6(…, isInt)`: 정수 열이면 `local_170[r]`(0이면 `3773aa` 다시)과 |a|가 1e-10 넘게 다르면 0. 단 row_cnt[r] == 2, |rhs| < 1e-10이면 다른 열을 찾으며 W += (2·건너뛴)·μ, 그 열에 플래그(+0x170)가 있고 |a_o/a| ≈ 1이면 2(`37f484` L1172–1249의 정수 쌍 경로). 기본값 38개에서는 플래그가 0이라 2가 나오지 않는다. 현재 코드는 2를 보존해 아래 처리를 적용한다.
- 추정 루프: `=` 행이 아니거나 row_cnt ≥ best인 노드에서 **이진 VUB 패턴**: nbin(+0xf0) ∈ [1, n), best ≥ 3, a < 0, vtype[j] == 'B', `<` 행, row_cnt 2, |rhs| ≤ 1e-10, inf_min = inf_max = 0, act_min − a ≥ −1e-10, act_max + a ≤ 1e-10이면 lim = best = 3(그래서 그 행만으로는 추정이 안 생긴다). a == −1이면 그 행의 다른 첫 열이 'B'일 때는 건너뛰고(W += (2·건너뛴)·μ) 아니면 3으로.
- 추정값: 정수 열은 `best·(c−2) + 1`, 연속은 `(c−2)·(best−1) + 1`.
- 피벗 고르기 조건: `(thr ≤ |a| && !int) || ρ·rmax ≤ |a| || (rmax·ρ2 ≤ |a| && cnt < 3 && !int)`.
- `3791fd`는 `local_170`도 무효화한다(행 r과 열 j의 행들).

- 결과 2 피벗을 뺀 뒤 남은 열 k에 타입·bound를 옮긴다. `a_k/a_j ≈ +1`: k가 C면 I로 바꾸고 `lb[k] = max(floor(lb[k]+IFT), −ub[j])`, `ub[k] = min(ceil(ub[k]−IFT), −lb[j])`. 비율 ≈ −1: k의 타입을 j로 바꾸고 같은 floor/ceil에 j의 원래 lb/ub를 교차 없이 교집합한다. 이는 원본의 **바깥 방향** floor/ceil이다. 타입이 바뀌면 k의 살아 있는 행들의 정수 캐시와 배율 상한 캐시를 무효화한다. 그 걷기에 `walk·3·μ`, 마지막에 `20·μ`를 더한 후 보통 `3791fd` 대입을 수행한다.

### 6.3 단계 B
- 열 걷기: row_cnt == 2인 행에서 그 행의 다른 첫 살아 있는 열이 'B'(또는 플래그 1)면 지금까지 모은 후보를 버리고 그 행만 남긴다(|a| > 1e6이면 포기). 그 행 다음 노드에서 멈춘다. `=` 행을 만나면 포기.
- 슬랙: 피벗 행의 옛 inf_min이 0이면 ub[j] = −(옛 act_min)/|a|(LP는 INF). 열 j의 새 기여는 이 bound로 더한다.

### 6.4 단계 C
현재 `ag_pair`는 통과한 쌍을 실제로 집계한다. 기본값 38개에서 이 단계까지 오는 행이 없다는 사실은 구현 완료의 근거로 사용하지 않는다.

- `395c97` MIP 가드: 짝 p/n 안에서 C 여부가 다르면 즉시 실패(작업량 없음). 목적 합 < −1e-10도 실패. 네 열이 모두 B이면 이 단계 전체를 건너뛰고 실패 표시를 남긴다. 쌍 검사를 통과한 뒤 네 열 중 정수가 있고 `|ρc − 1| > 1e-10`이면 그 행을 그만둔다(실패 표시 없음).
- 피벗 계수와 비율은 [1e-6, 1e6], growth 한도는 LP와 같다. `ub[n2]·ρ < ub[p1]`이면 첫 소거 열을 n2, 보존 열을 p1, f1 = 1/ρ로 바꾼다(아니면 p1 → n2, f1 = ρ). 둘째는 `ub[p2]·ρ ≤ ub[n1]`이면 p2 → n1, f2 = 1/ρ, 아니면 n1 → p2, f2 = ρ. free 노드가 두 소거 열 길이의 합보다 적으면 GC하고 그래도 적으면 포기한다.
- `39d14c`의 MIP 분기는 **type 21이 아니라 type 2 두 개**다. 각 소거 열 e마다 `{e, c0=0, ind=[k], val=[1/(1/f)]}`를 push하고 growth를 갱신한다. 원본은 값 −1인 합성 노드와 피벗 1/f로 이 레코드를 만들므로 역수 두 번의 순서도 유지했다. 각각 `5·μ`, 열 제거 +2·행 제거 +1. LP의 비용 흔들기는 적용하지 않는다.
- 행과 두 소거 열을 추적기에 표시한다. 정수가 하나라도 있으면, e가 B 또는 k가 C일 때 k의 타입을 e로 바꾸고 `ub[k] = floor(ub[e]+IFT)`. 전부 C이면 무한 bound를 유지하거나 `ub[e]/f`로 둔다. lb는 그대로다.
- 네 항의 행 노드를 죽이고 행 count = −3. 목적은 `obj[k] = f·obj[e]+obj[k]`, `obj[e]=0`. 보존 열 count −1, 소거 열 count = −3−r. 소거 열의 다른 노드를 f배로 보존 열에 접는다. 없는 항은 |v| ≥ 1e-13일 때 행·열 머리에 채우고, 합이 `max(1,10|v|)·1e-13` 이하이면 죽인다. 소거 열은 pending, 보존 열은 compact한다. activity는 호출자 `370047`에서 다시 계산한다.

### 6.5 오라클 확인
`mip_dump_check.py 37f484`: **78/78**. 바꾼 16호출: 단계 A 대입(csched007 71열, csched008, gmu-35-40/50, ic97_potential, neos-2657525-crna, neos-4338804-snowy, neos-4954672-berkel ×2, neos859080 40열, sp150x300d, timtab1, tr12-30 ×2), 단계 B 슬랙(graphdraw-domain 20열, type 11 레코드). 단계 B MIP 분기를 빼면 32호출이 작업량만 틀렸다(cls 0 `378c30` 작업량, 이진 2항 규칙). sweep의 `aggregate` 첫 불일치 2개가 사라졌다(graphdraw-domain 기본값 일치, default 12/38).

## 7. `4030c9` 지배 열 (`dominate.rs`, 기존 HEAD 구현 재검토)

`4030c9(st, env, T17)`은 `53c422` 후보 열의 연결 리스트를 열 CSR로 복사하고 `7722f`로 행 CSR을 만든다. 후보는 살아 있고 colflag가 0인 열이다. 비등식 행이 없거나, 후보 nnz 합 > 2e9 또는 후보 > 10개에서 평균 길이 > 5000이면 끝난다. 각 열에 `=`/양/음 계수 행의 `row % 31` 비트 서명을 만들고, `3dcc78`의 함의 bound가 원래 bound를 1e-9 안에서 지지하면 이동 방향 플래그를 준다.

- 단계 1: 함의 bound 쪽으로 움직일 수 있는 열 j가 다른 열 k를 지배하면 k를 lb 또는 ub에 고정한다. 방향별 가장 짧은 제한 행을 고르고, 길이 < 51 또는 활성 후보 < 6이면 그 행의 후보를 직접 비교한다. 그 밖에는 `dom_build`가 후보 부분집합의 행 CSR을 만들고 더 짧은 부분집합 행에서 비교한다.
- 단계 2: 유한한 bound(절댓값 ≤ 1e8)가 있는 j가 다른 열에 지배되면 j 자신을 고정한다. `=` 행에 있는 j는 이 단계에서 제외한다. 큰 행은 단계 1과 같은 부분집합 방식이며 고정 방향 플래그를 각각 소모한다.
- `40a768` (`dom_check`): 방향을 곱한 목적이 더 나쁘면 거절, 지배하는 열이 정수인데 상대가 C이면 거절, `=` 서명과 방향별 부호 서명으로 먼저 거절한다. 이어 두 열의 모든 행에서 방향별 계수를 양방향으로 비교한다(`=`은 차이 ≤ 1e-10, `<`은 지배 부등식). 통과하면 `3de18e`에 해당하는 공통 `fix_col`이 실제 열 제거·rhs·목적 상수·activity·추적기 및 type 1 레코드를 처리한다.
- 행 CSR 전치 `7722f`의 작업량에는 μ를 곱하지 않는다. 후보 탐색·서명·비교·부분집합은 원본 각 루프의 곱 순서를 유지한다. 타이머와 매 1e7 work의 예산 점검은 `dom_poll`에 있다.

현재 코드로 기본값 38개 상태를 **78/78 재비교**했다. 새 `dominated`는 작은 행에서 y를 0에 고정(type 1 한 개), `dominated_large`는 60항 행·활성 후보 ≥ 6에서 부분집합 경로를 거쳐 59열을 고정(type 1 59개)한다. 두 합성 상태 모두 **2/2 일치**: bound, 목적, rhs, row/col 리스트, nnz/free/pending, 카운터, activity, 추적기와 작업량 및 새 레코드 바이트를 비교했다. `dominate.rs` 자체를 이번에 다시 구현한 것은 아니며, 기존 `bd6d2e3` 구현을 검토하고 누락됐던 문서·새 회귀 검증을 추가했다.

## 8. 이번 보완의 검증 및 남은 범위 (2026-11-06)

### 8.1 재현 가능한 새 분기 검증

`tools/outer_b_oracle.py`가 모델과 파라미터·플래그 주입 명세를 관리한다. `bench/oracle/mip_outer_b/`에 LP 입력 및 진입/복귀 JSON gzip을 함께 저장했다. 원본은 첫 전체 activity 계산 직후 함수를 직접 부른다(`mip_state_dump.js --direct`). `4030c9`에는 한도 0 타이머를 준다. 플래그 입력은 SOS/반연속 전체 파이프라인 대신 해당 열의 플래그 값만 양쪽 진입 상태에 동일하게 넣는 **격리 검증**이다. 수집은 `gurun.sh`를 통해 한 Gurobi 프로세스씩 순차 실행한다.

| 묶음 | 수 | 실제 확인한 경로 |
|---|---:|---|
| `dual_*` | 7 | ± 계수/이진 부호 4조합, 절편, 폭 2 및 0.5, 두 후보 연속 집계, 기존 항 상쇄와 새 노드 채움, 배율 거절, 첫 패스만 검사 |
| `protected_*` | 6 | 정수 단계 A의 비율 ±1, 플래그 1 및 4에서 실제 대입·타입 변경·bound floor/ceil, 플래그 0에서 거절 |
| `pair_*` | 11 | 단계 C 연속 비율 2·3, 정수 비율 1, C/I 및 B/I 타입 합치기, 소수 bound floor, 채움·상쇄, 정수 비율≠1·혼합 짝·모두 B·growth 한도 거절 |
| `dominated*` | 2 | 작은 행 실제 고정, 60항 큰 행 부분집합에서 59열 고정 |

**26/26 상태 일치**. 무변경 검사만 통과하는 것을 막기 위해 각 사례의 열 제거 수를 별도로 단언한다. 테스트는 원본 상태의 bounds/types, 계수와 리스트 순서, activity, 작업량, nnz/free/pending, 13개 카운터, 추적기 및 레코드 바이트를 비교한다(주소는 상대 포인터, 패딩 제외). 이번에는 Python 상태 출력의 `node_cap`·`chunk`도 노출하고 비교기에 추가했으며 `col_flag`도 명시적으로 비교한다.

```sh
bash tools/build_local.sh
PYTHONPATH=$PWD/python ../.venv/bin/python tools/outer_b_oracle.py --capture
PYTHONPATH=$PWD/python ../.venv/bin/python -m pytest -q tests/test_outer_b.py
```

첫째 명령은 이 워크트리의 확장만 빌드한다. 둘째는 Gurobi/Frida가 있어야 하며, 저장한 픽스처만 검사하려면 `--capture`를 뺀다. 셋째는 Gurobi 없이 실행되며 최종 **26 passed**다.

### 8.2 기존 모델 회귀와 공통 검사

- 현재 최종 코드로 기존 38개 모델의 `418542`, `37f484`, `4030c9`, `414c03`, `414c53`, `36ee92`, `3e40d1` 진입/복귀 덤프를 재실행했다. **각각 78/78**, 합계 **546/546**. 이는 새 분기를 검증하는 위 26개와 별개의 회귀 검사다.
- `cargo test --workspace`: **43 passed**, 실패 0. `cargo clippy --workspace --all-targets -- -D warnings`: 통과(경고 0).
- `tools/build_local.sh` 후 `PYTHONPATH=$PWD/python ../.venv/bin/python -m pytest -q`: **61 passed, 1 xfailed**. 이후 추가한 큰 행 합성 사례를 포함해 `tests/test_outer_b.py`를 최종 다시 실행: **26 passed**. 기존 xfail은 그대로이며 새 예상 실패를 추가하지 않았다.
- 기존 PRELIMIT sweep 전체를 현재 코드로 재비교: **12/38 모델이 모든 저장 변화점과 기본값까지 일치**. 이 수는 기존 성과와 같으며 M2 전체 완료를 뜻하지 않는다. 남은 최초 불일치는 `xor`, `parallel`, `clique`, `ss2mip`, `probe0` 등 담당 밖 호출에 있다. 비교 도구가 출력하는 `default match`는 앞 변화점에서 실패하면 기본값 검사를 생략하므로, 엄밀히는 모든 체크포인트를 통과한 모델 수다.
- 상세 실행 로그는 세션 `/tmp/outer-b-{cargo,clippy-workspace,pytest,pytest-final-synth,regression-final,sweep}.log`. 영구 재현 입력은 위 저장된 합성 픽스처다. 기존 38개 덤프 및 sweep은 기존 세션 scratchpad를 읽기만 했다.

### 8.3 완료 판단과 제약

요청된 세 생략 분기(`418542`의 실제 `3bb6fe`, 단계 A의 플래그 정수 쌍, 단계 C의 MIP `39d14c`·타입·bound·열 병합)는 구현과 새 원본 비교를 마쳤다. `dominate.rs`의 마지막 구현도 §7과 회귀 입력으로 문서화했다. 담당 **선형 MIP outer B 보완은 완료**이며, M2 presolve 전체와 SOS/QC/일반 제약 파이프라인 완료를 주장하지 않는다.

- `3bb6fe`의 다른 호출자 `5431ee`와 즉시 activity 갱신/추가 통계 포인터 인터페이스는 outer A 범위다. 이번 helper는 `418542`가 실제 쓰는 mark 기반 경로를 구현한다.
- 원본 메모리 할당 실패(0x2711)는 Rust Vec 할당 실패로 재현하지 않는다. 타이머 소진·외부 콜백·1e7 예산 멈춤, 비기본 PRECANCELSELECT/NumericFocus/M 조합 전부를 새로 검증한 것은 아니다(§5.3). 큰 행 지배 열의 별도 단계 2 경로는 이번 새 합성 입력이 보장하지 않는다.
- 압축의 청크 사슬은 배열이 `node_cap`보다 길어진 것으로 표현한다. 기존 실제 압축 호출은 다시 일치했지만, 청크 증가 후 realloc 경로의 별도 원본 합성 검증은 여전히 없다. 이는 미구현 분기가 아니라 저장소의 arena 표현과 미검증 경로다.
- 이번 신규 검증은 함수 격리 상태와 레코드 생성 비교다. MIP postsolve 전체 실행이나 플래그가 있는 SOS/반연속 모델의 end-to-end 동등성을 검증한 것으로 확대 해석하지 않는다.

변경은 `m2-outer-b`에만 커밋하며 `crates/lp` 기존 파일 및 `docs/11_progress.md`, 다른 워크트리, 공유 `.venv`는 수정하지 않았다. 병합·브랜치/워크트리 삭제도 수행하지 않는다.
