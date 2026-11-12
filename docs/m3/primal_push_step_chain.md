# M3-04c-10 push 한 단계 선택·적용의 Rust 연결

작성 기준은 [스프린트 검증 기준](../15_sprint_verification.md)과 [계약 양식](../templates/sprint_acceptance.md)이다. §1–§5는 Rust 구현과 비교를 실행하기 전에 썼다. 착수 전 Python 시제품으로 해석을 확인했으며, 그 결과는 §6에 적고 근거로 세지 않는다.

## 1. 범위와 주장

- 작업 종류: 구현(연결 L1). 기준 main `2e4b12e`, branch `work/m3-04c-10`, worktree `../rsolver-m3-04c-10`. 미커밋 선행 변경 없음. 점유 `claude-m3-04c10-20261112`(사용자 "다음 M3 스프린트 진행해 너가 작업자야" 지시. M3 인계의 첫 후속 "push 루프 연결"은 범위가 커서 하위 번호로 나눴다. 이 행이 첫 조각이다).
- 새 행: 스프린트 목록에 M3-04c-10을 더한다.
- 완료할 동작: double primal push `134a87`의 **한 단계**, 즉 공급자 `25615c`가 후보 j와 FTRAN 열을 돌려준 직후부터 다음 공급자 호출 직전까지(재분해 제외)를 Rust 함수 둘로 옮긴다.
  - `push_pick`(C 1209–1591, 2063–2083): `S+0x2fc=j`·`S+0x328=0` → 방향식(feasible/위반, 위반 수 `local_1b0`로 경로 선택) → ratio(`055f68` 또는 `054eef`→게이트→`056973`) → `S+0x2f8` 쓰기 순서(feasible -1이면 쓰지 않고 theta=I, 위반 경로는 반환값을 바로 씀, ratio의 요청 2면 `S+0x294=2`·`S+0x2f8=-1`을 먼저 씀) → theta=I 재호출(방향 반전·열 부호 재반전·두 번째 ratio, 자유 열이고 다시 theta=I면 `S+0x2f8=-1`) → 판단(거리 flip·거부·pivot·보류).
  - `push_apply`(C 1589–1613, 2084–2222, 1614–1718): flip은 진입 열 상태(-1/-2/-3)와 이동량, pivot은 희소 위치 p를 마지막 원소로 채우고 count-1, `state[j]=r`, `S+0x32c=header[r]`, `header[r]=j`, 이탈 상태(-1/-2/-4), 행 r의 폭·basic 값·표시·위반 수. 두 경로 모두 반복 카운터 `S+0x2a0`+1. 합류점 `0x1c3004` 뒤 rhs 보정(진입 열과 pivot 이탈 열의 CSC, slack 진입은 rhs[j-n]+=slack), 남은 희소 원소의 basic 값 `-= 이동량·alpha`와 표시 재분류·위반 수.
- 입력 전제: double 경로(`S+0x28==0`), batch 0, 희소 FTRAN 열(count≥0), 지원 집합 모드의 보류는 공급자 몫이라 범위 밖.
- 제외와 후속:
  - 공급자 `25615c`(후보 순서·보류 목록·실패 횟수·FTRAN), factor 갱신·재분해 `05b257`와 그 요청 2·10 판단(C 1725–1997), 재분해 뒤 basic·폭·표시·PInf 재계산, 후보 구성·정렬·기준 조정, 재처리 라운드, 마무리 출력. 연쇄 재생은 이 자리에서 원본 상태로 다시 맞춘다(§3).
  - work(ratio의 work 누적을 0 계수로 부른다), x87, batch, dense 열, NaN 표시 분류.
  - 후속: push 루프 본체(공급자·factor 갱신 판단·재처리, 하위 번호로 더 나눔), M3-04d.
- 선행: 04b-1·04b-2·04b-3·04c-2·04c-7·04c-8의 자연 push 캡처(각 원본 독립 3회 검증 완료), 04c·04c-5·04c-6·04c-9의 ratio·판단 Rust.

## 2. 원본과 상태 경계

