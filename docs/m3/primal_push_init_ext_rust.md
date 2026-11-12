# M3-04c-1d Rust push 초기화 외부 값 경로: 검증 계약과 결과

작성 기준은 [스프린트 검증 기준](../15_sprint_verification.md)과 [계약 양식](../templates/sprint_acceptance.md)이다. §1–§5는 구현과 비교를 실행하기 전에 썼다.

## 1. 범위와 주장

- 작업 종류: **구현 비교**. 기준 main `e65ce10`(M3-04c-1b 병합), branch `work/m3-04c-1d`, worktree `../rsolver-m3-04c-1d`. 선행 미커밋 변경 없음(main의 미추적 `.claude/settings.local.json`은 무관, 보존). 사용자의 "다음 M3 스프린트 진행해 너가 작업자야" 지시로 이 세션이 작업자로 수행한다(작업자 선택 질문 생략). owner `claude-m3-04c1d-20261113`(M3-04c-1d·shared).
- 완료할 동작: 1a의 Rust `crossover_init::push_init`을 1b 원본 오라클 15사례·17 push 호출에 대해 **push:enter 상태 + 외부 값**으로 실행해, 초기화 중간 경계 사건과 루프 직전 상태(첫 `supply:enter`) 또는 retry 반환을 원본과 비트/정확 비교한다. 1a에서 자연 미관측이던 경로가 대상이다: 기준 조정과 재구성(-4/-5 생략, -1/-2→-3 승격), Markowitz 강화(`12f5d1`→`061bfe`, 한도 감소 포함), 잔차 > 1e-6 수치 카운터, 품질 retry(P > 1e10, PInf > 1e15)와 retry 뒤 재호출(attempt=1), slack -2(직전 xmm0), 열 -5, first=0.
- 외부 값(원본 캡처에서 주입, 계산 대상 아님):
  - τ_in(param_1, `init:tau`), 이전 PInf S+0x120·S+0x3c8·환경 flag(+0x2c54)·S+0x178·S+0xf0·S+0x294(`init:enter` extra).
  - **조건수**: `12f282`(first일 때 gate 직전 매 라운드)의 출력 *param_8을 캡처 `init:condition:leave` 값으로 차례로 공급한다. 조건수 계산 `116fd7`의 이식은 1c다. first=0이면 caller가 넘긴 *param_8(`init:enter` condition_in)을 쓴다.
  - **first=0의 dense FTRAN 출력**: dual simplex가 남긴 factor는 캡처에 없으므로 `adlittle_x1`만 `09cf9b` 출력을 캡처 `init:ftran:leave` 값으로 주입한다. 이 사례의 FTRAN 출력은 비교 대상이 아니며 그 뒤 계산(잔차·Markowitz·분류·gate·조정 gate·후보·루프 직전 상태)만 판정한다.
  - 합성 편집: 1b `synthetic.json`의 편집을 같은 시점에 적용한다. `enter` 편집(x·vstat)은 push:enter 상태에 반영하고(캡처 `edit` 사건의 before가 push:enter 값과 같음을 확인), `ftran` 편집(S+0x390·S+0x3c8)은 초기 재분해 뒤 강화 판단이 읽는 값으로 넣는다(초기 재분해의 Markowitz는 편집 전 S+0x390).
- 코드 변경 범위: `PushInitBackend`에 `12f282` 경계(`condition`, 기본은 값 유지)와 관측 사건(`observe`, 기본 무동작)을 더하고, `push_init`이 gate에서 이 조건수를 쓰도록 바꾼다. 1a 경로(조건수 없음 → `unknown`)는 결과가 바뀌지 않아야 한다. 계산 규칙 자체는 바꾸지 않는 것이 목표이며, 비교에서 차이가 나면 원본 C/asm을 다시 확인해 고치고 그 사실을 §6에 남긴다.
- 제외와 후속: 조건수 계산 `116fd7`(1c), first=0의 factor 상태·FTRAN 수치, 루프(24에서 검증, 이번에는 첫 `supply:enter`에서 멈춤), push:leave 이후, x87·batch·할당 실패, 0.25 후보·`061bfe` √m 항·정확 경계(1b 자료에 없음), cond > 1e14·> 1e30 gate(자료에 없음), 초기 재분해 반환 ≠ 0, 해 설치(04f), solver 경로 연결.

