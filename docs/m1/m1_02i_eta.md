# M1-02i 검증 계약과 결과: kb2 반복 23 d·w 끝자리 차이의 원인(BTRAN 행 eta 단계)

작성 기준은 [스프린트 검증 기준](../15_sprint_verification.md)이다. [M1-02h](m1_02h_aq_order.md) §7의 첫 다음 작업이다. 사용자의 "다음 스프린트 진행" 지시로 착수했다(2026-11-11). M2는 다른 세션이 직전에 M2-06f-4를 점유하고 worktree를 만들어 M1을 골랐다([시작 절차](../handoff/README.md) §1-3).

## 1. 범위와 주장

- 작업 종류: 조사·구현. 착수 때는 조사였고, 원인 규칙을 저장 자료 재생으로 확정한 뒤 같은 branch에서 Rust 수정을 둘째 커밋으로 더했다(§8, 범위 확대 기록).
- 조사 질문: kb2(Method=1, Presolve=0, Threads=1) 스냅숏 23에서 d 6·w 21·점수 3개가 끝자리만 다르다(02h 뒤에도 같은 값). 피벗 22에서 처음 다른 입력은 무엇이고 원본은 어떤 규칙으로 계산하는가.
- 포함 분기: dual DSE 피벗의 BTRAN `ρ_r = e_rᵀB⁻¹`(`9e376` → `221173`)에서 행 eta(LU+0x180 > 0)가 있는 경우. kb2 피벗 0–30(Rust U가 원본과 같은 범위).
- 제외 범위와 후속:
  - 피벗 30 갱신의 새 U 열(spike) 값 끝자리 차이(행 17·36, 피벗 31부터 U가 다름): FTRAN R 단계 8레인 내적의 선처리 정렬로 보이나 미확인. 후속 **M1-02l**(조사).
  - 희소 eta 경로(`0x104·nnz ≤ m`, `v ≠ 0.0`만 검사): kb2 실행 0회, C만.
  - e_r 밖의 BTRAN 입력에서 Uᵀ 조밀·희소 판정(`(int)(0x108+0x110)·nnz ≤ m`): M1-02k.
- 선행: main `63ec7ab`. 미커밋 선행 변경 없음.

## 2. 원본과 상태 경계

- Gurobi 13.0.3 `libgurobi130.so` SHA-256 `8e87fca2…0e98f9c`(`tools/m1_beta_check.py` `BIN`). 모델 `../re/bench/data/netlib/kb2.mps.gz`.

| 위치 | 내용 |
|---|---|
| `221173` `0x12de60` C:80–83 | `dVar2` = LU+0x10(ZERO 1e-13), `iVar19` = LU+0x104, `uVar3` = LU+8(m), `iVar25` = LU+0x180(eta 수), `lVar47` = LU+0x198(eta 레코드 {r, len, idx*, val*}) |
| C:86–90 | 일반 경로: nnz ≥ 0이고 `(int)(0x108+0x110)·nnz ≤ m`이면 희소 Uᵀ(`LAB_0022e567`), 아니면 조밀 Uᵀ |
| C:1197–1208 | `9e376`(param_4 = LU+0x1d0): U 행 r이 비면 일반 경로, 아니면 0x1d8 = r, 희소 Uᵀ `21a941` 강제. 결과를 0x1d0에 복사(C:1213–1227) |
| `21a941` `0x130b70` C:81 등 | 출력은 `ZERO < |x|`인 원소만 |
| C:1231–1236 | eta 없음 → Lᵀ(`LAB_0022ec37`) |
| C:1237–1383 | Uᵀ 출력이 희소이고 `m < 0x104·nnz`: 조밀 배열에 흩고 **조밀 eta**(최신 → 오래된). `v = w[r]`, `|v| ≤ ZERO`면 `w[r] = 0`으로 지우고 건너뛴다. 아니면 저장 순서로 `i = r`이면 `w[r] = η·v`, 아니면 `w[i] = η·v + w[i]`(2개씩 펼침, 연산은 곱 뒤 더하기). 출력 nnz = −1 |
| C:1384–1494 | 그 밖(`0x104·nnz ≤ m`): **희소 eta**. `v ≠ 0.0`만 검사, 새로 닿은 원소를 목록에 붙이고 끝에 `|w| ≥ ZERO`만 목록 순서로 남긴다. 출력 nnz = 남은 수, 0이면 반환 |
| C:1496–1572 | Uᵀ가 조밀이었으면(`LAB_0022e394`) 같은 조밀 eta |
| C:1573–1766 | 조밀 eta 뒤(출력 −1)는 조밀 Lᵀ: 위치 m−1 → 0, `ZERO < |v|`만 전파, 작은 값은 출력에 남는다. 출력 nnz = −1 |

