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
| `5431ee(st, env)` | 3,175 | round==0 && k==0 && PRETRANSRED | 이진 함의 그래프 SCC | 1 / 42(fhnw-binpack4-4) | 42/42(§7) |
| `578fd2(st, env)` | 1,281 | PRECOEFFICIENT | `<` 행 knapsack 계수 lifting(`6aec61`) | 11 / 78 | 78/78(§5.3) |
| `57627e(st, env)` | 10,175 | PRECOEFFICIENT | implied integer, type 9 | 4 / 78 | 78/78, `575405`는 bound를 바꿀 수 있으나 미포팅(§4) |
| `374521(st, env)` | 12,783 | DualReductions && p<3 && T4 | 쌍대 bound, 부등식→등식 | 2 / 78 | 78/78(§6) |
| `3ff744(st, env, T6, mode, flag, 0)` | 16,559 | (spine-b) | 평행 행·열 | 16 / 234 | 237/237(열 집계, mode 2 포함, §3) |

포팅 위치(crates/mip/src): `xorrows.rs`, `transred.rs`, `knapsack.rs`, `iint.rs`, `dualbnd.rs`, `parallel.rs`, 행·열 추가 `grow.rs`(`3bbf87`/`3bde19`, `Queue::grow` = `535d61`/`536369`). 확인: `tools/outer_dump.sh <dir>`로 dump를 만들고 `tools/mip_dump_check.py <fn> <dir>/*_<fn>.json`.

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
- 불가능 인덱스: 열은 +0x478, 행은 +0x470에 쓴다(LP 포트와 반대). asm으로 확인: 0x37bd0e에서 queue[head]의 열(r9)을 +0x478에, 0x37bb46에서 node.row(eax)를 +0x470에 쓴다.

### 6.3 오라클 확인
- 38개 78호출 전부 일치(`s_dual.py`): 연속 열이 없어 바로 끝난 호출 11, 1단계 67. 자유로 푼 bound ub 5,955·lb 187, ylo 올림 551(유한에서 100, 무한에서 451), yhi 내림 106(71/35), `=`로 바꾼 행 71(neos-4338804-snowy 21, csched007 50).
- 고정(`3de18e`) 경로, 불가능 반환, 10바퀴 제한, Presolve 2 분기는 오라클을 지나지 않았다.

## 7. `5431ee(st, env)` 이진 함의 그래프 (3,175 B, `00599080`)

### 7.1 흐름
리터럴 2n개: k < n은 x_k, k ≥ n은 ¬x_{k−n}. 보조 함수 반환값 ≠ 0이면 그 값으로 끝(0x2711 할당).
```
if 2e9 < (f64)n + (f64)n: return 0
p = PRETRANSRED(env+0x2574); p == −1이면 1
(deg, beg, adj, rowof, ne, flag) = 53dce0(env, st, mode=1, W)
if ne != 0:
  changed = 53fa62(env, st, 1, beg, deg, adj, W)        // SCC
  if changed: 그래프를 53dce0(mode 1)로 다시 만든다(ne == 0이면 끝)
  (order, res) = 5416c3(st, n, beg, deg, adj, W)        // 위상 정렬
  if res >= 0: 541d96(env, st, res, beg, deg, adj, rowof, order, W)   // 이행 축소
if p < 2 || n <= st+0xf0(이진 수) || flag == 0: return 0  // 기본값(p=1)은 여기서 끝
(p >= 2: mode 2로 같은 일을 한 번 더 하고 deg 합 작업량을 더한다. 해독 안 함)
```

### 7.2 `53dce0(env, st, mode, deg, beg, &adj, &rowof, &ne, &flag, W)` 그래프 (2,723 B)
```
deg[0..2n] = 0; lit1[i] = lit2[i] = −1; ne = 0; flag = 0
for i in 0..m:
  if row_cnt[i] != 2 || sense[i] != '<': continue                       // 작업량 없음
  b = rhs[i]
  if !(1e-10 >= |b|) && (mode != 1 || (n <= st+0xf4 && !(1e-10 >= ||b| − 1|))): continue
  c1 = c2 = −1; a1 = a2 = 0; vis = 0; 첫 살아 있는 노드로(죽은 노드마다 vis += 1)
  노드마다:
    unit = ||a| − 1| <= 1e-10
    d = ub[c] − lb[c]; if 1e-10 >= d: break                              // 고정 열: 짝 미완성
    mode 1: vtype[c]=='C'면 lb <= −1e30 || ub >= 1e30 || d > 1e4이면 FAIL, 아니면 b = b − lb[c]·a, scale = d
            아니면 |lb| > 1e-10 || |ub − 1| > 1e-10이면 FAIL, 아니면 vtype[c] = 'B', lb[c] = 0.0, ub[c] = 1.0, scale = 1
    mode 2: !unit이면 FAIL; scale = 1
    if c1 != −1: a2 = a·scale, c2 = c, break                            // 둘째 열은 vis에 안 셈
    a1 = a·scale, c1 = c; 다음 살아 있는 노드로(지난 노드마다 vis += 1)
  W += (μ·4)·vis
  ok = c2 >= 0
  mode 1이고 ok: s = 1/|a1|; a2·=s, a1·=s, b·=s; ||a2|−1| > 1e-10이면 !ok; |b| > 1e-10이고 ||b|−1| > 1e-10이면 !ok
  if !ok: (unit && |rhs[i]| <= 1e-10)이면 flag = 1; continue           // unit = 마지막으로 본 노드
  if a1 < 0 && a2 > 0: (c1,a1) ↔ (c2,a2)
  |b| <= 1e-10:  a1 > 0 && a2 < 0 → (lit1, lit2) = (c1, c2)             // x1 ≤ x2: x1 → x2
  b > 0.5:       a1 > 0 && a2 > 0 → (c1, ¬c2)                            // x1 + x2 ≤ 1
  그 밖:         a1 < 0 && a2 < 0 → (¬c1, c2)                            // x1 + x2 ≥ 1
  (¬k = (k + n) % 2n. |b|, b는 mode 1에서 배율을 곱한 값)
  if lit1[i] >= 0: ne += 2; deg[lit1]++; deg[¬lit2]++
W += (m·5 + ne)·μ
if ne == 0: return
beg = 앞 누적합(beg[0] = 0, beg[k] = beg[k−1] + deg[k−1], k < 2n); W += (2n + 2n)·μ
행 순서로 lit1[i] = u >= 0이면 adj[beg[u]++] = lit2[i], rowof = i; adj[beg[¬lit2]++] = ¬u, rowof = i
W += (ne·4 + m)·μ; beg를 다시 누적합; W += (2n + 2n)·μ
```
- 간선 u→v와 그 대우 ¬v→¬u를 둘 다 넣으므로 deg는 나가는 차수이고 들어오는 차수(v) = deg[¬v]다.
- mode 1은 [0,1] 정수 열을 'B'로, bound를 정확히 0/1로 쓴다(상태를 바꾼다. 38개에서 차이 없음).

