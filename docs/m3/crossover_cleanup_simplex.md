# M3-04f-4c crossover cleanup simplex 본체 조사 검증 계약과 결과

작성 기준은 [스프린트 검증 기준](../15_sprint_verification.md)이다. 선행: [04f-2 cleanup 진입](crossover_cleanup_entry.md), [04f-3 Rust cleanup 진입](crossover_cleanup_rust.md), [04f-4a 쌍대값](crossover_dual_values.md), [04f-4b 연결](crossover_dual_link.md). M1 simplex 명세: [primal](../m1/m1_primal_spec.md), [04 단체법](../04_simplex.md).

## 1. 범위와 주장

- 작업 종류: **조사**(읽기 전용 원본 관측 포함). Rust 구현은 하지 않는다. 사용자 지시 "다음 M3 스프린트 진행해 너가 작업자야"로 이 세션이 직접 작업한다(작업자 선택 질문 생략).
- 조사 질문.
  1. `13f716`이 부른 cleanup `0cda5b`는 어떤 경로를 타는가: 진입 직후 "이미 풀린 상태" 조기 반환(C 142–160: spx 존재, model+100 = spx+0xf0, alg = spx+0x358, 상태 2–5, env+0x2454 / `12dd49`), setup `02f7e1`, 웜스타트 `0325b4`·`194efc`, 엔진 분기(primal `12e6ec`→`0478d3`, dual `12ed7c`→`06268b`, sifting `12e738`), 종료 뒤 검사(`2e9ae`·`2e315`·`12f282`).
  2. 자연 사례에서 cleanup이 실제로 하는 일의 양: simplex 반복 수(spx+0x2a0 증분), 재분해(`036ade`)·FT 갱신(`05b257`) 수, primal↔dual 전환, 진입 basis(S+0x168) 대비 최종 basis 변화.
  3. cleanup이 crossover workspace에서 이어받는 입력 상태(factor·x·d·섭동·pricing 가중·허용오차·카운터)와 cleanup 시작에서 다시 만드는 상태를, 실제로 탄 경로의 C·asm으로 특정한다.
  4. M1 Rust simplex(`crates/lp/src/simplex.rs`: dual `06268b`, primal `0478d3`, 외곽 `drive`)를 이 경로에 재사용할 수 있는 범위와 빠진 것(상태 import, 이어받는 factor, 조기 반환 등).
  5. 최종 VBasis/CBasis 비교 경계와 후속 구현 번호.
- 포함: double(S+0x28 = 0), Threads = 1, Presolve = 0, Method = 2, 04f-2와 같은 사례 집합(Netlib 소형 45 × Crossover {−1, 1, 2, 3, 4}, 특수 자연 arms·retry·interior × 5값). mode 1(arms·retry) 사례는 04f-2처럼 cleanup 선택 이후만 본다.
- 제외와 후속: 합성 편집 사례(04f-2 `synthetic.json`, 이번에는 자연 경로만 조사), x87, Crossover 5·threads ≥ 2, 재시작(04g), Rust 구현과 동등성 판정.
- 선행 commit: main `8a12121`(Merge M3-04f-4b). 미커밋 선행 변경 없음(main의 미추적 `.claude/settings.local.json`은 무관).

## 2. 원본과 상태 경계

