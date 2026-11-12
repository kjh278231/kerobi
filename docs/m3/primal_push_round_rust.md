# M3-04c-20 보류·지원·라운드 Rust L1 검증 계약과 결과

## 1. 범위와 주장

- 종류: 구현 L1. 기준 main `55e09d6`(19 병합), 선행 미커밋 없음. main의 `.claude/settings.local.json`은 별도 보존한다.
- double·Threads=1·batch 용량/채움/소비0. 단계 보류의 후보 prefix 쓰기·실패 증가, 패스 준비, 승인수 초기화/증가, 공급 고갈 종료/지원 해제, sqrt(m) 지원 게이트, 후보 압축, BTRAN caller의 단위 입력·희소 표시 union·dense fallback, round factor 반복·오류/retry·재계산 경계·요청 지우기를 구현한다.
- 기존 solver 경로에 연결하지 않는다. 실제 BTRAN/factor/FTRAN 수치(21), 루프 연결(22), batch(23), 초기 후보 구성·정렬·x87·전체work/P3는 제외한다. 재계산 수치/분류는18의 별도 API이며 이번 factor는 재계산 직전까지다.

## 2. 원본과 상태 경계

- Gurobi13.0.3 SHA256 `8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c`.
- C `../re/decomp/c/001c0070_PRIVATE0000000000134a87.c` 1143–1176,1594/2089,1999–2062,2226–2418을 직접 읽었다. ELF asm `bench/oracle/m3_push_round/source.asm`의 `1c3b7b–1c3db4`와 BTRAN/패스 명령은19의 해시 고정 자료를 사용한다.
- 시작/종료와 P/S/F 필드는 [19 계약](primal_push_round_oracle.md) §2를 따른다. Rust는 소유한 후보/실패/표시 배열과 frame 승인수, 요청/status/numeric/failure/retry를 사용한다. 미초기화 marks/approved는 Option으로 유지하고 사용 지점에서만 요구한다.
- BTRAN backend는 caller가 만든 실제 단위 벡터를 받아 raw count/목록/값을 반환한다. 희소 값이0이어도 목록의 행을 표시한다. dense는 값에 관계없이 support=-1·표시 clear·압축 후 빈목록 종료 또는 round gate로 간다.
- round factor는 alpha pointer swap/quality를 수행하지 않는 별도 caller다. 반환1 반복,12 retry,다른>1 오류, status<-999 복구, attempt0 failure/numeric 게이트 순서를 보존한다. 재계산 반환 뒤 요청 clear는 별도 진입점으로 제공한다.

## 3. 비교 계약

| 대상 | 규칙·재구성 |
|---|---|
| 후보·실패·cursor/deferred·support·marks·approved·요청/status/numeric/failure/retry·종료 | 배열 순서·중복·정수·형식 정확 일치 |
| count_sum·BTRAN 입력/raw 값 | ±0 포함 f64 little-endian 비트; count_sum/successes 보존 |
| 호출 순서·인자 | BTRAN e_leave, factor request/반환/상태 변경을 원본 사건과 직접 비교 |
| 자연 경계 | pass/defer/accepted/BTRAN/exhausted/support/factor/clear 입력에서 원본 다음 사건 후상태와 비교; 실제 전체 루프 연결 주장은22로 유보 |

일반 유한 수와 내부 sentinel만 입력한다. NaN/IEEE무한대·정수 overflow는 제외한다. batch0 전제를 검사한다. sqrt는 비음수 m의 f64 제곱근을 사용한다. 미사용 marks null을 성공 출력으로 만들지 않고 실제 clear/사용 시만 배열을 만든다.

## 4. 사례와 원본 자료

- 19의 자연 arms/retry/interior·대조군 및 합성106사례(보류4·패스3·고갈69·BTRAN12·wrapper18), 각 독립3회와 manifest를 재사용한다. 신규 원본 실행은 필요 없으며 재사용과 신규를 구분한다.
- 자연 승인36·보류6·고갈9·BTRAN22·round factor7 등 관측 경계를 모두 Rust 입력에 투영한다. 자연 count1 이외 BTRAN·dense·오류는 합성 증거다. 전체751사건/새207사건의 출처·연결 검사는19 검사기로 유지한다.
- 입력 평문에는 전상태와 backend 반환만 넣는다. 원본 후상태는 Python 비교기에서만 읽는다. manifest는 원본 manifest·입력·Rust/도구·결과 해시를 연결한다.
- 승인 사건은 증가 직후에만 캡처되므로 바뀌지 않는 문맥 필드는 해당 사건에서 복구하고, 승인수 전값은 패스 진입/직전 승인 사건에서 계산한다. 첫 성공은 `round_begin` 뒤 증가를 실행한다. 관측되지 않은 승인수는 null 기대값으로 대조하지 않고 결과 키에서 제외한다. Rust 입력에는 이 필드의 **관측 여부1비트**만 추가하며, 출력값은 API가 만든 상태를 읽는다. 관측 범위는 자연 승인36·고갈/라운드 경계와 합성106이며 완전 루프 상태 전달은22에 남긴다.

## 5. 재실행 명령

