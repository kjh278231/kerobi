# M2 MIP 루트 LP 완화 작업 노트 (`5daa82` → `64da9f` → `5ce67b`)

MIP presolve(`3cd406`, G6) 다음 단계인 루트 LP 풀이를 옮기며 알아낸 사실. 범위: `Mip::new`(`5daa82`)에서 루트 노드 생성 → `64da9f` 안의 루트 LP 풀이까지. cut 루프, 루트 휴리스틱, propagate, 분기, B&B는 범위 밖이고 만난 곳은 "여기서 멈춤"으로 적었다. 브랜치 `m2-root`(worktree `../rsolver-m2-root`, `m2-mip-presolve` + `main` 병합).

## 0. 현재 상태 (2026-11-06)

| 기준 | 상태 |
|---|---|
| (1) miplib_small 38개, 기본값 + Threads=1, "Root relaxation" 줄 | **목적 38/38, 반복 17/38**. 열 형식 가중만 반영하면 12/38, 정수 열 crash 제외까지 반영하면 17/38(§8). 38에는 루트가 없는 enlight_hard(presolve가 전부 지움, 우리도 `None`)를 포함한다. 루트가 있는 37개만 세면 목적 37/37, 반복 16/37. 기준값 `bench/gurobi/<이름>/log.txt`는 Threads=1 로그다 |
| (2) 불일치 모델의 첫 분기점 | §4는 가중 반영 전 분류. 가중 반영 후 26개 불일치가 남는다. §7과 `bench/mip_root_weights.txt` 참조 |
| (3) clippy 0, `cargo test --release --workspace`, 이 노트 | 충족 |

코드: `crates/mip/src/root.rs`(`root_method` = `69f0bd` 규칙, `solve_root`), `Params`에 Method·Threads·ConcurrentMethod 추가, py `CoreModel.mip_root(params)`. 도구: `tools/mip_root_check.py`(Gurobi 불필요), `tools/mip_root_trace.js`(오라클).

## 1. 호출 구조 (Frida 트레이스, 37개 전부 같음)

```
5bbf7f(model, timer)
  5daa82(model, 0,0,0, &M, timer)          // Mip::new
    66bdf6 ...                              // domain store 생성(M+0x3298 → +0x10, lb +0xa0, ub +0xa8; n+m 칸)
    ... 5ff358 (루트 노드) ... 5ebfbc ...    // 5ebfbc: 해 제출(incumbent) → bound 조임
    64da9f(&root, timer)                    // 5daa82 L5507
      7892b1 → 69f0bd(M, ., &holder, ., .)  // LP holder: [0] M, [1] LP 모델(5ca999가 domain store bound로 만듦),
                                            //   +0x10 루트 method, +0x14 concurrent method, +0x18 노드 method
      5c97c5(로그 머리) ; 8ac5fe(루트 전 휴리스틱, 여기서 휴리스틱 LP를 5ce67b로 먼저 풀기도 함)
      5ce67b(&root, W, lp, ...)             // "Root relaxation: objective %.6e, %.0f iterations"
        5ccea5(...) → cda5b(lp, 1, vbasis=0, cbasis=0, 0,0,0,0, timer)   // 호출 지점 ra 0x4f9626
```

- **Method** (`69f0bd`, MILP): Method=-1이면 nnz(model+0xe8+0x10) ≥ 500000 && 행 > 999 && 열 > 19999 → 4(ConcurrentMethod 그대로), nnz ≥ 100000 && 행 > 999 && 열 > 9999 → 4(CM=3), 아니면 1. Threads=1이어도 자동 4는 내리지 않는다(명시 3–5만 Threads=1이면 1). NodeMethod(-1) → 1. 38개는 전부 1(트레이스의 `method` 인자로 확인). `root_method`에 옮기고 경계값 테스트를 붙였다.
- **시작 기저**: 37개 모두 루트 `cda5b` 진입 때 LP의 spx(model+0xe0)가 없고(`ws:false`) vbasis/cbasis도 0이다 → 새로 시작(crash). 휴리스틱 LP를 먼저 푼 모델(glass4 k0–2 등 11개)도 그 사이 spx가 해제되어 웜 스타트가 아니다.
- **5ccea5 분기**: method가 2 또는 3–5면 `98b671`(barrier/concurrent), RELAXPREP(0x2578)·PreDual에 따라 `c3d0d`(LP presolve 경로)도 있으나 38개에서는 안 탄다. 바로 `cda5b`.
- **루트 찾기**: 루트 전에 `5ce67b`가 휴리스틱 LP로 0–3번 먼저 불린다(같은 반환 주소). 루트 호출은 반환한 IterCount·ObjVal이 로그의 루트 줄과 같은 첫 `cda5b`로 찾았다(`rootk`).

## 2. 루트 LP = "presolved 모델 + domain bound + 열 형식"의 dual simplex

`mip_root_trace.js`로 루트 `cda5b` 진입 때 LP 모델을 `GRBwrite`, 환경을 `GRBwriteparams`했다. 같은 모델을 Gurobi LP로 따로 풀면(Presolve=0, Method=1, Threads=1, 같은 .prm) 반복 수가 대부분 다르다. 2f7e1(setup) 직후 spx를 비교하면 배열(스케일·비용·bound·rhs)은 비트까지 같고 차이는:

| spx | MIP 안 | LP 단독 | 의미 |
|---|---|---|---|
| +0xb8 (열 형식, n 바이트) | 'B'/'I'/'C' | 전부 'C' | `028c93` CHUZR 점수 정수 행 ×1.5, `048d5f` 섭동 w=1.5 (노트 13 §3.4.3) |
| +0x30 | 1 | 0 | env+0x2dc0 루트 로그 모드("Root simplex log...") |
| +0x08, +0x128 | 0, 0.0 | 1.5e-4, −0.0 | 의미 미확인 |

**기존 Gurobi 추적 결과**: LP 단독 실행에서 2f7e1 직후 spx+0xb8에 MIP의 열 형식을 써 넣으면(`PATCH_VT`) 37/37 모두 로그의 루트 반복 수와 같아진다(glass4 36 → 72). 즉 루트 LP 경로 = 그 모델·파라미터·열 형식의 dual simplex다. 후속 수정에서 `crates/lp`에도 열 형식을 보존하고 pricing, BFRT, GLOBAL 섭동 가중을 반영했다(§7). 이 수정만으로 rsolver의 전체 루트 경로가 같아지는 것은 아니다.

MIP LP 환경의 비기본 파라미터(.prm): Cutoff(루트 전 incumbent가 있으면), ObjScale −3(API로는 설정 불가 값), LPWarmStart 1, GURO_PAR_WARMBASIS 3, GURO_PAR_INTBASIS 1, GURO_PAR_UNSCALEDVIO 0, GURO_PAR_CUTOFFPIVOTS 0 등. 하나씩 빼 보면 반복 수를 바꾸는 것은 **INTBASIS=1**(ran14x18-disj-8 609↔622, cod105 3470↔3310)과 **UNSCALEDVIO=0**(gmu-35-50 482↔480)뿐이다. INTBASIS는 이름으로 보아 crash가 정수 열을 다루는 방식(시작 기저)으로 추정, 미확인. 둘 다 simplex 쪽 설정이라 G1에 넘긴다.

## 3. 입력 모델 차이: 루트 LP bound ≠ presolved bound (여기서 멈춤)

루트 LP 모델을 Gurobi의 `presolved.mps.gz`와 비교하면 행렬·목적·rhs·sense·objcon·하한은 37개 모두 같고, **상한만** 14개 모델에서 다르다(ran14x18-disj-8 1, gen-ip054 28, pk1 31, neos-911970 48, mas74 1, mas76 1, neos-860300 163, tr12-30 103, neos-4338804-snowy 21, neos-3046615-murg 25, mad 20, bppc4-08 1, glass4 10, supportcase26 40). LP는 `5ca999`가 domain store(M+0x3298→+0x10의 lb/ub 배열)로 만들므로, 그 배열의 한 열을 감시하며 `5daa82`·`64da9f`의 callee 진입/복귀를 추적했다(스크래치 `calltrace.js`, 아래 레시피):

| 바꾼 함수(또는 첫 관측 지점) | 예 |
|---|---|
| `66bdf6`(첫 관측: domain store 생성 때 이미 조인 값, 그 전 값은 못 읽음) | ran14x18 열 505: 179 → 18.000000000000004, neos-860300 열 1232: ∞ → 1, mas74 열 147 |
| `715361`(첫 관측), `670717`(5daa82 안) | tr12-30 열 453: presolved 14119, `715361` 뒤 처음 읽은 값 14099.000000000011, `670717`이 14099로 |
| `5ebfbc`(해 제출 → incumbent로 조이기) | gen-ip054 열 0: ∞ → 58, pk1 열 0: ∞ → 674.9999999324, mas74 열 147 |
| `8ac5fe`(루트 전 휴리스틱, 64da9f 안) | gen-ip054 열 0: 58 → 54, glass4 열 0: ∞ → 2433.3535… |

