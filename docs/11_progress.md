# 11. 진행 기록과 운영

M2-03c-2 재풀이 basis 오라클을 완료했다(main 병합). Gurobi 원본 읽기 전용 훅으로 자연 상태 12 재풀이 2건(nidda 비루트, gmu 루트)을 사례·모드별 독립 3회, 총 12회 캡처했다. 새 훅을 붙여도 03c-1 사건 투영이 저장본과 같음을 확인했다. 재풀이 입력 basis는 실패한 LP의 마지막 basis가 아니라 노드에 저장된 시작 basis다. nidda에서는 노드 8278의 `5d4427`→`113d92` export, `8e4682`의 `69e299` 복사, `69d237` 압축, `69d7ea` decode를 거쳐 첫 LP와 재풀이 `cda5b`에 같은 `I+0x98/0xa0`가 들어간다. 재풀이 method는 0이었다. 재풀이 뒤 원본의 VBasis export는 nidda(LP 6)에서 없고, gmu(LP 2)에서는 `5d4427`이 만든다. 재풀이에서는 `2f7e1`이 `ws+0xf0`를 12→1로 되돌린 뒤 `325b4`가 노드 basis를 다시 적재하고 분해를 새로 한다. dual norm 입력은 없다. 검사기 `--self-test` 종료코드 0, 변조 17종을 거부했다. LU·가중치 내용, Rust 동등성은 미검증이고 솔버 코드는 변경하지 않았다. [계약과 결과](m2/mip_retry_basis.md).

M3-04b primal push 분기 조사를 완료했다. `134a87`·`256781`의 double·x87 경로, 후보 정렬·방향·ratio·bound 이동·basis 교환·재분해와 signed remaining을 원본 C·ELF로 분할했다. `055f68` 합성 29사례를 원본 독립 3회, 총 87호출 실행해 사건 필드 9개와 수치 비트를 비교했고, CLI 변조 10개를 모두 거부했다. [계약·분기표·실행 결과](m3/primal_push_branches.md). 전체 push 자연 실행·Rust·x87·infeasible ratio·factor 동등성은 미검증이며 다음은 M3-04b-1 정상 push 한 단계 오라클이다. 솔버 코드는 변경하지 않았다.

M3-04a-1 정상 crossover 진입 오라클을 완료했다. 소형 LP 하나에서 PDHG·barrier mode 0과 시작 벡터 mode 1을 각각 원본 독립 3회 실행했고 경로별 상태·배열이 비트 단위로 일치했다. 검토자도 세 경로를 각 1회 재캡처해 일치시켰다. Crossover=0 대조군은 목표 0건×3회, 실제 캡처 오류 주입 7개와 추가 변조 검토·합성 self-test도 통과했다. 원본 분기에 따라 warm 설정을 Method=1에서 2로 바꿨으며 기존 실패 3개는 보존했다. [계약·원본 근거·실행 결과](m3/crossover_entry_oracle.md). Rust·시간·work·다른 모델 동등성은 미검증이다. 후속 primal push 조사는 맨 위 M3-04b 기록을 따른다.

M3-04a 정적 조사를 완료했다. PDHG·barrier·시작 벡터의 crossover 진입, 배열 순서·변환·할당 실패·환경 복원과 cleanup 내부 상태 배열 설치 경계를 원본 C·ELF로 확인했다. 정적 조사만으로 Rust 동등성을 판정하지 않는다. [조사 기록](m3/crossover_entry_contract.md). 후속 원본 오라클 결과는 위 M3-04a-1 기록을 따른다.

M2-03c-1은 **부분 완료**다(main 병합). 사용자 결정으로 계약을 바꿨다. Gurobi 원본에서 자연 상태 12 재풀이 2건(비루트·루트)의 입출력·파라미터 복구·basis 입력/header, 합성 5→12 재분류, 상태 3 호출자 `682037`을 각 3회 검증했다. 병합 전 원본 33회 재실행에서 비교 30건이 모두 일치했다. 재풀이 후 VBasis·입력 basis 출처·factor/norm은 M2-03c-2로 분리했다. 상태 10·자연 5→12(정상 산술에서 불가능, 미관측)는 미검증이다. Rust는 변경하지 않았고 동등성은 미검증이다. 상세는 [M2-03c-1](m2/mip_linear_child_trace.md) §10–13.

## 현재 작업 상태 (2026-11-10)

Herdr의 `gpt-6.1-terra`에게 M2-03c-1을 맡기고 별도로 검토했다. 첫 결과 `3e9fc00`은 목표 사건 0회로 미완료였다. 수정 라운드 중 작업자가 사용 한도로 중단돼 점유를 인계받아 커밋했다. 도달 판정·자연 스캔과 사용자 결정에 따른 계약 변경을 거쳐 부분 완료로 병합했다(맨 위 참조). 위임·검토 규칙은 [인수인계 절차](handoff/README.md) §5에 정리했다.

M2-03b-1 선형 무계 재분류·보류 소비 조사를 완료했다. 원본 `5ce67b`는 루트 캐시 2에서 LP 상태 5를 12로 바꾸고 즉시 수치 재풀이를 시도한다. 노드 상태 3은 `60145b`가 분기 대상으로 인정해 `627f3e`의 domain 기반 대체 분기로 보내며, 연속 변수만 남으면 상태 10인 새 자식을 만들 수 있다. 같은 노드 재등록은 상태 7·8·10에 한정된다. Gurobi 13.0.3 원본 독립 프로세스 3회에서 setter 54회·재등록 조기 반환 54회·분기 자격 48회, 총 156회가 일치했고 오류 주입 4개를 검출했다. 호출자 재분류·재풀이와 실제 자식 연결은 정적 조사이며 Rust 동등성은 미검증이다. 솔버 코드는 바꾸지 않았다. [계약·근거·검증·훅 경계](m2/mip_unbounded_reclassification.md). `work/m2-03b-1`의 독립 worktree에서 작성하고 검증 후 main에 병합한다. 다음 작업은 M2-03c-1 선형 호출자·대체 분기 오라클이다.

M2-05a-1 선행 변경 정리와 브랜치 운영 스킬 생성을 완료했다. 기존 dirty main의 코드·오라클, 측정 자료, 세션·검증 문서, barrier 백업을 네 커밋으로 나눠 `09dc65f`에 병합했다. 네 커밋의 tree는 원래 `7ea2301`과 동일하다. M2-03a의 `37d2e89`·`657d031`만 새 기준선에 재적용해 `ebb35d6`에 별도로 병합했고 재적용 전후 tree도 같다. 기존 솔버 코드는 수정하지 않았다. Rust workspace 113개·Clippy, Python 확장 재빌드 후 653개, 세션 도구 3개가 통과했다. 원본 type 4 1,000개×3회가 기존 fixture와 일치했고 type 10·11 실제 레코드 156개 및 두 모델의 축약해 복원을 재검증했다. M2-03a 저장 기준 13개×3회 재생도 통과했다. [정리 계약·명령·commit 대응](m2/baseline_reconciliation.md).

