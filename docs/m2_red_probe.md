# M2 MIP presolve: probing `4d3a1d` 해독 노트

`3cd406`의 probing `4d3a1d`(file+0x458790, 40,233 B, 디컴파일 5,495줄)와 기본값에서 그 안에서 도는 보조 함수의 해독이다(2026-11-06). 정적 해독(Ghidra + objdump)을 Frida로 확인했다(§4). 줄 번호 L은 `re/decomp/c/00458790_PRIVATE00000000004d3a1d.c` 기준, 주소는 file offset이다(Ghidra 주소 = +0x100000).

기호: st = presolve 상태(`3cd406`의 local_150, 4d3a1d의 param_3), env = param_4, μ = st+0x448, W = `*work`(work = st+0x458이 가리키는 double, 4d3a1d의 local_f0), INF = env+0x2458(1e30), ft = FeasibilityTol(env+0x2460, 1e-6), itol = env+0x2468(1e-5, IntFeasTol. Frida로 값 확인). n = st+0xc, m = st+0x10. `gup(x,t)`/`gdn(x,t)`는 LP 포트 `state.rs`의 `guarded_add(x, t, true/false)`와 같은 덧셈(합이 부정확하면 max 쪽은 위로, min 쪽은 아래로 1±2⁻⁵²배)이다. 작업량 식의 곱셈 순서는 원본 그대로 적었다(`a*4.0*μ`는 `(a*4.0)*μ`). 덧셈 +0.0은 생략했다.

## 0. 요약

- `3cd406`이 두 곳에서 부른다. 라운드 끝(probeMode ≠ 0일 때, 모드 p7=0)과 마무리 단계(조건부, p7=p8=p10=1, p9=lift 플래그). 기본값 38개에서 61회(첫 지점 42회, 마무리 19회). 스레드 분할(`ac761d`)은 한 번도 안 돈다(§1.4).
- 이 노트의 의사코드를 Python으로 다시 짜서 Frida 덤프와 비트 단위로 맞췄다: probing 행렬 구성(8개 모델), `4e48f7`(35개 모델 58회 전부, 작업량 포함), lifting(5개 모델), bound 반영(4개), 보조 함수(§3.5). §4.
- 한 번의 호출은 패스 1–2번이다. 패스마다: 모든 살아 있는 행으로 probing 행렬(CSR 복사, 행마다 정렬)을 만들고, 후보(이진 'B' 열 + 조건을 만족하는 `<` 행)를 `4e48f7`에 넘긴다. `4e48f7`이 x=0/x=1(행 후보는 행 분기)을 시험 고정·전파해서 ctx에 결과(전역 bound, 고정, 동치 집계, 등식이 된 행, 함의 목록, lifting 정보)를 남긴다.
- 4d3a1d가 결과를 상태에 쓴다: 집계(`564ce0`, 레코드 type 2), bound 강화(정수는 ceil/floor, +0x3f8), 고정(`3df916`, 레코드 type 1), 행 등식화(+0x400), lifting(PRELIFTGROW: 행에 계수 추가·수정, `3bea24`, +0x3fc), 함의 표 갱신(`5b86ce` + `5b88ce`), 끝에 `5b9aad`.
- GUB probing(PREGUBPROBE=−1)은 기본값에서 꺼져 있다(§2.4). QC/Q 블록도 MIP 38개에서 안 돈다.
- 모델을 바꾸는 호출(38개 중): gmu-35-40(고정 10), gmu-35-50(고정 29), timtab1(집계 12 + bound 5), tr12-30(ub 20, 마무리에서 계수 1), ic97_potential(lb 2, 마무리에서 계수 98), glass4·graphdraw-domain·neos-4954672-berkel(마무리 lifting으로 nz +16/+66/+5). PRELIMIT sweep의 "바뀐 k"가 모두 이 호출과 맞는다(§4).

## 1. 호출 지점 (`3cd406`)

### 1.1 라운드 끝 (call 0x3b8c00, ret 0x3b8c05)

```
// 579e67 뒤, 5a04fc 앞 (m2_presolve.md §2 "라운드 후")
if probeMode != 0 && adf236(&T9, W) < 0.8 {          // ebp = 3cd406 param_2(probeMode), 0x3b8b91
    ade8ca(&T9, W)                                   // 타이머 시작
    rc = 4d3a1d(p1 = [rsp+0xa90], p2 = mipctx(3cd406 param_3), st, env, &T9,
                p6 = max(probeMode, 2), p7 = 0, p8 = 0, p9 = 0, p10 = 0)
    ade770(&T9, W)                                   // 타이머 정지
    if rc != 0 { 오류 경로(LAB_004bc28e) }
    k += 1; if k >= st+0x1dc { 마무리(LAB_004b2415) }
    3ba5ac(env, st, 0, 0, -1) != 0 → 오류;  3ba9de(env, st, &stop) != 0 → 오류;  stop이면 라운드 탈출
}
```
- T9 = `3cd406`의 reduction 타이머 9번(rsp+0x5c0. 타이머 배열은 rsp+0x380 + 0x40·i, i=0..27). T8 = rsp+0x580(마무리 호출), T10 = rsp+0x600. 한도는 `3cd406` 앞부분(0x3b1435–0x3b15c7, 다른 경로 0x3b4484–0x3b4641)이 모든 타이머에 같은 값으로 넣고, st+0xf8 ≠ 0 또는 probeMode ≥ 3이면 T8·T9·T10만 ×3, 0x3bdd47에서 st+0xf8 = 1로 만들 때 ×2 한다. 이 값의 식은 구간 노트(spine) 몫이다.
- probeMode(`3cd406` param_2) = `GRBpresolvemodel`에서 PREPROBE(0x2828, 기본 1). `3cd406` L887–925(0x3b12b1)가 n > 10000 등일 때 3으로 올린다. 38개 모델은 모두 1이다(p6 = 2로 확인).
- 라운드가 여러 번이면 라운드마다 부른다(gmu-35-40/50, enlight_hard 2번, timtab1 2번).
- PRELIMIT 카운터(ebx)는 probing이 실제로 불린 때만 +1이다. 가드에서 건너뛰면 세지 않는다.

### 1.2 마무리 단계 (call 0x3bcd40, ret 0x3bcd45)

asm 0x3bcb36–0x3bcd45(디컴파일 L6900–7050):
```
r12 = probeMode; lift = false
if probeMode == 1 && env+0x2728 == 0 && n > st+0xf4 {                     // st+0xf4: 정수 열 수(assign1-5-8 130 = 'B' 열 수)
    cont = #{ j < n : col_cnt[j] >= 0 && vtype[j] == 'C' }
    if (double)cont > (double)(n − st+0x408)·0.05 { lift = true(rsp+0xc50); r12 = 2 }
}
if probeMode <= 1 && !lift → 건너뜀
if adf236(&T8, W) >= 0.8 → 건너뜀
p9 = 0
if PRELIFTGROW(0x28a4) != 0 {
    if PREPROBE != 1 || Presolve(0x2424) != -1 || (mipctx != 0 && (mipctx+0x32a8 == 0 || *(mipctx+0x32a8)+4 <= 0)) { p9 = 1 }
    else if 목적 계수(st+0xb0) 중 0 아닌 것(NaN 포함)이 2개 이상 && (r12 >= 4 || lift) { p9 = 1 }
}
if p9 == 0 && probeMode <= 1 → 건너뜀
ade8ca(&T8, W); rc = 4d3a1d(p1, mipctx, st, env, &T8, p6 = max(r12, 2), p7 = 1, p8 = 1, p9, p10 = 1); ade770(&T8, W)
rc != 0 → 오류;  k += 1; k >= st+0x1dc → 마무리;  3ba5ac(env, st, 0, 0, -1)
```
- 기본값(probeMode 1)에서는 "살아 있는 연속 열이 (n − 제거 열)의 5%를 넘고 목적 계수가 2개 이상 0이 아닐 때"만 부른다. 그때 p6 = 2, p9 = 1이다. 38개 중 19개(§4.2).
- 목적 계수 검사는 첫 0 아닌 위치를 찾고 그 뒤에서 두 번째를 찾는다(SIMD 2개씩, 0x3bcbcc–0x3bccd7). `ucomisd`라 NaN도 0 아님이다.

### 1.3 반환값·예산·PRELIMIT

- 반환: 0, 3(불가능. st+0x470 = 열 번호), 0x2711(메모리), 그리고 `3ba5ac`/`3ba9de`/보조 함수의 0 아닌 값을 그대로. 정상 종료에서도 마지막에 `3ba5ac(env, st, 0, 0, -1)`을 한 번 더 부르고 그 값을 돌려준다(L5048).
- 4d3a1d 안에는 PRELIMIT 카운트가 없다. `3ba9de(env, st, 0)`가 전체 예산을 넘겨 st+0x1dc를 −1로 만들면(LP 노트 §0의 규칙) 그 뒤 판정(`st+0x1dc < 0`)에서 조용히 끝낸다(반환 0).
- 타이머 함수(`adf*`, `ade*`, 구조체 {t, wall, limit, …, +0x28 running, +0x2c wall 플래그}):
  - `ade8ca`(시작): running=1, t = (t − W) + 1.0. `ade770`(정지): running=0, t = t + W.
  - `adf00e`(경과) = t + (running ? W : 0). `adf1f2`(한도) = limit. `adf205` = limit ≠ 0.
  - `adf236`(비율) = limit<0이면 1, limit>0이면 경과/limit, 아니면 0. `adf2be` = 비율 ≥ 1.
- probing 자체의 작업 한도는 ctx+0xd8(§2.6)이다. `4e48f7`이 이것으로 후보를 중간에 끊는다(ctx+0xe8 = 1. bppc4-08, cod105, csched007/008, mad, neos-4338804-snowy, neos-860300, neos-911970, neos17, gmu 2번째 호출에서 일어남).

### 1.4 스레드

4d3a1d L2941:
```
if ncand < 50 || (double)ncand * (double)nnzP < 1e9 || ctx.budget < 2e9 { b = 1 }
else { b = min(ac7d4e(), 12); 후보를 b 덩어리로 나눔(L2955–2985, LAB_0055f05a) }
```
- nnzP = probing 행렬의 항목 수(local_4b8), ctx.budget = §2.6의 ctx+0xd8. 38개 모델은 첫 조건들에서 이미 b = 1이다. Threads=1이면 `ac7d4e()`도 1이라 두 이유로 단일 경로다. cov2와 이번 추적 모두 `ac761d`, `4e4875`(스레드 작업 함수), `ac7d4e` 호출 0회.
- b > 1 경로(포팅 안 함): ctx를 b개 복사(0x228 B, 버퍼는 새로), μ' = μ·b^0.33(`pow`), 스레드별 `4e48f7`, 끝에 bound는 max/min으로, 행 sense '='는 OR로, 집계는 앞 스레드 우선으로 합친다(L3555–3650).

## 2. `4d3a1d` 본체

### 2.1 인자와 처음 값 (L365–429)

`4d3a1d(p1 model, p2 mipctx, st, env, timer, p6 mode, p7 final, p8, p9 lift, p10)`.

| 지역 | 값 |
|---|---|
| local_130 | (double)(n+m)·4.0. 2e9 넘으면 바로 끝 |
| local_128 | (double)(n+m)·1000.0 + 1e7 (예산 기본) |
| local_118 | local_128·10.0 |
| local_108 / local_100 | (double)n·3.0 / (double)n·20.0 |
| local_168 | PREGUBPROBE(0x2830) |
| local_178 / local_180 | st+0x1f8(Q 수) / st+0x27c(QC 수). p6 < 3이면 local_180 = 0 |
| local_190 / local_198 | st+0x3b0 / st+0x3a8 (QC 블록용) |
| local_c0 | 0. 두 번째 패스 표시 |
| local_188 | 0. "더 돌지 마라" 표시(작업 한도) |

### 2.2 패스 루프

```
loop {                                                    // do { … } while(true), L429–5465
    rc = 3ba5ac(env, st, 0, 0, -1); rc==0이면 rc = 5b9aad(env, st, G = *(st+0x3f0), 0, NULL, work)   // 대기 함의 쌍을 conflict graph에 합침(§3.5)
    rc != 0 → 끝(rc)
    본체(§2.3–2.9); 끝에 반복 판정(§2.9.7)이 local_c0 = 1로 두면 다시, 아니면 local_c0 = 0
    정리: ctx·버퍼 해제(L5051–5460)
    if rc != 0 || local_c0 == 0 { return rc }            // 정상 끝은 rc = 3ba5ac(env, st, 0, 0, -1)
}
```
- 두 번째 패스는 상태(줄어든 행렬)로 처음부터 다시 만든다. 다른 점은 행 후보를 넣지 않는 것(§2.3.4)뿐이다. 패스는 최대 2번이다(두 번째 패스 끝에서는 local_c0 ≠ 0이라 반복 판정이 끝으로 보낸다).

### 2.3 probing 행, 열 개수, 후보 (L429–1514)

#### 2.3.1 할당
cand(int, n+m) = malloc((n+m)·4)(n+m ≥ 1일 때), rowmap(int, m), colcnt(int, n + 0).

#### 2.3.2 행 선택 (rowmap: 행 → probing 행 번호, 제외 −1)
```
rowmap[..] = -1
selective = p6 < 2 || (p6 == 2 && p7 == 0 && st+0xf8 != 0 && 3n < m && 20n < st+0xd8)
if !selective {                                   // 기본값(p6=2, st+0xf8=0)
    R = 0; for i in 0..m { if row_cnt[i] > 0 { rowmap[i] = R; R += 1 } }
    W += (m + m)·μ
} else {                                          // LAB_00558c94, 기본값에서 안 돎
    for i in 0..m: row_cnt[i] >= 1이고
        sense '=' → 넣음
        아니면 inf_min = st+0x78[i]:
          inf_min < 1: maxabs = max |a|(살아 있는 노드, 리스트 순서, `maxabs <= |a|`면 갱신), W += 3·(방문 노드)·μ
                       −act_min[i] < maxabs·5.0 이면 넣음        // act_min = st+0x98 = min activity − rhs
          inf_min = 1, 2 → 넣음
    W = 7·m·μ + W(n ≥ 1이면 저장)
}
```
R = local_260(probing 행 수). 행 번호 순서다.

#### 2.3.3 열 개수
- 기본 경로: colcnt[j] = max(col_cnt[j], 0). W += μ·(n + n).
- selective: colcnt[j] = 열 j의 살아 있는 노드 중 rowmap ≥ 0인 행 수, 열마다 W = μ·4.0·(방문)+W, 끝에 W += μ·(n+n).

#### 2.3.4 열 후보 (L736–795)
```
nnzP = 0; nnzB = 0; ncand = 0
for j in 0..n {
    c = colcnt[j]; if c <= 0 continue
    if ub[j] - lb[j] < 1e-10 {
        if ub[j] + ft < lb[j] { st+0x470 = j; return 3 }
        rc = 3de18e(xmm0 = lb[j], xmm1 = INF, env, st, j); rc != 0 → 끝      // 기본값 38개에서 0회
    } else {
        nnzP += c
        if vtype[j] == 'B' { cand[ncand] = j; ncand += 1; nnzB += c }
    }
}
W = (double)n·4.0·μ + W
rc = 3ba5ac(env, st, 0, 0, -1)
if ncand < 1 || 2e9 < (double)nnzP + (double)nnzP { 끝(local_c0 = 0) }
```
local_250 = nnzP, local_238 = nnzB, local_258 = ncand. 'I' 열(일반 정수)은 후보가 아니다.

#### 2.3.5 행 후보 (첫 패스만, L1483–1514, asm 0x461f86–0x46204d)
```
if local_c0 == 0 {
  for i in 0..m {
    k = rowmap[i]
    if k < 0 || sense[i] != '<' || row_cnt[i] < 2 || row_cnt[i] > 100 || inf_min[i] != 0 { continue }
    v = 0; ok = true
    for node in row i (리스트 순서) {
        if node.col >= 0 && (vtype[col] == 'C' || |a| + act_min[i] < -1e-10) { ok = false; break }
        v += 1                                        // 죽은 노드도 셈
    }
    W = (double)v·4.0·μ + W
    if ok { cand[ncand] = k + ncolP; ncand += 1 }     // ncolP = n (GUB 열 없음)
  }
}
```
"한 변수라도 최솟값에서 |a|만큼 움직이면 rhs에 닿는(또는 넘는)" 정수 `<` 행이다(집합 포장 꼴). 후보 번호 ≥ n이면 probing 행 번호 + n이다. cod105는 1024행이 모두 행 후보다(ncand = 2048).

### 2.4 GUB probing (기본값에서 안 돎, L808–1470)

```
if PREGUBPROBE == -1 { gub = (p6 > 2 || st+0xf8 != 0) }   // 기본값: p6 = 2, st+0xf8 = 0(38개 모두 확인) → 0
else gub = PREGUBPROBE != 0
```
gub이면 `451bed`(clique 표, st+0x390)로 GUB 행을 찾고 `456f1d`/`adf891`(작업 큐, `4cedf1`/`4cef05`)로 행마다 slack 열(local_1a0개, bound ±1e30, `4f0f39`로 계산)과 행(local_1a8)을 더한다. 38개 모델에서 `456f1d`·`4f0f39`·`adf891` 0회, 45938b의 `451bed` 0회. 그래서 아래에서 GUB 열 수 = 0, ncolP = n, nrowP = R로 둔다. 포팅하지 않는다.

### 2.5 probing 행렬 만들기 (L1476–2540)

