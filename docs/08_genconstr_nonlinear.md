# 08. 일반 제약, 2차·비볼록, 비선형(NL), 전파

포팅 원본: [port_map.tsv](port_map.tsv)의 `08_genconstr_nl`(350개). 진입 함수는 번역 `9ee0cf`, NL `5b1161`·`897f79`, OBBT `7aef0b`(angr 디컴파일), 국소 NLP 드라이버 `b2bba4` → `b1ffe1`이다. 근거: 노트 17 전체, 12 §2.4(QC/SOS 진입), 16 §3.5(OA/envelope 자리). 읽기 보조 문헌: Belotti et al. 2013(MINLP 서베이: factorable 재정식화, spatial B&B, FBBT, OBBT), McCormick 1976, Speakman & Lee 2017(부피 점수), Tawarmalani–Sahinidis(envelope), Gleixner et al. 2017(OBBT), Duran–Grossmann 1986(OA), Vielma–Ahmed–Nemhauser 2010(PWL 정식화·인코딩), Bonami et al. 2015(indicator), Wächter & Biegler 2006(내점법 NLP).

**핵심 결정(노트 17 §6)**: 독립 CP 엔진은 만들지 않는다. 논리·PWL 제약은 presolve에서 MIP로 번역하고, 함수·NL 제약은 비선형 제약 목록으로 옮겨 같은 B&B 안에서 envelope cut + 전파 + 공간 분기로 푼다. alldiff/cumulative 같은 전역 제약은 API에 없다.

## 1. 번역 (`mip::genconstr::translate`) (노트 17 §3.1–3.6)

presolve 마무리 단계에서 호출(`P.genconstrs` 비어 있지 않을 때). Presolve=0이어도 번역은 한다.

```
saved = FuncPieces ; FuncPieces == 0 → -2 (상대 오차 자동)
lock[] = (Presolve && DualReductions) ? 목적 부호·행 계수로 계산 : 0
버퍼: 새 변수, 새 행, 새 SOS, 새 2차 행 ; nl = Vec<NlConstr>
NL 결과변수·VARIABLE 노드 사용 횟수 세기
for gc:
   if gc.kind == Nl || (gc.kind.is_func() && (gc.func_nonlinear == 1 || (== -1 && env.FuncNonlinear /*기본 1*/))):
        sense = '=' ; Nl이고 PreCrush==0이고 y 사용 1회이고 lock 한쪽이면 '<' 또는 '>'로 완화
        nl.push(NlConstr { kind, sense, y, x/expr })
   else: translate_one(gc)        // 아래 표
함수 PWL 근사가 있으면 FuncMaxVal로 x,y bound를 자르고 구간 전파
버퍼 반영 → presolved 모델에 열/행/SOS/2차 행 추가 ; nl → P.nonlinear
국소 모드(OptimalityTarget=1)에서 새 정수/SOS가 생겼으면 10016 "A general constraint introduced discrete structures into the model"
```