모두 propagate·휴리스틱 범위라 옮기지 않았다. `solve_root`는 presolved bound를 그대로 쓴다(`shortcut:` 주석). 목적값은 bound 차이와 관계없이 같다(37/37). 반복 수에는 3개 모델(gen-ip054 39/30, pk1 47/67, neos-860300 484/467: rsolver presolved/루트 bound)에서만 영향이 있다.

또 하나: `5ca999`는 slack 상한 ≤ 1e-10인 부등식 행(조건 `7889a5` 거짓)을 '='로 바꿔 LP에 싣는다. 37개 루트 LP에서는 sense 차이가 없었다(위 비교).

## 4. 38개 표

아래는 열 형식 가중을 반영하기 전 측정이다. 반영 후 결과는 `bench/mip_root_weights.txt`에 있다.

열: Gurobi 루트(로그) / 루트 LP 모델의 상한 차이 수 / Gurobi LP 단독(같은 모델, 파라미터 없음) / +MIP .prm / +.prm+열 형식(PATCH_VT) / rsolver(루트 LP 모델) / rsolver(presolved, 실제 `mip_root`) / 첫 분기점.

첫 분기점 판정 순서: rsolver가 맞으면 일치. 아니면 (A) bound 차이가 rsolver 반복 수를 바꾸면 입력 모델. (C) rsolver(루트 모델) ≠ Gurobi LP 단독이면 simplex 경로(G1: LP 자체 parity). (B1) LP 단독 ≠ +.prm이면 MIP LP 파라미터(INTBASIS/UNSCALEDVIO). (B2) 남은 것은 열 형식 가중(spx+0xb8). C로 분류된 모델 대부분도 B2 차이를 함께 갖는다(LP 단독 ≠ 로그).

| 모델 | 루트 | ub차 | LP | +prm | +prm+vt | rs(루트) | rs | 첫 분기점 |
|---|---|---|---|---|---|---|---|---|
| ran14x18-disj-8 | 762 | 1 | 609 | 622 | 762 | 583 | 583 | C simplex 경로 (B1 INTBASIS도) |
| gen-ip002 | 39 | 0 | 37 | 37 | 39 | 37 | 37 | B2 열 형식 |
| gen-ip054 | 31 | 28 | 30 | 30 | 31 | 30 | 39 | A 입력 bound |
| ic97_potential | 560 | 0 | 516 | 516 | 560 | 516 | 516 | B2 열 형식 |
| pk1 | 52 | 31 | 67 | 67 | 52 | 67 | 47 | A 입력 bound |
| neos859080 | 3 | 0 | 3 | 3 | 3 | 3 | 3 | 일치 |
| neos-911970 | 182 | 48 | 221 | 221 | 182 | 229 | 229 | C simplex 경로 |
| markshare2 | 23 | 0 | 19 | 19 | 23 | 19 | 19 | B2 열 형식 |
| mas74 | 53 | 1 | 44 | 44 | 53 | 53 | 53 | 일치(우연: Gurobi LP 단독은 44) |
| exp-1-500-5-5 | 621 | 0 | 621 | 621 | 621 | 621 | 621 | 일치 |
| markshare_4_0 | 13 | 0 | 12 | 12 | 13 | 12 | 12 | B2 열 형식 |
| mas76 | 47 | 1 | 55 | 55 | 47 | 54 | 54 | C simplex 경로 |
| neos-860300 | 487 | 163 | 484 | 484 | 487 | 467 | 484 | A 입력 bound |
| tr12-30 | 364 | 103 | 363 | 363 | 364 | 363 | 363 | B2 열 형식 |
| neos5 | 107 | 0 | 104 | 104 | 107 | 95 | 95 | C simplex 경로 |
| enlight_hard | – | – | – | – | – | – | – | 일치(루트 없음) |
| sp150x300d | 206 | 0 | 206 | 206 | 206 | 206 | 206 | 일치 |
| mik-250-20-75-4 | 81 | 0 | 82 | 82 | 81 | 82 | 82 | B2 열 형식 |
| neos17 | 574 | 0 | 545 | 545 | 574 | 557 | 557 | C simplex 경로 |
| neos-4954672-berkel | 405 | 0 | 385 | 385 | 405 | 398 | 398 | C simplex 경로 |
| neos-4338804-snowy | 399 | 21 | 400 | 400 | 399 | 400 | 400 | B2 열 형식 |
| neos-2657525-crna | 177 | 0 | 129 | 129 | 177 | 128 | 128 | C simplex 경로 |
| neos-3627168-kasai | 1253 | 0 | 1311 | 1311 | 1253 | 1360 | 1360 | C simplex 경로 |
| neos-3046615-murg | 87 | 25 | 87 | 87 | 87 | 87 | 87 | 일치 |
| neos-3754480-nidda | 201 | 0 | 197 | 197 | 201 | 167 | 167 | C simplex 경로 |
| csched008 | 563 | 0 | 493 | 493 | 563 | 487 | 487 | C simplex 경로 |
| mad | 34 | 20 | 9 | 9 | 34 | 9 | 9 | B2 열 형식 |
| bppc4-08 | 921 | 1 | 778 | 778 | 921 | 627 | 627 | C simplex 경로 |
| glass4 | 72 | 10 | 36 | 36 | 72 | 36 | 36 | B2 열 형식 |
| gmu-35-50 | 692 | 0 | 482 | 480 | 692 | 542 | 542 | C simplex 경로 (B1 UNSCALEDVIO도) |
| supportcase26 | 207 | 40 | 207 | 207 | 207 | 207 | 207 | 일치 |
| fhnw-binpack4-4 | 389 | 0 | 390 | 390 | 389 | 390 | 390 | B2 열 형식 |
| cod105 | 7428 | 0 | 3470 | 3310 | 7428 | 4277 | 4277 | C simplex 경로 (B1 INTBASIS도) |
| gmu-35-40 | 452 | 0 | 369 | 369 | 452 | 350 | 350 | C simplex 경로 |
| graphdraw-domain | 192 | 0 | 160 | 160 | 192 | 160 | 160 | B2 열 형식 |
| timtab1 | 141 | 0 | 68 | 68 | 141 | 68 | 68 | B2 열 형식 |
| csched007 | 988 | 0 | 727 | 727 | 988 | 784 | 784 | C simplex 경로 |
| assign1-5-8 | 415 | 0 | 326 | 326 | 415 | 363 | 363 | C simplex 경로 |

요약: 일치 7(우연 1 포함), A 3, B2 12, C 16. G1이 열 형식 가중(B2)을 넣으면 B2 12개는 rsolver(루트 모델) = LP 단독이므로 바로 맞을 가능성이 높다(tr12-30·snowy·glass4·mad처럼 bound 차이가 있어도 rsolver 반복 수가 안 바뀌는 모델 포함). C는 LP parity 문제다.

## 5. 다시 확인하는 법

```
tools/build_local.sh
PYTHONPATH=$PWD/python ../.venv/bin/python tools/mip_root_check.py            # 목적 38/38, 반복 17/38 (Gurobi 불필요)
tools/gurun.sh node tools/mip_root_trace.js <이름> <dir>                        # events.jsonl, <k>.mps, <k>.prm, load_<i>.mps, log.txt
SPX_K=<k> tools/gurun.sh node tools/mip_root_trace.js <이름> <dir>              # + spx_<k>.json (2f7e1 직후 spx, vtype 포함)
LP=<dir>/<k>.prm SPX_K=1 tools/gurun.sh node tools/mip_root_trace.js <dir>/<k>.mps <dir2>             # LP 단독
PATCH_VT=<vt 파일> LP=<dir>/<k>.prm SPX_K=1 tools/gurun.sh node tools/mip_root_trace.js <dir>/<k>.mps <dir3>  # + 열 형식
```
- 루트 k: events.jsonl에서 `cda5b` 복귀의 `iter`/`obj`가 log.txt 루트 줄과 같은 첫 호출. vt 파일 = `spx_<k>.json`의 `vtype[:n]` 바이트.
- bound를 바꾸는 함수 찾기: `5daa82`·`64da9f`의 callee(`re/decomp/callgraph.tsv`)를 전부 걸고, `5e6ed8` 진입에서 M을 잡아 `*(*(M+0x3298)+0x10)+0xa8`의 한 열을 진입/복귀마다 읽는다(이번 세션 스크래치 `calltrace.js`; 필요하면 `mip_root_trace.js` 틀로 다시 만든다).
- 결과 .mps는 정수 MARKER 없이 연속 LP로 쓰인다(열 형식은 spx에만 있다).

## 6. 남은 일

