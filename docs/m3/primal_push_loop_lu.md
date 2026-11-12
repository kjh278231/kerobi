# M3-04c-24 실제 Rust 직렬 backend를 push 루프에 설치: 검증 계약과 결과

작성 기준은 [스프린트 검증 기준](../15_sprint_verification.md)과 [계약 양식](../templates/sprint_acceptance.md)이다. §1–§5는 구현과 비교를 실행하기 전에 썼다.

## 1. 범위와 주장

- 작업 종류: **구현 연결**. 기준 main `6340a20`(M3-04c-22 병합), branch `work/m3-04c-24`, worktree `../rsolver-m3-04c-24`. 선행 미커밋 변경 없음. main의 미추적 `.claude/settings.local.json`은 무관하며 보존한다. 사용자의 "다음 M3 스프린트 진행해 너가 작업자야" 지시로 이 세션이 작업자로 수행한다(작업자 선택 질문 생략). 점유 owner `claude-m3-04c24-20261112`.
- 완료할 동작: 22의 `push_loop`에 tape 대신 **실제 Rust 직렬 수치 backend**(`crossover_backend::SerialBackend`)를 넣는다. backend는 21b–21h의 `crossover_lu::LuState`(FTRAN `2207de`·BTRAN `221173`·FT 갱신 `05b257` 요청 0·재번호)와 `crossover_refactor::refactor`(`036ade`+C:2570–2760, `1da0b2` 블록 단계 포함)를 다음 경계에 연결한다.
  - 공급 FTRAN(`09d008`): spike 인자가 있으면(지원 0) wrapper clear 뒤 save=1·spike F1c8, 없으면 save=0. raw = 출력 header(count·목록·값).
  - 지원 BTRAN(`09e376`→`221173`): cache F+0x1d0, F+0x200 work 0. raw = 출력 header.
  - `05b257` 요청 0: header를 넣고 FT 갱신(이탈 위치 S+0x2f8). 요청≠0: S+0x2c0 블록 상태로 `refactor`, 이어 LuState 재번호(C:2006–2090)와 같은 순열을 루프에 돌려준다.
  - 재계산 rhs FTRAN(`09cf9b`, count -1, save 0, spike 없음): raw = dense 출력 m값.
  - factor 판단 통계를 backend 상태에서 만든다: F+0x18c·F+0x188·F+0x184는 LuState 값, S+0x46c는 시작값 고정(루프 안에서 쓰는 곳 없음, 자연 불변), **F+0x190**은 FT 갱신마다 작업 열 S+0x210(= wrapper가 넣은 현재 alpha)의 `count+1`(dense면 1+|v|>S+0x3a8 개수)을 더하고(`05b257` C:144–251) 재분해에서 0(C:1379), **F+0x1e0 = m + F+0x58[m]**, **F+0x1e8 = F+0xb0[m] + m**(재분해 끝 C:1978–1986; F+0xb0[m]은 압축된 U 행 끝 = U 비영 수로 둔다).
- 초기 backend 상태: 루프 전 유일한 수치 연산인 초기화 factor(요청 10, 자연 6사례 모두 `lu:enter` 1회)를 21h 첫 G와 같은 입력(호출 전 header·S+0x2f8·vstat, ZERO, Markowitz, 할당 4필드, 첫 `036ade` 진입 블록 상태)으로 Rust가 만든다. 그 뒤 원본 factor·수치 출력 주입은 **0회**다.
- 루프 시작 push 상태(모델·후보·basis·rhs 등)는 22와 같은 `supply:enter`/`pass:enter` 입력이다. Rust 초기화 factor의 재번호 header가 이 시작 header와 같아야 시작한다(같지 않으면 실패).
- `PushLoopBackend::factor`는 push 상태(header·S+0x2f8·vstat)와 작업 열을 받도록 서명만 넓힌다. 루프 제어 동작은 바꾸지 않는다.
- 제외와 후속: 품질 helper `089f61`(15a, 자연 0회; backend는 호출되면 실패), factor 반환 1·12·오류와 FT 실패·재할당(`cd9cc`·`1f2069`), pricing 객체, batch(23), x87, 종료 검사·콜백, work·시간, 초기화의 나머지(후보 구성·정렬, C 1143 이전)와 해 설치, 기존 solver 경로 연결. 수치 카운터 S+0x178/0x184/0xf0은 자연 경로에서 `05b257`이 바꾸지 않으므로 그대로 둔다(바꾸는 실패 경로는 범위 밖).