| 종류 | 번역 |
|---|---|
| MAX `y = max(x_i, c)` (MIN은 부호 반전) | 전처리: 피연산자 bound로 y bound·정수성 전파. n==1·상수 없음 → `y − x = 0`. y가 한쪽 lock & PreCrush==0 → `y ≥ x_i`만. y 이진 → `y ≥ x_i`, `y ≤ Σx_i`. 일반: 슬랙 `s_i ≥ 0`(Presolve·유한 bound면 `ub = U(y) − L(x_i)`), 이진 `b_i`, `y − x_i − s_i = 0`, (상수) `y − s_c = c`, `Σb_i = 1`, `SOS1{s_i, b_i}` |
| ABS | `max(x, −x)` |
| AND / OR | 변수들을 'B'로; `min(x, 1)` / `max(x, 0)` |
| NORM 0 | 'B' n개 + SOS1 (경고 "Greater-than sense for 0-norm…") |
| NORM 1 | `x = x⁺ − x⁻`, SOS1{x⁺,x⁻}, `y = Σ(x⁺ + x⁻)` |
| NORM ∞ | `|x_i|`의 MAX |
| NORM 2 | 2차 행 `Σx_i² ≤ y²`(원뿔) 하나 |
| INDICATOR `z = v ⇒ aᵀx {≤,=} b` | z를 'B'로. 특수: 식이 ±x 하나·rhs≈0이면 슬랙 없이. 일반: 활동도 범위로 슬랙 bound(무한·≥1e8이면 1e100), 계수·변수·rhs 모두 정수면 슬랙 'I'. `≥`는 부호 반전. 슬랙 s, 행 `aᵀx − s {≤,=} b`. v==1: SOS1{z, s}; v==0: z' 추가, `z + z' = 1`, SOS1{z', s} |
| PWL `y = f(x)` 점 (x_j, y_j) | 국소 모드면 10016. 조각 1개: 'C' 2개 + 등식 2 + SOS1. 일반: λ_0..λ_k, `x = Σλ_j x_j`(첫·마지막 조각은 기울기로 lb/ub까지 연장), `y = Σλ_j y_j`, `Σλ = 1`, SOS2{λ}, 점프 구간(x_j == x_{j+1})마다 SOS1{λ_j, λ_{j+1}(, λ_{j+2})} |
| 함수(POLY…LOGISTIC) PWL 경로 | pieces = gc 속성 또는 FuncPieces; ≥2면 등간격(구간 = min(ub−lb, 주기)/pieces, SIN 2π, TAN π, 최소 1e-5); <0이면 오차 기반 적응(−1 절대, −2 상대; f, f′, f″ 평가로 중단점) ; FuncPieceLength ; 조각 > 2e8이면 오류 ; → PWL 번역 |

big-M은 번역에서 넣지 않는다. 이후 SOS 인코딩(§2)이 bound가 유한할 때 big-M으로 바꾼다.

## 2. SOS 인코딩 (`mip::presolve::sos_encode`) (노트 17 §3.9)

presolved 모델에 SOS가 남아 있고 PreSOS1BigM/PreSOS2BigM ≠ 0이면: 자동 big-M = 1000(NumericFocus==0이면 SOS1은 1e6). bound가 유한하고 big-M 한도 안이면 이진 재정식화. `PreSOS1Encoding/PreSOS2Encoding` 0–3: 0 자동, 1 **multiple-choice**(SOS1: `x_i ≤ u_i z_i`, `Σz = 1`; SOS2: 인접 쌍 z), 2 **incremental**, 3 **logarithmic**(Vielma 2010). 바뀌지 않은 SOS는 B&B의 SOS 분기(`sos_brrat` 비율로 분할점 선택)로.

## 3. 2차 (`mip::quad`) (노트 17 §3.8)

- **PSD 검사** `check_psd(Q)`: 대각뿐이면 이진 변수 대각은 건너뛰고 `q_ii < −PSDTol` → 10020. 아니면 Cholesky(`lp::barrier::cholesky` 재사용)로 필요한 대각 보정량 δ: `δ ≤ PSDTol` → "Warning: diagonal adjustment of %.1e performed to make Q PSD", `δ > PSDTol` → 10020 "Objective Q not PSD (diagonal adjustment of %.1e would be required)". 제약 Q는 "Constraint Q not PSD…" 10020. 시점은 `qpsd_check`(기본 -1: NonConvex≠0이면 지연 → NUMERIC일 때만 사후 검사).
- **비볼록**(NonConvex ∈ {-1, 2}): 비PSD 목적 Q → **epigraph** `t ≥ xᵀQx`(2차 제약 추가, 목적 Q 삭제). 2차 등식·비볼록 2차 제약 → **bilinear 행** `z_k = x_i x_j`(bilinear.c 역할)로 분해. 이후 `classify`가 "이산"으로 판정 → B&B. `pre_qlinearize>0`: 이진×변수 곱을 보조변수 + 선형 행(big-M = PreSOS1BigM, 음수면 1000)으로. `PreMIQCPForm` -1/2면 분해 SOC 형태. `rot_cone_as_bilin`.
- 완화: bilinear 행은 **McCormick 포락선** 4부등식, 2차 제약은 OA(접선) cut, 목적 Q는 QP 완화(QP simplex / QP barrier, M4). `MIQCPMethod` 0 = 노드마다 QCP 완화(barrier), 1 = OA LP.
- 연속 QCP/SOCP: barrier([05 §2.7]), `socp_outer`면 OA 루프.

## 4. NL 제약 (`mip::nl`) (노트 17 §3.7)

```
저장: NlExpr { opcode[n], data[n], parent[n] } (전위, parent[i] < i), opcode 0..19 (CONSTANT, VARIABLE, PLUS, MINUS, MULTIPLY, DIVIDE, UMINUS, SQUARE, SQRT, SIN, COS, TAN, POW, EXP, LOG, LOG2, LOG10, LOGISTIC, SIGNPOW ... gurobipy 번호 그대로)
presolve:
  decompose/normalize (POW는 상수 인자 하나 이상 "POW should have at least one constant argument")
  build_dag: 공통 부분식 공유(해시, nlconstrhash)
  dag_presolve (pre_nl, nldag_*): log/exp 평탄화, signomial 재정식화, sin/cos 표준화, sin²+cos²=1 인식, DIVIDE/PLUS/MULTIPLY 단순화, chain·중립 연산 제거, 고정, LOGISTIC/SIGNPOW 인식, "NL equations manipulated"
  fbbt: DAG 구간 전파 (앞으로: 구간 산술, 뒤로: 역상), 반복 고정점(work 한도)
  bounding_box: 무한 범위 노드에 인위 bound ±bounding_box(기본 1e6) "Nonlinear expressions terms artificially bounded to be in [%g,%g]"
  disaggregate: DAG의 각 비선형 노드에 보조변수 → 제약 재방출: 단변수 `y = f(x)`(type 10–18), 쌍선형 `z = x·y`(bilinear 행), 다항 `y = Σ a_k x^k`(POLY), 일반 NL(남는 것) "Added %i variables to disaggregate expressions"
