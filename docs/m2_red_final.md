# M2 MIP presolve: 마무리 단계 reduction 포팅 노트 (`m2-final`)

[m2_spine_d.md](m2_spine_d.md)(마무리 사슬의 호출 위치·가드·카운트, §5.2 쌍대 고정, §5.3 implied int 척도)에 없는 해독만 적는다. 2026-11-06, 디컴파일 + objdump(libgurobi130.so 13.0.3), 주소는 file offset.

## 0. 상태

| 조각 | 코드 | 확인 |
|---|---|---|
| inline implied int 척도 (§5.3) | `finalinl.rs` `implied_int_scale` | 덤프 38/38(앞뒤 덤프, §6), sweep gen-ip054·crna·nidda·neos-911970·neos-860300 끝까지 일치, 레코드 type 9 24개(neos-911970) 일치 |
| inline 쌍대 고정 (§5.2) | `finalinl.rs` `dual_fix_empty_cols` | 덤프 38/38 + 합성 `syn_dfix` 2/2(실제로 고정하는 경우), 레코드 일치, pytest |
| `57999f` 작은 계수 | `finalinl.rs` `r57999f` | 덤프 38/38 |
| `470936` sparsify | `sparsify.rs` | 덤프 38/38, neos-860300 sweep 끝까지 일치 |
| `52cba4` 연결 성분 | `components.rs` | 덤프 38/38. 하위 풀이 `5307f4` 미포팅(shortcut, §3) |
| `521497` biconnected | `components.rs` | 덤프 38/38. 블록 탐색·`522aa2` 미포팅(shortcut, §3) |

sweep(기본값 38개): **33/38**(이전 28/38). 남은 첫 불일치는 `knapsack` 2(gmu), `mod_k`(enlight_hard), `ss2mip`(ran14x18), `probe0`(tr12-30)로 모두 이 담당 밖이다.

## 1. implied int 척도

spine-d §5.3 의사 코드 그대로 옮겼고 그대로 맞았다. 덧붙일 것:
- `a7bea8`은 `iint.rs`의 `gcd_f64`(57627e 포팅 때 옮김)를 그대로 쓴다. spine-d §8.4의 걱정(비트 일치)은 5개 모델 덤프·sweep 일치로 해소.
- `row_scale`(B)의 "첫 x = 0 열"과 A의 피벗 열은 같은 계산이라 `iscale_ratios` 하나로 둔다. 0x3bc433 경로가 c2를 먼저 q2에 넣는 것과 계산 뒤 넣는 것은 결과가 같다.
- spine-d §8.3(B가 실제로 x를 정했는지)은 따로 재지 않았다. 포트는 A·B 모두 돌고 결과가 맞는다.

## 2. `470936` sparsify (0x419650, 13KB)

인자 `(st, env, T20, a4, a5)`. a4 = mipctx→+0x620→+0x7fc, a5 = →+0x808이고 `Model.presolve()`에서는 0. W = st+0x458, μ = st+0x448, W0 = 진입 때 W.