## 2. 원본과 상태 경계

- Gurobi 13.0.3 `libgurobi130.so`, SHA-256 `8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c`. 주소는 ELF file offset.
- 디컴파일 `../re/decomp/c/001c0070_PRIVATE0000000000134a87.c` C 150–1142(1a §2와 같음). `12f282` 호출은 C 868–872 `if (local_88 != 0)`이며 `local_88 = param_7`(C 219, first)이다. gate C 873–896은 호출 뒤 *param_8을 읽는다. 따라서 first=1이면 라운드마다 `12f282`, first=0이면 호출 없이 진입 *param_8.
- 시작: `push:enter`(합성 `enter` 편집 반영). 종료: 첫 `supply:enter` 직전의 루프 시작 상태(`InitExit::Loop`) 또는 retry 반환(`InitExit::Retry`, 원본 `push:leave` retry=1).
- 입력: 1b 캡처의 push:enter(모델 CSC·bound·rhs·vstat·header·x·slack·허용오차·카운터·first·attempt·model_status) + 위 외부 값. 초기 재분해 입력(ZERO·Markowitz·할당 4필드·블록 상태·S+0x46c)과 S+0x2fc/S+0x2f8는 1b에 없으므로 같은 기저 모델의 1a 입력(`bench/oracle/m3_push_init_rust/input.txt`, 21e·21h에서 투영)을 쓰고, 1b push:enter(편집 전)의 vstat·header·x·slack·rhs·S+0x390·카운터가 1a enter 줄과 같음을 확인한다(같지 않으면 실패). S+0x2f8는 renumber의 leave에만 쓰이며 1b에 관측 필드가 없어 비교하지 않는다.
- retry 재호출(quality·quality_pinf call 2): call 1의 Rust factor 객체를 그대로 이어 받고(S의 factor는 caller가 바꾸지 않는다고 가정), caller가 쓴 S+0x390(=0.9, call 2 push:enter 값)을 재분해 Markowitz로 둔다. 이 가정이 틀리면 call 2의 초기 재분해·FTRAN 비교가 실패한다.

## 3. 비교 계약

Rust 하네스는 호출마다 canonical 줄을 내고, 비교기는 1b 캡처(run 1)의 사건에서 같은 줄을 만든다.

| 줄·필드 | 원본 사건 | 규칙 |
|---|---|---|
| `lu` ret·header·vstat | `init:lu`(first=1) | 정확(반환·재번호 뒤 header·vstat) |
| `ftin` 값 m개 | `init:ftran:enter` values | 비트(라운드마다; 조정 뒤 재구성 rhs 포함) |
| `ftout` 값 m개 | `init:ftran:leave` values | 비트(first=0은 주입이라 제외) |
| `res` 값 m개 | `init:residual:leave` out | 비트(`089d8f` header 순서) |
| `mk` 새 값·잔차 | `init:strengthen:enter` markowitz_new·residual | 비트, 사건 유무 정확 |
| `cond` PInf·위반 수·직전 조건수·S+0x390·S+0x3c8·F+0x188·S+0x178 | `init:condition:enter`(first=1) | 비트·정확 |
| `gate` flag·후보 수·셈 3개·τ_in·PInf | `init:adjust-gate` | 정확·비트(first=0은 PInf 슬롯 미초기화라 제외) |
| `retry` S+0x178·S+0x390·S+0x3c8·F+0x188·S+0x120 | `push:leave` retry=1의 extra | 정확·비트(retry 사례만) |
| `start` 후보·지원·vstat·header·x·slack·rhs·basic·폭·표시·PInf·반복·카운터·S+0x390·F+0x188·F+0x18c·F+0x1e0·F+0x1e8 | 첫 `supply:enter` | 순서 포함 정확, 수치 비트(first=0은 factor 통계 제외: factor 미보유) |
| 사건 순서 | 위 사건의 호출 안 순서 | 정확(줄 종류의 순서와 개수) |

