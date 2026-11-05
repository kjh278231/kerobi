# M2 MIP presolve 라운드 끝 reduction 해독 (앞쪽: `58d4ce`, `5577ba`, `5a9282`, `5ab429`, `59c193`, `55a779`, `594c0c`)

`3cd406` 라운드 끝 단계(호출 위치·가드·timer·카운트는 [m2_spine_c.md](m2_spine_c.md) §2–§6)의 앞쪽 reduction 해독과 포팅 노트. `5a9282`는 패스 루프 앞(프렐류드 끝, [m2_spine_b.md](m2_spine_b.md) §2)에서도 한 번 불린다. 표기는 [m2_red_inner.md](m2_red_inner.md)와 같다.
- `W += x·μ`는 `*(state+0x458) += x * *(state+0x448)`. 곱의 순서는 원본 그대로 적는다(`(μ·5)·k`는 `μ*5.0*(double)k`).
- 파일 오프셋 = Ghidra 주소 − 0x100000. 공용 보조 함수(`3de18e` = `fix_col`, `3dd6f1` = `bound_change`, `39ab33` = `push_fix` 등)는 m2_red_inner.md §1·§11의 이름을 쓴다.
- "확인"은 `tools/mip_dump_check.py`로 Gurobi 진입 상태(`tools/mip_state_dump.js … all`)에서 포트를 돌린 결과가 복귀 상태와 모든 필드(배열 비트, 리스트 순서, 개수, 카운터, 작업량, 추적기, 레코드 바이트)에서 같다는 뜻이다. 덤프는 세션 스크래치 `re/d/<모델>_<함수>.json`(`re/dump.sh <fn>`).

## 0. 요약

| 함수 | 크기 | 가드(3cd406) | 하는 일 | 바뀌는 모델(기본값 38개) | 확인 |
|---|---|---|---|---|---|
| `58d4ce(env, st, W)` | 361 | Aggregate·DualReductions·+0x1e4 | Q 대각이 0인 열 → `3c2f36`. Q가 없으면 바로 0 | 없음 | §1 |
| `5a9282(st, env)` | 2,465 | PREBOUND·DualReductions | 행의 singleton 열 지배: 비용 비율이 좋은 singleton을 activity가 작은 쪽 bound에 고정 | gmu-35-40, gmu-35-50(프렐류드 호출, 72열) | 80/80(§2) |
| `5ab429(st, env)` | 1,113 | 같음 | 정수 행의 비용 singleton(|a| = 1)을 계수 gcd g로 x = r + g·y 대입(`574764`) | 없음(작업량도 0) | 42/42 + 합성 9(§3) |
| `59c193(st, env, W)` | 7,021 | PREVUBLIFT | VUB 행 쌍 정리, 클릭 같은 행을 공유하는 VUB 합치기 | 없음(작업량만) | 42/42 + 합성 20(§4) |
| `55a779(env, st, T20)` | 10,702 | round 0, PreSparsify | 긴 행을 열 종류별로 잘라 계수 패턴 사전을 만들고, 두 번 나온 패턴을 새 열로 묶음(2부) | 없음(작업량만, 2부는 0회) | 38/38(§5) |

## 1. `58d4ce` Q 대각 열 (`005c9200`)

```
if env.DualReductions(0x2430) == 0 || st+0x250(Q 대각) == NULL || n <= 0 { return 0 }
for j in 0..n {
  if Q[j] == 0.0 && vtype[j] != 'B' && colflag[j] & 0xfffffff4 == 0 && col_cnt[j] == 0 {
    d = ub[j] − lb[j]
    if 1e-10 < d && d <= 1e6 {
      half = 0
      if colflag[j] & 3 != 0 && 1e-13 < |lb[j]| { if 1e-13 < |ub[j]| { continue }; half = 1 }
      rc = 3c2f36(env, st, j, half, W); if rc != 0 { return rc }
    }
  }
}
return rc(마지막 값, 0)
```
- **선형 모델에서는 Q(+0x250)가 NULL이라 작업량도 없이 0을 돌려준다.** 기본값 38개 42회 모두 상태 변화 없음(coverage에서 `3c2f36` 0회).
- 포트: `q_diag`는 0을 돌려주는 빈 메서드(`roundend.rs`). Q 분기는 `shortcut:`.

## 2. `5a9282` 행의 singleton 열 지배 (`005d7d00`)

