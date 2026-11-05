# 03. Presolve와 Postsolve

포팅 원본: [port_map.tsv](port_map.tsv)의 `03_presolve`(415개). 진입 함수는 LP `390989`, MIP `3cd406`, postsolve `363164`, crush `365159`다. 표의 "근거" 열 함수가 각 reduction의 원본이다. 근거: 노트 12 전체. 읽기 보조 문헌: Andersen & Andersen 1995(LP), Achterberg, Bixby, Gu, Rothberg, Weninger 2020(MIP).

## 1. 구조

```
lp::presolve::run(model, p, budget)  -> (Presolved, PostsolveStack)      // LP(연속) presolve
mip::presolve::run(model, p, ctx)    -> (Presolved, PostsolveStack)      // MIP presolve, 재시도 래퍼 포함
model::postsolve::{uncrush_primal, uncrush_dual, crush_row}
```

- 공통 뼈대(노트 12 §0.2): **상태 구조체 + 변경 추적 worklist + reduction 함수 목록 + 진척도 기반 패스 반복**.
- LP와 MIP는 상태의 행렬 저장이 다르다(12 §3.10): LP는 CSR+CSC 쌍, MIP는 **행·열 연결 리스트**(삭제·추가가 잦음) + 필요 시 CSR 캐시. 둘 다 `PresolveState` trait로 같은 reduction을 공유할 수 있게 한다(activity, 고정, 행 삭제, 레코드 push).
- LP 포팅(`crates/lp/src/presolve/`, 진행 기록 [m1_presolve.md](m1_presolve.md)): 삭제는 지연 방식이다. 행 CSR 항목은 열 번호 −1로, 열 CSR 항목은 행 번호 −1로 표시하고, 지운 행·열은 살아 있는 개수(`+0x60/+0x68`)를 음수로 둔다. presolved 모델(`3b0917`)은 열 CSR 순서 그대로 살아 있는 항목만 쓰고, 행·열은 원래 순서를 지킨다. 비교 오라클은 숨은 파라미터 `GURO_PAR_PRELIMIT`(reduction 호출 횟수 상한)로 reduction 하나씩 뜬다.
- 결정: **LP presolve(`390989`, CSR+CSC)와 MIP presolve(`3cd406`, 연결 리스트)를 원본대로 따로 포팅한다.** 같은 reduction이어도 두 상태의 순회 순서가 달라 결과 모델이 다르다. 하나로 합치면 P0(presolved 모델 일치)를 맞출 수 없다. LP presolve는 M1, MIP presolve는 M2다.

## 2. 상태(`PresolveState`)

| 필드 | 내용 | 근거 |
|---|---|---|
| `rows: LinkedMat`, `cols` | 연결 리스트 행렬. 노드 {coef, row, col, next_in_row, next_in_col}. col=-1이면 삭제. GC는 garbage > 2×live일 때 | 12 §2.3 |
| `row_act: Vec<Activity>` | {min_fin, max_fin, min_inf_cnt, max_inf_cnt}. 무한 기여 수를 따로 센다. 덧셈 반올림이 감지되면 1ulp 보수적 방향으로 민다 | 12 §3.5 |
| `lb, ub, obj, vtype, row_lo, row_hi, obj_const` | | |
| `stats` | removed_rows/cols, tightened, coef_changed, ineq_to_eq, type_changed, sos_removed … (로그와 진척도 계산용) | 12 §3.6 |
| `work: WorkTimer[N_RED]` | reduction별 누적 work와 한도 | 12 §4.4 |
| `tracker: ChangeTracker` | 레벨 0/1/2 큐. 바뀐 행·열만 다음 reduction이 본다 | 12 §4.5 |
| `post: PostsolveStack` | §5 | |
| `cutoff` | min(Cutoff, incumbent − 1e-10(|obj|+1)) (재시작 presolve에서) | 12 §3.8 |

