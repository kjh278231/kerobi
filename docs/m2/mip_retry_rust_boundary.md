# M2-03d-1 검증 계약과 결과: Rust 재풀이 비교 경계

작성 기준은 [스프린트 검증 기준](../15_sprint_verification.md)이다. 원본 근거는 [M2-03c-2](mip_retry_basis.md)와 [M2-03c-1](mip_linear_child_trace.md) §10–13이다.

## 1. 범위와 주장

- 작업 종류: 조사·오라클 설계. Rust 솔버 코드는 바꾸지 않고 Gurobi도 실행하지 않는다. 저장된 원본 캡처만 다시 읽는다.
- 질문:
  1. Rust에서 자식 LP의 Numeric은 어디서 처리되며, 원본 재풀이 사건(nidda·gmu)과 어떻게 대응하는가?
  2. Rust가 원본 두 사례와 같은 입력 상태에 자연 실행으로 도달할 수 있는가? 없다면 비교 경계를 어디에 두는가?
  3. Rust 계측 사건의 형식과 원본 캡처의 투영은 무엇인가?
  4. 필드별 비교 규칙은 무엇인가?
  5. 노드 저장 basis, method `param_3[3]` 규칙, 상태 2 basis 저장에 대응하는 Rust 자료 구조가 있는가? 없으면 무엇이 빠졌는가?
- 판정 규칙:
  - 각 질문에 원본 C 줄, 캡처 사건, Rust `파일:줄` 중 하나 이상의 근거로 답하면 조사 완료다.
  - 투영 도구는 저장 캡처 3회에서 같은 결과를 내야 하고 변조를 모두 거부해야 한다.
  - 조사만으로 Rust 동등성을 주장하지 않는다.
  - 이 규칙은 결과와 같은 커밋(`3408c2f`)에 들어갔다. 그래서 "결과 확인 전 고정"을 git으로 보일 수는 없다.
- 제외: Rust 구현(03d-2), 노드 LP 원본 캡처(03d-3), gmu 분기 구현.
- 선행: main `de9863a`(M2-03c-2 병합 뒤). 미커밋 선행 변경은 없다.

## 2. 원본과 상태 경계

- Gurobi 13.0.3, SHA-256 `8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c`. 주소는 file offset이다.
- 이번에 새로 읽은 C(`../re/decomp/c/`):

| 함수 | 줄 | 내용 |
|---|---|---|
| `5ce67b` | 351–361 | cold 조건: (`I+8`이 null이거나 `I+8`의 `+0x38 ≠ 0xb`) 그리고 `I+0x98 == 0` 그리고 `115e51(lp) == 0`. 성립하면 첫 LP method는 `param_3+0x10`이고, 아니면 `param_3+0x18`(=`param_3[3]`)이다 |
| `5ce67b` | 362–455 | `param_7≠0`이고 SimplexPricing(env+0x2440)=−1이면 적응 선택으로 첫 LP method가 바뀔 수 있다. 재풀이 method는 이 결과가 아니라 `param_3[3]`을 다시 읽는다(C:926) |
| `5ce67b` | 769–786 | 재풀이 진입 조건: LP 12이고 (캐시 `M+0xa8`=2 또는 현재 노드가 루트). FEASHELPER==−2이거나 `local_58+0x1a0≠0`이면 `5ff754(node,3)`로 끝난다 |
| `5ce67b` | 787–923 | `I+0x98==0`, `I+0xa8≠0`, `a7f923≠0` 중 하나면 이 블록에 들어온다. 그 안에서 `a7f923==0`이고 (`local_110≠2` 또는 `2baa47≠0`)일 때만 `0c3d0d`를 부른다. 이때 Presolve=1(env+0x2578≥0이면 그 값), Threads=1, Method=`local_110`(0·1·2 밖이면 1)로 설정하고 반환 뒤 복구한다(gmu). 조건이 맞지 않으면 재풀이 LP가 없다 |
| `5ce67b` | 924–933 | 그 밖에는 `method = (param_3[3]==0)`으로 `cda5b(lp, method, I+0x98, I+0xa0, 0,0,0,0)`을 직접 부른다(`a7f0ad≠0`이면 `14abfb`). nidda가 이 경로다 |
| `5ce67b` | 944–1035 | 재풀이 뒤 정규화: LP 2이고 목적값≥cutoff이면 6. LP 4는 캐시에 따라 3·5·12가 된다. 캐시 2에서는 5→12, 4→3이다. 6은 경우에 따라 0xb가 된다 |
| `5ce67b` | 2244–2257 | 정규화 뒤에도 12이면 `I+0xb0`·`I+0x98`를 해제하고 `I+0x94=0`으로 둔 뒤 `600d25`로 간다. 결과는 노드 3이다([03b-1](mip_unbounded_reclassification.md) 표) |
| `5ff358` | 77–84 | 새 노드에 부모가 있으면 `69e299(child, 0, parent)`로 부모 basis를 복사한다. 분기 함수 `627f3e`가 이 함수를 부른다 |
| `8e4682` | 355 | `5ce67b(&local_78, local_58, local_90, 0, -1, 0, 0, …)` 호출이므로 `param_7=0`이다. nidda 노드 8287의 첫 LP에는 적응 선택이 없다 |
| `600d25` | — | LP 상태를 노드 상태로 바꾼다. CUTOFF(6)→5 ([B&B 노트](../../../re/notes/15_mip_branch_and_bound.md) `600d25` 행) |

