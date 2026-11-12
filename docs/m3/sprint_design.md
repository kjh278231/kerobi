# M3 스프린트 설계 보완

대상은 [1주 작업 목록](../14_sprint_plan.md)의 crossover, concurrent와 보조 LP 경로다. barrier·PDHG 본체의 기존 검증 범위는 유지한다. 원본 미확인 부분은 아래 조사에서 계약을 확정한 뒤 구현 번호를 추가한다.

착수와 완료 판정은 [동작 동등성 검증 기준](../15_sprint_verification.md)을 따른다. 이 문서의 비교 경계를 작업별 [검증 계약](../templates/sprint_acceptance.md)의 필드·사례·명령으로 구체화한다.

## 1. Crossover 연결부

입력 계약은 내부점의 x·y·slack, 행·열 bound, 스케일, 원래·축약 열 매핑과 시작 basis 정보다. 출력 계약은 simplex가 소비할 basis·열 상태·원시·쌍대 해와 종료 상태다. 이 목록의 실제 필드와 필수 여부는 M3-04a에서 원본 `2baaa4` 호출 전후와 노트14를 따라 확정한다.

| 단계 | 조사 질문 | 검증 경계 |
|---|---|---|
| 진입 | 어떤 해 공간에서 어떤 상태일 때 시작하는가 | barrier·PDHG 종료 직후와 crossover 입력 |
| primal push | 무엇을 bound로 보내고 어떤 순서로 basis를 바꾸는가 | push 한 단계의 후보·이동량·basis |
| dual push | dual feasibility와 기본 변수 교환을 어떻게 처리하는가 | push 한 단계의 y·reduced cost·basis |
| cleanup | simplex로 넘길 때 어떤 값을 재계산하는가 | 설치 직후와 첫 피벗 |
| 재시작 | 수치 실패·정체·한도별로 어디로 돌아가는가 | 상태 전이와 보존 데이터 |

04c·04e는 조사에서 특정한 push 분기 하나만 구현한다. 전체 push phase를 완료하려면 조사 결과에서 필요한 분기별 번호를 모두 추가한다. cleanup 연결은 필요한 분기가 검증된 다음 수행한다. Netlib 최종 목적 일치만으로 내부 basis 패리티를 판정하지 않는다.

04b의 [primal push 조사](primal_push_branches.md)는 double·x87 정밀도, 후보·방향·ratio·이동·basis·재분해·signed remaining 경계를 특정했다. 합성 ratio 29사례×3회는 helper 경계만 검증했다. 04b-1에서 정상 실행의 필수 상태와 실제 pivot·flip을 캡처한 뒤 04c의 구현 분기 하나를 고른다. 나머지 초기화·infeasible ratio·batch·재분해·x87은 조사 §4의 분할 후보에서 계약을 별도로 확정한다.

04c-10·11은 희소 double 한 단계의 선택·적용을 연쇄 재생했다. [04c-12 루프 조사](primal_push_loop.md)는 나머지를 공급자(13→14), factor 제어(15→16), 재분해 뒤 재계산(17→18), 보류·지원·라운드(19→20)로 나눈다. 각 쌍은 원본 오라클 뒤 L1 제어 구현 순서다. 수치 FTRAN/BTRAN/factor는 외부 backend로 구분하며, 실제 Rust Lu 연결은 21 조사에서 오라클·구현을 다시 분할한다. 22는 초기화된 상태에서 이 제어를 연결하는 작업이고 전체 primal push 완료가 아니다. batch는23, 초기화·정렬·기준 조정과 x87·최종 외부 해 설치는 별도다. support 경로에서는 basis 교환 뒤 factor 호출을 생략할 수 있으므로 현재 header만으로 LU를 재구성하지 않는다.