```
nnzP2 = nnzP + 0 + 0 - (0 - 0); nnzP2 > 2e9 → 끝               // local_270
ctx = calloc(1, 0x228)  (필드 §2.6)
// 행 복사 (L1718–1760)
for i in 0..m (행 순서): k = rowmap[i]; k >= 0이면 rowsense[k] = sense[i], rowrhs[k] = rhs[i], prow[k] = i
W = ((double)m + (double)R·5.0)·μ + W
plb = lb, pub = ub (memcpy, n); pvt = vtype (memcpy, n); aggto[0..n) = -1
// 행 CSR (L1952–2225): 행마다 리스트 순서, 살아 있는 노드만
pos = 0
for k in 0..R {
    rbeg[k] = pos; nb = pos; v = 0
    for node in row prow[k] { if node.col >= 0 { if pvt[col] != 'B' { nb += 1 }; rcol[pos] = col; rval[pos] = a; pos += 1 }; v += 1 }
    W = (double)v·5.0·μ + W
    rnbend[k] = nb                                    // 비이진 항목 끝(정렬 뒤 [rbeg, rnbend)가 비이진)
}
W = (double)R·6.0·μ + W
3ba5ac;  rbeg[R] = pos;  3ba5ac
plb2 = plb, pub2 = pub (memcpy)
for k in 0..R { 396a2d(rbeg[k+1]-rbeg[k], &rcol[rbeg[k]], &rval[rbeg[k]], pvt, plb, pub, aggmul(임시), work) }   // §3.1, 행 정렬
W = W + (double)R·μ
3ba5ac
// 행 activity (L2340–2510): 정렬된 CSR 순서, -rhs에서 시작
for k in 0..R {
    mn = -rowrhs[k]; mx = -rowrhs[k]; imn = 0; imx = 0; rng = 0.0
    for p in rbeg[k]..rbeg[k+1] {
        a = rval[p]; l = plb[c]; u = pub[c]                // c = rcol[p]
        if a <= 0.0 {
            if u < 1e30 { mn = gdn(mn, a*u) } else { imn += 1 }
            if -1e30 < l { mx = gup(mx, a*l) } else { imx += 1 }
        } else {
            if u < 1e30 { mx = gup(mx, a*u) } else { imx += 1 }
            if -1e30 < l { mn = gdn(mn, a*l) } else { imn += 1 }
        }
        if rng < 1e100 { rng = if l <= -1e30 || 1e30 <= u { 1e100 } else { max(rng, |a|*(u-l)) /* rng <= d면 d */ } }
    }
    W = (double)len·4.0·μ + W
    if imn > 0 { mn = (double)(-1 - imn) * 2^400 }
    if imx > 0 { mx = (double)(imx + 1) * 2^400 }
    pmin[k] = mn; pmax[k] = mx; prange[k] = if rng < 1e100 { rng } else { 2.5·2^400 (0x5904000000000000) }
}
W = (double)max(R,0)·9.0·μ + W
3ba5ac
7886b(ncolP, nrowP, rbeg, rcol, rval, cbeg, crow, cval, work)        // 열 CSR, W에 μ 없음 (§3.5)
```
- `gdn`/`gup` 판정은 LP와 같다: |acc| < |t|면 (s − t ≠ acc), 아니면 (s − acc ≠ t)일 때 보정. min 쪽은 s ≥ 0이면 ×(1−2⁻⁵²), s < 0이면 ×(1+2⁻⁵²). max 쪽은 s > 0이면 ×(1+2⁻⁵²), s ≤ 0이면 ×(1−2⁻⁵²).
- 1e30은 INF 파라미터가 아니라 상수다(이 블록 전체).
- Q(`a85f71`), QC(`a86425`, `a7d68e`/`a7d7cf`/`a7d9ff`) 블록은 local_178 = local_180 = 0이라 안 돈다(0회 확인).

### 2.6 ctx 구조체 (0x228 B, calloc)

4d3a1d가 쓰는 필드와 `4e48f7`가 돌려주는 필드. `4e48f7` 쪽 의미는 §3(4e48f7 절)이 정본이다.

| 오프셋 | 내용 |
|---|---|
| +0x00/+0x08/+0x10 | p1(model) / st / env |
| +0x18 | μ (b > 1이면 μ·b^0.33) |
| +0x20 / +0x24 | nrowP(= R + GUB 행) / R |
| +0x28 / +0x2c | ncolP(= n + GUB 열) / n |
| +0x30 / +0x34 | QC 수(0) / 호출 때 st+0x408 |
| +0x38 / +0x40 / +0x48 | 열 CSR: cbeg(int, ncolP+1) / crow(int) / cval(double) — `7886b` |
| +0x50 / +0x58 | rbeg(int, nrowP+1) / rnbend(int, nrowP) |
| +0x60 / +0x68 | rcol(int) / rval(double) |
| +0x70 / +0x78 | GUB 보조(NULL) |
| +0x80 / +0x88 / +0x90 | rowrhs(double) / prange(double) / QC(NULL) |
| +0x98 | pvt(char, ncolP) |
| +0xa0–+0xc8 | Q/QC(NULL) |
| +0xd0 | itol(env+0x2468) |
| +0xd8 | 작업 예산(아래) |
| +0xe0 | 1.0 |
| +0xe8 | 출력: 작업 한도로 중간에 멈춤(§3.4 H: 후보 i마다 경과 > f·budget, f = clamp(((nbin·20+100) + nagg·10 + nchg)/(i+100), 0.01, 100)) |
| +0xf0 | p7 |
| +0xf4 / +0xf8 / +0xfc | 출력: 공통 암시로 바뀐 bound 수(lb·ub 따로, 열 < n) / 그중 이진 열 / '='가 된 행 수 |
| +0x100 / +0x108 | plb / pub (입력 복사본, 출력은 강화된 bound) |
| +0x110 / +0x118 | plb2 / pub2 (복사본) |
| +0x120 | rowsense(char, nrowP) |
| +0x128 / +0x130 | pmin / pmax (min/max activity − rhs) |
| +0x138 | GUB(NULL) |
| +0x140 | 출력: 집계 수 |
| +0x148 / +0x150 / +0x158 | aggto(int, −1) / aggmul / aggoff (double). aggmul은 `396a2d`의 임시 버퍼로도 쓴다 |
| +0x160 / +0x168 | liftcnt0 / liftcnt1 (int, calloc, ncolP): 후보 x의 x=0 / x=1 시나리오에서 바뀐 열 수(두 시나리오가 다 가능할 때만) |
| +0x170–+0x1a8 | QC(NULL) |
| +0x1b0 | lifting 정보 모음: env+0x2728==0 && p8 && (p10 || st+0x3c8 ≠ 0) |
| +0x1b4 / +0x1b8 | 출력: 중복 행 기록 용량 / 수(local_668, 최대 1e6) |
| +0x1c0–+0x1d8 | 출력: 중복 행 기록 head(int[R], −1) / next / lit / slack(f64) (`4e48f7`이 할당, §3.4 H REC_REDUNDANT) |
| +0x1e0 | 2e9 (b > 1이면 2e9/b) |
| +0x1e4 | 함의 모음: env+0x2728 == 0 (기본 1) |
| +0x1e8 / +0x1ec | 출력: 함의 묶음 수(= 처리한 후보 수) / 함의 목록 길이 |
| +0x1f0 | 함의 목록 용량 = 2·nnzB + 1000 (2e9 넘으면 2e9) |
| +0x1f8 / +0x200 | 함의 offsets(int, 2·ncand+1) / 함의 literal(int, 용량) |
| +0x208 | `4eb08d`의 보류 행 목록(int, 2·nrowP + Q + QC) |
| +0x210 / +0x218 / +0x220 | IntSet 큐 A / 큐 B / literal 집합 C(`4e48f7`이 만들고 해제) |

예산(+0xd8, L1515–1545):
```
base = if st+0xf8 == 0 { local_128 } else { local_118 }       // (n+m)·1000+1e7, ×10
budget = if p6 >= 4 || (Presolve == 2 && p6 == 3) { base·1000.0 }
         else if Presolve != 2 && p7 == 0 && p6 != 3 { base }
         else { base·10.0 }
// 4e48f7 직전 (L3002–3011)
if adf205(timer) { budget = min(budget, adf1f2(timer) − adf00e(timer, W)) }   // (한도 − 경과) <= budget이면 그것
```
기본값: 첫 지점 budget = (n+m)·1000 + 1e7(타이머에 안 걸림, 38개 확인), 마무리 = min(10배, 한도 − 경과)(예: assign1-5-8 19,200,295).

### 2.7 `4e48f7` 호출 (L2995–3063)

```
rc = 3ba5ac(env, st, 0, 0, -1); rc → 끝
rc = 3ba9de(env, st, 0); rc != 0 || st+0x1dc < 0 || adf2be(timer, W) → 끝(local_c0 = 0)
budget 갱신(위)
ctx+0x1e0 = 2e9
if ctx+0x1e4 { ctx+0x1f8 = malloc((2·ncand+1)·4); ctx+0x200 = malloc(cap·4); ctx+0x1f0 = cap; ctx+0x208 = malloc((2·nrowP+Q+QC)·4) }
rc = 4e48f7(st, ctx, cand, ncand, work); rc != 0 → 끝(rc)
nlift = ctx+0x1b8
if ctx+0xe8 != 0 || adf2be(timer, W) { local_188 = 1 }
```

### 2.8 후처리 (L3653–4935)

`3ba5ac(env, st, 0)` 뒤 차례로:

#### 2.8.1 집계 (ctx+0x140 ≠ 0)
```
for j in 0..n { if pub[j] - plb[j] < 1e-10 { aggto[j] = -1 } }     // probing으로 고정된 열은 집계하지 않음
W = ((double)n + (double)n)·μ + W
rc = 564ce0(st, env, aggto, aggmul, aggoff, 1, &tmp)     // §3.5. x_j = aggoff + aggmul·x_k, 레코드 type 2
```
timtab1: 12열(레코드 type 2 12개, 열 −12).

#### 2.8.2 bound 강화와 고정 (ctx+0xf4 > 0, asm 0x45f64c–0x45f84e)
```
for j in 0..n {
    if col_cnt[j] < 0 { continue }
    if plb[j] > lb[j] + 1e-6 {
        lb[j] = if vtype[j] != 'C' { ceil(plb[j] - itol) } else { plb[j] }
        5379f1(tracker, j); st+0x3f8 += 1
    }
    if pub[j] < ub[j] - 1e-6 {
        ub[j] = if vtype[j] != 'C' { floor(pub[j] + itol) } else { pub[j] }
        5379f1(tracker, j); st+0x3f8 += 1
    }
    if lb[j] > ub[j] + 1e-6 { st+0x470 = j; return 3 }
    if lb[j] > ub[j] - 1e-10 { rc = 3df916(xmm0 = ft, env, st, j); rc → 끝 }   // 고정·열 제거, 레코드 type 1
}
W = (double)n·5.0·μ + W
370047(st, env, 0, NULL)          // 모든 행 activity 다시 계산(LP 포트 agg_activity와 같은 함수)
```
- activity는 bound를 바꾸는 동안 갱신하지 않고 끝에 한꺼번에 다시 계산한다.
- `3df916`은 이번에 바뀌지 않았어도 lb > ub − 1e-10인 살아 있는 열이면 부른다.
- gmu-35-40: ub 10개 → 고정 10(레코드 type 1 10개, +0x3f8 +10, +0x408 +10). tr12-30: ub 20(ctx+0xf4 = 32 중 20개만 1e-6을 넘음). ic97_potential: lb 2. timtab1: lb 1 + ub 4.

#### 2.8.3 등식화 (ctx+0xfc > 0)
```
for i in 0..m { if sense[i] != '=' && rowmap[i] >= 0 && rowsense[rowmap[i]] == '=' { sense[i] = '='; st+0x400 += 1; 53763f(tracker, i) } }
```
작업량 없음. 38개 모델에서 ctx+0xfc = 0이었다.

#### 2.8.4 계수 lifting (nlift = ctx+0x1b8 > 0. 새 nonzero는 p9(마무리 가드의 PRELIFTGROW 판정)일 때만. L3812–4580)

`4e48f7`은 ctx+0x1b0 = 1일 때(기본값에서 마무리 호출)만 "중복 행 기록"을 만든다(§3.4 H REC_REDUNDANT). 이진 후보 x의 시나리오가 가능했고 그 시나리오에서 `<` 행 q의 max activity − rhs < −0.001이면 {lit, v = rhs − max activity} 하나를 행 q 목록에 붙인다. lit = x(x=0 시나리오) 또는 −1−x(x=1 시나리오)다. 그래서 x=1 시나리오의 v는 계수에 +v(행 q에 v·x를 더함), x=0 시나리오의 v는 계수 −v·rhs −v(v·(1−x)를 더함)로 쓰인다. 4d3a1d가 행마다 하나(literal 하나 또는 clique 하나)를 골라 계수를 키운다(계수 lifting).

```
fillmax = if p9 == 0 { 0 } else { t = 2e9 − (int)nnzP; if p6 < 3 { min(t, nnzP) } else { t } }   // 새 nonzero 한도
lv = calloc(2n, f64); litlist = malloc(2n·4); nodeof = calloc(n, ptr)
W0 = W; nfill = 0; nmod = 0; ndel = 0; ncl = -1(아직 451bed 안 부름)
for k in 0..R {
    i = prow[k]
    if W − W0 > 1e7 {                                            // 주기 확인
        rc = 3ba5ac(env, st, 0, 0, -1); rc==0이면 rc = 3ba9de(env, st, 0)
        if rc != 0 || st+0x1dc < 0 { break(rc) }; W0 = W
    }
    if row_cnt[i] < 1 || sense[i] == '=' { continue }
    // 1) literal 모으기
    nl = 0; cnt = 0
    for e in list(k) {                                           // head[k], next[e]
        q = lit[e]; v = val[e]
        (L, c) = if q < 0 { (-1-q, -1-q) } else { (n+q, q) }      // L < n: x_c, L >= n: 1−x_c
        if col_cnt[c] > 0 { if lv[L] == 0.0 { litlist[nl] = L; nl += 1 }; lv[L] = if lv[L] <= v { v } else { lv[L] } }
        cnt += 1
    }
    W = (W + (double)cnt·5.0·μ) + ((double)1 + (double)1)·μ      // 스레드 1개
    if nl < 1 { continue }
    // 2) 행 노드 표
    t = 0; for node in row i { if node.col >= 0 { nodeof[node.col] = node }; t += 1 }
    W = ((double)t + (double)t)·μ + W
    // 3) clique 표 (패스마다 한 번)
    if ncl == -1 { rc = 451bed(env, st, m, 0, 1, &o1, &o2, *(st+0x390), 0, &ncl, &csense, 0, &cbeg, &cmem, &jbeg, &jcl); rc → break }
       // 출력: ncl clique, clique c의 원소 cmem[cbeg[c]..cbeg[c+1]](≥0: x_j, <0: 1−x_{−1−j}), csense(char),
       //       열 j가 든 clique 번호 jcl[jbeg[j]..jbeg[j+1]](≥0 / <0 = −1−c) (glass4 Frida 덤프로 확인)
    if ncl > 0 && clsc == NULL { clsc = calloc(ncl, f64); cllist = malloc(ncl·4) }
    // 4) 점수
    best = 0.0; pick = -1; ncll = 0
    for t in 0..nl {
        L = litlist[t]; c = L mod n
        v = if nfill < fillmax { lv[L] } else if nodeof[c] == NULL { lv[L] = 0.0; 0.0 } else { lv[L] }
        s = 0.0
        if v != 0.0 {
            cc = if L < n { liftcnt0[L] } else { liftcnt1[L-n] }          // ctx+0x160 / +0x168. +v 쪽(x=1 기록)이 x=0 시나리오 개수를 쓴다
            // st+0x3c8(LP 해) == NULL일 때(presolve는 항상 NULL. 38개 확인), asm 0x4610c5
            s = ((0.1·v)/(v + 100.0) + 1.0) · ((double)cc / (100.0 + (double)cc))
        }
        if s == 0.0 { lv[L] = 0.0; continue }
        if best < s { best = s; pick = L }
        if clsc != NULL {
            b = jbeg[c]; e = min(jbeg[c+1], b + 1000)
            for p in b..e { q = jcl[p]; if q >= 0 { if clsc[q] == 0.0 { cllist[ncll] = q; ncll += 1 }; clsc[q] = clsc[q] + s } }   // 주의 1
            W = (double)(e − b)·3.0·μ + W
        }
    }
    W = W + (double)nl·5.0·μ
    for t in 0..ncll { q = cllist[t]; s = clsc[q]; clsc[q] = 0.0; if best < s { best = s; pick = q + 2n } }
    W = W + ((double)ncll + (double)ncll)·μ
    // 5) 적용
    if pick >= 0 {
        mem = if pick < 2n { [pick] } else { c = pick − 2n; cmem[cbeg[c]..cbeg[c+1]]를 L로: x ≥ 0 → x, x < 0 → n + (−1−x) }
        for L in mem {
            c = L mod n; v = lv[L]; if v == 0.0 { continue }
            5379f1(tracker, c)
            nd = nodeof[c]
            if nd == NULL { nfill += 1; rc = 3bea24(xmm0 = 0.0, env, st, i, c, &nd); rc → 끝; nodeof[c] = nd }   // 새 노드(값 0)
            else { nmod += 1 }
            if L < n { nd.val = v + nd.val } else { nd.val = nd.val − v; rhs[nd.row] = rhs[nd.row] − v }
            if |nd.val| < 1e-10 { nmod −= 1; nodeof[nd.col] = NULL; ndel += 1; row_cnt[nd.row] −= 1; col_cnt[nd.col] −= 1; nd.col = −1 }
        }
        W = (double)|mem|·3.0·μ + W
        370047(st, env, 1, &i)            // 이 행 activity
        53763f(tracker, i)
    }
    // 6) 정리
    for t in 0..nl { lv[litlist[t]] = 0.0 };  W = ((double)nl + (double)nl)·μ + W
    t = 0; for node in row i { if node.col >= 0 { nodeof[node.col] = NULL }; t += 1 };  W = ((double)t + (double)t)·μ + W
}
st+0x3fc += nmod + nfill + ndel
3ba5ac(env, st, 0, 0, -1)
```
- 주의 1: 원본은 `(L >= 0) == (q >= 0)`을 비교한다(asm 0x4606cb–0x4606e2). L은 늘 ≥ 0이라 q ≥ 0(양의 소속)만 더한다. 그대로 옮길 것.
- 4)의 5번 곱 W 항은 literal 루프가 끝난 뒤 한 번이다(`W + (double)(마지막 t+1)·5.0·μ`, L4380).
- 첫 호출(p9 = 0)에서는 fillmax = 0이라 이미 있는 계수만 바꿀 수 있다. 기본값에서 첫 호출은 ctx+0x1b0 = 0이라 목록 자체가 없다(nlift = 0, 38개 확인).
- clique 원소의 lv가 0이면 그 원소는 건너뛴다. clique를 골라도 lv가 0이 아닌 원소만 바뀐다.
- glass4(마무리): nlift 16, `451bed` 1회, `3bea24` 16회, +0x3fc +16, nz 1799 → 1815. graphdraw-domain 66/66, neos-4954672-berkel 5/5, ic97_potential 새 97 + 수정 1(rhs 35개), tr12-30 수정 1(rhs 1, `3bea24` 0회).
- **Frida 재현**: 4e48f7 복귀 때의 lifting 목록(head/next/lit/val, liftcnt0/1)과 `451bed` 출력을 덤프해 위 의사코드를 Python으로 돌렸다(`simlift.py`). glass4(literal 4 + clique 6), graphdraw-domain(literal 32 + clique 17), ic97_potential, neos-4954672-berkel, tr12-30의 마무리 호출에서 바뀐 행 계수·rhs·+0x3fc·`3bea24` 호출 순서가 복귀 덤프와 모두 같다.
- `mipctx`(p2) ≠ 0이고 mipctx+0x32a8에 SOS류 표가 있을 때의 local_748 블록(L3866–3905, L4535–4555)은 presolve에서 p2 = 0이라 안 돈다.

