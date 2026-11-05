# 05. Barrier, Crossover, PDHG, Concurrent LP

포팅 원본: [port_map.tsv](port_map.tsv)의 `05_barrier_pdhg_concurrent`(236개). 진입 함수는 barrier `2bb770`/`2bf1ea`, ordering `3047bb`·`309dd0`, 밀집 열 `2ffcb4`, crossover `13f716`(push `134a87` 등), PDHG `34c004`·`355602`, concurrent `172cc6`이다. 근거: 노트 14 전체. 읽기 보조 문헌: Mehrotra 1992, Gondzio 1996, Andersen–Andersen / Xu–Hung–Ye(HSD), Amestoy–Davis–Duff(AMD), George(ND), Bunch–Kaufman, Bixby–Saltzman 1994, Applegate et al. 2021(PDLP), Lu & Yang 2024.

## 1. 진입과 분기 (노트 14 §1.2)

- `Method=2`이고 `Crossover=0`일 때만 `lp::barrier::solve`가 직접 불린다. 기본값(Crossover=-1)은 LP 드라이버가 presolve → barrier → crossover → uncrush를 맡는다. 즉 **barrier 모듈은 presolve와 crossover를 자기 안에서 조건부로만 한다**(QP/QCP는 crossover 없음).
- `barrier::solve`: 빈 LP는 dual simplex로; PreDual(-1 자동 판단, 1 쌍대 LP 생성); presolve가 필요하면 `lp::presolve`; SOCPOUTER면 outer approximation(§7); 본체 → postsolve → 상태별: OPTIMAL이고 crossover 켜짐 → crossover; 3/4/5/9/11/16/17 → 해 벡터만 정리; 그 밖(7, 12, 13) → crossover 시도.

## 2. Barrier 본체 (`lp::barrier`) (노트 14 §3.1)

### 2.1 형태

표준형 `min cᵀx, Ax = b, l ≤ x ≤ u`로 변환(부등식 행은 slack 열, 자유 변수는 분할 또는 정규화). **정규방정식 `A D Aᵀ Δy = r`** 기본. 증강계 `[-D Aᵀ; A 0]`는 숨은 `bar_kkt`로 선택(M6). QP는 `D = (X⁻¹Z + Q)⁻¹` 대신 증강계(`Q` 포함) — M4.

### 2.2 작업 구조체 `BarrierState`

norms(‖b‖₂,∞, ‖c‖₂,∞), tol = BarConvTol(QCP면 BarQCPConvTol), 원/쌍대 실행가능 임계 `(√(m+1)+‖b‖₂)·tol`, `(√(n+1)+‖c‖₂)·tol`, 이력 버퍼 100개(원·쌍대 잔차, μ), homogeneous 플래그(BarHomogeneous -1이면 꺼진 채 시작), τ/κ, 타이머 11개(분해·predictor·corrector·갱신·로그·ordering·분석), 안전 모드(NumericFocus ≥ 2), crossover 필요 플래그(Crossover≠0 && SOC 없음).

### 2.3 구조 분석 단계

```
for trial in 0..3:                              // 밀집 열 처리 방식 시도
   dense_cols = detect_dense(m ≥ 1000, nnz_j ≥ max(500, 50·avg))     // bar_dense_thresh/count
   pattern = AAᵀ (+ 밀집 열은 증강 행으로)
   ordering → nnz(L), factor ops
   trial 0에서 방식이 안 바뀌면 중단; trial 1에서 밀집 처리 없는 쪽이 같거나 나으면 중단
symbolic factorization (supernode 분할, 열 count, etree)
ops ≤ 1e6이면 스레드 1
로그 "Barrier statistics:" Dense cols / Free vars / AA' NZ / Factor NZ / Factor Ops / Threads
nnz(L)·8 > bar_mem_limit → MEM_LIMIT
```

