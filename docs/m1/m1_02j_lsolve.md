# M1-02j 검증 계약과 결과: degen2 첫 차이(희소 L 출력 순서)

작성 기준은 [스프린트 검증 기준](../15_sprint_verification.md)이다. [M1-02h](m1_02h_aq_order.md) §7의 후속이다. 사용자의 "다음 스프린트 이어서" 지시로 착수했다(2026-11-11). M2는 다른 세션이 직전에 M2-06c를 점유해 M1을 골랐다([시작 절차](../handoff/README.md) §1-3).

## 1. 범위와 주장

- 작업 종류: 조사(새 읽기 전용 캡처 포함). 구현은 원본 규칙과 필요한 상태를 Rust 값으로 정할 수 있을 때만 같은 branch 둘째 단계로 넣는다(§7에서 넣음).
- 조사 질문: degen2(Method=1, Presolve=0, Threads=1) `pathcmp` 첫 차이 피벗 288(진입 509 같음, 이탈 원본 164 대 Rust 738)의 원인. 처음 다른 입력과 그 원본 규칙은 무엇인가. 02h가 근사로 둔 희소 L(`21b9de`)·행 eta(`21f83a`, 인라인 희소) 출력 순서의 원본 규칙은 무엇인가.
- 포함 분기: dual DSE 피벗(`8b473` ← `28c93`)의 두-RHS FTRAN `09d112`가 부르는 `21b9de`(네 호출 지점)와 `21f83a`(두 지점). degen2 피벗 0–633, kb2 피벗 0–37.
- 제외 범위와 후속:
  - 반복 263 w 차이(§6 (II), τ 값 끝자리): 별도 후속 번호.
  - `21b9de` 경로 B(LU+0xe4 위 꼬리를 순차로 훑는 경로): degen2·kb2에서 0회면 C 구조만 기록하고 미검증.
  - primal `2207de`·`9d0b7`의 `21b9de` 호출: 이 캡처 범위 밖.
- 선행: main `faae34d`. 미커밋 선행 변경 없음.

## 2. 원본과 상태 경계

- Gurobi 13.0.3 `libgurobi130.so` SHA-256 `8e87fca2…0e98f9c`. 모델 `../re/bench/data/netlib/degen2.mps.gz`, `kb2.mps.gz`.

| 위치 | 내용 |
|---|---|
| `21b9de` `0x12a990` C:60–80 | 인자 `(tol, LU, m, L = LU+0x20, in, out, mark, work, bucket, &work)`, 입력·출력은 L 인덱스 공간(`09d112` C:670–677이 LU+0x30으로 바꿈). 경로: `m − e4 < 6` 또는 `10(m − e4)² ≤ m` 또는 `beg[m] − beg[e4] ≤ 10(m − e4)`(e4 = LU+0xe4)이면 A, 아니면 B. A 안에서 LU+0x128 < 0 또는 ≥ 0.1m이면 A1, 아니면 A2 |
| C:81–257 (A1) | Kahn FIFO. 목록 = 입력 순서, 각 입력 차수 +1, 목록을 앞에서 훑으며 L 열 원소(저장 순서)의 차수 +1, 처음 닿은 행을 덧붙임. 값 단계: 입력 순서대로 x = 값, 차수 −1, 0이면 큐. 큐 FIFO: `|v| ≥ tol`이면 출력하고 열 원소에 `x_i = −(l·v) + x_i`, 차수 −1, 0이면 큐. 작으면 차수만 내림 |
| C:259–605 (A2) | 레벨 버킷. LU+0x118 레벨, LU+0x120 레벨 시작. 입력을 입력 순서로 각 레벨 버킷에 넣고 최소 레벨부터 올라간다. 버킷 안은 넣은 순서. `tol ≤ |v|`이면 출력·전파, 처음 닿은 행은 그 레벨 버킷 끝에 넣는다. 작은 값은 전파하지 않는다 |
| C:607–893 (B) | e4 아래만 Kahn(`|v| > tol` 출력), e4 이상은 닿은 최소 인덱스부터 m−1까지 순차(`|v| > tol`) |
| `036ade` C:5764–5830 | L 레벨: `LU+0x118 ≠ 0`이고 m ≥ 250이고 m > LU+0x104일 때 열 j = 0..m−1을 앞으로 훑어 `lev[i] = max(lev[i], lev[j]+1)`, LU+0x128 = 최대 레벨, 시작 = 레벨별 수의 누적합. 아니면 LU+0x128 = −1 |
| `09d112` C:1376–1553 (인라인 희소 R) | 입력 행 표시, 입력 행들의 LU+0x1a0(그 행을 읽는 첫 eta) 최솟값부터 최신 eta까지 `x_r = η·x`. `|v| ≤ tol`이면 x_r = 0, 아니면 표시 안 된 r을 목록 끝에 덧붙인다. 끝에 목록 순서로 `|x| ≥ tol`만 남긴다 |