[04c-13 공급자 오라클](primal_push_supplier_oracle.md)은 자연51공급·합성18경계 각3회에서 후보·보류·통계·FTRAN raw와 별칭·dense 압축을 검증했다. interior에서 raw dense9회를 관측했고 반환 시 희소화됨을 확인했다. 합성18경계는 FTRAN을 명시적으로 대체한 제어 증거다. 14는 이 입력을 사용하는 L1 제어 구현이며 실제 수치 backend는21, 공급자 사이 caller 상태를 포함한 연쇄는22로 구분한다.

[04c-14 공급자 Rust L1](primal_push_supplier_rust.md)은 위69입력의 후보·보류·호출·raw 통계·dense 압축을 1,311필드 묶음으로 비교해 통과했다. 비교기 변조29종·Rust 변이22종을 거부했다. backend는 캡처 raw FTRAN/spike를 제공하며 실제 수치 계산은 미검증이고 solver 루프에는 연결하지 않았다. 다음은15 factor 제어 오라클이다.

[04c-15 factor 제어 오라클](primal_push_factor_oracle.md)은 자연 판단28(호출6·생략22)과 원본 선택/wrapper 명령 구간의 합성55사례를 독립3회 검증한다. 자연 helper 수치는 실행되지 않았고, 합성 factor·quality 계산은 외부 backend다. 재계산 직전에서 중단하며 실제 재계산17→18과 연결22는 별도다. 16은 이 제어 경계의 L1 구현이고 품질 helper `089f61`의 수치 오차·Markowitz는15a로 분리한다.

[04c-16 factor 제어 Rust L1](primal_push_factor_rust.md)은 자연 판단28·wrapper6와 합성55, 총89입력·978필드의 요청·호출·복원·retry/status·품질 주기 비교를 통과했다. 비교기 변조55종·Rust 동작 변이30종 모두 거부, workspace132·clippy0와 기존 supplier/chain 회귀도 통과했다. 원본 독립3회 자료를 재사용했으며 실제 factor/quality 계산은 외부 backend다. 요청 비영은 재계산 직전에서 멈춘다. 다음은17 재계산 오라클이며 실제 계산18·루프 연결22는 별도다.

[04c-17 재계산 오라클](primal_push_recompute_oracle.md)은 두 진입점의 자연8회·119행 및 합성120사례·152행을 독립3회 재현하고 새 header·rhs 복사·raw FTRAN·shift·폭·분류·PInf·retry·별칭 재생을 통과했다. 변조82종을 거부하고 선행 회귀·Rust236단계 비트 일치를 유지한다. 자연 미관측 별칭 true·free/fixed·retry와 정확 경계는 원본 명령 구간의 합성 증거다. 18은 이 입력의 shift·분류·PInf·retry L1 구현이며 실제 수치LU21·루프22와 구분한다. 독립 검토자의 자연/합성 재캡처 일치·추가15변조 거부를 확인했다.

[04c-18 재계산 Rust L1](primal_push_recompute_rust.md)은 raw FTRAN 이후 basic 복사/제자리·shift·width·분류·PInf·retry·request 보존을 구현했다. 자연8+합성130, 총138입력·289행·2,958필드 비트/정확 일치, trace변조84종·Rust변이33종 모두 거부했다. 기존 합산 사례의 역순 미검출을 추가 원본10사례×3회로 보강했고 독립검토 재캡처도 일치한다. workspace133·clippy0·factor/supplier/chain 회귀가 통과했다. FTRAN 수치는 외부 입력이며 실제LU21·루프22·batch23는 별도다. 다음은19 보류·지원·라운드 오라클이다.

[04c-19 보류·지원·라운드 오라클](primal_push_round_oracle.md)은 자연3모델의 새207사건(전체751)과 원본 명령 합성106경계를 각각 독립3회 검증했다. 승인수36·보류6·고갈9·round factor7·종료3, 자연BTRAN22회는 모두count1이며 다른 raw 표현은 합성이다. 원본 재현·선행17 비간섭·제어 재생·157변조·독립검토를 통과했다. 초기 생존 caller 연결3종은 직전 applied→BTRAN→다음 공급자 P/factor 상태 연결로 해결했다. 다음20은 이 제어의 Rust L1이며 수치BTRAN/factor는 외부 backend, 실제LU21·루프22·batch23은 별도다.

