# 11. 진행 기록과 운영

M2-03d-11 DynamicRows 관문 입력 오라클을 완료했다(사용자의 "다음 스프린트 진행" 지시로 03d-10 첫 제안 항목 착수). 새 읽기 전용 도구 `tools/mip_dive_gate.js`(03d-8 훅 유지)가 `5fb8ec` C:526(재활성)·C:1581(선택) 관문의 모든 평가에서 절 입력 원시 값(dyn, `I+0x2c`, `W+0x34`, `I+0x28`, `I+0xa0`·`I+8` null, `param_3+0x18`)과 양성 훅 결과(`510961`, `51305f`–`5131c2`, `a7f0ad`·`a7f923` 반환)를 캡처한다. 03d-8 네 사례(nidda H, DynamicRows=0 대조군, gmu-35-40, assign1-5-8)가 각 3회 바이트 동일이다. Python `predict`가 평가 160,353회의 결과를 정확히 재현했고, C:526과 같은 호출의 C:1581은 노드·dyn·깊이·`W+0x34`가 같았다(호출별 대조). 판별된 절은 dyn, 두 깊이 절, 상태=2(실패 값 4·5), `param_3+0x18`<2의 값 1 쪽이다. null 절·`a7f0ad`·`a7f923`·`+0x18`≥2는 C만이다. 38모델 sweep도 모두 일치했고 env+0x2b48은 언제나 −1이었다. 새 중간 훅은 함수형 probe로 붙였다. 객체형은 `[rsp]`를 덮어 `0x5131c2` 근처에서 프로세스가 죽었다. gmu는 03d-7의 포인터 유래 LP 번호만 03d-8과 달라 비간섭 검사를 그 번호를 뺀 투영으로 바꿨다(계약 변경). 변조 31종을 거부했다. Rust는 바꾸지 않았고 관문 이식은 다음 작업이다. [계약과 결과](m2/mip_dive_gate.md).

M2-03d-10 DynamicRows 비활성 목록 유지와 D 교체를 Rust로 이식했다(구현 L1, 사용자의 "다음 스프린트 진행" 지시로 03d-9 제안 항목 착수). `crates/mip/src/dive_rows.rs`의 `RowList`가 `10e5fc`(비활성 표시), `10e662`(전부 재활성), `edd71` 목록 블록(C:8020–8417, 행 오름차순 붙이기·마지막 원소 옮겨 빼기), `10f448`(열마다 목록 행 항목을 두 포인터 교환으로 끝에 보내고 길이·rhs 사본을 만든 뒤 D와 맞바꿈)과 `10f5ae`를 옮겼다. 원본 연산 열만 넣은 재생에서 03d-8 세 사례(nidda H, gmu-35-40, assign1-5-8)의 X 목록 변경 2,929건이 순서까지 원본 캡처와 같고, nidda 대상 노드 16948에서 줄인 D(열 앞부분 길이·ind·val, rhs)가 03d-6 `y:ws`와, 복원이 API LP와 비트 단위로 같다(`tools/mip_dive_list_rust.py --self-test`, 변조 5종·변이 4종 거부). 관문은 03d-8 캡처에 통과 사건 입력만 있어 제외했다. 남는 목록의 빼기, 중복 비활성, 분할 교환 순서(입력 D가 이미 분할됨)는 실행 0회라 C만이고, 판별되지 않는 변이 3종이 통과함을 고정했다. `bnb.rs`에는 연결하지 않았다. [계약과 결과](m2/mip_dive_list_rust.md).

M3-04c-7 primal push 위반 basis의 재시도 ratio `056973`을 오라클과 Rust로 완료했다(범위 한정). 사용자가 다음 스프린트로 04c-7을 골랐다. 스프린트 행의 theta=I 반대 방향 재호출(위반·feasible)은 `056973`과 코드 경로를 공유하지 않아 새 행 M3-04c-8로 나눴다. push는 `054eef`가 위치<0이거나 \|pivot\|<0.001이면 `056973`을 부른다. 열 계수를 1e-3보다 작게 둔 재시도 LP(18×10, PStart·DStart)에서 primal-first·dual-first 각 원본 독립 3회 8단계가 모두 재시도를 거쳤다. 재시도 결과가 `054eef`와 다른 단계(Harris 대신 최소 ratio, B·A의 둘째 breakpoint), 같은 단계, theta=I 비재호출 거리 flip, 승인·보류, 이탈 `B`(a<0)→-2·`A`(a≥0)→-1을 처음 관측했다. `056973` 내부(1단계 breakpoint·기울기, t=0, Hoare quickselect와 8원소 차선 합 (A+B)+(C+D), 2단계 ×2 유무, 대체 최소 ratio, sparse N/U의 cap 미검사, 재진입과 `12dd49` 요청 2)는 asm 판독 명세로 정리하고 합성 직접 호출 74건×3회로 고정했다. 스택 인자 순서는 `054eef`와 다르다. Rust `retry_ratio`·게이트 `violated_ratio`가 합성 68·자연 16호출에서 위치·행·theta·pivot·work 비트 일치했고 재진입 6건은 거부한다. `PushEntry::recall`로 theta=I 비재호출 단계를 거리 flip으로 판단하며, 단계 판단·게이트 연결 153단계가 일치했다. 검사기 변조 93종, Rust 변조 `056973` 22종 중 21·판단·게이트 54종 중 43(2종 연결 검출)을 거부했다. 재진입 패스 Rust, `posA-order`, 재호출 조건의 `local_1d8`=-I 해석(C만), 자연 dense, 재시도 뒤 거부는 미검증이다. 다음은 M3-04c-8 또는 M3-04c-5다. [계약과 결과](m3/primal_push_retry.md).

M2-03d-9 DynamicRows 다이브 행 선택을 Rust로 이식했다(구현 L1, 사용자의 "다음 스프린트 진행" 지시로 03d-8 제안 항목 착수). `crates/mip/src/dive_rows.rs`의 `select`가 `5fb8ec` 선택 블록(cutoff′, 모델 행 플래그·`7889a5`·maxact/minact·cutoff 경로, cut 플래그·종류·`+0x40`·slack, 순서)과 `66661c`를 옮겼고, `a7cec0`은 기존 `knapsack::act_sum`을 썼다. `66661c` 비교식은 asm 분기 형태(비순서 포함, 폭 자르기 `minsd`)다. 03d-8 run 1 캡처 세 사례(nidda H, gmu-35-40, assign1-5-8)의 입력 필드만으로 만든 재생 입력에서 선택 2,093건의 목록·순서와 `66661c` 호출 26,278건(1 반환 328)이 원본 캡처와 정확히 같다(`tools/mip_dive_rows_rust.py --self-test`, trace 변조 5종·코드 변이 5종 거부). 관문·목록 유지(`10e5fc`·`edd71`·`10e662`)와 `bnb.rs` 연결은 하지 않았다. 실행에서 판별되지 않은 절과 work 증가는 C 근거만이다. [계약과 결과](m2/mip_dive_rows_rust.md).

M2-03d-8 DynamicRows 다이브 행 선택 규칙 오라클을 완료했다(사용자의 "다음 스프린트 진행" 지시로 03d-7 제안 항목 착수). 새 읽기 전용 도구 `tools/mip_dive_rows.js`는 03d-7 훅을 그대로 포함하고 `5fb8ec`의 C:1581 관문(`0x512468`), 선택 끝(`0x513951`), `66661c`를 훅한다. 각 선택의 입력(모델 행 CSR·rhs·sense·vtype, 노드 bound, 행 플래그, `7889a5` 배열, cut 종류·`+0x40`·slack, rc, FeasTol·DualReductions·cutoff′)과 결과 목록을 캡처한다. nidda H seed 1(03d-7 설정), gmu-35-40·assign1-5-8 기본 설정(NodeLimit 3000), nidda DynamicRows=0 대조군을 각 3회 돌렸고 사례마다 바이트가 같다. nidda의 03d-7 사건 투영은 03d-7 캡처와 같다(비간섭). `tools/mip_dive_rows_rule.py`의 Python 재구성(`a7cec0`·`66661c` asm 순서 포팅)이 선택 2,093건의 목록·순서와 `66661c` 호출 26,278건(1 반환 328)을 정확히 재현했다. top-level 목록은 `10e5fc` 인자와 같다(nidda 1145). 대조군은 선택 0회이고, 재시작 `6a6563`의 `10e662` 1회는 DynamicRows 관문이 없다(C:123–124, 계약 변경 기록). 절 변종 V로 판별력을 나눴다. 모델 행 플래그, maxact ≤ rhs+0.1·FeasTol, DualReductions, '=' cutoff 제외, `66661c`, slack>1e-5, 순서는 실행으로 판별됐다. cut 플래그·`7889a5`·'=' 최소 활동값·cutoff′<1e100·cut 0x14·`+0x40`·`a7f0ad`류 관문 절은 C만이다. 변조 21종을 거부했다. 탐색 sweep 38모델(1회씩, 판정 아님)도 선택 8,365건이 모두 맞았다. nidda H 설정은 실행 중 DualReductions가 0이다. Rust·Python은 바꾸지 않았다. Rust 이식은 M2-03d-4의 선행(번호 미배정)이다. [계약과 결과](m2/mip_dive_rows.md).

M2-03d-7 B&B 노드 LP simplex ws 빈 행의 출처를 확인했다(조사, 사용자의 "다음 스프린트 진행" 지시로 03d-6 제안 항목 착수). 다이브 행 비활성이다(C로는 숨은 `GURO_PAR_DYNAMICROWS`(env+0x2b48, nidda 실행값 -1)가 관문, 0 대조는 없음). 다이브 첫 노드에서 `5fb8ec` C:528이 이전 목록을 `10e662`로 전부 재활성한다. 그 노드 LP를 푼 뒤 C:1856이 행을 골라 `10e5fc`로 표시하고 `edd71`이 X=`D+0x140`에 반영한다. 매 `cda5b` 동안 C:120 `10f448`이 D 길이·rhs를 X 사본(목록 행 항목을 열 끝으로, rhs +0)과 바꾸고 C:2110 `10f5ae`가 되돌린다. ws는 D 배열을 그대로 가리키므로 03d-6의 두 갈래(`08b2fe` 다른 자료, `02f7e1` 안 변경)는 둘 다 아니었다. 새 읽기 전용 도구 `tools/mip_blank_rows.js`로 원본 3회(바이트 동일)를 캡처했다. 대상 노드의 D·ws 해시가 03d-6과 같았고, X 목록이 R 96행과 같았다. 교체 전후 nnz는 4955↔1840이다. X 목록 633단계를 플래그 재생으로 순서까지 맞혔고(목록을 남기는 빼기는 실행에 없어 C만), 변조 22종을 거부했다. 탐색 probe 1회는 `08b2fe`=`lp+0xe8`과 ws·D 별칭을 보였고 history로 보존했다. R은 노드 16948이 아니라 다이브 첫 노드 3966(깊이 11)에서 고른 행이다. 행 선택 조건(노드 bound 중복 모델 행, slack>1e-5 cut 행 등)은 C만 읽었고 대조는 M2-03d-8(제안)이다. M2-03d-4 실제 경로에는 이 메커니즘이 필요하다. Rust·Python은 바꾸지 않았다. [계약과 결과](m2/mip_blank_rows.md).

