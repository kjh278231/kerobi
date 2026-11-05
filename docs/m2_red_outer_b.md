# M2 MIP presolve 바깥 패스 B 해독 (`4030c9`, `40d2bc`, `36ee92`, `3e40d1`, `37f484` MIP, `418542`)

`3cd406` 바깥 패스 가운데(m2_spine_b.md §2, 카운트 20–28)의 reduction 해독과 포팅 노트. 표기·규칙은 [m2_red_inner.md](m2_red_inner.md) §1과 같다(`W += x·μ`, 괄호가 곱 순서, `gup`/`gdn`, 노드·리스트·추적기, 보조 함수 §1.4–1.9와 §11 API). 근거는 디컴파일(`re/decomp/c/`)과 `objdump`(주소는 파일 오프셋), 런타임 근거는 `tools/mip_state_dump.js`로 뜬 기본값 38개 모델의 모든 호출(진입·복귀 상태)이다. 바깥 패스는 추적기 레벨 1이다(`538f30`·`5399fc`·`53af0f`는 레벨 1 목록과 캐시).

## 0. 요약

| 함수 | 크기 | 가드(3cd406) | 하는 일 | 38개 호출 | 덤프 비교 |
|---|---|---|---|---|---|
| `3e40d1(st, env)` | 2,622 | DualReductions | 쌍대 논리로 `<` 행을 `=`로 | 78(바꾼 것 5) | 78/78 |

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