- 시작 경계: 피벗 k의 `09d112` 호출 진입(`8b473`에서, 반환 주소 `0x120200`). 종료 경계: 그 `09d112` 반환.
- 읽는 상태: `21b9de` 입력 벡터(인덱스·값·순서), L(열 시작·행·값, 레벨·레벨 시작·LU+0x128, LU+0xe4, 행→L 순열 LU+0x30). tol은 xmm0이라 훅에서 읽지 못한다. 모델은 Rust `ZERO`(1e-13)를 쓰고, 출력 일치로 간접 확인한다.

## 3. 비교 계약

| 검사 | 규칙 |
|---|---|
| M 입력 | 새 캡처 degen2·kb2 각 3회 gz SHA-256 고정·바이트 동일, 정상 종료, 바이너리·모델·도구 해시 |
| N 비간섭 | 새 캡처 `aq_out` 인덱스 = 02a-1 형식 캡처(`tools/m1_tau_path.js`, degen2 새 1회·kb2 `bench/oracle/m1_02a1_tau/kb2_1`)의 같은 피벗 `aq_out` |
| L 커널 | 모든 `21b9de` 호출: §2 모델(경로 판정 포함)이 캡처 L·입력으로 계산한 출력 인덱스 순서와 값 = 캡처 출력(비트) |
| R 순서 | 모든 `21f83a` 호출: 희소 쪽 출력 = L 출력 순서에서 작아진 행을 빼고 새 행을 끝에 덧붙인 것(앞부분 순서 보존). 덧붙는 행의 eta 순서는 캡처에 eta가 없어 Rust 연결 검사(§7)로만 판정 |
| −1 행 | 희소 U(`21686f`·`21841f`)이고 α_q 쪽 L이 희소인 피벗: 최종 α_q의 LU+0xf8 = −1 행 순서 = L 출력의 −1 행 순서 |
| T 변조 | (`--self-test`) 바이너리 해시 0, 빈 사건, 출력 두 원소 교환, 값 한 비트, A2를 A1로 바꾼 모델, 버킷 안 역순 모델. 모두 거부 |

- 값은 IEEE 비트 비교다. 순서는 정확 비교다.

## 4. 사례와 원본 자료

| 사례 | 실제 / 합성 | 노리는 분기 | 입력 | 원본 출력 |
|---|---|---|---|---|
| degen2 Method=1 Presolve=0 Threads=1 | 원본 실제 실행 | m = 444 ≥ 250: A2(레벨), `21f83a` 두 지점, 인라인 희소 R | `degen2.mps.gz` | `bench/oracle/m1_02j/degen2_ls_{1,2,3}.json.gz`, `degen2_tau_1.json.gz`(02a-1 도구), `degen2_260_300.jsonl.gz`(spx_oracle 반복 260–300) |
| kb2 같은 설정 | 원본 실제 실행 | m < 250: A1(Kahn) | `kb2.mps.gz` | `bench/oracle/m1_02j/kb2_ls_{1,2,3}.json.gz` |

- 캡처 도구 `tools/m1_lsolve.js`(새, 읽기 전용): `8b473`·`09d112` 경계는 `tools/m1_tau_path.js`와 같고, `21b9de`·`21f83a` 입출력과 L 덤프(바뀔 때마다)를 더했다.
- 적용하지 않은 범주: 경로 B(두 모델에서 0회), A1의 작은 값 분기(실행 여부는 결과에서 셈).

## 5. 재실행 명령

