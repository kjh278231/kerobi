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

### 2.1 초기화 단계(실행 순서)

C 줄은 위 디컴파일 파일, asm은 ELF offset이다. C의 label 배치 순서가 아니라 실행 순서로 적는다.

| 단계 | 처리와 분기 | 읽기 → 쓰기 | 근거 |
|---|---|---|---|
| 0 진입 | param 저장, `*retry=0`, 품질 주기 50, 조정 flag 0. 스레드 수 `ac7d4e`≤64, batch 수 `min(threads, trunc(0.5·log(10m)))`, 2 미만이면 0 | S, param_1(τ_in) → 지역값 | C 162–233, asm `0x1c030f`·`0x1c0339` |
| 1 할당 | batch 레코드(행 m+1개 int·double 3쌍) × batch 수, 후보 실패 횟수 m, threads>1이면 스레드별 scratch, n+m·m 크기 배열. 실패는 10001과 `remaining=0`(C 326–332) 또는 10001 | → scratch | C 234–400 |
| 2 초기 factor | **first(param_7)≠0**: S+0x294=10, `05b257` 반환 1 반복(caller `0x1c07c6`). 반환 12 → retry=1. 반환 ≥2 → 그 코드 반환. 반환 0/1에서 S+0xf0 ≤ -1000이면 S+0xf0=1·retry=1. 아니면 attempt≠0 또는 (S+0x184=0이고 S+0x178 < S+0x178(진입)+10)일 때 S+0x294=0, header 다시 읽고 계속, 아니면 retry=1. first=0은 바로 3 | S·factor → factor·header·S+0x294 | C 1017–1051, asm `0x1c07c1–0x1c08aa`·`0x1c0b3f` |
| 3 rhs·후보 | S+0x1d8 \|= 1. S+0xa0 → (S+0x228)+8 복사. 열 j 오름차순: -1 lb, -2 ub, -3 x(후보, 거리 `ub+lb > 2x ? x-lb : ub-x`), -4 lb, -5 ub, 그 외 음수 0; v≠0(NaN 포함)이면 CSC 순서로 `r -= v·a`. -1·-2는 `x-lb`·`ub-x`가 τ_in·{0.001,0.01,0.1}보다 큰 수를 셈. slack i: -1 v=0(slack 값으로 같은 셈), -3 v=slack(후보, 거리=slack), **-2 v=직전 xmm0**(§2.2), -4·-5·기저는 건너뜀; `r_i -= v` | vstat, x, slack, bound, CSC → rhs scratch, 후보·거리 | C 401–596, asm `0x1c0b5e–0x1c0fc3`·`0x1c4f29–0x1c51bc` |
| 4 정렬 | key 열 `-(len_j + minsd(d,0.99))`, slack `-1 - minsd(d,0.99)`; `aabbe3` 내림차순(동률 순서는 원본 3-way quick/shell 이동). rhs scratch → S+0x200 복사(별칭이면 생략) | 후보·거리 → 후보 순서, S+0x200 | C 598–645, asm `0x1c1090–0x1c114a` |
| 5 basic | count=-1로 `09cf9b(F, S+0x228, S+0x220)`(dense rhs FTRAN). max\|rhs\|, `089d8f`로 S+0x1f8 += B·x_B, max\|x_B\|, max\|rhs-Bx_B\|, S+0x1f8 다시 0 | factor, rhs → x_B, 잔차 | C 646–785, asm `0x1c11cc`·`0x1c1282`·`0x1c149d` |
| 6 Markowitz | 잔차 >1e-8 또는 max\|x_B\| > 1e10·max\|rhs\| → 0.5, >4e-9 또는 >1e9배 → 0.25, 아니면 1e-4. S+0x390보다 크면 `12f5d1`(double은 `061bfe`: S+0x4b8→+0x100→+0x2c54 < 1이면 갱신 한도 S+0x3c8·F+0x188을 `max(50, trunc(min(0.9·S+0x3c8, trunc(sqrt m))))`가 기존보다 작을 때만 줄임) 뒤 S+0x390 갱신·경고. 잔차 >1e-6이면 S+0x178++ | → S+0x390·0x3c8·F+0x188·S+0x178 | C 786–796, `061bfe` C 1–31 |
| 7 shift·폭·분류 | x_B → S+0x1e8 복사, header 변수 lb > -I면 basic -= lb, 폭 = ub ≥ I면 I 아니면 ub-lb(lb ≤ -I면 I). 표시: 양쪽 무한이면 F, `v < -τ` B, `v < τ`면 폭0 X 아니면 L, `v < w-τ` 또는 w ≥ I면 N, `w+τ ≤ v` A, 나머지 U. 위반 수와 PInf(B: -v, A: v-w)를 위치 순으로 누적 | x_B, header, bound → S+0x1e8, 폭, S+0x280, PInf | C 797–866 |
| 8 품질 | first면 `12f282`(조건수, double은 `116fd7`)를 *condition에 씀. attempt=0이고 cond>1e14 또는 P>1e10 또는 cond·P>1e16(P는 진입 상태 param_3=2일 때 PInf, 아니면 1)이면 retry=1. cond>1e30 → retry=1. PInf>1e15 → S+0x178++, retry=1 | factor → condition, retry | C 867–896 |
| 9 기준 조정 | 조정 안 했고 PInf>1이고 S+0x120(**이전 push의 PInf**) < τ이고(M3-04c-1a에서 asm `0x1c1873`으로 방향 정정) 후보 수 > (0.1τ_in 초과 셈)이면 1회: τ_in에 0.001(0.001 셈 < 후보 수)·0.01·0.1을 곱하고 rhs 재구성. 열 -1은 `lb+τ' ≤ x`면 -3, -2는 `x ≤ ub-τ'`면 -3, **-4·-5 열은 rhs 보정 없이 건너뜀**; slack -1은 `τ' ≤ slack`이면 -3, slack -2·-3은 항상 -3(거리=slack). 그다음 4로 돌아감 | vstat·rhs·후보 재작성 | C 931–1012·1063–1141, asm `0x1c1ab2–0x1c1dd2` |
| 10 루프 준비 | S+0x120=PInf, S+0x2a0 += 1, `d07d1` 로그, 지원 flag `sqrt(m) ≤ 후보 수`, 후보 0이면 종료 경로, 품질 caller 상수(5m², 1.5m, 2τ, ±100τ) 준비 후 `0x1c1ff8` | → S+0x120·0x2a0, 지원 | C 897–930 |

