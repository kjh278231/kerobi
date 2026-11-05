# M1 LP presolve 작업 노트 (`390989`)

LP presolve와 postsolve 포팅 중에 알아낸 사실. 확정된 내용은 [03](03_presolve.md)로 옮긴다. 작업 방식은 [10 §9](10_porting.md)의 재구성 + 오라클 비트 검증이다.


## 0. 현재 상태 (2026-11-06, G2 완료)

### G2 기준별 상태
| 기준 | 상태 | 근거 |
|---|---|---|
| (1) P0 기본값 presolved 모델 64/64 | **충족** | `tools/presolve_check.py <def 오라클>` match 64/64(`af8f2c2`). PRELIMIT 체크포인트 표본 15개(k = 18–92) × 64 전부 일치. 덤으로 MIPLIB small 38개 LP 완화 38/38, 합성 대칭 LP 7/7, sym_paths 1/1 |
| (2) postsolve 64/64 비트 일치 | **충족** | `tools/postsolve_check.py <기본값 uncrush 캡처>` match 64/64(5개는 Gurobi도 uncrush 없음, 항등) |
| (3) 47c445·대칭 게이트 포함 기본값 reduction 전부 | **충족** | `47c445`(`38ea4e7`), 대칭 `5577ba`/`551cc0`(`5ccc35a`, Netlib 호출 21개 local_f4·작업량 21/21) |
| (4) clippy 0, 테스트, 문서 | **충족** | `cargo clippy --release --workspace --all-targets` 경고 0, `cargo test --release --workspace`·pytest 27개 통과. 03 §10, 이 메모, 해독 노트(집계·derived·deprow·symmetry·postsolve) |
| (P2) 작업량 | **기본값 64/64 비트 일치** | `wcmp.py work/def.json` 64/64, PRELIMIT 단계별 증분(`wsteps.py`, k = 1..160) 64개 전부 일치 |

### 이번 세션(2차)에 한 것
- 작업량: 생성자 `5118e6` 문제 유형 스캔(`bc9f229`), `50de61` 항목과 "조인 bound 0이면 즉시 끝" 조기 종료(`d1beaf9`), `516cb4`의 두 번째 `6n`(`c3f9bef`), 집계 전체(aggwork 하위 에이전트, `af8f2c2` 병합).
- 예산 정지 `3ba9de`(`9130f4f`, 대칭은 병합 때 연결): 예산 = +0xd8·1000 + 1e7(Presolve 1이면 ·200, PrePasses > 1이나 Presolve 2면 1e100), 기준 0. `390989` 자신의 세 호출은 예산을 1e100으로 바꿔 두어 멈추지 않고, `47c445`(100바퀴마다)·`3ff744`(쌍 1만 번마다)·`551cc0`(1e7마다)에서만 PRELIMIT이 −1이 된다. 오라클로는 확인 못 했다(Netlib·합성 모두 예산의 34% 이하).
- 대칭 병합(`5ccc35a`): `5577ba`가 열 CSR을 압축하는 상태 변화까지(ganges 후반 작업량 차이 해소). 병합하면서 생성자 유형 5/6의 ub 풀기를 옮겨 sym_cycles·sym_paths·cod105 완화의 1.002 bound 차이를 고쳤고, 행·열이 모두 지워진 결과의 이름을 `Presolved`로 했다(enlight_hard 완화).
- 도구(2차 세션 스크래치 `/tmp/claude-1000/-home-sds-workspace-gurobi-sandbox-rsolver-presolve/4dd2adfa-0778-4c24-bb6b-2ced1e335a0c/scratchpad/`, 커밋 안 함): `work/lim{1..160}.json`·`def.json`(`tools/work_oracle.py`), `wsteps.py`(단계별 증분), `wfirst.py`, `wcmp.py`, `dep_oracle.py`(`47c445` 호출 지점 0x38f28f/0x38df02 전후), `slot_oracle.py`, `file_work.py`, `budget_test.py`.

