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

- 환경: WSL2 x86-64 Linux 6.6.87.2, Python 3.13 venv, Node 22, Rust stable, 바이너리 `8e87fca2…`. 계약 commit `f2b3419`, 구현 `f5acc48`·`0d77af0`, 회귀 기록 `4ce33a3`. 원본 실행 16회(시험 stair 1 + 정식 15), Gurobi 한 프로세스씩(`tools/gurun.sh`).
- **구현**: `lu::factspace`(`045a76` 식, 분기별 13사례 단위 시험 `factspace_cases`: 64모델 캡처의 분기 조합 9 + SIMPLEXALLOCS 7 → 1922 + grow15 c 0·1 → 131,899·197,849 + stair), `Short::next_cap`(Grow 값 / ×1.5, `crossover_refactor`도 이것을 쓴다), `Engine.pool`(시작 = FactSpace(m, n, Σ`spx.len`, 5, 0)), `refactor()`가 `Lu::factor_dense(…, self.pool)`를 부르고 −1이면 `self.pool = next_cap`으로 S+0x2c0을 올리지 않고 다시 부른다. 무척도 재확인 엔진(`simplex.rs` `e2`, driver `check`)은 `block`처럼 `pool`을 넘겨받는다(원본 대조 없음).
- **실행 중 포팅한 분기 — `local_70` 모드 → 조밀 전환(C:3405 → `LAB_001d88bf`, `local_70` ≠ 0)**: 처음 빌드에서 grow15·grow22 재생 12호출과 자연 실행이 "not ported" assert로 멈췄다. 이 빌드의 노출 측정(`bench/oracle/m1_02w3/wip1/`)에서 lp_check 13실행(grow15 M1·M−1, grow22 M1·M0·M−1, maros-r7 M1·M−1, pilot M1·M0·M−1, pilot87 M1·M0·M−1)이 같은 assert로 멈췄고, 다른 panic 종류는 없었다. C를 읽고 옮겼다(`lu.rs` `mode70`의 게이트): 조밀 n = local_238 = m − steps(C:3964가 mode70 피벗마다 갱신), 위치는 mode70 열 버킷 순서(`local_1f0` 머리, +4 링크, `local_78` → 위치, 그 순번이 `local_1e0`), 행은 살아 있는 mode70 행의 행 순서(`local_68`), 값은 옛 배열에서 |v| ≥ ZERO(`local_88` = param_1)만, 새 배열은 param_15[1] + param_15[3]·8(C:3409)이라 정렬은 그때 Pool p3의 짝홀. LU·변환은 기존 `dense_switch`의 뒷부분을 `dense_steps`로 나눠 함께 쓴다. 빈 열·행이 있는 경우는 assert로 남겼다(5모델에서 0회).
- **원본 재현성**(`--repro`): 5모델 × 3회 사건(대각 포함)·출력 정확 일치. 모든 15실행에서 첫 호출 F+0x158 = FactSpace(rsolver `NumNZs`, k 5, c 0), 다음 호출 진입 = 수명 규칙, F+0x160 = 0·F+0x168 = F+0x158 − 1. 호출 수: grow15 32(조밀 3, `local_70` 진입 4, −1 1회, F+0x158 131,899 → 226,935), grow22 34(조밀 9, `local_70` 16, −1 1회, 193,844 → 274,696), perold 27(조밀 11), pilot4 18(조밀 14, 시작에서 `036ade` 5회 중 4·5번째 조밀), stair 13(조밀 7). Keep(×1.5) 경로는 0회.
- **주입 재생**(`--replay`): **124/124 호출 일치**. 다음 F+0x158, 블록 반환 상태, LU+0xd8·0xdc·0xe0·0xe4·0x100(5필드 전부. 처음 실행은 M1-01e처럼 3필드였고 검토 뒤 5필드로 넓혀도 같았다), dense n, `local_70` 진입, 반환 0 뒤 첫 `21a941`(F+0x18c = 0)의 대각 **106회 전부 비트 일치**(grow15 29, grow22 31, perold 22, pilot4 13, stair 11). tol은 15실행 모두 S+0x390 = 0.0078125.
- **비교 규칙 변경(실행 중)**: 처음 규칙에서 −1 호출 2개가 실패했다(grow15 호출 28 k 1020: LU+0x100 원본 7 / Rust 0, mode70 원본 참 / Rust 거짓; grow22 호출 20 k 781: 11 / 0, 같은 mode70 차이). 원인: (1) 0xd85b4는 `movb $1, local_70`(asm)로 모드 진입 표지이고 풀 검사(C:2572–2620)보다 앞이다. Rust `Lu.mode70`은 모드를 끝낸 경우에만 채워지므로, 탐침·재생의 mode70을 Rust 풀 이벤트 "mode70"(모드 진입) 유무로 바꿨다. (2) 원본 LU+0x100은 블록 단계가 쓰는 t = s3(`1da0b2` C:1514–1534)이고 풀 부족 전에 이미 쓰여 있다. Rust는 t를 `finish`에서 쓰는데 −1이면 `finish`에 가지 않는다. 그래서 −1 호출에서만 원본 LU+0x100을 Rust s3(LU+0xe0)와 비교한다(두 호출 모두 7·11로 일치). −1 뒤에는 재호출이 이 값들을 다시 쓰므로 다음 상태에 영향이 없다. 실패 줄은 `bench/oracle/m1_02w3/mutations.txt`에 남겼다.
- **자연 궤적**(`--rust`, head가 원본과 같은 범위): grow15 5, grow22 8, perold 7, pilot4 12, stair 12 호출이 진입 블록 상태·F+0x158·다음 F+0x158·반환 상태·LU 필드·dense n·mode70까지 일치(`NEED`로 고정). pilot4는 조밀 호출 9개(k 0의 두 번, 77–327)가 자연 비교에 들어간다. 범위 끝은 기존 궤적 차이다. stair는 원본도 13호출(k 356에 두 번, x2c0 12·13)이고, 13번째는 무척도 재확인 엔진의 분해다. head 집합은 같고 순서만 다르며(355자리), 진입 블록 상태 (13, 9, 0)와 cap 142,294는 원본과 같다. 그래서 `e2`의 블록·풀 인계는 이 한 호출에서 맞는다. dense n은 입력 순서가 달라 80 대 78이다. grow15·grow22·perold의 조밀 재분해는 자연 궤적이 먼저 갈라져 재생으로만 확인했다.
- **Engine 재호출 사슬**(`--rust`가 Rust 탐침 전체를 본다. head 차이 뒤도 포함): 풀 부족이면 다음 줄의 x2c0·head가 같고 cap = 새 크기, 특이(rc > 0)면 x2c0이 같고, 그 밖에는 x2c0 + 1이고 cap이 유지돼야 한다. 5모델 모두 통과. Rust 자연 실행도 grow15·grow22에서 풀 부족을 1회씩 겪는다(원본과 다른 궤적의 다른 호출: grow15 Rust 분해 28 → 215,271). `--pool64`(64모델 Rust 실행): 첫 cap이 `bench/oracle/m1_02w/pool` 원본 첫 진입값과 **64/64 일치**(Σ`spx.len` = nnz(A) 확인). 사슬도 통과, Rust 풀 부족은 grow15·grow22 각 1회.
- **판별력**: 입력 변이 cap 1,000,000과 cap ×10이 각각 5/5 모델에서 거부됐다. grow15는 풀 부족이 사라져 거부됐고, 나머지 4모델은 대각 1–2 ulp 차이로 거부됐다(정렬). Rust 임시 변이 `local_70` 조밀 정렬 뒤집기(p3 짝홀)는 grow15 1호출·grow22 7호출에서 대각으로 거부됐다(되돌림). 원본 변조 7종도 거부됐다(빈 사건·조밀 0회·−1 0회·해시 0을 grow15·grow22 세 실행 모두에 넣은 변조 포함). Engine 변이(되돌림): 크기를 늘 ×1.5로 하면(Grow를 Keep으로) 사슬 검사가 grow15·grow22에서 거부한다. 풀 부족 뒤 `continue`를 지운 변이는 거부되지 않았다. 중단된 `Lu`는 특이 목록이 비어 있어 특이 수리 분기가 아무것도 바꾸지 않고, 같은 head와 새 크기로 다시 부르므로 동작이 같은 동등 변이다. 조밀 끔은 기준 빌드로 확인했다: stair urow185 종료 1, pathcmp stair 피벗 211에서 차이. 기록 `bench/oracle/m1_02w3/mutations.txt`.
- **stair e_r 185**: `m1_urow_cmp.py` 기준 빌드 종료 1(대각 53·행 189 차이) → 이번 빌드 **종료 0**.
- **stair e_r 347**: k 337 조밀 재분해는 재생 대각과 자연 LU 필드가 비트 일치한다. 그 뒤 갱신 9회(338–346)의 새 대각도 일치한다(`m1_diag_peel_check`). 처음 갈리는 곳은 e_r 347 BTRAN 출력 값이다(위치·형식은 같음, 출력 0번째 175 ulp). 이어서 피벗 347 FT 새 대각이 8 ulp, 348–355 BTRAN 값이 다르다. 재분해 차이가 아니라 그 뒤 풀이 차이다. 원인은 열어 두고 **M1-02w-4**로 넘긴다. `m1_lt_check` `KNOWN_VAL`에 stair 347–355를 고정했다(고정 호출이 같아지면 실패).
- **회귀**(`bench/oracle/m1_02w3/regress.sh`, 기준 `base/` 대 `final/`, M3 push `push/`):

