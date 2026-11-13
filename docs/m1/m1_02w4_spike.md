# M1-02w-4 stair e_r 347 BTRAN 값 차이: FT 새 대각의 spike 0 처리 검증 계약과 결과

작성 기준은 [스프린트 검증 기준](../15_sprint_verification.md)이다.

## 1. 범위와 주장

- 작업 종류: **조사로 시작해 구현으로 넓힘.** 14의 M1-02w-4는 "stair e_r 347 BTRAN 출력 값 175 ulp 차이의 첫 단계 특정, `m1_lt_check` `KNOWN_VAL` stair 347–355 해제"를 목표로 둔 조사다. 탐색(§4, 원본 3회 실행, 각 1회)에서 원인을 원본 C로 특정했고 수정이 한 줄이어서 같은 번호에서 구현까지 진행한다. 계약은 탐색 뒤에 썼다(탐색 결과를 보고 쓴 계약임을 밝혀 둔다).
- 조사 질문: e_r 347에서 처음 갈리는 단계와 그 원인.
- 답(탐색): e_r 347 BTRAN의 행 eta 10개는 원본과 비트 일치하고, Uᵀ 출력 81개 중 행 199 하나가 8 ulp 다르다. e_r 347 시점 U 행 356개는 같고 대각 LU+0x50[43] 하나만 다르다. 그 대각은 e_r 346 뒤 FT 갱신(r 43)의 새 대각 η·S다. 원본 η(LU+0x1d0)와 원본 S(LU+0x1c8)로 원본의 합산 순서를 흉내 내면 원본 대각과 비트까지 같다. Rust와 다른 것은 S 한 칸(행 305)뿐이다. 원본은 0이고 Rust는 `0x3cc4000000000000`(≈5.6e-16 < ZERO)이다.
- 원인: `09d112`는 LU+0x1c8에 spike 목록 칸만 쓴다. 조밀 쪽은 `ZERO < |x|`인 위치만 쓰고(C:1744–1786), 희소 쪽은 목록 idx 칸만 흩뿌린다(C:1853–1880). `05b257`은 갱신에서 목록 값을 U로 옮기며 0으로 지우고(C:1222–1223, C:1246–1247), 재분해 경로도 목록 칸을 0으로 지운다(C:2006–2030). 그래서 목록 밖 칸은 0이다. Rust `Lu::update`는 η·S에 조밀 spike 배열을 그대로 읽어 ZERO 이하 잔차를 더했다. stair는 조밀 재분해(337) 뒤 LU+0x104 = m이 되어 진입 열이 조밀 L(`214344`, 잔차를 지우지 않음)로 가면서 처음 드러났다.
- 구현: `Lu::update`의 새 대각 η·S에서 |S_i| ≤ ZERO인 행은 0을 읽는다(`crates/lp/src/lu.rs`). 진단 탐침 `RSOLVER_STAGE_PROBE=k0,k1`(`Lu::btran_row_stages`, `btran_probe`)을 더한다.
- 포함 분기: dual(Method 1) FT 갱신에서 η가 있는 새 대각(C:321–372 경로). 조밀 spike(목록은 `dense_order`)와 희소 spike(목록은 `09d112` 순서). Rust 목록과 "|S| > ZERO인 행"은 같다(`debug_assert` "spike list = spike support").
- 제외·후속: η가 없는 갱신의 대각 `spike[rp]`(|S_r| ≤ ZERO이면 원본은 0, Rust는 잔차다. 둘 다 `|diag| ≤ ZERO`라 같은 재분해 판정이며 값은 쓰이지 않는다), U 새 열 값(이미 `> ZERO` 필터), crossover·push의 F+0x1c8(`crossover_lu.rs`가 원본 배열 표현을 따로 가짐, M3), Method 0 primal 갱신 경로(같은 `update`를 타지만 원본 대조 없음), 재분해 337의 L 열 직접 비교(미실행. 간접 근거는 §6).
- 선행: main `7398456`(M1-02w-3 병합 뒤), 미커밋 선행 변경 없음.

