# M2 MIP presolve 내부 루프 reduction 해독 (`3cd406` L1731–2030)

`3cd406`의 내부 루프에서 매 반복 이 순서로 불리는 reduction 7개와 보조 함수의 해독 노트. 포팅(뼈대 `crates/mip` 위)은 다음 단계다. 표기:
- `W += x·μ`는 `*(state+0x458) += x * *(state+0x448)`이다. 곱의 순서는 원본 그대로 적었다(`(double)n * 7.0 * μ`면 `(n·7)·μ`). μ = 1인 모델이 대부분이라 순서 차이는 μ ≠ 1에서만 드러난다.
- `gup(a, t)`/`gdn(a, t)`는 LP 포트의 `guarded_add(a, t, true/false)`(`crates/lp/src/presolve/state.rs`)와 비트 단위로 같다. 합이 부정확하면 max 쪽은 위로, min 쪽은 아래로 1 ulp(×(1±2⁻⁵²)) 민다.
- 파일 오프셋 = Ghidra 주소 − 0x100000. `LAB_004bxxxx`는 파일 0x3bxxxx.
- "확인"은 `mip_state_dump` 진입 상태를 Python 모형(세션 스크래치 `inner/bin/sim.py`)에 넣어 돌린 결과가 복귀 상태와 모든 필드(배열 비트, 리스트 순서, 개수, 카운터, 작업량, 추적기, 레코드)에서 같았다는 뜻이다(§9).

## 0. 요약

| 함수 | 크기 | 가드(3cd406) | 하는 일 | 오라클 |
|---|---|---|---|---|
| `3e39a6(st, env)` | 9,318 | 없음(DualReductions=0이면 고정 열만, PRESINGLECOL은 singleton 열만 거른다) | 열 패스: 고정·빈·singleton 열, 쌍대 고정 | 38개 147/147(§2.4) |
| `36ddf2(st, env)` | 1,965 | 없음 | 빈 행, 중복 행, 작은 계수 | 38개 185/185(§3.4) |
| `37168e(st, env)` | 4,058 | PREBOUND(env+0x2544) ≠ 0 | activity로 bound 강화, 정수 반올림. forcing 행은 NumericFocus ≥ 1에서만 | 38개 225/225(§4.4) |
| `36c5cf(st, env, −1, NULL, &cnt)` | 2,503 | 없음 | singleton 행 → bound(`=`면 고정) | 38개 147/147(§5.4, 경로는 좁음) |
| `56f793(st, env)` | 2,349 | PRECOEFFICIENT(env+0x2548) ≠ 0 | 정수 열 계수 강화 | 38개 147/147(§6.4) |
| `56c80e(st, env, 0, &status)` | 6,228 | Aggregate(env+0x242c) ≠ 0 | doubleton 등식 집계(사슬에 쌓았다가 노드 열 번호 바꾸기) | 13개 72/72 + 합성 31(§7.4) |
| `36e6b6(st, full)` | 770 | 없음 | 죽은 노드 GC | 38개 225/225(§8.4) |

반복 안의 순서(3cd406 L1731–2030, spine-b 노트가 제어 흐름 전체를 다룬다): `3c546c` 스냅숏, 추적기 레벨 2(`536971(tr, 2)`)와 reset(`5369b8`), `3e39a6`, `36ddf2`, [PREBOUND] `37168e`, `36c5cf`, [`local_950` > 1e6이면 `36e6b6(st, 1)` + `36ddf2`], 인라인 이진 표시(아래), [PRECOEFFICIENT] `56f793`, [Aggregate] `56c80e`, `36e6b6(st, 0)`, `3c5595` 진척도. 각 reduction 뒤 PRELIMIT 카운터가 1 늘고 `3ba5ac`/`3ba9de` 검사가 끼어 있다.
- 인라인 이진 표시(L1892–1935): `5383d0`이 준 열 목록(레벨 2 a-목록, `local_938`/`local_974`)의 열 j마다 `col_cnt[j] ≥ 0 && vtype[j]=='I' && lb[j]==0.0 && ub[j]==1.0`이면 vtype = 'B', `5379f1(tr, j)`, `+0x40c += 1`. 끝에 `W += (방문 수·4)·μ`.
- `36c5cf`의 다섯째 인자(3cd406의 rsp+0x348, Ghidra `local_950`)에 그 호출에서 지운 비영 수가 들어간다(덮어쓰기). 3cd406은 그 값이 1e6보다 크면(asm 0x3b1fae `cmpq $0xf4240`) `36e6b6(st, 1)`과 `36ddf2`를 한 번 더 돌린다(§5.2 끝).

## 1. 공통 규칙

### 1.1 상태 필드 (이 노트에서 쓰는 것)
| 오프셋 | 내용 |
|---|---|
| +0x0c / +0x10 | n / m |
| +0x60 / +0x68 | row_cnt / col_cnt (살아 있는 비영 수. 음수 = 지움. 행: −1 빈 행·singleton 행(36c5cf)·집계 행(56c80e), −2 중복 행(36ddf2)·forcing 행(37168e), −3 singleton 열 대입(3e39a6). 열: −1 빈 열, −2 고정, −3 집계(56c80e), −3−r 행 r로 대입(3e39a6)) |
| +0x70 / +0x78 | inf_max / inf_min (max/min activity의 무한 항 수) |
| +0x90 / +0x98 | act_max / act_min = (유한 부분 max/min activity) − rhs |
| +0xa0 / +0xa8 / +0xb0 | lb / ub / obj |
| +0xb8 / +0xc0 | rhs / sense(`<` 또는 `=`. `>` 행은 생성 때 `<`로 바뀐다) |
| +0xc8 | objcon |
| +0xd8 | nnz 카운터(m1_presolve_agg §2.3 규칙: 노드가 열 리스트에서 빠질 때만 −1) |
| +0xfc | MIP 플래그. 3cd406 상태에서 1(38개 모두). 0이면 LP식 분기(레코드 내용, 36ddf2 계수 기준) |
| +0x150 | vtype(`C`/`B`/`I`, …). NULL이면 모두 `C` 취급(37168e·36c5cf는 NULL 검사, 56f793은 검사 없음) |
| +0x170 | 열 플래그(u32). 기본 38개에서 진입 때 모두 0(§2 `dump2.js`). 0x08·0x80(PWL 목적)·0x18·0x30(QC) 비트를 읽는다 |
| +0x178 / +0x180 | 행 머리 / 열 머리(노드 32 B: f64 값, i32 행 @+8, i32 열 @+0xc(−1 죽음), 행 다음 @+0x10, 열 다음 @+0x18) |
| +0x188/+0x190, +0x198/+0x1a0 | pending 개수/머리, free 개수/머리 |
| +0x1e8 | IntFeasTol(= env+0x2468, `3a484d` L1136) |
| +0x250/+0x260/+0x278/+0x27c | Q 목적, Q 열, QC 관련. 기본 38개에서 0 |
| +0x368 / +0x370 | 레코드 arena / 레코드 목록 머리(새것이 앞) |
| +0x3b8 | 행 크기 정수 작업 배열 |
| +0x3d8 | 추적기 |
| +0x3f8.. | 카운터: +0x3f8 bound 강화, +0x3fc 계수 변경, +0x400 부등식→등식, +0x404 행 제거, +0x408 열 제거, +0x40c I→B |
| +0x470 / +0x478 | 불가능을 낸 열 / 행 |

env: +0x2458 INFBOUND(1e30, 무한 bound는 ±1e30로 저장된다), +0x2460 FeasibilityTol(ft), +0x2468 IntFeasTol(ift), +0x2470 OptimalityTol, +0x2718 NumericFocus(nf), +0x2544 PREBOUND, +0x2548 PRECOEFFICIENT, +0x242c Aggregate, +0x2430 DualReductions, +0x257c PRESINGLECOL.

### 1.2 추적기와 목록
- 내부 루프는 레벨 2다. 반복 머리에서 `5369b8`(reset)이 레벨 2 목록을 "지난 반복에 표시된 것"으로 압축하고 a = b로 둔다. 반복 안의 표시(`5379f1`/`53763f`)는 목록 b 뒤에 붙고 a는 그대로다. 그래서 **이번 반복의 목록은 reset 때 고정**되고, 이번 반복의 표시는 다음 반복 목록이 된다. 표시 함수는 `tracker.rs`의 `mark`와 같다(플래그 비트 1/2 레벨 1, 4/8 레벨 2).
- `538f30`(행 목록)·`5399fc`(열 목록)는 결과를 레벨별로 캐시한다(행 tr+0x94+4L/+0xa0+8L, 열 tr+0xb8+4L/+0xc8+8L, −1 = 없음, reset 때 지움). 한 반복에서 처음 부르는 함수만 계산하고 작업량을 낸다. 내부 루프에서 `538f30`을 처음 부르는 것은 `36ddf2`이고, `37168e`·`56f793`·`56c80e`는 같은 캐시(36ddf2 시점의 목록)를 받는다. 캐시 안에는 그 뒤에 지워진 행도 남아 있으므로 각 함수가 `row_cnt`를 다시 본다.
- **연결 리스트 분기(+0xe0 = 0)**: 조밀 판정(`n ≤ 5·nc || m ≤ 4·nr`, 아니면 `m < 3·nr + Σ max(col_cnt,0)`)과 결과·작업량 순서는 LP 포트 `rows_list`와 같고, 희소 경로에서 표시된 열을 도는 부분만 다르다: 열 리스트(+0x18)의 모든 노드를 방문하며 살아 있는 노드의 행을 넣고 **열마다 `W += (μ·5)·방문`**(죽은 노드 포함, CSR은 `(μ·3)·len`). `5399fc`는 대칭으로 표시된 행마다 행 리스트를 돌며 **`W += (μ·4)·방문`**(§2.2.7).
- `538353`(행)·`5383d0`(열)은 레벨의 원시 a-목록(죽은 것 포함, 작업량 없음)을 준다. `36c5cf`, `36e6b6(…, 0)`, 인라인 이진 표시가 쓴다.
- `537da8(tr, st, i)`(행 i의 열 표시): 행 리스트 노드를 모두 방문해 살아 있는 열을 `5379f1`처럼 표시, `W += (μ·4)·방문`. `53802c(tr, st, j)`(열 j의 행 표시): 열 리스트를 모두 방문해 살아 있는 노드의 행을 표시, `W += (μ·5)·방문`. CSR 분기는 둘 다 `3·len`.

### 1.3 노드 연산
내부 루프 reduction은 노드를 지울 때 `node.col = −1`만 쓰고 `row_cnt`/`col_cnt`를 줄인다. 노드가 리스트에서 빠져 pending/free로 가고 +0xd8이 주는 것은 `36e6b6`에서만 일어난다. `56c80e`는 새 노드를 만들지 않고 살아 있는 노드의 열 번호를 바꿔 루트 열 리스트 앞으로 옮긴다(§7.2.6, +0xd8·free·pending 그대로).

### 1.4 `3dd6f1(ol, ou, nl, nu, INF, st, j)`: bound 변경의 activity 증분 (2,906 B)
Ghidra는 앞의 double 다섯 개를 보여 준다(37168e·36c5cf의 호출부 asm에서 xmm0–4 = 옛 lb, 옛 ub, 새 lb, 새 ub, INF 확인). 열 j 리스트(+0x18) 순서로 살아 있는 노드(행 r, 계수 a)마다:
```
case (nl == ol, nu == ou):
  (true, true)  => return            // 작업량도 없음
  (true, false) => ops = [U-, U+]
  (false, true) => ops = [L-, L+]
  _             => ops = [U-, L-, U+, L+]
for op in ops:                        // 노드마다 이 순서
  b = 그 bound(옛/새), 유한 = (U: b < INF, L: -INF < b)
  쪽 = a <= 0 ? (U → min, L → max) : (U → max, L → min)
  항 = 빼기면 -(b*a), 더하기면 a*b      // 곱은 교환법칙상 비트 같음
  유한이면 act_max[r] = gup(act_max[r], 항) / act_min[r] = gdn(act_min[r], 항)
  아니면 inf_max[r] / inf_min[r] 을 빼기 −1, 더하기 +1
W += (μ·6)·방문(죽은 노드 포함)        // 바뀐 것이 있을 때만
```
1e-13 영 처리와 행 표시는 없다(호출자가 열을 표시한다). LP `4fe49b`와 순서·반올림이 같고 CSR 대신 열 리스트를 돈다.

### 1.5 `3de18e(v, INF, env, st, j, rec)`: 열 고정 (2,681 B)
asm: xmm0 = v, xmm1 = INF, rdi = env, rsi = st, edx = j, ecx = rec.
```
ol = lb[j]; ou = ub[j]
if colflag[j] & 0xffffff77 == 0 {
    if rec != 0 { (LP식이면 3a0f5b) ; 39ab33(v, env, st, j)?  실패 → return 0x2711 }
} else { rec = 0 }                       // 다른 플래그 비트가 있으면 레코드 없이
objcon = obj[j]*v + objcon; obj[j] = 0
[Q 목적 +0x250, Q 열 +0x260: 기본 38개에서 안 돎]
colflag[j] &= !0x08
[colflag & 0x80(PWL 목적) 블록: 안 돎]
if col_cnt[j] >= 1 {
    53802c(tr, st, j)                    // 행 표시, W += (μ·5)·방문
    for node in col j list (+0x18) where node.col >= 0 {
        a = node.val; t = v*a; r = node.row
        node.col = -1; row_cnt[r] -= 1; rhs[r] = rhs[r] - t
        if a <= 0 { if ou < INF { act_min[r] = gdn(act_min[r], -(ou*a)) } else { inf_min[r] -= 1 }
                    if -INF < ol { act_max[r] = gup(act_max[r], -(a*ol)) } else { inf_max[r] -= 1 } }
        else      { if ou < INF { act_max[r] = gup(act_max[r], -(ou*a)) } else { inf_max[r] -= 1 }
                    if -INF < ol { act_min[r] = gdn(act_min[r], -(a*ol)) } else { inf_min[r] -= 1 } }
        act_max[r] = gup(act_max[r], t); act_min[r] = gdn(act_min[r], t)   // rhs가 t만큼 줄어든 몫
    }
    W += (col_cnt[j]·7)·μ                // 방문 수가 아니라 col_cnt(고정 전 값)
    if rec == 0 { if col_cnt[j] > 0 { col_cnt[j] = 0; 5379f1(tr, j) } }    // 빈 열로 남긴다
    else { col_cnt[j] = -2 }
} else if rec != 0 { col_cnt[j] = -2 }
lb[j] = v; ub[j] = v
return 0
```
노드는 리스트에 남는다(col = −1). `+0x408`(열 제거)은 `39ab33`이 올린다. LP `4fa83a`와 같은 일을 하지만 activity를 노드마다 "옛 bound 기여 빼고 t 더하기"로 고친다(4fa83a는 CSR).

### 1.6 `39ab33(v, env, st, j)`: type 1 레코드 (582 B)
- MIP(+0xfc ≠ 0 또는 +0x278 ≠ 0): `+0x408 += 1`, arena(+0x368)에서 0x28 B. 레코드 = {i32 type=1 @+0, data 포인터 @+8(= rec+0x18), next @+0x10(옛 머리)}, 목록 머리 = rec. data 16 B = {i32 j, 4 B 패딩(arena 내용), f64 v}. 작업량 없음. 반환 0 / 0x2711.
- LP식(둘 다 0): {j, pad, v, obj[j], status(ub−lb < 1e-10: −4, lb < v < ub: −3, v ≥ ub: −2, v ≤ lb: −1), col_cnt, (row, val)…}. 3cd406에서는 안 돎.