## 2. 원본과 상태 경계

- Gurobi 13.0.3 `libgurobi130.so`, SHA-256 `8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c`. 주소는 ELF file offset.
- `../re/decomp/c/000f15c0_PRIVATE000000000005b257.c`: 요청 0 FT 경로 C:144–1335(F+0x190 누적 C:144–251, F+0x18c++ C:1327), 재분해 C:1337–(F1f0 = F190/k, F190 = 0 C:1372–1379, S+0x2c0++ C:1399), 재번호 C:2006–2090, L/U 통계 C:1978–1986, 공통 출구 C:2765–2770. 나머지 조각의 근거는 21b·21d·21f·21g·21h 문서 §2.
- 캡처 확인(21e run 1, 6사례): 루프 전 수치 연산은 `lu:enter` 요청 10 한 번. 공급 FTRAN 107회는 save = spike F1c8 여부(1/0 = 51/56), 재계산 13회는 count -1·save 0·spike 없음이고 rhs는 바뀌지 않으며 raw = `ftran:leave` dense 출력, 공급 raw = 출력 header(107/107 일치). BTRAN 32회 모두 cache F1d0·x200 0. BTRAN 출력 header의 진입 count는 dense Lᵀ에서 입력 후상태 count로만 옮겨지고 출력에는 쓰이지 않으므로 0을 넣는다. factor:leave의 numeric·failure·status·q는 모든 호출에서 진입과 같다.
- 시작 시점: 초기화 `lu:enter`(factor 생성)와 첫 `roundctl:pass:enter`(루프). 종료 시점: `roundctl:end`.

## 3. 비교 계약

22의 canonical 사건열(줄·순서·필드)을 그대로 쓰고 두 가지를 더한다.

| 줄 | 비교 필드 | 원본 출처 | 규칙 |
|---|---|---|---|
| `init`(사례 첫 줄) | 초기화 재분해 뒤 header·S+0x2f8·vstat | 초기화 `lu:leave` | 정확 |
| `fcall`에 `stats` | 호출 뒤 k(F18c)·limit(F188)·q(S+0x46c)·C(F190)·E(F184)·L(F1e0)·U(F1e8) | `factor:leave.factor_ctl.state` | 정확 |

- 22에서 tape이던 값(공급 FTRAN raw, BTRAN raw, 재분해 순열, 재계산 raw)은 이제 Rust가 만든 값이며, 각각 `supply`(alpha)·`btran`(raw)·`fcall`(hdr2·vstat·leave2)·`recompute`(basic·width·marks·PInf) 줄에서 원본과 비트/정확 비교된다. tape 입력 파일에는 수치 출력이 없다(입력 줄: case·model·cfg·state·round·stats(초기화 뒤)·init).
- 줄 수·순서·태그가 원본과 같아야 하며, backend가 실패(미구현 경로·unknown 셀 읽기)하면 그 사례는 실패다.
- 수치는 ±0 포함 f64 비트 비교. 허용오차 비교 없음.
- 원본 재현성: 21e 캡처(독립 3회·비간섭·변조 54 거부)와 21h 블록 상태 캡처(독립 3회·21e 비간섭)를 재사용한다. 도구는 두 검사기의 controls·acceptance를 먼저 통과시킨다. 새 원본 실행은 하지 않는다.

## 4. 사례

22와 같은 자연 6사례(arms·retry·interior·adlittle·scsd1·capri), 집계 고정값도 22 §4와 같다(패스14·공급119·단계107·BTRAN32(dense2)·판단67·`05b257` 루프 호출49·재계산13·round12·종료6) + init 6. 수치 분기: FT 갱신 36(행 eta 포함), 재분해 13(초기화 6 + 루프 13 = 19, capri 23회째 B 단계 블록), FTRAN A2·spike dense/sparse, BTRAN 희소/dense Lᵀ·cache.

미관측(판정 안 함): 22 §4의 목록, FT cache 재사용(F1d8=r), BTRAN eta>0, 품질 helper.