#### 2.8.5 함의 표 갱신 (ctx+0x1e4 ≠ 0, L4699–4930)
```
if 5b86ce(env, st, G) { 끝(local_c0 = 0) }               // conflict graph가 한도를 넘으면 함의도 끝 5b9aad도 없이 끝
nadd = 0
for t in 0..ctx+0x1e8 {                                   // 처리된 후보만
    j = cand[t]
    if j < n && col_cnt[j] >= 0 && lb[j] <= 0.5 && ub[j] >= 0.5 {
        (s0, s1, s2) = (off[2t], off[2t+1], off[2t+2])     // ctx+0x1f8
        for p in s0..s1 {                                  // x_j = 0일 때 참인 literal
            q = lits[p]; c = if q >= 0 { q } else { -1-q }
            if col_cnt[c] >= 0 && lb[c] < 0.5 && ub[c] > 0.5 {
                rc = 5b88ce(env, st, *(st+0x3f0), -1-j, q, 0, &full, work); rc → 끝
                if full { W = (double)(p−s0)·3.0·μ + W; goto done }
                nadd += 1
            }
        }
        W = (double)(s1−s0)·3.0·μ + W
        for p in s1..s2 { 같은 검사; 5b88ce(env, st, *(st+0x3f0), j, q, 0, &full, work); full이면 W += (p−s1)·3·μ; goto done; nadd += 1 }
        W = (double)(s2−s1)·3.0·μ + W
    }
}
done: W = (double)t·4.0·μ + W                              // t = 끝낸(또는 멈춘) 후보 위치
rc = 5b9aad(env, st, *(st+0x3f0), 0, &tot, work); rc → 끝
```
- 후보 j 쪽은 lb ≤ 0.5 ≤ ub(닫힌), 함의 쪽 열은 lb < 0.5 < ub(열린) 판정이다(asm 같음).
- 묶음 t는 cand[t]의 것이다. cand는 첫 호출에서 `4e48f7`이 점수순으로 제자리 정렬한 순서다(§3.4 G). 행 후보와 건너뛴 후보도 빈 묶음을 가진다.
- literal 인코딩(conflict graph와 같음): j = x_j, −1−j = ¬x_j. lits[s0..s1)은 "x_j = 0이면 거짓이 되는 literal"(x_k = 1이 함의되면 ¬x_k = −1−k, x_k = 0이면 k), [s1..s2)는 "x_j = 1이면"이다(§3.4 H RECORD_IMPLICATIONS). 간선 (¬x_j, e) / (x_j, e)는 "둘이 함께 참일 수 없다"다(§3.5).
- gmu-35-40: `4e48f7` 암시 502개 중 489개만 `5b88ce`로 간다. 나머지는 고정 뒤 열이 죽었거나 bound가 0.5를 넘지 않는다.
- `5b88ce` 호출: fhnw-binpack4-4 4112, glass4 312, gmu-35-40 489, gmu-35-50 1093, ic97_potential 1460(2패스), neos-2657525-crna 3302, neos-4338804-snowy 3692, neos-860300 1826, sp150x300d 6, timtab1 40, tr12-30 8. 모두 첫 지점 호출이다. 함의 표는 상태 덤프에 없는 숨은 상태다(`5b88ce` 절).

#### 2.8.6 반복 판정
```
if tot − nadd <= ncand / 4 || local_188 != 0 || local_c0 != 0 { local_c0 = 0; 끝 } else { local_c0 = 1; 다음 패스 }
```
ncand/4는 0 쪽 버림(`(x + ((x>>1)>>30)) >> 2`). tot = `5b9aad`의 `*out` = 2·(합친 쌍) − (중복 간선)(§3.5)이라 tot − nadd ≈ 새로 생긴(중복 아닌) 간선 수다. Frida: ic97_potential 첫 호출 패스 1은 tot 1656, nadd 1454 → 202 > 449/4 = 112라 패스 2(tot 10, nadd 6)로 갔다. gmu-35-40은 554 − 489 = 65 ≤ 228이라 1패스다.

### 2.9 작업량 확인 (assign1-5-8, gmu-35-40, cod105, timtab1, glass4)

보조 함수마다 진입·복귀의 W를 Frida로 재서, 그 사이(4d3a1d 자신)의 증가를 위 식과 비교했다. 모두 정확히 같다.

| 구간 | 식 | assign1-5-8 (n 156, m 161, R 161) | gmu-35-40 (n 1205, m 424) | cod105 (n=m=1024) |
|---|---|---|---|---|
| 5b9aad 뒤 → 3ba5ac | 2m + 2n + 4n | 1258 | 8078 | 8192 |
| 행 후보 + 행 복사 + 행 CSR | 4·(행 후보 방문) + (m + 5R) + (5·방문 + 6R) | 18857(행 후보 0) | — | 528384(= 4·57344 + 6144 + 292864) |
| 396a2d 뒤 | R | 161 | 357 | 1024 |
| 행 activity | 4·Σlen + 9R | 14989 | 20137 | 238592 |
| bound 루프 | 5n (3df916 사이 포함) | — | 6025 | — |
| 함의 | 3·(literal 범위) + 4·(처리 후보) | 520(130 후보) | 범위마다 3·len, 끝 3672 = 24 + 4·912 | 192 = 4·48 |
| 집계 앞 / 집계 뒤 bound 루프 | 2n / 5n | — | — | (timtab1, n 397: 794 / 1985) |

`396a2d`와 `7886b` 안의 W는 μ를 곱하지 않는다(assign1-5-8 40100 / 24339). glass4 마무리 lifting은 행마다 `3bea24`–`370047` 사이 3·(원소 수)가 맞았다.

## 3. 보조 함수

### 3.1 `396a2d(len, cols, vals, vtype, lb, ub, buf, work)` — 행 정렬 (1,821 B)

probing 행 하나를 [비이진 | 이진]으로 나누고 각 부분을 정렬한다. W 증가는 μ 없이 work에 바로 더한다.
```
nb = 0
for i in 0..len { if vtype[cols[i]] != 'B' { swap(cols[i], cols[nb]); swap(vals[i], vals[nb]); nb += 1 } }   // 앞으로 모으기(안정 아님)
W += (double)len·3.0
for i in nb..len { if vals[i] < 0 { cols[i] = -1 - cols[i]; vals[i] = -vals[i] } }     W = W + (double)(len−nb)·2.0
aabbe3(len − nb, &vals[nb], &cols[nb])                     // |a| 내림차순(LP 포트 common.rs sort_desc)
W += (double)(len−nb)·5.0
for i in nb..len { if cols[i] < 0 { cols[i] = -1 - cols[i]; vals[i] = -vals[i](부호 비트 뒤집기) } }   W = W + (double)(len−nb)·2.0
for i in 0..nb { buf[i] = if lb[c] <= -1e30 || 1e30 <= ub[c] { 1e100 } else { |vals[i]|·(ub[c] − lb[c]) } }
W = W + (double)nb·5.0
aae2ff(nb, buf, vals, cols)                                 // buf 내림차순, vals·cols 함께 이동
```
- 이진 부분은 부호를 떼고 |a|로 정렬한 뒤 부호를 되돌린다. 같은 |a|의 순서는 `aabbe3` 알고리즘이 정한다.
- `aae2ff`는 `aabbe3`과 같은 구조(100 이상 3-way quicksort, 깊이 50, 그 밑 Shell 간격 n/2, ÷2.2)에 payload가 둘(f64, i32)이다. Shell 부분은 디컴파일로 같음을 봤고, quick 부분은 같은 꼴로 보았다(비트 단위 대조는 안 함).
- W 식의 각 조각(len·3, (len−nb)·2 …)은 len, nb가 0일 때 해당 덧셈 자체를 건너뛰는 분기가 있다(L205–335). 0을 더하는 것과 결과가 같다.

### 3.2 `370047(st, env, len, list)` — activity 재계산

LP 포트 `agg.rs::agg_activity`(m1_presolve_agg.md §1.3)와 같은 함수다. list가 NULL이면 0..m 전부다. 행마다 7·(방문 노드)·μ, 끝에 8·(처리 행 수)·μ.

### 3.3 타이머·정렬·추적기
- 타이머: §1.3.
- `5379f1(tr, j)` / `53763f(tr, i)`: 열·행 표시(LP 포트 tracker.rs).
- `aabbe3`: LP 포트 common.rs.

### 3.4 probing 엔진 `4e48f7`과 그 보조 함수 (`4e28ef`, `4e7515`, `4e7b3e`, `a7d0ab`, `4eba40`, `4ed528`, `4eb08d`, IntSet `ac3c8b`…`ac4168`)

`4d3a1d`가 만든 probe 문맥(ctx, `calloc(1, 0x228)`)과 후보 목록을 받아 이진 변수를 하나씩 0/1로 시험 고정하고 전파한다. 결과는 **ctx 안에만** 쓴다(state는 바꾸지 않는다. 예외는 1e7 작업마다 `st+0x128 = 0`). state 반영은 `4d3a1d` 몫이다.

**검증 상태**: 아래 내용을 Python으로 다시 짠 시뮬레이터(세션 스크래치 `probe/py/sim48.py`)에 `4e48f7` 진입 시점의 ctx·state 덤프(`probe/js/x_4e48f7.js`)를 넣어 돌렸다. 기본값 35개 모델의 `4e48f7` 호출 58회 전부에서 반환 시점의 ctx lb/ub, activity, sense, ctx[0x29..0x2d], 카운터, 암시 목록(ctx[0x3f]/[0x40]), 중복 행 기록(ctx[0x37..0x3b]), 정렬된 후보 순서, 그리고 **작업량 `*work`가 비트 단위로 같다**(§검증). "검증 안 됨"이라고 쓴 경로는 기본값 38개에서 한 번도 지나가지 않아 asm만 보고 적은 것이다.

#### A. 서명, 반환값, 호출 지점

```
4e48f7(st: *State (rdi), ctx: *Ctx (rsi), cand: *mut i32 (rdx), ncand: i32 (ecx), work: *mut f64 (r8)) -> i32
```
- 호출: `4d3a1d` asm 0x461b97(decomp L3052) `4e48f7(local_38=st, puVar19=ctx, puVar15=cand, local_258=ncand, local_f0=work)`. `work`는 state+0x458이다.
- 스레드판 0x45ee6c는 `ac761d`가 부르는 `4e4875(args)` → `4e48f7(args[0], args[1], args[2], (i32)args[3], args[8])`, 결과를 args+0x4c에, args+0x48 = 1. Threads=1에서는 `local_b8 = ac7d4e() == 1`이라 0x461b97 쪽만 탄다(cov2: `ac761d` 0회, `4e48f7` 58회 전부 0x461b9c 복귀).
- 반환값:
  - 0: 정상, 예산 소진으로 멈춤(ctx+0xe8 = 1), **x=0과 x=1이 둘 다 모순인 후보가 나와 멈춤**(반환값이 아니라 ctx lb[x] = 1, ub[x] = 0으로 알린다. 그 자리에서 곧바로 끝나며 끝맺음 작업량도 더하지 않는다), `st+0x1dc < 0`(1e7 점검에서 PRELIMIT/예산이 끊김).
  - 0x2711: 메모리.
  - `3ba5ac`/`3ba9de`의 0 아닌 코드(1e7 작업마다 점검).
  - `a83b2f`(+0x100 객체) 코드: ctx[1] == NULL이거나 ctx+0xec ≠ 0일 때만. 기본값에서는 안 탄다.
- 후보 `cand[i]`: 값 < nc(=ctx+0x28)면 열 j, ≥ nc면 행 r = 값 − nc(=probe 행 번호). **정렬은 제자리**(cand 배열이 바뀐 채로 돌아간다).

#### B. ctx 필드 (4e48f7 기준)

`4d3a1d` L1515–2540에서 채운다. 기본값 38개에서 늘 nr = nrow0(=GUB 행 없음), nc = n(=GUB 열 없음), k6 = 0, st+0x1f8 = 0(SOS 없음), ctx+0x1e4 = 1, env+0x2cd0 = 0, st+0x490 = 1e100, st+0x250/+0x330/+0x3c0 = 0이었다.

| 오프셋 | 형 | 내용 | 4e48f7 |
|---|---|---|---|
| +0x00 | ptr | `4d3a1d` param_1(모델 객체). +0x100을 `a83b2f`에 넘김 | 읽기(기본값 안 씀) |
| +0x08 | ptr | st | 읽기 |
| +0x10 | ptr | env | 읽기 |
| +0x18 | f64 | μ = st+0x448 | 읽기 |
| +0x20 | i32 | nr: probe 행 수(= 선택된 원래 행 + GUB 행) | 읽기 |
| +0x24 | i32 | nrow0: 선택된 원래 행 수 | 읽기 |
| +0x28 | i32 | nc: probe 열 수(= n + GUB 열) | 읽기 |
| +0x2c | i32 | n | 읽기 |
| +0x30 | i32 | k6: QC 수(p6 ≥ 3일 때 st+0x27c, 아니면 0) | 읽기 |
| +0x38 / +0x40 / +0x48 | ptr | 열 CSR: 시작(nc+1) / 행 번호 / 값. `7886b`가 행 CSR에서 만든다 | 읽기 |
| +0x50 / +0x58 / +0x60 / +0x68 | ptr | 행 CSR: 시작(nr+1) / 이진 구간 시작 / 열 번호 / 값 | 읽기 |
| +0x80 | f64[nr] | rhs | 읽기 |
| +0x88 | f64[nrow0+k6] | 행 maxrange = max\|a\|·(ub−lb), 무한이면 2.5·BIG(=0x5904000000000000) | 읽기 |
| +0x98 | u8[nc] | vtype('B','I','C') | 읽기 |
| +0xa0 / +0xa8 | ptr | 열 → SOS 목록(st+0x1f8 > 0일 때) | 기본값 NULL |
| +0xd0 | f64 | IntFeasTol(env+0x2468, 1e-5) | 읽기 |
| +0xd8 | f64 | 예산 B. 첫 호출 (n+m)·1000+1e7, 마무리 호출은 다른 식(4d3a1d 몫) | 읽기 |
| +0xe0 | f64 | 예산 계수 f. 진입 1.0, 후보마다 갱신 | **쓰기** |
| +0xe8 | i32 | 예산 소진으로 멈춤 = 1 | **쓰기** |
| +0xec | i32 | 1이면 1e7 점검에 `a83b2f` 사용. 0 | 읽기 |
| +0xf0 | i32 | p7(마무리 호출 표시) | 읽기 |
| +0xf4 | i32 | 공통 암시로 바뀐 bound 수(열 < n, lb·ub 따로 셈). 진입 시 0 | **쓰기** |
| +0xf8 | i32 | 그중 이진 열의 수. 진입 시 0 | **쓰기** |
| +0xfc | i32 | `<`→`=`로 바꾼 행 수. 진입 시 0 | **쓰기** |
| +0x100 / +0x108 | f64[nc] | 전역 lb / ub 사본 | **쓰기**(고정·강화) |
| +0x110 / +0x118 | f64[nc] | 원래 lb / ub(`4ed528`의 조기 종료 판정) | 읽기 |
| +0x120 | u8[nr] | sense 사본 | **쓰기**(`=`) |
| +0x128 / +0x130 | f64[nrow0+k6] | actmin / actmax = (min/max activity − rhs), §C.3 부호화 | **쓰기**(공통 암시 반영) |
| +0x138 | i32[2(nr−nrow0)] | GUB 행 보조 | 기본값 NULL |
| +0x140 | i32 | 집계 수 | **쓰기** |
| +0x148 | i32[nc] | 집계 상대 x(없으면 −1). 4d3a1d가 −1로 채운다 | **쓰기** |
| +0x150 / +0x158 | f64[nc] | 집계 x_j = c2a·x + c2b의 c2a / c2b | **쓰기** |
| +0x160 / +0x168 | i32[nc] | 두 시나리오가 다 가능했던 후보 x의 바뀐 열 수(x=0 / x=1) | **쓰기** |
| +0x1b0 | i32 | 중복 행 기록 켬. env+0x2728 == 0 && p8 && (p10 \|\| st+0x3c8) → 마무리 호출에서 1 | 읽기 |
| +0x1b4 / +0x1b8 | i32 | 중복 행 기록 용량 / 개수 | **쓰기** |
| +0x1c0..+0x1d8 | ptr | 중복 행 기록: 행 머리 i32[nrow0](−1), 다음 i32[], 리터럴 i32[], 여유 f64[] | **할당·쓰기** |
| +0x1e0 | i32 | 암시 목록 용량 상한 2e9 | 읽기 |
| +0x1e4 | i32 | 암시 모드(env+0x2728 == 0이면 1). 용량을 못 늘리면 0으로 끈다 | 읽기·쓰기 |
| +0x1e8 | i32 | 암시 묶음 수(=처리한 후보 수) | **쓰기** |
| +0x1ec | i32 | 암시 목록 길이 | **쓰기** |
| +0x1f0 | i32 | 암시 목록 용량 | 쓰기 |
| +0x1f8 | i32[2·ncand+1] | 묶음 경계 | **쓰기** |
| +0x200 | i32[cap] | 암시 리터럴 | **쓰기**(realloc) |
| +0x208 | i32[] | `4eb08d`의 보류 행 목록 | 작업 공간 |
| +0x210 / +0x218 / +0x220 | IntSet* | 큐 A / 큐 B / 리터럴 집합 C | 4e48f7이 만들고 끝에 해제 |