### 2.2 asm으로 판정한 원본 사실

- **slack -2의 rhs 보정은 직전 xmm0이다.** 첫 slack 스캔에서 -2는 `0x1c4f3d`에서 `0x1c4f68`(`rhs_i -= xmm0`)으로 바로 가며 xmm0을 쓰지 않는다. 값은 직전 slack -1(0)·-3(slack 값), 앞선 slack이 없으면 마지막 음수 상태 열의 v(그 외 음수는 0), 열도 없으면 `0x1c0b68`의 0이다. C의 `dVar49` 재사용과 같은 의미다. 정상 bound(-2 slack의 ub=0)라면 0을 빼야 하므로 원본 특유 동작이며, 자연 6사례에는 slack -2가 없다.
- **기준 조정 재구성은 -4·-5 열을 rhs에서 빼지 않는다.** `0x1c1cae`에서 상태 < -3이면 다음 열로 간다. 첫 구성(`0x1c4fdc–0x1c4fe8`)은 -4 lb·-5 ub를 뺀다. 자연 6사례 중 arms(1)·capri(16)에 값이 0이 아닌 -4 열이 있으나 조정이 0회라 결과 차이는 관측되지 않았다.
- 거리 상한은 `minsd(d, 0.99)`(`0x1c10f5`·`0x1c111e`)라 d ≥ 0.99와 NaN이 0.99가 된다. 열 key는 `cvtsi2sd(len) + min`을 부호 반전(xorps), slack key는 상수 -1.0에서 뺀다.
- `-3` 거리의 중점 비교는 `comisd 2x, ub+lb; jbe`라 NaN이면 `ub - x`다.
- 정렬 `aabbe3`는 불안정 정렬이다. 저장소 `crates/lp/src/common.rs::sort_desc`가 같은 이동을 이식했고, 도구는 이를 Python으로 옮겨 사용했다.

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