## 5. 재실행 명령

- 입력: `python3 tools/push_loop_rust.py --lu --write`(21e·21h 검사 선행).
- 재생·비교: `cargo test -p rsolver-lp --test push_loop_lu && python3 tools/push_loop_rust.py --lu --compare` 종료 0.
- 판별력: `python3 tools/push_loop_rust.py --lu --self-test`(trace 변조·구조 변조 전부 거부), `--lu --mutate-rust <새 경로>`(backend·루프 변이; 자연 경로를 지나는 변이는 거부되어야 하며 생존은 캡처로 분류).
- 회귀: 22 tape 재생(`--write`·`push_loop`·`--compare`), 21h `push_lu_rust.py --block` 재생 비교, `cargo test --release --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`. 기존 solver 경로는 바꾸지 않으므로 확장 설치·pytest·Netlib·P3는 제외한다.

## 6. 실행 결과

- 기준 main `6340a20`, 계약 `778ae39`, 구현·첫 비교 `19e693f`. x86-64 Linux(WSL2), rustc stable. Gurobi 새 실행 0회. 자료 해시는 [manifest](../../bench/oracle/m3_push_loop_lu/manifest.json)에 고정했다.
- 구현: `crates/lp/src/crossover_backend.rs`(`SerialBackend`: `initial`·`PushLoopBackend` 구현, F190·F1e0·F1e8 통계), `crossover_loop.rs`(`factor`가 push 상태와 작업 열을 받음, `LoopEvent::Factor`에 호출 뒤 통계), `crates/lp/tests/loop_io/mod.rs`(22·24 공통 입력·canonical 줄), `crates/lp/tests/push_loop_lu.rs`, `tools/push_loop_rust.py --lu`. 22 tape 재생은 공통 모듈로 옮긴 뒤에도 입력·trace가 바뀌지 않았다(719줄·5,502필드 그대로 일치).
- 착수 검사: 21e `push_lu_check` controls·acceptance와 21h `push_block_check` controls·acceptance를 통과한 뒤 투영했다. 입력 파일은 사례마다 case·model·cfg·state·round·lu 6줄이며 수치 출력 tape이 없다.
- `python3 tools/push_loop_rust.py --lu --write` → `cargo test -p rsolver-lp --test push_loop_lu` → `--lu --compare`: **6사례·725줄·5,575필드 전부 일치**(첫 비교부터). 줄 구성은 22의 719줄 + init 6, 필드는 22의 5,502 + init 24 + fcall 통계 49. Rust 초기화 factor의 재번호 header는 6사례 모두 루프 시작 header와 같았다. 수치 경로: 공급 FTRAN 107(spike 저장 51)·지원 BTRAN 32(dense 2)·FT 갱신 36·재분해 19(초기화 6 + 루프 13: pivot 1·round 12, capri 블록 단계 포함)·재계산 FTRAN 13. [비교 출력](../../bench/oracle/m3_push_loop_lu/compare.txt).
- 판별력: `--lu --self-test` **변조 108종 전부 거부**(줄 태그·필드별 첫 값 101, 구조 6, 머리 1). [기록](../../bench/oracle/m3_push_loop_lu/self-test.txt).
- Rust 변이 `--lu --mutate-rust`: **44종(backend 19 + 22 루프 25) 중 거부 31(시간 초과 1), 생존 13**. [기록](../../bench/oracle/m3_push_loop_lu/rust-mutations.json). 거부: F190 누적 생략·+1 생략·재분해 0 생략, F1e8 식·갱신 생략, S+0x46c, wrapper clear 생략, 공급 save·spike 인자, FT 이탈 위치, 블록 단계 끔(capri 23회째 재분해 header 차이), 재계산 FTRAN save, 작업 열 미전달, 22 루프 변이 18종(그중 `exhausted-skip`은 tape 고갈이 없어 루프가 끝나지 않아 120초 시간 초과). 생존은 모두 자연 자료가 그 값을 읽지 않는 경우다.
  - `ft-header-stale`: FT 갱신·FTRAN·BTRAN은 F+0x18을 읽지 않는다(LuState의 header는 재번호 출력과 스냅숏용). 동치.
  - `refactor-no-lu-renumber`·`refactor-prev-none`: 재분해 뒤 spike·cache·F1d8·scratch의 차이. 다음 spike FTRAN은 wrapper clear로 spike를 먼저 지우고, spike 없는 FTRAN은 F1c8을 읽지 않으며, scratch는 호출 사이 늘 0이고, cache는 FT의 F1d8=r 재사용(자연 0회)에서만 읽힌다.
  - `refactor-block-no-raise`·`initial-block-no-raise`: S+0x2c0은 `1da0b2` C:1494–1510에서 `> 20`일 때만 읽히는데 push는 1에서 시작해 사례당 재분해가 최대 4회다.
  - `btran-no-cache`: cache는 BTRAN 출력에 영향이 없고 F1d8=r 재사용(자연 0회)에서만 소비된다.
  - 22와 같은 루프 생존 7종(`no-clear-request` 동치, `violations-not-saved`·`request-not-synced`·`period-not-saved`·`updates-stale`·`resume-restarts`·`resume-no-period`)은 22 §6의 분류와 같다.