### 7.3 `53fa62(env, st, mode, beg, deg, adj, &changed, W)` Tarjan SCC (3,547 B)
```
W += (n + n)·μ; idx = low = −1, onstack = 0
for r in 0..2n: if idx[r] < 0:
  idx[r] = low[r] = 0; counter = 1; 스택 [r], DFS [r](간선 위치 0)
  loop: u = DFS 꼭대기, e = 그 간선 위치, ed = 0
    while e < deg[u]: v = adj[beg[u] + e]
      idx[v] < 0: 위치 e+1 저장, idx[v] = low[v] = counter++, 스택·DFS에 v, W += 10·μ, W += (μ·4)·ed, v로 내려감
      onstack[v]: low[u] = min(low[u], idx[v]); W += 2·μ;  e++, ed++
      그 밖: e++, ed++
    W += (μ·4)·ed
    low[u] == idx[u]: 스택에서 u까지 꺼낸다. u가 아닌 원소 w마다 MEMBER(u, w); W += (μ·5)·(w 수)
    DFS pop; 비면 다음 r. 부모 p: low[p] = min(low[p], low[u]); W += 4·μ
W += (μ·7)·2n
```
index는 뿌리마다 0부터 다시 센다(원본 그대로). MEMBER(u, w): 같은 값이 되는 리터럴. 변수 xu, xw 중 없앨 쪽 elim을 고른다(타입이 같으면 xw, 단 colflag[xw] ≠ 0이면 xu; xw가 'C'면 xw; xw가 'I'이고 xu가 'C'면 xu; …). colflag[elim] == 0이고 두 col_cnt ≥ 0이면 `3bb6fe(±d_elim/d_keep, …)`(집계, 2,389 B, 해독 안 함) 또는 x ≡ ¬x이면 정수는 불가능(3)·연속은 `3de18e`, changed = 1. **38개에서 크기 2 이상 SCC가 없어 이 경로는 오라클을 지나지 않았다.**

### 7.4 `5416c3(st, n, beg, deg, adj, order, lvl(0), cnt, &res, W)` 위상 정렬 (1,248 B)
```
order = [k for k in 0..2n if deg[¬k] == 0 && deg[k] > 0]           // 들어오는 간선 없는 시작점
W += ((2n + 2n) + len(order))·μ
cnt = deg 복사
t = 0; while t < len(order):                                       // BFS, 목록이 자란다
  u = order[t]; 간선 u→v마다: lvl[v] = max(lvl[v], lvl[u] + 1); cnt[¬v] −= 1; 0이 되면 order에 v
  W += (5·μ)·deg[u]; t += 1
W += μ·(q + q)                                                     // q = len(order)
res = q; 간선 k→v 중 lvl[v] <= lvl[k]가 하나라도 있으면 res = −1     // 순환
W += Σ_k deg[k]·(4·μ)  — 벡터화된 합: 8칸(L0은 W에서, 나머지 0에서 시작) → ((L0+L2)+(L4+L6)) + ((L1+L3)+(L5+L7)), 나머지(2n mod 8, 짝수)는 2칸 → P0+P1
```

### 7.5 `541d96(env, st, q, beg, deg, adj, rowof, order, W)` 이행 축소 (2,038 B)
```
W0 = W(진입 값); dist = −1, done = 0; W += (n + n)·μ; removed = 0
for idx = q−1 down to 0:
  if W > ((removed·1000) + (nnz·100 + 1e7)) + W0: return 0     // nnz = st+0xd8
  u = order[idx]; if done[u]: continue
  dist[u] = 2n; vis = []
  CHAIN:
    vis.push(u); 스택 [u]; pops = 0
    스택에서 w를 꺼내 간선 w→v마다: dist[v] == −1이면 v를 스택·vis에; dist[v] = max(dist[v], dist[w] + 1); W += (4·μ)·deg[w]; pops += 1
    W += (pops + pops)·μ
    for k in 0..deg[u]: v = adj[beg[u]+k]; if dist[u] + 1 < dist[v] && row_cnt[rowof] >= 0: 415f4f(st, rowof), removed++
    W += (μ·3)·deg[u]
    done[u] = 1
    ¬u의 간선을 차례로 보아 done[¬v] == 0인 첫 ¬v를 x로(못 찾으면 −1); W += (3·μ)·k(k = 찾은 위치 또는 deg[¬u])
    x != −1이면 dist[x] = dist[u] − 1, u = x, CHAIN으로(dist와 vis는 그대로 이어 간다)
  vis의 dist를 −1로; W += (len(vis) + len(vis))·μ
```
`415f4f(st, r)`(69 B): 행 리스트의 살아 있는 노드마다 `col_cnt −= 1`, `col = −1`; `row_cnt[r] = −2`; `+0x404 += 1`. **추적기 표시도 작업량도 없다.** 뜻: u에서 v로 가는 더 긴 경로가 있으면 간선 u→v(행 r)는 다른 함의에서 나오므로 지운다.

### 7.6 오라클 확인
- 38개 42호출 전부 일치(`s_scc.py`). 그래프가 있는 호출 12(모두 DAG), 함의 행 2,321, CHAIN 3,411, 지운 행 1(fhnw-binpack4-4).
- 오라클을 지나지 않은 것: SCC 원소 처리(`3bb6fe` 집계, x ≡ ¬x), changed 뒤 재구성, 예산 멈춤, PRETRANSRED ≥ 2의 mode 2 단계, `53dce0` mode 1이 실제로 bound/타입을 바꾸는 경우.

## 3. `3ff744(st, env, T6, mode, flag, 0)` 평행 행·열, MIP 연결 리스트 분기 (16,559 B, `004cded0`)

LP 포트 `crates/lp/src/presolve/parallel.rs`(CSR 콜백, m1_presolve §7)와 뼈대가 같다. 아래는 MIP(+0xe0 = 0, +0xfc = 1)에서 다른 점과 MIP 전용 블록이다. 3cd406은 바깥 패스에서 mode 3 flag 0, mode 1 flag 1, mode 0 flag 1 순으로 부른다(spine-b). mode 2는 `3b0917`의 행 분할(§3.8), mode 4/5는 QC.

### 3.1 콜백과 플래그
| mode | 해시 | 값 해시(척도) | 비교 | 쌍 |
|---|---|---|---|---|
| 0 행 | `3e7ec1` | `3e8260` | `3e92c4` | `3ee2bb` |
| 1, 3 열 | `3e7e33` | `3e80fd` | `3e8a37` | `3eaf1d` |
| 2 행 분할 | `3e7ec1` | `3e8260` | `3e92c4` | `3f26d9` |
- mode 0: 진입에서 `+0x42c = 0`(진입 값은 기억), flag && PreCrush(env+0x26f4)==0 && DualReductions면 `iv41 = GURO_PAR_PREPWLRED(env+0x256c)`(기본 1, P/T 행 2단계). mode 1/3: 같은 조건이면 `f0 = GURO_PAR_PREOBJPWLRED(env+0x2570)`(기본 1, 열 집계 단계).
- **콜백에는 작업량이 없다**(LP CSR판은 있다). 함수 전체의 작업량은 처음 한 번 `W += ((cnt·20)·nnz/(dim+1))·μ`(nnz = st+0xd8, dim = 행이면 m, 열이면 n)와 쌍 함수·2단계 안의 것뿐이다.
- 해시: 살아 있는 항목마다 `h = h + 1/((f64)index + 1)`(행은 열 번호, 열은 행 번호), 한 갈래로 차례로 더한다(LP는 두 갈래).
- 값 해시: 행(`3e8260`)은 **최소** |a|(1e100에서 시작, `amin <= |a|`가 아니면 갱신), 열(`3e80fd`)은 최대 |a|(`|a| <= amax`가 아니면 갱신). 부호는 가장 작은 index 항목의 부호(a ≤ 0이면 −1). 반환 `Σ |a|/(index+1) / 척도`, 척도 = 부호·(min 또는 max).
- 비교: 개수(row_cnt/col_cnt)가 다르면 작은 쪽이 −1. 척도의 |최대| > 1000이면 둘 다 1000/최대를 곱한다. x의 a/s1을 scratch(+0x3b0, 행 리스트 순서)에 쓰고 y의 a/s2와 비교해 차이 나는 가장 큰 index를 hi/lo로 모은 뒤 x에 남은 칸을 정리(LP와 같은 구조, 반환 hi<lo → 1, 같으면 0, 아니면 −1). 행 허용치는 1e-10. **열 허용치**는 `t = |s1| > |s2| ? (ub_x−lb_x) + (ub_y−lb_y)·(s2/s1) : (ub_y−lb_y) + (ub_x−lb_x)·(s1/s2)`, `tol = clamp(|t|·1e-6, 1e-13, 1e-10)`, `d = a_y/s2 − tmp; d > tol → hi, d < −tol → lo`.

