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