- `param_3`은 호출자가 넘기는 구조체이고, nidda에서는 `8e4682`의 `local_90`이다. `+0x18` 값 자체는 캡처하지 않았다.
- 시작 경계: 노드 LP 처리(`5ce67b`) 진입. 끝 경계: 그 반환. 모델·설정·RNG는 M2-03c-2와 같다(Threads=1, Seed=0, TimeLimit 없음).

## 3. 비교 계약 (03d-2용 L1)

경계를 정한 근거는 §6 질문 2다.

**사건 형식 `rsolver.mip_retry.v1`**

Rust가 노드 하나의 LP 처리에서 내보내는 JSON이다.
- 사건은 순서대로 기록한다.
- 배열은 Gurobi 코드다. VBasis는 열 n개(0, −1, −2, −3), CBasis는 행 m개(0, −1)다.
- 상태도 Gurobi 코드(2·3·5·6·7·12)를 쓴다.

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
- `input`·`injected`: Rust 재생 하네스가 기준 자료에서 읽어 넣는다. LP 호출 결과는 원본 값을 주입하므로 **합성 주입**으로 표시한다.
- `events`: Rust 제어 코드가 직접 만들어야 한다. 기준 자료의 사건을 복사해 내보내면 비교 의미가 없다. 검사기는 이를 막지 못하므로 코드 검토로 확인한다.

**원본 투영** (`tools/mip_retry_rust_expect.py`)

| 필드 | 원본 캡처 사건 |
|---|---|
| `input.node_basis` | `x:decode:leave` |
| `input.warm_method` | 첫 `x:cda5b:enter.method`. `param_3[3]`은 캡처하지 않았다. 노드 basis가 있으면 C:361 분기를 타고, `param_7=0`이라 적응 선택이 없으므로 두 값이 같다(C 추론) |
| `input.root`·`cache` | `solve:enter.node.root`, `retry.node.cache` |
| `node_lp` | k=1 `x:cda5b:enter`. `vb_rel/cb_rel`이 `I98/Ia0`이고 내용이 노드 basis와 같을 때만 투영한다 |
| `retry` | `retry=true`인 `x:cda5b:enter`. 조건은 위와 같다 |
| `*_done.status`, `injected` | 해당 `x:cda5b:leave.lp.lp_status` |
| `node_done` | `solve:leave.node.status`와 `.lp.status`, 재풀이 뒤 자연 `x:export`·`x:save_lp2:enter` 유무 |

**필드별 규칙**

| 필드 | 규칙 | 근거 |
|---|---|---|
| `schema`·`case`·`branch`·`input`·`injected` | JSON 정확 일치 | 같은 입력 상태 |
| 사건 순서·개수·필드 집합 | 정확 일치. 추가 사건·필드도 실패 | 기준 §7 |
| `node_lp.vbasis/cbasis` | 정수 배열 정확 일치(= 노드 basis). `null`이면 실패 | `5ccea5`, 캡처 사건 4 |
| `retry.vbasis/cbasis` | 정수 배열 정확 일치(= 노드 basis). 실패한 LP의 마지막 basis를 넘기면 실패 | C:929, 캡처 사건 18 |
| `node_lp.method`·`retry.method` | 정수 정확 일치. 재풀이 method는 `warm_method≠0`이면 0(캡처+C)이고, `warm_method==0`이면 1(C만) | C:361, C:926 |
| `node_done.node_status`·`lp_status`·`basis_saved` | 정확 일치 | 캡처 사건 32, `600d25` |
| 분해 횟수, 반복 수, 최종 vstat, LU·pricing | 비교 대상 아님(미검증) | LP 궤적은 L2와 M1 범위 |