```rust
if PreSparsify < 0 {                                   // 자동: 시험 실행으로 판정
    if !(m > 9 && n > 9 && st+0xfc != 0) { return 0; }
    // 살아 있는 열(col_cnt > 0): 개수 na, 이진 nB, 서로 다른 col_cnt 값의 수 nd(calloc(m+1) 히스토그램)
    if nd < 10 || (a4 + a5 == 0 && nB < na) { return 0; }          // a = 0이면 살아 있는 열이 전부 'B'여야 한다
    // nr = #{row_cnt > 0}, z = Σ row_cnt(>0)
    if z < 40·nr || z < 40·na || 10·na < nr || (a5 < 3 && 10^a5·1e6 < nr) || 2e9 < 1.5·z { return 0; }
    removed = trial()?;                                 // 아래. 풀 넘침(1)이면 return 0
    if removed < 0.5·z || (a5 == 0 && removed < 0.8·z) { return 0; }
}
if adf2be(T20, W) { return 0; }
r = 3ba9de(env, st, NULL); if r != 0 || st.prelimit < 0 { return r; }
// st+0xe0(CSR 상태)는 MIP에서 0. 그때 슬랙 없는 경로는 PreSparsify < 1 ∧ nbin(+0xf0) ≥ n ∧ nnz(+0xd8) ≥ 1e7뿐
slacks = PreSparsify >= 1 || nbin < n || nnz < 1e7;
if slacks { for i: if row_cnt[i] > 9 && sense[i] != '=' { k += 1; inf_max(+0x70)[i] += 1; } }
3bbf87(env, st, k);                                    // add_cols, k = 0이어도 부른다
if slacks { for i (같은 조건): free 리스트 노드 → (i, n0+t, 1.0)을 행·열 머리에, row_cnt[i]++, col_cnt = 1, nnz++, sense[i] = '='; }
cnt = calloc i32[m], mult = f64[m], pos = st+0x1b0(열마다 노드 포인터)
wchk = W0
for i in 0..m (원래 m) where row_cnt[i] > 9 && sense[i] == '=' {
    if adf2be(T20, W) { break; }
    if 1e7 < W − wchk { 3ba5ac, 3ba9de (≠0 → return), prelimit < 0 → return 0(슬랙 제거 없이); wchk = W; if budget(+0x1f0)/5 < W − W0 { break; } }
    minabs = min |a| over 살아 있는 노드(초기 1e100, `<=`로 갱신); 행이 있고 minabs < 1e-13이면 다음 행
    tol = max(1e-10 / minabs, 1e-6)
    // 1단계: 행 i의 살아 있는 노드 (i,k,a)마다 열 k의 살아 있는 노드 (r,k,b): ratio = (−b)/a
    //   cnt[r] == 0: mult[r] = (1e4 ≤ |ratio| ∨ |ratio| ≤ tol) ? 0 : ratio, cnt[r] = 1
    //   tol < |ratio|: |ratio − mult[r]| < 1e-10 → cnt[r]++, 아니고 |ratio| < |mult[r]| → mult[r] = ratio, cnt[r] = 1
    //   그 밖: mult[r] = 0, cnt[r] = 0
    //   열마다 W = (col_cnt[k]·10.0)·μ + W
    // 2단계: 같은 순회. r ≠ i ∧ cnt[r]·(−2) + row_cnt[i] ≤ −max(10, row_cnt[i]/2)이면
    //   W = ((row_cnt[i] + row_cnt[r])·5.0)·μ + W
    //   (39dded type 22 레코드는 st+0xfc == 0 ∧ st+0x278 == 0, 즉 LP일 때만. MIP는 레코드 없음)
    //   pos: 행 r 노드 열 = 0, 행 i 노드 열 = 노드. 행 r 노드마다 pos가 있으면 v = pos.val·mult + x.val,
    //   |v| < 1e-10이면 row_cnt−1, col_cnt−1, 노드 열 −1(nnz는 그대로). 남은 행 i 노드는 free 노드로
    //   (r, 열, y.val·mult)를 열·행 머리에(row/col_cnt+1, nnz+1). rhs[r] = mult·rhs[i] + rhs[r]; 370047(st, env, 1, &r)
    //   방문한 (r,k)마다(조건과 무관) cnt[r] = 0
}
for j in n0..n0+k {                                    // 손대지 않은 슬랙 제거
    if col_cnt[j] == 1 && 첫 살아 있는 노드 val == 1.0 && sense[row] == '=' {
        row_cnt−1, col_cnt = −1, sense = '<', col 머리 NULL, inf_max[row]−1, 노드 열 −1 → pending 리스트(+0x190/+0x188),
        cols_removed(+0x408)+1, nnz−1          // 레코드 없음
    }
}
// "Sparsify removed %d nonzeros" 로그, st+0xe0 ≠ 0이면 510953. return 0
```
시험 실행(0x419c2f–0x41ae82): 살아 있는 행렬을 자체 풀(⌊1.3·z⌋ 노드)에 복사한다. 행 안은 열 순서, 열 리스트도 열 순서(열 리스트를 따라 채움). 행 길이 len = max(row_cnt, 0). 본 루프와 다른 점: 행 조건이 `len[i] > 9`뿐('<' 행도), 1단계에 tol이 없고(|ratio| ≥ 1e4만 0), 2단계 조건에 sense[i] ≠ '='이면 +1, 0 판정이 `|v| ≤ 1e-10`, work 식이 `(μ·10.0)·col_cnt[k]`와 `(μ·5.0)·(len_i + len_r)`(곱 순서가 본 루프와 반대, asm 0x41a9b7·0x41aabc), budget/5 검사 없음, 새 노드는 카운터가 z+1부터 `< cap`일 때만(넘치면 1 → return 0), rhs·activity는 건드리지 않는다. 결과 removed = z − Σ max(len, 0).

포트 메모:
- free 노드는 `setpack.rs`의 `add_node`(469c12와 같은 순서)를 같이 쓴다. free가 비면 `3b85ea`(풀 늘리기)가 필요하지만 다른 포트처럼 assert로 둔다.
- 38개 중 실제로 바뀌는 모델은 neos-860300뿐이다(행 32개에 남은 슬랙이 이후 implied int 척도의 type 9 열이 된다).

## 3. `52cba4`, `521497`