```

B&B 안(노트 17 §3.7 표):

| 단계 | 설계 |
|---|---|
| 완화 | 단변수 `y = f(x)`: 볼록 구간에서는 접선(하한)·할선(상한), 오목은 반대, 변곡점·주기(sin/cos: fmod로 구간 분할)별 처리 → `Envelope` cut(kind 0x12). 쌍선형: McCormick. 2차: OA(kind 0x11). RLT(두 clique 표 + 곱 행), BQP, PSD cut은 기본값 1이므로 M4 |
| 전파 | 노드마다 `propagate` 워크리스트([§5])가 `y=f(x)`의 x↔y 구간(역함수 포함), 곱 구간, 선형 행을 함께 |
| 분기 | `branch` 디스패치에 공간 후보: 위반 비중 × McCormick 부피 감소 `((x̂−l)²+(u−x̂)²)w_y²/6`의 `log(1+ΔV)+1`; 단변수는 위반점에서 분할. `spatial_branch` 자동 |
| OBBT | 루트에서 `OBBT` 레벨 > 0: 목적 cutoff 행("cutoff")을 넣은 LP 완화에서 변수마다 min/max(`solve_core` 재진입, Obj 바꿔가며), 새 bound = 교집합. 레벨 1 축소 모드, >2 범위 확대 |
| 휴리스틱 | `NLPHeur`(-1 → 3): 정수 고정 서브모델에 OptimalityTarget=1을 넣어 NL barrier로 국소해(M5) |
| 국소 모드 | OptimalityTarget=1 → `nl_barrier`(§6) |

## 5. 도메인 전파 (`mip::propagate`) (노트 17 §2.4, 6.3)

```rust
pub struct Propagator { queue: VecDeque<ConstrRef>, in_queue: BitSet, work_limit: f64 /* 100 × 제약 수 */ }
impl Propagator {
    pub fn run(&mut self, dom: &mut Domain, cons: &ConstraintSet) -> Result<Changes, Infeasible>
    // 범주별: 선형 행(activity 기반 bound), SOS1/SOS2, 2차·곱 항(x², xy 구간), 단변수 NL(y=f(x) 역상), bilinear
    // bound 변경 → 그 변수를 포함한 제약을 큐에 (sense '<'/'>' 별)
}
```
호출처: node presolve, 루트, probing, diving/fix-and-propagate, feasibility pump 반올림, sub-MIP 고정. MIP presolve의 bound 강화와 같은 activity 코드를 공유한다.

## 6. NL barrier (`nl::barrier`) — M5 (노트 17 §2.5, 3.7 "국소 풀이")

정수·SOS·PWL·다목적·lazy가 있으면 거부(10016/10024). 원본 NL barrier(`b2bba4` → `b1ffe1`, NLPALG)를 포팅한다. 노트로 알려진 구성은 자동 미분(forward 1차 + 희소 Hessian: DAG 역방향), KKT 행렬 `[W + Σ  Jᵀ; J  −D]` 인수분해(barrier의 LDLᵀ 재사용), line search(`nlbar_ls`), 초기 μ(`nlbar_init_mu`), 파라미터 NLBarIterLimit/PFeasTol/DFeasTol/CFeasTol이다. 세부가 이 서술과 다르면 원본을 따른다. 로그 "NLbar: status %d, obj %.6e, iters %d, viol %.2e", "Hessian NZ", "Jacobian NZ". 상태 18 "First-order optimal solution" / 19 "First-order infeasible model". PStart가 있으면 "Loaded warm-start solution". `nlp_alg=2`(Ipopt 자리)는 비목표.

## 7. API 매핑

`Model.addGenConstrMax/Min/Abs/And/Or/Norm/Indicator/PWL/Poly/Exp/Expa/Log/Loga/Pow/Sin/Cos/Tan/Logistic/NL`(gurobipy 시그니처), `nlfunc.sin(x) …`가 `NLExpr` 트리를 만들어 `addGenConstrNL(y, expr)`로. 속성 `FuncPieces/FuncPieceLength/FuncPieceError/FuncPieceRatio/FuncNonlinear`는 제약별 덮어쓰기.

## 8. 포팅으로 확정할 지점 (노트 17 §7.3)

- 노트의 파일 매핑 중 `inf`/`edge`(주소 연속성 추정) 함수의 실제 소속.
- double형 숨은 파라미터 오프셋(특히 NLCFEASTOL 0x23e0) 재확인. OBBT(0x2808)는 파라미터 표에 넣는다([01 §1.2](01_model_and_api.md)).
- 국소 NLP 경로(`c23877` 2581–2615 → `b2bba4`)의 상태 18/19 판정.

## 9. 테스트

- 패리티 P0(합격 기준): 각 일반 제약 예제의 presolved 모델(열·행·SOS·2차 행 수, "Presolved model has %d SOS …")이 Gurobi `Model.presolve()`와 같다.

- 번역 단위: 각 종류를 작은 모델에 넣고 presolved 모델의 행/열/SOS 수와 Gurobi 로그("Presolved model has %d SOS")가 같은지(bench 환경에서 수집), 해 round-trip.
- gurobipy 예제 `genconstr.py`, `piecewise.py`, `gc_pwl.py`, `gc_pwl_func.py`, `bilinear.py`, `qcp.py`, `sos.py`, `multiobj.py`를 돌려 목적값 비교.
- FBBT: 구간 산술 단위 테스트(각 opcode 전방/후방).
- 비볼록: `bilinear.py`(x·y 곱, NonConvex=2) 전역 최적값 1e-6.
- NL barrier: Hock–Schittkowski 소형 문제 10개로 국소해 일치.