**Ordering**(`lp::barrier::ordering`): 기준선 = 1단계 이분할 후 AMD. `BarOrder` 조건(1, 또는 예상 ND 시간 < 0.1s, 또는 n>1000이고 ops ≥ 500·nnz(AAᵀ)…)이면 **nested dissection을 여러 seed(Seed+i+11)로 병렬 시도**(최대 5라운드, ordering 시간이 예상 분해 시간의 5%를 넘으면 중단), 최소 ops 채택. 구현: 원본 ordering(`3047bb`, `309dd0`)을 포팅한다. 여러 seed의 병렬 ND 시도까지 M3에 넣는다. ordering이 다르면 Factor NZ와 반복당 시간이 달라져 P3가 맞지 않는다.

### 2.4 반복 루프

```
x,y,z,s 초기점 (Mehrotra 최소제곱 두 후보 중 선택, bar_start; τ=1, κ=0)
log "Iter       Primal          Dual         Primal    Dual     Compl     Time"
loop:
   잔차·목적 계산; mark = (τ<1e-6 || (τ<0.01 && κ>1e4·τ)) ? '*' : ' '
   log " %3d%c %15.8e %15.8e  %.2e %.2e  %.2e  %4.0fs"; 콜백 where=BARRIER
   if homogeneous && κ==0: 켬
   예측 work = work + 0.75·(solve+factor+2·갱신)·corrector 수 → 결정적 concurrent 정지점 검사
   st = terminate_check(); if st != Running break                           // §2.5
   if !iterative_mode: 수치 분해 (§2.6)
   정규화 크기 reg = safe ? 0 : max(1e-8, ‖(rp,rd)‖/10)
   // predictor
   if κ>0: homogeneous 보조 해법
   Δ_aff = solve(mode=Affine); α_p^aff, α_d^aff; μ_aff
   r = μ_aff/μ; σ = r²·min(r, Q ? 0.9 : 0.1); μ_target = σ·μ
   // correctors (k=1 Mehrotra 2차항, k≥2 Gondzio, 목표를 [μ/10, 10μ]로 clip)
   K = BarCorrectors ≥ 0 ? max(1, BarCorrectors) : auto_K ; iterative면 1
   for k in 1..=K: Δ = solve(mode=Corrector(k)); (α_p, α_d);
        stop if α_p,α_d ≥ 1.001 || (k>1 && α_pα_d < 1e-100) || (k > max(1,K/10) && 0.99·α_pα_d < prev) || ‖Δ‖ > 1e30
   // corrector 수 자동 조정: ratio = t(추가 corrector)/(t(factor)+t(pred)+t(corr1)+t(update)); <0.25 → K++ (≤20); >0.25이고 avg α_pα_d > 0.1 → K--
   μ<1e-4이면 잔차가 임계를 넘지 않게 α 축소; QP면 α_p = α_d
   x += α_pΔx, (y,z) += α_dΔ…, τ,κ
   정체: 최근 3회 α_pα_d<1e-3 && μ 감소율>0.98 → 정규화 키우고 재시작(한 번)
   발산: BarHomogeneous==-1 && !homog && (‖rp‖>1e15·max(1,‖b‖∞) || ‖rd‖>1e15·… || (iter≥10 && 1e8배 && 무진전)) → homogeneous로 재시작
log "Barrier solved model in %d iterations and %.2f seconds (%.2f work units)" | "Barrier performed %d iterations in …"
```

### 2.5 종료 판정 (노트 14 §3.1.6)

| 반환 | 조건 |
|---|---|
| OPTIMAL | `gap_rel = |pobj−dobj|/(|pobj−objconst|+1) < tol` && `ncomp·μ/(|pobj|+1) < 100·tol` && P-feas && D-feas && `κ ≤ 100·tol·τ`. crossover 없으면 gap 측도를 `max(gap_rel, |gap|/(100|pobj|+1))` |
| INFEASIBLE / UNBOUNDED | homogeneous에서 compl < tol이고 κ > 1e6·compl: `bᵀy > 1e-10` → 3, `cᵀx < −1e-10` → 5 |
| CUTOFF | dobj > cutoff + (|cutoff|+1)·1e-6 |
| ITERATION_LIMIT | iter ≥ BarIterLimit(1000) |
| INTERRUPTED | 예측 work > work 상한(결정적 concurrent) |
| SUBOPTIMAL(13) | NaN, 목적 > 1e100, μ > 1e20 증가, 정체(마지막 재시작+200 반복, 30회 이후 잔차 최저점 9회 무갱신) |

