# M2 MIP presolve 상태와 뼈대 (`3a484d` MIP 분기, `3cd406` 앞부분·마무리, `3b0917`)

코드: `crates/mip`(`rsolver-mip`). `state.rs` 상태·생성자·`370047`·`36d89a`·모델 생성, `lib.rs` Params·prelude·마무리, `tracker.rs`는 LP 포트의 복사본(LP 하위 모듈은 private이고 `crates/lp`는 고치지 않는다). 상위 메모는 [m2_presolve.md](m2_presolve.md).

## 0. 상태와 검증 (2026-11-06)

| 확인 | 결과 |
|---|---|
| MPS 라이터: `rsolver.read(원본)._core.to_mps()` == Gurobi가 쓴 원본(`$S/orig`) | **38/38**(NAME 줄 포함 전부 같음) |
| `presolve_check.py $S/or/pre0 --param Presolve=0` | **38/38** |
| `$S/or/lim0`, `lim1`, `lim2` (`GURO_PAR_PRELIMIT=0/1/2`) | **38/38**씩 |
| 작업량·상태(`work_oracle.py --mip`: work, +0x08, +0xf4, +0xf0, +0xf8, +0x404, +0x408) Presolve=0·PRELIMIT 0/1/2 | **38/38**씩(neos-860300은 μ = +0x448 ≠ 1) |
| 합성 모델 9개(문제 유형 1·4, +0xf8, 연속 열 목적 검사 4가지, MAX, 정수 bound 반올림, 자유 행) 같은 네 설정, 모델·작업량·카운터 | **9/9**씩 |
| `3e39a6` 호출 시점(호출 지점 0x3b1dd0 훅) work·카운터·추적기 레벨, `GURO_PAR_PREBOUND=0` | **38/38** |
| `$S/or/lim3` | 36/38(gmu-35-40, gmu-35-50: 셋째 카운트 `5a9282`가 모델을 바꾼다, §4) |
| 정수 열이 2.5에 고정된 모델 | Gurobi와 같이 `GurobiError(-1, "Unable to create presolved model")` |

`cargo clippy --release --workspace --all-targets` 경고 0, `cargo test --release --workspace`, `pytest tests/`(36 통과, 기존 xfail 1) 통과. LP 출력은 바뀌지 않는다(라이터의 정수 분기는 정수 열이 있을 때만).

## 1. 상태 배치 (MIP, `3a484d(model, mipctx=0, 1, work)`)

세 번째 인자 1이 +0xfc에 들어가고 +0xe0은 0으로 남는다(+0xe0 = 1은 LP이고 SOS가 없을 때만). 그래서 행렬은 `5118e6`의 CSR이 아니라 0x20바이트 노드의 연결 리스트로 만든다.

