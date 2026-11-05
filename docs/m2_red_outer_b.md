# M2 MIP presolve 바깥 패스 B 해독 (`4030c9`, `40d2bc`, `36ee92`, `3e40d1`, `37f484` MIP, `418542`)

`3cd406` 바깥 패스 가운데(m2_spine_b.md §2, 카운트 20–28)의 reduction 해독과 포팅 노트. 표기·규칙은 [m2_red_inner.md](m2_red_inner.md) §1과 같다(`W += x·μ`, 괄호가 곱 순서, `gup`/`gdn`, 노드·리스트·추적기, 보조 함수 §1.4–1.9와 §11 API). 근거는 디컴파일(`re/decomp/c/`)과 `objdump`(주소는 파일 오프셋), 런타임 근거는 `tools/mip_state_dump.js`로 뜬 기본값 38개 모델의 모든 호출(진입·복귀 상태)이다. 바깥 패스는 추적기 레벨 1이다(`538f30`·`5399fc`·`53af0f`는 레벨 1 목록과 캐시).

## 0. 요약

| 함수 | 크기 | 가드(3cd406) | 하는 일 | 38개 호출 | 덤프 비교 |
|---|---|---|---|---|---|
| `3e40d1(st, env)` | 2,622 | DualReductions | 쌍대 논리로 `<` 행을 `=`로 | 78(바꾼 것 5) | 78/78 |
| `418542(st, env)` | 2,645 | Aggregate && DualReductions | 쌍대 집계: 행의 이진 열과 다른 열 묶기(`3bb6fe`) | 78(`3bb6fe` 0회: 작업량만) | 78/78 |
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

### 3.3 오라클 확인과 지름길
`mip_dump_check.py 418542`: **78/78**(작업량 비트 일치, 둘째 pass를 도는 작은 모델 포함). 포트는 `3bb6fe`를 옮기지 않았다(`shortcut:`): 후보에 닿으면 집계하지 않은 것으로 센다. `5431ee`의 SCC 분기와 함께 옮길 것.

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
