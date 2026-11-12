# M3-04c-22 초기화된 직렬 push 루프 연결 검증 계약과 결과

작성 기준은 [스프린트 검증 기준](../15_sprint_verification.md)과 [계약 양식](../templates/sprint_acceptance.md)이다. §1–§5는 Rust 루프 구현과 비교를 실행하기 전에 썼다.

## 1. 범위와 주장

- 작업 종류: **구현 L1(연결)**. 기준 main `abc7a4e`(M3-04c-21h 병합), branch `work/m3-04c-22`, worktree `../rsolver-m3-04c-22`. 선행 미커밋 변경 없음. main의 미추적 `.claude/settings.local.json`은 무관하며 보존한다. 사용자의 "다음 M3 스프린트 진행해 너가 작업자야" 지시로 이 세션이 작업자로 수행한다(작업자 선택 질문 생략). 점유 owner `claude-m3-04c22-20261112`.
- 완료할 동작: double primal push `134a87`의 **초기화된 루프 본체**(C 1143–2418)를 Rust 함수 하나(`crossover_loop::push_loop`)로 연결한다. 첫 패스 준비(`0x1c1ff8`) 직전 상태에서 시작해 원본 중간 상태를 다시 넣지 않고 다음을 이어 간다.
  - 패스 준비(표시 clear·압축, `round_pass`) → 공급자(`supplier_next`) → 첫 성공의 승인수 초기화(`round_begin`) → 단계 선택·적용(`push_pick`·`push_apply`) → 승인 증가·단계 보류(`round_accept`·`round_defer`) → 승인 pivot의 지원 BTRAN(`round_btran`, dense면 round로) → factor 판단(`factor_select`) → 호출 wrapper(`factor_run`)·재분해 뒤 재계산(`push_recompute`, pivot 지점)·품질 주기·요청 지우기 → 다음 공급자.
  - 공급 고갈 → 종료 gate(`round_exhausted`) → 지원 gate·압축·요청 10(`round_restart`) → round factor(`round_factor`) → 재계산(round 지점) → 요청 지우기 → 후보가 남으면 다음 패스, 없으면 종료.
  - 재분해(`05b257` 요청≠0) 뒤 header·vstat·S+0x2f8 재번호(21e 규칙5 (a)–(c), `05b257` C:2036–2090)는 backend가 돌려준 순열 F28(o)·F38(n)로 Rust 상태에 적용한다.
- 기존 L1 조각의 동작은 바꾸지 않는다. `crossover_factor`에는 재계산 뒤 같은 wrapper의 품질 검사부터 재개하는 진입점(`factor_resume`, C 1982–1996)만 더한다.
- 입력 전제: double(`S+0x28==0`)·Threads=1·batch 용량/채움/소비 0, 희소 공급 열(dense raw는 공급자가 압축), pricing 객체 null(S+0x478/0x480), 루프 중 종료 검사(`9a91f3` 콜백·`a83b2f`·`8b129`)는 0을 반환해 계속한다고 가정한다.
- **수치 backend 외부**: 공급 FTRAN raw, 지원 BTRAN raw, `05b257`의 반환·수치 카운터(numeric/failure/status)·factor 통계(F18c·F188·S+0x46c·F190·F184·F1e0·F1e8)·재분해 순열, 재계산 FTRAN raw, 품질 `089f61`은 원본 캡처의 출력 tape를 순서대로 돌려준다. backend가 받은 입력(공급 열·spike 인자, e_leave, factor 요청·header·이탈 위치, 재계산 rhs)은 trace에 기록해 원본 입력과 비교한다. 실제 Rust LU(21b–21h)를 루프에 설치하는 연결은 이번 범위가 아니다.
- 제외와 후속: 초기화(첫 factor·후보 구성·정렬·기준 조정, C 1143 이전), 최종 외부 해 설치(C 2420 이후 `LAB_002c4050`), batch(23), x87, 품질 helper 수치(15a), 종료 검사·콜백, work·시간, 실제 수치 backend 설치(후속 번호로 등록), retry/오류 종료 뒤 cleanup(`LAB_002c08d5`).
- 선행: 04c-10(pick/apply), 14(공급자), 16(factor 제어), 18(재계산), 20(라운드)의 Rust L1과 21e 지연 LU 연쇄 오라클 캡처(독립 3회·비간섭·변조 거부 검증 완료).

## 2. 원본과 상태 경계