#### C. 공통 부품

##### C.1 정수 집합 `ac3c8b` 계열 (IntSet)
`{i32 cnt; i32 lo; i32 hi; i32 items[] @+0x10; i32 pos[] @+0x18}`, pos는 [lo, hi) 위치표다. −1 = 없음, −2 = 막힘, ≥0 = items 안 위치. 스택(LIFO)이다.
- `ac3c8b(env, lo, hi, &set, work)`: 만든다. pos 전부 −1. `*work += (hi − lo)`(**μ 없음**).
- `ac3f7f(set, x)`: pos[x] == −1일 때만 push. 막힌(−2) 원소는 넣지 않는다.
- `ac4062(set)`: pop(마지막 원소, pos = −1). 비었으면 −2e9.
- `ac4040(set)`: cnt. `ac4326(set, x)`: pos[x] ≥ 0. `ac40d4(set, x)`: pos == −1이면 −2. `ac4168(set, x)`: pos == −2면 −1.
- `ac426e(set, work)`: 전부 pop. `*work = (k + k) + *work`(k = 뺀 개수, **μ 없음**). `ac3eef`: 해제.
- 쓰임: 큐 A·B는 [−nr, nr + st+0x1f8 + k6), 원소 r ≥ 0은 행 r의 "≤ 쪽", −1−r은 `=` 행의 "≥ 쪽". A는 꽉 찬(여유 ≥ −1e−10) 행, B는 나머지다. 늘 A를 먼저 꺼낸다. 집합 C는 [−n, n), 리터럴 j = "x_j → 0", −1−j = "x_j → 1".

##### C.2 작업 타이머 `ade736`/`adf00e`
`ade736(T, work, 0)`: T[0] = (0.0 − *work) + 1.0, T+0x28 = 1. `adf00e(T, work)` = T[0] + *work. 그래서 경과 t = ((0 − w₀) + 1) + w(이 순서 그대로)다.

##### C.3 activity 부호화와 반올림
- actmin[r] = min activity − rhs, actmax[r] = max activity − rhs. 무한 기여가 k ≥ 1개면 actmin = −(k+1)·BIG, actmax = (k+1)·BIG(BIG = 2.5822498780869086e120 = 0x58f0000000000000)이고 유한 부분은 버린다(`4d3a1d` L2465).
- **내림 덧셈** `rdown(acc, p)`: s = acc + p. |acc| < |p|면 s − p == acc, 아니면 s − acc == p일 때 정확하다고 본다. 부정확하면 s ≥ 0일 때 s·(1−2⁻⁵²), s < 0일 때 s·(1+2⁻⁵²)이다.
- **올림 덧셈** `rup`: 정확성 판정은 같다. 부정확하면 s > 0일 때 s·(1+2⁻⁵²), s ≤ 0일 때 s·(1−2⁻⁵²)이다.
- 상수 `TOL4` = 9.999999999999999e−05 = **0x3f1a36e2eb1c432c**(1e−4보다 1ulp 작다). −TOL4 = 0xbf1a36e2eb1c432c.

##### C.4 `4e7515` (actmin 갱신)과 `4e7b3e` (actmax 갱신), `a7d0ab`
```
upd_min(c>0, old, new, L, U, r, A, work):                 // 4e7515(xmm0..2, ctx, L, U, r, &A[r], work)
  if old <= -1e30:                                        // 무한 기여 하나가 사라짐
      A[r] = BIG + A[r]
      if A[r] != -BIG: return                             // 아직 무한 기여가 남음
      acc = -rhs[r]                                       // 마지막 무한 기여가 사라짐: 처음부터 다시
      for t in row r (순서대로): a = val[t]; acc = rdown(acc, a<=0 ? U[col]*a : L[col]*a)
      *work = μ·3.0·len + *work;  A[r] = acc;  return
  if A[r] <= -BIG: return
  d = new - old
  if 1e10 <= d: A[r] = a7d0ab(1.0, row r, L, U, work) - rhs[r]; return
  A[r] = rdown(A[r], c·d)
upd_max(c>=0, old, new, ...):                             // 4e7b3e
  if old >= 1e30: A[r] = A[r] - BIG; if A[r] != BIG: return
      acc = -rhs[r]; for t: acc = rup(acc, a>=0 ? U·a : L·a); *work = μ·3.0·len + *work; A[r] = acc; return
  if BIG <= A[r]: return
  d = old - new
  if 1e10 <= d: A[r] = -rhs[r] - a7d0ab(-1.0, row r, L, U, work); return
  A[r] = rup(A[r], -(c·d))
a7d0ab(sg, len, idx, val, L, U, work):                    // 행 min activity(sg·a)
  acc = 0
  for t: c = val[t]·sg; b = (c <= 0) ? U[j] : L[j]
         if (c<=0 && b >= 1e30) || (c>0 && b <= -1e30): acc = -1e100; break
         acc = rdown(acc, c·b)
  *work = (처리한 수)·3.0 + *work                          // μ 없음
  return acc
```
- 다시 계산하는 두 경로의 연산 순서가 다르다. 인라인 경로는 −rhs에서 시작해 μ·3·len을 더하고, `a7d0ab` 경로는 0에서 시작해 −rhs를 나중에 빼며 3·len을 μ 없이 더한다. 비트 단위로 맞추려면 둘을 따로 옮겨야 한다.
- 기본값 58회에서 무한 bound 전환과 1e10 이상의 변화는 한 번도 없었다(검증 안 됨).

#### D. `4eba40`: bound 하나 바꾼 뒤 열의 행 갱신과 암시 표 따라가기

```
eba40(s: ±1.0, old, new, ctx, L, U, depth, flag, j, cm, cl, &ncl, &nrl, rm, rl, amin, amax, extra, work) -> i32
// 호출자가 L[j] 또는 U[j]를 이미 new로 써 둔다. s = +1: ub 변화, s = -1: lb 변화
//   (ub 1→0: (1.0, 1.0, 0.0), lb 0→1: (-1.0, 0.0, 1.0); 이진은 실제 옛 값 대신 이 상수를 넘긴다)
if cm[j] == 0: cl[ncl++] = j; cm[j] = 1
so = s·old; sn = s·new
for k = len(j)-1 downto 0:                                 // 열 CSR을 뒤에서부터
    c = colval[k]·s; r = colrow[k]
    if (rm[r] & 1) == 0: rl[nrl++] = r; rm[r] |= 1
    if c < 0:
        if r < nrow0:
            upd_min(-c, -so, -sn, L, U, r, amin)
            if amin[r] > TOL4: return 1 (아래 작업량 더한 뒤)
            if flag && 1e-6 < maxrange[r] + amin[r]: push(amin[r] >= -1e-10 ? A : B, r)
        elif flag && (vtype[j] != 'B' || extra[2(r-nrow0)] == j || extra[2(r-nrow0)+1] == j): push(A, r)   // GUB
    else:
        if r < nrow0:
            upd_max(c, so, sn, L, U, r, amax)
            if sense[r] == '=' && amax[r] < -TOL4: return 1
            if sense[r] == '<' && amax[r] < -0.001: rm[r] |= 8          // 이 시나리오에서 행이 남음(중복)
            if flag && sense[r] == '=' && amax[r] - maxrange[r] < -1e-6: push(-amax[r] >= -1e-10 ? A : B, -1-r)
        elif (GUB, 위와 같음)
*work = ((len - k - 1)·6.0 + 5.0)·μ + *work               // k = 멈춘 위치(끝까지 돌면 -1)
if j >= n: return 0
if flag && st+0x1f8 > 0 && so >= 0 && sn < -IntFeasTol: SOS 목록을 A에 push(+nr)  // 기본값 없음
if k6 > 0: 4e818b(...)                                     // QC, 기본값 없음
if (u32)(depth+1) <= 1000 && vtype[j] == 'B' && T = st+0x3f0, T.starts(+8) != NULL:   // 암시 표
    lit = new > 0.5 ? 2j : 2j+1                            // x_j = 1이면 2j, 0이면 2j+1
    for e in T.starts[lit] .. T.starts[lit+1]:   v = T.lits[e]  (i64 starts, i32 lits)
        if v >= 0:  kk = v   ("x_kk = 0")
            if colcnt[kk] (st+0x68) >= 0 && U[kk] > 0.5:
                if L[kk] > 0.5: ret = 1; break
                if depth == -1: if C.has(-1-kk): ret = 1; break;  C.push(kk)
                else: U[kk] = 0; ret = eba40(1.0, 1.0, 0.0, ..., depth+1, flag, kk, ...); if ret: break
        else: kk = -1-v ("x_kk = 1")
            if colcnt[kk] >= 0 && L[kk] < 0.5:
                if U[kk] < 0.5: ret = 1; break
                if depth == -1: if C.has(kk): ret = 1; break;  C.push(-1-kk)
                else: L[kk] = 1; ret = eba40(-1.0, 0.0, 1.0, ..., depth+1, flag, kk, ...); if ret: break
    *work = 3.0·μ·(e - T.starts[lit]) + *work            // e = 멈춘 위치
return ret
```
- depth: 이진 후보의 첫 호출은 −(ctx+0x1e4 ≠ 0)(기본값 −1 = "암시를 C에 모으기만"), 행 후보는 0(재귀 적용, 1001단계까지), `4ed528`·C 처리에서는 −2(암시 표 안 따라감).
- `flag`(param_8) = 0이면 활동량 갱신과 모순 검사만 하고 행을 큐에 넣지 않는다. `4ed528`은 "변화가 크다"일 때만 1을 준다.
- 암시 표 T(st+0x3f0)는 clique 모듈 몫이다. 첫 `4e48f7` 호출에서는 아직 비어 있다(starts = NULL). 같은 presolve의 두 번째 이후 호출에서는 차 있다(gmu-35-40 call 2, ic97 call 2·3, timtab1 call 2·3, glass4/sp150x300d/tr12-30 call 2).

#### E. `4ed528`: 행 하나의 한쪽으로 bound 강화 (asm 기준. Ghidra는 두 분기의 부호를 바꿔 보여 준다)

행 r의 CSR은 **일반(비이진) 항목이 먼저, 그다음 이진 항목**이다. 일반 구간은 |a|·(ub−lb) 내림차순, 이진 구간은 |a| 내림차순이다(`396a2d`, §3.1. 8개 모델 ctx 비트 비교 §4.2). ng = ctx[0xb][r] − ctx[10][r]이다.
```
ed528(s: +1 (≤ 쪽) | -1 (= 행의 ≥ 쪽), ctx, r, L, U, mode, cm, cl, &ncl, &nrl, rm, rl, amin, amax, extra, work) -> i32
if s > 0: slack = -amin[r]; act = amax[r] > 1e-6;  if !act: rm[r] |= 2
else:     slack = amax[r];  act = amin[r] < -1e-6; if !act: rm[r] |= 4
*work = 10.0·μ + *work
if slack < -TOL4: return 1                                  // 모순
if !act: return 0                                           // 이쪽은 이미 남는 쪽
if slack == 2·BIG (5.164499756173817e120):                  // 무한 기여가 정확히 하나
    if mode != 0: return 0
    sm = 0; inf = -1
    for e in 0..ng:  c = val·s; if c > 0: (L[j] > -1e30 ? sm = sm + c·L[j] : inf = e) else (U[j] >= 1e30 ? inf = e : sm = sm + c·U[j])
    for e in ng..len: sm = sm + (c > 0 ? c·L[j] : c·U[j])  // 평범한 덧셈(반올림 보정 없음)
    *work = 4.0·μ·ng + *work;  *work = *work + 3.0·μ·(len - ng)
    c = val[inf]·s; rest = rhs[r]·s - sm; j = col[inf]; l = L[j]; u = U[j]; v = rest / c
    if c > 0: um = u - 1e-6
        if v < um: v = (vtype[j]=='C') ? (l > v && v > l-1e-6 ? l : v) : floor(v + IntFeasTol)
        if v < um && (u < 1e30 || v < 1e8) && v > -1e8:
            flag = max(min(|u|,|v|), 1e-4) < u - v;  U[j] = v;  ret = eba40(1.0, u, v, ..., -2, flag, j, ...)
    else:  lp = 1e-6 + l
        if v > lp: v = (vtype[j]=='C') ? (v > u && u+1e-6 > v ? u : v) : ceil(v - IntFeasTol)
        if v > lp && (l > -1e30 || v > -1e8) && v < 1e8:
            flag = max(min(|l|,|v|), 1e-4) < v - l;  L[j] = v;  ret = eba40(-1.0, l, v, ..., -2, flag, j, ...)
    *work = 7.0·μ + *work;  return ret
slack = max(slack, 0)        // maxsd: NaN이면 0
e = 0
if mode == 0:                                              // 일반 항목
  while e < ng:
    c = val[e]·s; j = col[e]; u = U[j]; l = L[j]; um = u - 1e-6
    if (um - l)·|c| <= slack:
        if slack >= ((ub0[j] - 1e-6) - lb0[j])·|c|: break   // 원래 범위로도 못 줄임 → 뒤 항목도 못 줄임
    elif c > 0:  v = slack / c + l
        if v < um: v = 'C' ? (l > v && v > l-1e-6 ? l : v) : floor(v + IntFeasTol)
        if v < um && (u < 1e30 || v < 1e8):
            flag = (u - v) > ((-(l - 1e-6)) + u)·0.01 || (u - v) > max(min(|u|,|v|), 1e-4)
            U[j] = v; ret = eba40(1.0, u, v, ..., -2, flag, j, ...)
    else:        v = slack / c + u;  lp = 1e-6 + l
        if v > lp: v = 'C' ? (v > u && 1e-6+u > v ? u : v) : ceil(v - IntFeasTol)
        if v > lp && (l > -1e30 || v > -1e8):
            flag = (v - l) > ((-(l - 1e-6)) + u)·0.01 || (v - l) > max(min(|l|,|v|), 1e-4)
            L[j] = v; ret = eba40(-1.0, l, v, ..., -2, flag, j, ...)
    e += 1; if e >= ng || ret: break
  *work = 7.0·μ·e + *work                                    // e = 멈춘 위치
thr = 1e-6 + slack
for e in ng..len (ret == 0인 동안):                         // 이진 항목: |c|가 여유보다 크면 고정
    c = val[e]·s; if thr > |c|: break
    j = col[e]; if L[j] == U[j]: continue
    if c > 0:  (x_j = 0이어야 함)  mode ? (C.has(-1-j) ? ret=1,break : C.push(j)) : (U[j]=0; ret = eba40(1.0,1.0,0.0, ..., -2, 1, j))
    else:      (x_j = 1)           mode ? (C.has(j)    ? ret=1,break : C.push(-1-j)) : (L[j]=1; ret = eba40(-1.0,0.0,1.0, ..., -2, 1, j))
*work = *work + 4.0·μ·(e - ng)
return ret
```
- mode ≠ 0("암시 모으기")이면 일반 항목과 무한 하나 경우는 건너뛰고 이진 항목도 적용하지 않는다. 대신 C에 리터럴을 넣고 충돌만 본다.
- 정수 반올림은 `floor(v + IntFeasTol)` / `ceil(v − IntFeasTol)`이다(`4d3a1d`·LP 포트의 `37168e`와 같은 꼴).
- 무한 하나 경우는 기본값 58회에서 102,971번 들어왔지만 bound를 바꾼 적은 없다(바꾸는 분기는 검증 안 됨).

#### F. `4eb08d`: 큐로 도는 행 전파

```
eb08d(ctx, L, U, mode, cm, cl, &ncl, &nrl, rm, rl, amin, amax, extra, T(타이머), work) -> i32
limit = ctx.f · ctx.B                       // 진입 때 한 번 계산
deferred = []; st = 0
if A.cnt > 0 || B.cnt > 0:
  loop:
    if !(limit > elapsed(T)): break
    *work = μ·2.0 + *work
    item = A.cnt > 0 ? A.pop() : B.pop();  r = item >= 0 ? item : -1-item
    B.block(item); A.block(item)
    d = 0
    if r >= nr + st+0x1f8: (QC)  elif r >= nr: (SOS, st+0x208..+0x220)  elif r >= nrow0: (GUB: mode ? d = 1 : 4f1bbd/4f0f39)
    elif item >= 0: if !(rm[r] & 2): st = ed528(+1.0, r, ...); if st == 0 && mode: d = vtype[col[rowstart[r]]] != 'B'
    else:           if !(rm[r] & 4): st = ed528(-1.0, r, ...); (같음)
    B.unblock(item); A.unblock(item)
    if d: deferred.push(item)               // 일반 변수가 있는 행: mode 1에서는 못 다루니 나중으로
    if A.cnt > 0 || B.cnt > 0: if st: return st; continue
    if st: return st;  break
if limit > elapsed(T) && deferred 비어 있지 않음:
  for item in deferred: r = decode(item)
      set = (r < nr + st+0x1f8 && r < nrow0) ? (item < 0 ? (-amax[r] >= -1e-10 ? A : B) : (amin[r] >= -1e-10 ? A : B)) : A
      set.push(item)
return st
```
- 보류 목록은 ctx[0x41](+0x208)을 쓴다. mode 1에서 미룬 행을 큐에 되돌려 놓으면, 뒤이은 mode 0 호출이 처리한다.
- QC·SOS·GUB 분기(`6181e3`/`618398`/`4f1bbd`/`4f0f39`)는 기본값 38개에서 0회다(st+0x1f8 = 0, k6 = 0, nr = nrow0).

#### G. `4e28ef`: 후보 점수 (첫 호출과 st+0x3c0 ≠ NULL일 때만), 그다음 `aabbe3` 내림차순 정렬