[verified-task-branches 스킬](../.agents/skills/verified-task-branches/SKILL.md)을 저장소 원본과 개인 스킬 경로에 생성했다. 최신 main에서 하위 작업별 독립 worktree를 만들고 계약 검증 후 `--no-ff`로 병합한다. 미병합 상세 결과를 main에 미리 복사하는 절차를 바꾸고 AGENTS·인수인계를 연결했다. 운영 변경은 `work/shared-branch-policy`에서 작성해 병합한다. 종료 상태는 main의 미커밋·미병합 작업 없이 정리하며 시작 상태와 원래 스냅숏 이력은 `safety/main-dirty-20261110`·`safety/m2-03a-snapshot-20261110` ref에 보존한다. 당시 다음 솔버 작업은 M2-03b-1이었다. 이번 조사 결과는 위 기록을 따른다. 자식 무계·보류 재처리·ray·presolve 내부 중단과 탐색 궤적 차이는 남는다. 아래 미커밋·미병합 설명은 각 작업 당시의 기록이다.

M2-03a 직접 무계 확정 조건 조사를 완료했다. `600d25`의 직접 노드 상태 11은 추가 제약 문맥에서 재풀이·domain predicate를 통과한 경우이며, 순수 선형 입력은 노드 상태 3으로 보류된다. 루트 무계는 호출자 `5ce67b`가 별도로 확정하고, 루트 캐시가 2이면 현재 LP 상태 5를 12로 재분류한다. Gurobi 13.0.3 바이너리 해시를 고정한 호출 없는 합성 상태 13개를 독립 프로세스 3회, 총 39개 확인했고 상태·카운터·보존 필드 변조 3개를 모두 검출했다. 직접 상태 11·재풀이·ray·실제 자식 연결은 실행 미검증이며 Rust는 변경하지 않았다. 다음은 M2-03b-1의 재분류·보류 후 재처리 조사다. [계약·조건표·재실행 결과](m2/mip_unbounded_entry.md). 기존 main 미커밋 변경을 `7ea2301` 스냅숏으로 보존한 독립 worktree `../rsolver-m2-03a`, branch `sprint/m2-03a`에서 작업했다. 작업 결과는 선택 커밋하고 main에는 미커밋으로 복사한다. branch는 미병합이며 main의 기존 변경을 보존했다.

스프린트 검증의 핵심 기준을 [동작 동등성 검증 기준](15_sprint_verification.md)으로 작성했다. 실제 Gurobi 13.0.3 바이너리 실행을 기준으로 같은 입력 상태의 판단·출력·상태 변경을 구간·연결·전체 층에서 비교한다. 원본 오라클 방식·재현성·필드별 비교·시간과 work·사례와 오류 검출·기준 자료·완료 판정을 명시하고 [계약 양식](templates/sprint_acceptance.md)을 추가했다. AGENTS·스프린트 목록·영역별 설계·인수인계 시작 절차에서 필수 참조한다. P0~P3 관문 수치는 유지하며 집계 일치가 내부 동등성의 증명이라는 표현을 수정했다. 문서 링크·참조·변경 공백을 확인했고 솔버 코드·오라클은 실행하지 않았다. 이번 문서와 기존 변경은 main worktree에 미커밋이며 별도 미병합 worktree는 없다.

새 세션과 세 영역 동시 작업을 위한 시작 절차를 `AGENTS.md`에서 [인수인계 안내](handoff/README.md)로 연결했다. 영역별 다음 작업·선행 변경·검증 상태를 M1·M2·M3 인수인계에 기록했다. `tools/session.py`는 같은 git common-dir의 영역 점유, 환경 확인과 미커밋 변경을 보존한 오프라인 bundle 생성을 지원한다. 별도 컴퓨터 간 자동 점유 동기화는 제공하지 않으며, 기존 원격 금지 정책을 유지해 비공개 패키지 전달과 영역 사전 배정을 사용한다. 임시 저장소에서 점유 충돌·다른 worktree의 중복 점유·소유자 해제·수정·삭제·미추적 파일 복원 및 index 보존 검사가 통과했고 현재 환경 doctor를 확인했다. 솔버 오라클은 재실행하지 않았다. 기존 변경과 설정은 main worktree에 미커밋이며 별도 미병합 worktree는 없다.

1주 스프린트를 기준으로 기존 24개 번호를 상위 작업으로 유지하고 [하위 작업 목록](14_sprint_plan.md)을 추가했다. 각 항목에 종류·범위·선행 조건·종료 확인을 적었으며, 원본 미해독 영역은 설계 조사 뒤 구현을 추가한다. M1·M2·M3의 상태 소유권·데이터 전달·오라클 경계를 설계 보완 문서로 정리했다. 문서 링크·번호·변경 공백을 확인했으며 솔버 검사나 원본 오라클은 재실행하지 않았다. 기존 코드 변경과 이번 문서는 main worktree에 미커밋이며 별도 미병합 worktree는 없다.

M1·M2·M3를 식별 가능한 작업 번호로 나누고 상태·완료 확인·상세 기록을 [작업 단위 목록](13_work_units.md)에 정리했다. 기존 G1~G6 관문은 유지한다. 문서 변경은 main worktree에 미커밋이며 별도 worktree는 없다.

자식 LP 무계 처리를 조사했다. 원본 `600d25`는 조건에 따라 무계 방향을 검사해 노드 상태 11 또는 3을 선택하며 `5c239f`는 상태 11을 MIP 무계로 전달한다. 직접 진입 조건과 방향 검사 전체, 자식 노드 재현·오라클은 아직 미확인이라 구현은 바꾸지 않았다. [조사 기록](m2/mip_recovery_bnb.md). main의 기존 변경과 이 문서는 미커밋이며 별도 worktree는 없다.

MIP 복원 후속: type 4 flag 1·2의 정수 보정을 `35d235` 및 어셈블리와 대조해 추가했으며 합성 Gurobi 오라클 1,000개와 일치한다. type 10·11과 빈 축약 모델도 연결했다. mas74·mas76은 축약 경로의 원본 해 검사를 통과했고 enlight_hard는 축약 후 복원해 최적 목적 37을 확인했다. neos-860300·graphdraw-domain은 rsolver가 30초 내 해를 찾지 못했으나 Gurobi가 각 축약 모델에서 찾은 해를 복원해 원본 제약·정수성·목적 3201·19848을 검증했다. 38개 재측정은 전부 축약 경로, 최적 1·시간 초과 37, 원본 해 검사 22개 통과다. 진단 API 추가 뒤 Rust workspace 테스트·Clippy와 Python 테스트 653개가 통과했다. [작업 기록](m2/mip_recovery_bnb.md). 변경은 main worktree에 미커밋이며 별도 미병합 worktree는 없다.

