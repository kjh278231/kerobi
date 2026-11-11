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

- commit: 구현 `da84da0`(기준 main `43466cc`). 원본 캡처 `03c339f`. 자료 해시는 `bench/oracle/m1_02x/manifest.json`.
- 원본(Gurobi 실행 12회 + 탐색 1회): 4모델 × 독립 3회 사건 전체 동일. 새 캡처를 02k 필드로 투영하면 02k plain 사건과 같다(비간섭). 판정 규칙은 모든 사건에서 성립(도달 3,676, 미도달은 모두 조밀 출력). 대상(도달·nnz > 0·LU+0x140 ≥ 0.1m)은 **stair 6회뿐**(그중 e_r 102·118, LU+0x140 43, m 356), 대조(0 ≤ LU+0x140 < 0.1m) stair 423·degen2 735·fit1p 1,195·bnl1 1,317. 도달·nnz = 0인 사건은 없었다.
- Rust(`tools/m1_lt_check.py --self-test`, `bench/oracle/m1_02x/check.txt`): 같은 상태 범위 stair 185/356·degen2 356/634·fit1p 117/568·bnl1 174/1050 e_r 호출에서 **LU+0x140 832/832 일치, 출력 형태 832/832 일치**(stair 102·118 포함, 둘 다 조밀·값 비트 일치). 변조 16종 모두 의도한 규칙으로 거부(메시지 확인, `check.txt`).
- 계약 변경(결과를 본 뒤): 출력 값·순서 비교에서 이번 판정과 무관한 기존 차이가 나왔다. 전체 차이 목록(`bench/oracle/m1_02x/diffs.py`)을 최종 빌드(`diffs_final.txt`)와 판정 조건만 뺀 빌드(`diffs_nolt.txt`)에서 뽑으면, 다른 점은 stair 102·118 형태 2건뿐이다.
  - stair 값 104호출(e_r 31부터 KNOWN_VAL, idx 순서는 같음): 첫 차이 e_r 31 출력 5번째 값 원본 `3fa7974ca7ca0daf`·Rust `…0dae`(1 ulp, 행 eta 16개, 갱신 U). 뒤로 갈수록 커져 |v| ≥ 1e-12 값의 최대 상대 차이가 1.1e-9(e_r 172)다. |v| < 1e-12인 165값은 0·부호까지 다르다(조밀 출력이 남기는 소거 잔차). 104호출 중 호출 최대 ulp ≤ 4는 24, ≤ 1000은 18, 나머지 62는 그보다 크다.
  - bnl1 출력 순서 7호출(130·134·144·153·159·162·173, KNOWN_IDX): 레벨 경로(LU+0x140 = 6), 같은 행 집합에서 행 587의 출력 위치만 다르다. 그중 4호출에서 행 587 값이 상대 9e-16 이하로 다르다.
  - 규칙: 고정 목록의 호출은 차이가 실제로 있어야 하고(사라지면 실패), 값 고정 호출은 idx 순서가 같아야 하며, 목록 밖 차이는 실패다. 형태·LU+0x140은 모든 호출에서 비교한다. 같은 상태 범위는 넓히지 않았다 → **M1-02y**.
- 02k 검사기: `KNOWN_FORM`을 비우고 `m1_btran_check.py --self-test` 통과(`final/btran.txt`).
- 변이(`mutate.txt`) 6종 중 3 거부(조건 제거, 0.1 → 0.2, 갱신 U 경로에서 조건 제거). 생존 3종:
  - `<` → `<=`: LU+0x140 = 0.1m인 사례 없음(stair 35.6은 정수가 아님).
  - nnz 0 처리 제거: 도달·nnz = 0 사건 없음.
  - 대각 U 경로(`btran_ut`)에서 조건 제거: 대상 6회가 모두 갱신 U 경로다. 두 경로 모두 같은 `lt_sparse_ok`를 부른다.
- 회귀(`bench/oracle/m1_02x/{base,final}`, 같은 worktree에서 변경 전·후): cargo `-p rsolver-lp` 통과, `--release --workspace` 127 통과·12 무시(전후 같음), clippy 0, pytest 657, M1 검사기 13종 통과. `pathcmp.py all` 출력 바이트 동일(27/64·중앙 0.752, stair 첫 차이 211 그대로). `lp_check` 274행 중 시간 열 밖 차이는 pilot87 Method 1·−1 두 행(반복 12609 → 11904, 둘 다 정답, 경로 오라클 없음). 정답 91/91·IterCount 31/64 그대로. 착수 때 정한 회귀 규칙을 만족한다.

## 7. 판정과 인수인계

- 작업 수행: 완료. 동등성: 판정 부분 **통과**, e_r 출력 값·순서 행 **실패(기존 차이 111호출, 고정 목록)**. LU+0x140 값과 희소/조밀 Lᵀ 판정(출력 형태)이 4모델의 같은 상태 범위 832호출에서 원본과 같다. 대상 stair e_r 102·118은 출력 형태와 값이 비트까지 같다. 계약 §3의 e_r 출력 값 비교는 stair 104·bnl1 7호출에서 원본과 다르며, 판정 조건을 빼도 똑같으므로 이번 변경의 결과가 아니다(M1-02y).
- 남은 한계: 대각 U 경로의 조건, `<=` 경계, nnz 0 처리는 자연 사례가 없어 판별하지 못했다(변이 생존). Method 0·quad·MIP 호출자는 미검증. 같은 상태 범위 밖(stair 185 이후 등)도 미검증.
- 후속: **M1-02y**(stair e_r 31의 1 ulp에서 시작해 커지는 값 차이, bnl1 레벨 경로 행 587 순서. 첫 명령 `PYTHONPATH=$PWD/python ../.venv/bin/python tools/m1_lt_check.py --rust`로 고정 목록 확인, stair e_r 31의 행 eta 단계 입력부터 대조), **M1-02w**, **M1-02t**, **M1-02u**.
- branch `work/m1-02x`, worktree `../rsolver-m1-02x`. shared를 얻은 뒤 main `dcadef1`(M2-06h-4, `crates/mip`만 변경)을 branch에 병합해 재검증하고(`bench/oracle/m1_02x/after_main_merge/`: workspace 테스트·clippy·pytest·`m1_btran_check`·`m1_lt_check` self-test) 진행·로드맵·스프린트 표를 갱신한 뒤 main에 `--no-ff`로 병합했다. worktree·branch는 병합과 함께 정리한다.
- 교훈: `docs/lessons/original.md`(BTRAN 희소 Lᵀ 세 조건, e_r 출력 기존 차이), `docs/lessons/tooling.md`(함수 중간 훅으로 판정 입력 얻기, 검사기 메시지를 먼저 만드는 문제).