### 3.2 후보와 P/T 행
```
mode 0: 53a487(행 인접 목록)에서 row_cnt > 1이고 36dce1(이미 넘칠 수 없는 행)이 아닌 행
mode 1: 53af0f(열 인접 목록)에서 col_cnt > 0;  mode 3: 그중 vtype != 'C'
(SOS·PWL·QC 등 특수 구조가 있으면 colflag ≠ 0 열을 뺀다. 지금 모델은 없음)
if len < 2: 끝(아래 3.6의 마무리만)
mode 0 (MIP): 후보 행마다 살아 있고 col_cnt == 1인 열(행에만 있는 slack 열)을 센다:
  그런 열에 colflag ≠ 0이 있거나, iv41 == 0인데 2개 이상이면 → 후보에서 뺀다
  0개 → 그대로; (1개 또는 모두 연속) && (DualReductions || sense == '=')이면 그 노드들을 숨긴다:
    node.col = −2 − col, row_cnt[i] −= 1, sense = (= → 'T', < → 'P'), 후보로 남긴다; 그 밖은 뺀다
W += ((cnt·20)·nnz/(dim+1))·μ                       // cnt = 거른 뒤 개수
```
숨긴 노드는 col < −1이라 모든 콜백·쌍 함수에서 죽은 노드로 보인다. 함수 끝(3.6)에서 되살린다.

### 3.3 묶기·정렬 (LP와 같음, 정렬은 `ab9789`)
1단계 해시로 정렬하고 첫 원소와 해시 차 > 1e-13이면 새 묶음, 크기 ≥ 2인 묶음은 값 해시로 다시 정렬해 첫 원소와 차 > 1e-10이면 새 소그룹. 2단계 정확 비교(크기 2는 한 번, 3 이상은 비교 함수로 Shell 정렬 n/2, ÷2.2, 2→1 뒤 첫 원소와 다시 가름). 3단계 크기 ≥ 3 묶음:
- 열 mode이고 묶음이 모두 정수(vtype ≠ 'C')면 키 = |척도|, 아니면 키 = cost/척도(행 rhs, 열 obj). 정렬.
- 정수가 아닌 원소가 있으면 LP처럼 첫 키 + 1e-10 안의 블록(3개 이상)을 |척도|로, **모두 정수이고 mode ≠ 3이면** 같은 블록을 |cost|로 정렬한다. 모두 정수이고 mode 3이면 둘째 정렬은 없다. 블록 루프는 `q < e − 2`인 동안.

### 3.4 쌍 처리 (LP와 같은 순서)
크기 2는 한 번, 3–6은 모든 쌍(앞 원소가 죽으면 다음), 7 이상은 앞으로 한 번·뒤로 한 번(이끄는 원소 규칙, mode 0은 순위 'P'0 'T'1 '<'2 '='3, |척도| ≤ 1e13 …). 쌍 1만 개마다 `3ba5ac`/`3ba9de`(상태 변화 없음).

#### 3.4.1 `3eaf1d(s1, s2, st, env, x, y, mode)` 열 쌍 (4,539 B + 0x3c8efc.. 떨어진 조각)
|s| > 1e13이면 0. |척도|가 큰 쪽을 x로(|s1| < |s2|면 맞바꿈), r = s_y/s_x, q = c_y/r, unit = (||r|−1| ≤ 1e-10).
- st+0x100 == 0이면 tx = 1e-9/max(|s_x|,1), ty = 1e-9/max(|s_y|,1), 그리고 **유한 bound가 있고 ub − lb > t인 열만** `3dcc78(INF, 0, st, j, mode 1, single 0, −1)`로 함의 bound(lo, hi)를 구한다(열 리스트 전체, `W += (μ·7)·방문`, 정수 열은 ceil/floor). 아니면 (−INF, INF).
- 경우(정수 열은 상대도 정수이고 unit일 때만 경우를 가진다):
  T1 x lb 함의(lb_x ≤ −INF 또는 −tx ≤ lo_x − lb_x)이고 `!(c_x <= q − 1e-10)` → 1; T2 x ub 함의(INF ≤ ub_x 또는 hi_x − ub_x ≤ tx)이고 `!(q + 1e-10 <= c_x)` → 2; T3 y lb 함의이고 `!(c_y <= c_x·r − 1e-10)` → 3; T4 y ub 함의이고 `c_y < c_x·r + 1e-10` → 4 (T4의 정수 조건: y가 'C'이거나 (x 정수 && unit)). 먼저 성립한 것. mode 3이고 경우 0이면 3.4.2(경우 5)를 먼저 본다.
- 경우 4: (r<0 && ub_x<INF) 또는 (r>0 && lb_x>−INF)면 x를 (r<0 ? ub_x : lb_x)에 `3de18e(…, rec 1)`. 아니면 p = r·c_x; c_y > p − 1e-10이면 병합 검사, p − 100·ot > c_y이고 |s_x| > 1e-9면 비유계(+0x470 = x, +0x474 = y, 5). 경우 3/2/1도 LP와 같은 대칭(3: x를 (r>0 ? ub_x : lb_x), 2: y를 (r<0 ? ub_y : ly), 1: y를 (r>0 ? ub_y : ly); 병합 검사 조건 1e-10 + p > c_y, c_x > q − 1e-10, 1e-10 + q > c_x; 비유계 기준은 |s_x| 또는 |s_y|).
- 병합 검사(경우 0 포함): `w = (((ub_y−lb_y)/|r|) + (ub_x−lb_x))·|c_x − q|`. w > 1e-6이거나 PreCrush ≠ 0이면 끝. |c_x − q| < 1e-10이고 (둘 다 연속 또는 둘 다 정수이고 unit)이면 병합. 아니면 PREPARALLEL(env+0x2560) == 2일 때만 다른 변형(해독 안 함).
- 병합: type 4 레코드 `39a1e9`(0x50 B, data {i32 x, i32 y, f64 lb_x, lb_y, ub_x, ub_y, r, i32 0}, +0x408++), `5379f1(x)`, x가 'B'면 `5b91ad`(st+0x3f0 블록의 열 표시, dump 밖), x bound = LP와 같은 식(r>0: [lb_x + r·lb_y, ub_x + r·ub_y], r≤0: [lb_x + ub_y·r, ub_x + r·lb_y], 무한이면 무한), x가 정수면 lb = ceil(lb − ift), ub = floor(ub + ift), [0,1] 안이면 'B' 아니면 'I'. `3dd6f1(y: (lb_y,ub_y) → (0,0))`, `3dd6f1(x: 옛 → 새)`, obj_y = 0, col_cnt_y = −1, y 열의 살아 있는 노드마다 행 표시(`53763f`), row_cnt −1, col = −1. 작업량은 `3dd6f1` 안의 것뿐.

