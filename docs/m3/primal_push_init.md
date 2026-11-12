# M3-04c-1 primal push 초기화 조사

작성 기준은 [스프린트 검증 기준](../15_sprint_verification.md)과 [계약 양식](../templates/sprint_acceptance.md)이다.

## 1. 범위와 주장

- 종류: **조사**. 기준 main `60c6179`, branch `work/m3-04c-1`, worktree `../rsolver-m3-04c-1`. 선행 미커밋 변경 없음. main의 미추적 `.claude/settings.local.json`은 작업과 무관하며 보존한다.
- 질문: double primal push `134a87`의 진입부터 첫 패스 준비(`0x1c1ff8`, C 1143)까지 초기화는 어떤 순서로 무엇을 읽고 쓰는가? 04c-22·24의 `push_loop` 입력(후보 목록·vstat·rhs S+0x200·basic·폭·표시·PInf·반복 수·지원 flag)을 push 진입 상태에서 만들려면 어떤 원본 규칙과 상태가 필요한가? 기존 원본 자료로 어디까지 판별할 수 있고 무엇을 새로 캡처해야 하는가?
- 완료 조건:
  1. C 150–1142의 단계·분기·상태 읽기/쓰기를 C와 asm 근거로 표로 고정한다. 디컴파일 표현이 의심스러운 지점은 asm으로 판정한다.
  2. 기존 원본 자료(M3-04c-21h `bench/oracle/m3_push_block/`, 자연 6사례 × 독립 3회)의 `push:enter` 입력에서 C 규칙으로 다시 계산한 **후보 집합·정렬 순서, rhs S+0x200, 표시·위반 수·PInf, 폭, 반복 수, 지원 flag**를 첫 `supply:enter`의 원본 값과 정확/비트 비교하는 재실행 가능한 조사 도구를 만든다. basic 값(FTRAN 결과)은 이 도구의 비트 비교 대상이 아니다.
  3. 도구의 판별력을 입력 변조와 규칙 변이로 확인한다. 자연 자료가 구별하지 못하는 규칙은 미관측으로 분류한다.
  4. 자연 자료에서 실제 지난 분기(상태 -1…-5, slack -2, 기준 조정, Markowitz 강화, 품질 retry, 첫 push 여부)를 집계하고, 후속 오라클·구현 번호와 필수 비교 필드를 등록한다.
- 제외: 새 원본 실행, Rust 신규 구현, basic 값·조건수(`12f282`→`116fd7`) 수치 동등성, x87(`256781`), batch 할당 뒤 batch 동작(23), 해 설치(C 2419 이후, 04f 계열). 초기화 구현의 동등성은 **미검증**으로 남긴다.

## 2. 원본과 상태 경계

- 원본 Gurobi 13.0.3, 라이브러리 SHA-256 `8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c`(로컬 `../.venv/.../libgurobi130.so`와 21h manifest의 `binary_sha256`이 같음을 도구가 확인한다).
- 디컴파일 `../re/decomp/c/001c0070_PRIVATE0000000000134a87.c` C 150–1142, asm은 `python3 tools/dc.py 134a87 --asm`의 `0x1c0070–0x1c1ff8` 구간과 out-of-line 블록 `0x1c4f29–0x1c51bc`. 정렬 `aabbe3@0x8bccb0`, 잔차 `089d8f@0x115d60`, Markowitz 강화 `12f5d1@0x1bc0e0`→`061bfe@0xf62c0`, 조건수 `12f282@0x1bbf80`. 주소는 ELF file offset(Ghidra = +0x100000).
- 시작: `134a87` 진입(`push:enter`). 종료: 첫 `25615c` 호출 직전(`supply:enter`, 그 직전의 `roundctl:pass:enter/ready`). 품질 retry·할당 실패로 루프에 들어가지 않는 반환도 분기 목록에 포함하되 자연 자료에 없으면 미관측으로 둔다.
- 설정: 21h 캡처의 Threads=1, Seed=0, Presolve=0, Method=2, Crossover=3(primal-first), LPWarmStart=2. 모델 arms·retry·interior(저장소 LP)와 Netlib adlittle·scsd1·capri.
- 입력 상태(`push:enter`): 모델 CSC·목적·rhs S+0xa0·bound S+0xa8/b0, vstat S+0x168, header, primal x(param_5)와 slack(param_6, x 뒤 연속 별칭), 허용오차 S+0x370/380/390/398/3b0, 카운터, 첫 push flag(param_7)·시도 번호(param_4)·진입 상태(param_3). FeasibilityTol 인자 xmm0(param_1)는 캡처되지 않았다. 도구는 이를 S+0x380과 같다고 가정하지 않고, 이 값이 결과에 영향을 주는 분기(기준 조정)가 실행됐는지를 따로 판정한다.
- 출력·변경 상태: 후보 목록(S+0x208+8)·총수, vstat(기준 조정 시 -1/-2 → -3), rhs S+0x200, basic S+0x1e8, 폭 (S+0x208)+0x10, 표시 S+0x280, PInf S+0x120, 반복 S+0x2a0(+1), 수치 카운터 S+0x178, Markowitz S+0x390과 갱신 한도 S+0x3c8·F+0x188, 지원 flag(local_3c), factor(요청 10 재분해).

