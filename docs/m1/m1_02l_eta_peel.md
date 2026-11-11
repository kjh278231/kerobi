# M1-02l 검증 계약과 결과: kb2 피벗 30 spike 끝자리(행 eta 내적의 정렬 선처리)

작성 기준은 [스프린트 검증 기준](../15_sprint_verification.md)이다. [M1-02i](m1_02i_eta.md) §7의 후속이다. 사용자의 "다음 스프린트 진행" 지시로 착수했다(2026-11-11). M2는 다른 세션이 직전에 M2-06f-5를 점유해 M1을 골랐다([시작 절차](../handoff/README.md) §1-3).

## 1. 범위와 주장

- 작업 종류: 조사(새 읽기 전용 캡처 포함).
- 조사 질문: kb2(Method=1, Presolve=0, Threads=1) 피벗 30 갱신의 새 U 열 16에서 행 17·36이 끝자리로 다르다(피벗 31부터 U가 다름, 02i §6). 처음 다른 연산은 무엇이고 원본 규칙은 무엇인가. 그 규칙이 정하는 입력(행 eta 값 배열 주소의 16바이트 정렬)은 어디서 오며 Rust가 가진 상태로 계산할 수 있는가.
- 포함 분기: dual DSE 피벗의 두-RHS FTRAN `09d112` R 단계(행 eta 내적)와 FT 갱신 `05b257`의 새 대각 η·S(같은 eta 풀 값). kb2 피벗 0–37의 모든 갱신 피벗.
- 제외 범위와 후속:
  - Rust 수정: 정렬 출처가 Rust 상태로 계산 가능하다는 근거가 나오면 같은 branch 둘째 커밋으로 넣고 §8에 범위 확대를 기록한다. 아니면 후속 행으로 둔다.
  - `09d112`의 다른 R 경로(`21f83a`, 희소 인라인)와 primal `2207de`의 R 단계, `gather`·`row_cost_estimate`의 선처리(다른 버퍼): C 구조 확인만, 실행 판별은 kb2에 없으면 미검증.
  - 풀 용량 증가(`05b257` C:800–860 재할당)가 kb2에서 일어나지 않으면 그 경로는 미검증.
- 선행: main `bb77396`. 미커밋 선행 변경 없음.

## 2. 원본과 상태 경계

- Gurobi 13.0.3 `libgurobi130.so` SHA-256 `8e87fca2…0e98f9c`. 모델 `../re/bench/data/netlib/kb2.mps.gz`(`tools/m1_beta_check.py` `MODEL_SHA`).

| 위치 | 내용 |
|---|---|
| `09d112` `0x125f50` C:1136–1355 | 두 RHS 조밀 R 단계(인라인): eta 레코드 LU+0x198 {r, len, idx*, val*}(0x18바이트)마다 두 RHS의 내적. `val & 0xf ≠ 0`이고 `val & 7 = 0`이면 선처리 1개(C:1157–1236). `n < 8` 또는 `n < peel + 8`이면 전부 순차(C:1149–1153, C:1160·LAB_00229699). 결과 `|v| ≤ ZERO`면 0(C:1340–1351) |
| asm `0x1272d7`–`0x12742f` | 위 첫 RHS: `and $0xf, val` → 0이면 peel 0, `test $7` 실패면 순차, 아니면 peel 1. 선처리 합을 레인 0에 넣고(`movsd %xmm5,%xmm9`) 8레인 `mulpd` 정렬 메모리 피연산자(값 배열), 결합 `((0+2)+(4+6)) + ((1+3)+(5+7))`(`addpd` 3번 + `unpckhpd`/`addsd`), 꼬리 순차. 곱은 `x[idx]·val`, 융합 없음 |
| `05b257` `0xf15c0` C:906–918 | 새 eta 레코드: idx* = LU+0x148 + 4·LU+0x160, val* = LU+0x150 + 8·LU+0x160, 그 뒤 LU+0x160 += len. 즉 eta 값은 풀에 연속으로 쌓인다 |
| `05b257` C:800–860 | 남은 용량 LU+0x168 − LU+0x160이 모자라면 풀을 1.5배로 다시 잡는다(`ac645d`) |
| m1_notes §8.10 3, `05b257` C:321–372 | 새 대각 S[r] = Σ S[η.idx]·η.val, 같은 선처리 규칙. 읽는 값 배열은 풀이 아니라 LU+0x1d0(Uᵀe_r 결과) 버퍼다(§6에서 바로잡음) |