1. (G1) INTBASIS=1·UNSCALEDVIO=0의 효과 해독. 열 형식 보존과 pricing, BFRT, GLOBAL 섭동 가중은 반영했다. 등식 crash의 정수 열 제외까지 반영한 뒤 가중 전 B2 모델 중 남은 것은 mad(32/34)다. 동일 입력 재실행으로 simplex가 아닌 루트 전 bound 차이로 확정했다(§9).
2. (G1) LP 단독 parity: C 16개.
3. (propagate/휴리스틱 범위) 루트 전 domain store: `66bdf6`의 초기 조이기, `715361`/`670717`, `5ebfbc` incumbent 조이기, `8ac5fe` 루트 휴리스틱. 들어오면 `solve_root`가 domain bound를 받도록 바꾼다.
4. 자동 concurrent(4)·barrier 루트(5ccea5의 `98b671` 분기), RELAXPREP/PreDual로 가는 `c3d0d` 분기: 38개에 없어 미포팅.
5. 확인 못 한 것: spx+0x08·+0x128 차이의 의미(PATCH_VT만으로 37/37이 맞아 영향 없다고 봄), `66bdf6` 등이 조인 모든 열의 출처(열 하나씩만 추적), INTBASIS의 실제 의미, 루트 시간·work 단위.

## 7. 열 형식 가중 반영 (2026-11-06)

`m2-root`를 `m2-mip-presolve`에 fast-forward로 반영했다. `Spx`는 `Model.vtype`을 스케일링과 unscaled 재풀이에서도 보존한다. 정수·이진 열의 계수는 1.5, 연속 열과 기존 slack의 계수는 1.0이다. pricing의 모든 점수 계산은 비용 모드 보정 후 이 계수를 곱한다. BFRT 선택 가중과 GLOBAL 섭동은 열 길이에 `(계수 * 0.3 + 0.7)`을 곱한다. BFRT slack 가중 0.5와 섭동 slack 가중 1은 그대로다. 근거는 저장된 `15d2d`, `028c93`, `1c35c5`, `048d5f` 디컴파일이며 새 Gurobi 추적은 수행하지 않았다.

동일한 38개 모델을 수정 전후 각각 한 번씩 실행했다. 모든 루트는 status 2이고 로그의 표시 정밀도(목적값 `.6e`)에서 일치한다. `enlight_hard`는 양쪽 모두 루트가 없다. 반복 수는 7/38에서 12/38로 바뀌었다. 새로 일치한 모델은 gen-ip002, markshare2, tr12-30, mik-250-20-75-4, neos-4338804-snowy, fhnw-binpack4-4다. 기존에 우연히 일치했던 mas74는 53회에서 52회로 바뀌었다. 모델별 전후 수치는 `bench/mip_root_weights.txt`에 있다.

가중 전 B2로 분류했던 ic97_potential, markshare_4_0, mad, glass4, graphdraw-domain, timtab1은 여전히 다르다. 기존의 "가중만 넣으면 12개가 바로 맞을 가능성"은 검증 결과 성립하지 않았다. 나머지 simplex 경로, MIP LP 파라미터와 루트 전 bound 변경을 구현하기 전에는 전체 동작 일치를 선언할 수 없다.

회귀 검증은 Rust의 두 pricing 모드와 점수 갱신, GLOBAL 섭동의 난수 순서 및 연속 열 유지, BFRT 진입 변수 선택을 확인한다. `tests/test_mip_root.py`는 새로 일치한 6개 모델의 목적값·반복 수와 루트 없는 모델을 저장된 Gurobi 로그와 비교한다.

검증 결과: `cargo test --release --workspace` 61개 통과, `cargo clippy --release --workspace --all-targets -- -D warnings` 통과, `PYTHONPATH=$PWD/python ../.venv/bin/python -m pytest -q` 70개 통과·기존 xfail 1개. 연속변수 Netlib 64개도 `bench/lp_check.csv`의 method 1 기준 status·목적값·반복 수를 유지했다. 목적값 허용 오차는 `1e-12 * max(1, |obj|)`, 반복 수는 정확히 비교했다.


## 8. 등식 crash의 정수 열 제외 (2026-11-09)

`markshare_4_0`(Gurobi 13, rsolver 14)을 최소 추적 대상으로 골랐다. `mip_root_trace.js`에 `SPX_TRACE=1`을 추가해 LP 단독 실행의 SIMPLEX 콜백마다 `spx_oracle.dump`를 저장한다. 같은 반복의 마지막 콜백을 남긴다. MIP 루트 k=0에서 얻은 MPS·prm·열 형식을 LP 단독 실행에 넣으면 13회가 재현된다. LP 단독은 c3d0d(k=0) 안의 cda5b(k=1)에서 setup이 실행되므로 **SPX_K=1**이어야 열 형식 패치가 적용된다. SPX_K=0이면 패치되지 않아 12회다.

첫 분기는 반복 0의 기저다. Gurobi는 네 slack이 basic인데 rsolver는 정수 열 32를 넣고 마지막 slack을 fixed로 내렸다. 원본 `1e7531`(file offset 0xeaaa0) L396–397은 spx+0x4e8 > 0일 때 열 형식을 읽는다. L1075의 등식 pass 1은 최종 후보가 정수 열이면 선택을 버린다(다음 후보를 고르지 않음). L1126의 pass 2는 정수 열을 후보에서 제외한다. 기본 spx+0x4e8 = 1(`b08b12` L237). `start.rs`에 이 두 조건을 반영했다. 이 수정은 INTBASIS 파라미터 해독과 별개다.

수정 후 `markshare_4_0`의 반복 0–5는 vstat·xB·d·DSE w가 참조와 비트 동일하다. 반복 6부터 d는 일부 비트가 다르지만 마지막 반복 13까지 vstat·xB·w는 같다.

38개 전체 비교: 상태는 루트가 있는 37개 모두 2, 목적은 로그 표시 정밀도에서 38/38, 반복 수는 **12/38 → 17/38**. 기존 일치 모델은 유지됐고 다음 다섯 개가 새로 일치했다.

| 모델 | 수정 전 | 수정 후 | Gurobi |
|---|---:|---:|---:|
| ic97_potential | 505 | 560 | 560 |
| markshare_4_0 | 14 | 13 | 13 |
| glass4 | 36 | 72 | 72 |
| graphdraw-domain | 177 | 192 | 192 |
| timtab1 | 67 | 141 | 141 |

`mad`는 9 → 32로 바뀌었으나 참조는 34다. 반면 csched008은 467 → 1185(참조 563), csched007은 727 → 1230(참조 988)으로 멀어졌다. 전체 경로 일치를 뜻하지 않는다. 다음 추적은 남은 여섯 모델 중 유일하게 해결되지 않은 `mad`에서 입력 root bound를 고정하고 시작 기저와 첫 피벗 분기를 비교한다. 그 뒤 INTBASIS·UNSCALEDVIO 및 기존 C 분류 모델을 잇는다.

새로 일치한 다섯 모델을 `tests/test_mip_root.py`에 추가했다. 먼저 markshare_4_0의 14 != 13 실패를 확인한 뒤 수정했다. 원본 Gurobi 추적 파일은 `/tmp/root-markshare*`에 있으며 영구 fixture는 기존 `bench/gurobi/*/log.txt`를 사용한다.

검증: `cargo test --release --workspace` 통과(61개), clippy `-D warnings` 통과, 전체 pytest 71 passed·1 xfailed. 이후 추가한 네 모델까지 포함한 루트 테스트는 12 passed. `node --check tools/mip_root_trace.js`, `git diff --check` 통과.
연속변수 Netlib Method 1도 `bench/lp_check.csv` 기준 상태·목적·반복 수 64/64 유지.


## 9. mad 32/34 분리 검증 (2026-11-09)

`SPX_K=0 tools/gurun.sh node tools/mip_root_trace.js mad /tmp/root-mad`로 새 루트 입력을 캡처했다. k=0의 cda5b는 행 40, 열 220, 새 기저, 목적 0, 반복 34다. 원본 MIP의 presolved 모델과 루트 입력을 비교하면 행렬·목적·rhs·sense·하한·열 형식이 모두 같고 상한 20개만 다르다. 열 200–219가 전부 infinity(1e30)에서 6.9639999992036으로 바뀐다.

단일 변수 대조 실험(열 형식·MIP LP prm 동일, bound만 교체):

| 입력 | Gurobi LP 단독 | rsolver |
|---|---:|---:|
| 캡처된 루트 bound | 34 | 34 |
| presolve 직후 bound | 32 | 32 |

Gurobi LP 단독은 `SPX_K=1 PATCH_VT=<vtype.bin> LP=<0.prm> SPX_TRACE=1`로 실행했다. 루트 bound에서 반복 0–26의 vstat·xB·정규화 d·DSE w는 전부 비트 동일하다. 반복 27부터 일부 w의 끝 비트가 다르지만 반복 수는 34로 같다. 끝 반복의 basis refactor까지 비트 동일하다는 뜻은 아니다.

따라서 mad를 맞추기 위해 simplex를 더 바꾸면 안 된다. 루트 전 domain store/휴리스틱 bound 반영이 필요하다. 이번 단계에서는 상한을 하드코딩하거나 presolve 출력 자체를 바꾸지 않았다. 실제 `mip_root`의 전체 일치 수는 여전히 17/38이다. 다음 구현 대상은 G1의 다른 C 모델 또는 MIP의 루트 전 domain store이며, mad의 기존 원인 미분류 상태는 해결됐다.