[04c-20 round Rust L1](primal_push_round_rust.md)은 보류·실패 증가·승인수·지원 표시·순서/중복 보존 압축·고갈 종료·요청10·BTRAN caller·round factor/retry·재계산 뒤 요청 clear를 구현했다. 자연110+합성106, 총216경계·14,365필드 정확/비트 일치와 trace변조4,093종·Rust변이39종 거부, workspace134통과·clippy0·선행 회귀를 확인했다. 원본19 독립3회 자료 재사용·새 원본0회이며 독립검토자는 사용하지 않았다. 현재 API는 기존 solver 루프에서 호출하지 않고 수치 BTRAN/factor는 외부 backend다. 다음21은 FTRAN/BTRAN 목록·spike·지연 LU·재분해 번호 대응의 수치 연결 조사이며,22의 루프 연결·23의 batch와 구분한다.

[04c-21 직렬 수치 backend 조사](primal_push_lu_survey.md)는 한 RHS FTRAN의 입력 변환·L/R 뒤 spike·버퍼 교환, BTRAN의 지원 좌표·Uᵀ cache, 현재 header와 지연 factor 상태·재분해 번호 대응을 C/asm으로 특정했다. 선행19 대표 자연3모델751사건에서 FTRAN42·BTRAN22·factor16(FT5)·재계산8을 집계했고 신규 수치 동등성은 미검증이다. 후속은 FTRAN21a→21b, BTRAN21c→21d, 지연 LU/재분해21e→21f(각 오라클→실제 수치 구현)다. 초기 factor 스냅숏의 같은 상태를 쓰는 커널 검증과 Rust factor 생성 검증을 구분하며 원본 중간 출력 재주입은 금지한다. 외부 backend L1 루프22·품질15a·batch23·실제 backend를 루프에 설치하는 연결 검증은 별도다. [04c-21a FTRAN 오라클](primal_push_ftran_oracle.md)은 자연50호출의 초기 factor·커널 경계·spike·raw를 독립3회 캡처하고 원본 C 규칙 판정·변조36 거부로 고정했다. 21b는 그 `ftran:enter` factor를 초기 상태로 Rust 한 RHS FTRAN을 비트 비교한다. [04c-21e 지연 LU 연쇄 오라클](primal_push_lu_chain_oracle.md)은 `05b257` 진입·반환 factor 전체와 `09e47c`·`036ade` 경계를 FTRAN/BTRAN 스냅숏과 이어, push 안 연산 사이 factor 보존·FT 갱신 판정·재분해 재번호를 55호출·188연결에서 C 규칙으로 고정했다. 21f는 push 첫 factor를 초기 상태로 이 연쇄를 원본 중간 factor 재주입 없이 재현한다. [04c-21f 직렬 backend](primal_push_lu_rust.md)는 재분해 지점의 `036ade` 결과만 주입하고 FT 갱신·FTRAN·BTRAN·재번호를 207연산 비트 일치로 재현했다. [04c-21g 재분해 생성](primal_push_refactor_rust.md)은 M1 `Lu::factor_tol` 결과를 원본 배열로 옮겨 주입점을 없앴고 재분해 18/19와 그 위 연산이 일치한다. 남은 capri 23회째(`1da0b2` 2×2 블록)는 [21h 블록 단계 이식](primal_push_block_rust.md)이 닫았다: 블록 상태(S+0x2c0/0x4e0/0x4e4)를 캡처하고 `Lu::factor_block`으로 재분해 19/19·219연산이 일치한다. M1 simplex에서 켜는 일은 M1-02w다.

[04c-22 루프 연결](primal_push_loop_rust.md)은 14·16·18·20과 10의 L1 조각을 `push_loop` 하나로 이어 첫 패스 준비부터 종료까지 원본 상태 재주입 없이 비교한다. 수치는 21e 캡처 tape이고 재분해 재번호만 루프가 적용한다. 자연 6사례 719줄이 일치했다. 다음 24는 tape 대신 21b–21h의 실제 Rust backend를 설치하는 연결이며, batch23·품질15a·초기화·해 설치는 여전히 별도다.