- 시작 경계: 피벗 k의 `9e376` 호출(진입 전 LU 상태 = 02a-1 캡처 피벗 k `ft`의 U·LU+0x104·0x180). 종료 경계: `221173` 반환의 ρ(02a 캡처 `bt`, `8b473`에 넘기는 압축 ρ와 같은 벡터).
- 읽는 상태: U(대각 LU+0x50, ord LU+0x48, U 행 LU+0xb0/0xb8/0xd0/0xc8 저장 순서), 행 eta 레코드, L(Rust와 같다고 가정, §6 R이 간접 확인), LU+0x104.

## 3. 비교 계약

새 `tools/m1_eta_check.py`가 판정한다. 실패 하나면 종료코드 1.

| 검사 | 규칙 |
|---|---|
| M 입력 | 02a β 캡처 3회·02a-1 τ 캡처 3회 gz SHA-256(기존 고정값), 바이너리·모델·파라미터, 3회 사건 같음. Rust 덤프 `rust_bt_kb2.jsonl.gz`·패치 SHA-256 |
| U 상태 | 덤프의 피벗 k(`btran_updated` 호출, 피벗 0·6은 대각뿐이라 없음)마다 Rust U(대각 비트, ord, U 행 저장 순서·값 비트) = 캡처 `ft`. 피벗 0–30 필수, 첫 차이 피벗 출력 |
| E eta | 피벗 0–30에서 Rust eta 수 = LU+0x180, 각 eta = 그 갱신 피벗 캡처 U로 계산한 `Uᵀ⁻¹e_r`(r 먼저 U 위치 오름차순, `ZERO < |z|`)과 인덱스·순서·비트 같음 |
| Y Uᵀ | 피벗 0–30에서 Rust Uᵀ 결과 = 캡처 U에 조밀 Uᵀ(§2)를 적용한 값(비트) |
| R 재생 | eta가 있는 피벗마다 Rust Uᵀ 결과에 원본 규칙(§2 C:1237–1572)과 조밀 Lᵀ(Rust L)를 적용한 ρ = 캡처 ρ(행별 비트). 옛 Rust 규칙(항상 희소 eta)은 정확히 피벗 [22, 25]에서 틀려야 한다(판별) |
| F 새 실행 | (`--fresh`, 수정 뒤) 이 worktree 빌드의 `RSOLVER_SNAP` β(스냅숏 k+1) = 캡처 β, 피벗 0–30. `statecmp.py kb2` 0–30 전부 OK |
| T 변조·변이 | (`--self-test`) 캡처 ρ 한 비트, Rust U 행 순서, eta 원소 하나 삭제, Uᵀ 결과 끝비트 변조는 거부. 규칙 변이 rust·sparse(항상 희소)·fma(융합 곱셈) 거부, dense(항상 조밀)는 생존 기대(kb2 희소 eta 0회) |

- 값 비교는 모두 IEEE 비트 비교다. ±0은 비트로 구분한다(캡처 ρ의 0은 `+0`).

## 4. 사례와 원본 자료

