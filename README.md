# rsolver

`re/decomp`(디컴파일 결과)를 포팅 원본으로, `re/notes`를 지도로 삼아 원본과 같은 성능을 목표로 만드는 LP/MIP/NL 솔버. Rust 코어 + PyO3 + Python 레이어.
출처 정책과 법적 위치(비공개·내부 연구용): [docs/00_overview.md §2](docs/00_overview.md).

## 설계 문서

| 문서 | 내용 |
|---|---|
| [00_overview](docs/00_overview.md) | 목표, 출처 정책, 계층 구조, 노트↔모듈 대응, 공통 규약, 핵심 결정 |
| [01_model_and_api](docs/01_model_and_api.md) | Model 자료구조, 파라미터·속성, PyO3 경계, Python API 범위 |
| [02_driver](docs/02_driver.md) | optimize() 드라이버: 분류, Method 확정, 자동 조정, 상태 확정, 다목적·시나리오, 요약 로그 |
| [03_presolve](docs/03_presolve.md) | LP/MIP presolve reduction 목록·단계, work 예산, postsolve 스택, uncrush |
| [04_simplex](docs/04_simplex.md) | LU, FTRAN/BTRAN, dual/primal simplex, pricing, ratio test, 수치 안전장치, MIP 계약 |
| [05_barrier_pdhg_concurrent](docs/05_barrier_pdhg_concurrent.md) | barrier, crossover, PDHG, concurrent LP |
| [06_branch_and_bound](docs/06_branch_and_bound.md) | 트리, 노드 선택, 노드 처리, reliability branching, 종료, 재시작, 병렬 |
| [07_cuts_heuristics](docs/07_cuts_heuristics.md) | 루트 cut 루프, separator 디스패치·선택, 휴리스틱 스케줄러와 개별 휴리스틱 |
| [08_genconstr_nonlinear](docs/08_genconstr_nonlinear.md) | 일반 제약 번역, SOS 인코딩, 2차·비볼록, NL DAG/FBBT/envelope/OBBT, 전파, NL barrier |
| [09_roadmap](docs/09_roadmap.md) | 마일스톤 M0–M6, 완료 조건, 검증 전략, 저장소 구성, 위험 |
| [10_porting](docs/10_porting.md) | 성능 패리티 P0–P3, 포팅 절차, 수치 충실도 규칙, 차등 검사 하네스, 제외 범위 |
| [port_map.tsv](docs/port_map.tsv) | 코어 함수 4,362개의 모듈 배정과 포팅 여부 (`tools/port_map.py`로 생성) |

## 현재 코드 (M0 이전 프로토타입)

```
crates/core   순수 Rust: Problem/solve, lp.rs(2단계 primal simplex, 밀집), mip.rs(B&B)   cargo test -p rsolver-core
crates/py     PyO3 확장 rsolver._core: solve_problem()  (solve 동안 GIL 해제)
python/       rsolver: Model / Var / LinExpr / GRB
tests/        pytest
```

빌드·테스트: `. ~/.cargo/env && ../.venv/bin/maturin develop && ../.venv/bin/python -m pytest`