- 회귀(`bench/oracle/m3_push_loop_lu/regressions.json`, 12명령 모두 종료0): 22 tape 재투영(입력 파일 불변 `git diff --exit-code`)·재생·비교, factor·round·supplier·recompute·chain 재생 비교, 21h `push_lu --block` 재생성·비교, 21e controls·acceptance, py_compile, `cargo test --release --workspace` **145통과·0실패·ignored12**(새 `push_loop_lu` 1), `cargo clippy --workspace --all-targets -- -D warnings` 0, `git diff --check`. 변이 도구의 시간 초과 처리 수정 뒤 py_compile·22/24 비교·self-test·clippy를 다시 실행해 통과했다.
- 실행하지 않은 검사: 기존 solver 경로·Python 경계 미변경이라 확장 설치·pytest·Netlib·P3는 제외했다. 독립 검토자는 사용하지 않았다.

## 7. 판정과 인수인계

- 작업 수행 **완료**, 구현 동등성 **계약 범위 통과**. 자연 6사례에서 Rust가 초기화 factor 입력만 받아 초기 factor를 만들고, 그 뒤 `push_loop`이 실제 Rust 수치 backend(FTRAN·BTRAN·FT 갱신·재분해·블록 단계·재번호·rhs FTRAN)와 그 통계로 첫 패스부터 종료까지 원본과 비트/정확 일치한다. 원본 수치 출력·factor 재주입 0회.
- 남은 한계: 초기화의 나머지(첫 factor 이전 상태 구성·후보 구성·정렬, C 1143 이전)와 해 설치(C 2420 이후)는 입력/비교 밖이며 `push_loop`은 기존 solver 경로에서 호출하지 않는다. 품질 helper(15a), batch(23), x87, 종료 검사·콜백, work·시간 미구현. 자연 미관측: factor 반환 1·12·오류와 FT 실패·재할당, FT cache 재사용(F1d8=r), BTRAN eta>0, S+0x2c0>20, 그리고 22 §7의 루프 미관측 경로. 수치 카운터 S+0x178/0x184/0xf0과 F+0x1f0은 갱신하지 않는다(자연 경로에서 불변, F1f0은 루프가 읽지 않음).
- branch `work/m3-04c-24`, worktree `../rsolver-m3-04c-24`, owner `claude-m3-04c24-20261112`. 최종 commit/병합은 main `Merge M3-04c-24` 이력을 따른다. 미커밋 선행 변경 없음.
- 갱신: `docs/11_progress.md`·`12_roadmap_diagram.md`·`14_sprint_plan.md`(24 완료)·`m3/sprint_design.md`·`handoff/M3.md`. 교훈은 `docs/lessons/original.md`(F190·F1e0·F1e8 규칙), `tooling.md`(Ghidra 10진 오프셋, tape→실제 backend 교체와 기본 인자 함정).
- 다음: **M3-04c-23**(batch 조사) 또는 초기화·해 설치를 이어 solver 경로에서 push를 부르는 연결(04f 계열, 후보 구성 `C 1143 이전` 조사 포함). 기준 재검사 `python3 tools/push_loop_rust.py --lu --write && cargo test -p rsolver-lp --test push_loop_lu && python3 tools/push_loop_rust.py --lu --compare`.