- 원본 캡처(작업 worktree 루트, Gurobi 한 프로세스씩): `for r in 1 2 3; do for m in degen2 kb2; do tools/gurun.sh node tools/m1_lsolve.js ../re/bench/data/netlib/$m.mps.gz bench/oracle/m1_02j/${m}_ls_$r.json.gz; done; done`, `tools/gurun.sh node tools/m1_tau_path.js ../re/bench/data/netlib/degen2.mps.gz bench/oracle/m1_02j/degen2_tau_1.json.gz`.
- 판정: `python3 tools/m1_lsolve_check.py --self-test`. Rust 연결(§7): `tools/build_local.sh && PYTHONPATH=$PWD/python ../.venv/bin/python tools/m1_lsolve_check.py --rust --self-test`.
- 상태 대조: `zcat bench/oracle/m1_02j/degen2_260_300.jsonl.gz > <tmp>/d.jsonl; PYTHONPATH=$PWD/python ../.venv/bin/python tools/proto/statecmp.py degen2 <tmp>/d.jsonl`.

## 6. 실행 결과(조사)

- 상태 대조(`tools/spx_oracle.py` degen2 반복 260–300, `statecmp`): 260–262 같음. 처음 다른 것은 반복 263의 w(var 755) 끝자리이고(아래 (II)), 가격 목록 순서는 275(mode 0)·288(mode 1)에서 다르다.
- **(I) 288의 원인(순서).** 피벗 288 이탈 후보 var 164(위치 1)와 738(위치 183)은 점수·w(26.75)·|xB|(18.5)가 양쪽에서 비트까지 같은 동률이다. 동률은 mode 1 목록 순서가 가르고, 그 꼬리는 피벗 287이 붙인 α_q 목록(원본 `[73, 1, 183, 159, …]`, Rust `[73, 183, 1, 159, …]`)이다. 둘의 차이는 LU+0xf8 = −1 행 순서이고, α_q 목록은 피벗 3부터 이미 다르다(동률이 없어 경로는 288까지 같았다). −1 행 순서는 희소 L 커널 `21b9de`의 출력 순서다.
- `python3 tools/m1_lsolve_check.py --self-test` 종료 0(캡처 고정 해시, 3회 바이트 동일 degen2 `db345431…`, kb2 `6160a0c3…`):

```
degen2: 21b9de 695 calls, paths {'A2': 695}; 21f83a {'same': 50, 'appended/dropped': 78}; -1 rows = L output on 267 sparse-U pivots (162 with >= 2 such rows)
kb2: 21b9de 19 calls, paths {'A1': 19}; 21f83a {'appended/dropped': 3}; -1 rows = L output on 7 sparse-U pivots (4 with >= 2 such rows)
T 변조·모델 변이 7종 모두 거부
```

  - L 커널: degen2(m = 444, LU+0x128 1–30) 695회 전부 A2 레벨 경로, kb2(m = 41, LU+0x128 = −1) 19회 전부 A1 Kahn FIFO. 출력 인덱스 순서와 값이 모델과 비트까지 같다(tol = 1e-13로 계산). A1의 작은 값 분기와 경로 B는 0회.
  - N: 새 캡처 `aq_out` = 02a-1 형식 캡처(degen2 새 1회, kb2 `m1_02a1_tau/kb2_1`)의 같은 피벗.
  - R: `21f83a` 128회. 희소 쪽 출력은 L 출력 순서를 유지하고 작아진 행을 뺀 뒤 새 eta 행을 끝에 붙인다(덧붙임 78회, 뺌 포함). 조밀 쪽 출력은 nnz −1.
  - −1 행: 희소 U이고 α_q 쪽 L이 희소인 피벗 267(+kb2 7)에서 최종 α_q의 −1 행 순서 = L 출력의 −1 행 순서. R 단계는 −1 행을 더하지 않는다(eta 대상 행은 꼬리).
- **(II) 반복 263 w(값, 별개).** 피벗 261 τ 위치 216·217이 원본 0.0, Rust −4.44e−16이고, 피벗 262 τ[317]이 원본 −283.0000000000001, Rust −283.0이다. β·α_r·α_q 쪽은 같다. 두 피벗의 ρ 쪽은 조밀 L(`214344`)이라 (I)과 무관하다. ρ의 L·R 뒤 값이 캡처에 없어 L·R·U 중 어디인지는 미확인이다(후속).
- 탐색: degen2 L 덤프 15회의 LU+0xe4는 Rust `n_singleton`과 경로가 같은 분해 9회에서 모두 같다.

## 7. 구현(같은 branch 둘째 단계, 범위 확대)

§6이 원본 규칙과 필요한 상태(L 열 저장 순서, L 레벨, LU+0xe4 = `n_singleton`)를 Rust가 가진 값으로 정했으므로 구현을 넣었다. 계약 §3에 Rust 연결 검사(아래 K·U·S)를 더했다. 줄인 판정은 없다.

