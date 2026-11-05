# M2 MIP presolve 바깥 패스 A묶음 해독 (`3e5363`, `3ff744` MIP, `57627e`, `578fd2`, `374521`, `5431ee`)

`3cd406` 바깥 패스 앞쪽에서 이 순서로 불리는 reduction의 해독 노트(호출 위치·가드·타이머·PRELIMIT 카운트는 [m2_spine_b.md](m2_spine_b.md) §2–3). 표기와 공통 규칙은 [m2_red_inner.md](m2_red_inner.md) §1과 같다.
- `W += x·μ`는 `*(st+0x458) += x * *(st+0x448)`. 곱의 순서는 원본 그대로(`(n·6)·μ`와 `(μ·3)·n`은 다르게 적는다).
- 노드 = 32 B {f64 값, i32 행 @+8, i32 열 @+0xc(−1 죽음), 행 다음 @+0x10, 열 다음 @+0x18}. "방문"은 죽은 노드를 포함한 리스트 노드 수.
- 파일 오프셋 = Ghidra 주소 − 0x100000. 주소는 파일 오프셋.
- "확인"은 dump 진입 상태를 Python 모형(세션 스크래치 `outer/bin/`)에 넣어 돌린 결과가 복귀 상태와 모든 필드(배열 비트, 행·열 리스트 순서, 개수, 카운터, 작업량, 추적기 플래그·목록·캐시, 새 레코드)에서 같았다는 뜻이다(§9).

## 0. 요약

| 함수 | 크기 | 3cd406 가드 | 하는 일 | 기본값 38개에서 상태를 바꾼 호출 | 오라클 |
|---|---|---|---|---|---|
| `3e5363(env, st)` | 5,451 | PREXORRED && PreCrush==0 | XOR(패리티) 행 묶음 제거, type 25 | 0 / 78 | 78/78(§2) |
| `5431ee(st, env)` | 3,175 | round==0 && k==0 && PRETRANSRED | 이진 함의 그래프 SCC | 1 / 42(fhnw-binpack4-4) | (§7) |
| `578fd2(st, env)` | 1,281 | PRECOEFFICIENT | `<` 행 knapsack 계수 lifting(`6aec61`) | 11 / 78 | (§5) |
| `57627e(st, env)` | 10,175 | PRECOEFFICIENT | implied integer, type 9 | 4 / 78 | (§4) |
| `374521(st, env)` | 12,783 | DualReductions && p<3 && T4 | 쌍대 bound, 부등식→등식 | 2 / 78 | (§6) |
| `3ff744(st, env, T6, mode, flag, 0)` | 16,559 | (spine-b) | 평행 행·열 | (§3) | (§3) |

"상태를 바꾼"은 작업량 말고 다른 필드(bound, 타입, 행렬, 카운터, 레코드)가 달라진 호출이다. PRELIMIT sweep의 변화점이 이 함수들의 카운트에 떨어져도(예: timtab1 k=25 = `3e5363`) 그 호출이 아무것도 안 바꿀 수 있다: k−1(내부 루프 끝, 추적기 레벨 2)과 k(레벨 1로 reset 뒤)의 마무리가 `538353`/`5383d0`로 다른 목록을 반올림하기 때문이다(m2_spine_b §8-1). 그래서 판정은 상태 dump로 한다.

## 1. 공통

- 도구(세션 스크래치 `outer/bin/`, 저장소에 넣지 않음): `dumpo.js <모델|.lp> <fn,fn,…> <out.jsonl>`는 `tools/mip_state_dump.js`에 여러 함수 동시 훅, 모든 호출, 추적기(레벨별 a/b/목록, `538f30`/`5399fc` 캐시, `53a487`/`53af0f` 캐시 +0xe0/+0xf0·+0x108/+0x118), 열 플래그 +0x170, mlo/mhi(+0x80/+0x88), 상태 머리 0..0x4a0을 i32로, 첫 메시지에 env 0x2400..0x2d00, 인자(`3ff744`는 타이머 내용)를 더했다. `runall.sh`(38개 모델 2분), `simo.py`(inner의 `sim.py` St 확장, 레코드 디코더, 비교), `chg.py fn`(작업량 말고 바뀐 필드가 있는 호출 목록), `s_<함수>.py`(모형).
- 레코드 머리는 inner와 같다: {i32 type @+0, data 포인터 @+8(= rec+0x18), next @+0x10}, 목록 머리 +0x370(새것이 앞). 아래 "data+x"는 data 포인터 기준.
- `3de18e`(열 고정, type 1), `3dd6f1`, `370047`, `53802c`, `537da8`, `5379f1`/`53763f`는 inner 노트 §1 그대로다.

