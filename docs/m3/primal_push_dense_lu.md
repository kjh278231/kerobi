# M3-04c-21i push 재분해 `036ade` 조밀 전환(자체 조밀 LU) 검증 계약과 결과

작성 기준은 [스프린트 검증 기준](../15_sprint_verification.md)이다. 선행: [21g 재분해 생성](primal_push_refactor_rust.md), [21h 블록 단계](primal_push_block_rust.md), [04f-1 해 설치](primal_push_install.md)(grow7 분류된 실패).

## 1. 범위와 주장

- 작업 종류: **구현**(원본 경계 캡처 확장 포함). 기준 main `3e3ea69`(04f-1 병합), branch `work/m3-04c-21i`, worktree `../rsolver-m3-04c-21i`. 선행 미커밋 없음. main 미추적 `.claude/settings.local.json`은 무관하며 보존한다. 사용자의 "다음 M3 스프린트 진행해 너가 작업자야" 지시로 이 세션이 직접 작업한다(작업자 선택 질문 생략).
- 조사로 확정한 원인(계약 전 탐색, 판정 근거 아님): 04f-1의 grow7 실패는 push 8번째 재분해(`036ade` 진입 S+0x2c0 = 8)에서 커널 89단계부터 원본이 **조밀 전환**(LAB_001d88bf, 남은 n = 51)으로 넘어가기 때문이다. 탐색 21h agent 1회를 21h `--block` 하네스에 넣으면 재분해 8회 중 7회는 일치하고 이 재분해의 `x20[89]`(원본 59, Rust 90)에서 갈린다(xe4 = 2, 블록 상태 (8, 7, 0) → (8, 0)). 조사 probe(`tools/push_dense_survey*.{py,js}`)로 Netlib 104개 중 면허 크기 안 62개를 1회씩 돌려, push 재분해의 조밀 전환은 **grow7 1·grow22 2·perold 2·pilot4 3·stair 2 = 10회**(n 51–84), 모두 n < 1001이라 MKL `dgetrf`가 아닌 **자체 조밀 LU**(C:3968–4920, `0xdd1b2`)를 탔고 재시도·dgetrf는 0회였다. grow22에는 다른 미이식 경로(`local_70` 조밀 배열 Markowitz 모드 `0xd85b4`, 풀 부족 −1)가 있다.
- 완료할 동작: M1 `Lu`의 커널(`036ade` Markowitz)에 조밀 전환을 이식한다.
  1. **전환 판정**(C:1526–1626, asm `0xd6695`–): singleton이 없어 Markowitz 탐색에 들어가기 전, 남은 활성 n = (m − 단계) − 빈 열 수에 대해 `0.36·n² < 채움 추정`, `4m < n²`, `n > 50`, `m + n² < 2e9`이면 활성 열 개수 합 Σ(count+1)을 다시 세어 추정을 바꾸고, 그 값이 `0.325·n²`을 넘으면 조밀로 간다. 채움 추정은 기저 열 비영 수 합(C:296–329)에서 시작해 Markowitz 피벗마다 `(r−1)(c−1) − 2`(C:2023)를 더한다.
  2. **조밀 배열 구성**(C:3394–3470): 행은 행 개수 bucket 0부터 각 순환 목록 순서로 조밀 번호를 받고, 열은 활성 위치 오름차순으로 열 우선(lda = n) 배열에 남은 원소 값을 놓는다. 풀 여유 검사(C:3396–3398) 실패는 범위 밖.
  3. **자체 조밀 LU**(C:3968–4920, `0xdd1b2`–`0xde68c`): 4열 묶음 left-looking 갱신(SSE2 2-lane 누적, 주소 정렬에 따른 peel, 짧은 길이의 스칼라 경로), 행 안 최대 |a| 피벗(첫 최대 유지)과 열 목록·열 교환, 묶음 안 위치별 나눗셈·갱신, 마지막 4열 미만 경로. 피벗 |a| < ZERO의 재시도(LAB_001d8a7d, bVar23 = false)는 C 근거만 이식하고 자연 0회면 미검증으로 남긴다.
  4. **LU 뒤 변환**(LAB_001de68c, C:4921–) : 조밀 결과를 L 열·U 행과 피벗 순서(`036ade` 반환 순열)로 옮긴다. ZERO 버림 규칙 포함.
