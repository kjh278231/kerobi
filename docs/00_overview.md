# 00. 전체 설계 개요

대상: Gurobi 13.0.3 코어(`re/decomp/c/`의 디컴파일 결과)를 **포팅 원본**으로, `re/notes/00–20`을 그 지도로 삼아 LP / QP / MILP / MIQP / 일반 제약 / 비선형(전역·국소) 문제를 푸는 솔버를 Rust 코어 + PyO3 + Python 레이어로 만든다. 목표는 원본과 **같은 성능**이다.

## 1. 목표와 비목표

**목표**
1. Python API(`Model/Var/Constr/LinExpr/QuadExpr/GRB`, `optimize()`, 속성·파라미터)로 같은 종류의 문제를 푼다.
2. 내부 구조는 노트의 구조를 따른다: 하나의 드라이버 → presolve → (LP 엔진 | 분기 절단 엔진) → postsolve. 일반 제약과 비선형은 별도 엔진 없이 presolve 번역 + 같은 B&B 안에서 푼다(노트 17 §6).
3. 결과가 **검증 가능**하다: 모든 해는 원래 모델에서 위반량을 재고(`re/bench/bench.py:violation`과 같은 기준), LP는 Netlib, MIP는 MIPLIB 소형 인스턴스(`re/bench/data`)로 Gurobi·OR-Tools 결과와 대조한다.
4. 결정적(deterministic)이다: 같은 입력·파라미터·스레드 수면 같은 결과. 시간이 아니라 **work 단위**로 한도와 동기화를 잰다(노트 11 §1, 13 §3.3, 14 §3.4.3, 15 §3.12).
5. **원본과 같은 성능**: 같은 인스턴스에서 같은 presolved 모델, 같은 반복 수·노드 수, 같은 work, 비슷한 시간. 정의와 합격선은 [10 §1](10_porting.md)의 패리티 단계 P0–P3이다.

## 2. 출처 정책 (반드시 지킨다)

사용자 결정(2026-11-05): 원본 성능을 그대로 가져가기 위해 디컴파일 결과를 포팅 원본으로 쓴다. 이전 정책(알고리즘 본문은 문헌·OR-Tools로 새로 쓴다)은 폐기한다.

- **알고리즘 본문은 `re/decomp/c/`의 해당 함수를 옮긴다.** 제어 흐름, 분기 조건, 상수, 연산 순서, 난수·정렬·해시를 그대로 둔다. 바꾸는 것은 이름과 Rust 관용 구조(소유권, `Result`, 구조체 필드)뿐이다. 절차와 수치 규칙은 [10](10_porting.md).
- **어느 함수를 어느 모듈로 옮기는지**는 [port_map.tsv](port_map.tsv)가 정한다(`tools/port_map.py`가 노트와 호출 그래프로 생성). 각 설계 문서의 "포팅 원본" 줄은 그 모듈의 진입 함수다.
- **코드에 출처 태그를 달지 않는다**(사용자 결정 2026-11-05). 원본 함수와 Rust 모듈의 대응은 `port_map.tsv`가 모듈 단위로 기록하고, 진척은 패리티 결과로 잰다.
- **저장소는 로컬 git만 쓴다.** 원격 저장소에 올리지 않는다.
- **`re/notes`는 지도다.** 함수의 역할, 오프셋의 의미, 파라미터 대응을 읽는 데 쓴다. 노트가 "미확인"·"추론"으로 남긴 곳은 문헌으로 메우지 않고 해당 함수를 읽어 확정한다.
- **문헌과 OR-Tools는 읽기 보조**다. 의사코드가 무슨 알고리즘인지 이해하는 데 쓰고, 코드의 출처로 쓰지 않는다.
- **포팅하지 않는 것**: 라이선스·WLS·batch·튜닝·서버 코드(`excluded`), GPU, 분산, MKL·OpenSSL 내부. MKL은 링크해서 같은 루틴을 부른다.
- **법적 위치**: 결과물은 Gurobi 코드의 2차적 저작물이다. 저장소는 비공개, 배포·공개 금지, 내부 연구용으로 한정한다([09 §4](09_roadmap.md)).


