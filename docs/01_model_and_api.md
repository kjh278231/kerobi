# 01. 모델 자료구조, 파라미터, Python API

포팅 원본: [port_map.tsv](port_map.tsv)의 `01_model_api`(569개, optimize 경로 305개)와 gurobipy Cython `_core`(노트 01, 02). 근거: 노트 01, 02, 11 §4(model/env/M 오프셋), 17 §4(genconstr/NL 구조체). 오프셋은 Rust 필드에 대응시키고 필드 doc comment에 남긴다([10 §3](10_porting.md)).

## 1. 크레이트 `model`

### 1.1 `Model` (사용자 모델, 원래 공간)

```rust
pub struct Model {
    pub name: String,
    pub sense: Sense,                 // Min | Max (엔진은 Min으로 변환해서 받음)
    pub cols: Cols,                   // n: obj, lb, ub, vtype, name
    pub rows: Rows,                   // m: lo, hi, name  (sense/rhs/range는 API에서 변환)
    pub a: SparseMat,                 // CSC + 지연 CSR 캐시
    pub q_obj: Option<SymSparse>,     // ½xᵀQx 의 Q (대칭, 상삼각 저장)
    pub q_rows: Vec<QuadRow>,         // xᵀQ_k x + aₖᵀx {≤,=,≥} b
    pub sos: Vec<Sos>,                // type 1|2, vars, weights
    pub genconstrs: Vec<GenConstr>,   // §1.3
    pub pwl_obj: Vec<Option<PwlObj>>, // 변수별 PWL 목적
    pub multiobj: Vec<ObjN>,          // priority, weight, reltol, abstol, con, name
    pub scenarios: Vec<Scenario>,     // 시나리오별 obj/lb/ub/rhs 덮어쓰기
    pub starts: Vec<MipStart>,        // 부분 시작해 (NaN = 미지정)
    pub hints: VarHints, pub priority: Vec<i32>, pub partition: Vec<i32>, pub lazy: Vec<i32>,
    pub warm: WarmStart,              // VBasis/CBasis, PStart/DStart
    pub pending: Pending,             // update() 전 지연 변경 (UpdateMode)
    pub result: Result,               // §1.4
}
```

- **지연 갱신**(노트 02 §단계 2, 11 §3.17): `addVar/addConstr`는 `pending`에 쌓고, `update()` 또는 `optimize()` 진입에서 반영한다. gurobipy의 UpdateMode=1 동작(추가 직후 참조 가능, 속성 질의는 update 뒤)을 따른다.
- `SparseMat`: CSC 본체(`col_start: Vec<i64>`, `row_idx: Vec<i32>`, `val: Vec<f64>`)와 필요할 때 만드는 CSR 캐시. 64비트 nnz. 값 0은 저장하지 않는다.
- 지문(`Model fingerprint: 0x%08x`): 원본 지문 함수를 포팅해 **같은 값**을 낸다. 지문이 같으면 같은 모델을 같은 순서로 읽은 것이고, 이것이 패리티 P0의 첫 검사다. 계산 함수는 `Fingerprint` 속성을 쓰는 `9828a1`·`c23877` 주변에서 찾는다.

### 1.2 `Env` / `Params`

- 파라미터는 **이름 → (타입, 기본값, 범위, 설명)** 표 하나로 정의한다(`params/table.rs`, `gurobipy/_paramdetails.py`의 214개를 기준). 접근은 `env.get::<i32>("Method")`가 아니라 **정적 필드**다: `env.p.method`, `env.p.cuts` …(빌드 스크립트가 표에서 구조체와 이름 사전을 생성).
- 숨은 파라미터(`GURO_PAR_*`)는 **전부** 내부 필드로 둔다(예: `pre_bound`, `lu_interval`, `sb_max_iters`, `aspire`, `min_root_cut_passes`, `plp_concurrent`). 표는 등록 함수 `ac81d7`을 포팅해 만든다: `re/notes/env_param_offsets.tsv` 198개에 빠진 OBBT(env+0x2808, 노트 17 §7.3)를 더한다. 숨은 파라미터의 기본값은 정적 표(env+0x23e0 → 표+0x28, 노트 15 §6.2-8)에서 읽어 옮긴다. `setParam`으로는 설정할 수 없고 테스트는 `Env::internal_mut()`로 만진다.
- **드라이버는 한 번의 optimize 동안만 유효한 자동 조정**을 한다(노트 11 §3.1). 구현: `let p = env.p.clone(); adjust(&mut p); run(&p)`. 원본은 손대지 않는다. 숨은 "env+0x2728 sub-solve 깊이"는 `p.subsolve_depth`로 둔다.
- `Env`에는 로그 싱크(콘솔/파일/콜백), 종료 플래그(`Arc<AtomicBool>`, `terminate()`가 세움, 다목적·sub-MIP 자식 env와 공유), 난수 seed, 스레드 풀 핸들이 있다.