## 3. Reduction 목록과 단계

표의 "단계"는 **기본 파라미터에서 꺼져 있는 reduction에만** 적용한다. 기본값에서 도는 reduction은 표의 단계와 관계없이 LP면 M1, MIP면 M2에 포팅한다. 하나라도 빠지면 presolved 모델이 달라진다. 기본값에서 도는지는 가드 파라미터의 기본값과 P0 diff로 판정한다. 가드는 파라미터명(공개는 그대로, 숨은 것은 소문자 내부 필드).

### 3.1 빠른 내부 루프 (매 패스, MIP·LP 공통)

| # | reduction | 가드 | 레코드 | 단계 | 근거 |
|---|---|---|---|---|---|
| 1 | 열 패스: 고정 열, 빈 열(쌍대 고정: 비용 부호로 bound 선택, 무한이면 UNBOUNDED), singleton 열(등식의 free singleton → 대입, 아니면 쌍대 고정 type 3) | pre_single_col | 1, 2, 3 | M1 | 12 §2.3 `3e39a6` |
| 2 | 빈 행·항상 만족 행 제거, 아주 작은 계수 정리(`nnz·range·|a| < 0.01·FeasTol`, `|a|<1e-3`) | – | – | M1 | `36ddf2` |
| 3 | activity 기반 bound 강화 + forcing 행(모든 열을 행이 요구하는 bound로 고정). 정수는 ceil/floor(IntFeasTol), 연속은 `|b|<1e12` | pre_bound | 1 | M1 | `37168e` |
| 4 | singleton 행 → 열 bound(등식이면 고정) | – | 7 | M1 | `36c5cf` |
| 5 | I→B: [0,1] 정수를 이진으로 | – | – | M2 | |
| 6 | 계수 강화(Savelsbergh): 무한 기여 없는 `≤`행, 정수 열 | pre_coefficient | – | M2 | `56f793` |
| 7 | doubleton 등식 집계 `x_j = a x_k + b`, slack≈0 등식 집계 | Aggregate | 2 | M1 | `56c80e` |
| 8 | GC(연결 리스트 정리) | work>1e6 | | M1 | |

내부 루프 종료: MIP는 진척도 `ρ ≤ 0.001·max(0,i)` (i는 −5에서 시작), LP는 `ρ ≤ 1e-11·max(0,i−5)` (i는 0부터).

### 3.2 바깥 reduction (패스마다 한 번, timer 사용률 < 0.8일 때)