루프 뒤 13이고 잔차가 tol 안·gap < 10·tol이면 2로 올린다.

### 2.6 수치 분해 (`lp::barrier::cholesky`)

- **supernodal LDLᵀ**, 병렬은 etree 서브트리 단위(스레드 풀). 밀집 대각 블록: Bunch–Kaufman 1×1/2×2 피벗(α=(1+√17)/8), 작은 피벗은 `±max(1e-12, min(1e-6, 1e-10·|d|))`로 대체, 2×2 0 대각은 1e-100. 갱신 블록은 밀집 GEMM/TRSM이다. 원본처럼 MKL `dgemm`/`dtrsm`을 부른다([10 §5](10_porting.md)).
- 밀집 열: 정규방정식에 열을 추가 행으로 붙이는 부분 증강계(F+0x138 보정). `2ffcb4`를 포팅해 M3에 넣는다.
- `bar_iterative`: factor+solve ops가 `(2·nnz(A)+1)·2000·20`을 넘는 반복에서 재분해를 생략하고 이전 인수로 전처리한 PCG 1회 corrector — M6.
- 메모리·시간 추정: 밀집 GEMM 벤치로 "roughly %.0f seconds per iteration".

### 2.7 QP / QCP (노트 14 §3.1.8) — M4

| 항목 | QP | QCP/SOCP |
|---|---|---|
| crossover | 없음 | 없음 |
| tol | BarConvTol | BarQCPConvTol 1e-6 |
| step | α_p = α_d | — |
| σ 상한 | 0.9 | 0.1 |
| 분해 | 증강계에 Q 포함, "Q statistics:" 로그 | 콘 준비, 정규화 모드 2 |
| 후처리 | — | QCPCLEAN(해 정리), QCPDual이면 KKT를 풀어 쌍대 |

## 3. Crossover (`lp::crossover`) (노트 14 §3.2)

```
crossover(model, barrier_sol):
  Crossover==5 && threads ≥ 2 → concurrent crossover (복제 2개: 1 dual-first, 3 primal-first; 먼저 끝난 쪽)
  MarkowitzTol = max(·, 0.5)
  barrier 해 없음 → "Barrier solution isn't available for crossover": early_quit(concurrent 비선호 슬롯)이면 포기, 아니면 simplex 처음부터
  CrossoverBasis 판단(-1): n<10000 || n<3m || gap>0.1 → 안 함; 아니면 tol 1e-6..1e-7로 bound 근접 변수 수 세어 n < 2·max(0, cnt−m)이면 "축소 문제 내부 barrier"로 시작 basis 후보 좁힘 (M6)
  해 품질(gap > 0.5, 상태 12, 잔차 > 1000) 나쁘면 push 생략 경로
  S = SimplexEngine::new(mode=crossover)
  변수 분류: bound에 붙은 것 / 내부(superbasic 후보)
  for pass in 0..=1 (re_crossover):       // 조건수 > 1e14면 "Restart crossover..." MarkowitzTol 0.9, 순서 뒤집음
     초기 basis: cand ≥ m이고 crossover_lu==2면 후보 열 직접, 아니면 LU(getrf) 기반 "%d Columns dropped for crossover initial basis"
     dual_first = Crossover ∈ {1,2} || (auto && cand ≤ 10m·밀도조건)
     if dual_first { dual_push(); primal_push() } else { primal_push(); dual_push() }
        로그 " %7d DPushes remaining with DInf %e", " %7d PPushes remaining with PInf %e", "Push phase complete: Pinf %e, Dinf %e"
  basis 해 계산; cleanup: Crossover ∈ {2,4} → dual simplex, {1,3} → primal, auto → 상태에 따라
  "Crossover time: %.2f seconds (%.2f work units)"; 상태가 바뀌면 "Crossover changed status from %s to %s"
```