M3-04c-2 primal push 위반 basis 방향식·infeasible ratio·`A`·`B` 이탈 arm을 오라클과 Rust로 완료했다(범위 한정). 위반 수는 LP 전체에서 세므로 후보 없는 위반 행 하나를 둔 블록 대각 LP(45×25, PStart·DStart)로 모든 단계를 위반 경로에 올렸다. primal-first·dual-first 각 원본 독립 3회(19단계)에서 σ(B 행 alpha 합-A 행 합)를 쓰는 방향식 10종단, `054eef` 단계, 거부·거리 flip·승인, 이탈 `B`→-1·`A`→-2를 관측했다. 재시도 `056973`은 새 양성 훅으로 0회를 확인했고 발동 LP에서 비간섭도 확인했다. σ=0이면 절끼리 가려지므로 절마다 꺼지면 반대 방향이 되는 판별 묶음을 넣었다. 위반 절 모델 변조 11종 중 10종을 판단만으로 거부했다(`σ>=0`은 구조상 생존). `054eef` 합성 57호출×3에서 dense 음수 `U`·`X`만 `<`임을 원본으로 확인했다. Rust `push_direction`(위반 표시 인자)·`infeasible_ratio`(꼬리를 `feasible_ratio`와 공유)가 위반 38단계(고유 19)·연결 137단계·합성 55호출에서 원본과 비트 일치했고 재진입 2호출은 설계대로 거부한다. Rust 변조는 판단 48종 중 41, `054eef` 19종 중 17을 거부했다. workspace 테스트·clippy 통과, Python 확장은 경계 변경이 없어 미실행. 위반 경로의 보류·재호출·`B`→-2·`A`→-1은 `056973` 뒤에만 오며, `056973`과 theta=I 재호출(feasible 포함)은 M3-04c-7로 분리했다. Rust에는 `056973` 게이트가 아직 없다. [계약과 결과](m3/primal_push_violated.md).

M2-03d-6 일반 B&B 노드 재풀이의 노드 LP 오라클을 완료했다. 사용자의 "다음 스프린트 진행" 지시로 인수인계의 제안 항목을 착수했다. 03d-5 적중 노드(nidda H seed 1, 노드 16948)를 03d-3 캡처 도구(변경 없음)로 캡처했다. 계측 없는 기준 1회와 계측 3회(파일 SHA-256까지 같음)이고, 검사 F로 같은 노드임을 확인했다(`5fb8ec` 프레임, 부모·깊이·`I+0x94` 488·restarts 1). 노드 LP는 253×488(모델 402 + '<' 86행)이다. 재풀이(primal, 노드 basis)는 상태 2이고 목적값 13652.82는 Cutoff 13949.64보다 작다. 노드 처리는 `save_lp2`와 export 1회다. 새 사실이 둘 있다. simplex ws는 API LP에서 96행(모델 32, cut 64)을 비운 LP다(계수 없음, b=0). 그리고 모델 '<' 2행이 '='로 바뀌었다(E 계약 실패를 결과로 기록). 빈 행의 원본 출처는 미확인이다(C 검색에서 찾은 `lp+0xf8` 쓰기는 QP 드라이버뿐, 실행 관측 없음). bound 중복 규칙과도 맞지 않아 R을 캡처값으로 고정했다. Rust용 `nidda_bnb_H_s1.node_lp.json`에 `simplex_blanked_rows`를 넣었다. `tools/mip_retry_node_lp_check.py --self-test` 종료 0이고 변조 33종을 거부했다(F·G·고정 R 표적 9종 포함). 기존 기준 파일은 바이트 단위로 같다. Rust·Python은 바꾸지 않아 해당 검사를 실행하지 않았다. M2-03d-4(L2)는 여전히 M2-10b·M1 요청이 먼저이고, 빈 행 출처 조사와 tier A 확장은 후속이다. [계약과 결과](m2/mip_retry_bnb_node_lp.md).

M2-03d-5 일반 B&B 노드 자연 재풀이 스캔을 완료했다(사용자 결정). 새 읽기 전용 스캔 `tools/mip_retry_bnb_scan.js`가 `5ce67b` 직접 호출자와 `5bbf7f` 중첩 깊이로 재풀이를 tier A(재시작 없는 본 탐색 노드, `5fb8ec`·깊이 1·부모 있음)·B(재시작 뒤 본 탐색 노드)·C(휴리스틱·sub-MIP·루트)로 나눈다. 대조군 nidda·gmu 재풀이는 03d-3 기록대로 C로 분류됐다. 고정 매트릭스 228회(MIPLIB 38개 × stress+Heuristics·RINS 0, 여기에 Cuts 0 × seed 3)가 모두 정상 종료했고, tier A/B 합계 노드 LP 약 984만 건 중 재풀이는 tier B 1건이다. tier A는 0건이다(tier A만의 분모는 미측정). 적중은 `neos-3754480-nidda` seed 1의 노드 16948(부모 16946, 깊이 30, 재시작 1회 뒤, 노드 basis 488행)이며 TimeLimit 없이 NodeLimit 32898로 독립 3회 재현했다. 검사기는 manifest·매트릭스 완전성·파라미터·재현 일치를 확인하고 `--self-test`는 변조 18종을 거부했다. 재시작과 cut 86행 때문에 Rust 자연 실행으로는 이 노드에 도달할 수 없다. 다음은 이 노드의 LP 캡처(M2-03d-6 제안)이고, tier A 확장 스캔과 M2-03d-4 진행 방식은 사용자 확인이 필요하다. Gurobi 외 Rust·Python 검사는 코드 변경이 없어 실행하지 않았다. [계약과 결과](m2/mip_retry_bnb_scan.md).

M3-04b-3 primal push 거부·보류·미관측 이탈 arm 오라클을 완료했다. 원본 공개 API의 시작 벡터(PStart·DStart, Method=2, LPWarmStart=2, ScaleFlag=0)로 arm마다 노린 블록 대각 LP(42열×25행, `tools/push_arm_lp.py`)를 만들었다. primal-first·dual-first 각 독립 3회(25단계씩)와 Crossover=0 대조군 3회를 캡처했다. 거부 두 arm, pivot 보류, 고정폭 -4, `U`(a<0) -2, `U`(a≥0) -1·-2, `L`(a<0) -2·-1, 첫 절 방향 1, 방향 1 거리 flip, 진입 slack을 관측했다. 보류는 사건 부재 추정 대신 `0x1c3b7b` 양성 훅(새 agent)으로 확인했다. 들어오는 경로는 승인 `ja` 두 개뿐이다. face5에서 새 사건을 빼면 04b-2 캡처와 같다(비간섭). 04b-2 판단 모델과 04c-6 Rust(소스 무변경)가 arm 50단계(고유 43)와 연결 ratio 99단계를 원본과 비트 일치로 재구성했다. 모델 변조는 32종 중 27종을 판단만으로 거부했고, Rust 소스 변조는 36종 중 30종을 거부했다(04c-6 24종 중 9). 남은 생존은 C 근거로 분류했다. `A`·`B`·a<0 기본 arm은 위반 수 `local_1b0`이 0일 때만 feasible 경로로 가므로 도달 불가다. `X`는 `S+0x3b0`=0에서 고정폭에 가려진다. 그 밖에 ±0 pivot, 재호출 경로, `U`(a≥0)의 abs(a)t==2h 정확 경계(미검증)가 남는다. 보류의 `theta*pivot` 쪽과 구조 열의 첫 절 참도 미관측이다. Rust workspace 테스트·clippy·Python 확장은 Rust·Python 소스 변경이 없어 미실행이다(재생 테스트 `push_decision`·`push_ratio`는 실행). 다음은 M3-04c-2(infeasible ratio·위반 basis) 또는 M3-04c-5(재진입)다. [계약과 결과](m3/primal_push_arms_oracle.md).

M3-04c-6 primal push 단계 판단을 Rust(`crates/lp/src/crossover.rs`)로 구현했다. 진입 열의 r·dL·dU·방향식·열 부호 반전, 거리 flip·거부·pivot 승인·보류, 이탈 상태와 루프 상수를 asm 비교 형태대로 옮겼다. 04b-1·04b-2 저장 캡처(순서별 원본 3회)를 원시 필드로 재생했다. 대상 40단계(고유 입력 20)에서 방향·반전 alpha·단계 종류·이탈 상태가 비트 일치했다. theta는 거부가 관측되지 않아 ratio 값이 그대로 나오므로 판별 근거로 세지 않는다. frame이 있는 22단계와 비간섭 보조 9단계에서는 r·dL·dU·상수까지 일치했다. Rust가 반전한 열로 돌린 04c `feasible_ratio`도 49단계 모두 원본 `ratio:leave`와 같았다(연결 검사). 오라클 모델 변조 20종 중 19종을 Rust 소스 변조로 옮겨 Rust 전용 5종과 함께 돌렸다. 판단 4·r 비트 1·상수 4로 거부했고 오라클과 분류가 같다. 생존 15경계와 미관측 arm(첫 절 방향 1, 진입 slack, 방향 1 거리 flip, 거부, 보류, 고정폭·`X`·`A`·`U`·`B`·`L`(a<0))은 미검증이다. Gurobi는 실행하지 않았다. Rust workspace 테스트·clippy는 통과했고 Python 확장은 변경이 없어 미실행이다. 다음은 M3-04b-3 또는 M3-04c-5다. [계약과 결과](m3/primal_push_decision_rust.md).