## 3. 계층 구조

```
[Python]   rsolver/python/rsolver/      Model, Var, Constr, LinExpr, QuadExpr, GenExpr, GRB, Env, 콜백
     │  (PyO3, GIL 해제한 채 solve. 콜백 시점에만 GIL 재획득: 노트 02)
[PyO3]     crates/py                    rsolver._core: PyModel(핸들), optimize(), get/set attr·param, 콜백 브리지
     │
[Rust]     crates/model                 Model 자료구조(CSC+CSR), Env/Params, Attrs, MPS/LP 파일 IO, 로그
           crates/lp                    LP presolve, 스케일링, LU, dual/primal simplex, barrier, crossover, PDHG, concurrent
           crates/mip                   MIP presolve, 일반 제약/NL 번역·DAG, cut pool·separator, 휴리스틱, B&B 트리, 전파
           crates/driver                optimize() 드라이버(문제 분류, Method 확정, 상태 확정, 요약 로그), 다목적, 다중 시나리오
```

- 의존 방향: `model ← lp ← mip ← driver ← py`. `lp`는 `mip`를 모른다. `mip`는 `lp`의 `SimplexEngine`(hot start)과 `tableau_row()`만 쓴다.
- 지금 있는 `crates/core`(밀집 simplex + 단순 B&B)는 `lp`/`mip`가 각각 첫 테스트를 통과하는 시점에 지운다.

## 4. 노트 ↔ 모듈 대응표

함수 단위의 전체 배정은 [port_map.tsv](port_map.tsv)에 있다. 아래 "진입 함수"는 각 모듈 포팅의 시작점이다.

| 노트 | 진입 함수(포팅 원본) | 모듈 | 설계 문서 |
|---|---|---|---|
| 01, 02 | gurobipy `_model.so` → `GRBoptimize` | `py`, `python/` | [01](01_model_and_api.md) |
| 10, 11 | `GRBoptimize` → `9828a1` → `c23877` | `driver::optimize` | [02](02_driver.md) |
| 12 | `390989`(LP), `3cd406`(MIP), `363164`(postsolve) | `lp::presolve`, `mip::presolve`, `model::postsolve` | [03](03_presolve.md) |
| 13 | `cda5b`, `06268b`, `0478d3`, `036ade`, `05b257` | `lp::simplex`, `lp::lu` | [04](04_simplex.md) |
| 14 | `2bb770`/`2bf1ea`, `13f716`, `34c004`, `172cc6` | `lp::barrier`, `lp::crossover`, `lp::pdhg`, `lp::concurrent` | [05](05_barrier_pdhg_concurrent.md) |
| 15 | `5bbf7f`, `5c239f`, `6e8eb3`, `5fb8ec`, `62a05d` | `mip::bnb`, `mip::node`, `mip::branch` | [06](06_branch_and_bound.md) |
| 16 | `64da9f`, `7b77cd`, `6d8986`/`6de7ca`, `8ad3a3` | `mip::cuts`, `mip::heur` | [07](07_cuts_heuristics.md) |
| 17 | `9ee0cf`, `5b1161`, `897f79`, `646052`, `7aef0b` | `mip::genconstr`, `mip::nl`, `mip::propagate` | [08](08_genconstr_nonlinear.md) |
| 03, 04, 05, 20, 21 | 배치, 로그 앵커, 파라미터, OR-Tools 비교 | 로그 문자열·파라미터 표·벤치 | [09](09_roadmap.md) |
| 전체 | `port_map.tsv`, 패리티 하네스 | `tools/` | [10](10_porting.md) |

## 5. 공통 규약

