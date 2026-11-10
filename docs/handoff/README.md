# 세션 시작과 컴퓨터 이동

새 세션은 채팅 기록 대신 이 저장소의 지침·작업 번호·인수인계 파일로 상태를 복구한다. 세 세션을 동시에 열면 M1·M2·M3를 각각 한 세션이 맡는다. 구현 범위와 선행 조건은 [스프린트 목록](../14_sprint_plan.md), 설계는 해당 `docs/m*/sprint_design.md`를 따른다.

## 1. 시작

1. `python3 tools/session.py status`로 변경·worktree·현재 컴퓨터의 점유를 확인한다.
2. `docs/11_progress.md` 맨 위와 `docs/12_roadmap_diagram.md`를 읽는다.
3. [M1](M1.md)·[M2](M2.md)·[M3](M3.md) 중 사용자가 지정한 영역을 선택한다. 지정이 없으면 미점유 영역 중 M2, M1, M3 순으로 다음 후보를 확인한다. 사용자가 특정 작업을 지시하면 그 지시를 우선한다.
4. 인수인계에 적힌 branch·commit·미커밋 선행 변경이 실제로 있는지 확인한다. 없으면 구현을 시작하지 않고 해당 스냅숏 또는 branch를 가져온다.
5. `python3 tools/session.py claim M2-03a --owner 컴퓨터명-세션명`처럼 하위 작업을 점유한다. 같은 영역의 다른 작업도 동시에 점유할 수 없다. 점유 실패 시 기존 담당 상태를 확인하고 다른 미점유 영역을 선택한다.
6. 해당 영역의 인수인계와 스프린트 목록에서 선행 조건·설계를 확인한다. [검증 기준](../15_sprint_verification.md)을 읽고 작업 기록에 [검증 계약](../templates/sprint_acceptance.md)을 작성한다. 입력 상태·비교 경계·오라클이 미정이면 조사 작업부터 선택한다. 1주 범위를 넘으면 하위 번호로 더 나눈다.
7. [브랜치 운영 스킬](../../.agents/skills/verified-task-branches/SKILL.md)을 읽고 최신 로컬 main에서 `work/<하위번호>`와 독립 worktree를 만든다. 미커밋 변경이 선행 조건이면 먼저 아래 export로 보존하고 검증 가능한 묶음으로 정리한다. 스냅숏에서 임시 작업한 경우 전체 선행 변경을 검토하고 작업 커밋만 깨끗한 기준선에 재적용한다. 현재 작업 폴더에서 다른 branch로 전환하거나 기존 변경을 지우지 않는다.

점유는 git common-dir에 저장되므로 같은 저장소의 worktree끼리는 공유된다. 별도 clone이나 다른 컴퓨터와는 공유되지 않는다. 시간이 지났다는 이유로 점유를 해제하지 않는다. 담당 세션 종료를 확인하고 기록된 task·owner로 `release`한다.

## 2. 세 영역의 변경 경계

| 영역 | 주 작업 경로 | 인수인계 |
|---|---|---|
| M1 | LP simplex·LU·scaling·LP presolve | `docs/handoff/M1.md` |
| M2 | `crates/mip` | `docs/handoff/M2.md` |
| M3 | LP barrier·PDHG와 후속 crossover·concurrent | `docs/handoff/M3.md` |

`crates/driver`, `crates/py`, `python/rsolver`, 공통 파라미터·모델, 시작 지침과 전체 진행 문서는 공유 경계다. 수정 전 `claim shared --owner ...`를 추가로 얻고 다른 영역의 변경과 충돌 여부를 확인한다. 한 branch를 여러 세션이 함께 수정하지 않는다. 테스트·성능 측정의 공유 자원 제한도 적용한다. Gurobi는 한 프로세스, 공유 가상환경 설치는 main worktree에서만 수행한다. 변경한 `tools/build_local.sh`의 동작을 확인하지 않은 채 성공했다고 보고하지 않는다.

## 3. 세션 종료와 인수인계

영역별 인수인계 파일에 현재 task·상태·branch·commit·미커밋 선행 변경, 실제로 통과한 명령과 결과, 미확인 범위, 다음에 실행할 구체적인 명령을 갱신한다. 현재 컴퓨터의 절대경로 대신 저장소 상대경로와 commit을 기준으로 기록한다. 검증하지 못한 작업을 완료로 표시하지 않는다.

작업 수행 완료와 동등성 통과·실패·미검증을 구분하고 원본 자료의 식별값·비교 규칙·재실행 명령을 연결한다. 저장 기준으로만 검사한 경우 원본 캡처까지 다시 실행한 것처럼 기록하지 않는다.

