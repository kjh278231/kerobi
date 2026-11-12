# M3-04c-18 재계산 Rust L1 검증 계약과 결과

## 1. 범위와 주장

- 종류: 구현 L1. raw FTRAN 반환 뒤 basic 복사(별도 버퍼) 또는 제자리 사용(별칭), 유한 하한 이동·폭·F/B/X/L/N/A/U 분류·위반수·PInf·retry를 구현한다. pivot/round의 같은 계산을 공유하며 요청을 지우기 전에서 끝낸다.
- 선행: main `484243d`(M3-04c-17 병합), branch `work/m3-04c-18`, worktree `../rsolver-m3-04c-18`. 선행 미커밋 없음. main의 미추적 `.claude/settings.local.json`은 보존한다.
- 전제: double, Threads=1, batch=0, 유한 입력 및 유한 중간값, 유효한 새 header와 bound, I>h>0. 입력 raw는 외부 FTRAN의 결과다.
- 제외: rhs 준비·FTRAN 호출/수치/입력 버퍼 변형(21), 로그·전체 work/P3, 품질 helper(15a), 보류·지원·라운드(19→20), solver 루프 연결(22), batch(23), x87. rhs/input 별칭은 외부 경계이며 Rust가 구현했다고 주장하지 않는다.

## 2. 원본과 상태 경계

- Gurobi 13.0.3, 라이브러리 SHA-256 `8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c`.
- C `../re/decomp/c/001c0070_PRIVATE0000000000134a87.c` 1897–1980, 2329–2418. 원본 확인은 main 위치에서 수행했다. asm은 선행 `bench/oracle/m3_push_recompute/source.asm` 전체4036줄 및 SHA를 사용한다.
- 시작: FTRAN 복귀 `0x1c38e3`(pivot)/`0x1c3e25`(round). 종료: 분류·retry 뒤 `0x1c3aed`/`0x1c400a`, retry 쓰기 뒤 cleanup `0x1c08d5`. round의 요청0 쓰기는 `0x1c400e`이므로 포함하지 않는다.
- 새 header는 F+0x18, bound는 열 좌표, raw/basic/width/marks는 basis 위치 좌표다. basic은 별칭이면 제자리, 별도면 raw를 복사한다. m=0은 빈 분류와 +0 PInf이며 FTRAN 호출은 외부 경계다.
- 읽기: header, lb/ub, raw, I/h, attempt/model_status. 쓰기: basic, width, marks, 위반수, PInf, retry(조건 충족 때만1). request와 입력 배열은 보존한다. PInf는 basis 위치 오름차순으로 순차 가감한다.

## 3. 비교 계약

| 필드 | 규칙 | 재구성 |
|---|---|---|
| basic·width·PInf | ±0 포함 little-endian f64 비트, 오차 없음 | 원본 raw·bound·header를 Rust에 공급하고 원본 leave와 직접 비교 |
| marks·위반수·retry·exit·request | 정확 | 원본 leave에서 추출, retry 초기0 |
| raw/basic 별칭 | 정확, 안전한 Rust enum으로 제자리/별도 버퍼 선택 | 원본 aliases.raw_basic과 실제 Rust 버퍼 주소 관계 |
| header·lb/ub 보존 | 배열 순서·비트 정확 | 원본 preserved에서 추출 |

NaN/IEEE 무한대는 제외하고 내부 sentinel I는 포함한다. 시간·전체 work·RNG 소비는 해당 계산에 없다. 원본 독립3회는 17의 manifest·controls·acceptance를 다시 확인해 재사용하며 새 원본 실행은 필요하지 않다.

## 4. 사례와 원본 자료

- `bench/oracle/m3_push_recompute/manifest.json`과 자연 arms/retry/interior 각3회·대조3회, 합성3회를 사용한다. 첫 실행의 자연8회·119행 및 합성120사례·152행을 Rust에 투영한다(반복을 사례 수로 중복 집계하지 않음).
- 자연 두 경계(pivot1/round7), 합성 두 경계 각각의 free·고정·한쪽 무한·비영 하한·±h/w±h의 인접/동률·±0·basis 역순 누적·PInf=1e10 전후와 attempt/status 교차·별칭4조합·m=0을 포함한다.
- 자연 raw/basic 별칭true·F/X·retry·정확 경계의 도달성은 미검증이며 합성 제어 증거로만 보고한다. FTRAN 수치는 외부 입력이므로 해당 구현 동등성을 주장하지 않는다.
- 새 입력·Rust trace·비교·오류 검출·회귀·환경·해시는 `bench/oracle/m3_push_recompute_rust/`에 보존한다. Rust 입력에 원본 정답은 넣지 않는다.

## 5. 재실행 명령

```bash
python3 tools/push_recompute_rust.py --write
cargo test -p rsolver-lp --test push_recompute
python3 tools/push_recompute_rust.py --compare
python3 tools/push_recompute_rust.py --self-test
python3 tools/push_recompute_rust.py --mutate-rust bench/oracle/m3_push_recompute_rust/mutations.json
```

비교는 128입력의 모든 계약 필드·순서를 확인하고 첫 차이에서 실패한다. 빈 목표·누락·중복키·변경된 비트·분류·요청·retry 등의 trace 변조와 Rust 경계/누적/요청 변이를 거부해야 한다. 소스 변이는 반드시 원복 후 정상 trace를 재생한다.

회귀: `push_recompute_check.py --controls --acceptance --self-test`, 선행 factor/supplier의 controls·acceptance·self-test, factor/supplier Rust 재생, `push_chain_oracle.py --check --self-test`, `cargo test --release --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `push_step_chain_rust.py --compare`, py_compile·diff 검사. 기존 solver/API 경로 미변경으로 확장 설치·pytest·Netlib·P3는 제외한다.

## 6. 실행 결과

구현 전. 실행 후 건수·해시·판별력·회귀 결과를 기록한다.

## 7. 판정과 인수인계

- 현재: 계약 작성, 구현 동등성 미검증.
- 다음 작업은 19 오라클(보류·지원 BTRAN·고갈·재처리 라운드)이다. 이번 작업 완료 후 인계·진행·로드맵·스프린트·설계를 갱신하고 검증 범위에서 병합한다.
- 교훈은 종료 전 `docs/lessons/`에 기록한다. 독립 검토자는 사용하지 않고 작업자가 원본·비교기·전체 diff를 직접 검토한다.