### 2.1 서명·가드·반환
`5a9282(st, env) -> i32`. 호출: 프렐류드 끝(카운트 3)과 라운드 끝(PREBOUND && DualReductions, m2_spine_c §2 D). 자체 가드: `st+0x100 == 0 && env.DualReductions != 0 && st+0x330(PWL 목적 수) == 0 && st+0x1e0(PreCrush) != 2`. 아니면 작업량 없이 0. 반환은 0 또는 `3de18e`의 값(기본에서는 늘 0).

### 2.2 알고리즘
```
rows = 538f30(tr, st)                       // 라운드 끝·프렐류드 모두 추적기 레벨 0: 원시 목록(지운 행 포함), 작업량 없음
for r in rows {
  if row_cnt[r] < 0 { continue }
  for s in [-1.0, +1.0] {                    // −1은 '=' 행만
    if sense[r] != '=' && s != 1.0 { continue }
    rest = s·rhs[r]; top = 0; ns = 0; v = 0
    int_rest = no_int = no_cont = true
    single(j, a) = col_cnt[j]==1 && colflag[j]==0 && (vtype[j]=='C' || |1 − |a|| < 1e-10)
    for node in row r (행 리스트, 죽은 노드 포함해 v를 센다) where node.col >= 0 {
      a = s·node.val
      if single(j, a) {
        top = top + (a < 0 ? lb[j] : ub[j])·a; ns += 1
        vtype[j]=='C' ? no_cont = false : no_int = false
      } else {
        rest = rest − (a <= 0 ? ub[j] : lb[j])·a
        if vtype[j]=='C' || 1e-10 < a − floor(a + 1e-10) { int_rest = false }
      }
    }
    W += (μ·5)·v
    if !(1 < ns && ns != row_cnt[r] && rest < 1e10 && top < 1e10 && rest + 1e-6 <= top
         && (no_int || (int_rest && rhs[r] − floor(rhs[r] + 1e-10) <= 1e-10 && no_cont))) { continue }
    // singleton 모으기: 열 j, a(+0x398[j]), 키 −obj[j]/a(+0x3b0[k]), 목록 +0x390
    for node in row r (v2를 센다) where node.col >= 0 && single(j, a) { 목록에 (j, a, −obj[j]/a) }
    W += (μ·10)·v2
    aaedd9(k, 키, 열)                         // 키 오름차순(LP sort.rs와 같은 순열), 뒤에 +0x3b0을 0으로
    l = 0
    while l < k { j = 열[l]; d = |(ub[j] − lb[j])·a_j|; if 1e10 < d { break }
                  top −= d; if top < rest − 1e-10 { break }; l += 1 }
    W = (μ·4)·l + W
    for 앞 l개 { 3de18e(a_j <= 0 ? ub[j] : lb[j], INF, env, st, j, rec=1) }   // type 1 레코드, 열 −2
    W = W + (μ·3)·l
  }
}
W += (μ·5)·len(rows)
```
- 뜻: 행 `s·a·x ≤ s·rhs`에서 singleton 아닌 열을 모두 최소 activity에 두어도 남는 몫 `rest`보다 singleton들의 최대 activity `top`이 크면, −c/a가 작은 singleton부터 activity가 작은 쪽 bound에 고정한다. 남은 singleton만으로 `rest`를 덮을 수 있는 동안만 한다.
- `'='` 행은 s = −1 쪽을 먼저 하고, 거기서 고정한 결과(죽은 노드, `row_cnt`)를 s = +1 쪽이 본다. `row_cnt ≥ 0` 검사는 두 쪽 앞에서 한 번이다.
- `+0x398`/`+0x390`/`+0x3b0`은 호출 안의 scratch다(`+0x3b0`은 정렬 뒤 0으로 되돌린다). 포트는 지역 벡터로 둔다.
- vtype NULL 분기(정수 판정 없이 모두 singleton)는 MIP 상태에서 안 돈다.

### 2.3 오라클 확인
38개 모델 80호출(프렐류드 38 + 라운드 끝 42) 모두 일치. 바뀌는 것은 gmu-35-40·gmu-35-50의 프렐류드 호출뿐이다(행 40개에서 열 72개 고정, type 1 레코드 72개, `+0x408` +72). 나머지는 작업량만 바뀐다. sweep: 두 gmu 모델의 첫 불일치가 k=3(`r5a9282`)에서 k=28(`knapsack`)로 옮겨 갔다.

### 2.4 미해결
- `st+0x100`의 뜻은 모른다(기본 38개에서 가드를 통과하므로 0).

