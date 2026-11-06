# M2 MIP 루트 LP 완화 작업 노트 (`5daa82` → `64da9f` → `5ce67b`)

MIP presolve(`3cd406`, G6) 다음 단계인 루트 LP 풀이를 옮기며 알아낸 사실. 범위: `Mip::new`(`5daa82`)에서 루트 노드 생성 → `64da9f` 안의 루트 LP 풀이까지. cut 루프, 루트 휴리스틱, propagate, 분기, B&B는 범위 밖이고 만난 곳은 "여기서 멈춤"으로 적었다. 브랜치 `m2-root`(worktree `../rsolver-m2-root`, `m2-mip-presolve` + `main` 병합).

## 0. 현재 상태 (2026-11-06)

| 기준 | 상태 |
|---|---|
| (1) miplib_small 38개, 기본값 + Threads=1, "Root relaxation" 줄 | **목적 38/38, 반복 7/38**(둘 다 맞는 것 7/38). 38에는 루트가 없는 enlight_hard(presolve가 전부 지움, 우리도 `None`)를 포함한다. 루트가 있는 37개만 세면 목적 37/37, 반복 6/37. 기준값 `bench/gurobi/<이름>/log.txt`는 이미 Threads=1 로그다 |
| (2) 불일치 모델의 첫 분기점 | §4 표. 입력 모델(bound) 3, 열 형식 가중(G1) 12, simplex 경로(G1) 16 |
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

**확정**: LP 단독 실행에서 2f7e1 직후 spx+0xb8에 MIP의 열 형식을 써 넣으면(`PATCH_VT`) 37/37 모두 로그의 루트 반복 수와 같아진다(glass4 36 → 72). 즉 루트 LP 경로 = 그 모델·파라미터·열 형식의 dual simplex다. `crates/lp`는 열 형식을 읽지 않는다(G1 범위, 고치지 않음).

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
PYTHONPATH=$PWD/python ../.venv/bin/python tools/mip_root_check.py            # 목적 38/38, 반복 7/38 (Gurobi 불필요)
tools/gurun.sh node tools/mip_root_trace.js <이름> <dir>                        # events.jsonl, <k>.mps, <k>.prm, load_<i>.mps, log.txt
SPX_K=<k> tools/gurun.sh node tools/mip_root_trace.js <이름> <dir>              # + spx_<k>.json (2f7e1 직후 spx, vtype 포함)
LP=<dir>/<k>.prm SPX_K=1 tools/gurun.sh node tools/mip_root_trace.js <dir>/<k>.mps <dir2>             # LP 단독
PATCH_VT=<vt 파일> LP=<dir>/<k>.prm SPX_K=1 tools/gurun.sh node tools/mip_root_trace.js <dir>/<k>.mps <dir3>  # + 열 형식
```
- 루트 k: events.jsonl에서 `cda5b` 복귀의 `iter`/`obj`가 log.txt 루트 줄과 같은 첫 호출. vt 파일 = `spx_<k>.json`의 `vtype[:n]` 바이트.
- bound를 바꾸는 함수 찾기: `5daa82`·`64da9f`의 callee(`re/decomp/callgraph.tsv`)를 전부 걸고, `5e6ed8` 진입에서 M을 잡아 `*(*(M+0x3298)+0x10)+0xa8`의 한 열을 진입/복귀마다 읽는다(이번 세션 스크래치 `calltrace.js`; 필요하면 `mip_root_trace.js` 틀로 다시 만든다).
- 결과 .mps는 정수 MARKER 없이 연속 LP로 쓰인다(열 형식은 spx에만 있다).

## 6. 남은 일

1. (G1) `crates/lp`가 열 형식을 읽어 CHUZR 점수 ×1.5·섭동 w=1.5를 넣기(`028c93`, `048d5f`), INTBASIS=1·UNSCALEDVIO=0의 효과 해독. B2 12개·B1 3개가 걸려 있다.
2. (G1) LP 단독 parity: C 16개.
3. (propagate/휴리스틱 범위) 루트 전 domain store: `66bdf6`의 초기 조이기, `715361`/`670717`, `5ebfbc` incumbent 조이기, `8ac5fe` 루트 휴리스틱. 들어오면 `solve_root`가 domain bound를 받도록 바꾼다.
4. 자동 concurrent(4)·barrier 루트(5ccea5의 `98b671` 분기), RELAXPREP/PreDual로 가는 `c3d0d` 분기: 38개에 없어 미포팅.
5. 확인 못 한 것: spx+0x08·+0x128 차이의 의미(PATCH_VT만으로 37/37이 맞아 영향 없다고 봄), `66bdf6` 등이 조인 모든 열의 출처(열 하나씩만 추적), INTBASIS의 실제 의미, 루트 시간·work 단위.