### 1.7 `370047(st, env, len, list)` / `36fcb5`: activity 재계산 (702 B / 747 B)
LP 포트 `agg_activity`(agg.rs)와 같다. list = NULL이면 0..m−1. 행마다(`row_cnt ≥ 0`) `36fcb5(INF, st, i, &inf_min[i], &act_min[i], &inf_max[i], &act_max[i], W)`가 행 리스트 순서로 0에서부터 더하고(a ≤ 0: max += lb·a, min += a·ub; a > 0: max += ub·a, min += a·lb; 무한 bound는 개수), `W += (μ·7)·방문`. 그다음 `act_max = gup(max, −rhs)`, `act_min = gdn(min, −rhs)`, `0 < act_min < |rhs|·1e-13 → 0`, `act_max < 0 && −act_max < |rhs|·1e-13 → 0`. 끝에 `W += (μ·8)·반복 수`(len 또는 m). `36ddf2` 끝부분은 이 함수의 인라인 사본이다.

### 1.8 `3dcc78(INF, a, st, j, mode, single, r, &lo, &hi)`: 열의 암시 bound (1,990 B)
Ghidra가 둘째 double(a, xmm1)을 보여 준다. 내부 루프에서는 `3e39a6`이 mode = 1, single = 1로 부른다. 바깥(3cd406 L3120 등)은 mode = 2.
```
lo = -INF; hi = INF; it = vtype[j] (NULL이면 'C'); ift = st+0x1e8
if mode & 1 {
  if single == 0 {                         // 열 j의 모든 행
    for node in col j list where live: (a, r) = (node.val, node.row)
      if a > 1e-9 {
        if inf_min[r]==0 { c = lb - act_min[r]/a; hi = (c <= hi) ? c : hi }
        else if inf_min[r]==1 && lb <= -INF { c = -act_min[r]/a; hi = (c <= hi) ? c : hi }
        if sense[r]=='=' { if inf_max[r]==0 { c = ub - act_max[r]/a; lo = (lo <= c) ? c : lo }
                           else if inf_max[r]==1 && INF <= ub { c = -act_max[r]/a; lo = (lo <= c) ? c : lo } }
      } else if a < -1e-9 {
        if inf_min[r]==0 { c = ub - act_min[r]/a; lo = max식 } else if inf_min[r]==1 && INF <= ub { c = -act_min[r]/a; lo = max식 }
        if sense[r]=='=' { inf_max 0: c = lb - act_max[r]/a → hi; 1이고 lb <= -INF: -act_max[r]/a → hi }
      }
    W += (μ·7)·방문
  } else {                                  // 행 r 하나, 계수 a
    같은 식을 행 r에만(초기값 -INF/INF에서 한 번), W += μ·3
  }
  if it != 'C' { lo = ceil(lo - ift); hi = floor(hi + ift) }
}
if mode & 2 && colflag[j] & 0xfffffffc == 0 {      // 쌍대(목적) 논리
  c = obj[j]; U2 = (-1e-10 < c) ? lb : INF; L2 = (c < 1e-10) ? ub : -INF; v = 0
  for node in col j list {
    if live { r = node.row
      if sense[r]=='=' { U2 = INF; L2 = -INF; break }           // 방문 수에 안 셈
      a = node.val
      if a < -1e-13 { if inf_max[r]==0 { t = lb - act_max[r]/a; if it!='C' { t = ceil(t - 1e-13) }; U2 = (U2 <= t) ? t : U2 } else { U2 = INF } }
      else if a > 1e-13 { if inf_max[r]==0 { t = ub - act_max[r]/a; if it!='C' { t = floor(t + 1e-13) }; L2 = (t <= L2) ? t : L2 } else { L2 = -INF } }
    }
    v += 1
  }
  W += (μ·4)·v
  if colflag[j] & 3 != 0 { U2 = (U2 <= 0) ? 0 : U2; L2 = (0 <= L2) ? 0 : L2 }
  (l2, u2) = (L2, U2)
  if U2 < L2 { if 0 < U2 || L2 < 0 { x = (|L2| <= |U2|) ? L2 : U2; (l2, u2) = (x, x) } else { (l2, u2) = (0, 0) } }
  lo = (lo <= l2) ? l2 : lo; hi = (u2 <= hi) ? u2 : hi
}
*lo_out = lo; *hi_out = (hi <= lo) ? lo : hi
```

### 1.9 내부 루프 밖에서만 불리는 보조 함수
- `3df916(ft, env, st, j)`(1,472 B, 3cd406 FINAL L7667, 2개 모델 77회): `3e39a6` 앞부분의 "거의 고정된 열" 블록의 독립 사본(§2). r = ub−lb. `ft+ft < −r`이면 +0x470 = j, 3. `1e-13 < r`이면 amax = max|a|(살아 있는 노드, `amax <= |a|`면 갱신), `W += (n·3)·μ`, `ft < amax·r`이면 0(그대로). 고정값 v = `ft·0.01 <= |r·c|`면 (c ≤ 0 ? ub : lb), 아니면 (lb > 0 || ub < 0 ? (ub+lb)·0.5 : 0). 마스크 0x88(+0x248 ≠ 0이고 lb < 0.5면 0x888)로 `colflag & !mask == 0`이면 `39ab33(v)` 레코드와 colflag = 0. objcon = c·v + objcon, obj = 0, `53802c`, 열 노드마다 rhs[r] = rhs[r] − v·a, row_cnt[r]−−, col = −1, `W += (n·6)·μ`, col_cnt = (colflag == 0) ? −2 : 0. **activity는 고치지 않는다**(lb ≈ ub라 max−rhs, min−rhs가 그대로라고 보는 것).
- `3a061e(d1, d2, env, st, i)`(131 B, 3cd406 FINAL L8945와 바깥 reduction `57627e`·`579e67`·`574764`·`54807f`): type 9 레코드, 0x30 B, data 24 B = {i32 i, 패딩, f64 d1, f64 d2}. 3cd406 L8945는 (0, …)으로 부른다.

## 2. `3e39a6` 열 패스 (`004bfa50`)

`3e39a6` (file+0x3bfa50, ghidra 004bfa50, 9,318 B). 보조: `3e2b99` 쌍대 고정(2,052 B), `39b9e0` MIP 분기(type 2 레코드), `3a0f5b`(Q 목적, 안 돎), 추적기 `5399fc`/`53802c`/`537da8`의 연결 리스트 분기. 공용 보조 `3dcc78`(§1.8), `3de18e`(§1.5), `39ab33`(§1.6)는 의미만 적는다.

표기: μ = `+0x448`, `W += x` 는 `*(+0x458) = x + *(+0x458)`(포인터가 NULL이면 생략, presolve에서는 늘 있음). `(a·b)·μ`처럼 괄호가 곱셈 순서다. 노드 = {v, row, col(−1 죽음), next_row(+0x10), next_col(+0x18)}. "행 r 리스트"는 `+0x178[r]`에서 +0x10으로, "열 j 리스트"는 `+0x180[j]`에서 +0x18로 걷는다. 방문 수는 죽은 노드도 센다.

### 2.1 서명·가드·반환값

- `3e39a6(st, env) -> i32`. `3cd406` 내부 루프 첫 reduction(L1771, 인자 `(state, env)`). L4354·L7344의 다른 호출은 spine 노트.
- 자체 파라미터 가드는 없다. DualReductions(env+0x2430)=0이면 고정 열(A)만 처리한다. PRESINGLECOL(env+0x257c)은 singleton 열(C)만 거른다.
- 반환값
  - 0: 정상.
  - 3: 불가능. `FT + ub < lb`, 또는 고정 열에서 `2·FT < lb − ub`(앞 검사 때문에 사실상 안 걸림). `+0x470 = j`.
  - 5: 무한. 빈 열이 비용을 줄이는 쪽 bound가 무한일 때, 또는 `3e2b99`. `+0x470 = j`.
  - 0x2711: 레코드 할당 실패(`39ab33`, `39b9e0`, `3a0f5b`).
  - `3de18e`·`3e2b99`가 돌려준 값(PWL이면 3, 기본에서는 0).
- 3/5/0x2711로 일찍 나가면 끝의 `W += 6·len·μ`와 행 캐시 지우기를 건너뛴다(캐시 `+0x3b0`가 남는다. presolve는 거기서 끝난다).

env·상태 값:

| 이름 | 위치 | 기본값 |
|---|---|---|
| FT | env+0x2460 FeasibilityTol | 1e-6 |
| OT | env+0x2470 OptimalityTol | 1e-6 |
| INF | env+0x2458 | 1e30 |
| AGG | env+0x26c0 GURO_PAR_AGGMULTLIMIT | 1e14 |
| NF | env+0x2718 NumericFocus | 0 |
| PSC | env+0x257c GURO_PAR_PRESINGLECOL | −1 |
| DR | env+0x2430 DualReductions | 1 |
| SUB | env+0x2728 sub-solve 깊이 | 0 |
| IFT | st+0x1e8 IntFeasTol(`3dcc78`가 씀) | 1e-5 |
| PC | st+0x1e0 PreCrush(`3e2b99`가 씀) | 0 |

- `fixtol` = NF==1이면 1.0000000000000001e-11, NF≥2이면 1e-12, 아니면 1e-10.
- `ld0`(local_d0, mode 1·2에서 목적 계수가 있는 행의 최대 길이):
  - PSC ≥ 4: `st+0xfc≠0`이면 n+1, 아니면 0.
  - PSC < 4: `st+0xfc≠0 && m > 10 && SUB==0 && PSC==−1`이면 10, 아니면 0.
  - MIP 기본: m > 10이면 10, m ≤ 10이면 0(markshare2: m=7이라 0).
- 상태 필드
  - `+0x80`/`+0x88`: 열마다 배율 하한·상한 mlo/mhi. 처음은 1.0이고 `39b9e0`이 갱신한다.
  - `+0x170`: 열 플래그 워드. 38개 모두 0이다.
  - `+0x390`(int 목록)/`+0x3b0`(행마다 f64): 정수 행 캐시. 호출 안에서만 유효하고 끝에 0으로 되돌린다.
  - `+0x404`/`+0x408`: 제거한 행·열 카운터.
  - `+0xfc`: MIP 플래그(1). `+0xe0`: 0(연결 리스트 상태).
  - `+0x248`·`+0x278`·`+0x250`(Q 대각)·`+0x260`(Q 비대각): 0 또는 NULL.
  - `+0x330`–`+0x358`: PWL 목적.

### 2.2 알고리즘

```rust
fn col_pass(st, env) -> i32 {
    let list = cols_list();                   // 5399fc, §2.2.7. 길이는 호출 때 고정
    let mut nrc = 0;                          // +0x390 목록 길이(local_68)
    for &j in &list {
        let cj = col_cnt[j];
        if cj < 0 { continue }
        let (u, l) = (ub[j], lb[j]);
        if FT + u < l { st.bad = j; return 3 }
        if u - l <= fixtol { fixed(j)?; continue }          // A (DR 무관)
        if !DR { continue }
        if flags[j] != 0 { e2b99(j)?; continue }             // 기본 38개에서 안 돎
        if cj == 0 { empty(j)?; continue }                   // B
        let mut r11 = 0;                                     // 1 = singleton의 행이 부등식
        if cj == 1 { match singleton(j)? { Done => continue, Skip(x) => r11 = x } }  // C
        if cj + r11 > 1 { e2b99(j)? }                        // D: cj ≥ 2, 또는 부등식 행 singleton을 못 지웠을 때
    }
    W += (6.0 * list.len()) * μ;
    if nrc > 0 { for r in +0x390[..nrc] { +0x3b0[r] = 0.0 }  W += (nrc * 3.0) * μ }
    0
}
```

- `cj`는 매번 `+0x68[j]`의 현재 값이다. 앞 열의 대입으로 줄어든 개수가 보인다(murg: 열 258–273은 열 17이 행 496을 지운 뒤 31로 보인다).
- 목록은 호출 시작에 한 번 만든다. 처리 중 죽은 열은 `cj < 0`으로 건너뛴다. 끝의 6·len은 건너뛴 항목도 센다.

#### 2.2.1 A: 고정 열 (`ub − lb ≤ fixtol`)

```rust
let d = ub[j] - lb[j];
if 2.0*FT < -d { st.bad = j; return 3 }
if 1e-13 < d {
    let (mut mx, mut cnt) = (0.0, 0);
    for nd in col_list(j) { if nd.col >= 0 { mx = if mx > |nd.v| { mx } else { |nd.v| } }  cnt += 1 }   // maxsd
    W += (cnt * 3.0) * μ;
    if mx * d > FT { continue }                  // 고정 안 함, 다음 열
}
let c = obj[j];
let v = if 0.01*FT <= |d * c| { if c > 0.0 { lb[j] } else { ub[j] } }
        else if 0.0 < lb[j] || ub[j] < 0.0 { (lb[j] + ub[j]) * 0.5 } else { 0.0 };
let mask = 0x88 | if st+0x248 != 0 && lb[j] < 0.5 { 0x800 } else { 0 };
if flags[j] & !mask == 0 {
    if flags[j] & 0x80 != 0 { /* PWL 목적: 기본 38개에서 안 돎 */ }
    if st.mip == 0 && st+0x278 == 0 && st+0x250 != NULL { 3a0f5b(v, env, st, j)? }    // LP+Q만
    39ab33(v, env, st, j)?;                      // MIP: type 1 {j, v}, +0x408++ (W 없음)
    flags[j] = 0;
}
objcon = (obj[j] * v) + objcon;  obj[j] = 0.0;    // Q 항(+0x250/+0x260): 안 돎
53802c(j);                                       // 살아 있는 노드의 행 표시, W += (μ·5)·방문
let mut cnt = 0;
for nd in col_list(j) { if nd.col >= 0 { rhs[nd.row] = rhs[nd.row] - v * nd.v; row_cnt[nd.row] -= 1; nd.col = -1 }  cnt += 1 }
W += (cnt * 6.0) * μ;
col_cnt[j] = if flags[j] == 0 { -2 } else { 0 };
```

- activity(`+0x70`–`+0x98`)는 건드리지 않는다. `+0x90`/`+0x98`이 activity − rhs라서 고정 열을 빼도 값이 그대로다(lb=ub=v일 때 정확히 같다).
- 노드는 col=−1만 되고 두 리스트에 남는다. `+0xd8`과 free/pending은 바뀌지 않는다(GC `36e6b6`가 정리).
- 열 자신은 표시하지 않는다.
- `fixtol` 판정은 `fixtol >= ub−lb`(comisd, NaN이면 거짓)다.

#### 2.2.2 B: 빈 열 (`col_cnt == 0`, DR)

```rust
let c = obj[j];
let v = if c > OT { if !(lb[j] > -INF) { st.bad = j; return 5 }  lb[j] }
        else if -OT > c { if !(ub[j] < INF) { st.bad = j; return 5 }  ub[j] }
        else if lb[j] > -INF { lb[j] } else if INF > ub[j] { ub[j] } else { 0.0 };
39ab33(v, env, st, j)?;                          // type 1 {j, v}, +0x408++
objcon = (v * obj[j]) + objcon;  obj[j] = 0.0;  col_cnt[j] = -1;
```

- W와 추적기 표시가 없다. lb·ub는 그대로 둔다.

#### 2.2.3 C: singleton 열 (`col_cnt == 1`, DR)