MIP type 4 flag 0 복원을 기존 LP 분할로 연결하고 `363164` 합성 오라클 4개와 Rust 회귀 검사를 통과했다. 실제 mik-250-20-75-4는 축약 경로의 원본 해 검사를 통과했고 timtab1은 축약 경로에서 시간 내 해가 없었다. Rust workspace 테스트·Clippy와 로컬 확장 사용 Python 테스트 652개가 통과했다. type 4 flag 1·2, type 10·11과 빈 축약 모델은 아직 우회한다. 상세 [작업 기록](m2/mip_recovery_bnb.md). 작업 중 별도 문서 재배치가 `f86edd8`로 main에 커밋됐다. MIP 변경은 그 위 worktree에 미커밋이며 별도 미병합 worktree는 없다.

문서 탐색을 위해 상세 기록을 `docs/m1/`, `docs/m2/`, `docs/m3/`로 묶고 [문서 안내](README.md)를 추가했다. 기존 작업은 main worktree에 미커밋이며 별도 미병합 worktree는 없다.

`AGENTS.md`에 원본 복원 기준과 함께 프로젝트 구조, 현재 상태·원본·오라클을 찾는 경로, 빌드·검증과 공유 환경 규칙을 정리했다. 프로젝트 구조와 원본 경로는 개조식으로 구분했다. 경로와 기존 문서를 확인했으며 솔버 동작은 변경하지 않았다. 지침과 이 기록은 main worktree에 미커밋이고 별도 미병합 worktree는 없다.

MIP 해 복원·B&B 동기화를 시작했다. Gurobi 13.0.3 `363164`의 type 9 식과 역순을 합성 원본 오라클로 확인하고 적용했다. 서로 다른 이동·배율과 고정 레코드를 섞은 회귀 검사 및 B&B Python 테스트 404개가 통과했다. 38개를 5초 재측정하여 축약 31·미지원 우회 6·빈 축약 우회 1, 원본 해 검증 21개 통과, rsolver 시간 초과 38을 기록했다. 기존 type 9 대상 여섯 모델 중 세 모델의 해를 검증했고 두 모델은 해가 없었으며 하나는 빈 축약 우회였다. type 4·10·11의 실제 오라클 검증과 구현, 자식 LP 무계와 presolve·전파 시간 제한은 남았다. 호출 대응표·레코드 빈도·측정·한계는 [작업 기록](m2/mip_recovery_bnb.md)에 있다. 변경은 main worktree에 미커밋이며 별도 미병합 worktree는 없다.

`m2-mip-presolve`에서 B&B, 루트 전파·휴리스틱, presolve 후 B&B 및 고정·치환 변수 복원, Python MIP `optimize()` 연결을 구현했다(`13ce2fe`, main 병합 `3e5c9d4`). 다른 복원 레코드와 빈 축약 모델은 원래 모델을 직접 푼다. 나머지 MIP 복원과 MIP 콜백은 남았다. 별도 검증: G6 기본 축약 모델·postsolve 레코드·PRELIMIT 저장 변화점 및 최종 모델 각 38/38, workspace clippy 경고 0·Rust 테스트 통과·Python 테스트 498개 통과. 38개에서 실행되지 않은 하위 풀이 등은 미구현이다.

사용자 요청에 따라 `main` worktree 하나만 남겼다. 제거한 `rsolver-barrier`의 미커밋 진행 문서와 미추적 파일은 `worktree-backups/rsolver-barrier/`에 복사했다. 이 백업과 현재 worktree의 상태 기록은 미커밋이다.

`m1-lp-driver`를 `87ad61e`로 main에 병합했다. LP·MIP 경로의 네 파일 충돌과 MIP 테스트 모델의 RNG 초기화를 해결했다. 병합 결과를 임시 복제본에서 검증하여 Rust workspace 테스트, Clippy 경고 0, Python 테스트 650개가 통과했다. `m2-root`와 `m2-mip-presolve`를 포함해 남은 로컬 브랜치는 모두 main에 병합된 상태다. G5의 etamacro 요약 목적값과 simplex 추가 반복의 수치 차이는 남는다(`docs/m1/m1_driver.md` §11).

2026-11-09 MIPLIB B&B 첫 기준 측정: Threads=1, 내부 TimeLimit=2초, 외부 감시 6초에서 rsolver 37개 시간 초과·1개(`cod105`) 외부 감시 중단, Gurobi 5개 최적·1개 불가능·32개 시간 초과였다. rsolver 해 20개는 원래 모델에서 검증했다. 별도 presolve 레코드 조사에서 12/38에 미지원 유형이 나왔으나 실제 원본 직접 풀이 경로 비율은 아직 계측하지 않았다. presolve 전체 시간 제한이 동일하게 적용되지 않는 한계와 모델별 수치는 [기준표](m2/mip_bnb_baseline.md)에 있다. 기준표·원시 결과·측정 도구와 이 문서 변경은 main worktree에서 미커밋이며 따라서 커밋된 main에는 아직 반영되지 않았다. 단일 worktree가 main을 사용하므로 별도 미병합 작업 브랜치는 없다. 기존 미추적 AGENTS.md와 백업도 남아 있다.

2026-11-09 후속: `cod105`의 2초 제한 초과는 presolve·휴리스틱을 끈 루트 simplex에서 재현됐고 반복 제한 1에서는 즉시 종료했다. simplex 반복에 마감 확인을 연결하고 presolve부터 같은 시계를 사용한 뒤 2초 재현이 약 2.04초에 반환됐다. 실제 풀이 경로 계측을 추가해 5초 재측정한 결과 rsolver 38개 시간 초과·외부 감시 중단 0, 축약 풀이 26·미지원 레코드로 원본 풀이 12, 원래 모델 해 검증 21개 통과였다. Gurobi는 최적 6·불가능 1·시간 초과 31이다. B&B·루트 Python 테스트 436개, Rust workspace 테스트와 workspace Clippy가 통과했다. 단일 presolve·전파·LP 연산 중의 즉시 중단은 아직 지원하지 않는다. 상세 [5초 기준표](m2/mip_bnb_5s.md). 코드·검증 결과·문서는 main worktree에 미커밋이며, 별도 미병합 브랜치는 없다.

## 1. 운영 규칙 (2026-11-05 사용자 지시)

이 절의 Herdr 탭·branch 표는 당시 운영 기록이다. 현재 새 세션의 작업 선택·점유·이동 절차는 `AGENTS.md`와 [인수인계 안내](handoff/README.md)를 따른다.

오케스트레이터(Herdr 워크스페이스 w4 1번 탭의 Claude 세션)가 M 단계 작업을 탭별 에이전트에게 나눠 맡긴다.

