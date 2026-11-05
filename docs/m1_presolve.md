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

## 4. 일치율

| 설정 | 일치 | 비고 |
|---|---|---|
| Presolve=0 | 64/64 | 상태 생성의 `>` → `−1·<` 변환만 |
| 기본값 | 0/64 | 진행 중 |
