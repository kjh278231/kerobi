# M1 LP 드라이버 작업 노트 (`9828a1` → `c23877` → `0c3d0d`, 파라미터 `ac81d7`)

G5([11 §4](11_progress.md)) 작업 중에 알아낸 사실. 확정된 내용은 [02](02_driver.md)로 옮긴다. 브랜치 `m1-lp-driver`(worktree `rsolver-presolve`).

## 0. 현재 상태 (2026-11-06)

| G5 기준 | 상태 | 근거 |
|---|---|---|
| (1) 파라미터 표 548개 = gurobipy | **충족** | §2, `tests/test_params.py` 548/548 |
| (2) Optimize 헤더 로그 102개 | **충족** | §3, `tests/test_header.py` 102/102 |
| (3) 분류·Method 확정·자동 조정·상태·요약 로그 | **충족(연속 LP 몫)**: Method 확정·자동 조정 단위 테스트, 경로별 로그 앵커 3/3(afiro·grow7). MIP 분류·요약은 범위 밖 | §4, `crates/driver` 테스트, `tests/test_driver.py` |
| (4) Presolve 켠 연속 경로 (Method 2·6 반복 수, Method 1 상태·목적) | **충족**: Method 2·Crossover 0 상태·목적·BarIterCount **64/64**, Method 6·Crossover 0 상태·목적·PDHGIterCount **64/64**(bnl1 WORK_LIMIT는 Work까지 비트 일치), Method 1 상태·목적 **64/64** | §4–§6, `tests/test_driver.py` |
| (5) clippy 0, 테스트, 문서 | **충족**: clippy 경고 0, cargo test 전부, pytest 140 통과(1 xfail은 기존 MIP 배낭), barrier·PDHG Presolve=0 64/64 유지 | |

## 1. 호출 범위 (2026-11-06)

### 1.1 정적 호출 그래프로는 갈리지 않는다

`re/decomp/callgraph.tsv`에서 `9828a1`·`c23877`·`0c3d0d`는 모두 같은 강연결 덩어리(닿는 함수 2,767개: common 804, cut·휴리스틱 377, presolve 365, simplex 313, B&B 264, genconstr 234, barrier·PDHG 223, driver 149)에 들어간다. sub-MIP·다목적·휴리스틱이 `98b671` → `c23877`로 되돌아오기 때문이다. 그래서 범위는 **실제로 도는 경로**를 Frida로 떠서 정했다.

방법: `c23877`·`9828a1`·`0c3d0d`·`172cc6`·`2bb770`·`34f033`·`9b029d`와 그 직접 callee 181개에 진입·복귀 훅을 걸고(`tools/fn_trace.js`, `leave: []`), afiro를 Presolve=-1, OutputFlag=1, Threads=1로 Method 기본·1·2(Crossover=0)·6(Crossover=0)에서 풀었다. 스펙 생성기와 트리 출력기는 세션 스크래치에 있다(재현: callgraph에서 위 7개의 callee를 뽑아 `{"name","off","leave":[]}` 목록을 만든다).

### 1.2 연속 LP에서 도는 경로 (관찰)

```
9828a1
  a841df(→a7f063)  edd71(update)  bb8b1f(batchmode 검사 →a7f821)  edd71  9a8287  a7f063 ×2
  c23877(model, 1, timer)
    a7f063 ×2 (isMIP)  115e51 (웜 basis)  115faf (시작 벡터)
    a8c184 (지문)  a8aa6a ("Model has …")  a7f063  a9a78e (계수 범위 통계)
    19febe (다목적 수)  a7f0ad/a7fb80/a7f923 반복 (QP·2차·QCP 판정, Method 확정)
    86113 (이전 결과 해제)  75a3b0 (Lazy 배열 정리)
    ├ Method 기본·1: 0c3d0d                                   → a6a8da (LP 결과 속성: IterCount 등)
    ├ Method 2, Crossover 0: 2bb770                            → a6ad23 (비기저 해 속성), a7f923
    └ Method 6, Crossover 0: 34f033                            → a6ad23
  9a8287  a84513(→a7f063)
```

`0c3d0d` 안 (Method 1, afiro):