## 3. 비교 계약

| 필드·사건 | 의미·좌표·순서 | 비교 규칙·근거 | 캡처·재구성 방법 |
|---|---|---|---|
| 후보 목록 | `pass:enter`·`supply:enter`의 candidates, 원본 정렬 뒤 순서 | 정확(순서 포함). key: 열 `-(len_j + min(d,0.99))`, slack `-1 - min(d,0.99)`, `aabbe3` 내림차순(C 617–640, asm `0x1c10e0–0x1c112b`) | push:enter 입력에서 거리 d를 C 규칙으로 계산하고 `aabbe3`의 Python 이식으로 정렬 |
| vstat | supply:enter의 n+m 상태 | 정확. 기준 조정이 없으면 진입과 같아야 하고, 있으면 -1/-2→-3 변화만 허용(C 1069–1137) | push:enter와 비교 |
| rhs S+0x200 | 제약 행 m개 | **비트**. S+0xa0 복사 → 열 j 오름차순·CSC 순서로 `r -= v·a`(v≠0일 때) → slack 순서로 `r -= v`. slack -2의 v는 asm 그대로 직전 xmm0(§4.2) | 진입 입력에서 같은 순서로 재계산 |
| 폭 | header 위치별 (S+0x208)+0x10 | 비트. lb ≤ -I 또는 ub ≥ I면 I, 아니면 ub-lb(C 800–815) | 진입 bound·header로 재계산 |
| 표시·위반 수·PInf | S+0x280, S+0x120 | 표시 정확, PInf 비트. 원본 basic·폭으로 C 816–866 분류를 다시 수행 | supply:enter의 원본 basic·폭을 입력으로 사용(basic 자체는 비교 안 함) |
| 반복·카운터 | S+0x2a0, S+0x178, S+0x294 | 정확. 반복은 진입+1(C 902–903) | push:enter·supply:enter 비교 |
| 지원 flag | pass:enter support | 정확. `sqrt(m) <= total`(C 909–910) | 재계산 |
| Markowitz·갱신 한도 | S+0x390, factor limit | 정확. 강화 시 `061bfe` 규칙 | 진입 대비 변화 기록, 강화 0회면 불변 확인 |

- 수치 오차 허용 없음. basic 값은 FTRAN(`09cf9b`) 결과이므로 여기서 비교하지 않고, 진단용 잔차 `max|B(basic+lb) - rhs|`만 기록한다(판정에 사용하지 않음). NaN·무한대는 입력 sentinel I(S+0x370)로만 다룬다.
- 시간·work·병렬: Threads=1, batch 수는 0(C 221–233)인 자연 자료만 사용한다. work는 비교하지 않는다.
- 재현성: 기존 자료의 3회 독립 캡처가 초기화 관련 필드에서 서로 같은지 도구가 확인한다. 새 원본 실행은 하지 않는다.

## 4. 사례와 원본 자료

| 사례 | 실제 / 합성 | 전제·노리는 분기 | 입력 경로·해시 | 원본 출력·분기 실행 증거 |
|---|---|---|---|---|
| arms·retry·interior·adlittle·scsd1·capri | 실제(정상 실행 훅) | primal-first 첫 push, 요청 10 재분해, 후보 구성·정렬·분류 | `bench/oracle/m3_push_block/*-{1,2,3}.json(.gz)`, manifest 해시 | 각 3회 push:enter·factor·pass·supply 사건 |
| control | 실제(Crossover=0) | push 미진입 대조군 | 같은 manifest | 사건 0 |
| 기준 조정·Markowitz 강화·품질 retry·-4/-5·slack -2 | 자연에서 탐색 | 집계로 발생 여부 판정 | 위와 같음 | 발생 0이면 미관측으로 남김 |

- 적용하지 않는 범주: 합성 입력·새 원본 캡처(이번 조사 범위 밖), dual-first 뒤 push(param_7=0), 두 번째 시도.

## 5. 재실행 명령

- `python3 tools/push_init_survey.py --check` — manifest 해시 확인, 3회 일치, 사례별 재계산 비교, 분기 집계를 `bench/validation/m3_push_init/survey.json`과 바이트 비교.
- `python3 tools/push_init_survey.py --write` — 집계 갱신(최초 작성 시에만).
- `python3 tools/push_init_survey.py --self-test` — 입력 변조와 규칙 변이마다 비교가 실패하는지(또는 자연 자료가 구별하지 못하는지) 출력하고, 필수 변조가 하나라도 통과하면 0이 아닌 값으로 종료.
- 회귀: `python3 tools/push_block_check.py --controls --acceptance`(21h 자료 무결성), `python3 -m py_compile tools/push_init_survey.py`. Rust 변경이 없으므로 Cargo·clippy·확장·pytest는 실행하지 않는다.

## 6. 실행 결과

(작성 예정)

## 7. 판정과 인수인계

(작성 예정)
