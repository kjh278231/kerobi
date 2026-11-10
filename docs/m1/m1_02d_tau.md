# M1-02d 검증 계약과 결과: DSE τ의 원본 U 단계와 U 행 저장 순서

작성 기준은 [스프린트 검증 기준](../15_sprint_verification.md)이다. [M1-02a-1](m1_02a1_tau.md)·[M1-02a-2](m1_02a2_urow.md)의 후속 구현이고, 사용자의 "다음 스프린트 진행" 지시로 착수했다(2026-11-11). M2는 다른 세션이 M2-06a를 점유해 진행 중이라 [시작 절차](../handoff/README.md) §1-3 순서대로 M1을 골랐다.

## 1. 범위와 주장

- 작업 종류: 구현.
- 이번에 완료할 동작:
  1. Rust `Lu`의 행 방향 U(`urow`) 원소 순서를 원본 LU+0xb0/0xb8/0xd0/0xc8 저장 순서와 같게 만든다.
     - 인수분해 직후: singleton 단계 행은 열의 재분해 전 기저 위치 오름차순(`1da0b2` C:530–560, 단계 4 행은 C:657–677의 활성 행 사본 = 위치 오름차순), kernel 단계 행은 `kernel_u[단계]` 목록 순서(`036ade` C:600–690·C:2258–2265·C:2366–2393).
     - FT 갱신(`05b257`): 나가는 열 원소는 그 자리에 행의 마지막 원소를 옮겨 지운다(C:556–567), row eta면 행 r을 비운다(C:970–1037, 기존과 같음), 새 열 원소는 행 끝에 붙인다(C:1302–1304, 기존과 같음).
  2. DSE τ = B⁻¹ρ의 U 단계를 원본 `09d112` 두-RHS 경로 규칙으로 계산한다(C:2086–2402). L·R 단계는 기존 `ftran_lower` 그대로. U 단계: 꼬리 열(U 위치 ≥ LU+0x100)을 U 위치 내림차순으로 나누고(ZERO drop) 꼬리 행에만 흩뿌린다. α_q 지지 위치의 접두 행은 LU+0xf8 ≥ 0(U 행 있음)이면 `(x_i − Σ u·τ_j)/d_i`를 U 행 저장 순서로 빼서 모으고(8항 이상이면 asm `0x129369–0x129465`의 2레인×4 누적), −1(빈 행)이면 `x_i/d_i`(drop 없음), 꼬리 행이면 흩뿌린 값을 쓴다.
- 포함 분기와 입력 전제: Method=1(dual), DSE 가중치 갱신 경로의 모든 피벗. α_q 계산(`ftran_upper`, 모든 행 흩뿌림)은 바꾸지 않는다(02a-1 I 검사에서 원본 α_q와 비트 일치).
- 제외 범위와 후속:
  - 원본 `09d112`는 피벗마다 희소·인라인·조밀 경로를 고른다. 이번 구현은 τ에 모으기 규칙을 항상 쓴다. 규칙이 원본 τ를 재현한다는 실행 근거는 kb2 피벗 0–21(세 경로 모두 포함)뿐이다.
  - 8항 이상 모으기의 첫 원소 떼기(값 배열 주소가 8바이트 정렬이고 16바이트 비정렬이면 1개를 먼저 순차로 뺌)는 원본 행 저장 위치(풀 정렬)를 모델링하지 않아 하지 않는다(shortcut 주석). 8항 이상 행은 kb2 캡처에 없다(C·asm만).
  - 2×2 블록 단계·조밀 전환(Rust kernel에 없음), Uᵀ BTRAN·η 계산(원소 순서와 무관한 흩뿌림이라 값 불변)은 바꾸지 않는다.
  - Method=1 Netlib 64 재측정 판정은 M1-02c다. 이번 Netlib 실행은 회귀 확인이다.
- 선행: main `e46e1a5`(M1-02a-2 병합 `f7d8102` 포함). 미커밋 선행 변경 없음.