```
ac7d4e (스레드 수)  10f448  88c65 (이전 presolved 모델 해제)  3b88f5
88237 (웜스타트 모드: 10a423, a7f0ad, 11603b, 115faf)  10a423  115faf  1160a1  88068 (웜스타트 구조체 해제)
f7dc1(→83696)  a7f0ad
c31c5 → f7dc1, ★390989 (presolve), a7f0ad          ← PreDual 판정 겸 presolve 래퍼 (노트 13 §1의 "c379f = presolve"는 틀림)
b9b82 (PreDual 판정)  a7f0ad ×2
cda5b (presolved 모델 simplex: 7d3ce 스케일, a83b2f 한도, a6a8da)
d10b1  a7aa8f ×2
c379f (SolutionTarget < 1이면 곧바로 반환. 아니면 36578f)
★363daa (LP uncrush)  12ef7f  a8ff5e
88c65 (presolved 모델 해제)  3b88f5  a7f0ad
cda5b (원래 모델, uncrush 기저로 웜스타트: "Extra simplex iterations after uncrush")
d10b1  a7aa8f  c9457 (상태 메시지 "Optimal objective …")  d10b1 ×2  10f5ae
```

`2bb770`(barrier 진입): a7f0ad, a7f923, 86113, 83696, 2d0596, 2ee55d, f7dc1, ★390989(→2baa47), ★2bd9ab(barrier 본체), ★36578f(LP uncrush 드라이버), 88068, 2ce5c6, 88c65 ×2.
`34f033`(PDHG 진입): a7f0ad, a7f923, 86113, 83696, 88237, 1160a1, 2baa47, f7dc1, ★390989, ade552 ×8, adec59, ★34c004(PDHG 본체), ★36578f, 10b63c ×2, 7a2ee.

### 1.3 범위와 이미 포팅된 부분

| 묶음 | 함수 | 상태 |
|---|---|---|
| 파라미터 표 | `ac81d7`(이름 → env 오프셋 등록, `accc50`/`accde4`/`accefe`) + `.data` 정의 표 | §2. `ac81d7`은 값의 저장 위치만 정하고, 이름·타입·최소·최대·기본값은 `.data`의 40바이트 레코드 표에 있다 |
| 진입 | `9828a1`: update, 최대화 부호 반전, 비볼록 재시도 | 연속 LP에서는 `c23877` 한 번 |
| 드라이버 | `c23877` A·C·F·H·I·K·L 단계([02 §2](02_driver.md)), 분류 `a7f063`·`a7f0ad`·`a7fb80`·`a7f923`·`a7fbf4`, 결과 속성 `a6a8da`·`a6ad23`, 상태 메시지 `c9457` | 미포팅 |
| 헤더 로그 | `a8c184` 지문(포팅됨, `fingerprint.rs`), `a8aa6a`, `a9a78e` | 지문만 |
| LP 드라이버 | `0c3d0d`: 웜스타트 판정, `c31c5`(presolve), PreDual `b9b82`, `cda5b` 두 번, uncrush `363daa` | presolve `390989`·uncrush `363daa`는 포팅됨(G2). `cda5b`는 `simplex.rs` `solve_model`(main의 simplex 에이전트 소유, 이 브랜치에서 고치지 않음). 웜스타트 입력이 없어 uncrush 뒤 재풀이는 아직 못 한다 |
| barrier·PDHG | `2bb770`/`34f033`: presolve → 본체 → `36578f` | 본체는 포팅됨(G3·G4, Presolve=0 경로). `36578f`(비기저 해 uncrush, "big uncrush violation")는 미포팅 |

범위 밖: MIP 경로(`3db15a`, `5bbf7f`, 휴리스틱), 다목적 `1b39db`, 다중 시나리오 `7a0103`, concurrent(`172cc6`, `9b029d`), network `170d99`, sifting, QP `14abfb`, crossover, 원격·라이선스.

## 2. 파라미터 표 (`ac81d7` + `.data` 정의 표)