#### 3.4.2 경우 5 (mode 3, 정수 두 열을 정수 하나로)
PreCrush == 0, 둘 다 정수, s_x/s_y가 정수(frac ≤ 1e-10, `v − floor(v + 1e-10)`), |c_x/s_x − c_y/s_y| ≤ 1e-10, lb_x + 0.5 < ub_x, lb_y + 0.5 < ub_y, 네 bound 모두 유한. |s_y| < |s_x|이면(보통): f0 = (s_y ≥ 0 ? lb_y : −ub_y), f8 = (s_y ≥ 0 ? ub_y : −lb_y), c128 = (s_x ≥ 0 ? ub_x : −lb_x), c120 = (s_x ≥ 0 ? lb_x : −ub_x), k = floor(|s_x|/|s_y| + 1e-10). (|s_x| == |s_y|면 x·y 역할을 바꾼 식.) `|hi−lo| ≤ 1e-10 || k ≤ (1 − f0) + f8`이면 f0 = f0 + c120·k, d = f8 + k·c128, 그리고 (INF < f0 || d < INF)이면: s_y < 0이면 (f0, f8) = (−d, −f0) 아니면 f8 = d. 남기는 열 keep = y(보통), 없앨 열 elim = x. keep이 'B'면 `5b91ad`, type 4 레코드 {keep, elim, lb_keep, lb_elim, ub_keep, ub_elim, s_x/s_y, **1**}, keep 표시, lb/ub = (f0, f8), vtype = 'I', obj_elim = 0, col_cnt_elim = −1, elim 열의 살아 있는 노드마다 행 표시·row_cnt −1·col = −1. **activity는 고치지 않는다**(z = y + k·x의 기여가 두 열 기여의 합과 같다). 조건이 안 되면 병합 검사로.

#### 3.4.3 `3ee2bb(s1, s2, st, env, r1, r2)` 행 쌍 (6,050 B, 작업량 없음)
|s| > 1e13이면 0. smin = min(|s1|,|s2|), smax = max, ft = FeasibilityTol, ftc = min(ft, 1e-6): NumericFocus ≤ 0이면 smax = max(smax, 1), smin = min(smin, 0.1); 1이면 max(·,10), min(·,0.01); 2 이상이면 max(10·smax, 100), min(0.1·smin, 0.001). tol2 = ftc/smax, tol = max(1e-6, ft)/smin. a는 'T' 또는 '=' 행을 우선(r2가 'T'이고 r1이 'T'가 아니면, 또는 r2가 '='이고 r1이 'T'/'='가 아니면 맞바꿈). r = s_a/s_b. P/T 행의 숨긴 노드가 2개 이상이면 0.
- a '<', b '<' 또는 'T'(b 'P'면 0): q_a = rhs_a/s_a, q_b = rhs_b/s_b. 같은 방향(s_a, s_b 같은 부호)이면 느슨한 쪽을 지운다(s_a>0: q_a < q_b → b, 아니면 a; s_a≤0: q_b < q_a → b, 아니면 a). 마주 보면 (s_a ≤ 0, s_b ≥ 0) q_a > tol + q_b → 불가능(+0x478 = a, +0x47c = b, 3); q_a > 10·ft + q_b → 아무것도; `q_b − tol2 < q_a`면 **=로 합침**, 아니면 `+0x42c += 1`. (s_a > 0, s_b ≤ 0은 q_a·q_b 역할을 바꾼 같은 식.) =로 합침(MIP는 type 5 레코드를 쓰지 않는다): `rhs_a' = ((s_a·rhs_b)/s_b + rhs_a)·0.5`, sense_a = '=', d = rhs_a − rhs_a'가 0이 아니면 act_max_a = gup(act_max_a, d), act_min_a = gdn(act_min_a, rhs_a − rhs_a'), 행 a 표시, b 지움.
- 행 지움(DROP): 살아 있는 노드마다 열 표시(`5379f1`)·col_cnt −1·col = −1, row_cnt = −2, +0x404++.
- a '=': b 'P'면 숨긴 열 x_b에 대해 v = (rhs_b·r − rhs_a)/(r·a_xb), a_xb ≤ 0이면 lb' = max(lb, v) 아니면 ub' = min(ub, v), 정수면 ceil/floor(±ift), (lb' ≤ ft + ub && lb − ft ≤ ub')이면 bound를 쓰고(바뀌면 열 표시, **`3dd6f1` 없음**) |r| ≥ 1e-6이면 b의 숨긴 노드를 되살리고 sense '<'로 바꿔 DROP, 아니면 불가능. b '='면 |q_a − q_b| ≤ ft, b '<'/'T'면 (s_b ≤ 0 || q_a − ft ≤ q_b) && (s_b ≥ 0 || q_b ≤ ft + q_a)일 때 |r| ≥ 1e-6이면 b DROP, 아니면 불가능.
- a 'P', b 'P', |r − 1| ≤ 1e-10: 숨긴 열 x_a, x_b의 계수·타입·bound(1e-10)·rhs가 같고 obj_a·obj_b ≥ 0이면 type 2 레코드 `39df2d`(x_b = 0 + 1·x_a, 0x50 B, data {i32 j, pad, f64 c0, i32 cnt=1, ptr ind, ptr val, i32 k, f64 v}, +0x408++), obj_a += obj_b, obj_b = 0, x_a 표시, b의 숨긴 노드를 되살려 b의 모든 살아 있는 노드를 DROP처럼 지우고 col_cnt[x_b] = −3, row_cnt[b] = −2, +0x404++. (sp150x300d 60쌍.)
- a 'T'(숨긴 열 x_a, 계수 d): b '='면 v = (rhs_a − rhs_b·r)/d가 [lb−ft, ub+ft] 안이고 (연속이거나 v − floor(ift + v) ≤ ift)이면 노드를 되살려 `3de18e(v, …, rec 1)`, row_cnt[a] += 1, sense_a '=', 행 a 표시, b DROP; 아니면 불가능. b '<'면 r·d > 0이면 lb' = max(lb, v) 아니면 ub' = min(ub, v), 정수면 ceil/floor 하고 반올림 차·|d| > ft이면 b는 남긴다(good = false); 범위가 맞으면 bound가 바뀔 때 노드를 잠시 되살려 `3dd6f1` 후 다시 숨기고 x_a 표시, good이면 b DROP; 아니면 불가능.
- a 'T', b 'T': k = (d_b·r)/d_a, c = (rhs_a − rhs_b·r)/d_a. flag: x_a 연속이면 0, x_b 연속이면 1, 둘 다 정수면 k와 c가 정수면 0, 아니면 1/k와 c/k가 정수면 1, 아니면 2(아무것도 안 함). flag 1: x_b의 bound로 x_a bound를 좁히고(k>0: lb_a' = max(lb_b·k + c, lb_a)(maxsd), ub_a' = min(ub_b·k + c, ub_a); k≤0은 lb/ub 바꿔), 정수면 ceil/floor, lb' > ft + ub'이면 불가능; 바뀌면 `3dd6f1`(노드를 잠시 되살림), type 2 레코드 (x_b = −c/k + (1/k)·x_a), obj_a = obj_b/k + obj_a, **objcon = objcon − (c·obj_b)/k**, obj_b = 0, x_a 표시, b의 숨긴 노드를 되살려 b DROP, col_cnt[x_b] = −3. flag 0: 역할을 바꿔(k1 = 1/k, c1 = −c/k) x_b의 bound를 좁히고 type 2 (x_a = −c1/k1 + (1/k1)·x_b), obj_b = obj_a/k1 + obj_b, objcon −= (c1·obj_a)/k1, a DROP, col_cnt[x_a] = −3. (timtab1 2쌍.)