### G2 이후 추가로 한 일 (2026-11-06)
- 최대화(`04198f7`): presolve는 최소화형(비용·상수 부호 반전)으로 돌고 출력 때 되돌린다. 전부 지워진 결과는 새 최소화 모델 `Presolved`(최소화형 상수). MPS 라이터는 비용을 읽은 그대로 쓴다(리더 규약과 맞춤). 확인: 행이 남는 MAX 모델 1개, 전부 지워지는 모델 3개(min·max, 상수 ≠ 0) Gurobi와 같음.
- 문제 유형 1·4(`58322e7`, assert 대신 포팅): 유형 1은 자동 PreDepRow 켜기와 `50de61` 전체 행 경로. 유형 4(네트워크)는 `390989` 축소 모드 `local_38`(시작 reduction·`51a144` 생략, 패스 10, 내부 1라운드, early 묶음·평행·대칭 생략, 집계는 n < 20m일 때 비율 0.1·첫 호출도 `53c9ee` 목록). PreDepRow 자동값은 reduction 전에 정하고 PRESYMMPASSES > 0이면 0. 확인: 합성 네트워크 LP 3개·±1 LP 2개에서 PRELIMIT=1과 기본값의 모델·작업량이 Gurobi와 같음.
- 큰 합성 등식계(n ≤ 1990, nnz ≤ 12만) 4개에서 기본값 모델·작업량이 Gurobi와 비트 단위로 같음(예산의 약 52%).
- 회귀(`58322e7`): Netlib P0 64/64, 작업량 64/64, postsolve 64/64, MIPLIB 완화 38/38, 합성 대칭 7/7, pytest 27, clippy 0.

### G5에서 바꾼 것 (2026-11-06, `m1-lp-driver`)
- Method·Crossover·SolutionTarget(`state+0x100`) 분기 8곳을 넣었다. Method 2·6 + Crossover 0의 presolved 모델이 8/64 → 64/64. presolved 모델 RNG 상태(`Model::rng`)도 설정한다. 표와 근거는 [m1_driver.md §5](m1_driver.md).
- `reduce`/`finish`가 reduction이 낸 상태(3·5)를 돌려주고 `run_post`가 `390989` 끝 규칙(DualReductions, `a80155`, `a7fc52`)을 적용해 `Post::status`에 담는다([m1_driver §9](m1_driver.md)). postsolve에 기저 없는 uncrush `365159`(`uncrush_nonbasic`)를 더했다([m1_driver §8](m1_driver.md)).

### 남은 것
- **Presolve=1 예산이 틀림(조사 중 멈춤)**: Gurobi는 Presolve=1에서 +0x1f0 = 1e100(+1e7)인데 포트는 200·nnz + 1e7이다. 그래서 Presolve=1·PreDepRow=1의 합성 b12·b14에서 포트만 `47c445`가 예산에 걸려 작업량이 다르다(모델은 같음). `3cd406` L778–790이 PrePasses 자동값을 Presolve=1이면 20으로 정하는데, 그 값이 `390989`의 `1 < PrePasses` 판정에 들어가는지(그러면 1e100) 확인해 `run_post`의 예산 식을 고치면 된다. 재현: 2차 세션 스크래치 `budget_p1.py budget b12 b14`.
- 예산 정지가 Gurobi에서도 실제로 걸리는 사례는 아직 없다(기본값은 예산의 52%까지만 확인).
- Presolve=1·2의 다른 경로는 오라클로 전수 확인하지 않았다(Presolve=2 전용 reduction은 지름길).
- postsolve worktree `../rsolver-postsolve`, 브랜치 `m1-postsolve`(병합됨), pane `w4:pP`/`w4:pQ` 정리는 자동 모드가 거부해 남겨 두었다. 셸 pane `w4:pF`는 1차 세션이 만든 빌드용이다.

