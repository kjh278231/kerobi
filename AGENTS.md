# 프로젝트와 시작점

- 목표: Gurobi 13.0.3의 동작과 성능 재현.
- 구성: Rust LP/MIP 솔버와 PyO3·Python API.
- 정책: private 저장소이며 대중에게 공개하지 않는다. 설계와 출처 정책은 `docs/00_overview.md` 참조.

작업을 시작할 때 `git status --short --branch`와 `git worktree list`로 기존 변경과 작업 위치를 확인하고 보존한다. 현재 상황과 남은 한계는 `docs/11_progress.md`의 맨 위와 `docs/12_roadmap_diagram.md`를 먼저 읽는다. README의 마일스톤 표시는 오래됐을 수 있다. 문서 구성은 `docs/README.md`를 참고한다. 상세 인수인계와 패리티 기준은 `docs/10_porting.md`, 해당 영역의 `docs/m1/`, `docs/m2/`, `docs/m3/`에서 찾는다.

## 새 세션·동시 세션·컴퓨터 이동

새 세션에서 이어서 작업하거나 컴퓨터를 옮겼으면 반드시 `docs/handoff/README.md`의 시작 절차를 따른다. `python3 tools/session.py status`와 해당 영역의 `docs/handoff/M1.md`·`M2.md`·`M3.md`에서 다음 작업, 선행 변경과 검증 상태를 복구한다. `docs/13_work_units.md`는 상위 범위, `docs/14_sprint_plan.md`는 1주 하위 작업이며 구현 전에 해당 `docs/m*/sprint_design.md`를 읽는다.

작업은 하위 번호로 점유하고 독립 branch·worktree에서 수행한다. 같은 영역은 한 세션이 맡고 공유 파일 변경은 추가 shared 점유를 얻는다. 점유는 같은 git common-dir 안에서만 유효하므로 다른 컴퓨터의 담당 영역은 이동 절차에 따라 인계한다. 원격 동기화 허용 여부와 오프라인 패키지 절차도 `docs/handoff/README.md`를 따른다.

브랜치 생성·검증·병합·미커밋 선행 변경 정리에는 [verified-task-branches 스킬](.agents/skills/verified-task-branches/SKILL.md)을 먼저 읽는다. 하위 작업별로 최신 main에서 분기하고 검증 후 `--no-ff`로 병합한다. 스프린트는 일정 단위이며 브랜치 수명을 결정하지 않는다.

## 코드와 원본 찾기

- `crates/model`: 모델·입출력.
- `crates/lp`: LP presolve·simplex·barrier·PDHG.
- `crates/mip`: MIP presolve·루트·B&B.
- `crates/driver`: 풀이 드라이버.
- `crates/py`, `python/rsolver`: Python 경계와 API.
- `../re/decomp/c/`: 원본 디컴파일 결과. 저장소 밖에 있다.
- `../re/decomp/asm/`, `../re/decomp/index.tsv`, `../re/decomp/callgraph.tsv`: 어셈블리, 함수 인덱스, 호출 그래프.
- `../re/notes/`: 원본 함수와 구조를 찾는 지도.
- `docs/port_map.tsv`, `docs/00_overview.md` §4: 원본 함수와 모듈의 대응. 영역별 설계는 `docs/01_*.md`–`docs/08_*.md` 참조.
- 기존 오라클과 비교기는 `tools/`, 기준 데이터는 `bench/`, 테스트는 `tests/`와 각 크레이트에 있다. LP 상태·경로는 `tools/spx_oracle.py`, `tools/path_oracle.py`, presolve·복원은 `tools/presolve_check.py`, `tools/mip_rec_check.py`, `tools/mip_postsolve_oracle.py` 등을 작업에 맞게 확인한다. 오라클의 적용 범위와 실행법은 관련 작업 기록에서 확인한다.

# 구현 기준

솔버의 동작을 구현하거나 수정할 때는 반드시 해당 원본의 디컴파일 결과를 먼저 확인하고, 그 제어 흐름·자료 구조·수치 및 상태 처리에 맞춰 복원한다. 관련 원본 위치와 확인한 근거를 기록하고 오라클 또는 재현 가능한 검증으로 대조한다. 디컴파일 결과가 없거나 해석이 불확실한 부분은 추정 구현을 원본과 일치한다고 보고하지 말고, 미확인 범위와 남은 한계를 명시한다.

연산 순서, 난수·정렬 순서, 상태와 허용오차가 풀이 궤적에 영향을 준다. SIMD나 호출 인자가 불명확하면 어셈블리도 확인한다. 상세 절차와 P0 구조 → P1 궤적 → P2 work → P3 시간의 비교 기준은 `docs/10_porting.md`를 따른다. 원본 함수와 근거는 작업 문서에 기록하며 코드에 출처 태그는 추가하지 않는다.

## 빌드와 검증

스프린트 착수·검증·완료 판정 전에는 반드시 `docs/15_sprint_verification.md`를 읽고 해당 작업 기록에 검증 계약을 작성한다. 기준 원본은 Gurobi 13.0.3 바이너리 실행이다. 구현 완료는 같은 입력 상태의 판단·출력·상태 변경을 계약 범위에서 원본과 비교해 통과한 경우에만 선언한다. 오라클이 없거나 핵심 비교가 미실행이면 조사·오라클 구축 작업으로 분리하고 구현 일치는 미검증으로 남긴다. 계약 양식은 `docs/templates/sprint_acceptance.md`를 사용한다.

- Rust 변경은 해당 크레이트의 `cargo test -p rsolver-lp` 등으로 먼저 확인하고, 범위에 맞춰 `cargo test --release --workspace`와 `cargo clippy --workspace --all-targets -- -D warnings`를 실행한다.
- Python 확장과 통합 검증은 main worktree에서 `. ~/.cargo/env && ../.venv/bin/maturin develop --release && ../.venv/bin/python -m pytest`를 사용한다. 공유 `.venv`를 바꾸므로 다른 worktree에서는 `tools/build_local.sh`와 해당 스크립트의 안내를 확인해 격리 빌드한다.
- Gurobi를 사용하는 오라클은 한 번에 한 프로세스만 실행한다. 사용 중 오류가 나면 다른 실행이 끝난 뒤 재시도한다. 입력·파라미터·기준선과 실제 검증 범위를 기록하고, 실행하지 못한 검사는 이유를 남긴다.

# 작업 완료 기록

작업을 마치기 전에 현재 worktree와 main의 `docs/11_progress.md`, `docs/12_roadmap_diagram.md`를 구현·검증 결과와 비교한다. 상세 결과와 변경 상태는 작업 branch에서 갱신해 병합으로 main에 반영한다. 미병합이면 shared 점유를 얻은 병합 담당자가 main 인수인계에 branch·commit·위치·검증 상태만 기록한다. 상세 파일을 main에 미리 복사하지 않는다. worktree의 미커밋·미병합 상태와 남은 한계를 명시하며, 갱신할 수 없으면 완료 보고에 이유와 누락된 문서를 적는다.

종료 전 해당 `docs/handoff/M*.md`에 하위 작업 번호, branch·commit·미커밋 선행 변경, 실제 검증 명령·결과와 다음 명령을 기록하고 얻었던 점유를 해제한다. 다른 컴퓨터로 이동할 때는 미커밋 변경을 포함한 최신 패키지를 생성한다.

답변은 한국어로 한다.