### 3.5 2단계
- 행(iv41 ≠ 0): 각 묶음(크기 ≥ 2)에서 살아 있는 행을 모아 P/T 행이 하나라도 있으면 `3fa2f6`(1,338줄, 해독 안 함)에 넘긴다. **38개에서 이런 묶음은 한 번도 없었다.**
- 열(f0 ≠ 0): 크기 ≥ 2 묶음마다 원소를 차례로 보며 col_cnt == 1인 살아 있는 열이 나오면 그 묶음을 버리고(그 자리까지), `W += ((t − b)·μ)·3`(t = 멈춘 위치). col_cnt ≥ 2인 살아 있는 열이 2개 이상이면 집합으로 모은다. 끝에 `W += (nb·2)·μ`(nb = 묶음 수). 집합이 있으면 **새 열과 새 `=` 행으로 묶음을 하나의 변수로 집계**한다. 38개에서 exp-1-500-5-5 2번째, mik-250-20-75-4 2번째 호출이 여기 온다(asm 0x3d0767–0x3d1940, 포트와 일치).
```
W += (n·0.5)·μ                                   // 척도 배열 복사, n = 그때의 열 수
집합마다(L = 크기, cols = 살아 있는 원소, 묶음 순서):
  L < 3 또는 AggFill+10 >= ((L−1)·col_cnt[cols0] − 1) − L(AggFill −1은 0)이면 건너뜀
  s = 척도[c]: best = |s|가 가장 큰 열(차 > 1e-13, 또는 차 >= −1e-13이고 s > 0이면 바꿈), smin = minsd, allint
  W += (L+L)·μ; best 없음, smax < 1e-10, smax > smin·1e6이면 건너뜀
  base = s_best; allint이고 g = a7bea8(|s|)가 [0.001, 1000]이면 base = ±g(s_best 부호)
  z = 3bbf87(1)                                   // 빈 'C' 열 [0, INF], 추적기 레벨 0 목록 끝 + 표시
  best의 살아 있는 노드: col = z, val = (base/s_best)·val, 3c3abf(행, 노드, best, 새, 옛), 행 표시, row_cnt −= L−1
  W += (방문·3)·μ; col_head[z] = col_head[best], col_head[best] = NIL, col_cnt[z] += col_cnt[best], col_cnt[best] = 0
  원소마다 모든 노드 col = −1, col_cnt = 0, W += (방문+방문)·μ;  W += (L·3)·μ
  f = minsd(maxsd(1, max_k 1/|s_k/base|), |base|·10)
  r = 3bde19(1); sense[r] = '='; 3bea24(−f, r, z)
  원소마다 t = s/base: int &= vtype≠'C' && t − floor(t+1e-10) ≤ 1e-10; 3bea24(f·t, r, c);
    lo/hi += (t>0 ? lb,ub : ub,lb)·t (gdn/gup, ±1e30이면 무한으로 고정); 열 표시
  W += (L·12)·μ; int이면 vtype[z] = 'I', lo = ceil(lo − ift), hi = floor(hi + ift); lb/ub[z] = lo/hi
  370047(r); lo ≤ −1e30 또는 hi ≥ 1e30이면 z의 살아 있는 노드 행마다 370047, W += (방문+방문)·μ
W += (집합 수·2)·μ
```
`3bde19`: 빈 `<` 행, rhs 0, activity 0, 추적기 `536369`(레벨 0 목록 끝, 표시). 새 행·열은 이름이 없고 MPS에는 `R<번호>`/`C<번호>`(presolve 모델 안 번호)로 나온다(mik: R75, C266).

### 3.6 마무리
`+0x42c = max(진입 값, 지금 값)`. mode 0(MIP)이면 후보 목록(정렬된 순서)의 'P'/'T' 행마다 col < −1 노드를 `col = −2 − col`로 되살리고 row_cnt += 1, sense를 'T' → '=', 'P' → '<'로.

### 3.8 mode 2 (`3b0917`, 모델 생성)
`3b0917`: +0x42c ≠ 0이고 (+0xfc ≠ 0 또는 PreCrush ≤ 1)이고 PRESPLITROWS(env+0x258c) ≠ 0, env+0x2728 == 0, (PRESPLITROWS > 0 또는 +0xf4 < n)이면 `3ff744(st, env, 0, 2, 0)`. mode 2: +0x42c = 0(끝의 max도 0 기준), 후보 = 모든 행 중 row_cnt > 1, sense ≠ '=', `36dce1` 아님. +0x3e0/+0x3e8 새로 할당. 크기 2 묶음만 쌍 함수 `3f26d9`(작업량 없음): s1·s2 ≤ 0이고 a(양 척도)·b에서 rhs_b/s_b ≤ rhs_a/s_a − ft이면, PRESPLITROWS < 0일 때 row_cnt[a] < 100이거나 a에 'C' 아닌 열이 있으면 0. 아니면 type 14 `399cbe`(data {a, b, k, cnt, rhs_a, s_a·rhs_b/s_b, s_a/s_b, ind @+0x38, val @+0x40+((4cnt−1)&!7)}), +0x3e0[k] = a, +0x3e8[k] = s_a·rhs_b/s_b, +0x42c++, a 표시, +0x404++, sense_a = '=', b의 살아 있는 노드마다 열 표시·col_cnt −1·col = −1, row_cnt[b] = −2. 모델 생성은 살아 있는 열 뒤에 분할마다 열('C', 비용 0, [0, rhs_a − +0x3e8[k]], 행 a에 1)을 붙인다. 기본값 38개: ic97_potential, neos-3046615-murg, supportcase26에서 불리지만 분할은 없다(100 미만 또는 정수 열).

### 3.7 오라클 확인
- 38개 237호출(3cd406 안 234 + `3b0917` mode 2 3) 모두 포트와 일치(`mip_dump_check.py`).
- 지난 경로: 쌍 3,581, 열 경우 0 667·병합 5·경우 5 4, 행 '<''<' 0번 1,888·1번 763·3번 47, P-P slack 병합 60, T-T 2, 숨긴 행 P 1,742·T 465. 열 경우 1–4와 고정, 행 '=' 쌍, T/'='·T/'<', 2단계 행(`3fa2f6`)·열 집계는 오라클을 지나지 않았다.

## 4. `57627e(st, env)` implied integer, 정수 등식 (10,175 B, `005b8f70`)

### 4.1 흐름
```
(cnt, list) = 538353(tr)                                   // 레벨 1 원시 행 a-목록
for i in list: if row_cnt[i] > 1 && sense[i] == '=':
  rhs0 = rhs[i]; classes = []; allok = true; cont = null; ints = []; fixed = 0
  행 리스트의 살아 있는 노드(c, a)마다:
    if ub[c] − lb[c] <= 1e-13: node.col = −1, fixed = fixed + lb[c]·a, row_cnt[i]−−, col_cnt[c]−−   // 고정 열을 rhs로(레코드 없음)
    else:
      if !allok || lb <= −1e30 || 1e30 <= ub || ift + ub < lb || vtype ∉ {B, I}: allok = false
      else: |a| < 0.001 || |a| > 2e9이면 allok = false, 아니면 같은 |a|의 클래스(|a − v| ≤ max(|a|,v)·1.6e-9)에
            개수 +1, 범위 [lo, hi] += (a<0 ? [−ub, −lb] : [lb, ub]); 없으면 새 클래스(최대 5개, 6번째면 allok = false)
      vtype == 'C'면 cont가 이미 있으면 중단(이 행은 끝), 아니면 cont = 노드; 아니면 ints.push(|a|)  // +0x398 scratch
  W += (row_cnt[i]·5)·μ                                      // 고정 열을 뺀 개수
  rhs[i] −= fixed; rhs0 −= fixed
  if 중단 || row_cnt[i] <= 1: continue
  if cont: |a_c| < 0.01 || |a_c| > 100이면 continue; |rhs[i]| > 1e-10이면 ints.push(|rhs[i]|)
  g = a7bea8(ints); if !(0.001 < |g|): continue
  if cont: IMPLIED_INT; continue
  INT_ROW
W += (cnt·10)·μ
if 바꾼 행 목록: 370047(st, env, k, 목록)                    // activity 재계산
```
IMPLIED_INT (연속 열 xc 하나, 나머지 정수): d = g/|a_c|. d ≤ 1e8, d ≥ 1e-8, colflag[xc] == 0, (lb ≤ −INF || −1e8·d ≤ lb), (INF ≤ ub || ub ≤ 1e8·d)이면 x_c = d·x′(x′ 정수)로 바꾼다:
- type 9 레코드 `3a061e(0.0, d, env, st, xc)`(0x30 B, data {i32 xc, pad, f64 0.0, f64 d}).
- `53802c(xc)`(행 표시, `W += (μ·5)·방문`), `5379f1(xc)`.
- lb > −INF면 lb′ = ceil(lb/d − ift), `3dd6f1(lb, ub, d·lb′, ub)`, lb = lb′. ub < INF면 ub′ = floor(ub/d + ift), `3dd6f1(lb_now, ub, lb_now, d·ub′)`, ub = ub′(activity는 원래 척도의 bound로 고친다).
- 열 리스트의 모든 노드 a ← a·d, `3c3abf(new, new, …)`(|new| ≤ 1e-13이면 노드 제거, 아니면 아무것도).
- `W += (col_cnt[xc]·2)·μ`; IMPLIEDINT(env+0x2820) ≠ 0이면 vtype = 'I'; obj[xc] ·= d; ft + ub < lb면 +0x470 = xc, 3.