| 오프셋 | 필드(`State`) | 내용 |
|---|---|---|
| +0x08 | `ptype` | 문제 유형. 1: 계수가 모두 ±1(둘 다 있음)이고 최대 열 길이 ≥ 5, `=` 행이 절반 초과. 4: 최대 열 길이 ≤ 2, ±1만, +1 수 = −1 수, 앞에서부터 이어지는 `=` 행 수 = m. 11: 목적 gcd 조건(포팅 안 함, §6). 나머지 0 |
| +0x0c / +0x10 | `n`, `m` | 열·행 수. +0xe8 / +0xe4(`n0`, `m0`)는 생성 때 값 |
| +0x60 / +0x68 | `row_cnt`, `col_cnt` | 살아 있는 항목 수. 음수 = 지움(−1 빈 행, −2 자유 행) |
| +0x70 / +0x90, +0x78 / +0x98 | `inf_max`/`act_max`, `inf_min`/`act_min` | max/min activity − rhs의 무한 개수와 유한 부분(LP와 같은 뜻) |
| +0x80 / +0x88 | `mult_lo`, `mult_hi` | 1.0 |
| +0xa0 / +0xa8 / +0xb0 | `lb`, `ub`, `obj` | 목적은 최소화형(c·sense) |
| +0xb8 / +0xc0 | `rhs`, `sense` | `>` 행은 계수·rhs를 뒤집어 `<`로(LP 포트와 같은 D 규약) |
| +0xc8 | `objcon` | 최소화형 상수 |
| +0xd8 | `nnz` | 모델 비영 수(|a| ≤ 1e-13 포함) |
| +0xf0 / +0xf4 | `nbin`, `nint` | `3cd406` 반올림 루프가 센다(생성자에서는 0) |
| +0xf8 | `eq_rows` | `3cd406` L938: 이진 ≥ 99%, m > 1.2n, `=` 행 × 1000 > m |
| +0xfc | (항상 1) | MIP 플래그 |
| +0x150 | `vtype` | 모델 타입 복사본 |
| +0x158 | `node_cap` | 노드 용량 = spare + nnz |
| +0x160 | `nd` | 노드 배열. 노드 = 값 +0x00, 행 +0x08, 열 +0x0c(−1 = 지움), 행 다음 +0x10, 열 다음 +0x18 |
| +0x170 | `col_flag` | 열 플래그(0x40 MINBPFORBID, 1/2 SOS, 4 semicontinuous). 지금 모델은 모두 0 |
| +0x178 / +0x180 | `row_head`, `col_head` | 행·열 리스트 머리 |
| +0x188 / +0x190 | `npend`, `pend` | pending 리스트(0, 비어 있음) |
| +0x198 / +0x1a0 | `nfree`, `free` | free 리스트: 노드 nnz..nnz+spare−1이 순서대로. 개수 spare |
| +0x1a8 | `chunk` | `3b85ea`가 더 잡을 크기 = spare(SOS가 있으면 max(spare, SOS 길이)) |
| +0x1dc | `prelimit` | GURO_PAR_PRELIMIT |
| +0x1e0 / +0x1e4 / +0x1e8 | `precrush`, `opt_target_off`, `intfeastol` | PreCrush, OptimalityTarget ≠ 1, IntFeasTol(env+0x2468) |
| +0x1f0 | `budget` | 1e100(`3cd406`은 이 값을 바꾸지 않는다. 작업 한도는 reduction 타이머 쪽) |
| +0x3d8 | `tracker` | `534a8b`. 레벨 0/1/2, 처음에는 모두 들어 있고 플래그 5 |
| +0x3f8 .. +0x408 | 카운터 | bound 강화, 계수 수정, 부등식→등식, 지운 행, 지운 열 |
| +0x448 / +0x450 / *(+0x458) | `work_mult`, `work_base`, `work` | μ = nnz ≤ 1e5면 1, 아니면 sqrt(log10(nnz/1e4)). 시작 work 0 |
| +0x470 / +0x478 | `bad_col`, `bad_row` | 불가능을 증명한 열·행(+0x474..+0x484도 −1) |
| +0x490 | `cutoff` | 1e100(MIP 문맥·해 없음) |
| +0x390 | (생성자 지역) | 행마다 연속 열 정수성 검사용 칸(§2 G1·G3). 생성자 지역 `int_rows`로만 두었다(뒤 reduction이 이 내용을 읽는지는 확인 안 함) |
| +0x498 / +0x4a0 | (없음) | 목적 gcd와 상수. 포팅 안 함(§6) |

spare = nnz/10 ≤ 2e9이면 max(nnz/10, n, m, 10000), 아니면 2e9(LP와 같음).

## 2. 생성자 MIP 분기의 순서와 작업량

work 항목은 원본 순서 그대로(μ = +0x448). 곱셈 순서는 asm으로 확인한 것(괄호가 실제 순서). neos-860300(μ ≠ 1)에서 비트 단위로 맞는다.