- commit `c4f2f21`(도구·집계), 기준 자료 `bench/oracle/m3_push_block/` manifest(21h)의 capture 해시 18개와 binary SHA를 도구가 확인. 도구 SHA-256 `8908c9f0a71816f6e55aa38cb1e1b12a1f1fe580775108188041c60e819c0b4b`. 원본 새 실행 0회.
- 원본 3회: 6사례 모두 push:enter·pass:enter·supply:enter가 pid/run_id·공급자 문맥을 빼고 3회 동일.
- 재계산 비교: **6/6사례 첫 실행부터 통과**, 비교 필드 1,480개(후보 103개 순서, rhs·폭·표시 각 m개, PInf·반복·지원·요청·총수·초기 factor 호출). 실패·skip·xfail 0.

| 사례 | n·m | 후보 | 동률 key | 안정 정렬과 다름 | 0.99 상한 | 비영 -4 열 | 표시 | 진단 잔차 max |
|---|---|---|---|---|---|---|---|---|
| arms | 42·25 | 23 | 21 | 예 | 17 | 1 | N21·L3·X1 | 1.7e-16 |
| retry | 18·10 | 6 | 6 | 아니오 | 6 | 0 | A2·B3·L2·N3(위반5, PInf≤1) | 0 |
| interior | 12·8 | 9 | 7 | 예 | 8 | 0 | N7·L1 | 1.3e-15 |
| adlittle | 97·56 | 22 | 18 | 예 | 20 | 0 | N49·L5·X2 | 1.1e-13 |
| scsd1 | 760·77 | 12 | 0 | 아니오 | 0 | 0 | N19·X58 | 2.8e-17 |
| capri | 353·271 | 31 | 11 | 예 | 14 | 16 | N232·L17·X8·F14 | 5.7e-13 |

- 자연 분기 집계: 6사례 모두 first=1·attempt=0, 진입 상태(param_3)는 arms·retry 1, 나머지 2. 초기 `05b257`(요청 10, caller `0x1c07c6`) 반환 0. 기준 조정·Markowitz 강화·수치 카운터 증가·품질 retry **0회**(최종 PInf ≤ 1, S+0x390·갱신 한도 불변). 관측 상태: 열 -1(6사례)·-2(interior·retry·capri)·-3(6사례)·-4(arms·capri), slack -1(adlittle·capri)·-3(arms 2)·-4(4사례). **열 -5·slack -2는 0회.** 후보 수 < 100이라 `aabbe3` quick 경로 0회(shell만). 기저 위치 재번호는 초기 재분해 순열로 5사례에서 발생(vstat ≥ 0 값만 변함, 기저 집합·`vstat[header[p]]=p` 일치).
- 판별력(`bench/validation/m3_push_init/self_test.txt`): 입력·출력 변조 8종(후보 교환, rhs·폭 1비트, 표시, 반복, 입력 rhs·후보 x 1ulp, 지원 flag)과 자료 변조 3종(바이너리 해시 0, 사건 비우기, 첫 supply 제거) **11종 모두 거부**. 규칙 변이 7종 중 거부 4(안정 정렬, 0.99 상한 제거, 거리 항상 x-lb, slack 후보 먼저), **생존 3**: slack -2 = 0(자연 0회), -τ 경계 strict(경계값 0회), -4를 ub로(-4 변수는 6사례 모두 lb=ub라 동치).
- 회귀(`bench/validation/m3_push_init/regression.txt`): survey `--check`·`--self-test`, `push_block_check.py --controls --acceptance`, py_compile 모두 종료 0. Rust 변경이 없어 Cargo·clippy·확장·pytest는 실행하지 않았다.
- 실행하지 못한 검사: basic 값 비트 비교(FTRAN 결과, 이번 범위 밖), 조건수·잔차·Markowitz 판단 입력값(캡처 없음), τ_in(param_1, 캡처 없음).
- 계약 변경: vstat 비교를 "진입과 같음"에서 "비기저 상태 같음 + 기저 집합·header 일치"로 바꿨다. 첫 실행에서 초기 재분해 순열로 기저 위치 번호가 바뀌는 것이 드러났기 때문이며(원본 C:1017–1051의 재분해 뒤 재번호), 이 변경 전 결과(vstat 5사례 실패)는 기저 번호 차이뿐이었다.