## 2. 원본과 상태 경계

- Gurobi 13.0.3, `libgurobi130.so` SHA-256 `8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c`. 모델 `../re/bench/data/netlib/kb2.mps.gz`(Method=1, Presolve=0, Threads=1).
- 원본 위치:

| 위치 | 내용 |
|---|---|
| `09d112` `0x125f50` C:2279–2402, asm `0x12925f–0x129495` | α_q 지지 위치마다 LU+0xf8 −1: `x/d`, −2: 흩뿌린 값, ≥0: U 행 모으기. 행 길이 <8이면 순차 빼기. ≥8이면 lane0 = (v, 0)부터 `subpd` 4쌍 (l0,l1)(l2,l3)(l4,l5)(l6,l7) 누적, `((l0+l2)+(l4+l6))` + `((l1+l3)+(l5+l7))`(asm `0x12941b–0x129433`, 디컴파일의 왼쪽 결합 표기와 다름), 나머지 순차 빼기, `/d` |
| `09d112` C:2205–2263 | 꼬리 열 흩뿌림은 접두 행 구간을 건너뛴다(02a-1) |
| `1da0b2` `0xdf800` C:530–560 | 단계 1–3 뒤 남은 열을 위치 오름차순으로 돌며 이미 피벗된 행의 U 행 끝에 붙인다. C:530 이전에 U 값(local_80/78) 쓰기 없음 |
| `1da0b2` C:625–680 | 단계 4(활성 열 singleton): 새 단계의 U 행은 그 행의 활성 사본(위치 오름차순)에서 아직 피벗되지 않은 열 |
| `036ade` C:600–690, C:2258–2265, C:2366–2393 | kernel 행 목록(앞에 넣기)과 U 행 쓰기(02a-2) |
| `05b257` C:556–567, C:970–1037, C:1302–1304 | 갱신의 삭제·row eta·추가(02a-2) |
| `9e60b` | 재분해 뒤 비용 추정이 행 U(LU+0xb0/0xd0)를 저장 순서로 읽는다(m1_notes §8). Rust `row_cost_estimate`도 `urow` 순서로 접으므로 값이 바뀔 수 있다 |

- 시작·종료 경계: 피벗 k의 `09d112` 진입(LU 상태 = 피벗 k−1 갱신 또는 재분해 뒤)과 τ 출력. Rust는 `simplex.rs` τ 계산 지점의 `Lu` 상태와 τ.
- 입력 상태: ρ(02a·02b에서 원본과 비트 일치), L·R(η), U 열·행·대각·U 순서(LU+0x48)·접두 t(LU+0x100), α_q 지지.
- 출력: α_q 지지 위치(피벗 위치 제외)의 τ 비트, 그 시점 Rust 행 U의 위치별 원소 순서·값 비트, 접두 t.

## 3. 비교 계약

Rust 쪽 자료는 `RSOLVER_SNAP` 스냅숏에 새 필드로 낸다. τ 계산 지점의 `Lu`에서 위치별 행 U(열 위치, 값 비트)와 접두 t를 `last_dse`와 함께 저장한다(피벗 k의 값은 스냅숏 k+1). 판정은 `tools/m1_tau_check.py`(갱신)다.

| 검사 | 규칙 |
|---|---|
| O 행 U 순서 | Rust 경로가 원본과 같은 피벗(최소 0–21, 재분해 피벗 15 포함)마다 Rust 행 U = 캡처 `ft.urow`(모든 위치, 열 번호 순서와 값 비트), Rust 접두 t = 캡처 `ft.t`. 비교한 피벗 수와 마지막 피벗을 보고한다 |
| R 규칙 다리 | 고정 Rust LU 덤프(`rust_lu_kb2.jsonl.gz`, 재생성하지 않음)에 원본 규칙(`tau_original`)을 적용한 τ = 새 Rust 스냅숏 τ(피벗 0–21, α_q 최종 목록 위치) |
| T 목표 | 새 Rust τ = 원본 τ(피벗 0–21, 269 위치 전부, ±0 포함 비트). 차이 1개라도 실패 |
| 기존 M·B·X·A·W·U·I·G·F | 그대로 유지. F(스냅숏 12 w 차이 위치)는 이제 차이가 없어야 하므로 "스냅숏 0–22 w가 저장 오라클과 같음"으로 바꾼다(저장 오라클 범위 안) |