- 정렬: 자체 LU의 연산 순서가 조밀 배열 주소의 16바이트 정렬(=`param_15[1] + param_15[3]·8`, 풀 채움 F+… 의 짝홀)에 따라 바뀐다. Rust는 이 정렬을 입력 상태로 받아 같은 peel을 고른다(캡처 `mat_mod16`·`scratch_mod16`). 풀 채움과의 관계를 C로 확정하지 못하면 캡처값을 입력으로 쓰고 그 사실을 기록한다.
- 포함: 위 10회 push 재분해와 각 모델 push의 다른 재분해 전부(조밀 전환이 없는 재분해에서 Rust가 전환하지 않는지 포함), push 밖 조밀 LU 11회의 (A) 경계.
- **사전 명시 제외·예상 실패**: (i) grow22 push에서 `local_70` 모드나 풀 부족 −1이 먼저 나오는 재분해와 그 뒤(후속 **M3-04c-21j**). 비교기는 그 재분해를 예상 실패로 따로 센다. (ii) MKL `dgetrf`(n ≥ 1001이고 n² > m)·전치 경로, 1e99 피벗 상한(local_120), 빈 열이 있는 조밀 전환(local_f0 > 0), 풀 여유 부족, work(`param_19`)·x87. 자연 0회이면 C만 남긴다.
- M1 simplex: 조밀 전환은 M1 kernel의 shortcut이다. 이번 이식은 M3 push 생성 경로에 켜고, M1 `factor_tol`(블록 단계 OFF)은 **같은 커널을 쓰므로 함께 바뀐다**. M1 경로 영향은 workspace 회귀와 M1 pathcmp 기준선으로 확인하며, 결과가 바뀌면 그 사실을 기록한다(조밀 전환이 원본 동작이므로 바뀌는 쪽이 원본에 가깝지만 M1 궤적 검증은 M1 범위).
  - **계약 변경(구현 중, Rust 비교 전)**: 조밀 배열의 정렬이 풀 크기 F+0x158(`param_15[2]`)에서 나오는데 M1 `Lu`는 이 값을 모른다. 그래서 전환은 풀 크기를 받는 새 진입 `Lu::factor_dense`(push adapter `crossover_refactor::refactor`가 F+0x158로 호출)에서만 켜고, M1 `factor_tol`/`factor_block`은 지금처럼 끈 채 둔다(21h 블록 단계와 같은 결정). M1에서 켜기는 M1 풀 크기 수명 확인과 pathcmp 회귀가 필요한 M1 후속이다.

## 2. 원본과 상태 경계

- Gurobi 13.0.3 `libgurobi130.so` SHA-256 `8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c`. CPU 13th Gen Intel i7-13700F(WSL2). 자체 LU는 SSE2만 쓰므로 CPU 분기 없음(asm). MKL 경로는 CPU 분기(`mkl_lapack_ps_*_dgetrf_small`)가 있어 범위 밖.
- C: `../re/decomp/c/000d4190_PRIVATE0000000000036ade.c` C:296–329(채움 초기), C:799–840(루프 머리), C:1526–1626(전환 판정), C:2005–2030(Markowitz 피벗의 추정 갱신), C:3394–3470(조밀 배열), C:3968–4920(자체 LU), C:4921–(변환). asm `0xd6695`, `0xd88bf`, `0xd8a7d`, `0xdd1b2`, `0xde68c`(Ghidra 주소 −0x100000).
- 캡처 경계(`tools/push_dense_agent.js`): 21h agent 사본에서 push 재분해 경계 사건만 내보내고(push:enter CSC, factor:enter Markowitz, 재분해 lu:enter header·이탈·vstat, lu:refactor 036ade 진입·반환 순열, lu:block, 재분해 lu:leave factor 전체), 함수형 probe `dense:enter`(0xd88bf: n·단계 수), `dense:fill`(0xd8a7d), `dense:lu`(0xdd1b2: n·행렬 n²·열/행 목록·ZERO·첫 시도·정렬), `dense:done`(0xde68c: 행렬·목록)을 더한다. probe는 asm에서 명령 경계·재배치 창 안 착지점 0을 확인했다. 행렬·열 목록 포인터는 레지스터와 스택 슬롯이 같은지 agent가 검사한다.
- 시작 경계: push 재분해 `036ade` 진입(21h와 같은 입력: push 진입 CSC, 재분해 진입 header, ZERO, Markowitz, 블록 상태). (A)는 `0xdd1b2` 진입 상태.
- 종료 경계: `036ade` 반환 순열과 재분해 `lu:leave` factor 전체(21g 비교 대상과 같음). (A)는 `0xde68c` 진입 상태.