- Gurobi 13.0.3 `libgurobi130.so`, SHA-256 `8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c`. 주소는 ELF file offset.
- C `../re/decomp/c/001c0070_PRIVATE0000000000134a87.c`: 패스 준비 1143–1176, 공급 1177·2232, 단계 1177–1718·2063–2222, 승인수 1594·2089, BTRAN 1999–2062, factor 판단 1723–1752, wrapper 1753–1997(재계산 1897–1980, 품질 1982–1995, 요청 지우기 1996), 보류 2226–2231, 고갈·round 2235–2418, 루프 조건 2418(`while (local_20cc != 0)`) → `LAB_002c4050`. 재분해 재번호 `000f15c0…05b257.c` 2036–2090. 각 조각의 asm 근거는 선행 작업 문서(10·14·16·18·20·21e) §2를 따른다.
- 직접 다시 읽은 C 사실: 첫 공급이 -1이면 승인수 `local_238`을 초기화하지 않고 이전 값으로 gate를 판단한다(C 1177–1178, 2234). 승인 flip은 `0x1c2994`에서, pivot은 2089에서 증가한다. flip(S+0x2f8<0)은 rhs·basic 갱신 뒤 바로 다음 공급자로 간다(C 1718–1721). support=0이면 BTRAN 없이 판단(`LAB_002c336d`), support≠0이면 BTRAN 뒤 희소 출력이면 판단으로, dense면 support=-1·표시 clear·압축 뒤 후보가 있으면 `LAB_002c3c0c`(지원 gate), 없으면 종료(C 2023–2062). pivot 지점 재계산 뒤에는 같은 wrapper의 품질 검사로 이어지고(C 1980–1982), round 재계산 뒤 요청을 0으로 지우고 후보 수가 0이 아니면 패스 준비로 돌아간다(C 2414–2418).
- 시작 시점: push 안 첫 `roundctl:pass:enter`(초기 factor 반환 뒤 `0x1c1ff8`). 입력 상태:
  - 모델: `push:enter`의 CSC·cost·lb·ub. 상수: S+0x380(h)·0x398·0x370(I)·0x3b0, 공급 drop S+0x3a8, model_status·attempt, numeric_limit(local_f8), 품질 주기 초기 50(C 199).
  - P 문맥: `pass:enter`의 후보·cursor·deferred·실패·support·표시(미초기화면 없음)·count_sum·successes, 승인수는 미초기화(None).
  - S 상태: 첫 `supply:enter`의 vstat·header·x/slack·basic·폭·표시·rhs·반복 카운터·요청·PInf(S+0x120). 패스 준비는 S를 바꾸지 않는다. S+0x2f8·0x2fc·0x328은 `pass:ready`의 selected. 위반 수 local_1b0은 표시의 B/A 수.
  - backend 초기 통계: 초기화 factor `factor:leave`의 factor_ctl.
- 종료 시점: `roundctl:end`(고갈 gate 종료 또는 round 재계산 뒤 빈 목록). 이후 해 설치는 비교하지 않는다. retry/오류 종료는 LoopExit로 반환만 한다.
- 원본 출력·사건 순서: 아래 §3의 canonical 사건열. 원본 쪽은 21e 캡처의 사건을 같은 순서의 줄로 투영한다(lazy 훅: `step:leave`·`pass:ready`는 다음 공급자 진입에서 기록되므로 Rust도 같은 지점에서 낸다).

## 3. 비교 계약

Rust는 시작 상태에서 끝까지 자기 상태를 이어 간다. **원본 상태 재주입 0회**: 입력 파일에는 시작 상태와 backend 출력 tape만 있고, 원본 후상태는 Python 비교기만 읽는다. tape 소비 순서가 어긋나면 첫 차이로 실패한다.

| 줄(순서대로) | 비교 필드 | 원본 출처 | 규칙 |
|---|---|---|---|
| `leave` 단계 뒤(다음 공급 진입 직전, flip·pivot만) | vstat·header·x/slack·basic·폭·표시·rhs·반복·PInf | `step:leave.state` | 정수 정확·f64 비트 |
| `pass` 패스 준비 뒤 | 후보(총수까지)·cursor·deferred·support·표시·요청 | `roundctl:pass:ready` | 정확 |
| `ftran` 공급 FTRAN 입력 | count·행·값·spike 인자 유무 | `supplier:ftran:enter` | 정확·비트 |
| `supply` 공급 반환 | 반환 열·cursor·deferred·후보·실패 배열·support·표시·count_sum·successes, 성공이면 압축 뒤 alpha | `supply:leave` | 정확·비트 |
| `step` 단계 | 경로 F/V, ratio별 S+0x2f8/방향/요청·alpha·위치/theta/pivot, frame r·dL·dU, 단계 종류·theta, sel, 적용점 상태(vstat·header·basic·폭·표시·반복), 승인수(flip/pivot) 또는 보류 뒤 deferred·후보·해당 열 실패 수 | `ratio:*`·`retry:leave`·`step:enter`·`roundctl:accepted`·`step:applied`·`step:defer`·`roundctl:defer:leave` | 정확·비트 |
| `btran` 지원 BTRAN | e_leave, raw(count·목록·값), 희소면 뒤 support·표시·후보·cursor·deferred | `roundctl:btran:enter/leave/applied` | 정확·비트 |
| `select` factor 판단 | 결정 요청·호출 여부 | 다음 `factorctl:call`의 요청 또는 호출 없는 `factorctl:clear` | 정확 |
| `fcall` 각 `05b257` 호출 | 지점(pivot/round)·요청·진입 S+0x2f8·진입 header, 반환, 반환 뒤 header·vstat·S+0x2f8 | `factor:enter`·`lu:enter`·`factor:leave`·`lu:leave` | 정확 |
| `recompute` | 지점, FTRAN 입력 rhs·header, basic·폭·표시·PInf·위반 수·retry·종료 | `recompute:enter/leave` | 정확·비트 |
| `clear` pivot 판단 종료 | 품질 주기 local_100 | `factorctl:clear.frame.period` | 정확 |
| `exhausted` | cursor·deferred·총수·support·승인수(live)·gate 결과(종료/계속) | `roundctl:exhausted` + 다음 사건 | 정확 |
| `round` round factor 직전 | 후보·cursor·deferred·support·표시·요청 | `roundctl:factor` | 정확 |
| `end` | 후보·cursor·deferred·support·표시·요청 | `roundctl:end` | 정확 |