[04c-24 실제 backend 설치](primal_push_loop_lu.md)는 tape을 `crossover_backend::SerialBackend`(21b–21h의 LuState·refactor·블록 단계, 판단 통계 F190·F1e0·F1e8)로 바꿔, 초기화 factor 입력만 받고 같은 6사례를 원본 수치 재주입 없이 비교한다. 다음은 batch23, 품질 helper 15a, 그리고 초기화(후보 구성)와 해 설치(04f)를 이어 solver 경로에 연결하는 일이다.

[04c-1 초기화 조사](primal_push_init.md)는 C 1143 이전 초기화를 10단계로 고정하고, 21h 자연 6사례의 push:enter만으로 후보 순서·rhs·폭·표시·PInf가 원본과 같게 다시 계산됨을 확인했다(1,480필드). 다음 1a는 이 규칙을 Rust `push_init`으로 옮겨 24의 `SerialBackend`·`push_loop`과 이어 push 진입부터 종료까지 비교한다. 중간 경계·미관측 분기는 1b, 조건수 `116fd7`은 1c다. 해 설치(04f)는 그 뒤다.

## 2. Method 선택과 Concurrent

자동 선택은 입력 분류→Method 결정→실행 경로를 추적한다. M3-05a는 명시 Method, 자동 Method, 모델 분류, 관련 파라미터에 대한 원본 결정표를 만든다. 선택 분기를 고치는 작업과 concurrent 엔진을 만드는 작업은 독립 번호다.

Concurrent 조사는 다음 수명 문제를 해결해야 한다.

- 모델·basis·RNG·로그·콜백·work가 엔진별 소유인지 공유인지.
- 스레드와 자원 배분이 언제 확정되고 언제 해제되는지.
- 승자 결정, 다른 엔진 중단과 결과 설치 순서.
- 한도·수치 실패·사용자 중단 시 반환 상태와 보존 해.
- 결정적 Method의 결과와 반복 수를 비교할 시점.

복제형과 공유형은 05c·05d에서 별도로 조사한다. 조사 후에는 모델 준비, 엔진 시작, 승자 선택, 중단·정리, 결과 설치를 각각 검증 가능한 구현 작업으로 등록한다. 아직 원본 계약이 확정되지 않았으므로 임의의 동시 실행 API를 확정하지 않는다.

## 3. PreDual·Network·Sifting

각각 06a·06b·06c의 독립 조사다. PreDual은 행·열 및 해·목적의 역변환, network는 인식과 전용 풀이, sifting은 부분 문제 선택과 basis 교환을 분리한다. 각 조사 산출물에는 원본 함수, 최소 실행 모델, 필요한 LP 기능, 이후 구현·검증 번호를 포함한다.

## 4. 검증

기본 조건은 기존 barrier·PDHG 기록과 동일하게 명시한다. Crossover 또는 Method를 바꾼 실험은 기존 Crossover=0의 64개 결과와 별도 표로 보관한다. Method=4는 같은 설정 3회 비교를 사용하고, Netlib 91 전체 상태·목적과 P1·P3는 M3-07의 별도 작업으로 판정한다. 아직 구현되지 않은 경로의 검증 완료를 미리 표시하지 않는다.

[04c-1a 초기화 Rust 연결](primal_push_init_rust.md)은 `push_init`으로 push:enter 상태에서 루프 시작 상태를 만들고 24의 실제 backend 루프와 이어 자연 6사례 731줄·5,743필드(basic 비트 포함)를 원본과 맞췄다. 캡처에 없는 param_1·S+0x120·S+0x3c8·조건수는 `Option` 입력이며, 자연 경로가 읽는 조건수만 gate에서 unknown으로 남는다. 1b는 이 값들과 미관측 분기의 원본 오라클, 1c는 조건수 이식이다.