## 3. 비교 계약

| 대상 | 규칙 |
|---|---|
| 캡처 재현성 | 5모델 × 독립 3회 사건열 정확 일치 |
| 캡처 비간섭 | 21e/21h 7사례: 새 agent 사건열 − dense 사건 = 21h 정식 `*-1`에 같은 거르기를 한 사건열(JSON 정확) |
| (A) 조밀 LU 경계 | push 10 + push 밖 11 = 21회: `dense:lu` 입력(행렬·목록·ZERO·정렬) → Rust 조밀 LU → `dense:done` 행렬(±0 포함 비트)·열 목록(정확) |
| (B) 재분해 생성 | 5모델 push 재분해 전부: 21h `--block` 입력 규칙(첫 재분해 블록 상태만 주입, 이후 Rust 연쇄, header는 각 재분해 원본 진입값)으로 Rust가 만든 `036ade` 반환 순열·factor 전체 = 원본. 조밀 재분해는 추가로 Rust의 전환 단계·n = `dense:enter`, Rust 조밀 입력 행렬·목록 = `dense:lu` |
| (C) 연결 | 04f-1 `push_install_rust.py --compare`의 grow7 분류된 실패 제거 → 설치 59/59, 21h `--block --compare` 219/219 유지, 24 `--lu`·1a `--init`·1c·1d 회귀 |

- 정수·목록·순서 정확, f64는 ±0 포함 비트. NaN이 나오면 실패. 원본 조밀 상태(`dense:*`)는 (B)에서 비교기만 읽는다.
- 판별력: (a) 캡처 검사기 변조(바이너리 해시, 빈 사건, 회차 차이, 비간섭 투영, dense 행렬 1비트·목록 교환·n), (b) (A)/(B) 기대값 변조, (c) Rust 소스 변이(임시 Cargo 사본): 전환 끔(**필수 거부**, grow7), 0.36 → 0.4, 0.325 → 0.36, n > 50 → n ≥ 50, 4m 조건 제거, 추정 갱신 −2 제거, 행 순서(bucket 역순), 열 순서, peel 무시(정렬 고정), 2-lane 대신 단일 누적, 피벗 동률 마지막 유지, 열 교환 생략, 변환 ZERO 버림 제거. 생존은 따로 센다.

## 4. 사례와 원본 자료

| 사례 | push 재분해 조밀(n) | push 밖 조밀(n) | 비고 |
|---|---|---|---|
| grow7 | 1 (51) | 1 (51) | 04f-1 분류된 실패 |
| grow22 | 2 (51·54) | 0 | `local_70`·풀 부족 −1 동반(예상 실패 범위 확인) |
| perold | 2 (54·63) | 3 (61·63·63) | |
| pilot4 | 3 (56·58·66) | 5 (66) | |
| stair | 2 (73·84) | 2 (76·77) | |

- 위 n은 조사 1회의 값이며, 정식 캡처 뒤 검사기가 다시 센다. 자료: `bench/oracle/m3_push_dense/<모델>-{1,2,3}.json.gz`(결정적 gzip), manifest. 비간섭 대조군은 21h 정식 `bench/oracle/m3_push_block/*-1`.
- 모델: `../re/bench/data/netlib/<모델>.mps.gz`(해시는 manifest). 설정 Threads = 1, Seed = 0, Presolve = 0, Method = 2, Crossover = 3, LPWarmStart = 2(21e·04f-1 Netlib과 같음).
- 적용하지 않은 범주: 면허 크기 초과 Netlib 42개(실행 불가), MKL·전치·재시도·빈 열(자연 0회).

## 5. 재실행 명령

