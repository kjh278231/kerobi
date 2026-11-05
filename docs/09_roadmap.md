# 09. 로드맵, 검증 전략, 위험

## 1. 마일스톤

각 단계는 "완료 조건"을 통과해야 다음으로 간다. 완료 조건은 패리티 단계 P0–P3([10 §1](10_porting.md))로 적는다. 추정 규모는 [00 §7](00_overview.md).

**마일스톤마다 속도 스냅숏**: 관문을 통과하면 Gurobi·OR-Tools·rsolver의 속도를 한 표로 남긴다. 분석이 아니라 확인용이다.
- `re/bench/bench.py`의 `LP_SOLVERS`(M2부터 `MIP_SOLVERS`에도)에 `rsolver_M<n>` 항목을 더한다. 설정은 옆 Gurobi 항목과 같게 한다(LP는 Threads=1, 기본 Method). 이전 마일스톤 항목은 지우지 않는다.
- `python re/bench/bench.py lp`(M2부터 `mip`도)를 돌린다. 이미 있는 (인스턴스, 솔버) 쌍은 건너뛰므로 새 rsolver 줄만 쌓이고, Gurobi·OR-Tools는 기존 값을 쓴다.
- `python re/bench/report.py lp`의 요약 표를 그 마일스톤의 "상태" 줄 아래에 붙인다.
- 기계가 한가할 때 돌린다. 다른 세션이 돌고 있으면 시간이 부풀어 비교가 안 된다.

기간: 이전 추정(합 30–40주)은 사람이 재작성하는 기준이라 폐기한다. 아래 괄호의 주 수는 이전 값이고 참고용이다.

**재산정 (2026-11-05, 에이전트 실측 기준)**
- 실측: 설계와 M0에 약 2.8시간(11:00–13:49) 들었다. M1의 반복 0 상태 비트 일치(스케일링, 시작 기저, singleton LU, FTRAN·BTRAN, DSE 가중치)까지 약 1.2시간 들었다. 이때 손댄 함수는 10개, 의사코드 약 2.5만 줄이다. 기본 경로만 옮겼으므로 실효 속도는 시간당 약 1만 줄로 본다.
- 남은 양은 약 108만 줄이다(`port_map.tsv` optimize 경로 121만 줄 기준). 반복마다 오차가 쌓이는 코드(simplex 반복, B&B, cut)는 더 느리므로 시간당 3천–8천 줄로 잡는다.

| 단계 | 의사코드 | 에이전트 작업 시간 |
|---|---:|---:|
| M1 나머지 | 약 27만 줄 | 15–40시간 |
| M2 | 약 45만 줄 | 60–150시간 |
| M3 | 약 12만 줄 | 15–40시간 |
| M4 | 약 8만 줄 | 10–25시간 |
| M5 | 약 10만 줄 | 15–35시간 |
| M6 | 약 5만 줄 | 15–40시간 |
| 합계 | | 130–330시간 |

- 하루 10시간, 에이전트 1개 기준으로 약 2–5주다. 병렬로 돌려도 Gurobi 한 프로세스 제한 때문에 절반까지는 줄지 않는다.
- 근거가 반복 0 구간의 1.2시간 측정 하나라 오차가 크다. M1 반복 단계를 몇 시간 진행한 뒤 다시 잰다. 크게 틀어질 수 있는 곳은 세 가지다: 반복 1 이후의 P1 90%, M2 노드 수 일치, 사용량 한도로 인한 중단.

### M0 — 뼈대 (1–2주) — 관문 통과 2026-11-05
- `model` 크레이트: `Model`, `SparseMat`, `Params`(표에서 생성), `Attrs`, MPS/LP 읽기·쓰기, 로그 싱크, `Work`/`Limits`.
- `driver::optimize` 골격: 분류, Method 확정, 부호 반전, 상태 확정, 요약 로그. 엔진은 현재 `crates/core`의 밀집 simplex·B&B를 임시로 연결.
- `py` + `python/rsolver`: gurobipy 모양 API 1차 범위([01 §3]), 파라미터 set/get, 속성 `__getattr__`, 콜백 브리지(where=POLLING/MESSAGE만).
- 로컬 git 저장소, `tools/parity.py`(차등 하네스, [10 §6](10_porting.md)), 원본 MPS 리더·지문·파라미터 등록(`ac81d7`) 포팅.
- 상태(2026-11-05): **P0 지문 102/102 일치로 관문 통과.** `crates/model`(모델 배열, MPS 리더, 지문), PyO3 `read_model`, Python `read`/`Env`/모델 속성/콜백 상수, 하네스의 rsolver 실행이 끝났다. 파라미터 표와 드라이버 골격은 M1에서 LP 드라이버와 함께 만든다.
- 완료: Netlib 소형 10개를 MPS로 읽어 밀집 엔진으로 OPTIMAL; `tests/test_api.py` 통과; `cargo clippy` 경고 0; **P0 지문**: Netlib 64개 + `miplib_small` 38개의 지문이 Gurobi와 같음; 파라미터 기본값 전부 일치; parity 하네스가 Gurobi 쪽 추적을 저장함.

