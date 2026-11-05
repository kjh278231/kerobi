# M1 LP presolve 작업 노트 (`390989`)

LP presolve와 postsolve 포팅 중에 알아낸 사실. 확정된 내용은 [03](03_presolve.md)로 옮긴다. 작업 방식은 [10 §9](10_porting.md)의 재구성 + 오라클 비트 검증이다.

## 1. 호출 그래프: presolve는 LP를 풀지 않는다 (2026-11-05)

`re/decomp/callgraph.tsv`에서 `390989`가 닿는 함수는 2,976개다(04_simplex 313개, 05_barrier 223개 포함). 경로를 하나씩 확인한 결과, presolve 경로에서 simplex·barrier·LP 드라이버를 부르는 곳은 없다.

| 정적 경로 | 실제 |
|---|---|
| `088c65`(모델 해제) → `5e1674` → `68b379` → `5fe9cd` → … → `c23877` | MIP 문맥을 해제할 때 남은 해 큐를 처리하는 경로다. presolve가 해제하는 것은 이전 presolved LP 모델(`model+0x1d8`)이므로 돌지 않는다. `3b0917`, `4ba6e0`, `312e55`, `9a4ba6`에서 `c23877`로 가는 경로도 모두 여기(또는 `0edd71` → `5e1674`)를 거친다 |
| `47c445`(PreDepRow) → `9a91f3(…, where=2)`, `0d07d1`(simplex 반복 로그), `474ae5` | `47c445`는 crossover `13f716`, simplex 초기화 `1e7531`과 같이 쓴다. 콜백·로그 분기는 `param_3`(spx 문맥)가 있을 때만 돈다(`local_68 == 2`). presolve는 `47c445(work, state, 0, env, 0, mode 1|2, …)`로 부르므로 들어가지 않는다. presolve 경로는 자체 LU와 MKL `dgetrf`만 쓴다 |
| `312e55` → barrier Cholesky(`2d8e22`, `313802`) | QP의 PSD 검사. LP에는 해당 없음 |
| `9a4ba6` → `98f6d2` → `07d3ce`(스케일링) | PWL 목적 변환. LP에는 해당 없음 |
| `2baa47` → `2d0596` | Method 2/6일 때의 판정 함수 |

- LP 드라이버 래퍼 `ba2e9`가 띄우는 보조 스레드 `ba1df`는 `Model.presolve()`(P0 경로 `999702` → `390989`)에서는 돌지 않는다.
- 뒤로 미룬 것: `47c445`(LU 의존, 34KB). PreDepRow 자동(−1)이면 `Method≠6 && INTBASIS<1`일 때 `nnz > 2.5·ncols`인 모델에서 돈다.

## 2. 도구

| 도구 | 내용 |
|---|---|
| `tools/presolve_oracle.py <dir> [--param N=V ...]` | Gurobi `Model.presolve()` 결과를 `<dir>/<name>.mps`(비압축)로 저장. 숨은 파라미터(`GURO_PAR_PREBOUND` 등)도 설정된다. Gurobi는 한 프로세스만 |
| `tools/presolve_check.py <dir> [--param N=V ...]` | rsolver presolved 모델을 같은 형식으로 써서 줄 단위로 비교. Gurobi 불필요 |

빌드: 공유 `.venv`에 설치하지 않는다. `PYO3_PYTHON=../.venv/bin/python cargo build --release -p rsolver` 뒤 `target/release/lib_core.so`를 `python/rsolver/_core.cpython-313-x86_64-linux-gnu.so`로 복사하고 `PYTHONPATH=$PWD/python`으로 실행한다(`.venv`의 `rsolver.pth`는 main 트리를 가리킨다).

## 3. MPS 쓰기 (`a4590c`, P0 비교 대상)