M2-03d-3 재풀이 노드 LP 오라클과 조사를 완료했다. 원본 두 사례의 노드 LP를 공개 API로 캡처해 3회 재현했다. 대상은 행렬·bound·목적, env 파라미터 184개, cutoff다. 새 사건을 뺀 캡처는 M2-03c-2 저장본과 같아 계측 영향이 없다. simplex 작업 배열(nidda)도 캡처와 비트 단위로 같다. Rust가 읽을 기준 파일은 `bench/oracle/mip_retry_node_lp/*.node_lp.json`이다. 두 사례 모두 일반 B&B 노드가 아니다. nidda 노드 8287은 재시작 뒤 본 탐색에서 RINS(`8bc185`→`8e3397`→`8e4682`)가 정수를 모두 고정해 푼 연속부 LP다. 행 402개는 모델 행의 유리수 배수(부호 반전 100, 1/5배 1)이고 62행은 모델에 없는 '<' 행이다(cut 추정). gmu는 깊이 2 sub-MIP(200×81→110×53)의 루트다. `tools/mip_retry_node_lp_check.py --self-test` 종료코드 0이고 변조 12종을 거부했다. Rust는 바꾸지 않았다. M2-03d-4의 L2는 이 LP 재생으로 할 수 있다(nidda는 재풀이 LP 하나, gmu는 `0c3d0d` 안의 k=3만). 그러나 `bnb.rs` 실제 경로를 검증하려면 일반 B&B 노드의 자연 재풀이 사례가 따로 필요하다(현재 없음). [계약과 결과](m2/mip_retry_node_lp.md).

M3-04b-2 primal push 판단 오라클을 완료했다. ratio 뒤 거부·이탈 상태가 읽는 `0x1ec8`·`0x1eb0`·`0x1e88(%rsp)`는 루프 준비에서 h=`S+0x380`의 `100h`·`-100h`·`h+h`로 한 번 쓰는 상수임을 asm·CFG(`tools/push_slot_reach.py`)와 새 agent의 frame 비트로 확인했다. 판단 모델(`tools/primal_push_decision_check.py`)이 04b-1 저장 캡처 18단계와 목적 있는 `face5.lp` primal-first·dual-first 각 독립 3회(11단계, 방향 1 3건, r≠0 7건)에서 방향·alpha 반전·단계 종류·theta·이탈 상태를 원본과 같게 재구성했다. 새 agent는 frame 필드를 빼면 04b-1 사건과 같다. 방향 1 3건은 모두 r>0 절이다. 캡처·판단 변조 24종을 거부했다. 모델 변조 20종 중 판단으로 3종, r 비트로 1종을 거부했고 상수 4종은 frame 값 대조와 같은 정보다. 2h는 값만 확인했고 읽는 arm은 미관측이다. 거부 판정 발동·pivot 보류·`A`·`U`·`B`·`X`·고정폭·`L`(a<0) arm은 9개 LP 탐색에서 미관측이며 생존 12경계와 함께 M3-04b-3으로 넘긴다. 당시 다음 구현은 M3-04c-6(Rust 판단)이었고 결과는 위 기록을 따른다. 솔버 코드는 바꾸지 않았다. [계약과 결과](m3/primal_push_decision_oracle.md).

M3-04c feasible ratio `055f68` 첫 패스를 Rust(`crates/lp/src/crossover.rs`)로 구현했다. 04b-1 자연 캡처의 ratio 9입력(primal-first·dual-first 각 3회)과 04b 합성 29사례×3회를 저장 기준으로 재생해 위치·theta·pivot이 비트 단위로 일치했고, 합성은 work도 일치했다. 자연 호출의 xmm 인자는 asm으로 `S+0x380/0x398/0x370`임을 확인해 재구성했다. Rust 소스 변조 14종 중 10종을 거부했고, 살아남은 2차 상한 동률·후보 있는 I 상한·작은 pivot 임계값 4경계는 미검증이다. 재진입 패스는 원본에서 관측되지 않아 Rust가 거부한다. 인계 문구의 pivot·flip 판정·적용은 캡처에 없는 `0x1ec8(%rsp)`(출처 미확정)·`local_250`을 읽어 이번 경계에서 빼고 M3-04b-2로 넘겼다. Gurobi는 실행하지 않았다. Rust workspace 테스트·clippy 통과, Python 확장은 변경이 없어 미실행. [계약과 결과](m3/primal_push_ratio_rust.md).

M3-04b-1 정상 primal push 한 단계 오라클을 완료했다. `interior.lp`(8×12)에서 primal-first·dual-first를 각각 원본 독립 3회 실행해 후보·ratio·pivot·flip·교환 직후·factor 후 상태가 비트 단위로 일치했고, 각 실행에 pivot 7건·flip 2건이 있다. Crossover=0 대조군 3회는 push 0건이다. 실제 캡처 변조 15종×2를 모두 거부했다. 두 순서의 primal push 단계 순서는 같고, infeasible ratio·batch·x87·factor 내부·Rust 동등성은 미검증이다. 솔버 코드는 바꾸지 않았다. 다음은 M3-04c다. [계약과 결과](m3/primal_push_step_oracle.md).

M2-03d-2 재풀이 판단 제어를 구현했다(사용자 승인: L1 합성 재생, 03d-2 우선). `crates/mip/src/retry.rs`의 `process_node_lp`는 LP를 클로저로 받아 첫 LP·재풀이의 basis·method와 재풀이 결과 처리만 결정한다. B&B에는 연결하지 않아 실제 경로는 `bnb.rs:176`의 Numeric 종료 그대로이고, dual·cold 대체도 하지 않았다. nidda 기준 자료에 원본 LP 결과(12, 6)를 주입한 L1 trace가 `tools/mip_retry_rust_expect.py --compare`를 통과했다. 재풀이 method·저장·노드 상태 변이 3종은 실패로 검출했다. `cargo test --release --workspace`와 clippy 종료코드 0이다. L1 통과는 원본 LP 결과를 주입한 판단 구간의 일치일 뿐 Rust LP 동등성이나 실제 경로 검증이 아니다. M1 인수인계에 primal warm start·LP cutoff 요구를 추가했다. 다음은 M2-03d-3 노드 LP 캡처(62행·sub-MIP 조사)이고, 실제 경로 연결과 LP 포함 동등성은 M2-03d-4다. [계약과 결과](m2/mip_retry_control.md).

M2-03d-1 Rust 재풀이 비교 경계 조사를 완료했다. 원본 재풀이 두 사례는 Rust 자연 실행으로 같은 입력에 도달할 수 없다. nidda 노드 8287의 LP는 253열×464행으로 모델 402행보다 62행이 많고(cut 등 추정), `8e4682`가 부모 없이 만든 노드다. gmu는 53열×110행으로 모델 1919×435와 달라 sub-MIP 루트로 추정한다. Rust `Numeric`은 primal phase 1에서만 생겨 원본 12와 원인도 다르다. 그래서 03d-2 경계를 원본 LP 결과(첫 12, 재풀이 6)를 주입한 재풀이 제어 재생(L1)으로 정했다. 비교 대상은 첫 LP·재풀이의 basis·method, 노드 상태, basis 저장 여부다. 사건 형식 `rsolver.mip_retry.v1`과 nidda 기준 자료를 `tools/mip_retry_rust_expect.py`로 저장 캡처 3회에서 투영했고 `--self-test` 종료코드 0, 변조 18종을 거부했다. Rust에는 노드 basis·상속·노드 method·primal warm start·LP cutoff 상태가 없으며 뒤 둘은 M1 영역 선행 조건이다. Gurobi는 실행하지 않았고 솔버 코드는 바꾸지 않았다. 다음 M2-03d-2(L1 구현)는 "자연" 전제 변경과 진행 순서(M2-10b basis 저장·상속, M1 선행 먼저?)의 사용자 확인 대기다. 노드 LP 원본 캡처와 LP 포함 재생은 M2-03d-3이다. [계약과 결과](m2/mip_retry_rust_boundary.md).

M2-03c-2 재풀이 basis 오라클을 완료했다(main 병합). Gurobi 원본 읽기 전용 훅으로 자연 상태 12 재풀이 2건(nidda 비루트, gmu 루트)을 사례·모드별 독립 3회, 총 12회 캡처했다. 새 훅을 붙여도 03c-1 사건 투영이 저장본과 같음을 확인했다. 재풀이 입력 basis는 실패한 LP의 마지막 basis가 아니라 노드에 저장된 시작 basis다. nidda에서는 노드 8278의 `5d4427`→`113d92` export, `8e4682`의 `69e299` 복사, `69d237` 압축, `69d7ea` decode를 거쳐 첫 LP와 재풀이 `cda5b`에 같은 `I+0x98/0xa0`가 들어간다. 재풀이 method는 0이었다. 재풀이 뒤 원본의 VBasis export는 nidda(LP 6)에서 없고, gmu(LP 2)에서는 `5d4427`이 만든다. 재풀이에서는 `2f7e1`이 `ws+0xf0`를 12→1로 되돌린 뒤 `325b4`가 노드 basis를 다시 적재하고 분해를 새로 한다. dual norm 입력은 없다. 검사기 `--self-test` 종료코드 0, 변조 17종을 거부했다. LU·가중치 내용, Rust 동등성은 미검증이고 솔버 코드는 변경하지 않았다. 다음은 M2-03d-1 Rust 재풀이 비교 경계 설계이고, 구현은 M2-03d-2다. [계약과 결과](m2/mip_retry_basis.md).

M3-04b primal push 분기 조사를 완료했다. `134a87`·`256781`의 double·x87 경로, 후보 정렬·방향·ratio·bound 이동·basis 교환·재분해와 signed remaining을 원본 C·ELF로 분할했다. `055f68` 합성 29사례를 원본 독립 3회, 총 87호출 실행해 사건 필드 9개와 수치 비트를 비교했고, CLI 변조 10개를 모두 거부했다. [계약·분기표·실행 결과](m3/primal_push_branches.md). 전체 push 자연 실행·Rust·x87·infeasible ratio·factor 동등성은 미검증이며 다음은 M3-04b-1 정상 push 한 단계 오라클이다. 솔버 코드는 변경하지 않았다.

