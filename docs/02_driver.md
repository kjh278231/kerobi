# 02. optimize() 드라이버

포팅 원본: [port_map.tsv](port_map.tsv)의 `02_driver`(169개). `GRBoptimize → 9828a1 → c23877`을 `driver` 크레이트 하나로 옮긴다. 근거: 노트 10, 11 전체.

**포팅 상태(2026-11-06, G5)**: 연속 LP 몫이 `crates/driver`에 있다(헤더 `header.rs`, `optimize`·`resolve_method`·`adjust`·세 LP 경로 `lib.rs`). 파라미터 표는 `crates/model/src/params.rs`(548개, gurobipy와 같음). Netlib 64개에서 Presolve 기본값의 Method 2·6(Crossover 0)은 상태·목적·BarIterCount/PDHGIterCount 64/64, Method 1은 상태·목적 64/64. 연속 LP의 상태 규칙(§7), uncrush(`363daa`, `365159`), 요약·반복 표 로그(§8)를 구현했다. 최신 simplex 웜스타트와 비스케일 재확인을 연결했고, barrier 공통 계수 열의 합산 경로를 보완했다. 시간 필드를 가린 요약은 barrier 64/64, dual 63/64이며 `etamacro`와 simplex 추가 반복의 차이는 남아 있다. 세부·지름길은 [m1_driver.md](m1/m1_driver.md).

## 1. 진입 순서

```
driver::optimize(model, env, cb) -> Result<(), SolverError>
 1. model.apply_pending()                         지연 변경 반영. 비용이 모델 크기 추정식을 넘으면 "Warning: excessive time spent in model updates."
 2. 배너 로그: 버전, CPU, "Thread count: %d physical cores, %d logical processors, using up to %d threads", "Non-default parameters:"
 3. p = env.p.clone()                             이번 optimize 전용 파라미터 사본 (자동 조정은 여기에만)
 4. 최대화면 obj, Cutoff, BestObjStop, BestBdStop 부호 반전                                   (노트 10 §1)
 5. scenarios.len() > 0 → multi_scenario(model)   확장 모델 만들어 solve_core 재귀           (노트 11 §3.16)
    아니면 rc = solve_core(model, p, first_call = true)
 6. rc ∈ {Q_NOT_PSD, QCP_EQUALITY} && 연속 모델 && (NonConvex==2 || (NonConvex==-1 && QCPDual==0))
       → "Continuous model is non-convex -- solving as a MIP|with NL barrier"; force_mip = true; solve_core(first_call = false)   (노트 11 §3.17)
 7. ResultFile 처리(.sol/.mps/.lp 쓰기). IIS는 비목표
 8. 부호 복원, Runtime/Work 속성 기록
```

`solve_core` = `c23877`. 다목적·sub-MIP·휴리스틱·시나리오가 70여 곳에서 재진입하므로(노트 11 `98b671`) **재진입 가능**해야 한다: 전역 상태 없음, env의 종료 플래그와 로그 싱크만 공유.

## 2. `solve_core` 단계

| 단계 | 내용 | 근거 |
|---|---|---|
| A 초기화 | `isMIP = classify(model)` (§3). 이전 presolved 모델이 OptimalityTarget이 바뀐 채 남았으면 폐기. ObjNumber·ScenarioNumber·SolutionNumber·StartNumber를 0으로 | 11 §3.1 |
| B 병렬 플래그 | `par = isMIP && ConcurrentMIP ≥ 2 && Threads ≠ 1`. 분산은 비목표 | 11 §3.2 |
| C 헤더 | 최상위 모델이고 first_call일 때 "Optimize a model with %d rows, %d columns and %lu nonzeros (%s)", 지문, "Model has …", "Variable types …", 계수 범위 통계(`a9a78e` → `a71757`). 102/102 로그와 같다([m1_driver §3](m1/m1_driver.md)) | 11 §3.3 |
| D 국소 최적화 제약 | OptimalityTarget=1이면 다중 시나리오·정수·SOS·PWL 목적·다목적·lazy가 있을 때 오류 10016/10024 | 11 §3.4 |
| E 다목적 | `multiobj.len() > 1` → `multiobj::solve` (§6) 후 종료 처리로 | 11 §3.5 |
| F Method 확정 | §4 | 11 §3.6 |
| G 볼록성 사전 검사 | QP/QCP/NL이고 NonConvex==0, 최상위: Presolve=0·로그 끈 presolve를 돌려 PSD/QCP 검사만 하고 결과는 버림 | 11 §3.8, 12 §3.1 |
| H 자동 조정 | NumericFocus, MIPFocus, Presolve=0→DualReductions=0, LazyConstraints→PreCrush≥1, MIP·NumericFocus=0→ObjScale=-3(내부값) 등 §5 표 | 11 §3.9 |
| I LP 디스패치 | 연속 모델: §4.2 | 11 §3.10 |
| J MIP 경로 | MIP start → presolve 전 휴리스틱 → MIP presolve → 결과 로그 → `OptimalityTarget==1 ? nl_barrier : bnb` → partition 휴리스틱 → ImproveStart 단계 | 11 §3.11 |
| K 상태 확정·uncrush | §7 | 11 §3.12 |
| L 요약 로그 | §8 | 11 §3.14 |