- `ac81d7`(env 생성 때 `GRBemptyenvadvinternal`·`ad0297`·`bb32bc`가 부름)은 이름마다 `accefe`/`accc50`/`accde4`로 런타임 표(env+0x23e0, 0x40B 항목)를 찾아 env 오프셋을 적고 기본값(+0x28)을 env에 복사할 뿐이다. 이름·타입·범위·기본값의 원본은 `.data`의 **40바이트 레코드 표**다: +0 이름 포인터(R_X86_64_RELATIVE 재배치), +8 플래그(int, 의미 미확인), +12 타입(1 int, 2 double, 3 string, -1 표식), +16/+24/+32 min/max/default(double). `Cutoff`에서 시작해 공개 208개, 표식 `EndPublic`, 숨은 `GURO_PAR_*` 340개, 끝 표식 `End` 순이다.
- 포팅: `tools/param_table.py <libgurobi130.so>`가 이 표를 그대로 `crates/model/src/param_table.rs`(548개, 원래 순서)로 생성한다. `params.rs`는 `Def`, 대소문자 무시 `find`, 문자열 기본값 `default_str`만 둔다. 값 저장(env 오프셋 대응)은 드라이버가 쓸 때 붙인다.
- 기준값: `tools/param_oracle.py`가 gurobipy `getParamInfo`를 548개 이름(숨은 것 포함, 모두 받아 줌)에 대해 떠서 `bench/gurobi_params.tsv`에 둔다. Python `Model.getParamInfo`와 비교해 이름·타입·최소·최대·기본값 548/548 같다.
- gurobipy 규약: double의 ±1e100(GRB_INFINITY)은 ±inf로 보고한다(예: Cutoff -inf..inf, 기본 inf; GURO_PAR_INFBOUND 최대 inf). 문자열의 min/max는 `''`.
- 교차 확인: M2 브랜치의 `docs/m2_param_offsets.tsv`(`5627199`, `ac81d7` 등록 순서의 env 오프셋 416개)와 이름·최소·최대·기본값 416/416 같다. 오프셋이 필요하면 그 파일을 본다.
- 표 밖 기본값은 `NodefileDir = "."` 하나다(표에는 빈 문자열). 이 값을 넣는 코드는 찾지 않았다.
- 노트 11 §5.1의 "562개 항목"은 이 표(550 레코드, 표식 2개 포함)와 다르다. 다시 센 값은 548개다.
- 생략: docs/01 §1.2의 정적 필드 구조체(빌드 스크립트 생성)와 `setParam` 범위 검사·오류 10007/10008. 드라이버는 이름으로 값을 읽는다.

## 3. Optimize 헤더 (`c23877` C 단계)

`crates/driver/src/header.rs`. 최상위 첫 호출에서 차례로 찍는다(노트 11 §3.3).

- `"Optimize a model with %d rows, %d columns and %lu nonzeros (%s)"`: D+8, D+0xc, D+0x10, ModelSense가 -1이면 `Max` 아니면 `Min`.
- `"Model fingerprint: 0x%08x"`(`a8c184`, M0에서 포팅). GURO_PAR_FINGERPRINT=-1이면 sub-solve·로그 꺼짐·연속 모델의 LP 웜스타트일 때 생략한다(웜스타트는 아직 없음).
- `a8aa6a`: `|c_j| > 1e-13`인 열 수. `|objcon| > 1e-13`이면 `" and an objective constant of %s"`, 수는 `a8030d`(정수면 `%d`, 아니면 `%.15g`, 되읽어 다르면 `%.16e`) = 기존 `mps_write::fmt_num`. Q·2차 제약·SOS·PWL·일반 제약 줄은 LP 범위 밖이라 옮기지 않았다.
- `a8a3b4`(MIP일 때만): vtype 바이트로 `C`/`I`/`B`/`S`/`N`을 센다. "integer"는 I+B. S+N > 0이면 "Semi-Variable types" 줄.
- `a9a78e(model, 1)` → `a71757`: **model+0x2d8 통계 배열을 채우는 함수가 이것이다**(docs/02 §10의 미확인 항목). 배열 [0,1] 목적 max/min, [2,3] Q 목적, [4,5] bound, [6,7] RHS, [8,9] 행렬("MaxCoeff"/"MinCoeff"), [10..15] Q·QL·QRHS. 각 항목은 0이 아닌 |값|의 max·min이고 시작값이 (0, 1e100)이라 값이 없으면 끝에서 `min = min(min, max)`로 (0, 0)이 된다. bound는 lb·ub 모두, `|v| < 1e30`만. 출력은 `"  %-16s [%.0e, %.0e]"` 꼴로 min, max 순.
- 경고(`a9a78e`): `log10(max+1e-10) − log10(min+1e-10) ≥ 12`이면 "large matrix coefficient range", 아니면 행렬 max > 1e8이면 "large matrix coefficients". 목적·RHS·bound max > 1e9면 각각 "large objective coefficients"/"large rhs"/"large bounds". 경고가 하나라도 있고 NumericFocus(env+0x2718) == 0이면 "Consider reformulating …" 두 줄.
- 검증: `tests/test_header.py`가 `bench/gurobi/*/log.txt` 102개(Netlib 64 + miplib_small 38)의 "Optimize a model" 줄부터 첫 빈 줄 전까지와 글자 단위로 비교한다. 102/102(objcon 있는 e226, 경고 있는 mas74·mas76 포함).