### 주의점
- 운영 규칙: 이 브랜치에만 커밋, `crates/lp`의 main 기존 파일 수정 금지(`presolve/`는 이 브랜치 소유), `docs/11_progress.md` 수정 금지, 공유 `.venv` 설치 금지(`tools/build_local.sh` + `PYTHONPATH=$PWD/python`), Gurobi는 한 프로세스(`tools/gurun.sh`).
- 오라클·추적 파일은 세션 스크래치에 있어 새 세션에서는 다시 떠야 한다: 기본값 `tools/presolve_oracle.py <dir>/def`, 체크포인트 `--param GURO_PAR_PRELIMIT=k`(64개 약 10초), uncrush 캡처는 postsolve 에이전트 스크래치(`/tmp/claude-1000/-home-sds-workspace-gurobi-sandbox-rsolver-postsolve/d95408ba-cd48-4d17-a9d4-d68b0caaad89/scratchpad/ordef`, 지워졌으면 `tools/uncrush_oracle.py`로 다시), 작업량 `tools/work_oracle.py`.
- 모델별 PRELIMIT 호출 번호는 `mod.rs`의 `calls += 1` 뒤에 임시 `eprintln!("CALL {} L{}", calls, line!())` 추적을 넣어 얻는다(줄 번호로 reduction을 안다). 커밋하지 않는다.
- 병렬 에이전트는 반드시 worktree를 따로 쓴다. 같은 폴더에서 동시에 cargo 빌드하면 서로 깨진다. 에이전트 pane 입력란에 오케스트레이터가 적어 둔 글이 남아 있을 수 있으니 prompt 전 확인.
- `crates/core`·`crates/py`의 clippy 정리(`85c4f28`)는 main이 이미 바꾼 곳이라 main 병합 때 충돌할 수 있다(main 쪽을 택하면 된다).

### 코드·문서 위치
- 코드 `crates/lp/src/presolve/`: `mod.rs`(순서), `state.rs`(상태·레코드·생성자 작업량), `reduce.rs`, `parallel.rs`, `agg.rs`, `derived.rs`, `deprow.rs`, `postsolve.rs`, `tracker.rs`, `sort.rs`.
- 해독 노트: 집계 [m1_presolve_agg.md](m1_presolve_agg.md), `5b488d` [m1_presolve_derived.md](m1_presolve_derived.md), `47c445` [m1_presolve_deprow.md](m1_presolve_deprow.md), postsolve [m1_postsolve.md](m1_postsolve.md). 요약 [03](03_presolve.md) §10.
- 테스트: `tests/test_presolve.py`(`bench/gurobi_presolve/`), `tests/test_postsolve.py`(`bench/gurobi_uncrush/`).

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
| PRELIMIT 1–17 | 64/64 | 내부 루프 전부(`4fe113`, `4ffe11`, `4ff158`, `50cc90(0)`, 진척도 반복), 바깥 `50cc90(1)`, `50d878`, `38e49a`, `50495e`, `505077`, `3ff744(1)`, `3ff744(0)`, `50fe2b`, 그리고 모델 생성의 PRESPLITROWS |
| PRELIMIT 18–100 | 불일치는 모두 rsolver가 멈춘 뒤 | 18: 59/64, 20: 59/64, 25: 45/64, 30: 27/64, 50: 7/64, 100: 3/64. 도달 범위 검사로 k = 14..100에서 닿는 체크포인트의 불일치가 0임을 확인했다. 동작한 모델 수: `3ff744(1)` 25개, `3ff744(0)` 10개, `50fe2b` 23개(행·열·등식 카운터 기준) |
| 집계 직후(모델별 k) | 64/64 | 2026-11-05. ganges·scrs8은 k > 70 |
| 기본값 | 64/64 | 2026-11-05 |
| PRELIMIT 1–100(전수) | 64/64 | 긴 세 모델은 154까지 |

`mod.rs`의 `reduce()`는 아직 포팅하지 않은 reduction에 닿으면 거기서 멈춘다(PRELIMIT에 닿은 것과 같은 효과). 그래서 PRELIMIT 비교는 멈춘 지점까지 유효하다.

**도달 범위 검사**(2026-11-05): 모델마다 rsolver가 실제로 센 호출 수 c를 알면, k ≤ c인 체크포인트의 불일치만 진짜 버그다. 임시로 `calls += 1;` 뒤에 `RS_TRACE` 환경 변수일 때 `eprintln!("TRACE call {calls} line {}")`을 넣고 빌드해, 모델별 최대 호출 번호와 PRELIMIT k 불일치 목록을 겹쳐 본다(커밋 전에 지운다). 같은 추적에 `rows_removed`·`cols_removed`·`ineq_to_eq`를 찍으면 어느 reduction이 실제로 동작했는지도 보인다. 오라클은 k = 1..100을 떠 두었다(체크포인트 하나에 1초 안팎).

## 7. 다음 단계 (순서대로)

