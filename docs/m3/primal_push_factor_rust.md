# M3-04c-16 직렬 factor 제어 Rust L1

## 1. 범위와 주장

- 구현: 승인 pivot 이후 요청 선택과 batch=0 직렬 wrapper. factor·quality 수치는 외부 backend다. solver 실행 경로 연결은22, 실제 LU21·품질 수치15a·재계산17→18·batch23·x87·전체 work/P3는 제외한다.
- 기준 main `5358cf7`, branch `work/m3-04c-16`, worktree `../rsolver-m3-04c-16`. 담당 `codex-m3-04c16-20261112`, M3-04c-16·shared 점유. 선행 미커밋 없음; main 별도 미추적 설정 보존.

## 2. 원본과 상태 경계

- Gurobi13.0.3 x86-64 Linux, SHA256 `8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c`.
- C `../re/decomp/c/001c0070_PRIVATE0000000000134a87.c`1725–1997, asm ELF offset `0x1c336d–0x1c3b6e`·retry `0x1c44d5–0x1c4581`; 선행 사본 `bench/oracle/m3_push_factor/source.asm`과 추가로 저장한 `bench/oracle/m3_push_factor_rust/source.asm`. 구현 전에 직접 읽었다. 선행 사본의 생략 구간은 `tools/dc.py --asm --range`로 실제 바이너리를 읽어 보완했다.
- 시작: select 직전 또는 wrapper 진입. 입력: m·request·numeric·k/limit/q·C/E/L/U·support; wrapper status/failure·attempt·numeric_limit·period·enter와 workspace/current alpha·spike 별칭. 유효 signed 정수 합과 유한 산술만 허용한다.
- 종료: 정상 request clear, retry/status 쓰기 직후, 오류 반환, 또는 비영 요청의 **재계산 진입 직전**. 재계산을 수행한 것으로 간주하지 않는다.
- 원본 context helper의 의미는15의 명시적 대체 경계 밖이다. Rust backend는 caller가 제공한 버퍼 참조와 상태를 받는다. batch0의 workspace 임시 교체·복원과 spike 보존을 실제 참조로 검사한다.

## 3. 비교 계약

| 대상 | 규칙 | 원본·재구성 |
|---|---|---|
| 요청·호출 여부·우선순위 | 정수/boolean 정확 | 자연 판단28 + 합성 선택24 |
| 정수합 뒤 f64 변환·비용식 | 원본 asm 순서 유지, 판단 정확; 중간값 bit 주장 제외 | signed 상쇄 포함 기존24경계 |
| ret1 반복·ret12/오류·retry/status·attempt 게이트 | 사건 순서·최종 상태 정확 | 합성 wrapper31 + 자연 wrapper6 |
| alpha 교체/복원·spike 보존 | 참조 동일성·세 sentinel 버퍼 내용 보존 | 자연 별칭 + 합성 sentinel; 자연 수치 불변성 주장 제외 |
| 품질 enter·주기·요청6 재호출 | 호출 순서·정수 정확 | 합성 외부 backend 결과 |

- 원본 Threads1·Seed0의 기존 독립3회 자료를 재사용한다. 새 원본 실행 없이 controls·acceptance·self-test를 재검사한다. NaN/Inf·정수 overflow·시간/work·병렬은 제외한다.

## 4. 사례와 원본 자료

- `bench/oracle/m3_push_factor/manifest.json`이 원본·입력·도구·독립3회 캡처 해시를 고정한다. 자연 arms/retry/interior 판단28(호출6·support 생략22), 합성55(선택24+wrapper31).
- 자연 wrapper6 중 요청10 한 건은 restored 상태의 재계산 직전까지만 비교한다. 나머지5건은 clear까지 비교한다. 수치 backend의 캡처 후상태를 입력으로 주며 계산 동등성으로 세지 않는다.
- Rust 입력·trace·manifest·변조/변이·회귀 기록은 `bench/oracle/m3_push_factor_rust/`에 저장한다. expected는 원본 캡처에서만 만든다.

## 5. 재실행과 완료 기준

1. `python3 tools/push_factor_check.py --controls`, `--acceptance`, `--self-test`를 각각 실행.
2. `python3 tools/push_factor_rust.py --write` → `cargo test -p rsolver-lp --test push_factor` → `python3 tools/push_factor_rust.py --compare`.
3. 비교기 변조와 Rust 동작 변이를 실제 출력 차이로 거부한다. 누락·빈 목표·순서·별칭·retry·주기·요청 경계를 포함한다.
4. supplier69입력·chain236단계 회귀, `cargo test --release --workspace`, `cargo clippy --workspace --all-targets -- -D warnings` 통과. API/기존 실행경로 미변경으로 확장 설치·pytest·Netlib·P3 제외.
5. 전체 diff·원본을 작업자가 직접 검토하고 기록/교훈/인계를 갱신한다. 독립 검토자를 사용한 것으로 보고하지 않는다. 계약 범위 통과 뒤 로컬 main --no-ff 병합·worktree 정리·점유 해제.

## 6. 실행 결과

