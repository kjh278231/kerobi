# M2-05a-1 선행 변경 보존과 통합 검증

## 검증 계약

종류는 기존 변경 정리·검증이다. 새 솔버 동작을 구현하지 않는다. main `f86edd8` 위의 미커밋 선행 변경을 `7ea2301`과 바이트 단위로 동일하게 분리 커밋한 뒤, M2-03a의 두 작업 커밋을 새 기준선에 재적용한다. 스냅숏의 전체 선행 변경을 조사 작업의 일부로 병합하지 않는다.

시작 시점 main은 기존 코드·문서·측정·백업과 M2-03a 복사본이 미커밋이다. `sprint/m2-03a`의 HEAD는 `657d031`, 그 부모 조사 커밋은 `37d2e89`, 선행 스냅숏은 `7ea2301`이다. 시작 시 `tools/session.py export ../rsolver-baseline-start-20261110`으로 현재 상태를 추가 보존한다. 작업 점유는 `M2-05a-1`과 `shared`, owner는 `codex-baseline-20261110`이다.

### 비교 경계와 판정

| 대상 | 비교 규칙과 산출물 |
|---|---|
| 기존 선행 파일 전체 | 보존 커밋 4개 뒤 tree가 `7ea2301`과 동일해야 함. mode와 파일 내용 포함 |
| 솔버·Python·기존 테스트 | 정리 전후 파일 내용 동일. 신규 로직과 임계값 수정 없음 |
| M2-03a 재적용 | 재적용한 두 커밋 뒤 tree가 `657d031`과 동일해야 함 |
| 기존 복원 회귀 | Gurobi 13.0.3 합성 원본 호출, 저장 type 4 1,000개와 재생 결과 비교 |
| 통합 회귀 | Rust release workspace·Clippy, main 가상환경 재빌드 후 Python 전체 테스트 |
| 세션 도구 | `python3 -m unittest discover -s tools -p test_session.py` |
| M2-03a 저장 기준 | 원본 저장 결과 3×13개를 checker로 재생. 실제 자식·ray 실행 증거로 취급하지 않음 |
| 작업 종료 | main에 결과가 병합되고 status가 깨끗함. 미검증 solver 범위 유지, 점유 해제 |

복원 기준 함수는 Gurobi 13.0.3 `363164`와 `35d235`다. 원본·합성 사례의 기존 조사와 범위는 [MIP 복원 기록](mip_recovery_bnb.md)에 있다. 원본 바이너리 SHA-256은 `8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c`다. 기존 저장 fixture와 이번 재생을 구분한다. 시간 제한은 기존 회귀를 실행하며, 원본의 중단 위치·상태 동등성을 새로 검증했다고 주장하지 않는다.

스킬·운영 지침은 선행 변경 및 조사 병합 후 최신 main에서 별도 branch로 작성한다. main의 dirty 파일을 정리하기 전 원본 내용이 시작 스냅숏과 같은지 확인하고 stash 및 보존 ref를 남긴다. 원격 업로드는 하지 않는다.

## 실행 명령과 결과

실행 환경은 Linux x86-64, Rust 1.99.1, main의 Python 3.13 가상환경과 Gurobi 13.0.3이다. 솔버·테스트의 파일 내용은 정리 전후 동일하다. 검사 로그와 commit별 파일 목록은 `bench/validation/baseline_20261110/`에 보관한다. 저장 로그의 마지막 빈 줄만 정리했다.

| 실제 명령 | 결과 |
|---|---|
| `. ~/.cargo/env` 후 `cargo test --release --workspace` | 113개 통과, 실패·무시 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | 통과, 경고 0 |
| main에서 `../.venv/bin/maturin develop --release` | 확장 재빌드·설치 통과 |
| `../.venv/bin/python -m pytest -q` | 653개 통과, skip·xfail 0 |
| `python3 -m unittest discover -s tools -p test_session.py` | 3개 통과 |
| `../.venv/bin/python tools/mip_postsolve_oracle.py` | type 1·9 혼합, type 1·10·11 혼합, type 4 flag 0 네 사례 통과 |
| 복사한 오라클의 `--generate-parallel`, 독립 프로세스 3회 | 매회 1,000개, 기존 fixture SHA-256과 바이트 일치 |
| `../.venv/bin/python tools/mip_postsolve_oracle.py --actual-records` | 실제 type 10 레코드 136개·type 11 레코드 20개에 합성 값을 넣은 원본 비교 통과 |
| `../.venv/bin/python tools/mip_recovery_real_oracle.py` | neos-860300 목적 3201, graphdraw-domain 목적 19848 원본 해 검사·축약 목적 일치 |
| `python3 tools/mip_unbounded_entry_oracle.py --check bench/oracle/mip_unbounded_entry.json` | 저장 원본 상태 13개×3회 재생 통과. 원본 재캡처가 아님 |