```rust
// PSC 게이트. vt = vtype[j] (배열이 없으면 'C')
match PSC as u32 { 0 => return Skip(0), 1 => if vt != 'C' { return Skip(0) }, 2 => if vt == 'B' { return Skip(0) }, _ => {} }
let (mut dead, node) = (0, 열 j 리스트의 첫 살아 있는 노드);   // dead = 그 앞의 죽은 노드 수
W += (dead + dead) * μ;
let (r, a, c, u, l) = (node.row, node.v, obj[j], ub[j], lb[j]);
if !(1e-6 <= |a| * (u - l)) && |c| * (u - l) > 1e-6 { return Skip(0) }
let s = sense[r];
let r11 = if row_cnt[r] == 1 { if s == '=' { return Skip(0) } 1 }    // singleton '=' 행은 36c5cf 몫
          else if s == '=' { 0 } else { 1 };
if r11 == 1 && ((c > 1e-9 && a > 0.0) || (c < -1e-9 && a < 0.0)) { return Skip(1) }   // → 3e2b99
let mut free = true;  let mut have = false;  let (mut lo, mut hi);
if l > -INF || u < INF {
    have = true;  (lo, hi) = 3dcc78(INF, a, st, j, 1, 1, r);         // 행 r만 본 x_j의 암시 bound, W += 3μ
    if (l > -INF && (l - 1e-9) > lo) || (INF > u && hi > 1e-9 + u) {
        free = false;
        let t = FT * 100.0;
        if lo > u + t || (l - t) > hi { return Skip(r11) }
        if s != '=' { return Skip(r11) }
    }
}
if vt != 'C' && !int_ok(j, r, a) { return Skip(r11) }               // §2.2.3.1
if free { mode0(j, r, a, c, node) } else { mode12(j, r, a, c, u, l, node, have, lo, hi) }
```

- Ghidra는 `row_cnt[r]==1 && s=='='`를 진행(local_198=1)으로 보여 준다. asm(3c0910 → 3c1e38: `cmpb $0x3d; je 3c0d7c`)은 건너뛴다. 오라클도 asm과 같다.
- `Skip(x)`는 col_cnt[j]=1로 D의 `1 + x > 1` 검사로 간다. 부등식 행(x=1)이면 `3e2b99`를 부른다.

##### 2.2.3.1 int_ok (정수 열: 대입한 식이 정수를 지키는지)

```rust
fn frac(q) = q - floor(1e-10 + q);
if row_cnt[r] < 10 {
    if !(1e-10 >= frac(rhs[r] / a)) { return false }
    let mut cnt = 0;
    for nd in row_list(r) {
        if nd.col >= 0 && (vtype[nd.col] == 'C' || 1e-10 < frac(nd.v / a)) { W += (cnt*4.0)*μ; return false }
        cnt += 1;
    }
    W += (cnt * 4.0) * μ;  return true;
}
let g = +0x3b0[r];
if g != 0.0 { return !(|a| > g) }                         // 이 호출에서 이미 계산한 행
let mut dd = |rhs[r]|;  if !(dd > 1e-10) { dd = 1e100 }
let (mut ok, mut cnt) = (true, 0);
for nd in row_list(r) { if nd.col >= 0 { if vtype[nd.col] == 'C' { ok = false; break }  dd = if |nd.v| < dd { |nd.v| } else { dd } }  cnt += 1 }
W += (cnt * 4.0) * μ;                                     // 'C'에서 멈췄으면 그 노드는 세지 않음
if ok && 1e-10 >= frac(rhs[r] / dd) {
    cnt = 0;
    for nd in row_list(r) { if nd.col >= 0 && 1e-10 < frac(nd.v / dd) { ok = false; break }  cnt += 1 }
    W += (cnt * 4.0) * μ;
} else { ok = false }
let g = if ok { dd + 1e-10 } else { 1e-10 };              // 실패 값 = 0x3ddb7cdfd9d7bdbb
+0x3b0[r] = g;  +0x390[nrc] = r;  nrc += 1;
!(|a| > g)
```

- 행 r에는 j 자신의 노드도 살아 있다(a/a = 1이라 통과).
- floor 인자는 `1e-10 + q`, 판정은 `1e-10 >= q − floor`다.
- `row_cnt < 10` 경로는 캐시를 쓰지 않는다.

##### 2.2.3.2 G: 배율 성장 검사 (mode0, mode12 공통)

```rust
let (mut mn, mut mx, inv) = (1.0, 1.0, 1.0 / AGG);  let mut cnt = 0;
for nd in row_list(r) {
    let k = nd.col;
    if k >= 0 && k != j {
        let t = |nd.v / a|;
        let v = { let x = mlo[j] * t; if mlo[k] < x { mlo[k] } else { x } };     // minsd
        if mn > v { mn = v; if inv > v { W += (cnt+cnt)*μ; return false } }
        let w = { let x = mhi[j] * t; if mhi[k] > x { mhi[k] } else { x } };     // maxsd
        if w > mx { mx = w; if w > AGG { W += (cnt+cnt)*μ; return false } }
    }
    cnt += 1;
}
W += (cnt + cnt) * μ;  true
```

- 실패하면 `Skip(r11)`. 계산한 mn/mx는 버린다(배열 갱신은 `39b9e0`이 한다).

##### 2.2.3.3 mode0: (암시) 자유 singleton 대입, 행 삭제

```rust
if !G { return Skip(r11) }
node.col = -1;
let c0 = rhs[r] / a;
39b9e0(c0, a, env, st, 0, j, r, rowhead[r], row_cnt[r] - 1)?;    // §2.2.5, +0x404++, +0x408++
objcon = (c0 * c) + objcon;
let f = c / a;
obj[j] = 0.0;  row_cnt[r] = -3;  col_cnt[j] = -3 - r;
let mut cnt = 0;
for nd in row_list(r) {
    if nd.col >= 0 { 5379f1(nd.col); obj[nd.col] = obj[nd.col] - nd.v * f; col_cnt[nd.col] -= 1; nd.col = -1 }
    cnt += 1;
}
W += (cnt * 5.0) * μ;  Done
```

- 행 r의 rhs·activity는 그대로다(행이 없어진다). 행 r은 표시하지 않는다.
- 부등식 행(r11=1)도 같은 식이다. 앞의 목적 부호 검사로 최적에서 행이 꼭 맞는다.
- c0 = rhs/a는 −0.0일 수 있다(murg: 0/−1). 레코드에 그대로 들어간다.

##### 2.2.3.4 mode12: `=` 행, 한쪽 bound가 암시되지 않음 → x_j를 행의 slack으로

```rust
let ac = |c|;
if ac >= 1e-10 && (1e-6 > |a| || ac > 1e6 || ld0 < row_cnt[r]) { return Skip(r11) }
let mode = if u >= INF { 1 } else if -INF >= l { 2 } else {
    if !have { (lo, hi) = 3dcc78(INF, a, st, j, 1, 1, r) }
    if 1e-9 + u > hi { 1 } else if lo > l - 1e-9 { 2 } else { return Skip(r11) }
};                                            // mode 1: 남는 bound가 l, mode 2: u
if !G { return Skip(r11) }
node.col = -1;
let c0 = rhs[r] / a;
39b9e0(c0, a, env, st, mode, j, r, rowhead[r], row_cnt[r] - 1)?;  // MIP 레코드에는 mode가 없다
if ac >= 1e-10 {
    objcon = (c0 * c) + objcon;  let f = c / a;  obj[j] = 0.0;
    let mut cnt = 0;
    for nd in row_list(r) { if nd.col >= 0 { obj[nd.col] = obj[nd.col] - nd.v * f }  cnt += 1 }
    W += (cnt * 4.0) * μ;
}
st.c404 -= 1;                                 // 39b9e0가 센 행 제거를 되돌림(행은 남는다)
node.col = j;                                 // 노드 복원
if (mode == 1 && a < 0.0) || (mode == 2 && a > 0.0) {      // 남는 bound 쪽이 '<'가 되도록 행 부호 반전
    let mut cnt = 0;
    for nd in row_list(r) { if nd.col >= 0 { nd.v = -nd.v }  cnt += 1 }   // 노드 공유라 열 리스트도 바뀜
    W += (cnt * 3.0) * μ;
    rhs[r] = -rhs[r];  swap(inf_max[r], inf_min[r]);
    (act_max[r], act_min[r]) = (-act_min[r], -act_max[r]);  // +0x90 ← −옛 +0x98, +0x98 ← −옛 +0x90
}
53763f(r);  537da8(r);                         // 행 r, 그 행의 살아 있는 열(j 포함). W += (μ·4)·방문
sense[r] = '<';
let rc = 3de18e(if mode == 1 { l } else { u }, INF, env, st, j, 0);
col_cnt[j] = -3 - r;
if rc != 0 { return rc }
Done
```

- `3de18e(v, INF, env, st, j, 0)`(인자 double 2개는 Ghidra에서 빠짐, asm 3c12c0–3c12db)는 레코드 없이 x_j를 v에 고정한다.
  - 열 j의 노드: rhs[r] −= v·a', 노드를 죽이고 row_cnt[r]−−.
  - activity에서 옛 bound 기여를 빼고 v·a'를 더한다(보호 덧셈).
  - W: `53802c` (μ·5)·방문, (col_cnt·7)·μ.
  - col_cnt[j] = 0, `5379f1(j)`, lb=ub=v.
  - 그다음 3e39a6이 col_cnt[j] = −3−r로 덮는다.
- 결과: 행 r은 `Σ a_k x_k ≤ b − a·bound`(부호 정규화 뒤)인 부등식이 된다. x_j는 type 2 레코드(x_j = c0 + Σ val_k x_k)로 복원한다.
- `ac < 1e-10`이면 obj[j]는 남는다. `3de18e`가 objcon += obj[j]·v를 하고 0으로 둔다.

#### 2.2.4 D: `3e2b99(INF, OT, env, st, j)` 쌍대 고정

인자: xmm0=INF, xmm1=OT, rdi=env, rsi=st, edx=j. 기본 38개에서 플래그는 0이므로 `f = flags & !0x40 == 0` 경로만 돈다.

```rust
let (c, u, l) = (obj[j], ub[j], lb[j]);
let mut down = false;                                   // x를 lb 쪽으로
if c >= -1e-10 {
    if st.mip != 0 && l <= 0.0 && 0.0 <= u && 1e-10 >= |c|
       && col_cnt[j] == 0 && (flags & !0x40 & !0x0b) == 0 && (Q 조건) {
        return 3de18e(0.0, INF, env, st, j, 1);          // 비용 0 빈 열을 0에: 3e39a6에서는 flags≠0 경로로만, 안 돎
    }
    down = !(l > -INF) || l > -1e8;                      // 유한한 lb ≤ −1e8이면 후보 아님
}
let mut up = c <= 1e-10 && (u >= INF || u < 1e8);       // 유한한 ub ≥ 1e8이면 후보 아님
if f != 0 { /* 0x4/0x10/0x20, +0x238, +0x2e8, +0x2f0: 기본 안 돎 */ }
if !up && !down { return 0 }                             // W 없음
let mut cnt = 0;
for nd in col_list(j) {
    if nd.col >= 0 {
        if sense[nd.row] == '=' { W += (cnt * 5.0) * μ; return 0 }   // 이 노드는 세지 않음
        up &= nd.v <= 0.0;  down &= 0.0 < nd.v;          // cmpnltsd / cmpltsd
    }
    cnt += 1;
}
W += (cnt * 5.0) * μ;
if !up && !down { return 0 }
if up {
    if u < INF { return 3de18e(u, INF, env, st, j, 1) }  // type 1 {j, u}, +0x408++, col_cnt = −2
    if PC != 0 { return 0 }
    if -OT > c { st.bad = j; return 5 }
    39e479(env, st, j, 1, colhead[j], down)?;             // 기본 38개에서 안 돎
} else {
    if l > -INF { return 3de18e(l, INF, env, st, j, 1) }
    if PC != 0 { return 0 }
    if c > OT { st.bad = j; return 5 }
    39e479(env, st, j, 0, colhead[j], down)?;
}
// 무한 bound 쪽(안 돎): 열의 살아 있는 노드마다 행 i: row_cnt[i] = −2, 행 i 리스트의 살아 있는 노드마다
// 5379f1(col), col_cnt[col]−−, col=−1; 행마다 W += (방문·3)·μ. 끝에 W += (열 방문·5)·μ, col_cnt[j] = −2
```

- `3de18e(v, INF, env, st, j, 1)`(asm 3c354a/3c3884의 tail call, double 인자는 Ghidra에서 빠짐)은 레코드가 있는 고정이다.
  - `39ab33` type 1 {j, v}, +0x408++.
  - objcon += obj·v.
  - 열 j의 행 표시(`53802c`), 노드를 죽이고 rhs·activity 갱신.
  - W: (μ·5)·방문, (col_cnt·7)·μ.
  - col_cnt[j] = −2, lb=ub=v.
- mas74 열 81: c=1e-5 > 0, lb=−0.0 유한, 모든 a > 0이라 lb에 고정한다. 레코드 v = −0.0이다(lb 값 그대로).

#### 2.2.5 `39b9e0` MIP 분기 (type 2 레코드)

인자 `(xmm0=c0, xmm1=a, env, st, mode, j, r, list, cnt)`. `st+0xfc ≠ 0 || st+0x278 ≠ 0`이면 이 분기다. mode와 r은 쓰지 않는다.

- 크기 = P + ((8·cnt − 1) & ~7) + 0x50. P = (4·cnt − 1) & ~7. 레코드 헤더는 {i32 type=2, pad, data=rec+0x18, next}이고 `+0x370`의 머리에 붙인다.
- data(rec+0x18) 배치:

| data 오프셋 | 내용 |
|---|---|
| +0x00 | i32 j |
| +0x04 | pad(초기화 안 됨) |
| +0x08 | f64 c0 = rhs[r]/a |
| +0x10 | i32 cnt |
| +0x18 | ptr ind = data+0x28 |
| +0x20 | ptr val = data+0x30+P |
| +0x28 | i32 ind[cnt] |
| +0x30+P | f64 val[cnt] |

- `+0x408`++, `+0x404`++.
- list(행 r, +0x10)를 걷는다. 살아 있는 노드마다 ind = col, val = (−v)/a(부호 반전 뒤 나눗셈)를 쓴다. col ≠ j이면 t = |v/a|로
  - `mlo[col] = mlo[col] < mlo[j]·t ? mlo[col] : mlo[j]·t`(minsd)
  - `mhi[col] = mhi[col] > mhi[j]·t ? mhi[col] : mhi[j]·t`(maxsd)
- `W += (5.0·μ)·방문`. **곱셈 순서가 LP 분기의 (방문·5)·μ와 다르다.**
- 3e39a6에서는 j의 노드를 미리 죽여 두므로 항목은 j를 뺀 행 r의 살아 있는 열(행 리스트 순서)이고 cnt = row_cnt[r] − 1이다. 의미: x_j = c0 + Σ val_k·x_k.
- LP 분기의 a, obj[j], mode', 열 부분(+0x58/+0x60)은 MIP에 없다.

#### 2.2.6 `39ab33` MIP 분기 (type 1, 참고)

- `+0x408`++. 0x28 B 레코드 {type=1, data=rec+0x18, next}.
- data 16 B = {i32 j, pad, f64 v}. W가 없다.

#### 2.2.7 추적기 연결 리스트 분기 (`+0xe0 == 0`)

- `5399fc`(열 목록)는 LP `cols_list`와 같은 구조다(판정 m ≤ 5·nr ∨ n ≤ 4·nc, 그다음 n < 3·nc + Σrow_cnt⁺).
  - 다른 점: 희소 경로에서 표시 행마다 **행 리스트를 걸으며**(죽은 노드 포함) 살아 있고 표시 안 된 열을 뒤에 붙인다.
  - 행마다 `W += (μ·4)·방문`이다. CSR은 (μ·3)·len.
  - 나머지 항(2·nr, 3·nc, 3·len, 조밀 2·n)은 LP와 같다.
  - 38개 147호출 중 66번이 희소 경로였고 모두 맞았다.
- `53802c(j)`(열 j의 행 표시): 열 리스트를 걷고 살아 있는 노드의 행을 표시한다. `W += (μ·5)·방문`. CSR은 (μ·3)·len.
- `537da8(r)`(행 r의 열 표시): 행 리스트를 걷는다. `W += (μ·4)·방문`. CSR은 (μ·3)·len.
- 표시 규칙(`53763f`/`5379f1`)은 LP `tracker.rs` `mark`와 같다. 오라클에서 플래그 5 → 13을 확인했다.