1. L837 `'S'/'N'` 세기: `+= n`(μ 없음, MIP만). L962 nnz 합: `+= n`.
2. L1397 `+= (1.5n + m)·μ`, L1562 `+= (n+n)·μ`(+0x80/+0x88), L1594 vtype 복사 `+= (0.5n)·μ`(MIP만).
3. L4004 free 리스트 `+= (spare−1)·μ`.
4. G1 (L4021, DualReductions·n > 1·Q 없음·sub-MIP 아님·semicontinuous 없음): 비용 |c| > 1e-10인 첫 연속 열 j0를 찾는다. `+= (j0+j0)·μ`. 찾으면 행마다 +0x390 = (rhs 정수면 0, 아니면 n), `+= (m+m)·μ`.
5. G2 행마다 항목 수(|a| > 1e-13): 열마다 `+= len·(μ·3)`(len은 모델 열 길이), 끝에 `+= (n+n)·μ`. 누적합 `+= (m+1)·μ`.
6. G3 채우기: 열 순서대로 각 항목을 그 행의 다음 칸 노드에 넣고 열 리스트에 잇는다. 그래서 **행 리스트는 열 번호 오름차순, 열 리스트는 모델의 항목 순서**(작은 항목은 빠진다). 값은 D 규약(`>` 행 부호 반전). G1이 켜졌으면 항목마다 +0x390 갱신(계수가 정수가 아니면 n, 연속 열의 ±1이면 +1, 연속 열의 다른 계수면 n). 열마다 `+= len·(μ·6)`, 끝에 `+= (n+n)·μ`.
7. G4 행 연결: 비지 않은 행마다 `+= (행 길이−1)·μ`, 끝에 `+= (3m)·μ`.
8. G5 문제 유형(+0x08 = 0에서 시작, ±1 수는 D 값 기준): 최대 열 길이 ≥ 5이고 +1·−1 둘 다 있고 다른 값 없으면 `=` 행 수 세기 `+= m·μ`, 2·eq > m이면 1. 최대 열 길이 ≤ 2이고 다른 값 없고 +1 수 = −1 수면 앞쪽 `=` 행 수 c, `+= c·μ`, c = m이면 4.
9. G6 (G1 조건일 때) 목적 검사: 열마다 |c| ≥ 1e-10이고 고정 아님·빈 열 아님일 때 정수 열이면 |c|를 모으고, 연속 열이면 모델 항목을 훑는다(+0x390 == 1인 `=` 행이 있으면 통과, 1이 아닌 행이 있으면 실패, 모두 1인 `<` 행이면 bound가 정수여야 통과). 훑은 항목마다 `+= (k·3)·μ`. 실패하면 거기서 끝. 끝에 `+= (j·7)·μ`(j = 멈춘 열 또는 n). 모두 통과하고 모은 값이 있으면 `a7b5d4`(최대)·`a7bea8`(gcd)로 +0x498/+0x4a0, 그리고 gcd = 1·모은 수 = 4(원본 그대로)·n, m > 500·±1 아닌 값 없음 등이면 유형 11.
10. 공통 꼬리(LP와 같음): `>` → `<` `+= m·μ`, 행 개수 `+= (m+m)·μ`, 열마다 `+= (len·μ)·2`, `+= (3n)·μ`. PREINFBND/PREINFOBJ는 무한(건너뜀).
11. 추적기 `534a8b`: 레벨마다 `+= m·(μ·3)`, `+= (μ·3)·n`, 끝에 `+= 48μ`.
12. MIP만: model+0x258 행 표시가 −1인 행 삭제(지금 모델은 표시 없음, 포팅 안 함). +0x1e4 = (OptimalityTarget ≠ 1).

## 3. `3cd406` 앞부분 (decompiled L731–1745)

`Model.presolve()`는 `999702` → `3db19d(model, PREPROBE, mipctx=0, timer)` → `3cd406(model, probe=PREPROBE, mipctx=0, forSolve=0, …)`. `999702`는 그동안 PRERESTART(env+0x25c8)를 0으로, `3db19d`는 AggFill −1을 0으로 둔다.

1. `3a484d`. `5b84b1(env, state+0x3f0)`(0x48바이트 0 블록), +0x450 = 시작 work, `3ba5ac`(콜백), `3ba9de`(한도). 상태 변화 없음.
2. mipctx 없음: GURO_PAR_DISTRIBUTED > 1이면 마무리. PrePasses −1이면 Presolve 2 → 2e9, 1 → 20, 그 밖 100(`Run::passes`).
3. **정수 bound 반올림**(L800–868, `State::round_integers`): 연속이 아닌 열마다 +0xf4++(이진이면 +0xf0++). lb == ub면: 이진이 [−tol, 1+tol] 밖이거나 `tol < l − floor(l+tol)`이면 불가능(+0x470 = j, 상태 3). `floor(l) ≠ l`이면 그 열은 'C'가 된다. 아니면 lb = ceil(lb − tol), ub = floor(ub + tol), 이진은 [0,1]로 자르고, 정수가 [0,1]이면 +0xf0++, 'B'. 작업량 `+= (처리한 열 수·3)·μ`. **카운트 1.**
4. Presolve ≠ 0일 때만: 이진 ≥ 0.99n이고 m > 1.2n이면 `=` 행을 세고 `+= m·μ`, eq·1000 > m이면 +0xf8 = 1. n > 10000, n/50+1 < 이진 < n/5, probe 1, PREPROBE 1이면 이진 열 길이 합이 max(0.5, 5·이진/n)·nnz보다 크고 비용 있는 이진이 50개 미만·그중 긴 것(> 20) 20개 미만일 때 probe = 3(연속·정수 열에 비용이 있으면 중단). `+= (μ·3)·j`. (이 큰 모델 분기는 크기 제한 라이선스 때문에 오라클로 확인 못 했다.)
5. 작업 한도(L1522, `Run::limit`): base = nnz(+Q·SOS·QC·일반 제약 크기, 지금 0). Presolve 2·PrePasses > 1·PREPROBE > 1·PreSparsify > 0이면 1e100, Presolve 1이면 ×500, 그 밖 ×2500, 그리고 +1e7. reduction 타이머 28개가 이 값(또는 ×0.5, ×2, ×4, 조건부 ×3)으로 잡힌다.
6. `36d89a` 빈·자유 행(`empty_and_free_rows`): 빈 행은 rhs가 FeasibilityTol을 어기면 불가능, 아니면 −1. `<` 행 rhs ≥ 1e20이면 노드 열을 −1로, 열 표시(`5379f1`), 열 개수 −1, `+= (행 개수·3)·μ`, 행 −2. 끝에 `+= (3m)·μ`. **카운트 2.**
7. PWL 함수가 있으면 `9f566a`(카운트). `370047(state, env, 0)` 전체 activity(`+= μ·7`/방문 노드, `+= (μ·8)·m`). `3ba5ac`.
8. **Presolve == 0이면 마무리로.**
9. 0.2n ≤ local_91c(늘 0)이면 `3ff744` mode 0(카운트): n = 0일 때만. `3ba9de`. 유형 11이면 0으로 되돌리고, DualReductions·SOLREDUCTION·해 정보가 있으면 `5b58f4`(카운트).
10. PREBOUND·DualReductions면 **`5a9282`(카운트 3)**. 그다음 Q(`42872b`, `429e1b`)·QC(`451195`)·NL(`5b1161`) 블록(각각 카운트), `3ba9de`.
11. 라운드 0, 패스 0, 내부 루프 i = −5: 스냅숏 `3c546c`, 추적기 레벨 2(`536971`), `5369b8`, `3ba5ac`, `3ba9de`, 그리고 **`3e39a6`**(호출 지점 0x3b1dd0, 기본값에서 카운트 4).

