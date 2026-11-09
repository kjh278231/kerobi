# M1 LP presolve: `5b488d` (GURO_PAR_PREDERIVED) 해독 노트

[m1_presolve.md](m1_presolve.md) §7 7단계다. 2026-11-05에 의사코드(`re/decomp/c/004dbf90_PRIVATE00000000005b488d.c`, 1,618줄)와 asm으로 해독했고, PRELIMIT 오라클과 uncrush 오라클로 확인했다. 코드는 `crates/lp/src/presolve/derived.rs`다. asm 주소는 파일 오프셋이다(ghidra 주소 − 0x100000, 예: `LAB_005dd7ad` = 0x4dd7ad).

## 0. 한 줄 요약

열 j의 행들을 모두 만족하는 x_j 값이 이웃 열이 bound 안에서 무엇을 하든 늘 있으면, j와 그 행들을 함께 지운다. j의 비용은 `=` 행 하나로 이웃 열에 넘긴다. postsolve는 type 24 레코드로 x_j를 되찾는다(`361ab1`, [m1_postsolve.md](m1_postsolve.md) type 24 절).

## 1. 호출과 상수

- 호출 조건: PREDERIVED(기본 1) && DualReductions && PreCrush==0. 모든 패스의 집계 뒤에서 돈다. PRELIMIT 카운트는 호출마다 1이다.
- 반환값은 0이다. 작업 배열 할당이 실패하면 10001이다.
- 무한 판정은 INFBOUND가 아니라 **상수 1e30**이다. 무한 기여 표시는 ±1e100이다. 허용치는 1e-10(비용), 0.001(기준 행 |a|), 1e-6(검사)이다.
- 작업량은 `*(+0x458)`에 바로 더한다. **μ(+0x448)를 곱하지 않는다**(함수에 +0x448 참조가 없다).
- 추적기 표시를 하지 않는다. `+0xd8`(nnz)과 activity도 건드리지 않는다.
- 작업 배열: slot(n, −1로 시작), neg/pos(n, 목록에 넣을 때만 0으로 초기화). 나머지는 스택 배열(이웃 20, 행 9, 항목 80)이다.

## 2. 후보 열

열 j = 0..n−1을 한 번 돈다. 앞 열에서 지운 결과가 뒤 열 판정에 바로 반영된다.

| 조건 | 내용 |
|---|---|
| 개수 | 0 < `col_cnt[j]` < 10, `col_flag[j]` == 0 |
| 행 길이 | 열 CSR을 순서대로 보며 live 행(`row_ind ≥ 0 && row_cnt ≥ 0`)의 `row_cnt` > 21이면 탈락(break) |
| 기준 행 piv | live `=` 행 중 \|a\|가 가장 큰 행. 시작값 0.001, `!(|a| <= best)`일 때만 바꾼다(동률은 앞 행) |
| 비용 | \|c_j\| ≤ 1e-10이거나 piv ≠ −1 |

## 3. 이웃 수집과 부호 거르기

- slot[j] = 1(목록에는 넣지 않음). j의 live 행마다 행 CSR을 순서대로 보며 live 열 q(j 포함)를 센다.
  - slot[q] < 0이면 목록 끝에 넣고 slot = 1, pos = neg = 0. 목록이 21개가 되면 그 행에서 곧바로 멈춘다(그 항목은 세지 않는다).
  - a < 0이거나 `=` 행이면 neg[q]++, a > 0이거나 `=` 행이면 pos[q]++.
  - 행마다 작업량 `t·6 + 3`(t = 본 항목 수).
- 목록이 21개면 탈락. slot을 모두 −1로 되돌리고 작업량 `cnt + cnt`.
- 거르기: neg > 0 && pos > 0인 열을 앞으로 모은다. 방법은 `swap(t, kept)`라 나머지 순서는 섞인다. 작업량 `cnt·4`.
- 남은 열 nk ≥ 9이면 탈락.
- 거른 열(한쪽 부호만)은 j의 행을 같은 방향으로만 조인다. 그래서 가장 나쁜 bound 하나로 고정해 보면 된다. neg만 있으면 lb, pos만 있으면 ub다.

## 4. 행 식

남은 이웃 p = 0..nk−1에 slot = p를 준다(작업량 `nk·3`). j의 live 행 r(열 CSR 순서)마다 다음을 만든다.