영구 fixture: `bench/oracle/mip_root/mad/{0.mps.gz,0.prm,spx_0.json,events.jsonl}`. MPS·prm·이벤트는 실제 Gurobi 캡처, spx JSON은 크기와 열 형식 필드만 남겼다. 새 도구는 Gurobi 없이 캡처 MPS에 열 형식을 복구해 실행하고 상태·목적을 참조 이벤트와 비교한다. --presolved 옵션은 행렬·비용·rhs·sense 일치를 확인한 뒤 bound만 교체해 다시 실행한다.

```
PYTHONPATH=$PWD/python ../.venv/bin/python tools/mip_root_replay.py bench/oracle/mip_root/mad --presolved ../re/bench/data/miplib/mad.mps.gz
# captured bounds: iterations 34, Gurobi 34
# lb 변화 없음, ub 변화 열 200–219
# presolved bounds: status 2, iterations 32
```

`tests/test_mip_root.py::test_mad_captured_root_bounds_explain_remaining_iteration_difference`는 두 입력의 차이가 위 20개 상한뿐인지와 34/32회 재현을 함께 검사한다. 루트 테스트 전체 13 passed, git diff --check 통과. 재실행 도구는 현재 38개에서 관측한 dual simplex 경로용이며 MIP LP의 비공개 파라미터를 별도로 구현하지 않는다.

## 10. 루트 도메인과 incumbent 입력 (2026-11-09)

`mad` 열 200의 상한을 함수 진입·복귀에서 추적하면 `66bdf6` 직후에는 1e30이고, `5ebfbc`의 해 제출 중 `6f2e4a` → `5e73b9` → `6c5042`를 거쳐 6.9639999992036이 된다. `6f45ed`는 incumbent 목적에서 `(abs(obj)+1)*1e-10`을 빼 개선 cutoff를 만든다. 프리솔브 전 로그의 휴리스틱 해는 정수 열 여러 개가 1인 해다. 이 해의 생성 휴리스틱과 원본 → presolved 해 변환은 아직 포팅하지 않았다.

`root_domain`은 별도 모델에 `66bdf6`의 등식 기반 암시적 정수 판정과 경계 반올림을 적용한다. `5ca999`는 도메인 형식(+0x650)이 아닌 별도 LP 형식(+0x660)을 사용하므로 LP의 열 형식은 원래 형식으로 유지한다. 선택 입력 incumbent는 **presolved 열 순서의 완전한 해**여야 하며, 각 열 경계·정수성·모든 행을 검사한 뒤 목적식의 나머지 열에 대한 최솟값으로 경계를 조인다. 목적 방향과 상수, 무한 경계, 정수 반올림을 처리한다. 누적 오차 여유를 두는 일반 구현이며, 원본 `6c5042`의 모든 사전 계산 자료구조와 후속 행 전파를 그대로 이식한 것은 아니다. 잘못된 해는 오류로 거부한다. 개선 cutoff가 모순을 보이는 경우 해당 경계는 적용하지 않으므로 최적성 판정이나 종료 처리는 제공하지 않는다. 부등식 기반 암시적 정수 판정 및 추가 전파도 남았다.

Python에서 `presolved.mip_root(params, incumbent)`로 해를 전달하고 `mip_root_bounds(params, incumbent)`로 적용 경계를 확인할 수 있다. 캡처 해 `bench/oracle/mip_root/mad/incumbent.json`은 테스트에만 쓰며 실행 경로는 fixture나 Gurobi 로그를 읽지 않는다. 해당 해를 검증해 입력하면 원본 루트 MPS와 전체 하한·상한이 비트 단위로 같고 루트 34회가 나온다. 해를 생략하면 32회다. 따라서 자동 휴리스틱 없이 기본 mad가 34회가 된 것은 아니다.

38개 기본 비교는 목적 38/38, 반복 17/38을 유지한다. 저장된 presolved MPS 비교는 102/102 일치했다. `tests/test_mip_root.py`는 mad 전체 bound, 반복 수와 잘못된 incumbent 거부를 검사한다.

검증: Rust workspace 테스트·clippy 통과, 전체 pytest 77 passed·1 xfailed, 루트 pytest 14 passed. 별도 API 검사에서도 mad 전체 bound·34회·presolved 입력 불변·잘못된 길이/NaN/무한 incumbent 거부 및 해 생략 시 32회를 확인했다. 구현은 사용자 요청에 따라 Terra 하위 에이전트가 수행했다.

## 11. 자동 초기 incumbent (2026-11-09)

`solve_root`와 `root_domain`은 incumbent 입력이 없으면 초기 휴리스틱을 실행한다. `heur.rs`는 경계 기반 후보 세 개(하한 우선, 상한 우선, 행 lock 방향)를 구성하고, 실행 가능한 후보의 목적값을 개선한다. lock 후보가 실행 불가능하고 `Heuristics > 0.005`이면 행 수리를 시도한다. 수리는 singleton 열을 먼저 사용하고, 나머지 후보를 기존 `Rng`로 섞는다. 열 이동은 위반 행 개수와 가중 위반량으로 선택하며, 남은 위반 행의 가중치를 두 배로 높인다. 최대 200패스와 모델 크기에 따른 연산 예산으로 탐색을 제한한다.

구현의 근거는 초기 휴리스틱 `914d54`의 경계 후보, `917e19`의 선형 목적 개선, `919394`의 선형 행 수리다. 이번 구현은 **presolved 좌표에서 동작하는 부분 이식**이다. 원본의 presolve 전 실행 위치, 원모델 해 변환, 전체 RNG 소비 순서, 재시작·섭동, 특수 변수·비선형·sub-MIP fallback은 이식하지 않았다. 작은 모델도 수리를 시도하고 패스별 slack을 다시 계산하므로 원본의 제어 흐름·수치 궤적 전체를 재현하지 않는다. `Heuristics`는 현재 활성화와 수리 진입 조건에 사용하며 전체 스케줄러의 시간 비율은 구현하지 않는다.

모든 후보는 초기 도메인의 열 경계·정수성·행 제약·유한 목적값 검사를 통과해야 incumbent가 된다. 개선 중 검사를 통과하지 못하면 개선 전 후보를 유지한다. 해를 찾지 못하면 기존 루트 LP 경로로 진행한다. 입력 모델은 변경하지 않는다. `RootLp.incumbent`는 검증된 해를 LP relaxation 해와 별도로 보관한다. 외부 incumbent 입력이 있으면 이를 우선 검증·적용하고 자동 탐색을 생략한다. `Heuristics=0` 또는 `GURO_PAR_PREPREHEUR=0`은 자동 탐색만 끈다. semi 변수 모델은 자동 탐색 대상에서 제외한다.

Python API는 다음과 같다. 모든 해는 presolved 열 순서이며 목적값은 모델의 원래 목적 방향과 상수를 사용한다.

```python
params = [('Threads', 1.0)]
pre = model.presolve(params)
solution = pre.mip_initial_incumbent(params)  # (objective, x), 실패하면 None
root = pre.mip_root(params)                 # 자동 탐색 후 incumbent 경계 반영
lb, ub = pre.mip_root_bounds(params)         # 같은 결정적 탐색의 적용 경계
baseline = pre.mip_root(params + [('Heuristics', 0.0)])
```

`mad`는 fixture 입력 없이 목적값 **5.9138**의 실행 가능한 해를 찾는다. 이 해를 자동 반영하면 루트 반복 수는 **34**, 비활성화하면 **32**다. 캡처 해의 목적값 6.964와 다르고 적용 상한도 다르므로 반복 수 일치는 원본 휴리스틱과의 전체 패리티를 뜻하지 않는다. 기존 캡처 해를 명시적으로 전달하는 경로는 전체 bound 비트 일치와 34회를 유지한다.

`PYTHONPATH=python ../.venv/bin/python tools/mip_root_check.py`로 비교한 38개 모델은 상태·루트 목적을 유지했다. 로그 정밀도의 목적값 일치는 38/38, 반복 수 일치는 17/38에서 20/38로 바뀌었다. 자동 휴리스틱의 성공 여부나 해 발견 순서가 원본과 같다는 합격 기준은 아니다.

38개 모델의 휴리스틱만 별도로 실행해 **19개**에서 incumbent를 얻었다. 이 19개 해는 Python에서 경계·정수성·원래 sense의 모든 행 활동값을 독립적으로 계산해 다시 검증했다. 전체 열이 제거된 모델도 분모 38에 포함한다.

검증: Rust workspace 68개 테스트와 clippy `-D warnings` 통과, 전체 pytest 78 passed·1 xfailed, 루트 pytest 15 passed. 추가한 Rust 테스트는 혼합 모델 수리·자동 경계 반영, 비활성화·명시적 incumbent 우선 적용, 최대화·고정 정수·음수 계수, 등식·불가능한 정수해 거부, 무한 경계를 검사한다.

## 12. 초기 수리의 재시작 (2026-11-09)