포트는 10의 `5a9282`(또는 PREBOUND 0이면 11의 `3e39a6`) 바로 앞에서 PRELIMIT에 닿은 것처럼 마무리로 간다. PREBOUND 0일 때 `3e39a6` 호출 시점의 work·카운터·추적기 레벨이 38/38 같다.

## 4. PRELIMIT 카운트 위치 (기본값, `local_68`을 state+0x1dc와 비교)

| k | 끝난 호출 | 근거 |
|---|---|---|
| 0, 1 | 1 = 정수 bound 반올림 루프(L869) | k = 0도 첫 비교가 루프 뒤라 1과 같다. lim0 = lim1(모델·작업량) |
| 2 | 2 = `36d89a`(L1606) | lim2 − lim1 작업량 = 3m·μ |
| 3 | 3 = `5a9282`(L6087, PREBOUND && DualReductions) | lim3이 gmu-35-40/50만 바꾼다. PREBOUND 0·PRELIMIT 3은 다른 변화(`3e39a6`) |
| 4 | `3e39a6` 첫 호출 | L1745 이후 첫 카운트 |

조건부로 끼는 카운트: PWL `9f566a`(7번), `3ff744` mode 0(n = 0), 유형 11 + 해 정보 `5b58f4`, Q·QC·NL 블록. 지금 38개에서는 하나도 없다. Presolve 0의 마무리 경로는 카운트 2 뒤다.

## 5. 마무리 (`LAB_004b2415`, L9601–9766)와 모델 생성

1. PWL(`9ee0cf`), NL(`b6fd6f`, `b84382`, `b832dc`), QC(`451195`, `586049`) 블록: 선형 모델에서는 지나간다(포팅 안 함).
2. `538353`: 추적기 현재 레벨의 행 원시 목록(지운 행 포함)마다 rhs가 정수에서 1e-13 이내면 그 정수로(timtab1 c32의 −2.84e-14 → 0). `+= (k+k)·μ`.
3. `5383d0`: 열 원시 목록마다 lb, ub, obj를 같은 규칙으로. `+= (μ·4)·k`.
4. `36d89a` 한 번 더(빈 행이 여기서도 지워진다: lim0의 neos859080도 4행 제거).
5. OptimalityTarget 1·QC면 `4ca023`(포팅 안 함).
6. `3b0917`(MIP 상태, +0xe0 = 0): 살아 있는 행·열을 원래 순서로, 열마다 **열 리스트를 따라** 열 번호 ≥ 0인 노드만(행 생존은 보지 않는다: 지운 행의 노드는 이미 열 −1), 열 길이는 +0x68(+ 분할 행 보정, 지금 0), 타입은 +0x150. 이름 `<모델 이름>_pre`. 전부 지워지면 LP처럼 `Presolved`(최소화형 상수). +0x498/+0x4a0은 새 모델의 D+0x68/+0x70으로 간다(MPS에는 안 나온다).
7. `3b0917` 뒤: QP/QC/PSD/PreQLinearize/MIQCP·일반 제약 블록, `a71757`(AGEQ를 썼을 때만), 로그, `3b8cf6`. 상태 3(불가능)이면 모델 생성을 건너뛰고 DualReductions면 4로 낮춘 뒤 `Model.presolve()`가 errno −1 "Unable to create presolved model"로 실패한다(합성 모델로 확인). forSolve = 0이라 정수 열이 남지 않은 결과는 LP 모델이 된다(L10193).