M3-04a-1 정상 crossover 진입 오라클을 완료했다. 소형 LP 하나에서 PDHG·barrier mode 0과 시작 벡터 mode 1을 각각 원본 독립 3회 실행했고 경로별 상태·배열이 비트 단위로 일치했다. 검토자도 세 경로를 각 1회 재캡처해 일치시켰다. Crossover=0 대조군은 목표 0건×3회, 실제 캡처 오류 주입 7개와 추가 변조 검토·합성 self-test도 통과했다. 원본 분기에 따라 warm 설정을 Method=1에서 2로 바꿨으며 기존 실패 3개는 보존했다. [계약·원본 근거·실행 결과](m3/crossover_entry_oracle.md). Rust·시간·work·다른 모델 동등성은 미검증이다. 후속 primal push 조사는 맨 위 M3-04b 기록을 따른다.

M3-04a 정적 조사를 완료했다. PDHG·barrier·시작 벡터의 crossover 진입, 배열 순서·변환·할당 실패·환경 복원과 cleanup 내부 상태 배열 설치 경계를 원본 C·ELF로 확인했다. 정적 조사만으로 Rust 동등성을 판정하지 않는다. [조사 기록](m3/crossover_entry_contract.md). 후속 원본 오라클 결과는 위 M3-04a-1 기록을 따른다.

M2-03c-1은 **부분 완료**다(main 병합). 사용자 결정으로 계약을 바꿨다. Gurobi 원본에서 자연 상태 12 재풀이 2건(비루트·루트)의 입출력·파라미터 복구·basis 입력/header, 합성 5→12 재분류, 상태 3 호출자 `682037`을 각 3회 검증했다. 병합 전 원본 33회 재실행에서 비교 30건이 모두 일치했다. 재풀이 후 VBasis·입력 basis 출처·factor/norm은 M2-03c-2로 분리했다. 상태 10·자연 5→12(정상 산술에서 불가능, 미관측)는 미검증이다. Rust는 변경하지 않았고 동등성은 미검증이다. 상세는 [M2-03c-1](m2/mip_linear_child_trace.md) §10–13.

## 현재 작업 상태 (2026-11-10)

Herdr의 `gpt-6.1-terra`에게 M2-03c-1을 맡기고 별도로 검토했다. 첫 결과 `3e9fc00`은 목표 사건 0회로 미완료였다. 수정 라운드 중 작업자가 사용 한도로 중단돼 점유를 인계받아 커밋했다. 도달 판정·자연 스캔과 사용자 결정에 따른 계약 변경을 거쳐 부분 완료로 병합했다(맨 위 참조). 위임·검토 규칙은 [인수인계 절차](handoff/README.md) §5에 정리했다.

M2-03b-1 선형 무계 재분류·보류 소비 조사를 완료했다. 원본 `5ce67b`는 루트 캐시 2에서 LP 상태 5를 12로 바꾸고 즉시 수치 재풀이를 시도한다. 노드 상태 3은 `60145b`가 분기 대상으로 인정해 `627f3e`의 domain 기반 대체 분기로 보내며, 연속 변수만 남으면 상태 10인 새 자식을 만들 수 있다. 같은 노드 재등록은 상태 7·8·10에 한정된다. Gurobi 13.0.3 원본 독립 프로세스 3회에서 setter 54회·재등록 조기 반환 54회·분기 자격 48회, 총 156회가 일치했고 오류 주입 4개를 검출했다. 호출자 재분류·재풀이와 실제 자식 연결은 정적 조사이며 Rust 동등성은 미검증이다. 솔버 코드는 바꾸지 않았다. [계약·근거·검증·훅 경계](m2/mip_unbounded_reclassification.md). `work/m2-03b-1`의 독립 worktree에서 작성하고 검증 후 main에 병합한다. 다음 작업은 M2-03c-1 선형 호출자·대체 분기 오라클이다.

M2-05a-1 선행 변경 정리와 브랜치 운영 스킬 생성을 완료했다. 기존 dirty main의 코드·오라클, 측정 자료, 세션·검증 문서, barrier 백업을 네 커밋으로 나눠 `09dc65f`에 병합했다. 네 커밋의 tree는 원래 `7ea2301`과 동일하다. M2-03a의 `37d2e89`·`657d031`만 새 기준선에 재적용해 `ebb35d6`에 별도로 병합했고 재적용 전후 tree도 같다. 기존 솔버 코드는 수정하지 않았다. Rust workspace 113개·Clippy, Python 확장 재빌드 후 653개, 세션 도구 3개가 통과했다. 원본 type 4 1,000개×3회가 기존 fixture와 일치했고 type 10·11 실제 레코드 156개 및 두 모델의 축약해 복원을 재검증했다. M2-03a 저장 기준 13개×3회 재생도 통과했다. [정리 계약·명령·commit 대응](m2/baseline_reconciliation.md).

[verified-task-branches 스킬](../.agents/skills/verified-task-branches/SKILL.md)을 저장소 원본과 개인 스킬 경로에 생성했다. 최신 main에서 하위 작업별 독립 worktree를 만들고 계약 검증 후 `--no-ff`로 병합한다. 미병합 상세 결과를 main에 미리 복사하는 절차를 바꾸고 AGENTS·인수인계를 연결했다. 운영 변경은 `work/shared-branch-policy`에서 작성해 병합한다. 종료 상태는 main의 미커밋·미병합 작업 없이 정리하며 시작 상태와 원래 스냅숏 이력은 `safety/main-dirty-20261110`·`safety/m2-03a-snapshot-20261110` ref에 보존한다. 당시 다음 솔버 작업은 M2-03b-1이었다. 이번 조사 결과는 위 기록을 따른다. 자식 무계·보류 재처리·ray·presolve 내부 중단과 탐색 궤적 차이는 남는다. 아래 미커밋·미병합 설명은 각 작업 당시의 기록이다.

M2-03a 직접 무계 확정 조건 조사를 완료했다. `600d25`의 직접 노드 상태 11은 추가 제약 문맥에서 재풀이·domain predicate를 통과한 경우이며, 순수 선형 입력은 노드 상태 3으로 보류된다. 루트 무계는 호출자 `5ce67b`가 별도로 확정하고, 루트 캐시가 2이면 현재 LP 상태 5를 12로 재분류한다. Gurobi 13.0.3 바이너리 해시를 고정한 호출 없는 합성 상태 13개를 독립 프로세스 3회, 총 39개 확인했고 상태·카운터·보존 필드 변조 3개를 모두 검출했다. 직접 상태 11·재풀이·ray·실제 자식 연결은 실행 미검증이며 Rust는 변경하지 않았다. 다음은 M2-03b-1의 재분류·보류 후 재처리 조사다. [계약·조건표·재실행 결과](m2/mip_unbounded_entry.md). 기존 main 미커밋 변경을 `7ea2301` 스냅숏으로 보존한 독립 worktree `../rsolver-m2-03a`, branch `sprint/m2-03a`에서 작업했다. 작업 결과는 선택 커밋하고 main에는 미커밋으로 복사한다. branch는 미병합이며 main의 기존 변경을 보존했다.

스프린트 검증의 핵심 기준을 [동작 동등성 검증 기준](15_sprint_verification.md)으로 작성했다. 실제 Gurobi 13.0.3 바이너리 실행을 기준으로 같은 입력 상태의 판단·출력·상태 변경을 구간·연결·전체 층에서 비교한다. 원본 오라클 방식·재현성·필드별 비교·시간과 work·사례와 오류 검출·기준 자료·완료 판정을 명시하고 [계약 양식](templates/sprint_acceptance.md)을 추가했다. AGENTS·스프린트 목록·영역별 설계·인수인계 시작 절차에서 필수 참조한다. P0~P3 관문 수치는 유지하며 집계 일치가 내부 동등성의 증명이라는 표현을 수정했다. 문서 링크·참조·변경 공백을 확인했고 솔버 코드·오라클은 실행하지 않았다. 이번 문서와 기존 변경은 main worktree에 미커밋이며 별도 미병합 worktree는 없다.

새 세션과 세 영역 동시 작업을 위한 시작 절차를 `AGENTS.md`에서 [인수인계 안내](handoff/README.md)로 연결했다. 영역별 다음 작업·선행 변경·검증 상태를 M1·M2·M3 인수인계에 기록했다. `tools/session.py`는 같은 git common-dir의 영역 점유, 환경 확인과 미커밋 변경을 보존한 오프라인 bundle 생성을 지원한다. 별도 컴퓨터 간 자동 점유 동기화는 제공하지 않으며, 기존 원격 금지 정책을 유지해 비공개 패키지 전달과 영역 사전 배정을 사용한다. 임시 저장소에서 점유 충돌·다른 worktree의 중복 점유·소유자 해제·수정·삭제·미추적 파일 복원 및 index 보존 검사가 통과했고 현재 환경 doctor를 확인했다. 솔버 오라클은 재실행하지 않았다. 기존 변경과 설정은 main worktree에 미커밋이며 별도 미병합 worktree는 없다.

1주 스프린트를 기준으로 기존 24개 번호를 상위 작업으로 유지하고 [하위 작업 목록](14_sprint_plan.md)을 추가했다. 각 항목에 종류·범위·선행 조건·종료 확인을 적었으며, 원본 미해독 영역은 설계 조사 뒤 구현을 추가한다. M1·M2·M3의 상태 소유권·데이터 전달·오라클 경계를 설계 보완 문서로 정리했다. 문서 링크·번호·변경 공백을 확인했으며 솔버 검사나 원본 오라클은 재실행하지 않았다. 기존 코드 변경과 이번 문서는 main worktree에 미커밋이며 별도 미병합 worktree는 없다.

M1·M2·M3를 식별 가능한 작업 번호로 나누고 상태·완료 확인·상세 기록을 [작업 단위 목록](13_work_units.md)에 정리했다. 기존 G1~G6 관문은 유지한다. 문서 변경은 main worktree에 미커밋이며 별도 worktree는 없다.

자식 LP 무계 처리를 조사했다. 원본 `600d25`는 조건에 따라 무계 방향을 검사해 노드 상태 11 또는 3을 선택하며 `5c239f`는 상태 11을 MIP 무계로 전달한다. 직접 진입 조건과 방향 검사 전체, 자식 노드 재현·오라클은 아직 미확인이라 구현은 바꾸지 않았다. [조사 기록](m2/mip_recovery_bnb.md). main의 기존 변경과 이 문서는 미커밋이며 별도 worktree는 없다.