- `lu.rs`: `finish`가 L 레벨(`036ade` C:5764–5830, Lᵀ 레벨과 같은 조건 m ≥ 250·LU+0x104 < m)과 LU+0x128, LU+0xe4 위 L 원소 수를 만든다. `l_sparse`가 `21b9de` A1·A2를 값까지 계산한다(경로 B 판정이 참이면 None → 조밀, `shortcut:` 주석). `ftran_lr`이 `09d112`의 한 RHS(L 희소 조건 `0x104·nnz ≤ m`·`0.1m > LU+0x128`, R 단계 목록 유지)를 처리해 U 단계가 받는 행 순서를 돌려준다. `ftran_r_list`가 R 단계 덧붙임·뺌. `ftran2`는 근사(입력 행 + 위치 오름차순) 대신 그 목록을 받는다.
- `simplex.rs`: `ftran_col`이 열 입력 순서로 `ftran_lr`(희소 L 값은 그 커널 값), dual DSE의 τ는 BTRAN 출력 순서의 ρ로 `ftran_lr`. 스냅숏 `lro`(α_q 쪽 L+R 순서, 위치). `fill` 필드 삭제.
- `python3 tools/m1_lsolve_check.py --rust --self-test`(worktree 빌드) 종료 0:

```
degen2 Rust: path = original on pivots 0-356, update count on 0-355 (pivot 356: Rust 40 updates, original 0); {'K': 356, 'U': 238, 'U >= 2': 137, 'S': 210, 'S inline R (no capture)': 146}
kb2 Rust: path = original on pivots 0-37, update count on 0-37; {'K': 38, 'U': 7, 'U >= 2': 4, 'S': 34, 'S inline R (no capture)': 4}
```

  - 비교 범위는 경로가 같고 FTRAN 시점 갱신 수(Rust는 피벗마다 마지막 스냅숏)가 같은 피벗이다. K = U 커널, U = α_q의 −1 행 순서, S = α_q 쪽 L+R 순서(R 없음은 L 출력, `21f83a`는 그 출력과 비교. 인라인 희소 R은 캡처가 없어 센다).
  - Rust 변조 3종(L+R 두 칸 교환, `lro` 없음, kb2 피벗 21 커널) 거부.
  - 코드 변이(임시 빌드, 되돌림): 레벨 버킷 LIFO → degen2 피벗 0 U·S 거부, R 덧붙임 제거 → degen2 피벗 130 S 거부, Kahn 큐 LIFO → kb2 피벗 0 U·S 거부. "큐를 뒤에서 읽는" 변이 하나는 생존했다(kb2 Kahn 큐가 짧아 같은 순서).
- 02h 검사기 `m1_aq_order_check.py --self-test`: F·A 38피벗 전부 규칙 안에서 원본과 같다(전에는 F 1–5·A 0–4가 근사로 범위 밖). M1 검사기 8종 PASS(`m1_eta_check`의 dense 변이 생존은 02i 기대대로).
- 회귀: `cargo test --release --workspace` 125 통과·10 무시, `cargo clippy --workspace --all-targets -- -D warnings` 0, worktree 빌드 pytest 657 passed.
- Netlib(기준선 main `faae34d`를 `../rsolver-m1-02j-base`에 따로 빌드, `tools/lp_check.py --methods 1,0,-1 --timeout 120`, `tools/proto/pathcmp.py all`; clippy 대응 `x −= l·v` 표기 변경 전 빌드, 같은 비트 연산):
  - 정답 90/91 ×3 불변. IterCount 일치 Method 1·−1 32 → 31/64(israel 150 → 146. 첫 차이 84는 그대로라 우연 일치가 깨진 것), Method 0 0/64.
  - pathcmp 완전 일치 26 → 27/64(bore3d), 중앙 첫 차이 비율 0.624 → 0.752. 늦춰진 모델: bore3d 57 → 끝, degen2 288 → 357, fffff800 56 → 181, forplan 101 → 128, scfxm2 280 → 366, scfxm3 359 → 488, standmps 73 → 96. 당겨진 모델: boeing1 381 → 368.
  - boeing1(탐색 캡처 `bench/oracle/m1_02j/explore/`): L 모델 576/576(A2). 반복 300에서 기준선도 이번 빌드도 이미 xB·d·w가 다르다(기준선 xB 35·w 130개, 이번 2·97개). 그래서 368은 이미 갈린 상태 안의 이동이다. Rust 연결 검사 실패는 U 커널 6피벗(188 등, Rust BTRAN이 ρ를 조밀로 냄 → M1-02k 영역, 이전 코드도 같음)과 L+R 순서 2피벗(345·355, 첫 원소의 위치 번호만 다름 → 재분해 단계 순서 차이 후보, 미확인).