**바깥 패스 순서**(기본 파라미터, `390989` 440–1080줄, 패스 k, 각 호출이 PRELIMIT 카운트 1):
1. 내부 루프가 끝나면 레벨 1. n==0이나 m==0이면 마무리. k>0이고 패스 시작 대비 ρ ≤ 1e-4·max(0,k−2)이면 마무리. 패스 스냅숏을 뜨고 추적기를 reset한다.
2. `50cc90(1)`(포팅), DualReductions면 `50d878`(포팅)와 `38e49a`.
3. `!local_38 && DualReductions && PreCrush<2 && (k<5 || Presolve==2)`이면 `50495e`(QC 없을 때), `505077`, (QC나 Presolve==2이면 `4fe113`), `4ffe11`, `4ff158`(state+0x278==0). 조건이 거짓이면(k≥5) `50495e`만 하고, Presolve==2이면 `4fe113`도 한다.
4. `3ba5ac`/`3ba9de`. PREPARALLEL이면(timer 판정은 늘 참) DualReductions일 때 `3ff744(mode 1, 열)`(Presolve==2면 그 앞에 `51f5ab`), 그다음 DualReductions와 관계없이 `3ff744(mode 0, 행)`. `3ff744`가 0이 아닌 값을 돌려주면 상태 코드로 끝난다.
5. PWL 열(+0x330)이 있으면 인라인 쌍대 고정. LP는 건너뛴다. DualReductions면 `50fe2b`(카운트는 다음 줄에서 +1).
6. Aggregate≠0이고 (k==0 또는 Aggregate>1)이면: state+0x278≠0 && env+0x2728==0 && AggFill>0일 때 `517fa8`, 그다음 `519a52`.
7. PREDERIVED && DualReductions && PreCrush==0이면 `5b488d`.
8. `538f30` 행 목록으로 `4f79a1` activity 재계산(카운트 없음).
9. PRETWOROWS(−1)는 Presolve==2일 때만 `51aba6`. k==0 && PRESYMMPASSES<0이면 대칭 블록 `5577ba`(Symmetry는 노드·작업 한도만 바꾼다. [m1_presolve_symmetry.md](m1_presolve_symmetry.md)).
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