## 2. `3e5363(env, st)` XOR·패리티 행 (5,451 B, `004c4ec0`)

### 2.1 가드·반환
- `st+0xf0`(이진 열 수) < 3이거나 DualReductions(env+0x2430) == 0이면 바로 0(작업량 없음).
- 반환: 마지막 `3de18e` 반환값(처음 0). `39ba81` 할당 실패면 0x2711.

### 2.2 알고리즘

```
(cnt, list) = 538353(tr)                    // 현재 레벨(1)의 원시 행 a-목록, 작업량 없음
for t in 0..cnt:
  i = list[t]; rc = row_cnt[i]
  if (rc != 3 && rc != 4) || sense[i] == '=': continue          // 작업량 없음
  if |rhs[i] − 2| <= 1e-13: e8 = 0  elif |rhs[i] + 1| <= 1e-13: e8 = 1  else continue
  // 씨앗 행 훑기
  nb = −1; best = 2e9; piv = −1; npos = 0; cols = []; vis = 0
  for nd in row i:                                   // 행 리스트 순서
    if nd.col >= 0:
      c = nd.col
      if vtype[c] != 'B':
        if |lb[c]| > 1e-13 || |ub[c] − 1| > 1e-13 || nb >= 0: ABORT   // 비이진은 [0,1] 연속 하나만
        nb = c
      a = nd.val
      if (e8 == 0 && ||a| − 1| > 1e-13) || (e8 == 1 && |a + 1| > 1e-13): ABORT
      cols.push(c); npos += (0 < a)
      if !(0.5 > ub[c] − lb[c]):                     // 고정 안 된 열만 pivot 후보
        if nb == c { best = col_cnt[c]; piv = c }    // 비이진 열이 있으면 그것이 pivot
        else { k = col_cnt[c]
               if k < best { if nb == −1 { piv = c; best = k } }
               elif k == best && obj[c] == 0.0 && nb == −1 { piv = c } }   // 같으면 뒤의 비용 0 열
    vis += 1
  W += (vis·6)·μ                                     // ABORT면 vis는 그 노드 앞까지. 그다음 continue
  if piv < 0 || best < 2 || obj[piv] != 0.0 || colflag[piv] != 0: continue
  if rc == 4 && e8 == 0 && npos != 3: continue
  // pivot 열 훑기: 같은 패리티의 짝 행 모으기, 나머지 행에서 pivot 부호가 한쪽인지
  rows = [i]; sgn = 0; vis = 0
  for pn in col piv (열 리스트 순서):
    if pn.col >= 0 && pn.row != i:
      r = pn.row
      if sense[r] == '=': ABORT2
      cand = rc == row_cnt[r] && rhs[r]가 (e8,rc)=(0,3): 0 / (0,4): 0 또는 2 / (1,3): 1 / (1,4): 1 또는 3 (|·| <= 1e-13)
      if cand:
        bad = false; np = 0; v2 = 0; ps = (pn.val < 0 ? −1 : 1)
        for nd in row r:
          if nd.col >= 0:
            if nd.col != piv && nd.col ∉ {cols[0], cols[1], cols[2], (rc != 3이면) cols[3]}: bad = true
            if ||nd.val| − 1| > 1e-13: bad = true
            if nd.val > 0: np += 1
          v2 += 1
        W += (μ·3)·v2
        fit = !bad && (e8,rc)=(0,3): np==1 / (0,4): rhs 0이면 np==1, 2면 np==3 / (1,3): np==2 / (1,4): rhs 1이면 np==2, 3이면 np==4
        if fit { rows.push(r); vis += 1; next }
        s = ps                                       // 맞지 않는 후보는 "다른 행"으로 친다
      else: s = (pn.val < 0 ? −1 : 1)
      if sgn == 0: sgn = s
      elif sgn != s:                                 // 정확히: sgn==−1인데 a>0, sgn==1인데 a<0이면
        ABORT2                                       // (다른 행 경로는 a==0을 +쪽으로 본다)
    vis += 1
  W += (vis·7)·μ                                     // ABORT2면 그 노드는 빼고. 그다음 continue
  cols에서 piv를 맨 앞으로: piv==cols[1]이면 0↔1, 아니면 cols[2]면 0↔2, 아니면 (row_cnt[i]==4이고) cols[3]이면 0↔3
  if sgn != 0:                                       // pivot이 다른 행에 한 부호로만 있다
    keep = []
    for r in rows:
      v = 0; for nd in row r { if nd.col >= 0 && nd.col == piv { if 0 < nd.val·(f64)sgn { keep.push(r) }; break }; v += 1 }
      W += v·μ
    W += (len(rows)·3)·μ
    if keep: 39ba81(env, st, e8, cols, keep, (piv+1)·sgn); DELROWS(keep)
    else:    W += (0·3)·μ
    for c in cols: if col_cnt[c] >= 0 { 5379f1(c) }      // pivot 포함
    W += len(cols)·μ
  else:                                              // pivot이 XOR 행에만 있다: pivot 소거
    39ba81(env, st, e8, cols, rows, 0); DELROWS(rows)
    for c in cols[1], cols[2], (row_cnt[i]==4이면 cols[3]):
      if col_cnt[c] == 0 && colflag[c] & 0xfffff7bf == 0:
        rc' = 3de18e(v = (obj[c] >= 0 ? 0.0 : 1.0), INF, env, st, c, rec=1); if rc' != 0 return rc'
    col_cnt[piv] = −3; st+0x408 += 1
    t = 1; while t < len(cols) { if col_cnt[cols[t]] >= 0 { 5379f1(cols[t]) }; t += 1 }
    W += t·μ
W += (cnt·3)·μ
```
`DELROWS(R)`: R의 행마다 행 리스트를 돌며 살아 있는 노드의 `col_cnt[col] −= 1`, `col = −1`, 끝에 `row_cnt = −1`, `W += (v+v)·μ`(v = 방문). 다 지운 뒤 `W += (len(R)·3)·μ`. 행 표시·`+0x404`는 여기서 하지 않는다(`39ba81`이 `+0x404 += len(R)`).

