# M1-02w-2 초기 원소 풀 크기 F+0x158 검증 계약과 결과

작성 기준은 [스프린트 검증 기준](../15_sprint_verification.md)이다. 기준 main `01549af`, 미커밋 선행 변경 없음. 작업자는 이 세션(Claude Code, "다음 스프린트 진행해 너가 작업자야" 지시. M2-06h-7은 다른 세션 점유라 M1 다음 작업). branch `work/m1-02w-2`, worktree `../rsolver-m1-02w-2`.

## 1. 범위와 주장

- 작업 종류: 조사.
- 조사 질문: M1 dual(Method=1, Presolve=0) 첫 `036ade` 진입의 F+0x158(원소 풀 크기, [M1-02w](m1_02w_lu108.md))을 누가 어떤 식으로 정하는가. 입력 필드와 그 출처는 무엇인가.
- 답의 형식: 쓰는 함수·명령 주소, C·asm으로 세운 식, 식이 읽는 필드의 출처(파라미터·모델 상태), 64모델 저장 캡처와 식의 대조, 직접 캡처로 입력 필드를 관측한 대조.
- 제외: Rust 구현(M1-02w-3), quad 경로(`1dec07`, spx+0x28 ≠ 0), 할당 실패 재시도(첫 시도 성공만), FT 갱신 ×1.5·`036ade` 풀 부족 재호출(M1-02w에서 수명 확인).
- 작업 순서 기록: 쓰는 함수를 찾은 직후 저장 캡처(`bench/oracle/m1_02w/pool/`)와 식을 먼저 대조했다(64/64). 이 계약은 그 뒤에 썼고, 아래 직접 캡처의 예측값은 캡처 전에 고정했다.

## 2. 원본과 상태 경계

- Gurobi 13.0.3, `libgurobi130.so` SHA-256 `8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c`.
- 쓰는 곳: `045a76`(0xe3130) 끝, `plVar3[0x2b] = lVar7`(C:454, asm 0xe3bf0 `mov %r15,0x158(%r12)`), F = spx+0x470. 같은 자리에서 F+0x168 = F+0x158 − 1, F+0x160 = 0. 로그 문자열 "Column=%d, Row=%d, Nonzero=%d, FactSpace=%d"의 FactSpace가 이 값이다. M1-02w의 정적 grep(`+ 0x158) =`)이 놓친 이유는 쓰기가 `long*` 첨자 `[0x2b]`로 나오기 때문이다.
- 호출: `12f54b`(0x1bc0b0)가 spx+0x28(quad 플래그) == 0이면 `045a76`, 아니면 `1dec07`로 tail jump. `12f54b`의 호출자는 `02f7e1` C:1228(spx 준비), C:141·166(Quad 파라미터 전환), `2d1de`·`2d3b2`(정밀도 전환). double M1은 `02f7e1` C:1228.
- 식(asm 0xe3697–0xe3bf0, 상수는 바이너리 .rodata에서 읽음): m = spx+0x64, n = spx+0x68, z = spx+0x3d0(nnz(A)), k = spx+0x50, c = `*(*(spx+0x4b8)+0xe8)+0x44`.
  - m < n: d = z/n, x = d²(m < 500), d·√d(m < 3000), m < 10000이고 m > 4d이면 q = d·d^¼, x = (0.25m > q ? q : 0.25m), 그 밖 d. D0 = min(⌊m/2⌋ + 2, x)·m.
  - m = 0: D0 = 100. 그 밖(m ≥ n > 0): d = z/m, x = d²(n < 500), d·√d(< 3000), d·d^¼(< 10000), d. D0 = 0.5z + min(⌊n/2⌋, x)·n.
  - r = (long)D0 + 1. z > 10n이면 D = 6r + 30·m·k, 아니면 m < n일 때 D = 6r + 3·m·z·k/n, m ≥ n일 때 D = 6r + 3·m·(m + z)·k/(m + 1 + n).
  - D ×1.5를 c번(c ≥ 8은 짝 mulpd 경로), 그다음 할당 루프(0xe3a7c): D ×0.75, 크기 (long)D + 1로 두 배열(4·8바이트) 할당, 실패하면 반복(최대 k − 10까지). 첫 시도 성공이면 F+0x158 = (long)(0.75D) + 1.
- k 출처: env+0x2420 `GURO_PAR_SIMPLEXALLOCS`(기본 5, `02f7e1` C:222 → spx+0x50; `crates/model/src/param_table.rs`).
- c 출처: 모델 쪽 카운터. 풀이를 마친 뒤 `0cda5b` C:2097–2105와 `14abfb` C:1602–1610이 spx+0x54 == 0(이고 `0cda5b`에서는 param_1+0xf0 == 0)이면 c = max(c, F+0x170)로 쓴다. F+0x170은 풀 재할당 수(M3-04c-21j, `05b257` C:1479–1513). 그래서 새 모델의 첫 풀이는 c = 0이다. 풀을 다시 잡은 모델을 warm start로 다시 풀면 c > 0이고 `reset()` 뒤는 0이다(§6).