## 4. 드라이버 (`crates/driver/src/lib.rs`)

### 4.1 구조

- `optimize(md, params, hooks)`: 헤더(§3) → `resolve_method`(c23877 F) → `adjust`(H) → 디스패치(I) → 각 경로가 presolve → 엔진 → uncrush → 상태 → 요약 로그. 파라미터는 `rsolver_model::params::Params`(표 순서의 값 배열, 이름으로 읽음)이고, 이번 optimize용 사본에만 조정을 쓴다(11 §3.1).
- Python: 읽은 연속 모델이고 Presolve ≠ 0이면 `Model._driver` → `CoreModel.optimize(pairs, bar_cb, pdhg_cb)`. 로그는 문자열로 받아 LogFile에 덧붙이고 OutputFlag·LogToConsole이 켜져 있으면 출력한다. Presolve=0과 API로 만든 모델은 기존 분기(`solve`, Method 2 `_barrier`, Method 6 `_pdhg`)를 그대로 탄다.
- 디스패치: Method 2·Crossover 0·SolutionTarget ≠ 1 → `interior(Barrier)`(`2bb770`), Method 6 같은 조건 → `interior(Pdhg)`(`34f033`), 나머지 → `simplex_driver`(`0c3d0d`). 지름길: crossover가 없어 Crossover ≠ 0인 Method 2·6도 crossover 없이 같은 경로로 간다. concurrent(3·4)는 스레드 1개일 때의 `0c3d0d`처럼 dual로 푼다. Method 5는 경고 뒤 3.
- `resolve_method`: 11 §3.6의 순수 LP 경로(웜스타트·PWL 목적은 아직 없음). nnz < 30,000이면 1, 아니면 3. SolutionTarget 1·0 규칙은 `a71757` 통계(`header::stats`)로 옮겼다(S[0] 목적 max, S[4] bound max, S[6] RHS max, S[8]/S[9] 행렬 max/min).
- `adjust`: 11 §3.9 표 중 연속 모델에 닿는 행(Presolve=0 → DualReductions=0, LazyConstraints, PRESYMMPASSES, NumericFocus 1·2·3). MIP 전용 행(ObjScale -3, MIPFocus)은 범위 밖.

### 4.2 simplex 경로와 `363daa` 입력

`0c3d0d`는 presolved 모델을 `cda5b`로 풀고 `363daa`로 되돌린다(§1.2). `363daa`는 presolved simplex 배열을 **열 다음 행 순서, 비스케일**로 읽는다. 엔진(`Engine`) 배열에서 이렇게 만든다(`driver::simplex`):

- x: 열은 `x'_j·s_j`, 행은 slack `x'_{n+i}/r_i`(= σ_i(b_i − a_i x), `>` 행은 σ = −1).
- dj: 상한 비기저(−2)는 엔진이 부호를 뒤집어 저장하므로 되돌린 d로, 열은 `d_j/(s_j·k)`, 행은 `d_{n+i}·σ_i·r_i/k`(k = 목적 배율). 행 항목은 쌍대값 y이고 slack 열의 d가 −y다.
- 상태: 엔진 `vstat` 그대로(기저는 위치 ≥ 0, −1/−2/−3/−4).
- 검증: `bench/gurobi_uncrush`의 Gurobi 입력(`pre_after`)과 비교해 afiro·forplan·fit1d는 x·dj·상태가 1e-7 안에서 같고, scfxm1·gfrd-pnc는 퇴화 LP의 다른 최적 기저라 일부만 다르다(스크래치 `cmpspx.py`, `CoreModel.presolved_spx`).
- 지름길: uncrush 뒤 원래 모델 simplex("Extra simplex iterations after uncrush")는 엔진에 웜스타트 입구가 없어 하지 않는다. Gurobi도 Netlib 64개 중 5개만 추가 반복이 있다.