```
e28ef(env, ctx, ncand, cand, score, work) -> i32
rng = (u64)(i64)(i32)env+0x2cd0        // a7b54f 상태(기본 0). 후보마다 한 번씩 뽑는다(종류 상관없이 순서대로)
cache = calloc(nrow0) (0 = 아직, -1 = 안 씀, >0 = 개수);  mx = 0.0
for k in 0..ncand:
  j = cand[k]
  if j >= nc: u = rng.f64(); rr = j - nc; score[k] = -(u + (f64)(rowstart[rr+1] - rowstart[rr])); continue
  score[k] = 1.0
  for e in colstart[j] .. colstart[j+1]:
     a = colval[e]; r = colrow[e]; w = (0 > a) ? 150.0 : 100.0
     if r >= nrow0: continue
     c = cache[r]
     if c == 0:                                         // 행 r의 "j와 같이 1이면 안 되는 이진" 수
        b = rowstart[r]; en = rowstart[r+1]; am = actmin[r]; dmin = -1.0
        if en - b < 2 || am > -FeasibilityTol || vtype[col[b]] != 'B': cache[r] = -1; continue
        x0 = |a| + am; cc = 0; valid = true
        for t in b..en:
            if col[t] == j: continue
            v = |val[t]|; if 0 > dmin: dmin = v
            if v + x0 > 1e-10: cc += 1
            else: valid = false
                  if cc == 0 && (t == en-1 || -1e-10 > (am + |val[t+1]|) + dmin): cache[r] = -1
                  break
        *work = μ·6.0·(t - b) + *work
        if valid: cache[r] = cc
        c = cc                                          // valid가 아니어도 이번 점수에는 cc를 쓴다(저장은 안 함)
     if c > 0: score[k] = (sense[r] == '=') ? (w·w)/c + score[k] : w·log(c) + score[k]
  *work = μ·10.0·(len) + *work
  u = rng.f64(); score[k] = (u·1e-4 + 1.0)·score[k]; mx = max(mx, score[k])
*work = ncand·(3.0·μ) + *work
if T.starts != NULL:
  for k: j = cand[k]; if j < nc: d1 = starts[2j+1]-starts[2j]; d2 = starts[2j+2]-starts[2j+1]
         if d1 > 0: score[k] = (d1·100.0 + mx) + score[k];  if d2 > 0: score[k] = (d2·100.0 + mx) + score[k]
  *work = *work + (3.0·μ)·ncand
```
- **cache는 그 행을 처음 만난 열의 |a|로 계산한 개수를 다음 열에도 그대로 쓴다.** 원본의 버릇이므로 그대로 옮긴다.
- st+0x3c0 ≠ NULL이면 4e28ef 대신 score = (f64)st+0x3c0[j](j < n), 0.0(n ≤ j < nc), −(행 길이)(행 후보)를 쓰고 work += 3μ·ncand 뒤 정렬한다(기본값 NULL).
- 정렬 `aabbe3(ncand, score, cand)`는 `crates/lp/src/common.rs::sort_desc`, 난수 `a7b54f`는 `common.rs::Rng::next_f64`에 이미 있다.
- 마무리 호출(p7 = 1, st+0x3c0 = NULL)에서는 점수도 정렬도 없이 `4d3a1d`가 준 순서를 쓴다.

#### H. 본체 `4e48f7`

```
probe(st, ctx, cand, ncand, work):
  T = timer(work)                                          // C.2
  ctx.nchg(+0xf4) = ctx.nbin(+0xf8) = ctx.neq(+0xfc) = 0
  lbS[0..2], ubS[0..2] = f64[nc] ×4 한 덩어리(lbS0, ubS0, lbS1, ubS1 순)    // 시나리오 슬롯 2개
  A = IntSet(-nr, nr + st+0x1f8 + k6); B = 같은 범위; if ctx+0x1e4: C = IntSet(-n, n)   // 각 work += 범위 크기
  cmS[0..2], clS[0..2] = calloc i32[4nc]: cm0, cm1, cl0, cl1
  rm = calloc i32[nr+k6]; rl = i32[nr+k6]; aminS, amaxS = f64[nrow0+k6]; extraS = i32[2(nr-nrow0)]
  if ctx+0x1b0: ctx.rowhead(+0x1c0) = i32[nrow0] 전부 -1; next/lit = i32[10000]; slack = f64[10000]; cap(+0x1b4) = 10000
  nS[0..2] = nSs[0..2] = 0
  memcpy 순서: lbS0←lb, ubS0←ub, lbS1←lb, ubS1←ub, aminS←ctx.amin, amaxS←ctx.amax, extraS←ctx[0x27]
  if p7 == 0 || st+0x3c0: score = f64[ncand]; e28ef(...) 또는 st+0x3c0 점수; aabbe3(ncand, score, cand)
  // 목적 컷오프(기본값에서는 꺼짐: st+0x490 = 1e100)
  use = DualReductions(env+0x2430); om = st+0xc8(objcon)
  if !(st+0x250 == NULL && st+0x330 == 0 && cutoff(st+0x490) < 5e99 && use): use = 0
  else: cnt = 0
        for j in 0..n: c = obj[j]; if c > 0: (lb[j] <= -1e30 ? (om = -1e100, 중단) : om = rdown(om, c·lb[j]), cnt++)
                                    elif c < 0: (ub[j] >= 1e30 ? 중단 : om = rdown(om, c·ub[j]), cnt++)
        if 중단 || !(-1e100 < om) || cnt == 0: use = 0
  if ctx+0x1e4: ctx.g(+0x1e8) = 0
  tolc = (|cutoff| + 1.0)·1e-6; last = 0.0
  for i in 0..ncand:
    t = elapsed(T)
    f = (((nbin·20.0 + 100.0) + nagg·10.0) + nchg) / (i + 100.0);  f = min(f, 100.0); f = max(f, 0.01)   // minsd/maxsd
    ctx.f = f
    if t > f·ctx.B: ctx+0xe8 = 1; break                    // 예산 소진
    if t - last > 1e7: last = t
        if ctx[1] != NULL && ctx+0xec == 0: *(st+0x128) = 0 (adf18e(st+0x108)); r = 3ba5ac(env, st, 0, 0, -1); r ||= 3ba9de(env, st, 0)
            if r: return r (끝맺음 작업량 없이);  if st+0x1dc < 0: return 0
        else: r = a83b2f(*(ctx[0]+0x100), work); if r: break → 끝맺음 작업량 뒤 return r
    if ctx+0x1e4: g = ctx.g; grp[2g] = grp[2g+1] = grp[2g+2] = ctx.used; ctx.g = g+1
    x = cand[i]
    if x >= nc: ROW_PROBE(x - nc); continue
    if !(lb[x] == 0.0 && ub[x] == 1.0 && ctx[0x29][x] < 0): continue
    BIN_PROBE(x)
  *work = i·12.0·μ + *work                                 // i = 멈춘 위치(다 돌았으면 ncand)
  free, return 0

ROW_PROBE(rr):            // '<' 행을 "모든 변수를 min activity 쪽 bound에 두기"로 시험
  slot 0 사용 (L = lbS0, U = ubS0); nrl = 0; ncl = 0; A.clear(); B.clear(); if ctx+0x1e4: C.clear()   // clear마다 work += 2·개수
  ret = 0; cnt = 0
  for e in rowstart[rr] .. rowstart[rr+1]:
      j = col[e]; u = U[j]; l = L[j]
      if !(0.5 > u - l):
          if val[e] > 0: U[j] = l; ret = eba40(1.0, u, l, L, U, 0, 1, j, slot0...)
          else:          L[j] = u; ret = eba40(-1.0, l, u, L, U, 0, 1, j, slot0...)
          cnt += 1
      e++; if ret: break
  *work = (e - rowstart[rr])·4.0·μ + *work
  if ret || (cnt >= 2 && eb08d(L, U, 0, ...) != 0): ctx.sense[rr] = '='; ctx.neq += 1
  rl의 행마다: rm = 0; aminS/amaxS ← ctx.amin/amax;   *work = nrl·6.0·μ + *work
  cl의 열마다: L/U ← ctx.lb/ub; cm = 0;                *work = ncl·6.0·μ + *work

BIN_PROBE(x):
  nf = 0                                                    // 가능했던 시나리오 수 = 다음 슬롯 번호
  for side in [0 (x=0), 1 (x=1)]:
     s = nf; L = lbS[s]; U = ubS[s]; nrl = 0; A.clear(); B.clear(); if ctx+0x1e4: C.clear()
     depth = ctx+0x1e4 ? -1 : 0
     ret = side == 0 ? (U[x] = 0; eba40(1.0, 1.0, 0.0, L, U, depth, 1, x, slot s...))
                     : (L[x] = 1; eba40(-1.0, 0.0, 1.0, L, U, depth, 1, x, slot s...))
     obj_ok = true
     if ret == 0:
        if ctx+0x1e4:                                       // 1단계: clique/암시만
           ret = eb08d(L, U, 1, ...)
           while C.cnt > 0:
              if ret: obj_ok = false; break
              if !(f·B > elapsed(T)): break                 // 시간 초과면 2단계 없이 목적 검사로
              lit = C.pop()
              if lit < 0: k = -1-lit; if L[k] == 0.0: (U[k] == 0.0 ? ret = 1 : (L[k] = 1; ret = eba40(-1.0,0.0,1.0, ..., -2, 1, k)))
              else: k = lit;          if U[k] == 1.0: (L[k] == 1.0 ? ret = 1 : (U[k] = 0; ret = eba40(1.0,1.0,0.0, ..., -2, 1, k)))
           nSs[s] = nS[s]                                   // 1단계까지 바뀐 열 수
           if ret: obj_ok = false
        if obj_ok:                                          // 2단계: 전체 전파 + 목적 검사
           if !(f·B <= elapsed(T)): ret = eb08d(L, U, 0, ...)
           if ret == 0 && use:
              ob = om
              for j in cl[s]: if j >= n || obj[j] == 0: continue
                  c = obj[j]; if c > 0 && L[j] > lb[j]: ob = rdown(ob, (L[j] - lb[j])·c)
                              if c < 0 && ub[j] > U[j]: ob = rdown(ob, (U[j] - ub[j])·c)
              if tolc < ob - cutoff: ret = 1
     for q in rl:                                           // 행 표시·activity 되돌리기
        if ret == 0 && ctx+0x1b0 && (rm[q] & 8): REC_REDUNDANT(q, side == 1 ? -1-x : x, -amaxS[q])
        rm[q] = 0; aminS[q] = ctx.amin[q]; amaxS[q] = ctx.amax[q]
     *work = nrl·6.0·μ + *work
     if ret == 0: nf += 1
     else: cl[s]의 열마다 L/U ← lb/ub, cm = 0;  *work = ncl·6.0·μ + *work;  nS[s] = nSs[s] = 0
  if nf == 0: ub[x] = 0; lb[x] = 1; return 0               // 문제 전체가 불가능(검증 안 됨)
  if nf == 2: ctx[0x2c][x] = nS[0]; ctx[0x2d][x] = nS[1]
  lst = (nf > 1 && nS[1] < nS[0]) ? cl[1] : cl[0]           // 짧은 쪽 목록
  nrl = 0
  for j in lst:                                             // 공통 암시
     lo = min_s lbS[s][j] (시작 1e30, minsd);  hi = max_s ubS[s][j] (시작 -1e30, maxsd)
     *work = (nf + nf)·μ + *work
     l = lb[j]; u = ub[j]; l0 = l; lch = uch = false
     if lo > 1e-6 + l:  if lo > u && 1e-6 + u > lo: lo = u
         lb[j] = lbS1[j] = lbS0[j] = lo; lch = true; l = lo; if j < n: nchg++, if 'B': nbin++
     if hi < u - 1e-6:  if hi < l && l - 1e-6 < hi: hi = l
         ub[j] = ubS1[j] = ubS0[j] = hi; uch = true;          if j < n: nchg++, if 'B': nbin++
     if lch || uch:
         for (r, a) in column j: if (rm[r]&1)==0: rl[nrl++] = r, rm[r] |= 1
             if r < nrow0:                                  // 전역 ctx.amin/amax, 전역 lb/ub로
                 if lch: a > 0 ? upd_min(a, l0, lo) : upd_max(-a, -l0, -lo)
                 if uch: a > 0 ? upd_max(a, u, hi)  : upd_min(-a, -u, -hi)
         *work = len·6.0·μ + *work
         if k6 > 0 && j < n: 4e818b(...)                    // QC
     // 집계: x와 j가 한 직선 위에 있는가(bound가 방금 바뀌었으면 바뀐 값으로 본다)
     if nf > 1 && j != x && j < n && x < n && ctx[0x29][j] < 0
        && aggm >= st+0x88[x] && st+0x80[x] >= 1/aggm:     // aggm = env+0x26c0 (1e14)
        l = lb[j]; u = ub[j]; w = -(l - u)
        if aggm >= w·st+0x88[j] && w·st+0x80[j] >= 1/aggm:
           if lbS0[j] > u - 1e-10 && 1e-10 + l > ubS1[j]:   // x=0이면 j=u, x=1이면 j=l
               ctx[0x29][j] = x; (|u| <= 1e-12) ? (c2a = l, c2b = 0) : (c2a = l - u, c2b = u); nagg++
           elif 1e-10 + l > ubS0[j] && lbS1[j] > u - 1e-10: // x=0이면 j=l, x=1이면 j=u
               ctx[0x29][j] = x; (|l| <= 1e-12) ? (c2a = u, c2b = 0) : (c2a = w, c2b = l); nagg++
  *work = |lst|·10.0·μ + *work
  rl의 행마다: rm = 0; aminS/amaxS ← ctx.amin/amax;   *work = *work + nrl·6.0·μ
  if nf > 1 && ctx+0x1e4: RECORD_IMPLICATIONS()
  for s in 0..nf: cl[s]의 열마다 L/U ← lb/ub, cm = 0; *work = |cl[s]|·6.0·μ + *work; nS[s] = nSs[s] = 0
  *work = nf·μ + *work

RECORD_IMPLICATIONS():   // 2단계에서 새로 찾은(nSs 뒤) 이진 암시만 ctx[0x40]에 모은다
  need = (((nS0 - nSs0) + nS1) - nSs1) + used
  if cap(+0x1f0) < need:
     if lim(+0x1e0) < need || (f64)lim < cap·1.5: ctx+0x1e4 = 0; return      // 이후 암시 모드 끔
     cap = max(need, (i32)(cap·1.5)); realloc ctx[0x40]
  v = used
  for (s, o) in [(0,1), (1,0)]:
     for q in nSs[s] .. nS[s]: j = cl[s][q]
        if j < n && vtype[j]=='B' && ctx[0x29][j] < 0:
           if lbS[s][j] == 1.0 && lbS[o][j] == 0.0: imp[v++] = -1-j       // (x = s) ⇒ x_j = 1
           elif ubS[s][j] == 0.0 && ubS[o][j] == 1.0: imp[v++] = j         // (x = s) ⇒ x_j = 0
     *work = ((nS[s] - nSs[s])·6.0 + 1.0)·μ + *work
     grp[2g - 1 + s] = v                                   // g = ctx.g(이 후보에서 이미 +1)
  used = v
  // 묶음 g' = g-1: imp[grp[2g'] .. grp[2g'+1]) = "x=0이면", imp[grp[2g'+1] .. grp[2g'+2]) = "x=1이면"

REC_REDUNDANT(q, lit, d):   // 마무리 호출(ctx+0x1b0): "lit이면 행 q가 d(= rhs - max activity > 0.001)만큼 남는다"
  g = ctx.n37(+0x1b8); if g == 1000000: return
  if cap(+0x1b4) == g: cap = min(2g, 1000000); realloc next/lit/slack (cap == 0이면 해제)
  lit[g] = lit; slack[g] = d (= -amaxS[q]); next[g] = head[q]; head[q] = g; n37 = g + 1
```
- **끝맺음 작업량 `i·12·μ`의 비대칭**: 마지막 후보가 이진 probe였다면 asm에서는 `465652` → `462c0b`로 바로 간다. 다만 work 포인터가 있으면 그 전에 `nf·μ`를 더하고 `46369c`로 가서 루프 끝에 12μ·i를 더하므로 결과는 같다. 포트는 "루프를 나올 때 늘 12μ·i"로 두면 되고, 예외는 nf == 0과 1e7 점검의 3ba5ac/3ba9de 반환(이때는 더하지 않음)뿐이다.
- 시나리오 실패 때 되돌리는 것은 그 슬롯의 lbS/ubS/cm뿐이다. 행 표시·activity 사본은 성공·실패와 상관없이 시나리오마다 되돌린다.
- nf == 1이면 남은 시나리오의 바뀐 bound가 모두 "공통 암시"가 된다. 그래서 x 자신도 고정된다(x는 eba40 첫 줄에서 cl에 들어 있다). gmu-35-40의 고정 10개가 모두 이 경로다(x=1이 모순 → ub[x] = 0).

#### I. 4e48f7이 내놓고 4d3a1d가 뒤처리에서 읽는 것

| ctx | 뜻 | 기본값에서 바뀐 모델(호출) |
|---|---|---|
| +0x100/+0x108 lb/ub | 고정·bound 강화. state 반영은 4d3a1d. 열 단위로 state 변경과 1:1(덤프 대조) | gmu-35-40(1) 10, gmu-35-50(1) 29, timtab1(1) 5, tr12-30(1) 20, ic97(1) 2 |
| 둘 다 모순인 후보: lb[x] = 1 > ub[x] = 0 | 불가능 표시 | 없음 |
| +0xf4/+0xf8 | 바뀐 bound 수 / 이진 | 위와 같음(tr12-30 32, gmu 10/10 등) |
| +0x120 sense, +0xfc | `<` 행 → `=` | 없음(행 후보 614개 시험, 바뀐 것 0) |
| +0x148/+0x150/+0x158, +0x140 | 집계 x_j = c2a·x + c2b | timtab1(1) 12개 → state 열 제거 12 + type 2 레코드 12 |
| +0x160/+0x168 | 후보별 바뀐 열 수(nf == 2) | 모든 호출 |
| +0x128/+0x130 | 공통 암시를 반영한 activity | bound가 바뀐 호출 |
| +0x1e8/+0x1f8/+0x200/+0x1ec | 후보별 새 이진 암시 묶음 | 첫 호출 다수(gmu-35-40 502개, gmu-35-50 1127, fhnw 4112, crna 3302, snowy 3692, neos-860300 1826, ic97 1454, glass4 312 …). `5b88ce` 호출 수와 비교해 볼 것(4d3a1d 몫) |
| +0x1b8..+0x1d8 | 마무리 호출의 행 중복 기록 | glass4(2) 16, graphdraw(2) 66, ic97(3) 189, berkel(2) 5, tr12-30(2) 12. glass4·graphdraw·berkel은 기록 수 = state 계수 변경 수 = nnz 증가(16/66/5)다. ic97은 189개 → 계수 98·nnz +97, tr12-30은 12개 → 계수 1이라 4d3a1d가 기록 중 일부만 쓴다 |
| +0xe8, +0xe0 | 예산 소진 / 마지막 f | stop 11회(bppc4-08, cod105, csched007/008, gmu-35-40(2), gmu-35-50(2), mad(1), neos-4338804, neos-860300, neos-911970(1), neos17(1)) |
| cand | 점수순 정렬된 후보 | 첫 호출 |