push 규칙(Bixby–Saltzman): dual push는 superbasic 중 reduced cost가 0에 가까운 열을 basis에 넣고 쌍대 위반을 줄이며, primal push는 내부 변수를 ratio test로 bound로 밀어낸다. 각 push는 `SimplexEngine`의 FTRAN/BTRAN/LU 갱신을 쓴다.

## 4. PDHG (`lp::pdhg`) (노트 14 §3.3) — M3

QP/QCP 거부. 진입: presolve(조건부) → 작업 모델(스케일링은 원본 포팅. 노트는 미확인이므로 `0b8afb`, `354824`, `352bcb`를 읽어 확정한다) → 본체 → postsolve → crossover(기본 켜짐).

```
threads = nnz < 150001 ? 1 : Threads ; 열·행 구간을 nnz 균형으로 분할, barrier 동기화
ω = ‖c‖₂/‖b‖₂ (0이면 1) ; η = (1−2ε)/‖A‖₂ (power iteration, ε ≥ 0.5면 0.01/‖A‖₂)
loop:
   τ = η/ω:  x̂ = proj_[l,u](x − τ(c − Aᵀy)) ; x_ref = 2x̂ − x
   σ = η·ω:  ŷ = y − σ(A x_ref − b), 부등식 행 부호 투영 ; y_ref = 2ŷ − y
   Halpern: z ← (k+1)/(k+2)·z_ref + 1/(k+2)·z⁰
   100회마다 평가: pobj, dobj, ‖r_p‖, ‖r_d‖, 상대 잔차, gap; 콜백 where=PDHG; 로그
       종료: (abs 잔차 < PDHGAbsTol && gap_rel < PDHGConvTol) || (RelTol>0 && rel 잔차 < RelTol && gap_rel < ConvTol) → OPTIMAL
             iter ≥ PDHGIterLimit → 7 ; |pobj|,|dobj| > 1e100 → NUMERIC
       restart: iter==100 ; g ≤ 0.2·g₀ ; g ≤ 0.5·g₀ && g > g₋ ; restart 후 반복 ≥ 0.36·전체
       restart 시 ω 갱신(PID형): e = log Δy − log Δx − log ω ; I ← 0.3I + e ; ω ← ω·exp(0.99e + 0.01I) ; 이상치면 "최선 균형 ω"로 복귀
       power iteration 예산(총 반복의 10%)으로 ‖A‖₂ 재추정 → η
"PDHG solved model in %.0f iterations and %.2f seconds" ; 해 종류 = 6 (비기저)
```
불가능성 검출은 원본 위치를 위 세 함수에서 찾아 포팅한다(노트 14 §6.2-5) — M6.

## 5. Concurrent LP (`lp::concurrent`) (노트 14 §3.4) — M3

