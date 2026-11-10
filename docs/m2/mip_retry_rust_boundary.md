# M2-03d-1 검증 계약과 결과: Rust 재풀이 비교 경계

작성 기준은 [스프린트 검증 기준](../15_sprint_verification.md)이다. 원본 근거는 [M2-03c-2](mip_retry_basis.md)와 [M2-03c-1](mip_linear_child_trace.md) §10–13이다.

## 1. 범위와 주장

- 작업 종류: 조사·오라클 설계. Rust 솔버 코드와 Gurobi 실행은 범위 밖이다. 저장된 원본 캡처만 다시 읽는다.
- 질문:
  1. Rust에서 자식 LP의 Numeric은 어디서 처리되며, 원본 재풀이 사건(nidda·gmu)과 어떻게 대응하는가?
  2. Rust가 원본 두 사례와 같은 입력 상태에 자연 실행으로 도달할 수 있는가? 없다면 비교 경계를 어디에 두는가?
  3. Rust 계측 사건의 형식과 원본 캡처의 투영은 무엇인가?
  4. 필드별 비교 규칙은 무엇인가?
  5. 노드 저장 basis, method `param_3[3]` 규칙, 상태 2 basis 저장에 대응하는 Rust 자료 구조가 있는가? 없으면 무엇이 빠졌는가?
- 판정 규칙(결과 확인 전 고정):
  - 각 질문에 원본 C 줄, 캡처 사건, Rust `파일:줄` 중 하나 이상의 근거로 답하면 조사 완료다.
  - 투영 도구는 저장 캡처 3회에서 같은 결과를 내야 하고, 변조를 모두 거부해야 한다. 그렇지 않으면 미완료다.
  - 조사만으로 Rust 동등성을 주장하지 않는다.
- 선행: main `de9863a`(M2-03c-2 병합 뒤). 미커밋 선행 변경은 없다.

## 2. 원본과 상태 경계

- Gurobi 13.0.3, SHA-256 `8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c`. 주소는 file offset이다.
- 새로 읽은 C(`../re/decomp/c/`):

| 함수 | 줄 | 내용 |
|---|---|---|
| `5ce67b` | 353–361 | 노드 basis(`I+0x98`)나 workspace가 있으면 첫 LP method는 `param_3+0x18`(=`param_3[3]`), 없으면 `param_3+0x10`이다 |
| `5ce67b` | 362–455 | `param_7≠0`이고 SimplexPricing(env+0x2440)=−1이면 적응 선택으로 첫 LP method가 바뀔 수 있다. 재풀이 method는 이 결과가 아니라 `param_3[3]`을 다시 읽는다(C:926) |
| `5ce67b` | 769–786 | 재풀이 진입: LP 12이고 (캐시 2 또는 현재 노드가 루트). FEASHELPER==−2이거나 `local_58+0x1a0≠0`이면 노드 3으로 끝난다 |
| `5ce67b` | 787–923 | `I+0x98==0`, `I+0xa8≠0`, `a7f923≠0` 중 하나면 `0c3d0d` 경로다. Presolve=1, Threads=1, Method=`local_110`(0·1·2 밖이면 1)로 설정하고 뒤에서 복구한다 (gmu) |
| `5ce67b` | 924–933 | 그 밖에는 `method = (param_3[3]==0)`으로 `cda5b(lp, method, I+0x98, I+0xa0, 0,0,0,0)`을 직접 부른다 (nidda) |
| `5ce67b` | 944–1035 | 재풀이 뒤 정규화. LP 2이면서 목적값≥cutoff이면 6이다. LP 4는 캐시에 따라 3·5·12가 되고, 캐시 2에서는 5→12, 4→3이다. 6이면 경우에 따라 0xb가 된다 |
| `5ce67b` | 2244–2257 | 정규화 뒤에도 12이면 `I+0xb0`·`I+0x98`를 해제하고 `I+0x94=0`으로 둔 뒤 `600d25`로 간다(노드 3, [03b-1](mip_unbounded_reclassification.md) 표 64행) |
| `5ff358` | 77–84 | 새 노드에 부모가 있으면 `69e299(child, 0, parent)`로 부모 basis를 복사한다. 분기 `627f3e`가 이 함수를 부르므로 일반 자식도 부모 basis를 물려받는다 |
| `8e4682` | 355 | `5ce67b(&local_78, local_58, local_90, 0, -1, 0, 0, …)`. `param_7=0`이므로 nidda 노드 8287의 첫 LP에는 적응 선택이 없다 |
| `600d25` | — | LP→노드 상태. CUTOFF(6)→5 ([B&B 노트](../../../re/notes/15_mip_branch_and_bound.md) 표 `600d25`) |

