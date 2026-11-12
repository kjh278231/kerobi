# M3-04c-1a Rust push 초기화와 루프 연결: 검증 계약과 결과

작성 기준은 [스프린트 검증 기준](../15_sprint_verification.md)과 [계약 양식](../templates/sprint_acceptance.md)이다. §1–§5는 구현과 비교를 실행하기 전에 썼다.

## 1. 범위와 주장

- 작업 종류: **구현 연결**. 기준 main `de29640`(M3-04c-1 병합), branch `work/m3-04c-1a`, worktree `../rsolver-m3-04c-1a`. 선행 미커밋 변경 없음. main의 미추적 `.claude/settings.local.json`은 무관하며 보존한다. 사용자의 "다음 M3 스프린트 진행해 너가 작업자야" 지시로 이 세션이 작업자로 수행한다(작업자 선택 질문 생략). 점유 owner `claude-m3-04c1a-20261113`(M3-04c-1a·shared).
- 완료할 동작: double primal push `134a87`의 초기화(C 150–1142 중 단계 2–10, [조사](primal_push_init.md) §2.1)를 Rust `crossover_init::push_init`으로 옮기고, 그 출력 상태로 04c-24의 `push_loop`+`SerialBackend`를 이어 **push:enter부터 루프 종료까지** 원본과 비교한다.
  - 단계 2: 첫 push면 S+0x294=10으로 `05b257` 호출(반환 1 반복), 반환 12·≥2·S+0xf0 ≤ -1000·수치 카운터 gate 처리, 성공이면 S+0x294=0과 재번호된 header. Serial에서는 21h 초기화 재분해 입력(할당·블록 상태·ZERO·Markowitz·S+0x46c)과 **push:enter header·S+0x2f8·vstat**로 Rust `refactor`를 부른다.
  - 단계 3–4: rhs scratch(S+0xa0 복사), 열·slack 스캔(slack -2 = 직전 xmm0, -4 lb·-5 ub, NaN v는 비영), 거리, 0.1·0.01·0.001 셈, key·`aabbe3`(`common::sort_desc`), S+0x200 복사.
  - 단계 5–6: dense rhs FTRAN(`09cf9b` count -1, 24의 재계산 FTRAN과 같은 backend 경로), max|rhs|·`089d8f`(B·x_B, FMA 없음 `0x115e27`)·max|x_B|·max|rhs−Bx_B|, Markowitz 후보 0.5/0.25/1e-4와 강화(`12f5d1`→`061bfe`, double만), 잔차 >1e-6이면 S+0x178++.
  - 단계 7: basic 복사·lower shift·폭·표시·위반 수·PInf(C 797–866, NaN은 C 비교 그대로).
  - 단계 8: 품질 gate. 조건수는 **외부 입력**(`Option<f64>`)이다(아래 제외).
  - 단계 9: 기준 조정 1회(τ' 선택, 재구성의 -4·-5 생략, -1/-2→-3 승격, 슬랙 승격) 후 4로 복귀. 진입 조건은 asm `0x1c185c–0x1c1890`대로 `!조정함 && PInf > 1 && S+0x120 < τ(h) && 0.1셈 < 후보 수`(조사 문서 §2.1의 "τ < S+0x120"은 비교 방향 오기, §6에 정정 기록).
  - 단계 10: S+0x120=PInf, S+0x2a0+1, 지원 flag `sqrt(m) ≤ 후보 수`, 후보 0이면 루프 없이 종료, numeric_limit = 진입 S+0x178+10, 품질 주기 50, 라운드 상태(위치·보류 0, 실패 횟수 0, 지원 표시 없음, count_sum 0, 성공 0, 요청·수치·실패·상태 = S+0x294·0x178·0x184·0xf0, retry 0).
- 입력 전제와 외부 값: τ_in(param_1)·이전 PInf(S+0x120)·S+0x3c8·환경 +0x2c54·조건수는 캡처가 없다(1b). Rust는 이들을 `Option`으로 받고 **값이 필요한 분기에 이르면 실패**(panic)한다. 단 조건수만은 자연 6사례가 모두 품질 gate에 이르므로 예외로, `None`이면 gate의 조건수 항을 거짓으로 두고 결과에 `condition: unknown`을 남긴다. 따라서 품질 gate는 **P·PInf 항만 검증**되고 조건수 항(cond > 1e14, cond·P > 1e16, cond > 1e30)은 미검증이다. S+0x1f8 잔차 scratch는 진입에서 0이라고 둔다(원본이 매 사용 뒤 0으로 되돌림, C 781–785).
- 제외와 후속: 조건수 `116fd7`(1c), 중간 경계·미관측 분기 원본 비교(1b), 할당 실패·batch(threads>1, 23)·x87(`1faa63`, S+0x28≠0)·로그 `d07d1`·S+0x1d8 flag, 해 설치(C 2419 이후, 04f), 기존 solver 경로 연결. 루프 쪽 제외는 24 §1과 같다.

## 2. 원본과 상태 경계

- Gurobi 13.0.3 `libgurobi130.so`, SHA-256 `8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c`. 주소는 ELF file offset.
- 디컴파일 `../re/decomp/c/001c0070_PRIVATE0000000000134a87.c` C 150–1142(초기 factor C 1017–1051, 스캔 401–596, 정렬·복사 598–645, FTRAN·잔차 646–785, Markowitz 786–796, 분류 797–866, gate 867–896, 준비 897–930, 조정 931–1012·1063–1141), `00115d60_…089d8f.c`(잔차), `…12f5d1.c`·`000f62c0_…061bfe.c`(강화), `…12f282.c`(조건수 분기). asm: gate·조정 조건 `0x1c1800–0x1c1890`, 스캔 out-of-line `0x1c4f29–0x1c51bc`, 정렬 key `0x1c10e0–0x1c112b`.
- 시작: `push:enter`(`134a87` 진입). 종료: `roundctl:end`(24와 같음). 초기화 경계 사건은 첫 `supply:enter`·`roundctl:pass:enter`·`roundctl:pass:ready`다.
- 설정·사례: 21e·21h 캡처와 같음(Threads=1, Seed=0, Presolve=0, Method=2, Crossover=3, LPWarmStart=2; arms·retry·interior·adlittle·scsd1·capri). 자연 6사례 모두 first=1, attempt=0, batch 0, precision 0.
- 입력(push:enter): 모델 CSC·목적·bound·rhs S+0xa0, vstat, header, x, slack, 허용오차 S+0x370/380/390/398/3b0, 반복 S+0x2a0, 카운터 S+0x178/0x184/0x294, S+0xf0(첫 `factor:enter`의 factor_ctl status, 그 사이 쓰기 없음), S+0x2fc·S+0x2f8(첫 `lu:enter`의 enter·leave, 그 사이 쓰기 없음), 공급 drop S+0x3a8(초기화가 쓰지 않음), first·attempt·model_status, 초기 factor 입력(24 `lu` 줄과 같음: ZERO·Markowitz·할당 4필드·21h 블록 상태·S+0x46c). S+0x328 direction은 초기화가 쓰지 않고 루프의 `push_pick`이 읽기 전에 0으로 쓰므로 0을 넣는다.
- 출력: 루프 시작 상태 전체(후보 순서·총수, vstat·header·S+0x2f8, rhs S+0x200, basic S+0x1e8, 폭, 표시, 위반 수, PInf, 반복, 지원, 요청, 카운터, numeric_limit, Markowitz S+0x390과 갱신 한도) 그리고 루프 사건열.

## 3. 비교 계약

24의 canonical 사건열(`init`·루프 725줄)을 그대로 쓰고 `start` 줄 하나를 사례마다 `init` 뒤에 더한다. 입력 파일에는 루프 시작 상태(22·24의 `state`·`round` 줄)가 없고 push:enter 상태만 있다.

| 줄·필드 | 의미 | 원본 출처 | 규칙 |
|---|---|---|---|
| `init` hdr·leave·vstat·stats | 초기 factor 재번호 뒤 상태와 통계 | 초기화 `lu:leave`·`factor:leave` | 정확(24와 같음, 입력이 push:enter header로 바뀜) |
| `start` cand·total·sup | 정렬 후보·총수·지원 flag | `roundctl:pass:enter` | 정확(순서 포함) |
| `start` vstat·header·leave | 루프 시작 basis와 S+0x2f8 | `supply:enter` state, `roundctl:pass:ready` selected.leave | 정확 |
| `start` rhs·basic·width | S+0x200, S+0x1e8, 폭 | `supply:enter` state | **비트**(basic은 이번에 처음 비교: Rust FTRAN 결과 − lb) |
| `start` marks·pinf·iter | 표시, S+0x120, S+0x2a0 | `supply:enter` state | 표시 정확, PInf·반복 비트 |
| `start` req·numeric·failure·status·retry·nlimit | S+0x294, S+0x178, S+0x184, S+0xf0, retry, numeric_limit | `supply:enter` counters, `roundctl:pass:enter` | 정확 |
| `start` mk·limit | S+0x390, 갱신 한도 F+0x188 | `supply:enter` tolerances 390, factor.limit | 비트·정확 |
| `start` adj·cond | 기준 조정 여부, 조건수 상태 | (원본 값 없음) | adj는 원본 vstat의 -1/-2→-3 변화 유무와 정확 비교, cond는 비교 대상이 아니라 Rust가 `unknown`을 출력하는지 확인(조건수 미검증 표시) |
| 루프 줄 | 24와 같음 | 24 §3 | 24와 같음 |

- 수치 오차 허용 없음. ±0 포함 f64 비트 비교. NaN·무한대는 입력 sentinel I로만 다룬다.
- 시간·work·병렬: Threads=1, batch 0, work 비교 안 함.
- 재현성: 21e 캡처(독립 3회·비간섭)와 21h 블록 상태 캡처(독립 3회)를 재사용한다. 도구는 두 검사기의 controls·acceptance를 먼저 통과시키고, push:enter·pass:enter·supply:enter가 3회 같음은 M3-04c-1 `push_init_survey.py --check`가 이미 고정했다. 새 원본 실행은 하지 않는다.

## 4. 사례

| 사례 | 실제 / 합성 | 전제·노리는 분기 | 입력 경로 | 원본 증거 |
|---|---|---|---|---|
| arms·retry·interior·adlittle·scsd1·capri | 실제(21e·21h 정상 실행 훅) | first=1 초기 재분해(반환 0), 열 -1·-2·-3·-4, slack -1·-3·-4, 동률 정렬(4사례), 0.99 상한, gate 통과, 조정 0회, Markowitz 강화 0회 | `bench/oracle/m3_push_lu_chain/*-1`(21e manifest), `bench/oracle/m3_push_block/`(21h manifest) | 사례마다 init 1·start 1·루프 사건 |

- 고정 집계: 24 §4(패스14·공급119·단계107·BTRAN32(dense2)·판단67·`05b257` 루프 49·재계산13·round12·종료6) + init 6 + start 6.
- 미관측(판정 안 함): 기준 조정과 재구성, Markowitz 강화·`061bfe`, 잔차 >1e-6, 품질 retry·조건수 항, 초기 factor 반환 ≠ 0·S+0xf0 ≤ -1000·numeric gate retry, first=0, slack -2, 열 -5, 후보 0, quick 정렬, 할당 실패. 이 경로의 Rust 코드는 C 그대로 옮기되 동등성은 미검증이다. 그중 외부 값이 필요한 경로는 값 없이 도달하면 실패한다.

## 5. 재실행 명령

- 입력: `python3 tools/push_loop_rust.py --init --write`(21e·21h 검사 선행) → `bench/oracle/m3_push_init_rust/input.txt`.
- 재생·비교: `cargo test -p rsolver-lp --test push_init_loop && python3 tools/push_loop_rust.py --init --compare` 종료 0.
- 판별력: `python3 tools/push_loop_rust.py --init --self-test`(trace 변조 전부 거부), `--init --mutate-rust <새 경로>`(초기화 변이 + 24 변이; 자연 경로 변이는 거부, 생존은 분류).
- 회귀: 24 재생(`--lu --write`가 입력을 바꾸지 않음, `push_loop_lu`, `--lu --compare`), 22 tape 재생, `push_init_survey.py --check`, `cargo test --release --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`. 기존 solver 경로·Python 경계는 바꾸지 않으므로 확장 설치·pytest·Netlib·P3는 제외한다.

## 6. 실행 결과

- 기준 main `de29640`, 계약 `00cfb02`, 구현·첫 비교 `daf9ec7`, clippy 정리·재비교 `d9aab36`. x86-64 Linux(WSL2), rustc stable. Gurobi 새 실행 0회. 입력 `bench/oracle/m3_push_init_rust/input.txt`(머리 = 21e manifest 해시, 사례마다 case·model·cfg·enter·lu 5줄, 루프 시작 상태·수치 출력 없음).
- 구현: `crates/lp/src/crossover_init.rs`(`push_init`, `PushInitBackend`, `InitEntry`·`InitResult`), `crossover_backend.rs`(`SerialStart`: 초기 재분해 입력을 들고 있다가 요청 10에서 Rust `refactor`, 이후 `SerialBackend`로 넘김; `SerialBackend::initial`은 같은 helper 사용), `crates/lp/tests/push_init_loop.rs`, `tools/push_loop_rust.py --init`.
- 착수 검사: `--init --write`가 21e·21h 검사기의 controls·acceptance를 통과한 뒤 투영했고, 사례마다 초기 factor 입력 header·vstat·S+0x2f8·Markowitz가 push:enter 상태와 같음을 확인했다(그 사이 쓰기 없음).
- 비교 `cargo test -p rsolver-lp --test push_init_loop && python3 tools/push_loop_rust.py --init --compare`: **6사례·731줄·5,743필드 전부 일치(첫 비교부터)**. 24의 725줄·5,575필드 + start 6줄·168필드. start 줄에서 후보 103개 순서, rhs·basic·폭(각 m개, 비트), 표시, PInf, 반복, 지원, 요청·카운터·numeric_limit, Markowitz·갱신 한도, 라운드 초기값이 원본과 같다. **basic(초기 재분해 → rhs 구성 → dense FTRAN → lower shift)은 이번에 처음 비트 비교했다.** [비교 출력](../../bench/oracle/m3_push_init_rust/compare.txt).
- 판별력: `--init --self-test` **변조 136종 전부 거부**([기록](../../bench/oracle/m3_push_init_rust/self-test.txt)).
- Rust 변이 `--init --mutate-rust`: **73종(초기화 29 + 24의 44) 중 거부 51(시간 초과 1), 생존 22**([기록](../../bench/oracle/m3_push_init_rust/rust-mutations.json)). 초기화 거부 20: 초기 재번호 생략·요청 유지·초기 factor 생략, -2·-4 값 0, slack -3 값 0, 거리 항상 x-lb, 0.99 상한 제거, 안정 정렬, S+0x200 미복사, 잔차 곱 생략·임계 1e-14(강화 경로에서 외부 값 없음 panic), 수치 카운터 항상, X→L, gate P·PInf 항상, 조정 PInf 조건(이전 PInf 없음 panic), 반복 미증가, numeric_limit, 상태 S+0xf0. 초기화 생존 9은 자연 자료가 구별하지 못하는 경우다.
  - `scan-m1-zero`·`no-lower-shift`·`width-ub`: 6사례 모두 -1 열과 기저 변수의 lb가 0(또는 -I)이다.
  - `scan-m4-ub`: -4 변수는 lb = ub. `scan-m5-zero`·`slack-m2-zero`: 자연 0회.
  - `mark-tau-strict`: basic이 정확히 -τ인 위치 0. `support-strict`: √m = 후보 수인 사례 없음.
  - `key-slack-as-column`: `-1 - c`와 `-(1 + c)`는 IEEE 반올림 대칭이라 같은 값(동치 변이).
  - 루프·backend 생존 13은 24 §6과 같은 목록·분류다.
- 회귀(`bench/oracle/m3_push_init_rust/regressions.json`, 11명령 종료0): 24 입력 재투영 불변·재생·비교(725줄), 22 입력 재투영 불변·재생·비교(719줄), `push_init_survey.py --check`, py_compile, `cargo test --release --workspace` **146통과·0실패·ignored12**(새 `push_init_loop` 1), clippy 0, `git diff --check`.
- 실행하지 않은 검사: 기존 solver 경로·Python 경계 미변경이라 확장 설치·pytest·Netlib·P3 제외. 독립 검토자는 사용하지 않았다.
- 계약과 다른 점: 없음. 조사 문서 §2.1의 기준 조정 조건 방향(τ < S+0x120)을 asm `0x1c1873`에 따라 `S+0x120 < τ`로 구현했고 조사 문서를 정정했다(자연 조정 0회라 결과에는 영향 없음).

## 7. 판정과 인수인계

- 작업 수행 **완료**, 구현 동등성 **계약 범위 통과**. 자연 6사례에서 push:enter 상태(모델·basis·x·slack·허용오차·카운터)와 초기 재분해 입력만으로 Rust가 초기 factor·후보 구성·정렬·rhs·basic·폭·표시·PInf·라운드 상태를 만들고, 이어 24의 실제 Rust backend 루프가 종료까지 원본과 비트/정확 일치한다. 원본 상태·수치 재주입 0회.
- 남은 한계: 품질 gate의 조건수 항(`116fd7`, 입력 없음, 1c)은 미검증이며 Rust 결과에 `cond unknown`으로 남는다. 외부 값(param_1, S+0x120, S+0x3c8·환경 flag)이 필요한 경로(기준 조정, Markowitz 강화)는 C 그대로 옮겼으나 값 없이 도달하면 실패하며 동등성 미검증이다. 자연 미관측: 기준 조정·재구성, Markowitz 강화, 잔차 >1e-6, 품질 retry, 초기 factor 반환≠0·S+0xf0 gate, first=0, slack -2, 열 -5, 후보 0, lb≠0인 -1 열·기저 변수, quick 정렬. 할당 실패·batch(23)·x87·해 설치(04f)·solver 경로 연결은 범위 밖.
- branch `work/m3-04c-1a`, worktree `../rsolver-m3-04c-1a`, owner `claude-m3-04c1a-20261113`. 최종 commit/병합은 main `Merge M3-04c-1a` 이력을 따른다. 미커밋 선행 변경 없음.
- 갱신: `docs/11_progress.md`·`12_roadmap_diagram.md`·`14_sprint_plan.md`·`m3/sprint_design.md`·`m3/primal_push_init.md`(조건 방향 정정)·`handoff/M3.md`. 교훈 `docs/lessons/original.md`(조정 조건 방향, basic 비트 재현), `tooling.md`(변이 도구 실행 중 소스 수정 금지, 캡처 없는 입력은 Option+필요 시 실패).
- 다음: **M3-04c-1b**(초기화 중간 경계·미관측 분기 오라클: `12f282` 입출력·param_1·S+0x120 캡처 포함) → 1c(조건수). 또는 23(batch 조사), 해 설치 04f. 기준 재검사 `python3 tools/push_loop_rust.py --init --write && cargo test -p rsolver-lp --test push_init_loop && python3 tools/push_loop_rust.py --init --compare`.
