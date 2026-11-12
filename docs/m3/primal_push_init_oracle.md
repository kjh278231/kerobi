# M3-04c-1b primal push 초기화 중간 경계·미관측 분기 오라클: 검증 계약과 결과

작성 기준은 [스프린트 검증 기준](../15_sprint_verification.md)과 [계약 양식](../templates/sprint_acceptance.md)이다. §1–§5는 첫 캡처 전에 썼다.

## 1. 범위와 주장

- 작업 종류: **오라클 구축**. 기준 main `d79b70b`(M3-04c-1a 병합), branch `work/m3-04c-1b`, worktree `../rsolver-m3-04c-1b`. 선행 미커밋 변경 없음(main의 미추적 `.claude/settings.local.json`은 무관, 보존). 사용자의 "다음 M3 스프린트 진행해 너가 작업자야" 지시로 이 세션이 작업자로 수행한다(작업자 선택 질문 생략). owner `claude-m3-04c1b-20261113`(M3-04c-1b·shared).
- 질문과 완료할 동작: double primal push `134a87` 초기화([조사](primal_push_init.md) §2.1 단계 2–10)의 **중간 경계**를 원본에서 읽기 전용으로 캡처하고, 자연 미관측 분기를 자연 탐색과 **원본 명령 합성**(정상 실행 중 push 입력 상태 일부를 바꾸고 이후 원본 명령이 그대로 실행)으로 실행시켜 독립 3회 재현과 검사기 판별력을 확보한다.
  - 중간 경계: τ_in(param_1, `0x1c0239`에서 쓰는 `0x1ee0(%rsp)`), 이전 PInf S+0x120, 첫 스캔 셈 3개(`0x1e88`·`0x1e80`·`0x1e78(%rsp)`)·후보 수(`0x1fb0`), 초기 재분해 반환(`05b257` caller `0x1c07c6`·`0x1c07ea`), `09cf9b` dense FTRAN 입력·출력(caller `0x1c11d1`), `089d8f` 잔차 입력·출력(caller `0x1c1287`·`0x1c14a2`), Markowitz 판단(강화 호출 `0x1c151c`의 새 값 `0x1e70(%rsp)`·잔차 `0x1eb8`, 강화 전후 S+0x390·S+0x3c8·F+0x188·환경 flag), `12f282` 호출 전후 *condition과 PInf(`(%rsp)`)·위반 수(`0x1f28`), 기준 조정 gate(`0x1c184d`)의 상태, push 반환과 retry.
  - 분기: 기준 조정과 재구성(-4·-5 생략), Markowitz 강화(`12f5d1`→`061bfe`), 품질 retry, slack -2(직전 xmm0), 열 -5.
- 입력 전제: Threads=1, Seed=0, Presolve=0, Method=2, LPWarmStart=2. 정식 사례는 Crossover=3(primal-first, first=1). 탐색은 Crossover 3·1 두 설정.
- 제외와 후속: Rust 구현 변경(이 자료로 1a의 외부 값 경로를 비교하는 일은 **M3-04c-1d**로 등록), 조건수 `116fd7` 이식(1c, 이번에는 입출력 기록만), x87(`1faa63`·`23e604`), batch(threads>1), 할당 실패, 해 설치(04f), 루프(24) 재검증. 합성 사례는 원본이 바뀐 입력에 대해 실제로 한 동작의 증거이며 자연 모델에서 그 분기가 일어난다는 증거가 아니다.

## 2. 원본과 상태 경계

