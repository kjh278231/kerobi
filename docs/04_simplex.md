# 04. 단체법: LU, FTRAN/BTRAN, dual/primal simplex, sifting, network, QP simplex

포팅 원본: [port_map.tsv](port_map.tsv)의 `04_simplex`(339개). 진입 함수는 외곽 `cda5b`, 설정 `02f7e1`, dual `06268b`·`048d5f`, primal `0478d3`, LU `036ade`·`05b257`, LU 객체 `045a76`이다. 근거: 노트 13 전체. 읽기 보조 문헌: Koberstein 2005, Forrest–Goldfarb 1992(DSE), Suhl & Suhl 1990(LU), Forrest–Tomlin 1972, Gilbert–Peierls, Harris 1973, Fourer/Maros(BFRT), Bixby 1992.

## 1. 진입점과 재사용 (노트 13 §0.2, §3.12)

```rust
pub struct SimplexEngine { /* spx: 모델 사본(CSC, 스케일 적용), 상태 배열, LU, pricing 객체, 허용오차, 통계 */ }
impl SimplexEngine {
    pub fn new(model: &PresolvedView, p: &Params) -> Self;                 // 02f7e1: 스케일링, LU 객체, 섭동량, sifting 판정
    pub fn set_params(&mut self, p: &Params);                               // spx가 있으면 파라미터만 갱신 (hot start 근거)
    pub fn load_basis(&mut self, vbasis: &[i8], cbasis: &[i8]);             // 325b4
    pub fn load_start(&mut self, x: Option<&[f64]>, y: Option<&[f64]>);     // 194efc (superbasic 포함)
    pub fn solve(&mut self, alg: Alg /*Primal|Dual|Auto*/, lim: &Limits) -> SpxStatus;   // cda5b 외곽 루프
    pub fn change_bounds(&mut self, j, lb, ub);  pub fn add_rows(&mut self, rows) ; pub fn remove_rows(..);  // MIP 노드/cut용
    pub fn tableau_row(&mut self, r) -> SparseVec;   pub fn binv_row(&mut self, r); pub fn ftran(&mut self, v); pub fn btran(&mut self, v);  // Gomory, GRBBinv*
    pub fn solution(&self) -> (x, y, d, basis);  pub fn kappa(&mut self) -> f64;
}
```

- **단일 진입점**: LP 드라이버, MIP 노드, strong branching, sifting 부분문제, concurrent, crossover cleanup이 모두 `solve`를 쓴다. 엔진이 살아 있으면 basis·LU·pricing 가중치를 그대로 이어 쓴다(hot start). "기저 인자 없음 + 이전 상태 최종 + 같은 alg"이면 즉시 반환.
- **정밀도 두 벌**: 노트 13 §0.1의 double/x87 long double 두 벌은 제네릭 `SimplexEngine<F: Real>` 하나로 옮기고 `F = f64 | F80`으로 쓴다. `F80`은 x86_64 `asm!`의 x87 명령으로 만든 80비트 타입이다. double-double로 대신하면 Quad 전환 뒤 궤적이 달라진다. `Quad` 파라미터와 "Switch to quad due to numerics" 경로는 M1에 넣는다. 어려운 Netlib 인스턴스에서 원본이 실제로 전환하면 빠졌을 때 P1이 맞지 않는다.

## 2. 외곽 루프 `solve` (노트 13 §3.2)

```
alg == Auto → Method==0 ? Primal : Dual
if dual && (Sifting>0 || sift_mode>0) && 조건 → sifting (§7)
loop:
   f = restarts < 6 ? 1 : restarts < 8 ? 0.5 : 0.2            // 재시작마다 허용오차 강화
   rc = alg == Primal ? primal(opttol·f) : dual(feastol·f)
   match rc:
     LuOverflow → 이미 했으면 NUMERIC, 아니면 "Restarting from slack basis due to floating point overflow", slack basis, continue
     SwitchPrecision → (M6) quad로, 아니면 basis recovery
     BasisRecovery → 재분해·복구 후 continue
     Optimal → 비스케일 위반 검사(UNSCALEDVIO): 위반이 FeasTol/OptTol 넘으면 스케일 풀고 같은 basis에서 Dual로 continue
                임시 고정(temp fixing)이 남았으면 "Release temp fixing" 후 continue (≤10회)
     SwitchAlg → restarts > 20 ? NUMERIC : alg = other, continue
     그 외(Infeasible, Unbounded, Cutoff, IterLimit, TimeLimit, WorkLimit, Interrupted) → break
사후: 목적 > Cutoff → CUTOFF ; kappa 로그(>1e12 "very big Kappa") ; |x|,|y| > 1e20 → "Numeric error: very large primal/dual solution", NUMERIC
```