- 계약 `741523b`, 구현 첫 동작 `792f86c`, 오류 검출·회귀 증거 `6938f34`. 환경은 [environment.json](../../bench/oracle/m3_push_factor_rust/environment.json); 저장 재생 Python3.10·x86-64 Linux, Rust/Cargo 버전과 원본 출처는 manifest에서 연결한다.
- `crossover_factor.rs`의 `factor_select`는 기존 요청·k==limit·5m² 비용·numeric·support 우선순위를 복원했다. `factor_run`은 현재 alpha 참조 설치·반복1·복원·12/오류 우선·status/attempt 게이트·재계산 중단·quality 주기와 요청6 재호출을 구현한다. 외부 backend가 수치 결과를 제공하며 solver 실행경로는 미연결이다.
- 원본 자연 판단28·wrapper6 + 합성 선택24·wrapper31 = **89입력, 978개 말단 필드 일치**. 정수·boolean·사건 순서·별칭을 정확 비교하고 세 sentinel 값은 f64 비트 보존을 확인했다. 수치 비용 중간값·자연 벡터 내용의 비트 비교로 확대하지 않는다. [결과](../../bench/oracle/m3_push_factor_rust/comparison.json).
- 자연 wrapper5건은 clear까지, 요청10인1건은 `restored`에서 관측한 상태를 유지한 채 재계산 직전 exit로 비교했다. 실제 자연의 재계산 뒤 quality gate·clear를 이번 Rust가 실행한 것으로 세지 않았다. 품질 호출은 자연0이며 합성 backend로만 비교한다.
- 원본 독립3회 자료를 재사용했다. 새 원본 실행 **0회**. 선행 factor controls(271/104/137/대조0·합성55)·acceptance·self-test41종 종료0, supplier controls/acceptance/self-test31종과 chain check/self-test8종도 종료0. [원본 회귀](../../bench/oracle/m3_push_factor_rust/oracle-regression.json).
- 새 비교기 `--self-test` **55종 모두 거부**: 빈/누락/순서·요청/종료/상태·주기·호출/별칭·소비량·중복 키. Rust `--mutate-rust` **30종 모두 출력 차이로 거부**: ==와>=·비용>0/5m²·정수합 변환·support·alpha 설치/복원·spike·반환1/12·오류 우선·status 임계/복구·attempt/failure/numeric·재계산·품질 enter/주기/요청/누적/clamp. 전부 cargo 재생 성공 뒤 비교기가 첫 차이를 검출했고, 소스 복구 뒤89입력을 다시 통과했다. [변조](../../bench/oracle/m3_push_factor_rust/self-test.json), [변이](../../bench/oracle/m3_push_factor_rust/mutations.json).
- `cargo test --release --workspace` **132통과·실패0·기존 ignored12**, `cargo clippy --workspace --all-targets -- -D warnings` 종료0·경고0. 기존 supplier69입력·1,311필드 묶음과 chain236단계·5,144필드 비트 일치(42구간·재분해 뒤 비교 제외6 유지). [명령·로그](../../bench/oracle/m3_push_factor_rust/rust-regression.json).
- `py_compile`·전체 작업 diff 공백 검사 통과. 작업자가 원본 C/asm·구현·입력 투영·비교기를 직접 검토했다. 독립 검토자는 사용하지 않았다. 기준 결과를 Rust 출력으로 만들거나 원본 캡처/선행 도구를 바꾸지 않았다. 계약 축소 없음.
- Python API/기존 solver 경로 미변경으로 확장 설치·pytest·Netlib·P3는 제외했다. 실제 LU/FTRAN·quality 오차/Markowitz·재계산 본문·batch·x87·전체 push/work는 미검증이다. Rust numeric backend용 참조 타입은 이 경계에서 읽는 세 벡터만 표현하며 전체 원본 workspace ABI 재현을 주장하지 않는다.


## 7. 판정과 인수인계

- 작업 **구현 L1 완료**, 동등성 **명시한89입력·제어 구간 통과**. 수치 backend와 solver 루프는 별도이며 전체 primal push 완료가 아니다.
- 다음은 **M3-04c-17** 재계산 오라클. `python3 tools/session.py status` → `python3 tools/session.py claim M3-04c-17 --owner <새-owner>` → `docs/m3/primal_push_loop.md` §2.5·4.3과 `.agents/skills/oracle-capture/SKILL.md`를 읽는다. rhs FTRAN 뒤 shift·폭·분류·PInf의 두 진입점 계약을 확정하고 원본 전후 상태를 확보한다. 후속 구현18·연결22·실제LU21·batch23과 품질 수치15a를 구분한다.
- branch `work/m3-04c-16`, worktree `../rsolver-m3-04c-16`, 선행 미커밋 없음. 최종 commit/병합은 main `Merge M3-04c-16` 이력에서 확인한다. 검증 뒤 로컬 --no-ff 병합과 같은 단계에서 worktree·branch·M3-04c-16/shared 점유를 정리한다. main 미추적 `.claude/settings.local.json` 보존.
- `docs/11_progress.md`, `12_roadmap_diagram.md`, `14_sprint_plan.md`, `m3/sprint_design.md`, `handoff/M3.md` 갱신. 교훈은 `docs/lessons/tooling.md`의 외부 backend 소비량 검증 항목이다. 원격 동작·외부 업로드 없음.
