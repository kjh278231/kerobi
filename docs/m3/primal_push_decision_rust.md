# M3-04c-6 primal push 단계 판단 Rust 구현

작성 기준은 [스프린트 검증 기준](../15_sprint_verification.md)과 [계약 양식](../templates/sprint_acceptance.md)이다.

## 1. 범위와 주장

- 작업 종류: 구현. 기준선 main `7038dd5`(M3-04b-2 병합), branch `work/m3-04c-6`, worktree `../rsolver-m3-04c-6`, 미커밋 선행 변경 없음.
- 완료할 동작: double primal push `134a87`의 feasible basis 한 단계 판단을 Rust(`crates/lp/src/crossover.rs`)로 복원한다.
  1. ratio 전: 진입 열 j의 r·dL·dU 계산, 방향식, 방향 1이면 FTRAN 열 부호 반전.
  2. ratio 뒤: 거리 flip, 거부(방향식 결과로 arm 선택, theta=I), pivot 승인, 보류.
  3. 승인 pivot의 이탈 변수 상태(-1·-2·-4).
  4. 루프 준비 상수 `0.01h`·`100h`·`-100h`·`h+h`(h=`S+0x380`).
- 포함 분기: 위 전부를 asm 비교 형태 그대로 구현한다. 실행 증거가 있는 분기는 §4, 없는 분기는 §7 미검증 목록으로 나눈다. 구현했다고 미관측 arm을 통과로 세지 않는다.
- 명시적 거부: ratio가 -1을 반환하거나 theta=I인 경우의 반대 방향 재호출(`0x1c28ee`→`0x1c497e`)은 이 함수 밖 흐름이라 `Err(Recall)`로 거부한다.
- 제외: 위반 basis 방향식(C 1333–1454줄), infeasible ratio `054eef`·`056973`, batch, x87 `256781`, 보류 뒤 동작(`0x1c3b7b`), pivot 적용(basis 교환·basic 값·rhs 보정·factor), push 루프 연결, work. 후속은 M3-04b-3(미관측 arm 오라클)·M3-04c-5(재진입)·push 연결 작업.

## 2. 원본과 상태 경계