1. **병렬**: 다른 파일·크레이트를 건드리고 따로 검증할 수 있는 단위는 별도 git worktree와 탭에서 동시에 돌린다. 관문 판정은 마일스톤 순서대로 하되, 독립 단위는 앞당겨 진행한다. 브랜치를 main에 합치는 일은 오케스트레이터가 한다.
2. **토큰 한도**: 3시간마다 한도로 멈춘 에이전트를 찾아 이어서 작업을 요청한다.
3. **컨텍스트 70%**: 15분마다 확인한다. 70% 이상이면 순서대로 한다.
   1. `/goal clear`
   2. 하던 작은 단위를 마치고 인수인계 메모(§2 표)를 갱신해 커밋하게 한다.
   3. 같은 탭에서 `/clear`로 새 세션을 연다.
   4. 메모를 읽고 이어 가게 하고, §4의 goal을 다시 건다. 화면에 `/goal active`가 뜨는지 확인한다. `/clear` 직후 처음 보낸 `/goal`은 goal로 걸리지 않고 지시로만 전달된다(2회 확인). 그래서 이어 가라는 지시를 먼저 보내고, 그다음 `/goal`을 보낸다("Goal set"이 뜬다).
4. **완료 기록**: 마일스톤 안의 단계(A, B, C…)나 관문이 끝나면 §3에 날짜·결과·커밋을 적는다.

공통 규칙(모든 에이전트):
- Gurobi는 한 프로세스만 돌린다(`gurobi_guard.pth`). "already running"이면 기다렸다 다시 시도한다.
- 공유 `.venv`에 `maturin develop`이나 `pip install`은 main 트리만 한다. worktree는 cargo로 빌드한 `.so`를 자기 `python/rsolver`에 두고 `PYTHONPATH=$PWD/python`으로 쓴다.
- worktree는 `crates/lp`의 기존 파일을 고치지 않는다(`lib.rs` 모듈 등록은 허용). 예외(2026-11-06): M1 LP 드라이버 흐름(G5)은 이미 끝난 흐름의 폴더 `crates/lp/src/presolve/`, `barrier/`, `pdhg/`와 `crates/model`을 고쳐도 된다. simplex 흐름의 파일(simplex·lu·spx·start·bfrt·scaling 등)은 계속 금지다. 고칠 때마다 barrier·PDHG 64/64가 유지되는지 확인한다. 커밋은 자기 브랜치에만, 단계마다 한다.
- 작업 완료 시 현재 worktree와 main의 진행 기록과 로드맵에 작업·검증 결과와 미병합 상태를 반영한다.
- 답변은 한국어로 한다.

## 2. 작업 흐름

| 탭 | 폴더 / 브랜치 | 담당 | 인수인계 메모 | goal |
|---|---|---|---|---|
| M1 심플렉스 | `rsolver` / `main` | LP simplex(dual·primal), LU, crash, Quad, LP 드라이버, `crates/core` 삭제 | `docs/10_porting.md` §9 | G1 |
| ~~M1 LP presolve~~ (완료, main 병합) | `rsolver-presolve` / `m1-lp-presolve` | LP presolve, postsolve | `docs/m1/m1_presolve.md` | G2 |
| ↳ ~~postsolve~~ (완료, `m1-lp-presolve`에 병합. pane `w4:pP`는 유휴) | `rsolver-postsolve` / `m1-postsolve` | postsolve `363daa` | `docs/m1/m1_postsolve.md` | G2의 일부 |
| ↳ ~~derived·PreDepRow~~ (완료, 병합, 브랜치 삭제) | — / `m1-derived`, `m1-deprow` | `5b488d`, `47c445` | `docs/m1/m1_presolve.md` | G2의 일부 |
| ↳ symmetry (presolve 하위 에이전트, pane `w4:pV`) | `rsolver-symmetry` / `m1-symmetry` | 대칭 검출 `5577ba`→`551cc0` | `docs/m1/m1_presolve.md` | G2의 일부 |
| ~~M3 barrier~~ (완료, main 병합) | `rsolver-barrier` / `m3-barrier` | barrier 본체(crossover 제외) | `docs/m3/m3_barrier.md` | G3 |
| ~~M3 PDHG~~ (완료, main 병합) | `rsolver-barrier` / `m3-pdhg` | PDHG 본체(crossover 제외) | `docs/m3/m3_pdhg.md` | G4 |
| M1 LP 드라이버 (main 병합, 수치 한계 남음) | `rsolver` / `main` | 파라미터 표, 드라이버, Presolve 켠 연속 경로 | `docs/m1/m1_driver.md` | G5 |
| M2 MIP presolve (별도 검증 후 main 병합) | `rsolver-barrier` / `m2-mip-presolve` | MIP presolve `3cd406` P0. 검증 범위·한계는 §3 | `docs/m2/m2_presolve.md` | G6 |
| ↳ MIP presolve 하위 작업 (상위 브랜치에 병합 완료) | `m2-mip-presolve`에 통합 | probing·clique·바깥 reduction·마무리 | `docs/m2/m2_presolve.md` | G6의 일부 |

## 3. 완료 기록