## 3. `5ab429` 비용 singleton의 법 g 대입 (`005d8b80`)

### 3.1 서명·가드·반환
`5ab429(st, env) -> i32`. 라운드 끝에서 `5a9282` 바로 뒤(같은 가드). 자체 가드 없음. 자체 작업량 없음(작업량은 `574764` 안에서만). 반환은 마지막 `574764`의 값(0) 또는 그 오류값.

### 3.2 알고리즘
```
for r in 0..m {                                   // 지운 행도 돈다(row_cnt 검사 없음)
  slot = [-1, -1]; res = rhs[r]; g = 1e100
  for node in row r where node.col >= 0 {
    (c, a) = (obj[j], node.val)
    if |c| > 1e-10 && col_cnt[j] == 1 && ||a| − 1| < 1e-10 && colflag[j] == 0 && slot[0 < c·a] == -1 {
      res = res − (c > 0 ? lb[j] : ub[j])·a; slot[0 < c·a] = j        // 비용이 원하는 bound
    } else if vtype[j] == 'C' { 다음 행 }                               // 연속 열이 있으면 행을 버린다
    else { g = a7c550(g, |a|) }
  }
  if |res| > 1e-10 { g = a7c550(g, |res|) }
  if !(1.5 > g) && !(g > 1e6) && g − floor(g + 1e-10) <= 1e-10 {
    for t in 0..2 { j = slot[t]; if j >= 0 && vtype[j] != 'C' {
      gi = (int)floor(g + 0.5)
      rem = (int)((i64)(obj[j] > 0 ? lb[j] : ub[j]) % gi)              // cvttsd2si 64비트, idiv
      rc = 574764(st, env, j, 1, gi, rem, &done); if rc != 0 { return rc } } }
  }
}
```
- `a7c550(a, b)`(264 B): 실수 gcd. |a|, |b| 중 하나라도 1e-10 미만이면 0. |a| ≥ 1e100이면 |b|, |b| ≥ 1e100이면 |a|. hi = max, lo = min으로 `lo > 1e-10`인 동안 `lo > hi`면 0, `t = hi − floor((1e-10 + hi)/lo)·lo; hi = lo; lo = max(t, 0)`. 끝에 hi < 1e-9면 0, 아니면 hi.

### 3.3 `574764(st, env, j, a, g, rem, &done)` 정수 열의 법 대입 (1,312 B)
`57627e`(implied int, outer-a 담당)도 부른다. 포트는 `State::mod_subst`(roundend.rs).
```
rem = rem % g; done = 0
r = 0
if rem != 0 { inv = 3c5996(|a|, g); r = ((i64)inv · (rem < 0 ? rem + g : rem)) % g; if a < 0 { r = −r } }
if lb[j] > −INF && INF > ub[j] {
  lim = NumericFocus 1: 1e4, 2: 100, ≥3: 0, 그 밖: 1e6
  t = ceil((lb[j] − r)/g − 1e-10)·g; if lim > t { r = (i64)(r + t) }   // r을 lb 쪽으로 옮긴다
}
3a061e(r, g, env, st, j)          // type 9 {j, r, g}; 실패하면 0x2711
53802c(tr, st, j); 5379f1(tr, j)
if lb[j] > −INF { nl = ceil((lb − r)/g − 1e-10); 3dd6f1(lb, ub, g·nl, ub, INF, st, j); lb[j] = nl }
if INF > ub[j]  { nu = floor((ub − r)/g + 1e-10); 3dd6f1(lb, ub, lb, g·nu, INF, st, j); ub[j] = nu }
if r != 0 {
  for node in col j (살아 있는 것만) { t = r·a_node; a_node = a_node·g; act_max[i] = act_max[i] + t; act_min[i] = act_min[i] + t; rhs[i] = rhs[i] − t }
  objcon = r·obj[j] + objcon; W = ((col_cnt[j]·6)·μ) + W
} else {
  for node in col j (죽은 노드 포함) { a_node = g·a_node }; W = ((col_cnt[j]·2)·μ) + W
}
obj[j] = g·obj[j]
if lb[j] > FeasibilityTol + ub[j] { +0x470 = j; return 3 }
done = 1; if lb[j] == ub[j] { return 3de18e(lb[j], INF, env, st, j, 1) }
return 0
```
- `3dd6f1`에 새 bound를 g·nl, g·nu로 넘긴다(노드 계수는 아직 옛 a). 그 뒤 r·a를 activity와 rhs 양쪽에 더하므로 `act − rhs`는 결국 y 기준 값이 된다(가드 없는 덧셈).
- `3c5996(a, m)`(131 B): 확장 Euclid 역원. m ≠ 0이면 (u, v) = (a, m), 계수 (s0, s1) = (1, 0)에서 `x = s1; g = v; q = u / v; s1 = s0 − q·x; s0 = x; (u, v) = (v, u % v)`를 v == 0까지. g == 0이면 0, 아니면 `(x + m) % m`.

