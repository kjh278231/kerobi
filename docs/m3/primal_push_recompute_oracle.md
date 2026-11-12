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
- frame: F +0x2048, rhs +0x1fd0, 입력/출력 벡터 +0x2000/+0x2008, basic +0x2010, marks +0x2018, width +0x1ff0, lb/ub +0x1f90/+0x1f88, m +0x2068/+0x1f50, I/-I +0x1fc8/+0x1f00, h/-h +0x2070/+0x1f08, attempt/status +0x1fa0/+0x1f98, 위반수 +0x1f28, 복사 길이(m*8) +0x1f10, retry 포인터 +0x20e8. round는 r12d=r13d=0으로 시작한다.
- 자연 계측은 읽기 전용이다. 합성은 원본 제어 명령을 그대로 실행하고 입출구와 FTRAN/로그만 대체한다. callee-saved·스택 정렬·출구를 확인한다. m=0도 FTRAN은 1회 호출하며 분류 루프만 생략한다.

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

저장소 루트에서 실행한다. 새 캡처는 `python3 tools/push_recompute_runs.py <새-출력디렉터리>`이며 기존 출력 덮어쓰기를 거부한다. 표준 검사기는 고정 기준 디렉터리를 읽는다.

```bash
python3 tools/push_recompute_check.py --controls
python3 tools/push_recompute_check.py --acceptance
python3 tools/push_recompute_check.py --self-test
```

출처·도구·입력·기준선·원본 C·전체 asm·캡처 해시는 [manifest](../../bench/oracle/m3_push_recompute/manifest.json), 사례별 오류 거부 이유는 [검증 결과](../../bench/oracle/m3_push_recompute/verification.json)에 있다.
회귀: 선행 factor/supplier controls·acceptance·self-test, `push_chain_oracle.py --check`·`--self-test`, `cargo test -p rsolver-lp --test push_chain`, `push_step_chain_rust.py --compare`. Rust/API 변경이 없어 workspace 전체·확장 설치·pytest·Netlib·P3는 제외한다.

## 6. 실행 결과

- 자연 arms/retry/interior/대조군 각각 독립3회, 283/112/149/0사건이 동일하고 선행15 투영·최종 출력과 일치했다. 자연 재계산8회(pivot1/round7), 119행에서 N89/L21/U3/A2/B4를 관측했다.
- 합성120사례·152행 독립3회 및 재생 일치. F8/X14/L16/N36/U14/A18/B46, retry2회(두 진입점 각각1회). 자연 별칭은 모두 false이며 별칭 true와 free/fixed·정확 경계·retry는 합성 증거다.
- 초기 합성 frame의 복사 길이 슬롯 +0x1f10 누락은 basic 비트 비교 실패로 잡았다. history/missing-copy-bytes-synthetic-{1,2,3}.json에 보존하고 m*8을 설정한 뒤 독립3회 재캡처했다. shifted 경계는 raw 좌표에서 nextafter를 적용한다.
- `python3 tools/push_recompute_check.py --write-manifest --controls --acceptance --self-test --report bench/oracle/m3_push_recompute/verification.json` 종료0. 변조82종을 모두 거부했다. 선행 factor controls/acceptance/self-test41종, supplier controls/acceptance/self-test31종, chain check/self-test8종도 종료0이다.
- `cargo test -p rsolver-lp --test push_chain` 1통과, `push_step_chain_rust.py --compare` 236단계·5,144필드 비트 일치(42구간·재분해 비교 제외6 유지). py_compile·node --check·전체 diff 검사 통과. [회귀 명령/결과·환경](../../bench/oracle/m3_push_recompute/regression.json). Rust/API/기존 solver 실행 경로 변경이 없어 workspace 전체·clippy·Python 확장 설치·pytest·Netlib·P3는 제외한다.
- 독립 검토 **통과**: `/root/m3_04c17_review`가 C/asm·두 경계·ABI·출구를 확인하고 자연 interior149사건과 합성120사례를 새 원본 프로세스로 각각1회 재캡처해 사건 전체·환경·식별값·자연 최종 출력/basis 일치를 확인했다. 추가 변조15종을 전부 거부했다. [검토 기록](../../bench/oracle/m3_push_recompute/review/review.md), [재캡처·해시/추가 변조 결과](../../bench/oracle/m3_push_recompute/review/review.json).
- 보존 asm이 dc.py 기본200줄에서 잘린 점은 전체4036줄로 보강하고 manifest를 갱신했다. 독립 검토자의 신선 asm 실행과 파일 전체가 일치한다. 정식 캡처15회(자연12·합성3), 탐색2회, 실패 합성3회, 독립 재캡처2회로 새 원본 실행은 총22회다. 탐색/실패를 합격 건수에 포함하지 않는다.

## 7. 판정과 인수인계

- 작업 수행: **오라클 구축 완료, 계약 범위 통과**. 오라클 경계는 자연8회·119행 및 합성120사례·152행에서 비트 일치한다. Rust 신규 구현 동등성은 미검증이며 18의 작업이다.
- 다음은 **M3-04c-18**. `python3 tools/session.py status` → `python3 tools/session.py claim M3-04c-18 --owner <새-owner>` 후 본 계약과 `primal_push_loop.md` §2.5를 읽고, raw FTRAN을 외부 입력으로 받아 basic·width·marks·위반수·PInf·retry L1을 구현한다. 행 순서를 보존하고 두 진입점의 요청 지우기 전 경계를 대조한다.
- 자연 별칭 true·free/fixed·정확 경계·retry의 도달성은 미검증(합성만). FTRAN/LU 수치 재구현21·caller 루프22·batch23·x87·전체 work/P3도 미검증이다.
- branch `work/m3-04c-17`, 기준 main `36f93ff`, 계약 `14a560d`, 도구 첫 동작 `2824795`, 정식 캡처/검사 `9f69dbf`. 최종 commit/병합은 main의 `Merge M3-04c-17` 이력을 따른다. 선행 미커밋 없음. 병합 후 같은 단계에서 worktree·branch와 17/shared 점유를 정리한다.
- 진행·로드맵·스프린트·설계·인계는 본 branch에서 갱신해 병합한다. 교훈은 `docs/lessons/original.md`(재계산 두 경계와 0행 FTRAN), `tooling.md`(fake frame 복사 길이·round의 rsp 쓰기·asm 보존)에 기록한다. 원격 동작/외부 업로드 없음.