- 정수·순서는 정확 비교, 값은 u64 비트. 허용오차 없음.
- 오류 검출 증거:
  - 수정 전 코드에 스냅숏 필드만 더한 빌드에서 O가 피벗 14(갱신 13→14 안정 삭제)와 피벗 15(재분해 초기 순서)에서, T가 피벗 11 위치 [20, 22]에서 실패해야 한다(출력 보존).
  - 자기검사 변조: 기존 16종을 유지·재조준하고, Rust 스냅숏 행 U 두 원소 교환(O), Rust τ 한 비트(T) 변조를 더한다.
- 수치 환경: x86-64 Linux(WSL2), Rust release 빌드.

### 회귀(동등성 판정과 별개, 기준 사전 고정)

- `cargo test -p rsolver-lp`, `cargo test --release --workspace`, `cargo clippy --workspace --all-targets -- -D warnings` 종료 0.
- 이 worktree 빌드(`tools/build_local.sh`)로 `pytest` 통과.
- `tools/m1_urow_check.py --self-test`, `tools/m1_beta_check.py --self-test` PASS(고정 자료, 영향 없어야 함).
- `tools/proto/statecmp.py kb2`(저장 오라클 반복 0–30)·`pathcmp.py kb2`: 수정 전후 첫 차이를 기록한다. 반복 12의 w 2개 차이가 사라져야 한다. 새 첫 차이가 생기면 다음 작업 단위로 기록하고 이번에 고치지 않는다.
- `tools/lp_check.py --methods 1 --timeout 120`(저장된 Gurobi 기준, Gurobi 실행 없음) 수정 전후 같은 worktree 빌드: 정답 90/91이 줄면 실패. IterCount 일치(수정 전 24/64)는 모델별 변화를 기록만 한다(판정은 M1-02c).

## 4. 사례와 원본 자료

| 사례 | 실제 / 합성 | 노리는 분기 | 입력 | 원본 증거 |
|---|---|---|---|---|
| kb2 Method=1, Presolve=0, Threads=1 | 원본 실제 실행(02a-1 캡처, 피벗 0–37) | 갱신 34회(row eta 있음·없음), 재분해 4회(U 행 있음 2회), τ 희소·인라인·조밀 경로, 모으기 2항 이상 90행, −1 행의 −0 | `kb2.mps.gz` | `bench/oracle/m1_02a1_tau/kb2_{1,2,3}.json.gz`(gz SHA-256 `64cc9dea…`), `bench/oracle/iter0-30/kb2.jsonl.gz` |

- 적용하지 않은 범주: 8항 이상 모으기·첫 원소 떼기(캡처 0회), fill 있는 kernel 행(kb2 없음), 다른 모델(재캡처 필요). Gurobi는 이번에 실행하지 않는다.

## 5. 재실행 명령

- 빌드: `tools/build_local.sh`.
- 판정: `PYTHONPATH=$PWD/python ../.venv/bin/python tools/m1_tau_check.py --self-test`.
- 회귀: §3 회귀 목록. `tools/lp_check.py`는 `PYTHONPATH=$PWD/python ../.venv/bin/python tools/lp_check.py --methods 1 --timeout 120 --csv <tmp>`.

## 6. 실행 결과

### 6.1 구현