- 기준: `python3 tools/push_round_check.py --controls --acceptance --self-test`(157변조).
- 구현: `python3 tools/push_round_rust.py --write` → `cargo test -p rsolver-lp --test push_round` → `python3 tools/push_round_rust.py --compare`.
- 오류 검출: `python3 tools/push_round_rust.py --self-test`; `--compare --mutate-rust <새 결과경로>`는 임시 사본에서 소스 동작 변이를 빌드해 원본 후상태 차이로 거부한다.
- 회귀: recompute/factor/supplier/chain Rust 재생·원본 controls/acceptance/self-test, `cargo test --release --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, py_compile·diff 검사.
- 기존 solver/API 경로 미변경으로 확장 설치·pytest·Netlib·P3 제외.

## 6. 실행 결과

- 착수 기준19 controls/acceptance/self-test 통과(자연751사건·합성106·변조157).
- 기준 `55e09d6`, 계약 `e8d7a4d`, 첫 Rust/재생 비교 `bcdbb8f`, 진행 인계 `31f835e`. 환경은 [environment](../../bench/oracle/m3_push_round_rust/environment.json), 전체 자료 해시는 [manifest](../../bench/oracle/m3_push_round_rust/manifest.json)에 연결한다.
- `push_round_rust.py --write` → `cargo test -p rsolver-lp --test push_round` → `--compare`: **216경계·14,365필드** 순서/정수/f64 비트 일치. 자연110·합성106. pass12·첫승인4·승인32·보류10·BTRAN34·고갈78·고갈gate7·support7·wrapper25·clear7이다. BTRAN 자연22는 전부count1이며 나머지12는 합성이다.
- `--self-test`: 원본 비교 결과의 필드별 변조와 빈목록/순서/입력해시를 포함한 **4,093종** 거부. `--compare --mutate-rust bench/oracle/m3_push_round_rust/rust-mutations.json`: 소스 동작 변이 **39종 전부 출력 차이로 거부**, 빌드/테스트 실패0·생존0. 원본 소스는 수정하지 않고 `target/m3_round_mutations`의 Cargo 사본에서 실행했다. [변조](../../bench/oracle/m3_push_round_rust/self-test.json), [변이](../../bench/oracle/m3_push_round_rust/rust-mutations.json).
- `cargo test --release --workspace`: **134통과·실패0·기존 ignored12**. `cargo clippy --workspace --all-targets -- -D warnings`: 경고0. py_compile·전체diff 검사 통과. [회귀 요약](../../bench/oracle/m3_push_round_rust/workspace.json), [전체 로그](../../bench/oracle/m3_push_round_rust/workspace.log).
- 선행 recompute controls/acceptance/self-test82·factor41·supplier31·chain8종 모두 종료0. 선행 Rust 재계산138/2,958·factor89/978·supplier69/1,311·chain236단계/5,144필드 일치(42구간·재분해 제외6 유지). supplier의 세 옵션을 한 번에 준 초기 CLI 호출은 상호배타 옵션 오류로 종료2였고, 각 옵션으로 나눠 재실행해 통과했다. [실제 명령·종료코드](../../bench/oracle/m3_push_round_rust/regressions.json).
- 원본19 독립3회·manifest/해시·대조/비간섭·제어 재생 자료를 재사용했고 새 원본 실행0회다. 이번 작업자의 직접 검토를 수행했으며 독립 검토자는 사용하지 않았다. Rust 수치 backend가 없는 상태이므로 확장설치·pytest·Netlib·P3는 제외했다.

### 초기 실패 보존

첫 비교는 마지막 interior 라운드의 `clear.approved`에서 실패했다. 원본 `0x1c400a`는 요청만0으로 쓰고, 후보가0이면 승인수를 유지한 채 종료한다. Rust 초기 `round_clear`는 승인수도None으로 바꿨다. 이 쓰기를 제거했다. 또 원본 agent가 다음 패스의 clear 사건에서 승인수 관측을 생략한 것을 null 출력과 혼동한 투영 오류를 고쳐, 관측 없는 키는 비교 대상에서 명시적으로 제외한다. 기존 관측이 있는 종료의 승인수 비교는 유지하고 그 손실 변이를 직접 거부했다. [초기 소스·비교기·실패 trace](../../bench/oracle/m3_push_round_rust/history/first-clear/trace.txt)를 보존했으며 입력/원본/종료 합격 기준을 완화하지 않았다.

## 7. 판정과 인수인계

- 작업 수행 **완료**, 구현 동등성 **계약 범위 통과**.double·batch0의 자연110/합성106경계에서 제어·호출·변경/보존 상태가 일치한다. 전체 라운드 루프를 연결한 동등성 주장이 아니다.
- `crossover_round.rs`는 기존 solver 경로에서 호출하지 않는다. BTRAN/factor 수치는 외부 backend이며, BTRAN 자연count0/다수/dense·wrapper 실패의 자연 도달성·수치LU21·루프22·batch23·x87·전체work/P3·최종 해 설치는 미검증이다. 승인수 비관측 경계의 메모리 값 자체는 비교하지 않는다.
- branch `work/m3-04c-20`, worktree `../rsolver-m3-04c-20`, owner `codex-m3-04c20-20261112`,20/shared 점유. 최종commit/병합은 main `Merge M3-04c-20` 이력을 따른다. 미커밋 선행 변경 없음. main에 `--no-ff` 병합한 같은 단계에서 worktree/branch·점유를 정리한다.
- `docs/11_progress.md`·`12_roadmap_diagram.md`·`14_sprint_plan.md`·`m3/sprint_design.md`·`handoff/M3.md`를 갱신했다. 교훈은 `docs/lessons/tooling.md`의 승인수 관측 여부/보존 항목이다.
- 다음은 **M3-04c-21** 직렬 수치 backend 연결 조사. `python3 tools/session.py status` → `python3 tools/session.py claim M3-04c-21 --owner <새-owner>` → `primal_push_loop.md` §2.2/2.3/4.2/4.3과 M1 LU 계약을 읽고 FTRAN/BTRAN 목록·spike·지연 LU·재분해 번호 대응을 오라클/구현 하위 작업으로 분할한다. 이번 기준 재생은 `python3 tools/push_round_rust.py --write` → `cargo test -p rsolver-lp --test push_round` → `python3 tools/push_round_rust.py --compare`.