## 3. 모델 분류 (`classify`)

```rust
pub struct Class { is_mip, has_q_obj, has_qc, has_nl, is_qp /*목적만 2차*/, is_qcp /*2차 제약, NL·genconstr 없음*/, is_socp, has_pwl_obj, multiobj, scenarios, local_nlp }
fn is_mip(m) = force_mip || n_int > 0 || sos > 0 || pwl_obj > 0 || bilinear > 0 || genconstr > 0 || nl > 0 || multiobj || scenarios
```
(노트 10 §4 `a7f063`, `a7fb80`, `a7f0ad`, `a7f923`.) 비볼록 2차는 presolve 뒤에 bilinear 행으로 바뀌어 "이산"이 되므로, 분류는 presolve 뒤에도 한 번 더 한다.

## 4. Method 확정 (`resolve_method`)

노트 11 §3.6의 규칙을 그대로 표로 옮긴다.

| 문제 | 조건 | Method |
|---|---|---|
| MIP | 항상 | 그대로 -1 → 루트 LP 코드가 정한다(루트는 dual, 큰 모델은 concurrent/barrier: 노트 05 §1). LPWarmStart=1, SolutionTarget=-1 |
| QP | 사용자 3/4/5 → 1 + 경고, -1 → 2, PWL 목적 → 2, 6 → 2. Crossover 사본은 0 | |
| QCP/SOCP | 항상 2 | |
| LP | NetworkAlg=1 → 0(웜스타트 폐기). 웜 basis & LPWarmStart≠0 → 1(LPWarmStart=2·Presolve≠0·Crossover≠0·PWL 없음이면 2). PStart+DStart → Crossover≠0·PWL 없음이면 2 아니면 1. nnz < 30,000(PWL 목적이면 평균 PWL 점수로 문턱 확대) → 1. 그 밖 → 3 | |
| LP | SolutionTarget=1 & Method=-1 → 3, 스레드<9면 ConcurrentMethod=1 | |
| LP | SolutionTarget=0 & Method=-1 & Crossover≠0 → 모델 크기 통계가 "큼" 2개 이상이면 3(+CM=1) 아니면 2 | |

LPWarmStart가 -1로 남으면 1(PDHG면 2). Method 5는 deprecated 경고 후 3, ConcurrentMethod=3.

### 4.2 LP 디스패치

```
match method {
  3|4|5 => if plp_concurrent == 0 || pwl_obj || user_concurrent_envs || warm_basis || pstart_dstart { lp::concurrent::solve_replicated }   // 9b029d
           else { lp::driver::solve (내부에서 presolve 뒤 concurrent) }                                                             // 0c3d0d → 172cc6
  2 if crossover == 0 && solution_target != 1 => lp::barrier::solve (presolve 포함) ; QCP면 qcp_clean
  6 if crossover == 0 && solution_target != 1 => lp::pdhg::solve
  _ => lp::driver::solve(method)         // primal/dual simplex, 또는 crossover 켜진 barrier/PDHG (presolve → 엔진 → crossover → uncrush)
}
```
끝나면 LP 결과 속성(x, pi, rc, basis 또는 비기저 해)을 `Result`에 쓴다. `ConcurrentWinMethod` 기록.

각 경로는 presolve(`390989`, env의 Method·Crossover·SolutionTarget이 `state+0x100`과 PreDepRow 등을 바꾼다) → 엔진 → uncrush 순서다. `2bb770`·`34f033`으로 넘어가는 것: 상한만 있는 열을 뒤집은 모델 데이터(`07d3ce`, D+0x408), presolved 모델의 RNG 상태(`3b0917`→`895c5`: 13·n + 7·m + 23·nnz + Seed), presolve 작업량이 쌓인 전역 작업량 타이머(WorkLimit 판정). 셋 다 반복 수를 바꾼다([m1_driver §5–6](m1/m1_driver.md)).