#### J. 검증 (Frida + 다시 짠 시뮬레이터)
- 덤프: `node probe/js/ptrace.js <model> <out> probe/js/x_4e48f7.js 4e48f7`(`tools/gurun.sh`로 감쌈). `4e48f7` 진입 때 ctx의 스칼라·배열 전부, state(obj, objcon, cutoff, col_cnt, +0x80/+0x88, 암시 표 st+0x3f0), env(0x2430, 0x2460, 0x26c0, 0x2cd0), cand, work를 덤프한다. 복귀 때는 ctx 출력 전부, cand, work를 덤프한다. 35개 모델 58회.
- 시뮬레이터 `probe/py/sim48.py`가 위 §C–H를 그대로 옮겼다. 결과(`probe/sim_all2.txt`): 58/58 호출에서 cand 순서, lb/ub, actmin/actmax, sense, ctx[0x29..0x2d], nchg/nbin/neq/nagg, stop, f, 묶음·암시 목록, 중복 행 기록, **work가 0 차이로 같다**.
- 지나간 경로(합계): 이진 probe nf=2 16,610 / nf=1 39(전부 고정), 행 후보 614, C에서 꺼낸 리터럴 1,625, 무한 하나 경우 102,971, 집계 12, 중복 행 기록 288, 예산 소진 11.
- 지나가지 않은 경로(asm만 보고 씀): nf = 0, 목적 컷오프로 시나리오 기각(Cutoff=−2406000으로 gmu-35-40를 돌려 계산 경로는 탔지만 기각은 없었고, 그때도 58/58과 같이 일치), `<`→`=`, 무한 bound 전환·1e10 이상 변화(활동량 재계산), 무한 하나 경우의 실제 강화, 암시 용량 초과로 끔, 1e7 점검의 `3ba5ac`/`3ba9de` 중단, GUB/SOS/QC 분기 전부.
- state 대조: `d/<model>.json`(4d3a1d 진입·복귀 state)와 비교했다. 4e48f7이 ctx에서 바꾼 열 집합이 4d3a1d 복귀 state의 bound 변경 열과 정확히 같다(gmu-35-40 10, gmu-35-50 29, timtab1 5, tr12-30 20, ic97 2).

#### K. 열린 질문
1. env+0x2728(암시 모드·중복 행 기록 스위치)은 파라미터 표에 없다. 기본 0이다. 무엇이 1로 만드는가(동시 실행/재시작?).
2. env+0x2cd0(4e28ef 난수 시드)도 표에 없다. 기본 0이다. Seed 파라미터와 연결되는지 Seed=1로 확인할 것.
3. (해결) 행 CSR 정렬은 `4d3a1d`가 행마다 부르는 `396a2d`다(§3.1). 일반 쪽 키는 |a|·(ub−lb)다.
4. ctx lb가 이진에서 −0.0으로 들어온다(gmu). `4d3a1d`는 st.lb를 memcpy하므로 −0.0은 state에 이미 있다. 결과에 새는지(예: `lb[x] == 0.0` 비교는 −0.0도 참)는 포트에서 bit 비교로 볼 것.
5. 1e7 점검의 `adf18e(st+0x108)` = `*(st+0x128) = 0`이 무슨 필드인지(state 쪽 타이머로 보임).
6. 목적 컷오프 경로: Model.presolve()에서는 st+0x490 = 1e100이라 늘 꺼진다. 최적화 경로(3db15a, 컷오프·incumbent가 있는 재시작)에서는 켜질 수 있다.


### 3.5 conflict graph와 나머지 보조 함수 (`5b9aad`, `5b86ce`, `5b88ce`, `7886b`, `3bea24`, `3de18e`, `3df916`, `564ce0`)

표기: `W` = `*(st+0x458)`(작업량 double, NULL이면 건너뜀), `μ` = `st+0x448`. 곱셈 괄호는 asm의 순서 그대로다(예: `(c·6)·μ + W`는 `cvtsi2sd c; mulsd 6.0; mulsd μ; addsd W`). 노드는 m1_presolve_agg §1.1의 32바이트 노드. 리터럴 인코딩: `x_j`는 `j`, `¬x_j`(= 1 − x_j)는 `−1−j`. `var(e) = e<0 ? −1−e : e`, `lit(e) = 2·var(e) + (e<0)`.

기본값 38개에서 probing(4d3a1d) 아래 실제 호출(Frida, `$S/probe/tr/*.json`):

| 함수 | 호출 지점(복귀 주소) | 모델 |
|---|---|---|
| `5b9aad` | 0x458ae9(진입, 62회), 0x460ed5(끝, 58회) | 38개 전부 |
| `7886b` | 0x45d4f8 | 4e48f7이 도는 35개(58회) |
| `5b86ce` | 0x460a0f | 같은 35개(58회) |
| `5b88ce` | 0x460bff(¬x_j 쪽) 4,792회, 0x460d59(x_j 쪽) 11,548회 | fhnw-binpack4-4, glass4, gmu-35-40/50, ic97_potential, neos-2657525-crna, neos-4338804-snowy, neos-860300, sp150x300d, timtab1, tr12-30 (모두 첫 호출 지점 0x3b8c05에서만) |
| `3bea24` | 0x461509 | glass4 16, graphdraw-domain 66, ic97_potential 97, neos-4954672-berkel 5 (모두 둘째 호출 지점 0x3bcd45) |
| `3df916` (+`39ab33`, `53802c`) | 0x45f7c2 | gmu-35-40 10, gmu-35-50 29 |
| `564ce0` (+`aa6825`, `55e383`×5, `39d9e1`×12) | 0x45f63c | timtab1 첫 호출 1회 |
| `3de18e` | (0x45c7d6) | **0회**. 해독만 했다 |
| `36e6b6`, `560ea1`, `562f34`, `3a0f5b` | `564ce0`/`3df916` 안 | 0회 |

#### 0. conflict graph `G = *(st+0x3f0)` (0x48바이트, `5b84b1`이 calloc)

`5b84b1(env, &st->G)`가 `calloc(1, 0x48)`로 만들고 `5b8561`이 해제한다. 리터럴 쌍 (a, b)는 "a와 b가 동시에 참일 수 없다"(x_a + x_b ≤ 1, 리터럴로)는 간선이다.

| 오프셋 | 형 | 내용 |
|---|---|---|
| +0x00 | i32 `n0` | CSR이 덮는 변수 수(마지막 `5b9aad` 때의 st.n). 0이면 CSR 없음 |
| +0x04 | i32 `rc0` | 마지막 정리 때의 `st+0x408`(제거 열 수) |
| +0x08 | i64* `ptr` | 리터럴 CSR 시작, 2·n0+1개. 리터럴 L의 이웃 = `adj[ptr[L]..ptr[L+1]]`. 전체 길이 `E = ptr[2·n0]` |
| +0x10 | i32* `adj` | 이웃 리터럴(인코딩된 i32) |
| +0x18 | i64 `npend` | 대기 쌍 수 |
| +0x20 | i64 `cap` | 대기 배열 용량 |
| +0x28 / +0x30 | i32* `pa` / `pb` | 대기 쌍 (pa[t] ≤ pb[t], 부호 있는 비교) |
| +0x38 | i32 `nflag` | flag 배열 길이 |
| +0x40 | u8* `flag` | 열마다 1이면 "다음 정리 때 그 열의 간선을 버림". `5b91ad(env, st, G, j)`가 켜고, `5b9aad`가 매번 해제한다 |

#### `5b9aad(env, st, G, p4, out: *i64, W)` — 대기 쌍을 CSR에 합치고 정리 (5,353 B)

4d3a1d는 두 번 부른다: 바깥 do-루프 맨 앞(L435, `p4=0, out=NULL`)과 끝(L4934, `p4=0, out=&local_878`). 끝에서 4d3a1d는 `local_878 − local_870 ≤ local_258/4`(C의 0 방향 정수 나눗셈) 또는 `local_188 ≠ 0` 또는 `local_c0 ≠ 0`이면 끝내고, 아니면 `local_c0 = 1`로 probing 전체를 한 번 더 돈다. 여기서 `local_870`은 성공한 `5b88ce` 추가 수다.

```
n = st.n; rc = st[+0x408]; cc = st.col_cnt (+0x68)
deg = NULL; mark = NULL; out_val = 0; ret = 0
minv = n; maxv = -1; kept = 0
// 1. 대기 쌍 걸러내기 (npend > 0일 때만)
if G.npend > 0:
    deg = calloc(2n, 4)            // 실패 → 0x2711
    for t in 0..G.npend:
        a = G.pa[t]; b = G.pb[t]; va = var(a); vb = var(b)
        keep = cc[va] >= 0 && (flag==NULL ? cc[vb] >= 0
                                          : flag[va]==0 && cc[vb] >= 0 && flag[vb]==0)
        if keep:
            G.pa[kept] = a; G.pb[kept] = b; kept += 1
            minv = min(minv, va, vb); maxv = max(maxv, va, vb)
            deg[lit(a)] += 1; deg[lit(b)] += 1
    W += (6·μ)·npend_원래            // asm: 6.0*μ 먼저, 그다음 ×개수
    G.npend = kept
// 2. CSR 크기 맞추기
if G.n0 == 0:
    if kept == 0: goto FINISH
    if flag != NULL: flag 배열을 n으로 늘리고(새 칸 0) G.nflag = n
    G.ptr = calloc(2n+1, 8)        // 실패 → 0x2711
else:
    if flag != NULL || !(n == G.n0 && G.rc0 == rc && kept == 0):
        if flag != NULL: flag 배열을 n으로 늘림, G.nflag = n
    elif flag == NULL: goto FINISH  // n 같고 바뀐 것 없음
    if G.n0 < n:                   // 새 변수 칸은 빈 리스트
        G.ptr = realloc(G.ptr, (2n+1)·8); e = G.ptr[2·n0]
        for u in 0..(n−n0): G.ptr[2·n0+1+2u] = e; G.ptr[2·n0+2+2u] = e
        W += (2·μ)·(n − n0)
G.n0 = n
// 3. 합치기: 리터럴마다 기존 이웃 뒤에 새 이웃 자리를 만든다 (kept > 0일 때)
if kept > 0:
    out_val = 2·kept
    e0 = G.ptr[2n]; f0 = e0 + 2·kept
    G.adj = realloc(G.adj, f0·4)   // f0 == 0이면 free
    if maxv + 1 < n:                       // 꼬리(변수 > maxv) 통째로 이동
        len = e0 − G.ptr[2·maxv+2]
        if len > 0: f0 -= len; e0 -= len; memmove(adj+f0, adj+e0, len)
        for v in maxv+1..n: G.ptr[2v+1] += 2·kept; G.ptr[2v+2] += 2·kept
        W += (2·μ)·(n − maxv − 1)
    v = maxv
    while v >= minv:                        // 뒤에서 앞으로
        f0 -= deg[2v+1]; len = e0 − G.ptr[2v+1]; G.ptr[2v+2] = f0     // 2v+1의 삽입 커서
        if len > 0: f0 -= len; e0 -= len; memmove(adj+f0, adj+e0, len)
        f0 -= deg[2v];   len = e0 − G.ptr[2v];   G.ptr[2v+1] = f0     // 2v의 삽입 커서
        if len > 0: f0 -= len; e0 -= len; memmove(adj+f0, adj+e0, len)
        v -= 1
    W += ((n − minv)·2.0 + (maxv − v)·4.0)·μ   // v = 루프 끝 값(= minv−1). asm: 두 항을 더한 뒤 ×μ
    for t in 0..kept:                       // 채우기(삽입 커서 = ptr[L+1])
        a = G.pa[t]; b = G.pb[t]
        G.adj[G.ptr[lit(a)+1]++] = b
        G.adj[G.ptr[lit(b)+1]++] = a
    W += (4·μ)·kept
    G.npend = 0
// 4. 정리 (p4==0 경로. p4≠0 && rc > G.rc0이면 s0 = 0)
if flag == NULL && maxv < minv: goto FINISH
s0 = (flag != NULL) ? 0 : minv
if minv <= maxv: mark = calloc(n, 1)        // 비트 1 = 양 리터럴, 2 = 음 리터럴
w = G.ptr[2·s0]
for v in s0..n:
    if cc[v] < 0 || (flag && flag[v]):
        G.ptr[2v] = w; G.ptr[2v+1] = w; continue
    for s in 0..2:
        L = 2v+s; b = G.ptr[L]; e = G.ptr[L+1]; G.ptr[L] = w
        dd = minv <= v && v <= maxv && e − b > 1
        for q in b..e:
            x = G.adj[q]; u = var(x)
            if cc[u] < 0 || (flag && flag[u]): continue
            if dd:
                bit = (x < 0) + 1
                if mark[u] & bit: out_val -= 1; continue      // 중복 간선
                mark[u] |= bit
            G.adj[w++] = x
        W += (μ·2)·(e − b)
        if dd:
            for q in G.ptr[L]..w: mark[var(G.adj[q])] = 0
            W += (μ·2)·(w − G.ptr[L])
W += (μ·4)·(n − s0)
G.ptr[2n] = w
if w == 0: free(G.ptr), free(G.adj), 둘 다 NULL
if w == 0 || s0 == 0: G.rc0 = rc
FINISH:
if out: *out = out_val
if G.ptr == NULL: G.n0 = 0
G.nflag = 0; free(G.flag); G.flag = NULL; free(deg); free(mark)
return ret
```

- 결과 리스트 순서: 기존 이웃(살아 있는 것, 원래 순서) 뒤에 새 이웃(대기 순서)이고, [minv, maxv] 범위의 길이 2 이상 리스트에서는 리터럴별 첫 출현만 남는다.
- 같은 간선이 두 번 들어오는 일이 흔하다. probing은 j를 시험해 k를 찾고, k를 시험해 j를 다시 찾는다. glass4 끝 호출에서는 kept 312, 중복 312, `*out` = 312, E = 312였다.
- `p4`가 0이면 minv보다 앞 변수의 리스트에 남은 죽은 열 항목은 지워지지 않는다(flag가 없을 때).

#### `5b86ce(env, st, G) -> bool` — conflict graph 한도 (226 B)

```
L = st.nnz(+0xd8) + 2·((n − st[+0x408]) + (m − st[+0x404]) + st[+0x1f8] + st[+0x240] + st[+0x27c] + st[+0x2f8])   // i64
lim = (PRECLIQUE(env+0x254c) < 2 && Presolve(env+0x2424) < 2) ? (f64)L·100.0 + 1e6 : (f64)L·500.0 + 1e7
return (i64)lim(절삭) < (G.ptr ? G.ptr[2·G.n0] : 0) + G.npend
```
작업량은 없다. 4d3a1d는 L4699(0x460a0a)에서 `ctx+0x1e4 == 0`이거나 이 함수가 참이면 간선 내보내기(아래 5b88ce 루프)를 건너뛴다. st+0x1f8/+0x240/+0x27c/+0x2f8은 SOS·일반 제약·QC 등의 개수로 보인다(기본 38개에서는 0이라고 추정. 확인하지 않음).

#### `5b88ce(env, st, G, a, b, nocheck, full: *i32, W) -> i32` — 대기 쌍 추가 (592 B)

```
if nocheck == 0 && 5b86ce와 같은 한도 초과:  { if full: *full = 1; return 0 }   // 추가하지 않음
if G.npend == G.cap:
    newcap = max(2·G.cap, 128)
    G.pa = realloc(G.pa, newcap·4); G.pb = realloc(G.pb, newcap·4)   // 실패 → 0x2711
    G.cap = newcap
G.pa[G.npend] = min(a, b); G.pb[G.npend] = max(a, b); G.npend += 1   // 부호 있는 i32 비교
W += μ·2.0
if full: *full = 0
return 0
```

4d3a1d 호출부(L4699–4930): 4e48f7이 낸 probe 결과 묶음(`ctx+0x3d`·8 = 후보 수 `cnt`, `ctx+0x3f`·8 = i32 경계 배열(2·cnt+1), `ctx+0x40`·8 = 리터럴 배열, 후보 열 목록)마다 다음을 한다.
- 후보 j가 살아 있고 `lb[j] ≤ 0.5`, `ub[j] ≥ 0.5`일 때:
  - 첫 구간 `[bd[2i], bd[2i+1])`의 리터럴 e마다 `5b88ce(G, −1−j, e, 0, &full)`.
  - 둘째 구간 `[bd[2i+1], bd[2i+2])`의 리터럴 e마다 `5b88ce(G, j, e, 0, &full)`.
- e의 조건은 cc[var(e)] ≥ 0이고 `lb < 0.5 < ub`(엄격)다. j의 조건(포함 부등호)과 다르다.
- full이 서면 남은 후보까지 모두 끝낸다(§2.8.5의 `goto done`, 디컴파일 L4790–4800). 성공할 때마다 `local_870`++.
- 작업량: 구간마다 `(방문 수·3)·μ`, 후보 묶음마다 `(후보 수·4)·μ`.
- 구간의 뜻(첫 구간 = x_j=0 시험의 함의인지)은 4e48f7 쪽 해독에 맡긴다.

#### `7886b(ncols, nrows, rbeg, cidx, val, cbeg, ridx, cval, W)` — CSR 전치 (638 B)

**μ를 곱하지 않는 작업량**에 주의한다. 4d3a1d L2534(0x45d4f3)에서 `(local_290 = n + GUB 열 수, local_288 = probe 행 수 + GUB 열 수, ctx[10], ctx[0xc], ctx[0xd], ctx[7], ctx[8], ctx[9], W)`로 probe 행렬의 열 사본을 만든다.