## 2. 원본과 상태 경계

- Gurobi 13.0.3, `libgurobi130.so` SHA-256 `8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c`.
- 원본 위치: `05b257`(0xf15c0) C:250–255(local_58/local_60 = LU+0x1c8의 idx·값), C:321–420(η·S, η = LU+0x1d0), C:1222–1247(목록 값 0으로 지움), C:2006–2030(재분해 경로에서 지움). `09d112`(0x125f50) C:1744–1786(조밀 쪽 목록 만들기, `dVar59 < ABS`), C:1853–1880(희소 쪽 목록 복사). `214344`(0x12bd30, 조밀 L, 잔차 유지). `221173`(0x12de60) C:1235–1376(희소 Uᵀ 뒤 조밀 행 eta, 0x12e912에서 조밀 Lᵀ로).
- 훅 근거: 0x12e912 `mov $0xffffffff,%r11d`는 0x12e8fe `movl $-1,0(%r13)`(*param_2 = −1) 바로 뒤라 r13 = param_2다. 캡처에서 r13 = 진입 param_2인지 매번 확인한다(`hit912` = 1). 05b257 진입 rdi = spx, LU = spx+0x470, r = spx+0x2f8(`tools/m1_eta_align.js`와 같은 근거).
- 시작·종료: 새 대각은 05b257 진입(LU+0x1c8·0x1d0)과 반환(LU+0x50[r]). e_r 출력은 221173 진입·반환(dual e_r = 9e376 반환 0xc51dd).
- 설정: Method=1, Presolve=0, Threads=1, Netlib.

## 3. 비교 계약

| 필드·사건 | 비교 규칙 | 방법 |
|---|---|---|
| 갱신 새 대각 (r, LU+0x50[r] 비트) | 정확 일치. `m1_diag_peel_check` 6모델 기존 3회 기준 자료(`bench/oracle/m1_02y`). stair 일치 범위가 337보다 늘어야 하고, 다른 모델 범위는 줄면 안 된다 | `tools/m1_diag_peel_check.py --rust --self-test` |
| e_r 출력 형태·idx·값 비트 | 정확 일치. `m1_lt_check` 4모델 3회 기준 자료(`bench/oracle/m1_02x`). stair 347–355가 같아져야 하고(그때만 `KNOWN_VAL` 해제) 목록 밖 새 차이 없음 | `tools/m1_lt_check.py --rust --self-test` |
| e_r 단계(행 eta, Uᵀ 출력, eta 뒤 벡터) | e_r 335–348 비트 일치(탐색 자료, 1회) | `bench/oracle/m1_02w4/stage_cmp.py` |
| 원본 전제: LU+0x1c8 목록 밖 값 | 탐색한 갱신 12개 모두 0 | `stair_upd_337_348` 집계 |
| 비간섭 | 새 캡처의 (r, LU+0x180, 출력 nnz)와 범위 안 출력 값이 `m1_02x` stair_1과 같음 | 집계 |

- 오차 허용 없음(비트 비교). 시간·work 비교 없음.
- 원본 반복: 판정에 쓰는 기준 자료는 기존 3회 자료(02x·02y)다. 새 캡처 두 종(§4)은 탐색 1회 자료이고 판정 근거가 아니라 원인 특정의 근거다.

## 4. 사례와 원본 자료

| 사례 | 실제/합성 | 노리는 분기 | 근거 |
|---|---|---|---|
| stair e_r 346 뒤 갱신(r 43) | 실제 | 조밀 spike, η 103항, 목록 밖 잔차 1칸 | 탐색 캡처 |
| stair·sc50a·scsd1·afiro·kb2·degen2 갱신 | 실제 | peel·비peel, 희소·조밀 spike | `bench/oracle/m1_02y` |
| stair·degen2·fit1p·bnl1 e_r | 실제 | 희소·조밀 Uᵀ/Lᵀ | `bench/oracle/m1_02x` |