- `param_3`은 호출자가 넘기는 구조체다. nidda에서는 `8e4682`의 `local_90`이다. `+0x18` 값 자체는 캡처하지 않았다.

## 3. 결과

### 질문 1. Rust 처리 위치와 원본 대응표

현재 Rust 경로:
- `crates/mip/src/bnb.rs:16` `Node { model, bound }`: basis가 없다.
- `bnb.rs:170` 자식 LP는 매번 `solve_model_deadline(&node.model, 1, …)`로 cold dual이다.
- `bnb.rs:176` `Numeric | Unbounded`이면 노드를 되돌려 넣고 MIP 전체를 `Status::Numeric`으로 끝낸다. 재풀이는 없다.
- 루트는 `crates/mip/src/root.rs:389` `Status::Numeric => MipStatus::Numeric`이다.

| 원본 사건 (nidda 사건 번호) | 원본 근거 | Rust 현재 | Rust 대응 자료 |
|---|---|---|---|
| 노드 basis 수명: export→pack→copy→decode (1–3) | `5d4427`/`113d92`, `69d237`, `69e299`, `69d7ea` | 없음 | `Node`에 basis 필드 없음. `LpResult::basis`(`crates/lp/src/simplex.rs:1677`)는 있으나 버려짐 |
| 일반 자식 basis 상속 | `5ff358` C:77–84 | 없음 | `branch()` `bnb.rs:198–210`이 모델만 복사 |
| 첫 LP: 노드 basis, method `param_3[3]` (4–12) | C:353–361, `5ccea5` | `bnb.rs:170` cold, method 1 고정 | 노드 method 파라미터 없음(NodeMethod는 `crates/model/src/param_table.rs:58`에만 있다) |
| 첫 LP 상태 12 (12–15) | 원본 simplex 수치 실패 | Rust `Numeric`은 primal phase 1의 무계 ratio(`simplex.rs:1415`)에서만 생긴다. 원본 12와 발생 원인이 다르다 | — |
| 재풀이 진입 판정 | C:769–786 | 없음(`bnb.rs:176` 종료) | 캐시(루트 LP 상태)·루트 여부를 노드 처리에서 읽을 수 없음 |
| 재풀이: 노드 basis, method `param_3[3]==0` (16–28) | C:924–933 | 없음 | warm start는 `solve_model_warm`(`simplex.rs:1695`)뿐이고 dual 전용(`solve_warm` `:1519`)이다. 원본 nidda 재풀이는 primal(method 0) |
| 재풀이 LP 6(cutoff) (27–29) | `simplex:leave` | — | Rust LP `Status`(`simplex.rs:23`)에 cutoff 없음. B&B가 Optimal 뒤 bound로 버린다(`bnb.rs:178–179`) |
| 정규화·노드 상태 5, basis 저장 없음 (30–32) | C:944–1035, `600d25` | — | 노드 상태 없음. 버림=삭제 |
| 재풀이 뒤 LP 2이면 basis 저장 | C:1358 `5d4427`(공통 꼬리). gmu 캡처 37–38 | — | 위와 같음 |
| 재풀이 뒤에도 12 | C:2244–2257 → 노드 3 | — | 노드 3 대체 분기 없음([03b-1](mip_unbounded_reclassification.md)) |
| gmu: basis 없음 → `0c3d0d`(Presolve 1, method 1) → `5d4427` → `68341b` Method 0 | C:787–923, gmu 캡처 13–52 | 없음 | 아래 질문 2 참고 |