- 수치 필드는 없다. NaN·시간·work·병렬 분기도 해당하지 않는다(Threads=1, TimeLimit 없음).
- `node_lp_done.status`, `retry_done.status`, `node_done.lp_status`는 주입값이 그대로 나오는 순서 표지다.
- Rust 논리를 판별하는 필드는 basis·method·`node_status`·`basis_saved`다. L1 통과는 "원본 LP 결과를 주입한 제어 구간 일치"로만 보고한다.

## 4. 사례와 원본 자료

| 사례 | 실제 / 합성 | 분기 | 입력·해시 | 원본 증거 |
|---|---|---|---|---|
| nidda stress, 노드 8287 | 원본 실제 실행. Rust 쪽 LP 결과는 합성 주입 | `node_basis_direct`(C:924–933), 재풀이 LP 6 | `neos-3754480-nidda.mps.gz` `63f322dd…a8bf7f`, Presolve 0·ScaleFlag 0·Quad 0·NumericFocus 0·MarkowitzTol 1e-4·NodeLimit 6000·Threads 1·Seed 0 | `bench/oracle/mip_retry_basis/nidda_stress_natural_{1,2,3}.json`(M2-03c-2 manifest) |
| gmu default, solve 26476 | 원본 실제 실행 | `0c3d0d` 경로. **03d-2 제외** | `gmu-35-50.mps.gz` `5652e6ad…adb8d1` | 같은 manifest |
| 재풀이 LP 2, `node_basis_direct` | — | 저장 경로 | 없음 | C:1358뿐. 캡처 없음 |
| 재풀이 뒤 12 | — | 노드 3 | 없음 | C:2244–2257뿐 |

- 기준 자료: `bench/oracle/mip_retry_rust/nidda_stress.json`(SHA-256 `91b8eda5…1fda69`).
  - 원본 캡처를 그대로 복사한 것이 아니다. 투영기(SHA-256 `dbefeac7…865deb`)가 M2-03c-2 manifest 해시를 확인한 캡처에서 매번 다시 만들어 비교한다.
  - 캡처 도구는 `tools/mip_retry_basis.js`(`a054ad44…a380c2`)다.
- 적용하지 않은 범주: 동률·허용오차 경계·수치 필드는 없다. 오류·중단 분기(재풀이 rc≠0, 12 잔존)는 원본 캡처가 없어 L1 사례로 넣지 않았다.

## 5. 재실행 명령

- 기준 자료 재생성: `python3 tools/mip_retry_rust_expect.py --write`.
- 저장 기준 확인: `python3 tools/mip_retry_rust_expect.py --self-test`. 기대 종료코드 0. 확인 내용은 사건 5개, 캡처 쪽 변조 8종 거부, trace 쪽 변조 10종 거부다.
- Rust trace 비교(03d-2): `python3 tools/mip_retry_rust_expect.py --compare <trace.json>`. 첫 차이의 사건·필드·기준값·실제값을 출력하고 종료코드 1이다.
- 원본 재캡처: M2-03c-2의 `tools/mip_retry_basis_runs.sh <출력>`(Gurobi 12회, 약 25분). 이번에는 실행하지 않았다.
- 회귀: 솔버 코드를 바꾸지 않았으므로 Rust·Python 검사는 대상이 아니다. `python3 tools/mip_retry_basis_check.py --self-test`는 계속 0이어야 한다.

## 6. 실행 결과

### 질문 1. Rust 처리 위치와 원본 대응표