- 시작 경계: 피벗 k의 `09d112` 호출 진입(`8b473`에서, 반환 주소 `0x120200`). 종료 경계: 피벗 k+1 `09d112` 진입의 U(새 U 열 = 피벗 k spike, 새 대각).
- 읽는 상태: a_q(캡처 `aq_in`), L(Rust 덤프, 02i §6이 간접 확인), 행 eta(캡처 U에서 `Uᵀ⁻¹e_r`로 재구성, 02i E 검사와 같은 규칙), eta 값 배열 주소 mod 16(새 캡처).
- 정렬 값은 주소에서 나온다. 저장은 풀 기준 주소 mod 16과 풀 오프셋만 한다. 3회 재현의 근거는 "같은 할당 순서면 할당기가 같은 정렬을 준다"는 관측이며, ASLR은 페이지 단위라 mod 16을 바꾸지 않는다.

## 3. 비교 계약

새 `tools/m1_eta_peel_check.py`가 판정한다. 실패 하나면 종료코드 1.

| 검사 | 규칙 |
|---|---|
| M 입력 | 새 캡처 3회 gz SHA-256 고정, 정상 종료, 바이너리·모델·파라미터·도구 해시, 3회 사건 완전 같음. 02i Rust 덤프·02a-1 캡처 고정 해시 재사용 |
| N 비간섭 | 새 캡처에서 `ft.pool`을 뺀 사건 = 02a-1 `kb2_1` 사건(JSON 전체) |
| P 풀 | 모든 피벗에서 eta 수 = LU+0x180, 각 레코드 len = 캡처 U로 재구성한 eta 길이, `va ∈ {0, 8}`, `va = (b150 + 8·vo) mod 16`, 한 분해 구간 안에서 `vo_{i+1} = vo_i + len_i`, `io = vo`. 분해 직후 LU+0x160(구간 첫 eta의 vo)을 구간별로 출력한다(규칙은 결과로 보고, 판정에 쓰지 않음) |
| S spike | 캡처 정렬(`va = 8` → peel 1)과 §2 규칙으로 계산한 spike = 피벗 k+1 캡처 새 U 열의 비대각 원소(비트), eta가 있는 모든 갱신 피벗(10–13, 18–24, 27–36) |
| D 대각 | 행 eta를 만든 갱신 피벗(9–13, 17–24, 26–36)마다 새 대각 = 그 eta·spike(같은 규칙, 캡처 정렬) = 피벗 k+1 캡처 대각(비트) |
| V 변이 | 저장 자료만으로 실행 전에 고정: 항상 peel 0(현 Rust `dot8`)은 S에서 정확히 [30, 32, 35], 항상 peel 1은 D에서 정확히 [33, 35] 거부. 캡처 정렬 반전은 S [30, 32, 35]·D [33, 35] 거부. 항상 순차는 S 생존(kb2 판별 불가), D에서 정확히 [33] 거부 |
| T 변조 | (`--self-test`) 바이너리 해시 0, 빈 사건, 회차 하나의 eta va 반전, vo 이동, 02a-1 필드 하나 변조(N), 캡처 U 새 열 한 비트, eta len 변조. 모두 거부 |

- 값 비교는 IEEE 비트 비교다.
- 판별 근거(실행 전, 저장 자료): kb2에서 정렬이 결과를 가르는 eta는 셋이다. 피벗 27 갱신 eta(길이 8, spike 피벗 30·32·35, peel 1 필요), 피벗 33 갱신 eta(길이 12, 그 대각, peel 0 필요), 피벗 35 갱신 eta(길이 9, 그 대각, peel 0 필요). 나머지는 길이 < 8이거나 값이 같다. 풀 연속 저장이면 이 셋의 오프셋은 구간 첫 오프셋 X에 대해 X+2·X+39·X+53이라 패리티가 저장 자료 요구와 맞는다(캡처 전 가설).

## 4. 사례와 원본 자료