### 질문 2. 같은 입력 상태 도달 가능성

두 사례 모두 Rust 자연 실행으로는 같은 입력에 도달할 수 없다.

- **nidda**:
  - 캡처의 LP는 253열 × 464행이다. 모델 파일은 253열 × 402행이다(Presolve=0). 남는 62행은 원본이 추가한 행(cut 등으로 추정, 미확인)이다. Rust B&B에는 cut이 없다.
  - 노드 8287은 휴리스틱 계열 `8e4682`가 부모 없이 만든 노드다. Rust에는 이 경로가 없다.
  - 저장 캡처에는 노드 LP의 bound·행·목적·cutoff가 없어서 Rust에서 그 LP를 다시 만들 수 없다.
- **gmu**:
  - 캡처의 LP는 53열 × 110행이고 solve 호출 26476번째의 "루트"다. 모델 파일은 1919열 × 435행이다. 본 MIP 루트가 아니라 sub-MIP 루트로 추정하며, 미확인이다.
  - Rust에는 sub-MIP가 없다.
- **첫 LP 상태 12**: 원본 수치 실패의 결과다. Rust LP가 같은 입력에서 같은 실패를 낼 근거가 없다(질문 1 표).

따라서 비교 경계를 두 층으로 나눈다.

| 층 | 경계 | 입력 | 출력·비교 | 지금 가능한가 |
|---|---|---|---|---|
| L1 재풀이 제어 재생 | 노드 LP 처리 진입 → 노드 처리 반환. LP 호출은 원본 결과를 주입하며 **합성 주입으로 표시**한다 | 노드 basis, `param_3[3]`(warm method), 캐시·루트 여부, LP 결과(첫 12, 재풀이 6) | 첫 LP·재풀이에 넘긴 basis·method, 노드 상태, basis 저장 여부 | 가능. 기준 자료 `bench/oracle/mip_retry_rust/nidda_stress.json` |
| L2 LP 포함 재생 | 위 경계 + 실제 Rust LP | L1 입력 + 노드 LP(행·bound·목적·cutoff) | 재풀이 LP 상태(6)와 그 뒤 basis | 불가. 노드 LP 원본 캡처와 Rust LP 기능(method 지정 warm start, cutoff 상태)이 먼저 필요하다 |

L1은 재풀이 판단, 즉 어떤 basis에서 어떤 method로 다시 풀고 결과를 어떻게 처리하는지만 검증한다. LP 궤적은 검증하지 않는다.

### 질문 3. Rust 계측 사건 형식 (`rsolver.mip_retry.v1`)

Rust가 노드 하나의 LP 처리에서 내보내는 JSON이다. 사건은 순서대로 기록한다. 배열은 Gurobi VBasis(열 n개: 0, −1, −2, −3)와 CBasis(행 m개: 0, −1) 코드이고, 상태는 Gurobi 코드(2·3·5·6·7·12)다.

```json
{"schema": "rsolver.mip_retry.v1", "case": "nidda_stress", "branch": "node_basis_direct",
 "input": {"cols": 253, "rows": 464, "root": false, "cache": 2, "warm_method": 1,
           "node_basis": {"vbasis": [...], "cbasis": [...]}},
 "injected": {"first_lp_status": 12, "retry_lp_status": 6},
 "events": [
  {"ev": "node_lp", "method": 1, "vbasis": [...], "cbasis": [...]},
  {"ev": "node_lp_done", "status": 12},
  {"ev": "retry", "method": 0, "vbasis": [...], "cbasis": [...]},
  {"ev": "retry_done", "status": 6},
  {"ev": "node_done", "node_status": 5, "lp_status": 6, "basis_saved": false}]}
```

- `node_lp`·`retry`: Rust가 LP 함수에 실제로 넘긴 method와 basis다. cold start이면 `null`이다.
- `node_done.basis_saved`: 이 노드 처리 안에서 노드 저장 basis를 새로 썼는지를 나타낸다.
- `input`·`injected`는 Rust 재생 하네스가 기준 자료에서 읽어 넣는다. `events`는 Rust 제어 코드가 직접 만들어야 한다. 기준 자료의 `events`를 복사해 내보내면 비교 의미가 없다.
- 원본 투영(`tools/mip_retry_rust_expect.py`):