## 3. 설정 `new` (노트 13 §3.3)

- 모델 사본: CSC에 slack 열을 붙인 `n+m` 열. 행 `lo ≤ aᵀx ≤ hi`는 slack `s` 범위 `[lo, hi]`로 저장(등식은 `[b,b]`).
- 스케일링(`ScaleFlag`): 기하평균 반복(행·열 번갈아, 최대 20회) 후 2의 거듭제곱으로 반올림. `ObjScale` 음수면 자동(|c|의 최대로 나눔).
- 허용오차: FeasibilityTol, OptimalityTol, MarkowitzTol(기본 0.0078125), pivot_tol 1e-7, big_tol, zero 1e-13, fix_tol, Cutoff.
- 섭동: PerturbValue 기본 2e-4. 실제 크기는 dual이고 기본값이면 m ≤ 10000에서 ×10, 아니면 ×25.
- 작업량 계수: 1.3(double), m > 10000이면 `×sqrt(log10(m/1000))`. 모든 커널이 `work += ops × 계수`.
- **재분해 주기** `lu_interval = max(20, m^(0.7 − 0.1·NumericFocus))`, NumericFocus=1이면 ≤1000, ≥2면 ≤100. dual에서 full pricing이면 `min(·, max(20, m^0.65))`.
- sifting 판정: dual, 웜스타트 아님, PWL 아님, `n ≥ max(1000, 5m)`일 때 Sifting 파라미터와 크기 문턱(`n < 20000 || n < 50m`면 0)으로 결정.

## 4. LU (`lp::lu`) (노트 13 §3.6)

```rust
pub struct Lu { m, l: EtaFile, u: CsrUpper, perm_row, perm_col, r_etas: Vec<RowEta>, spike: Option<SparseVec>, n_updates, max_updates, fill_l, fill_u }
impl Lu {
    pub fn factor(&mut self, cols: impl Fn(usize)->ColView, markowitz_u: f64, drop: f64) -> Result<(), LuError /*Singular, Overflow*/>;
    pub fn ftran(&mut self, rhs: &mut SparseVec, save_spike: bool);
    pub fn btran(&mut self, rhs: &mut SparseVec);
    pub fn update(&mut self, leaving_pos: usize) -> Result<(), LuError>;    // Forrest–Tomlin
}
```

- **분해**: (1) singleton 단계: 행·열 count==1 큐로 삼각 부분을 뽑는다. (2) bump에 Markowitz: count별 이중 연결 목록에서 작은 count부터, 후보 열/행 수를 `max(4, 0.5·min(..))`로 제한, 조건 `rowcnt·colcnt < best` 그리고 `|a_ij| > u·max|a_·j|`. (3) 활성 부분행렬이 조밀해지면(채움 > 0.36·n², n > 50) 밀집 LU(부분 피벗)로 전환. 밀집 단계도 원본 함수를 포팅하고, 원본이 MKL을 부르는 곳은 MKL을 부른다. `|pivot| > 1e99` 또는 피벗 없음 → Overflow.
- **갱신**: Forrest–Tomlin. FTRAN 때 저장한 spike를 U에 넣고, 이탈 위치 p와 spike 마지막 비영 l 사이 `[p, l]`을 순환 순열, p행을 소거해 row eta를 만든다. 재분해 조건: `n_updates ≥ max_updates`, `n_updates ≥ m^0.45`이고 채움 증가, `n_updates % 10 == 9`이고 경과 시간 > 14×분해 시간 + …
- **FTRAN/BTRAN**: 희소 입력이면 Gilbert–Peierls DFS로 L, R(row eta), U를 차례로. 예상 밀도(`nnz·평균 ≤ m`, 조밀 플래그 비율 < 0.1)로 희소/조밀 커널 선택. BTRAN은 역순. 두 RHS를 한 번에 푸는 변형(DSE의 α_q와 τ)을 둔다.
- 해 재계산 뒤 잔차 `res = ‖Bx−b‖/…`, 크기비 `big`으로 MarkowitzTol 목표를 올린다: res>1e-8|big>1e10 → 0.5, 4e-9|1e9 → 0.25, 1e-9|1e8 → 0.125, 4e-10|1e7 → 0.0625, 1e-10|1e6 → 0.03125. "Warning: Markowitz tolerance tightened to %g".