- 두 구현: (A) presolved 모델 공유형(기본, `plp_concurrent=-1`, `172cc6`), (B) 모델 복제형(PWL·웜스타트·사용자 concurrent env, `9b029d`). **둘 다 M3에 포팅한다.** 기본 경로가 (A)이고, 두 구현이 같은 결과를 낸다는 이전 가정은 확인되지 않았다(노트 11 §5.2).
- 알고리즘 배정 `jobmethod[CM][slot]`: CM0 {barrier, dual, primal}, CM1 {barrier, dual}, CM2 {barrier, primal}, CM3 {primal, dual}, CM4 {primal, dual, dual(쌍대 모델), primal(쌍대 모델)}. CM 자동: T ≥ 5 → 0, 아니면 1(또는 primal 선호면 2). CM3 && T>2 && PreDual=-1 && rows < 10·cols → CM4.
- 스레드: barrier 슬롯에 `T − nAlg + 1`, simplex 1개씩. barrier 슬롯 Crossover=0이면 1로 강제 + 경고. 비선호 슬롯에 `early_quit`.
- 실행: 슬롯 0은 호출 스레드, 나머지 스레드 풀. 승자 판정 `final = CONCURRENTMODE==1 ? status==2 : status ∈ {2,3,4..10,15,16}`. 승자는 다른 슬롯에 중단 요청 + **work 상한 = 승자 work**(결정적) 또는 즉시 중단 플래그(비결정적).
- 결정적(Method=4): 각 엔진은 자기 work가 상한을 넘기 직전 스스로 멈춘다(barrier는 예측 work, simplex는 반복 전 검사). 승자 = work 최소인 최종 상태 슬롯. "Concurrent spin time: %.2fs (can be avoided by choosing Method=3)".
- 승자 반영: basis가 있으면 VBasis/CBasis를 원 모델에 넣고 dual simplex cleanup → "Extra simplex iterations for concurrent cleanup: %d". "Solved with %s".
- ML(GBT) 선택(`171e36`)은 **구현하지 않는다**(기본 설정에서 비활성, 노트 20 주석).

## 6. 공통 인터페이스

```rust
pub trait LpSolver { fn solve(&mut self, m: &PresolvedView, p: &Params, lim: &Limits, log: &Log) -> LpResult; }
pub struct LpResult { status, x, y, rc, slack, basis: Option<Basis>, kind: SolKind /*Basic|Interior|Pdhg*/, iters, bar_iters, pdhg_iters, pobj, dobj, resid }
```
`lp::driver::solve`(= `0c3d0d`)가 presolve → 웜스타트 crush → `LpSolver` 선택 → uncrush → "Extra simplex iterations after uncrush" → 통계 로그를 맡는다.

## 7. SOCP outer approximation (`socp_outer`) — M6

QCP/SOCP를 LP + OA cut 반복으로 푸는 대안 경로. "Pass    Dual Objective  Primal Inf.  OA Cuts   Time".

## 8. 포팅으로 확정할 지점 (노트 14 §6.2)

- 밀집 열 보정(`2ffcb4`)이 Sherman–Morrison/Schur인지 부분 증강계인지.
- ordering 모드 3의 정확한 의미(`3047bb`, `309dd0`).
- 단정밀도 경로(`3279a1`, sgemm/strsm)를 쓰는 조건.
- BarIterative 모드의 Krylov 방법 위치.
- PDHG 전처리와 불가능성 검출(`0b8afb`, `354824`, `352bcb`).
- PDHG 조기 crossover의 승자 처리(env+0x2358).
- crossover push 내부(`134a87` 등 4개), auto cleanup 선택, `47c445` 초기 basis 규칙, `2c6a7a` 뒤 축소 문제 해의 반영.
- concurrent에서 T≥3일 때 CM=0을 고르는 env+0x2d8c의 작성자.
- MKL 스레드 수와 재현성(CBWR) 설정 호출부.

## 9. 테스트

- 패리티 P1(합격 기준): Netlib 64개에서 `BarIterCount`가 `re/bench/results/lp.csv`의 `grb_barrier`의 `bar=` 값과 같고, PDHG 반복 수가 `grb_pdhg`와 같다. Method=3/4의 승자 알고리즘(`ConcurrentWinMethod`)이 같다.
- 패리티 P3: barrier의 "Factor NZ", "Factor Ops"가 Gurobi 로그와 같고, 시간 기하평균이 1.2배 이내.

- barrier: Netlib 104개 Crossover=0에서 상대 gap < 1e-8, 잔차 < 1e-6; Crossover 켜면 simplex와 같은 목적값·basis 상태 일관성.
- Cholesky: 무작위 SPD·반정치 행렬에서 `L D Lᵀ` 잔차, 정규화 피벗 수 기록.
- PDHG: Netlib 소형 20개에서 1e-6 수렴, crossover 뒤 OPTIMAL.
- concurrent: Method=4 결과가 실행마다 같음(승자·반복 수 동일), Method=3은 목적값만 같음.
