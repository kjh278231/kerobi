# M1-02h 검증 계약과 결과: Rust α_q 목록 순서(`09d112` 경로·출력 순서)와 mode 1 갱신 순서

작성 기준은 [스프린트 검증 기준](../15_sprint_verification.md)이다. [M1-02g](m1_02g_list_order.md) §7의 후속 구현이고, 사용자의 "다음 스프린트 진행" 지시로 착수했다(2026-11-11). M2는 다른 세션이 M2-06f-2를 점유해 작업 중이라([시작 절차](../handoff/README.md) §1-3) M1을 골랐다.

## 1. 범위와 주장

- 작업 종류: 구현.
- 완료할 동작: Rust dual 반복이 피벗마다 원본 `8b473`의 최종 α_q 목록(spx+0x210) 순서를 만들고, mode 1 가격 목록 갱신(`price_touch`)이 그 순서를 쓴다. 순서는 다음 셋으로 정한다.
  1. `09d112` U 단계 경로: 희소 U(`21686f`·`21841f`)인지 조밀(`2126cd`·인라인)인지. L 단계(`21b9de` 조건)·R 단계(행 eta, `21f83a`·인라인)·U 단계 조건을 C 그대로 옮긴다(§2).
  2. 희소 U 출력 순서(02g P3): [LU+0xf8 = −1 접두 행, U 입력 순서] + [꼬리 행 U 위치 내림차순] + [LU+0xf8 ≥ 0 접두 행 LU+0xf8 오름차순]. 조밀이면 위치 오름차순.
  3. `8b473` 피벗 제거: 피벗 위치 r의 자리를 마지막 원소로 채운다(거르기 아님).
- 이를 위해 Rust LU에 원본 접두 행 목록 LU+0xf0/+0xf8/+0xec와 LU+0xdc·0x108·0x110을 더한다. 목록 유지 규칙은 `05b257`(§2)이다. 값 계산(조밀 FTRAN)은 바꾸지 않는다. 02g에서 피벗 21의 α_q 값·집합은 이미 같고 순서만 달랐기 때문이다.
- 포함 분기와 입력 전제: dual simplex DSE(mode 1·3에서 `09d112`), kb2 Method=1 Presolve=0 Threads=1의 피벗 0–37.
- 제외 범위(착수 때 고정, 후속 번호):
  - **희소 L·R 출력 순서**(`21b9de` Kahn FIFO, `21f83a`): LU+0xf8 = −1 행의 순서와 희소 spike의 순서를 정한다. Rust는 근사(a_q 열 저장 순서의 입력 원소 + 나머지 위치 오름차순)를 쓰고 `shortcut:` 주석을 단다. 검사기는 이 근사가 쓰이는 피벗(−1 행이 출력에 둘 이상 섞이거나 희소 spike가 원소 둘 이상인 피벗)을 "범위 밖"으로 따로 세어 출력한다. kb2에서는 피벗 0–5·21·24·30·32·34·36·37이 해당할 수 있다. 후속 **M1-02j**(조사).
  - 희소 경로의 값 연산 순서(희소 커널의 비트): 조밀 결과를 그대로 쓴다. 근거는 kb2 피벗 0–21 상태 비트 일치뿐이다.
  - BTRAN의 `(int)(LU+0x108 + LU+0x110)·nnz ≤ m`(Rust는 3·nnz ≤ m 고정): 이번에 LU+0x108·0x110을 계산하지만 BTRAN 경로 선택은 바꾸지 않는다(궤적 영향, 후속 M1-02k 제안).
  - kb2 반복 23 이후 d·w 끝자리 차이: M1-02i.
  - primal simplex의 FTRAN 순서, flip 패스(`1c8477`) touch 순서.
- 선행: main `577a443`(M1-02g 병합). 미커밋 선행 변경 없음.

## 2. 원본과 상태 경계

- Gurobi 13.0.3, `libgurobi130.so` SHA-256 `8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c`. 주소는 file offset이다. 모델 `../re/bench/data/netlib/kb2.mps.gz`.