- Gurobi 13.0.3 `../.venv/lib/python3.13/site-packages/gurobipy/.libs/libgurobi130.so`, SHA-256 `8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c`.
- C `../re/decomp/c/001c0070_PRIVATE0000000000134a87.c`. 지역 변수 대응(162–220줄): `local_118`=`S+0x168` 상태, `local_f0`=header, `local_c8`=basic(`S+0x1e8`), `local_e8`=폭(`S+0x208`→`+0x10`), `local_c0`=표시(`S+0x280`), `local_108`=rhs(`S+0x200`), `local_148/150`=lb/ub, `local_130`=x(param_5, slack과 이어진 할당이라 j≥n이면 slack[j-n]), `local_68`=h(`S+0x380`), `local_1d0`=-h, `local_110`=I, `local_1d8`=-I, `local_50`=`S+0x3b0`, `local_78`·`local_1b8`=n, `local_1f8`·`local_200`=진입 열의 dU·dL(1212–1218, slack은 I·slack), `local_220`·`local_1e0`=theta·pivot, `local_1b0`=위반 수, `local_48`=FTRAN 희소 열(`S+0x210`). `local_218`·`local_268`은 926–927줄에서 slack·rhs 앞을 n칸 당긴 포인터다.
- ABI·훅 경계는 기존 agent와 같다. `supply:leave`(`25615c` 반환, 후보·희소 열), `ratio:enter/leave`(`055f68@0xef960`·`054eef@0xeefa0`), `retry:enter/leave`(`056973@0xf0120`), `step:enter`(flip `0x1c2994`, pivot `0x1c2ab5`), `step:defer`(`0x1c3b7b`), `step:applied`(`0x1c3004`, 교환 직후·rhs 보정 전), `step:leave`(다음 공급자 진입 또는 push 반환 때의 상태), `factor:enter/leave`.
- 시작: 구간 첫 `supply:enter`의 상태(push 시작 또는 재분해 요청 10 뒤). 종료: 다음 재분해 요청 10 또는 push 반환.
- 읽는 상태: 모델 CSC·cost·lb·ub(`push:enter`), 상태·header·x·slack·basic·폭·표시·rhs·반복 카운터(구간 시작에서만), 허용오차 h·`S+0x398`·I·`S+0x3b0`, 매 단계 공급자 출력(후보 j, 반전 전 희소 열), `12dd49`가 돌려줄 factor 갱신 수(각 `ratio:enter`의 `factor.updates`, 입력).
- 변경 상태와 비교: §3.

## 3. 비교 계약

Rust는 구간 시작 상태에서 출발해 단계마다 자기 상태를 이어 간다. 원본 상태를 다시 넣는 것은 재분해 요청 10의 `factor:leave` 뒤 첫 `supply:enter`뿐이다(그 사이의 재분해·재계산은 범위 밖). 요청 0(factor 갱신)은 basis 배열을 바꾸지 않는다고 보고 다시 맞추지 않는다. 그 가정이 틀리면 다음 단계 비교가 실패한다.

| 필드·사건 | 비교 규칙 | 원본 출처 |
|---|---|---|
| 경로 F/V(위반 수 0 여부) | 정확 | `ratio:enter.name` |
| ratio 직전 `S+0x2fc`·`S+0x2f8`·`S+0x328` | 정확. 첫 `S+0x2f8`은 구간 시작값이 없어 첫 `ratio:enter` 값을 입력으로 쓰고 비교에서 뺀다 | `ratio:enter.selected` |
| 방향식 뒤(재호출이면 재반전 뒤) 희소 열 값 | 비트 | 각 `ratio:enter.alpha` |
| ratio 반환 위치·theta·pivot(재시도가 있으면 재시도 결과) | 정확·비트 | `ratio:leave`·`retry:leave` |
| 재호출 수 | 정확 | ratio 호출 수 |
| 단계 종류 flip/pivot/보류, theta | 정확·비트 | `step:enter.kind`·`theta`, `step:defer` |
| frame r·dL·dU(캡처에 있으면) | 비트 | `step:enter.frame` |
| `0x1c3004`의 상태·header·basic·폭·표시·반복 카운터, `S+0x2f8`(pivot 행 또는 -1)·`S+0x2fc`·`S+0x328` | 정확·비트 | `step:applied.state`·`selected` |
| 단계 뒤 상태·header·x·slack·basic·폭·표시·rhs·반복 카운터 | 정확·비트. 재분해 요청 10이 끼면 비교하지 않고 원본으로 다시 맞춘다(그 수를 센다) | `step:leave.state`(보류는 다음 `supply:enter`) |
| 요청 칸 `S+0x294` | 정확. ratio가 요청 2를 쓰지 않는 한 Rust는 0을 유지하고, 원본은 단계 뒤 factor 판단에서 바꿀 수 있어 `ratio:enter`의 값과만 비교한다 | `ratio:enter.state.counters.request` |

- 수치는 모두 비트 비교다. NaN·±0은 비트 그대로 비교하며 허용오차 비교는 없다.
- theta·pivot 지역 변수는 구간 시작에서 NaN으로 둔다. 원본에서 이전 값이 새어 나오면 비교가 실패한다.
- 시간·병렬: Threads=1 캡처, work는 비교하지 않는다.
- 원본 재현성: 각 캡처는 이전 작업에서 독립 3회 바이트 동일·대조군·비간섭을 검사했다. 이번 도구는 `primal_push_decision_check.load_all`로 그 검사를 다시 통과한 첫 실행을 쓴다.