### M1 — LP 엔진 (4–6주)
- `lp::lu`(Markowitz + FT + hypersparse + 밀집 단계), `lp::simplex::{dual, primal}`(원본 pricing·ratio test·섭동·안전장치·비스케일 검사), crash, Quad(`F80`), 스케일링, `lp::presolve`(`390989`) 중 기본값에서 도는 reduction 전부, postsolve, LP 드라이버(`0c3d0d`), 난수 `a7b4d6`·정렬·해시 등 `common`.
- 완료: Netlib 91개 전부 상태 일치, 목적 상대 1e-6(Presolve 켬/끔 모두); **P0** Netlib 64개 presolved 모델 일치; **P1** Netlib 64개의 Method 0/1/기본 `IterCount`가 Gurobi와 같은 인스턴스 ≥ 90%, 나머지는 ±5%이고 첫 분기점 기록; **P2** work ±5%; **P3** 시간 기하평균 ≤ 1.2배; `crates/core` 삭제.

### M2 — MIP 엔진 (6–8주)
- `mip::presolve`(`3cd406`, 기본값에서 도는 reduction 전부 + probing·대칭성·연결 성분), `mip::bnb`(노드 선택 규칙 12개 전부, reliability branching, 원본 pseudocost 갱신, termination, 재시작, 로그), `mip::propagate`, `mip::cuts`(기본값 1 이상인 separator 전부, pool, 선택, LP 안 cut 정리), `mip::heur`(스케줄러가 기본값에서 부르는 휴리스틱 전부, sub-MIP 엔진), 콜백 MIP/MIPSOL/MIPNODE + cbCut/cbLazy/cbSetSolution, MIP start.
- 완료: 모든 해 위반 ≤ 1e-6(`bench.py:violation`); **P0** `miplib_small` 38개 presolved 모델 일치; **P1** Threads=1에서 루트 LP 반복 수·루트 bound·"Cutting planes:" 수·노드 수가 Gurobi와 같은 인스턴스 ≥ 80%, 나머지는 첫 분기점 기록; 60초 상태가 `mip.csv`의 Gurobi 상태와 같음(OPTIMAL 20, INFEASIBLE 2); job-shop big-M 5개 OPTIMAL; **P3** OPTIMAL까지 시간 기하평균 ≤ 1.2배.

### M3 — Barrier, PDHG, Concurrent (4–5주)
- `lp::barrier`(원본 ordering + ND, 밀집 열, supernodal LDLᵀ + MKL, 반복 루프, 종료 판정), `lp::crossover`(push, cleanup, 재시작), `lp::pdhg`(원본 전처리 포함), `lp::concurrent`(공유형 + 복제형), Method 자동 선택 표 전체, PreDual, network simplex, sifting.
- 완료: Netlib 91개 Method=2(Crossover 0/기본), 6, 3, 4에서 상태·목적 일치; Method=4 결정성; **P1** `BarIterCount`가 `grb_barrier`의 `bar=`와, PDHG 반복 수가 `grb_pdhg`와 같음; Factor NZ 일치; **P3** 시간 기하평균 ≤ 1.2배.

### M4 — 일반 제약, SOS, 2차 (4–5주)
- `mip::genconstr::translate`, SOS 분기·인코딩, `mip::quad`(PSD 검사, epigraph, bilinear 분해, McCormick, OA cut, QP simplex, QP/QCP barrier, QCPDual), 공간 분기 점수, presolve Q reduction, NORM, PWL 목적.
- 완료: gurobipy 예제 `genconstr/piecewise/gc_pwl/sos/qp/qcp/bilinear` 목적값 일치; MIQP 소형(`miplib`의 QP 변형 5개) OPTIMAL.