메모(asm 0x3c5bbc 확인):
- `cols[3]` 고정 판정은 씨앗 행 i의 `row_cnt`를 **지운 뒤** 다시 읽는다(−1). 그래서 4열 XOR에서 넷째 열은 빈 열이 되어도 여기서 고정되지 않는다(원본 그대로 옮긴다).
- `3de18e` 인자: xmm0 = 고정값, xmm1 = INFBOUND(env+0x2458), rdi = env, rsi = st, edx = 열, ecx = 1. 고정값 비교는 `comisd obj, 0` → `obj >= 0`이면 0.0(NaN이면 1.0).
- pivot 판정의 `ub − lb`는 `!(0.5 > d)`(NaN이면 후보).

### 2.3 `39ba81(env, st, e8, cols, ncols, rows, nrows, code)` type 25 레코드 (929 B)
- `W += (nrows + nrows)·μ`(행 개수 합을 세는 루프. 합 nnz0 = Σ row_cnt는 할당 크기에만 쓴다), `+0x404 += nrows`, `W += ((ncols + nrows)·0.5)·μ`, 항목을 쓰고 `W += μ·(nrows·5 + nnz·3)`(nnz = 쓴 살아 있는 항목 수).
- 크기 = P_c + P_r + P_b + 8·nrows + P_i + 8·nnz0 + 0x80. P_c = (4·ncols−1)&~7, P_r = (4·nrows−1)&~7, P_b = (4·nrows+3)&~7, P_i = (4·nnz0−1)&~7.

| data 오프셋 | 내용 |
|---|---|
| +0x00 | ptr cols = data+0x48 |
| +0x08 | i32 ncols |
| +0x10 | ptr rows = data+0x50+P_c |
| +0x18 / +0x1c / +0x20 | i32 nrows / e8 / code((piv+1)·sgn, pivot 소거면 0) |
| +0x28 / +0x30 / +0x38 / +0x40 | ptr beg(nrows+1) / ind / val / rhs |
| +0x48 | i32 cols[ncols](pivot이 맨 앞) |
| 이어서 | i32 rows[nrows], i32 beg[nrows+1](행 시작 위치, 끝 = nnz), i32 ind[nnz], f64 val[nnz], f64 rhs[nrows] |

항목은 행 리스트 순서의 살아 있는 노드(열, 값), rhs는 그 시점의 rhs(D 규약). 뜻: XOR 행 묶음 전체(또는 pivot 부호가 맞는 행만)를 지우고 postsolve가 pivot을 패리티로 되살린다.

### 2.4 오라클 확인
- 38개 모델 78호출 전부 일치(`s_xor.py`). 씨앗 행 480개가 모두 pivot 열 단계에서 멈췄다(짝이 아닌 행에서 pivot 부호가 엇갈리거나 `=` 행). 씨앗 훑기 중단 22. 그래서 **기본값 38개에서 `3e5363`은 작업량만 바꾼다.**
- 레코드·삭제·고정 경로(`39ba81`, DELROWS, `3de18e`, 부호 경로)는 오라클을 지나지 않았다 → 합성 모델로 확인할 것(§9).