## 5. 자동 조정 표 (한 번의 optimize 동안만)

| 조건 | 값 | 근거 |
|---|---|---|
| Presolve=0 또는 PoolSearchMode>1 | DualReductions=0 | 11 §3.9 |
| LazyConstraints≠0 | DualReductions=0, PreCrush=max(1,PreCrush) | |
| MIP, NumericFocus=0, ObjScale=0 | ObjScale=-3 (내부: 자동 스케일) | |
| NumericFocus≥1 | SimplexEndFac 2 | |
| NumericFocus=1 / ≥2 | AggMultLimit 1e10 / 1e6, MarkowitzTol ≥ 0.5 / 0.9, (≥2) GomoryPasses 0 | |
| NumericFocus=2 / 3 | CutPasses 3 / 0, CutAggPasses 1 / 0, (3) Quad=1 | |
| MIPFocus 1 | Cuts=1, CutPasses=5, VarBranch=1, Heuristics 0.2, RINS=100, CutOrthTol 0.6 | |
| MIPFocus 2 / 3 | Cuts 2 / 3, Presolve=2, (3) CutAggPasses=8, Aspire 0.2 | |
| Cuts≥2 | CutOrthTol 0.99 | |
| 순수 이진 작은 모델(set partitioning 꼴) | Cuts=0, RootProbe=NodeProbe=0, RINS=0; 열≥5000·이진<11이면 Cuts=0, Heuristics=0, DegenMoves=0, VarBranch=1 | 11 §3.11 |

## 6. 다목적 (`driver::multiobj`)

노트 11 §3.15. 1차 범위는 **우선순위 계층(blended within priority)**:
1. Q 목적·PWL 목적이 있으면 10024. PoolSearchMode는 0으로.
2. 단일 목적 사본을 만든다. 우선순위를 내림차순으로 묶고, 같은 우선순위는 `Σ ObjNWeight_i·c_i`로 합친다.
3. MultiObjPre: MIP이고 단계>1이면 사본을 목적 0으로 presolve 한 번(PreCrush=2). MultiObjPre=2면 `1e6`배 가중합으로 presolve.
4. 단계 p마다: 단계 env(`multiobj_envs[p]`)가 있으면 그 파라미터를 적용, TimeLimit/WorkLimit는 전역 한도의 남은 양으로. 시작해는 이전 단계 해. `solve_core(사본)`.
5. 허용 열화 제약 추가: `base = max(bnd + MIPGap·|val|, bnd + MIPGapAbs, val)`, `tol = max(ObjNRelTol·|base|, ObjNAbsTol)`, `Σc_j x_j ≤ base + tol − const`. MIP에서 gap이 1e-4 이하면 계수 변수를 한계에 고정. LP는 reduced cost 문턱으로 고정 + basis 웜스타트.
6. 끝에 uncrush, "Multi-objectives: solved in %.2f seconds (%.2f work units), solution count %d".

## 7. 상태 확정과 uncrush (노트 11 §3.12)

연속 LP는 LP 드라이버가 직접 정한다(포팅됨, [m1_driver §4.2, §8, §9](m1/m1_driver.md)): presolve가 증명한 상태는 `390989` 끝 규칙, simplex 경로는 `0c3d0d`(3·5 → 4, 끝에서 `a7fc52`면 3), barrier·PDHG 경로는 `36578f`(3은 `a7fc52(INFBOUND)` 거짓이면 4, 5는 `a80155` 아니면 4, OPTIMAL의 큰 uncrush 위반은 13). uncrush는 simplex 경로 `363daa` 뒤 원래 모델 simplex, barrier·PDHG 경로 `365159`. 아래는 MIP 경로(`c23877` B&B 뒤)의 규칙이다.

