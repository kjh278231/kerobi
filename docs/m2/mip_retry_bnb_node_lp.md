# M2-03d-6 검증 계약과 결과: 일반 B&B 노드 재풀이의 노드 LP 원본 캡처

작성 기준은 [스프린트 검증 기준](../15_sprint_verification.md)이다. 배경은 [M2-03d-5](mip_retry_bnb_scan.md) §7이다. 03d-5 스캔은 본 탐색 일반 B&B 노드(`5fb8ec`, `5bbf7f` 깊이 1)에서 자연 상태 12 재풀이 1건을 찾았다. 이번 작업은 그 노드의 LP를 [M2-03d-3](mip_retry_node_lp.md)과 같은 도구로 캡처한다. 03d-6은 인수인계의 제안 항목이고, 사용자의 "다음 스프린트 진행" 지시로 착수했다(2026-11-10).

## 1. 범위와 주장

- 작업 종류: 오라클 구축. Rust 솔버 코드는 바꾸지 않는다.
- 완료할 동작:
  1. 03d-5 적중 노드의 재풀이 노드 LP를 캡처한다. 대상은 행렬·sense·rhs·bound·목적·vtype, LP env 파라미터(Cutoff 포함), 노드 basis, 재풀이 LP 호출과 결과, 노드 처리 사건(노드 상태 setter·`finish`·export·저장)이다. Rust가 읽을 `rsolver.node_lp.v1` 파일로 저장한다.
  2. 캡처한 LP가 simplex가 실제로 푼 LP인지 03d-3의 C1·C2로 확인한다.
  3. 원본 독립 3회에서 같은 결과를 얻는다. 같은 노드가 03d-5 적중 노드인지 확인한다.
- 조사 질문:
  - Q1. 노드 LP 행 수와 모델 대비 추가 행. 03d-5는 노드 basis 행 수 `I+0x94`=488을 봤다.
  - Q2. 재풀이 결과(LP 상태·목적값·cutoff)와 노드 처리 결과(노드 상태, basis 저장·export).
- 포함 사례: `neos-3754480-nidda`, 03d-5의 H 설정 seed 1, 노드 16948.
- 제외 범위와 후속:
  - Rust LP 재생(L2)과 `bnb.rs` 실제 경로 연결은 M2-03d-4다.
  - 추가 행(cut)의 종류·생성 순서, 재시작 내용, 첫 LP 상태 12의 원인(LU·pricing)은 다루지 않는다.
  - tier A(재시작 없는 노드) 확장 스캔은 사용자 결정 대기다.
- 선행: main `50d1b33`(M2-03d-5 병합). 미커밋 선행 변경은 없다.

## 2. 원본과 상태 경계