| 사례 | 실제 / 합성 | 노리는 분기 | 입력 | 원본 출력 |
|---|---|---|---|---|
| kb2 Method=1 Presolve=0 Threads=1 | 원본 실제 실행 저장 자료 | eta 피벗 10–14·18–25·27–30(17개, 모두 조밀 eta), `|v| ≤ ZERO` 지움 피벗 22·24·25 | `kb2.mps.gz` | `bench/oracle/m1_02a_beta/kb2_{1,2,3}.json.gz`(ρ·β), `bench/oracle/m1_02a1_tau/kb2_{1,2,3}.json.gz`(U·LU 필드) |

- Rust 측 자료: main `63ec7ab`에 `bench/oracle/m1_02i_eta/rust_bt_dump.patch`(임시 덤프 `RSOLVER_BTDUMP`·`RSOLVER_RHODUMP`, 커밋하지 않음)를 적용한 빌드의 kb2 실행 `rust_bt_kb2.jsonl.gz`.
- 새 원본 캡처는 하지 않았다(Gurobi 미실행). 기존 캡처 3회의 재현성 기록(02a·02a-1)을 그대로 쓴다.

## 5. 재실행 명령

- 저장 기준: `PYTHONPATH=$PWD/python ../.venv/bin/python tools/m1_eta_check.py --self-test`.
- 새 실행 포함(수정 뒤): `tools/build_local.sh && PYTHONPATH=$PWD/python ../.venv/bin/python tools/m1_eta_check.py --self-test --fresh`.
- 덤프 재생성: `git apply bench/oracle/m1_02i_eta/rust_bt_dump.patch`(main `63ec7ab`), 빌드 뒤 `RSOLVER_BTDUMP=<f> RSOLVER_RHODUMP=<f>`(같은 파일)로 kb2 Method=1 Presolve=0 실행, gzip `-n`.

## 6. 실행 결과(조사)

- 첫 차이 국소화: statecmp 반복 0–22 OK, 23에서 d 6·w 21·점수 3. 피벗별로 β를 보면 피벗 22가 처음 다르다(원본 `23.51288018963392`, Rust `…391`). α_r는 같다. 캡처 ρ를 Rust 합산 규칙(`sum_squares`)으로 더하면 원본 β가 나온다 ⇒ 합산이 아니라 ρ가 다르다. 덤프 ρ는 피벗 0–21 모두 비트 같고 피벗 22(FT 갱신 7, eta 5, 조밀 출력)에서 7원소가 끝자리로 다르다.
- `tools/m1_eta_check.py --self-test` 종료 0:

```
U: Rust U = capture (diag, ord, U rows in storage order) on pivots 0..30; first differing pivot 31
E/Y: Rust etas = U^T^-1 e_r of the captured U (61 eta checks) and Rust U^T result = dense U^T on the captured U, pivots 0..30
R (orig): rho replay = capture on 17/17 pivots with etas [10, 11, 12, 13, 14, 18, 19, 20, 21, 22, 23, 24, 25, 27, 28, 29, 30]; paths {'dense': 17, 'sparse': 0}; |v| <= ZERO cleared [(22, 3, '-1.11e-16'), (24, 3, '2.78e-17'), (25, 3, '-1.42e-14')]
R: the pre-M1-02i Rust rule (sparse etas always) fails on [22, 25] (discriminating)
T 변조 4종 거부, 규칙 변이 rust·sparse·fma 거부, dense 생존(기대)
```