행 수리가 정체됐을 때 종료하던 경로에 `919394`의 기준 후보 복원 동작을 추가했다. 이동이 없고 후보가 없거나, 이동 없는 패스가 세 번 연속 발생하면 일부 이진 열을 기준 후보값으로 되돌린다. 첫 패스에서 이 경로에 들어가면 선택한 기준값을 `1 - x`로 뒤집는다. 이후 패스에서는 기준값을 복원한다. 선택 횟수는 원본의 이진 열 개수·패스 비율 식을 사용하며, 기존 `Rng` 스트림에서 열 인덱스를 뽑는다. `[0, 1]`의 일반 정수 열도 대상이다.

고정 변수는 뒤집거나 복원하지 않는다. 재시작 후에는 행 slack을 다시 계산한다. 탐색은 기존 200패스·연산 예산 안에서만 계속하며, 최종 incumbent 채택 전 검증도 유지한다. 전체 원본 제어 흐름과 work 계산을 그대로 포팅한 것은 아니다. 원본의 실패 행 우선순위, 목적 cutoff 위반 수리, 원모델 해 변환, 특수 변수·비선형·sub-MIP 경로는 남아 있다.

38개 모델 중 incumbent를 찾는 모델은 **19개에서 22개**로 늘었다. 기존 19개 해는 목적값을 유지했고, 새로 `tr12-30`(194146), `sp150x300d`(108), `neos-3627168-kasai`(1135945.8500000015)에서 해를 찾았다. 22개 해 모두 Python에서 경계·정수성·모든 행 제약을 독립 재검증했다. `mad`의 휴리스틱 목적 5.9138도 유지한다.

재시작이 있어야 실행 가능한 해를 찾는 4변수 모델을 Rust 회귀 테스트로 추가했다. 재시작 호출만 비활성화하면 테스트가 실패하고, 복원하면 통과한다. 별도 테스트는 첫 패스 뒤집기·후속 복원, 고정 이진 열과 일반 정수 열의 보존을 검사한다.

검증: Rust workspace 71개 테스트와 clippy `-D warnings` 통과, 전체 pytest 78 passed·1 xfailed, `git diff --check` 통과. `tools/mip_root_check.py`의 루트 목적값 38/38·반복 수 20/38 일치를 유지했다.

## 13. 재시작 직후 실패 행 우선 처리 (2026-11-09)

`919394`의 행 대기열 구성 경로를 추가했다. 수리 루프가 마지막으로 처리한 위반 행을 기억하고, 재시작 직후 다음 패스에서도 그 행이 위반 상태이면 대기열 선두와 교환한다. 원본의 `bVar6 && 0 < last_row` 조건대로 행 0은 교환 대상에서 제외한다. 재시작이 없는 패스에서는 원래 행 순서를 유지한다. 마지막 처리 행과 재시작 여부 두 상태만 추가했다. 새로운 RNG 호출은 추가하지 않았으며 기존 난수 생성기를 사용한다. 행 순서가 바뀌면 이후 난수 소비 순서와 횟수는 달라질 수 있다. 열 이동 규칙·허용오차·예산·incumbent 검증은 유지한다.

이 우선순위가 없으면 해를 찾지 못하는 6변수 이진 모델을 Rust 회귀 테스트로 추가했다. 우선순위 교환만 비활성화했을 때 테스트 실패를 확인했고, 복원 후 통과했다. 최종 실행 가능성과 목적값 -3, 반복 호출의 결정성을 검사한다. 행 순서 변경으로 다른 모델의 탐색 결과도 바뀔 수 있으며, 모든 모델에서 성공률 개선을 보장하지 않는다.

MIPLIB 38개에서는 기존 22개 incumbent와 목적값을 모두 유지했다. 22개 해의 경계·정수성·모든 행 제약을 Python에서 독립 재검증했다. 원모델 해 변환, 목적 cutoff 위반 수리, 특수 변수·비선형·sub-MIP 경로와 전체 work 패리티는 여전히 남아 있다.

검증: Rust workspace 72개 테스트와 clippy `-D warnings` 통과, 복원된 최종 소스로 Python 확장을 재빌드한 뒤 전체 pytest 78 passed·1 xfailed, `git diff --check` 통과. `tools/mip_root_check.py`의 38개 출력은 직전 단계와 동일했고, 루트 목적값 38/38·반복 수 20/38 일치를 유지했다.

## 14. incumbent 없음에 대한 경계 반전 fallback (2026-11-09)

먼저 `919394`의 목적값 수리 조건을 확인했다. 현재 초기 휴리스틱의 유일한 호출 `914d54`는 `0x54b249ad2594c37d`(1e100)를 전달한다. 실제 기계어도 목적값을 `.rodata`의 고정 1e100과 비교한다. 예를 들어 file+0x74b9ea는 주소 0x2a7d360의 값을 사용한다. 따라서 이 경로는 기존 incumbent의 유한 cutoff를 넘은 후보를 개선하는 일반 수리와 같지 않다. 이번 단계에서는 이 상수를 `best_obj`로 치환하지 않았다. incumbent pool을 개선하는 `95556f` 호출은 별도 경로다.

대신 `914d54` 후반의 incumbent 없음 전용 경계 반전 패스를 추가했다. 기존 후보 세 개와 행 수리가 모두 실패했을 때만 진입한다. 하한에서 시작하되 절댓값이 1e6을 넘으면 0 또는 절댓값이 작은 경계를 고른다. 상한이 0.5를 넘는 첫 이진 열에는 1을 넣는다. 최대 두 번 열을 순회하며 `ub < 1e6`, `lb > -1e6`인 반대 경계를 시도한다. 이진 열을 0에서 1로 올렸는데 여전히 위반 행이 남으면 같은 열을 즉시 다시 방문해 내린다. `[0, 1]` 일반 정수 열은 이진 열로 취급하지 않는다. 원본의 연속·이진 열 블록 위치 대신 실제 `vtype`으로 이를 구분한다.

이 패스는 RNG를 호출하지 않는다. 두 번의 순회와 모델 크기에 따른 연산 예산으로 제한하며, 최종 후보는 기존 경계·정수성·행 제약·유한 목적 검증과 목적 개선을 거친다. 입력 모델과 기존 incumbent를 바꾸지 않는다. 초기 seed가 이미 실행 가능하면 즉시 검사 경로로 보내는 점, 기존 원본 모델 대신 presolved 좌표에서 실행하는 점은 원본 전체 제어 흐름과 다르다.

주 수리가 놓치는 4변수 이진 등식 모델을 행동 회귀 테스트로 추가했다. fallback 호출만 비활성화하면 실패하고, 복원하면 최소화·최대화와 목적 상수 4.25를 포함해 통과한다. 낮은 `Heuristics`에서도 이 패스를 실행하며, 0일 때는 기존 비활성화 규칙을 따른다. 별도 테스트는 고정·자유 열, 무한 경계, 1e6의 엄격 비교와 `[0, 1]` 일반 정수 열 구분을 검사한다.

검증: Rust workspace 74개 테스트와 clippy `-D warnings` 통과, mutation 소스를 복원한 뒤 최종 Python 확장을 재빌드하고 전체 pytest 78 passed·1 xfailed, `git diff --check` 통과. MIPLIB 38개에서는 incumbent 22개와 기존 목적값을 모두 유지했고, 22개 해를 Python으로 독립 검증했다. `tools/mip_root_check.py`의 38개 출력도 직전 단계와 동일하며 루트 목적값 38/38·반복 수 20/38 일치를 유지했다.

## 15. 이진 타입 도메인과 원래 모델 해 검증 (2026-11-09)

`B` 열에 느슨한 모델 경계 `[0, 2]`나 `[-1, 1]`를 주면 정수값 2·-1을 incumbent로 허용하고, 루트 LP에도 해당 경계를 넘기던 문제를 수정했다. 기존 검증기는 정수성만 검사했고, 휴리스틱 내부에서만 이진 경계를 좁혀 LP 도메인에는 반영하지 못했다. 회귀 테스트에서 잘못된 incumbent 채택과 목적값 -2(기대 -1)를 먼저 확인했다.

이제 공통 도메인 초기화가 `B` 경계를 `[0, 1]`과 교차한다. 휴리스틱의 중복 clip은 제거했다. 원본 `a91edf`처럼 타입 경계를 포함한 bound 위반에는 `FeasibilityTol`, `a9323a`처럼 정수성 위반에는 `IntFeasTol`을 따로 적용한다. 일반 정수 `I`의 값 2는 계속 허용한다. 이 단계에서 일반 정수 경계 반올림이나 전체 정수 infeasibility 판정을 추가하지 않았다.

정확한 교집합이 비어도 0 또는 1이 원래 두 경계를 `FeasibilityTol` 안에서 만족하면 해당 끝점으로 고정한다. 예를 들어 `[1 + 0.5·FeasibilityTol, 2]`의 1과 `[-1, -0.5·FeasibilityTol]`의 0을 허용한다. `IntFeasTol`만큼 모델 경계를 넓히는 방식은 쓰지 않는다. 명시적 해는 정규화 전 모델에서도 검사해 이 끝점 처리로 bound 허용오차가 두 번 적용되지 않도록 한다.