#### 2.2.8 `3a0f5b`

- Q 목적(+0x250 대각, +0x260 비대각)이 있는 열을 고정할 때의 레코드다. `st+0xfc == 0 && st+0x278 == 0 && st+0x250 ≠ NULL`일 때만 부른다.
- MIP에서는 부르지 않는다(cov2 0회).

### 2.3 LP 쌍둥이(`4fe113`, `crates/lp/src/presolve/reduce.rs` `singleton_cols`)와 차이

흐름(A 고정 → DR → 플래그 → B 빈 열 → C singleton → D 쌍대 고정)과 판정 상수(fixtol, 1e-13, 0.01·FT, 1e-6, 1e-9, 100·FT, 1e-10, 1e6)는 같다. 다른 점:

1. 자료구조: CSR이 아니라 연결 리스트다. 지운 항목은 col=−1로만 표시한다. 모든 W가 리스트 방문 수(죽은 노드 포함)로 센다.
2. A: LP `fix_near`(`4f6740`)와 식이 같다(중점 `(lb+ub)·0.5`는 교환해도 같은 비트).
   - 레코드: MIP는 `39ab33` 16 B {j, v}. LP는 상태·열 항목을 가진 FixCol이고 W 6·len.
   - W: MIP는 `53802c` 5·방문 + 6·방문. MIP는 플래그로 레코드를 거를 수 있다.
3. C singleton
   - MIP는 첫 살아 있는 노드를 그대로 쓴다(|a| > 1e-9 검사 없음, W 2·dead). LP는 0이 아닌 첫 항목, W 3·len.
   - MIP는 PRESINGLECOL 게이트가 있다.
   - MIP는 행이 singleton '='이면 건너뛴다.
   - MIP는 정수 열 검사(`int_ok`, 행 캐시 +0x390/+0x3b0)가 있다. LP에는 정수가 없다.
   - mode12의 행 길이 제한 ld0: MIP 기본 10(m>10). LP는 0이라 사실상 `|c| < 1e-10`일 때만 된다.
   - mode0 W: 2·방문(G) + 5·방문(`39b9e0`) + 5·방문. LP는 2·k + 6·len(`4f4d0a`) + 6·len.
   - mode12 행 반전: MIP는 노드를 공유하므로 행 리스트만 뒤집는다(W 3·방문). LP는 두 CSR(6·col_len + 5·len).
   - `537da8` W 4·방문. LP는 3·len.
4. 레코드: MIP type 2(`39b9e0` MIP 분기)는 {j, c0, (k, −a_k/a)}뿐이다. LP `PostRec::Subst`의 r, a, obj, mode, 열 부분은 없다. 배율 갱신은 같다.
5. D: MIP `3e2b99`와 LP `4fd703`(`dual_fix`)의 차이
   - 후보 판정: MIP `down = c ≥ −1e-10 ∧ ¬(−INF < l ≤ −1e8)`, `up = c ≤ 1e-10 ∧ ¬(1e8 ≤ u < INF)`. LP는 `−1e-10 < c`, `c < 1e-10`이고 ±1e8 조건이 없다.
   - 둘 다 후보가 아니면 MIP는 W 없이 끝난다.
   - W: MIP 5·방문. LP 4·len.
   - 유한 bound 고정: MIP `3de18e(…,1)`(type 1 16 B). LP `fix_at`.
   - 무한 쪽: MIP는 행을 −2로 둔다(LP는 −1). 레코드도 다르다(`39e479` 대 type 3). 기본에서는 안 돈다.
   - MIP에는 비용 0 빈 열을 0에 고정하는 특례(`st.mip`)가 있다.
6. 빈 열(B): 식이 같다. 레코드 형식만 다르다.

포트는 LP `singleton_cols`의 골격을 옮기고 위 차이만 바꾸면 된다. 리스트·W·레코드 부분은 새로 쓴다.

### 2.4 오라클 확인

- 도구
  - `mip_state_dump.js` 사본(세션 스크래치 `inner/bin/dump2.js`): 열 플래그 +0x170, mlo/mhi, 행 캐시 +0x3b0, misc(+0x1e0, +0x248, +0x250, +0x260, +0x278, +0xe0, +0xfc), 추적기 레벨·a/b·플래그·목록을 더 덤프한다.
  - Python 모델 세션 스크래치 `inner/bin/sim39.py`: 위 의사코드 그대로. `3dcc78` 단일 행, `3de18e` 비-Q, `3e2b99` 평범한 열, `39b9e0`·`39ab33` MIP, `5399fc` 조밀·희소.
  - 진입 덤프를 넣어 돌리고 복귀 덤프와 비교한다.
- **38개 모델의 `3e39a6` 147호출 전부 비트 일치.** 일치한 항목:
  - W(정확히 같음), objcon
  - lb/ub/obj/rhs/sense, row_cnt/col_cnt, inf_max/inf_min/act_max/act_min, mlo/mhi, 행 캐시
  - 행·열 리스트 전체(순서, col=−1, 값)
  - +0x404/+0x408
  - 새 레코드(type, j, v 또는 c0, 항목 (k, val))
  - 추적기 플래그 변화 ⊆ 예측 표시
- 실행: 세션 스크래치 `inner/bin/runall38.sh`, 결과 세션 스크래치 `inner/sim39_all.txt`.
- 거친 경로(횟수): 고정 212, 빈 열 1,166, 쌍대 고정 628, 3dcc78 5,050, mode0 C 15·I 12·B 2, mode1 C 415·B 3, mode2 I 45, 행 반전 456, 정수 검사 <10 431, 행 캐시 계산 101, mode12 목적 가드로 건너뜀 147, 희소 5399fc 66.
- 개별 예
  - murg 1호출: 열 16·17 mode0, 레코드 {16, −0.0, 16항목}, {17, −203, …}, objcon −203.
  - csched007 1호출: 열 1707 mode1, 행 200 '=' → '<', 레코드 {1707, 0, [(1692, −3), (1706, 1)]}, mhi[1692] 1 → 3.
  - mas74: 쌍대 고정 열 81(lb에), ub 1 → lb, act_max[0] 130 → 129, 레코드 v = −0.0(lb가 −0.0).
  - markshare2: 고정 열 7개, m=7이라 ld0=0이고 목적 가드 7회.
- 덤프 전제(38개 전부, 앞 2개 모델은 첫 실행에서만 확인): +0x170 = 0, +0x1e0 = +0x248 = +0x278 = 0, Q NULL, +0xe0 = 0, +0xfc ≠ 0.

### 2.5 미해결

- 오라클로 못 본 경로
  - 행 캐시 적중(같은 호출에서 같은 행을 두 번 보는 경우), G 실패
  - 무한 bound 쪽 쌍대 제거(`39e479`, 해독 안 함), 반환 3/5
  - PSC ≠ −1, NF ≠ 0
  - 열 플래그 ≠ 0 경로(PWL `+0x80` 비트, `3e2b99`의 0x4/0x10/0x20 특수 열)
  - 모두 asm대로 적었다.
- μ ≠ 1에서의 곱셈 순서(`(μ·5)·cnt` 대 `(cnt·5)·μ`)는 asm으로만 정했다. 검사한 38개가 모두 μ=1인지는 따로 확인하지 않았다.
- JSON 덤프(JSON.stringify)는 −0.0의 부호를 잃는다. 배열 비교와 Python `==`는 ±0을 구별하지 않는다. 부호 있는 0은 레코드 원시 바이트(murg c0, mas74 v)에서만 봤다.
- `st+0x248`(고정 마스크 0x800 조건)과 `st+0x278`의 의미는 모른다(MIP에서 둘 다 0).

## 3. `36ddf2` 행 정리 (`00477580`)

### 3.1 서명·가드·반환
`36ddf2(st, env) -> i32`. 가드 없음. 반환 0, 3(빈 행이 불가능: +0x478 = 행, `+0x404`는 더하지 않고 바로 반환).

### 3.2 알고리즘
```
ft = env.feastol
small = if st.mip(+0xfc)==0 && +0x27c==0 { env.opttol * 0.1 } else { 0.001 }    // MIP는 0.001
rows = 538f30()                                   // 이 반복의 첫 계산(§1.2)
removed = 0; changed = [] (+0x3b8)
for &i in rows {
  cnt = row_cnt[i]
  if cnt == 0 {
    if rhs[i] < -ft || (ft < rhs[i] && sense[i]=='=') { +0x478 = i; return 3 }
    row_cnt[i] = -1; removed += 1
  } else if cnt > 0 {
    keep = inf_max[i] != 0 || ft <= act_max[i]
           || (sense[i]=='=' && (inf_min[i] != 0 || act_min[i] <= -ft))
    if keep {                                     // 작은 계수 제거
      k = 0
      for node in row i list (+0x10) {
        j = node.col
        if j >= 0 && (row_cnt[i] as f64) * (ub[j] - lb[j]) * |node.val| < ft*0.01 && |node.val| < small {
          5379f1(tr, j); k += 1
          row_cnt[i] -= 1; col_cnt[j] -= 1          // row_cnt는 루프 안에서 바뀌고 다음 비교에 쓰인다
          rhs[i] = rhs[i] - node.val * lb[j]
          node.col = -1
        }
      }
      W += (row_cnt[i]·6)·μ                       // 줄어든 값. 머리가 NULL이면 원래 cnt
      if k > 0 { 53763f(tr, i); changed.push(i); +0x3fc += k }
    } else {                                      // 중복 행
      removed += 1
      537da8(tr, st, i)                           // 열 표시, W += (μ·4)·방문
      for node in row i list where node.col >= 0 { col_cnt[node.col] -= 1; node.col = -1 }
      W += (row_cnt[i]·3)·μ
      row_cnt[i] = -2
    }
  }
}
W += (len(rows)·9)·μ
+0x404 += removed
if changed 비지 않음 { 370047(st, env, len(changed), changed) }   // 인라인 사본, §1.7
return 0
```
- 비교 순서(asm 0x377871–0x3778a4): `((f64)row_cnt · (ub−lb)) · |a|`를 `ft·0.01`과, 그다음 `|a|`를 `small`과 비교한다.
- 행 지우기(−1, −2)에 레코드는 없다. activity는 작은 계수를 지운 행만 다시 계산하고(rhs가 바뀌었으므로), 지운 계수의 열 activity 기여는 따로 빼지 않는다(재계산이 대신한다).

### 3.3 LP 쌍둥이(`50cc90`, `reduce.rs::redundant_rows`)와 차이
- 작은 계수 기준이 `0.1·OptimalityTol`이 아니라 0.001(MIP). LP의 `mode` 인자가 없고 늘 작은 계수를 본다.
- 곱 순서: LP는 `cnt·|a|·(u−l)`, MIP는 `cnt·(u−l)·|a|`(비트 차이 가능).
- 작업량: LP는 중복 행 `4·len`, 남는 행 `9·len`. MIP는 중복 행 `3·row_cnt`(+`537da8`의 4·방문), 남는 행 `6·row_cnt`(줄어든 값), 끝에 `9·len(rows)`.
- 불가능 반환 때 LP 포트는 removed를 더하고 반환하지만 MIP는 더하지 않는다(어차피 끝).
- activity 재계산이 `4f79a1`(CSR, −rhs부터)이 아니라 `370047`(리스트, 0부터 더한 뒤 −rhs).
- LP는 CSR 열 쪽 항목도 −1로 지우지만 MIP는 노드 하나라 `col = −1` 하나로 끝난다.

### 3.4 오라클 확인
38개 모델의 모든 `36ddf2` 호출(내부 루프와 3cd406의 다른 호출부, 185회)에서 일치. 빈 행·중복 행·`538f30` 조밀/희소 경로는 지나갔다. 작은 계수 제거는 38개에서 한 번도 안 일어나 합성 모델 t3c(부록 A, `1e-8·v`, v ∈ [0, 1e-5])로 확인했다(제거, rhs, 줄어든 cnt 작업량, 인라인 370047 재계산까지 일치).

### 3.5 미해결
없음.

## 4. `37168e` activity bound 강화 (`00478470`)

### 4.1 서명·가드·반환
`37168e(st, env) -> i32`. 내부 루프(L1811–1813)는 PREBOUND(env+0x2544) ≠ 0일 때 부른다. 바깥 호출부(L2997·L4062·L4114, 가드는 spine 노트)도 같은 함수다. 반환 0, 3(행 불가능: +0x478 = 행, 또는 bound 교차: +0x470 = 열), forcing 분기의 `3de18e` 코드(0x2711).

### 4.2 알고리즘
```
INF, ft, nf = env.numeric_focus, ift = env+0x2468, pb = env.prebound
rows = 538f30()                       // 보통 36ddf2가 채운 캐시
tight = 0
for &i in rows {
  cnt = row_cnt[i]; if cnt <= 0 { continue }
  if (inf_min[i]==0 && ft < act_min[i]) || (sense[i]=='=' && inf_max[i]==0 && act_max[i] < -ft) { +0x478 = i; return 3 }
  if nf >= 1 && cnt == 1 { forcing 검사(§4.2.1). 지웠으면 continue }
  r = act_max[i] - act_min[i]
  r = if r > 1.0 { r } else { 1.0 }     // asm maxsd: NaN이면 1.0
  r = if 1e6 < r { 1e6 } else { r }      // minsd
  if nf == 1 { r *= 10.0 } else if nf >= 2 { r *= 100.0 }
  thr = r * 1e-10
  for node in row i list (+0x10) {
    j = node.col; if j < 0 { continue }
    a = node.val; aa = |a|
    if thr > aa { continue }
    ub = ub[j]; lb = lb[j]; am = act_min[i]; im = inf_min[i]    // 노드마다 다시 읽는다(앞 노드의 3dd6f1이 바꿈)
    U = ub; L = lb
    if a > 1e-8 {
      if im == 0 { t = if 0.0 <= am { 0.0 } else { am }; U = lb - t / a }
      else if im == 1 && lb <= -INF { U = -am / a }
    } else if a < -1e-8 {
      if im == 0 { t = 위와 같이; L = ub - t / a }
      else if im == 1 && INF <= ub { L = -am / a }
    }
    if sense[i] == '=' {                // 행을 뒤집어 한 번 더: a2 = -a, min 쪽 = -act_max
      a2 = -a; ax = act_max[i]; t2 = -ax; ix = inf_max[i]; U2 = ub; L2 = lb
      if a2 > 1e-8 { if ix == 0 { t = if 0.0 <= t2 { 0.0 } else { t2 }; U2 = lb - t / a2 }
                     else if ix == 1 && lb <= -INF { U2 = ax / a2 } }
      else if a2 < -1e-8 { if ix == 0 { t = 같이; L2 = ub - t / a2 }
                           else if ix == 1 && INF <= ub { L2 = ax / a2 } }
      L = if L2 <= L { L } else { L2 }
      U = if U <= U2 { U } else { U2 }
    }
    if vtype == NULL || vtype[j] == 'C' {
      if nf > 1 { continue }
      if L <= ft + ub { L = if ub < L { ub } else { L } }          // asm minsd(ub, L)
      if lb - ft <= U { U = if lb > U { lb } else { U } }          // maxsd(lb, U)
      if nf == 1 { if L != U && U - L < ft*100.0 { continue } }
      else if pb < 2 && L != U && U - L < ft*10.0 {                // != 는 ucomisd, NaN이면 참
        L = floor(L + 1e-13); U = ceil(U - 1e-13)                   // 오히려 넓힌다 → 아래 채택 검사에서 대개 버려진다
      }
    } else {                                                        // 정수
      Lc = ceil(L - ift); Uf = floor(ift + U); fl = floor(L); cu = ceil(U)   // 호출 순서도 이대로
      if fl < Lc && !(0.1*ft < (L - fl) * aa) { Lc = fl }          // 0.1은 상수 0x2a67dd8, 0.1*ft 순서
      if Uf < cu && !(0.1*ft < aa * (cu - U)) { Uf = cu }
      L = Lc; U = Uf
    }
    tol = if colflag[j] & 0x18 != 0 { ft*10.0 } else { ft*1000.0 }
    lb_ok = 1e12 > |L| && L > -1e8 && L > lb && (1e-13 >= U - L || L > lb + tol)
    ub_pre = 1e12 > |-U| && -U > -1e8 && -U > -ub
    if lb_ok { tight += 1; if !ub_pre { U = ub; goto apply } }
    else { L = lb; if !ub_pre { continue } }
    // 여기서 ub_pre. L은 새 lb(채택) 또는 옛 lb
    if 1e-13 >= U - L || -U > tol - ub { tight += 1 }
    else { U = ub; if !lb_ok { continue } }
  apply:
    3dd6f1(lb, ub, L, U, INF, st, j)
    lb[j] = L; ub[j] = U
    5379f1(tr, j)
    if U + ft < L { +0x470 = j; return 3 }
  }
  W += (row_cnt[i]·11)·μ
}
W += (len(rows)·8)·μ          // 반환 3이면 이 두 줄과 +0x3f8 갱신은 없다
+0x3f8 += tight
return 0
```
- 채택 검사는 asm(0x378d52–0x378ee8) 그대로다. lb 채택이 실패하면 L은 옛 lb로 되돌아가 ub 쪽 "고정" 판정(`U − L ≤ 1e-13`)에 쓰인다. `+0x3f8`은 lb·ub 채택을 따로 센다(둘 다면 2).
- 연속 변수의 lb는 "`ub + ft` 이하면 ub로 자른다", ub는 "`lb − ft` 이상이면 lb로 올린다". 그 밖(교차가 ft보다 큼)은 그대로 두었다가 마지막 `U + ft < L`에서 불가능이 된다.
- 무한 bound(±1e30): `L > −1e8`, `U < 1e8`, `|L|, |U| < 1e12`라 1e8 넘는 새 bound는 버린다.

