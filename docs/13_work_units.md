# 13. M1·M2·M3 작업 단위

2026-11-13 기준 작업 목록이다. `M1-01`처럼 마일스톤과 두 자리 번호로 작업을 식별한다. 번호는 식별자이며 실행 순서는 아니다. 기존 G1~G6은 검증 관문으로 유지한다. 이 표의 확인 항목은 각 작업의 산출물을 판정하기 위한 것이며, 마일스톤의 전체 합격 기준을 대신하지 않는다. P0~P3와 마일스톤 관문은 [포팅 가이드](10_porting.md)와 [기존 로드맵](09_roadmap.md)을 따른다.

이 문서의 24개 번호는 상위 작업이다. 실행 단위는 [1주 스프린트 작업 목록](14_sprint_plan.md)의 `M2-05a` 같은 하위 번호다. 작업당 집중 작업 3일과 검증·수정 여유 2일을 계획 예산으로 두고, 착수 때 범위가 넘으면 더 나눈다. 조사 전의 구현 항목은 설계 대기다.

하위 작업의 검증 계약과 완료 판정은 [동작 동등성 검증 기준](15_sprint_verification.md)을 따른다. 기존 완료 이력은 그때 기록한 검증 범위를 유지하며 새 기준의 모든 검사를 소급해 통과했다고 간주하지 않는다.

상태는 **완료(범위 한정)**, **진행**, **예정**으로 표시한다. 완료는 표에 적은 범위의 검증 완료를 뜻한다. 미측정 경로나 다른 설정까지 완료했다는 뜻은 아니다. 진행 기록에는 하위 작업 번호, 변경, 검증 조건과 남은 한계를 적는다. 상세 설계 경계는 [M1](m1/sprint_design.md), [M2](m2/sprint_design.md), [M3](m3/sprint_design.md)에 있다.

## M1 LP 엔진