현재 Rust 경로:
- `crates/mip/src/bnb.rs:16` `Node { model, bound }`: basis가 없다.
- `bnb.rs:170` 자식 LP는 매번 `solve_model_deadline(&node.model, 1, …)`로 cold dual이다.
- `bnb.rs:176` `Numeric | Unbounded`이면 노드를 되돌려 넣고 MIP 전체를 `Status::Numeric`으로 끝낸다. 재풀이는 없다.
- 루트는 `crates/mip/src/root.rs:389` `Status::Numeric => MipStatus::Numeric`이다.
- Rust LP의 `Numeric`은 `crates/lp/src` 전체에서 `simplex.rs:1415` 한 곳만 만든다. primal ratio test에서 나가는 행이 없고, 재분해 뒤에도 그대로이며, phase 1일 때다.

| 원본 사건 (nidda 사건 번호) | 원본 근거 | Rust 현재 | Rust 대응 자료 |
|---|---|---|---|
| 노드 basis 수명: export→pack→copy→decode (1–3) | `5d4427`/`113d92`, `69d237`, `69e299`, `69d7ea` | 없음 | `Node`에 basis 없음. `LpResult::basis`(`crates/lp/src/simplex.rs:1677`)는 계산하지만 버린다 |
| 일반 자식 basis 상속 | `5ff358` C:77–84 | 없음 | `branch()`(`bnb.rs:198–210`)는 모델만 복사한다 |
| 첫 LP: 노드 basis와 method `param_3[3]` (4–12) | C:351–361, `5ccea5` | `bnb.rs:170` cold, method 1 고정 | 노드 method 파라미터 없음(NodeMethod는 `crates/model/src/param_table.rs:58`에만 있다) |
| 첫 LP 상태 12 (12–15) | 원본 simplex 수치 실패 | Rust `Numeric`은 위 1415 조건에서만 생겨 원인이 다르다 | — |
| 재풀이 진입 판정 | C:769–786 | 없음(`bnb.rs:176`에서 종료) | 노드 처리에서 캐시(루트 LP 상태)와 루트 여부를 읽지 않는다 |
| 재풀이: 노드 basis, method `param_3[3]==0` (16–28) | C:924–933 | 없음 | warm start는 `solve_model_warm`(`simplex.rs:1695`) 하나이고 dual 전용이다(`solve_warm` `:1519`). 원본 nidda 재풀이는 primal(method 0)이다 |
| 재풀이 LP 6(cutoff) (27–29) | `simplex:leave` | — | Rust LP `Status`(`simplex.rs:23`)에 cutoff가 없다. B&B가 Optimal 뒤 bound로 버린다(`bnb.rs:178–179`) |
| 정규화, 노드 상태 5, basis 저장 없음 (30–32) | C:944–1035, `600d25` | — | 노드 상태가 없다. 버리면 삭제된다 |
| 재풀이 뒤 LP 2이면 basis 저장 | C:1358 `5d4427`(공통 꼬리). gmu 캡처 37–38 | — | 위와 같다 |
| 재풀이 뒤에도 12 | C:2244–2257 → 노드 3 | — | 노드 3 대체 분기 없음([03b-1](mip_unbounded_reclassification.md)) |
| gmu: basis 없음 → `0c3d0d`(Presolve 1, method 1) → `5d4427` → `68341b` Method 0 | C:787–923, gmu 캡처 13–52 | 없음 | 아래 질문 2 참고 |

### 질문 2. 같은 입력 상태 도달 가능성

두 사례 모두 Rust 자연 실행으로는 같은 입력에 도달할 수 없다.

- **nidda**:
  - 캡처의 LP는 253열 × 464행이다. 모델 파일은 253열 × 402행이다(Presolve=0). 남는 62행은 원본이 추가한 행이다(cut 등으로 추정, 미확인). Rust B&B에는 cut이 없다.
  - 노드 8287은 휴리스틱 계열 `8e4682`가 부모 없이 만든 노드다. Rust에는 이 경로가 없다.
  - 저장 캡처에는 노드 LP의 bound·행·목적·cutoff가 없어서 Rust가 그 LP를 다시 만들 수 없다.
- **gmu**:
  - 캡처의 LP는 53열 × 110행이고 solve 호출 26476번째의 "루트"다. 모델 파일은 1919열 × 435행이다.
  - 본 MIP 루트가 아니라 sub-MIP 루트로 추정한다(미확인). Rust에는 sub-MIP가 없다.
- **첫 LP 상태 12**: 원본의 수치 실패다. Rust LP가 같은 입력에서 같은 실패를 낸다는 근거가 없다.

따라서 비교 경계를 두 층으로 나눈다.