- 숫자 `a8030d`: `|x| < 2e9`이고 정수면 `%d`, 아니면 `%.15g`, 그것이 왕복하지 않으면 `%.16e`.
- 이름 `9c9635`: 이름 하나라도 비었거나 공백·콜론이 있으면(열은 중복도) 그 종류 이름 배열 전체를 버린다. 그러면 행은 `c<i>`, 열은 `<vtype><j>(<열 길이>)`로 쓴다(`9c8829`, `9c89c5`). forplan이 이 경우다.
- OBJ 행 RHS = `−(objcon − D+0x318)·sense`(asm `0x851d3d`). 열의 목적 줄은 `c·sense ≠ 0`이거나 빈 열일 때 쓴다.
- BOUNDS: `l ≤ −1e30`이면 FR 또는 MI+UP, `l == u`면 FX, 아니면 `l ≠ 0`일 때 LO, `u < 1e30`이면 UP(`u < 0`이고 `l == 0`이면 LO 0을 먼저).
- 모델 이름은 MPS NAME 줄의 첫 토큰이다(gurobipy `ModelName`으로 확인). 리더(`crates/model/src/mps.rs`)를 고쳤다.

## 4. 단계별 오라클: GURO_PAR_PRELIMIT

`state+0x1dc`는 숨은 파라미터 `GURO_PAR_PRELIMIT`(env+0x26ec, 기본 2e9)이다. `390989`는 reduction 호출마다 카운터를 올리고, 카운터가 이 값에 닿으면 마무리 reduction 없이 바로 모델 생성(`LAB_0048ed00`)으로 간다. 그래서 `--param GURO_PAR_PRELIMIT=k`로 뜬 오라클은 **k번째 reduction 직후의 presolved 모델**이다. 한 번에 reduction 하나씩 비트 단위로 맞출 수 있다.

```
S=<scratchpad>; tools/presolve_oracle.py $S/or/lim$k --param GURO_PAR_PRELIMIT=$k     # Gurobi, 64개 약 10초
PYTHONPATH=$PWD/python python tools/presolve_check.py $S/or/lim$k --param GURO_PAR_PRELIMIT=$k
```

호출 순서(기본 파라미터, `390989`): 1 `5034e9`, 2 `503985`, 3 `51a144`(PREBOUND≠0일 때), 그다음 패스마다 내부 루프 한 바퀴가 `4fe113`, `4ffe11`, `4ff158`(state+0x278==0일 때), `50cc90(0)`으로 4씩 늘어난다. 내부 루프는 진척도 `ρ ≤ 1e-11·max(0,i−5)`이면 끝나고, 그 뒤 바깥 reduction(`50cc90(1)`, `50d878`, `38e49a`, `50495e`, `505077`, …)이 이어진다(노트 12 §3.2).

## 5. 상태와 공통 루틴 (포팅 완료분)

코드: `crates/lp/src/presolve/`(`mod.rs` 순서, `state.rs` 상태·공통 루틴, `reduce.rs` reduction, `tracker.rs`, `sort.rs`).