- Gurobi 13.0.3 `libgurobi130.so` SHA-256 `8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c`. 주소는 ELF file offset.
- `0cda5b` = `../re/decomp/c/00153cc0_PRIVATE00000000000cda5b.c`(2,113줄), `02f7e1` = `000d06f0`, `0478d3` = `000e3db0`, `06268b` = `000f6a00`, `13f716` = `001bc5d0`(C 1523–1690 호출 지점).
- 시작 경계: `0cda5b` 진입(반환 주소가 `13f716` 안). 종료 경계: 그 `0cda5b` 반환.
- 관측(읽기 전용 훅, 쓰기 없음, `tools/xo_simplex_agent.js`): 진입 인자·model+100·spx+0xf0·spx+0x358·spx+0x35c·spx+0x2a0·spx+0x2a8·spx+0x20·spx+0x180·spx+0x4d8·spx+0x450·spx+0x294·spx+0x2f8·spx+0x2d0·spx+0x3ec·spx+0x340·S+0x380/0x388·factor 포인터·env+0x2454·env+0x2438·D+0x324·S+0x168(n+m). 내부 호출의 진입·반환 사건(함수별 cleanup당 40개까지, 그 뒤는 횟수): `02f7e1` 0xd06f0, `0325b4` 0xd3150, `194efc` 0x1fcd50, `12e6ec` 0x1bbb20, `12ed7c` 0x1bbd80, `12e738` 0x1bbb40, `12e790` 0x1bbb60, `0478d3` 0xe3db0, `06268b` 0xf6a00, `2e9ae` 0xcff40, `2e315` 0xcfbc0, `12f282` 0x1bbf80, `2d1de` 0xcec20, `2d3b2` 0xcedb0, `2de43` 0xcf8d0, `12ddac` 0x1bb730, `af361` 0x137ce0, `229999` 0x1387d0, `b16d3` 0x1415d0, `07373e` 0x105f20, `06899a` 0xf94d0, `b44ce` 0x142d70, `b3a19` 0x145520, `b2cdf` 0x142350, `b2fc2` 0x142500, `048d5f` 0xe45c0, `028c93` 0xc4fe0. 횟수만: `036ade` 0xd4190, `05b257` 0xf15c0, `12dd49` 0x1bb6f0, `08b129` 0x11f430.

## 3. 비교 계약(조사 산출물)

| 필드·사건 | 의미 | 판정 규칙 | 캡처 |
|---|---|---|---|
| 경로 사건열 | cleanup 안 호출 순서·횟수·반환 | 사례당 독립 3회 정확 동일 | 훅 |
| 반복·재분해 수 | spx+0x2a0 증분, `036ade`·`05b257` 횟수 | 정확(3회 동일) | 훅 |
| basis 변화 | 진입 S+0x168 대 반환 S+0x168, 최종 VBasis/CBasis | 정확 | 훅·Python |
| 조사 답 | Q1–Q5 | C·asm 위치와 관측 사건으로 근거 연결 | 문서 |

- 원본 재현성: 사례당 독립 3회의 사건열이 같아야 한다. 다르면 첫 차이 필드를 기록하고 원인을 조사한다.
- 수치 비교는 하지 않는다(조사). 시간·work는 기록만 한다.
- 판정: 조사 완료는 Q1–Q5에 근거로 답하고 미확인 범위·후속 번호를 특정했을 때. 동등성은 **미검증**(Rust 구현 없음).

## 4. 사례

| 사례 | 실제/합성 | 노리는 분기 |
|---|---|---|
| Netlib 45 × Crossover −1·1·3 | 실제 | primal cleanup(basis 인자 없음, remaining = 0) |
| Netlib 45 × Crossover 2·4 | 실제 | dual cleanup |
| arms·retry × 5값 | 실제(mode 1) | remaining ≠ 0 primal + `0325b4` 설치, 자연 쌍대 검사 → dual |
| interior × 5값 | 실제 | 04f-2 특수 자연 |

## 5. 재실행 명령

- 캡처: `tools/hrun.sh --timeout 590000 -- 'python3 tools/xo_simplex_runs.py --reps 3'`(작업 worktree, Gurobi 한 프로세스).
- 검사·집계: `python3 tools/xo_simplex_check.py --controls --summary`, 판별력 `--self-test`.

## 6. 실행 결과

- 환경: x86-64 WSL2 Linux, Gurobi 13.0.3(위 SHA-256), Threads = 1·Seed = 0·OutputFlag = 0·Presolve = 0·Method = 2·LPWarmStart = 2(arms·retry는 04f-2와 같은 ScaleFlag = 0·PStart/DStart). 계약 `29a29c2`, agent SHA-256 앞 16자 `0e68d3792f62ff11`, capture `4a8203474a4f6c3c`. 자료 `bench/oracle/m3_xo_simplex/`(720 gz, 정렬한 파일 SHA-256 목록의 해시 `da9124b7227cdb43`), 검사 출력 `bench/validation/m3_xo_simplex/check.txt`·`summary.json`.
- 원본 실행: 정식 **720회 모두 성공**(Netlib 45 × Crossover 5값 × 3 = 675, arms·retry·interior × 5값 × 3 = 45). 그 전 탐색 4회(afiro·adlittle × Crossover 2·3, scratchpad)와 첫 agent로 시작한 정식 실행 약 150회는 primal 시작 `af361`·devex pass `07373e`를 관측하지 않아 중단하고 지웠다(기준 자료 아님). 정식 뒤 스케일 확인용 탐색 8회(afiro·scsd1·degen2·bandm × Crossover 2·3, agent 사본에 S+0xe0·0xe8·0xd8·0x178만 더함, scratchpad)를 했다. 총 원본 실행 약 882회.
- 재현성(`--controls`): 240사례 × 3회 사건열·Python 결과(Status·VBasis·CBasis·IterCount·BarIterCount·ObjVal 비트)가 같다. 첫 실행에서 208사례가 다르다고 나왔는데 차이는 모두 반환값을 쓰지 않는 함수의 rax였다: `048d5f`(252사건, 힙 포인터를 돌려주고 `06268b` C 176·431·920이 버림)와 `b16d3`(14사건, void). 이 두 함수의 반환만 'void'로 바꿔 비교한다(그 밖의 반환값 변조는 거부, 아래).
- 검사기 판별력(`--self-test`): run 3 변조 반복 수·재분해 횟수·사건 순서·최종 VBasis·사용하는 반환값(`02f7e1`)·모듈 안 반환 주소 6종이 모두 거부되고, 반환 vstat에서 기저 하나를 비기저로 바꾸면 분류(basis 집합 동일 여부)가 바뀐다.