- 조사: `python3 tools/push_dense_survey.py OUT` → `--summary`.
- 원본 캡처: `python3 tools/push_dense_runs.py`(5모델 × 3회, 대조군 7사례 × 1회 `--controls`), Gurobi 한 프로세스씩.
- 캡처 검사: `python3 tools/push_dense_check.py --controls --acceptance --self-test`.
- Rust: `python3 tools/push_dense_rust.py --write` → `cargo test -p rsolver-lp --test push_dense` → `python3 tools/push_dense_rust.py --compare`, `--self-test`, `--mutate-rust <새 경로>`.
- 회귀: 21h `push_lu_rust.py --block --compare`, 21g `--generate --compare`, 21f `--compare`, 04f-1 `push_install_rust.py --write/--compare`, 24 `push_loop_rust.py --lu --compare`, 1a `--init --compare`, 1c·1d, `cargo test --release --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, M1 영향 확인(아래 §6에 명령 기록), `git diff --check`.

## 6. 실행 결과

- 환경: WSL2 x86-64 Linux 6.6.87.2(i7-13700F), Python 3.10, Node 22, Rust stable. 계약 `72adcbf`. 원본 실행: 조사 probe 62회(Netlib 1회씩, 면허 초과 42개는 실행 불가)·탐색 21h agent 1회(grow7)·풀 채움 탐색 3회, 정식 1차 22회(표지 없는 agent, 근거로 쓰기 전에 버림·이력 `cf1bd5e` 이전 commit에 있음), **정식 22회**(5모델 × 3 + 대조군 7).
- **계약 변경(Rust 비교 전)**: ① M1 경로는 끈 채 둔다(§1, 풀 크기 F+0x158을 M1이 모름). ② 정식 1차 캡처에 `local_70`·dgetrf 표지가 없어 grow22의 범위 밖 재분해를 사전 기준으로 가를 수 없었다. agent에 표지 probe 2개(0xd85b4, DGETRF 진입)를 더하고 정식 자료 전체를 다시 찍었다(섞지 않음). ③ 조밀 정렬은 캡처 주입 대신 C·asm으로 확정한 풀 첫 채움 식(`pool_first_fill`)으로 Rust가 만든다(탐색에서 9건, (B)에서 10건 일치).
- **캡처**(`bench/oracle/m3_push_dense/`, manifest): `push_dense_check.py --controls --acceptance --self-test` 종료0. 5모델 × 3회 JSON 동일(pid·run_id 제외), 대조군 7사례는 dense 사건이 없고 21h 정식 `*-1`에 같은 거르기를 한 사건열·결과·출력·파라미터와 같다(비간섭). 조밀 LU push 10·밖 11, 모두 자체 LU·첫 시도·결함 0·scratch 정렬 0, 행렬 정렬 0/8 모두 있음. 범위 밖 push 재분해는 grow22 lu 392·431·455(풀 부족 재호출·`local_70`)뿐. 변조 16종 거부.
- **원본 사실**(C·asm·캡처): 전환 판정·조밀 배치·자체 LU·정렬 규칙은 [교훈](../lessons/original.md)에 적었다. 조밀 배열 열 = 활성 행 오름차순(반환 순열 x20 쪽), 행 = 활성 위치의 열 개수 bucket 순(x28 쪽). 정렬 = 풀 첫 채움 `local_f8`의 짝홀(이후 chunk는 4의 배수). 꼬리 8-lane 합은 asm 0xde06e에서 `(A+B)+(C+D)`.
- **(A) 조밀 LU 경계**(`cargo test --test push_dense` → `push_dense_rust.py --compare`): **21회(push 10·밖 11, n 51–84, 정렬 0·8) 행렬·열/행 목록 88,651필드 비트·정확 일치**, 첫 실행부터.
- **(B) 재분해 생성**(`--test push_lu dense_chain_trace`): 5모델 push 재분해 31회 중 범위 안 **28/28 일치**(036ade 반환 순열·factor 전체·블록 상태, 필드 726,853). 조밀 재분해 **10/10**에서 Rust의 전환 단계·n·정렬·조밀 순서·입력 행렬이 `dense:enter`·`dense:lu`와 같고, 조밀이 아닌 18회에서 Rust도 전환하지 않았다. 범위 밖(grow22 lu ≥ 392, 사전 명시 §1(i)) 3재분해는 실패(첫 차이 grow22#467 `x20[229]`, 이후 블록 상태 0x4e0 13 ≠ 14: 풀 부족 재호출이 블록 단계를 한 번 더 센다).
- **(C) 연결**: 04f-1 `push_install_rust.py --compare` **설치 59/59**(grow7 분류된 실패가 통과로 바뀌어 `KNOWN`을 비움), (A) 111호출 그대로. 21h `--block` 219/219, 21g(블록 끔) 211/213(기존 예상 실패 그대로), 21f 207/207, 24 `--lu` 725줄, 1a 731줄, 1c·1d·21b·21d 일치.
- **판별력**: trace 변조 13종 거부(`--self-test`, 첫 시도에서 대상 지정 오류 2건 — n을 반환 칸에서 읽음, 범위 밖 grow22 줄을 고름 — 을 고친 뒤 재실행). Rust 소스 변이 23종(`bench/validation/m3_push_dense/rust_mutations.json`): **거부 20·생존 3**. 필수 `dense-off`는 grow7#164 `x20[89]`에서 거부. 생존: 판정 `n > 50 → n ≥ 50`(n = 50 사례 없음), `4m < n²` 제거(관측 n에서 늘 참), 꼬리 피벗 동률 마지막(동률 없음).
- **회귀**([요약](../../bench/validation/m3_push_dense/regression.txt)) 31명령 모두 종료0: 캡처 검사 3종, 입력 `--write` 9종 불변(`git diff --exit-code`), `cargo test --release --workspace --no-fail-fast` **152통과·0실패·ignored 12**(새 테스트 2), 비교 13종, clippy(첫 실행 `manual_is_multiple_of` 2건 수정), py_compile, `git diff --check`.
- 실행하지 않은 검사: M1 simplex는 조밀 전환을 끈 채(결과 불변)라 확장 설치·pytest·pathcmp는 workspace 회귀로 대신했다. 독립 검토자는 쓰지 않았다(사용자 지시로 이 세션이 작업자·마스터를 겸함).

## 7. 판정과 인수인계

- 작업 수행: **구현 완료(계약 범위 통과)**. 동등성: (A) 자체 조밀 LU 21회 비트 통과, (B) 5모델 push 재분해 범위 안 28/28 통과(조밀 10 포함), (C) 04f-1 grow7 연결 통과(설치 59/59). grow22 범위 밖 3재분해는 사전 명시 실패.
- 미검증: 조밀 LU 재시작(블록 피벗 < ZERO)·결함 행(꼬리 피벗 < ZERO)·빈 열 조밀 전환(자연 0회, C만), MKL dgetrf·전치 경로(n ≥ 1001), `local_70` 조밀 배열 Markowitz 모드·풀 부족 재호출(21j), 판정 경계(n = 50, 4m), 1e99 상한, work, M1 경로 활성화.
- 후속: **M3-04c-21j**(grow22: `036ade` 풀 부족 재호출 C:2592–2620과 `local_70` 모드 C:2569–3960), M1 후속(M1 `Lu`에 풀 크기 수명을 두고 조밀 전환 켜기, pathcmp 회귀), M3-04f-2(cleanup).
- 재실행: `python3 tools/push_dense_check.py --controls --acceptance --self-test` → `python3 tools/push_dense_rust.py --write` → `cargo test --release -p rsolver-lp --test push_dense --test push_lu` → `python3 tools/push_dense_rust.py --compare` / `--self-test` / `--mutate-rust <새 경로>`. 원본 재캡처는 `python3 tools/push_dense_runs.py`(+`--controls`).
- branch `work/m3-04c-21i`, worktree `../rsolver-m3-04c-21i`, 선행 미커밋 없음. 최종 병합은 main `Merge M3-04c-21i` 이력. main 미추적 `.claude/settings.local.json` 보존. 원격/업로드 없음.
- 교훈: `docs/lessons/original.md`(조밀 전환 판정·배치·자체 LU, 정렬 = 풀 첫 채움 짝홀), `tooling.md`(표지 probe 조사, agent 사본 거르기와 비간섭, 표지 추가 시 전체 재캡처).