## 5. Dual simplex (`lp::simplex::dual`) (노트 13 §3.4)

```
dual(spx):
  init: basis 적재(없으면 slack basis), 첫 분해, pricing 모드 결정 (§5.3), 박스 변수는 d_j 부호로 bound 쪽에 둠 (dual feasible화)
  while status == Running:
     if 대기 피벗 없음 && 재분해 요청 없음:
         y = BTRAN(c_B); d = c − Aᵀy; 박스 변수 flip; 정체(퇴화 비율 > 0.97 또는 목적 무진전)면 비용 섭동 시작/확대; sifting 전환 판정
         pricing 모드 점검 (3번째 재분해마다 Norm_Num vs Row_Num: 비가 3~4배 넘으면 devex로; 가중치 > 1e30이면 리셋)
     else: LU 갱신 또는 재분해(+가중치 갱신)
     x_B 재계산·잔차 검사(§4); 로그(100반복 또는 DisplayInterval); 한도 검사
     loop { rc = dual_iter() } until 실제 피벗 1회
     재분해 판단
  종료: OPTIMAL이면 비용 이동(cost shift)이 남았는지 확인 → 남으면 비용 복원, 1e-9 넘는 이동이 있으면 SwitchAlg(primal 정리)
        "Iteration: %7.0f   Optimal Objective = %16.7e"

dual_iter:
  r = chuzr()                         // §5.3. 없으면 최적 또는 재분해
  ρ_r = BTRAN(e_r), |ρ_i| ≤ zero 버림
  α_r = pivot_row(ρ_r)                // 행 방식(비기저 열 행 방향 사본 spx.row_major_nonbasic) vs 열 방식: n < 1.8·work면 열 방식
  ratio: pw_ratio == 0 ? harris_two_pass() : bfrt()        // §5.4
     후보 없음 → 갱신 이력 없으면 INFEASIBLE(Farkas: ρ_r), 아니면 재분해 후 재시도
     flip만 → 목적 갱신, x_B 갱신(flip 열 합을 한 번에 FTRAN), continue
  Δobj 반영, Cutoff 검사 → CUTOFF (MIP 노드용)
  α_q = FTRAN(a_q) (+ DSE면 τ = B⁻¹ρ_r 동시에)
  수치 검사: rel = |α_rq(행) − α_rq(열)| / (|α_rq(열)|+1); 부호 다르거나 rel > big_tol → 안전장치 사다리 (§5.5)
  |α_rq| < 1e-13 → ±1e-13
  x_B += θ·α_q ; basis_head[r] = q ; 이탈 변수 상태 ← bound ; 행 방향 사본 갱신(이탈 추가, 진입 제거) ; 가중치 갱신
```

### 5.3 pricing (CHUZR)

- 점수 `infeas² / w_r`, superbasic 목록 우선. NormAdjust 변형(열 스케일 비). **정수 변수 행 ×1.5, slack 행 ×0.9375**(MIP 편향, vtype 배열을 엔진이 안다).
- 두 방식: partial(순환, 후보 목록 재구성) / full(점수 최댓값의 1/10 이상인 행만 후보 목록에 남겨 재사용). `m < 15·k` 또는 `m < 반복 수`면 "Full pricing".
- 모드: 1 DSE 정확 초기 노름, 2 devex, 3 DSE 초기 가중치 1(quick-start), 0 없음. 초기: SimplexPricing<0 → 1; 웜스타트이고 저장된 노름 없으면 3 (단 `n > 100m && n > 0.1m²`면 1 유지).
- DSE 갱신 `w_i ← max(w_i − 2(α_i/α_r)τ_i + (α_i/α_r)²w_r, 1e-4)`, `w_r ← max(w_r/α_r², 1e-4)`; devex `w_i = max(w_i, (α_i/α_r)²w_r)`, 오차 비율 평균 > 5 또는 리셋 > 10이면 기준틀 재설정.
- 갱신 후 CHUZR 점수 배열과 후보 표식을 증분 갱신(다음 CHUZR은 목록만 훑음).

### 5.4 ratio test

