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

- 실행: worktree `../rsolver-m2-03d-6`, branch `work/m2-03d-6`(기준 main `50d1b33`), 계약 commit `2a5d5c7`. x86-64 WSL2 Linux, Gurobi 13.0.3(위 SHA-256), Threads 1. 다른 세션(M3-04c-2)과 Gurobi 잠금을 `tools/gurun.sh`로 나눠 썼다.
- pilot: `mip_retry_node_lp.js` 1회를 ignore된 `target/m2-03d-6-pilot/`에 돌렸다(약 14분). 재풀이 solve 1개, 노드 16948이었다. 이 pilot에서 C2·E 실패를 처음 봤다(아래 계약 변경). 기준 자료가 아니다.
- 원본 캡처: `tools/mip_retry_bnb_node_lp_runs.sh bench/oracle/mip_retry_node_lp`, 4회 모두 `ok`, 약 45분. 출력은 모두 `DONE 8 32898`이다. 계측 3회 파일은 SHA-256까지 같다(`9ef4c0ec…631c39`). 기준 실행 `nidda_bnb_H_s1_natural.json`은 `ad087a23…61adfdb`다. 노드 LP 기준 파일은 `nidda_bnb_H_s1.node_lp.json`이다.
- 검사: `python3 tools/mip_retry_node_lp_check.py --self-test` 종료코드 0이다.
  - manifest를 통과했다. 03c-2 manifest 포함, 새 파일 5개와 runner를 더했다.
  - 세 사례 모두 통과했다. 새 사례는 A·B·F·G·C1–C4·C2(빈 행 규칙)를 통과했고, E는 아래와 같이 보고한다. 03d-3 두 사례의 결과는 이전과 같다.
  - 변조 33종을 거부했다. 새 사례 21종, 기존 12종이다. 새 사례의 노드 id·basis 변조는 A가 먼저 잡으므로, F·G 판별력은 별도 9종으로 확인한다. 이 9종은 기준 실행에도 같은 변조를 넣어 A를 통과시킨다. 대상은 노드 id·부모·call·`I+0x94`·`5fb8ec` 프레임·restarts·`cda5b` basis·decode 길이·빈 행 하나 추가이고, 모두 표적 검사(F·G·C2 고정 R)에서 거부된다.
  - 디스크의 캡처 2회차에서 Cutoff 16진수 한 자리를 바꾸면 종료코드 1(`hash mismatch`)이고, 되돌리면 0이다.
- 회귀: `mip_retry_basis_check.py --self-test`, `mip_retry_rust_expect.py --self-test`, `mip_retry_bnb_scan_check.py`는 모두 0이다. 기존 기준 파일 `nidda_stress.node_lp.json`·`gmu-35-50_default.node_lp.json`은 바이트 단위로 같다(manifest에는 항목만 늘었다).

**결과** (캡처 3회와 기준 1회가 같다)

| 항목 | 값 |
|---|---|
| 동일 노드(F) | 재풀이 solve call 19901, 노드 16948, 부모 16946, 깊이 30, 비루트. `I+0x94`=488, `5bbf7f` 깊이 1, restarts 1, `root_is_self` false. backtrace에 `5fb8ec` 프레임이 있다. 재시작 `6a6563`은 call 1066–1102에서 1회 |
| 노드 LP | 253열 × 488행(모델 402 + 추가 86, 모두 '<'), 비영 4955. MIP LP(`M+8`)는 402×253이고 노드 LP와 다른 객체다. LP env+0x2728=0 |
| LP 호출 | k=1 `5ccea5+0x356` method 1, 노드 basis, 상태 12. k=2 재풀이 `5ce67b+0x4cf8` method 0(primal), 노드 basis, 상태 **2**. 둘 다 `lp+0x238` warm 있음. 분기 `node_basis_direct` |
| cutoff(C3) | `lp+0xf0` null. env Cutoff 13949.64499798452(`40cb3ed28f4b40c0`) = `ws+0x3b8` = `ws+0x3c0`. 재풀이 목적값 13652.817536684557 < cutoff, 상태 2와 모순 없음 |
| 노드 basis(G) | decode 결과 길이 253·488이 LP와 같고, k=1·k=2 `cda5b`에 넘긴 배열과 같다 |
| 노드 처리 | 재풀이 solve 안에 setter 호출 없음, `finish` 1회(ret 0), export 1회, `save_lp2`(`0x4f6a50`) 1회. `solve:leave` ret 0, 노드 상태 필드 2. LP 2일 때만 basis를 저장한다는 기록([교훈](../lessons/original.md) `5ce67b` 항목)과 모순 없다 |