- Gurobi 13.0.3 `libgurobi130.so`, SHA-256 `8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c`. 주소는 ELF file offset(Ghidra = +0x100000), 훅은 모듈 base + offset.
- 원본: `../re/decomp/c/001c0070_PRIVATE0000000000134a87.c` C 150–1142, asm `python3 tools/dc.py 134a87 --asm --max 100000`. 훅을 붙이는 명령 경계는 모두 직선 흐름 안이며 그 명령과 다음 명령을 가리키는 jump가 함수 안에 없음을 확인했다(`0x1c0242`, `0x1c11cc`, `0x1c151c`, `0x1c1521`, `0x1c17b3`, `0x1c17b8`, `0x1c184d`).
  - `0x1c0239 movsd %xmm0,0x1ee0(%rsp)`: τ_in 저장. `0x1c0242`에서 슬롯을 읽는다(Frida x64 문맥에는 xmm이 없다, lessons/tooling).
  - `0x1c11cc call 09cf9b(F=0x2048(%rsp), in=S+0x228 구조(count=-1), out=r15=S+0x220 구조, 0, work)`. 값 배열은 두 구조의 +0x10. 함수 진입 `0x1205d0`에 붙이고 반환 주소 `0x1c11d1`인 호출만 기록(`116fd7`도 `09cf9b`를 부른다).
  - `089d8f@0x115d60(S, x_B, out=S+0x1f8 값)`: header 순서로 `out[row] += a·x`(slack은 `out[i] += x`). 반환 주소 `0x1c1287`·`0x1c14a2`만 기록.
  - `0x1c14c8–0x1c155f`: 후보 값 0.5/0.25/1e-4와 S+0x390 비교, 강화면 `0x1c151c call 12f5d1(S)` 앞에서 새 값 `0x1e70(%rsp)`·잔차 `0x1eb8(%rsp)`, 잔차 > 1e-6이면 S+0x178++.
  - `0x1c17b3 call 12f282(S+0x4b8, param_8, param_12)` 앞에서 `(%rsp)`=PInf, `0x1f28`=위반 수, *param_8. `0x1c17b8`에서 *param_8(=condition).
  - `0x1c184d`: gate(`0x1c17cc–0x1c1847`)를 지나 기준 조정 판정 직전. `0x1ec0` 조정 flag, `0x1fb0` 후보 수, `0x1e88`(0.1셈)·`0x1e80`(0.001셈)·`0x1e78`(0.01셈), `0x1ee0` τ_in(조정 뒤에는 곱해진 값), `0x2070` τ(S+0x380), S+0x120.
- 시작: `134a87` 진입(`push:enter`). 종료: 첫 `supply:enter`(`25615c` 진입) 또는 그 전 push 반환(`push:leave`, retry 경로). 이후 사건은 기록하지 않는다(루프는 24 범위).
- 입력 상태: 21h와 같은 push:enter 상태(모델 CSC·bound·rhs·vstat·header·x·slack·허용오차·카운터·first·attempt·model_status) + S+0x120·S+0x3c8·F+0x188·환경 flag(S+0x4b8→+0x100→+0x2c54)·S+0x1f8 scratch 값.
- 출력·변경 상태: 위 중간 경계 사건열, 첫 supply:enter의 21h `snap(true)` 상태와 후보·지원, push:leave의 반환·retry·condition.
- 합성: agent 설정 `edits`로 push 진입(첫 호출만) 또는 `0x1c11cc`(첫 FTRAN 직전)에서 지정 필드만 쓴다(S+0x120, S+0x390, vstat 한 칸, x/slack 한 칸). 쓰기 전·후 값을 사건에 남긴다. 함수 호출·반환값·제어 흐름은 바꾸지 않는다.

## 3. 비교 계약