MIP 복원 후속: type 4 flag 1·2의 정수 보정을 `35d235` 및 어셈블리와 대조해 추가했으며 합성 Gurobi 오라클 1,000개와 일치한다. type 10·11과 빈 축약 모델도 연결했다. mas74·mas76은 축약 경로의 원본 해 검사를 통과했고 enlight_hard는 축약 후 복원해 최적 목적 37을 확인했다. neos-860300·graphdraw-domain은 rsolver가 30초 내 해를 찾지 못했으나 Gurobi가 각 축약 모델에서 찾은 해를 복원해 원본 제약·정수성·목적 3201·19848을 검증했다. 38개 재측정은 전부 축약 경로, 최적 1·시간 초과 37, 원본 해 검사 22개 통과다. 진단 API 추가 뒤 Rust workspace 테스트·Clippy와 Python 테스트 653개가 통과했다. [작업 기록](m2/mip_recovery_bnb.md). 변경은 main worktree에 미커밋이며 별도 미병합 worktree는 없다.

MIP type 4 flag 0 복원을 기존 LP 분할로 연결하고 `363164` 합성 오라클 4개와 Rust 회귀 검사를 통과했다. 실제 mik-250-20-75-4는 축약 경로의 원본 해 검사를 통과했고 timtab1은 축약 경로에서 시간 내 해가 없었다. Rust workspace 테스트·Clippy와 로컬 확장 사용 Python 테스트 652개가 통과했다. type 4 flag 1·2, type 10·11과 빈 축약 모델은 아직 우회한다. 상세 [작업 기록](m2/mip_recovery_bnb.md). 작업 중 별도 문서 재배치가 `f86edd8`로 main에 커밋됐다. MIP 변경은 그 위 worktree에 미커밋이며 별도 미병합 worktree는 없다.

문서 탐색을 위해 상세 기록을 `docs/m1/`, `docs/m2/`, `docs/m3/`로 묶고 [문서 안내](README.md)를 추가했다. 기존 작업은 main worktree에 미커밋이며 별도 미병합 worktree는 없다.

`AGENTS.md`에 원본 복원 기준과 함께 프로젝트 구조, 현재 상태·원본·오라클을 찾는 경로, 빌드·검증과 공유 환경 규칙을 정리했다. 프로젝트 구조와 원본 경로는 개조식으로 구분했다. 경로와 기존 문서를 확인했으며 솔버 동작은 변경하지 않았다. 지침과 이 기록은 main worktree에 미커밋이고 별도 미병합 worktree는 없다.

MIP 해 복원·B&B 동기화를 시작했다. Gurobi 13.0.3 `363164`의 type 9 식과 역순을 합성 원본 오라클로 확인하고 적용했다. 서로 다른 이동·배율과 고정 레코드를 섞은 회귀 검사 및 B&B Python 테스트 404개가 통과했다. 38개를 5초 재측정하여 축약 31·미지원 우회 6·빈 축약 우회 1, 원본 해 검증 21개 통과, rsolver 시간 초과 38을 기록했다. 기존 type 9 대상 여섯 모델 중 세 모델의 해를 검증했고 두 모델은 해가 없었으며 하나는 빈 축약 우회였다. type 4·10·11의 실제 오라클 검증과 구현, 자식 LP 무계와 presolve·전파 시간 제한은 남았다. 호출 대응표·레코드 빈도·측정·한계는 [작업 기록](m2/mip_recovery_bnb.md)에 있다. 변경은 main worktree에 미커밋이며 별도 미병합 worktree는 없다.

`m2-mip-presolve`에서 B&B, 루트 전파·휴리스틱, presolve 후 B&B 및 고정·치환 변수 복원, Python MIP `optimize()` 연결을 구현했다(`13ce2fe`, main 병합 `3e5c9d4`). 다른 복원 레코드와 빈 축약 모델은 원래 모델을 직접 푼다. 나머지 MIP 복원과 MIP 콜백은 남았다. 별도 검증: G6 기본 축약 모델·postsolve 레코드·PRELIMIT 저장 변화점 및 최종 모델 각 38/38, workspace clippy 경고 0·Rust 테스트 통과·Python 테스트 498개 통과. 38개에서 실행되지 않은 하위 풀이 등은 미구현이다.

사용자 요청에 따라 `main` worktree 하나만 남겼다. 제거한 `rsolver-barrier`의 미커밋 진행 문서와 미추적 파일은 `worktree-backups/rsolver-barrier/`에 복사했다. 이 백업과 현재 worktree의 상태 기록은 미커밋이다. 2026-11-10 사용자 요청으로 이 백업을 삭제했다. 내용은 2026-11-09 시점의 옛 문서 경로·상태이고 현재 문서로 대체됐다. 원본은 `git show 42ab9d0:worktree-backups/rsolver-barrier/<파일>`로 복구한다.

`m1-lp-driver`를 `87ad61e`로 main에 병합했다. LP·MIP 경로의 네 파일 충돌과 MIP 테스트 모델의 RNG 초기화를 해결했다. 병합 결과를 임시 복제본에서 검증하여 Rust workspace 테스트, Clippy 경고 0, Python 테스트 650개가 통과했다. `m2-root`와 `m2-mip-presolve`를 포함해 남은 로컬 브랜치는 모두 main에 병합된 상태다. G5의 etamacro 요약 목적값과 simplex 추가 반복의 수치 차이는 남는다(`docs/m1/m1_driver.md` §11).

2026-11-09 MIPLIB B&B 첫 기준 측정: Threads=1, 내부 TimeLimit=2초, 외부 감시 6초에서 rsolver 37개 시간 초과·1개(`cod105`) 외부 감시 중단, Gurobi 5개 최적·1개 불가능·32개 시간 초과였다. rsolver 해 20개는 원래 모델에서 검증했다. 별도 presolve 레코드 조사에서 12/38에 미지원 유형이 나왔으나 실제 원본 직접 풀이 경로 비율은 아직 계측하지 않았다. presolve 전체 시간 제한이 동일하게 적용되지 않는 한계와 모델별 수치는 [기준표](m2/mip_bnb_baseline.md)에 있다. 기준표·원시 결과·측정 도구와 이 문서 변경은 main worktree에서 미커밋이며 따라서 커밋된 main에는 아직 반영되지 않았다. 단일 worktree가 main을 사용하므로 별도 미병합 작업 브랜치는 없다. 기존 미추적 AGENTS.md와 백업도 남아 있다.

2026-11-09 후속: `cod105`의 2초 제한 초과는 presolve·휴리스틱을 끈 루트 simplex에서 재현됐고 반복 제한 1에서는 즉시 종료했다. simplex 반복에 마감 확인을 연결하고 presolve부터 같은 시계를 사용한 뒤 2초 재현이 약 2.04초에 반환됐다. 실제 풀이 경로 계측을 추가해 5초 재측정한 결과 rsolver 38개 시간 초과·외부 감시 중단 0, 축약 풀이 26·미지원 레코드로 원본 풀이 12, 원래 모델 해 검증 21개 통과였다. Gurobi는 최적 6·불가능 1·시간 초과 31이다. B&B·루트 Python 테스트 436개, Rust workspace 테스트와 workspace Clippy가 통과했다. 단일 presolve·전파·LP 연산 중의 즉시 중단은 아직 지원하지 않는다. 상세 [5초 기준표](m2/mip_bnb_5s.md). 코드·검증 결과·문서는 main worktree에 미커밋이며, 별도 미병합 브랜치는 없다.

## 1. 운영 규칙 (2026-11-05 사용자 지시)

이 절의 Herdr 탭·branch 표는 당시 운영 기록이다. 현재 새 세션의 작업 선택·점유·이동 절차는 `AGENTS.md`와 [인수인계 안내](handoff/README.md)를 따른다.

오케스트레이터(Herdr 워크스페이스 w4 1번 탭의 Claude 세션)가 M 단계 작업을 탭별 에이전트에게 나눠 맡긴다.

1. **병렬**: 다른 파일·크레이트를 건드리고 따로 검증할 수 있는 단위는 별도 git worktree와 탭에서 동시에 돌린다. 관문 판정은 마일스톤 순서대로 하되, 독립 단위는 앞당겨 진행한다. 브랜치를 main에 합치는 일은 오케스트레이터가 한다.
2. **토큰 한도**: 3시간마다 한도로 멈춘 에이전트를 찾아 이어서 작업을 요청한다.
3. **컨텍스트 70%**: 15분마다 확인한다. 70% 이상이면 순서대로 한다.
   1. `/goal clear`
   2. 하던 작은 단위를 마치고 인수인계 메모(§2 표)를 갱신해 커밋하게 한다.
   3. 같은 탭에서 `/clear`로 새 세션을 연다.
   4. 메모를 읽고 이어 가게 하고, §4의 goal을 다시 건다. 화면에 `/goal active`가 뜨는지 확인한다. `/clear` 직후 처음 보낸 `/goal`은 goal로 걸리지 않고 지시로만 전달된다(2회 확인). 그래서 이어 가라는 지시를 먼저 보내고, 그다음 `/goal`을 보낸다("Goal set"이 뜬다).
4. **완료 기록**: 마일스톤 안의 단계(A, B, C…)나 관문이 끝나면 §3에 날짜·결과·커밋을 적는다.

공통 규칙(모든 에이전트):
- Gurobi는 한 프로세스만 돌린다(`gurobi_guard.pth`). "already running"이면 기다렸다 다시 시도한다.
- 공유 `.venv`에 `maturin develop`이나 `pip install`은 main 트리만 한다. worktree는 cargo로 빌드한 `.so`를 자기 `python/rsolver`에 두고 `PYTHONPATH=$PWD/python`으로 쓴다.
- worktree는 `crates/lp`의 기존 파일을 고치지 않는다(`lib.rs` 모듈 등록은 허용). 예외(2026-11-06): M1 LP 드라이버 흐름(G5)은 이미 끝난 흐름의 폴더 `crates/lp/src/presolve/`, `barrier/`, `pdhg/`와 `crates/model`을 고쳐도 된다. simplex 흐름의 파일(simplex·lu·spx·start·bfrt·scaling 등)은 계속 금지다. 고칠 때마다 barrier·PDHG 64/64가 유지되는지 확인한다. 커밋은 자기 브랜치에만, 단계마다 한다.
- 작업 완료 시 현재 worktree와 main의 진행 기록과 로드맵에 작업·검증 결과와 미병합 상태를 반영한다.
- 답변은 한국어로 한다.