탐색 자료(`bench/oracle/m1_02w4/explore/`, 각 원본 1회, 기준 자료 아님):

- `stair_dense_335_348.json.gz`: `tools/m1_er_dense.js`(새 도구). e_r 335–348의 행 eta, `21a941` 반환 뒤 Uᵀ 출력, 0x12e912 조밀 벡터, 최종 출력.
- `stair_urow347.json.gz`: `tools/m1_urow_dump.js … 1 347`. e_r 347 `21a941` 진입의 행 U와 대각.
- `stair_upd_337_348.json.gz`: `tools/m1_upd_spike.js`(새 도구). e_r 337–348 사이 갱신 12개의 spike(LU+0x1c8 idx와 m칸 값), η 캐시(LU+0x1d0), 새 대각.
- Rust 탐침 출력: `rust_stair.spec.out`(수정 전), `rust_urow.spec.out`(수정 전, e_r 347 U), `rust_fix.spec.out`(수정 후).
- 자연 사례에 없는 분기: η 경로에서 |S_r| ≤ ZERO(그 갱신은 원본도 대각이 작아 재분해), |S| = ZERO 정확히 같은 값.

## 5. 재실행 명령

- 탐색 캡처(Gurobi 한 프로세스씩, `FRIDA_NODE=/home/sds/.npm/_npx/da7caeae9db9c589/node_modules/frida`):
  `tools/gurun.sh node tools/m1_er_dense.js ../re/bench/data/netlib/stair.mps.gz <out> 335 348`,
  `tools/gurun.sh node tools/m1_upd_spike.js ../re/bench/data/netlib/stair.mps.gz <out> 337 348`,
  `tools/gurun.sh node tools/m1_urow_dump.js ../re/bench/data/netlib/stair.mps.gz <out> 1 347`.
- Rust 단계: `: > S; RSOLVER_BTRAN_PROBE=S RSOLVER_STAGE_PROBE=335,348 PYTHONPATH=$PWD/python ../.venv/bin/python -c "import rsolver; m=rsolver.read('../re/bench/data/netlib/stair.mps.gz'); m.Params['Method']=1; m.Params['Presolve']=0; m.optimize()"`, 비교 `python3 bench/oracle/m1_02w4/stage_cmp.py bench/oracle/m1_02w4/explore/stair_dense_335_348.json.gz S.out`.
- 판정: `tools/build_local.sh && PYTHONPATH=$PWD/python ../.venv/bin/python tools/m1_diag_peel_check.py --self-test`, `… tools/m1_lt_check.py --self-test`.
- 회귀: `bench/oracle/m1_02w4/regress.sh final`(m1_02w3과 같은 구성 + debug_assert pathcmp + push 회귀). base는 `bench/oracle/m1_02w3/final`(그 commit `6704bdd`과 main `7398456`의 `crates/` 차이는 `assert!` 한 줄뿐이라 동작이 같다).
- 회귀 판정 규칙(착수 때 고정): `pathcmp` 첫 차이가 앞당겨진 모델 0, `lp_check` Method 1 정답 수가 90보다 줄지 않음, 바뀐 모델은 목록으로 남김(경로 오라클 없는 대형 모델은 미검증 궤적 변화). M1 검사기는 모두 통과하되 고정 목록이 같아져 실패하면 하나씩 분류하고 조건을 넓히지 않는다.
- 변이: (1) 마스크 제거(수정 전) → 거부돼야 함, (2) `>` → `>=` → 판별 사례가 없어 생존 예상(미검증으로 기록), (3) 마스크를 η 경로 밖 `spike[rp]`에도 적용 → 동치 예상.

## 6. 실행 결과

(작성 중)

## 7. 판정과 인수인계

(작성 중)