- Gurobi 13.0.3 `libgurobi130.so` SHA-256 `8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c`.
- C `../re/decomp/c/001c0070_PRIVATE0000000000134a87.c` 1209–1240줄(r·dL·dU·방향식), 2060–2177줄(거리 flip·거부·승인·이탈 상태). 판단식과 세 슬롯 출처는 [M3-04b-2 §2](primal_push_decision_oracle.md#2-원본과-상태-경계)를 따른다.
- 이번에 asm으로 다시 확인한 비교 형태(NaN 입력은 범위 밖이지만 asm 분기 형태를 그대로 쓴다):
  - 방향식 `0x1c26ab–0x1c2733`: 첫 절 `10*dL < dU && -(r*dL) < e`. 둘째 절은 `!(dL > 10*dU && e > dU*r) && 0 < r`(`jbe`가 unordered에서 r 검사로 간다).
  - 거리 flip `0x1c290d–0x1c293b`: 두 방향 모두 `gap-1e-10 <= theta`(방향 0 gap=dU `jb`, 방향 1 gap=dL `jae`, 모두 ordered). C의 `!(theta < dL-1e-10)`은 NaN에서만 다르다.
  - 거부 `0x1c294c–0x1c2989`, `0x1c494e–0x1c4979`: arm은 `r13b`.
  - 승인 `0x1c2a84–0x1c2aaf`: `1e-6 > pivot` 또는 `-1e-5 > theta*pivot`이면 `0x1c3b7b` 보류.
  - 이탈 상태 `0x1c2bc2–0x1c2d7f`: 고정폭 `ub-lb <= S+0x3b0`. 부호는 `0 > a`면 음수 arm(±0은 비음수). `B`는 `(t+1)*K > abs(b/a-t)`이면 -1, 아니면 -2. `L`(a<0)은 `K > abs(a)*t`이면 -1, 아니면 -2. `A`는 `(t+1)*K > abs((b-w)/a-t)`이면 -2. `U`(a≥0)는 `K > abs(a)*t`이면 -2.
  - rodata: `1e-10`(`0x2a61308`), `1e-6`(`0x2a61300`), `-1e-5`(`0x2a613f0`), `10`(`0x2a61348`), `1`(`0x2a612d0`), `100`(`0x2a61358`), `-100`(`0x2a613f8`). 루프 준비 `0x1c1f09–0x1c1fae`는 `100*h`, `-100*h`, `h+h`를 쓴다.
- 시작 경계: 후보 공급 `supply:leave`(candidate≥0) 직후. 입력은 push 진입 workspace cost·lb·ub(`push:enter.model`), 같은 단계 `ratio:enter.state`의 x·slack·header·허용오차, 반전 전 FTRAN 열(`supply:leave.alpha`), ratio 행 상태(`ratio:enter.basic/width/marks`), ratio 출력(`ratio:leave.chosen/theta/pivot`).
- 종료 경계: 단계 종류 결정(`step:enter`) 또는 보류, pivot이면 이탈 상태 기록(`step:applied`).

## 3. 비교 계약

Rust에는 원본 캡처의 원시 필드만 넣는다. Python 판단 모델(`primal_push_decision_check.decide`)의 계산값(r·방향·반전 alpha·상수)은 입력으로 쓰지 않는다.

| Rust 출력 | 원본 기준 | 규칙 |
|---|---|---|
| 방향 | `ratio:enter.selected.direction` | 정수 정확 |
| 반전 뒤 alpha | `ratio:enter.alpha.values`(인덱스는 입력과 같음) | f64 비트 |
| r·dL·dU·e·100h·-100h·2h | `step:enter.frame` | f64 비트. frame이 있는 단계만 |
| 단계 종류 | `step:enter.kind`, 다음 사건이 `step:enter`가 아니면 보류 | 정확(Rust 거리·거부 flip은 `flip`) |
| theta | `step:enter.theta` | f64 비트 |
| 이탈 상태 | `step:applied.state.vstat[header[row]]` | 정수 정확. pivot만 |
| 연결: Rust 반전 alpha로 돌린 `feasible_ratio` 위치·theta·pivot | `ratio:leave` | 비트. 04c-6 판정과 따로 기록 |

NaN 입력은 없다. I(`1e30`)·±0은 비트 그대로다. 시간·병렬 분기는 없다(Threads=1). 원본 재현성은 기존 검사(`primal_push_step_check.check`, `primal_push_decision_check.validate_new`)가 순서별 독립 3회 일치를 확인한 뒤 첫 실행을 투영한다.

판별력: M3-04b-2 모델 변조 20종을 Rust 소스 변조로 옮겨 원본 대조가 거부하는지 기록한다. 기대는 오라클과 같은 분류(판단 3·r 비트 1·상수 값 4 거부, 12 생존)다. 오라클이 거부한 변조를 Rust 하네스가 놓치면 하네스 결함으로 보고 고친다. 상수 값 거부는 판단 판별로 세지 않는다.

## 4. 사례와 원본 자료

| 사례 | 실제/합성 | 분기 | 원본 자료 |
|---|---|---|---|
| `interior.lp` primal·dual 각 9단계(04b-1, 각 3회) | 실제 | 방향 0(r=+0), 거리 flip 2, 승인 7, 이탈 `N` | `bench/oracle/m3_primal_push_step/` |
| `face5.lp` primal·dual 각 11단계(04b-2, 각 3회) | 실제 | 방향 1×3(r>0 절), r≠0 7, 음수 theta 승인, 이탈 `N`·`L`(a≥0), frame | `bench/oracle/m3_push_decision/face5-*.json` |
| `interior.lp` 비간섭 1회(04b-2) | 실제, 단일 실행 | 04b-1 primal과 같은 9단계의 frame(dL·dU 비영) | `interior-primal-noninterference.json`. 보조로 따로 센다 |

기대 집계: 대상 40단계(방향 1 6, pivot 36, flip 4, 보류 0), frame 22단계. 보조 9단계는 frame 9. Gurobi는 새로 실행하지 않는다(저장 기준 재생).

## 5. 재실행 명령

```bash
python3 tools/primal_push_decision_rust.py --write          # 원본 캡처 → Rust 입력 텍스트
cargo test -p rsolver-lp --test push_decision && python3 tools/primal_push_decision_rust.py --compare
python3 tools/primal_push_decision_rust.py --self-test      # 비교기 변조 거부
python3 tools/primal_push_decision_rust.py --mutate-rust <새 경로>.json   # Rust 소스 변조
```

## 6. 실행 결과

- 환경: x86-64 Linux(WSL2), rustc 1.99.1. Gurobi는 실행하지 않았다(저장 기준 재생). 투영 도구는 원본 C 해시도 검사하므로 저장소 옆 `../re/decomp/c/`가 있어야 한다.
- 원본 자료: 04b-1은 `primal_push_step_check.check`, face5는 `primal_push_decision_check.validate_new`로 해시·출처·순서별 독립 3회 일치를 먼저 검사한 뒤 첫 실행을 투영했다. 보조 비간섭 캡처는 agent·바이너리·모델 해시와 frame을 뺀 사건이 04b-1 primal-1과 같음을 검사했다. 기존 도구와 캡처는 바꾸지 않았다.
- Rust 입력 `bench/oracle/m3_push_decision_rust/input.txt`: 49단계(대상 40·보조 9), 본문 SHA-256 `3c5d7588180b1bad20c3d89846ff495559543e74535b04e6ddcb30f3fdc9a9a8`(파일 첫 줄). 대상 40단계의 고유 입력은 20개다. 두 순서의 단계 입력이 같기 때문이다(04b-1 9, face5 11).
- 재생 비교: `cargo test -p rsolver-lp --test push_decision && python3 tools/primal_push_decision_rust.py --compare` 종료 0. 49단계 전부 일치. Rust 재호출 거부 0, skip·누락 0. `--compare`는 판단 불일치면 종료 1, 연결 불일치만 있으면 종료 3이다.
  - 집계: 대상은 방향 1 6, pivot 36, flip 4(모두 거리), 보류 0, frame 22. 보조는 pivot 7, flip 2, frame 9.
  - 필드: 방향, 반전 뒤 alpha 전체, 단계 종류, theta, 이탈 상태(pivot 43건). frame이 있는 31단계는 r·dL·dU·e·100h·-100h·2h 비트도 비교했다.
  - 판별 필드: theta는 거부가 없으면 `push_step`이 쓰지 않아 `ratio:leave` 값을 그대로 내보낸다. 관측 단계에 거부가 없으므로 theta 일치는 Rust 판별 증거가 아니다(`reject-keep-theta` 생존과 같은 사실). 반전 alpha도 방향 0에서는 입력 그대로라 판별은 방향 1 6단계(고유 3)뿐이다. 독립 판별 필드는 방향·단계 종류·이탈 상태, frame 단계의 r·dL·dU·상수, 방향 1의 반전 alpha다.
  - Rust는 입력 줄의 순수 함수이므로 독립 증거는 대상 고유 입력 20개(04b-1 9, face5 11)와 보조 9단계다.
- 연결: Rust가 반전한 열로 돌린 `feasible_ratio`의 위치·theta·pivot이 49단계 모두 `ratio:leave`와 비트 일치했다. 04c가 보지 못한 방향 1 반전 열 6건과 음수 theta를 포함한다. 04c-6 판정과는 따로 센다.
- 비교기 변조: `--self-test` 종료 0. 방향, r·dU·2h 비트, 단계 종류, theta, 이탈 상태, alpha 부호, 재호출 거부, 사례 누락, 오래된 입력 해시, 입력 투영 변경 12종을 모두 거부했다. ratio theta 변조는 연결 불일치 1건으로 보고했다.
- Rust 소스 변조([기록](../../bench/oracle/m3_push_decision_rust/rust-mutation.json)): 04b-2 모델 변조 20종 중 19종을 옮겼다. `reject-arm-current`는 옮기지 않았다. Rust 방향은 방향식 결과 하나뿐이고, 현재 `S+0x328`과 다른 경우는 `Recall`로 거부하는 재호출 경로에만 있기 때문이다. Rust 전용 5종을 더해 24종을 돌렸다.
  - 거부 9종. 오라클과 같은 8종과 Rust 전용 1종이다.
    - 판단으로 거부 4종: `0<r`→`<=`, `-1e-5`→0, `L`(a≥0)→-2, alpha 반전 생략(이탈 상태에서 검출).
    - r 비트로 거부 1종: 합산 역순.
    - 상수 값으로 거부 4종: e, 거부 상수 두 종, 2h. frame 대조와 같은 정보이며 판단 판별로 세지 않는다.
  - 생존 15종.
    - 오라클 생존 12경계 중 옮긴 11종: 방향식 계수 10과 `<`, `-(r*dL)` 부호, 거리 flip의 `1e-10`·`<=`, 거부 arm 교환, pivot `1e-6`, 고정폭 `<=`, a<0 기본 -1, `X`→-4, `U`의 `<`.
    - Rust 전용 4종: 진입 slack의 dL·dU(진입 slack 단계 0건), ±0 pivot의 부호 arm, 거부 theta=I, theta=I 재호출 검사.
- 회귀: `primal_push_ratio_rust.py --compare`(04c 47사례), `primal_push_decision_check.py --old` 종료 0. `cargo test --release --workspace`는 종료 0(lp 36, push_decision 1, push_ratio 1, mip 66, clique 5, retry_l1 1, driver 4, model 3). `cargo clippy --workspace --all-targets -- -D warnings`도 종료 0이다. clippy 지적 두 가지를 반영했다.
  - `r = r - a*c`를 `r -= a*c`로 바꿨다. 같은 IEEE 연산이다.
  - 비음수 `A`·`U` arm을 match guard로 바꿨다. 분기 결과는 같다.
  - 반영 뒤 재생과 변조를 다시 실행했고, 위 결과가 그 실행이다.
- main 병합: 작업 중 main이 `8151f8a`(M2-03d-3 병합)로 움직여 branch에 병합했다. 병합 트리에서 Rust 재생·`--compare`·`--self-test`·04c `--compare`를 다시 실행해 종료 0이었다. `git diff 7038dd5 8151f8a -- crates python`이 비어 있어 workspace 테스트·clippy·변조는 다시 돌리지 않았다.
- 실행하지 않은 검사: Python 확장 재빌드·pytest. 새 함수는 호출자가 없고 `crates/py`·Python API를 바꾸지 않았다. 공유 `.venv` 규칙상 main worktree 빌드 대상이다.
- 해시: `crates/lp/src/crossover.rs` `28cc34cb6777baf745791fc5ae1cb7b3119b1f7981a019958a121de2515cc4aa`, `crates/lp/tests/push_decision.rs` `2894ae8f991077c3a649448638c91b573b18d319e98becefd269a32c31e0f65f`, `tools/primal_push_decision_rust.py` `1450ed74f1a0d37d32417a161df271a8086e8deda06bec7aa7ce5f5437698ba3`.

## 7. 판정과 인수인계

- 이후 변경: M3-04b-3이 같은 재생 도구와 입력에 arm 사례 50단계를 더했다. 위 §6의 입력 해시·도구 해시·변조 수는 이 작업 당시 값이다. 현재 값은 [M3-04b-3 §6](primal_push_arms_oracle.md#6-실행-결과)을 따른다.

- 작업 수행: 완료.
- 동등성: 계약 범위 **통과**(저장 기준 재생, 비트 비교). 실제 LP 2개×2순서(순서별 원본 3회)의 40단계(고유 입력 20)에서 방향·단계 종류·이탈 상태와 방향 1의 반전 alpha가 원본과 같다. frame이 있는 22단계(보조 9 추가)에서는 r·dL·dU와 루프 상수도 같다. theta는 거부 미관측이라 통과 값이며 판별 근거가 아니다.
- 실행 증거가 있는 분기: 방향 0(r<0·r=0)과 방향 1(r>0 절), 구조 열 진입, 거리 flip(방향 0), 승인 pivot(음수 theta 포함), 이탈 `N`(a 양·음)과 `L`(a≥0).
- 미검증(구현은 asm대로 했으나 원본 실행 증거 없음):
  - 방향식: 첫 절로 만든 방향 1, 진입 slack 열.
  - flip·거부·보류: 방향 1 거리 flip, 거부 발동(두 arm), pivot 보류.
  - 이탈 상태 arm: 고정폭, `X`, `A`, `U`, `B`, `L`(a<0), a=±0.
  - 생존 변조 15경계(§6). 재호출 경로는 `Recall`로 거부한다.
  - 위반 basis·infeasible ratio·batch·x87·pivot 적용·push 연결·work는 범위 밖이다.
- 후속: M3-04b-3(미관측 arm·거부·보류 오라클)이 생기면 같은 도구에 사례를 추가해 생존 경계를 줄인다. M3-04c-5(재진입)는 그대로다. push 한 단계 연결은 pivot 적용(basis 교환·basic 값·rhs 보정) 구현 뒤 별도 번호로 둔다.
- branch `work/m3-04c-6`, worktree `../rsolver-m3-04c-6`. 결과·인수인계·진행·로드맵은 이 branch에서 갱신해 `--no-ff`로 병합한다.
- 교훈: `docs/lessons/process.md`(오라클 모델 변조를 Rust 소스 변조로 옮겨 하네스 강도를 대조).