## 2. 작업 흐름

| 탭 | 폴더 / 브랜치 | 담당 | 인수인계 메모 | goal |
|---|---|---|---|---|
| M1 심플렉스 | `rsolver` / `main` | LP simplex(dual·primal), LU, crash, Quad, LP 드라이버, `crates/core` 삭제 | `docs/10_porting.md` §9 | G1 |
| ~~M1 LP presolve~~ (완료, main 병합) | `rsolver-presolve` / `m1-lp-presolve` | LP presolve, postsolve | `docs/m1/m1_presolve.md` | G2 |
| ↳ ~~postsolve~~ (완료, `m1-lp-presolve`에 병합. pane `w4:pP`는 유휴) | `rsolver-postsolve` / `m1-postsolve` | postsolve `363daa` | `docs/m1/m1_postsolve.md` | G2의 일부 |
| ↳ ~~derived·PreDepRow~~ (완료, 병합, 브랜치 삭제) | — / `m1-derived`, `m1-deprow` | `5b488d`, `47c445` | `docs/m1/m1_presolve.md` | G2의 일부 |
| ↳ symmetry (presolve 하위 에이전트, pane `w4:pV`) | `rsolver-symmetry` / `m1-symmetry` | 대칭 검출 `5577ba`→`551cc0` | `docs/m1/m1_presolve.md` | G2의 일부 |
| ~~M3 barrier~~ (완료, main 병합) | `rsolver-barrier` / `m3-barrier` | barrier 본체(crossover 제외) | `docs/m3/m3_barrier.md` | G3 |
| ~~M3 PDHG~~ (완료, main 병합) | `rsolver-barrier` / `m3-pdhg` | PDHG 본체(crossover 제외) | `docs/m3/m3_pdhg.md` | G4 |
| M1 LP 드라이버 (main 병합, 수치 한계 남음) | `rsolver` / `main` | 파라미터 표, 드라이버, Presolve 켠 연속 경로 | `docs/m1/m1_driver.md` | G5 |
| M2 MIP presolve (별도 검증 후 main 병합) | `rsolver-barrier` / `m2-mip-presolve` | MIP presolve `3cd406` P0. 검증 범위·한계는 §3 | `docs/m2/m2_presolve.md` | G6 |
| ↳ MIP presolve 하위 작업 (상위 브랜치에 병합 완료) | `m2-mip-presolve`에 통합 | probing·clique·바깥 reduction·마무리 | `docs/m2/m2_presolve.md` | G6의 일부 |

## 3. 완료 기록