```
status = engine.status
if status == INFEASIBLE && pool.len() > 0 { status = OPTIMAL }
if Presolve≠0 && DualReductions≠0 && status ∈ {INFEASIBLE, UNBOUNDED} {
    status = if !objective_bounded_by_var_bounds(model, min(INFBOUND, PREINFBND)) { INF_OR_UNBD } else if 해 없음 { INFEASIBLE } else { SUBOPTIMAL }
}
x = postsolve::uncrush(model.postsolve, presolved.x)        // 노트 12 §3.9
if |obj(x) − presolved.obj| > 1e-4·(|obj|+1) { warn "uncrushing solution improved|degraded objective value from %s to %s" }
if status == SUBOPTIMAL && pool.len()==0 { status = NUMERIC }
if status == INF_OR_UNBD && pool.len() > 0 { status = UNBOUNDED }
MIP cleanup(MIPCLEANUP): 목적 계수 0 아닌 변수 고정 후 재풀이 → "Cleanup yields a better solution"
```

presolve가 모든 행·열을 지운 경우는 postsolve 기록만으로 해를 복원하고, 목적 상수를 Cutoff와 비교해 CUTOFF/OPTIMAL을 정한다.

## 8. 요약 로그 (노트 11 §3.14)

MIP 경로에서만(국소 NLP 포함, 미포팅). 순서: "Explored %.0f nodes (%.0f simplex iterations) in %.2f seconds (%.2f work units)" → "Thread count was %d (of %d available processors)" → "Solution count %d: …" → 상태 문구 표 → "Best objective %s, best bound %s, gap %s". 상태 문구는 노트 11 §3.14 표 그대로 쓴다. 연속 모델은 LP 드라이버와 엔진이 찍는다(포팅됨): simplex `"Solved in %d iterations and %.2f seconds (%.2f work units)"` + `c9457`(`"Optimal objective %16.9e"`), barrier·PDHG `"… solved model in … iterations …"`/`"… performed …"` + `"Optimal objective %.8e"`, 그 앞의 "Ordering time"·"Barrier statistics"·반복 표([m1_driver §10](m1/m1_driver.md)).

## 9. 한도 검사 (`limits::check`)

`a83b2f`를 포팅한다(노트 15 §6.2-9: 내부 미확인). 매 커널 루프가 `limits.check(work, now)`를 부른다. 우선순위 MEM_LIMIT(17) → terminate 플래그(11) → TimeLimit(9) → WorkLimit(16). MIP 쪽은 여기에 NodeLimit, IterationLimit, SolutionLimit, BestObjStop/BestBdStop(15), MIPGap(2)이 붙는다([06 §7](06_branch_and_bound.md)).

## 10. 포팅으로 확정할 지점 (노트 11 §5.2)

- ~~`model+0x2d8` 통계 배열을 채우는 함수~~: `a71757`(계수 범위 통계, `a9a78e`가 부름). [0,1] 목적 max/min, [4,5] bound, [6,7] RHS, [8,9] 행렬. SolutionTarget=0/1 자동 선택에 쓴다([m1_driver §3](m1/m1_driver.md)).
- `0c3d0d` 경로의 concurrent(PLPCONCURRENT=-1 → `172cc6`)와 `9b029d`의 실제 차이(원래 모델 vs presolved 모델).
- `48c43a`(QCPCLEAN), `48dcf2`(MINLP 다듬기), `75bf1e`, `086113`, `085968`, `75bc23`의 역할.
- TRIGGERRESTART가 세우는 B&B 플래그(+0x600, +0x604)의 소비처.
- 다목적: `1ad72e` 블렌딩 루프, `ae31e6` 형식, `1a33af` 정리, `1ab5c3` 콜백 중계, `1a47b3` reduced cost 고정(줄 1410–2035). §6의 1차 범위 서술은 포팅 결과로 바꾼다.
- NumericFocus가 MarkowitzTol(0x2478)을 올리는 값(오프셋 대응 재확인).

## 11. 테스트

있는 것(G5): `tests/test_params.py`(파라미터 548개 = gurobipy), `tests/test_header.py`(헤더 102개), `tests/test_driver.py`(Presolve 켠 세 경로 64개와 로그 앵커), `crates/driver` 단위 테스트(Method 확정·자동 조정 표). 계획:


- 패리티 P0: 로그 앵커가 Gurobi와 같은 순서로 나온다(배너, "Optimize a model with …", 계수 범위 통계, 요약). 같은 인스턴스에서 `Method` 확정 결과와 `ConcurrentWinMethod`가 같다.
- LP/QP/QCP/MIP 분류 단위 테스트(각 요소 하나씩 넣은 모델).
- Method 확정 표 전수 테스트(표의 각 행).
- 최대화 부호 반전 round-trip, Cutoff 부호.
- 다목적: gurobipy `multiobj.py` 예제, 허용 열화 제약 값.
