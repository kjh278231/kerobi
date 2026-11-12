# M3-04c-11 push 연쇄의 생존 경계 원본 캡처

작성 기준은 [스프린트 검증 기준](../15_sprint_verification.md)과 [계약 양식](../templates/sprint_acceptance.md)이다. §1–§5는 본 캡처(각 3회) 전에 썼다. 탐색 캡처는 `bench/oracle/m3_push_chain/history/`에 보존하고 근거로 세지 않는다.

## 1. 범위와 주장

- 작업 종류: 오라클 구축 + 구현 재검증. 기준 main `47e29f7`, branch `work/m3-04c-11`, worktree `../rsolver-m3-04c-11`. 미커밋 선행 변경 없음. 점유 `claude-m3-04c11-20261112`(사용자가 Frida 설치를 허용했다. 같은 세션에서 shared 정리로 Frida 경로를 프리로드 해석으로 바꿨다, main `47e29f7`).
- 목표: [04c-10](primal_push_step_chain.md) 연쇄 재생에서 캡처 자료가 없어 살아남은 Rust 변이 10종의 경계를 자연 push로 밟는 원본 캡처를 만들고, 04c-10 도구의 사례로 더해 `push_pick`·`push_apply`를 다시 비교한다.
- 목표 사건(각 LP의 두 순서 중 어느 쪽에서든 1회 이상, 캡처 전에 고정):

| 태그 | 사건 | 노리는 변이 |
|---|---|---|
| `lb-enter` | 진입 구조 열 lb가 유한하고 0이 아닌 pivot | `pivot-no-lb-shift` |
| `lb-leave-shift` | 이탈 구조 열이 -1이 아닌 상태로 가고 lb≠0인 pivot | `pivot-shift-order` |
| `v-then-f` | 같은 구간(재분해 사이)에서 위반 경로 단계 뒤 feasible 경로 단계 | `pivot-viol-no-decrement`·`loop-viol-keep` |
| `free-flip` | 비어 있지 않은 자유 구조 열(x≠0)의 -3 flip | `flip-free-shift-sign`·`rhs-flip-free-shift` |
| `f-row-loop` | 반복 루프가 F 표시 행을 지나는 단계 | `loop-free-classified` |

- 제외: 분류 경계 동률 3종(`classify-*`, 정확 동률 설계 미착수), 동치 변이 6종(04c-10 §6).
- 선행: 04c-10(도구 `tools/push_step_chain_rust.py`, Rust `push_pick`·`push_apply`), shared Frida 프리로드(`tools/frida_resolve.cjs`).

## 2. 원본과 상태 경계

- Gurobi 13.0.3 `libgurobi130.so` SHA-256 `8e87fca2…`(도구가 실행마다 검사). Node Frida 17.26.2(`~/.local/share/frida-node`), x86-64 WSL2.
- 캡처 도구·agent: 04c-8의 `tools/primal_push_recall_capture.js`·`tools/primal_push_recall_agent.js`를 고치지 않고 `tools/gurun.sh`로 실행한다(해시는 04c-8 manifest와 같다). 훅 경계·사건은 04c-10 §2와 같다.
- 도구 사슬 확인: 같은 도구로 04c-8 `feasible.lp` primal을 재캡처해 저장 캡처와 사건 42·결과·출력이 같았다(shared 정리 commit `3f6316d` 기록).
- 설정: Threads=1, Seed=0, Presolve=0, Method=2, Crossover=3(primal)·1(dual), LPWarmStart=2, ScaleFlag=0, PStart·DStart는 `*.extra.json`.

## 3. 비교 계약

- 원본 재현성: LP마다 primal·dual 각 독립 3회. 세 실행의 `events`가 같아야 한다(바이트). 출처 필드(바이너리·도구·agent·모델 해시, params, starts)도 같아야 한다.
- 목표 합격: 위 5태그가 각 LP에서 정한 대로 1회 이상(shift: `lb-enter`·`lb-leave-shift`·`free-flip`·`f-row-loop`, resolve: `v-then-f`). 0회면 합격 검사가 0이 아닌 값으로 끝난다. 태그는 Rust가 아니라 원본 사건(`push:enter` 모델, `step:enter`·`step:applied` 상태)에서 센다.
- Rust 비교: 04c-10 계약 §3의 필드·규칙(정확·비트) 그대로. 새 사례를 더한 뒤 기존 166단계도 다시 통과해야 한다.
- 변이: 04c-10의 31종을 다시 돌려 위 7종이 거부로 바뀌는지 기록한다. 바뀌지 않으면 그 이유를 적는다.

## 4. 사례와 원본 자료

| 사례 | 실제/합성 | 내용 | 입력 |
|---|---|---|---|
| shift primal·dual | 실제 LP·시작 벡터 | 04b-3 arm 묶음 전체를 0.3만큼 옮김 + 자유 묶음 FZ(`fz`·`zz`, 등식 하나) | `bench/oracle/m3_push_chain/shift.{lp,extra.json}` |
| resolve primal·dual | 실제 LP·시작 벡터 | 04c-2 위반 묶음 중 SA(해소되는 A)·feasible 묶음 | `bench/oracle/m3_push_chain/resolve.{lp,extra.json}` |