### 1.3 일반 제약, NL 표현식 (노트 17 §4)

```rust
pub enum GenConstr {
    Max { res: Var, args: Vec<Var>, constant: f64 },  Min {..},  Abs { res, arg },
    And { res, args }, Or { res, args },
    Norm { res, args, which: NormKind /* L0 L1 L2 Linf */ },
    Indicator { bin: Var, bin_val: bool, row: LinRow, sense, rhs },
    Pwl { x: Var, y: Var, pts: Vec<(f64, f64)> },
    Func { x, y, kind: FuncKind /* Poly(coefs) Exp Expa(a) Log Loga(a) Pow(a) Sin Cos Tan Logistic */, opts: FuncOpts },
    Nl { res: Var, expr: NlExpr },
}
pub struct NlExpr { pub opcode: Vec<u8>, pub data: Vec<f64>, pub parent: Vec<i32> }  // 전위 배열, parent[0] = -1
```

- NL 검증(`9cc65e`와 같은 규칙): 노드 ≥ 1, opcode 0..19, VARIABLE의 data가 유효 열 인덱스, CONSTANT NaN 금지, `0 ≤ parent[i] < i`.
- opcode 20종은 gurobipy `GRB.OPCODE_*`(CONSTANT=0 … SIGNPOW=19)와 같은 번호를 쓴다. `nlfunc.py`의 연산자 오버로딩이 이 배열을 만든다.

### 1.4 결과와 속성

```rust
pub struct Result {
    pub status: Status, pub runtime: f64, pub work: f64,
    pub x: Option<Vec<f64>>, pub pi: Option<Vec<f64>>, pub rc: Option<Vec<f64>>, pub slack: Option<Vec<f64>>,
    pub vbasis: Option<Vec<i8>>, pub cbasis: Option<Vec<i8>>,
    pub obj_val: f64, pub obj_bound: f64, obj_bound_c: f64,
    pub iter_count: f64, pub bar_iter_count: i64, pub pdhg_iter_count: f64, pub node_count: f64,
    pub pool: SolutionPool,            // SolCount, Xn/PoolObjVal
    pub farkas: Option<Vec<f64>>, pub unbd_ray: Option<Vec<f64>>,   // InfUnbdInfo
    pub kappa: Option<f64>, pub max_vio: Option<f64>,
    pub concurrent_win_method: Option<i32>,
    pub multiobj: Vec<ObjNResult>, pub scenarios: Vec<ScenNResult>,
}
```

- 속성 접근은 **이름 사전 → getter/setter 함수 포인터** 표(`attrs/table.rs`)로 한다. gurobipy `_attrutil`의 역할. 모델/변수/제약/SOS/genconstr/QC 스코프별로 나눈다.
- `ObjBound`는 정수 목적 간격 반올림을 거친 값, `ObjBoundC`는 거치지 않은 값(노트 15 §2 `5e8239` k=2/k=0).

### 1.5 파일 IO

- 읽기: MPS(고정·자유, `.gz`, RANGES/BOUNDS/SOS/OBJSENSE), LP 형식. 쓰기: MPS, LP, SOL, `.ilp` 는 비목표.
- 읽기는 원본 리더를 포팅한다(port_map에서 "MPS read error" 문자열을 쓰는 `unreached` 함수들). 열·행 생성 순서, 중복 계수 합치기, 0 계수 처리가 같아야 지문과 presolve 순서가 맞는다.
- `re/bench/data/netlib`, `miplib`의 `.mps.gz`가 첫 테스트 입력이다.

## 2. PyO3 경계 (`crates/py`)

| 항목 | 설계 | 근거 |
|---|---|---|
| 핸들 | `#[pyclass] PyModel { inner: Arc<Mutex<Model>> }`, `PyEnv`. Python `Model`은 이 핸들을 `_cmodel`로 가진다 | 노트 01 §2 (`_util._ptr(self._cmodel)`) |
| optimize | `py.allow_threads(\|\| driver::optimize(&mut model, &cb))`. 콜백이 있으면 `cb`가 `Python::with_gil`로 사용자 함수를 부른다 | 노트 02 §해석 |
| 콜백 | `where` ∈ {POLLING 0, PRESOLVE 1, SIMPLEX 2, MIP 3, MIPSOL 4, MIPNODE 5, MESSAGE 6, BARRIER 7, MULTIOBJ 8, IIS 9, PDHG 10}. `cbGet(what)`, `cbCut`, `cbLazy`, `cbSetSolution`, `cbUseSolution`, `terminate` | 노트 14 §2 `9a91f3`, 16 §3.7 |
| 콜백 예외 | 사용자 콜백이 예외를 던지면 `terminate`를 세우고 솔브 뒤 `GurobiError`로 다시 던진다 | 노트 02 §콜백 경로 |
| 일괄 API | `add_vars(obj, lb, ub, vtype, names)`, `add_rows_csr(ptr, idx, val, lo, hi)`, `get_attr_array(name, scope, idx)`. 모두 numpy 버퍼 프로토콜을 받는다 | `_lowlevel`, `_matrixapi` 역할 |
| 오류 | Rust `SolverError{code, msg}` → Python `GurobiError(errno, message)` | 노트 02 §9단계 |
| 로그 | Rust 로그 싱크가 Python `LogCallback`/파일/stdout으로. `OutputFlag`, `LogToConsole`, `LogFile` | 노트 11 §3.17 |