**포팅 완료(2026-11-05, stage 3a–3b)**: `crates/lp/src/presolve/parallel.rs`(`3ff744`), `reduce.rs`의 `cost_tight_rows`(`50fe2b`), `state.rs`의 `adjacent_list`(`53af0f`/`53a487`)와 모델 생성의 분할 열.
- `3ff744`는 state+0xe0 = 1(CSR)이라 콜백 묶음 `508461`/`508879`/`508f07`/`509fd0`(열), `50866d`/`508bcd`/`509742`/`50b4e5`(행), `50c736`(mode 2)을 쓴다. state+0xfc = 0(LP)이라 'P'/'T' 행 표시 블록과 특수 열 거르기는 돌지 않는다. local_f0·iVar41(PWL 분기)은 CSR에서 0이다.
- 후보: mode 1은 `53af0f` 목록에서 열 개수 > 0, mode 0은 `53a487` 목록에서 행 개수 > 1이고 `36dce1`(이미 넘칠 수 없는 행)이 아닌 행, mode 2는 행 0..m 전부에서 개수 > 1, `<`, `36dce1` 아님. 후보가 2개 미만이면 아무것도 하지 않는다. 작업량 `((cnt·20·nnz)/(dim+1))·μ`.
- `53af0f`/`53a487`: 레벨 0이면 레벨 0 목록. 표시 개수·5 ≥ dim이면 살아 있는 전부. 아니면 상대 종류의 목록(`538f30`/`5399fc`, 그 캐시도 채운다)을 구해 Σmax(개수,0)를 보고 dim ≤ 2·표시 + 2·Σ이면 전부, 아니면 표시 항목 + 그 목록의 CSR 이웃. 캐시는 추적기 +0x108(열)/+0xe0(행), `5369b8`가 지운다.
- `ab9789`(간접 정렬): n < 32이면 key[idx]로 Shell 정렬(n < 100이라 quicksort는 안 돈다), 아니면 키를 4번째 인자 버퍼에 모아 `aaedd9`와 같은 quicksort. NaN이 없으면 둘 다 `sort_by_key`(모은 키)와 같은 순서다.
- 그룹 짓기: (1) 해시 정렬, 첫 원소와 해시 차 > 1e-13이면 새 묶음. 크기 2 이상 묶음은 척도 해시를 구해 다시 정렬하고 소그룹(첫 원소와 차 > 1e-10이면 새 소그룹)으로 나눈다. (2) 소그룹 크기 2는 비교 한 번(0이 아니면 가른다), 3 이상은 비교 함수로 Shell 정렬(간격 n/2, ÷2.2, 2 다음 1) 뒤 첫 원소와 비교해 다시 가른다. (3) 3개 이상인 묶음은 cost/척도(열은 obj, 행은 rhs)로 정렬하고, 첫 원소 키 + 1e-10 안의 블록(3개 이상)을 |척도|로 정렬한다(남은 원소가 3개 미만이면 멈춤). (4) 크기 2는 쌍 한 번, 3–6은 모든 쌍(앞 원소가 죽으면 다음 앞 원소로), 7 이상은 앞으로 한 번(이끄는 원소를 짝과 비교하고, 짝이 살아남으면 짝이 이끈다. 행은 이끄는 원소의 |척도| ≤ 1e13이고 (짝 |척도| > 1e13 또는 순위 'P'0 'T'1 '<'2 '='3이 짝이 낮거나, 같고 이끄는 쪽 |척도|가 작으면) 그대로), 뒤로 한 번(끝에서 죽은 원소를 건너뛰고 시작, 맨 앞 원소는 빼고). mode 2는 크기 2 묶음만 처리한다.
- `508f07`/`509742`(비교): 개수가 다르면 개수로(작은 쪽이 −1). 두 척도의 |최대| > 1000이면 둘 다 1000/최대를 곱한다. x의 a/s1을 +0x3b0 scratch에 쓰고, y의 a/s2와 비교해 차가 1e-10을 넘는 가장 큰 인덱스를 위(hi)/아래(lo)로 모은다(scratch에 없는 위치는 부호로). 결과는 hi < lo → 1, 같으면 0, 아니면 −1. 작업량 열 3/6/5, 행 4/4/4.
- `509fd0`(열 쌍, asm으로 읽음. 디컴파일은 점프 테이블을 놓친다): |척도| > 1e13이면 끝. |척도|가 큰 쪽을 x로, r = s_y/s_x. state+0x100==0이면 허용치 1e-9/max(|s|,1)과, 유한 bound가 있는 열만 `4f9fc2`로 함의 bound. 경우: (1) x의 lb가 함의되고(또는 −∞) c_x > c_y/r − 1e-10, (2) x의 ub가 함의되고 c_y/r + 1e-10 > c_x, (3) y의 lb, c_y > c_x·r − 1e-10, (4) y의 ub, c_y < c_x·r + 1e-10, (0) 나머지. 각 경우 r의 부호와 bound 유한성에 따라 한 열을 bound에 고정(`4fa83a`, 레코드 있음)하거나, 비용이 1e-10 안에서 같으면 병합, 100·opttol 넘게 어긋나고 해당 |척도| > 1e-9면 비유계(5, +0x470 = x). 병합(PreCrush==0일 때만): type 4 레코드(x, y, lb_x, lb_y, ub_x, ub_y, r), +0x408++, x 표시, x의 bound를 x + r·y의 범위로(r > 0이면 [lb_x + r·lb_y, ub_x + r·ub_y], 아니면 lb·ub를 바꿔 씀, 무한이면 무한), `4fe49b(y: → 0,0)`, `4fe49b(x)`, c_y = 0, y 개수 −1, y 항목의 행 표시·행 개수 감소, 작업량 len·5, y 슬롯 0.
- `50b4e5`(행 쌍, asm 확인): |척도| > 1e13이면 끝. tol = max(1e-6, feastol)/min(|s|, 1), tol2 = 1e-10/max(|s|max, 1)(NumericFocus 1이면 max(…,10), 2 이상이면 max(10·|s|max, 100)). r2가 `=`이고 r1이 `<`이면 바꿔 a가 `=` 쪽. a가 `=`이면 b가 맞는지(tol) 보고 b를 지우거나 불가능(3, +0x478 = a). 둘 다 `<`: 같은 방향이면 느슨한 쪽을 지우고, 마주 보면 폭 w = 위−아래가 −tol 미만이면 불가능, tol2 이하면 a를 `=`로(type 5 레코드 (a, b, s_a/s_b), a 표시, b 지움), 그보다 넓으면 +0x42c++만. 행 지우기: 열 표시, 열 개수 감소, 행 개수 −2, 슬롯 0, +0x404++, 작업량 len·4.
- PRESPLITROWS(`3b0917`, env+0x258c 기본 −1): +0x42c ≠ 0이고 PreCrush ≤ 1이면 모델 생성 전에 `3ff744(mode 2)`. 시작에 +0x42c = 0. `50c736`: 척도 부호가 반대이고 lo = rhs_b/s_b ≤ rhs_a/s_a − feastol(a는 양의 척도)이면 type 14 레코드(a, b, 번호 k, 항목 개수, rhs_a, rhs_b·s_a/s_b, s_a/s_b, a의 살아 있는 항목), +0x3e0[k] = a, +0x3e8[k] = s_a·rhs_b/s_b, +0x42c++, a 표시, +0x404++, a를 `=`로, b 지움(슬롯은 그대로). `3b0917`은 살아 있는 열 다음에(warm start가 없으면 local_120 = 0) 분할마다 열을 붙인다: 'C', 비용 0, [0, rhs_a − +0x3e8[k]], 행 a에 계수 1. 이름이 없어 MPS에는 `C<열 번호>`로 나온다. Netlib에서는 25fv47, agg, bandm, shell 등이 `3ff744(0)` 직후 체크포인트부터 이 열을 갖는다(25fv47의 1R1039/R1042 쌍이 C1535 ∈ [0, 25]).
- `50fe2b`(asm으로 읽음): `5399fc` 열 목록에서 개수 > 0, 특수 열 아님, ub − 1e-10 ≥ lb인 열. c > 1e-10이면 위쪽, c < −1e-10이면 아래쪽 후보를 막는다. `=` 행이 있으면 건너뛴다(작업량 len·4). a > 0인 `<` 행이 하나뿐이면 위쪽 행, a ≤ 0인 `<` 행이 하나뿐이면 아래쪽 행. 아래쪽은 lb가 −∞이거나 `4f9fc2`의 lo ≥ lb − 1e-9일 때, 위쪽은 ub가 ∞이거나 hi ≤ ub + 1e-9일 때 쓴다(함의 bound는 한 번만 구한다). 둘 다 되면 행 개수가 적은 쪽(같으면 위쪽). 고른 행을 `=`로(+0x400++, 행 표시), 그 방향 bound가 유한하면 0.001·(|b|+1)만큼 늦춘다(`4fe49b` 뒤 대입). 늦추기는 state+0x100 ≠ 0이거나 (INTBASIS ≤ 0 && env+0x2728 ≤ 1)일 때(기본값에서 참). 끝에 작업량 목록 길이·20. 호출 수는 DualReductions가 꺼져도 센다.
- `PostRec`에 type 4(`ParallelCol`), 5(`ParallelRow`), 14(`SplitRow`)를 더했다.