- **생성자 `3a484d`/`5118e6`**: 열 CSR은 모델 열 순서 그대로 복사하고 |a| ≤ 1e-13(NaN 포함) 항목을 뺀다(열 시작 위치는 그대로, 길이만 줄어 구간 끝에 틈). 행 CSR은 전치(행 안은 열 번호 오름차순), 틈 없음. `+0x60/+0x68` = 살아 있는 개수. `+0xd8` = 원래 nnz. `>` 행은 sense만 `<`로 바꾸고(D가 이미 뒤집힌 값을 가진 것으로 보임) 결과는 계수·rhs ×(−1)과 같다. `+0x448` 작업 배수 = nnz ≤ 1e5면 1, 아니면 `sqrt(log10(nnz/1e4))`. PREINFBND·PREINFOBJ 기본값은 ∞라 Model.presolve 경로에서는 bound를 자르지 않는다. 문제 유형 `+0x08`은 일반 LP에서 0(또는 10). 1/4/5/6은 ±1 계수 특수 구조다.
- **지연 삭제**: 행 CSR 항목은 열 번호 −1, 열 CSR 항목은 행 번호 −1로 지운다. 지운 행·열은 개수를 음수로 둔다(−1 빈 행/쌍대 제거, −2 고정·forcing, −3 대입된 행, 열은 −3−r = 행 r로 대입됨). 모델 생성 `3b0917`은 열 CSR을 순서대로 읽어 살아 있는 행 항목만 쓰고, 열 길이는 `+0x68`을 그대로 쓴다. 행·열 순서는 원래 순서 그대로다.
- **activity `4f79a1`**: `+0x90`/`+0x70`은 **max activity − rhs**(유한 부분/무한 개수), `+0x98`/`+0x78`은 min activity − rhs다. 노트 12의 min/max 이름은 반대다. 덧셈이 부정확하면 max는 위로, min은 아래로 1ulp(×(1±2⁻⁵²)) 민다.
- **추적기 `534a8b`(생성), `53763f`/`5379f1`(행/열 표시), `5369b8`(reset), `538f30`/`5399fc`(행/열 목록), `538353`(원시 행 목록)**: 레벨 0/1/2마다 목록과 길이 a(현재)/b(전체)를 둔다. 처음에는 모든 행·열이 모든 레벨에 들어 있고 플래그는 5다. 목록 getter는 표시가 많으면(`ncols ≤ 5·표시열`, `nrows ≤ 4·표시행`, `nrows < 3·표시행 + Σ표시열 개수`) 살아 있는 전부를 번호순으로 돌려준다.
- **정렬 `aaedd9`**: n ≥ 100이면 median-of-3 3-way quicksort(깊이 50), 그 밑은 간격 n/2, ÷2.2(2 다음은 1)인 Shell 정렬. 원본과 같은 동률 순서를 낸다.
- **열 고정 `4fa83a(v, INF, j, rec)`**: 레코드(type 1), objcon += c·v, 열의 행 표시, rhs·양쪽 activity 갱신, 열 −2. `rec=0`이면 레코드 없이 lb=ub=v인 빈 열로 남긴다. `51a144`에는 같은 코드가 인라인돼 있다. `4f6740`(거의 고정된 열)은 activity를 갱신하지 않는다.
- **bound 변경 `4fe49b(옛 lb, 옛 ub, 새 lb, 새 ub, INF, j)`**: 옛 기여를 빼고 새 기여를 더한다(반올림 보정 포함). 둘 다 바뀌면 순서는 ub 빼기, lb 빼기, ub 더하기, lb 더하기.
- **디컴파일에서 빠진 double 인자(asm 확인)**: `4fe113`의 `=` 행 slack 경로는 `4fa83a(mode==1 ? lb : ub)`, `4fd703`의 유한 bound 고정은 `4fa83a(위쪽 ? ub : lb)`, `4ffe11` code 3은 `4fa83a(x)`. `4ffe11`의 새 bound는 lb' = min(x, ub), ub' = max(x, lb')이다.
- env 오프셋: 0x2460 FeasibilityTol, 0x2470 OptimalityTol, 0x2458 INFBOUND(1e30), 0x26c0 GURO_PAR_AGGMULTLIMIT(1e14), 0x26e0 GURO_PAR_INTBASIS(−1), 0x2718 NumericFocus, 0x24d0 FIXTOL(−1, 그래서 `503985`는 n ≥ 10000일 때만 돈다).

## 6. 일치율

| 설정 | 일치 | 비고 |
|---|---|---|
| Presolve=0 | 64/64 | 상태 생성의 `>` → `−1·<` 변환만 |
| PRELIMIT 1–14 | 64/64 | 내부 루프 전부(`4fe113`, `4ffe11`, `4ff158`, `50cc90(0)`, 진척도 반복)와 바깥 `50cc90(1)`, `50d878` |
| PRELIMIT 15–100 | 불일치는 모두 rsolver가 멈춘 뒤 | 15: 63/64(fit1d는 15번째가 `3ff744(1)`), 16: 62/64, 20: 54/64, 30: 25/64, 100: 3/64. 바깥 `38e49a`, `50495e`(18개 모델에서 동작), `505077`(23개), 바깥 `4ffe11`·`4ff158`까지는 닿는 모든 체크포인트가 맞는다 |
| 기본값 | 3/64 | 진행 중 |

`mod.rs`의 `reduce()`는 아직 포팅하지 않은 reduction에 닿으면 거기서 멈춘다(PRELIMIT에 닿은 것과 같은 효과). 그래서 PRELIMIT 비교는 멈춘 지점까지 유효하다.

**도달 범위 검사**(2026-11-05): 모델마다 rsolver가 실제로 센 호출 수 c를 알면, k ≤ c인 체크포인트의 불일치만 진짜 버그다. 임시로 `calls += 1;` 뒤에 `RS_TRACE` 환경 변수일 때 `eprintln!("TRACE call {calls} line {}")`을 넣고 빌드해, 모델별 최대 호출 번호와 PRELIMIT k 불일치 목록을 겹쳐 본다(커밋 전에 지운다). 같은 추적에 `rows_removed`·`cols_removed`·`ineq_to_eq`를 찍으면 어느 reduction이 실제로 동작했는지도 보인다. 오라클은 k = 1..100을 떠 두었다(체크포인트 하나에 1초 안팎).

## 7. 다음 단계 (순서대로)

**바깥 패스 순서**(기본 파라미터, `390989` 440–1080줄, 패스 k, 각 호출이 PRELIMIT 카운트 1):
1. 내부 루프가 끝나면 레벨 1. n==0이나 m==0이면 마무리. k>0이고 패스 시작 대비 ρ ≤ 1e-4·max(0,k−2)이면 마무리. 패스 스냅숏을 뜨고 추적기를 reset한다.
2. `50cc90(1)`(포팅), DualReductions면 `50d878`(포팅)와 `38e49a`.
3. `!local_38 && DualReductions && PreCrush<2 && (k<5 || Presolve==2)`이면 `50495e`(QC 없을 때), `505077`, (QC나 Presolve==2이면 `4fe113`), `4ffe11`, `4ff158`(state+0x278==0). 조건이 거짓이면(k≥5) `50495e`만 하고, Presolve==2이면 `4fe113`도 한다.
4. `3ba5ac`/`3ba9de`. DualReductions && Presolve≠2 && PREPARALLEL이면, timer(local_700) 사용률이 0.8 미만일 때 `3ff744(mode 1, 열)`, 그다음 `3ff744(mode 0, 행)`. Presolve==2면 먼저 `51f5ab`.
5. PWL 열(+0x330)이 있으면 인라인 쌍대 고정. LP는 건너뛴다. DualReductions면 `50fe2b`(카운트는 다음 줄에서 +1).
6. Aggregate≠0이고 (k==0 또는 Aggregate>1)이면: state+0x278≠0 && env+0x2728==0 && AggFill>0일 때 `517fa8`, 그다음 `519a52`.
7. PREDERIVED && DualReductions && PreCrush==0이면 `5b488d`.
8. `538f30` 행 목록으로 `4f79a1` activity 재계산(카운트 없음).
9. PRETWOROWS(−1)는 Presolve==2일 때만 `51aba6`. k==0 && PRESYMMPASSES<0이면 대칭 블록(Symmetry>1일 때 `5577ba`, 세부 미확인).
10. k==0이고 PreDepRow&1이면 `47c445(mode 1)`, &2이고 DualReductions면 `47c445(mode 2)`. PreDepRow 자동값: `2.5·n < nnz`이면 3, 아니면 0(Method −1, INTBASIS −1 기준).
11. `3ba5ac`/`3ba9de`. ρ(마지막 내부 반복 스냅숏 대비) ≤ 1e-6이면 마무리. k+1 ≥ 패스 수여도 마무리. 아니면 다음 패스.

**마무리**(`LAB_0048dff3` → `e07f` → `e0de`): 레벨 0으로 reset, PREBOUND면 `51a144`. 그다음 {`50cc90(1)`, DualReductions면 `50d878`}을 `50d878`이 열을 더 지우지 않을 때까지 반복. PREBOUND면 `50de61`, 그 뒤 행이 줄었고 DualReductions면 `50d878`을 한 번 더 한다. 끝으로 `3b0917`.

**포팅 완료(2026-11-05, stage 2a–2c)**:
- `38e49a`(`cost_point_fix`): 열마다 비용이 고르는 값 x(c>0 → lb, c<0 → ub, c=0이면 lb·ub 중 0인 쪽, 0이 범위 밖이면 가까운 bound). 특수 열, 무한한 x, 범위 안에 0을 품은 비용 0 열이 하나라도 있으면 그만둔다. x에서 모든 `=` 행이 |Σ−rhs| ≤ 1e-10이어야 한다. 그 행들의 열을 x로 옮긴 max activity로 부등식 행을 보고(가드 덧셈), 넘칠 수 있는 행에서 그 열들이 min activity 쪽 bound에 있으면 표시. 모든 `<` 행이 x에서 만족되면 열 전부, 아니면(표시가 참일 때) `=` 행 열만 `4fa83a`로 고정하고 `50cc90(0)`. 작업량은 마지막 열 번호의 2배를 더하는 버릇까지 옮겼다. Netlib 64개에서는 동작하지 않는다.
- `50495e`(`implied_redundant_rows`)와 `50108e`(`propagate_bounds`): 레벨 1 행 목록으로 bound 사본(+0x398/+0x3a0)을 한 번 전파한다(min activity로, `=` 행은 max activity로도. 변화 100·feastol 미만이나 |값| ≥ 1e8은 무시). 사본은 bound에 쓰지 않는다(SolutionTarget==2일 때만 복사, 범위가 −1..1이라 안 일어남). 그 사본으로 행 activity를 다시 더해(가드 없음) 빈 행, 넘칠 수 없는 `<` 행(−2), 만족할 수 없는 행(3)을 가린다. 허용치는 feastol·min(max(max−min, 1), 1e4). `(m+1)·0.3 < nnz·|목록|`이면 전체 복사(작업량 없음), 아니면 목록 행의 열만 복사.
- `505077`(`dual_bound_rows`)와 `4f84ef`(`dual_propagate`): 열 상태 비트(lb 자유 1, ub 자유 2)를 무한 bound로 시작해, `state+0x100==0 && (Method≠2 || Crossover==0 || SolutionTarget>0)`이면(기본값에서는 NumericFocus ≤ 0) 열 개수 오름차순(안정 계수 정렬)으로 행이 bound를 함의하는지 본다(ub 패스, 그다음 lb 패스, 허용치 1e-9, 함의되면 그 행의 무한 개수 사본을 늘림). 상태 → 쌍대 부호 `'='`(3), `'>'`(1), `'<'`(2), `'F'`. 행 쌍대 y는 `<` 행 [−∞, 0], `=` 행 [−∞, ∞]. 열마다 −c+Σa·y의 범위(가드 덧셈)를 잡고 `4f84ef`가 원형 큐로 y 상자를 좁힌다(head==tail이면 멈추므로 마지막에 넣은 열은 처리하지 않는 버릇, 11바퀴 제한). 그 뒤 최대 < −opttol−|c|·1e-13이면 lb, 최소 > opttol+|c|·1e-13이면 ub로 `4fa83a`. y < −feastol인 `<` 행은 `=`(+0x400, 행 표시). Presolve=2 분기('F' 열 고정)도 옮겼지만 오라클은 없다.
- 바깥 패스는 이제 `50495e`, (조건부) `505077`, Presolve=2면 `4fe113`, `4ffe11`, `4ff158`까지 간다.

**새로 확인한 사실**:
- asm 확인: `505077`은 `4f84ef(xmm0=INFBOUND, xmm1=FeasibilityTol, …)`로 부른다. `50108e`의 `--auVar30 / a` 꼴(`=` 행, 무한 1개)은 부호를 두 번 뒤집은 것이라 maxact/(−a)다.
- env+0x2438 = Method, env+0x2ba0 = Crossover, env+0x2b9c = SolutionTarget.
- 숨은 파라미터 기본값(gurobipy `getParamInfo`): PREPARALLEL 1, PREDERIVED 1, PREBOUND 1, PRESYMMPASSES −1, PRETWOROWS −1, PREDUALREDUNDANT 1, Aggregate 1, AggFill −1(`390989`이 env+0x2428을 10, 모델에 +0xf0가 있으면 20으로 바꾼다), PreDepRow −1, Symmetry −1. 그래서 기본값에서 첫 패스는 `3ff744`×2, `50fe2b`, `519a52`, `5b488d`, `47c445`(nnz > 2.5n일 때)를 모두 부른다.
- timer `local_700`은 `ade552`가 0으로 채워 예산(timer[2])이 0이다. `adf236`이 0을 돌려주므로 "0.8 미만" 판정은 늘 참이다. `3ff744`의 `param_3`(timer)는 쓰이지 않는다.
- `LAB_0048db6b`의 +1은 `50fe2b`를 부르지 않아도(DualReductions=0) 센다.
- `5577ba`(대칭, k==0 && PRESYMMPASSES<0, 남은 열 ≥ 500 && 남은 행 ≥ 100일 때)는 PRELIMIT 카운트가 없다.

**다음 작업: `3ff744`**(멈춘 지점. fit1d는 PRELIMIT 15, degen2는 16에서 처음 드러난다). LP 경로만 정리하면:
- state+0xe0 = 1(`3a484d` L382: LP presolve는 param_3==0이고 model+0x18==0). 그래서 CSR 콜백 묶음을 쓴다. mode 1(열): 해시 `508461`, 척도 `508879`, 비교 `508f07`, 쌍 처리 `509fd0`. mode 0(행): `50866d`, `508bcd`, `509742`, `50b4e5`. PREPWLRED/PREOBJPWLRED 분기(`3fa2f6`, `3bbf87`, `3bde19`, `3bea24`, `370047`)는 CSR이 아닐 때만이라 LP에서는 돌지 않는다.
- 후보: mode 1은 `53af0f`(추적기 +0x108 캐시를 쓰는 별도 열 목록 getter, 3.8KB, `5399fc`와 다름)에서 열 개수 > 0. mode 0은 `53a487`(행 판)에서 행 개수 > 1이고 `36dce1`(inf_max==0 && act_max < feastol && (`<` 또는 inf_min==0 && act_min > −feastol), 즉 이미 중복인 행)이 아닌 행. mode 0 시작에 +0x42c = 0.
- `508461`: 살아 있는 행 i마다 1/(i+1.0)의 합. 2개 이상이면 SIMD 두 줄(짝수·홀수 위치)로 따로 더한 뒤 합치고 남은 하나를 더한다(덧셈 순서 주의). 작업량 len·2.
- `508879`: Σ|a|/(i+1.0)를 max|a|로 나눈 값을 돌려주고, +0x398[j]에 (행 번호가 가장 작은 항의 부호)·max|a|를 쓴다. 작업량 len·3.
- 흐름: 해시로 간접 정렬(`ab9789`, key[idx]로 비교하는 별도 quicksort, `aaedd9`와 다르다) → 해시 차 ≤ 1e-13 묶음마다 척도 해시 → 다시 정렬, 1e-10 이내로 소그룹 → 소그룹 크기 2면 비교 함수 한 번, 3 이상이면 비교 함수로 Shell 정렬(간격 n/2, ÷2.2) 뒤 다시 묶음 → 묶음 크기 2는 쌍 처리 한 번, 3–6은 모든 쌍, 7 이상은 앞에서 한 번·뒤에서 한 번 훑기(행이면 'P'/'T'/'<'/'=' 순위와 |척도| 1e13으로 대표를 바꾼다). 작업량 검사(`3ba5ac`, 1만 회마다)는 미포팅 예산과 묶여 있다.
- 그 뒤 순서: `50fe2b`(1.7KB, 부등식 → 등식), `519a52`(4.5KB, 내부에서 MIP 집계기 `37f484` 35KB), `5b488d`(10.5KB), activity 재계산, `47c445`(34KB, LU), 마무리 단계(`51a144`, {`50cc90(1)`, `50d878`} 반복, `50de61` 7.9KB). 마무리는 첫 패스가 끝나야 닿으므로 `3ff744`가 먼저다.

나머지 항목:
- `3b0917`의 PRESPLITROWS(기본 −1): 평행 행 처리(`50b4e5`)가 반대 방향 행 쌍을 남기면(`+0x42c`) LP에서도 열이 추가된다. 평행 행을 옮길 때 같이 한다.
- postsolve `363164`/`365159`와 비트 오라클(해·기저). 레코드는 `state.rs`의 `PostRec`에 쌓고 있다(type 1, 2, 3, 7, 8).
- 마무리 전에 clippy 경고(`sort.rs` 부정 비교, `state.rs` 범위 루프, 쓰지 않는 레코드 필드)를 정리한다.

## 8. 함께 알릴 것 (main 합칠 때)

- `crates/model/src/mps.rs`: 모델 이름을 NAME 줄의 첫 토큰으로 바꿨다(gurobipy `ModelName`과 같게).
- `crates/model/src/mps_write.rs`(새 파일), `crates/py/src/lib.rs`(`to_mps`, `presolve`), `python/rsolver/__init__.py`(`Model.presolve`, `Model.write`).

## 부록 A. 생성자 작업량 항목 (W += …, μ = +0x448, 아직 미포팅)

`3a484d`(A), `5118e6`(B, CSR 생성), `534a8b`(C, 추적기) 순서대로. 2026-11-05 서브에이전트 해독(줄 번호는 각 의사코드 파일).

1. A L962: `+= n`(μ 없음)
2. A L1397: `+= (n·1.5 + m)·μ`
3. A L1562: `+= (n+n)·μ`
4. A L4004: `+= (spare−1)·μ`, spare = nnz/10 > 2e9 ? 2e9 : max(10000, max(m,n), nnz/10)
5. B L219: `+= (n·0.5)·μ`
6. 열 시작이 연속이면 B L240: `+= (n·0.5 + nnz)·μ`. 아니면 열마다 B L208: `+= (len·4.0)·μ`, 그다음 B L210: `+= (n·3.0)·μ`
7. B L287: `+= (nnz·3.0)·μ`
8. B L344: `+= (m·4.0)·μ`
9. B L391, 열마다: `+= (원래 len·6.0)·μ`
10. B L536: `+= (n+n)·μ`, B L546: `+= (m+m)·μ`
11. 유형 1/4/5/6 판정용 추가 스캔: B L612, 626, 725, 837, 855, 887(일반 LP 경로에서 어느 것이 도는지 확인 필요)
12. A L4870: `+= m·μ`, A L4877: `+= (m+m)·μ`, A L4905 열마다: `+= (모델 len·μ)·2.0`, 그다음 A L4880: `+= (n·3.0)·μ`
13. PREINFBND가 켜지면 A L4961: `+= (n+n)·μ`. PREINFOBJ가 켜지면 A L4988: `+= (n·3.0)·μ`
14. C: 레벨 0..2마다 `+= m·(μ·3.0)`, 그다음 `+= (μ·3.0)·n`. 끝에 C L380: `+= 48.0·μ`

기타: `+0x100` = SolutionTarget==1, 또는 (SolutionTarget ≤ 0 && Method ∈ {2,6} && Crossover==0)이면 1. 그렇지 않고 SolutionTarget<2 && NumericFocus>0이면 2. 나머지는 0. `+0x1e0` = PreCrush, `+0x1e8` = IntFeasTol, `+0x1e4` = (OptimalityTarget ≠ 1), `+0x470..+0x484` = −1. `+0x80/+0x88` 열 배열은 1.0이다.

## 부록 B. `47c445`(PreDepRow)의 영향 범위 (2026-11-05)

PreDepRow=0으로 뜬 오라클(`--param PreDepRow=0`)을 기본값과 비교했다. 결과가 다른 모델은 **degen2, degen3 둘뿐**이다. 둘 다 종속 행을 2개씩 지운다("Presolve removed" 행 수 67 vs 65, 98 vs 96). 나머지 62개는 `47c445`가 돌아도 지우는 행이 없다. 결과가 이산값(어느 행을 지우는가)이라, 피벗 순서가 같아 같은 행을 고르면 P0는 맞는다. 다만 작업량(P2)과 PRELIMIT 카운트는 모든 모델에서 맞춰야 한다.

같은 방법으로 남은 reduction을 하나씩 끄고 기본값과 비교한 결과(다른 모델 수): Aggregate=0(`519a52`/`37f484`) **52개**, GURO_PAR_PREPARALLEL=0(`3ff744`) **33개**, GURO_PAR_PREDERIVED=0(`5b488d`) **0개**. `5b488d`는 Netlib에서 모델을 바꾸지 않지만 PRELIMIT 카운트와 작업량에는 들어간다.