| 사례 | 실제 / 합성 | 노리는 분기 | 입력 | 원본 출력 |
|---|---|---|---|---|
| kb2 Method=1 Presolve=0 Threads=1 | 원본 실제 실행(읽기 전용 Frida 훅) | 분해 구간 4개(피벗 0·6·15·26), eta 길이 2–15, peel 0·1 모두 | `kb2.mps.gz` | `bench/oracle/m1_02l_pool/kb2_{1,2,3}.json.gz`(새 캡처), `bench/oracle/m1_02a1_tau/kb2_1.json.gz`(비간섭) |

- 캡처 도구 `tools/m1_eta_pool.js`: `tools/m1_tau_path.js`와 같은 훅·필드에 `ft.pool`만 더했다.
- 다른 모델: 분해 직후 LU+0x160 규칙의 탐색 근거가 더 필요하면 1회씩 탐색 캡처하고 탐색으로만 기록한다.

## 5. 재실행 명령

- 원본 캡처(작업 worktree 루트, Gurobi 한 프로세스씩): `for r in 1 2 3; do tools/gurun.sh node tools/m1_eta_pool.js ../re/bench/data/netlib/kb2.mps.gz bench/oracle/m1_02l_pool/kb2_$r.json.gz; done`.
- 판정: `python3 tools/m1_eta_peel_check.py --self-test`.
- 새 Rust 실행(§8 수정 뒤): `tools/build_local.sh && PYTHONPATH=$PWD/python ../.venv/bin/python tools/m1_eta_peel_check.py --self-test --fresh`.
- 회귀: `cargo test --release --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, worktree 빌드 `PYTHONPATH=$PWD/python ../.venv/bin/python -m pytest -q`, M1 검사기 7종 `--self-test`, Netlib `tools/lp_check.py --methods 1,0,-1 --timeout 120`·`tools/proto/pathcmp.py all`(기준선 main `bb77396` 별도 worktree 빌드).

## 6. 실행 결과(조사)

- 저장 자료 재생(캡처 전): 피벗 30 spike를 02i 덤프의 L·eta로 다시 계산하면 R 단계를 모두 순차합으로 할 때 새 U 열 8원소가 캡처와 같고, 현 Rust `dot8`(8레인)은 행 17·36이 다르다. 행 17의 차이는 피벗 27 갱신 eta(길이 8)의 내적 하나에서 나오고 행 36은 그 하류다. C·asm(§2)상 길이 8 eta가 순차가 되는 경로는 "값 배열이 8바이트 정렬(16 아님) → 1개 선처리 → 8 < 1 + 8"뿐이다.
- 캡처 `tools/m1_eta_pool.js` kb2 3회: gz 바이트 동일(SHA-256 `cda2949c…3863a0`), `DONE 2 38`, `ft.pool`을 빼면 02a-1 `kb2_1` 사건과 JSON 전체가 같다(비간섭).
- `python3 tools/m1_eta_peel_check.py --self-test` 종료 0:

```
P: 117 eta records: r, len = U^T^-1 e_r of the capture, idx/val offset = LU+0x160 at the update, val mod 16 = 8*offset mod 16; LU+0x150 mod 16 [0], LU+0x1d0 values mod 16 [0]
G: LU+0x160 growth = model on 34/34 updates (row cap LU+0x188-0x18c bound 0 times); X: LU+0x160 after the factors at [0, 6, 15, 26] = 2*(nnz L + nnz U) [10, 112, 196, 338]
S/D (orig): spike = capture on 21/21 updates with etas, new diagonal on 24/24 eta updates; fail [] / []
T 변조 9종 거부, 규칙 변이 peel0 S [30, 32, 35]·peel1 D [33, 35]·invert S [30, 32, 35] D [33, 35]·seq D [33] (모두 §3 V의 사전 기대 그대로)
```

- **처음 다른 연산과 원본 규칙**: 피벗 30 FTRAN R 단계에서 피벗 27 갱신 eta(행 17, 길이 8)의 내적이다. 그 eta 값은 LU 풀 오프셋 409(홀수)에 있어 주소 ≡ 8 mod 16이고, 원본은 1개를 선처리하려다 `8 < 1 + 8`이라 전부 순차로 더한다. Rust `dot8`은 8레인으로 더했다.
- **정렬의 출처(캡처)**: 풀 LU+0x148(idx)/0x150(값)은 U 행·U 열·행 eta가 함께 쓰는 저장소다. 풀 기준 주소는 16 정렬(38/38 피벗)이라 값 정렬 = 8·오프셋 mod 16이다. 새 eta의 오프셋 = 그 갱신 시작의 LU+0x160(117/117 레코드). LU+0x160은 분해 직후 2·(nnz L + nnz U)(4/4 분해, L은 Rust 덤프)이고, 갱신마다 eta 길이 + 자리보다 긴 새 U 열(끝에 붙임, `05b257` C:1224–1227) + 새 원소를 받는 꽉 찬 U 행(끝으로 옮기며 `len + min(max(4, len/2), LU+0x188 − LU+0x18c)` 자리, C:1264–1282)만큼 는다(34/34 갱신, 분해 직후 행·열 자리 = 길이). LU+0x188 = 20(LUINTERVAL). 상한이 작동한 갱신은 kb2에 없다.
- **새 대각은 다른 버퍼**: `05b257` C:321–372의 η·S는 풀이 아니라 LU+0x1d0(Uᵀe_r 결과) 값 배열을 읽고, 그 배열은 16 정렬(38/38)이라 kb2에서는 항상 peel 0이다. 저장 자료가 요구한 대각 peel(33·35 갱신 peel 0)과 맞는다.
- 계약 변경(결과 확인 뒤, 이유 기록): P의 "한 구간 안에서 `vo_{i+1} = vo_i + len_i`(eta 연속 저장)" 가설은 틀렸다(풀을 U와 함께 씀). "eta 오프셋 = 갱신 시작 LU+0x160"과 G(증가 모형)·X(분해 직후 값) 검사로 바꿨다. D는 "캡처 eta 정렬"이 아니라 캡처한 LU+0x1d0 정렬을 쓴다(그 필드를 넣어 3회 다시 캡처했고, 첫 캡처 3회는 지웠다). §2 표의 m1_notes 인용("η.val 주소")도 LU+0x1d0 값 주소로 바로잡는다.
- 다른 R 경로: `21f83a` C:341–348, primal `2207de` C:363–369에도 같은 `& 0xf`/`& 7` 선처리 구조가 있다(C만, 실행 판별 없음).
- 탐색(고정하지 않음): maros(Method=1) 1회 경량 캡처(풀 필드만, 임시 도구, 저장하지 않음)와 수정 Rust의 피벗별 LU+0x160을 비교했다. 피벗 0–44는 같고, 474 전 갱신 23회가 모형과 다르다. 앞 7회(44·122·133·210·266·271·340)는 eta 길이가 같은데 원본 증가가 더 커서 모형에 없는 증가 경로가 있다(미확인). 388 이후는 eta 길이부터 달라 Rust U 차이의 하류다. 474 구간 원본 eta 중 길이 ≥ 8인 7개의 패리티가 Rust와 다르다.

## 7. 판정과 인수인계

- 작업 수행: 완료. 동등성(조사): kb2 피벗 0–37에서 원인 규칙이 캡처 spike 21/21·대각 24/24를 비트 재현했고, 정렬 출처(풀 오프셋 패리티)와 그 오프셋의 계산 규칙을 캡처 117 레코드·34 갱신·4 분해로 확인했다(저장 기준, 3회 재현).
- 남은 한계: 풀 증가 모형은 kb2 규칙뿐이다(maros 탐색에서 증가 경로 누락 확인, 후속 **M1-02m**). 풀 재할당(LU+0x168 초과)·자리 상한 작동·LU+0x1d0 비정렬 경우는 kb2에서 실행 0회. `21f83a`·`2207de`는 C만.
- 후속:
  - **M1-02m**(조사, 새 행): maros 피벗 44 갱신의 LU+0x160 증가(원본 3, 모형 2)부터 빠진 증가 경로를 찾는다. 첫 명령: `tools/m1_eta_pool.js`에 피벗 범위 제한을 더해 maros 피벗 40–50의 U 행·열 자리(LU+0xb0/0xb8/0xc0, 열 +0x88/0x90)를 캡처하고 `05b257` C:1224–1290 분기와 대조.
  - maros 첫 차이 475 → 474(§8.1) 원인 확인은 M1-02m 뒤 다시 본다.
- 교훈: [original.md](../lessons/original.md) "LP 행 eta 내적 정렬(LU 풀)", [tooling.md](../lessons/tooling.md) Python `sum()` 보정 합 함정.
- branch `work/m1-02l`: 계약 `bfa3953`, 캡처·검사기 `d6518c0`, Rust 수정 `d9ed495`, 기록 커밋. 미커밋 없음.

## 8. Rust 수정(범위 확대)

- 이유: §6이 규칙과 그 입력(풀 오프셋 패리티)을 C·캡처로 확정했고, 오프셋은 Rust가 가진 상태(nnz L·U, U 행·열 길이, eta 길이, 갱신 수, LUINTERVAL)로 계산된다(계약 §1). 수정은 M1 파일(`crates/lp/src/lu.rs`, `simplex.rs` 호출 인자 하나)에 머물고 같은 오라클로 판정된다. 조사 커밋 `d6518c0`은 수정 없이도 병합 가능한 상태다.
- 변경: `Lu`에 `pool`(LU+0x160)·`rcap`·`ccap`(U 행·열 자리)·`reta_odd`(eta 값 오프셋 홀수). `finish`가 2·(nnz L + nnz U)와 길이로 초기화하고, `update(…, interval)`가 eta를 만들 때 오프셋 패리티를 기록한 뒤 §6 증가 규칙을 적용한다. `ftran_r`은 `dot8_peel(…, odd)`(홀수면 1개 선처리, 9개 미만이면 순차)을 쓴다. 대각 `dot8`은 그대로(LU+0x1d0 16 정렬 가정을 `shortcut:`으로 적음). 풀 재할당 미모형도 `shortcut:`. 새 단위 테스트 `dot8_peel_order`.
- 공유 코드라 모든 FTRAN(dual α_q·τ, primal, 초기 x_B 등)의 R 단계에 적용된다.

### 8.1 실행 결과(worktree `../rsolver-m1-02l`, `tools/build_local.sh` 빌드)

- `tools/m1_eta_peel_check.py --self-test --fresh` 종료 0: §6 줄에 더해 `F: fresh Rust beta = capture on 38/38 pivots; differ []`(전: 32/38, 31·33–37 다름). `tools/m1_eta_check.py --fresh`도 `differing after: []`, statecmp kb2 31/31.
- 코드 변이(임시 적용 뒤 되돌림, 각각 재빌드): 패리티 무시(항상 peel 0)·패리티 반전·행 여유 최소 3·새 열 자리 무시 4종 모두 F 거부(β 31·33–37 차이).
- `cargo test -p rsolver-lp` 42(새 테스트 1), `cargo test --release --workspace` 125 통과·10 무시, clippy 0, worktree 빌드 pytest 657 passed. M1 검사기 7종(`m1_beta_check`·`m1_tau_check`·`m1_urow_check`·`m1_list_order_check`·`m1_aq_order_check`·`m1_bfrt_rust`·`m1_eta_check`) `--self-test` PASS.
- Netlib(기준선 = 같은 날 main `bb77396`을 `../rsolver-m1-02l-base`에 따로 빌드):
  - 정답 Method 1·0·−1 모두 90/91 그대로(pilot FAIL). IterCount 일치 Method 1 30 → 32/64, −1 30 → 32/64(israel 146 → 150, share1b 172 → 175 = 원본), 0은 0/64 그대로. 잃은 모델 없음.
  - pathcmp 완전 일치 25 → 26/64(share1b), 중앙 첫 차이 비율 0.624 그대로. 첫 차이가 늦춰진 모델 israel 83 → 84, **앞당겨진 모델 maros 475 → 474**(원인 미확인, §6 탐색의 풀 모형 누락이 후보).
  - 반복 수가 바뀐 모델은 첫 차이 뒤의 변화라 판정 근거가 아니다(Method=1 예: 25fv47 2328 → 2341, degen3 2303 → 2146, maros 1738 → 1871, stair 393 → 355).
- Gurobi: kb2 캡처 6회(첫 캡처 3회는 필드 추가로 다시 함), maros 탐색 1회. 한 프로세스씩 `tools/gurun.sh`.

### 8.2 판정

- 구현: 통과(계약 범위 kb2). kb2 dual 피벗 0–37의 β가 원본과 비트 일치, 스냅숏 0–30 일치. 영향 회귀 통과, Netlib 정답 불변·IterCount +2·경로 완전 일치 +1.
- 남은 한계: 풀 증가 모형 불완전(maros, M1-02m), maros 첫 차이 1피벗 앞당겨짐, 풀 재할당·LU+0x1d0 비정렬 미모형, 다른 R 경로는 C만.