공유 경계 점유를 얻어 작업 branch의 `11_progress.md`·`12_roadmap_diagram.md`와 인수인계를 갱신한다. 작업 파일만 선택해 커밋하고 계약 범위 검증을 통과하면 스프린트 종료를 기다리지 않고 `--no-ff`로 병합한다. main의 상세 결과는 병합으로 반영한다. 미병합 작업은 병합 담당자가 main 인수인계에 branch·commit·위치·검증 상태만 기록하며 상세 파일을 미리 복사하지 않는다. 남은 변경이 미커밋이면 이동용 패키지를 다시 만든다. 병합 결과와 깨끗한 status를 확인한 뒤 작업 worktree·브랜치를 정리하고 `release 하위번호 --owner ...`와 `release shared --owner ...`로 얻었던 점유만 해제한다.

## 4. 다른 컴퓨터로 이동

현재 출처 정책은 원격 업로드를 금지한다. 사용자가 private 원격 동기화를 허용하기 전에는 아래 오프라인 방식을 사용한다. 이 방식은 임의의 다른 컴퓨터가 자동으로 최신 상태를 받게 하지는 않는다. 사용자가 비공개 경로로 패키지를 전달해야 한다.

같은 영역은 한 컴퓨터에서만 작업한다. 이동 전 원본 세션을 멈추고 인수인계를 갱신한 뒤 패키지를 만든다. 다른 컴퓨터에서 이어받은 뒤 원본 세션은 재개하지 않는다. 여러 컴퓨터에서 동시에 작업할 때는 M1·M2·M3를 사전에 각각 배정하고 공유 파일 수정·Gurobi 실행은 한 컴퓨터에만 맡긴다. 오프라인 패키지는 분산 잠금을 제공하지 않는다.

원본 컴퓨터에서 저장소 바깥의 새 디렉터리를 지정한다.

```bash
python3 tools/session.py export ../rsolver-transfer
```

외부 자료도 함께 담으려면 `--include-reference`를 추가한다. `reference.tar.gz`를 대상 컴퓨터의 부모 디렉터리에 풀면 `re/`가 만들어진다. 패키지는 새 컴퓨터에서 수정한 뒤 다시 export할 수 있다.

이 명령은 별도 index에서 현재 추적 파일과 ignore되지 않은 미추적 파일을 스냅숏 commit으로 만들고 전체 git 이력과 함께 `rsolver.bundle`에 담는다. 현재 branch·HEAD·index·작업 파일은 보존한다. `manifest.json`에 스냅숏 branch와 원본 commit·미커밋 목록을 적는다. export 중에는 다른 세션의 파일 수정을 멈춘다.

외부 원본 자료는 bundle에 포함되지 않는다. 아래 자료를 저장소와 같은 부모의 `re/`로 별도 전달한다.

- `re/decomp/`의 C·asm·index·callgraph.
- `re/notes/`, `re/scripts/`, `re/bench/`, `re/README.md`.
- ignore된 기준 덤프가 현재 작업에 필요하면 그 경로를 인수인계에 적고 추가 전달한다.

대상 컴퓨터의 새 부모 디렉터리에서 manifest의 `branch`를 확인한 다음 복원한다.

```bash
git clone --branch <manifest의 branch> /비공개전달경로/rsolver.bundle rsolver
cd rsolver
python3 tools/session.py doctor
python3 tools/session.py status
```

clone의 origin은 로컬 bundle 파일이다. 공유 원격으로 바뀌었다고 간주하지 않는다. 새 컴퓨터 owner로 점유하고 스냅숏의 선행 변경·검증 상태부터 확인한다. 검증된 main 기준선이 준비됐으면 거기서 작업 branch를 만들고, 스냅숏에서 이어야 하면 임시 분기로 기록한 뒤 병합 전에 선행 변경을 분리한다. 스냅숏에서는 원본 컴퓨터의 미커밋 내용도 commit에 포함돼 있으므로 깨끗한 status가 정상이며, 그것만으로 검증 완료 기준선이 되지는 않는다. 원본 branch·미커밋 상태는 manifest에 남아 있다.

빌드 산출물·`.venv`·Gurobi 라이선스·환경변수·개인별 스킬은 전달하지 않는다. 대상은 기존 수치 검증 조건에 맞는 x86-64 Linux를 기준으로 Rust, Python과 PyO3 빌드 환경을 준비한다. `pyproject.toml`을 확인해 새 가상환경에서 확장을 빌드한다. 오라클을 돌릴 컴퓨터는 Gurobi 13.0.3·라이선스와 Frida 등 해당 도구 의존성을 별도로 준비한다. 경로·환경이 없으면 doctor 결과를 인수인계에 기록하고 원본 확인이 가능한 범위만 진행한다.