- `52cba4(st, env, T21, &ncomp)`: st+0x240 > 0(PWL)이면 ncomp = 0으로 끝. st+0x310(NL)이 있으면 `b832dc`/`b6fd6f`. 버퍼를 잡고(st+0x228 > 0이면 `585a6e`) `52da19(st, env, &nc, colbeg, cols, rowbeg, rows, size, W)`로 성분을 찾는다. 선형 모델의 `52da19`는 행 번호순으로 살아 있는 행에서 BFS(행 리스트 → 열 → 열 리스트 → 행)를 하고 크기 = Σ row_cnt(2e9 상한)를 적는다. work는 일반 제약·SOS·Q 분기에서만 쌓인다. nc > 1이면 크기순 정렬(`a9dcac`) 뒤 마지막(가장 큰) 성분 앞까지 `5307f4(st, env, ncols, cols, nrows, rows, map, &status, &solved, T21, W)`로 성분을 따로 풀고, 푼 성분의 열 수를 ncomp에 더한다(마지막 성분은 앞에서 실패한 성분이 있고 크기가 전체의 절반 미만일 때만 푼다). PRECONNECTED = −1이면 status ≠ 2에서 멈춘다.
- 오라클: `52da19`의 nc(OUT_I32=2)는 37개 모델에서 1, enlight_hard 0. 그래서 `5307f4`는 한 번도 불리지 않고 `52cba4`의 작업량 변화는 0이다. 포트는 `ncomp = 0`만 둔다(shortcut).
- `521497(st, env, T22)`: st+0x1f8(SOS)·+0x330·+0x27c(QC)·+0x240·+0x2f8 > 0이거나 st+0x310 ≠ 0이면 곧바로 0. 아니면 열-행 그래프의 블록(절점 분해)을 찾고 조건에 맞는 블록마다 `522aa2`(→ `98b671` 풀이)를 부른다. 상태를 직접 쓰는 곳은 없다. 38개 모델에서 `522aa2` 0회, 작업량 변화 0이라 포트는 0만 돌려준다(shortcut). 블록 조건(L590–770)은 해독하지 않았다.

## 4. `57999f` 작은 계수 (0x4bef60)

`W = nnz(+0xd8)·μ + W`를 먼저 한다. tol = min(1e-6, ft)(ft ≤ 1e-6이면 ft). 모든 행 i(죽은 행 포함, 죽은 노드는 건너뜀)에서 누적 s1 = Σ|(u−l)·a|, s2 = Σ|c·(u−l)|을 **받아들인 항목만** 더한다: 후보 t1 = |(u−l)·a| + s1 < 0.1·tol이고 t2 = |c·(u−l)| + s2 < 0.1·ot이면 row_cnt−1, col_cnt−1, rhs −= a·l, 노드 열 −1, (s1, s2) = (t1, t2). s1 > 0인 행은 `370047(st, env, 1, &i)`. nnz와 추적기는 그대로다. void라 반환값을 무시한다.

## 5. 쌍대 고정 합성 확인

38개에서는 고정이 없어서 `bench/gurobi_presolve_mip/syn_dfix.mps`를 만들었다: x ∈ [0, 1e-8], 비용 1, 행 y1 + y2 + x ≥ 1 하나에만 있다. `57999f`가 x의 계수를 지우고(|1e-8| < 1e-7) 쌍대 고정이 x = 0으로 고정한다(type 1 레코드, 그 뒤 고정 루프가 한 바퀴 더). 덤프 2/2, 레코드, presolved 모델 모두 일치(`tests/test_presolve_mip.py`).

## 6. 도구

- `tools/mip_state_dump.js`: `RET=<hex>`로 그 반환 주소에서 부른 호출만 세고 덤프한다. 인라인 블록은 앞뒤 호출을 이것으로 고정한다: 쌍대 고정 = `36ddf2`@`3b9ed4`(복귀) → `3ba5ac`@`3ba1dd`(진입), implied int = `3ba5ac`@`3ba1dd` → `3ba5ac`@`3bb9a8`. `mip_dump_check.py <fn> before.json --after after.json`.
- `tools/mip_dump_check.py`: 앞뒤 덤프는 다른 프로세스라 죽은 행(row_cnt < 0, 양쪽 모두)의 activity가 쓰레기 값으로 달라진다(neos859080 행 123). 이 경우만 비교에서 뺀다. 덤프 이름 `implied_int_scale`, `dual_fix_empty_cols`, `470936`, `57999f`(void), `52cba4`, `521497`.

## 7. 미확인

1. `5307f4`(성분 하위 MIP 풀이)와 `521497`의 블록 탐색·`522aa2`: 38개에서 안 돈다. 성분이 둘 이상인 모델이 들어오면 포팅해야 한다.
2. sparsify에서 free 노드가 바닥나는 경우(`3b85ea`)와 슬랙 없는 경로(nbin ≥ n ∧ nnz ≥ 1e7)는 오라클로 본 적 없다.
3. sparsify의 `adf2be`(T20)·1e7 work 폴링·budget/5 중단은 해독대로 넣었지만 38개에서 발동하지 않는다.