#### 4.2.1 forcing 분기 (nf ≥ 1이고 cnt == 1일 때만. 기본값 nf = 0에서는 안 돎)
```
s = 0
if inf_min[i]==0 && !(act_min[i] <= -1e-10 || ft/100 <= act_min[i]) {
   if 행의 살아 있는 노드 중 |a| < 1000·ft 가 없으면 s = 1     // W += (μ·5)·방문(찾은 곳까지)
} else if sense[i]=='=' && inf_max[i]==0 && act_max[i] < 1e-10 && -ft/100 < act_max[i] {
   같은 검사로 s = -1
}
if s != 0 {
   for node in row i where live { j = node.col; node.col = -1; col_cnt[j] -= 1
       r = 3de18e(s*a <= 0 ? ub[j] : lb[j], INF, env, st, j, 1); if r != 0 { return r } }
   W += (row_cnt[i]·5)·μ; row_cnt[i] = -2; +0x404 += 1; continue
}
// 아니면 위의 보통 경로(r에 nf 배율)
```

### 4.3 LP 쌍둥이
LP presolve에는 같은 함수가 없다(LP의 bound 전파 `propagate_into`는 사본 bound에 대한 다른 규칙). 활동량 갱신 `3dd6f1`은 LP `4fe49b`와 같은 규칙.

### 4.4 오라클 확인
38개 모델의 모든 호출(내부 루프와 바깥 호출부 전부, 225회)에서 bound·activity·작업량·추적기·`+0x3f8` 비트 일치(§9 표). lb 채택, ub 채택, lb만 채택(ub = 옛 ub), 정수 `floor(L)` 완화(0.1·ft, 32회) 경로를 지났다. 연속 변수의 floor/ceil 넓히기는 38개에서 일어나지 않아 합성 모델 s37(부록 A)로 확인했다(3회, ub 10 → 1). forcing 분기(nf ≥ 1)는 기본값 밖.

### 4.5 미해결
- `colflag & 0x18`이 서는 경우(tol = 10·ft)를 아직 모른다(38개 진입 때 플래그는 모두 0).
- nf ≥ 1 경로는 모형에 넣지 않았다.

## 5. `36c5cf` singleton 행 (`00476730`)

### 5.1 서명·가드·반환
`36c5cf(st, env, n, list, *cnt_out) -> i32`. 3cd406은 (st, env, −1, NULL, &local)로 부른다(asm 0x3b1f40: edx = −1, ecx = 0, r8 = 스택 주소). n == −1이면 `538353(tr, &n, &list)`로 레벨 원시 행 목록을 쓴다. 반환 0, 3(+0x478 = 행, +0x470 = 열), 0x2711(LP식 레코드), `3de18e` 코드.

### 5.2 알고리즘
```
ft = env.feastol; INF = env.infbound; lp_like = (+0xfc==0 && +0x278==0)
nnz = 0; nrem = 0
for &i in list[..n] {
  if row_cnt[i] != 1 { continue }
  node = 행 i 리스트에서 첫 살아 있는 노드; sk = 그 앞의 죽은 노드 수; W += (sk+sk)·μ
  a = node.val; aa = |a|; j = node.col
  if !(1e-8 <= aa) { continue }
  b = rhs[i]; x = b / a
  kind = if sense[i]=='=' { 3 } else if a <= 0 { (lb[j] < x) as 1 } else { if x < ub[j] { 2 } else { 0 } }
  if kind & 1 {                                   // 새 lb x
    if !(x <= ub[j]) {
      t = if a <= 0 { ft + b } else { b - ft }; x = if t/a <= ub[j] { ub[j] } else { t/a }
      if x <= ft + ub[j] { if ft + b < ub[j]*a { continue } }   // 행을 그대로 둔다
      else { goto conflict }
    }
  }
  if kind & 2 {                                   // 새 ub x
    if x < lb[j] {
      t = if a <= 0 { b - ft } else { ft + b }; x = if lb[j] <= t/a { lb[j] } else { t/a }
      if x < lb[j] - ft { goto conflict }
      if lb[j]*a < b - ft { continue }
    }
  }
  node.col = -1
  if kind != 0 {
    if lp_like { 39d60a(a, x, env, st, j, i, kind) 레코드; 연속 규칙 }
    if lp_like || vtype NULL || vtype[j]=='C' {   // 연속 규칙(LP 4ffe11과 같은 꼴)
      L = if kind & 1 { if ub[j] <= x { ub[j] } else { x } } else { lb[j] }
      U = if kind & 2 { if x <= L { L } else { x } } else { ub[j] }
    } else {                                      // 정수 규칙
      l0 = kind&1 ? x : lb[j]; u0 = kind&2 ? x : ub[j]
      L = ceil(l0 - ift); U = floor(ift + u0); fl = floor(l0); cu = ceil(u0)
      if fl < L && !(0.1*ft < (l0 - fl)*aa) { L = fl }
      if U < cu && !(0.1*ft < aa*(cu - u0)) { U = cu }
      if U < L { +0x478 = i; +0x470 = j; return 3 }
      if x != floor(x) && L <= lb[j] && ub[j] <= U { node.col = j; continue }   // 조이지 못하면 행을 남긴다
      if kind == 3 { x = L }
    }
    3dd6f1(lb[j], ub[j], L, U, INF, st, j); lb[j] = L; ub[j] = U
  }
  5379f1(tr, j)
  row_cnt[i] = -1; nnz += 1; col_cnt[j] -= 1
  if kind == 3 { nnz += col_cnt[j]; r = 3de18e(x, INF, env, st, j, 1); if r != 0 { goto out(r) } }
  nrem += 1
  continue
conflict:                                          // LAB_00476b3d
  if ft <= act_min[i] || (!(-ft < act_max[i]) && sense[i]=='=') { +0x478 = i; +0x470 = j; goto out(3) }
  // 아니면 아무것도 안 바꾸고 다음 행(원본은 node.col을 −1로 썼다가 되돌린다)
}
W += (n·11)·μ; +0x404 += nrem
out(r): if cnt_out != NULL { *cnt_out = nnz }   // 불가능·오류로 나갈 때도 쓴다. 이때는 W·+0x404 갱신 없음
return r (정상 0)
```
- `=` 행의 고정: 연속이면 x(잘린 값), 정수면 반올림한 L. 정수 '=' 행에서 x가 정수가 아니고 반올림 결과가 옛 bound보다 좁지 않으면 행을 남긴다.
- `3dd6f1`은 노드를 죽인 뒤 부르므로 지우는 행 i의 activity는 안 고친다.
- `*cnt_out`(= 지운 비영 수: 행 노드 1 + `=` 행이면 고정된 열의 나머지 비영)이 1e6을 넘으면 3cd406이 전체 GC를 부른다(§0).

### 5.3 LP 쌍둥이(`4ffe11`, `reduce.rs::singleton_rows`)와 차이
구조(첫 살아 있는 노드, x = b/a, kind 비트, `4fa83a`/`3de18e` 고정)는 같지만 판정이 다르다.
- LP는 x를 옛 bound와 1e-10 여유로 비교해 code(0–4)를 정하고, `ft/|a|`·`10·ft/|a|` 허용으로 "무시" 판정을 한다. MIP는 `lb < x`/`x < ub`(여유 없음)로 kind를 정하고, 교차하면 ft만큼 느슨한 x로 다시 계산해 `ft + b < ub·a`(lb 쪽)·`lb·a < b − ft`(ub 쪽)면 행을 남긴다.
- MIP는 정수 열 반올림 규칙이 있다. LP는 singleton 행 레코드(`PostRec::SingletonRow`)를 남기지만 MIP(+0xfc ≠ 0)는 bound 변경 레코드가 없다(`=` 행 고정의 type 1만. LP식 상태면 `39d60a`).
- 작업량: LP는 `2·row_len`(행마다), MIP는 첫 살아 있는 노드 앞 죽은 노드 수의 2배, 끝에 `11·n`.
- `*cnt_out` 출력은 LP에 없다.

### 5.4 오라클 확인
38개 모델 147회 모두 일치. 다만 기본값에서는 `37168e`가 먼저 bound를 조여 singleton 행이 거의 빈 행처럼 처리된다(연속 `=` 행 4건뿐). 그래서 PREBOUND=0(`GURO_PAR_PREBOUND=0`)으로 6개 모델(glass4, timtab1, csched007, tr12-30, sp150x300d, neos-860300) 38회와 합성 모델 s36(부록 A)을 더 돌렸다: kind 1 C, kind 3 C·I(정수 `=` 고정 18회), kind 1 I, kind 2 C(허용 오차 안 교차를 lb로 자르기), kind 2 I, 정수 비정수 x로 행을 남기는 경로까지 모두 일치. `=` 행 고정은 `3de18e`(rec=1)와 `39ab33` 레코드 바이트까지 확인했다. **교차 뒤 불가능/행 남기기(conflict) 경로만 못 봤다**(한 변수 행에서는 activity가 맞으면 늘 불가능이 된다).

### 5.5 미해결
- conflict 경로(불가능 반환, 또는 activity가 어긋나 행을 남기는 경우)는 오라클로 못 봤다.
- `39d60a`(LP식 레코드)는 MIP에서 안 돎.

## 6. `56f793` 계수 강화 (`005b8640`)

### 6.1 서명·가드·반환
`56f793(st, env)`. 3cd406은 PRECOEFFICIENT(env+0x2548) ≠ 0일 때 부르고 반환값을 버린다. eax = 이번 호출에서 바꾼 계수 수.

### 6.2 알고리즘
```
ft, INF
rows = 538f30()
total = 0
for &i in rows {
  if !(row_cnt[i] > 0 && sense[i] != '=' && inf_max[i] == 0 && ft <= act_max[i]) { continue }   // 작업량 없음
  k = 0; drhs = 0.0; dmin = 0.0; big = false
  for node in row i list (+0x10) {
    j = node.col
    if j < 0 || vtype[j] == 'C' { continue }        // vtype NULL 검사 없음
    if !(0.5 <= ub[j] - lb[j]) { continue }
    a = node.val; d = act_max[i]                     // d는 루프 안에서 안 바뀐다
    if a <= 0 {
      e = -(a + d)
      if ft < e && (-1e10 <= a || lb[j] == 0.0) {
        big = big || a < -1e10
        node.val = -d
        l = lb[j]; u = ub[j]
        drhs = drhs + e*l
        if INF <= u { dmin = gdn(dmin, a*l); dmin = gdn(dmin, l*act_max[i]) }
        else { w = u - l; dmin = gdn(dmin, -(a*w)); dmin = gdn(dmin, -(w*act_max[i])) }
        5379f1(tr, j); k += 1
      }
    } else {
      e = a - d
      if ft < e && (a <= 1e10 || ub[j] == 0.0) {
        big = big || 1e10 < a
        node.val = d
        u = ub[j]; l = lb[j]
        drhs = drhs - e*u
        if l <= -INF { dmin = gdn(dmin, a*u); dmin = gdn(dmin, -(u*act_max[i])) }
        else { w = u - l; dmin = gdn(dmin, a*w); dmin = gdn(dmin, -(w*act_max[i])) }
        5379f1(tr, j); k += 1
      }
    }
  }
  W += (row_cnt[i]·7)·μ
  if k > 0 {
    53763f(tr, i)
    rhs[i] = drhs + rhs[i]
    if big { 370047(st, env, 1, &i) }               // asm 0x4b8e4f: edx = 1, rcx = &행(스택 0xb4)
    else { act_min[i] = gdn(act_min[i], dmin) }      // act_max(= d)는 수학적으로 그대로라 안 고친다
    total += k
  }
}
W += (len(rows)·8)·μ
+0x3fc += total
return total
```
- 새 계수는 `|a'| = d`(max activity − rhs). 부호는 그대로. rhs는 a ≤ 0이면 `+e·lb`, a > 0이면 `−e·ub`.
- `dmin`은 min activity − rhs의 변화량 `e·(u−l)`(무한 bound 쪽이면 `−e·lb`/`+e·ub`)을 두 번의 아래 반올림 덧셈으로 쌓은 것이다.

### 6.3 LP 쌍둥이
없다(정수 전용). 행 activity 재계산은 `370047`.

### 6.4 오라클 확인
38개 모델 모든 호출 일치(계수 값, rhs, act_min 비트, 작업량, 추적기, `+0x3fc`, 반환값). a ≤ 0·a > 0 경로 모두 지났다(neos-860300 354,304건, sp150x300d 236건 등), a ≤ 0에서 ub 무한 1건. `big`(|a| > 1e10) 경로와 a > 0의 lb 무한 경로는 일어나지 않았다.

### 6.5 미해결
없음(big·lb 무한 경로는 해독만).

## 7. `56c80e` doubleton 등식 집계 (`005b50a0`)

`56c80e`(file+0x4b50a0, 6,228 B)와 그것만 쓰는 보조 `566cc5`(2항 등식), `569830`(이진 변수가 정하는 등식), `39d9e1`(type 2 레코드), `3c3abf`(작아진 계수 제거), `5382b5`(행 일괄 표시), `560ea1`·`562f34`(Q·QC, 기본값에서 도달 불가). 메모리 보조 `ac645d`(env, bytes) = malloc, `ac64e6`(env, n, size) = calloc, `ac661f`(env, p) = free. 할당 실패는 0x2711.

LP의 단계 D(`37f484`)와 이름만 비슷하고 구조가 다르다. 대입을 바로 하지 않고 **열마다 부모 사슬(x_s = m·x_parent + o)을 쌓았다가 끝에 한꺼번에 적용**한다. 적용은 fill-in이 아니라 **노드의 열 번호를 바꿔 루트 열로 옮기는 것**이고, 그 뒤 행 안의 중복 열을 합친다.