### 3.4 오라클 확인
- 기본값 38개 42호출: 모두 아무것도 바꾸지 않는다(작업량도 그대로). 포트 42/42 일치.
- `574764` 경로는 합성 모델(세션 스크래치 `re/syn/m1–m4.lp`, 정수 행 `2 z1 + 4 z2 + x ≥ b`, x만 비용 singleton)로 확인: r ≠ 0(m1, x = 1 + 2y), r = 0(m3), lb 쪽으로 옮긴 r = 5(m4, objcon 15). 9호출 모두 일치(레코드 type 9 바이트 포함).
- NumericFocus ≥ 1의 lim, a < 0, 불가능 반환, y가 고정되는 `3de18e` 경로는 확인하지 않았다(해독만).

## 4. `59c193` VUB lifting (`005cf120`)

### 4.1 서명·가드·반환
`59c193(st, env, W) -> i32`(W = 작업량 포인터 `+0x458`). 가드 PREVUBLIFT(env+0x2c30, timer 13). 자체 가드: `n == st+0xf0`(모두 이진) 또는 `st+0xf0 == 0`이면 작업량 없이 0. 반환은 0, 할당 실패 0x2711, `3bea24`/`3ba9de`의 값. 포트 `vub_lift`(vublift.rs).

### 4.2 알고리즘
행마다 노드 두 개 `B[i]`(이진), `C[i]`(그 밖) 배열을 둔다(후보가 아니면 둘 다 NULL; 늘 함께 세우고 함께 지운다).
```
W0 = W; budget = nnz·1e4 + 1e6
// 1. VUB 후보: '<' 행, row_cnt == 2, inf_min == 0
for i in 0..m where 위 조건 {
  행 노드(살아 있는 것)를 이진이면 B[i], 아니면 C[i]에 넣으며 nb, nc를 센다
  W += (μ·4)·2
  a = C.val, x = C.col
  if nb == 1 && nc == 1 && !(0.001 > |a|) && !(|B.val/a| > 1e8) && |(|a|·(ub[x] − lb[x])) + act_min[i]| < 1e-10 { vubs += 1 }
  else { B[i] = C[i] = NULL }
}
W += (m·3)·μ
if vubs < 2 { return 0 }
// 2. 같은 x, 같은 이진 k의 VUB 둘 (n > 0일 때)
P−[k] = P+[k] = −1 (k = 열), Q−/Q+ = 행
for j in 0..n where col_cnt[j] >= 0 && vtype[j] != 'B' && !(1e-13 >= ub[j] − lb[j]) {
  for node in col j (살아 있고 C[i] != NULL, i = node.row) {
    a = C[i].val; k = B[i].col; (P, Q) = a < 0 ? (P−, Q−) : (P+, Q+)
    if !(j <= P[k]) { W += μ·3; Q[k] = i; P[k] = j; continue }
    i2 = Q[k]; b = B[i].val; b2 = B[i2].val
    if !(0 > b·b2) {                                         // 같은 부호: 하나를 지운다
      r2 = |b2 / C[i2].val|; W += μ·20
      drop = |b/a| > r2 ? (Q[k] = i, i2) : i
      415f4f(st, drop); B[drop] = C[drop] = NULL; gone += 1; continue
    }
    d = minsd(|b/a|, |b2/C[i2].val|)                          // 반대 부호: x의 bound를 d만큼
    if a < 0 { v = d + lb[j]; W += μ·20; if vtype[j] != 'C' { v = ceil(v − 1e-5) }; v = minsd(v, ub[j])
               if !(v > lb[j]) { continue }; 3dd6f1(lb, ub, v, ub); 5379f1(j); lb[j] = v }
    else     { v = ub[j] − d; W += μ·20; if vtype[j] != 'C' { v = floor(v + 1e-5) }; v = maxsd(v, lb[j])
               if !(ub[j] > v) { continue }; 3dd6f1(lb, ub, lb, v); 5379f1(j); ub[j] = v }
    for node in col j (살아 있고 B[row] != NULL) { gone += 1; B[row] = C[row] = NULL }
    +0x3f8 += 1; break                                        // 다음 열
  }
}
if vubs − gone < 2 { return 0 }
// 3. 클릭 같은 행: 모든 이진이 |a| = 최소(또는 '=' 행이면 최대) activity의 여유
for i in 0..m where row_cnt[i] >= 2 {
  for s in [1, −1] {
    slack = s == 1 ? (inf_min[i] == 0 ? act_min[i] : 건너뜀) : (sense == '=' && inf_max == 0 ? −act_max[i] : 건너뜀)
    행 노드(살아 있는 이진)마다 nb += 1; if |(|a|) + slack| > 1e-10 { 중단 }
    W += (μ·4)·row_cnt[i]
    if 중단 없음 && nb > 1 { sign[i] = s; cl += 1; break }
  }
}
W += (cl + m·5)·μ
if cl == 0 { return 0 }
// 4. 합치기: 두 번(pass 0: C.val < 0, pass 1: C.val > 0), 열마다
for pass in 0..2 { for j in 0..n {
  if W > W0 + budget { break }                                 // 이번 pass 끝
  if j % 1000 == 0 { 3ba9de(env, st, &stop) }                  // stop은 P0에서 서지 않는다
  if col_cnt[j] < 0 || vtype[j] == 'B' { continue }
  R = col j의 살아 있는 노드 중 C[row] != NULL이고 C.val이 pass 쪽인 행; W += (μ·5)·col_cnt[j]
  if |R| < 2 { continue }
  for i in R { b = B[i]; for node in col b.col (살아 있는 것) { r = node.row
      if 0 < node.val·b.val·sign[r] { if cnt[r] == 0 { met에 r 추가 }; cnt[r] += 1 } }
    W += (μ·7)·col_cnt[b.col] }
  kept = met 중 cnt ≥ 2(순서 유지, 나머지는 cnt = 0); W = W + μ·(|met| + |met|)
  ab72b6(kept, cnt): cnt 내림차순 간접 정렬; kept의 cnt = 0; W += (|kept| + |kept|)·μ
  for rq in kept {
    rq의 살아 있는 노드마다 mark[col] = sign[rq]·val > 0 ? 1 : −1; W = (μ·4)·row_cnt[rq] + W
    G = col j의 살아 있는 노드 중 C[row] != NULL, mark[B.col]·B.val > 0, C.val이 pass 쪽인 행; W = W + (μ·6)·col_cnt[j]
    rq의 mark를 0으로
    if |G| >= 2 {
      i0 = G[0]; s = 1/|C[i0].val|; B[i0].val *= s; C[i0].val *= s; rhs[i0] *= s
      for it in G[1..] { f = B[it].val / |C[it].val|; 3bea24(f, env, st, i0, B[it].col, NULL)
                         if 0 > f { rhs[i0] += f }; 415f4f(st, it); B[it] = C[it] = NULL }
      370047(st, env, 1, [i0]); 53763f(i0); 537da8(i0); B[i0] = C[i0] = NULL
      W += (|G|·15 + 10)·μ
    }
  }
} }
```
- `415f4f(st, i)`(69 B): 행 i의 살아 있는 노드를 죽이고(col = −1, 그 열 col_cnt −1), row_cnt = −2, +0x404 += 1. 작업량·표시 없음. 포트 `drop_row`.
- `3bea24(v, env, st, row, col, NULL)`: 새 노드를 행·열 리스트 앞에(probing 포트의 `new_node`).
- `ab72b6(n, key, idx, buf)`(1,672 B): int 키 `key[idx]` 내림차순 간접 정렬. n < 32면 바로 Shell 정렬, 아니면 키를 buf에 모아 같은 알고리즘(100 이상 3-way quicksort, 아래는 Shell n/2, ÷2.2). probing의 `sort_desc`(f64 키)와 같은 순열이라 포트는 키를 f64로 바꿔 그것을 쓴다(작은 정수라 정확).
- `cnt`/`mark`는 `+0x3a8`(크기 max(n, m) 정수 scratch)이고 쓸 때마다 0으로 되돌린다. 포트는 지역 벡터.