| 필드·사건 | 의미 | 규칙·근거 | 캡처·재구성 |
|---|---|---|---|
| 재현성 | 모든 사례의 사건열 | 3회 정확(pid·run_id·도구 출력 문자열 제외) | 독립 프로세스 3회 |
| 21h 비간섭 | 자연 6사례 push:enter·첫 supply:enter | 21h 캡처와 같은 필드가 JSON 전체 일치 | 투영 비교 |
| τ_in·셈 | `0x1ee0`, 0.1·0.01·0.001셈, 후보 수 | 셈과 후보 수는 push:enter 입력과 캡처 τ_in으로 C 401–596 규칙(`x−lb`·`ub−x`·slack 값 > τ_in·c, -1/-2만)을 다시 세어 정확 일치 | Python 재계산 |
| FTRAN 입력 | `09cf9b` in 값(m) | 비트. 조정 없음: push:enter로 계산한 rhs(조사 `build`)와 같고 첫 supply:enter S+0x200과 같다. 조정 뒤: §2.1 9의 재구성 규칙으로 계산한 값과 같다 | Python 재계산 |
| FTRAN 출력 | `09cf9b` out 값(m) | 비트. basic(S+0x1e8)[p] = out[p] − lb(header[p]) (lb > −I일 때) 또는 out[p]. 이후 089d8f 입력과 같은 배열 | 캡처 사이 일관성 |
| 잔차 | `089d8f` 전 scratch, 후 출력 | 전은 전부 +0.0 비트, 후는 header 순서 `out[r] = a·x + out[r]`(mulsd·addsd, FMA 없음)의 Python 재계산과 비트 | Python 재계산 |
| Markowitz 판단 | max\|rhs\|·max\|x_B\|·max\|rhs−Bx_B\| → 후보 값, 강화 여부, S+0x390, S+0x178 | 판단은 정확, 값은 비트. 강화 사건이 있으면 새 값 `0x1e70`과 잔차 `0x1eb8`이 Python 값과 비트. 강화 전후 S+0x3c8·F+0x188은 `061bfe` 규칙(환경 flag < 1이면 `max(50, trunc(min(0.9·S+0x3c8, trunc(√m))))`가 작을 때만)과 정확 | Python 재계산 |
| 품질 gate | condition, PInf, 위반 수, retry | condition은 기록만(1c). PInf·위반 수는 Python 분류(조사 `classify`, 원본 basic 입력)와 비트·정확. gate 결과(진행/retry)는 C 867–896 식을 캡처 condition으로 계산한 값과 일치 | Python 재계산 |
| 조정 gate·재구성 | flag, τ', 승격 vstat, 재정렬 후보 | gate 판단 정확, τ' 비트, 재구성 뒤 FTRAN 입력·첫 supply:enter의 vstat·후보 순서·rhs 정확/비트 | Python 재계산 |
| 합성 분기 실행 증거 | 사례가 노린 분기 | 사례마다 결과 보기 전에 정한 필수 사건(아래 §4)이 3회 모두 있어야 함. 없으면 실패 | 사건 검사 |

- 수치 오차 허용 없음. ±0 포함 f64 비트. NaN·무한대는 sentinel I로만 다룬다.
- 시간·work·병렬: Threads=1, batch 0만. work 비교 안 함.
- 재현성: 같은 바이너리·입력·설정·agent로 사례마다 독립 3회. 탐색 실행은 기준 자료가 아니며 분기 발견용이다.

## 4. 사례와 원본 자료

| 사례 | 실제 / 합성 | 노리는 분기 | 입력 | 필수 증거 |
|---|---|---|---|---|
| arms·retry·interior·adlittle·scsd1·capri | 실제 | 중간 경계 전체(조정·강화 0회 기대) | 21h와 같은 모델·설정 | 사례마다 tau·lu·ftran·residual·gate·첫 supply 사건 |
| 자연 탐색 | 실제 | 조정·강화·retry·slack -2·-5·first=0·잔차 >1e-6 | Netlib 소형 45(라이선스 크기 한도 안) × Crossover 3·1, 저장소 LP | 발견된 분기는 해당 모델을 정식 사례로 3회 캡처 |
| adjust | 합성 | 기준 조정 1회와 재구성 | 자연 사례 하나에 S+0x120 := 0 및 PInf>1이 되도록 -3 후보 x 한 칸 변경(탐색으로 고정) | 조정 gate flag 0→1, FTRAN 2회, -1/-2→-3 승격 |
| markowitz | 합성 | 강화(`12f5d1`→`061bfe`) | 첫 FTRAN 직전 S+0x390 := 1e-5 | 강화 사건, S+0x390 = 후보 값 |
| quality | 합성 | 품질 retry(P > 1e10, model_status=2) | -3 후보 x 한 칸을 크게 변경 | push:leave retry=1, supply 없음 |
| slack-m2 | 합성 | slack -2 = 직전 xmm0 | 등식 행(lb=ub=0) slack -1 → -2, 직전 xmm0 ≠ 0인 위치 | FTRAN 입력이 stale 규칙과 같고 0 규칙과 다름 |
| col-m5 | 합성 | 열 -5 = ub | -4 열(lb=ub≠0) → -5 | FTRAN 입력이 ub 규칙과 같음 |