| 필드 | 원본 캡처 사건 |
|---|---|
| `input.node_basis` | `x:decode:leave` |
| `input.warm_method` | 첫 `x:cda5b:enter.method`. `param_3[3]`은 캡처하지 않았다. 노드 basis가 있으면 C:361 분기를 타고, `param_7=0`이라 적응 선택이 없으므로 두 값이 같다(C + 캡처) |
| `input.root`·`cache` | `solve:enter.node.root`, `retry.node.cache` |
| `node_lp` | k=1 `x:cda5b:enter`. `vb_rel/cb_rel == I98/Ia0`이고 내용이 노드 basis와 같아야 투영한다 |
| `retry` | `retry=true`인 `x:cda5b:enter`. 조건은 위와 같다 |
| `*_done.status`, `injected` | 해당 `x:cda5b:leave.lp.lp_status` |
| `node_done` | `solve:leave.node.status`, `.lp.status`, 재풀이 뒤 자연 `x:export`·`x:save_lp2:enter` 유무 |

### 질문 4. 필드별 비교 규칙 (03d-2 L1 계약)

| 필드 | 규칙 | 근거 |
|---|---|---|
| `schema`·`case`·`branch`·`input`·`injected` | JSON 정확 일치 | 같은 입력 상태 |
| 사건 순서·개수·필드 집합 | 정확 일치. 추가 사건·필드도 실패 | §7 순서 |
| `node_lp.vbasis/cbasis` | 정수 배열 정확 일치(= 노드 basis). `null`이면 실패 | `5ccea5`, 캡처 사건 4 |
| `retry.vbasis/cbasis` | 정수 배열 정확 일치(= 노드 basis). 실패한 LP의 마지막 basis이면 실패 | C:929, 캡처 사건 18 |
| `node_lp.method`·`retry.method` | 정수 정확 일치. 재풀이 method는 `warm_method==0`이면 1, 아니면 0 | C:361, C:926 |
| `node_done.node_status`·`lp_status`·`basis_saved` | 정확 일치 | 캡처 사건 32, `600d25` |
| 분해 횟수, 반복 수, 최종 vstat, LU·pricing | 비교 대상 아님(미검증) | LP 궤적은 L2와 M1 범위 |

- 수치 필드는 없다. NaN·시간·work·병렬 분기도 해당하지 않는다(Threads=1, TimeLimit 없음).
- L1 통과는 "원본 LP 결과를 주입한 제어 구간 일치"로만 보고한다. 자연 실행 동등성으로 보고하지 않는다.

### 질문 5. Rust 자료 구조와 공백 (03d-2 선행 조건)

| 원본 | Rust 현재 | 03d-2에 필요한 것 |
|---|---|---|
| 노드 저장 basis `I+0x98/0xa0`(압축 `I+0xb0`) | 없음 | `Node`에 `Option<(Vec<i32>, Vec<i32>)>`. 압축 형식은 저장 공간 최적화라 의미 비교 대상이 아니다 |
| 부모 basis 상속(`5ff358`→`69e299`) | 없음 | `branch()`에서 자식에 복사 |
| LP 2 뒤 저장(`5d4427`→`113d92`) | `LpResult::basis`를 손으로 변환한다(`gurobi_basis`, `simplex.rs:1681`, −4→−1) | 원본 `113d92`는 열 플래그·`ws+0x1d8`·`lp+0x64`에 따라 다르다([교훈](../lessons/original.md)). L1에서는 저장 여부만 비교하고, 내용은 L2에서 비교한다 |
| `param_3[3]` warm method | 없음(`Params.method`만 있음, `crates/mip/src/lib.rs:202`) | 노드 warm method 입력. NodeMethod=−1일 때 실제 값이 무엇인지는 미확인이다(nidda는 1) |
| 첫 LP warm start | `solve_model_warm`(dual 전용) | method 1이면 기존 API로 충분하다 |
| 재풀이 primal warm start(method 0) | 없음 | `crates/lp` 변경이 필요하다. **M1 영역**이므로 M1 작업 또는 shared 협의로 분리한다. L1은 LP를 주입하므로 이 기능 없이도 비교할 수 있지만, 실제 경로는 이 기능 없이 완성되지 않는다 |
| LP cutoff 상태 6 | 없음 | L2와 실제 경로에 필요하다(M1). L1은 주입한 6을 처리하는 제어만 검사한다 |
| 재풀이 뒤 12 → basis 해제 → 노드 3 | 없음 | 노드 3 대체 분기(`627f3e`)가 없다. 03d-2에서는 범위 밖으로 두고 기존 Numeric 종료를 유지한다 |