- **처음 다른 입력과 원본 규칙**: 피벗 22 BTRAN의 eta 단계. Uᵀ 출력 nnz 5, LU+0x104 = 37, m = 43이라 `m < 0x104·nnz`로 원본은 조밀 eta를 탄다. 최신에서 넷째 eta(행 3, 피벗 18 갱신)의 `v = w[3]`가 소거 잔차 `−1.11e−16`이고, 원본은 `|v| ≤ ZERO`로 `w[3] = 0`을 쓰고 그 eta를 건너뛴다. Rust `btran_updated`는 항상 희소 규칙(`v ≠ 0.0`)이라 그 eta를 적용했다. 피벗 25(`−1.42e−14`)도 같은 원인이고, 피벗 24(`2.78e−17`)는 지워도 ρ가 같다.
- U·eta·Uᵀ 결과는 피벗 0–30에서 원본과 같다(U는 캡처 대조, eta·Uᵀ는 캡처 U에 원본 규칙을 적용한 값과 대조). 즉 `lu.rs` `btran_updated` 주석의 "eta가 교과서 FT 승수" shortcut은 이미 사실이 아니다(현재 Rust `update`도 `Uᵀ⁻¹e_r`를 저장).
- 피벗 31 이후: Rust U가 다르다. 피벗 30 갱신으로 들어간 새 U 열 16의 행 17(`−274.08375732342705` 대 원본 `…71`)·행 36(`−6147.615950512084` 대 `…085`). 행 17의 eta(피벗 27 갱신)는 길이 8이라 spike의 R 단계 8레인 내적(`dot8`, 선처리 없음 shortcut)을 탄다. 행 36의 eta(피벗 28 갱신, 길이 4)는 행 17 값을 계수 22.43으로 읽으므로 행 36 차이는 행 17의 하류로 보인다. 근원 후보는 길이 8 eta 하나의 내적이다(n = 8에서 원소 하나를 선처리하면 남은 7개가 순차 합이 된다, 추정). 이번 범위 밖(M1-02l).
- 실행하지 않은 검사: Gurobi 새 캡처(저장 자료로 충분, 3회 재현 기록 재사용), 희소 eta 경로(kb2 0회).

## 7. 판정과 인수인계(조사)

- 작업 수행: 완료. 동등성: 원인 규칙이 kb2 eta 피벗 17/17에서 캡처 ρ를 비트 재현(저장 기준). 희소 eta·조밀 eta 뒤 Lᵀ 강제는 C만.
- 후속: Rust 수정(§8), **M1-02l**(조사, 새 행): 피벗 30 spike 행 17 끝자리(길이 8 eta의 FTRAN R 단계 내적 선처리 정렬 후보, 행 36은 그 하류로 보임). 첫 명령 `PYTHONPATH=$PWD/python ../.venv/bin/python tools/m1_eta_check.py`(U 첫 차이 31 출력), 그다음 `09d112` R 단계(인라인, C:700–1372)의 8레인 내적 정렬 조건을 C로 읽는다.
- 교훈: [original.md](../lessons/original.md) "LP BTRAN 행 eta", [tooling.md](../lessons/tooling.md) 검사기(β 첫 차이에서 ρ·합산 가르기), [process.md](../lessons/process.md) 점유 owner 항목에 이번 관측 추가.
- branch `work/m1-02i`: 조사 커밋 `e8c9de8`, 수정 커밋 `8ae74dd`, 기록 커밋(진행·로드맵·스프린트 표·M1 인수인계·교훈). 미커밋 없음.

## 8. Rust 수정(범위 확대)

- 이유: §6이 규칙을 C와 저장 자료 재생으로 확정했고 수정이 M1 파일 한 곳(`crates/lp/src/lu.rs` `btran_updated`)에 머문다. 같은 오라클(§3 F·S)로 바로 판정할 수 있어 같은 branch 둘째 커밋으로 넣었다. 조사 커밋 `e8c9de8`은 수정 없이도 병합 가능한 상태로 먼저 만들었다.
- 변경: 행 eta가 있고 `m < LU+0x104·nnz(Uᵀ 출력)`이면 조밀 eta(`|v| ≤ ZERO`면 `w_r = 0`, 건너뜀)와 조밀 Lᵀ(작은 값 포함 전체를 넘김, 출력 조밀), 아니면 기존 희소 규칙. 낡은 "교과서 FT 승수" shortcut 주석을 지우고 남은 근사(Uᵀ 조밀 계산, C:89 Uᵀ 경로 판정 미반영 → M1-02k)를 `shortcut:`에 적었다. 공유 코드라 모든 BTRAN 호출자(dual ρ, 쌍대값·primal의 c_B, 초기 DSE 가중치)에 적용된다. 초기 가중치는 재분해 직후라 eta가 없어 영향이 없다.
- `tools/m1_beta_check.py`: 고정 첫 차이 `FIRST`를 `(23, "state")` → `None`(스냅숏 0–30 전부 같음, 이전 값은 주석에 보존).