## 7. 판정과 인수인계

- 작업 수행 **완료**(조사). 질문에 대한 답: 초기화는 §2.1의 10단계이며 `push_loop` 입력 중 후보 순서·rhs S+0x200·폭·표시·PInf·반복·지원 flag는 push:enter 상태와 원본 규칙만으로 결정됨을 자연 6사례에서 비트/정확 일치로 확인했다. basic 값은 초기 factor의 dense rhs FTRAN이 필요하고, 품질 gate는 조건수 `116fd7`이 필요하다.
- 동등성: 이 조사 도구의 Python 재계산이 계약 범위에서 원본과 일치한다. **Rust 초기화 구현은 없음(미검증).** 자연 미관측: 기준 조정(재구성·-4/-5 생략), Markowitz 강화·`061bfe`, 수치 카운터 증가, 품질 retry, 초기 factor 반환 ≠ 0, first=0, slack -2(직전 xmm0), -5, quick 정렬 경로, 할당 실패.
- 후속 등록(`docs/14_sprint_plan.md`):
  - **M3-04c-1a 구현**: Rust `push_init`(단계 2–7·10, 기준 조정 경로 포함 C 그대로)을 `SerialBackend`로 만들고 `push_loop`과 이어 push:enter부터 종료까지 6사례 비교(초기화 factor 입력 대신 push:enter header로 재분해). 조건수는 원본 condition 입력 또는 1c 전까지 외부 값으로 두고 품질 gate의 판정만 비교한다.
  - **M3-04c-1b 오라클**: 초기화 중간 경계(`09cf9b` 출력, `089d8f` 잔차, Markowitz 판단 값, `12f282` 입출력, param_1, 첫 스캔 셈)와 미관측 분기(기준 조정, Markowitz 강화, 품질 retry, slack -2, -5) 자연 탐색 + 원본 명령 합성.
  - **M3-04c-1c 조사·구현**: 조건수 추정 `116fd7`(3,024 byte) 이식과 원본 비교.
- branch `work/m3-04c-1`, worktree `../rsolver-m3-04c-1`, owner `claude-m3-04c1-20261112`. 최종 commit/병합은 main `Merge M3-04c-1` 이력을 따른다. 미커밋 선행 변경 없음.
- 갱신: `docs/11_progress.md`·`12_roadmap_diagram.md`·`14_sprint_plan.md`·`m3/sprint_design.md`·`handoff/M3.md`.
- 교훈: `docs/lessons/original.md`(slack -2 직전 xmm0, 재구성의 -4/-5 생략, 초기 재분해 뒤 기저 재번호), `tooling.md`(기존 캡처의 진입·첫 공급 쌍을 조사 대조에 쓰는 방법과 vstat 기저 번호 함정).
- 재실행: `python3 tools/push_init_survey.py --check && python3 tools/push_init_survey.py --self-test`.