- Gurobi 13.0.3, SHA-256 `8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c`. 주소는 file offset이다.
- 원본 근거와 훅은 [03d-3 §2·§3](mip_retry_node_lp.md#2-원본과-상태-경계)과 같다. 재풀이 `5ce67b`(`0x4f0e10`) C:924–950, `GRBXgetvars`(`0x7a13a0`), `02f7e1`(`0xd06f0`) C:247–259다. 노드 처리 함수 `5fb8ec`는 `0x5101d0`(크기 21510, `../re/decomp/index.tsv`)이다.
- 캡처 도구는 바꾸지 않는다. `tools/mip_retry_node_lp.js`(03d-3 manifest 해시 고정)와 무영향 기준용 `tools/mip_retry_basis.js` natural 모드(03c-2 manifest 해시 고정)를 그대로 쓴다.
- 계측 사건: 공개 API 덤프(`y:lp`)는 재풀이 판단 뒤 노드 LP의 첫 top-level `cda5b` 진입에서 한 번만 일어난다. 나머지 훅은 메모리만 읽는다.
- 시작·끝 경계: 재풀이가 일어나는 `5ce67b` 호출의 진입과 반환. 그 안의 노드 상태 setter `0x5180f0`, `finish` `0x518170`, export `0x1a6300`, 저장 `0x4f8470`·`0x4f6a50` 사건도 같은 버퍼에 들어간다.
- 설정: `{"Threads":1,"Seed":1,"LogToConsole":0,"NodeLimit":32898,"Presolve":0,"ScaleFlag":0,"Quad":0,"NumericFocus":0,"MarkowitzTol":0.0001,"Heuristics":0,"RINS":0}`. TimeLimit은 없다. 03d-5 재현 3회와 같은 파라미터다.

## 3. 비교 계약

검사는 `tools/mip_retry_node_lp_check.py`에 사례 `nidda_bnb_H_s1`로 추가한다. 03d-3 두 사례의 규칙과 결과는 바꾸지 않는다. 하나라도 실패하면 종료코드 1이다.

| 검사 | 규칙 | 결과를 모를 때의 갈래 |
|---|---|---|
| A 무영향 | `y:` 사건을 뺀 사건 전체와 `DONE` 줄이 기준 실행(`mip_retry_basis.js` natural, 같은 설정, **1회**)과 정확히 같다. 기준 실행의 `tool_sha256`은 03c-2 manifest의 `mip_retry_basis.js` 해시와 같아야 한다 | 다르면 실패. 계측 영향으로 기록한다 |
| B 재현 | 3회의 `y:` 사건과 모델 덤프가 정확히 같다 | 다르면 실패 |
| F 동일 노드 | 재풀이 solve가 정확히 1개다. 그 노드 id 16948, 부모 16946, 노드 깊이 30, 비루트, `I+0x94` 488, `y:where`의 깊이 1·restarts 1·`root_is_self` false다. ACCURATE 또는 FUZZY backtrace에 `5fb8ec` 범위의 프레임이 있다. 재풀이 solve의 `call`은 19901이다(03d-5 `solve_serial`은 0부터, 이 도구는 1부터 `5ce67b` 진입을 센다. 재풀이 전에는 다른 증가가 없다) | 다르면 실패. 03d-5 적중과 다른 노드다 |
| C1 LP 불변 | 노드 LP의 top-level `cda5b` 진입마다 `D` 해시가 같고 API 덤프 시 해시와 같다. 덤프 시 `D` scale 배열이 없다 | 재풀이가 `cda5b` 직접(`node_basis_direct`)이면 노드 LP 진입은 정확히 2회다. `0c3d0d` 경로이면 03d-3 gmu 규칙(노드 LP 진입 해시 일치, `D+0x3c`=0)을 쓰고 C2는 노드 LP ws만 본다 |
| C2 ws 일치 | ScaleFlag 0이므로 scale이 없어야 한다. 재풀이 ws의 열 c·l·u, CSC, 행 b·sense가 API 덤프와 비트 단위로 같다('>' 행은 부호 반전). 첫 LP ws 해시가 재풀이 ws 해시와 같다 | 다르면 실패 |
| C3 cutoff | `lp+0xf0`가 null이면 `ws+0x3b8` = env Cutoff, `ws+0x3c0` = max(Cutoff, env+0x24b8) | `lp+0xf0`가 null이 아니면 `ws+0x3b8` = 1e100(`02f7e1` C:247–259)을 요구하고 그 사실을 기록한다 |
| C4 상태 | 재풀이 ws 상태가 `x:cda5b:leave` 상태와 같다. 상태가 2·6이면 "목적값 ≥ cutoff ⇔ 6"을 요구한다 | 12·3·5 등 다른 상태는 실패가 아니라 결과로 기록한다(B가 3회 일치를 보장) |
| E 모델 대조 | LP 열 수 = 모델 열 수. 앞 402행은 모델 행의 정확한 유리수 배수(rhs 포함, λ<0이면 '<'↔'>'), obj·ModelSense·ObjCon 같음, vtype은 같거나 'I'[0,1]→'B'. 추가 행 수·sense, bound 차이 수를 보고한다 | 재시작 뒤라 실패할 수 있다. 실패하면 첫 차이(행·열·값·λ)를 §6 계약 변경에 기록하고, 규칙은 결과를 본 뒤 넓히지 않는다. E 실패는 이 사례의 조사 결과로 남기고 나머지 검사로 오라클 판정을 한다 |
| G basis | Rust용 파일의 노드 basis(`x:decode:leave`) 길이가 LP 열·행 수와 같고, 첫 LP와 재풀이 `x:cda5b:enter`에 넘긴 vbasis·cbasis와 같다 | 길이가 다르면(예: `I+0x94`≠LP 행) 파일에는 `cda5b` 진입 배열을 쓰고 그 사실을 기록한다 |
| 노드 처리 | 재풀이 solve 버퍼의 setter 값·`finish`·export 수·저장 호출·`solve:leave` 노드 상태를 보고한다. 비교는 A(기준 실행과 정확 일치)와 3회 일치로 한다 | — |

- 부동소수는 u64 비트 패턴 16진수다. 오차 허용은 없다.
- 시간·work: Threads 1, TimeLimit 없음. 해당 없음.
- 자기검사(`--self-test`): 기존 12종 + 새 사례 변조(바이너리 해시 0, 모델 해시, 재풀이 solve 제거, 노드 id, 비영 계수, Cutoff, dhash, `y:` 이외 사건, 실행 간 차이)를 모두 거부해야 한다.

## 4. 사례와 원본 자료

| 사례 | 실제 / 합성 | 분기 | 입력 | 원본 증거 |
|---|---|---|---|---|
| nidda H seed 1 노드 16948 | 원본 실제 실행(계측 1지점) | `5fb8ec` 노드 처리 안의 재풀이(재시작 1회 뒤) | `neos-3754480-nidda.mps.gz` `63f322dd…a8bf7f` | `bench/oracle/mip_retry_node_lp/nidda_bnb_H_s1_{1,2,3}.json`, 기준 `nidda_bnb_H_s1_natural.json` |

- 적용하지 않은 범주: 동률·허용오차 경계는 해당 없다(입력 캡처). tier A 노드는 사례가 없다(03d-5).

## 5. 재실행 명령

- 원본 캡처: `tools/mip_retry_bnb_node_lp_runs.sh bench/oracle/mip_retry_node_lp`. 기준 1회와 캡처 3회, Gurobi를 한 프로세스씩 `tools/gurun.sh`로 돌린다.
- 기준 파일·manifest: `python3 tools/mip_retry_node_lp_check.py --write-manifest`.
- 저장 검사: `python3 tools/mip_retry_node_lp_check.py --self-test`. 기대 종료코드 0.
- 회귀: `python3 tools/mip_retry_basis_check.py --self-test`, `python3 tools/mip_retry_rust_expect.py --self-test`, `python3 tools/mip_retry_bnb_scan_check.py`가 0이어야 한다. 기존 기준 파일(`nidda_stress.node_lp.json`, `gmu-35-50_default.node_lp.json`)은 바이트 단위로 바뀌지 않아야 한다.

## 6. 실행 결과

(실행 후 작성)

## 7. 판정과 인수인계

(실행 후 작성)