표기: `W += x·μ`는 `*(state+0x458) = x·μ + W`(μ = state+0x448). 곱셈 순서는 asm 그대로 적는다(μ ≠ 1이면 결과 비트가 달라질 수 있다). `INF` = env+0x2458(1e30), `FT` = env+0x2460(FeasibilityTol), `AM` = env+0x26c0(GURO_PAR_AGGMULTLIMIT, 1e14), `IFT` = state+0x1e8. `mlo`/`mhi` = state+0x80/+0x88(열마다 누적 배율의 최소·최대, LP 노트의 mult_lo/mult_hi). `flag` = state+0x170(열 플래그, 비트 8 = Q 목적, 0x10/0x20 = QC, 0x80 = PWL 목적). 행 activity는 LP 규약과 같다: `amax`(+0x90)/`imax`(+0x70) = max activity − rhs의 유한 부분/무한 개수, `amin`(+0x98)/`imin`(+0x78) = min activity − rhs.

### 7.1 서명·가드·반환값

```
56c80e(state, env, mode: i32, status: &mut i32) -> i32
```
- 3cd406 L1991: `if Aggregate(env+0x242c) != 0 { r = 56c80e(state, env, 0, &local_978) }`, `r != 0 || local_978 != 0`이면 presolve 끝(`LAB_004b2dee`). 호출 지점은 이 하나뿐이고 mode는 늘 0이다.
- mode: `PREQSUBST(env+0x2584) == 0`이면 0으로 바꾼다. 보조 함수 안에서 `forbid = !(mode==0 ? 0 : mode==1 ? 0x20 : 0x38)`이다. **mode = 0이면 forbid = 0xffffffff: flag ≠ 0인 열은 대입하지 않는다.** 그래서 `560ea1`/`562f34`(flag 비트를 가진 열을 대입했을 때만 부름)와 `PREQSUBST < 2 && flag & 0x30` 검사는 기본값에서 도달할 수 없다.
- 반환: 0; 0x2711(할당 실패, `39d9e1` 실패); `3ba5ac`·`3ba9de`의 0 아닌 값(콜백 중단·예산); `566cc5`/`569830`이 넘긴 `3de18e`의 반환값. `*status = 3`은 불가능 판정이다. 열 때문이면 state+0x470 = 열, 행 때문이면 state+0x478 = 행이다. status ≠ 0이면 그 자리에서 끝낸다. 2단계(대입 적용)는 하지 않지만, 이미 지운 행은 지운 채로 남는다.
- 주기 검사: 시작에 `3ba5ac(env, st, 0, 0, -1)`를 한다. 1단계는 행 번호 idx % 10000 == 0마다(idx = 0 포함) `3ba5ac(env, st, 0, 0, -1)`와 `3ba9de(env, st, 0)`를, 2단계는 10000개마다 `3ba5ac(env, st, 0)`를 부른다. 끝에 `3ba5ac`를 한 번 부르고, 반환이 0이면 `LAB_005b63a8`에서 한 번 더 부른다. 어느 것이든 0이 아니면 그 값으로 돌아간다.

### 7.2 알고리즘

#### 7.2.1 준비 (asm 0x4b50a0–0x4b5285)
```
(Rcnt, R) = 538f30(tracker, st)      // 현재 레벨(내부 루프는 2) 행 목록. R == NULL이면 0..Rcnt-1
tmp  = malloc(4n); list = malloc(4n); parent = calloc(n, 4)   // parent[j] = 부모+1, 0이면 루트
mult = st+0x398 (f64[n]), off = st+0x3a0 (f64[n])             // 작업 버퍼. parent≠0인 열만 읽는다
touched = st+0x390 (i32[]), rmark = st+0x3a8 (i32[m], 0이라고 가정)
pos = st+0x1b0 (node*[n], NULL이라고 가정, 끝나면 다시 NULL)
nsub = 0; nsub2(local_f8) = 0; dead(local_a8) = 0; ntouched = 0; roots = []
```
- `538f30`의 연결 리스트 분기(+0xe0 == 0): 표시 열마다 열 리스트 전체(죽은 노드 포함)를 걷고 `W += (5.0·μ)·방문`을 더한다. 살아 있는 노드의 행을 처음 볼 때만 추가한다. CSR 분기(3·len)와 다르다. 결과는 레벨별로 캐시된다. 내부 루프에서는 보통 앞 reduction이 이미 채운 캐시를 쓴다(오라클에서 작업량 증가 0).

#### 7.2.2 1단계: 후보 행 (L95–135)
```
for idx in 0..Rcnt:
    r = R[idx]
    if row_cnt[r] > 1 && sense[r] == '=':
        if row_cnt[r] == 2:          ret = 566cc5(st, env, mode, r, tmp, list, parent, mult, off, &nsub, &nsub2, &dead, status)
        elif imin[r] < 1 && imax[r] < 1 && |amin[r] + amax[r]| <= 9.999999999999999e-05 (0x3f1a36e2eb1c432c):
                                     ret = 569830(같은 인자)
        if *status != 0 || ret != 0: goto 끝
W += (Rcnt·10.0)·μ
```
- 1단계 동안에는 대입된 열의 노드도 그대로 남아 있다. 행 activity도 바뀌지 않는다(`3dd6f1`만 예외). 뒤에 오는 행에 이미 대입된 열이 있으면 보조 함수가 사슬을 따라 루트로 바꿔 읽는다.

#### 7.2.3 사슬 풀기 (566cc5·569830·56c80e에 같은 코드가 인라인돼 있다)
```
fn chain(j) -> (root, m, o):            // parent[j] != 0일 때만 부른다
    k = 0; t = j
    loop { tmp[k++] = t; t = parent[t] - 1; if parent[t] == 0 { break } }   // root = t
    W += (k + k)·μ
    m = mult[tmp[k-1]]; o = off[tmp[k-1]]
    for q in (0..k-1).rev():            // k-2부터 0까지
        i = tmp[q]; m = m·mult[i]; o = o·mult[i] + off[i]
        mult[i] = m; off[i] = o; parent[i] = root + 1   // 경로 압축
    W += 566cc5·569830: (4.0·μ)·(k-1),  56c80e 2단계: ((k-1)·4.0)·μ
```

#### 7.2.4 `566cc5` 2항 등식 (file+0x4b27a0, 6,348 B)
```
rhs = st.rhs[r]
if rhs != 0 && |rhs| < 1e-6: return 0                       // 작업량 없음
c1 = c2 = -1; vis = 0
for node in row r:                                           // 행 리스트 순서(+0x10)
    if node.col >= 0:
        if ub[c] - lb[c] < 1e-10: W += ((10 + vis) + vis)·μ; return 0     // vis = 이 노드 앞까지
        if c1 < 0 { c1 = c; a1 = val } else { c2 = c; a2 = val }        // c2 = 마지막 live
    vis += 1
W += ((10.0 + vis) + vis)·μ
o1 = o2 = 0
if parent[c1] != 0: (c1, m, o) = chain(c1); o1 = o·a1; a1 = a1·m
if parent[c2] != 0: (c2, m, o) = chain(c2); o2 = o·a2; a2 = a2·m
f1 = flag[c1], f2 = flag[c2]                                 // 아래 &forbid (mode 0 = 그대로)
if f1&forbid != 0 && f2&forbid != 0: return 0
A1 = |a1|, A2 = |a2|
if A1/A2 < 1/AM || AM < A1/A2 || A1 < 1/AM || AM < A1 || A2 < 1/AM || AM < A2: return 0   // 이 순서
rr = (rhs - o1) - o2
if rr != 0 && (|rr| < 1e-6 || 1e6 < |rr|): return 0
```
**남길 열(t)과 대입할 열(s)** (기본은 keep = c2):
```
t1 = vtype[c1], t2 = vtype[c2]
if (t1=='C') == (t2=='C'):
    if t1 != 'C':                                            // 둘 다 정수
        lo = A2 <= A1 ? A2 : A1;  hi = A1 <= A2 ? A2 : A1
        if 1e-10 < hi/lo - floor(hi/lo + 1e-10): return 0
        if 1e-10 < rr/lo - floor(rr/lo + 1e-10): return 0
        if A1 != A2 { if A2 < A1 { keep = c1 }; goto chosen }  // |a|가 큰 쪽을 남긴다
    if f2&forbid == f1&forbid:
        if (f1 == 0) == (f2 == 0):                           // 플래그 원값
            if t1 == t2 { if col_cnt[c2] <= col_cnt[c1] { keep = c1 } }   // 개수 많은 쪽을 남김, 동률이면 c2 대입
            elif t1 == 'B' { keep = c1 }                     // B를 남긴다
        elif f2 != 0 { keep = c1 }
    elif f2&forbid != 0 { keep = c1 }
elif t1 != 'C': keep = c1                                    // 정수를 남기고 연속을 대입
chosen:
(s, a_s, t, a_t) = keep == c1 ? (c2, a2, c1, a1) : (c1, a1, c2, a2)
```
- 한쪽만 forbid 비트를 가지면 그 열이 s로 뽑혀 바로 아래에서 실패한다(코드 그대로).

검사와 t의 bound 강화:
```
if flag[s] & forbid != 0: return 0
q = |a_t / a_s|
if 1e-10 < |q - 1|:
    if ub[s] < 1e30 && 1e6 < |ub[s]|: return 0
    if -1e30 < lb[s] && 1e6 < |lb[s]|: return 0             // 1e30은 상수(INF 아님)
if mlo[s]·q < 1/AM || AM < mhi[s]·q || 1e6 < q || q < 1e-6: return 0
big = 열 s의 live 노드 중 1e8 < |val|가 있는가            // 열 리스트(+0x18) 순서, 찾으면 멈춤
W += (col_cnt[s]·3.0)·μ; if big: return 0
lt0 = lb[t]; ut0 = ub[t]; done = false
if t == s: (같은 루트, 아래 2.4.1)
f = -a_s / a_t
lo = lb[t]; hi = ub[t]
if -1e30 < lb[s]: v = rr/a_t + lb[s]·f;  if 0 <= f { lo = v } else { hi = v }
if ub[s] <  1e30: v = rr/a_t + ub[s]·f;  if 0 <= f { hi = v } else { lo = v }
if vtype[t] != 'C': lo = ceil(lo - IFT); hi = floor(hi + IFT)        // asm 0x4b36a3: 인자는 ±IFT
F = min(|f|, 1.0)                                                     // minsd
lbt = lb[t]; ubt = ub[t]
b16 = (lbt + 1e-10) + F·1e-6 < lo;            if b16 { +0x3f8++; lb[t] = lo; lbt = lo; ubt = ub[t] }
b17 = hi < (ubt - 1e-10) - F·1e-6;            if b17 { +0x3f8++; ub[t] = hi; ubt = hi; lbt = lb[t] }
if lbt - ubt <= IFT { if 0 < lbt - ubt { if ubt == hi { ub[t] = lbt } else { lb[t] = ubt } } }
else { st+0x478 = r; *status = 3 }
if b16 || b17: 3dd6f1(lt0, ut0, lb[t], ub[t], st, t)                 // asm 0x4b3887. 불가능이어도 부른다
if *status == 3: return 0
if ub[t] - lb[t] <= 1e-10:                                            // t가 고정됐다 → s가 정해진다
    v = lb[t]·(-a_t/a_s) + rr/a_s
    if vtype[s] != 'C':
        if 1e-6 < v - floor(v + 1e-6): st+0x470 = s; *status = 3; return 0
        v = floor(v + 0.5)                                           // asm 0x4b3cbb
    if FT < v - ub[s] || FT < lb[s] - v: return 0                    // 행은 남긴다
    ls = lb[s]; us = ub[s]; hi2 = max(v, ls); lo2 = min(v, us)        // maxsd/minsd
    if lo2 > ls + 1e-6:
        +0x3f8++; lb[s] = lo2
        if hi2 < ub[s] - 1e-6 { +0x3f8++; ub[s] = hi2 }
        3dd6f1(ls, us, lb[s], ub[s], st, s)
    elif hi2 < us - 1e-6: +0x3f8++; ub[s] = hi2; 3dd6f1(ls, us, lb[s], ub[s], st, s)
    done = true
if vtype[t] != 'B' && vtype[s] == 'B': return 0                       // bound 변경은 남고 행도 남는다
if !done:
    o = rr / a_s
    v1 = o - (a_t·lb[t])/a_s;  v2 = o - (a_t·ub[t])/a_s               // s의 bound가 암시되는가(1000·FT)
    for v in [v1, v2]:                                                // v1을 다 본 뒤 v2
        if v - lb[s] <= FT·-1000.0 && -1e30 < lb[s]: return 0
        if FT·1000.0 <= v - ub[s] && ub[s] < 1e30: return 0
    parent[s] = t + 1; mult[s] = -a_t/a_s; off[s] = o
    x = mlo[s]·|mult[s]|; if x <= mlo[t] { mlo[t] = x }
    x = mhi[s]·|mult[s]|; if mhi[t] <= x { mhi[t] = x }
    list[nsub++] = s; nsub2 += 1
// 행 r 지우기
for node in row r: if node.col >= 0 { col_cnt[node.col] -= 1; node.col = -1 }
W += (row_cnt[r]·2.0)·μ
dead += row_cnt[r]; if st.nnz(+0xd8) < dead { 36e6b6(st, 2); dead = 0 }
row_cnt[r] = -1; +0x404 += 1; return 0
```
- 노드는 행·열 리스트에 남는다(lazy). rhs·activity·+0xd8은 바뀌지 않는다. 레코드는 여기서 만들지 않는다(2단계의 `39d9e1`).
- t가 이미 고정돼 행이 지워지는 경우(done)는 레코드도 대입도 없다. 두 열은 lb = ub로 남고, 다음 열 패스에서 고정 열로 지워진다.

##### 7.2.4.1 같은 루트 (t == s, 사슬을 풀었더니 두 항이 같은 열)
```
d35 = rr / a_s;  d45 = 1.0 - (-a_t / a_s);  ret = 0
if 1e-13 < |d35|:
    if d45 != 0 && (|d35/d45| < 1e-6 || 1e6 < |d35/d45|): *status = 0; return 0
    if d45 == 0: if |d35| < 1e-6 { *status = 0; return 0 } else { st+0x470 = t; *status = 3; return 0 }
    *status = 0; v = d35 / d45; if vtype[t] != 'C' { v = floor(v + IFT) }
    if ub[t] + FT < v || v < lb[t] - FT: st+0x470 = t; *status = 3; return 0
    ret = 3de18e(v, INF, env, st, t, 0); done = ret == 0           // asm 0x4b3559: xmm0 = v, xmm1 = INF
else:
    if |d45| <= FT: done = true; *status = 0; goto 강화(lbt/ubt 다시 읽지 않음)
    if FT < lb[t] || ub[t] < -FT: st+0x470 = t; *status = 3; return 0
    *status = 0
    if lb[t] != 0 || ub[t] != 0: ret = 3de18e(0.0, INF, env, st, t, 0); if ret != 0 { goto 검사 }
    done = true
검사: if *status != 0 || ret != 0: return ret
lbt = lb[t]; ubt = ub[t]; 이어서 위 "f = -a_s/a_t"부터(t == s로)
```