자동 해도 `validated_initial_incumbent`에서 원래 presolved 모델의 bound·행·타입·유한 목적값 검사를 통과해야 채택한다. 암시적 정수 추론이 원래 `C` 열의 상한 0.999995를 1로 넓혀 만든 후보 1은 원래 bound 검사를 통과하지 못하므로 버린다. 자동 후보가 탈락하면 오류를 내지 않고 incumbent 없이 루트 LP로 진행한다. 이 원래 모델 재검증만 끄면 해당 회귀 테스트가 실패함을 확인했다.

허용되는 이진 끝점도 없는 빈 도메인은 `root_domain`과 Python `mip_root_bounds`에서 오류로 알린다. `solve_root`와 `mip_root`는 simplex를 호출하지 않고 `Infeasible`(3), 반복 0을 반환한다. 이 조기 반환은 목적값을 NaN, 해·기저·trace를 빈 배열, incumbent를 None으로 두어 실행 가능한 해처럼 표시하지 않는다. 이때 명시적 incumbent를 주면 검증 오류로 거부한다. 휴리스틱 활성화 여부와 관계없이 같은 규칙을 적용하며 입력 모델은 변경하지 않는다.

최종 검증: Rust workspace 테스트 80개와 clippy `--workspace --all-targets -- -D warnings` 통과. 최종 소스로 `maturin develop --release`를 실행한 뒤 전체 pytest는 84 passed·1 xfailed다. MIPLIB 38개 중 incumbent 22개를 얻었고, Python에서 유한값·경계·이진 타입·정수성·모든 행 활동값·목적식을 독립 검증했다. 22개 모두 §14 기준 목적값을 유지했다. `tools/mip_root_check.py`는 목적값 38/38, 반복 수 20/38 일치를 유지한다.

## 16. 선형 수리 작업량과 중단 경계 (2026-11-09)

`914d54`는 `919394`에 작업량 한도 `100*nnz+100000`과 최대 200패스를 전달한다. `919394`의 `ade736`은 초기 slack·목적·위반 수 계산이 끝난 후 전역 work를 타이머 기준으로 저장하며, `adf1a6`가 한도를 설정한다. `adf2be`는 양수 한도에서 `(현재 work - 시작 work + 1) / 한도 >= 1`을 검사한다. 음수 한도는 즉시 중단, 0은 무제한이지만 이 호출의 한도는 항상 양수다. `driver::Timer`를 재사용해 초기화 work 뒤의 기준점과 double 비교를 적용한다. 휴리스틱의 work는 아직 전역 카운터와 공유하지 않으며 내부 누적은 정수이므로 매우 큰 work의 누적 반올림은 원본과 다를 수 있다.

검사 위치를 후보 내부에서 패스 시작, 대기열 각 행의 시작, 재시작 직전으로 옮겼다. 한 행의 후보 처리 도중 한도를 넘더라도 그 행은 끝내고 다음 행 전에 중단한다. 선형 분기의 작업량에 원본의 행 스캔 `3*m`, 후보 수집 `8*row_len`, shuffle `2*횟수`, 후보 평가의 열 스캔 `5*col_len`과 후보당 9, 수락 갱신 `5*col_len+8`, 처리 행당 5, 가중치·위반 수 스캔 `4*m`, 재시작 샘플링과 slack·목적·위반 수 재계산을 대응시켰다. 행 처리 5는 원본처럼 대기열 처리가 끝난 뒤 누적하고, 행 경계에서 예산을 넘더라도 처리 행 비용과 가중치·위반 수 스캔을 마친다. 불필요한 매 패스 slack 재계산은 제거했다. 첫 성공 샘플에서 빠져나갈 때 그 샘플에 대한 3은 원본처럼 더하지 않는다.

3개 이진 열과 두 위반 행의 내부 회귀 테스트는 작은 한도 20에서 첫 행의 두 후보를 모두 적용하고 다음 행 전에 중단하는지 검사한다. 같은 모델·초기 후보·seed·한도를 Gurobi 13.0의 `919394`에 직접 전달한 `tools/heur_repair_oracle.py`도 `[1,1,0]`과 위반 행 1개를 반환한다. 전역 초기 work 0과 1000에서 각각 117과 1117이 되어 시작점 차감도 확인했다. Rust 회귀도 초기화 비용 28, 행 처리 76, 후처리 13을 포함해 두 값에 정확히 일치한다. 재현 도구는 확인한 13.0.3 라이브러리 SHA-256을 검사한 후에만 고정 주소를 호출한다. `tools/gurun.sh ../.venv/bin/python tools/heur_repair_oracle.py`로 재현한다. 원본의 `local_d8` 최저 위반 수와 목적값 1e100 초과 수리, 전역 work 공유, 원모델 좌표 및 전체 RNG·후보 상태는 아직 완전히 복원하지 않았으므로 전체 work 또는 탐색 패리티를 주장하지 않는다. 특히 원본과 같은 후보 입력·난수 상태를 확보하는 내부 호출 캡처는 후속 작업이다.

MIPLIB 38개에서 incumbent는 22개에서 20개로 줄었다. `sp150x300d`는 패스 58에서 work 184727 / 한도 184400, `neos-3627168-kasai`는 패스 25에서 424917 / 423400에 도달해 중단했다(임시 계측 후 제거). 나머지 20개 해는 Python에서 유한성·경계·타입·정수성·모든 행·목적식을 독립 검증했고 이전 목적값을 유지했다. 이 결과는 수정된 부분 구현의 동작이며 원본이 두 해를 놓친다는 증거는 아니다. 루트 비교는 목적 38/38, 반복 20/38을 유지했다.

검증: 최종 Rust workspace 81개 테스트 및 clippy `--workspace --all-targets -- -D warnings` 통과, release Python 확장 재빌드 후 루트 pytest 21 passed, 전체 pytest 84 passed·1 xfailed. 네이티브 fixture 두 초기 work 값과 `git diff --check`도 통과했다.

## 17. 동일 입력의 원본 수리 재생과 정수 변수 수 (2026-11-09)

§16에서 incumbent를 놓친 `sp150x300d`, `neos-3627168-kasai`의 Rust 수리 진입 직전 모델·후보·RNG 상태를 임시 계측으로 캡처했다. 두 모델 모두 정규화 후 목적 방향은 최소화이고, 이진·연속 열만 포함한다. 후보와 reference는 같은 배열로 시작하고 RNG의 실제 64비트 상태는 20, 초기 work는 0, 한도는 각각 `100*nnz+100000`, 최대 패스는 200이다. 임시 계측은 소스에서 제거했다.

`bench/oracle/mip_repair/*.json.gz`에 입력과 양쪽 결과를 보존했다. `tools/gurun.sh ../.venv/bin/python tools/heur_repair_replay.py`는 Gurobi 13.0.3 라이브러리의 SHA-256을 확인한 뒤 동일 입력을 원본 `919394`에 전달한다. MPS를 사용하지 않는다. 공개 API가 열 순서를 정렬하므로 캡처 순서를 복원하고, 일부 하한 `-0.0`을 `+0.0`으로 바꾸므로 하한 비트도 복원한다. 열·행 계수와 순서, 목적·상수·방향·행 우변·sense·타입·경계를 `struct.pack`으로 비트 검사하며, 원본이 생성한 CSR도 검사한다. 후보·reference·경계는 캡처 숫자를 직접 double 배열로 전달하고 RNG는 Python 정수에서 64비트로 전달한다.

| 모델 | 남은 위반 행 | 최종 work | 최종 RNG | 후보 목적 |
|---|---:|---:|---:|---:|
| sp150x300d | 2 | 190286 | 7358319300641140486 | 300 |
| neos-3627168-kasai | 14 | 446603 | 3766084259005654666 | 1116984.38 |

두 사례 모두 최종 후보와 reference의 모든 double 비트, 위반 수·work·RNG가 원본과 Rust에서 일치한다. 원본도 이 **동일한 presolved 좌표 수리 호출**에서는 해를 찾지 못하므로 §16의 22→20 감소를 원본과의 차이로 간주할 근거는 없다. 이는 원본 전체 휴리스틱이 원모델에서 같은 결과를 낸다는 뜻도, 모든 중간 패스가 같다는 증거도 아니다. 패스별 첫 분기는 캡처하지 않았다.

별도로 원본 `919394`의 재시작 횟수 식에서 모델 `+0x2c`는 이진 수가 아닌 전체 정수 수다. B/I/C 각 한 열인 Gurobi 모델의 `+0x2c=2`, `+0x30=1`을 직접 확인했다. `perturb`가 B만 세던 오류를 B와 I를 합산하도록 수정했다. 16개 `[0,1]` 일반 정수의 마지막 패스에서 두 열을 복원하고 두 RNG 호출 후 상태를 확인하는 회귀를 추가했으며, B 16개에서도 같은 결과를 확인했다. 두 MIPLIB 캡처는 I가 없어 이 수정과 무관하다.