| 항목 | 규약 | 근거 |
|---|---|---|
| 목적 방향 | 엔진 내부는 **항상 최소화**. 드라이버가 ModelSense=-1이면 목적·Cutoff·BestObjStop·BestBdStop 부호를 뒤집고 끝에서 되돌린다 | 노트 10 §1, 11 §3.17 |
| 무한대 | `f64::INFINITY`. 사용자 bound는 `|b| ≥ 1e30`(`GRB.INFINITY`=1e100 입력도 허용)이면 무한으로 본다 | 노트 11 §4.4 INFBOUND 1e30 |
| 행 저장 | `lo ≤ aᵀx ≤ hi` ranged row 하나로 통일. sense `<,=,>`와 RHS/Range는 API 레이어에서 변환 | 노트 12 §4.1 (`>`는 −1배로 `<` 저장) |
| 허용오차 기본값 | FeasibilityTol 1e-6, OptimalityTol 1e-6, IntFeasTol 1e-5, MIPGap 1e-4, BarConvTol 1e-8, PDHG* 1e-6 | 노트 05, 14 §4.1 |
| 상태 코드 | gurobipy와 같은 번호: 1 LOADED … 17 MEM_LIMIT, 18/19 FIRST_ORDER_OPTIMAL/INFEASIBLE | 노트 11 머리말 |
| 오류 코드 | 10001 OUT_OF_MEMORY, 10003 INVALID_ARGUMENT, 10016 NOT_FOR_MIP, 10020 Q_NOT_PSD, 10021 QCP_EQUALITY, 10024 NOT_SUPPORTED | 노트 11 머리말 |
| work 단위 | 각 커널이 "연산 수 추정 × 계수"를 `Work` 누산기에 더한다. `WorkLimit`, 결정적 concurrent 정지, 휴리스틱 예산, presolve 예산이 모두 이 값을 쓴다 | 노트 13 §3.3, 12 §3.4, 16 §3.6.2 |
| 로그 | Gurobi 로그와 같은 문자열 형식을 쓴다(노트 04의 앵커 문자열). 사용자·테스트가 로그를 파싱할 수 있게 하기 위함 | 노트 04 |
| 스레드 | 원본의 스레드 생성·동기화를 옮긴다(`std::thread`). 작업 훔치기 풀은 쓰지 않는다. 라운드 경계에서만 공유 상태를 합친다 | 노트 15 §3.12, 10 §5 |
| 난수·정렬·해시 | 원본 함수를 옮겨 쓴다(난수 `a7b4d6`, 정렬 `aabbe3` 등). 외부 crate의 RNG·`sort_unstable`·`HashMap` 순회 순서에 기대지 않는다 | 10 §4 |
| 부동소수 | 연산 순서 보존, FMA 금지, 빌드 대상 `x86-64`. 벡터화 루프는 원본 레인 구조대로 | 10 §4 |

## 6. 핵심 결정 (대안과 이유)

