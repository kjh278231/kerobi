# M3-04c-16 직렬 factor 제어 Rust L1

## 1. 범위와 주장

- 구현: 승인 pivot 이후 요청 선택과 batch=0 직렬 wrapper. factor·quality 수치는 외부 backend다. solver 실행 경로 연결은22, 실제 LU21·품질 수치15a·재계산17→18·batch23·x87·전체 work/P3는 제외한다.
- 기준 main `5358cf7`, branch `work/m3-04c-16`, worktree `../rsolver-m3-04c-16`. 담당 `codex-m3-04c16-20261112`, M3-04c-16·shared 점유. 선행 미커밋 없음; main 별도 미추적 설정 보존.

## 2. 원본과 상태 경계

- Gurobi13.0.3 x86-64 Linux, SHA256 `8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c`.
- C `../re/decomp/c/001c0070_PRIVATE0000000000134a87.c`1725–1997, asm ELF offset `0x1c336d–0x1c3b6e`·retry `0x1c44d5–0x1c4581`; 고정 사본 `bench/oracle/m3_push_factor/source.asm`. 구현 전에 직접 읽었다.
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

착수 계약. 아직 구현·비교 미실행.

## 7. 판정과 인수인계

작업 진행 중, 동등성 미검증. 다음은17 재계산 오라클이다.