| 번호 | 작업 단위 | 상태와 완료 확인 | 근거 |
|---|---|---|---|
| M1-01 | 스케일링·시작 기저·LU | 진행. 반복 0의 주요 상태는 검증됐다. 남은 설정과 수치 경로는 원본 상태 오라클로 비교한다. | [LP 작업 노트](m1/m1_notes.md) |
| M1-02 | Dual simplex 반복 경로 | 진행. Method=1 현재 수치(M1-02w-3 뒤): Netlib 91개 lp_check 정답 90/91(pilot, 원본 비교 수단부터 M1-02y-2), 면허 크기 안 64개 IterCount 일치 35/64·pathcmp 완전 일치 28/64(중앙 첫 차이 0.896). M1 재분해는 2×2 블록 단계(M1-01e)와 조밀 전환·`local_70`·풀 부족 재호출(M1-02w-3, F+0x158 = `045a76` FactSpace 수명)을 원본 상태 수명으로 켠다. 원본 조밀 5모델은 주입 재생 124/124 일치. work 계수기(02q·02r·02s·02v)는 kb2·degen2 재생 범위에서 저장 값이 원본과 비트 일치하나 재분해 판정에는 미연결이다. 다음은 02w-4 → 02y-2 → 02t → 02u이고 나머지 미완 항목은 하위 작업 표에 있다. 첫 분기점과 G1 기준을 확인한다. | [M1-02w-3 조밀 켜기](m1/m1_02w3_dense.md), [M1-02w-2 풀 크기](m1/m1_02w2_factspace.md), [M1-01e 결과](m1/m1_01e_block_on.md#6-실행-결과), [work 계수기 02v](m1/m1_02v_iteration_work.md), [하위 작업](14_sprint_plan.md#2-m1-하위-작업), [M1 인수인계](handoff/M1.md), [G1 현황](10_porting.md) |
| M1-03 | Primal simplex 반복 경로 | 진행. Method=0의 Netlib 64에서 반복 수 일치와 ±5% 이내가 각각 0개다. 첫 분기점과 G1 기준을 확인한다. | [G1 현황](10_porting.md), [primal 명세](m1/m1_primal_spec.md) |
| M1-04 | LP presolve 기본 경로 | 완료(범위 한정). 기본 설정 presolved 모델 64/64가 일치한다. 미실행 분기는 별도 범위다. | [G2 기록](m1/m1_presolve.md) |
| M1-05 | LP postsolve 기본 경로 | 완료(범위 한정). 기본 검사기 64/64가 통과했다. 다른 설정과 레코드 조합은 이 결과에 포함하지 않는다. | [postsolve 기록](m1/m1_postsolve.md) |
| M1-06 | LP 드라이버·웜스타트 | 진행. main에 병합됐다. etamacro 목적값과 simplex 추가 반복의 차이를 원본과 대조한다. | [드라이버 기록 §11](m1/m1_driver.md#11-simplex-웜스타트-연결-2026-11-06-복구-세션) |
| M1-07 | LP 전체 관문과 속도 | 예정. Presolve 켬·끔의 Netlib 상태와 목적, Method별 P1, work 및 속도를 기존 M1 기준으로 평가한다. | [M1 완료 기준](09_roadmap.md#1-마일스톤) |

## M2 MIP 엔진

| 번호 | 작업 단위 | 상태와 완료 확인 | 근거 |
|---|---|---|---|
| M2-01 | MIP presolve 기본 경로 | 완료(범위 한정). 38개 모델의 축약 모델, 레코드, PRELIMIT sweep과 최종 모델을 검증했다. 실행되지 않은 하위 풀이는 남았다. | [G6 기록](m2/m2_presolve.md) |
| M2-02 | 루트 LP와 종료 상태 | 진행. 루트 전파·휴리스틱과 LP 연결이 있다. 원본의 루트 상태·경계·종료 규칙을 사례별로 대조한다. | [루트 기록](m2/m2_root.md) |
| M2-03 | 자식 LP 무계 상태 | 진행. 원본의 노드 상태 11과 MIP 상태 5 전파를 확인했다. M2-03a에서 직접 확정 조건을 조사했고, 선형 입력은 상태 3으로 보류됨을 합성 원본 호출로 확인했다. M2-03b-1에서 호출자 재분류·즉시 재풀이·대체 분기와 재등록 경계를 조사했고 원본 합성 구간 156회가 일치했다. M2-03c-1(부분 완료)에서 자연 상태 12 재풀이 2건과 상태 3 호출자 `682037`의 원본 동작을 3회 검증했다. M2-03c-2에서 재풀이 입력 basis 출처(노드 저장 basis), 재풀이 뒤 원본 export, 적재·분해 호출을 원본 3회로 확인했다. LU·가중치 내용, 상태 10 생성, ray 검사, Rust 동등성은 남는다. 현재 Rust는 Numeric으로 중단한다. | [진입 조건·원본 호출](m2/mip_unbounded_entry.md) |
| M2-04 | 시간 제한 전파 | 진행. simplex 반복과 presolve 시작 시계를 연결했다. presolve·전파 내부 및 simplex 한 반복의 중단 지점을 원본과 비교한다. | [5초 기준표](m2/mip_bnb_5s.md) |
| M2-05 | 해 복원 레코드와 빈 축약 모델 | 진행. type 1·2·4·9·10·11과 빈 축약 모델을 연결했다. 해가 없었던 모델의 복원 사례와 나머지 레코드를 원본 오라클로 검증한다. | [복원 기록](m2/mip_recovery_bnb.md) |
| M2-06 | 노드 선택·분기·pseudocost | 진행. 실제 B&B(`bnb.rs`)는 아직 최선 경계 선택과 최대 분수성 분기를 쓴다. 원본 규칙을 옮긴 Rust 모듈은 각 기록에 적힌 범위에서 원본 캡처 재생(L1)을 통과했고 모두 `bnb.rs`에 미연결이다: 전역 힙 `NodeHeap`·힙 키 `child_key`(06b·06b-2), `plunge`(06d·06d-3), `Pseudocost`(06f), `NodeLife`(06f-3), `strong_branch::run`(06h-2~06h-6, 비전파 경로 포함). 전파 모순·연속 열 후보는 원본 사례와 기준 자료가 있고(06h-7) 이식은 M2-06h-9·06h-10이다. 실제 경로 연결에는 노드 basis 재사용 설계 M2-10b 등이 먼저 필요하다. 나머지 미완 항목은 하위 작업 표에 있다. 연결 뒤 원본 노드 선택·reliability branching·갱신을 실제 경로에서 대조한다. | [힙](m2/mip_node_heap_rust.md)·[키](m2/mip_node_key.md), [plunge](m2/mip_plunge_rust.md)·[규칙 4·7·9](m2/mip_plunge_r47_rust.md), [pseudocost](m2/mip_pseudocost_rust.md), [노드 수명](m2/mip_node_life_rust.md), [SB 루프](m2/mip_sb_loop_rust.md)·[타이머](m2/mip_sb_timer_rust.md)·[비전파](m2/mip_sb_direct_rust.md)·[모순·연속 사례](m2/mip_sb_contra_cases.md), [연결 전 입력](m2/mip_node_life.md#71-q3-rust-bb-입력-판정), [하위 작업](14_sprint_plan.md#3-m2-하위-작업), [M2 인수인계](handoff/M2.md), [B&B 설계](06_branch_and_bound.md) |
| M2-07 | Cut 생성·선택·관리 | 예정. 기본 설정에서 실행되는 separator와 pool을 원본에 맞춰 연결하고 유효성과 루트 cut 수를 확인한다. | [cut 설계](07_cuts_heuristics.md), [M2 기준](09_roadmap.md#1-마일스톤) |
| M2-08 | 전파와 휴리스틱 | 진행. 루트 전파와 휴리스틱이 있다. 기본 스케줄과 실행 경로를 원본과 대조하고 누락된 경로를 분리한다. | [루트 기록](m2/m2_root.md), [cut·휴리스틱 설계](07_cuts_heuristics.md) |
| M2-09 | MIP 콜백과 시작해 | 예정. MIP·MIPSOL·MIPNODE 콜백과 관련 제출 API, MIP start의 원본 동작을 각각 검증한다. | [M2 기준](09_roadmap.md#1-마일스톤) |
| M2-10 | B&B 성능·전체 관문 | 진행. 38개 5초 기준 측정이 있다. 노드별 비용은 [M2-10a](m2/mip_bnb_node_cost.md)에서 측정했다(노드 LP cold start가 91–99%, 개선 대상은 basis 재사용). 60초 상태, P1 및 속도를 기존 M2 기준으로 평가한다. | [기준표](m2/mip_bnb_5s.md), [복원 후 측정](m2/mip_recovery_bnb.md#복원-경로-확장-후-38개-재측정) |

## M3 Barrier·PDHG·Concurrent

| 번호 | 작업 단위 | 상태와 완료 확인 | 근거 |
|---|---|---|---|
| M3-01 | Barrier 본체 | 완료(범위 한정). Presolve=0, Crossover=0, Threads=1의 Netlib 64에서 상태·목적과 반복 수를 검증했다. | [barrier 기록](m3/m3_barrier.md) |
| M3-02 | Ordering·분해·해법 | 완료(범위 한정). Netlib 64에서 ordering과 Factor NZ를 확인했다. 분해·해법의 비트 비교는 기록된 사례에 한정한다. | [분해 기록](m3/m3_chol.md) |
| M3-03 | PDHG 본체 | 완료(범위 한정). Presolve=0, Crossover=0, Threads=1의 Netlib 64에서 반복 수를 검증했다. 미실행 전처리 분기는 별도 범위다. | [PDHG 기록](m3/m3_pdhg.md) |
| M3-04 | Crossover | 진행. primal push가 검증 하네스에서 진입 → 초기화(04c-1a·1b·1c·1d) → 루프(04c-22) → 실제 LU backend(04c-24, 21b–21j) → 해 설치(04f-1)까지 이어진다: 설치 연결 59/59(21i), push 재분해 31/31(21j). 드라이버에는 미연결이다(`crates/driver/src/lib.rs`의 crossover 미이식 주석 그대로, Method 2·6은 crossover 없이 실행). 다음은 cleanup 진입 조사 M3-04f-2이고, dual push(04d·04e), 드라이버 연결, 전체 모델 검증(04h)이 남았다. 나머지 미완 항목은 하위 작업 표에 있다. | [초기화 연결](m3/primal_push_init_rust.md), [루프](m3/primal_push_loop_rust.md)·[LU backend 연결](m3/primal_push_loop_lu.md), [해 설치](m3/primal_push_install.md), [조밀 전환 21i](m3/primal_push_dense_lu.md#6-실행-결과), [풀 부족·local_70 21j](m3/primal_push_mode70.md#6-실행-결과), [하위 작업](14_sprint_plan.md#4-m3-하위-작업), [M3 인수인계](handoff/M3.md), [기존 로드맵](09_roadmap.md#1-마일스톤) |
| M3-05 | Concurrent와 Method 자동 선택 | 예정. concurrent 경로와 Method 선택을 원본과 비교하고 Method=4 결정성을 검증한다. | [기존 로드맵](09_roadmap.md#1-마일스톤) |
| M3-06 | PreDual·network simplex·sifting | 예정. 각 경로의 원본 호출 조건과 구현 범위를 확인하고 독립 검증 사례를 만든다. | [기존 로드맵](09_roadmap.md#1-마일스톤) |
| M3-07 | 전체 관문과 속도 | 예정. Netlib 91에서 지정 Method의 상태·목적, 반복 수, Factor NZ와 속도를 기존 M3 기준으로 평가한다. | [M3 완료 기준](09_roadmap.md#1-마일스톤) |

## 다음 작업을 고르는 법

영역별 다음 작업은 각 인수인계 맨 위의 "다음"이 기준이다(2026-11-13). M1은 M1-02z(bnl1 재분해 U 위치 번호), 이어서 M1-02w → 02y-2 → 02t → 02u다([M1](handoff/M1.md)). M3-04c-21h가 넘긴 2×2 블록 단계 켜기는 M1-01e다. M2는 M2-06h-6(VarBranch=3 비전파 오라클)이다([M2](handoff/M2.md)). M2-06 모듈과 M2-03d 재풀이·다이브 행 모듈은 L1 재생까지이고 `bnb.rs` 연결(M2-03d-4 등)에는 M2-10b 등이 먼저 필요하다. M3는 M3-04f-2(cleanup 진입 조사) 또는 M1 `Lu`에 조밀 전환·local_70을 켜는 M1 후속이다([M3](handoff/M3.md)). 하위 번호의 선행 조건과 산출물은 스프린트 작업 목록에서 확인한다. 새 기능을 시작하기 전에는 해당 작업의 원본 위치, 입력 조건, 검증 방법을 상세 기록에 남긴다.