### 4.3 barrier·PDHG 경로

- 경로: presolve → `barrier::solve`/`pdhg::solve`(presolved 모델) → uncrush → 상태. 지름길: `36578f`(원시 `363164`, 쌍대 `365159`, "big uncrush violation"으로 13 만들기)를 옮기지 않고, x는 `363daa`에 slack과 모두 기저인 가짜 상태를 넣어 되돌린다(원시값만 씀, 쌍대는 내보내지 않음). Netlib 64개 기준값은 모두 상태 2라 13 규칙이 걸린 사례가 없다.
- presolve가 Method를 알아야 했다(§5), 상한만 있는 열을 뒤집어야 했다(§6.1), presolved 모델의 RNG와 작업량 타이머를 원본대로 넘겨야 했다(§6.2–6.3). 셋을 넣은 뒤 64/64.
- 기준값: `bench/gurobi_bar_pre`(Method 2, Crossover 0), `bench/gurobi_pdhg_pre`(Method 6, Crossover 0), `bench/gurobi_dual_pre`(Method 1). 모두 Presolve 기본값, Threads 1, WorkLimit 60, `tools/parity.py capture gurobipy … --lp-only`. 상태: barrier 64×2, PDHG 63×2 + bnl1 16(WORK_LIMIT), dual 64×2.
- 점검: `PYTHONPATH=$PWD/python python tools/driver_check.py <기준값> [--iters BarIterCount|PDHGIterCount] --param …`(Gurobi 불필요).

## 5. presolve의 Method 분기 (`crates/lp/src/presolve/`, 2026-11-06 수정 허용)

`3a484d`가 `state+0x100`을 정한다: SolutionTarget = 1이거나 (SolutionTarget ≤ 0, Q·QC 없음, Method ∈ {2,6}, Crossover = 0)이면 1, 아니고 SolutionTarget < 2이며 NumericFocus > 0이면 2, 나머지 0(`Params::sol_mode`). 이전 포트는 Method·Crossover·SolutionTarget을 기본값으로 가정해 NumericFocus로 대신했다. 바꾼 곳(원본 줄):

| 원본 | 규칙 | 포트 |
|---|---|---|
| `390989` L250–270 | PreDepRow −1: PRESYMMPASSES ≥ 1이나 Presolve 1이면 0, Presolve 2·Method 2·state+0x100 ≠ 0이면 3, 아니면 Method ≠ 6일 때 유형 1이나 2.5n < nnz면 3 | `mod.rs` `reduce` |
| `5118e6` L554 | 유형 6(0/1 행 완화)은 Method ≠ 2일 때만 | `state.rs` `constructor_work` |
| `505077` L118 | 함의 bound를 자유로 보는 것은 state+0x100 = 0이고 (Method ≠ 2 또는 Crossover = 0 또는 SolutionTarget > 0)일 때 | `reduce.rs` `dual_bound_rows` |
| `51a144` L91 | 쌍대 singleton 고정은 state+0x100 = 0도 요구 | `mod.rs` 두 호출 |
| `50de61` L769 | Method 2나 state+0x100 ≠ 0이면 자유 열에만 함의 bound | `reduce.rs` `bound_rows` |
| `37f484` L512–516 | state+0x100 ≠ 0이면 첫 라운드도 전파 없이 `53c9ee` 목록 | `agg.rs` `agg_round` |
| `37f484` L1432 | 쌍 집계 비용 보정은 state+0x100 = 0일 때만 | `agg.rs` |
| `509fd0` L58 | 평행 열 허용치·함의 bound는 state+0x100 = 0일 때 | `parallel.rs` |

- `50fe2b`의 완화 여부는 state+0x100 = 0이면서 INTBASIS > 0이거나 env+0x2728 > 1일 때만 꺼져서 기본값에서는 그대로다. `+0x104`(Method 2/6·crossover 없음이면 0)는 sparsify `470936`만 읽어 기본값과 무관하다.
- 검증: `tools/presolve_oracle.py <dir> --param Method=2|6 --param Crossover=0`과 `presolve_check.py`가 8/64 → **64/64**(두 Method 모두). 기본값 P0·postsolve·대칭 테스트(`test_presolve`, `test_postsolve`, `test_symmetry`)도 그대로 통과.

