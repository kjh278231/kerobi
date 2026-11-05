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

## 6. `374521(st, env)` 쌍대 bound (12,783 B, `00479450`)

LP `505077`+`4f84ef`(`crates/lp/src/presolve/reduce.rs` `dual_bound_rows`/`dual_propagate`)를 한 함수에 인라인한 MIP판이다. 흐름은 같고 아래가 다르다: 연결 리스트·μ 작업량, **연속이 아닌 열도 특수 열로 친다**, 열 쌍대 범위를 0에서 더한 뒤 −c를 마지막에 더한다, 행 쌍대를 옮길 때 차이를 한 번에 더한다, 문턱 1000·ft, 고정 판정에 |c|·1e-13 항과 비유계(5)가 없다.

### 6.1 서명·반환
반환 0, 3(불가능), 0x2711(할당), 또는 마지막 `3ba5ac`/`3ba9de` 값. 가드는 3cd406 쪽(DualReductions && p<3 && T4 0.8 판정). INF = env+0x2458, ft = env+0x2460, ot = env+0x2470(OptimalityTol). scratch: +0x168(열 상태 바이트), +0x1c0(정렬 순서), +0x1d0(계수 정렬 버킷, m+2), +0x398/+0x3a0(행 쌍대 ylo/yhi).

### 6.2 알고리즘