- 자연 탐색에서 분기가 나오면 그 모델을 같은 이름 접두 `natural-`로 정식 사례에 더하고 합성 사례도 유지한다.
- 적용하지 않는 범주: 할당 실패·batch·x87(원본 설정으로 도달 불가 또는 범위 밖), first=0과 attempt≠0은 탐색에서 나오면 기록만 하고 계약 판정은 하지 않는다(이 경우 후속 번호로 등록).

## 5. 재실행 명령

- 원본 캡처: `tools/hrun.sh --timeout 580000 -- 'python3 tools/push_init_runs.py'`(Gurobi 한 프로세스씩, 기존 출력 덮어쓰기 거부). 탐색: `python3 tools/push_init_runs.py --survey <OUT>`.
- 검사: `python3 tools/push_init_check.py --controls --acceptance`(manifest 해시·3회·비간섭·재계산 규칙·필수 증거), `--self-test`(변조가 하나라도 통과하면 종료 1), `--write-manifest`(최초 작성).
- 회귀: `python3 tools/push_block_check.py --controls --acceptance`, `python3 tools/push_init_survey.py --check`, `python3 -m py_compile tools/push_init_check.py tools/push_init_runs.py`. Rust 변경이 없으므로 Cargo·clippy·확장·pytest는 실행하지 않는다.

## 6. 실행 결과

