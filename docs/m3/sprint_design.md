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