| 항목 | base | final |
|---|---|---|
| cargo lp·workspace·clippy·pytest(657) | 통과 | 통과 |
| M1 검사기 16종 | 통과 | 통과(btran stair `KNOWN_D` 해제 → 356/356 끝까지, diag_peel stair 범위 179 → 337, lt stair 범위 356 + 값 고정 347–355; lt `--rust --self-test`는 따로 실행해 통과, 변조 15종 거부) |
| bnl1 urow 126·129·130 / stair urow 185 | 일치 / 차이 | 일치 / **일치** |
| `m1_block_check --repro --rust --replay --lu` | (없음) | 통과 |
| pathcmp 완전 일치 / 중앙 첫 차이 | 27/64, 0.886 | **28/64, 0.896** |
| lp_check 정답(M1·M0·M−1) | 90·90·90 | 90·90·90 |
| IterCount 일치(Method 1) | 34/64 | **35/64** |
| M3 push 저장 기준 19명령 | — | 전부 종료 0 |
| debug_assert 빌드 pathcmp all | — | panic 0, 출력이 일반 빌드와 같음(`final/pathcmp_debug_assert.txt`) |

- pathcmp가 바뀐 모델: stair 211 → 356(완전 일치), pilot4 134 → 450. grow22(2046 → 2146 피벗)와 perold(1207 → 1253)는 첫 차이가 그대로다. 앞당겨진 모델은 없다. lp_check는 정답·상태 변화 0이고 반복 수·목적값 끝자리만 바뀌었다. stair는 IterCount가 원본과 같아졌다. maros-r7·pilot·pilot87(면허 밖)은 이제 panic 없이 정답이지만 원본 조밀 경로와 대조하지 않았다.
- 실행하지 못한 것: Keep(×1.5) 경로(0회), 조밀 전환 진입의 work 저장(`LAB_001d88bf` 직후 `param_19` += 0.3·n(n+1)/2·S+0x148, Rust 조밀 경로는 work를 더하지 않는다), 빈 열·행이 있는 조밀 전환(assert 유지), `e2`·driver `check`의 풀 크기 인계 원본 대조, −1 호출의 work 저장(Rust는 호출마다 `work.factor`를 부른다. work는 판정에 미연결이고 M1-02t 범위), Method 0·−1의 조밀 경로 원본 대조(lp_check 정답만 확인).
- 영향 범위: MIP(`crates/mip` `bnb.rs`·`root.rs`)도 `simplex::solve_model_deadline` → `Engine::refactor`를 거치므로 노드 LP의 분해가 조밀 모델에서 바뀐다(pytest 657 통과, MIP 원본 대조는 하지 않음).
- 독립 검토: 결론을 주지 않고 diff와 계약만 준 검토 에이전트(Claude Opus)를 따로 돌렸다. 결과는 아래 §7.
- 계약 변경: (1) 위 −1 호출 비교 규칙 두 가지. (2) 변이 "cap + 1"을 cap 1,000,000으로 바꿨다. cap + 1은 cap/5 항의 짝홀을 거의 바꾸지 않고, 그 항이 지지 않으면 정렬도 바뀌지 않는다(M1-02w §6). (3) 조밀 끔 변이는 기준 빌드 결과로 대신했다. (4) `local_70` → 조밀 전환 포팅을 범위에 넣었다. 계약 §1이 `local_70` 모드를 포함 분기로 정했고, 포팅 없이는 grow15·grow22 자연 실행이 panic이었다.