- 다음 첫 차이(degen2, 이번 범위 밖):
  - 피벗 8: 희소 U 출력의 접두 행(LU+0xf8 ≥ 0) 순서. 원본 `[239(6), 183(5), 202(4), 50(0)]`, Rust 슬롯 오름차순. `21686f` C(param_3 = LU+0x20): `m ≤ nnz·D²`이면 목록 순, 아니면 접두 입력을 "구간 2" 목록에 두고 꼬리를 힙(U 위치 내림차순)으로 풀며, `2.5·local_b8 ≤ LU+0xec`이면 꼬리 출력 열의 접두 부분([+0x78, +0x70))을 흩뿌려 처음 닿은 순, 아니면 LU+0xf0 목록 순으로 접두를 낸다(C:160–420). degen2 희소 U 267피벗 중 접두 순서가 슬롯 오름차순이 아닌 것 `21686f` 57·`21841f` 12. `21841f` C는 읽지 않았다.
  - 피벗 13 → 14: LU+0xf0 목록에서 여러 행이 함께 빠질 때 채움 순서(02h 미검증 항목, `05b257`).
  - 피벗 356: 원본은 갱신 39개 뒤 재분해하고 Rust는 계속한다(재분해 판정). 경로 첫 차이 357의 직접 후보.

## 8. 판정과 인수인계

- 작업 수행 상태: **완료(조사·구현)**.
- 동등성:
  - 조사: 질문에 답했다. 288 = 정확한 동률을 가르는 mode 1 목록 순서 = α_q −1 행 순서 = `21b9de` 출력 순서. A1·A2·R 목록 규칙은 원본 캡처와 **통과**(degen2·kb2, 비트).
  - 구현: **통과**(계약 범위: degen2 피벗 0–355, kb2 피벗 0–37의 U 커널, α_q −1 행 순서, α_q 쪽 L+R 순서). 인라인 희소 R 순서는 원본 출력 캡처가 없어 **미검증**(Rust 규칙은 C:1376–1553과 `21f83a` 관측에서 옮김).
- 남은 한계: 경로 B(LU+0xe4 위 꼬리 순차, C:607–893) 미이식·0회. A1 작은 값 분기 0회. 다른 호출자(`2207de` primal·`9d0b7`)의 L 커널 값은 같은 함수를 쓰지만 순서·경로 캡처 없음. tol은 훅에서 못 읽어 1e-13 가정(출력 일치로 간접 확인). boeing1 L+R 위치 번호 2피벗 미해명.
- 후속:
  - **M1-02m**(조사·구현): 희소 U 커널 `21686f`·`21841f`의 접두 행 출력 순서(§7 첫 항). 첫 대조 degen2 피벗 8. 첫 명령 `python3 tools/dc.py 21841f` 뒤 `21686f` C:160–420과 대조.
  - **M1-02n**(조사): degen2 피벗 13 → 14 LU+0xf0 여러 행 이탈 순서와 피벗 356 재분해 판정.
  - **M1-02o**(조사): degen2 피벗 261·262 τ 값(§6 (II)). ρ 쪽 L·R 뒤 값을 캡처에 더해 L·R·U 중 처음 다른 단계를 찾는다.
- branch `work/m1-02j`(기준 main `faae34d`), worktree `../rsolver-m1-02j`, 기준선 빌드 `../rsolver-m1-02j-base`(병합 뒤 정리). 미커밋 선행 변경 없음.
- 인수인계·진행: `docs/handoff/M1.md`, `docs/11_progress.md`·`docs/12_roadmap_diagram.md` 맨 위, `docs/14_sprint_plan.md` M1-02j 행과 02m·02n·02o 새 행.
- 교훈: `docs/lessons/original.md`(`21b9de` 세 경로와 L 레벨, R 단계 목록, 동률이 가린 순서 차이), `docs/lessons/tooling.md`(재분해 뒤 스냅숏이 같은 반복으로 두 번 써짐).