- 기준 main `d79b70b`, 계약 `519ee93`, 도구·탐색 `564e518`, 정식 캡처 `0f31888`, 검사기 `5f7d8fa`. x86-64 Linux(WSL2), Node Frida 17, Python 3.10. 바이너리 SHA는 매 캡처가 확인했다. manifest `bench/oracle/m3_push_init/manifest.json`(도구 3·캡처 45·`synthetic.json` 해시).
- **탐색**(`history/survey-x3/`, `survey-x1/`, 기준 자료 아님, 정식 agent 이전 판으로 실행): Netlib 소형 45모델 × Crossover 3·1 = 90회 모두 성공, push 호출 90회. 기준 조정·Markowitz 강화·품질 retry·수치 카운터 증가·초기 재분해 반환≠0·slack -2·열 -5·x87 **모두 0회**. 모든 첫 push에서 S+0x120(이전 PInf)이 4.6e-15–1.3e-7로 τ(1e-6)보다 작아 조정 gate의 이전 PInf 조건은 자연 상태에서 이미 참이고, 조정을 막는 것은 PInf ≤ 1이다(tuff만 PInf 3.1e-6 > 0). S+0x390은 모든 사례 0.1이라 강화에는 잔차 > 4e-9 또는 max\|x_B\| > 1e9·max\|rhs\|가 필요한데 자연 0회다. **Crossover=1은 45모델 모두 dual 뒤 first=0 push**를 불렀다 → `adlittle_x1`을 자연 사례로 추가(계약 변경, 아래).
- **합성 설계**: 편집 값은 21h 진입 상태와 numpy 기저 풀이로 골랐다(arms x[13]=50 → PInf 42 예상). 판정은 원본 캡처로만 한다. 필수 증거(`require`)는 탐색 캡처 전에 `synthetic.json`에 적었다.
- **원본 3회**: 15사례 × 3회 = 45회 모두 성공. pid·run_id·stdout을 뺀 JSON 전체(사건 251개/회)가 3회 같다. 4사례(adjust·quality·adlittle_x1·capri)는 4회째 독립 재캡처도 저장 run1과 같다.
- **21h 비간섭**: 자연 6사례의 push:enter·첫 supply:enter가 21h 캡처와 JSON 전체 일치(21h의 `step` 필드만 제거).
- **재계산 비교**(`python3 tools/push_init_check.py --controls --acceptance`, [출력](../../bench/validation/m3_push_init_oracle/check.txt)): **15사례·17 push 호출 전부 첫 실행부터 통과**. 실패·skip 0. 비교한 것: 편집 기록=명세, S+0x1f8 진입 0, 초기 재분해 요청 10·반환, FTRAN 입력(라운드마다 m개 비트), FTRAN 출력=잔차 입력, 잔차 scratch +0, `089d8f` 출력(m개 비트), 세 최댓값으로 정한 Markowitz 후보·강화 여부·새 값·잔차 슬롯(비트), `061bfe` 한도, S+0x178, gate 직전 PInf·위반 수(비트·정확), condition으로 계산한 gate 결과(진행/retry 종류), 조정 gate의 flag·후보 수·τ(조정 뒤 τ' 비트)·셈 3개·S+0x380·S+0x120, 조정 여부, 재구성 rhs·승격 vstat·후보 순서, 첫 supply:enter의 후보 순서·총수·지원·비기저 vstat·header·S+0x200·basic·폭·표시·S+0x120·반복·S+0x178·S+0x390.

| 사례 | 종류 | 관측 분기(검사기 판정) | 주요 값 |
|---|---|---|---|
| arms·retry·interior·adlittle·scsd1·capri | 자연 | 조정·강화·retry 없음 | τ_in = 1e-6(=S+0x380), 셈 adlittle [12,7,2]·scsd1 [2,0,0]·나머지 0, condition arms·retry·interior 1.0, adlittle 313.6, scsd1 54.6, capri 6196 |
| adlittle_x1 | 자연(Crossover=1) | first=0: 초기 재분해·조건수 호출 없음, gate는 진입 *param_8(505.6) 사용 | 후보 16 |
| adjust | 합성(arms x[13]=50, x[1]=0.5) | 조정 1회(셈 [1,1,1] < 23 → τ' = τ·0.001 = 1e-9), -1 승격, 재구성에서 -4 열(27, lb=ub=2) 생략, 2라운드 | PInf 42 → 44.5, 후보 23 → 24 |
| mk_low | 합성(FTRAN 직전 S+0x390=1e-5) | 후보 1e-4 > 1e-5 → 강화, 한도 불변(S+0x3c8=20 < 50) | S+0x390 → 1e-4 |
| mk_limit | 합성(+ S+0x3c8=200) | 강화 + `061bfe` 한도 200 → 50(S+0x3c8·F+0x188) | |
| quality | 합성(adlittle x[11]=1e12) | 잔차 1.1e-4 → 후보 0.5 강화, 잔차 > 1e-6 → S+0x178+1, P=PInf 2.6e12 > 1e10 → retry. caller가 attempt=1·S+0x390=0.9로 push를 다시 불러 정상 초기화 | |
| quality_pinf | 합성(arms x[13]=1e17, model_status 1) | P=1, PInf 2e17 > 1e15 → S+0x178+1·retry. 강화 없음. 재호출 attempt=1 | |
| slack_m2 | 합성(retry slack 3: -4→-2) | rhs₃ −= 직전 xmm0(= 열 17의 x 5), 0 규칙과 다름. PInf 5.3 → 조정·slack -2 승격 | |
| col_m5 | 합성(arms 열 27: -4→-5) | -5 = ub(2), 0 규칙과 다름 | |
| col_m5_ub | 합성(retry 열 3: -1→-5) | -5 = ub(10), 0·lb 규칙과 다름. PInf 10.3 → 조정(승격 없음, 재구성에서 -5 생략) | |

- **판별력**(`--self-test`, [기록](../../bench/validation/m3_push_init_oracle/self_test.txt)): 변조 **25종 전부 거부**(바이너리 해시 0, 사건 비우기, run2 1비트, 21h 비간섭 1비트, FTRAN 입력·출력, 잔차 출력·scratch, τ_in, 셈+1, 강화 사건 제거·새 값, 061bfe 한도, gate PInf, 조정 2회째 FTRAN 제거·flag·τ', retry 0, 후보 교환, 승격 되돌림, basic, 편집 제거·값, null-null, first 변경). 규칙 변이 14종 중 **필수 9종 거부**(slack -2=0, -5=0, -5=lb, 재구성 -4/-5 포함, τ' 항상 0.1배, 0.5 임계, 수치 카운터 임계, gate P 항상 PInf, 잔차 역순). **생존 5종**은 자료에 경계가 없는 경우다: -1 승격 strict(lb+τ'=x 없음), 0.25 임계(0.25 후보 사례 없음), `061bfe`의 √m 대신 m(두 값 모두 50 미만이라 하한 50), 셈 ≥(경계 같음 없음), lower shift 생략(기저 변수 lb가 모두 0 또는 −I).
- **회귀**([기록](../../bench/validation/m3_push_init_oracle/regression.txt)): `push_block_check.py --controls --acceptance`, `push_init_survey.py --check`, py_compile, `node --check` 두 도구, `git diff --check` 모두 종료 0. Rust 변경이 없어 Cargo·clippy·확장·pytest는 실행하지 않았다.
- **원본에서 새로 확인한 사실**: (1) 첫 push 진입 S+0x120은 이전 단계가 남긴 작은 PInf(4e-15 등)이다. (2) τ_in(param_1)은 자연 사례 모두 1e-6이며 S+0x380과 비트가 같다. (3) 품질 retry 뒤 caller는 attempt=1, S+0x390=0.9로 push를 다시 부르고 그때 x·slack은 편집 전 값이다(PInf 0). (4) 작은 사례의 condition은 1.0이다(1c 입력 확인 대상). (5) first=0 경로의 gate는 caller가 넘긴 condition(*param_8)을 그대로 쓴다.
- 계약 변경: (1) 탐색에서 first=0이 자연 발생해 `adlittle_x1`을 정식 사례로 추가하고 같은 규칙으로 판정했다(§4의 "기록만" 문구 대신). (2) `init:tau`의 `0x2070`은 `0x1c0284`에서야 쓰이는 슬롯이라 첫 탐색 뒤 제거했고, first=0일 때 `(%rsp)` PInf 슬롯은 미초기화라 기록하지 않는다. (3) `mk_limit`(S+0x3c8 편집), `quality_pinf`, `col_m5_ub`를 계약 표에 더했다(자연 S+0x3c8=20이라 한도 감소가 관측되지 않음, PInf>1e15 경로와 lb≠ub -5 판별). 기존 사례·규칙은 줄이지 않았다.
- 실행하지 않은 것: 조건수 `116fd7`의 내부(입출력만 기록), x87·batch, 할당 실패, Rust 비교(1d).

## 7. 판정과 인수인계

- 작업 수행 **완료**(오라클 구축). 범위: 자연 7사례(first=1 6·first=0 1)와 합성 8사례의 초기화 중간 경계·분기가 원본 독립 3회 재현되고, 검사기가 C 규칙 재계산과 비트/정확 일치를 확인하며 변조 25종·필수 변이 9종을 거부한다.
- 동등성: 이 오라클은 원본 동작의 기준 자료이며 **Rust 동등성은 미검증**이다(1a의 외부 값 경로 비교는 1d). 합성 사례는 바꾼 입력에 대한 원본 동작의 증거이며 자연 모델에서 그 분기가 일어난다는 증거가 아니다. 자연 Netlib 90회에서 조정·강화·retry·slack -2·-5는 0회였다.
- 남은 한계: 0.25 Markowitz 후보, `061bfe` 한도의 √m 항(하한 50에 가림), 승격·셈의 정확 경계, lb ≠ 0인 기저 변수 shift, cond > 1e14·cond > 1e30 gate, 초기 재분해 반환 ≠ 0·S+0xf0 ≤ −1000 retry, 할당 실패는 미관측이다.
- 후속: **M3-04c-1d** Rust `push_init`을 이 15사례(외부 값 τ_in·S+0x120·S+0x3c8·환경 flag·condition 주입)로 비교, **M3-04c-1c** 조건수 `116fd7`(이 캡처의 condition 값이 기준). 재실행 `python3 tools/push_init_check.py --controls --acceptance && python3 tools/push_init_check.py --self-test`.
- branch `work/m3-04c-1b`, worktree `../rsolver-m3-04c-1b`, owner `claude-m3-04c1b-20261113`. 최종 commit/병합은 main `Merge M3-04c-1b` 이력을 따른다. 미커밋 선행 변경 없음.
- 갱신: `docs/11_progress.md`·`12_roadmap_diagram.md`·`14_sprint_plan.md`·`handoff/M3.md`. 교훈 `docs/lessons/original.md`(S+0x120 진입값과 조정 gate, retry 뒤 재호출, first=0 condition), `tooling.md`(합성 편집 agent 패턴, 미초기화 스택 슬롯).