검증: Rust workspace 테스트 82개와 clippy `--workspace --all-targets -- -D warnings` 통과, 임시 계측을 제거한 최종 소스로 release Python 확장 재빌드 후 전체 pytest 84 passed·1 xfailed. MIPLIB 38개 중 20개 incumbent는 Python으로 유한값·경계·타입·정수성·모든 행·목적을 독립 검증했고 이전 목적값을 유지했다. 루트 비교는 목적 38/38, 반복 20/38이다. 원본 두 모델 재생과 혼합 타입 개수 검사, `git diff --check`도 통과했다.

## 18. 원모델 초기 해와 presolve 뒤 root 연결 (2026-11-09)

원본 `c23877`은 `914d54` 초기 휴리스틱을 presolve 전에 실행한다. 기존 `mip_root`는 presolved 모델 좌표에서만 초기 해를 찾으므로, 원모델 좌표의 해와 원래 목적을 보관하면서 presolve 결과에는 그 해의 최소화형 목적 상한만 전달하는 별도 원샷 경로를 추가했다. 원모델 x를 presolved x로 간주하거나 외부에서 스칼라 상한을 입력받지 않는다. presolve는 기존 `crate::presolve`를 그대로 호출하며 cutoff를 presolve State에 넣지 않는다. 결과 P0 자체는 수정하지 않고 clone한 root 도메인에만 상한을 반영한다.

Rust `root::solve_root_from_original`은 `PresolveRootRun { original_incumbent, presolve }`을 반환한다. 원래 해는 원모델 열 순서와 원래 방향의 목적값이다. presolve 성공은 수정되지 않은 P0, 실제 root LP bound, `Option<RootLp>`를 반환하고, 실패는 정확한 `Presolve(status)`를 반환하면서 이미 검증된 원래 해를 유지한다. 원래 해가 없는 경우에도 P0 초기 휴리스틱을 다시 호출하지 않는다. 내부 `VerifiedUpperBound`는 입력 모델에서 검증한 해의 최소화형 목적만 담으며, 기존 standalone 해 기반 경로와 목적 조임 및 LP dispatch를 공유한다. `root.incumbent`는 이 원샷 경로에서는 None이다.

Python Core API 사용법:

```python
incumbent, (status, p0, root, bounds) = raw.mip_presolve_root([('Threads', 1.0)])
# incumbent: None 또는 (원래 목적값, 원모델 순서 x)
# status 0: p0는 CoreModel, bounds는 P0 순서 (lb, ub), root는 LP 요약 또는 None
# presolve 실패: 원래 해는 유지하고 (status, None, None, None)
```

지원 범위는 선형 C/B/I MIP와 열이 없는 모델이다. 비어 있지 않은 순수 C 모델은 기존 `CoreModel.presolve`가 별도 LP 백엔드를 사용하므로 이 MIP 전용 경로에서 `NotImplementedError`다. S/N 등 미지원 타입도 `NotImplementedError`이며 가짜 수치 status를 만들지 않는다. 열이 없는 모델도 기존 MIP presolve를 거쳐 모순 상수 행은 status 3을 유지한다. 성공적으로 열이 모두 제거되면 P0와 빈 bound `([], [])`, root None을 반환한다. 비어 있지 않은 빈 도메인은 기존 root 규칙대로 status 3, 반복 0, NaN 목적을 반환한다. `Heuristics=0` 또는 `GURO_PAR_PREPREHEUR=0`이면 원모델 휴리스틱 해와 상한이 없다. 기존 standalone `mip_root`, `mip_root_bounds`, `mip_initial_incumbent`, `Model.presolve`는 그대로다.

개선 cutoff는 기존과 같은 `U-(abs(U)+1)*1e-10`과 정수 반올림·수치 여유·모순 조임 생략을 사용한다. 최대화 목적과 상수를 포함한 U는 원모델 검증 시 최소화 척도로 한 번 계산한다. presolve 고정열 기여는 P0 목적 상수에 이미 포함되며 중복 가산하지 않는다. 이 경로는 tolerance 기반 해 검증과 목적 동등한 presolve를 전제로 하는 부분 이식이다. 개선 cutoff는 원래 feasible 집합 전체의 보존을 보장하지 않으며 엄밀한 유리수 feasibility 증명, 원본의 전체 cutoff 전파, 해 crush, 전체 MIP 종료·최적성 판정을 제공하지 않는다. presolve 자체는 여전히 원본의 incumbent-aware MIP context를 사용하지 않으며, root의 후속 전파와 컷도 범위 밖이다.

원본 근거와 구현 선택은 원모델 해를 독립 보관하고 presolve 실패를 별도 표현하면서 P0와 실제 bound를 돌려주는 설계에 따른다. 원본 `6f45ed`의 목적 격자·사용자 cutoff 및 `6c5042`의 전체 구조는 구현하지 않았다. 원모델에서 검증한 해가 수치 허용오차 안에서만 실행 가능할 수 있고 presolve의 수치 변환·dual reduction과의 완전한 목적 동등성은 모든 모델에 대해 증명되지 않았다.

실제 검증: raw `mad`에서 목적 6.964와 저장된 원모델 해 전체를 생성했고, P0는 standalone presolve MPS와 일치하며 실제 root 전체 bound는 캡처와 같고 root는 34회다. 비활성화 시 32회다. 최대화·상수 7 모델에서 고정열 삭제 후 원래 3열→P0 2열, 상수 10, 원래 해 목적 12.5 및 root 연속열 하한 0→0.50000000135를 확인해 목적 방향·상수 기여와 좌표 축소를 함께 검증했다. 원모델에서 해가 없지만 P0 휴리스틱은 해를 찾는 `neos-4338804-snowy`에서 원샷 bound는 P0 휴리스틱 비활성화 결과와 같고 P0 기본 결과와 달라 재탐색하지 않음을 확인했다. presolve 3/5, 5에서 원래 해 유지, 빈 모델·빈 도메인, 미지원 타입과 입력 불변도 회귀로 검사했다.

최종 검증: Rust workspace 86개 테스트와 clippy `--workspace --all-targets -- -D warnings` 통과, 최종 소스 release 확장 재빌드, 전체 pytest 87 passed·1 xfailed 및 최종 재빌드 후 root pytest 24 passed. MIPLIB 38개 **원모델 좌표** 자동 해 20개를 유한성·경계·타입·정수성·원래 행 전체·목적식으로 독립 검증했다. 새 원샷 경로는 38개 전부 presolve 성공, 각 P0가 별도 presolve MPS와 일치하며 root 목적 38/38·반복 20/38이었다. 기존 **standalone P0 경로**도 별도 실행해 목적 38/38·반복 20/38을 유지했다. `git diff --check` 통과.


## 19. 목적 격자와 사용자 cutoff (2026-11-09)

`cutoff.rs`에 선형 목적 격자 계산(`5e2931`)과 개선 cutoff(`6f45ed`)를 추가했다. 기존 `iint::gcd_f64`(`a7bea8`)를 재사용한다. 목적 계수의 절댓값이 1e-10보다 큰 연속 열이 있으면 격자를 사용하지 않는다. 정수 열의 계수들에서는 GCD가 `IntFeasTol * max|c|`보다 클 때만 격자를 채택한다. 단일 계수와 영 목적의 원본 예외도 유지한다. `GURO_PAR_OBJGCD=0`은 격자 추론을 끈다.

계산은 최소화 좌표에서 수행한다. 사용자 `Cutoff`가 incumbent보다 작으면 사용자 값을 그대로 반환한다. incumbent가 없으면 유한 사용자 cutoff만 사용한다. incumbent가 있고 양수 격자가 없으면 기존 `U-(abs(U)+1)*1e-10`을 사용한다. 양수 격자에서는 `objcon + step * floor(((U-objcon)-max(1e-10,step*1e-10))/step)`에 `max((abs(U)+1)*1e-6,step*IntFeasTol,OptimalityTol)`을 더한다. 합의 반올림 보정과 기존 개선 상한과의 최솟값도 원본 순서대로 계산한다. 이는 다음 격자점 자체를 엄격한 상한으로 쓰는 계산과 다르다.

`Cutoff`는 입력 모델의 목적 방향으로 받고, 최대화에서는 유한 값을 한 번 부호 변환한다. 기본값 1e100은 방향과 관계없이 비활성화한다. 변환 근거는 `9828a1`의 env+0x24a8 처리다. 원샷 경로는 원모델 도메인에서 계산한 격자와 최소화형 목적 상수를 보관한다. P0 격자가 더 약해졌고 두 상수의 차이가 원래 격자의 정수 배수이면 `5e2931`처럼 원래 격자를 유지한다. cutoff와 격자는 복제한 루트 도메인에 적용하며 P0 모델 자체는 바꾸지 않는다.

예를 들어 정수 목적 `2*x+3*y`, incumbent `(2,2)`에서는 상한이 `(5,3)`에서 `(4,3)`으로 바뀐다. `GURO_PAR_OBJGCD=0`이면 기존 `(5,3)`이다. `Cutoff=6`을 주고 휴리스틱을 꺼도 상한은 `(3,2)`가 된다. 최대화와 목적 상수 7을 함께 사용하는 Rust 회귀에서도 사용자 cutoff의 방향을 확인했다. 원샷 경로와 standalone 경로를 모두 검사한다.

