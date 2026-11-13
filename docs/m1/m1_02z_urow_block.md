# M1-02z bnl1 재분해 U 위치 번호 검증 계약과 결과

작성 기준은 [스프린트 검증 기준](../15_sprint_verification.md)이다. 기준 main `3f19629`, 미커밋 선행 변경 없음. 작업자는 이 세션(Claude Code, "다음 스프린트 진행해 너가 작업자야" 지시. M2-06h-6은 다른 세션이 점유 중이라 사용자 선택으로 M1-02z). branch `work/m1-02z`, worktree `../rsolver-m1-02z`.

## 1. 범위와 주장

- 작업 종류: 조사.
- 조사 질문: bnl1(Method=1, Presolve=0) e_r 126 직전 재분해 직후 행 U·대각의 위치 번호가 588–607에서 원본과 다른 원인은 무엇인가([M1-02y §4](m1_02y_diag_peel.md#4-사례와-원본-자료)).
- 답의 형식: 원인 단계(재분해 안의 어느 단계·상태)를 Rust 재현으로 특정하고, 구현에 필요한 상태와 후속 번호를 정한다.
- 제외: 구현(원본처럼 켜는 일과 상태 수명)은 **M1-01e**(14에 이미 등록된 `1da0b2` 2×2 블록 단계 켜기). 원본 상태 캡처, pathcmp·Netlib 회귀, t(F+0x100) = s3 정정도 M1-01e다.
- 선행: M1-02y(`tools/m1_urow_dump.js`, 탐색 원본 덤프 `bench/oracle/m1_02y/explore/bnl1_urow{126,129,130}.json.gz`), M3-04c-21h(`Lu::factor_block`, `BlockState`).

## 2. 원본과 상태 경계

- Gurobi 13.0.3, `libgurobi130.so` SHA-256 `8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c`.
- 비교 지점: k번째 dual e_r BTRAN(`221173` via `9e376`, 반환 0xc51dd)의 `21a941` 진입에서 행 방향 U(LU+0xb0/0xb8/0xd0/0xc8)와 대각 LU+0x50(`tools/m1_urow_dump.js`). Rust는 같은 e_r 호출 번호에서 `Lu::u_rows`(위치별 (열 위치, 값))와 `diag_of`.
- 관련 원본: `1da0b2` C:681–1510(2×2 블록 단계, 실행 조건 C:681–684: 활성 4…5·10⁸, S+0x4e4 ≥ 0, S+0x4e4 = 2e9 또는 활성 ≥ 50), `02f7e1` C:270–281(LU2X2BLOCK → S+0x4e4, 기본 → 0), `05b257` C:1399(S+0x2c0 +1). 근거 정리는 [M3-04c-21h](../m3/primal_push_block_rust.md).
- M1 Rust 현황: `Lu::factor_tol`이 `BlockState::OFF`(S+0x4e4 = −1)로 블록 단계를 끈다.

## 3. 비교 계약

| 필드 | 규칙 | 근거 |
|---|---|---|
| 행 U(위치별 열 위치 목록, 저장 순서 포함) | 정확 일치 | `tools/m1_urow_cmp.py` |
| 행 U 값·대각 | u64 비트 일치 | 같음 |

- 원본 자료는 M1-02y 탐색 1회분이다(기준 자료 아님). 이번 작업은 조사라 원본 재현성 3회 대신 저장 덤프만 쓴다. Gurobi는 다른 세션이 쓰고 있어 새로 실행하지 않았다.

## 4. 사례와 원본 자료

| 사례 | Rust 블록 단계 | e_r 126 | 129 | 130 |
|---|---|---|---|---|
| bnl1 Method=1 | 끔(현재, `BlockState::OFF`) | 대각 20·행 32 차이 | 같은 차이 | 같은 차이 |
| bnl1 Method=1 | 켬(실험, 재분해마다 `{0,0,0}`) | **전부 일치**(643행) | 일치 | 일치 |

- 끔: 위치 588–609에서 Rust 번호가 원본보다 2 작다(예: 행 527 열 원본 602 ↔ Rust 600, 원본 대각 591–609 = Rust 589–607). 원본은 588·590에 빈 행·부호가 반대인 대각 쌍(`bfea47ae147ae148`, `3fea47ae147ae148`)을 두고 589에 행 [590]을 둔다. 블록 단계가 피벗한 두 단계가 단계 4 꼬리 앞에 끼어든 모양이다.
- 켬: [block_on.patch](../../bench/oracle/m1_02z/explore/block_on.patch)를 적용한 빌드에서 세 호출 모두 행 U·대각이 비트로 같다.
- 실험의 한계: 상태를 재분해마다 `{x2c0 0, x4e0 0, x4e4 0}`으로 새로 만들었다. 원본은 S에 상태를 남기고 재분해마다 S+0x2c0을 올리며, 블록 피벗이 20단계 이상이면 S+0x4e4를 올리고 S+0x2c0 > 20에서 −1 또는 2e9로 고정한다(`lu.rs` 1459–1464, C:1494–1510). 실험은 이 전환을 한 번도 타지 않았다. 반복 수도 원본 1050, Rust 끔 1087, 켬 1031로 아직 다르다(e_r 130 뒤 다른 차이 또는 상태 수명).
- 증상 연결: 켬 빌드에서 `tools/m1_lt_check.py --rust`는 bnl1 고정 차이(e_r 130)가 사라져 "pinned order difference not seen"으로 실패한다([로그](../../bench/oracle/m1_02z/explore/lt_check_block_on.txt)). 고정 목록 `KNOWN_IDX`의 bnl1 7호출을 비운 임시 사본은 4모델(stair·degen2·fit1p·bnl1) 같은 상태 범위 전부가 일치해 통과했다([로그](../../bench/oracle/m1_02z/explore/lt_check_block_on_unpinned.txt), 임시 사본은 지웠다). 그래서 14의 증상(e_r 130 등 7호출)도 블록 단계로 설명된다. 범위 밖(bnl1 e_r 174 뒤 LU+0x108, M1-02w)은 보지 않았다.
- 원본에서 블록 단계 실행을 직접 캡처하지는 않았다. "켜면 같아진다"는 Rust 재현 증거이며 원본 상태 연쇄는 M1-01e 캡처로 확인한다.

## 5. 재실행 명령

```bash
tools/build_local.sh   # worktree 격리 빌드
: > S.spec; RSOLVER_BTRAN_PROBE=S.spec RSOLVER_UROW_PROBE=126 PYTHONPATH=$PWD/python ../.venv/bin/python -c \
 "import rsolver; m=rsolver.read('../re/bench/data/netlib/bnl1.mps.gz'); m.Params['Method']=1; m.Params['Presolve']=0; m.optimize()"
python3 tools/m1_urow_cmp.py bench/oracle/m1_02y/explore/bnl1_urow126.json.gz S.spec.out   # 0 = 같음, 1 = 차이
```

켬은 `git apply bench/oracle/m1_02z/explore/block_on.patch` 뒤 다시 빌드해 같은 명령이다. 증상 확인: 켬 빌드에서 `PYTHONPATH=$PWD/python ../.venv/bin/python tools/m1_lt_check.py --rust`(고정 목록 그대로면 bnl1 e_r 130에서 실패, `KNOWN_IDX`를 비우면 통과). 원본 덤프 재생성: `tools/gurun.sh node tools/m1_urow_dump.js ../re/bench/data/netlib/bnl1.mps.gz <out> 1 126`.

## 6. 실행 결과

- 실행 commit: 기준 main `3f19629` + 이 branch의 탐침(`simplex.rs` `RSOLVER_UROW_PROBE`)·`tools/m1_urow_cmp.py`. x86-64 Linux(WSL2).
- 끔: [126](../../bench/oracle/m1_02z/explore/bnl1_urow126_block_off.txt)·[129](../../bench/oracle/m1_02z/explore/bnl1_urow129_block_off.txt)·[130](../../bench/oracle/m1_02z/explore/bnl1_urow130_block_off.txt) 모두 대각 20·행 32 차이(같은 위치 집합), 종료 1.
- 켬: [126](../../bench/oracle/m1_02z/explore/bnl1_urow126_block_on.txt)·[129](../../bench/oracle/m1_02z/explore/bnl1_urow129_block_on.txt)·[130](../../bench/oracle/m1_02z/explore/bnl1_urow130_block_on.txt) 차이 0, 종료 0.
- 검사기 판별력: 같은 원본 덤프에 대해 끔 빌드는 종료 1, 켬 빌드는 종료 0이라 위치 번호 차이를 구분한다.
- 회귀(탐침만 있는 빌드): `cargo test --release -p rsolver-lp` 종료 0, 68 통과·0 실패·1 무시([로그](../../bench/oracle/m1_02z/cargo_lp.txt)), `cargo clippy --workspace --all-targets -- -D warnings` 종료 0([로그](../../bench/oracle/m1_02z/clippy.txt)). 탐침은 `RSOLVER_UROW_PROBE`가 있을 때만 동작한다.
- 실행하지 못한 것: 원본 재캡처(다른 세션이 Gurobi 사용 중), M1 재분해의 원본 블록 상태 연쇄, 켬 빌드의 다른 모델·검사기 회귀(M1-01e 범위).

## 7. 판정과 인수인계

- 작업 수행: **완료(조사)**. 원인: M1 재분해가 `1da0b2` 2×2 블록 단계를 끈 채 돈다. 켜면 bnl1 e_r 126·129·130의 행 U·대각이 원본과 비트로 같고, `m1_lt_check` 같은 상태 범위의 bnl1 7호출 차이도 사라진다. 14 행은 "조사·구현"이었으나 구현은 이미 등록된 M1-01e와 같은 일이라 조사로 닫는다. 동등성 판정은 하지 않는다(조사).
- 후속: **M1-01e**(구현). M1 spx에 S+0x2c0/0x4e0/0x4e4 수명을 두고 재분해 호출자에서 넘긴다. 원본 `036ade` 진입·반환의 상태를 M1 Method=1에서 캡처해 연쇄를 대조한다(`tools/push_block_agent.js`의 `lu:block` 훅 재사용 검토). 블록 단계가 활성 ≥ 50 재분해의 피벗 순서를 바꾸므로 pathcmp·lp_check·`m1_lt_check`(bnl1 KNOWN 7호출)·`m1_btran_check`·`m1_diag_peel_check` 회귀가 필요하다. t = s3 정정도 같은 작업이다.
- branch `work/m1-02z`, worktree `../rsolver-m1-02z`. 미커밋 선행 없음.
- 교훈: `docs/lessons/original.md`(bnl1 U 위치 차이 = 블록 단계 끔).