### 4.3 오라클 확인
- 기본값 38개 42호출 일치. 기본값에서는 아무것도 바꾸지 않고 작업량만 쌓는다. 지나는 단계: 1단계까지 32개(후보 < 2), 2단계 9개(exp-1-500-5-5, fhnw-binpack4-4, neos-2657525-crna, neos-3627168-kasai, neos-3754480-nidda, neos859080, ran14x18-disj-8, sp150x300d, tr12-30; 지운 행·bound 변화 0), 3단계에서 클릭 행이 있는 것 3개(exp-1-500-5-5 50, neos-2657525-crna 36, fhnw-binpack4-4 2), 4단계에서 |R| ≥ 2인 열은 없음.
- 합성 모델(세션 스크래치 `re/syn/v1–v7.lp`, `PrePasses=0 GURO_PAR_PREBOUND=0`이면 패스 없이 원래 모델이 라운드 끝에 온다): 같은 부호 지우기(v2, v6, a > 0·a < 0), 반대 부호 bound(v3 연속 a > 0, v5 정수 a < 0), 합치기(v1 pass 1, v4 정수 pass 0과 kept 2개 정렬, v7 '=' 행의 s = −1)까지 20호출 모두 일치.

## 5. `55a779` 반복 계수 패턴 (`005ab810`)