#### 7.2.5 `569830` 이진 변수가 정하는 등식 (file+0x4b4070, 4,132 B)
행 r(개수 ≥ 3, 무한 항 없음, rhs가 activity 구간의 가운데)에서 이진 b가 activity 폭 전체를 차지하면, 나머지 열은 b = 0일 때 한쪽 bound에, b = 1일 때 다른 쪽 bound에 있어야 한다. 그래서 각 열을 b의 1차식으로 대입한다.
```
Mx = -rhs[r]; Mn = -rhs[r]; b = -1
for node in row r (live):
    a, u = ub[c], l = lb[c]
    if 0 < a { Mx = Mx + u·a; Mn = Mn + a·l } else { Mn = Mn + u·a; Mx = Mx + a·l }
    if vtype[c]=='B' && l == 0 && u == 1 && ||a| + amin[r]| < 9.999999999999999e-05: b = c; ab = a   // 마지막이 이긴다
W += (row_cnt[r]·7.0)·μ
if b < 0 || 1e-6 < |Mx + Mn| || 1e-6 < ||ab| + Mn|: return 0
b0 = b; if parent[b] != 0: (b, m, _) = chain(b); ab = ab·m           // 오프셋은 버린다
k0 = nsub2; cnt0 = row_cnt[r]
if rowhead[r] != NULL:
    P = Mx - |ab|;  N = -(Mn + |ab|)
    for node in row r:
        c = node.col; if c < 0 || c == b0: continue
        ac = node.val; if parent[c] != 0: (c, m, _) = chain(c); ac = ac·m    // 오프셋은 버린다
        if flag[c] & forbid != 0: continue
        w = ub[c] - lb[c]; if w < 1e-8: continue
        if vtype[c] == 'C' { if !(N/|ac| <= 1e-10 && P/|ac| <= 1e-10) { continue } }
        else if 0.5 < N/|ac| || 0.5 < P/|ac| { continue }
        if 1e6 < w || AM < mhi[c]·w || mlo[c]·w < 1/AM: continue
        big = 열 c의 live 노드 중 1e8 < |val|; W += (col_cnt[c]·3.0)·μ; if big: continue
        w = ub[c] - lb[c]; cc = lb[c]
        if 0 <= ac·ab { w = -w; cc = ub[c] }                // x_c = cc + w·x_b
        if c == b: (같은 루트: 7.2.4.1과 같은 꼴로 d33 = 1 - w, cc를 쓴다. 대입하지 않고 다음 노드로) continue
        parent[c] = b + 1; mult[c] = w; off[c] = cc; list[nsub++] = c
        mlo[b] = minsd(mlo[b], mlo[c]·|w|); nsub2 += 1; mhi[b] = maxsd(mhi[b], mhi[c]·|w|)
W += (row_cnt[r]·8.0)·μ                                      // 머리가 NULL이어도
if nsub2 - k0 == cnt0 - 1:                                   // b를 뺀 모든 열을 대입했으면
    for node in row r: if live { col_cnt[col] -= 1; col = -1 }
    W += (row_cnt[r]·3.0)·μ; dead += row_cnt[r]; if nnz < dead { 36e6b6(st, 2); dead = 0 }
    row_cnt[r] = -1; +0x404 += 1
return ret(0 또는 3de18e 결과)
```
- 같은 루트 분기: `1e-13 < |cc|`이면 `d33 == 0 || 1e-6 ≤ |cc/d33| ≤ 1e6`이고 `d33 ≠ 0`일 때 v = cc/d33을 쓴다(정수면 floor(v + IFT)). v가 [lb−FT, ub+FT] 밖이면 status 3, 안이면 `3de18e(v, INF, env, st, b, 0)`. `d33 == 0`이고 `1e-6 ≤ |cc|`이면 status 3, 그 밖은 status 0으로 다음 노드. `|cc| ≤ 1e-13`이면 `|d33| ≤ FT`일 때 다음 노드, 아니면 b의 bound가 0을 포함하지 않을 때 status 3, 포함하면 `lb ≠ 0 || ub ≠ 0`일 때 `3de18e(0, …)`.

#### 7.2.6 2단계: 대입 적용 (L141–487)
`nsub > 0`이면 roots 버퍼(malloc 4n)를 잡고 `list` 순서(발견 순서)대로 돈다.
```
for j in list[0..nsub]:
    (root, m, o) = parent[j] != 0 ? chain(j) : (j, 1.0, 0.0)
    if !(1e-8 <= |m|): continue            // 아무것도 안 함(정의 행은 이미 지워졌다. 기본값에서 미관측)
    if root != j:
        cnt0 = col_cnt[j]; head = colhead[j]; colhead[j] = NULL; tail = NULL
        eq = 동치(j, root, m)                 // 아래. 노드마다 다시 계산하지만 값은 같다
        for node in 열 j 리스트(원래 순서):
            if node.col < 0: tail 뒤에 붙인다(tail == NULL이면 colhead[j] = node); continue
            r = node.row; a = node.val
            if !eq: 기여 빼기(r, a, j)         // 평범한 뺄셈, 보정 없음
            node.col = root; node.val = m·a
            3c3abf(st, r, &node.col, j; new = m·a, old = a, INF)   // |m·a| ≤ 1e-13이면 노드를 죽인다(7.2.8)
            if !eq && node.col >= 0: 기여 더하기(r, m·a, root)
            if o != 0: d = a·o; amax[r] = amax[r] + d; amin[r] = amin[r] + d; rhs[r] = rhs[r] - d   // 이 순서
            node.nextcol = colhead[root]; colhead[root] = node; col_cnt[root] += 1   // 맨 앞에 붙인다
            if rmark[r] == 0: rmark[r] = 1; touched[ntouched++] = r
            col_cnt[j] -= 1
        W += (cnt0·16.0)·μ
        if tail: tail.nextcol = NULL           // 열 j에는 죽은 노드만 원래 순서로 남는다
        col_cnt[j] = -3; +0x408 += 1
        obj[root] = obj[j]·m + obj[root]; objcon(+0xc8) = obj[j]·o + objcon; obj[j] = 0
        39d9e1(m, o, env, st, j, root)        // asm 0x4b67aa: xmm0 = m, xmm1 = o, edx = j, ecx = root
    roots.push(root)                          // root == j여도 넣는다
W += nsub·μ
```
- **동치(j, k, m)**: 기여가 그대로인가를 본다. 상수 1e30을 쓰고 o는 보지 않는다(o의 몫은 따로 `+d`로 넣는다).
  - m > 0: (lb_j == lb_k·m 또는 lb_j == lb_k == −1e30) 그리고 (ub_j == ub_k·m 또는 ub_j == ub_k == 1e30).
  - m < 0: (lb_j == ub_k·m 또는 lb_j == −1e30 && ub_k == 1e30) 그리고 (ub_j == lb_k·m 또는 ub_j == 1e30 && lb_k == −1e30).
- **기여 빼기/더하기(r, a, c)**(1단계 밖 모든 곳 공통, asm 0x4b574a·0x4b58e3):
  - a > 0: lb_c > −INF면 amin −= lb·a(더하기는 amin = lb·a + amin), 아니면 imin ∓ 1. ub_c < INF면 amax −= a·ub, 아니면 imax ∓ 1.
  - a < 0: lb는 amax/imax로, ub는 amin/imin로 간다.
  - a == 0이면 아무것도 하지 않는다. 1±2⁻⁵² 보정은 없다.
- 노드 수는 그대로다. free·pending·+0xd8도 바뀌지 않는다. 행 리스트의 노드 위치도 그대로라서, 옮긴 노드는 행 안에서 원래 자리에 열 번호만 바뀐 채 남는다. 행 리스트는 더 이상 열 번호순이 아니다.

#### 7.2.7 3단계: 건드린 행의 중복 열 합치기 (L488–699)
`touched`(처음 건드린 순서)의 행마다 다음을 한다. 머리가 NULL이거나 머리.row < 0이면 건너뛴다.
```
vis = 0; dups = 0
for node in row r:
    c = node.col
    if c >= 0:
        if pos[c] == NULL: pos[c] = node
        else:
            p = pos[c]                                   // 앞선 노드가 남는다
            기여 빼기(p.row, p.val, p.col); 기여 빼기(r, node.val, c)
            s = p.val + node.val
            if |s| <= 1e-13 { p.val = 0.0 } else { p.val = s; 기여 더하기(p.row, s, p.col) }
            col_cnt[c] -= 1; dead += 1; dups += 1; row_cnt[node.row] -= 1; node.col = -1
    vis += 1
W += (dups·11.0 + vis·3.0)·μ
vis = 0; z = 0
for node in row r:
    if node.col >= 0:
        pos[node.col] = NULL
        if node.val == 0.0: col_cnt[col] -= 1; dead += 1; z += 1; row_cnt[node.row] -= 1; node.col = -1
    vis += 1
W += ((vis + vis) + z·5.0)·μ
// (머리 검사로 건너뛴 행도) rmark[r] = 0
```
`ntouched > 0`이면 끝에 `W += (ntouched + ntouched)·μ`, `5382b5(tracker, ntouched, touched)`를 한다. 2·3단계 전체는 `nsub > 0`일 때만 돈다.

#### 7.2.8 4단계와 끝 (L700–761)
```
ret = 3ba5ac(env, st, 0, 0, -1)
if ret == 0 && (flag8 없음 || 560ea1(st, env, 1, parent, mult, off) == 0) && (flag30 없음 || 562f34(...) == 0):
    for c in roots: if 0 < col_cnt[c] { 5379f1(tracker, c) }
    if roots 비지 않음: W += (nroots + nroots)·μ
    if 1000000 < dead: 36e6b6(st, 2); dead = 0
if ret == 0: ret = 3ba5ac(env, st, 0, 0, -1)
free(tmp, list, parent, roots); return ret
```
- flag8/flag30은 대입한 열 j의 `flag & 8`/`flag & 0x30`을 모은 것이다. mode 0이면 늘 거짓이다.

#### 7.2.9 보조 함수
- **`39d9e1(xmm0 = m, xmm1 = o, env, st, j, root)`**(261 B, asm 확인):
  - m ≠ 1.0이면 `mlo[root] = minsd(mlo[root], mlo[j]·|m|)`, `mhi[root] = maxsd(mhi[root], |m|·mhi[j])`이다. minsd/maxsd는 같을 때 둘째 인자를 고른다.
  - 그다음 `ac68d7(env, st+0x368, 0x50)`로 레코드를 잡아 목록 머리(st+0x370)에 넣는다. 실패하면 0x2711.
  - **type 2 레코드(MIP 꼴, 0x50 B)**:

    | rec 오프셋 | data 오프셋 | 내용 |
    |---|---|---|
    | +0x00 | | i32 type = 2 |
    | +0x08 | | data = rec+0x18 |
    | +0x10 | | next(옛 +0x370) |
    | +0x18 | +0x00 | i32 j |
    | +0x1c | +0x04 | 패딩(초기화 안 됨) |
    | +0x20 | +0x08 | f64 o |
    | +0x28 | +0x10 | i32 개수 = 1 |
    | +0x2c | +0x14 | 패딩 |
    | +0x30 | +0x18 | i32* ind = rec+0x40 |
    | +0x38 | +0x20 | f64* val = rec+0x48 |
    | +0x40 | +0x28 | i32 root |
    | +0x44 | +0x2c | 패딩 |
    | +0x48 | +0x30 | f64 m |

    뜻은 x_j = o + m·x_root다. o는 −0.0일 수 있다(`rr/a_s`에서 rr = 0, a_s < 0). 작업량은 없다. LP type 2(`39b9e0`, j·r·c0·a·obj·cnt·ind·val·mode·열 항목)와 필드 배치가 다르다.
  - `39d9e1`은 `55e383`, `56b360`, `56c89d`도 부른다.
- **`3c3abf(xmm0 = new, xmm1 = old, xmm2 = INF, st, r, &node.col, j_old, lb, ub)`**(689 B, Ghidra 인자가 틀려 asm으로 읽음):
  - |new| > 1e-13(2a68188)이거나 node.col < 0이면 돌아간다.
  - 아니면 old 계수의 기여를 **j_old의 bound로, 보정 덧셈으로** 뺀다.
    - old > 0: ub ≥ INF면 imax−−, 아니면 amax = gup(amax, −(ub·old)). lb ≤ −INF면 imin−−, 아니면 amin = gdn(amin, −(old·lb)).
    - old ≤ 0: ub ≥ INF면 imin−−, 아니면 amin = gdn(amin, −(ub·old)). lb ≤ −INF면 imax−−, 아니면 amax = gup(amax, −(lb·old)).
    - g(x, d): s = x + d. 정확성은 |x| ≥ |d|면 (s − x) == d, 아니면 (s − d) == x로 본다. 부정확하면 gup은 s > 0일 때 s·(1+2⁻⁵²), 아니면 s·(1−2⁻⁵²)이고, gdn은 s < 0일 때 s·(1+2⁻⁵²), 아니면 s·(1−2⁻⁵²)이다.
  - 그다음 row_cnt[r]−−, col_cnt[node.col]−−(호출 때 node.col은 이미 root라 col_cnt[root]−−), node.col = −1.
  - 56c80e에서 eq가 거짓이면 old 기여가 **두 번 빠진다**(인라인 평범한 뺄셈 한 번, 3c3abf 보정 뺄셈 한 번). eq가 참이면 한 번만 빠진다.
  - 개수는 56c80e의 col_cnt[root]++와 상쇄되고, col_cnt[j]만 1 준다. 죽은 노드도 root 열 리스트 앞에 붙는다. o ≠ 0이면 rhs·activity의 `+d`도 그대로 한다.
  - Gurobi 동작 그대로 옮긴다. 기본값에서는 관측하지 못했다.
  - `3c3abf`는 `3ff744`, `57627e`, `56b360`도 부른다.
- **`5382b5(tracker, cnt, list)`**(188 B): 목록의 행마다 `53763f`와 같은 표시를 한다(레벨 1·2 큐, tracker+0x8c/+0x90 활성이면 b 뒤에, 아니면 a 뒤에 넣고 비트 1/4, 활성이면 비트 2/8). 작업량은 없다. `56b360`도 쓴다.
- **`560ea1`(2,656 B)**: Q 목적(+0x250 대각, +0x260/+0x268/+0x270 비대각)에 대입을 반영한다. **`562f34`(3,484 B)**: QC 제약(+0x27c개, +0x290–+0x2e0)에 반영한다. 둘 다 이 호출 지점에서는 도달하지 않는다(§7.1).
- **`36e6b6(st, 2)`**: `566cc5`/`569830`의 행 지우기에서 dead > +0xd8일 때, 56c80e 끝에서 dead > 1e6일 때 부르고, 부른 뒤 dead = 0. 기본값 오라클에서는 한 번도 불리지 않았다(`36e6b6` 해독은 §8).

#### 7.2.10 작업량 요약 (μ = st+0x448, 모두 `W = x + W`)
| 위치 | x |
|---|---|
| 538f30(캐시 없을 때) | 연결 리스트 분기 규칙 |
| 566cc5 진입 | ((10 + vis) + vis)·μ (고정 열에서 멈추면 vis는 그 앞까지) |
| 사슬(566cc5·569830) | (k + k)·μ, (4·μ)·(k−1) |
| 566cc5 열 s 검사 | (col_cnt[s]·3)·μ |
| 566cc5 행 지우기 | (row_cnt·2)·μ |
| 569830 | (cnt·7)·μ; 후보 열마다 (col_cnt·3)·μ; 루프 끝 (cnt·8)·μ; 행 지우기 (cnt·3)·μ |
| 1단계 끝 | (Rcnt·10)·μ |
| 2단계 | 열마다 (k + k)·μ, ((k−1)·4)·μ, (cnt0·16)·μ. 끝 nsub·μ |
| 3단계 | 행마다 (dups·11 + vis·3)·μ, ((vis + vis) + z·5)·μ. 끝 (nt + nt)·μ |
| 4단계 | (nroots + nroots)·μ |
| `3dd6f1` | 그 함수의 몫(합성 모델에서 노드 3개 열에 18 = 6·3) |

### 7.3 LP 쌍둥이와 차이 (`37f484` 단계 D, m1_presolve_agg §8)
같은 것은 두 가지다. 행 지우기가 lazy인 점(노드 col = −1, row_cnt 음수), 그리고 `mlo`/`mhi` 배율 검사 아이디어. 나머지는 모두 다르다. **LP `agg.rs` 코드는 재사용할 수 없다.** 쓸 수 있는 것은 추적기 표시(`mark_row`/`mark_col`)와 LP의 보정 덧셈 루틴(3c3abf용) 정도다.

