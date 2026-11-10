# M2-03d-5 검증 계약과 결과: 일반 B&B 노드의 자연 재풀이 스캔

작성 기준은 [스프린트 검증 기준](../15_sprint_verification.md)이다. 배경은 [M2-03d-3](mip_retry_node_lp.md) §7이다. 03c 재풀이 오라클의 두 사례(nidda RINS 고정 LP, gmu sub-MIP 루트)는 일반 B&B 노드가 아니므로 `bnb.rs` 실제 경로 검증에 쓸 수 없다. 사용자는 M2-03d-4 전에 일반 B&B 노드의 자연 재풀이 사례를 찾는 스캔을 먼저 하기로 정했다(2026-11-10).

## 1. 범위와 주장

- 작업 종류: 조사(원본 읽기 전용 스캔). Rust 솔버 코드는 바꾸지 않는다.
- 조사 질문:
  - Q1. 공개 파라미터만 쓴 자연 실행에서, 본 탐색 B&B 노드(아래 tier A)의 LP가 상태 12로 끝나 `5ce67b` 재풀이 블록(`0x4f53a8`)에 들어가는 사례가 있는가?
  - Q2. 있으면 독립 3회 재현되는가? 없으면 고정한 매트릭스에서 tier A 노드 LP를 몇 건 관측했는가?
- 사례 분류(재풀이 진입 시점의 값, 걸러내지 않고 모두 기록):

| tier | 조건 | 의미 |
|---|---|---|
| A | 활성 `5ce67b`의 직접 호출자 `5fb8ec`(노드 처리), `5bbf7f` 중첩 깊이 1, 노드 부모 있음, 재시작 0(`M+0x620==M`) | 최상위 MIP 본 탐색의 일반 B&B 노드 |
| B | A와 같은 호출자·깊이·부모이나 재시작 뒤(`M+0x620≠M` 또는 restarts>0) | 재시작 뒤 본 탐색 노드. Rust에는 재시작이 없다 |
| C | 그 밖(휴리스틱 호출자 `8e4682`·`8f39d7`·`8faaa4` 등, 깊이 ≥2 sub-MIP, 루트 `654502`·`64da9f`·`7559f7`·`6dec7d`) | 일반 B&B 노드가 아님 |

- tier A 사례가 M2-03d-4에 주는 것: 일반 B&B 노드에서 상속 basis로 재풀이하는 LP의 존재와 위치. Rust가 그 노드에 같은 LP로 도달한다는 뜻은 아니다(cut·휴리스틱·presolve 차이). 노드 LP·basis 상세 캡처는 후속 번호다.
- 제외: 노드 LP·basis·cutoff 상세 캡처(후속 번호), 재풀이 결과 처리 검증, Rust 동등성, 5→12 재분류(정상 산술에서 불가능, [M2-03c-1 §10](mip_linear_child_trace.md)).
- 선행: main `e476024`. 미커밋 선행 변경 없음.

## 2. 원본과 상태 경계

- Gurobi 13.0.3, SHA-256 `8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c`. 주소는 file offset이다. 도구가 시작 시 바이너리 해시를 확인한다.
- 훅(모두 읽기 전용, `tools/mip_retry_bnb_scan.js`):

| 지점 | 근거 | 읽는 값 |
|---|---|---|
| `5bbf7f`(`0x4e36e0`) 진입·반환 | [M2-03d-3](mip_retry_node_lp.md) §6의 MIP 중첩 깊이 | 깊이, 모델·presolve 뒤 크기, 호출자 |
| `5ce67b`(`0x4f0e10`) 진입·반환 | 호출자 14개(`tools/dc.py 5ce67b`). `5fb8ec`는 B&B 워커 `5c239f`·`5c3438`·`5c37fd`가 부르는 노드 처리(`../re/notes/16_cuts_and_heuristics.md` §3.4) | 반환 주소를 `../re/decomp/index.tsv` 함수 범위로 이름화. 중첩 스택 |
| 정규화 전후 `0x4f1a98`·`0x4f1aca` | `tools/mip_linear_child_scan.js`와 같은 오프셋(노드 `rsp+0x3b0`, LP 상태 `r13+0x38`) | (호출자, 깊이, 루트, 부모 유무, 캐시, LP 상태) 횟수 |
| 재풀이 진입 `0x4f53a8` | 같은 문서 §10 | 위 값 + 노드 id·부모·깊이, `mipId`(restarts, `M+0x620==M`), 노드 basis 포인터(`I+0x98/0xa0/0xa8/0xb0` null 여부), MIP LP 크기 |
| 노드 상태 setter `0x5180f0` | 같은 문서 §10 | 값 횟수 |