| # | reduction | 가드 | 레코드 | 단계 | 근거 |
|---|---|---|---|---|---|
| 9 | 평행 행/열 (해시 → 비교 → 처리): 중복 행 삭제, `x_j = x_k` 집계, 평행 열 병합(postsolve가 bound 안에서 나눔) | pre_parallel | 2, 4, 5, 14, 20 | M2 | `3ff744` |
| 10 | 쌍대 bound(complementary slackness): 승수가 0이 아닌 행 → 등식, Presolve=2면 reduced cost 부호 확정 열 고정 | DualReductions | – | M2 | `374521` |
| 11 | lock 하나뿐인 열의 막는 부등식 → 등식 | DualReductions | – | M2 | `3e40d1` |
| 12 | 지배 열(dominated columns): lock 서명 부분집합 + 계수 비교 | pre_dominate | 1 | M2 | `4030c9` |
| 13 | 비영 소거(행/열): 등식 행의 배수를 더해 nnz 감소, 배수 비 한도 NumericFocus로 0.1–1000 | pre_cancel | 10 | M6 | `40d2bc` |
| 14 | 행 포함(subsumption)·clique 기반 행 중복 | pre_subsume, pre_cliqsub | – | M6 | `416036`, `414ca3` |
| 15 | 일반 집계(substitution), fill 한도 `AggFill`(MIP 자동 0, LP 10/20) | Aggregate | 2, 6, 11 | M2 | `37f484` |
| 16 | 쌍대 집계(VUB형 부등식) | Aggregate+Dual | 2 | M6 | `418542` |
| 17 | 이진 함의 그래프 SCC → 동치 literal 집계·고정 | pre_transred | 2 | M2 | `5431ee` |
| 18 | 쌍대 정수→이진 아핀 변환 `x = l + (u−l)x'` | pre_dualconvtobin | 9 | M6 | `3c2f36` |
| 19 | clique 추출·강화, set packing 행 처리 | pre_clique | 1 | M2 | `45885d`, `52be10` |
| 20 | VUB 병합, two-row bound 강화 | pre_mergevubs, pre_tworows | – | M6 | `5466a7`, `45feaa` |
| 21 | 선형 종속 행 제거(Markowitz LU, `lp::lu` 재사용) | PreDepRow | – | M3 | `47c445` |
| 22 | implied integer 검출(등식의 정수 계수 → 연속 열을 'I'로) | implied_int | 9 | M2 | `57627e` |
| 23 | 목적 부분 소거(등식 행에 비례하는 비용) | pre_partobj | – | M2 | inline |
| 24 | 열 implied bound(행 activity로 bound 교체, 1000·FeasTol 이상 개선 시) | pre_bound+Dual | – | M2 | inline |
| 25 | XOR/패리티 구조, knapsack 행 강화(lifting 재사용) | pre_xorred, pre_power | 25 | M6 | `3e5363`, `578fd2` |
| 26 | Q 대입, QP 열 축약(`min ½qx²+cx` 해석해), QC presolve | pre_qsubst, qpreduce | 12, 16 | M4 | `56b360`, `42872b`, `451195` |

### 3.3 라운드 후 단계 (MIP, 최대 5라운드)

probing(이진 0/1 시험 고정 → 고정·bound·함의·동치, `pre_probe` 모드 0..3, 스레드 분할) M2(단일 스레드)·M6(병렬); SOS 축약(type 17) M4; SOS big-M 인코딩(`PreSOS1BigM/PreSOS2BigM/PreSOS*Encoding`) M4; 대칭성(orbit, `Symmetry`) M6; 연결 성분(`pre_connected`) M6; sparsify(`PreSparsify`, type 22) M6; 아주 작은 모델 직접 해결(행 1개) M6; 해 기반 축약(`sol_reduction`) M6.

### 3.4 마무리

일반 제약 번역(`9ee0cf` → [08](08_genconstr_nonlinear.md)) → NL DAG presolve → SOS 인코딩 → 최종 쌍대 고정 → presolved 모델 생성(`to_model()`), PSD 검사(QP/QC) → 결과 로그 "Presolve removed|added %d rows and %d columns", "Presolve tightened %d bounds and modified %d coefficients", "Presolve changed %d inequalities to equalities", "Presolve time: %.2fs", "Presolved: %d rows, %d columns, %lu nonzeros", "Presolved model has %d SOS / quadratic objective terms / quadratic constraint(s) / bilinear / nonlinear constraint(s)".

## 4. 패스·예산·진척도 (노트 12 §3.4)