### M5 — NL, OBBT, 국소 NLP, 다목적·시나리오·해 풀 (5–6주)
- `mip::nl`(DAG, dag_presolve, FBBT, disaggregate, envelope cut), OBBT, NLPHeur, `nl::barrier`, 다목적 계층, 다중 시나리오 확장 모델, PoolSearchMode/PoolSolutions, node file, ImproveStart, zero-objective/MinRel/int2bin/presolve 전 휴리스틱/MIPCLEANUP, Inf proof cut.
- 완료: `gc_pwl_func.py`(FuncNonlinear 0/1), `multiobj.py`, `multiscenario.py`, `poolsearch.py` 결과 일치; HS 문제 10개 국소해.

### M6 — 병렬·성능·나머지 (6–8주)
- 결정적 병렬 B&B, 병렬 strong branching·probing, concurrent MIP, 재시작, 적응형 separator/휴리스틱 제어, 나머지 separator(flow cover/path/network, strongCG, mixing, mod-k, L&P, projected/dual implied, RLT/BQP/PSD), NoRel, partition, sifting, network simplex, 공유형 concurrent LP, 밀집 열·ND ordering·bar_iterative, PDHG 불가능성 증명, LP presolve 전용 상태, double-double simplex(Quad), 대칭성, 연결 성분, sparsify, 비영 소거.
- 완료: Threads=8 결정성; **P1** Threads=8에서도 노드 수가 Gurobi와 같은 인스턴스 ≥ 80%; **P3** `miplib_small` 38개 기본 스레드 시간 기하평균 ≤ 1.2배; `bench.py`에 `rsolver` 백엔드 추가해 비교표(노트 21 형식) 생성.

## 2. 검증 전략

| 층 | 방법 | 도구 |
|---|---|---|
| 단위 | 각 알고리즘의 교과서 예제, 경계 조건, 상태 전이 표 전수 | `cargo test`, `proptest` |
| 수치 | LU 잔차, Cholesky 잔차, FBBT 구간 포함성 | 단위 테스트 + 무작위 행렬 |
| 등가성 | presolve 켬/끔, Method 0/1/2/3/6, Threads 1/8의 목적값 일치 | `tests/equivalence.rs` (Netlib, miplib_small) |
| 유효성 | 모든 해를 **원래 모델**에서 위반 재계산(행·bound·정수성·SOS·genconstr·2차·NL) | `model::verify::max_violation` (bench.py:violation 이식) |
| cut 유효성 | 작은 IP의 정수점 전수 열거로 cut이 유효한지 | proptest |
| 결정성 | 같은 입력을 3회 돌려 NodeCount/IterCount/ObjVal 비트 일치 | CI |
| 회귀 | `bench.py`에 `rsolver` 추가, 결과 CSV를 커밋해 시간·노드·반복 추이 추적 | `re/bench` 확장 |
| 패리티 | P0 presolved 모델 diff, P1 콜백 추적·첫 분기점, P2 work, P3 시간 | `tools/parity.py`, `bench/parity.csv` |
| API | gurobipy 예제 스크립트를 import만 바꿔 실행 | `tests/test_examples.py` |
| 로그 | 앵커 문자열(노트 04)이 같은 순서로 나오는지 | 로그 스냅숏 테스트 |

성능(P0–P3)이 **합격 기준**이다. 패리티를 깨는 단순화는 `shortcut:` 주석과 함께 `bench/parity.csv`에 원인을 남긴다.

## 3. 저장소 구성 (목표)

```
rsolver/
  Cargo.toml                  workspace: crates/{model,lp,mip,driver,py}
  crates/model/src/{lib,model,sparse,params/{table,gen},attrs,io/{mps,lp,sol},log,work,verify}.rs
  crates/lp/src/{lib,driver,presolve/..,scaling,lu/{factor,update,solve},simplex/{engine,dual,primal,pricing,ratio,perturb},barrier/{..,ordering,cholesky},crossover,pdhg,concurrent}.rs
  crates/mip/src/{lib,presolve/..,bnb/{mip,tree,node,select,branch,pseudocost,termination,restart,parallel,nodefile},cuts/{pool,select,dispatch,gomory,mir,cover,clique,implied,zerohalf,..},heur/{scheduler,submip,rounding,fixprop,diving,rins,pump,..},propagate,genconstr,quad,nl/{expr,dag,fbbt,envelope,obbt,barrier}}.rs
  crates/driver/src/{lib,optimize,classify,method,adjust,multiobj,scenario,summary}.rs
  crates/py/src/lib.rs
  python/rsolver/{__init__,grb,core,expr,model,matrix,helpers,nlfunc,exception,tupledict}.py
  tests/                      pytest
  docs/                       이 문서들, port_map.tsv(생성물)
  tools/                      port_map.py(함수→모듈 배정), parity.py(차등 하네스)
  bench/                      re/bench 연동 스크립트, parity.csv
```