원본 합성 1,000개 생성은 기존 fixture를 덮어쓰지 않도록 시작 패키지 옆 검증 디렉터리에 같은 오라클 파일을 복사하고 `bench/oracle/`을 만든 뒤 실행했다. 도구의 `Path(__file__)` 기준 출력 위치만 달라졌고 소스는 동일하다. fixture SHA-256은 `990be82822d9c224c9104d8b9b743b5458fdd333c5cea18206ba30836991667a`다. Rust 회귀는 저장 숫자와 정확 비교한다. 최초 실행 래퍼는 가상환경 Python 심볼릭 링크를 해석해 환경 밖 실행 파일을 호출하면서 import 단계에서 실패했다. `.venv/bin/python` 경로를 유지한 재실행이 통과했으며 최초 실패에서 원본 내부 함수를 호출하지 않았다.

실제 모델 오라클은 Threads=1·TimeLimit=30으로 각 축약 모델을 풀었다. neos-860300의 Gurobi 상태는 2, graphdraw-domain은 9였다. Gurobi가 찾은 모델별 해 하나의 복원 검사이며 rsolver의 해 발견 성능이나 모든 복원 레코드 경로의 동등성을 뜻하지 않는다. 예전 38개 시간 측정은 JSONL 중복·오류 필드를 검사했으며 재측정하지 않았다.

## 커밋과 병합

| 묶음 | commit |
|---|---|
| 기존 복원·마감 전파 코드와 관련 오라클·회귀 | `a8c3081` |
| 기존 벤치마크 도구·측정 자료 | `7a7ab24` |
| 세션 점유·스프린트 계약·운영 문서 | `bf53bed` |
| 이전 barrier worktree 인수인계 백업 | `42ab9d0` |

`git diff --exit-code 7ea2301 42ab9d0 --`가 통과했다. 네 커밋은 파일 mode까지 원래 선행 스냅숏과 같다. 이 기준선을 main에 `--no-ff`로 병합한 commit은 `09dc65f`다.

최신 main에서 `work/m2-03a`를 만들고 원래 `37d2e89`·`657d031`만 각각 `b80ece9`·`d535cac`로 재적용했다. `git diff --exit-code 657d031 d535cac --`가 통과했고 저장 원본 checker도 통과했다. main 병합은 `ebb35d6`이며 그 tree도 `657d031`과 같다. 이후 운영 스킬·지침·현재 상태 갱신만 별도 `work/shared-branch-policy`에서 추가했다.

main을 비우기 직전 별도 index로 모든 추적·미추적 파일의 tree를 계산해 시작 스냅숏 `a52cdd2`와 일치함을 확인했다. 그 뒤 원본을 stash와 `safety/main-dirty-20261110` ref로 보존했다. 원래 M2-03a 스냅숏 이력은 `safety/m2-03a-snapshot-20261110` 태그가 보존한다. 원본을 삭제하거나 스냅숏 commit을 조사 변경에 섞어서 병합하지 않았다.

## 스킬과 종료 판정

[verified-task-branches](../../.agents/skills/verified-task-branches/SKILL.md)를 저장소 원본으로 추가하고 개인 `~/.codex/skills/verified-task-branches`에 같은 내용을 설치했다. 형식 검사가 통과했으며 AGENTS·시작/종료 지침에서 필수 참조한다. 하위 작업별 최신 main 직접 분기, 검증 후 수시 `--no-ff` 병합, 미병합 상세 파일의 main 사전 복사 중단을 적용한다.

보존·검증·이력 정리는 완료다. 기존 코드의 원본 동등성 범위는 위 검사와 기존 기록에 한정한다. 운영 변경 병합 후 main의 status·선행 코드 보존·작업 commit 도달 여부를 확인하고 작업 worktree·브랜치 및 M2-05a-1·shared 점유를 정리한다. 다음 솔버 작업은 M2-03b-1이며 최신 main에서 시작한다.

## 남은 한계

자식 LP 무계·상태 3 재처리, ray 검사, presolve·전파 내부의 시간 제한과 MIP 탐색 궤적 차이는 기존 한계다. 이번 작업의 종료 판정은 변경 보존·검증·이력 정리이며 전체 M2 동등성 통과가 아니다.
