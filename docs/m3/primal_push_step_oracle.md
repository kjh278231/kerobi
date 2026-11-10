# M3-04b-1 정상 primal push 한 단계 오라클

## 1. 착수 검증 계약

- 종류: 정상 실행 오라클 구축. 기준선 main `297d44b`, branch `work/m3-04b-1`, 미커밋 선행 변경 없음.
- 목표: 읽기 전용 Frida 훅으로 double `134a87`의 진입→후보 공급→ratio→pivot 또는 bound 이동→다음 후보 공급과 반환을 연결한다. primal-first와 dual-first를 각각 독립 프로세스 3회 실행하며, 각 순서에서 실제 pivot·flip을 최소 한 건씩 요구한다.
- 원본: Gurobi 13.0.3 ELF SHA-256 `8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c`. C `001c0070_PRIVATE0000000000134a87.c`, `001c5310_PRIVATE000000000025615c.c`, `000ef960_PRIVATE0000000000055f68.c`, `000eefa0_PRIVATE0000000000054eef.c`, `000f15c0_PRIVATE000000000005b257.c`, 호출자 `001bc5d0_PRIVATE000000000013f716.c`를 사용한다. 경로는 저장소 옆 `re/decomp/c/` 기준이다.
- 시작·종료: push 함수 진입·반환, 후보 공급자 전후, ratio 전후, ELF `0x1c2994` bound 이동 직전·`0x1c2ab5` pivot 승인 직전부터 다음 공급자 진입까지. 같은 push 호출·후보 순번으로 사건을 연결한다. 내부 훅의 S는 r14이며 ELF로 확인했다.
- 입력: workspace 차원·CSC·목적·rhs·bound·vstat·basis header, x/slack 연속 배열, basic 값·폭·표시, FTRAN 희소 벡터, 후보 순서·소비 위치·실패 횟수·지원 집합, factor 요청·반복·허용오차를 의미 기반으로 캡처한다. 주소를 비교하지 않는다. 초기화 전 scratch·출력값은 읽지 않고 해당 수명 경계에서만 기록한다.
- 반환: C 반환·workspace 상태·retry·signed remaining·condition과 x/slack을 각각 저장한다. ratio 선택은 sparse 위치와 행을 구분하고 theta·pivot을 저장한다.
- 비교: 정수·후보 순서·사건 순서·상태·별칭은 정확 비교, double은 little-endian 8바이트 hex로 ±0까지 비트 비교한다. 초기화되지 않은 scratch나 메모리 padding은 대상이 아니다. NaN은 허용하지 않는다. 내부 sentinel은 비트 비교한다.
- 환경: Threads=1, Seed=0, Presolve=0, Method=2, Crossover=1 또는 3. 다른 파라미터와 시작값은 캡처에 고정한다. 원본 allocator·factor 전체와 RNG 내부 상태의 독립 재생, x87·실패·중단·batch·다른 모델·Rust 구현·시간 및 전체 work 패리티는 제외한다. factor 내부를 복원하지 못해도 캡처를 factor 재생 오라클로 보고하지 않는다.
- 사례: 선행 소형 LP와 필요한 실제 LP를 사용해 목표 도달을 확인한다. 입력 탐색의 실패·목표 0건도 history에 보존한다. Crossover=0 대조군은 목표 0건을 별도 판정한다. 동점·퇴화·허용오차 경계의 강제 입력은 이번 정상 실행 범위에 포함하지 않는다.
- 합격: 같은 설정 원본 독립 3회에서 모든 계약 필드가 일치하고 각 순서에 pivot·flip이 존재한다. 목표 0건·핵심 상태 누락·합성 출처·바이너리/입력 해시 변조를 CLI에서 거부한다. 완성된 검사기는 실제 캡처 변조로 검사한다.
- 회귀: 기존 ratio·crossover entry 저장 검사를 실행한다. 솔버·Python API 변경이 없으므로 Rust·확장 재빌드는 대상이 아니다. 명령·입력·도구·소스·캡처 해시와 실제 결과는 아래에 추가한다.

## 2. 실행 결과

진행 중이다. `history/small-primal-2.json`은 정상 push 반환이지만 후보 0건이고, `history/face-primal-1.json`은 pivot 1건·flip 0건이다. 초안 검사기로 face 캡처의 상태·basis 교환을 확인했으며 마지막 목표 개수 검사에서 불합격했다. 이 탐색 자료를 합격 기준으로 사용하지 않는다. 독립 3회와 목표 두 종류를 확보하기 전 완료로 판정하지 않는다. 