## 6. barrier·PDHG로 넘기는 것

### 6.1 상한만 있는 열 뒤집기 (`07d3ce`)

presolved perold에는 하한 −∞·상한 유한 열이 1개 있고 Netlib 원본 64개에는 없다(G3·G4가 못 본 경우). `07d3ce`는 모델 데이터 표시 D+0x408 = 1인 열을 c·a 부호 반전, (lb, ub) → (−ub, −lb)로 바꾼다(L395–500, D+0x3c = 1). 조건은 D+0x3c = 0, Q·QC·PWL 없음. `363daa`는 같은 표시로 x·dj를 되돌린다(postsolve 포트 입력은 되돌린 뒤의 값). 표시를 세우는 함수는 못 찾았지만 Gurobi가 presolved MPS를 읽어 풀어도 같은 결과라 모델 수준 규칙(하한 −∞, 상한 유한)으로 옮겼다(`driver::flip_upper_only`, 엔진 전 뒤집고 x를 되돌림). 이것으로 perold BarIterCount 99 → 94(= Gurobi). 지름길: Presolve=0 경로(`Model._barrier`·`_pdhg`)는 이 변환을 거치지 않는다. 그런 열이 있는 모델을 그 경로로 풀 일이 생기면 barrier·PDHG setup으로 옮긴다.

### 6.2 presolved 모델의 RNG (`3b0917` → `895c5`)

Frida로 `a7b4d6`/`a7b54f`의 상태 포인터를 따라가면 presolve 경로의 PDHG는 presolved 모델의 RNG(model+0xb0)에서 power iteration 시작 벡터를 뽑는데, 그 상태가 0이 아니다(afiro 757). `3b0917`이 `895c5(presolved, 13·n + 7·m + 23·nnz)`로 정하고 `895c5`가 Seed(env+0x2cd0)를 더한다(n, m, nnz는 presolved 모델). 포트: `Model::rng` 필드(모델 데이터, 새 모델은 0), `presolve::run_post`가 설정, `pdhg::solve`가 `md.rng`에서 시작(전에는 0 고정). Gurobi도 presolved MPS를 Presolve=0으로 풀면 상태 0이라 우리 이전 값과 같았다(afiro 828, seba 310, vtpbase 1257). 이 수정으로 PDHGIterCount 9/64 → 63/64.

### 6.3 작업량 타이머

`34f033`은 전역 타이머(`param_2`)를 그대로 `34c004`에 넘기므로 PDHG 작업량은 presolve 작업량(`Post::work`, state+0x458)에서 시작한다(`pdhg::Params::work0`). WorkLimit 판정이 이 합을 본다. bnl1: PDHGIterCount 1,470,820과 Work 60.00225091963458이 Gurobi와 비트 단위로 같다 → 64/64. barrier는 반복 판단에 내부 단계 타이머만 써서 영향이 없고, Work 속성에는 presolve 작업량을 더해 둔다. simplex 작업량은 아직 없다(G1 P2).

## 7. 안 한 것·남은 위험

- 범위 밖: MIP 경로·분류(Python이 `IsMIP`이면 기존 오류), 다목적, concurrent(스레드 1개 dual로 대신), crossover(Crossover ≠ 0인 Method 2·6도 crossover 없이 끝냄), PreDual 쌍대화, network·sifting, 웜스타트.
- `36578f` 미포팅: barrier·PDHG 해는 `363daa`에 가짜 기저로 원시값만 되돌린다. 쌍대값·기저는 내보내지 않고 "big uncrush violation" → 13 규칙도 없다(64개 기준값에 사례 없음).
- simplex 경로: uncrush 뒤 원래 모델 simplex(웜스타트)를 하지 않는다. Netlib 64개 중 Gurobi도 5개만 추가 반복이 있다. 최대화 모델의 `363daa` 입력 규약은 확인하지 않았다(Netlib은 모두 최소화).
- 반복 로그(barrier·PDHG·simplex의 Iter 표)와 barrier의 "Ordering time"/"Barrier statistics"는 드라이버 로그에 아직 없다. 로그 앵커 테스트는 헤더·presolve·요약 줄만 본다.
- `presolve_status`의 UNBOUNDED → INF_OR_UNBD는 `a80155`를 읽지 않은 근사다(Netlib에 사례 없음).