```
// 0. 특수 열 (st+0xfc != 0, MIP)
special(j) = vtype[j] != 'C' || colflag[j] != 0
j0 = 첫 !special 열(없으면 n); W += (j0+j0)·μ
if j0 == n: return 0                                    // 연속 보통 열이 없으면 끝 (38개 중 11호출)
cnt[j] = col_cnt[j] − (m+1) (special이면) — 원본은 col_cnt를 직접 깎았다가 0.4에서 되돌린다
W += (μ·3)·n
imax = inf_max 복사, imin = inf_min 복사(m개, 작업량 없음)
for j: if 0 < cnt[j]: st[j] = lb>−INF ? (ub<INF ? 0 : 2) : (ub<INF ? 1 : 3)   // bit0 lb 자유, bit1 ub 자유. 아니면 scratch 그대로
W += (n·4)·μ
// 1. 함의된 bound를 자유로 (st+0x100 == 0일 때. 기본값 참)
if st+0x100 == 0:
  안정 계수 정렬: 0 < cnt[j]인 열을 cnt 오름차순(같으면 번호순). W += (n+n)·μ, W += m·μ, W += (μ·3)·n
  for upper in [true, false]:                           // ub 패스 먼저, 그다음 lb 패스
    for j in 정렬 순서:
      if upper && st[j] > 1 || !upper && st[j] & 0xfd == 1: continue      // 작업량 없음
      far = upper ? lb[j] <= −INF : !(ub[j] < INF);  k = far ? 1 : 0
      v = 0; 열 리스트에서 살아 있는 노드 (a, i)마다 (찾으면 그 노드는 v에 안 셈):
        upper: a <= 0 → sense[i]=='=' && imax[i]==k && (far ? −act_max[i]/a : lb − act_max[i]/a) < ub + 1e-9
               a > 0  → imin[i]==k && (far ? −act_min[i]/a : lb − act_min[i]/a) < ub + 1e-9
        lower: 0 <= a → sense[i]=='=' && imax[i]==k && lb − 1e-9 < (far ? −act_max[i]/a : ub − act_max[i]/a)
               a < 0  → imin[i]==k && lb − 1e-9 < (far ? −act_min[i]/a : ub − act_min[i]/a)
      W += (μ·8)·v
      찾으면: st[j] = (st[j]==0 ? (upper ? 2 : 1) : 3)
              열 리스트의 살아 있는 노드마다 ((a <= 0) == upper ? imin : imax)[i] += 1; W += (μ·5)·방문
    W += (μ·4)·len(정렬)                               // 패스마다
  3ba5ac(env, st, 0, 0, −1), 3ba9de(env, st, 0)        // 패스마다 끝에(반환값 ≠ 0이면 그 값으로 끝)
// 2. 열의 쌍대 부호
sense_c[j] = cnt[j] < 1 ? 'F' : {3:'=', 1:'>', 2:'<'}[st[j]] (그 밖 'F');  W += (n+n)·μ
// 3. 행 쌍대 상자
for i: row_cnt[i] < 1 ? (ylo, yhi) = (0, 0) : (−INF, sense[i]=='=' ? INF : 0);  W += (m·4)·μ
special 열 개수를 되돌림; W += (μ·3)·n
// 4. 열마다 −c + Σ a·y의 범위 (dmax/dmin 유한 부분, cmax/cmin 무한 개수)
for j in 0..n (모든 열):
  dmax = dmin = 0; cmax = cmin = 0; 열 리스트의 살아 있는 노드 (a, i):
    a <= 0: ylo>−INF ? dmax = gup(dmax, ylo·a) : cmax++;  yhi<INF ? dmin = gdn(dmin, a·yhi) : cmin++
    a > 0:  yhi<INF ? dmax = gup(dmax, yhi·a) : cmax++;   ylo>−INF ? dmin = gdn(dmin, a·ylo) : cmin++
  dmax = gup(dmax, −c[j]); dmin = gdn(dmin, −c[j])     // LP는 −c에서 시작
  W += (μ·8)·방문
W += (n·6)·μ
// 5. 원형 큐 전파 (LP 4f84ef 구조)
queue = col_cnt >= 0인 열(번호순), inq 전부 1; W += (n·3)·μ
if len(queue) == 0: 끝(아래 6으로)
if len(queue) >= 2:
  head = 0; tail = len−1; rounds = 0; pops = 0
  loop:
    j = queue[head]; s = sense_c[j]
    if s ∈ {'<','='} && cmin[j]==0 && ft < dmin[j]:  st+0x478 = j, return 3
    if s ∈ {'=','>'} && cmax[j]==0 && dmax[j] < −ft:   st+0x478 = j, return 3     // (디컴파일 읽기, 오라클 미경유)
    열 리스트의 살아 있는 노드 (a, i)마다:            // LP와 같은 nl/nh/tl/th 계산(±1e-8, k 0/1, d0 = min(d, 0))
      (nl, th)를 LP와 같은 식으로 구한다
      if −1e8 < nl && ft·1000 + ylo[i] < nl:
        if !(nl <= yhi[i] + ft): st+0x470 = i, return 3
        if −INF < ylo[i]:  행 i의 살아 있는 노드 (b, k)마다 t = (nl − ylo[i])·b; b<=0 ? dmax[k] = gup(dmax[k], t) : dmin[k] = gdn(dmin[k], t); PUSH(k);  W += (방문·5)·μ
        else:              b<=0 ? (cmax[k]−−, dmax[k] = b·nl + dmax[k]) : (cmin[k]−−, dmin[k] = b·nl + dmin[k]) (가드 없음); PUSH(k);  W += (방문·6)·μ
        ylo[i] = nl
      if th < 1e8 && th < yhi[i] − ft·1000:
        if th < ylo[i] − ft: st+0x470 = i, return 3     // ylo는 위에서 바뀐 값
        if yhi[i] < INF:   t = (yhi[i] − th)·b; b<=0 ? dmin[k] = gdn(dmin[k], −t) : dmax[k] = gup(dmax[k], −t); PUSH(k);  W += (방문·5)·μ
        else:              b<=0 ? (cmin[k]−−, dmin[k] = b·th + dmin[k]) : (cmax[k]−−, dmax[k] = b·th + dmax[k]); PUSH(k);  W += (방문·6)·μ
        yhi[i] = th
    W += (방문·10)·μ
    head += 1; inq[j] = 0
    if head == n: rounds += 1; head = 0; if rounds > 10: break
    pops += 1
    if head == tail: break
W += (pops·7)·μ                                         // 큐가 1개면 0
PUSH(k): inq[k]==0이면 tail = (tail+1 == n ? 0 : tail+1), inq[k] = 1, queue[tail] = k
// 6. 고정 (Presolve != 2)
3ba5ac, 3ba9de
for j in 0..n: if col_cnt[j] > 0 && 1e-10 < ub − lb && colflag[j] == 0:     // 정수 열도 된다
  if cmax[j] != 0 || −ot <= dmax[j] || lb <= −INF:
    if cmin[j] == 0 && ot < dmin[j] && ub < INF: 3de18e(ub, INF, env, st, j, 1)
  else: 3de18e(lb, INF, env, st, j, 1)
  (반환값 ≠ 0이면 그 값으로 끝)
W += (n·7)·μ
for i: if row_cnt[i] > 0 && (ft < ylo[i] || yhi[i] < −ft) && sense[i] != '=': st+0x400 += 1; sense[i] = '='; 53763f(i)
W += (m·3)·μ
return 3ba5ac(env, st, 0, 0, −1)
```
- nl/th 식(LP와 같음, s = sense_c[j]): nl = ylo, nh = yhi에서 시작. s ∈ {'<','='}이면 (dm, k) = (dmin, cmin): a < −1e-8이면 k==0: nl = yhi − min(dm,0)/a, k==1 && INF <= yhi: nl = −dm/a. a > 1e-8이면 k==0: nh = ylo − min(dm,0)/a, k==1 && ylo <= −INF: nh = −dm/a. th = nh. s ∈ {'=','>'}이면 a2 = −a, dx = dmax, (tl, th) = (ylo, yhi)에서 같은 식(min(−dx, 0), k = cmax, k==1이면 dx/a2), 그다음 tl = max(tl, nl) → nl, th = min(th, nh).
- Presolve == 2 분기(디컴파일만): 'F' 열은 cmax==0 && dmax < 1e-10 && lb > −INF면 lb, 아니면 cmin==0 && dmin > −1e-10 && ub < INF면 ub. 다른 열은 위와 같다. 끝 작업량 (μ·8)·n.
- 불가능 인덱스: 디컴파일상 열은 +0x478, 행은 +0x470에 쓴다(LP 포트와 반대). 오라클로 확인하지 않았다.

### 6.3 오라클 확인
- 38개 78호출 전부 일치(`s_dual.py`): 연속 열이 없어 바로 끝난 호출 11, 1단계 67. 자유로 푼 bound ub 5,955·lb 187, ylo 올림 551(유한에서 100, 무한에서 451), yhi 내림 106(71/35), `=`로 바꾼 행 71(neos-4338804-snowy 21, csched007 50).
- 고정(`3de18e`) 경로, 불가능 반환, 10바퀴 제한, Presolve 2 분기는 오라클을 지나지 않았다.
