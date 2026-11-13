# M1-02w-3 M1 재분해 조밀 전환 켜기 검증 계약과 결과

작성 기준은 [스프린트 검증 기준](../15_sprint_verification.md)이다. 기준 main `9cec276`, 미커밋 선행 변경 없음. 작업자는 이 세션(Claude Code, "다음 스프린트 진행해 너가 작업자야" 지시. M2·M3는 다른 세션 점유라 M1 다음 작업). branch `work/m1-02w-3`, worktree `../rsolver-m1-02w-3`.

## 1. 범위와 주장

- 작업 종류: 구현.
- 완료할 동작: M1 `Engine::refactor`가 `036ade`의 조밀 전환을 켠 `Lu::factor_dense(cap)`로 분해한다. cap은 F+0x158 수명을 따른다. (1) 초기값은 `045a76` FactSpace 식([M1-02w-2](m1_02w2_factspace.md), m·n·nnz(A), k = 5, c = 0). (2) `036ade` 풀 부족(−1)이면 `05b257` C:1479–1513처럼 새 크기(`Grow`면 `036ade`가 남긴 값, `Keep`이면 ×1.5)로 S+0x2c0을 다시 올리지 않고 처음부터 다시 부른다. (3) 그 값은 다음 재분해로 이어진다.
- 포함 분기: 조밀 전환(dense:enter), `local_70` 모드, 풀 부족 재호출(grow15·grow22). 대상 5모델 stair·grow15·grow22·perold·pilot4(64모델 중 원본이 조밀 전환을 쓰는 전부, [M1-02w](m1_02w_lu108.md) §4).
- 제외와 후속: FT 갱신 풀 부족 ×1.5(`05b257` C:802–846, 64모델 0회), `GURO_PAR_SIMPLEXALLOCS` ≠ 5 연결과 모델 카운터 c(warm start 재풀이, Rust에 모델 카운터 없음), 할당 실패 재시도, Presolve ≠ 0 차원에서의 식, quad. −1 반환 호출의 work 저장(work는 판정에 미연결, M1-02t).
- 선행: M1-02w-2(식), M3-04c-21i/21j(`factor_dense`·풀 부족 루프, push 경로 검증).

## 2. 원본과 상태 경계

- Gurobi 13.0.3, `libgurobi130.so` SHA-256 `8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c`.
- `05b257`(0xf15c0) C:1399 S+0x2c0 += 1 → C:1427 `LAB_001f3392` 루프: F+0x168 = F+0x158 − 1, `036ade(ZERO, S+0x390, S, …)`. 반환 −1이면 풀 해제, F+0x158 ≠ F+0x168(= `036ade`가 크기를 안 바꿈)일 때 ×1.5, F+0x170 += 1, 재할당 후 루프 처음으로(S+0x2c0 증가 없음). 반환 0이면 이어서 특이 수리·설정.
- `036ade`(0xd4190): 조밀 전환 C:1526–1626·C:3394–3470·C:3968–5300(dense:enter 0xd88bf, `local_70` 0xd85b4, dgetrf 0x9fef00), 풀 부족 Grow C:2592–2620.
- 시작 경계: `036ade` 진입(head args[8], S 블록 상태, S+0x390 tol, F+0x158/0x160/0x168). 종료 경계: 반환(rc, S 블록 상태, F+0x158/0x168, LU+0xd8/0xdc/0xe0/0xe4/0x100), 그리고 반환 0 뒤 첫 `21a941`(F+0x18c = 0, 갱신 전)에서 대각 LU+0x50.
- 설정: Method=1, Presolve=0, Threads=1, 그 밖 기본.

## 3. 비교 계약

