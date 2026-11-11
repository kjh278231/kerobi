# M1-02x BTRAN 희소 Lᵀ의 LU+0x140 조건 검증 계약과 결과

작성 기준은 [스프린트 검증 기준](../15_sprint_verification.md)이다.

## 1. 범위와 주장

- 종류: 구현. 기준 main `43466cc`, 미커밋 선행 변경 없음(main의 미추적 `.env`는 사용·변경하지 않음).
- 동작: BTRAN `221173`이 Lᵀ를 희소로 푸는 세 조건 중 Rust에 없는 세 번째 조건 `(double)LU+0x140 < 0.1·(double)m`(C:265–268)을 `Lu::btran_ut`(대각 U)과 `Lu::btran_updated`(갱신·일반 U)의 희소 Lᵀ 판정에 더한다. 거짓이면 원본은 앞 두 조건(`nnz ≥ 0`, `LU+0x104·nnz ≤ m`)이 거짓일 때와 같은 조밀 Lᵀ(asm 0x12fee6, C:1192–1576)로 간다. 이때 Lᵀ 입력 nnz가 0이면 출력 nnz도 0이다(C:1580–1590).
- 노리는 차이: M1-02k에서 고정 목록으로 남긴 stair e_r 102·118(원본 조밀 출력, Rust 희소 출력). Rust Lᵀ 깊이 43 ≥ 0.1m = 35.6이 원인 후보였다.
- 제외: LU+0x108 재분해 차이(M1-02w, stair 185·forplan 82·bnl1 174에서 같은 상태 범위가 끝남), LU 풀이 work LU+0x200(M1-02t), 희소 Lᵀ 레벨 경로·힙 경로 자체의 순서(기존 구현, 이번에 바꾸지 않음), Method 0·quad·Threads > 1·MIP 호출자.

## 2. 원본과 상태 경계

- Gurobi 13.0.3, SHA-256 `8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c`.
- 원본: `../re/decomp/c/0012de60_PRIVATE0000000000221173.c` C:263–268(세 조건), C:1185–1194(희소 경로 끝·조밀 경로로 빠짐), C:1576–1590(조밀 Lᵀ, nnz < 1 처리). asm 0x12ec5a `test r11d`(nnz ≥ 0) → 0x12ec63–0x12ec6d `LU+0x104·nnz ≤ m`(ebp = m) → **0x12ec77 `mov 0x140(%rbx),%esi`** → 0x12ec8d `comisd` `jae 0x12fee6`. 세 조건 실패가 모두 0x12fee6으로 간다. 0x12ec73–0x12ec96 안으로 들어오는 분기는 없다(objdump 전체 grep).
- LU+0x140은 `036ade` C:5850–5912에서 재분해마다 정한다(`m < 250`이거나 `m ≤ LU+0x104`면 −1, 아니면 Lᵀ DAG 깊이). Rust `Lu::lt_depth`(`level` 최댓값, 없으면 −1).
- 자연 경계: `221173` 진입·반환. 진입 때 LU+0x140, 0x12ec77 도달 시 (r11d = Lᵀ 입력 nnz, ebp = m, LU+0x140), 반환 때 출력 nnz와 dual e_r 호출(`9e376` 반환 0xc51dd)의 출력 idx·값 비트.
- 설정: Method=1, Presolve=0, Threads=1. 모델은 levels가 생기는 m ≥ 250 Netlib: stair(356), degen2(444), fit1p(627), bnl1(643). 같은 행 수 조건의 대조군(0 ≤ LU+0x140 < 0.1m 이고 0x12ec77 도달, nnz > 0)이 이 넷에 없으면 m ≥ 250 모델을 더한다.

## 3. 비교 계약