```
PrePasses 자동: LP → Presolve=1 ? 5 : 100 ; MIP → Presolve=2 ? ∞ : Presolve=1 ? 20 : 100
budget = (nnz + Qnnz + SOS·QC·genconstr 크기) × { LP: Presolve=1 200, 기본 1000 ; MIP: 500 / 2500 ; Presolve=2|PrePasses>1|probe>1|PreSparsify>0: ∞ } + 1e7
재시작 presolve(mipctx): ×500 또는 MIP 진행 work의 ½
reduction별 timer 한도 = budget × {0.5,1,2,4}; 사용률 ≥ 0.8이면 그 reduction 건너뜀
전체 초과 → 남은 reduction 모두 중단
ρ = Δcols/n + (Δrows + ΔSOS + ΔQC…)/(m+…) + 0.1·Δcoef/nnz + 1e-4·(Δbound + Δtype…)/n + 0.5·Δineq_to_eq/(m+…) + 0.001·Δ기타/(m+…)
종료(MIP): 내부 ρ ≤ 0.001·max(0,i) ; 패스 ρ ≤ 1e-4 ; 라운드 ρ ≤ 0.01(0.05) ; 라운드 ≤ 5
종료(LP, 390989): 내부 ρ ≤ 1e-11·max(0,i−5) ; 패스 시작 대비 ρ ≤ 1e-4·max(0,k−2) ; 패스 끝 ρ ≤ 1e-6 ; PrePasses 자동 100(Presolve=1이면 5)
```
work 측정: reduction마다 `work += 처리 원소 수 × 상수`. TimeLimit/WorkLimit/terminate는 `limits::check`로 매 reduction 뒤 확인. 콜백 where=PRESOLVE(1)를 DisplayInterval마다 호출하고 "%s removed %d rows and %d columns (presolve time = %.0fs)..." 진행 로그.

## 5. Postsolve 스택 (노트 12 §3.9, §4.2)

```rust
pub enum PostRec {
    FixCol { j, v, rc_info },                                   // 1
    Subst  { j, c0, terms: Vec<(usize, f64)>, dual_row: Option<..> },   // 2
    DualFix{ j, rows: Vec<(r, c_r, coef)>, is_int, kind: MinMax },  // 3  (행들이 허용하는 끝값, 정수면 floor/ceil)
    MergeParallelCols { j, k, ratio, lb, ub },                  // 4   (bound 안에서 둘로 나눔)
    ParallelRowsDual { r0, r1, ratio },                         // 5   (LP 50b4e5: 평행 행의 쌍대 재분배; 노트 12의 "열"은 틀림)
    AggRowCol { row, col, .. },                                 // 6
    SingletonRow { r, j, a, side },                             // 7   y[r] = −d[j]/a
    ForcingRow { r, coefs },                                    // 8
    Affine { j, a, b },                                         // 9   x = b + a·x'
    CancelCol { j, c, a, k, b },                                // 10
    AggGeneral{..}, QpFix{..}, ModelGen{..}, ParallelRows2{..}, // 11, 12, 13/15, 14
    QuadSubst { j, c, terms: Vec<(a, p, q)> },                  // 16
    SosReduce { a, b },                                         // 17  둘에서 min을 뺌
    GenConstrResult { j, f: ResultFn /*min,max,norm2,maxabs*/, rows },  // 18
    Symmetry{..}, SplitManyCols{ cols, eq_rhs }, AggDual{..}, Sparsify { i, k, a }, Derived{..}, Xor{..},  // 19, 20, 21, 22, 24, 25
}
pub struct PostsolveStack { recs: Vec<PostRec> /* push 순서, 역순 순회 */, col_map: Vec<i32>, row_map: Vec<i32>, orig_n, orig_m, obj_const, orig_row_sense }
```

- **원시해 복원** `uncrush_primal(x_pre) -> x`: `x[j] = x_pre[col_map[j]]` 뒤 레코드를 **역순**으로 적용(위 type 1,2,3,4,9,10,16,17,18,20,24,25만 원시해에 영향).
- **쌍대해 복원** `uncrush_dual(y_pre, d_pre)`: type 1,2,3,4,5,6,7,8,21,22,24. 끝에 원래 `≥` 행의 쌍대 부호를 되돌리고 basis 상태(VBasis/CBasis)를 행 활동량으로 맞춘다.
- **crush** `crush_row(row in 원래 공간) -> Option<row in presolved 공간>`: 열마다 그 열을 없앤 레코드를 찾아 1은 상수로, 2는 대입, 9/10은 아핀, 3/16/18/24/25는 **실패**. 사용자 cut/lazy에 쓴다. `PreCrush=1`이면 실패를 일으키는 reduction(9 outer, 17, 18, 25, derived)을 끈다.
- 재시작 presolve는 presolved 모델을 다시 presolve하므로 스택이 **계층**으로 쌓인다(`Vec<PostsolveStack>`, 바깥부터 안쪽 순).