## 3. Python 레이어 (`python/rsolver/`)

gurobipy 공개 API 중 다음을 1차 범위로 한다(노트 01 §1의 모듈 목록 순서).

| gurobipy 모듈 | rsolver 파일 | 1차 범위 |
|---|---|---|
| `_grb.py`, `_attrconst`, `_paramconst`, `_statusconst`, `_errorconst`, `_callbackconst` | `grb.py` | 상수 전부 (생성 스크립트로 `_paramdetails.py` 이름 목록에서 뽑음) |
| `_core` (Env, Var, Constr, LinExpr, QuadExpr, GenExpr, tupledict, callbackstub) | `crates/py`의 PyO3 클래스 + `core.py` 얇은 래퍼 | Var/Constr/SOS/QConstr/GenConstr 핸들, LinExpr/QuadExpr/TempConstr, `tupledict.sum/select/prod`. 원본이 Cython이라 PyO3로 옮겨야 모델 구성 속도가 맞는다([00 §6-8](00_overview.md)) |
| `_model` (Model) | `model.py` | `addVar(s)`, `addConstr(s)`, `addSOS`, `addQConstr`, `addGenConstr*`, `addMVar/MConstr`(2차), `setObjective(N)`, `setPWLObj`, `update`, `optimize(callback)`, `read/write`, `getVars/getConstrs/getRow/getCol`, `remove`, `copy`, `relax`, `fixed`, `presolve`, `computeIIS`(비목표), `setParam/getParamInfo`, `Params`, `terminate`, 속성 `__getattr__` |
| `_matrixapi` (MVar, MLinExpr, MQuadExpr) | `matrix.py` | numpy·scipy.sparse 기반 2차 범위 |
| `_helpers` (quicksum, multidict) | `helpers.py` | 1차 |
| `nlfunc` | `nlfunc.py` | `sin/cos/exp/log/... (Var|LinExpr)` → `NLExpr` 트리 → opcode 배열 |
| `_exception` | `exception.py` | `GurobiError(errno, message)` |

- `Var.X`, `Constr.Pi`, `Model.ObjVal` 같은 속성은 `__getattr__` → `_core.get_attr(name, scope, index)` 한 경로로 간다(`_attrutil` 역할). 해가 없으면 `GurobiError(10005, "Unable to retrieve attribute 'X'")`.
- `Model.Params.Method = 1` 와 `m.setParam("Method", 1)` 둘 다 지원. 이름은 대소문자 무시.
- `optimize()`: `update()` → `_core.optimize(self._cmodel, callback)` (GIL 해제) → 반환 코드가 0이 아니면 `GurobiError`.

## 4. 1차 범위에서 뺀 API

`Model.tune`, `computeIIS`, `feasRelax`, `Batch`, `Env(computeserver)`, `linearize`(2차 범위), `singleScenario`, `cbStopOneMultiObj`, `getTuneResult`, `discardConcurrentEnvs`(concurrent env는 3차).

## 5. 테스트

- `tests/test_api.py`: gurobipy 예제(`ref/or-tools/examples/python`의 LP/MIP와 gurobipy `examples/python`의 `mip1`, `qp`, `sos`, `genconstr`, `piecewise`, `multiobj`, `callback`)를 그대로 돌려 결과 값 비교.
- `tests/test_attr_param.py`: 파라미터 214개 set/get round-trip, 범위 밖 값은 `GurobiError(10002)`.
- Rust: `model` 크레이트에 MPS round-trip 테스트(Netlib 104개 읽기 → 쓰기 → 다시 읽기 → 동일).
- 패리티 P0: Netlib 64개와 `miplib_small` 38개의 지문이 Gurobi 로그의 지문과 같다. 파라미터 214개의 기본값·범위가 gurobipy `getParamInfo`와 같다.
- 패리티 P3(모델 구성): `ref/or-tools`와 gurobipy 예제 중 큰 모델 구성 스크립트의 `addVars`/`addConstrs`/`quicksum` 시간이 gurobipy의 1.2배 이내.