- inv = 1.0 / a_j(나눗셈 한 번, asm 0x4dc890). sense_r = 행 sense. 단 `<` 행이고 `!(0 <= inv)`이면 `>`(x_j의 하한).
- v_r = rhs·inv. 그 행의 live 열 q ≠ j(행 CSR 순서)마다 다음을 한다.
  - 남은 이웃이면 그 이웃의 항목 수를 센다.
  - neg ≠ 0이고 pos == 0이면 lb로 둔다. lb ≤ −1e30이면 v = (0 < a·inv ? 1e100 : −1e100), 아니면 v = v − (lb·inv)·a.
  - neg == 0이면 ub로 둔다. ub ≥ 1e30이면 v = (a·inv ≤ 0 ? 1e100 : −1e100), 아니면 v = v − (ub·inv)·a.
  - ±1e100이 된 뒤에도 뒤 항목의 뺄셈이나 대입이 이어진다(원본 그대로).
  - 행마다 작업량 `len·9 + 6`.
- 이웃별 시작 위치를 누적합으로 구한다(작업량 nk). 행을 다시 돌며 이웃 p의 (r, c = (−inv)·a)를 이웃별로 모은다. 행마다 작업량 `len·11 + 3`. 그 뒤 작업량 nk, slot을 되돌리고 `nk + nk`.
- 결과: x_j (sense_r) v_r + Σ_p c·x_p.

## 5. 행별 검사

행 r마다 상자 위에서 식의 범위를 구한다. lo_r = min, hi_r = max이고 둘 다 v_r에서 시작한다.

| c | lo_r (−1e30 < lo_r일 때) | hi_r (hi_r < 1e30일 때) |
|---|---|---|
| c ≤ 0 | ub ≥ 1e30이면 −1e30, 아니면 lo_r + ub·c | lb ≤ −1e30이면 1e30, 아니면 hi_r + lb·c |
| c > 0 | lb ≤ −1e30이면 −1e30, 아니면 lo_r + lb·c | ub ≥ 1e30이면 1e30, 아니면 hi_r + c·ub |

이웃마다 작업량 `cnt·7 + 2`. 그다음 판정한다. `>` 행은 ub_j + 1e-6 < hi_r이면 탈락이다. `<` 행은 lo_r < lb_j − 1e-6이면 탈락이다. `=` 행은 둘 다 본다. 이 검사와 아래 꼭짓점 검사가 함께 [lb_j, ub_j] 안의 x_j가 있음을 보장한다.

## 6. 꼭짓점 열거 (asm 0x4dd730–0x4ddd41)

- 시작점은 모든 이웃이 lb다. val_r = v_r(±1e30 이상이면 0으로 두고 plus/minus 개수 1)에 c·x_p를 더한다. x_p ≤ −1e30이면 c·x_p ≤ 0일 때 minus++, 아니면 plus++. 작업량은 `nr·3`을 두 번 더하고 `nk·5`.
- 꼭짓점마다 hi = 1e30, lo = −1e30에서 다시 시작한다(0x4de59f의 xmm3/xmm5 복사로 확인. 의사코드는 이를 누적처럼 보이게 쓴다). 행 r = 0..nr−1을 차례로 본다.
  - `>`가 아니면: 무한 개수가 0이면 hi = minsd(hi, val_r), 즉 `hi < val ? hi : val`. minus > 0이면 hi = −1e100.
  - `<`가 아니면: 무한 개수가 0이면 lo = maxsd(lo, val_r). plus > 0이면 lo = 1e100.
  - hi + 1e-6 < lo이면 멈추고 탈락한다. 작업량 `r·4`.
- 다음 꼭짓점은 2진 카운터로 간다. 마지막 이웃이 가장 빨리 바뀐다. p = nk−1부터 내려가며 lb == ub인 이웃은 건너뛴다. x_p == lb면 lb → ub, 아니면 ub → lb로 바꾼다.
  - 옛 값의 기여를 뺀다. |옛 값| ≥ 1e30이면 c·옛 값 > 0일 때 plus−−, 아니면 minus−−. 유한하면 val −= c·옛 값.
  - 새 값의 기여를 같은 식으로 더한다(val += c·새 값).
  - 빼기와 더하기 뒤에 각각 작업량 `cnt·3 + 6`.
  - 새 값 == ub면 그 꼭짓점을 검사한다. 아니면(올림) 다음 p로 간다. p < 0이면 모든 꼭짓점을 통과한 것이다.