| 날짜 | 단계 | 결과 | 커밋 |
|---|---|---|---|
| 2026-11-05 | M0 관문 | 지문 102/102 | `fc0dcd0` |
| 2026-11-05 | M1 A 스케일링 | 열·행·목적 스케일 64/64 비트 | `503542d` |
| 2026-11-05 | M1 C singleton LU·FTRAN | 반복 0 `xB` 60/60 비트 | `e0eb167` |
| 2026-11-05 | M1 C BTRAN·`d`·DSE 가중치 | 60/60 비트 | `cb66a2b` |
| 2026-11-05 | M1 B 시작 기저 | `vstat` 64/64 | `fff2399` |
| 2026-11-05 | M1 presolve 0–1b | Presolve=0 64/64, PRELIMIT 1–3 64/64 | `87c95d8` (`m1-lp-presolve`) |
| 2026-11-05 | M1 presolve 1c–1g | 열 패스, singleton 행, forcing 행, 중복 행, 바깥 패스. PRELIMIT 1–14 64/64 | `3bed233` (`m1-lp-presolve`) |
| 2026-11-05 | M1 LP 엔진 종단 연결 | dual·primal simplex가 스케일된 작업 공간에서 돈다. 읽은 모델에서 Python `optimize()` 동작 | `ac30617` |
| 2026-11-05 | M3 준비 | 호출 범위 확인(simplex·LU·presolve와 독립, 공유는 스케일링 `07d3ce`뿐), Method=2 기준값, barrier 오라클·모듈 뼈대 | `8405f7c` (`m3-barrier`) |
| 2026-11-05 | M1 dual 반복 경로 | BFRT·cost shifting, 수치 안전장치 사다리, 가중치 리셋, 재분해 뒤 flip. `crates/core` 삭제(만든 LP도 LP 엔진 사용), Markowitz count bucket | `c9fddee`, `33a2342` |
| 2026-11-05 | M1 postsolve 준비 | uncrush 오라클(`363daa` 훅), 저장·종료 규칙과 LP uncrush 경로를 03에 반영 | `e707fea`, `f0622fa` (`m1-lp-presolve`) |
| 2026-11-05 | M3 정규방정식·ordering | 결정성 64/64, small-pivot 규칙의 LDLᵀ, A·Θ·Aᵀ 조립, AAᵀ 패턴 64/64, AMD 순열·symbolic 53/64 | `66054a0` (`m3-barrier`) |
| 2026-11-05 | M1 presolve 2a–2c | 비용 선호점 고정, 함의 중복 행, dual bound 전파. PRELIMIT 1–14 64/64. PreDepRow는 Netlib에서 degen2·degen3만 바꿈 | `eb471f7` (`m1-lp-presolve`) |
| 2026-11-05 | M3 ordering·symbolic | AAᵀ NZ, 순열(AMD + bisection), supernode, Factor NZ/Ops 64/64 | `f1d0841`, `8bc322b` (`m3-barrier`) |
| 2026-11-05 | M3 IPM 프로토타입 | 초기점, predictor, Mehrotra/Gondzio corrector, line search, Gurobi pivot 규칙 LDLᵀ와 refinement | `06308ed` (`m3-barrier`) |
| 2026-11-05 | M1 dual 회귀 수정(israel) | 인공 bound 배율에서 flip 뒤 `x_B` 재계산 | `6013997` |
| 2026-11-05 | M3 IPM 프로토타입 64개 | 상태 64/64, 목적 ≤1e-6 64/64, **BarIterCount 56/64**(±5% 57/64). Barrier statistics 로그 64/64. 불일치 8개는 해법 반올림 누적 | `e905669`, `9b4c612` (`m3-barrier`) |
| 2026-11-05 | M1 dual CHUZR 원본 이식 | 순서 있는 후보 목록(표시·재구축 허용량), BTRAN 비용 점수, 인공 bound 단계(1e19). 반복 0 목록 63/64 | `899c192`, `e412440` |
| 2026-11-05 | M1 presolve 3a–3b | 평행 열·행, PRESPLITROWS, 비용으로 조이는 부등식 행. PRELIMIT 1–17 64/64(기본값 최종 모델은 3/64) | `aba84fe` (`m1-lp-presolve`) |
| 2026-11-05 | M3 Rust IPM | Method=2 경로, 상태·목적 64/64, Barrier statistics 64/64, `2d8ee5` pivot 규칙으로 **BarIterCount 59/64**(92%, G3 P1 기준 90% 넘음), PRIMINF 반복 0 64/64 | `a4ab223`, `ddc3040`, `df38e2a` (`m3-barrier`) |
| 2026-11-05 | M1 dual BFRT·DSE 갱신 | 원본 규칙의 BFRT(말뭉치 99.4%), 정확한 DSE 갱신, Gurobi 정규화 규약의 reduced cost. Netlib 64: 정답 64/64, 반복 수 ±5% 21개, 기하평균 1.180배 | `1979049`, `33b944b`, `93e51af` |
| 2026-11-05 | M1 presolve 4 집계 | 연결 리스트 행렬 위의 aggregation(`519a52`/`37f484` A–D 단계). 집계 뒤 점검 지점 62/62 | `3bbdba0` (`m1-lp-presolve`) |
| 2026-11-05 | M1 postsolve uncrush | `363daa` 레코드 1·2·3·5·7·8·14와 검사기, 57/58 | `94c4195` (`m1-postsolve`) |
| 2026-11-05 | M3 재시작·Cholesky | `2bf1ea` 재시작 이식(perold 125회, Gurobi 130회), BarIterCount ±5% 62/64. supernodal 분해 프로토타입이 실제 MKL로 L·d 비트 일치(7개 × 반복 0–2) | `bf116b4`, `6a0057f` (`m3-barrier`) |
| 2026-11-05 | M1 dual 재분해 일정 | `6268b` 재분해 기준(갱신 수·간격·갱신 파일 비용). Netlib 64: 정답 64/64, **반복 수 같음 7**, ±5% 34 | `23cad5d` |
| 2026-11-05 | M1 presolve 5 마무리 단계 | 바깥 패스 나머지, 다음 패스, 마무리(`50de61` 함의 bound). **기본값 presolved 모델 62/64**(`5b488d`·`47c445`는 아직 빈 함수). postsolve 병합, 집계 뒤 uncrush 비트 일치(PRELIMIT 1–17) | `6aa22b7`, `44ef311`, `d9a4874` (`m1-lp-presolve`) |
| 2026-11-05 | M3 Cholesky 해법 | `2ee6c5` 해법 프로토타입(8-lane 내적, right-looking forward, backward), dx·dy 비트 일치 7개 | `37e1c19` (`m3-barrier`) |
| 2026-11-05 | M1 dual 반복 원본화 | CHUZR(모드 0/1)·BFRT(코퍼스 99.4%)·DSE·d 갱신·재분해 시점 이식. Netlib 64: 64/64 정확, 반복 일치 7, ±5% 34, sc50a 반복 0–6 비트 일치 | `899c192`, `1979049`, `33b944b`, `93e51af`, `23cad5d` |
| 2026-11-05 | M1 presolve PREDERIVED·postsolve | `5b488d`(PREDERIVED) 비트 일치 이식, type 24 레코드. 집계 레코드 6/11/21 uncrush, 기본값 postsolve 53/59. rsolver-lp clippy 경고 0 | `25a1494`, `b53fd32`, `15260ee`, `d537b7e` (`m1-lp-presolve`) |
| 2026-11-05 | M3 비트 일치 분해·해법 | Rust supernodal 분해·해법(`chol.rs`)과 MKL DGEMM/DTRSM 흉내(`mkl.rs`)가 Gurobi와 비트 일치. IPM을 이 위에 올려 **BarIterCount 63/64** | `5557d2b`, `506a26e` (`m3-barrier`) |
| 2026-11-05 | M1 presolve 기본값 64/64 | `47c445`(PreDepRow) 이식. 기본 파라미터 presolved 모델 64/64 | `38ea4e7` (`m1-lp-presolve`) |
| 2026-11-05 | M1 postsolve 64/64 | 행·열을 지우지 않은 경우의 항등 uncrush까지, 검사기 64/64. Gurobi 없이 도는 pytest 회귀 | `15b7b65`, `c024356` (`m1-postsolve` → `m1-lp-presolve`) |
| 2026-11-05 | **M3 barrier 본체 (G3) 완료** | 오케스트레이터가 따로 검증: 상태 64/64, 목적 ≤1e-6 64/64, **BarIterCount 64/64**, clippy 0, cargo test·pytest 통과, 검증 중 Gurobi 미사용. main에 병합(`crates/core` 삭제 유지, SIMPLEX·BARRIER `cbGet` 통합) | `c1a5124`, `3e8a744`, `03d2ca4`, 병합 `e6760c3` |
| 2026-11-05 | M1 dual 시작·섭동·난수 | 자유 열 루프·재분해 간격(m^0.7) 원본화, cost shifting, `common::Rng`(libgurobi 난수와 비트 일치), `48d5f` GLOBAL 섭동. Netlib 64: 정답 63/64, **반복 수 같음 11**, ±5% 39, 기하평균 1.014배 | `9c5614b`, `a8cf69e`, `04622c8` |
| 2026-11-05 | M1 presolve 작업량(생성자) | `5118e6` 문제 유형 검사 포함, PRELIMIT=1 작업량 64/64 | `bc9f229` (`m1-lp-presolve`) |
| 2026-11-05 | M3 PDHG 0–1 | 호출 범위·의존 확인, Method=6 기준값, 결정성 64/64, PDHG 상태 오라클 | `984af95`, `1e820a2` (`m3-pdhg`) |
| 2026-11-05 | M1 Netlib 91 Method 1 점검 | 86/91 정답(라이선스 밖 27개 중 23) | `9fba738` |
| 2026-11-05 | M1 presolve 작업량 예산 | `50de61`·`516cb4` 작업량 항, `3ba9de` 예산 정지 | `9130f4f` (`m1-lp-presolve`) |
| 2026-11-05 | M1 대칭 검출 게이트 | `5577ba`/`551cc0` LP 경로 이식, 합성 대칭 LP 회귀 | `fcb1bb2`, `6e30a98` (`m1-symmetry`) |
| 2026-11-05 | M3 PDHG (G4) 에이전트 보고 완료 | Rust PDHG(Method=6)가 Netlib 64에서 Gurobi와 비트 일치, 콜백별 상태 비교 도구. **오케스트레이터 검증·병합 대기** | `60b0b84`, `1479067` (`m3-pdhg`) |
| 2026-11-06 | **M3 PDHG (G4) 완료** | 오케스트레이터가 따로 검증(기준값 로그의 파라미터 그대로, WorkLimit 60): 상태 64/64, 목적 ≤1e-6 64/64, **PDHGIterCount 64/64**. clippy 0, cargo test·pytest 통과. main 병합 뒤 barrier 64/64도 다시 확인 | `60b0b84`, `1479067`, 병합 `1d0118c` |
| 2026-11-06 | **M1 LP presolve·postsolve (G2) 완료** | 오케스트레이터가 따로 검증: presolved 모델을 처음 뜬 Gurobi 기준값(`bench/gurobi/*/presolved.mps.gz`)과 비교해 **64/64**, postsolve 64/64, clippy 0, cargo test·pytest 31 통과. main 병합 뒤 barrier·PDHG 64/64 다시 확인 | `a8e01e7`, `6a92185`, 병합 `df70174` |
| 2026-11-06 | M2 presolve 1단계 | 호출 그래프와 Frida로 확인한 결과 `3cd406`은 LP를 풀지 않는다. 실행 순서대로 reduction 목록 정리, 파라미터 오프셋 표(`ac81d7` + getParamInfo), PRELIMIT 훑기 도구, MIP 상태 덤프 오라클 | `077cfa2`, `5627199`, `647088d` (`m2-mip-presolve`) |
| 2026-11-06 | M1 primal simplex 원본 이식 | `0478d3` 명세의 primal(행 표시, 복합 phase 1, 부분 목록 CHUZC, Harris·`54eef` ratio, 진입 flip), `b1da8` 퇴화 bound shift, `cda5b` primal↔dual 넘김. Method 0: Netlib 91 중 84, Method 1: 64 중 63 | `7affabb` |
| 2026-11-06 | M1 드라이버: 파라미터 표·헤더 로그 | Frida로 `9828a1`/`c23877`/`0c3d0d` 호출 범위 확인. 파라미터 표가 libgurobi130에서 **548/548 = gurobipy**(G5 (1)). 새 driver 크레이트의 Optimize 헤더 로그 **102/102**(G5 (2)) | `0d22211`, `320fba5` (`m1-lp-driver`) |
| 2026-11-06 | M2 MPS 쓰기(정수) | 정수 MARKER, BV/LI bound를 `a4590c`처럼 씀(MIPLIB 원본 38/38) | `b5a93c1` (`m2-skel`) |
| 2026-11-06 | M1 LU Markowitz 수정 | 허용 가능한 피벗 없이 4줄을 지나도 기저를 singular로 판정하지 않음(정상 기저에 slack 29–62개가 들어가던 문제). Method 1: Netlib 91 중 89 | `736c3b5` |
| 2026-11-06 | M1 드라이버: 연속 모델 `optimize()` | Method 확정, 자동 조정, presolve → simplex·barrier·PDHG → uncrush, 상태·요약 로그, Presolve≠0이면 Python `_driver` 분기, `tools/driver_check.py`. Method 2·6 + Presolve 켬: 상태·목적 64/64, presolved 모델 8/64, BarIterCount 20/64. presolve의 Method 분기가 남음 | `9f5a723` (`m1-lp-driver`) |
| 2026-11-06 | M2 presolve 골격·패스 해독 | `crates/mip` 상태(연결 리스트), `3cd406` 앞부분·마무리·모델 생성, 패스 루프·라운드 끝·마지막 단계 해독(하위 에이전트 skel, spine-b/c/d 병합) | `54f3923`, `e69eb66`, `4d5b7b0`, `1b1e9b3` (`m2-mip-presolve`) |
| 2026-11-06 | M1 unscaled 재검사·CHUZR 비용·BTRAN | `2e9ae` 원 공간 재검사와 재풀이(라이선스 밖 Netlib 최적값 정확), 행 비용 가중치 갱신, Markowitz 분해의 희소 BTRAN. blend는 Gurobi 피벗 경로 전체와 같음. Netlib 64: **반복 수 같음 15**, 경로 전체 일치 6 | `61676e0`, `bf35530`, `4474ff8` |
| 2026-11-06 | M1 presolve Method 분기 | Method·Crossover·SolutionTarget 분기(`state+0x100`). Method 2·6, Crossover 0에서 presolved 모델 Gurobi와 **64/64** | `ce24d8e` (`m1-lp-driver`) |
| 2026-11-06 | M2 presolve 제어 흐름·해독 | `3cd406` 라운드·패스·내부 루프·라운드 끝·마지막 단계 제어 흐름(reduction은 빈 함수), probing·clique·내부 루프 해독 노트(하위 에이전트 inner, probe, clique 병합) | `f374378`, `0de8d15` (`m2-mip-presolve`) |
| 2026-11-06 | M1 dual 원본 규칙 추가 | 이탈 변수 bound 방향(`28c93`), 분해 뒤 CHUZR 목록 재구축, `48d5f` 섭동 발동 규칙, pricing 카운터. Netlib 64: 정답 64/64, **반복 수 같음 18–19**, ±5% 40, 기하평균 1.000 | `f8e9c59`, `549e828`, `a19d442`, `835441c` |
| 2026-11-06 | M1 드라이버: PDHG 시드·G5 보고 | presolve 모델 난수(`3b0917`/`895c5`)로 PDHG 시드, Presolve 켠 PDHG 64/64. 에이전트가 **G5 기준 충족**을 보고함(검증 대기) | `c8f7170`, `1ea1977` (`m1-lp-driver`) |
| 2026-11-06 | M2 clique 표 | `451bed` clique 표 이식 | `cf52001` (`m2-mip-presolve`) |
| 2026-11-06 | M2 presolve 내부 루프 병합 | 내부 루프 reduction 병합. 기본값 presolved 모델 **9/38**. XOR 행(`3e5363`) 해독(하위 에이전트 outer-a) | `f070a16`, `f4320c9` (`m2-outer-a`) |
| 2026-11-06 | M1 LU 갱신(Forrest–Tomlin) | `5b257` 원형 FT 갱신(제자리 열 교체 또는 row eta, 8레인 합) | `670afe9` |
| 2026-11-06 | M2 probing A·내부 루프 도구 | probe 행렬·엔진 `4e48f7`·충돌 그래프 이식, 레코드·덤프 검사 도구. dual bound(`374521`)·함의 그래프(`5431ee`) 해독 | `ce3522e`, `4b425d6`, `1010d5a` (`m2-mip-presolve`, `m2-outer-a`) |
| 2026-11-06 | M1 FT 갱신 세부 규칙 | `5b257` prefix·빈 열 캐시, U 행이 비면 row eta 생략, 순열 교환 | `4fa5ef6` |
| 2026-11-06 | M1 드라이버: 기저 없는 uncrush·상태 규칙 | `36578f`/`365159` 기저 없는 uncrush(Method 2·6, 오라클과 64/64 비트), 원본 상태 규칙, big-uncrush-violation 검사 | `fcd853e` (`m1-lp-driver`) |
| 2026-11-06 | M2 probing B·clique 그룹 | probing 결과 반영(대입·bound·고정·lifting)과 호출 13개 상태 일치, clique 덮개 강화(`45885d`)·set packing 쌍 그래프(`52be10`)·clique 행 정규화(`46a62d`) | `0789b47`, `a91287e`, `2ba8870`, `955a8be` (`m2-mip-presolve`) |
| 2026-11-06 | M1 Markowitz 커널·CHUZR 모드 1 | 비트 일치 프로토타입에서 `036ade` Markowitz 커널 이식, `1bae9c`/`2896a3` CHUZR 모드 1 임계값, 시작 단계 섭동 카운터(tuff가 Gurobi처럼 반복 27에서 섭동) | `f513cd2`, `5b21c30`, `6aa3b46` |
| 2026-11-06 | M1 드라이버: 엔진 로그 | barrier·PDHG·simplex 반복 로그 행, 증명 상태 로그, `tools/log_check.py`, 전체 로그 테스트 | `73491b9` (`m1-lp-driver`) |
| 2026-11-06 | M2 바깥 reduction A·B | 함의 그래프 추이 축소(`5431ee`), dual bound(`374521`), 부등식→등식(`3e40d1`), 노드 아레나 압축(`36ee92`), dual 집계(`418542`). 덤프 호출 78/78 | `f280d7c`, `0ea348a`, `8624df7`, `d1d083d`, `2ebdf67` (`m2-outer-a`, `m2-outer-b`) |
| 2026-11-06 | M1 simplex 속도 개선 | 비트는 그대로(피벗 경로 64/64 동일). BTRAN 출력 재사용, α 지지 집합만 순회, LU 평면 배열, FT 갱신에서 U 위치 역순열 유지 | `61ccd6b`, `2cb6739` |
| 2026-11-06 | M2 round end·바깥 reduction 추가 | 지배된 singleton 열(`5a9282`), 비용 singleton mod gcd(`5ab429`), VUB lifting(`59c193`), 계수 패턴 사전(`55a779` 1부), XOR 행(`3e5363`), 함의 정수(`57627e`), 평행 행·열 MIP 분기(`3ff744`), 비영 소거(`40d2bc`), MIP 집계(`37f484`) | `dff8109`, `91c379b`, `2b791f0`, `79ae5aa`, `f001cff`, `b14499b` (`m2-*`) |
| 2026-11-06 | M2 MIP presolve (G6) 에이전트 검증 완료 | 에이전트가 `m2-outer-b` 병합 후 재검증했다고 보고함. 기본값 presolved 모델, PRELIMIT sweep 변화점, postsolve 레코드 각각 **38/38** 일치. probing·대칭·연결 성분 탐색 포함, reduction stub 삭제. clippy 경고 0, cargo test 통과, pytest 37 passed·1 xfailed. 당시 별도 검증·main 병합 대기 | `a683710`, 병합 `83f4fc3`, 최종 `2f28f77` (`m2-mip-presolve`) |