## 6. MPS 정수 규칙 (`a4590c`)

- MIP(`a7f063`: 정수 열이 하나라도 있음)일 때 COLUMNS에서 'B'/'I'/'N' 열의 연속 구간 앞에 `    MARKER    'MARKER'                 'INTORG'`, 끝에 `'INTEND'`('C', 'S'는 밖). 마지막 열이 정수면 끝에 INTEND.
- BOUNDS(MIP): 'B'는 [max(lb,0), min(ub,1)]로 잘라 쓴다. lb == ub면 `FX` 다음 `BV`, 아니면 `BV`를 쓰고 [0,1]이 아니면 이어서 LO/UP. 'I'이고 ub ≥ 1e30이면 `LI lb`(lb가 0이어도). lb ≤ −1e30이면 FR 또는 MI+UP. 'S'/'N'은 FX 대신 LO와 `SC ub`. 나머지는 LP 규칙 그대로.

## 7. reduction을 붙이는 법 (공개 API)

- `rsolver_mip::presolve(&Model, &Params) -> Result<Model, i32>`: `State::new` → `prelude` → `finish` → `State::to_model`. Err(3)이면 불가능.
- `prelude(&mut State, &Params) -> Result<Run, i32>`: `Run { calls, passes, probe, limit }`은 `3cd406` 지역 변수(local_68, local_928, local_58, local_c90). reduction 하나가 끝날 때마다 `if run.counted(&st) { return Ok(run) }`(= `local_68++`, `state+0x1dc <= local_68`이면 마무리). 첫 reduction은 `prelude` 끝의 `5a9282` 자리(PREBOUND·DualReductions)이고, 그다음 내부 루프 `3e39a6`가 이미 레벨 2·reset까지 맞춰 둔 상태에서 들어간다.
- 상태는 지연 삭제: 노드 열 −1, 행·열 개수 음수. 모델 생성은 열 리스트 순서를 쓰므로 노드를 옮기는 reduction은 free/pending 리스트까지 원본대로 흉내 내야 한다(LP 집계 `crates/lp/src/presolve/agg.rs`의 `Mat`과 같은 방식).
- 검증 도구: `tools/presolve_oracle.py --param GURO_PAR_PRELIMIT=k [--src DIR]`, `tools/work_oracle.py --mip`(호출 지점 0x3b2960), py `CoreModel.mip_presolve_state(params, finish)` = (work, +0x08, +0xf4, +0xf0, +0xf8, +0x404, +0x408, 추적기 레벨). 다른 호출 지점 훅은 `work_oracle.py`의 site·서명만 바꿔 쓴다(이번에 0x3b1dd0 `3e39a6(state, env)`).

## 8. 지름길과 열린 문제

- `shortcut:` 목적 gcd `a7bea8`(+0x498/+0x4a0, 유형 11). `Model.presolve()`에서는 유형 11이 reduction 전에 0으로 돌아가고 두 값은 MPS에 안 나온다. 재시작 presolve(MIP 문맥)나 이 값을 읽는 reduction을 옮길 때 포팅.
- `shortcut:` SOS, semicontinuous(assert), Q, QC, 일반 제약, PWL, NL, MINBPFORBID(+0x170 0x40), model+0x258 행 표시, MIP 문맥(재시작 블록 L1022, cutoff). 마무리의 해당 블록과 `3b0917` 뒤 처리도 같다.
- `shortcut:` 불가능 결과는 Gurobi처럼 실패만 하고 상태 4(INF_OR_UNBD)/3 구분은 돌려주지 않는다.
- n > 10000 큰 모델 probe 분기는 크기 제한 라이선스(변수 2000개) 때문에 오라클로 확인하지 못했다.
- 확인 못 한 것: +0x390(G1/G3) 값 자체는 work로만 간접 확인했다. ptype 1/4와 +0xf8은 합성 모델 각 1개로만 확인했다.