INT_ROW (모두 정수):
- ratio = rhs/g, tolr = min(ft/min(g,1), 0.1), fl = floor(tolr + ratio); ratio − fl > tolr면 +0x478 = i, 3. rhs = fl.
- 행 리스트의 노드마다(방문 v) 살아 있으면 a ← floor(a/g + 0.5); 0이면 행·열 표시, row_cnt−−, col_cnt−−, col = −1; 아니면 |a|가 가장 작은 노드를 기억(mn). `W += (v·5)·μ`. |g − 1| > 1e-10이면 행을 재계산 목록에.
- 클래스 분석(allok, 클래스 ≥ 2, 개수 1인 클래스가 있을 때): G = a7bea8(클래스 값), G ≥ 0.001이면 rq = rhs0/G(나누기 전 rhs), infeas = (rq − floor(rq + 0.01) > 0.01). rq가 정수(1e-10)이면 rq = floor(rq + 0.5), 클래스 값 v_k = floor(|a_k|/G + 0.5)(정수가 아니면 중단), 범위가 ±1e9 밖이거나 Σv·lo, Σv·hi(와 rhs를 뺀 값)가 ±2e9 밖이면 중단; 중단이 아니면 infeas면 3. |rq| < 1e8이면 개수 1 클래스마다 나머지의 gcd G1(ift + 1 ≤ G1 < 99999999), bound가 ±1e8·G1 안, `r3 = rq − (G1 − 1)·v_k·sgn`, `(r3 − floor(ift + r3))/G1`가 정수면 `574764`(대입, x_k = G1·z + r)를 부르고 다음 행으로. 아니면 `575405(&클래스 구조)`(9,181 B, 작은 배낭 열거로 bound를 좁힘, `571c44`)와 그 결과의 bound 적용(`3dd6f1`). 중단이면 infeas일 때 3.
- 가장 작은 계수의 대입: thr = (mn + 0.5)·g, ints 중 thr 이상을 floor(·/g + 0.5)로 모으고 `W += len(ints)·μ`. 하나만 빠졌으면 나머지의 gcd G2 ≥ 1.5, rhs가 정수이고 |rhs| ≤ 1e8, `(rhs − (G2−1)·a_min)`의 나머지 조건이 맞으면 `3c5996`(모듈러 역원)과 `574764`.

### 4.2 `a7bea8(n, v)` double gcd (2,630 B, 작업량 없음)
n = 0 → 0, n = 1 → |v0|. mn = min|v|, mx = max|v|(같으면 앞 것). mx > 2e9이거나 mn < 1e-5면 0. mx − mn ≤ 1e-10이면 mn. 모든 |v|/mn이 정수(허용 max(1/mn,1)·1e-10)면 mn. mx·600 ≤ 2e9면 모든 |v|·600이 정수(6e-8)일 때 floor(|v|·600 + 0.5)의 정수 gcd/600(정수 gcd: 0이 있으면 0, 2e9 이상인 쪽은 상대를 돌려줌). 아니면 mn ≥ 1.9999999999이고 모두 정수(1e-10)면 정수 gcd. 아니면 double Euclid(v0부터, 몫 floor((큰 + 1e-10)/작은), 나머지 ≤ 0이면 0, 작은 ≤ 1e-10까지, 결과 < 1e-9면 0) → g ≤ 1e-10이면 0; 1/g가 정수(1e-11)이고 모든 |v|/g가 정수면 g; 아니면 g의 연분수(분모 ≤ 1000, 오차 1e-13)로 p/q를 찾고 모든 |v|·q/p가 정수면 p/q, 아니면 0.

### 4.3 오라클 확인
- 38개 78호출 전부 일치(`s_iint.py`). IMPLIED_INT 495번(csched007 246, csched008 238, markshare2 7, markshare_4_0 4: vtype 'I'와 type 9 레코드, d = 1이라 bound·계수는 그대로), 클래스 분석 251번 → `575405` 251번(cov2의 호출 수와 같다).
- **`575405`는 해독하지 않았다.** 디컴파일을 보면 클래스 합의 bound를 좁혀 클래스 lo/hi 배열에 쓰고 불가능 플래그(param_1[5])와 개수(param_1[6])를 세운다. 즉 bound를 **바꿀 수 있다**. 38개에서는 바꾼 적이 없어(덤프에 bound 변화 없음) 포트는 아무것도 하지 않는다(`shortcut:`). `571c44`와 함께 포팅해야 한다. `574764`는 round-end의 `State::mod_subst`(roundend.rs)로 포팅되어 있다. 57627e의 두 호출 자리(0x4bb04a 클래스 대입, 0x4bb3a7 가장 작은 계수 대입)는 38개에서 지나지 않아 인자 해독만 일부 하고 연결하지 않았다(`shortcut:`). `574764`, `3c5996`, 고정 열 접기, 계수 0, 행 나누기(g ≠ 1), 불가능 반환은 오라클을 지나지 않았다. a7bea8의 Euclid·연분수 가지는 gmu-35-40/50에서 지났다.

## 5. `578fd2(st, env)` knapsack 계수 강화 (1,281 B, `005bea50`)

### 5.1 호출부
```
(cnt, rows) = 538f30(tr, st);  k_chg = 0;  tmp = 열 크기 0 배열(+0x3b0)
for i in rows: row_cnt[i] > 0 && sense[i] != '='일 때
  ind/val = 행 리스트의 살아 있는 노드;  W += (μ·5)·row_cnt[i]
  rc = 6aec61(env, &k, ind, val, &rhs[i], lb, ub, vtype, PREPOWER(env+0x2564), 2, &changed, W)   // W 포인터에 μ 없이 더한다
  changed면: chg에 i, 행 표시, tmp[ind] = val, W += (μ·3)·k, row_cnt[i] = k,
    행의 살아 있는 노드마다 nv = tmp[col], tmp[col] = 0, |a − nv| > 1e-10이면 열 표시, nv == 0이면 col_cnt −1·col = −1 아니면 a = nv
    W += (μ·4)·row_cnt[i]
W += (cnt·6)·μ;  k_chg > 0이면 +0x3fc += k_chg, 370047(chg)
```