| 2026-11-09 | M2 풀이 경로 진행 | `m2-mip-presolve`에서 B&B와 루트 전파·휴리스틱을 구현했고, 고정·치환 레코드를 역순으로 복원하는 presolve→B&B 경로와 Python MIP `optimize()`를 연결했다. 미지원 레코드, 빈 축약 모델, 복원 검증 실패는 원래 모델을 직접 푼다. 원래 제약과 목적값으로 복원 해를 검증한다. Rust MIP 60개, Python B&B·모델 405개 통과(작은 이진 MIP 200개를 완전탐색과 비교). LP presolve→simplex→postsolve, 다른 MIP 복원 레코드, MIP 콜백은 미완료. 이후 `3e5c9d4`로 main에 병합했다. | `13ce2fe`, 병합 `3e5c9d4` |

### G6 검증 범위와 남은 작업 (2026-11-06)

- 위 결과는 `m2-mip-presolve`의 `2f28f77`에서 에이전트가 보고한 값이다. 2026-11-09 별도 검증 후 `3e5c9d4`로 main에 병합했다. 검증 절차와 상세 결과는 해당 브랜치의 `docs/m2/m2_presolve.md` §0에 있다. `git show 2f28f77:docs/m2_presolve.md`로 확인한다.
- 검증 범위는 miplib_small 38개다. 그 범위에서 실행되지 않은 covers 적용, type 24 레코드 바이트, Q·QC·SOS·NL, 재시작 문맥은 오라클로 확인하지 못했다.
- 하위 풀이 `5307f4`·`522aa2`와 doubleton 등식 처리 `56b360`은 건너뛴다. 여러 연결 성분이 있어 하위 풀이가 필요한 모델에서는 Gurobi와 결과가 달라질 수 있다.
- `cliquerows.rs`·`setpack.rs`의 노드 풀 확장 `3b85ea`는 아직 `assert!`로 막혀 있어 더 큰 모델에서 중단될 수 있다. 이 한계는 병합 후에도 남는다.