- 수치 오차 허용 없음. ±0 포함 f64 비트. NaN·무한대는 sentinel I로만 다룬다.
- 시간·work·병렬: Threads=1, batch 0만. work 비교 안 함.
- 재현성: 1b 자료(15사례 × 독립 3회 바이트 동일, manifest)를 재사용하고, 입력 작성 전에 `push_init_check.py --controls --acceptance`를 통과시킨다. 새 원본 실행은 하지 않는다.

## 4. 사례

| 사례 | 실제 / 합성 | 노리는 분기 | 원본 증거(1b) |
|---|---|---|---|
| arms·retry·interior·adlittle·scsd1·capri | 실제 | 조건수 주입 gate(1a의 unknown 해소), τ_in 셈 | 1b §6 표 |
| adlittle_x1 | 실제(Crossover=1) | first=0: 초기 재분해·`12f282` 없음, 진입 condition gate | FTRAN 출력 주입 |
| adjust | 합성 | 기준 조정·τ'·재구성(-4 생략)·-1 승격, 2라운드 | flag 0→1 |
| mk_low·mk_limit | 합성 | 강화, `061bfe` 한도 200→50 | strengthen 사건 |
| quality | 합성 | 0.5 강화·수치 카운터·P > 1e10 retry → call 2(attempt=1, S+0x390=0.9) | push:leave retry=1, call 2 supply |
| quality_pinf | 합성 | PInf > 1e15 retry → call 2 | 같음 |
| slack_m2·col_m5·col_m5_ub | 합성 | slack -2 직전 xmm0, 열 -5 = ub, 조정과 승격 | FTRAN 입력 |