| 위치 | 내용 |
|---|---|
| `09d112` `0x125f50` 인자 | `(LU, α_q 출력, a_q, τ 출력, ρ, &n, r, work)`. m = LU+8, iVar26 = m − LU+0x100(꼬리 수), iVar52 = LU+0xec, iVar41 = LU+0x104, D = LU+0x108 + LU+0x110 |
| `09d112` C:150–700 L 단계 | RHS마다 nnz ≥ 0이고 `0x104·nnz ≤ m`이고 `0.1·m > LU+0x128`이면 희소 L `21b9de`, 아니면 조밀(`214344`·인라인, 출력 nnz = −1) |
| `09d112` C:700–1372 R 단계 | LU+0x180 = 0이면 생략. a_q 쪽이 조밀(nnz < 0 또는 `m < 0x104·nnz`)이면: ρ 쪽도 조밀이면 인라인(둘 다 nnz = −1, C:1356–1358), 아니면 `21f83a`(ρ 쪽 희소). a_q 쪽 희소이고 ρ 쪽 조밀이면 `21f83a`(a_q 쪽 희소 유지), 둘 다 희소면 인라인 희소(C:1374–) |
| `09d112` C:1744–1786 spike | a_q 쪽 nnz < 0이면 인덱스 오름차순 `|x| > ZERO`, 아니면 그 목록 순서(memcpy, C:1845–1850) |
| `09d112` C:1792–1884 U 단계 | ρ 쪽 nnz ≥ 0이고 (`nnzρ + iVar26 + ec < m` 또는 `D²·nnzρ < 2m`)이면: a_q 쪽이 nnz < 0이거나 (`m ≤ iVar26 + ec + nnz` 그리고 `2m ≤ D²·nnz`)이면 `2126cd`(출력 오름차순, 피벗 마지막), 아니면 `21686f`. 그 밖에 a_q 쪽 nnz ≥ 0이고 (`iVar26 + ec + nnz < m` 또는 `D²·nnz < 2m`)이면 `21841f`, 아니면 인라인 조밀(C:1888–2430, 오름차순) |
| `036ade` C:5725–5740 | LU+0x108 = (U 원소 수 − 행 시작[t])/(m − t) + 1(t < m, 아니면 1.5), LU+0x110 = (행 시작[t] − 행 시작[0xdc])/(t − 0xdc) + 1(0xdc < t, 아니면 1.5). `param_12` = LU+0x20 |
| `1da0b2` C:1514–1532 | LU+0xd8·0xdc·0x100(= `param_21`+0xb8·0xbc·0xe0, `param_21` = LU+0x20): singleton 단계 경계. Rust `factor_tol`의 행 singleton 끝 s1, slack 끝 s2, 열 singleton 끝 s3에 대응(kb2 인수분해 9회에서 0xdc = s2, t = s3, 임시 출력으로 확인) |
| `05b257` C:2595–2690 재분해 뒤 | LU+0xf8 = −1(위치 < 0xdc), 0,1,…(0xdc ≤ 위치 < t, 목록 +0xf0 = 그 위치 오름차순), −2(꼬리). ec = t − 0xdc |
| `05b257` C:445–449 | spike 목록에서 나가는 행 r의 자리를 마지막 원소로 채운다 |
| `05b257` C:535–551 | 옛 열 제거 중 U 행이 비는 행(원소 1개)이고 위치 ≤ local_40이면 목록에서 마지막 원소로 채워 빼고 −1 |
| `05b257` C:1040–1059 | local_148 = 1(p 위의 빈 열이 p 자리를 차지)이면 ord[p] 행이 접두로 들어온다: U 행이 비지 않았으면 목록 끝에 붙이고, 아니면 −1 |
| `05b257` C:1085–1097 | `l ≠ p` 또는 `p ≠ t−1` 또는 spike 다른 원소가 있으면 r을 목록에서 빼고(있으면) −2 |
| `05b257` C:1240–1262 | spike(r 뺀) 순서로 새 열 원소를 넣으며 새 위치 ≤ local_40이고 +0xf8 < 0인 행을 목록 끝에 붙인다 |
| `8b473` C:466–482 | α_q 목록에서 피벗을 마지막 원소로 채워 뺀다 |
| `1bae9c` C:466–700 | r 처리 뒤 α_q 목록 순서로 mode 1 갱신(02g) |

- 시작 경계: Rust 스냅숏 k(반복 k의 SIMPLEX 콜백 = 피벗 k−1 갱신 뒤). 원본 대응은 02a-1 캡처 피벗 k의 `09d112` 진입(`ft`)과 `8b473` 반환(`aq`).
- 종료 경계: 피벗 k의 최종 α_q 목록과 다음 스냅숏의 가격 목록.
- 읽는 상태: Rust LU(위치·U 행·접두 t·행 eta 수·0x104), 새 접두 목록, a_q·ρ 입력 nnz, L·R 뒤 nnz(조밀 결과의 0 아닌 원소 수, §3 K로 간접 확인).

## 3. 비교 계약

새 `tools/m1_aq_order_check.py`가 판정한다. 하나라도 실패하면 종료코드 1이고, 첫 차이의 피벗·필드·기준값·실제값을 출력한다. Rust 쪽은 이 worktree 빌드의 `RSOLVER_SNAP`(새 필드 `f0` 접두 목록(위치), `aqo` 최종 α_q 순서, `uk` 예측 U 경로)이다.