- 설정: Threads 1, LogToConsole 0. 스캔 실행은 TimeLimit을 탐색 예산으로만 쓴다. 적중 재현은 TimeLimit 없이 한다(§8, snowy 교훈).

## 3. 비교 계약

| 필드·사건 | 규칙 |
|---|---|
| 대조군 분류 | nidda stress → 재풀이 1건, 호출자 `8e4682`, 깊이 1, 부모 null, 노드 8287, restarts 1(tier C). gmu default → 재풀이 1건, 호출자 `654502`, 깊이 2, 루트(tier C). 정확 일치 |
| 양성 대조 | 같은 대조군에서 `5fb8ec`·깊이 1·비루트·부모 있음 정규화 사건이 1건 이상(tier A 모집단이 계측됨) |
| 적중 재현 | 적중 사례마다 TimeLimit 없이 NodeLimit=스캔의 최종 NodeCount로 독립 3회. 재풀이 사건 전체(호출자·깊이·노드 id·부모·노드 깊이·solve 순번·LP 상태·노드 basis null 여부·restarts)가 3회 정확 일치 |
| 매트릭스 집계 | 실행별 ok, tier별 재풀이 수, tier A 정규화 사건 수(분모). 실패 실행은 별도 집계하고 통과 수에 넣지 않는다 |

- 수치 비교 없음. 시간은 탐색 예산이며 판정 근거가 아니다.

## 4. 사례와 매트릭스(착수 전 고정)

- 모델: `../re/bench/data/miplib/*.mps.gz` 38개(03c-1 스캔과 같음). `jsplib`은 MPS가 아니라 제외.
- 설정(모두 `Threads=1`, `TimeLimit=20`, `NodeLimit=50000`, 실행당 timeout 120 s), seed 0·1·2:
  - `H`: 03c-1 stress(`Presolve=0 ScaleFlag=0 Quad=0 NumericFocus=0 MarkowitzTol=1e-4`) + `Heuristics=0 RINS=0`. 휴리스틱 호출자를 줄인다.
  - `P`: `H` + `Cuts=0`. 적중 노드 LP가 모델 행 + 분기 bound가 되도록 한다.
- 총 38×2×3 = 228회. 기존 03c-1 스캔(default·stress, 76회)은 tier A 재풀이 0건이 확정이라(저장본 합계: 재풀이 2건 모두 C) 대조로만 쓴다.
- 중단 규칙: 매트릭스를 한 번 끝까지 돈다. 적중이 있으면 서로 다른 (모델, 설정, seed) 최대 3건을 재현한다. 매트릭스 확장(seed·NodeLimit 추가)은 결과를 본 뒤 하지 않고 후속 번호로 제안한다.
- 0건이면 그 자체가 결과다(Q2의 분모와 함께 기록).

## 5. 재실행 명령

- 대조군: `tools/mip_retry_bnb_scan.sh controls <출력>`.
- 매트릭스: `tools/mip_retry_bnb_scan.sh matrix <출력>`(Gurobi 한 프로세스씩, `tools/gurun.sh`).
- 집계·검사: `python3 tools/mip_retry_bnb_scan_check.py <출력>`; `--self-test`는 저장 대조군 분류와 변조 거부를 확인한다.

## 6. 실행 결과

- 실행: 기준 main `e476024`, branch `work/m2-03d-5`. 도구 SHA-256 `mip_retry_bnb_scan.js` `039177d0…d32517`(모든 결과 JSON의 `tool_sha256`이 같다), 바이너리 해시 일치. Gurobi는 `tools/gurun.sh`로 한 프로세스씩 실행했다(다른 세션의 M3-04b-3과 잠금 공유).
- 대조군(`controls/`): nidda stress → 재풀이 1건 `8e4682`·깊이 1·부모 null·노드 8287·restarts 1. gmu default → 재풀이 1건 `654502`·깊이 2·루트. 03d-3 기록과 같다. 양성 대조: nidda 35,854건, gmu 24,446건의 `5fb8ec`·깊이 1·비루트·부모 있음 정규화 사건을 관측했다.
- 매트릭스(`matrix/{H,P}/s{0,1,2}/`): 228회 모두 ok(오류·비정상 종료 0). tier A/B 모집단 정규화 사건은 H 3,818,540건, P 6,018,841건이다.