- **Harris 2-pass**: pass 1 `θ_max = min (d_j + OptTol)/α_j` (후보 저장), pass 2 `d_j/α_j ≤ θ_max` 중 |α_j| 최대.
- **BFRT(bound flipping)**: breakpoint 후보를 모아 기울기 `δ`(이탈 행 위반)에 `|α_j|(u_j − l_j)`를 더하며 bound를 넘긴다(부분 정렬: 힙). 넘긴 변수 목록은 x_B 갱신에 한 번에 반영. **비용 이동(cost shifting)**: dual infeasible `d_j`를 만나면 위반이 `OptTol·0.0005` 미만(또는 `OptTol·0.05` 미만이고 range·위반 < OptTol·0.1)이면 `(1+rand)` 배율로 작게, 아니면 `(1+(rand%500+111)/1000)` 배율로 이동. 이동량은 끝에 되돌린다.
- `max|α| < pivot_tol/10` → INFEASIBLE 또는 재분해; `< pivot_tol·10` → "switch to primal because the chance of primal infeasibility" SwitchAlg.
- dual phase 1은 따로 두지 않는다(박스 변수 flip + 비용 이동/섭동 + primal 정리 = Koberstein "cost shifting + primal cleanup").

### 5.5 안전장치 사다리 (노트 13 §0.6)

행 피벗과 열 피벗이 어긋나면 순서대로: 갱신 횟수 > 0 → "Refactor triggered due to numerics." 재분해 → MarkowitzTol 0.5 → (quad 전환: M6) → "Basis recovery due to numerics." (≤6회) → 후보 거부 목록(reject list)에 넣고 "Rejection triggered due to numerics." → MarkowitzTol 0.9.

### 5.6 섭동 (노트 13 §3.4.5)

`δ_j = (rand%250+1)·(pert/500)·log2(1 + nnz_j·(0.7 + 0.3w_j))`, 정수 변수 w=1.5. 매번 3배로 키우고 상한 0.01(기본 PerturbValue) 또는 1000×PerturbValue. 수준 3이고 ≥0.01이면 SwitchAlg. 끝에 원래 비용으로 되돌리고 남은 dual 위반은 primal로 정리.

## 6. Primal simplex (`lp::simplex::primal`) (노트 13 §3.5)

```
init: 시작 basis(crash 또는 웜스타트, PStart superbasic "Primal warm-start: %d superbasic variables."), pricing 모드 자동 (기본 partial 0; 재시작≥3·수치 문제면 SE 1 아니면 devex 2; 웜스타트면 devex 단 n ≥ 3m이면 0), partial 청크 = n²/(m·max(n, k))
loop while Running:
   대기 피벗/재분해 → LU 갱신(05b257)
   해·쌍대 재계산; 위반 수 0 → phase 2
   한 반복: CHUZC (partial 또는 max d_j²/w_j 후보 58개) → FTRAN α_q → ratio (phase 2 / phase 1 composite / piecewise-linear phase 1) → "Numeric Warning: primal small pivot" 처리 → SE 가중치 갱신(BTRAN e_r, 행 계산, FTRAN)
phase 1: composite(위반량 목적) + piecewise-linear ratio test로 breakpoint 건너뜀
UNBOUNDED: 이탈 후보 없음 (광선 = α_q)
```
crash basis(`SIMPLEXCRASH`): 원본 crash를 포팅해 M1에 넣는다. 시작 basis가 다르면 반복 수가 처음부터 달라진다.

## 7. Sifting (노트 13 §3.7) — M6

dual이 `-1003`(열이 매우 많음)을 내면 진입. 작은 reduced cost 열 집합으로 부분 모델(`siftsub`)을 만들어 SiftMethod(-1 concurrent, 0 primal, 1 dual, 2 barrier)로 풀고, 전체 모델 BTRAN으로 가격을 매겨 음의 reduced cost 열을 추가. 로그 "Starting sifting (using %s for sub-problems)...", "    Iter     Pivots    Primal Obj      Dual Obj        Time", "Sifting complete".

## 8. Network simplex (노트 13 §3.8) — M6

조건: 웜스타트 없음, NetworkAlg=1 또는 자동이고 모델 등급이 순수 네트워크(모든 행 등식, 모든 열이 +1/−1 두 원소, 하한 유한). Σb≠0이면 즉시 INFEASIBLE. 구현은 원본 네트워크 단체법(노트 13 §3.8의 함수)을 포팅한다. 결과 basis를 `SimplexEngine::load_basis`로 넘겨 정리.

## 9. QP simplex (노트 13 §3.9) — M4