### 5.1 서명·가드·반환
`55a779(env, st, T20) -> i32`. 라운드 끝 round 0에서만(PreSparsify ≠ 0, m2_spine_c §2 F, timer 20). nnz(+0xd8) > 2e9면 0. 반환은 0 또는 할당 실패 0x2711. 포트 `agg_pass`(aggpass.rs).

### 5.2 1부: 패턴 사전
```
rows = 53a487(tr, st)                          // 레벨 0: 원시 목록
for r in rows where row_cnt[r] > 19 {
  class(j, a) = (a <= 0) + (vtype[j] == 'I' ? 2 : vtype[j] == 'C' ? 4 : 0)     // B+, B−, I+, I−, C+, C−
  살아 있고 |a| > 0.001인 노드마다 cnt[class]++ ; W += (μ·4)·방문(죽은 노드 포함)
  끝 위치 = 종류별 누적합. 같은 노드를 다시 돌며 min[k] = (|a| <= min[k]) ? |a| : min[k](초깃값 1e100),
  종류 k의 끝에서부터 앞으로 (열, a)를 채운다 ; W += (μ·5)·방문
  for k in 0..6 where 그 종류 개수 c > 19 {
    s = coefclasssign[k]·min[k]   (coefclasssign = [1, −1, 1, −1, 1, −1]); 값마다 a / s ; W = c·μ + W
    a9f78f(c, 열, 값): 열 오름차순(한 행 안의 열은 서로 달라 순서가 하나뿐)
    adfbe7(표, −1): 해시 55866f가 같고 55880a(같은 열, 값 차 ≤ 1e-10)가 참인 저장 패턴(새것부터)
    없으면: 패턴 저장, W = ((c + 5) + c)·μ + W, adfdc3로 표에 넣는다(id = 순번)
    있으면: dups += 1
    출현 기록(패턴 id, s, 행별 출현 수) — 2부에서만 쓴다
  }
}
W = (|rows| + |rows|)·μ + W
if dups > 0 && !adf2be(T20) { 2부 }
```
- `55866f`(해시): `h = (h·31 + 열)·31 + (int)(i64)clamp(floor(값·1e6 + 0.5), ±4.611686018427388e18)`(i32 wrap). 표 `adf891`은 체인 해시표이고 찾기는 저장된 해시가 같은 것만 `55880a`로 비교한다. 그래서 포트는 해시 → id 목록(새것부터 비교)으로 같다. 해시·비교·정렬·표에는 작업량이 없다.
- 2부(`dups > 0`): 반복 패턴마다 새 열(`3bbf87`)과 정의 행(`3bde19`)을 만들고 출현한 행의 그 부분을 새 열 하나로 바꾼다(`3bea24`, `36e6b6`, `370047`, `36c5cf`). **해독·포팅하지 않았다(`shortcut:`).** 기본값 38개에서 `55880a`가 한 번도 불리지 않았다(Frida 호출 수, 세션 스크래치 `re/cov_594b.json`): 같은 해시의 패턴이 없어 dups = 0이다.

### 5.3 오라클 확인
38개 38호출 일치(작업량만 바뀜). 패턴이 만들어지는 모델(해시 호출 > 0) 25개에서 나누기·저장 작업량까지 맞았다(예: cod105 패턴 1024개, neos-860300 519개).