1. **밀집이 아니라 희소 revised simplex + LU부터 만든다.** 밀집 tableau는 Netlib 중간 크기(`d2q06c`, `dfl001`)에서 메모리로 막힌다. 첫 LP 마일스톤부터 CSC·LU·hypersparse FTRAN을 넣는다.
2. **dual simplex가 기본 LP 엔진이고 MIP 노드도 dual hot start다.** primal은 crossover cleanup·전환용으로 2순위. (노트 13 §0.2, §3.12)
3. **barrier는 정규방정식 AAᵀ + 원본 supernodal LDLᵀ 포팅**. 밀집 블록 커널은 원본처럼 MKL(`dgemm`, `dtrsm`, `dgetrf`)을 부른다. 자체 커널이나 `faer`로 바꾸면 P3를 맞출 수 없다. (노트 03 §4, 14 §1.1.2)
4. **B&B는 "best-bound 전역 힙 + worker별 plunge 목록" 2단 구조**, 분기는 reliability branching(pseudocost + strong branching, product score). (노트 15 §3.5, §3.7)
5. **cut은 버퍼 → pool → LP 3단**, 선택은 efficacy·평행도·밀도. separator는 고정 순서, auto는 성공률로 끈다. (노트 16 §3.2–3.3)
6. **일반 제약은 presolve에서 MIP로 번역**(MAX/MIN/ABS/AND/OR → 슬랙+이진+SOS1, INDICATOR → 슬랙+SOS1, PWL → λ+SOS2, NORM2 → 2차 행). NL은 DAG → 보조변수 분해 → envelope cut + 공간 분기. (노트 17)
7. **결정적 병렬**: 트리 탐색은 work 예산 라운드, concurrent LP는 승자 work를 정지점으로. 비결정적 모드(Method=3)는 선택 사항.
8. **Python 레이어의 핫 클래스는 PyO3**로 쓴다. gurobipy의 `_core`(Var, Constr, LinExpr, QuadExpr, TempConstr, tupledict)는 Cython이라, 순수 Python으로는 모델 구성 속도를 맞출 수 없다. 나머지(Model 메서드 묶음, 상수, helpers)는 Python이다. 일괄 함수(`add_vars_bulk`, `add_rows_csr`)는 그대로 둔다. (노트 01, 02)
9. **노트가 비워 둔 동작은 포팅으로 채운다.** 이전 설계가 문헌 표준으로 대신한 곳(노드 선택 규칙 1/4/6/7/11, pseudocost 갱신 지점, cut aging, PDHG 전처리, local branching 등)은 각 문서의 "포팅으로 확정할 지점"에 함수와 함께 적었다.
10. **기본값에서 도는 것은 늦추지 않는다.** 기본 파라미터에서 실행되는 reduction·separator·휴리스틱·ordering은 그 엔진의 첫 마일스톤에 들어간다. 하나라도 빠지면 P1(반복 수·노드 수)이 맞지 않는다.

## 7. 크기 추정과 단계

포팅 대상은 optimize 경로에서 `excluded`를 뺀 함수 3,228개, 의사코드 약 120만 줄이다([10 §2](10_porting.md)). Ghidra 의사코드가 원 소스보다 3–5배 길다고 가정하면 Rust 코드는 **약 24만–40만 줄**이다. 이전 재작성 설계의 추정(5만 9천 줄)보다 4–7배 크다. 이것이 "같은 성능"의 값이다.

| 단계 | 내용 | 주 모듈 (port_map) | 의사코드 줄(모듈 합계를 나눈 대략값) |
|---|---|---|---:|
| M0 | 모델·파라미터 표·MPS 읽기·지문·드라이버 골격·Python API, `tools/parity.py` | 01_model_api, 02_driver 일부 | 약 10만 |
| M1 | LP: LU, dual/primal simplex, 스케일링, crash, LP presolve 전체, postsolve, LP 드라이버 | 04_simplex, 03_presolve(LP), common 일부 | 약 30만 |
| M2 | MIP: MIP presolve 전체, B&B, 기본값에서 도는 separator·휴리스틱 전부, 전파, 재시작 | 03_presolve(MIP), 06_bnb, 07_cuts_heur | 약 45만 |
| M3 | barrier(ordering·밀집 열 포함) + crossover, PDHG, concurrent(공유형), Method 자동 선택, network, sifting | 05_barrier_pdhg_concurrent, 04_simplex 나머지 | 약 12만 |
| M4 | 일반 제약 번역, SOS, 2차, 비볼록 bilinear + 공간 분기 | 08_genconstr_nl 일부 | 약 8만 |
| M5 | NL DAG·FBBT·envelope·OBBT·NL barrier, 다목적·시나리오·해 풀·node file | 08_genconstr_nl 나머지, 02_driver 나머지 | 약 10만 |
| M6 | 결정적 병렬 B&B, concurrent MIP, 병렬 SB·probing, 기본값 밖 기능 | 06_bnb·07 병렬 경로, common 나머지 | 약 5만 |

상세는 [09_roadmap.md](09_roadmap.md).