`a7f0ad`(순수 QP)이면 `cda5b` 대신 `qp_simplex`. 초기화: Q 대각 검사, 목적 0인 LP를 primal로 풀어 실행가능 basis. 루프: reduced-gradient 계열 active-set primal(superbasic 집합 관리). 실패 6회 초과면 FeasTol 10배 완화 "Warning: QP simplex relaxed tolerance to %g.".

## 10. 불가능·무한 증명, 종료 후 처리 (노트 13 §3.10–3.11)

- dual ratio에 진입 후보 없음 → INFEASIBLE, Farkas 증명 = ρ_r을 `FarkasDual`로. primal 이탈 후보 없음 → UNBOUNDED, 광선 = α_q를 `UnbdRay`로. `InfUnbdInfo=1`일 때만 계산·저장.
- Cutoff: dual 매 반복 + 외곽.
- UNSCALEDVIO: 비스케일 해의 위반이 FeasTol/OptTol을 넘으면 스케일을 모두 풀고 같은 basis에서 다시 푼다(반복 한도 20% 여유).
- 상태 메시지 "Optimal objective %.8e", "Warning: unscaled primal/dual violation = %g and residual = %g".

## 11. 로그

헤더 `Iteration    Objective       Primal Inf.    Dual Inf.      Time`, 행 ` %7.0f   %14.7e   %12.6e   %12.6e   %6.0fs`. DisplayInterval(초) 간격, `SDISPINTERVAL==0`이면 100반복마다. MIP 루트 문맥이면 "Root simplex log...".

## 12. MIP과의 계약 (노트 13 §3.12)

| 용도 | 호출 |
|---|---|
| 노드 LP 재풀이 | `engine.change_bounds(..); engine.solve(Dual, lim)` (basis·LU·가중치 유지). QP면 `qp_simplex` |
| strong branching | `engine.solve(Dual, lim.with_iter(k))`, 끝나면 bound 되돌림. 병렬 SB는 엔진 복제(`clone_for_thread`) |
| cut 추가/제거 | `add_rows`(slack basic으로 들어감, LU에 열 추가 = 갱신), `remove_rows` |
| Gomory | `tableau_row(r)` (= B⁻¹ 행 × A_N) |
| Cutoff | `p.cutoff`로 매 반복 검사 → CUTOFF 상태 5 |
| RCFIX | 엔진이 reduced cost를 내주면 MIP가 bound를 조인다 |

## 13. 포팅으로 확정할 지점

- LU 재분해 조건의 "경과 시간 > 14×분해 시간"이 벽시계인지 work인지. 벽시계면 P1 비교에서 그 지점 뒤 차이를 허용한다([10 §4](10_porting.md)).
- 섭동 크기와 비용 이동 배율의 난수 호출 순서. 난수 `a7b4d6`의 호출 횟수가 하나만 달라도 이후 섭동이 모두 달라진다.
- full/partial pricing 후보 목록을 다시 만드는 시점과, 벡터화된 점수 루프의 레인 순서(objdump로 확인).

## 14. 테스트와 검증

- 패리티 P1(합격 기준): Netlib 64개에서 Method 0(primal)·1(dual)·기본의 `IterCount`가 Gurobi와 같다(`re/bench/results/lp.csv`의 `grb_primal`, `grb_dual`, `grb_default`의 `it=` 값과 실시간 재실행). 다르면 SIMPLEX 콜백 추적의 첫 분기점을 `bench/parity.csv`에 남긴다.
- 패리티 P3: 같은 64개에서 시간 기하평균이 Gurobi의 1.2배 이내.

- LU: 무작위 희소 행렬·Netlib basis에 대해 `L·U = P·B·Q` 잔차 < 1e-10, FT 갱신 50회 뒤 잔차, 특이 행렬 감지.
- dual/primal: Netlib 104개 **전부 OPTIMAL**이고 목적값이 공개 최적값(`netlib_readme.txt`)과 상대 1e-6 이내. 불가능 인스턴스(`klein*`, `forplan` 등)는 상태 일치. 실행 시간은 보고만.
- hot start: 노드 LP 재풀이 반복 수가 cold start보다 작은지(통계 단언).
- 수치 사다리: 의도적으로 나쁜 스케일의 LP(`pilot87`, `greenbea`)에서 NUMERIC 없이 끝나는지.
- Rust `proptest`: 작은 무작위 LP를 밀집 참조 구현(현재 `crates/core`)과 비교.