| 층 | 경계 | 입력 | 출력·비교 | 지금 가능한가 |
|---|---|---|---|---|
| L1 재풀이 제어 재생 | `5ce67b` 진입 → 반환. LP 호출은 원본 결과를 주입한다(합성) | 노드 basis, `param_3[3]`, 캐시·루트 여부, LP 결과(첫 12, 재풀이 6) | 첫 LP·재풀이에 넘긴 basis·method, 노드 상태, basis 저장 여부(§3) | 가능 |
| L2 LP 포함 재생 | 위 경계 + 실제 Rust LP | L1 입력 + 노드 LP(행·bound·목적·cutoff) | 재풀이 LP 상태(6)와 그 뒤 basis | 불가. 노드 LP 원본 캡처와 Rust LP 기능(method 지정 warm start, cutoff 상태)이 먼저 필요하다 |

### 질문 3·4

§3의 사건 형식·원본 투영·필드 규칙으로 답했다. 투영 실행 결과는 아래 "투영기 실행"에 있다.

### 질문 5. Rust 자료 구조와 공백

| 원본 | Rust 현재 | 필요한 것과 담당 |
|---|---|---|
| 노드 저장 basis `I+0x98/0xa0`(압축 `I+0xb0`) | 없음 | `Node`의 선택적 basis 필드. 압축 형식은 저장 공간 최적화라 의미 비교 대상이 아니다. 저장·복구 설계는 **M2-10b**(노드 LP basis 재사용 설계, `14_sprint_plan.md`) 범위다 |
| 부모 basis 상속(`5ff358`→`69e299`) | 없음 | `branch()`에서 복사한다. M2-10b 범위다. L1 기준 자료로는 검증하지 못한다(nidda는 `8e4682` 복사) |
| LP 2 뒤 저장(`5d4427`→`113d92`) | `LpResult::basis`를 손으로 변환한다(`gurobi_basis`, `simplex.rs:1681`, −4→−1) | 원본 `113d92`는 열 플래그·`ws+0x1d8`·`lp+0x64`에 따라 결과가 다르다([교훈](../lessons/original.md)). M2-10b 범위다. 이 분기는 캡처가 없다 |
| `param_3[3]` warm method | 없음(`Params.method`만 있음, `crates/mip/src/lib.rs:202`) | 노드 warm method 입력. NodeMethod=−1일 때 실제 값은 미확인이다(nidda는 1) |
| 첫 LP warm start | `solve_model_warm`(dual 전용) | method 1이면 기존 API로 충분하다. 다만 모든 자식에 켜면 전체 B&B 궤적과 기존 회귀 기준(38개 측정, pytest)이 바뀐다. M2-10b와 함께 결정한다 |
| 재풀이 primal warm start(method 0) | 없음 | `crates/lp` 변경이 필요하다. **M1 영역**이고 담당 번호는 아직 없다 |
| LP cutoff 상태 6 | 없음 | L2와 실제 경로에 필요하다(M1, 번호 없음) |
| 재풀이 뒤 12 → basis 해제 → 노드 3 | 없음 | 노드 3 대체 분기(`627f3e`)가 없으므로 기존 Numeric 종료를 유지한다 |

**03d-2 범위 (제안)**
- 포함: basis가 있는 노드의 LP 처리 제어. 첫 LP와 재풀이에 노드 basis를 넘기고, 재풀이 method를 `param_3[3]` 규칙으로 정하고, 주입된 LP 6을 노드 버림·저장 없음으로 처리한다. L1 비교로 판별하는 범위는 이것뿐이다.
- 제외: basis 상속, LP 2 뒤 저장, 모든 자식의 warm start(M2-10b), primal warm·cutoff(M1), gmu 분기.
- 실제 경로: M2-10b와 M1 기능이 생기기 전에는 노드에 basis가 없다. 그동안 `bnb.rs:176`의 Numeric 종료를 그대로 둔다. dual warm start로 method 0을 대신하거나 cold 재풀이로 대신하는 것을 금지한다. 원본과 다른 동작을 일치한다고 보고하게 되기 때문이다.
- 결과적으로 03d-2는 하네스 안에서만 실행되는 제어 코드가 된다. 선행 작업을 먼저 할지는 사용자 결정이 필요하다(§7).

### 투영기 실행