| | LP 단계 D | MIP `56c80e` |
|---|---|---|
| 지울 열 | |a|가 큰 쪽 | 타입 규칙(연속 > 정수, 정수끼리는 |a| 작은 쪽, 동률이면 개수 적은 쪽·I보다 B 남김) |
| 적용 시점 | 행마다 즉시 fill-in(`upd`, 새 노드) | 사슬에 쌓았다가 끝에 노드 재표시(새 노드 없음) |
| activity | 갱신 안 함(`370047`가 다시 계산) | 2·3단계에서 평범한 덧셈으로 갱신 |
| bound | 옮기기 + type 6 | 정수 ceil/floor(±IFT), 1e-10 + F·1e-6 문턱, `3dd6f1`. 레코드 없음 |
| 레코드 | type 2(LP 꼴, 열 항목 포함) | type 2(MIP 꼴, 항목 1개, 0x50 B, `39d9e1`) |
| +0xd8·free·pending | 바뀜 | 안 바뀜 |
| 지운 행·열 표시 | row −3, col −3−r | row −1, col −3 |
| 3항 이상 | 없음 | `569830` |

### 7.4 오라클 확인 (2026-11-06)
도구:
- 스크래치 `inner/bin/dump56.js`: `mip_state_dump.js` 사본에 다음을 더했다. 노드 주소, mlo/mhi/flag/IFT, env 값, 함수 안 `538f30` 출력과 작업량, `36e6b6` 호출, 추적기(플래그, a/b, 레벨별 목록).
- `inner/bin/sim56.py`: 이 노트대로 짠 Python 참조 시뮬레이터. 진입 상태에서 56c80e를 돌려 복귀 상태와 비트 단위로 비교한다. 비교 항목은 lb/ub/obj/rhs, amax/amin/imax/imin, row/col_cnt, 카운터, objcon, 작업량, mlo/mhi, 행·열 리스트 순서와 값, 레코드 필드, 추적기다.

| 모델 | 호출 | 결과 |
|---|---|---|
| neos-3046615-murg | 3 | 3/3 일치. 1번 호출: 행 240개 지움, 대입 120개(B/B 동률 규칙), 같은 루트 120개(|d35| ≤ 1e-13, |d45| ≤ FT 경로) |
| sp150x300d | 6 | 6/6. 대입 84개(C/C), 건드린 행 127개(중복 합치기 포함) |
| neos-2657525-crna | 4 | 4/4. 대입 72개(B/B, I/I, B/I 동률), 같은 루트 2개 |
| tr12-30 | 13 | 13/13. 대입 12개(C/C, keep2) |
| csched008 / csched007 | 7 / 5 | 전부 일치. 대입 10개 / 1개(B/C 혼합 포함) |
| timtab1, enlight_hard, neos859080, ic97_potential, mik-250-20-75-4, gmu-35-40, markshare2 | 34 | 전부 일치(대입 없음, `569830` 진입 1–39회·작업량 7·cnt 포함) |
| 합성 t569(이진 등식 + 2항) | 8 | 전부 일치. `569830` 루프·대입 2개·행 지우기, 행 안 중복 합치기(남는 노드 = 앞선 노드), 레코드 바이트 |
| 합성 t566 / t566b / tint | 2 / 5 / 10 | bound·카운터·리스트·레코드 일치. b17 강화(ub 10→6), "t 고정 → s 결정"(ub 두 개 0), 정수 비율 규칙(I/I keep2, B/I keep1), o = −0.0 레코드. `3dd6f1`이 바꾼 activity와 작업량은 비교에서 뺐다(`3dd6f1`은 노드 3개 열에 W 18) |

기본값 38개에서 실제로 도는 경로:
- `566cc5`: 6개 모델. 동률 규칙과 혼합 규칙, 같은 루트의 "d45 ≈ 0" 경로가 돈다. bound 강화·t 고정·`3de18e`는 한 번도 돌지 않는다. 앞선 `37168e`가 먼저 강화하기 때문이다.
- `569830`: 11개 모델에서 불리지만 모두 루프 앞에서 돌아간다.
- `3c3abf`: 옮긴 노드마다 불리지만 노드를 죽인 적은 없다.
- `36e6b6`(56c80e 안): 0회.
- μ는 모두 1이었다.

### 7.5 미해결
1. μ ≠ 1(nnz > 1e5) 모델에서 작업량 곱셈 순서((4·μ)·k와 (k·4)·μ)를 확인하지 못했다. asm 순서대로 적었다.
2. `3c3abf`가 노드를 죽이는 경로(old 기여가 두 번 빠지는 것으로 읽힘), 같은 루트에서 `3de18e`를 부르는 경로, `569830`의 같은 루트 분기, status 3 경로, `36e6b6(st, 2)` 호출은 오라클로 확인하지 못했다. 합성 모델로는 앞선 reduction(`36ddf2` 작은 계수 제거, `37168e`)이 먼저 처리했다.
3. `569830`과 2단계에서 |m| < 1e-8이면 사슬의 오프셋을 버리거나 대입을 건너뛴다. 정의 행이 이미 지워진 뒤라 Gurobi 버그로 보인다. 그대로 옮기면 되지만 관측하지는 못했다.
4. st+0x1b0(pos)과 st+0x3a8(rmark)이 진입 때 NULL·0이라는 가정은 오라클에서 1b0만 확인했다(진입·복귀 모두 NULL 0개). 3a8은 덤프하지 않았다.
5. 레코드 패딩 바이트는 arena 쓰레기값이다. 포트와 비교 도구는 필드만 비교해야 한다.

## 8. `36e6b6` 노드 GC (`00477d30`)

### 8.1 서명·가드·반환
`36e6b6(st, full)`, void. 3cd406 내부 루프 끝은 full = 0, `local_950` > 1e6 분기와 바깥(L2948)·`56c80e`는 full = 1.

### 8.2 알고리즘
```
if full == 0 {
  pool = pending(+0x190, 개수 +0x188)
  (nr, R) = 538353(tr); (nc, C) = 5383d0(tr)          // 레벨 원시 a-목록
} else {
  if pending 머리 != NULL {                            // pending을 free 앞에 잇는다
     끝까지 걸으며 링크 수 L(= 길이−1)을 센다; W += (L·5)·μ
     끝.next = free 머리; +0x198 += +0x188; free 머리 = pending 머리; pending = NULL, +0x188 = 0
  }
  pool = free(+0x1a0, +0x198); R = 0..m−1; C = 0..n−1
}
live = 0
for &j in C {                                           // 열 먼저
  if 열 머리 == NULL { continue }
  for node in col j list (+0x18, 순서대로) {
    if node.col < 0 { node를 pool 머리에 넣는다(node.next_col = pool; pool = node); pool 개수 += 1; +0xd8 -= 1 }
    else { 살아 있는 노드끼리 순서대로 다시 잇는다; live += 1 }
  }
  W += (방문·μ)·8; 마지막 링크 = NULL
}
W += (len(C)·3)·μ
for &i in R {                                           // 행: 죽은 노드를 떼기만(개수 변화 없음)
  if 행 머리 == NULL { continue }
  살아 있는 노드만 순서대로 다시 잇는다; W += (방문·μ)·8
}
W += (len(R)·3)·μ
if full { +0xd8 = live }                                // 살아 있는 열 노드 수로 다시 맞춘다
```
- pool에 넣는 순서는 열 순서·리스트 순서대로 머리 삽입이라, 나중에 꺼내는 순서(56c80e·37f484의 새 노드)는 그 역순이다.
- full = 0은 이번 반복 목록(reset 때 고정, §1.2)에 든 행·열만 청소한다. 이번 반복에 죽은 노드가 다른 행·열에 있으면 다음 반복까지 남는다.
- full = 0의 pending 노드는 아직 다른 행 리스트에 남아 있을 수 있어 free로 못 보낸다. full = 1은 모든 행을 청소하므로 pending을 free로 합칠 수 있다.

### 8.3 LP 쌍둥이(`53d3d9`, m1_presolve_agg §2.3)와 차이
`53d3d9`는 플래그 0x10이 선 행·열만 청소하고 pending을 늘 free에 합치며 죽은 노드를 바로 free로 보낸다. 작업량도 2·방문, 3·b. `36e6b6`은 추적기 a-목록 또는 전체를 돌고, 열 방문마다 8, 목록마다 3이다.

### 8.4 오라클 확인
38개 모델 225회 모두 일치(full = 0/1 둘 다. 덤프에 인자가 없어 두 가지를 돌려 맞는 쪽을 골랐다: full = 1로만 맞는 호출 66회, gmu·timtab1·tr12-30 등). free/pending 개수, +0xd8, 리스트 순서, 작업량 비트 일치.

### 8.5 미해결
없음.

## 9. 오라클 확인 방법과 결과

도구(세션 스크래치 `inner/bin/`, 저장소에는 넣지 않았다. 포팅 때 비교 기준으로 쓴다):
- `dump_tr.js`: `tools/mip_state_dump.js` 사본에 추적기(레벨, 행·열 플래그, 레벨별 a/b/목록, `538f30`/`5399fc` 캐시, active)를 더한 것. `dump2.js`(§2, 열 플래그 +0x170·mlo/mhi·행 캐시 +0x3b0·misc 필드), `dump56.js`(§7, 노드 주소·env 값·함수 안 `538f30` 출력)도 같은 계열이다. 포트 비교에는 이 필드들이 필요하므로 포팅할 때 저장소 도구에 합친다.
- `sim.py`(§1·3–6·8), `sim39.py`(§2), `sim56.py`(§7): 이 노트의 의사 코드를 그대로 옮긴 Python 모형. 진입 덤프를 넣고 돌려 복귀 덤프와 비교한다(double은 `!=`, 리스트는 [행, 열, 값] 순서, 레코드는 type과 필드, 추적기 플래그·목록).
- `run.sh <fn> <모델…>`(모든 호출 덤프), `chk.py <fn> <모델…>`, `chk_gc.py`, `chk1.py`.

결과(기본값, Threads=1, WorkLimit 60, MIPLIB small 38개. "호출"은 3cd406의 모든 호출부를 합친 수):

| 함수 | 모델 | 호출 | 일치 | 지난 경로(모형 계수) |
|---|---|---|---|---|
| `3e39a6` | 38 | 147 | 147 | 고정 212, 빈 열 1,166, 쌍대 고정 628, mode0 29, mode1/2 463, 행 반전 456, `5399fc` 희소 66 |
| `36ddf2` | 38 (+합성 3) | 185 (+25) | 전부 | 빈 행 157, 중복 행 834, `538f30` 조밀 83·희소 64. 작은 계수는 합성 t3c에서 1 |
| `37168e` | 38 (+합성 3) | 225 (+22) | 전부 | 적용 3,829(lb 342, ub 3,527, lb만 302), 정수 floor 완화 32. 연속 넓히기는 합성 s37에서 3 |
| `36c5cf` | 38 (+PREBOUND=0 6개, 합성 3) | 147 (+38, +21) | 전부 | 기본: `=` 연속 4. PREBOUND=0·합성: kind 1 C/I, kind 2 C/I, kind 3 C/I, 정수 행 남기기. conflict 0 |
| `56f793` | 38 | 147 | 147 | a ≤ 0 354,894(ub 무한 1), a > 0 1,064. big 0 |
| `56c80e` | 13 + 합성 5 | 72 + 31 | 전부 | §7.4 |
| `36e6b6` | 38 | 225 | 225 | full = 0으로 맞음 159, full = 1로만 맞음 66(인자는 덤프에 없어 두 가지를 돌려 골랐다) |
| `370047` 첫 호출(목록 NULL) | 38 | 38 | 38 | |

- neos-860300은 μ = 1.2588…(nnz > 1e5)이라 그 모델에서 지난 경로는 작업량 곱셈 순서까지 확인됐다. 나머지 37개는 μ = 1이다.
- 3dd6f1, 3de18e(rec = 1, 0), 39ab33 레코드 바이트, 537da8/53802c/5379f1/53763f, 538f30·5399fc 연결 리스트 분기와 캐시는 위 함수들을 통해 같이 확인됐다. `3dcc78`은 mode 1 단일 행만(§2), mode 2는 확인하지 않았다. `3df916`·`3a061e`는 내부 루프 밖이라 확인하지 않았다.
- JSON 덤프는 −0.0의 부호를 잃는다. 부호 있는 0은 레코드 원시 바이트에서만 봤다(murg c0, mas74 v, 합성 o).

## 10. 미해결 (모음)
1. 오라클이 지나지 않은 경로(해독만): `36c5cf` conflict, `37168e` nf ≥ 1 경로, `56f793` big(370047)·a > 0 lb 무한, `3e39a6` 행 캐시 적중·G 실패·반환 3/5·PRESINGLECOL ≠ −1·열 플래그 ≠ 0, `56c80e` 3c3abf가 노드를 죽이는 경로·같은 루트의 `3de18e`·status 3·`36e6b6(st, 2)`. 합성 모델로 확인할 것.
2. `39e479`(3e2b99의 무한 bound 쪽 쌍대 제거)는 해독하지 않았다. 기본 38개에서 안 돈다.
3. 열 플래그 +0x170: 38개 모두 0. 0x18 비트(37168e tol = 10·ft), 0x80(PWL), 0x08의 출처를 모른다. `st+0x248`, `st+0x278`의 뜻도 모른다(MIP에서 0).
4. `56c80e`: 사슬의 |m| < 1e-8이면 대입을 건너뛰는데 정의 행은 이미 지워진 뒤다(Gurobi 버그로 보임, 그대로 옮긴다). 3c3abf가 노드를 죽이면 옛 기여가 두 번 빠지는 것으로 읽힌다. st+0x3a8(행 표시)이 진입 때 0이라는 가정은 덤프로 확인하지 않았다.
5. μ ≠ 1의 곱셈 순서는 neos-860300에서 지난 경로만 확인됐다. `56c80e`(그 모델에서 대입 없음)의 `(4·μ)·k`와 `(k·4)·μ` 같은 차이는 asm으로만 정했다.
6. `3dcc78` mode 2(바깥 reduction용)는 오라클 확인 전이다.
7. 레코드 패딩 바이트는 arena 쓰레기값이다. 포트·비교 도구는 필드만 비교한다.

## 부록 A. 합성 모델 (세션 스크래치 `inner/syn/`, `dump_tr.js`는 `.lp` 경로도 받는다)
§7 확인에 쓴 t566·t566b·t569·tint(§7.4)도 같은 디렉터리에 있다.

t3c.lp(36ddf2 작은 계수, 기본값):
```
Minimize
 x + y - v + 0.5 w
Subject To
 R0: 1000000 v - x = 0
 R1: 0.00000001 v + y + w >= 1
 R2: x + y - w <= 3.7
 R3: x + 3 y - 2 w >= -2
Bounds
 0 <= x <= 10
 0 <= v <= 0.00001
 0 <= y <= 4
 0 <= w <= 4
Generals
 x
End
```
s36.lp(36c5cf, `GURO_PAR_PREBOUND=0`):
```
Minimize
 x1 + x2 + x4 + x6 + y1
Subject To
 c0: x1 + x2 + x4 + x6 + y1 = 5
 s1: 2 x1 <= 7
 s2: 3 x2 >= 4.5
 s4: x4 <= 0.5000001
 s5: y1 <= -0.0000005
 s7: x6 <= 3.999995
Bounds
 0 <= x1 <= 10
 0 <= x2 <= 10
 0 <= x4 <= 10
 0 <= x6 <= 4
 0 <= y1 <= 10
Generals
 x1 x2 x4 x6
End
```
s37.lp(37168e 연속 넓히기, 기본값):
```
Minimize
 x + y + z
Subject To
 e0: 2 x + y = 3
 e1: x + z - u = 2.5
 r2: z + u <= 8
Bounds
 0 <= x <= 10
 1 <= y <= 1.000001
 0 <= z <= 5
 0 <= u <= 5
Generals
 u
End
```