| 2026-11-09 | G6 별도 검증·main 병합 | 축약 모델·레코드·PRELIMIT 저장 변화점과 최종 모델 각 38/38, clippy 0, Rust 테스트 통과, Python 498개 통과. 검증은 병합 전 동일 코드에서 수행했다. | `13ce2fe`, 병합 `3e5c9d4` |

## 4. goal 원문

- **G1**: M1에서 LP presolve·postsolve를 뺀 나머지를 완료한다. 완료 기준(Presolve=0, Threads=1):
  1. Netlib 91개에서 Method 0/1/기본 모두 상태가 Gurobi와 같고, 목적 상대오차 ≤1e-6이다. 크기 제한 밖 27개는 공개 최적값과 비교한다.
  2. P1: Netlib 64개에서 IterCount가 Gurobi와 같은 인스턴스가 90% 이상이다. 나머지는 ±5%이고 첫 분기점을 기록한다.
  3. P2: work ±5%.
  4. P3: 시간 기하평균 ≤1.2배. 기계가 한가할 때 잰다.
  5. dual·primal simplex, Markowitz·FT LU, crash, Quad, 필요한 common을 포팅한다. cargo clippy 경고 0, 테스트 전부 통과.

  범위 밖은 LP 드라이버·파라미터 표·Presolve 켠 경로(G5), presolve·postsolve·barrier·PDHG(완료)다. M1 관문 속도 스냅숏은 G1·G5가 끝난 뒤 오케스트레이터가 한다.