## 3. 비교 계약

| 필드·사건 | 규칙 | 도구 |
|---|---|---|
| 저장 캡처의 첫 `036ade` 진입 F+0x158 대 식(z = rsolver `NumNZs`, k 5, c 0) | 정확 일치 64/64 | `tools/m1_factspace.py` |
| 직접 캡처 `045a76` 반환 F+0x158 대 같은 호출 진입에서 읽은 m·n·z·k·c로 계산한 식 | 정확 일치, 모든 호출 | `tools/m1_factspace.js` + `m1_factspace.py --direct` |
| 직접 캡처 F+0x158 = 첫 `036ade` 진입 F+0x158 | 정확 일치(사이에 다른 쓰기 없음) | 같은 캡처 |
| 직접 캡처 z = rsolver `NumNZs`, k·c 관측값 | 정확 일치 | 같은 캡처 |
| 3회 독립 실행 | 기록(events) 동일 | 같은 캡처 |

- 판별 사례 예측(캡처 전 고정): afiro `GURO_PAR_SIMPLEXALLOCS` = 7 → k 7, F+0x158 **1922**. grow15을 같은 프로세스에서 `reset()` 뒤 다시 풀 때 `045a76`이 다시 불리면 c = 1(첫 풀이의 풀 재할당 1회), F+0x158 **197,849**. 다시 불리지 않으면 그 사실을 기록하고 c 판별은 미검증으로 둔다. (결과: §6)
- 검사기 self-test: 식의 k·c·10n 판정·half 상한·0.75를 바꾼 변이가 저장 캡처 중 하나 이상에서 거부돼야 한다.

## 4. 사례와 원본 자료

| 사례 | 노리는 분기 | 실행 |
|---|---|---|
| afiro | m < n, m < 500, z ≤ 10n | 3회 |
| 25fv47 | m < n, 500 ≤ m < 3000 | 3회 |
| agg | m ≥ n, n < 500 | 3회 |
| fit1d | z > 10n | 3회 |
| stair | M1-02w-3 대상 | 3회 |
| afiro, SIMPLEXALLOCS 7 | k ≠ 5 | 3회 |
| grow15 `reset()` 뒤 두 번 풀기 | c ≠ 0(결과: 관측 c = 0, c ≠ 0은 warm start 탐색 3회, §6) | 3회 |

- 0회 분기(C·asm만): m ≥ 3000, m ≥ n이면서 n ≥ 500, m = 0, c ≥ 8, 할당 실패 재시도, quad, Presolve ≠ 0 차원.

## 5. 재실행 명령

```bash
export FRIDA_NODE=/home/sds/.npm/_npx/da7caeae9db9c589/node_modules/frida
PYTHONPATH=../rsolver/python ../.venv/bin/python tools/m1_factspace.py --self-test      # 저장 64모델
bench/oracle/m1_02w2/runs.sh final                                                        # 원본 21회
# c 수명 탐색: reset 뒤 재풀이 / 목적 계수 1e-7 변경 뒤 warm start 재풀이
tools/gurun.sh node bench/oracle/m1_02w2/explore_c.js ../re/bench/data/netlib/grow15.mps.gz <out>.json.gz '{}' twice
tools/gurun.sh node bench/oracle/m1_02w2/explore_c_mod.js ../re/bench/data/netlib/grow15.mps.gz <out>_{1,2,3}.json.gz '{}' twice
PYTHONPATH=../rsolver/python ../.venv/bin/python tools/m1_factspace.py --direct bench/oracle/m1_02w2/final
```

## 6. 실행 결과