- 줄 수·순서·태그가 원본과 같아야 하며 원본에 없는 줄이나 빠진 줄은 실패다. 승인수 비관측(첫 공급 전 고갈) 경계는 자연 자료에 없으면 미관측으로 남긴다.
- 수치는 모두 ±0 포함 f64 비트 비교, NaN/IEEE무한대 입력은 범위 밖. 허용오차 비교 없음.
- 원본 재현성: 21e 캡처는 사례별 독립 3회 일치·21c 비간섭·대조군 0사건·변조 54 거부가 검증됐다. 이번 도구는 `push_lu_check`의 controls·acceptance를 먼저 통과시킨 뒤 각 사례 run 1을 쓴다. 새 원본 실행은 하지 않는다.
- 시간·work: Threads=1, work는 비교하지 않는다(ratio·FTRAN work 인자는 0 또는 무시).

## 4. 사례와 원본 자료

| 사례 | 실제/합성 | 노리는 분기 | 입력 |
|---|---|---|---|
| arms | 실제 | 지원 모드 다수 패스, 보류 실패 누적, 고갈→round 3회, 전체보류 support0 종료 | `bench/oracle/m3_push_lu_chain/arms-1.json` |
| retry | 실제 | 위반 경로·`056973`, 보류, round 2회 뒤 종료 | `retry-1.json` |
| interior | 실제 | support 0의 FT 갱신 호출·pivot 지점 재분해(요청 10)와 재계산·round 뒤 빈 목록 종료 | `interior-1.json` |
| adlittle | 실제(Netlib) | dense BTRAN → round, FT 갱신 다수 | `adlittle-1.json.gz` |
| scsd1 | 실제(Netlib) | flip 연속, 단일 round | `scsd1-1.json.gz` |
| capri | 실제(Netlib) | dense BTRAN → round, FT 갱신 다수, `1da0b2` 블록 재분해(순열만 사용) | `capri-1.json.gz` |

- 기대 집계(캡처에서 세어 고정): 패스 14, 공급 성공 107·고갈 12, flip 32·pivot 69·보류 6, BTRAN 32(dense 2), 판단 67(호출 37 = FT 36 + pivot 재분해 1), round factor 12, 재계산 13(pivot 1·round 12), 종료 6. `05b257` 호출 49(초기화 6 제외).
- 미관측(자연 0회, 이번 비교에서 판정하지 않음): factor 반환 1·12·오류, status<-999, retry 게이트, 품질 helper 호출·요청 6, 요청 2(`k==limit`·ratio 재진입), 재계산 PInf retry, 첫 공급 -1에서의 미초기화 승인수, BTRAN count0/다수.
- 대조군 control(Crossover=0)은 push 사건 0이라 루프 입력이 없다.

## 5. 재실행 명령

- 입력 투영: `python3 tools/push_loop_rust.py --write`(controls·acceptance 선행 실행).
- 재생·비교: `cargo test -p rsolver-lp --test push_loop && python3 tools/push_loop_rust.py --compare` 종료 0. 사례·줄·필드 수를 출력한다.
- 비교기 판별력: `python3 tools/push_loop_rust.py --self-test`(trace 줄·필드 변조, 줄 누락·추가·순서 바꿈이 모두 거부돼야 함).
- Rust 변이: `python3 tools/push_loop_rust.py --mutate-rust <새 결과 경로>`(target 사본에서 `crossover_loop.rs`·`crossover_factor.rs` 동작 변이, 각 변이가 비교 실패로 거부되는지 기록).
- 회귀: `cargo test -p rsolver-lp --test push_factor && python3 tools/push_factor_rust.py --compare`, round·supplier·recompute·chain 재생 비교, `cargo test --release --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`. 기존 solver 경로는 바꾸지 않으므로 확장 설치·pytest·Netlib·P3는 제외한다.

## 6. 실행 결과

(실행 후 작성)

## 7. 판정과 인수인계

(실행 후 작성)