- **G2**: LP presolve(`390989`)와 postsolve를 완료한다. 완료 기준:
  1. P0: Netlib 64개에서 기본 파라미터의 presolved 모델(`presolved.mps.gz` SHA-256)이 Gurobi와 64/64 같다.
  2. presolved 모델의 해·기저를 postsolve로 원래 공간에 되돌린 결과가 Gurobi와 비트 단위로 같다. 64/64, 오라클로 검증한다.
  3. 미뤄 둔 `47c445`(PreDepRow)와 대칭 검출 게이트(`5577ba`→`551cc0`)를 포함해 기본값에서 도는 reduction을 모두 옮긴다.
  4. cargo clippy 경고 0, 테스트 통과. `docs/03_presolve.md`와 `docs/m1/m1_presolve.md`를 갱신한다.

  범위 밖은 simplex·드라이버(main)와 barrier다.
- **G3**: barrier(Method=2) 본체를 완료한다(crossover 제외). 완료 기준(Presolve=0, Threads=1, Crossover=0):
  1. Netlib 64개에서 상태가 Gurobi와 같고, 목적 상대오차 ≤1e-6이다.
  2. P1: BarIterCount가 Gurobi와 같은 인스턴스가 90% 이상이다. 나머지는 ±5%이고 첫 분기점을 기록한다.
  3. ordering 결과와 Factor NZ가 Gurobi 로그와 같다.
  4. 반복마다 BARRIER 콜백 오라클과 비교하는 도구를 둔다.
  5. cargo clippy 경고 0, 테스트 통과. `docs/05_barrier_pdhg_concurrent.md`와 `docs/m3/m3_barrier.md`를 갱신한다.

  범위 밖은 crossover, PDHG, concurrent, presolve, simplex다.
- **G4**: PDHG(Method=6) 본체를 완료한다(crossover 제외). 완료 기준(Presolve=0, Threads=1, Crossover=0):
  1. Netlib 64개에서 상태가 Gurobi와 같고, 목적값이 Gurobi PDHG 결과와 상대 1e-6 이내다.
  2. P1: Gurobi가 보고하는 PDHG 반복 수와 같은 인스턴스가 90% 이상이다. 나머지는 ±5%이고 첫 분기점을 기록한다.
  3. 반복별 오라클 비교 도구를 둔다.
  4. cargo clippy 경고 0, 테스트 통과. `docs/05_barrier_pdhg_concurrent.md`와 `docs/m3/m3_pdhg.md`를 갱신한다.

  범위 밖은 crossover, concurrent, barrier(완료), presolve, simplex다.
- **G5**: M1 LP 드라이버를 완료한다. 기준은 `docs/02_driver.md`이고, 대상은 `9828a1`→`c23877`의 연속 모델 경로와 LP 드라이버 `0c3d0d`다. 완료 기준:
  1. 파라미터 표(`ac81d7` 등록): 모든 파라미터의 이름·기본값·최소·최대가 gurobipy와 같다.
  2. Optimize 헤더 로그("Optimize a model with …", 지문, 계수 범위 통계)가 `bench/gurobi`의 Netlib 64개 + miplib_small 38개 `log.txt`와 같다.
  3. 모델 분류, Method 확정 표, 자동 조정, 상태 확정, 요약 로그를 원본대로 옮긴다.
  4. Presolve를 켠 연속 경로(presolve → simplex·barrier·PDHG → postsolve → 상태)가 돈다. Netlib 64개에서 Presolve=-1이고 Method=2·Crossover=0, Method=6·Crossover=0일 때 상태·목적(≤1e-6)·BarIterCount/PDHGIterCount가 Gurobi와 같다(기준값은 새로 뜬다). Method=1은 상태·목적만 본다.
  5. cargo clippy 경고 0, 테스트 통과. `docs/02_driver.md`와 `docs/m1/m1_driver.md`를 갱신한다.

  범위 밖은 simplex 내부(G1)와 MIP다.
- **G6**: M2 MIP presolve(`3cd406`)의 P0를 완료한다. 완료 기준:
  1. miplib_small 38개에서 기본 파라미터(Threads=1)의 presolved 모델이 `bench/gurobi/<이름>/presolved.mps.gz`와 38/38 같다.
  2. 기본값에서 도는 MIP reduction(probing·대칭성·연결 성분 포함, docs/03 기준)을 모두 원본대로 옮긴다.
  3. MIP presolve의 postsolve 레코드도 원본대로 쌓는다.
  4. cargo clippy 경고 0, 테스트 통과. `docs/03_presolve.md`와 `docs/m2/m2_presolve.md`를 갱신한다.

  첫 단계는 호출 그래프로 `3cd406` 경로가 LP 풀이·simplex·barrier를 부르는지 확인하는 것이다. 범위 밖은 B&B, cut, 휴리스틱, simplex다.