### 5.2 `6aec61` (p10 = 2, p9 = 0 경로; 작업량은 모두 W에 μ 없이)
보조: `a7cec0(s)` = Σ bound·(s·a)(s·a ≤ 0이면 ub, 아니면 lb, 무한이면 −1e100), W += 3·방문. maxact = −a7cec0(−1), minact = a7cec0(1), 1e101 = 낡음. drop = 고정 열(ub−lb < 1e-10)과 `ft·0.01 > (d·|a|)·k`인 열을 rhs로(a > 0 → lb, 아니면 ub), |a| < 1e-10 제거, W += 4k. compact = |a| ≤ 1e-10 제거, W += 3k. cont(local_38) = 1 연속 열 있음, 0 없음, −1 모름.
```
maxact; −ft ≤ rhs − maxact면 중복(rhs = 0, k = 0, changed)
drop(있으면 changed, maxact 다시); 다시 중복 검사
앞쪽 연속 열 수 c(cont = 1), W += 3c; 모두 연속이면 끝
rhs 정수 && cont≠1: 모든 |a|가 같고(ft·d 안) 정수면 rhs/d 정수일 때 끝(W += 2·본 수)
2 < k ≤ PREKNAPLIFTLEN(−1 → 512) && cont≠1: 모두 [0,1] 이진이면 6b4738(k<11) / 6b5e11(b0 ≠ 0이면 끝)
주 루프(최대 10회, 바뀌는 동안):
  A  s = maxact − rhs > 0: d = 최소 |a|(연속 있으면 0); 정수 열마다 δ = |a|>s ? |a|−s : (s−100ft ≤ |a| 또는 d+|a| ≤ s ? 0 : min(|a|−s/2, |a|+(d−s))),
     δ > 1e-10이면 a를 0 쪽으로 δ, rhs −= ub·δ(a>0) / rhs += lb·δ; W += 3k
  B  k > 2, minact ≥ −1e10, s2 = rhs − minact ∈ [1000ft, 1e10]: dmin(최소 |a|), m2 = min(2dmin, |a| > dmin+ft인 최소 |a|)(W += k);
     계수 루프(W += 5k): 연속·작은 정수 열은 sumC += (ub−lb)·|a|, s2−ft보다 큰 계수 처리, 조건이 맞으면 sumC로 rhs/계수 재구성(W += 3k)
  C  모두 정수: g = a7bea8(val)(1e4 초과면 계수가 정수일 때 1), rhs/g − ft > floor(100ft + rhs/g)이면 rhs = floor, 계수 /g 반올림
  D  모두 정수·lb ≥ 0: i = 1..9, f = −(i + min(0.01, 1000ft))/slack, r = rhs·f − maxact·f ∈ [ft, 1e8], c2 = ceil(r);
     모든 계수가 ceil(|f·a|−1e-10)·(r/c2) 근처이고 하나라도 엄격하면 계수 = ±ceil(...), rhs = −c2 + Σ a·(a≤0 ? lb : ub), compact
  E  |minact| < 1e6: amin, amax, sumC(연속 열의 |a·(ub−lb)|), W += k; 6b33dd(1, sumC), 6b33dd(i/amax, sumC) i=1..5,
     6b33dd(i/(2·amin), sumC) i=1..10, 바뀌면 멈춤
a7c752 단계: k > 2, cont≠1, 모든 계수가 정수이고 |a| ≤ 2e7이고 같지 않으면 a7c752(다른 계수들의 gcd, 세그먼트 트리,
  W += 2·size, 2·size, 4·size), 처음으로 gcd > 1인 정수 열 t에서 6b33dd(1/gcd_t, 0) 한 번, W += 3t
changed면 drop 한 번 더(cont 1 → −1)
s = maxact − rhs: ft < s < 1e10이면 6b33dd(1/s, 0)
```
`6b33dd(f, sumC)`: k < 2, f ∉ [1e-4, 1000], sumC·f ≥ 1이면 끝. r = rhs·f − act·f(f ≤ 0이면 maxact, 아니면 minact) ∈ [ft, 1e8]. fl = floor(r + 1e-10), fr = r − fl. 앞에서부터 합(연속: |fa|·(ub−lb), 정수: 소수부 fa ≥ fr + 1e-10이면 (1−fa)·d를 합과 fr에, 아니면 d·fa)이 1 미만인 동안, W += 5t. fr ∉ (0.001, 0.999)이거나 fr − 합 ≤ min(f,1)·(−ft)이면 끝. rhs = fl, 계수 = 반올림(연속 0, fa ≥ fr2+1e-10이면 |fa|+1−fa), W += 5k, rhs += Σ a·(a ≤ 0 ? ub : lb), W += 3k, compact, cont = 0, 캐시 낡음.
- 확인: 78/78 일치(`6b4738`, `6b5e11` 포함). PREPOWER ≠ 0 블록(0x5863aa–)은 미포팅(`shortcut:`).

### 5.3 정확 이진 knapsack (2 < k ≤ PREKNAPLIFTLEN, 모두 [0,1] 이진, 연속 열 없음)
6aec61은 c = rhs + Σ_{a<0}|a|로 부르고, 돌아오면 `ac`면 changed, **`b0`(local_b0) ≠ 0이면 바로 0을 반환**한다(주 루프·마무리 없음).

`6b4738(c, …)` (k < 11, 3,858 B, `knapsack.rs::knap_small`, 확인됨): tol = 100·ft. 계수 i마다(표시 안 됐고 |a_i| > 1e-13):
- 다른 계수 j(|a_j| > 1e-13)의 부분합 목록 S(정렬, {0}에서 시작): B = S + |a_j| 중 c + tol 이하만(넘으면 멈춤), `far &= tol < |(B − c) + |a_i||`.
  B가 비면 j 건너뜀. `below`(처음 참)는 c − tol > max B인 동안만 참. 거짓이 된 뒤 far가 거짓이면 i를 끝낸다.
  S와 B 병합(|차| < 1e-10이면 B를 버림, B ≤ S면 B 먼저), W += (소비한 B 수 + 소비한 S 수)·5. B에서 들어간 것이 없으면 S 그대로.
- below면 단계 0: δ = min(c − max S, |a_i|) > tol이면 a < 0: a += δ, 아니면 a −= δ·rhs −= δ, c −= δ, |a| < 1e-13이면 a = 0으로 끝.
  far면 단계 1: S에서 c − |a_i| + tol보다 큰 값을 버리고 δ = (c − |a_i|) − max S > tol이면 a > 0: a += δ, 아니면 a −= δ·rhs −= δ.
- |a_i| > 1e-13이면 뒤쪽 계수 중 |a|가 1e-10 안에서 같은 것을 표시(건너뜀).
- 끝: ac면 |a| > 1e-10만 남기고 캐시 낡음. b0 = 1.

`6b56d2(v, pA, pB, pC, pD, pE, pF, pcnt, pcap, j, W)` (`knapsack.rs::add_sums`, 부분합 + 비트마스크, 최대 64개): x = v[j], bit = j < 64 ? 1≪j : 0. x > *pcap → 1, x ≤ 1e-13 → 0, 목록 비면 1. T = S + x(cap 넘으면 멈춤, 마스크 | bit), 없으면 1. 병합(|차| < 1e-10이면 S 마스크 &= T 마스크, T가 작거나 같으면 T 먼저) 최대 64개, 새 값이 있으면 개수 갱신, 64개면 *pcap = 64번째 값, 버퍼 교체. W += (소비 수 합)·5. 0 반환.

