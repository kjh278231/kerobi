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
| PRELIMIT 1–6 | 64/64 | `5034e9`, `503985`(Netlib에서는 돌지 않음), `51a144`, `4fe113`, `4ffe11`, `4ff158` |
| 기본값 | 2/64 | 진행 중 |

## 7. 다음 단계 (순서대로)

1. `50cc90(mode)`(중복 행·작은 계수, `53763f`·`537da8` 사용) → PRELIMIT 7. 그다음 내부 루프 반복과 진척도 `3c5595`(이미 `State::progress`) → PRELIMIT 8–24(오라클 lim4..lim24는 스크래치에 있으니 다시 뜬다).
2. 바깥 reduction: `50d878`, `38e49a`, `50495e`, `505077`, `51f5ab`(조건부), `3ff744`(평행 행/열, 해시 순서 주의), inline 쌍대 고정(`390989` 627–878줄), `50fe2b`, 집계 `519a52`/`517fa8`, `5b488d`(PREDERIVED), `47c445`(PreDepRow, LU), 마무리 `51a144`·`50cc90(1)`·`50d878`·`50de61`. 각 단계에서 PRELIMIT 오라클로 확인한다.
3. 작업량: 생성자의 작업량 항목(서브에이전트 보고, `3a484d`/`5118e6`/`534a8b`)과 `3ba9de` 예산 정지를 옮긴다. 바깥 reduction의 "timer 사용률 < 0.8" 판정이 work에 달려 있다.
4. `3b0917`의 PRESPLITROWS(기본 −1): 평행 행 처리(`50b4e5`)가 반대 방향 행 쌍을 남기면(`+0x42c`) LP에서도 열이 추가된다. 평행 행을 옮길 때 같이 한다.
5. postsolve `363164`/`365159`와 비트 오라클(해·기저).

## 8. 함께 알릴 것 (main 합칠 때)

- `crates/model/src/mps.rs`: 모델 이름을 NAME 줄의 첫 토큰으로 바꿨다(gurobipy `ModelName`과 같게).
- `crates/model/src/mps_write.rs`(새 파일), `crates/py/src/lib.rs`(`to_mps`, `presolve`), `python/rsolver/__init__.py`(`Model.presolve`, `Model.write`).