## 7. 판정과 인수인계

- 작업 수행: **완료(구현)**.
- 동등성: **통과**, 범위는 Method=1·Presolve=0의 5모델(stair·grow15·grow22·perold·pilot4)이다. 원본 `036ade` 124호출을 주입 재생했을 때 풀 수명·조밀 전환·`local_70`·풀 부족 재호출·대각 비트가 모두 일치했다. 첫 F+0x158 = FactSpace(15실행)이고, 자연 궤적 44호출이 일치했다. 수치 비교는 모두 비트 일치다.
- 남은 한계: stair e_r 347–355 BTRAN 값 차이(M1-02w-4). k ≠ 5, c > 0(warm start), Keep ×1.5, 빈 줄 조밀 전환, FT 갱신 ×1.5, Presolve ≠ 0 차원은 미검증이다. 64모델 밖 조밀 모델(maros-r7·pilot·pilot87 등)은 원본과 대조하지 않았다.
- 후속: **M1-02w-4**(조사): stair e_r 347 BTRAN 값 차이. 원본 e_r 347의 단계별 출력(R eta·Uᵀ·Lᵀ)을 Rust 탐침과 비교해 첫 단계를 찾는다. 이어서 M1-02y-2, M1-02t, M1-02u.
- branch `work/m1-02w-3`(기준 main `9cec276`), worktree `../rsolver-m1-02w-3`. 병합 전.
- `docs/progress/M1-02w-3.md` 작성.
- 교훈: `docs/lessons/original.md`(`local_70` → 조밀 전환 수집 규칙, 0xd85b4 진입 표지와 −1 호출의 LU+0x100), `docs/lessons/tooling.md`(대각으로 정렬을 판별하는 재생, 조밀 켜기의 노출 측정, `git apply -p0`).