- val은 꼭짓점마다 새로 더하지 않고 그 자리에서 고친다. 그래서 반올림이 열거 순서에 따라 달라지고, 이 순서를 그대로 따라야 비트가 맞는다.
- 최대 2^8 = 256개 꼭짓점, 각 9행이다.

## 7. 비용 이전

\|c_j\| > 1e-10일 때만 한다(그러면 piv가 있다).
- mult = 1.0. 행 piv에서 열 j 항목을 찾으면 mult = (−c_j)/a(못 찾으면 1.0이 남는다).
- 행 piv의 live 열 q마다(j 포함) obj[q] += mult·a_q. 그다음 obj[j] = 0, objcon −= mult·rhs[piv].
- 레코드의 y_idx = piv, y_add = mult다. 비용이 0이면 −1 / 0.0이다.

## 8. 레코드 type 24와 삭제

레코드는 비용 이전 뒤에 쌓는다. 그래서 lb/ub는 그대로이고 obj는 이미 0이다. 필드와 바이트 배치는 [m1_postsolve.md](m1_postsolve.md) type 24 절과 같다(`PostRec::ColRows`).

| 위치(data = rec+0x18) | 내용 |
|---|---|
| rec+0x00 | 0x18 |
| data+0x00 / +0x08 / +0x10 / +0x18 | j / lb / ub / 'C' |
| data+0x1c / +0x20 / +0x28 | y_idx / y_add / c(행 수) |
| data+0x30 → rec+0x78 | 행 시작(c+1개 int) |
| data+0x38 / +0x50 / +0x58 / +0x40 / +0x48 | 행 번호 / 항목 열 / 항목 값 / sense / rhs |

- 크기 = (8K−8) + ((4K−1)&~7) + ((4c+3)&~7) + ((4c−1)&~7) + ((c−1)&~7) + 8c + 0xa0. K = 행들의 `row_cnt` 합이다.
- 행은 열 j의 열 CSR 순서다. 항목은 행 CSR 순서이고, live 열(j 포함)만 넣는다.
- 삭제는 j의 live 행마다 한다. `row_cnt = −1`로 두고, 그 행의 live 열마다 `col_cnt −1`, `row_ind = −1`로 지운다. 그다음 열 CSR 항목을 −1로 두고 `+0x404`++. 끝으로 `col_len[j] = −1`, `col_cnt[j] = −2`, `+0x408`++. 레코드와 삭제에는 작업량이 없다.

## 9. 검증 (2026-11-05)

| 대상 | 결과 |
|---|---|
| forplan k=27, finnis k=39, beaconfd k=60·79, ganges k=79와 그 뒤 체크포인트 | 일치 |
| PRELIMIT 17..102(64개) | 62/64. 틀리는 것은 degen2·degen3뿐이고 k=20부터다(`47c445`) |
| PRELIMIT 1..16 | `5b488d`는 기본값에서 빨라야 19번째 호출이라 영향 없음(17은 64/64) |
| 기본값 | 62/64(degen2·degen3) |
| uncrush 오라클 기본값(`tools/postsolve_check.py`) | 57/59. 병합 직후 53/59에서 forplan·finnis·beaconfd·ganges가 맞게 됐다. 남은 둘은 degen2·degen3(`47c445`) |

Netlib 기본값에서 type 24는 위 4개 모델에 14개 생긴다(m1_postsolve.md). 회귀 테스트는 `tests/test_presolve.py::test_derived_bounds_checkpoint`(finnis k=39, `bench/gurobi_presolve/finnis_lim39.mps`)다.

## 10. 확인하지 않은 것

- 작업량 항목은 의사코드대로 옮겼지만 오라클로 확인하지 않았다(P2).
- NaN 경로: 시작점의 ±1e30 판정은 4개씩 묶는 SIMD 부분과 나머지 스칼라 부분이 NaN을 다르게 다룬다. 포트는 스칼라 쪽을 따른다.
- 두 번째 작업 배열의 할당 크기(neg/pos, 2n개 int로 추정)는 결과에 드러나지 않아 보지 않았다.
- 계수 0인 항목(\|a\| ≤ 1e-13은 생성 때 빠지지만 집계가 0을 만들 수 있다)은 neg도 pos도 늘지 않아 ub 쪽 분기로 간다. Netlib에서 일어나는지는 보지 않았다.