```
if ncols >= 0: memset(cbeg, 0, (ncols+1)·4)
for r in 0..nrows:                                  // 1. 개수
    for k in rbeg[r]..rbeg[r+1]: cbeg[cidx[k]+1] += 1
    W = (rbeg[r+1]−rbeg[r])·2.0 + W
W = W + (f64)nrows                                  // (nrows ≥ 1일 때, 마지막 행 뒤에)
acc = rbeg[nrows]                                   // 2. 뒤에서부터 시작 위치
for c in (1..=ncols).rev(): acc -= cbeg[c]; cbeg[c] = acc   // cbeg[c+1] = 열 c 시작
W = W + 0.0                                         // asm상 남는 카운터가 늘 0
for r in 0..nrows:                                  // 3. 채우기 (행 순서 → 열 안은 행 번호 오름차순)
    for k in rbeg[r]..rbeg[r+1]:
        p = cbeg[cidx[k]+1]; ridx[p] = r; cval[p] = val[k]; cbeg[cidx[k]+1] = p+1
    W = (rbeg[r+1]−rbeg[r])·5.0 + W
W = W + (f64)nrows·3.0
```
끝나면 cbeg[0] = 0, cbeg[c+1]은 열 c의 끝이다. nrows < 1이면 W는 `+0`만 받는다.

#### `3bea24(val: f64 [xmm0], env, st, row, col, out: *node) -> i32` — 행렬에 새 노드 (1,046 B)

```
if st.free_head(+0x1a0) == NULL:
    N = max(3000, m)
    chunk = malloc(N·32)                        // 실패 → 0x2711
    for i in 0..N−1: chunk[i].next_col(+0x18) = &chunk[i+1]; chunk[N−1].next_col = NULL
    청크 체인(st+0x1b8: 16바이트 셀 {ptr, next})의 끝에 calloc(1,0x10) 셀을 붙여 chunk 기록   // 실패 → chunk free, 0x2711
    st.free_cnt(+0x198) += N
node = st.free_head; st.free_head = node.next_col; st.free_cnt -= 1
node.row = row; node.col = col; node.val = val
node.next_col = colhead[col]; colhead[col] = node; col_cnt[col] += 1
st.nnz(+0xd8) += 1
node.next_row = rowhead[row]; rowhead[row] = node; row_cnt[row] += 1
if out: *out = node
return 0
```
작업량·추적기 표시·activity 갱신은 없다(호출자가 한다).

4d3a1d 리프팅 루프(0x461504)는 항상 `val = 0.0`(pxor)으로 부른다. 받은 노드 포인터를 열별 캐시 `local_770[col]`에 넣는다. 그다음 계수를 쌓는다.
- 양 리터럴: `*node += a`.
- 음 리터럴: `*node −= a`, `rhs[row] −= a`.
- `|*node| < 1e-10`이면 노드를 죽인다(col = −1, 두 개수 −1, `local_788`++).
- 행마다 끝에 `370047(st, env, 1, …)`(activity 재계산)과 `53763f(tr, row)`.

dump 확인: glass4·graphdraw-domain·ic97_potential·neos-4954672-berkel 둘째 호출에서 새 노드가 열 리스트 맨 앞에 붙는다. free 수 감소 = nnz 증가 = 3bea24 호출 수(16/66/97/5)다.

#### `3de18e(v [xmm0], INF [xmm1], env, st, j, rec) -> i32` — 열 고정(activity 갱신 포함) (2,681 B)

4d3a1d L769(0x45c7d1)의 호출 인자는 `(lb[j], env+0x2458, env, st, j, 1)`이다. 시작 단계에서 `ub−lb < 1e-10`인 살아 있는 열에 쓴다. **기본값 38개에서는 probing 아래 0회**다. 다른 reduction이 많이 부르는 공용 함수다(호출자 40여 개).

```
lb0 = lb[j]; ub0 = ub[j]; f = st.cflag(+0x170)[j]
if (f & 0xffffff77) == 0:                       // 0x08·0x80만 허용
    if rec:
        if st[+0xfc]==0 && st[+0x278]==0 && st.Qdiag(+0x250) && 3a0f5b(env, st) != 0: return 0x2711
        if 39ab33(v, env, st, j) != 0: return 0x2711    // 레코드 type 1, +0x408++
else: rec = 0
st.objcon(+0xc8) = obj[j]·v + objcon; obj[j] = 0
if Qdiag && Qdiag[j] != 0: objcon = ((v·0.5)·v)·Qdiag[j] + objcon; Qdiag[j] = 0
(Q 비대각 +0x258/+0x260/+0x268/+0x270: 짝 열 i의 obj[i] += Qij·v, 짝 열의 항목 삭제·당김, W += (cnt·2)·μ, 5379f1(tr,i). MIP 선형 모델에는 없다)
cflag[j] &= ~0x08
if f & 0x80: PWL 목적(+0x338 시작, +0x340 개수, +0x348 기울기, +0x350 절편, +0x358 꺾는점) 구간을 골라 obj·objcon에 반영, +0x330−−, W += ((구간 수)+3.0)·μ, cflag &= ~0x80 (기본 모델에는 없다)
if col_cnt[j] >= 1:
    53802c(tr, st, j)                            // 열 j의 행을 추적기에 표시, W += (μ·5)·방문
    for node in col j (+0x18 링크), node.col >= 0:
        a = node.val; t = v·a; i = node.row
        node.col = −1; row_cnt[i] −= 1; rhs[i] = rhs[i] − t
        if a <= 0:
            if ub0 < INF: actmin[i] = addDown(actmin[i], −(ub0·a)) else infmin[i] −= 1
            if −INF < lb0: x = addUp(actmax[i], −(a·lb0)) else { infmax[i] −= 1; x = actmax[i] }
        else:
            if ub0 < INF: actmax[i] = addUp(actmax[i], −(ub0·a)) else infmax[i] −= 1
            if −INF < lb0: actmin[i] = addDown(actmin[i], −(a·lb0)) else infmin[i] −= 1
            x = actmax[i]
        actmax[i] = addUp(x, t); actmin[i] = addDown(actmin[i], t)
    W += (col_cnt[j]·7)·μ                        // 방문 수가 아니라 col_cnt
    if !rec: col_cnt[j] = 0; 5379f1(tr, j); goto SET   // (col_cnt > 0이므로)
col_cnt[j] = rec ? −2 : col_cnt[j]               // rec가 없고 col_cnt < 1이면 그대로
SET: lb[j] = v; ub[j] = v; return 0
```
- `actmax/infmax` = +0x90/+0x70(max activity − rhs), `actmin/infmin` = +0x98/+0x78.
- `addUp(p, q)`: s = p+q가 정확하지 않으면(|p|<|q|이면 s−q≠p, 아니면 s−p≠q) s>0 → s·1.0000000000000002, s≤0 → s·0.9999999999999998.
- `addDown`은 반대다: s<0 → ×1.0000000000000002, s≥0 → ×0.9999999999999998. (m1 §5의 `4fe49b` 규칙과 같다.)
- 노드는 열 리스트에서 빠지지 않는다. nnz(+0xd8)와 free 개수도 그대로다(지연 삭제).

**`39ab33(v [xmm0], env, st, j)` — 레코드 type 1**:
- `st+0x408`++.
- MIP 상태(`st+0xfc ≠ 0` 또는 `st+0x278 ≠ 0`): arena(`ac68d7(env, st+0x368, 0x28)`) 0x28바이트 `{i32 type=1, pad, ptr data=rec+0x18, ptr next}`에 data `{i32 j, pad4, f64 v}`(16바이트)를 담아 `st+0x370` 앞에 붙인다. 작업량은 없다.
- LP 상태: 열 j의 항목(행, 값)과 상태 코드(−4 고정 / −3 사이 / −2 ub / −1 lb), obj를 함께 저장하는 긴 레코드다. W += (방문+방문+기록·4)·μ.

**`53802c(tr, st, j)`**: `st+0xe0 == 0`(연결 리스트) 경로이고 LP 추적기 열→행 표시와 같다.
- 열 j의 살아 있는 노드의 행 r마다 플래그 바이트 `tr+0x18`[r]을 본다.
- `&3 == 0`이면 레벨 1 목록(`tr+0x48`, 길이 +0x2c / `tr+0x8c` 모드면 +0x38)에 넣는다. `&0xc == 0`이면 레벨 2 목록(`tr+0x50`, +0x30 / `tr+0x90` 모드면 +0x3c)에 넣는다.
- 비트 1/4를 켜고, 모드면 2/8도 켠다.
- W += (μ·5)·(방문한 모든 노드).

#### `3df916(tol [xmm0], env, st, j) -> i32` — 거의 고정된 열 제거(activity 갱신 없음) (1,472 B)

4d3a1d 0x45f7bd의 호출 인자는 `(FeasibilityTol env+0x2460, env, st, j)`다. 호출자가 probing으로 bound를 반올림(`ceil(plb − itol)`/`floor(pub + itol)`, itol = env+0x2468, §2.8.2)한 직후에 쓴다. 새 lb > ub + 1e-6이면 불가능이고, lb > ub − 1e-10이면 이 함수를 부른다(호출자 쪽 조건). LP 포트의 `4f6740`에 해당한다.

```
d = ub[j] − lb[j]
if tol + tol < −d: st[+0x470] = j; return 3
if d > 1e-13:
    amax = 0; cnt = 0
    for node in col j: { if node.col >= 0: amax = max(amax, |a|)  // maxsd
                         cnt += 1 }
    W += (cnt·3)·μ
    if tol < amax·d: return 0                     // 고정하지 않음
c = obj[j]
if tol·0.01 <= |d·c|: v = (c <= 0) ? ub[j] : lb[j]
else: v = (lb[j] > 0 || ub[j] < 0) ? (ub[j] + lb[j])·0.5 : 0.0
mask = 0x88; if st[+0x248] != 0: mask += (lb[j] < 0.5) ? 0x800 : 0
f = cflag[j]
if (f & ~mask) == 0:
    if f & 0x80: PWL 구간 선택(3de18e와 같은 배열), W += (구간 수)·μ, +0x330−−, obj[j]·objcon 반영
    if st[+0xfc]==0 && st[+0x278]==0 && Qdiag && 3a0f5b(env, st, j) != 0: return 0x2711
    if 39ab33(v, env, st, j) != 0: return 0x2711     // type 1 {j, v}, +0x408++
    cflag[j] = 0; c = obj[j]
objcon = c·v + objcon; obj[j] = 0
if Qdiag[j] != 0: objcon = ((v·0.5)·v)·Qdiag[j] + objcon; Qdiag[j] = 0
(Q 비대각: 짝 열 obj += v·Qij, 5379f1(tr,i), 항목 당김; W += (찾은 위치)·μ, (당긴 수·2)·μ, 끝에 (cnt·5)·μ)
53802c(tr, st, j)                                    // W += (μ·5)·방문
cnt = 0
for node in col j: { if node.col >= 0: { rhs[row] = rhs[row] − v·a; row_cnt[row] −= 1; node.col = −1 }
                     cnt += 1 }
W += (cnt·6)·μ
col_cnt[j] = (cflag[j] == 0) ? −2 : 0
return 0
```
- activity(+0x70–+0x98)는 건드리지 않는다. lb = ub = v이면 (activity − rhs)가 그대로라서 맞는다.
- lb[j]·ub[j]도 쓰지 않는다(호출자가 이미 맞춰 두었다).
- gmu-35-40에서 열마다 ΔW = 11·col_cnt였다. d = 0이라 첫 루프가 없고, 5(53802c) + 6을 더한 값이다. 레코드는 `{j, pad, 0.0}`, col_cnt −2, +0x408 +1이 dump와 맞는다.

#### `564ce0(st, env, map, sc, of, updAct, out: *i32) -> i32` — 열 대입 x_j = sc_j·x_rep + of_j 묶음 적용 (3,462 B)

4d3a1d 0x45f637의 호출 인자는 `(st, env, local_6d0, ctx[0x2a], ctx[0x2b], 1, &local_6a8)`다.
- `map[j]` = 대표 열(−1이면 없음). 4d3a1d는 고정 열(ub−lb<1e-10)을 −1로 지운 뒤 부른다.
- `sc`·`of`는 열마다 double이다.
- timtab1 첫 호출에서 받은 map 12개는 모두 대표로 바로 간다(대표의 map은 −1). 받은 값은 x_j = x_k와 x_j = 1 − x_k(sc −1, of 1)다.
- 공용 함수다(5466a7, 5577ba, 522aa2 등도 부른다).

```
Q = (env+0x2584 PREQSUBST == −1) ? 2 : PREQSUBST
// 1. 경로 압축 (작업량 μ 없음)
for j in 0..n:
    k = map[j]; if k < 0: continue
    s = sc ? sc[j] : 1.0; o = of ? of[j] : 0.0; steps = 0
    while map[k] >= 0 && map[k] != k && map[k] != j:
        q = map[k]; r = map[q]; steps += 1
        if r < 0 || r == q:      next = q; compose(k)        // q가 뿌리
        elif r == k:             map[q] = −1; next = map[k]; compose(k)   // 2-순환 끊기
        elif r == j:             next = q; compose(k)
        else:                    // k를 r로 바로 잇는다
            if sc: of[k] = of[q]·sc[k] + of[k]; sc[k] = sc[k]·sc[q]
            map[k] = r; next = r; if sc: compose(k)
        k = next
    W = steps·4.0 + W
    map[j] = k; if sc: sc[j] = s; of[j] = o
    if map[k] == j: map[k] = −1
  // compose(k): o = o + of[k]·s (곱 먼저); s = s·sc[k]
W = (f64)n·3.0 + W
// 2. 후보 모으기
cnt = 0; bSOS = bQ8 = bQ30 = false
for j in 0..n:
    r = map[j]; if r < 0: continue
    f = cflag[j]
    if (f & 0xffffffc6)==0 && col_cnt[j] >= 0 && col_cnt[r] >= 0 && (Q != 0 || (f & 0x38)==0):
        s = sc ? sc[j] : 1.0; o = sc ? of[j] : 0.0
        if sc && o != 0.0 (NaN 포함) && ((f & 1) || (Q < 2 && (f & 0x30))): map[j] = −1; continue
        bSOS |= f&1; bQ8 |= f&8; bQ30 |= f&0x30
        R[cnt] = r; J[cnt] = j; S[cnt] = s; O[cnt] = o; cnt += 1
    else: map[j] = −1
W = (n·3.0)·μ + W
if cnt == 0: { *out = 0; return 0 }
// 3. 대표 기준 정렬: sc 있으면 aa6825(cnt, R, J, S, O), 없으면 a9dcac(cnt, R, J).
//    키 R만 본다. crates/lp sort.rs의 aaedd9와 같은 알고리즘(≥100이면 3-way quicksort, 아니면 Shell n/2, ÷2.2)이고
//    J·S·O가 함께 움직인다(timtab1 순서 238,384,243 / 248,244,254 / 262,373,264를 시뮬레이션으로 재현).
// 4. 그룹마다 대입
acc = 0; g0 = 0
while g0 < cnt:
    g1 = g0; while g1 < cnt && R[g1] == R[g0]: { acc += col_cnt[J[g1]]; g1 += 1 }
    if st.nnz(+0xd8) < acc: { 36e6b6(st, 1); acc = 0 }          // GC (inner 쪽 노트)
    rc = 55e383(st, env, R[g0], g1−g0, &J[g0], sc ? &S[g0] : NULL, sc ? &O[g0] : NULL, updAct)
    if rc: { *out = cnt; return rc }
    g0 = g1
W = (cnt·3.0)·μ + W
// 5. (bSOS) SOS 집합(st+0x1f8개, +0x208 type, +0x210/+0x218 구간, +0x220 원소): type 1 집합 원소의 cflag bit1을 지운다.
//    원소를 map으로 바꾸고 bit1을 다시 켠다. st+0x3a8 카운터로 중복을 찾는다. 두 번 나온 원소는 lb ≤ st+0x1e8, ub ≥ −st+0x1e8이면
//    3de18e(0.0, …, j, rec=0)로 0 고정(이미 0/0이면 생략), 아니면 st[+0x480]=집합, return 3.
//    작업량은 (원소 수·2)·μ와 ((집합 수)+(집합 수))·μ. 기본 모델에서는 0회.
// 6. bQ8 → 560ea1(st, env, 0, map, sc, of); bQ30 → 562f34(...) (Q 관련, 기본 모델에서 0회)
*out = cnt; return 0
```

**`55e383(st, env, k, cnt, J, S, O, updAct)` — 그룹 대입 (2,034 B)**. `pos = st+0x1b0`은 행마다 노드 포인터 칸이다. 호출 사이에 모두 NULL이어야 한다.

```
for node in col k, node.col >= 0:
    pos[row] = node
    if updAct: remove(row, a, lb[k], ub[k])
W += (col_cnt[k]·3)·μ
for t in 0..cnt:
    j = J[t]; s = S ? S[t] : 1.0; o = O ? O[t] : 0.0
    node = colhead[j]; colhead[j] = NULL
    while node:
        nxt = node.next_col; node.next_col = NULL
        if node.col < 0: { node.next_col = colhead[j]; colhead[j] = node; node = nxt; continue }   // 죽은 노드는 j에 남김(역순)
        i = node.row; a = node.val
        if updAct: remove(i, a, lb[j], ub[j]); actmax[i] = a·o + actmax[i]; actmin[i] = a·o + actmin[i]
        rhs[i] = rhs[i] − o·a
        if pos[i] == NULL:          // 행 i에 k가 없음 → 노드를 열 k로 옮김(행 리스트 위치는 그대로)
            pos[i] = node; node.next_col = colhead[k]; node.col = k; node.val = s·a
            colhead[k] = node; col_cnt[k] += 1
        else:                       // 이미 있음 → 합치고 j 노드는 죽임
            node.col = −1; node.next_col = colhead[j]; colhead[j] = node; row_cnt[i] −= 1
            pos[i].val = s·a + pos[i].val
        node = nxt
    W += (col_cnt[j]·12)·μ          // col_cnt[j]는 아직 원래 값
    c = obj[j]; col_cnt[j] = −3; st[+0x408] += 1
    obj[k] = c·s + obj[k]; objcon = obj[j]·o + objcon; obj[j] = 0
    39d9e1(s, o, env, st, j, k)     // 레코드 type 2
for node in col k, node.col >= 0:
    i = node.row; pos[i] = NULL; a = node.val
    if |a| <= 1e-10: node.col = −1; col_cnt[k] −= 1; node.val = 0.0; row_cnt[i] −= 1
    elif updAct: add(i, a, lb[k], ub[k])
    53763f(tr, i)
W += (col_cnt[k]·6)·μ
5379f1(tr, k)
return 0
```
- `remove(i,a,lb,ub)`(보정 없는 단순 덧셈):
  - a>0: lb>−INF면 actmin −= lb·a, 아니면 infmin−−. ub<INF면 actmax −= a·ub, 아니면 infmax−−.
  - a<0: lb>−INF면 actmax −= lb·a, 아니면 infmax−−. ub<INF면 actmin −= a·ub, 아니면 infmin−−.
  - a==0이면 아무것도 하지 않는다.