### 6.1 조사 답

**Q1 경로.** 240사례 모두 조기 반환(`0cda5b` C 142–160)을 타지 않는다. 진입에서 model+100 = 2(barrier 결과)인데 spx+0xf0 = 1(crossover가 설정)이라 첫 비교(model+100 = spx+0xf0)가 거짓이다(spx+0x358도 2로 alg와 다르다). 이어 `02f7e1`이 spx가 이미 있는 경로(C 131–570: 파라미터·허용오차·카운터를 다시 쓰고 spx+0xf0 = 1, spx+0x2a0·0x2a8 = 0, spx+0x358 = alg, spx+0x35c = PRIMDUALSWITCH 값)를 거쳐 C 569에서 0을 반환한다. basis(S+0x168)·header·x·factor 포인터는 그대로 남는다. basis 인자가 있으면(arms·retry primal) `0325b4`, 없으면 바로 엔진이다. 엔진 분기는 C 1628–1677: sifting은 0회, primal `12e6ec`→`0478d3`, dual `12ed7c`→`06268b`. 끝나면 `2e9ae`(비스케일 검사)가 240회 모두 1번 불리고 재풀이 없이 0을 반환한다. `2e315`·`12f282`·quad 전환(`2d1de`·`2d3b2`)은 0회. 모든 cleanup은 반환 0, spx+0xf0 = 2, model+100 = 2.

| 경로(깊이 ≤ 1) | 사례 |
|---|---|
| `02f7e1` → `12e6ec`·`0478d3` → `2e9ae` | 136 |
| `02f7e1` → `12ed7c`·`06268b` → `2e9ae` | 95 |
| `02f7e1` → `0325b4` → `0478d3` → `06268b` → `2e9ae` | 5 (arms −1·1·3, retry 1·3) |
| `02f7e1` → `0478d3` → `06268b` → `2e9ae` | 2 (scsd1 −1·1) |
| `02f7e1` → `06268b` → `0478d3` → `2e9ae` | 2 (arms 2·4) |

**Q2 일의 양.** Netlib 45 × 5값 = 225사례와 interior 5사례, 합 **230사례는 cleanup 반복이 0**(spx+0x2a0 = 0)이고 **기저 집합과 비기저 상태가 바뀌지 않는다**. 반환 S+0x168이 진입과 다른 곳은 모두 기저 변수의 위치 번호뿐이다(재분해 순열). 사건은 다음 셋이다.
- primal 0회(136): `af361`(시작) → `229999`(웜 시작 상태) → `b2cdf`(재분해 판단, `05b257`→`036ade` 1회) → `b2fc2` → pass 1회(`07373e` devex 또는 `06899a` partial) → `b3a19`가 1을 반환하며 spx+0xf0 = 2. `b44ce`(반복) 0회.
- dual 0회(92): `06268b` → `048d5f`(섭동 처리) 1회 → `028c93` 1회가 이탈 행을 찾지 못하고 spx+0xf0 = 2. 그 사이 재분해(`05b257`→`036ade`) 1회(순서는 횟수만 셌다).
- scsd1 Crossover −1·1(2): primal이 `b16d3` 시작 shift(spx+0x180 = 1)를 넣은 뒤 dual로 넘기고 dual이 반복 0으로 끝난다(재분해 2회).
반복이 있는 사례는 mode 1(PStart/DStart) arms·retry 10개뿐이다(spx+0x2a0 증분 3–20, 재분해 2–3회, primal ↔ dual 전환 1회, 기저 1–5개 교체).