## 4. 사례와 원본 자료

| 사례 | 실제/합성 | 노리는 분기 | 입력 |
|---|---|---|---|
| 04b-1 primal·dual | 실제 | 정상 pivot·flip, 목적 0 | `bench/oracle/m3_primal_push_step/{primal,dual}-1.json` |
| face5 primal·dual | 실제 | 방향 1, 요청 0 갱신 사이 연쇄 | `bench/oracle/m3_push_decision/face5-*-1.json` |
| arms primal·dual | 실제 | 이탈 arm `U`·`L`·`N`·`X`·-4, 보류, 거부 flip | `bench/oracle/m3_push_arms/` |
| violated primal·dual | 실제 | 위반 경로 `054eef`, 이탈 `A`·`B`, 표시 재분류·위반 수 증감 | `bench/oracle/m3_push_violated/` |
| retry primal·dual | 실제 | `056973` 게이트, 보류, theta=I flip | `bench/oracle/m3_push_retry/` |
| recall feasible·violated × 두 순서 | 실제 | theta=I 재호출, 자유 열 -3, feasible -1의 `S+0x2f8` 미기록 | `bench/oracle/m3_push_recall/` |

- 적용하지 않은 범주: dense 열·batch·x87(캡처 없음), 재분해 직후 단계의 basic 재계산(범위 밖), ratio 재진입 요청 2의 자연 발생(캡처 없음, 04c-5·04c-9 합성만).
- 기대 집계(캡처에서 세어 고정): flip 45, pivot 109, 보류 12, 재분해가 끼는 pivot 6, 구간 수는 도구가 출력한다.

## 5. 재실행 명령

- 원본 캡처: 새로 실행하지 않는다(위 자료의 manifest 명령).
- 입력 투영: `python3 tools/push_step_chain_rust.py --write`
- 재생·비교: `cargo test -p rsolver-lp --test push_chain && python3 tools/push_step_chain_rust.py --compare` 종료 0. 단계 수·필드 수·재동기 수를 출력한다.
- 비교기 판별력: `python3 tools/push_step_chain_rust.py --self-test`(trace 변조가 모두 실패해야 함).
- Rust 변이: `python3 tools/push_step_chain_rust.py --mutate-rust`(임시 복사본, 결과를 `bench/oracle/m3_push_step_chain/rust-mutation.json`에 기록).
- 회귀: `cargo test -p rsolver-lp --test push_decision && python3 tools/primal_push_decision_rust.py --compare`, 04c·04c-2·04c-5·04c-7·04c-9 재생, `cargo test --release --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`.

## 6. 실행 결과

- 실행 commit: 구현 `9571d31`(기준 main `2e4b12e`). x86-64 Linux(WSL2), rustc stable. Gurobi 새 실행 0회.
- 원본 자료: 위 14캡처(각 사례 run 1). `load_all`이 각 사례의 독립 3회 바이트 동일·대조군·비간섭 검사를 다시 통과했다. 투영 입력 `bench/oracle/m3_push_step_chain/input.txt` SHA-256 `7f216b4c…`(본문 해시 `cc6b85ad…`는 파일 첫 줄), 도구 `tools/push_step_chain_rust.py` `bec31342…`.
- 착수 전 시제품: §3의 적용 해석을 Python으로 154단계(pivot 109·flip 45)에 맞춰 보니 `step:applied`와 재분해 없는 `step:leave`가 모두 비트 일치했다. 이 결과는 근거로 세지 않고 Rust 비교를 근거로 한다.
- Rust 재생·비교: `cargo test -p rsolver-lp --test push_chain && python3 tools/push_step_chain_rust.py --compare` 종료 0. **166단계(flip 45·pivot 109·보류 12), 34구간, 비교 필드 3,606개 전부 일치.** 재호출 18단계, 위반 경로 68단계, 재분해로 단계 뒤 비교를 건너뛴 단계 6(계약 기대와 같음), 구간 첫 `S+0x2f8` 입력 34. 구간 시작 theta·pivot을 NaN으로 두었는데 새어 나온 값이 없었다. 요청 0(factor 갱신) 뒤에도 Rust 상태를 다시 맞추지 않고 이어 갔으며 다음 단계가 일치했다.
- 비교기 판별력: `--self-test` 종료 0. trace 변조 24종이 모두 거부됐다(경로·`S+0x2f8` 이월·alpha 비트·ratio 위치·theta·재호출 누락·단계 종류·보류·sel·적용 상태 5필드·단계 뒤 rhs·basic·x/slack·폭·보류 상태·frame·단계 누락·순서 바꿈·여분 필드). 첫 구현에서 frame 변조가 frame 없는 04b-1 단계를 골라 미검출이었고, 검사기 대상 선택을 고쳤다(구현 코드는 그대로).
- Rust 변이 `--mutate-rust`(`bench/oracle/m3_push_step_chain/rust-mutation.json`): 31종 중 **15 거부, 16 생존**.
  - 거부: feasible -1에서 `S+0x2f8` 쓰기, 위반 경로 선택, 재호출 재반전·방향 반전, flip 뒤 `S+0x2f8` 미기록, flip 반복 카운터, flip 이동량 gap, pivot 폭 I, pivot 부호 복원, 이동량 부호, 이탈 상태별 basic 보정, 자유 행 `F`, pivot rhs의 bound 보정, 이탈 열 rhs, slack rhs.
  - 동치로 판별 불가(6): `pick-no-inf-theta`(push가 cap=I로 부르고 `055f68`은 후보 없으면 theta=cap), `pick-free-leave-kept`(뒤의 flip이 `S+0x2f8=-1`을 다시 씀), `pivot-remove-keep-order`(남은 행은 서로 독립), `rhs-mul-order`(IEEE 덧셈 교환), `classify-x-any-width`(폭은 음수가 아님), `rhs-leave-bound-ub`(-4는 폭≤`S+0x3b0`=0이라 lb=ub).
  - 자료 부족(10): 진입 열 lb≠0(`pivot-no-lb-shift`·`pivot-shift-order`, 모든 캡처의 lb가 0 또는 -I), 위반 수가 0으로 떨어지는 단계(`pivot-viol-no-decrement`·`loop-viol-keep`, 위반 LP는 영구 위반 행 W를 둔다), 자유 열 비어 있지 않은 -3 flip(`flip-free-shift-sign`·`rhs-flip-free-shift`, 캡처의 자유 열은 빈 열), 반복 루프의 자유 행(`loop-free-classified`), 분류 경계 동률 3종.