| 날짜 | 단계 | 결과 | 커밋 |
|---|---|---|---|
| 2026-11-05 | M0 관문 | 지문 102/102 | `fc0dcd0` |
| 2026-11-05 | M1 A 스케일링 | 열·행·목적 스케일 64/64 비트 | `503542d` |
| 2026-11-05 | M1 C singleton LU·FTRAN | 반복 0 `xB` 60/60 비트 | `e0eb167` |
| 2026-11-05 | M1 C BTRAN·`d`·DSE 가중치 | 60/60 비트 | `cb66a2b` |
| 2026-11-05 | M1 B 시작 기저 | `vstat` 64/64 | `fff2399` |
| 2026-11-05 | M1 presolve 0–1b | Presolve=0 64/64, PRELIMIT 1–3 64/64 | `87c95d8` (`m1-lp-presolve`) |
| 2026-11-05 | M1 presolve 1c–1g | 열 패스, singleton 행, forcing 행, 중복 행, 바깥 패스. PRELIMIT 1–14 64/64 | `3bed233` (`m1-lp-presolve`) |
| 2026-11-05 | M1 LP 엔진 종단 연결 | dual·primal simplex가 스케일된 작업 공간에서 돈다. 읽은 모델에서 Python `optimize()` 동작 | `ac30617` |
| 2026-11-05 | M3 준비 | 호출 범위 확인(simplex·LU·presolve와 독립, 공유는 스케일링 `07d3ce`뿐), Method=2 기준값, barrier 오라클·모듈 뼈대 | `8405f7c` (`m3-barrier`) |
| 2026-11-05 | M1 dual 반복 경로 | BFRT·cost shifting, 수치 안전장치 사다리, 가중치 리셋, 재분해 뒤 flip. `crates/core` 삭제(만든 LP도 LP 엔진 사용), Markowitz count bucket | `c9fddee`, `33a2342` |
| 2026-11-05 | M1 postsolve 준비 | uncrush 오라클(`363daa` 훅), 저장·종료 규칙과 LP uncrush 경로를 03에 반영 | `e707fea`, `f0622fa` (`m1-lp-presolve`) |
| 2026-11-05 | M3 정규방정식·ordering | 결정성 64/64, small-pivot 규칙의 LDLᵀ, A·Θ·Aᵀ 조립, AAᵀ 패턴 64/64, AMD 순열·symbolic 53/64 | `66054a0` (`m3-barrier`) |
| 2026-11-05 | M1 presolve 2a–2c | 비용 선호점 고정, 함의 중복 행, dual bound 전파. PRELIMIT 1–14 64/64. PreDepRow는 Netlib에서 degen2·degen3만 바꿈 | `eb471f7` (`m1-lp-presolve`) |
| 2026-11-05 | M3 ordering·symbolic | AAᵀ NZ, 순열(AMD + bisection), supernode, Factor NZ/Ops 64/64 | `f1d0841`, `8bc322b` (`m3-barrier`) |
| 2026-11-05 | M3 IPM 프로토타입 | 초기점, predictor, Mehrotra/Gondzio corrector, line search, Gurobi pivot 규칙 LDLᵀ와 refinement | `06308ed` (`m3-barrier`) |
| 2026-11-05 | M1 dual 회귀 수정(israel) | 인공 bound 배율에서 flip 뒤 `x_B` 재계산 | `6013997` |
| 2026-11-05 | M3 IPM 프로토타입 64개 | 상태 64/64, 목적 ≤1e-6 64/64, **BarIterCount 56/64**(±5% 57/64). Barrier statistics 로그 64/64. 불일치 8개는 해법 반올림 누적 | `e905669`, `9b4c612` (`m3-barrier`) |
| 2026-11-05 | M1 dual CHUZR 원본 이식 | 순서 있는 후보 목록(표시·재구축 허용량), BTRAN 비용 점수, 인공 bound 단계(1e19). 반복 0 목록 63/64 | `899c192`, `e412440` |
| 2026-11-05 | M1 presolve 3a–3b | 평행 열·행, PRESPLITROWS, 비용으로 조이는 부등식 행. PRELIMIT 1–17 64/64(기본값 최종 모델은 3/64) | `aba84fe` (`m1-lp-presolve`) |
| 2026-11-05 | M3 Rust IPM | Method=2 경로, 상태·목적 64/64, Barrier statistics 64/64, `2d8ee5` pivot 규칙으로 **BarIterCount 59/64**(92%, G3 P1 기준 90% 넘음), PRIMINF 반복 0 64/64 | `a4ab223`, `ddc3040`, `df38e2a` (`m3-barrier`) |
| 2026-11-05 | M1 dual BFRT·DSE 갱신 | 원본 규칙의 BFRT(말뭉치 99.4%), 정확한 DSE 갱신, Gurobi 정규화 규약의 reduced cost. Netlib 64: 정답 64/64, 반복 수 ±5% 21개, 기하평균 1.180배 | `1979049`, `33b944b`, `93e51af` |
| 2026-11-05 | M1 presolve 4 집계 | 연결 리스트 행렬 위의 aggregation(`519a52`/`37f484` A–D 단계). 집계 뒤 점검 지점 62/62 | `3bbdba0` (`m1-lp-presolve`) |
| 2026-11-05 | M1 postsolve uncrush | `363daa` 레코드 1·2·3·5·7·8·14와 검사기, 57/58 | `94c4195` (`m1-postsolve`) |
| 2026-11-05 | M3 재시작·Cholesky | `2bf1ea` 재시작 이식(perold 125회, Gurobi 130회), BarIterCount ±5% 62/64. supernodal 분해 프로토타입이 실제 MKL로 L·d 비트 일치(7개 × 반복 0–2) | `bf116b4`, `6a0057f` (`m3-barrier`) |
| 2026-11-05 | M1 dual 재분해 일정 | `6268b` 재분해 기준(갱신 수·간격·갱신 파일 비용). Netlib 64: 정답 64/64, **반복 수 같음 7**, ±5% 34 | `23cad5d` |
| 2026-11-05 | M1 presolve 5 마무리 단계 | 바깥 패스 나머지, 다음 패스, 마무리(`50de61` 함의 bound). **기본값 presolved 모델 62/64**(`5b488d`·`47c445`는 아직 빈 함수). postsolve 병합, 집계 뒤 uncrush 비트 일치(PRELIMIT 1–17) | `6aa22b7`, `44ef311`, `d9a4874` (`m1-lp-presolve`) |
| 2026-11-05 | M3 Cholesky 해법 | `2ee6c5` 해법 프로토타입(8-lane 내적, right-looking forward, backward), dx·dy 비트 일치 7개 | `37e1c19` (`m3-barrier`) |
| 2026-11-05 | M1 dual 반복 원본화 | CHUZR(모드 0/1)·BFRT(코퍼스 99.4%)·DSE·d 갱신·재분해 시점 이식. Netlib 64: 64/64 정확, 반복 일치 7, ±5% 34, sc50a 반복 0–6 비트 일치 | `899c192`, `1979049`, `33b944b`, `93e51af`, `23cad5d` |
| 2026-11-05 | M1 presolve PREDERIVED·postsolve | `5b488d`(PREDERIVED) 비트 일치 이식, type 24 레코드. 집계 레코드 6/11/21 uncrush, 기본값 postsolve 53/59. rsolver-lp clippy 경고 0 | `25a1494`, `b53fd32`, `15260ee`, `d537b7e` (`m1-lp-presolve`) |
| 2026-11-05 | M3 비트 일치 분해·해법 | Rust supernodal 분해·해법(`chol.rs`)과 MKL DGEMM/DTRSM 흉내(`mkl.rs`)가 Gurobi와 비트 일치. IPM을 이 위에 올려 **BarIterCount 63/64** | `5557d2b`, `506a26e` (`m3-barrier`) |
| 2026-11-05 | M1 presolve 기본값 64/64 | `47c445`(PreDepRow) 이식. 기본 파라미터 presolved 모델 64/64 | `38ea4e7` (`m1-lp-presolve`) |
| 2026-11-05 | M1 postsolve 64/64 | 행·열을 지우지 않은 경우의 항등 uncrush까지, 검사기 64/64. Gurobi 없이 도는 pytest 회귀 | `15b7b65`, `c024356` (`m1-postsolve` → `m1-lp-presolve`) |
| 2026-11-05 | **M3 barrier 본체 (G3) 완료** | 오케스트레이터가 따로 검증: 상태 64/64, 목적 ≤1e-6 64/64, **BarIterCount 64/64**, clippy 0, cargo test·pytest 통과, 검증 중 Gurobi 미사용. main에 병합(`crates/core` 삭제 유지, SIMPLEX·BARRIER `cbGet` 통합) | `c1a5124`, `3e8a744`, `03d2ca4`, 병합 `e6760c3` |
| 2026-11-05 | M1 dual 시작·섭동·난수 | 자유 열 루프·재분해 간격(m^0.7) 원본화, cost shifting, `common::Rng`(libgurobi 난수와 비트 일치), `48d5f` GLOBAL 섭동. Netlib 64: 정답 63/64, **반복 수 같음 11**, ±5% 39, 기하평균 1.014배 | `9c5614b`, `a8cf69e`, `04622c8` |
| 2026-11-05 | M1 presolve 작업량(생성자) | `5118e6` 문제 유형 검사 포함, PRELIMIT=1 작업량 64/64 | `bc9f229` (`m1-lp-presolve`) |
| 2026-11-05 | M3 PDHG 0–1 | 호출 범위·의존 확인, Method=6 기준값, 결정성 64/64, PDHG 상태 오라클 | `984af95`, `1e820a2` (`m3-pdhg`) |
| 2026-11-05 | M1 Netlib 91 Method 1 점검 | 86/91 정답(라이선스 밖 27개 중 23) | `9fba738` |
| 2026-11-05 | M1 presolve 작업량 예산 | `50de61`·`516cb4` 작업량 항, `3ba9de` 예산 정지 | `9130f4f` (`m1-lp-presolve`) |
| 2026-11-05 | M1 대칭 검출 게이트 | `5577ba`/`551cc0` LP 경로 이식, 합성 대칭 LP 회귀 | `fcb1bb2`, `6e30a98` (`m1-symmetry`) |
| 2026-11-05 | M3 PDHG (G4) 에이전트 보고 완료 | Rust PDHG(Method=6)가 Netlib 64에서 Gurobi와 비트 일치, 콜백별 상태 비교 도구. **오케스트레이터 검증·병합 대기** | `60b0b84`, `1479067` (`m3-pdhg`) |
| 2026-11-06 | **M3 PDHG (G4) 완료** | 오케스트레이터가 따로 검증(기준값 로그의 파라미터 그대로, WorkLimit 60): 상태 64/64, 목적 ≤1e-6 64/64, **PDHGIterCount 64/64**. clippy 0, cargo test·pytest 통과. main 병합 뒤 barrier 64/64도 다시 확인 | `60b0b84`, `1479067`, 병합 `1d0118c` |
| 2026-11-06 | **M1 LP presolve·postsolve (G2) 완료** | 오케스트레이터가 따로 검증: presolved 모델을 처음 뜬 Gurobi 기준값(`bench/gurobi/*/presolved.mps.gz`)과 비교해 **64/64**, postsolve 64/64, clippy 0, cargo test·pytest 31 통과. main 병합 뒤 barrier·PDHG 64/64 다시 확인 | `a8e01e7`, `6a92185`, 병합 `df70174` |
| 2026-11-06 | M2 presolve 1단계 | 호출 그래프와 Frida로 확인한 결과 `3cd406`은 LP를 풀지 않는다. 실행 순서대로 reduction 목록 정리, 파라미터 오프셋 표(`ac81d7` + getParamInfo), PRELIMIT 훑기 도구, MIP 상태 덤프 오라클 | `077cfa2`, `5627199`, `647088d` (`m2-mip-presolve`) |
| 2026-11-06 | M1 primal simplex 원본 이식 | `0478d3` 명세의 primal(행 표시, 복합 phase 1, 부분 목록 CHUZC, Harris·`54eef` ratio, 진입 flip), `b1da8` 퇴화 bound shift, `cda5b` primal↔dual 넘김. Method 0: Netlib 91 중 84, Method 1: 64 중 63 | `7affabb` |
| 2026-11-06 | M1 드라이버: 파라미터 표·헤더 로그 | Frida로 `9828a1`/`c23877`/`0c3d0d` 호출 범위 확인. 파라미터 표가 libgurobi130에서 **548/548 = gurobipy**(G5 (1)). 새 driver 크레이트의 Optimize 헤더 로그 **102/102**(G5 (2)) | `0d22211`, `320fba5` (`m1-lp-driver`) |
| 2026-11-06 | M2 MPS 쓰기(정수) | 정수 MARKER, BV/LI bound를 `a4590c`처럼 씀(MIPLIB 원본 38/38) | `b5a93c1` (`m2-skel`) |
| 2026-11-06 | M1 LU Markowitz 수정 | 허용 가능한 피벗 없이 4줄을 지나도 기저를 singular로 판정하지 않음(정상 기저에 slack 29–62개가 들어가던 문제). Method 1: Netlib 91 중 89 | `736c3b5` |
| 2026-11-06 | M1 드라이버: 연속 모델 `optimize()` | Method 확정, 자동 조정, presolve → simplex·barrier·PDHG → uncrush, 상태·요약 로그, Presolve≠0이면 Python `_driver` 분기, `tools/driver_check.py`. Method 2·6 + Presolve 켬: 상태·목적 64/64, presolved 모델 8/64, BarIterCount 20/64. presolve의 Method 분기가 남음 | `9f5a723` (`m1-lp-driver`) |
| 2026-11-06 | M2 presolve 골격·패스 해독 | `crates/mip` 상태(연결 리스트), `3cd406` 앞부분·마무리·모델 생성, 패스 루프·라운드 끝·마지막 단계 해독(하위 에이전트 skel, spine-b/c/d 병합) | `54f3923`, `e69eb66`, `4d5b7b0`, `1b1e9b3` (`m2-mip-presolve`) |
| 2026-11-06 | M1 unscaled 재검사·CHUZR 비용·BTRAN | `2e9ae` 원 공간 재검사와 재풀이(라이선스 밖 Netlib 최적값 정확), 행 비용 가중치 갱신, Markowitz 분해의 희소 BTRAN. blend는 Gurobi 피벗 경로 전체와 같음. Netlib 64: **반복 수 같음 15**, 경로 전체 일치 6 | `61676e0`, `bf35530`, `4474ff8` |
| 2026-11-06 | M1 presolve Method 분기 | Method·Crossover·SolutionTarget 분기(`state+0x100`). Method 2·6, Crossover 0에서 presolved 모델 Gurobi와 **64/64** | `ce24d8e` (`m1-lp-driver`) |
| 2026-11-06 | M2 presolve 제어 흐름·해독 | `3cd406` 라운드·패스·내부 루프·라운드 끝·마지막 단계 제어 흐름(reduction은 빈 함수), probing·clique·내부 루프 해독 노트(하위 에이전트 inner, probe, clique 병합) | `f374378`, `0de8d15` (`m2-mip-presolve`) |
| 2026-11-06 | M1 dual 원본 규칙 추가 | 이탈 변수 bound 방향(`28c93`), 분해 뒤 CHUZR 목록 재구축, `48d5f` 섭동 발동 규칙, pricing 카운터. Netlib 64: 정답 64/64, **반복 수 같음 18–19**, ±5% 40, 기하평균 1.000 | `f8e9c59`, `549e828`, `a19d442`, `835441c` |
| 2026-11-06 | M1 드라이버: PDHG 시드·G5 보고 | presolve 모델 난수(`3b0917`/`895c5`)로 PDHG 시드, Presolve 켠 PDHG 64/64. 에이전트가 **G5 기준 충족**을 보고함(검증 대기) | `c8f7170`, `1ea1977` (`m1-lp-driver`) |
| 2026-11-06 | M2 clique 표 | `451bed` clique 표 이식 | `cf52001` (`m2-mip-presolve`) |
| 2026-11-06 | M2 presolve 내부 루프 병합 | 내부 루프 reduction 병합. 기본값 presolved 모델 **9/38**. XOR 행(`3e5363`) 해독(하위 에이전트 outer-a) | `f070a16`, `f4320c9` (`m2-outer-a`) |
| 2026-11-06 | M1 LU 갱신(Forrest–Tomlin) | `5b257` 원형 FT 갱신(제자리 열 교체 또는 row eta, 8레인 합) | `670afe9` |
| 2026-11-06 | M2 probing A·내부 루프 도구 | probe 행렬·엔진 `4e48f7`·충돌 그래프 이식, 레코드·덤프 검사 도구. dual bound(`374521`)·함의 그래프(`5431ee`) 해독 | `ce3522e`, `4b425d6`, `1010d5a` (`m2-mip-presolve`, `m2-outer-a`) |
| 2026-11-06 | M1 FT 갱신 세부 규칙 | `5b257` prefix·빈 열 캐시, U 행이 비면 row eta 생략, 순열 교환 | `4fa5ef6` |
| 2026-11-06 | M1 드라이버: 기저 없는 uncrush·상태 규칙 | `36578f`/`365159` 기저 없는 uncrush(Method 2·6, 오라클과 64/64 비트), 원본 상태 규칙, big-uncrush-violation 검사 | `fcd853e` (`m1-lp-driver`) |
| 2026-11-06 | M2 probing B·clique 그룹 | probing 결과 반영(대입·bound·고정·lifting)과 호출 13개 상태 일치, clique 덮개 강화(`45885d`)·set packing 쌍 그래프(`52be10`)·clique 행 정규화(`46a62d`) | `0789b47`, `a91287e`, `2ba8870`, `955a8be` (`m2-mip-presolve`) |
| 2026-11-06 | M1 Markowitz 커널·CHUZR 모드 1 | 비트 일치 프로토타입에서 `036ade` Markowitz 커널 이식, `1bae9c`/`2896a3` CHUZR 모드 1 임계값, 시작 단계 섭동 카운터(tuff가 Gurobi처럼 반복 27에서 섭동) | `f513cd2`, `5b21c30`, `6aa3b46` |
| 2026-11-06 | M1 드라이버: 엔진 로그 | barrier·PDHG·simplex 반복 로그 행, 증명 상태 로그, `tools/log_check.py`, 전체 로그 테스트 | `73491b9` (`m1-lp-driver`) |
| 2026-11-06 | M2 바깥 reduction A·B | 함의 그래프 추이 축소(`5431ee`), dual bound(`374521`), 부등식→등식(`3e40d1`), 노드 아레나 압축(`36ee92`), dual 집계(`418542`). 덤프 호출 78/78 | `f280d7c`, `0ea348a`, `8624df7`, `d1d083d`, `2ebdf67` (`m2-outer-a`, `m2-outer-b`) |
| 2026-11-06 | M1 simplex 속도 개선 | 비트는 그대로(피벗 경로 64/64 동일). BTRAN 출력 재사용, α 지지 집합만 순회, LU 평면 배열, FT 갱신에서 U 위치 역순열 유지 | `61ccd6b`, `2cb6739` |
| 2026-11-06 | M2 round end·바깥 reduction 추가 | 지배된 singleton 열(`5a9282`), 비용 singleton mod gcd(`5ab429`), VUB lifting(`59c193`), 계수 패턴 사전(`55a779` 1부), XOR 행(`3e5363`), 함의 정수(`57627e`), 평행 행·열 MIP 분기(`3ff744`), 비영 소거(`40d2bc`), MIP 집계(`37f484`) | `dff8109`, `91c379b`, `2b791f0`, `79ae5aa`, `f001cff`, `b14499b` (`m2-*`) |
| 2026-11-06 | M2 MIP presolve (G6) 에이전트 검증 완료 | 에이전트가 `m2-outer-b` 병합 후 재검증했다고 보고함. 기본값 presolved 모델, PRELIMIT sweep 변화점, postsolve 레코드 각각 **38/38** 일치. probing·대칭·연결 성분 탐색 포함, reduction stub 삭제. clippy 경고 0, cargo test 통과, pytest 37 passed·1 xfailed. 당시 별도 검증·main 병합 대기 | `a683710`, 병합 `83f4fc3`, 최종 `2f28f77` (`m2-mip-presolve`) |

