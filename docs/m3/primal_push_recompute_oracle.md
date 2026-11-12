# M3-04c-17 재분해 뒤 재계산 오라클

## 1. 범위와 주장

- 종류: 오라클 구축. factor 종료 뒤 두 재계산 진입점의 rhs 복사·dense FTRAN 호출·basic 복사·하한 이동·폭·분류·PInf·retry를 고정한다.
- 선행 main `36f93ff`(M3-04c-16), branch `work/m3-04c-17`, worktree `../rsolver-m3-04c-17`. 미커밋 선행 없음. main의 별도 `.claude/settings.local.json`은 보존한다.
- 포함: double, Threads=1, batch=0. 자연 arms·retry·interior 및 Crossover=0 대조군. 자연 미관측 정확 경계는 원본 명령 구간 합성으로 별도 분류한다.
- 제외: Rust 신규 구현(18), 실제 LU 수치 재구현(21), 보류·지원·라운드 제어(19→20), 연결(22), batch(23), x87·전체 work·P3. 합성은 FTRAN 및 로그 helper를 명시적으로 대체한다.

## 2. 원본과 상태 경계

- 원본 Gurobi 13.0.3, 라이브러리 SHA-256 `8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c`.
- C `../re/decomp/c/001c0070_PRIVATE0000000000134a87.c` 1897–1980, 2331–2418. ELF offset 기준이며 Ghidra 주소는 +0x100000.
- 시작: pivot wrapper 뒤 `0x1c3872`, round factor 뒤 `0x1c3db4`. 새 F+0x18 header를 읽기 직전이다.
- FTRAN `09cf9b@0x1205d0`: 호출 `0x1c38de`/`0x1c3e20`, 복귀 `0x1c38e3`/`0x1c3e25`. 인자 F, 입력 벡터, 출력 벡터, 0, work. 입력 count=-1.
- 정상 종료 `0x1c3aed`/`0x1c400a`는 분류·PInf·retry 게이트 뒤다. retry 쓰기 뒤 cleanup 진입 `0x1c08d5`도 관측한다. 후속 quality/공급자 실행은 범위 밖이다.
- frame: F +0x2048, rhs +0x1fd0, 입력/출력 벡터 +0x2000/+0x2008, basic +0x2010, marks +0x2018, width +0x1ff0, lb/ub +0x1f90/+0x1f88, m +0x2068/+0x1f50, I/-I +0x1fc8/+0x1f00, h/-h +0x2070/+0x1f08, attempt/status +0x1fa0/+0x1f98, 위반수 +0x1f28, retry 포인터 +0x20e8. round는 r12d=r13d=0으로 시작한다.
- 자연 계측은 읽기 전용이다. 합성은 원본 제어 명령을 그대로 실행하고 입출구와 FTRAN/로그만 대체한다. callee-saved·스택 정렬·출구를 확인한다. m=0도 호출·분류 생략 경로를 확인한다.

## 3. 비교 계약

| 필드·사건 | 규칙 | 방법 |
|---|---|---|
| 새 header·차원·attempt/status·request | 정수·배열 순서 정확 | 시작 frame와 F |
| rhs·lb/ub·I/h·raw FTRAN·basic·width·PInf | little-endian f64 비트(±0 포함), 오차 허용 없음 | 전후 전체 배열 |
| 입력 count·인자·별칭·호출 순서 | count=-1, F/입출력 벡터/flag=0, rhs=input 및 raw=basic 별칭을 명시 | 함수 진입·반환 |
| marks·위반수·retry·요청 보존 | 정확, PInf는 basis 순서 순차 누적 | 분류 후·retry 게이트 후 |
| 보존 상태 | header·lb/ub·rhs(별칭 시 변동 구분), 자연 이전 사건 전체 투영 | 내용/별칭·선행15와 비교 |

합성은 유한값과 내부 sentinel I를 사용하며 NaN/IEEE 무한대는 제외한다. retry는 model_status=2, attempt=0, PInf>1e10 세 조건을 비교한다. 시간과 전체 work는 비교하지 않는다. 동일 설정 원본 독립3회, 전후 사건 순서·모델 결과·환경을 대조한다. 빈 목표·필수 필드 누락·해시 오류·null-null·잘못된 분류/수치/순서 변조는 실패 종료해야 한다.

## 4. 사례와 원본 자료

- 자연: 선행15와 동일 모델·파라미터·PStart/DStart, arms·retry·interior 각3회. 두 진입점은 전체 자연 집합에서 각각 1회 이상 필수다. Crossover=0은 3회·목표0회.
- 합성: 두 진입점 각각 free, 한쪽 무한, 비영 하한, 폭0, ±h 및 인접 representable 값, w±h 및 인접값, 위반0/비영·누적순서, PInf=1e10 전후·attempt/status 교차, m=0, rhs/입력·raw/basic 별칭 유무. 원본 재분해에서 도달했다고 주장하지 않는다.
- `bench/oracle/m3_push_recompute/`에 탐색·정식 캡처·명령·환경·asm·manifest·오류 검출 결과를 보존한다. 도구·입력·기준선·캡처 해시와 스키마를 manifest에 고정한다.

## 5. 재실행 명령

예정: `python3 tools/push_recompute_runs.py <새-출력디렉터리>`; `python3 tools/push_recompute_check.py --controls`, `--acceptance`, `--self-test`.
회귀: 선행 factor/supplier controls·acceptance·self-test, `push_chain_oracle.py --check`·`--self-test`, `cargo test -p rsolver-lp --test push_chain`, `push_step_chain_rust.py --compare`. Rust/API 변경이 없어 workspace 전체·확장 설치·pytest·Netlib·P3는 제외한다.

## 6. 실행 결과

착수 계약. 아직 새 원본 캡처나 동등성 검증을 실행하지 않았다.

## 7. 판정과 인수인계

작업 수행: 진행 중. 동등성: 미검증. 완료 전 독립 검토를 받는다. 다음 단계는 asm 경계와 별칭 대응 확인 후 탐색1회다. 교훈과 최종 인계는 실행 후 갱신한다.