### 8.1 실행 결과(worktree `../rsolver-m1-02i`, `tools/build_local.sh` 빌드)

- `tools/m1_eta_check.py --self-test --fresh` 종료 0: §6 줄에 더해 `F: fresh Rust beta = capture on pivots 0..30 (31/31); differing after: [31, 33, 34, 35, 36, 37]`, `S: statecmp kb2 31/31 snapshots OK`(전: 반복 23부터 차이).
- ρ 직접 대조(일회, 덤프 패치의 `simplex.rs` 부분만 수정 빌드에 임시 적용 뒤 되돌림): 피벗 0–30 ρ가 캡처와 집합·순서·비트 모두 같다. 31·33–37은 다르다(§6의 U 차이), 32는 같다.
- 기존 검사기 `--self-test`: `m1_beta_check`(위 고정값 변경 뒤) `m1_tau_check` `m1_urow_check` `m1_list_order_check` `m1_aq_order_check` `m1_bfrt_rust` 모두 PASS.
- `cargo test -p rsolver-lp` 41(무시 1), `cargo test --release --workspace` 124 통과·10 무시, `cargo clippy --workspace --all-targets -- -D warnings` 0, worktree 빌드 `pytest` 657 passed.
- Netlib(기준선 = 같은 날 main `63ec7ab`을 `../rsolver-m1-02i-base`에 따로 빌드, 둘 다 `tools/lp_check.py --methods 1,0,-1 --timeout 120`·`tools/proto/pathcmp.py all`):
  - 정답 Method 1·0·−1 모두 90/91 → 90/91(pilot FAIL 그대로). IterCount 일치 30/64·0/64·30/64 그대로.
  - pathcmp 완전 일치 24 → 25/64(beaconfd 첫 차이 80 → 완전 일치), 중앙 첫 차이 비율 0.589 → 0.624. 첫 차이 위치가 바뀐 다른 모델은 없다.
  - 반복 수가 바뀐 모델(Method=1, 원본): 25fv47 2461 → 2328(2552), bnl1 978 → 981(1050), boeing1 626 → 632(651), degen2 628 → 621(634), degen3 2343 → 2303(2212), israel 145 → 146(150), maros 1739 → 1738(1975), perold 1341 → 1287(1273), 기준 없음 d2q06c 5317 → 5161, greenbea 3843 → 3805, greenbeb 6134 → 5776, maros-r7 5872 → 7511, nesm 1963 → 1909, pilot 4990 → 4545, pilot87 11806 → 11214. Method=0은 bnl1·boeing2·ganges·greenbea·perold·wood1p·woodw가 바뀌었다(IterCount 일치 0/64 그대로). 모두 첫 차이 뒤의 변화라 일치·불일치 판정 근거가 아니다.
- Gurobi는 실행하지 않았다(저장 기준).

### 8.2 판정

- 구현: 통과(계약 범위). kb2 dual 피벗 0–30의 ρ·β와 스냅숏 0–30(vstat·x_B·d·w·가격 상태)이 원본과 비트 일치. 영향 회귀 통과.
- 남은 한계: 희소 eta 경로와 조밀 eta 뒤 Lᵀ 강제의 판별 사례 없음(kb2 희소 eta 0회, dense 변이 생존). 피벗 31 이후 U 차이(M1-02l). e_r 밖 입력의 Uᵀ 경로(M1-02k). kb2 밖 모델은 첫 차이 위치 변화(beaconfd)만 확인.