| 검사 | 규칙 |
|---|---|
| M 입력 | 02a-1 캡처 3회 gz SHA-256과 저장 오라클 SHA-256 고정, 캡처 3회의 `f8`·`aq` 같음 |
| N 경로 일치 범위 | Rust 스냅숏 0..k의 vstat가 저장 오라클과 같고(반복 0–30), 31 이후는 02f pathcmp가 같은 피벗이라 LU 구조 비교만 한다. 비교한 피벗 수를 출력하고 38 미만이면 실패 |
| F 접두 목록 | 피벗 k마다 Rust 스냅숏 k의 목록(위치, 순서 포함) = 캡처 피벗 k `ft.f8`에서 만든 목록(+0xf8 오름차순), ec 같음. 피벗 0–37 |
| K 경로 | 피벗마다 Rust 예측 U 경로(`21686f`·`2126cd`·`21841f`·인라인) = 캡처 `ft.calls`의 U 커널(없으면 인라인). 피벗 0–37. 판별 기록: 희소 L이면서 조밀 U인 피벗, `21f83a`이면서 `21841f` 없는 피벗, `2126cd` 피벗을 따로 센다 |
| A α_q 순서 | 피벗 k마다 Rust 최종 α_q 목록(순서 포함) = 캡처 `aq.idx`. 단 §1 제외 범위의 근사가 순서를 정하는 피벗은 "범위 밖"으로 세고 일치·불일치를 그대로 출력한다(합격 판정에서 뺌, 개수 출력) |
| S statecmp | `tools/proto/statecmp.py kb2` 반복 0–22 OK(반복 23 이후 차이는 M1-02i, 첫 차이 반복을 출력) |
| G 02g | `tools/m1_list_order_check.py --self-test` 종료 0 |
| T 변조·변이 | (`--self-test`) 캡처 f8 두 원소 교환(F 거부), 캡처 calls에서 `21841f` 제거(K 거부), 캡처 aq 두 원소 교환(A 거부). Rust 코드 변이(임시 빌드): spike r 거르기(마지막 채우기 대신), 피벗 제거를 거르기로, D = 3 고정, 꼬리 오름차순 — 각각 어느 검사에서 거부·생존하는지 기록 |

- 비교는 정수 위치·순서 정확 비교다. 값은 비교하지 않는다(값 비트는 S가 본다).

## 4. 사례와 원본 자료

| 사례 | 실제 / 합성 | 노리는 분기 | 입력 | 원본 출력 |
|---|---|---|---|---|
| kb2 Method=1 Presolve=0 Threads=1 | 원본 실제 실행 저장 자료 | 희소 U 피벗 0–4·21·24, `2126cd` 5, 희소 L·조밀 U 30·32·36·37, `21f83a`·조밀 U 34, 인라인 나머지. 재분해 0·6·15·26(목록 초기화) | `kb2.mps.gz` | `bench/oracle/m1_02a1_tau/kb2_{1,2,3}.json.gz`, `bench/oracle/iter0-30/kb2.jsonl.gz` |

- 적용하지 않은 범주: LU+0x128 ≥ 0.1m(레벨 경로), LU+0xdc < t인데 U 행이 빈 접두 행(kb2 없음), local_148 = 1 경로(kb2 발생 여부는 실행 결과에 기록), 다른 모델(원본 캡처 없음, Netlib 회귀로만).

## 5. 재실행 명령

- 빌드: `tools/build_local.sh`.
- 판정: `PYTHONPATH=$PWD/python ../.venv/bin/python tools/m1_aq_order_check.py --self-test`.
- statecmp: `zcat bench/oracle/iter0-30/kb2.jsonl.gz > <tmp>/kb2.jsonl; PYTHONPATH=$PWD/python ../.venv/bin/python tools/proto/statecmp.py kb2 <tmp>/kb2.jsonl`.
- 02g·기존 검사기: `python3 tools/m1_list_order_check.py --self-test`, `PYTHONPATH=$PWD/python ../.venv/bin/python tools/m1_tau_check.py --self-test`, `tools/m1_beta_check.py --self-test`.
- 회귀: `cargo test -p rsolver-lp`, `cargo test --release --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, 이 worktree 빌드 pytest, `tools/lp_check.py --methods 1 --timeout 120`(수정 전후 정답·IterCount 일치 수), `tools/proto/pathcmp.py all`(수정 전후 완전 일치 수).

## 6. 실행 결과

(실행 후 작성)

## 7. 판정과 인수인계

(실행 후 작성)