- 합성 사례는 바꾼 입력에 대한 원본 동작의 증거다. 자연 모델에서 그 분기가 일어난다는 증거가 아니다.
- 판별력: 비교기 변조(사례마다 각 줄 종류 1비트·순서·누락)와 Rust 변이(조정 τ' 배율, 재구성 -4/-5 포함, -1 승격 strict, slack -2 = 0, -5 = lb, 강화 임계, 수치 카운터 임계, `061bfe` 한도, gate P 규칙, PInf 1e15 카운터, 조건수 무시, first=0에서 `12f282` 호출, retry 상태 등)를 실행해 필수 변이가 거부됨을 확인한다. 생존 변이는 자료가 구별하지 못하는 이유로 분류한다.

## 5. 재실행 명령

- 입력: `python3 tools/push_init_rust.py --write`(1b `--controls --acceptance` 선행) → `bench/oracle/m3_push_init_ext_rust/input.txt`.
- 재생·비교: `cargo test -p rsolver-lp --test push_init_ext && python3 tools/push_init_rust.py --compare`.
- 판별력: `python3 tools/push_init_rust.py --self-test`, `--mutate-rust`.
- 회귀: 1a(`push_loop_rust.py --init --write`가 입력 불변, `push_init_loop`, `--init --compare`), 24(`--lu` 재생·비교), `push_init_check.py --controls --acceptance`, `cargo test --release --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, py_compile, `git diff --check`. 기존 solver 경로·Python 경계는 바꾸지 않으므로 확장 설치·pytest·Netlib·P3는 제외한다.

## 6. 실행 결과

- 기준 main `e65ce10`, 계약 `b824642`, 구현·비교 `d4f2cff`. x86-64 Linux(WSL2), rustc stable, Python 3.10. Gurobi 새 실행 0회. 입력 `bench/oracle/m3_push_init_ext_rust/input.txt`(머리 = 1b manifest·1a 입력 해시 `1f858090…`, 호출마다 case·model·cfg·enter·ext·lu 6줄). `--write`는 1b `controls`·`acceptance`를 통과한 뒤 투영했고, first=1 call 1의 편집 전 push:enter가 1a enter 줄(21e·21h)과 같음, 초기 재분해 Markowitz = push:enter S+0x390, S+0x1f8 진입 0, 편집 before = push:enter 값을 확인했다.
- 코드: `crossover_init.rs`에 `PushInitBackend::condition`(`12f282`, 기본 값 유지)·`observe`(`InitEvent`: Residual·Strengthen·Condition·AdjustGate)와 `InitResult::condition`을 더하고, gate가 이 조건수를 쓴다. 테스트 `crates/lp/tests/push_init_ext.rs`, 도구 `tools/push_init_rust.py`.
- **첫 비교**([기록](../../bench/oracle/m3_push_init_ext_rust/first-compare.txt)): 17호출 중 1곳 불일치 — `quality` call 1 retry 반환의 S+0x390이 원본 0.5(강화 값), Rust 0.1. 원인은 `InitResult.markowitz`를 루프 준비에서만 갱신해 retry 반환이 진입 값을 보고한 것(계산과 backend의 Markowitz는 맞았고 보고 필드만 틀림, 1a 자연 자료는 retry가 없어 미검출). 강화 지점(C 791)에서 결과 필드도 갱신하도록 고쳤다.
- **재비교** `cargo test -p rsolver-lp --test push_init_ext && python3 tools/push_init_rust.py --compare`: **17호출·132줄·727필드 전부 일치**, 불일치·skip 0([출력](../../bench/oracle/m3_push_init_ext_rust/compare.txt), [줄 집계](../../bench/oracle/m3_push_init_ext_rust/coverage.txt)). 줄 종류: lu 16·ftin 20·ftout 19(adlittle_x1 주입 1 제외)·res 20·mk 3·cond 19·gate 18·start 15·retry 2.
  - 기준 조정: adjust·slack_m2·col_m5_ub가 2라운드(gate flag 0→1, τ' 비트, 재구성 rhs·-1/-2 승격·-4/-5 생략, 후보 23→24·6→7·6→6)로 원본과 같다.
  - Markowitz 강화: mk_low(1e-4, 한도 불변)·mk_limit(`061bfe` 200→50, S+0x3c8·F+0x188)·quality(0.5, 잔차 > 1e-6 수치 카운터 +1).
  - retry: quality(P > 1e10)·quality_pinf(PInf > 1e15, 카운터 +1)의 반환 상태와, call 1 factor를 이어 받은 call 2(attempt=1, S+0x390=0.9 재분해)의 header·vstat·FTRAN·basic·factor 통계·후보까지 일치.
  - slack -2 = 직전 xmm0, 열 -5 = ub(col_m5·col_m5_ub)의 FTRAN 입력 비트 일치.
  - first=0(adlittle_x1): 초기 재분해·`12f282` 없음, 진입 condition gate, 잔차·후보 16·루프 직전 상태 일치(FTRAN 출력은 주입).
  - 자연 6사례: 조건수를 주입한 gate가 통과로 같고, 1a의 `cond unknown`이 해소됐다(조건수 값 자체는 1c 범위의 주입).
- **판별력**: `--self-test` 기대 줄 변조 **710종 전부 거부**([기록](../../bench/oracle/m3_push_init_ext_rust/self-test.txt), 호출마다 줄 종류·필드별 1비트/정수+1, 순서 교환, 마지막 줄 누락). `--mutate-rust` **29종 중 거부 24·생존 5·무효 0**([기록](../../bench/oracle/m3_push_init_ext_rust/rust-mutations.json)). 첫 실행에서 `mk-tighten-not-stored`는 retry 수정으로 원문이 바뀌어 무효였고, 원문을 고쳐 29종 전체를 다시 돌렸다(첫 실행 결과는 보관하지 않음, 나머지 28종 판정은 같음).
  - 거부 24: τ' 배율 고정·미적용, 재구성 -4/-5 포함, -1 승격 안 함, 재구성 slack -2 유지, 조정 안 함·두 번, slack -2 = 0, -5 = lb·0, 0.5 임계, 강화 안 함·저장 안 함, 한도 감소 없음·환경 flag 무시, 수치 카운터 임계, gate P 항상 1·항상 PInf, PInf 1e15 카운터 생략·임계, `12f282` 미호출·first=0 호출, first=0 재분해, retry S+0x390 보고. 이 중 9종은 줄 비교 전에 tape 소진/잔여(라운드 수·`12f282` 호출 수 차이)로, `first0-refactor`는 factor 입력 없음 panic으로 거부됐다(나머지 14종은 줄·필드 첫 차이).
  - 생존 5(자료에 경계 없음): `promote-m1-strict`(x = lb + τ' 없음), `adjust-prev-pinf-ignored`(모든 사례 S+0x120 < τ, 1b §6), `adjust-count-ignored`(PInf > 1인 사례 모두 0.1셈 < 후보 수), `gate-attempt-ignored`(attempt=1 호출은 PInf 0·P 1·cond 작음), `cond-product-ignored`(P·cond > 1e16 단독 결정 사례 없음).
- 회귀([기록](../../bench/oracle/m3_push_init_ext_rust/regression.txt), 모두 종료 0): 1a `--init --write`(입력 불변)·`push_init_loop`·`--init --compare` 6사례 731줄·5,743필드, 24 `--lu --write`(불변)·`push_loop_lu`·`--lu --compare` 725줄·5,575필드, 1b `push_init_check.py --controls --acceptance`, `cargo test --release --workspace` **147통과·0실패·ignored 12**(새 `push_init_ext` 1), `cargo clippy --workspace --all-targets -- -D warnings` 0, py_compile, `git diff --check`.
- 실행하지 않은 검사: 기존 solver 경로·Python 경계 미변경이라 확장 설치·pytest·Netlib·P3 제외. 독립 검토자는 사용하지 않았다.
- 계약과 다른 점: 없음. `start` 줄의 support marks는 원본 supply:enter가 0 배열, Rust가 `None`(표시 없음)이라 0 배열을 `none`으로 정규화해 비교했다(표현 차이, 비영 표시가 있으면 그대로 비교).

## 7. 판정과 인수인계

- 작업 수행 **완료**, 구현 동등성 **계약 범위 통과**. 1b의 자연 7·합성 8사례 17 push 호출에서 push:enter 상태와 외부 값(τ_in·S+0x120·S+0x3c8·환경 flag·조건수 출력, first=0의 FTRAN 출력)만으로 Rust `push_init`이 초기 재분해·FTRAN 입력·잔차·Markowitz 강화와 `061bfe` 한도·수치 카운터·분류·품질 gate·retry·기준 조정과 재구성·루프 직전 상태를 원본과 비트/정확 일치로 만든다. 1a에서 미검증이던 조정·강화·잔차 >1e-6·retry(P·PInf)·slack -2·열 -5·first=0 경로가 이 범위에서 검증됐다.
- 동등성 한계: 합성 사례는 바꾼 입력에 대한 원본 동작과의 일치이며 자연 모델에서 그 분기가 일어난다는 증거가 아니다. 조건수 값은 원본 출력 주입이며 `116fd7` 계산은 1c다. first=0의 FTRAN 수치(dual factor)는 미검증. retry 재호출은 call 1 factor를 이어 받는 가정으로 일치했다(2사례). 미관측·생존 경계: x = lb+τ' 승격, S+0x120 ≥ τ, 0.1셈 ≥ 후보 수, attempt=1에서 결정적인 gate 항, cond > 1e14·P·cond > 1e16·cond > 1e30, 0.25 후보, `061bfe` √m 항, 초기 재분해 반환 ≠ 0·S+0xf0 ≤ −1000, 후보 0, lb ≠ 0 기저 shift.
- branch `work/m3-04c-1d`, worktree `../rsolver-m3-04c-1d`, owner `claude-m3-04c1d-20261113`. 최종 commit/병합은 main `Merge M3-04c-1d` 이력을 따른다. 미커밋 선행 변경 없음.
- 갱신: `docs/11_progress.md`·`12_roadmap_diagram.md`·`14_sprint_plan.md`·`m3/sprint_design.md`·`handoff/M3.md`. 교훈 `docs/lessons/original.md`(retry 재호출의 factor 연속성·`12f282` 호출 조건), `tooling.md`(tape 소진 판별, 조기 반환 보고 필드).
- 다음: **M3-04c-1c**(조건수 `116fd7`, 1b 캡처 condition 19값이 기준이며 이 하네스의 조건수 tape를 Rust 계산으로 바꿔 재비교), 또는 23(batch 조사)·04f(해 설치). 기준 재검사 `python3 tools/push_init_rust.py --write && cargo test -p rsolver-lp --test push_init_ext && python3 tools/push_init_rust.py --compare`.