- 생성기 `tools/push_chain_lp.py`(기존 생성기 두 개의 묶음을 import, 원본 파일은 수정하지 않음).
- 탐색: `history/shift-probe-primal.json`(5태그 중 4 발생), `history/resolve-v1*`(W만 뺀 위반 LP, B 7개가 끝까지 남아 `v-then-f` 0), `history/resolve-probe-primal.json`(`v-then-f` 1).

## 5. 재실행 명령

- 입력: `python3 tools/push_chain_lp.py shift bench/oracle/m3_push_chain/shift`, `... resolve ...`(기존 파일과 다르면 거부).
- 캡처: `tools/gurun.sh node tools/primal_push_recall_capture.js <primal|dual> bench/oracle/m3_push_chain/<lp>.lp bench/oracle/m3_push_chain/<lp>-<kind>-<i>.json "$(cat bench/oracle/m3_push_chain/<lp>.extra.json)"`, i=1..3, 한 번에 한 프로세스.
- 검사: `python3 tools/push_chain_oracle.py --check`(재현성·출처·목표 태그, 실패 시 종료 1), `--self-test`(변조 거부).
- Rust: `python3 tools/push_step_chain_rust.py --write && cargo test -p rsolver-lp --test push_chain && python3 tools/push_step_chain_rust.py --compare`, `--self-test`, `--mutate-rust <새 경로>`.

## 6. 실행 결과

- 실행 commit: 계약 `15bb3ec`, 캡처 `4eb5af3`(기준 main `47e29f7`). WSL2 x86-64, Node v22.22.1, Frida 17.26.2, Gurobi 13.0.3 체험판 라이선스.
- 원본: shift primal 192·dual 190사건, resolve primal 71·dual 69사건. 네 사례 모두 독립 3회의 사건·결과·출력이 같았다. `python3 tools/push_chain_oracle.py --check` 종료 0. manifest `bench/oracle/m3_push_chain/manifest.json`.
- 목표 태그(run 1): shift primal `lb-enter` 16·`lb-leave-shift` 3·`free-flip` 1·`f-row-loop` 1, dual 15·3·1·1. resolve primal·dual `v-then-f` 각 1. 5태그 모두 발생.
- 검사기 판별력: `--self-test` 변조 8종 거부(바이너리 해시 0, 사건 비움, lb 0으로 목표 0회, feasible 경로 제거로 `v-then-f` 0회, 실행 간 차이, 도구 해시, 모델 해시, x87 사건).
- Rust 재비교(Rust 코드 변경 없음): 사례를 더한 `python3 tools/push_step_chain_rust.py --write && cargo test -p rsolver-lp --test push_chain && python3 tools/push_step_chain_rust.py --compare` 종료 0. **236단계(flip 70·pivot 148·보류 18), 42구간, 필드 5,144개 전부 일치.** 새 사례만 72단계(flip 25·pivot 39·보류 6), 재분해 건너뜀 0. 기대 집계는 Rust 실행 전에 투영에서 세어 고정했다. `--self-test` 변조 24종 거부.
- 변이 재실행(`bench/oracle/m3_push_step_chain/rust-mutation-v2.json`, 04c-10 기록은 그대로 보존): 31종 중 **21 거부**(04c-10은 15). 생존→거부: `pivot-no-lb-shift`·`pivot-shift-order`(shift), `flip-free-shift-sign`·`rhs-flip-free-shift`·`loop-free-classified`(FZ), `pivot-viol-no-decrement`(resolve, 위반 해소 뒤 원본은 F 경로인데 변이는 V).
- 남은 생존 10: 동치 6(04c-10 §6), 분류 동률 3(범위 밖), `loop-viol-keep` 1. resolve의 위반 행(SA의 A)은 pivot 행에서 해소됐고, 반복 루프에서 B/A 행이 다시 분류되는 단계가 위반 해소 전에 없었다. 그래서 루프의 감소 누락이 경로를 바꾸지 않는다.
- 회귀: `cargo test --release --workspace` 130 통과, `cargo clippy --workspace --all-targets -- -D warnings` 0, `git diff --check`. 재생 비교 `primal_push_decision_rust`·`primal_push_ratio_rust`·`push_violated_rust`·`push_retry_rust`·`primal_push_reentry_rust`·`push_violated_reentry_rust`·`push_step_chain_rust`의 `--compare`와 `push_chain_oracle.py --check` 모두 종료 0.

## 7. 판정과 인수인계

- 작업 수행: 완료.
- 오라클: 계약 범위 **통과**(독립 3회 재현, 목표 5태그 발생, 변조 거부). 구현 재검증: 04c-10 계약 범위에 새 자연 사례를 더해 **통과**(Rust 변경 없음).
- 미검증: `loop-viol-keep`(루프 안 B/A 행이 해소 전에 분류되는 자연 사례), 분류 동률 3종, 04c-10 §7의 범위 밖 항목.
- 후속: M3-04c-12(push 루프 본체 조사), M3-04d. `loop-viol-keep`은 위반 행을 고치지 않는 pivot이 그 행을 지나가는 묶음(위반 행 계수가 다른 열에도 있음)을 resolve에 더하면 판별할 수 있다. 필요하면 별도 행으로 둔다.
- branch `work/m3-04c-11`, worktree `../rsolver-m3-04c-11`.
- 교훈: `docs/lessons/original.md`(F 행은 ratio 후보에서 빠져 자유 열이 양방향 -1 → -3 flip, 위반 묶음의 후보 없는 B 행은 W 없이도 해소되지 않음).