- `add`는 `+=`와 `++`로 같은 모양이다(`actmin = lb·a + actmin` 순서). INF는 env+0x2458.
- 1e30 보정과 ulp 밀기(addUp/addDown)는 **없다**.
- timtab1에서 ΔW = 3·397 + 3·397 + 3·12 + Σ_그룹(3·cc_k + 12·Σcc_j + 6·cc_k') = 2,679로 Frida 값과 정확히 같다.

**`39d9e1(s [xmm0], o [xmm1], env, st, j, k)` — 레코드 type 2** (56c80e 집계와 같은 레코드):
- `s ≠ 1.0`이면 `st+0x80[k] = min(st+0x80[k], st+0x80[j]·|s|)`, `st+0x88[k] = max(st+0x88[k], |s|·st+0x88[j])`(열 스케일 범위로 보임. 비교는 `≤`).
- arena 0x50바이트 레코드 `{i32 type=2, pad, data=rec+0x18, next}`. data 0x38바이트는 아래와 같다.

| data 오프셋 | 내용 |
|---|---|
| +0x00 | i32 j |
| +0x08 | f64 o |
| +0x10 | i32 1 (항 수) |
| +0x18 | ptr → data+0x28 (열 배열) |
| +0x20 | ptr → data+0x30 (계수 배열) |
| +0x28 | i32 k |
| +0x30 | f64 s |

뜻은 x_j = o + s·x_k다. timtab1 dump 12개 레코드가 이 배치와 바이트 단위로 같다. 순서는 그룹 순서와 그룹 안 J 순서다.

#### 3.5의 Frida 확인

- 함수별 호출 지점·횟수(표 맨 위): `tools/gurun.sh node $S/probe/js/ptrace.js <모델> … 3de18e,3df916,564ce0,55e383,39d9e1,5b86ce,5b88ce,5b9aad,7886b,3bea24,451bed,39ab33,36e6b6,53802c`, 38개 모델(`$S/probe/run_tr.sh`, 결과 `$S/probe/tr/`).
- `5b9aad`(glass4 끝 호출): G 필드 진입/복귀(`$S/probe/js/x3.js`)가 npend 312→0, n0 0→322, E 312, `*out` 312였다. 위 의사코드로 계산한 ΔW 7,958이 실측과 같다. gmu-35-40: npend 489, `*out`=E=554. 진입 호출은 두 모델 모두 대기 0이라 바뀐 것이 없다.
- `5b88ce`: glass4 첫 호출의 쌍이 (min, max)로 저장되고 같은 쌍이 양방향에서 두 번 들어온다.
- `3df916`: gmu-35-40 10회에서 col_cnt → −2, +0x408 +1씩, ΔW = 11·col_cnt였다. 레코드 type 1 16바이트 `{j, pad, 0.0}`는 4d3a1d dump(`$S/probe/d/gmu-35-40.json`)와 같다.
- `564ce0`/`55e383`/`39d9e1`: timtab1 map 12쌍(`js/x2.js`), 그룹 순서, ΔW 2,679, type 2 레코드 12개 바이트 배치, col_cnt[j] = −3, rhs 4행 변경(1 − x_k 대입)을 확인했다.
- `3bea24`: 4개 모델 둘째 호출 dump에서 새 노드가 열 맨 앞에 붙고, free −1 = nnz +1 = 호출 수였다.
- `3de18e`: 기본값에서 probing 아래 0회라 Frida 확인은 없다(정적 해독만).

#### 3.5의 열린 질문

1. `5b88ce` 두 구간(첫 = `¬x_j` 쪽, 둘째 = `x_j` 쪽)의 정확한 뜻은 4e48f7이 경계 배열에 무엇을 넣는지에 달렸다(4e48f7 쪽 노트).
2. `st+0x1f8/+0x240/+0x27c/+0x2f8`(5b86ce 한도), `st+0x80/+0x88`(39d9e1)의 이름을 확인하지 않았다. 기본 38개에서 앞의 넷이 0인지도 보지 않았다.
3. `cflag(+0x170)` 비트의 뜻은 추정이다. 0x01 SOS, 0x08/0x30 Q 쪽, 0x80 PWL 목적, 0x800은 st+0x248과 관련된다. 기본 모델에서는 0이라 결과에 영향이 없다(timtab1 12개 모두 통과).
4. `564ce0`의 SOS 블록과 `560ea1`/`562f34`는 기본값에서 돌지 않아 개략만 적었다. 2-순환(r==k) 경로도 실측하지 않았다.


## 4. 오라클 확인

도구는 세션 스크래치(`probe/`)에 있다. 포팅 때 `tools/`로 옮길 것.
- `js/ptrace.js <model> <out> <extra.js|-> fn,…`: 4d3a1d가 도는 동안 지정 함수 호출을 복귀 주소·인자·반환값과 함께 기록한다. 4d3a1d 진입의 10개 인자와 `3cd406`의 ebx(PRELIMIT 카운터)도 남긴다. extra.js(`x_main`, `x_work`, `x_ctx`, `x_lift`, `x_451`, `x_bnd`, `x_pass`, `x_4e48f7`)가 메모리를 읽는다.
- `tools/mip_state_dump.js <model> 4d3a1d all`: 38개 모델 전부(`d/`). 요약은 `py/diff.py`, `py/tsum.py`, `py/wseg.py`(작업량 구간).
- 재현기: `py/simctx.py`(§2.3–2.5·§3.1), `py/sim48.py`(§3.4), `py/simlift.py`(§2.8.4).

### 4.1 무엇이 바뀌나 (상태 덤프, 38개 × 모든 호출)

| 모델 | 호출 | 바뀐 것(진입 → 복귀) | PRELIMIT k(ebx+1) / sweep |
|---|---|---|---|
| gmu-35-40 | 1 (첫 지점) | ub 10 → 고정 10, 레코드 type 1 ×10, +0x3f8 +10, +0x408 +10, 행 activity 35 | 155: cols−10 nz−102 |
| gmu-35-50 | 1 | 고정 29, type 1 ×29 | 148: cols−29 nz−355 |
| timtab1 | 1 | 집계 12(type 2 ×12, 열 −12), lb 1·ub 4, rhs 4, +0x3f8 +5 | 97: cols−12 bnd5 rhs4 |
| tr12-30 | 1 / 2(마무리) | ub 20 / 계수 1·rhs 1(+0x3fc +1) | 134: bnd20 / 209: coef1 rhs1 |
| ic97_potential | 1 / 2 | lb 2 / 새 nz 97·수정 1·rhs 35(+0x3fc +98) | 97: bnd2 / 99: coef1 nz+97 rhs35 |
| glass4 | 2 | 새 nz 16(+0x3fc +16) | 76: nz+16 |
| graphdraw-domain | 2 | 새 nz 66 | 92: nz+66 |
| neos-4954672-berkel | 2 | 새 nz 5 | 136: nz+5 |
| 나머지 30개 모델의 모든 호출 | | 바뀐 것 없음(W만 증가) | |

- k: Frida로 4d3a1d 진입 때 `3cd406`의 ebx(카운터)를 읽었다. 호출 뒤 ebx+1이 PRELIMIT과 비교된다. sweep에서 바뀐 k = ebx+1과 위 8개 모델 11곳이 모두 맞는다.
- 다른 sweep 변화(예: gmu-35-40 52:[bnd12])는 probing이 아니다(그 k에서 4d3a1d가 안 불림).

### 4.2 경로 확인 (`ptrace.js`, 38개 모델, 61호출)
- p6 = 2(모든 호출), 첫 지점 (p7..p10) = (0,0,0,0), 마무리 = (1,1,1,1). 마무리 호출은 assign1-5-8, exp-1-500-5-5, glass4, graphdraw-domain, ic97_potential, mad, mik-250-20-75-4, neos-3046615-murg, neos-3627168-kasai, neos-3754480-nidda, neos-4954672-berkel, neos-911970, neos17, neos5, ran14x18-disj-8, sp150x300d, supportcase26, timtab1, tr12-30.
- st+0xf8 = 0, st+0x3c8 = NULL, PREGUBPROBE = −1, env+0x2728 = 0, itol = 1e-5: 모든 호출. → GUB 안 돎, lifting 점수는 LP 해 없는 식.
- 불린 보조: `5b9aad`(패스마다 2), `396a2d`(행마다), `7886b`, `4e48f7`, `5b86ce`(패스마다 1); 조건부 `3df916`(gmu), `564ce0`(timtab1), `370047`, `451bed`·`3bea24`(마무리 lifting), `5b88ce`. 한 번도 안 불린 것: `3de18e`, `397110`, `4f0f39`, `456f1d`, `adf891`, `a85f71`, `a86425`, `a7d68e`, `a7d7cf`, `a7d9ff`, `6138a4`, `ac7d4e`, `ac761d`.
- 후보 수: 열 'B' 중 colcnt > 0 + 행 후보. cod105 2048(행 1024), 나머지는 열만이거나 적은 행.
- `4e48f7` 출력 ctx+0xe8(중단) = 1: bppc4-08(1454 중 244 처리), cod105(2048 중 48), csched007(794/1457), csched008(907/1284), mad(197/201), neos-4338804-snowy(965/1260), neos-860300(22/1248), neos-911970(795/840), neos17(200/300), gmu-35-40/50 두 번째 호출.
- 작업량: §2.9.
- **bound 반영 재현**: `4e48f7` 복귀 때 plb/pub를 읽어 §2.8.2 식(ceil(plb − itol)/floor(pub + itol), 1e-6 판정)을 적용하면 tr12-30(20), ic97_potential(2), timtab1(5), gmu-35-40(10, 고정 10)의 복귀 lb/ub와 +0x3f8 증가가 같다.
- **probing 행렬 재현**(`simctx.py`): §2.3–2.5와 §3.1(정렬은 LP 포트 `sort_desc`를 그대로 옮김)을 진입 덤프에서 Python으로 돌려, `4e48f7` 진입 때의 ctx(후보 목록, rbeg, rnbend, rcol, rval, rowrhs, rowsense, pmin, pmax, prange, cbeg, crow, cval)와 비트 단위로 비교했다. assign1-5-8, glass4, bppc4-08(행 길이 최대 331, quicksort 경로), neos-860300(692, μ ≠ 1), cod105(행 후보 1024), mik-250-20-75-4(비이진 70), tr12-30, supportcase26 모두 같다.
- `451bed`(probing 안, glass4 마무리): ncl = 36, 모든 csense '=', cmem·jcl 모두 ≥ 0, jbeg는 열 번호 순 누적. tr12-30·ic97_potential·neos-4954672-berkel은 ncl = 0이다.
- **`4e48f7` 재현**(`sim48.py`, §3.4 J): 35개 모델 58회 전부에서 cand 순서, lb/ub, actmin/actmax, sense, 집계, 함의 묶음, 중복 행 기록, 카운터, 작업량이 0 차이로 같다.
- **보조 함수**(§3.5의 Frida 확인): `5b9aad` 작업량(glass4 7,958), `564ce0`(timtab1 2,679, type 2 레코드 바이트), `3df916`(gmu 11·col_cnt), `3bea24`(free −1 = nnz +1).
- **2패스 판정**: ic97_potential 202 > 112 → 2패스, gmu-35-40 65 ≤ 228 → 1패스(§2.8.6).

## 5. 열린 질문

1. 마무리 호출 가드의 5% 판정 분자·분모(st+0xf4, st+0x408의 어느 시점 값)를 spine-d 노트와 맞출 것. 38개의 호출 여부(19개)는 cov와 맞는다.
2. T8/T9 타이머 한도 식(`3cd406` 0x3b1435/0x3b4484 경로)은 spine 몫이다. 마무리 호출의 예산이 이 한도 − 경과로 정해지므로(19개 중 다수), 포트는 타이머 상태를 정확히 들고 와야 한다.
3. `aae2ff`의 quick 부분(한 행의 비이진 항목이 100개 이상일 때)을 확인하지 못했다. 8개 모델 재현에서 비이진 항목은 최대 70개였다. `aabbe3`의 quick 경로(이진 100개 이상)는 bppc4-08·neos-860300에서 맞았다.
4. lifting의 clique 소속 부호 비교(§2.8.4 주의 1)는 원본 동작을 그대로 적은 것이다. 음의 소속(jcl < 0)이 있는 모델에서 확인하지 못했다(glass4·graphdraw-domain은 모두 ≥ 0).
5. env+0x2728(함의 모음·중복 행 기록 스위치)과 env+0x2cd0(`4e28ef` 난수 시드)은 파라미터 표에 없다. 둘 다 기본 0이다. 0x2cd0이 Seed를 따르는지 Seed=1로 볼 것.
6. `4e48f7`의 1e7 점검이 부르는 `adf18e(st+0x108)`(= `*(st+0x128) = 0`)의 뜻. cov2에서 neos-860300만 1회 불렸다(그 모델의 `4e48f7` 작업량이 1e7을 넘음).
7. 기본값 38개에서 안 지나간 경로(asm만 보고 씀): 두 시나리오 모두 모순(nf = 0), 목적 컷오프 기각(Model.presolve는 st+0x490 = 1e100), `<`→`=` 행, 무한 bound 하나 경우의 실제 강화, 활동량 재계산(무한 전환·1e10 이상), 암시 용량 초과, 1e7 점검 중단, `3de18e`(거의 고정 열), selective 행 선택, GUB, SOS, QC, 스레드.
8. `5b86ce` 한도 식의 st+0x1f8/+0x240/+0x27c/+0x2f8와 `39d9e1`이 고치는 st+0x80/+0x88(열 스케일 범위로 보임)의 이름.

## 6. 포팅 상태 (2026-11-06, part A)

상태를 읽기만 하는 부분을 옮겼다. 상태 쓰기(집계 `564ce0`, bound 반영, `3df916` 고정, lifting `3bea24`, `probe` stub 교체)는 part B다.

| 파일 | 내용 |
|---|---|
| `crates/mip/src/conflict.rs` | conflict graph(`5b84b1`의 0x48 구조체): `merge`(`5b9aad`, p4 = 0), `push`(`5b88ce`), `limit`(`5b86ce`의 한도) |
| `crates/mip/src/probe.rs` | `Ctx`(0x228 구조체), `State::probe_build`(§2.3–2.7: 행 선택, 열 개수, 후보, 행 정렬 `396a2d`, activity, 전치 `7886b`, 예산), `State::probe_export`(§2.8.5 함의 내보내기), `sort_desc`(`aabbe3`/`aae2ff`), 오라클 적재 `State::load_rows` |
| `crates/mip/src/probe_engine.rs` | `State::probe_engine`(`4e48f7`)과 `4e28ef`, `4eba40`, `4ed528`, `4eb08d`, `4e7515`/`4e7b3e`/`a7d0ab`, IntSet, 난수 `a7b54f` |

- 작업량은 `w: &mut f64`로 받아 더한다. 상태 필드는 쓰지 않는다. `3ba9de`의 예산 초과는 `Stop::Done` / `EngineEnd::Budget`으로 알리고, PRELIMIT −1은 호출자가 쓴다.
- 거의 고정된 열(`3de18e`)은 `Stop::NearFixed(j)`로 돌려준다. part B에서 후보 루프 안에서 부른다(작업량 덧셈 순서 때문).
- 확인: `tools/probe_trace.js <model> <out.json>`(모든 4d3a1d 패스의 시작 상태·conflict graph·ctx 입출력·함의 내보내기 전후)과 `tools/probe_check.py <dir>`(py 바인딩 `mip_probe_pass`로 패스마다 다시 돌려 비교). 38개 모델, 61호출, 62패스 전부 같다: merge 뒤 graph와 W, ctx 전체(후보 순서, CSR, activity, range, 예산), 엔진 출력(bound, activity, sense, 집계, 함의 묶음, 중복 행 기록, 카운터, 중단 표시, 마지막 f)과 W, 함의 내보내기 뒤 pending과 마지막 merge의 graph·`*out`·W.
- 비교에서 ctx+0x150/+0x158(aggmul/aggoff)은 집계된 열만 본다. +0x150은 `396a2d`의 임시 버퍼로도 쓰이고 +0x158은 초기화되지 않는다.
- Frida 덤프가 가끔 멈춘다(gmu-35-50, neos-4338804-snowy에서 한 번씩, 다시 돌리면 정상).

## 7. 포팅 상태 (2026-11-06, part B)

`probe` stub을 `State::probe`(crates/mip/src/probe_apply.rs)로 바꿨다. 4d3a1d 전체가 들어갔다.
- 패스 루프와 시작 merge, 후보 루프 안의 거의 고정 열(`fix_col`, 레코드 type 1), 집계(`subst.rs`의 `substitute` = `564ce0`/`55e383`/`aa6825`, `push_agg`로 type 2), bound 반영(정수 ceil/floor, `fix_near`), `=` 행, lifting(`451bed`는 clique 쪽 `clique_table(None, true)`, 새 노드 `new_node` = `3bea24`), 함의 내보내기와 끝 merge, 2패스 판정.
- conflict graph는 `State::cg`(st+0x3f0)에 둔다. 타이머는 `driver::Timer::acc()/running()`으로 `timed(9|8)` 안에서 시작된 값과 같은 view를 만든다.
- 확인:
  - `tools/probe_apply_check.py <state_dir> <trace_dir>`: `mip_state_dump.js`의 4d3a1d 진입 상태와 `probe_trace.js`의 graph·타이머로 호출 전체를 돌려 복귀 상태(bound, rhs, sense, 개수, activity, 행·열 리스트, 추적기, 카운터, free 수, work, 새 레코드 바이트)를 비교한다. 13호출(gmu-35-40/50 고정, timtab1 집계, tr12-30·ic97 bound와 lifting, glass4·graphdraw-domain·neos-4954672-berkel lifting, assign1-5-8·cod105·fhnw-binpack4-4) 전부 같다.
  - `tools/probe_check.py`: 62/62 패스 그대로 같다.
  - `tools/sweep_check.py`: 첫 차이가 `probe`인 모델은 없다(glass4는 k=76 lifting을 지나 k=93 `dominated`에서 갈라진다). default 일치는 9/38 그대로다.