**C2: simplex ws와 API LP의 차이** (캡처, 출처 미확인)

- `02f7e1` 반환 시 ws(m 488, n 253)는 API LP와 같다. 단 행 집합 R(96행)이 비어 있다. 비어 있다는 것은 계수가 없고 `b`=0이며, sense는 그대로('<')이고 slack은 [0, INFBOUND]라는 뜻이다.
- R은 모델 행 32개와 추가 행 64개로 이뤄진다. 그중 58행은 API rhs가 0이 아니다(예: 모델 행 125 rhs 10000, cut 405 rhs −0.446).
- R 밖의 항목은 순서·값까지 비트 단위로 같다. c·l·u도 같다. 비어 있는 행의 항목은 API 열에서 모두 열 끝에 있다(ws 열 = API 열의 앞부분).
- 첫 LP(k=1)와 재풀이(k=2)의 ws 해시가 같다. 두 LP 호출은 같은 비운 LP를 푼다.
- R은 노드 bound에서 중복인 '<' 행(max a·x ≤ rhs, 수치 비교)과 같지 않다. 중복 행은 120개이고 그중 R과 겹치는 것은 일부다. 그래서 검사기는 R을 캡처값(96행)으로 고정하고 3회 일치를 요구한다. 생성 규칙은 모른다.
- 출처: `02f7e1`은 `08b2fe`가 고른 자료(`lp+0xf8` → `lp+0xf0` → `D`)의 CSC 포인터를 ws에 그대로 둔다(C:109–122). 그런데 ws 길이가 API(`D`)와 다르다. 따라서 둘 중 하나다. `08b2fe`가 `D`가 아닌 자료를 돌려줬거나, `cda5b` 진입(API 덤프)과 `02f7e1` 반환 사이에 `D`의 열 길이가 바뀌었다. 둘 다 실행에서 관측하지 않았다. C 검색(subagent, 전수 보장 없음)에서 `lp+0xf8`에 쓰는 곳은 QP simplex 드라이버 `14abfb`(정방 KKT형) 하나만 찾았다. R을 비우는 원본 코드는 확인하지 못했다. **바로잡음(M2-03d-7)**: 두 갈래 모두 아니다. ws는 D 배열을 그대로 가리키고, `cda5b` C:120 `10f448`이 DynamicRows 비활성 목록으로 만든 길이·rhs 사본을 D에 잠깐 바꿔 끼운다. R은 다이브 첫 노드 3966에서 `5fb8ec` C:1856이 고른 행이다([M2-03d-7](mip_blank_rows.md)).
- Rust 재생에 주는 뜻: simplex가 푼 LP는 API LP가 아니라 R을 비운 LP다. 기준 파일에 `simplex_blanked_rows`(R)를 넣었다. 03d-3 nidda(RINS LP)는 R이 비어 있었고 지금도 그 규칙(R=∅)으로 검사한다.

**E: 모델 행 대조** (계약대로 실패, 보고)

- 400행은 03d-3 규칙을 통과한다(λ=−1 100행, λ=1/5 1행). 비영 패턴·계수·rhs 차이는 없다.
- 2행(225, 375)은 모델 '<' rhs 0인데 노드 LP에서 '=' rhs 0이다(λ=1). 첫 차이는 행 225(sense `=` vs 모델 `<`, λ 1)다. 재시작 뒤 행이 등식으로 바뀐 것으로 보인다(원인 미확인).
- 열: obj·ModelSense·ObjCon 같음, 정수 50열 'I'→'B', bound 253열 모두 모델과 다름. 정수 중 31열이 고정돼 있다(수치 비교로 0이 15열, 1이 16열). 검사기의 `int_fixed`는 16진수 문자열로 세어 ±0을 구분한다.

**계약에서 바꾼 것** (결과를 보고 바꾼 이유와 함께)

