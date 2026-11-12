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
| 호출 순서·인자 | BTRAN e_leave, factor request/반환/状態 변경을 원본 사건과 직접 비교 |
| 자연 경계 | pass/defer/accepted/BTRAN/exhausted/support/factor/clear 입력에서 원본 다음 사건 후상태와 비교; 실제 전체 루프 연결 주장은22로 유보 |

일반 유한 수와 내부 sentinel만 입력한다. NaN/IEEE무한대·정수 overflow는 제외한다. batch0 전제를 검사한다. sqrt는 비음수 m의 f64 제곱근을 사용한다. 미사용 marks null을 성공 출력으로 만들지 않고 실제 clear/사용 시만 배열을 만든다.

## 4. 사례와 원본 자료

- 19의 자연 arms/retry/interior·대조군 및 합성106사례(보류4·패스3·고갈69·BTRAN12·wrapper18), 각 독립3회와 manifest를 재사용한다. 신규 원본 실행은 필요 없으며 재사용과 신규를 구분한다.
- 자연 승인36·보류6·고갈9·BTRAN22·round factor7 등 관측 경계를 모두 Rust 입력에 투영한다. 자연 count1 이외 BTRAN·dense·오류는 합성 증거다. 전체751사건/새207사건의 출처·연결 검사는19 검사기로 유지한다.
- 입력 평문에는 전상태와 backend 반환만 넣는다. 원본 후상태는 Python 비교기에서만 읽는다. manifest는 원본 manifest·입력·Rust/도구·결과 해시를 연결한다.

## 5. 재실행 명령

- 기준: `python3 tools/push_round_check.py --controls --acceptance --self-test`(157변조).
- 구현: `python3 tools/push_round_rust.py --write` → `cargo test -p rsolver-lp --test push_round` → `python3 tools/push_round_rust.py --compare`.
- 오류 검출: `python3 tools/push_round_rust.py --self-test`; `--compare --mutate-rust <새 결과경로>`는 임시 사본에서 소스 동작 변이를 빌드해 원본 후상태 차이로 거부한다.
- 회귀: recompute/factor/supplier/chain Rust 재생·원본 controls/acceptance/self-test, `cargo test --release --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, py_compile·diff 검사.
- 기존 solver/API 경로 미변경으로 확장 설치·pytest·Netlib·P3 제외.

## 6. 실행 결과

- 착수 기준19 controls/acceptance/self-test 통과(자연751사건·합성106·변조157).
- 구현·비교·회귀 결과는 실행 뒤 기록한다.

## 7. 판정과 인수인계

- 작업 수행 중. 구현 동등성 미검증.
- branch `work/m3-04c-20`, worktree `../rsolver-m3-04c-20`, owner `codex-m3-04c20-20261112`, M3-04c-20 점유. 다음은 계약 커밋 뒤 Rust/API·입력 전용 재생 구현이다.
- 후속21 수치 backend 조사·22 루프 연결·23 batch. 종료 전 진행/로드맵/스프린트/인계·교훈을 갱신한다.