**최종 VBasis/CBasis.** 240사례 모두 cleanup 반환 S+0x168과 Python VBasis/CBasis가 다음 대응으로 같다(불일치 0): 기저(≥ 0) → 0, −1·−2 그대로, −4(고정 비기저) → −1 또는 −2. −4 37,520개 중 37,065개는 −1, 455개(35사례)는 −2로 보고됐다. −4의 하한/상한 보고 규칙은 basis 출력 단계(`0cda5b` 뒤 uncrush·`08b129`·`a83b2f` 쪽)에 있으며 이번에 확정하지 않았다. 반복 0인 230사례에서는 **최종 VBasis/CBasis = cleanup 진입 basis(= push 결과)**이고, cleanup은 판정(status 2)과 위치 순열만 바꾼다.

**Q3 이어받는 상태와 다시 만드는 상태.** C와 관측으로 정리하면 다음과 같다.
- 이어받는 것: S+0x168 vstat·header(위치), push가 남긴 x(비기저 위치값)와 superbasic 목록(S+0x440·0x448·0x450, basis 인자 경로), 스케일된 모델(탐색 8회 모두 열·행 스케일 포인터 있음, bandm 목적 스케일 2.0, 정식 자료 밖), spx+0x180 = 1(crossover가 설정, primal `b16d3` 시작 shift 조건), spx+0x4d8 = 1(웜 시작 표시), spx+0x178(재시작 횟수, 자료에서는 0).
- 다시 만드는 것: `02f7e1`의 파라미터·카운터(위), factor(primal은 `af361` C 84가 spx+0x294 = 1을 두어 첫 `b2cdf`에서 재분해, 230사례 모두 `036ade` ≥ 1), primal은 `229999`가 x_B·행 표식 등 시작 상태와 devex 참조(비기저 1.0, 기저 0, `af361` C 309–356)를, dual은 시작(`06268b` C 141–144의 `12f4f3`, 이번 훅 밖)과 `048d5f`를 거친다.
- primal pricing(spx+0x3ec, `af361` C 158–185): primal 시작 145회 모두 "superbasic(S+0x450) > 0 → 0, 아니면 spx+0x4d8 = 1·spx+0x178 = 0에서 3m ≤ n → 0(partial), 그 밖 → 2(devex)"와 같다(2: 125회, 0: 20회 = 3m ≤ n 15 + superbasic 5). 마지막 엔진이 dual인 cleanup 102사례는 반환 spx+0x3ec가 모두 3(SE)이다(마지막이 primal인 138사례는 2 또는 0).

**Q4 M1 Rust simplex 재사용.** `crates/lp/src/simplex.rs`는 Gurobi 궤적과 같은 dual(`06268b`, Method 1 Netlib pathcmp 27/64)과 근사 primal(`0478d3` 명세 기반, devex·SE 없음, x_B를 값으로 보관 — 코드 주석의 shortcut)을 갖고, 웜 시작은 `Engine::solve_warm`(VBasis/CBasis 코드 → dual, superbasic 0 처리)뿐이다. 이 cleanup 경로에 바로 쓸 수 없는 점:
1. crossover workspace 상태 입력이 없다. cleanup 진입은 위치까지 정해진 header, push x, superbasic 목록, spx+0x180 = 1, spx+0x4d8 = 1, 스케일된 모델이고, `solve_warm`은 코드만 받아 basis 순서·위치·x를 다시 만든다.
2. primal 시작 `af361`·`229999`·devex pass `07373e`·`b3a19` 최적 판정과 `b16d3` shift가 Rust primal에 없다(M1 primal은 궤적 미검증).
3. dual 쪽은 `12f4f3` 웜 시작·`048d5f`의 웜 처리·`028c93` 한 번의 최적 판정이 필요하다. 이 판단 자체는 M1 dual 경로와 같은 함수이나, crossover 진입 상태(섭동 없음, 이미 쌍대 가능)에서의 동작은 비교한 적이 없다.
4. 재분해는 push 쪽 Rust(`crossover_refactor`, 21g–21j)와 M1 `Lu`가 같은 `036ade`를 옮겼으므로 재사용할 수 있다. 위치 순열 비교도 이 경로로 할 수 있다.
반복 0 경로는 "재분해 → x_B·d 계산 → 최적 판정 → 상태 2"뿐이라 M1 반복 궤적 일치를 기다리지 않아도 된다. 반복이 있는 mode 1 사례는 M1 primal devex 궤적과 primal ↔ dual 전환이 선행이다.