| 필드·사건 | 규칙 | 방법 |
|---|---|---|
| 원본 3회 | 사건 전체 JSON 동일 | 독립 프로세스 3회 |
| 비간섭 | 새 캡처 사건을 02k 필드(11개)로 투영하면 `bench/oracle/m1_02k/<모델>_plain` 사건과 같음 | 투영 비교 |
| 원본 판정 | 0x12ec77 도달·nnz > 0: 출력 nnz ≥ 0 ⇔ LU+0x140 < 0.1m. 도달·nnz = 0: 출력 nnz 0. 미도달: 출력 nnz < 0 | 캡처 |
| 대상·대조 사례 | 도달·nnz > 0·LU+0x140 ≥ 0.1m 1회 이상(대상), 도달·nnz > 0·0 ≤ LU+0x140 < 0.1m 1회 이상(대조) | 캡처 집계 |
| Rust LU+0x140 | 같은 상태 범위(02k 규칙: r·eta 수 일치, KNOWN_D에서 끝)의 e_r 호출마다 Rust `lt` = 원본 진입 LU+0x140 | 탐침 `RSOLVER_BTRAN_PROBE` |
| Rust e_r 출력 | 같은 범위에서 출력 형태(조밀 여부)·idx 순서·값 비트 일치. stair 102·118 포함 | 탐침에 출력 idx·값 추가 |
| 02k 검사기 | `KNOWN_FORM`을 비워도 통과(다른 판정·출력은 그대로) | `tools/m1_btran_check.py --self-test` |

- 수치 오차 없음(비트 일치). 시간·work 비교 없음.
- 원본 반복 실행: 같은 설정 독립 3회.
- Rust e_r 출력 값 비교가 이번 변경과 무관한 이유로 같은 범위 안에서 먼저 어긋나면 첫 차이를 기록하고 범위를 넓히지 않는다(실패로 보고).

## 4. 사례와 원본 자료

| 사례 | 실제/합성 | 노리는 분기 | 입력 |
|---|---|---|---|
| stair e_r 102·118 | 실제 | LU+0x140 ≥ 0.1m → 조밀 Lᵀ(대상) | Netlib stair |
| degen2·fit1p·bnl1의 도달 호출 | 실제 | 대조(레벨 경로)·대상 여부는 캡처로 확인 | Netlib |
| m < 250 모델 | 실제 | LU+0x140 = −1, 변화 없음(회귀로 확인) | Netlib 64 |

- 자료: `bench/oracle/m1_02x/<모델>_{1,2,3}.json.gz`, 도구 `tools/m1_lt_path.js`·`tools/m1_lt_runs.sh`, 검사기 `tools/m1_lt_check.py`, manifest `bench/oracle/m1_02x/manifest.json`.
- 합성 사례 없음(자연 실행에서 대상 사건이 이미 있음).

## 5. 재실행 명령

- 캡처: `tools/hrun.sh --timeout 580000 -- 'tools/m1_lt_runs.sh bench/oracle/m1_02x'`
- 비교: `tools/build_local.sh && PYTHONPATH=$PWD/python ../.venv/bin/python tools/m1_lt_check.py --self-test`
- 02k: `PYTHONPATH=$PWD/python ../.venv/bin/python tools/m1_btran_check.py --self-test`
- 회귀: `bench/oracle/m1_02x/regress.sh <base|final>`(cargo lp·workspace·clippy·pytest·M1 검사기·pathcmp·lp_check).
- 회귀 판정 규칙(착수 때 고정): m < 250 모델의 `pathcmp` 출력 줄이 base와 같고, 바뀐 모델은 목록으로 남기며 그 모델의 첫 차이가 앞당겨지지 않는다. `lp_check` 정답 수가 줄지 않는다. 바뀐 행은 목록으로 남긴다(경로 오라클 없는 대형 모델은 미검증 궤적 변화).
- 변이: Rust 조건 제거, `<` → `<=`, 0.1 → 0.2, nnz = 0 처리 제거. 각각 검사기나 테스트가 거부하는지 기록한다.

## 6. 실행 결과

(작성 예정)

## 7. 판정과 인수인계

(작성 예정)