| 설정 | 재풀이 | tier A | tier B | tier C |
|---|---|---|---|---|
| H(seed 0·1·2) | 1 | 0 | 1 | 0 |
| P(seed 0·1·2) | 0 | 0 | 0 | 0 |

- 정규화 직전 LP 상태 분포(전체, 깊이 1 `5fb8ec`): 2 외에 6(cutoff) 605,090, 3 170,617, 9 46, **12 1**. 상태 12는 아래 적중 1건뿐이다.
- 적중(`H/s1/neos-3754480-nidda`, 스캔은 TimeLimit 종료 NodeCount 32,898):

| 필드 | 값 |
|---|---|
| 호출자·깊이 | `5fb8ec`, `5bbf7f` 깊이 1(mipmain 0, 최상위) |
| 노드 | id 16948, 부모 16946, 노드 깊이 30, 비루트, 캐시 2 |
| MIP | restarts 1, `M+0x620≠M`(재시작 뒤 본 탐색), `M+0x628==M`. MIP LP(`M+8`) 402×253 |
| LP | 정규화 전 상태 12, solve 순번 19900 |
| 노드 basis | `I+0x98`·`I+0xa0` 있음, `I+0xa8`·`I+0xb0` null, `I+0x94`=488(모델 402행보다 86행 많음, cut 추정) |

- 재현(`repro/`): TimeLimit 없이 NodeLimit 32898, Seed 1, H 설정으로 독립 3회. 3회 모두 NodeLimit 종료(상태 8, NodeCount 32898)이고 재풀이 사건 전체가 서로, 그리고 TimeLimit 스캔과 같다. `python3 tools/mip_retry_bnb_scan_check.py` 종료코드 0.
- 검사기 판별력: `--self-test` 종료코드 0. 대조군 분류 변조 6종(호출자·깊이·부모·노드 id·restarts·재풀이 추가), 양성 모집단 삭제, 실행 실패 표시, 재현 변조 4종(solve 순번·basis null·2회·재풀이 없음)을 모두 거부했고 tier 함수 5사례가 맞았다.
- 실행하지 않은 것: 적중 노드의 LP·basis·cutoff 내용, 재풀이 뒤 LP 상태와 노드 처리 결과, 86행의 정체. Rust·Python 검사는 코드 변경이 없어 실행하지 않았다.
- 계약 변경: 없음. 대조군은 도구 작성 직후 같은 도구 해시로 실행한 결과를 저장했다.

## 7. 판정과 인수인계

- 작업 수행 상태: 완료(조사).
- Q1: 고정 매트릭스 228회에서 tier A(재시작 없는 본 탐색 일반 노드) 재풀이는 0건이다. tier B 1건(nidda H seed 1, 재시작 1회 뒤 본 탐색의 일반 B&B 노드 16948)이 있다. `5fb8ec` 노드 처리 경로에서 노드 basis를 가진 채 재풀이 블록에 들어간다.
- Q2: tier B 적중은 독립 3회 재현됐다. 분모는 tier A/B 정규화 사건 약 984만 건이다. Cuts=0(P)에서는 0건이라, 적중 노드 LP에는 cut 행이 있다(`I+0x94`=488).
- M2-03d-4에 주는 것: 일반 B&B 노드(`5fb8ec`)의 자연 재풀이 원본 사례가 처음 생겼다. 다만 재시작 뒤 노드이고 LP에 86행이 더 있어서 Rust 자연 실행으로 같은 노드에 도달할 수는 없다. 이 사례로도 L2(LP 재생)와 원본 `5fb8ec` 경로의 결과 처리 관측까지만 가능하다.
- 후속 제안:
  - **M2-03d-6**(오라클): 이 노드(solve 19900)의 노드 LP·basis·cutoff·재풀이 결과·노드 처리 결과를 `tools/mip_retry_node_lp.js` 방식으로 독립 3회 캡처한다. 설정은 `{"NodeLimit":32898,"Seed":1,` + H}이다. M2-03d-4의 L2에 nidda RINS 사례 외에 일반 노드 사례를 더한다.
  - tier A를 더 찾으려면 매트릭스를 넓혀야 한다(seed·NodeLimit 추가, 다른 모델군). 0건이 나올 가능성이 크므로 사용자 결정으로 남긴다.
- 작업 branch·commit: `work/m2-03d-5`. 병합 여부는 main의 `Merge M2-03d-5` 이력으로 확인한다.
- 교훈: `docs/lessons/original.md`(일반 노드 재풀이 빈도·적중 사례), `docs/lessons/tooling.md`(`5ce67b` 호출자 스택으로 문맥 판별)에 추가했다.