## 4. 실행 결과

- 명령:
  - `python3 tools/mip_retry_rust_expect.py --write`: 기준 자료를 만든다.
  - `python3 tools/mip_retry_rust_expect.py --self-test`: 종료코드 0.
- 결과:
  - M2-03c-2 manifest 해시가 모두 일치했다.
  - nidda natural 3회의 투영이 서로 같고, 저장 기준 자료와도 같다(사건 5개).
  - 변조 18종을 모두 거부했다. 캡처 쪽 8종은 재풀이 method, 별칭, basis 내용, 노드 basis null, 재풀이 제거, 첫 LP 상태, 실행 간 차이, 자연 export다. trace 쪽 10종은 method, 실패 basis 사용, 첫 LP cold, 저장 여부, 노드 상태, 추가 필드, 추가 사건, 빈 사건, 주입값, schema다.
  - 같은 trace는 통과한다.
  - 디스크의 기준 자료에서 재풀이 method 하나를 바꾸면 종료코드 1이고, 첫 차이로 `event 2 (retry) field method`를 출력한다. 되돌리면 0이다.
- `--compare bench/oracle/mip_retry_rust/nidda_stress.json`은 PASS다. 이것은 하네스 확인일 뿐 Rust 결과가 아니다.
- Gurobi는 실행하지 않았다. Rust·Python 회귀는 솔버 코드를 바꾸지 않았으므로 대상이 아니다.

## 5. 판정과 인수인계

- 작업 수행 상태: **조사 완료**. 동등성: **미검증**(Rust 미변경).
- 결론:
  - "Rust 자연 상태 12 재풀이"는 원본 두 사례와 같은 입력으로 만들 수 없다.
  - 03d-2 비교 경계는 L1(원본 LP 결과를 주입한 재풀이 제어 재생, nidda 분기 `node_basis_direct`)으로 한다.
  - 작업 이름의 "자연" 전제가 바뀌므로 사용자 확인이 필요하다.
- 후속 분할 제안:
  - **M2-03d-2**(구현, L1): 노드 basis 저장·상속, 첫 LP warm start, 재풀이 method 규칙, 재풀이 뒤 처리를 구현하고 `tools/mip_retry_rust_expect.py --compare <Rust trace>`로 PASS를 받는다.
    - 실제 경로의 primal warm start와 cutoff는 M1 선행이다. 그 전까지는 L1 통과로만 보고한다.
  - **M2-03d-3**(오라클, L2): 재풀이 시점 노드 LP(행·bound·목적·cutoff)의 원본 캡처, Rust LP 포함 재생.
  - gmu 분기(`0c3d0d`, sub-MIP 루트 추정, `68341b`)는 후속 번호를 아직 배정하지 않았다. sub-MIP 여부부터 확인해야 한다.
- 남은 한계:
  - `param_3[3]`의 일반 값(NodeMethod=−1일 때).
  - 62행의 정체.
  - gmu sub-MIP 여부.
  - 재풀이 LP 2인 `node_basis_direct` 분기의 캡처. 저장 근거는 gmu의 공통 꼬리와 C뿐이다.
- 교훈: [원본 사실](../lessons/original.md), [절차](../lessons/process.md)에 추가했다.