- `crates/lp/src/lu.rs`
  - `factor_tol`: 행 U를 `finish()`에서 단계 오름차순으로 다시 만들지 않고 만들 때부터 원본 순서로 쌓는다. singleton 행은 재분해 전 기저 위치를 오름차순으로 돌며 그 열의 원소를 붙이고(`1da0b2` C:530–560·C:657–677), kernel 행은 `kernel_u[단계]` 목록 순서를 그대로 쓴다.
  - `update`: 나가는 열 원소를 `retain`(안정 삭제) 대신 `swap_remove`(마지막 원소로 채움, `05b257` C:556–567)로 지운다.
  - 새 `ftran_tau`(τ U 단계)와 `gather`(asm 레인 결합). `ftran_upper`(α_q)는 그대로다.
  - 스냅숏용 `u_rows`(위치별 행 U, 접두 t).
- `crates/lp/src/simplex.rs`: τ를 `ftran_lower` 뒤 `ftran_tau(α_q 지지)`로 계산한다. τ는 α_q 지지 위치만 채우고 나머지는 0이다(쓰는 곳은 지지 위치뿐). `RSOLVER_SNAP` 스냅숏에 `urow`·`t`(τ 지점의 행 U·접두 t)를 더했다.
- C 추가 확인(계약 뒤, 판정 근거는 아님): `1da0b2`의 U 값 쓰기(local_80/78)는 C:530 앞에 없다. 단계 1(행 singleton)·3(길이 1 열과 slack) 열은 앞선 행에 원소가 없으므로, singleton 행의 U 원소는 단계 4 열이거나 kernel 열이고 모두 C:530–560 또는 C:657–677에서 열 위치 오름차순으로 쓰인다. 02a-2가 미정으로 둔 "singleton 열 원소를 가진 singleton 행"도 같은 규칙이다(C만, kb2는 해당 없음).

### 6.2 판정

- 실행: worktree `../rsolver-m1-02d`, 기준 main `e46e1a5`, 계약 `c784f39`, 구현·검사기 `e8f3458`, 최종 `4e391eb`(clippy·단위 테스트·β 검사기 재조준). x86-64 Linux(WSL2). Gurobi는 실행하지 않았다(고정 캡처 사용).
- 오류 검출 증거(수정 전 코드 + 스냅숏 필드만 더한 빌드, 아래 경계 수정 전 검사기, 출력 `bench/oracle/m1_02d_tau/tau_check_before_fix.txt`): 종료 1. O가 피벗 14 위치 9(`[6, 15]` 대 원본 `[15, 6]`, 갱신 13→14 안정 삭제)와 피벗 15 위치 32(`[39, 41, 42]` 대 원본 `[41, 42, 39]`, 재분해 초기 순서)에서, R이 피벗 1·11–14·18–21에서, F가 스냅숏 12(w 위치 [20, 22])에서 실패했다. T는 R이 먼저 실패해 해당 피벗을 건너뛰므로 (11, [20, 22])를 직접 보고하지 않았다(F가 같은 위치를 보였다).
- 수정 뒤 `PYTHONPATH=$PWD/python ../.venv/bin/python tools/m1_tau_check.py --self-test` 종료 0(P 줄 생략):

```text
M: 3 runs, binary/model/params/output, pinned file sha, tool = current file b41afe7503934821, Rust LU dump and patch pinned
B: 38 pivots, 3 runs equal
X: one 09d112 call per pivot; its rho input = M1-02a compacted rho, beta = M1-02a, alpha_r/beta = stored oracle
A: alpha_q list and aligned tau at the 8b473 return = 09d112 outputs with the pivot swapped out (38 pivots)
W: 1bae9c DSE update from the captured alpha_q, tau, beta, alpha_r = next weights on 34 pivots (3 refactor boundaries not checked)
U: Rust LU dump = original U rows (sets, values), diagonal, U order and prefix t on 22 pivots (946 rows); LU+0xf8 = -2 tail, -1 empty prefix row, >=0 prefix row
I: Rust rho = original 09d112 rho input; Rust FTRAN of the original a_q = original alpha_q output (pivots 0-21)
R: original rule on the dump (captured U row order) = current Rust snapshot tau (pivots 0-21, final alpha_q positions)
T: Rust tau = original tau on all 269 alpha_q positions of pivots 0-21 (bits, signed zeros included); the pre-M1-02d all-row scatter would differ at 34 positions, first [(11, [20, 22])]
G: original rule (tail scatter to tail rows, prefix gather over the stored U row order) = original tau on all 269 positions; 90 gathered rows with 2+ terms, 0 with 8+ (C only)
F: Rust snapshots 0-21 w = stored oracle
O: Rust row-wise U (column order, value bits) and prefix t = capture on pivots 0-21 (same basis as the stored oracle through snapshot 21)
self-test: 19 tampers rejected (아래 목록)
PASS
```