네이티브 근거는 저장된 `5e2931`, `6f45ed`, `5e7dec`, `9828a1` 해독본과 실제 기계어다. `6f45ed`의 `5e7dec` 호출에서 두 번째 double 인자가 목적 격자임을 기계어로 확인했다. `tools/mip_cutoff_oracle.py`는 기존과 같은 Gurobi 13.0.3 SHA-256을 검사한 뒤 직접 호출한다. `bench/oracle/mip_cutoff/`에 cutoff 900개와 격자 44개 결과를 TSV로 보존했다. Rust 회귀는 저장된 결과와 double 비트를 비교하며 Gurobi 없이 실행한다. 다음 명령은 원본 호출을 다시 실행해 저장 파일과 비교한다.

```bash
tools/gurun.sh ../.venv/bin/python tools/mip_cutoff_oracle.py
```

`tools/mip_root_check.py --original`은 원샷 경로를 비교하고 각 P0가 별도 presolve의 MPS와 같은지도 검사한다. 인자 없이 실행하면 기존 standalone 경로를 비교한다.

범위는 선형 C/B/I 목적과 기존 목적 기반 경계 조임이다. 이차 목적의 격자, 원본 모델의 저장된 추가 격자 메타데이터, 전체 `6c5042` 전파는 포함하지 않는다. 모순이 되는 경계 조임은 기존처럼 생략한다. 따라서 cutoff는 전체 목적 제약으로 강제되지 않으며, 반환된 root 상태는 cutoff 만족이나 MIP 최적성을 판정하지 않는다. incumbent가 사용자 cutoff를 넘더라도 검증된 해 자체는 따로 유지한다.

최종 검증: Rust workspace 테스트 91개와 clippy `--workspace --all-targets -- -D warnings` 통과. release Python 확장을 다시 빌드한 뒤 전체 pytest 88 passed·1 xfailed, 루트 pytest 25 passed다. 네이티브 cutoff 900개와 격자 44개는 Rust에서 double 비트가 일치했고, 원본 재호출도 저장된 TSV와 일치했다. MIPLIB 38개는 standalone과 원샷 경로 모두 루트 목적 38/38·반복 20/38을 유지했다. 원샷의 P0는 38개 모두 별도 presolve MPS와 같았다. `git diff --check`도 통과했다.

## 20. 전체 MIP 종료 판정과 후속 전파 (2026-11-09)

`termination.rs`는 LP 상태와 별도로 전체 MIP 상태를 판정한다. 근거는 저장된 `5eda00`, `5edf6b`, `a83b2f`, `5bbf7f` 해독본이다. `5eda00`의 `8950f3` 호출은 도메인 +0x154의 충돌 플래그를 읽는다. 열린 노드 수를 읽는 함수가 아니다. 호출자는 도메인 충돌이나 전체 탐색 소진을 `State.exhausted`로 전달한다.

판정 순서는 기존 상태, 메모리 중단, 사용자 중단, 시간과 work, 탐색 소진, 무한 bound, cutoff, 상대·절대 gap, 목적 정지값, 노드·반복·해 개수 제한이다. gap 비교는 엄격한 `<`이며, 0은 해당 gap 검사를 끈다. work 제한도 원본처럼 `work > WorkLimit`이다. `lattice_bound`는 목적 상수를 뺀 bound에 원본의 허용오차를 적용하고 `ceil`로 격자점까지 올린다. `improve_start`는 종료 상태와 별도로 단계 전환 여부를 반환한다. `finish`는 사용자 cutoff가 있고 수락한 해가 없는 최종 OPTIMAL·INFEASIBLE을 CUTOFF(6)로 바꾼다.

`propagate.rs`는 각 행의 최소 activity로 열 경계를 조인다. 등식은 양방향 부등식으로 처리한다. 최소화 좌표의 목적 cutoff도 하나의 행으로 넣는다. 경계가 바뀌면 해당 열을 포함하는 행을 다시 큐에 넣으므로 목적식에서 시작한 조임이 여러 행으로 전달된다. 정수 반올림, 음수 계수, 무한 경계, 중복 행렬 원소, 빈 행을 처리한다. activity 누적 오차와 FeasibilityTol을 조임에 반영한다. 개선 영역이 모순이면 충돌을 반환한다. 기존 경로처럼 모순 조임을 생략하지 않는다.

이는 원본 `6c5042`의 캐시·추론 구조를 그대로 이식한 구현은 아니다. 연속 행이 점근적으로 수렴하는 경우를 위해 행 처리 횟수를 `100 * (부등식 수 + 열 수 + 1)`로 제한한다. 예산 소진은 충돌이 아니며, 그때까지 얻은 경계는 유지한다. `propagation_complete`는 충돌을 증명했거나 큐가 비었을 때 true다.

Python의 `mip_root_progress(params, incumbent=None)`는 같은 모델 좌표에서 다음 값을 반환한다.

```python
(status, bound, incumbent_objective, incumbent, root_lp, (lb, ub), propagation_complete)
```

`status`는 MIP 상태이고 `root_lp`의 상태는 LP 상태다. 분수 루트가 남으면 MIP 상태는 Running(1)이다. 정수 루트해는 원래 모델의 열·행·정수성 검사를 거쳐 제출하고, 새 cutoff를 다시 전파한다. unbounded LP만으로 MIP unbounded를 선언하지 않는다. 목적값과 bound는 입력 모델의 목적 방향으로 반환한다. 반환 경계는 incumbent보다 나은 해를 찾는 영역의 경계이며, incumbent 자체를 포함하지 않을 수 있다.

`mip_presolve_progress(params)`는 원모델 휴리스틱, presolve, 후속 전파와 종료 판정을 연결한다. 반환 형식은 다음과 같다.

```python
(original_incumbent, (presolve_status, p0, progress))
```

원모델 incumbent는 원모델 좌표와 목적 방향으로 남는다. `progress`의 새 루트해와 목적값·bound는 P0 좌표와 목적 방향을 쓴다. 목적 정지 파라미터는 원모델 방향에서 최소화 좌표로 변환한다. 원모델 incumbent의 목적은 종료 판정에 전달하지만 P0 해 벡터를 추측하지 않는다. 사용자 cutoff 밖의 휴리스틱 해는 진단용 원모델 incumbent에 남을 수 있으나, 수락된 해로 세지 않는다. P0 자체는 변경하지 않는다. 기존 `mip_root`, `mip_root_bounds`, `mip_presolve_root`는 기존 루트 LP 비교 경로를 유지한다.

현재 루트 경로는 실제 simplex 반복 제한과 경과 시간 검사를 연결한다. 시간은 휴리스틱·presolve와 루트 처리 경계에서 검사하며, 실행 중인 simplex나 전파를 선점하지 않는다. 공통 판정 함수의 work·메모리·중단 카운터는 향후 탐색 실행기가 공급해야 한다. ImproveStart 스케줄러, 해 풀 검색 모드, cuts, 분기와 B&B 탐색은 아직 연결하지 않았다. 따라서 Running은 전체 MIP 풀이 완료를 뜻하지 않는다.

검증 명령은 다음과 같다.

```bash
cargo test --release --workspace
cargo clippy --release --workspace --all-targets -- -D warnings
PYTHONPATH=$PWD/python ../.venv/bin/python -m pytest -q
PYTHONPATH=$PWD/python ../.venv/bin/python tools/mip_progress_check.py
PYTHONPATH=$PWD/python ../.venv/bin/python tools/mip_root_check.py
PYTHONPATH=$PWD/python ../.venv/bin/python tools/mip_root_check.py --original
```

Rust 회귀는 종료 조건 우선순위, gap 경계, 목적 격자, 최대화 정지값, cutoff 최종 상태를 검사한다. 작은 정수 모델 256개에서 모든 정수점을 열거해 전파가 실행 가능한 점을 제거하지 않는지도 확인한다. Python 회귀는 LP와 MIP 상태 분리, 연쇄 전파, cutoff 충돌, 제한 상태, 빈 모델, 원모델과 P0 좌표를 검사한다. `mip_progress_check.py`는 MIPLIB 전체의 원모델 해와 P0 해를 독립 검증하고, P0 보존과 알려진 실행 가능 목적값에 대한 bound를 검사한다.

검증 결과: Rust workspace 테스트 100개와 clippy `-D warnings` 통과. 전체 pytest는 95 passed·기존 1 xfailed이며, 최종 재빌드의 루트 회귀 32개도 통과했다. MIPLIB 38개 progress 검사는 P0 보존·해 검증·bound 검사에 모두 통과했다. 상태는 Running 37개, OPTIMAL 1개다. 모든 모델의 전파는 예산 안에 완료됐다. 기존 standalone과 원샷 루트 비교는 모두 목적 38/38·반복 20/38을 유지했다. 새 전파의 원본 실행 순서·double 비트 패리티는 검증하지 않았다. `git diff --check` 통과.