| 2026-11-09 | M2 풀이 경로 진행 | `m2-mip-presolve`에서 B&B와 루트 전파·휴리스틱을 구현했고, 고정·치환 레코드를 역순으로 복원하는 presolve→B&B 경로와 Python MIP `optimize()`를 연결했다. 미지원 레코드, 빈 축약 모델, 복원 검증 실패는 원래 모델을 직접 푼다. 원래 제약과 목적값으로 복원 해를 검증한다. Rust MIP 60개, Python B&B·모델 405개 통과(작은 이진 MIP 200개를 완전탐색과 비교). LP presolve→simplex→postsolve, 다른 MIP 복원 레코드, MIP 콜백은 미완료. 이후 `3e5c9d4`로 main에 병합했다. | `13ce2fe`, 병합 `3e5c9d4` |

### G6 검증 범위와 남은 작업 (2026-11-06)

- 위 결과는 `m2-mip-presolve`의 `2f28f77`에서 에이전트가 보고한 값이다. 2026-11-09 별도 검증 후 `3e5c9d4`로 main에 병합했다. 검증 절차와 상세 결과는 해당 브랜치의 `docs/m2/m2_presolve.md` §0에 있다. `git show 2f28f77:docs/m2_presolve.md`로 확인한다.
- 검증 범위는 miplib_small 38개다. 그 범위에서 실행되지 않은 covers 적용, type 24 레코드 바이트, Q·QC·SOS·NL, 재시작 문맥은 오라클로 확인하지 못했다.
- 하위 풀이 `5307f4`·`522aa2`와 doubleton 등식 처리 `56b360`은 건너뛴다. 여러 연결 성분이 있어 하위 풀이가 필요한 모델에서는 Gurobi와 결과가 달라질 수 있다.
- `cliquerows.rs`·`setpack.rs`의 노드 풀 확장 `3b85ea`는 아직 `assert!`로 막혀 있어 더 큰 모델에서 중단될 수 있다. 이 한계는 병합 후에도 남는다.

| 2026-11-09 | G6 별도 검증·main 병합 | 축약 모델·레코드·PRELIMIT 저장 변화점과 최종 모델 각 38/38, clippy 0, Rust 테스트 통과, Python 498개 통과. 검증은 병합 전 동일 코드에서 수행했다. | `13ce2fe`, 병합 `3e5c9d4` |

## 4. goal 원문

- **G1**: M1에서 LP presolve·postsolve를 뺀 나머지를 완료한다. 완료 기준(Presolve=0, Threads=1):
  1. Netlib 91개에서 Method 0/1/기본 모두 상태가 Gurobi와 같고, 목적 상대오차 ≤1e-6이다. 크기 제한 밖 27개는 공개 최적값과 비교한다.
  2. P1: Netlib 64개에서 IterCount가 Gurobi와 같은 인스턴스가 90% 이상이다. 나머지는 ±5%이고 첫 분기점을 기록한다.
  3. P2: work ±5%.
  4. P3: 시간 기하평균 ≤1.2배. 기계가 한가할 때 잰다.
  5. dual·primal simplex, Markowitz·FT LU, crash, Quad, 필요한 common을 포팅한다. cargo clippy 경고 0, 테스트 전부 통과.

  범위 밖은 LP 드라이버·파라미터 표·Presolve 켠 경로(G5), presolve·postsolve·barrier·PDHG(완료)다. M1 관문 속도 스냅숏은 G1·G5가 끝난 뒤 오케스트레이터가 한다.
- **G2**: LP presolve(`390989`)와 postsolve를 완료한다. 완료 기준:
  1. P0: Netlib 64개에서 기본 파라미터의 presolved 모델(`presolved.mps.gz` SHA-256)이 Gurobi와 64/64 같다.
  2. presolved 모델의 해·기저를 postsolve로 원래 공간에 되돌린 결과가 Gurobi와 비트 단위로 같다. 64/64, 오라클로 검증한다.
  3. 미뤄 둔 `47c445`(PreDepRow)와 대칭 검출 게이트(`5577ba`→`551cc0`)를 포함해 기본값에서 도는 reduction을 모두 옮긴다.
  4. cargo clippy 경고 0, 테스트 통과. `docs/03_presolve.md`와 `docs/m1/m1_presolve.md`를 갱신한다.

  범위 밖은 simplex·드라이버(main)와 barrier다.
- **G3**: barrier(Method=2) 본체를 완료한다(crossover 제외). 완료 기준(Presolve=0, Threads=1, Crossover=0):
  1. Netlib 64개에서 상태가 Gurobi와 같고, 목적 상대오차 ≤1e-6이다.
  2. P1: BarIterCount가 Gurobi와 같은 인스턴스가 90% 이상이다. 나머지는 ±5%이고 첫 분기점을 기록한다.
  3. ordering 결과와 Factor NZ가 Gurobi 로그와 같다.
  4. 반복마다 BARRIER 콜백 오라클과 비교하는 도구를 둔다.
  5. cargo clippy 경고 0, 테스트 통과. `docs/05_barrier_pdhg_concurrent.md`와 `docs/m3/m3_barrier.md`를 갱신한다.

  범위 밖은 crossover, PDHG, concurrent, presolve, simplex다.
- **G4**: PDHG(Method=6) 본체를 완료한다(crossover 제외). 완료 기준(Presolve=0, Threads=1, Crossover=0):
  1. Netlib 64개에서 상태가 Gurobi와 같고, 목적값이 Gurobi PDHG 결과와 상대 1e-6 이내다.
  2. P1: Gurobi가 보고하는 PDHG 반복 수와 같은 인스턴스가 90% 이상이다. 나머지는 ±5%이고 첫 분기점을 기록한다.
  3. 반복별 오라클 비교 도구를 둔다.
  4. cargo clippy 경고 0, 테스트 통과. `docs/05_barrier_pdhg_concurrent.md`와 `docs/m3/m3_pdhg.md`를 갱신한다.

  범위 밖은 crossover, concurrent, barrier(완료), presolve, simplex다.
- **G5**: M1 LP 드라이버를 완료한다. 기준은 `docs/02_driver.md`이고, 대상은 `9828a1`→`c23877`의 연속 모델 경로와 LP 드라이버 `0c3d0d`다. 완료 기준:
  1. 파라미터 표(`ac81d7` 등록): 모든 파라미터의 이름·기본값·최소·최대가 gurobipy와 같다.
  2. Optimize 헤더 로그("Optimize a model with …", 지문, 계수 범위 통계)가 `bench/gurobi`의 Netlib 64개 + miplib_small 38개 `log.txt`와 같다.
  3. 모델 분류, Method 확정 표, 자동 조정, 상태 확정, 요약 로그를 원본대로 옮긴다.
  4. Presolve를 켠 연속 경로(presolve → simplex·barrier·PDHG → postsolve → 상태)가 돈다. Netlib 64개에서 Presolve=-1이고 Method=2·Crossover=0, Method=6·Crossover=0일 때 상태·목적(≤1e-6)·BarIterCount/PDHGIterCount가 Gurobi와 같다(기준값은 새로 뜬다). Method=1은 상태·목적만 본다.
  5. cargo clippy 경고 0, 테스트 통과. `docs/02_driver.md`와 `docs/m1/m1_driver.md`를 갱신한다.

  범위 밖은 simplex 내부(G1)와 MIP다.
- **G6**: M2 MIP presolve(`3cd406`)의 P0를 완료한다. 완료 기준:
  1. miplib_small 38개에서 기본 파라미터(Threads=1)의 presolved 모델이 `bench/gurobi/<이름>/presolved.mps.gz`와 38/38 같다.
  2. 기본값에서 도는 MIP reduction(probing·대칭성·연결 성분 포함, docs/03 기준)을 모두 원본대로 옮긴다.
  3. MIP presolve의 postsolve 레코드도 원본대로 쌓는다.
  4. cargo clippy 경고 0, 테스트 통과. `docs/03_presolve.md`와 `docs/m2/m2_presolve.md`를 갱신한다.

  첫 단계는 호출 그래프로 `3cd406` 경로가 LP 풀이·simplex·barrier를 부르는지 확인하는 것이다. 범위 밖은 B&B, cut, 휴리스틱, simplex다.