- 자기검사 변조 19종: 기존 유지 14종(binary·3회·파일·도구·덤프 해시, 빈 사건, 원본 τ 비트, α_q 원소, ρ 비트, w 비트, 캡처 U 행 순서, 덤프 L 값, 캡처 U 값, a_q 입력 비트), 재조준 2종(Rust τ 피벗 11을 수정 전 규칙 값으로 → R, 원본 τ 피벗 11을 수정 전 규칙 값으로 → W), 새 3종(Rust τ 피벗 13 한 ulp → R, Rust 행 U 피벗 15 두 원소 교환 → O, Rust 접두 t 피벗 14 → O)이 모두 거부됐다.
- O 범위: 피벗 0–21(갱신 20회, 재분해 피벗 15 포함). 피벗 21의 진입 선택이 원본과 달라(아래 pathcmp) 스냅숏 22부터 기저가 갈리므로 재분해 피벗 26과 피벗 22 이후 갱신은 Rust 경로로 대조할 수 없다.
- 계약 변경(결과 확인 뒤, 넓히거나 줄인 판정 대상 없음, 경계 수정):
  - O의 "같은 경로" 조건을 스냅숏 0..k+1에서 0..k로 고쳤다. 피벗 k의 τ 지점 LU는 피벗 k−1 뒤 상태라 스냅숏 k+1(피벗 k의 출력)에 의존하지 않는다. 처음 조건으로는 피벗 21이 빠져 "최소 0–21" 요구를 못 채웠다.
  - F를 "스냅숏 0–22"에서 "0–21"로 고쳤다. 스냅숏 22는 피벗 21의 출력이고, 피벗 21의 진입 변수는 수정 전부터 원본과 다르다(원본 25, Rust 4, τ와 무관한 분기). §3의 "0–22"는 한 칸 어긋난 오기다.
  - `tools/m1_beta_check.py` F의 고정 첫 차이 `(12, 'w')`를 `(22, 'state')`로 바꿨다(β 판정 R·D는 그대로). 수정 뒤 첫 실행에서 이 고정값 때문에 종료 1이었다.
- 회귀(실행 commit을 구분한다):
  - `63d192f` 빌드(최종과 솔버 코드 동일, 이후 차이는 테스트 모듈 import뿐): `tools/m1_tau_check.py --self-test`·`m1_beta_check.py --self-test`·`m1_urow_check.py --self-test` PASS, statecmp·pathcmp·`lp_check`(아래 값), 이 worktree 빌드(`rsolver.__file__`가 worktree 안) pytest 657 passed.
  - `4e391eb`: `cargo test -p rsolver-lp` 41 통과, `cargo test --release --workspace` 122 통과·무시 5(새 `gather_lane_order` 포함), `cargo clippy --workspace --all-targets -- -D warnings` 종료 0.
  - main `35818ce`(M2-06a, docs·tools·bench만) 병합 뒤에는 다시 실행하지 않았다(Rust·Python 패키지 변경 없음).
  - `tools/proto/statecmp.py kb2`: 수정 전 반복 12부터 w 차이(반복 12 w 2개·score 2개 …). 수정 뒤 반복 0–21 모두 OK, 첫 차이는 반복 22(vstat 2개 [4, 25], 피벗 21 진입 선택).
  - `tools/proto/pathcmp.py kb2`: 수정 전후 모두 첫 피벗 분기 21(원본 진입 25, Rust 4, 이탈 78).
  - `tools/lp_check.py --methods 1 --timeout 120`: 수정 전후 정답 90/91, IterCount 일치 24/64 그대로. 상태 변화 없음. 반복 수가 바뀐 모델 4개: degen2 639→644(원본 634), maros 1769→1825(원본 1975), fit2p 7326→8234, greenbeb 5946→5835(둘 다 원본 기준 없음).
  - `tools/m1_urow_check.py --self-test` 종료 0(고정 자료, Rust 미사용 부분), `tools/m1_beta_check.py --self-test` 종료 0(F 재조준 뒤, 변조 12종 거부).