1. C2 "ws 열 길이·항목이 API와 같다"는 pilot에서 실패했다. 첫 차이는 열 0 길이다(ws 207 vs API 265). 규칙을 "API LP에서 R(ws에 항목이 없는 행)을 비우고 b=0으로 둔 것과 비트 단위로 같다"로 바꿨다. R 밖은 원래 규칙 그대로다. 03d-3 사례에는 R=∅를 요구해 원래 판정과 같다. 넓힌 것은 R의 존재뿐이다. R은 새 사례 설정에 캡처값으로 고정했다(bound 중복 규칙과 맞지 않음). 빈 행을 하나 더 넣는 변조는 거부된다.
2. E는 계약의 갈래대로 실패를 결과로 남긴다. 구현상 sense 차이만 기록하고 계속한다. 패턴·계수·rhs 차이는 여전히 실패다(변조 'bnb model row'가 계수 규칙에서 거부된다).
3. 기준 파일에 `simplex_blanked_rows`·`simplex_note` 키를 새 사례에만 더했다. 기존 두 파일은 바이트 단위로 같아야 해서 넣지 않았다.
4. 계약 §1 완료 동작 2의 "C2가 simplex LP임을 확인"을 "ws와 API LP의 관계를 확정"으로 읽는다. simplex 입력 자료의 출처는 미확인으로 남긴다.

## 7. 판정과 인수인계

- 작업 수행 상태: **완료(오라클)**. 03d-5 적중 노드와 같은 노드임을 F로 확인했다. 노드 LP·basis·cutoff·재풀이 결과·노드 처리를 기준 1회와 계측 3회(바이트 동일)로 캡처했고, 계측 무영향(A, 재풀이 solve 반환까지)과 변조 검출을 확인했다. Rust용 기준 파일도 만들었다.
- 동등성: 해당 없음(Rust 미변경). Rust LP 재생은 **미검증**이다.
- 조사 답:
  - Q1: 노드 LP는 253×488, 모델 402행 + '<' 86행이다. simplex ws는 그중 96행을 비운 LP다(출처 미확인). 2행은 '<'→'='로 바뀌었다.
  - Q2: 재풀이(primal, 노드 basis)는 상태 2, 목적값 13652.82 < cutoff 13949.64다. 노드 처리에서 basis 저장(`save_lp2`)과 export 1회가 있다.
- M2-03d-4에 주는 것: 일반 B&B 노드(`5fb8ec`)의 L2 사례가 생겼다. nidda RINS는 6, 이 사례는 2다. 재생 입력은 `nidda_bnb_H_s1.node_lp.json`의 LP에서 `simplex_blanked_rows`를 비운 LP, 노드 basis, method 0, Cutoff이고 기대 상태는 2다. 첫 LP(method 1, 상태 12)는 LU·pricing 문제라 별도다.
- 남은 한계: R을 비우는 원본 코드와 조건(코드는 M2-03d-7에서 확인, 선택 조건은 미대조), 2행 sense 변경의 원인, 86행의 cut 종류, 재시작 내용, `5fb8ec`가 재풀이 뒤 노드를 어떻게 처리하는지(재풀이 solve 밖), A의 범위(재풀이 solve 반환까지, `DONE` 줄은 NodeLimit 값이라 판별력 낮음).
- 후속:
  - M2-03d-4(L2)는 여전히 M2-10b와 M1 요청(primal warm start·LP cutoff)이 먼저다.
  - 빈 행 R의 출처 조사(새 하위 번호 제안): `02f7e1` 진입 시 `08b2fe` 반환값이 `lp+0xe8`과 같은지, D의 열 길이 배열이 언제 줄어드는지 읽기 전용 훅으로 본다.
  - tier A 확장 스캔은 사용자 결정 대기다.
- 작업 branch: `work/m2-03d-6`. 검사 통과 뒤 main에 `--no-ff`로 병합한다.
- 교훈: [원본 사실](../lessons/original.md)(B&B 노드 LP ws의 빈 행, 재시작 뒤 sense 변경, 재풀이 2와 `save_lp2`), [도구](../lessons/tooling.md)(앞선 검사가 표적 변조를 가리는 문제, `lp+0xf8` 오해)에 추가했다.