**새로 확인한 사실**:
- `3ff744`의 `+0x42c`는 호출 끝에 max(호출 전 값, 새 값)이 된다(mode 2는 호출 전 값을 0으로 본다). 그래서 바깥 패스 전체에서 "마주 보는 쌍이 한 번이라도 있었다"는 표시로 쓰인다.
- `509fd0`의 `4fa83a` 인자는 (env, state, j, rec=1, xmm0=값, xmm1=INFBOUND)다. `4fe49b`의 다섯 번째 double이 INFBOUND다.
- `519a52` 개요(미포팅, 742줄): 처음에 CSR 상태를 MIP용 연결 리스트(+0x160 노드 풀, 노드 0x20바이트, +8 열, +0xc 행, 행 목록 +0x178의 다음 +0x10, 열 목록 +0x180의 다음 +0x18)로 바꾸는 것으로 보이고, `53c507`(추적기) 뒤 최대 5번 `3ba5ac`, `37f484`(MIP 집계기, 3,928줄, 콜리 20여 개), state+0xe0 = 2, 추적기 원시 목록(`53844d`/`5384ca`, 거르기 `53dc94`/`53dcba`)의 행과 바뀐 열의 행을 모아 `370047`을 부른다. 이번 라운드에 지운 행 수 ≥ m·local_98(0.01, Presolve<2이고 문제 유형 4면 0.1)이고 바뀐 것이 있으면 반복한다. 400–742줄은 할당이 많아 CSR로 되돌리는 부분으로 보이나 확인하지 않았다.