## 6. LP uncrush와 "big uncrush violation" (노트 12 §3.9, 확인 결과는 [m1_postsolve.md](m1_postsolve.md))

기본 LP(simplex) 경로(`0c3d0d`)는 다음 순서로 진행한다.
1. presolved 모델을 simplex로 푼다.
2. **`363daa`**로 uncrush한다. x·쌍대·기저를 함께 복원하며, 레코드를 LIFO로 처리한다.
3. 원래 모델에서 warm start simplex를 다시 돈다(기저와 x는 넘기고 쌍대는 넘기지 않음). 반복이 있으면 "Extra simplex iterations after uncrush: %d"를 찍는다.

`363164`·`365159`와 아래의 big uncrush violation 검사는 `36578f` 경로(SolutionTarget≥1, simplex 없는 경로)에서만 돈다. 최종 X/Pi/RC는 3단계가 다시 계산한 값이다. 그래서 postsolve의 비트 비교 대상은 `363daa`의 출력이고, 오라클은 `tools/uncrush_oracle.py`다.

```
(36578f 경로)
if status ∈ {3,4,5}: DualReductions·Presolve 켜져 있으면 §7(02_driver)의 규칙으로 4로 낮춤
x = uncrush_primal
if status == OPTIMAL:
    viol = max_violation(model, x)
    tol = max(1e4, 1/min_scale) × max(FeasibilityTol, presolved_viol)
    if viol > tol: "Warning: Changing status from GRB_OPTIMAL to GRB_SUBOPTIMAL due to big uncrush violation %g (presolved model = %g)."; status = 13
uncrush_dual ; QCPDual이면 KKT로 쌍대 계산
```

## 7. 재시도 래퍼와 상태 보정 (노트 12 §3.6–3.8)

- MIP 래퍼: Q 대입 후 비PSD(`q_non_psd`) 또는 공격적 집계 뒤 계수 통계 이상(`max|a| > 1e12·min|a|`, `>1e8`, bound·rhs·obj > 1e9)이면 결과를 버리고 `pre_qsubst=0, ageq=0`으로 한 번 더. 메시지 "Q matrix is non-PSD after presolve substitutions… Trying again without substitutions" / "Presolved model has numerical issues… Trying again with more conservative settings".
- 드라이버: presolve가 INFEASIBLE이고 MIP start가 있고 DualReductions≠0이면 DualReductions=0으로 한 번 더.
- 상태 보정: DualReductions≠0 & INFEASIBLE → INF_OR_UNBD; UNBOUNDED인데 확인 못하면 4; 목적이 변수 한계로 유계면 3 확정.
- MIP 재시작 presolve(`mip::restart`): "Restarting search with another presolve..." / "Performing another presolve...". cutoff를 incumbent로, work 예산을 MIP 진행량에 묶음.

## 8. 포팅으로 확정할 지점 (노트 12 미확인 사항)

- 라운드 후 단계의 고급 reduction `5aaaf7`, `5a1293`, `419ffc`, `423e35`, `594c0c`, `59c193`, `5ab429`, `5a9282`, `58d4ce`, `5a7c0f`, `5a04fc`의 정체. 지금은 가드와 레코드 type만 안다.
- `838543` 세 번 호출(mode 2/3/5)과 PRESS2MIP·PREPROBE0·PRECOVERS의 대응.
- `3cd406` 바깥 루프 후반의 실행 순서. §3.3의 순서는 Ghidra의 블록 배치 순서이므로, 포팅 때 objdump의 분기 흐름으로 실제 순서를 정한다.
- env+0x2dc0("Root relaxation presolve" 플래그)과 env+0x2728을 설정하는 곳.
- `ba2e9`가 띄우는 `ba1df` 스레드의 보조 모델.
- 레코드 type 11–15, 19, 23의 복원 의미(`363164`, `365159`). §5의 `PostRec`는 포팅 결과로 채운다.
- 대칭성 `5577ba`/`551cc0`, probing `4d3a1d`(스레드 분할, GUB probing), clique `45885d`/`451bed`.
- `GURO_PAR_PRESINGLECOL` 값별 의미, `3ff744` 콜백(`3ee2bb`, `3f26d9`, `3f1a7b`) 내부.