## 4. 위험과 대응

| 위험 | 영향 | 대응 |
|---|---|---|
| **법적**: 디컴파일 결과를 포팅하므로 결과물은 Gurobi 코드의 2차적 저작물이고 EULA·저작권 위반 소지가 크다(사용자 결정 2026-11-05) | 공개·배포 불가가 확정된다 | 로컬 git만 쓰고 원격에 올리지 않는다. 내부 연구용으로 한정. 공개가 필요해지면 해당 코드는 클린룸으로 다시 쓰고 법률 검토를 거친다. 라이선스 코드는 포팅하지 않는다 |
| 규모: 포팅 대상이 의사코드 120만 줄(Rust 24만–40만 줄 추정)로 이전 설계의 4–7배 | 일정 | 기본 경로부터, 마일스톤마다 패리티로 끊는다. M0에서 속도를 재 일정을 다시 잡는다 |
| Ghidra 의사코드 오류(goto 재배치, 빠진 인자, 잘못된 타입, 벡터화 루프 레인 손실) | 비트 불일치, 잘못된 동작 | objdump 대조(10 §3), 차등 검사의 첫 분기점으로 범위를 좁힘 |
| **M1 실측(2026-11-05)**: 수치 함수 대부분에서 의사코드가 호출 인자를 잃고 SIMD를 조각내, 줄 단위 옮겨 쓰기가 사실상 불가능하다. 스케일링 하나의 기본 경로 해독에 asm 대조가 여러 번 들었다 | M1 일정이 수 주 단위로 늘어남 | 방식 결정 대기: 함수별 재구성 + 상태 오라클(`tools/spx_oracle.py`)로 비트 검증, 필요하면 함수 단위 런타임 훅 추가 |
| 비트 단위 일치 실패(FMA·연산 순서·libm·MKL 설정) | P1 불일치 | 10 §4 규칙, MKL 설정 호출부 포팅, 차이는 첫 분기점 기록 후 ±5% 허용 |
| 크기 제한 라이선스(변수·제약 2,000개) | 큰 모델은 Gurobi와 나란히 못 돌림 | 현재 벤치 102개는 모두 한도 안. 큰 모델은 저장 결과와 비교만 |
| 수치 안정성(LU·barrier)에서 Netlib 어려운 인스턴스(`pilot87`, `greenbea`, `dfl001`) 실패 | M1/M3 지연 | 안전장치 사다리 전체 구현, double-double 폴백(M6), 실패 인스턴스를 회귀 목록으로 |
| 노트가 "미확인"·"추론"으로 남긴 부분 | 패리티 불일치 | 각 문서의 "포팅으로 확정할 지점"에 함수를 적어 두었다. 문헌으로 메우지 않고 원본을 읽어 확정 |
| Python 레이어 성능 | 큰 모델 구성 느림 | 핫 클래스를 PyO3로(00 §6-8), 일괄 API(numpy 버퍼)와 `MVar` |
| 범위 폭주 | 완료 못 함 | 마일스톤 완료 조건을 테스트로 고정. 숨은 파라미터·숨은 separator는 "자리만" |

## 5. 작업 규칙

- 모든 새 모듈은 설계 문서의 절 번호를 doc comment 첫 줄에 적는다(`//! 04 §5.4 BFRT`).
- 코드에 출처 태그는 달지 않는다. 구조체 필드 doc comment의 원래 오프셋(10 §3-2)은 패리티 디버깅용으로 둔다.
- 숨은 파라미터는 파라미터 표에 둔다. 원본 코드 안의 상수는 옮긴 자리에 그대로 두고 상수 이름만 붙인다.
- 원본에 없는 규칙을 더하지 않는다. 불가피하면 `shortcut:` 주석을 달고 `bench/parity.csv`에 기록한다.
- `shortcut:` 주석 규칙(현재 코드와 같음)으로 미구현·단순화를 남기고, `/ponytail-debt`로 집계한다.