- 실행: worktree `../rsolver-m1-02w-2`, 계약 commit `24bcb0f`, x86-64 Linux(WSL2), 바이너리 `8e87fca2…`. Gurobi는 M2-06h-7 세션의 긴 캡처와 잠금을 나눴다(첫 시도는 `gurun.sh` 1시간 대기 초과 99로 끝나 재시도 루프로 다시 실행).
- **저장 64모델**: `tools/m1_factspace.py --self-test` 64/64 정확 일치(z = rsolver `NumNZs`, k 5, c 0). 변이 거부: k 4·k 6·c 1·0.75 → 0.8 각 64, half 상한(⌊m/2⌋+2 → ⌊m/2⌋) 4(beaconfd·fit1d·forplan·grow7), 10n → 11n 1(forplan, 4,220 < 4,564 < 4,642). 지난 분기: m<n·m<500, m<n·m<3000, m<n half 상한, m≥n·n<500, m≥n half 상한, z > 10n.
- **직접 캡처**(`bench/oracle/m1_02w2/final`, 7사례 × 3회 = 21회): `tools/m1_factspace.py --direct` **DIRECT PASS**(afiro_k7의 k 7·1922와 grow15 reset 재풀이의 c 0·131,899는 검사기가 고정값으로도 확인). 모든 `045a76` 호출(24 = 단일 6사례 × 3 + grow15 2호출 × 3)에서 spx+0x28 = 0, rc 0, F+0x158 = 진입 필드로 계산한 식, F+0x168 = F+0x158 − 1, F+0x160 = 0, 다음 `036ade` 진입 F+0x158 같음, spx+0x3d0 = rsolver `NumNZs`. 반환 주소는 모두 0xd1cb3(`02f7e1` C:1228, `12f54b`가 tail jump라 그 호출 지점). 사례별 3회 events 동일.
- **판별 사례**: afiro `GURO_PAR_SIMPLEXALLOCS` = 7 → k 7 관측, F+0x158 **1922** = 사전 예측. grow15 `reset()` 뒤 재풀이는 `045a76`이 다시 불렸지만 c = **0**, F+0x158 131,899로 사전 예측(c 1, 197,849)과 **달랐다**.
- **c 탐색**(reset 1회·warm start 3회, `bench/oracle/m1_02w2/explore_c.js`·`explore_c_mod.js`, 자료 `explore/`): 첫 풀이 안에서 `036ade` 풀 부족 −1 뒤 F+0x170 = 1, `0cda5b` 반환 때 c = 1로 써진다. `reset()` 뒤 다음 `0cda5b` 진입에서 c = 0(reset이 지움). `reset()` 없이 목적 계수 하나를 1e-7 바꿔 다시 풀면 `0cda5b` 진입 c = 1, `045a76` 진입 c 1, F+0x158 **197,849** = 식(c 1)과 사전 예측 값. 그래서 c는 같은 모델의 이전 풀이에서 쌓이고 `reset()`이 0으로 돌린다. warm start 탐색은 3회(`explore/grow15_mod_c_{1,2,3}.json.gz`) events 동일, reset 탐색은 1회(같은 사실은 `final/grow15_twice_*` 3회가 c 0으로 확인).
- 실행하지 못한 것: 0회 분기(m ≥ 3000, m ≥ n이면서 n ≥ 500, m = 0, c ≥ 8 짝 mulpd, 할당 실패 재시도, quad `1dec07`, Presolve ≠ 0 차원), `14abfb` 쪽 c 쓰기(이번 실행에서 불리지 않음).
- 계약 변경: grow15 재풀이 사례의 c 판별을 `reset()` 사례(c 0 확인)와 warm start 탐색(c 1, 3회)으로 나눠 기록했다. 3회 기준 자료의 합격 조건은 바꾸지 않았다.

## 7. 판정과 인수인계

- 작업 수행: **완료(조사)**. 답: M1 dual의 초기 F+0x158은 `045a76`(LU 객체 할당, `02f7e1` C:1228 → `12f54b`)이 §2 식으로 쓰는 FactSpace다. 입력은 m·n·nnz(A)·`GURO_PAR_SIMPLEXALLOCS`(기본 5)·모델 카운터 c(이전 풀이의 F+0x170 최댓값, `reset()`이 0으로). 저장 64모델 정확 일치, 직접 캡처 7사례 3회 일치, k 판별 통과. 동등성 판정은 하지 않는다(조사).
- 후속 **M1-02w-3**(구현): Rust M1 재분해를 `Lu::factor_dense(cap)`로 바꾸고 cap = 이 식(c = 0 첫 풀이; warm start 재풀이는 c 수명이 필요하나 Rust에 아직 모델 카운터 없음)으로 시작, 풀 부족 −1 재호출(`crossover_refactor`) 뒤 F+0x158 갱신. 식은 `tools/m1_factspace.py`의 `factspace`를 Rust로 옮긴다. 대상·회귀는 [M1-02w §7](m1_02w_lu108.md#7-판정과-인수인계).
- branch `work/m1-02w-2`. 코드(`crates/`) 변경 없음. 새 도구 `tools/m1_factspace.js`·`tools/m1_factspace.py`, 자료 `bench/oracle/m1_02w2/`(`final/` 21, `explore/` 4, `runs.sh`).
- `docs/progress/M1-02w-2.md` 작성. 교훈: `docs/lessons/original.md`(FactSpace 식과 c 수명), `docs/lessons/tooling.md`(`long*` 첨자 쓰기 grep 함정, `gurun.sh` 1시간 포기).