**다음 작업: `519a52`/`37f484`**(기본값의 첫 멈춤 지점. 집계는 Netlib 52개 모델을 바꾼다, 부록 B). 연결 리스트 저장소와 그 순회 순서(삽입 위치)를 먼저 정해야 한다. 그 뒤 `5b488d`(10.5KB), activity 재계산, `47c445`(34KB, LU), 마무리 단계(`51a144`, {`50cc90(1)`, `50d878`} 반복, `50de61` 7.9KB).

**포팅 완료(2026-11-05, stage 4–5)**:
- 집계 `519a52`/`37f484`: [m1_presolve_agg.md](m1_presolve_agg.md)대로 옮겼고 집계 직후 체크포인트 64/64가 첫 시도에 맞았다. 확인한 것: 노드 arena 여유는 `3a484d`의 spare = max(10000, max(m,n), nnz/10)(LP는 model+0x18과 QC 수가 0), +0x198 초기값 = spare. 단계 C의 n1/n2 맞바꿈은 asm(0x389dcc)에서 계수도 함께 바꾼다(디컴파일은 N1을 안 바꾼 것처럼 보인다). B1/B2는 리스트 순서와 activity가 같고 pending·+0xd8만 다르다.
- 집계 뒤: `5b488d`(DualReductions && PreCrush==0, 카운트), `538f30` 행 목록으로 `4f79a1`, PRETWOROWS는 Presolve 2에서만, 대칭 블록(`symmetry.rs`, 열 CSR 압축 포함), `47c445` mode 1·2(pass 0, PreDepRow 비트), 그다음 ρ(마지막 내부 반복 스냅숏 대비) ≤ 1e-6이면 마무리.
- PreDepRow 자동(`390989` L251): Presolve==2이면 3, Presolve==1이면 0, 아니면 Method 2·state+0x100·문제 유형 1이면 3, Method≠6이고 INTBASIS < 1이면 `2.5·n < nnz`(state+0xd8, 시작 시점)일 때 3, 아니면 0.
- 마무리(`LAB_0048dff3`): 추적기 레벨 0과 reset, PREBOUND면 `51a144`(카운트만, 한도 검사 없음). 카운트가 PRELIMIT보다 작을 때만 `LAB_0048e0de`: (QP·QC·PreSparsify==2 블록은 LP 기본값에서 안 돎) {`50cc90(1)`, DualReductions면 `50d878`}을 `50d878`이 +0x408을 늘리는 동안 반복, PREBOUND면 `50de61`, 그것이 행을 지웠고 DualReductions면 `50d878`. 그다음 `3b0917`.
- `50de61`(7.9KB): bound 사본 lc/uc와 activity 사본으로 시작한다. 살아 있는 열 수·1000 ≤ 그 열들의 nnz이면 행 전부를 목록으로 바로 3단계. 아니면 (1) 무한 개수 조건(`inf_min+inf_max ≥ 1`이고 `inf_min ≤ 1` 또는 (`=`이고 `inf_max ≤ 1`))을 만족하는 행의 작업 목록으로, **무한 bound인 쪽만** 함의 bound를 구한다(R = min(max(max−min,1),1e6), |a| ≥ R·1e-10, `=` 행은 max activity로 반대쪽도). 새 bound가 들어가면 그 열의 행 사본 activity를 그대로 더해 고치고(가드 없음) 행을 다시 목록에 넣는다(flag += 2, 다음 라운드는 flag ≥ 2인 행만). (2) 행마다 R = min(max(max−min,1),1e4): `=`는 실행 불가 판정, `<`는 inf_max==0 && max < −ft·R이면 지운다(−2, 열 표시), 나머지는 목록. (3) 그 목록으로 `50108e`(시작에서 bound를 사본에 다시 복사한다: (m+1)·0.3 < nnz·|목록|이면 전부, 아니면 목록 행의 열만. 그래서 (1)의 값은 목록에 안 걸린 열에만 남는다). (4) Method 2·state+0x100·m·0.05 < +0x488이 아니면, 유한 bound가 하나라도 있는 열마다 |lc| < 1e7이면 lb = max(lb, lc − (|lc|+1)·0.001), |uc| < 1e7이면 ub = min(ub, uc + (|uc|+1)·0.001). (5) 행마다 lc/uc로 activity를 다시 더해 실행 불가·넘칠 수 없는 `<` 행 삭제를 판정하고, 남는 행은 새 lb/ub로 activity를 다시 더해 저장한다(가드 없음). 기본값 Netlib 59개가 여기서 bound를 얻는다(예: israel A301 ub 305 → 305.306).