- 실행하지 못한 검사: 8항 이상 모으기·첫 원소 떼기(kb2 없음, asm만), fill 있는 kernel 행, 다른 모델의 τ·행 U(원본 캡처 없음), Method 0/−1 Netlib과 MIP 노드 LP 궤적(같은 dual DSE 경로, pytest만 덮음).
- 고정 덤프 패치(`bench/oracle/m1_02a1_tau/rust_lu_dump.patch`, `m1_02a2_urow/rust_factor_dump.patch`)는 이 코드에 `git apply --check`가 실패한다. 02a-1·02a-2 문서의 덤프 재생성 명령은 각 기준 commit(`79d1d5c`, `7924343`)에서만 쓸 수 있다. 검사기는 고정 덤프 해시로 읽으므로 판정에는 영향 없다.

## 7. 판정과 인수인계

- 작업 수행 상태: **완료(구현)**.
- 동등성: **통과**(계약 범위). kb2 피벗 0–21에서 Rust τ 269개가 원본과 비트(±0 포함)까지 같고, 같은 피벗의 Rust 행 U 순서·값·접두 t가 원본 캡처와 같다. 수정 전 코드는 같은 검사에서 실패했다.
- 남은 한계:
  - τ 모으기 규칙을 모든 피벗에 쓴다. 원본의 희소·커널 경로와 같다는 근거는 kb2 피벗 0–21뿐이다(shortcut 주석).
  - 8항 이상 모으기 레인 결합은 asm 근거만, 첫 원소 떼기는 모델링하지 않았다(shortcut 주석).
  - 행 U 순서: 재분해 피벗 26과 피벗 22 이후 갱신, fill 있는 kernel 행, singleton 열 원소를 가진 singleton 행(C 규칙은 같음)은 실행 대조가 없다.
  - `row_cost_estimate`(`9e60b`)는 행 U를 저장 순서로 접는데, 이번에 순서가 바뀌었다. kb2 statecmp는 `row_cost`를 비교하지만 수정 전후 모두 반복 0–21에서 차이가 없어 접는 순서를 판별하지 못한다(판별 안 됨).
- 후속:
  - **M1-02e**(조사, 새 행 제안): kb2 피벗 21의 진입 선택 차이(원본 25, Rust 4, 이탈 78). 반복 21까지 저장 오라클 상태가 모두 같으므로 dual ratio test(`48d5f` 계열) 입력부터 본다.
  - **M1-02c**(검증): Method=1 Netlib 64 재측정(이번 회귀 24/64가 그대로 출발점).
- 작업 branch·commit: `work/m1-02d`, worktree `../rsolver-m1-02d`. 병합 상태는 [M1 인수인계](../handoff/M1.md)와 main 이력으로 확인한다.
- 갱신 위치: [M1 인수인계](../handoff/M1.md), [스프린트 목록](../14_sprint_plan.md) M1-02d 행과 새 행, [작업 단위](../13_work_units.md) M1-02, [진행 기록](../11_progress.md)·[로드맵](../12_roadmap_diagram.md) 맨 위.
- 교훈 기록: [원본 사실](../lessons/original.md) "DSE τ" 항목(8레인 결합 asm)과 "U 행 저장 순서" 항목(singleton 행 규칙 C 근거, Rust 반영), [도구](../lessons/tooling.md) "피벗 k의 진입 상태와 스냅숏 경계".