| 필드·사건 | 규칙 | 도구 |
|---|---|---|
| 원본 3회 독립 실행 | 사건 목록(대각 포함) 정확 일치 | `tools/m1_dense_check.py --repro` |
| 주입 재생: 원본 호출마다 원본 head·진입 블록 상태·tol·F+0x158로 Rust `factor_dense` | rc(0 / −1), 반환 F+0x158(Grow 값 또는 그대로), 블록 반환 상태, LU 5필드, dense n 목록, mode70 유무, 대각(재번호 뒤 위치별 u64 비트) 정확 일치 | `--replay`(`crates/lp/tests/m1_dense_replay.rs`) |
| F+0x158 수명: 첫 호출 진입 = FactSpace 식, 다음 호출 진입 = (−1이면 Grow 값 / ×1.5, 아니면 직전 반환) | 정확 일치, 모든 호출 | `--repro` 안의 수명 검사 |
| 자연 궤적: Rust `RSOLVER_BLOCK_PROBE`에 cap·rc·dense n | head가 원본과 같은 범위의 호출에서 위 필드 정확 일치, 모델별 최소 비교 수 | `--rust` |
| stair e_r 185 행 U·대각 | `m1_urow_cmp.py` 종료 0(기준 빌드 종료 1, `bench/oracle/m1_02w3/base/urow185.txt`) | regress |

- 수치: 대각은 비트 비교. 오차 허용 없음.
- 검사기 판별력: 재생에서 cap 짝홀을 바꾼 변이(cap + 1), 풀 부족 무시(Grow를 성공으로), 조밀 끔(`factor_block`)이 각각 하나 이상 거부돼야 한다. Rust 변이는 임시 사본에서 넣고 최종 코드에 남기지 않는다.
- 회귀 규칙(실행 전 고정, M1-02w §3): stair를 `m1_btran_check.py` `KNOWN_D`에서 빼고 `m1_btran_check`·`m1_diag_peel_check`·`m1_lt_check`의 stair 범위를 늘린다. 기존 M1 검사기 전부 통과(바뀌면 원인 기록), pathcmp 첫 차이가 앞당겨진 모델 0·완전 일치 수 감소 0, lp_check 정답 수(Method 1·0·−1) 감소 0, `cargo test --release --workspace`·clippy 종료 0, M3 push 저장 기준 회귀(`bench/oracle/m1_01e/push_regress.sh`) 종료 0. 기준선은 `bench/oracle/m1_02w3/base/`.
- stair e_r 347 175 ulp 차이(M1-02w): 재생으로 피벗 337 재분해 자체(대각)가 다른지, 그 뒤 갱신이 다른지 가른다. 남으면 원인과 후속 번호를 기록한다.

## 4. 사례와 원본 자료

| 사례 | 노리는 분기 | 실행 |
|---|---|---|
| stair | 조밀 전환 7재분해, pathcmp 완전 일치 후보 | 3회 |
| grow15 | 조밀 + `local_70` + 풀 부족 Grow 1회 | 3회 |
| grow22 | 조밀 + `local_70` 16 + 풀 부족 1회 | 3회 |
| perold | 조밀 11재분해 | 3회 |
| pilot4 | 첫 분해부터 조밀 | 3회 |

- 자료 `bench/oracle/m1_02w3/formal/<모델>-<회>.json.gz`(`tools/m1_dense_state.js`, `runs.sh`). 시험 캡처 `explore/stair_try.json.gz`.
- 0회 분기: Keep 경로 ×1.5(5모델에서 관측되면 포함), FT 갱신 ×1.5, 할당 실패, 특이 기저 + 조밀.

## 5. 재실행 명령

```bash
export FRIDA_NODE=/home/sds/.npm/_npx/da7caeae9db9c589/node_modules/frida
bench/oracle/m1_02w3/runs.sh formal                       # 원본 15회
tools/build_local.sh
PYTHONPATH=$PWD/python ../.venv/bin/python tools/m1_dense_check.py --repro --replay --rust --self-test
bench/oracle/m1_02w3/regress.sh final                     # 회귀(기준선 base)
bench/oracle/m1_01e/push_regress.sh                        # M3 push 저장 기준
```

## 6. 실행 결과

(실행 뒤 작성)

## 7. 판정과 인수인계

(실행 뒤 작성)