- 실행 위치는 worktree `../rsolver-m2-03d-1`, commit `3408c2f`다.
- `python3 tools/mip_retry_rust_expect.py --write` 뒤 `--self-test` 종료코드는 0이다.
  - M2-03c-2 manifest 해시가 모두 일치했다.
  - nidda natural 3회의 투영이 서로 같고, 저장 기준 자료와도 같다(사건 5개).
  - 변조 18종을 모두 거부했다. 캡처 8종은 재풀이 method, 별칭, basis 내용, 노드 basis null, 재풀이 제거, 첫 LP 상태, 실행 간 차이, 자연 export다. trace 10종은 method, 실패 basis 사용, 첫 LP cold, 저장 여부, 노드 상태, 추가 필드, 추가 사건, 빈 사건, 주입값, schema다.
  - 같은 trace는 통과한다.
- 디스크의 기준 자료에서 재풀이 method 하나를 바꾸면 종료코드 1이고, 첫 차이로 `event 2 (retry) field method`를 출력한다. 되돌리면 0이다.
- `--compare bench/oracle/mip_retry_rust/nidda_stress.json`은 PASS다. 이것은 하네스 확인일 뿐 Rust 결과가 아니다.
- `python3 tools/mip_retry_basis_check.py --self-test`는 0이다.
- 실행하지 않은 것: Gurobi(새 캡처 불필요), Rust·Python 회귀(솔버 미변경).

## 7. 판정과 인수인계

- 작업 수행 상태: **조사 완료**. 동등성: **미검증**(Rust 미변경).
- 결론:
  - "Rust 자연 상태 12 재풀이"는 원본 두 사례와 같은 입력으로 만들 수 없다.
  - 03d-2 비교 경계는 L1(원본 LP 결과를 주입한 재풀이 제어 재생, nidda `node_basis_direct`)으로 한다.
- 2026-11-10 사용자가 아래 두 결정을 승인했다. L1 합성 재생, 03d-2 우선 진행. 결과는 [M2-03d-2](mip_retry_control.md)에 있다. LP 포함 재생과 실제 경로 연결은 M2-03d-4로 옮겼고, M2-03d-3은 노드 LP 캡처와 62행·sub-MIP 조사다.
- 당시 사용자 결정 필요:
  1. 작업 이름의 "자연" 전제를 L1(합성 주입) 경계로 바꾸는 것을 승인하는가?
  2. 03d-2를 지금 할 것인가, 아니면 선행 작업을 먼저 할 것인가? 지금 하면 하네스 안에서만 실행되는 코드가 된다. 선행 작업은 M2-10b 노드 basis 재사용 설계와 M1 primal warm·cutoff이며, M1 쪽은 담당 번호가 없다.
- 후속 번호:
  - **M2-03d-2**(구현, L1): 위 사용자 확인 후 진행한다. 범위는 §6 "03d-2 범위". 완료 기준은 Rust trace의 `python3 tools/mip_retry_rust_expect.py --compare <trace>` PASS와 회귀 통과다.
  - **M2-03d-3**(오라클, L2): 재풀이 시점 노드 LP(행·bound·목적·cutoff)의 원본 캡처와 Rust LP 포함 재생.
  - gmu 분기는 번호를 배정하지 않았다. sub-MIP 여부부터 확인한다.
- 남은 한계:
  - `param_3[3]`의 일반 값.
  - 62행의 정체.
  - gmu sub-MIP 여부.
  - 재풀이 LP 2인 `node_basis_direct` 분기와 재풀이 뒤 12의 캡처.
- 작업 위치: branch `work/m2-03d-1`(commit `3408c2f`와 문서 갱신 commit). 검사 통과 뒤 main에 `--no-ff`로 병합한다. 미커밋 변경은 없다.
- 갱신한 문서: `docs/11_progress.md`·`docs/12_roadmap_diagram.md` 맨 위, `docs/13_work_units.md`, `docs/14_sprint_plan.md` M2-03d 행과 §5, `docs/m2/sprint_design.md` §2, `docs/handoff/M2.md`.
- 교훈: [원본 사실](../lessons/original.md)(상속, 재풀이 method, 12 잔존, 사례 문맥), [절차](../lessons/process.md)(Rust 입력 재구성 가능성 확인), [도구](../lessons/tooling.md)(투영기 재사용)에 추가했다.