## 9. 테스트

- 패리티 P0(합격 기준): Netlib 64개와 `miplib_small` 38개에서 Gurobi `Model.presolve()` 결과를 MPS로 쓴 것과 rsolver presolved 모델이 행·열 순서까지 같다. "Presolve removed …" 통계와 presolve work도 같다.
- reduction 단위 테스트: 각 reduction에 교과서 예(Achterberg 2020 그림)를 넣고 presolved 모델과 **uncrush 뒤 원래 해의 실행가능성·목적값**을 확인.
- 속성 테스트(proptest): 무작위 작은 LP/MIP를 presolve 있음/없음으로 풀어 목적값 일치(1e-6), uncrush 해 위반 ≤ FeasTol.
- Netlib 104개: presolve 켠/끈 결과 일치.

## 10. LP presolve 포팅 결과 (M1, `crates/lp/src/presolve/`)

위 §3의 표는 MIP·LP 공통 설계다. LP presolve `390989`가 실제로 부르는 순서와 가드는 아래와 같다(기본 파라미터 기준, 근거와 세부는 [m1_presolve.md](m1_presolve.md), 집계는 [m1_presolve_agg.md](m1_presolve_agg.md)). 호출마다 숨은 파라미터 `GURO_PAR_PRELIMIT` 카운트가 1 늘고, 한도에 닿으면 마무리 없이 모델 생성으로 간다. 이 카운트가 reduction 하나씩 비교하는 오라클이다.

| 단계 | 함수 | 포트 | 가드 | 레코드 |
|---|---|---|---|---|
| 시작 | `5034e9` 빈 행·자유 행, `503985` 거의 고정 열, `51a144` 쌍대 singleton 고정 | `state.rs`, `reduce.rs` | `503985`는 FIXTOL < 0이면 n ≥ 10000일 때만, `51a144`는 PREBOUND | 1 |
| 내부 루프 | `4fe113` 열, `4ffe11` singleton 행, `4ff158` forcing 행, `50cc90(0)` 중복 행 | `reduce.rs` | 진척도 ρ ≤ 1e-11·max(0,i−5)이면 끝 | 1, 2, 3, 7, 8 |
| 바깥 패스 | `50cc90(1)`, `50d878` 빈 열, `38e49a` 비용 점 고정, `50495e` 함의 중복 행, `505077` 쌍대 bound 행(k < 5), `4ffe11`, `4ff158` | `reduce.rs` | `50d878`·`38e49a`·`505077`은 DualReductions | 1, 3 |
| | `3ff744` 평행 열(mode 1)·행(mode 0) | `parallel.rs` | PREPARALLEL, 열은 DualReductions | 4, 5 |
| | `50fe2b` 비용이 묶는 부등식 → 등식 | `reduce.rs` | DualReductions(카운트는 늘 1) | – |
| | `519a52`/`37f484` 집계(연결 리스트 행렬, 단계 A–D, 최대 5라운드) | `agg.rs` | Aggregate, pass 0만 | 2, 6, 11, 21 |
| | `5b488d` 이웃이 무엇을 해도 만족되는 열과 그 행 제거(비용은 `=` 행 하나로 넘김) | `derived.rs` | PREDERIVED, DualReductions, PreCrush==0, 모든 패스 | 24 |
| | `538f30`+`4f79a1` activity 재계산 | `state.rs` | 카운트 없음 | – |
| | `47c445` 종속 행: 희소 Markowitz 소거로 mode 1은 종속 `=` 행 삭제, mode 2는 자유 열 중 종속 열 고정 | `deprow.rs` | pass 0, PreDepRow 자동 = 2.5·n < nnz이면 3(대칭 검출 `5577ba`가 대칭을 찾으면 0), mode 2는 DualReductions | (mode 2만) 1 |
| | 종료 판정 | `mod.rs` | 패스 시작 대비 ρ ≤ 1e-4·max(0,k−2), 패스 끝 ρ ≤ 1e-6, PrePasses 자동 100 | |
| 마무리 | `51a144`, {`50cc90(1)`, `50d878`} 반복, `50de61` 함의 bound(0.001·(\|b\|+1) 여유), `50d878` | `mod.rs`, `reduce.rs` | PREBOUND, DualReductions | 1 |
| 모델 생성 | `3b0917`: 살아 있는 행·열을 원래 순서로, 열 CSR 순서로 항목을 쓴다. 마주 보는 평행 행 쌍은 PRESPLITROWS로 `=` 행 + 범위 열(type 14) | `state.rs`, `parallel.rs` | PreCrush ≤ 1 | 14 |