`6b5e11` (k ≥ 11, 16,193 B, `knapsack.rs::knap_big`, 포팅됨). ac = *p9(local_58), b0 = *p10(local_60, 처음 1). ft, tol = 100ft. 정렬 V = |a|(`aaedd9`, idx 함께), 합 = vsum(버퍼 16바이트 정렬: 8개 블록 2-lane 누산 4개, (a0+a1)+(a2+a3), lo+hi, 나머지 순차), W += 3n.
- c < 0 → ac 0, b0 1. c + ft ≥ 합, max < tol, max/min > 1e12 → b0 0. min == max이고 c ∈ {1, n−1} → b0 1.
- 합 − min ≤ c + ft: V = 1(W += n), c ≠ n−1이면 c = n−1 → 쓰기. s2 = 합 − c; 합 − min < s2 − tol(하나도 안 들어감): V = 1, c ≠ 0이면 c = 0 → 쓰기.
- n88: 끝에서 V − c > tol인 항목 제외(s2 −= V, W += n−n88+1, 쓰기 때 2c). l138 = V[n88−1]. c + tol < V[0]+V[1]: V[..n88] = 1, c = 1 → 쓰기(집합 포장). 모두 같고(1e-13) min < c: q = floor(c/min + ft), 모두 1이고 c ≤ q면 b0 1로 끝, 아니면 c = q(1이 아니면 V = 1).
- u: V[u−1] ≤ c − ft인 끝(W += n88−u+1). u = 0이거나 (V[u−1] − min ≤ 1e-10이고 c/min이 정수)이면 ac 0, b0 1.
- e8: V[e8−1] + min ≤ c + ft인 끝(> 1, W += n88−e8; e8 = 1이면 0). 앞 e8 합 ≤ c+ft면: ft + c/e8 < V[e8−1]이면 c ≤ V[e8]일 때 b0 1, 아니면 V[e8..n88] = c; 그렇지 않으면 c = max(e8,1), V[..e8] = 1, V[e8..n88] = c. 합 − min ≤ c+ft이고 (e8 = n88 또는 V[e8−1]/(e8−1) − V[e8] ≥ −ft): V[..e8] = 1, 나머지·c = e8−1. 모두 → 쓰기.
- 앞으로(`6b56d2`, cap f0 = max(c, s2+l138) + tol): t = e8에서 c+tol 넘는 첫 합 k가 있으면 L = min(S[k−1], c)로 V[t..n88]를 L로 자르거나 L/c배, c = L. `6b56d2`가 1이면 멈춤; l138 + f0 ≤ min(s2, c) + tol이면 **ac·b0를 쓰지 않고** 반환(V 변경 버림).
- 뒤로 1(t = e8−1..0, 표시 1 건너뜀, V ≤ 1e-13이면 멈춤, f0 + V ≤ c + tol이면 b0 0으로 멈춤): 단계 0 a = c, 1 a = c − V[t]. 합이 64개이고 tol + a ≥ f0면 건너뜀(b0 후보 0). 위에서부터 a + tol 이하이고 t 비트 없는 합(t ≥ 64면 비트 무시, W += (n−l)·(비트 있으면 2, 없으면 1)), d = a − S > tol이면 단계 0: c, V −= d(V ≤ max(1,d)·1e-13이면 c −= V[t], V[t..=1]을 c에서 빼고 0, **V[0]은 그대로**, b0 0, 끝), 단계 1: V += d. 그 뒤 64개 미만이면 비트 있는 합 제거(W += 2·개수), `6b56d2`(V[t]), b0 후보 0. b0 = 후보. V[t] > 1e-13이면 아래쪽 같은 값(1e-10) 표시.
- 앞 e8이 정렬이 깨졌으면 다시 `aaedd9`(local_50 = 1). n88 = e8이면: ac면 s2 = vsum − c(W += n88); u = V < s2 − ft인 끝(**local_e8 = u**, 뒤로 2의 시작). min < s2−ft ≤ V[1]+min: V[..u] ≠ 0이면 1, V[u..n88] = 2, c = 2(n88−u) − 2 + (1 개수). 앞 u 합 < s2−ft: V[..u] = 0, 나머지 1, c = n88−u−1. 합 − min < s2−ft: V[u−1] −= 합 − s2(> ft), V[u..]는 s2 + ft 넘으면 앞 값으로, c = 그 합. 아니고 u < n88: s2 − tol 이상 첫 합부터 s2 + 1e-13 이하 마지막 합 k(W += k, k = 0이면 b0 0으로 끝), V[u..n88]를 S[k](k = 개수면 s2)로(`6b56d2`), W += n88−u, c −= (S[k] − s2) + Σ차이.
- 뒤로 2(local_50 = 0이고 b0 = 0일 때, a73 = ac면 vsum − c 아니면 s2, 시작 local_e8, 표시 2): 단계 0 a = a73, 1 a = a73 − V[t]. a − tol 이상인 첫 합(비트 있으면 비트 없는 것, 없으면 마지막 합; W += 2(개수 − l), 비트 없으면 W += l), d = S − a > tol이면 단계 0: a73, V += d, 단계 1: V, c −= d(0 처리는 뒤로 1과 같음). 합 제거·`6b56d2`는 뒤로 1과 같음.
- 쓰기(ac2d는 ac = 1, b5e2는 ac일 때만): V[n88..] = 2c, W += n − n88; rhs = c, a ≤ 0이면 a = −V·rhs −= V 아니면 a = V, W += 3n; |a| > 1e-10만 남김, 캐시 낡음.
- 확인: 5개 모델의 96호출(tools/knap_trace.js로 입출력 캡처) 비트 일치(테스트 `knap_big_matches_gurobi`, 10호출 fixture; `KNAP_CASES=<캡처>`로 전체).

## 9. 도구와 확인 요약 (세션 스크래치 `outer/`)

| 파일 | 내용 |
|---|---|
| `bin/dumpo.js`, `bin/runall.sh` | 여섯 함수를 한 번에 훅하는 상태 덤프(§1), `d/<모델>.jsonl`(38개 422 MB) |
| `bin/cnto.js` | 함수별·반환 주소별 호출 수(common 모듈 함수도), `cov6ae.json` |
| `bin/rod.py` | .rodata 상수 읽기 |
| `bin/simo.py` | St 확장, 레코드 디코더(type 1, 2, 4, 5, 9, 25), 비교, 목록 getter(`538f30`/`5399fc`/`53af0f`/`53a487`), `ab9789` 정렬, `3dcc78` mode 1 |
| `bin/s_xor.py`, `s_par.py`, `s_iint.py`, `s_dual.py`, `s_scc.py` | 함수별 모형 |
| `a_<id>.txt` | objdump |

| 함수 | 호출 | 일치 | 오라클을 지나지 않은 것 |
|---|---|---|---|
| `3e5363` | 78 | 78 | 축소 전부(레코드 25, 행 삭제, 고정) |
| `3ff744` | 237 | 237 | 열 경우 1–4·고정·비유계, 행 `=`쌍·T/=·T/<, `3fa2f6`, mode 2 분할 |
| `57627e` | 78 | 78 | `575405`(해독 안 함, 변화 없음 가정), `574764`, 고정 열 접기, g ≠ 1 |
| `578fd2` | 78 | 78 | PREPOWER ≠ 0 미포팅 |
| `374521` | 78 | 78 | 고정, 불가능, Presolve 2 |
| `5431ee` | 42 | 42 | SCC 원소(`3bb6fe`), PRETRANSRED 2 |

## 10. 미해결
1. ~~`578fd2`: `6b5e11`(+`6b56d2`, §5.3)~~ 포팅됨: 78/78, presolve_check·mip_rec_check 38/38.
2. `57627e`: `575405`/`571c44`(bound를 바꿀 수 있음), `574764` 두 호출 자리 연결(`State::mod_subst`).
3. `3ff744`의 `3fa2f6`, `5431ee`의 `3bb6fe`(SCC 원소 집계): 38개에서 지나지 않음.
4. 합성 XOR 모델(`tools/outer_syn/xor1.lp`, `xor3.lp`)로 `3e5363` 축소 경로 확인됨(§2).