- 회귀: `cargo test --release --workspace` 130 통과(기존 129 + `push_chain`), `cargo clippy --workspace --all-targets -- -D warnings` 0, `git diff --check`. 04c-6 판단 재생 `primal_push_decision_rust.py --compare`, 04c `primal_push_ratio_rust.py`, 04c-2 `push_violated_rust.py`, 04c-7 `push_retry_rust.py`, 04c-5 `primal_push_reentry_rust.py`, 04c-9 `push_violated_reentry_rust.py`의 `--compare` 모두 종료 0. Python 확장은 경계 변경이 없어 미실행.
- 실행하지 못한 검사: 새 원본 캡처로 생존 경계(lb≠0, 위반 해소, 비어 있지 않은 자유 열)를 줄이려 했으나 이 컴퓨터에는 Frida가 없다(기존 캡처 도구의 require 경로 `/home/sds/...` 없음, venv에 frida 모듈 없음). 설치하지 않고 후속으로 넘겼다.
- 계약 변경: clippy(`assign_op_pattern`)로 `rhs[i] = d*a + rhs[i]`를 `rhs[i] += d*a`로 바꿨다. IEEE 덧셈 교환으로 비트가 같고 재비교도 통과했다.

## 7. 판정과 인수인계

- 작업 수행: 완료.
- 동등성: 계약 범위 **통과**(자연 캡처 저장 재생, 연쇄, 정확·비트 비교). push 한 단계의 선택과 적용이 원본 자료의 모든 단계에서 일치한다.
- 미검증: 변이 생존 10경계(lb≠0, 위반 수 0 도달과 그 뒤 F 경로, 비어 있지 않은 자유 열 flip, 루프의 자유 행, 분류 동률), dense 열·batch·x87, 재분해 뒤 재계산, work, 자연 요청 2.
- 후속:
  - **M3-04c-11**(오라클): Frida가 있는 컴퓨터에서 하한을 옮긴 arm LP와 W 없는 위반 LP(위반 해소)를 원본 두 순서 각 3회 캡처해 이 도구에 사례로 더한다. `tools/push_arm_lp.py`·`push_violated_lp.py`의 묶음을 새 생성기에서 재사용한다.
  - **M3-04c-12**(조사·구현): push 루프 본체. 공급자 `25615c`(후보 순서·보류·실패 횟수), factor 갱신 판단(C 1725–1997, 요청 2·10), 재분해 뒤 재계산을 하위 번호로 나눈다.
  - M3-04d(dual push 조사).
- branch `work/m3-04c-10`, worktree `../rsolver-m3-04c-10`.
- 교훈: `docs/lessons/original.md`(crossover: 적용 구간의 이동량·rhs·표시 규칙, 요청 0 갱신은 basis 배열을 바꾸지 않음), `docs/lessons/tooling.md`(이 컴퓨터의 Frida 부재, 변조 대상 선택은 필드 존재를 확인).
