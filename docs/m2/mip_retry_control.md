# M2-03d-2 검증 계약과 결과: 재풀이 판단 제어 (L1)

작성 기준은 [스프린트 검증 기준](../15_sprint_verification.md)이다. 비교 경계는 [M2-03d-1](mip_retry_rust_boundary.md) §3을 따른다.

## 1. 범위와 주장

- 작업 종류: 구현.
- 사용자 결정(2026-11-10):
  - L1 합성 재생으로 검증한다. 원본 LP 결과를 주입하는 방식이다.
  - 재풀이 판단은 LP 계산과 분리하고, 실제 경로에서도 다시 쓸 수 있는 구조로 만든다.
  - primal warm start와 cutoff는 dual·cold start로 우회하지 않는다.
  - 기존 Numeric 종료를 유지하고, 실제 경로는 미검증으로 표시한다.
- 완료할 동작: `5ce67b`의 노드 LP 처리 중 노드에 basis가 있는 분기(`node_basis_direct`)의 판단이다.
  1. 첫 LP: 노드 basis와 method `param_3[3]`을 쓴다(C:351–361).
  2. 재풀이 진입: 첫 LP 상태가 12이고 (캐시 2 또는 루트)일 때다(C:769–770).
  3. 재풀이: 같은 노드 basis를 쓰고, method는 `param_3[3]==0`이면 1, 아니면 0이다(C:924–933).
  4. 재풀이 결과: LP 6이면 노드 5이고 basis를 새로 저장하지 않는다(`600d25`, 캡처).
- 포함하는 판단 경계:
  - 노드 basis가 없으면 `0c3d0d` 경로로 가야 하지만 구현하지 않는다. 결과는 "미지원"이다.
  - 재풀이 결과가 6이 아니면 "미이식"을 반환한다. 12 잔존(C:2244–2257)과 2(C:963의 cutoff 비교, C:1358의 저장)는 원본 근거가 C뿐이고, Rust에는 노드 3 처리나 저장 구조가 없다.
  - 호출자는 미지원·미이식이면 기존 동작을 유지한다.
- 제외:
  - 실제 B&B 연결(`bnb.rs:176` 유지).
  - basis 상속·저장(M2-10b), 첫 LP 적응 선택(`param_7≠0`), 재풀이 전 정규화(5→12 등, M2-03c 범위).
  - primal warm start·cutoff(M1), gmu 분기, L2(M2-03d-3).
- 선행: main `b8fe2b7`. 미커밋 선행 변경은 없다.

## 2. 원본과 상태 경계

- Gurobi 13.0.3, SHA-256 `8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c`.
- 근거: `5ce67b`(`0x4f0e10`) C:351–361, 769–786, 924–1035, 2244–2257, 그리고 `600d25`. 상세는 [M2-03d-1](mip_retry_rust_boundary.md) §2에 있다.
- 시작 경계: 노드 LP 처리 진입. 입력은 노드 basis(VBasis·CBasis), `param_3[3]`, `param_3+0x10`, 캐시, 루트 여부다.
- 종료 경계: 노드 처리 결과. 노드 상태, 마지막 LP 상태, basis 저장 여부를 내거나, 미지원·미이식을 낸다.
- LP 호출은 바깥에서 주입한다. 판단 코드는 LP 계산을 하지 않는다.

## 3. 비교 계약

[M2-03d-1](mip_retry_rust_boundary.md) §3의 `rsolver.mip_retry.v1` 필드 규칙을 그대로 쓴다. 모든 필드는 정확 비교이고, 추가 사건이나 필드가 있어도 실패다. 판단을 판별하는 필드는 넷이다.

- 첫 LP와 재풀이의 basis
- 첫 LP와 재풀이의 method
- `node_status`
- `basis_saved`

LP 상태 필드는 주입값이 그대로 나오는 순서 표지다.

## 4. 사례와 원본 자료

| 사례 | 실제 / 합성 | 분기 | 원본 증거 |
|---|---|---|---|
| nidda stress 노드 8287 | 원본은 실제 실행. Rust의 LP 결과는 합성 주입 | basis 있음, 12→재풀이→6→노드 5 | `bench/oracle/mip_retry_rust/nidda_stress.json`(M2-03c-2 캡처 3회의 투영) |
| 판단 경계 단위 검사 | 합성 | basis 없음→미지원, 12 아님→재풀이 없음, 비루트·캐시≠2→재풀이 없음, 재풀이 2·12→미이식, `param_3[3]=0`→재풀이 method 1 | C만. 원본 실행 증거가 아니므로 동등성 근거로 쓰지 않는다 |

## 5. 재실행 명령

- `python3 tools/mip_retry_rust_expect.py --write`: 기준 자료와 Rust 입력 텍스트를 다시 만든다.
- `cargo test -p rsolver-mip --test retry_l1`: Rust trace를 `target/mip_retry_rust/nidda_stress.trace.json`에 쓴다.
- `python3 tools/mip_retry_rust_expect.py --self-test --compare target/mip_retry_rust/nidda_stress.trace.json`: 기대 종료코드는 0이다.
- 회귀:
  - `cargo test -p rsolver-mip`
  - `cargo test --release --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`
