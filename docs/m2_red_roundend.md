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