나머지 항목:
- postsolve `363164`/`365159`와 비트 오라클(해·기저). 레코드는 `state.rs`의 `PostRec`에 쌓고 있다(type 1, 2, 3, 4, 5, 7, 8, 14). type 4·14는 `363daa`에서 아직 해독하지 않았다(`35d235`가 4).
- 마무리 전에 clippy 경고(`sort.rs` 부정 비교, `state.rs` 범위 루프, 쓰지 않는 레코드 필드)를 정리한다.
- 숨은 파라미터 PREPARALLEL, PRESPLITROWS, GURO_PAR_INTBASIS는 `Params`에 없고 기본값으로 가정한다.

## 8. 함께 알릴 것 (main 합칠 때)

- `crates/model/src/mps.rs`: 모델 이름을 NAME 줄의 첫 토큰으로 바꿨다(gurobipy `ModelName`과 같게).
- `crates/model/src/mps_write.rs`(새 파일), `crates/py/src/lib.rs`(`to_mps`, `presolve`, uncrush 진입점), `python/rsolver/__init__.py`(`Model.presolve`, `Model.write`).
- 도구: `tools/presolve_oracle.py`, `presolve_check.py`, `uncrush_oracle.py`, `postsolve_check.py`, `build_local.sh`, `gurun.sh`. 테스트 `tests/test_presolve.py`와 고정 파일 `bench/gurobi_presolve/`.
- presolve는 아직 드라이버(`optimize()`)에 연결하지 않았다. 연결과 uncrush 뒤 원래 모델 simplex("Extra simplex iterations after uncrush")는 드라이버(main) 몫이다.

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
14. C: 레벨 0..2마다 `+= m·(μ·3.0)`와 `+= (μ·3.0)·n`(작업량 오라클로 확인, 레벨마다 둘 다). 끝에 C L380: `+= 48.0·μ`

기타: `+0x100` = SolutionTarget==1, 또는 (SolutionTarget ≤ 0 && Method ∈ {2,6} && Crossover==0)이면 1. 그렇지 않고 SolutionTarget<2 && NumericFocus>0이면 2. 나머지는 0. `+0x1e0` = PreCrush, `+0x1e8` = IntFeasTol, `+0x1e4` = (OptimalityTarget ≠ 1), `+0x470..+0x484` = −1. `+0x80/+0x88` 열 배열은 1.0이다.

## 부록 B. `47c445`(PreDepRow)의 영향 범위 (2026-11-05)

PreDepRow=0으로 뜬 오라클(`--param PreDepRow=0`)을 기본값과 비교했다. 결과가 다른 모델은 **degen2, degen3 둘뿐**이다. 둘 다 종속 행을 2개씩 지운다("Presolve removed" 행 수 67 vs 65, 98 vs 96). 나머지 62개는 `47c445`가 돌아도 지우는 행이 없다. 결과가 이산값(어느 행을 지우는가)이라, 피벗 순서가 같아 같은 행을 고르면 P0는 맞는다. 다만 작업량(P2)과 PRELIMIT 카운트는 모든 모델에서 맞춰야 한다.

같은 방법으로 남은 reduction을 하나씩 끄고 기본값과 비교한 결과(다른 모델 수): Aggregate=0(`519a52`/`37f484`) **52개**, GURO_PAR_PREPARALLEL=0(`3ff744`) **33개**, GURO_PAR_PREDERIVED=0(`5b488d`) **0개**. `5b488d`는 Netlib에서 모델을 바꾸지 않지만 PRELIMIT 카운트와 작업량에는 들어간다.