- 저장: CSR+CSC, 지연 삭제(§1). 집계만 MIP식 연결 리스트로 바꿨다가 CSR로 되돌린다. 리스트 순서가 결과 CSR 순서가 되므로 노드 풀(free·pending)과 +0xd8 카운터까지 원본대로 흉내 낸다.
- activity는 덧셈이 부정확하면 1ulp 안전한 쪽으로 미는 가드 덧셈을 쓴다. 단 `50de61`과 `50495e`/`50108e`의 행 범위는 가드 없이 더한다.
- 일치(Netlib 64, 기본값): presolved 모델 **64/64**, PRELIMIT 체크포인트 64개 × k = 1..100(카운트가 긴 beaconfd·ganges·scrs8은 154까지) 전부 일치. uncrush(`363daa`)는 오라클 `pre_after`를 넣은 비트 비교로 59/59(Gurobi가 uncrush를 부르지 않는 5개 제외).
- 작업량(+0x458, μ = +0x448)과 예산: 생성자(`3a484d`/`5118e6`/`534a8b`, 문제 유형 스캔 포함)와 reduction마다 원본 순서로 더한다. 비교는 PRELIMIT 단계별 증분(`m1_presolve.md` §0의 `wsteps.py`). 예산 = +0xd8·1000 + 1e7(Presolve 1이면 ·200, PrePasses > 1이나 Presolve 2면 1e100), 기준 0. `390989` 자신의 `3ba9de` 세 곳은 호출 동안 예산을 1e100으로 바꿔 두어 정지하지 않는다. 정지는 reduction 안의 `3ba9de`(`47c445` 100바퀴마다, `3ff744` 쌍 1만 번마다, `551cc0`)가 PRELIMIT을 −1로 바꾸고 뒤 한도 검사가 멈추는 식이다. Netlib은 예산의 34% 이하라 정지가 없다.
- 포팅하지 않은 것(지름길, 코드에 `shortcut:` 주석): 대칭 검출 `551cc0`(결과는 PreDepRow 자동값 게이트에만 쓰인다. Netlib에서는 PRELIMIT 카운트로 늘 "대칭 없음"임을 확인. 작업량과 열 CSR 압축은 상태를 바꾼다), 집계의 작업량 항목(P2), Presolve=2 전용 reduction(`51f5ab`, `51aba6`), 집계 prescan 특수 경로, `503985`(n ≥ 10000), Params에 없는 숨은 파라미터(PREPARALLEL, PRESPLITROWS, MarkowitzTol, GURO_PAR_ZERO, INTBASIS 등)는 기본값 가정.