**Q5 비교 경계와 후속 번호.**
- **M3-04f-5a**(오라클): cleanup 반복 0 경로의 입력·판정 캡처. `0cda5b` 진입 상태(vstat·header·x·superbasic·bound·비용·스케일 모델·spx 허용오차·pricing 입력)와 primal `af361`→`b3a19`(x_B, 행 표식, 쌍대 위반 판정, pricing 모드, 재분해 순열), dual `12f4f3`→`028c93`(섭동 처리 결과, CHUZR 판정), `2e9ae`(비스케일 x·d 재검사)를 3회 재현으로 고정한다.
- **M3-04f-5b**(구현): 04f-5a 경계를 Rust로 옮겨 230사례의 상태 2·위치 순열·최종 VBasis/CBasis(−4 보고 규칙 포함)를 비교한다. 진입 상태는 원본 입력(dual push가 Rust에 없으므로). −4 → −1/−2 보고 규칙은 5a에서 출처 함수를 특정한다.
- **M3-04f-6**(조사·구현, M1 primal devex 궤적 선행): 반복이 있는 cleanup(mode 1 arms·retry 10사례, `b16d3` shift, primal ↔ dual 전환, superbasic pricing 0).
- 전체 E2E(push → cleanup → 최종 VBasis): 모든 Crossover 값의 자연 경로가 dual push(`13b3d1`)를 거치므로 04d·04e 뒤. 반복 0 범위에서는 최종 VBasis/CBasis가 cleanup 진입 basis와 같으므로, dual push 이식과 04f-5b가 끝나면 Netlib 225사례의 최종 basis 비교가 가능해진다.

### 6.2 미관측·미확인

- 조기 반환 경로(C 142–160), sifting, quad 전환, `2e9ae` 재풀이, `2e315`·`12f282`, 수치 실패(0x2732)·−1003·−1004 이외 상태는 자연 0회.
- 스케일 확인은 탐색 8회(정식 자료 밖)뿐이다. crossover workspace의 스케일이 처음 `02f7e1`(`07d3ce`)의 결과와 같은지 확인하지 않았다.
- `229999`·`b3a19`·`028c93`·`048d5f` 내부 수치(x_B·d·섭동량)는 관측하지 않았다(04f-5a).
- −4의 VBasis 보고(−1/−2) 규칙의 원본 위치.
- Crossover 5·threads ≥ 2, x87, 재시작(local_98), 합성 편집 사례.

## 7. 판정과 인수인계

- 작업 수행 상태: **완료(조사)**. Q1–Q5에 C 위치와 원본 720회 관측으로 답했고 미확인 범위와 후속 번호(04f-5a·5b·6)를 정했다. 동등성: Rust 구현이 없으므로 **미검증**.
- 확인한 범위: double·Threads = 1·Presolve = 0·Method = 2에서 Netlib 45 × Crossover 5값과 arms·retry·interior × 5값의 cleanup `0cda5b` 경로·반복 수·재분해 수·basis 변화·최종 VBasis/CBasis 대응(240사례 × 3회 재현).
- 후속: **M3-04f-5a**(반복 0 cleanup 판정 오라클), **M3-04f-5b**(Rust 이식), **M3-04f-6**(반복 있는 cleanup), 04d·04e(dual push). 재검사 `python3 tools/xo_simplex_check.py --controls --summary --self-test`. 원본 재캡처 `python3 tools/xo_simplex_runs.py --reps 3 && python3 tools/xo_simplex_runs.py --cases --reps 3`(새 디렉터리에서, 기존 출력은 건너뜀).
- branch `work/m3-04f-4c`, worktree `../rsolver-m3-04f-4c`. Rust·shared 코드 변경 없음(도구·자료·문서만).
- 교훈: `docs/lessons/original.md`(crossover cleanup simplex 본체), `docs/lessons/tooling.md`(반환값을 쓰지 않는 함수의 rax, 꼬리 호출 dispatcher의 반환 주소).