- 오류 검출: 임시 복사본에서 재풀이 method 규칙을 뒤집으면 compare가 실패해야 한다. 이 변경은 남기지 않는다.

## 6. 실행 결과

- 환경: worktree `../rsolver-m2-03d-2`, 구현 commit `19c6675`. 입력 SHA-256 앞 16자리:
  - `crates/mip/src/retry.rs` `2dd678a8a6e4d3e5`
  - `tools/mip_retry_rust_expect.py` `bb364f4f5d2371d9`
  - `bench/oracle/mip_retry_rust/nidda_stress.input.txt` `73bf1fef03c20aef`
  - 기준 자료 `nidda_stress.json`은 M2-03d-1과 같다. `--write`로 다시 만들어도 같았다.
- 구현:
  - `crates/mip/src/retry.rs`의 `process_node_lp(ctx, basis, solve)`. LP는 `FnMut(LpCall) -> i32`(Gurobi 상태 코드)로 받는다.
  - 판단 코드에는 계측이나 테스트 분기가 없다. trace는 하네스(`crates/mip/tests/retry_l1.rs`)가 LP 클로저에 들어온 실제 인자와 반환된 `Outcome`으로 만든다.
  - `bnb.rs`는 바꾸지 않았다. Numeric 종료를 유지하고 dual·cold 대체도 없다.
- L1: `cargo test -p rsolver-mip --test retry_l1`을 실행한 뒤 `python3 tools/mip_retry_rust_expect.py --self-test --compare target/mip_retry_rust/nidda_stress.trace.json`은 종료코드 0이다.
  - 기준 자료는 사건 5개다. 캡처·trace 변조 18종을 거부했고 Rust 입력 텍스트도 기준 자료와 일치했다.
  - compare PASS. 첫 LP와 재풀이 basis가 노드 basis(253·464)와 같고, method는 1→0, 노드 5, 저장 없음이다.
- 오류 검출: 임시로 `retry.rs`를 바꿔 trace를 다시 만들었고, 세 경우 모두 compare가 실패했다. 원본은 복원했고 복원 뒤 PASS다.
  - 재풀이 method를 `warm_method`로 바꿈: `event 2 (retry) field method: expected 0 got 1`
  - `basis_saved: true`: `event 4 (node_done) field basis_saved`
  - 노드 상태 2: `event 4 (node_done) field node_status`
- 합성 단위 검사 `retry::tests::retry_rules`: C 규칙의 분기 7개다. 원본 실행 증거는 아니다.
- 회귀: `cargo test --release --workspace` 종료코드 0, `cargo clippy --workspace --all-targets -- -D warnings` 종료코드 0. Herdr pane에서 실행했고 로그는 worktree의 `target/m2-03d-2-*.log`다.
- 실행하지 않은 것:
  - Python 확장 재빌드와 pytest. Python 경계와 실제 풀이 경로를 바꾸지 않았다.
  - Gurobi. 새 캡처가 필요 없었다.

## 7. 판정과 인수인계

- 작업 수행 상태: **완료**(구현).
- 동등성: **L1 통과**. 범위는 nidda `node_basis_direct` 한 사례에서 원본 LP 결과를 주입한 재풀이 판단 구간이다. Rust LP와 원본 LP의 동등성이나 실제 B&B 경로를 뜻하지 않는다.

| 동작 | 상태 | 근거 |
|---|---|---|
| 노드 basis가 있으면 첫 LP에 노드 basis와 `param_3[3]` | L1 통과 | 캡처 사건 4 |
| 첫 LP 12, 캐시 2 → 재풀이 | L1 통과 | 캡처 사건 15·18 |
| 재풀이에 같은 노드 basis를 쓰고 method는 `warm≠0`→0 | L1 통과 | 캡처 사건 18 |
| 재풀이 6, 캐시 2 → 노드 5, 저장 없음 | L1 통과 | 캡처 사건 32, `600d25` |
| `warm==0`→재풀이 method 1 | 구현, 미검증(C만) | C:926 |
| 루트이면서 캐시≠2 → 미이식 반환 | 구현, 미검증(C만, 캐시 갱신 부수효과를 이식하지 않음) | C:1016–1022 |
| 재풀이 2·12 → 미이식 반환 | 미이식 | C:963, 1358, 2244 |
| 실제 B&B 연결, Rust LP primal warm·cutoff | 미구현·미검증 | M2-10b, M1 |
| 첫 LP 적응 선택, 재풀이 전 정규화, `a7f923`·FEASHELPER·`local_58+0x1a0` 조건, dual norm | 전제로 두고 미이식 | §1 |

- 후속:
  - M2-03d-3: 노드 LP 원본 캡처. 추가 62행의 정체와 gmu sub-MIP 가능성 조사를 포함한다.
  - M2-03d-4: 실제 경로 연결과 LP 포함 동등성. M2-10b, M1 요청, M2-03d-3 뒤에 한다.
  - M1 요청: [M1 인수인계](../handoff/M1.md).
- branch: `work/m2-03d-2`. 검증 뒤 main에 `--no-ff`로 병합한다.
- 교훈: [원본 사실](../lessons/original.md)의 확인 사실과 미검증 가설, [절차](../lessons/process.md), [도구](../lessons/tooling.md).
