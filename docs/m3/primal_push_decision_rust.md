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

(실행 후 작성)

## 7. 판정과 인수인계

(실행 후 작성)
